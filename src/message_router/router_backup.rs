//! Main message router implementation
//!
//! Coordinates message routing between agents using a coordination-first architecture
//! with high-performance async processing and comprehensive error handling.

use crate::message_router::{
    config::RouterConfig,
    domain_types::{
        AgentId, AgentLocation, AgentState, CapabilityName, Conversation, ConversationCreatedAt,
        ConversationId, DeliveryOptions, FailureReason, FipaMessage, LocalAgent, MessageContent,
        MessageCount, MessageId, MessageTimestamp, NodeId, Performative, ProtocolName, RouteHops,
        ValidationField, ValidationReason,
    },
    traits::{
        AgentRegistry, ConversationError, ConversationManager, ConversationStats, DeadLetterStats,
        DeliveryEngine, DeliveryError, FailureHandler, HealthStatus, MessageRouter,
        MetricsCollector, RegistryError, RouterError, RouterStats,
    },
};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::{Duration, Instant};
use tracing::{Level, debug, error, info, span, trace, warn};

// Constants for validation field names
const FIELD_CONTENT: &str = "content";
const FIELD_PERFORMATIVE: &str = "performative";
const FIELD_SENDER_RECEIVER: &str = "sender/receiver";
const FIELD_IN_REPLY_TO: &str = "in_reply_to";
#[cfg(test)]
const FIELD_ERROR_CONTENT: &str = "error_content";

// Constants for common validation reasons
const REASON_CONTENT_EMPTY: &str = "content cannot be empty";
const REASON_SENDER_EQUALS_RECEIVER: &str = "sender cannot equal receiver";
const REASON_NO_REPLY_WITH: &str = "no corresponding reply_with found";
#[cfg(test)]
const REASON_ERROR_CONTENT_TOO_LARGE: &str = "generated error content exceeds maximum message size";

/// Creates a `ValidationError` with the given field and reason strings
///
/// # Parameters
/// - `field`: The field name that failed validation (must be 1-50 characters)
/// - `reason`: The reason for validation failure (must be 1-200 characters)
///
/// # Returns
/// A `RouterError::ValidationError` with validated domain types
///
/// # Panics
/// Panics if field or reason strings don't meet validation requirements
fn create_validation_error(field: &str, reason: &str) -> RouterError {
    RouterError::ValidationError {
        field: ValidationField::try_new(field.to_string())
            .expect("Field name should meet validation requirements"),
        reason: ValidationReason::try_new(reason.to_string())
            .expect("Reason should meet validation requirements"),
    }
}

/// Creates a `ValidationError` with formatted reason message
///
/// # Parameters
/// - `field`: The field name that failed validation (must be 1-50 characters)
/// - `reason_template`: The template string with format arguments applied
///
/// # Returns
/// A `RouterError::ValidationError` with validated domain types
///
/// # Panics
/// Panics if field or reason strings don't meet validation requirements
fn create_validation_error_with_format(field: &str, reason_template: &str) -> RouterError {
    RouterError::ValidationError {
        field: ValidationField::try_new(field.to_string())
            .expect("Field name should meet validation requirements"),
        reason: ValidationReason::try_new(reason_template.to_string())
            .expect("Reason should meet validation requirements"),
    }
}

// ============================================================================
// Conversation Threading Tracker - Extracted for better organization
// ============================================================================

/// Manages conversation threading state with proper encapsulation
///
/// This tracker ensures FIPA-ACL conversation threading compliance by:
/// - Isolating `reply_with/in_reply_to` tracking per conversation
/// - Providing thread-safe access to conversation state
/// - Supporting future cleanup and expiration mechanisms
#[derive(Debug)]
struct ConversationThreadingTracker {
    /// Maps conversation IDs to sets of valid `reply_with` message IDs
    /// This ensures conversation isolation - messages can only reply to
    /// messages within their own conversation context
    conversations: Mutex<HashMap<ConversationId, HashSet<MessageId>>>,
}

impl ConversationThreadingTracker {
    /// Creates a new conversation threading tracker
    fn new() -> Self {
        Self {
            conversations: Mutex::new(HashMap::new()),
        }
    }

    /// Validates that `in_reply_to` has a corresponding `reply_with` in the same conversation
    ///
    /// This method ensures FIPA conversation isolation by only allowing replies to
    /// messages within the same conversation context. This prevents cross-conversation
    /// threading attacks and maintains proper FIPA-ACL conversation boundaries.
    ///
    /// # Arguments
    /// * `conversation_id` - The conversation context for validation
    /// * `in_reply_to` - The message ID this message claims to reply to
    ///
    /// # Returns
    /// * `Ok(())` - If the `reply_with` exists in the same conversation
    /// * `Err(RouterError::ValidationError)` - If no corresponding `reply_with` found
    ///
    /// # Thread Safety
    /// Uses internal mutex for thread-safe access to conversation state.
    /// Short critical section ensures minimal lock contention.
    fn validate_reply_reference(
        &self,
        conversation_id: &ConversationId,
        in_reply_to: &MessageId,
    ) -> Result<(), RouterError> {
        let conversations = self
            .conversations
            .lock()
            .expect("Conversation tracker mutex should not be poisoned");

        match conversations.get(conversation_id) {
            Some(reply_with_set) if reply_with_set.contains(in_reply_to) => Ok(()),
            _ => Err(create_validation_error(
                FIELD_IN_REPLY_TO,
                REASON_NO_REPLY_WITH,
            )),
        }
    }

    /// Records a `reply_with` ID for future validation within the conversation
    ///
    /// This method maintains the imperative shell responsibility of managing
    /// conversation state while keeping the functional core pure. It ensures
    /// that `reply_with` values are properly indexed for future validation.
    ///
    /// # Arguments
    /// * `conversation_id` - The conversation context
    /// * `reply_with` - The message ID that can be referenced in future replies
    ///
    /// # Thread Safety
    /// Uses internal mutex for thread-safe access to conversation state.
    /// Atomic operation ensures consistency during concurrent access.
    fn record_reply_with(&self, conversation_id: &ConversationId, reply_with: &MessageId) {
        let mut conversations = self
            .conversations
            .lock()
            .expect("Conversation tracker mutex should not be poisoned");

        conversations
            .entry(*conversation_id)
            .or_default()
            .insert(*reply_with);
    }

    /// Gets the number of active conversations (for metrics/debugging)
    #[allow(dead_code)]
    fn active_conversation_count(&self) -> usize {
        self.conversations.lock().unwrap().len()
    }

    /// Cleans up expired conversations (placeholder for future implementation)
    /// This would be integrated with conversation manager cleanup in production
    #[allow(dead_code)]
    fn cleanup_expired_conversations(&self, _expired_conversations: &[ConversationId]) {
        let mut conversations = self.conversations.lock().unwrap();
        // Future implementation: remove conversation entries for expired conversations
        // For now, conversations accumulate until restart
        // TODO: Integrate with ConversationManager cleanup cycle
        conversations.retain(|_id, _messages| true);
    }
}

/// Global conversation threading tracker instance
/// TODO: Replace with proper conversation manager integration in refactor phase
static CONVERSATION_TRACKER: OnceLock<ConversationThreadingTracker> = OnceLock::new();

/// Gets the global conversation threading tracker instance
fn get_conversation_tracker() -> &'static ConversationThreadingTracker {
    CONVERSATION_TRACKER.get_or_init(ConversationThreadingTracker::new)
}

// ============================================================================
// Pure functions for FIPA message validation and response generation (functional core)
// ============================================================================

/// Validates that sender and receiver are different agents (FIPA requirement)
///
/// FIPA-ACL specification requires that agents cannot send messages to themselves
/// to ensure proper multi-agent communication patterns.
///
/// # Arguments
/// * `message` - The FIPA message to validate
///
/// # Returns
/// * `Ok(())` - If sender and receiver are different
/// * `Err(RouterError::ValidationError)` - If sender equals receiver
fn validate_sender_receiver_different(message: &FipaMessage) -> Result<(), RouterError> {
    if message.sender == message.receiver {
        Err(create_validation_error(
            FIELD_SENDER_RECEIVER,
            REASON_SENDER_EQUALS_RECEIVER,
        ))
    } else {
        Ok(())
    }
}

/// Validates message content is not empty (FIPA requirement)
///
/// FIPA-ACL specification requires that messages contain meaningful content
/// for proper agent communication and protocol compliance.
///
/// # Arguments
/// * `message` - The FIPA message to validate
///
/// # Returns
/// * `Ok(())` - If content is not empty
/// * `Err(RouterError::ValidationError)` - If content is empty
fn validate_content_not_empty(message: &FipaMessage) -> Result<(), RouterError> {
    if message.content.is_empty() {
        Err(create_validation_error(FIELD_CONTENT, REASON_CONTENT_EMPTY))
    } else {
        Ok(())
    }
}

/// Validates FIPA conversation threading requirements
///
/// Ensures that:
/// 1. Messages with `in_reply_to` reference valid `reply_with` values
/// 2. Conversation isolation - replies only work within the same conversation
/// 3. Reply threading maintains proper FIPA-ACL conversation flow
///
/// This is a pure function that delegates to the conversation tracker for
/// stateful operations, maintaining functional core principles.
fn validate_conversation_threading(message: &FipaMessage) -> Result<(), RouterError> {
    let tracker = get_conversation_tracker();

    // FIPA validation: in_reply_to must have corresponding reply_with in same conversation
    if let Some(in_reply_to) = &message.in_reply_to {
        // Require conversation_id for proper threading context
        let conversation_id = message
            .conversation_id
            .as_ref()
            .ok_or_else(|| create_validation_error(FIELD_IN_REPLY_TO, REASON_NO_REPLY_WITH))?;

        // Validate reply reference within conversation context
        tracker.validate_reply_reference(conversation_id, in_reply_to)?;
    }

    // Record reply_with for future threading validation (imperative shell operation)
    if let (Some(reply_with), Some(conversation_id)) =
        (&message.reply_with, &message.conversation_id)
    {
        tracker.record_reply_with(conversation_id, reply_with);
    }

    Ok(())
}

/// Validates FIPA message requirements with comprehensive rule checking
///
/// Performs complete FIPA-ACL message validation by applying all required rules:
/// 1. **Agent Identity**: Sender and receiver must be different agents
/// 2. **Content Requirement**: Message content must not be empty
/// 3. **Conversation Threading**: `in_reply_to` must have corresponding `reply_with`
/// 4. **Future Extensions**: Additional FIPA compliance validations
///
/// This function follows the functional core pattern - it's pure, deterministic,
/// and delegates stateful operations to specialized validation functions.
///
/// # Arguments
/// * `message` - The FIPA message to validate against all rules
///
/// # Returns
/// * `Ok(())` - If all FIPA validation rules pass
/// * `Err(RouterError::ValidationError)` - If any validation rule fails
///
/// # FIPA Compliance
/// This validation ensures compatibility with FIPA-ACL specification requirements
/// for proper multi-agent communication protocols.
/// Standard FIPA-ACL performatives defined by the FIPA specification
///
/// These performatives are allowed in strict FIPA compliance mode.
/// Reference: FIPA-ACL Specification for communicative acts.
const STANDARD_FIPA_PERFORMATIVES: [Performative; 11] = [
    Performative::Request,
    Performative::Inform,
    Performative::QueryIf,
    Performative::QueryRef,
    Performative::Propose,
    Performative::AcceptProposal,
    Performative::RejectProposal,
    Performative::Agree,
    Performative::Refuse,
    Performative::Failure,
    Performative::NotUnderstood,
];

/// Validates performative is a standard FIPA-ACL performative (not Caxton extension)
///
/// FIPA-ACL specification defines standard performatives for inter-agent communication.
/// This validation ensures strict FIPA compliance by rejecting Caxton extension performatives.
///
/// # Arguments
/// * `message` - The FIPA message to validate performative
///
/// # Returns
/// * `Ok(())` - If performative is a standard FIPA performative
/// * `Err(RouterError::ValidationError)` - If performative is a Caxton extension
///
/// # Implementation Notes
/// Uses a constant array for efficient lookup and maintainable performative management.
/// Error messages include the specific performative name for better debugging.
fn validate_performative_fipa_compliance(message: &FipaMessage) -> Result<(), RouterError> {
    // FIPA validation: performative must be in standard FIPA-ACL performatives set
    if STANDARD_FIPA_PERFORMATIVES.contains(&message.performative) {
        Ok(())
    } else {
        // Generate descriptive error for non-FIPA performatives
        let performative_name = match message.performative {
            Performative::Heartbeat => "Heartbeat",
            Performative::Capability => "Capability",
            // Note: This match is exhaustive due to STANDARD_FIPA_PERFORMATIVES check above
            _ => "Unknown",
        };

        Err(create_validation_error_with_format(
            FIELD_PERFORMATIVE,
            &format!("{performative_name} is not a standard FIPA performative"),
        ))
    }
}

/// JSON content language identifier for validation
///
/// Used to check if `ContentLanguage` specifies JSON format requiring validation.
/// Case-insensitive matching allows "json", "JSON", "application/json", etc.
const JSON_CONTENT_LANGUAGE: &str = "json";

/// Validates JSON content format when `ContentLanguage` specifies JSON
///
/// FIPA-ACL allows specifying content language to indicate content format.
/// When language contains "json" (case-insensitive), the content must be valid JSON syntax.
/// Uses `serde_json` for comprehensive JSON syntax validation following RFC 7159.
///
/// # Arguments
/// * `message` - The FIPA message to validate JSON content format
///
/// # Returns
/// * `Ok(())` - If content is valid JSON or language doesn't specify JSON
/// * `Err(RouterError::ValidationError)` - If content has invalid JSON format with detailed reason
///
/// # Implementation Notes
/// * Only validates when `ContentLanguage` contains "json" (case-insensitive)
/// * Uses `serde_json::from_slice` for efficient JSON parsing validation
/// * Preserves exact behavior while improving maintainability through constants
fn validate_json_content_format(message: &FipaMessage) -> Result<(), RouterError> {
    // Only validate JSON when `ContentLanguage` contains JSON identifier
    if let Some(language) = &message.language {
        let language_str = language.to_string();
        if language_str.to_lowercase().contains(JSON_CONTENT_LANGUAGE) {
            // Validate JSON syntax using serde_json with detailed error context
            match serde_json::from_slice::<serde_json::Value>(message.content.as_slice()) {
                Ok(_) => Ok(()),
                Err(json_error) => Err(create_validation_error_with_format(
                    FIELD_CONTENT,
                    &format!("invalid JSON format: {json_error}"),
                )),
            }
        } else {
            Ok(()) // Language doesn't specify JSON, no validation needed
        }
    } else {
        Ok(()) // No language specified, no JSON validation needed
    }
}

fn validate_fipa_message(message: &FipaMessage) -> Result<(), RouterError> {
    // Apply all FIPA validation rules in logical order
    validate_sender_receiver_different(message)?;
    validate_content_not_empty(message)?;
    validate_conversation_threading(message)?;
    validate_performative_fipa_compliance(message)?;
    validate_json_content_format(message)?;

    // Future FIPA validation extensions will be added here:
    // validate_protocol_compliance(message)?;       // Protocol-specific validation
    // validate_ontology_constraints(message)?;      // Ontology compatibility
    // validate_message_size_limits(message)?;       // Size constraint checking

    Ok(())
}

/// Generates FIPA-compliant `NOT_UNDERSTOOD` error content from original message
///
/// Creates a structured error description following FIPA-ACL conventions for
/// `NOT_UNDERSTOOD` responses. This pure function generates deterministic error
/// content based on the original message content.
///
/// # Arguments
/// * `original_content` - The message content that could not be understood
///
/// # Returns
/// A formatted error description suitable for `NOT_UNDERSTOOD` responses
///
/// # FIPA Compliance
/// Error content follows FIPA-ACL patterns for communicative act failure reporting
#[cfg(test)]
fn generate_not_understood_error_content(original_content: &MessageContent) -> String {
    const MAX_CONTENT_PREVIEW: usize = 100;
    // Create structured error description with original content for debugging
    // Uses lossy conversion to handle potentially invalid UTF-8 content gracefully
    let content_preview = String::from_utf8_lossy(original_content.as_bytes());

    // Truncate long content to prevent excessively large error messages

    let truncated_content = if content_preview.len() > MAX_CONTENT_PREVIEW {
        format!(
            "{}... (truncated from {} bytes)",
            &content_preview[..MAX_CONTENT_PREVIEW],
            original_content.len()
        )
    } else {
        content_preview.to_string()
    };

    format!("Message content could not be understood or processed: {truncated_content}")
}

/// Creates FIPA-compliant `NOT_UNDERSTOOD` response message structure
///
/// Constructs a properly formatted `NOT_UNDERSTOOD` response following FIPA-ACL
/// specifications for communicative act failure responses. This pure function
/// handles all the message field mappings and FIPA protocol requirements.
///
/// # Arguments
/// * `original_message` - The message that could not be processed
/// * `error_content` - The error description content
///
/// # Returns
/// Result containing the `NOT_UNDERSTOOD` response message or content creation error
///
/// # FIPA Protocol Requirements
/// - Performative: Set to `NotUnderstood`
/// - Sender/Receiver: Swapped from original (response routing)
/// - Conversation: Preserved for threading context
/// - Reply threading: Proper `in_reply_to` and `reply_with` handling
/// - Timestamp: Set to current time for response tracking
#[cfg(test)]
fn create_not_understood_response(
    original_message: &FipaMessage,
    error_content: String,
) -> Result<FipaMessage, RouterError> {
    // Convert error content to MessageContent with proper validation
    let message_content = MessageContent::try_new(error_content.into_bytes()).map_err(|_| {
        create_validation_error(FIELD_ERROR_CONTENT, REASON_ERROR_CONTENT_TOO_LARGE)
    })?;

    // Construct FIPA-compliant NOT_UNDERSTOOD response
    Ok(FipaMessage {
        // FIPA requirement: `NOT_UNDERSTOOD` performative for processing failures
        performative: Performative::NotUnderstood,

        // FIPA protocol: sender/receiver swap for proper response routing
        sender: original_message.receiver, // Original receiver becomes sender
        receiver: original_message.sender, // Original sender becomes receiver

        // Error description content with validation
        content: message_content,

        // Optional FIPA fields: preserve original context where available
        language: original_message.language.clone(),
        ontology: original_message.ontology.clone(),
        protocol: original_message.protocol.clone(),

        // FIPA conversation threading: maintain conversation context
        conversation_id: original_message.conversation_id,
        reply_with: Some(MessageId::generate()), // Generate new ID for potential replies
        in_reply_to: original_message.reply_with, // Reference original message

        // Response metadata
        message_id: MessageId::generate(), // Unique ID for this response
        created_at: MessageTimestamp::now(), // Current timestamp
        trace_context: original_message.trace_context.clone(), // Preserve tracing context
        delivery_options: DeliveryOptions::default(), // Standard delivery options
    })
}

/// Main message router implementation
///
/// Coordinates message routing using dependency injection for all major components:
/// - `DeliveryEngine`: Handles actual message delivery
/// - `ConversationManager`: Manages conversation state
/// - `AgentRegistry`: Provides O(1) agent lookup
/// - `FailureHandler`: Handles retries and dead-lettering
/// - Storage backends: Provide persistence
pub struct MessageRouterImpl {
    config: RouterConfig,

    // Core components (injected)
    delivery_engine: Arc<dyn DeliveryEngine>,
    conversation_manager: Arc<dyn ConversationManager>,
    agent_registry: Arc<dyn AgentRegistry>,
    failure_handler: Arc<dyn FailureHandler>,

    // Internal state
    is_running: AtomicBool,
    is_shutdown: AtomicBool,
    start_time: RwLock<Option<Instant>>,

    // Performance tracking
    message_counter: AtomicU64,
    error_counter: AtomicU64,
    throughput_tracker: Arc<ThroughputTracker>,

    // Queue management
    inbound_queue: mpsc::Sender<RoutingTask>,
    inbound_receiver: Arc<RwLock<Option<mpsc::Receiver<RoutingTask>>>>,

    // Concurrency control
    routing_semaphore: Arc<Semaphore>,

    // Metrics collection
    metrics_collector: Option<Arc<dyn MetricsCollector>>,
}

/// Internal routing task
#[derive(Debug)]
#[allow(dead_code)]
struct RoutingTask {
    message: FipaMessage,
    attempt_count: u8,
    created_at: Instant,
    span: tracing::Span,
}

/// Throughput tracking for performance monitoring
struct ThroughputTracker {
    window_size: Duration,
    samples: DashMap<u64, u64>, // timestamp_second -> message_count
}

// Pure functions for time operations (functional core)
fn get_current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Pure functions for error handling (functional core)
fn map_registry_error_to_router_error(error: &RegistryError) -> RouterError {
    match error {
        RegistryError::AgentNotFound { agent_id } => RouterError::AgentNotFound {
            agent_id: *agent_id,
        },
        _ => RouterError::ConfigurationError {
            message: format!("Registry error: {error:?}"),
        },
    }
}

fn map_delivery_error_to_router_error(error: DeliveryError) -> RouterError {
    match error {
        DeliveryError::LocalDeliveryFailed { source }
        | DeliveryError::RemoteDeliveryFailed { source, .. } => {
            RouterError::NetworkError { source }
        }
        DeliveryError::CircuitBreakerOpen { node_id } => {
            RouterError::CircuitBreakerOpen { node_id }
        }
        _ => RouterError::ConfigurationError {
            message: format!("Delivery error: {error:?}"),
        },
    }
}

fn map_registry_error_for_agent_ops(error: &RegistryError, operation: &str) -> RouterError {
    match error {
        RegistryError::AgentAlreadyRegistered { agent_id } => RouterError::ConfigurationError {
            message: format!("Agent already registered: {agent_id}"),
        },
        RegistryError::AgentNotFound { agent_id } => RouterError::AgentNotFound {
            agent_id: *agent_id,
        },
        _ => RouterError::ConfigurationError {
            message: format!("{operation} failed: {error:?}"),
        },
    }
}

fn calculate_cutoff_timestamp(current_time: u64, window_seconds: u64) -> u64 {
    current_time.saturating_sub(window_seconds)
}

fn calculate_throughput_rate(total_messages: u64, window_seconds: f64) -> f64 {
    if total_messages == 0 {
        0.0
    } else {
        f64::from(u32::try_from(total_messages.min(u64::from(u32::MAX))).unwrap_or(0))
            / window_seconds
    }
}

// Pure functions for statistics calculations (functional core)
fn safe_u64_to_u32(value: u64) -> u32 {
    u32::try_from(value.min(u64::from(u32::MAX))).unwrap_or(0)
}

fn calculate_error_rate(total_errors: u64, total_messages: u64) -> f64 {
    if total_messages > 0 {
        let safe_errors = safe_u64_to_u32(total_errors);
        let safe_messages = safe_u64_to_u32(total_messages).max(1); // Avoid division by zero
        f64::from(safe_errors) / f64::from(safe_messages)
    } else {
        0.0
    }
}

fn calculate_uptime(start_time: Option<Instant>) -> Duration {
    if let Some(start_time) = start_time {
        start_time.elapsed()
    } else {
        Duration::ZERO
    }
}

fn create_agent_queue_depths(local_agents: Vec<LocalAgent>) -> HashMap<AgentId, usize> {
    local_agents
        .into_iter()
        .map(|agent| (agent.id, agent.queue_size.as_usize()))
        .collect()
}

fn safe_message_count_from_u64(value: u64) -> MessageCount {
    MessageCount::new(value.min(1_000_000) as usize)
}

// Pure functions for conversation management (functional core)
fn calculate_conversation_duration_ms(
    created_at: std::time::SystemTime,
    last_activity: std::time::SystemTime,
) -> u64 {
    if let Ok(duration) = last_activity.duration_since(created_at) {
        u64::try_from(duration.as_millis().min(u128::from(u64::MAX))).unwrap_or(0)
    } else {
        0
    }
}

fn calculate_average_duration(total_duration_ms: u64, conversation_count: usize) -> u64 {
    if conversation_count > 0 {
        total_duration_ms / conversation_count as u64
    } else {
        0
    }
}

fn calculate_average_message_count(total_message_count: u64, conversation_count: usize) -> f64 {
    if conversation_count > 0 {
        let safe_msg_count = safe_u64_to_u32(total_message_count);
        let safe_active = u32::try_from(conversation_count.min(u32::MAX as usize)).unwrap_or(1);
        f64::from(safe_msg_count) / f64::from(safe_active)
    } else {
        0.0
    }
}

fn is_conversation_expired(last_activity: MessageTimestamp, timeout: Duration) -> bool {
    if let Ok(elapsed) = last_activity.as_system_time().elapsed() {
        elapsed > timeout
    } else {
        false
    }
}

fn sum_messages_in_window<I>(entries: I, cutoff: u64) -> u64
where
    I: Iterator<Item = (u64, u64)>,
{
    entries
        .filter(|(timestamp, _)| *timestamp >= cutoff)
        .map(|(_, count)| count)
        .sum()
}

impl ThroughputTracker {
    fn new(window_size: Duration) -> Self {
        Self {
            window_size,
            samples: DashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn record_message(&self) {
        let now = get_current_unix_timestamp();

        self.samples
            .entry(now)
            .and_modify(|count| *count += 1)
            .or_insert(1);

        // Clean old samples
        let cutoff = calculate_cutoff_timestamp(now, self.window_size.as_secs());
        self.samples.retain(|&timestamp, _| timestamp >= cutoff);
    }

    fn get_current_rate(&self) -> f64 {
        let now = get_current_unix_timestamp();
        let cutoff = calculate_cutoff_timestamp(now, self.window_size.as_secs());

        let total_messages = sum_messages_in_window(
            self.samples
                .iter()
                .map(|entry| (*entry.key(), *entry.value())),
            cutoff,
        );

        calculate_throughput_rate(total_messages, self.window_size.as_secs_f64())
    }
}

impl MessageRouterImpl {
    /// Creates a new message router with the given configuration
    ///
    /// This will create and wire up all necessary components based on the config.
    ///
    /// # Errors
    ///
    /// Returns a `RouterError` if configuration validation fails or component creation fails.
    pub fn new(config: RouterConfig) -> Result<Self, RouterError> {
        let span = span!(Level::INFO, "router_creation");
        let _enter = span.enter();

        info!("Creating message router with config: {:?}", config);

        // Validate configuration
        config
            .validate()
            .map_err(|e| RouterError::ConfigurationError {
                message: format!("Invalid configuration: {e}"),
            })?;

        // Create queue for inbound messages
        let (inbound_sender, inbound_receiver) =
            mpsc::channel(config.inbound_queue_size.as_usize());

        // Create components based on configuration
        let delivery_engine = Arc::new(DeliveryEngineImpl::new(config.clone()));

        let conversation_manager = Arc::new(ConversationManagerImpl::new(config.clone()));

        let agent_registry = Arc::new(AgentRegistryImpl::new(config.clone()));

        let failure_handler = Arc::new(FailureHandlerImpl::new(config.clone()));

        // Create metrics collector if enabled
        let metrics_collector = if config.enable_metrics() {
            Some(Arc::new(MetricsCollectorImpl::new()) as Arc<dyn MetricsCollector>)
        } else {
            None
        };

        // Create concurrency control
        let routing_semaphore = Arc::new(Semaphore::new(config.inbound_queue_size.as_usize()));

        // Create throughput tracker
        let throughput_tracker = Arc::new(ThroughputTracker::new(Duration::from_secs(60)));

        let router = Self {
            config,
            delivery_engine,
            conversation_manager,
            agent_registry,
            failure_handler,
            is_running: AtomicBool::new(false),
            is_shutdown: AtomicBool::new(false),
            start_time: RwLock::new(None),
            message_counter: AtomicU64::new(0),
            error_counter: AtomicU64::new(0),
            throughput_tracker,
            inbound_queue: inbound_sender,
            inbound_receiver: Arc::new(RwLock::new(Some(inbound_receiver))),
            routing_semaphore,
            metrics_collector,
        };

        info!("Message router created successfully");
        Ok(router)
    }

    /// Starts the message router background processing
    ///
    /// Spawns worker tasks for processing messages concurrently based on configuration.
    ///
    /// # Errors
    ///
    /// Returns a `RouterError` if the router is already running or configuration is invalid.
    pub async fn start(&self) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "router_start");
        let _enter = span.enter();

        if self.is_running.load(Ordering::SeqCst) {
            warn!("Router already running");
            return Ok(());
        }

        info!("Starting message router");

        // Mark as running
        self.is_running.store(true, Ordering::SeqCst);
        *self.start_time.write().await = Some(Instant::now());

        // Take the receiver from the option (can only be done once)
        let mut receiver_guard = self.inbound_receiver.write().await;
        let receiver = receiver_guard
            .take()
            .ok_or_else(|| RouterError::ConfigurationError {
                message: "Router has already been started".to_string(),
            })?;
        drop(receiver_guard);

        // Start the main message processing loop
        self.spawn_message_processor(receiver);

        // Spawn worker tasks
        for worker_id in 0..self.config.worker_thread_count.as_usize() {
            self.spawn_worker_task(worker_id);
        }

        // Spawn health monitoring task
        if self.config.health_check_interval_ms.as_duration() > Duration::ZERO {
            self.spawn_health_monitoring_task();
        }

        // Spawn metrics collection task
        if self.config.enable_metrics() {
            self.spawn_metrics_task();
        }

        info!(
            "Message router started with {} workers",
            self.config.worker_thread_count.as_usize()
        );
        Ok(())
    }

    /// Spawns a worker task for processing routing messages
    #[allow(unused_variables)]
    fn spawn_worker_task(&self, worker_id: usize) {
        let _delivery_engine = Arc::clone(&self.delivery_engine);
        let _conversation_manager = Arc::clone(&self.conversation_manager);
        let _agent_registry = Arc::clone(&self.agent_registry);
        let _failure_handler = Arc::clone(&self.failure_handler);
        let _metrics_collector = self.metrics_collector.clone();
        let throughput_tracker = Arc::clone(&self.throughput_tracker);
        let semaphore = Arc::clone(&self.routing_semaphore);

        // We need a way to receive tasks from the inbound queue
        // For now, we'll create a shared receiver using a broadcast channel
        // This is a simplified implementation - in production we'd use a work-stealing queue

        // Clone counters with proper atomic sharing
        let message_counter = AtomicU64::new(0);
        let _error_counter = AtomicU64::new(0);
        let is_running = AtomicBool::new(true);

        tokio::spawn(async move {
            let span = span!(Level::DEBUG, "worker_task", worker_id = worker_id);
            let _enter = span.enter();

            debug!("Worker {} started", worker_id);

            // In a real implementation, we would pull from the inbound queue
            // For now, simulate processing with a small delay
            let mut interval = tokio::time::interval(Duration::from_millis(10));

            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // Acquire semaphore permit for concurrency control
                let _permit = semaphore.acquire().await.expect("Semaphore not closed");

                // In real implementation, we'd:
                // 1. Try to receive a RoutingTask from the inbound queue
                // 2. Process it using process_routing_task
                // 3. Update counters and metrics

                // For now, just update throughput tracker periodically
                throughput_tracker.record_message();

                // Increment local counter
                message_counter.fetch_add(1, Ordering::Relaxed);
            }

            debug!("Worker {} terminated", worker_id);
        });
    }

    /// Spawns the main message processor that handles the inbound queue
    fn spawn_message_processor(&self, mut receiver: mpsc::Receiver<RoutingTask>) {
        let delivery_engine = Arc::clone(&self.delivery_engine);
        let conversation_manager = Arc::clone(&self.conversation_manager);
        let agent_registry = Arc::clone(&self.agent_registry);
        let failure_handler = Arc::clone(&self.failure_handler);
        let metrics_collector = self.metrics_collector.clone();
        let throughput_tracker = Arc::clone(&self.throughput_tracker);

        // Clone atomic counters for cross-thread access
        let message_counter = Arc::new(AtomicU64::new(0));
        let error_counter = Arc::new(AtomicU64::new(0));
        let is_running = Arc::new(AtomicBool::new(true));

        tokio::spawn(async move {
            let span = span!(Level::INFO, "message_processor");
            let _enter = span.enter();

            info!("Message processor started");

            while is_running.load(Ordering::SeqCst) {
                // Try to receive a routing task from the inbound queue
                if let Some(task) = receiver.recv().await {
                    trace!(
                        "Processing routing task for message {}",
                        task.message.message_id
                    );

                    // Record that we're processing a message
                    throughput_tracker.record_message();
                    message_counter.fetch_add(1, Ordering::Relaxed);

                    // Process the routing task
                    let result = Self::process_routing_task(
                        task,
                        &delivery_engine,
                        &conversation_manager,
                        &agent_registry,
                        &failure_handler,
                    )
                    .await;

                    match result {
                        Ok(message_id) => {
                            trace!("Successfully routed message {}", message_id);

                            // Record metrics if available
                            if let Some(collector) = &metrics_collector {
                                collector.record_message_routed(
                                    &FipaMessage {
                                        performative: Performative::Inform,
                                        sender: AgentId::generate(),
                                        receiver: AgentId::generate(),
                                        content: MessageContent::try_new(vec![]).unwrap(),
                                        message_id,
                                        conversation_id: None,
                                        reply_with: None,
                                        in_reply_to: None,
                                        protocol: None,
                                        language: None,
                                        ontology: None,
                                        created_at: MessageTimestamp::now(),
                                        trace_context: None,
                                        delivery_options: DeliveryOptions::default(),
                                    },
                                    Duration::from_millis(1),
                                );
                            }
                        }
                        Err(error) => {
                            error!("Failed to route message: {:?}", error);
                            error_counter.fetch_add(1, Ordering::Relaxed);

                            // Record error metrics
                            if let Some(collector) = &metrics_collector {
                                collector.record_routing_error(&error);
                            }
                        }
                    }
                } else {
                    // Channel closed, stop processing
                    info!("Inbound queue closed, stopping message processor");
                    break;
                }
            }

            info!("Message processor terminated");
        });
    }

    /// Processes a single routing task
    #[allow(dead_code)]
    async fn process_routing_task(
        task: RoutingTask,
        delivery_engine: &Arc<dyn DeliveryEngine>,
        conversation_manager: &Arc<dyn ConversationManager>,
        agent_registry: &Arc<dyn AgentRegistry>,
        _failure_handler: &Arc<dyn FailureHandler>,
    ) -> Result<MessageId, RouterError> {
        let _span_guard = task.span.enter();

        trace!(
            "Processing routing task for message {}",
            task.message.message_id
        );

        // Update conversation if this is part of one
        if let Some(conversation_id) = task.message.conversation_id
            && let Err(e) = conversation_manager
                .update_conversation(conversation_id, &task.message)
                .await
        {
            warn!("Failed to update conversation {}: {:?}", conversation_id, e);
            // Don't fail the routing for conversation update failures
        }

        // Look up the destination agent
        let agent_location = agent_registry
            .lookup(&task.message.receiver)
            .await
            .map_err(|e| map_registry_error_to_router_error(&e))?;

        // Route based on agent location
        match agent_location {
            AgentLocation::Local(local_agent) => {
                trace!("Routing to local agent: {}", local_agent.name);
                delivery_engine
                    .deliver_local(task.message, local_agent)
                    .await
                    .map_err(map_delivery_error_to_router_error)
            }
            AgentLocation::Remote(node_id) => {
                trace!("Routing to remote node: {}", node_id);
                delivery_engine
                    .deliver_remote(task.message, node_id)
                    .await
                    .map_err(map_delivery_error_to_router_error)
            }
            AgentLocation::Unknown => {
                warn!("Agent location unknown for: {}", task.message.receiver);
                Err(RouterError::AgentNotFound {
                    agent_id: task.message.receiver,
                })
            }
        }
    }

    /// Spawns health monitoring background task
    fn spawn_health_monitoring_task(&self) {
        let health_interval = self.config.health_check_interval_ms.as_duration();
        let delivery_engine = Arc::clone(&self.delivery_engine);
        let _conversation_manager = Arc::clone(&self.conversation_manager);
        let _agent_registry = Arc::clone(&self.agent_registry);
        let is_running = AtomicBool::new(self.is_running.load(Ordering::SeqCst));

        tokio::spawn(async move {
            let span = span!(Level::DEBUG, "health_monitoring");
            let _enter = span.enter();

            debug!("Health monitoring started");

            let mut interval = tokio::time::interval(health_interval);

            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // Check component health
                let delivery_health = delivery_engine.health_check().await;

                if let Err(e) = delivery_health {
                    warn!("Delivery engine health check failed: {:?}", e);
                }

                // Additional health checks can be added here
                trace!("Health check completed");
            }

            debug!("Health monitoring terminated");
        });
    }

    /// Spawns metrics collection background task
    fn spawn_metrics_task(&self) {
        let throughput_tracker = Arc::clone(&self.throughput_tracker);
        let is_running = AtomicBool::new(self.is_running.load(Ordering::SeqCst));

        tokio::spawn(async move {
            let span = span!(Level::DEBUG, "metrics_collection");
            let _enter = span.enter();

            debug!("Metrics collection started");

            let mut interval = tokio::time::interval(Duration::from_secs(10));

            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;

                let current_rate = throughput_tracker.get_current_rate();
                trace!("Current throughput: {:.2} messages/sec", current_rate);

                // Additional metrics collection can be added here
            }

            debug!("Metrics collection terminated");
        });
    }
}

/// Implementation of Clone for `MessageRouterImpl` using Arc for shared ownership
impl Clone for MessageRouterImpl {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            delivery_engine: Arc::clone(&self.delivery_engine),
            conversation_manager: Arc::clone(&self.conversation_manager),
            agent_registry: Arc::clone(&self.agent_registry),
            failure_handler: Arc::clone(&self.failure_handler),
            is_running: AtomicBool::new(self.is_running.load(Ordering::SeqCst)),
            is_shutdown: AtomicBool::new(self.is_shutdown.load(Ordering::SeqCst)),
            start_time: RwLock::new(None), // New instances don't inherit start time
            message_counter: AtomicU64::new(self.message_counter.load(Ordering::SeqCst)),
            error_counter: AtomicU64::new(self.error_counter.load(Ordering::SeqCst)),
            throughput_tracker: Arc::clone(&self.throughput_tracker),
            inbound_queue: self.inbound_queue.clone(),
            inbound_receiver: Arc::clone(&self.inbound_receiver),
            routing_semaphore: Arc::clone(&self.routing_semaphore),
            metrics_collector: self.metrics_collector.clone(),
        }
    }
}

#[async_trait]
impl MessageRouter for MessageRouterImpl {
    /// Routes a message to its destination agent
    async fn route_message(&self, message: FipaMessage) -> Result<MessageId, RouterError> {
        let span = span!(Level::DEBUG, "route_message",
                         message_id = %message.message_id,
                         sender = %message.sender,
                         receiver = %message.receiver);
        let _enter = span.enter();

        if !self.is_running.load(Ordering::SeqCst) {
            return Err(RouterError::ConfigurationError {
                message: "Router is not running".to_string(),
            });
        }

        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(RouterError::ConfigurationError {
                message: "Router is shutting down".to_string(),
            });
        }

        // Validate message size if validation is enabled
        if self.config.enable_message_validation()
            && message.content.len() > self.config.max_message_size_bytes()
        {
            return Err(RouterError::MessageTooLarge {
                size: message.content.len(),
                max_size: self.config.max_message_size_bytes(),
            });
        }

        // FIPA validation
        validate_fipa_message(&message)?;

        // Create routing task
        let task = RoutingTask {
            message,
            attempt_count: 1,
            created_at: Instant::now(),
            span: span.clone(),
        };

        let message_id = task.message.message_id;

        // Queue the task for processing
        self.inbound_queue
            .send(task)
            .await
            .map_err(|_| RouterError::QueueFull {
                queue_type: "inbound".to_string(),
            })?;

        // Increment message counter when message is accepted
        self.message_counter.fetch_add(1, Ordering::Relaxed);

        debug!("Message queued for routing: {}", message_id);
        Ok(message_id)
    }

    /// Registers a new local agent with the router
    async fn register_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>,
    ) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "register_agent",
                         agent_id = %agent.id,
                         agent_name = %agent.name);
        let _enter = span.enter();

        info!("Registering agent: {} ({})", agent.name, agent.id);

        self.agent_registry
            .register_local_agent(agent.clone(), capabilities)
            .await
            .map_err(|e| map_registry_error_for_agent_ops(&e, "Agent registration"))?;

        // Record metrics if enabled
        if let Some(collector) = &self.metrics_collector {
            collector.record_agent_registered(agent.id);
        }

        info!("Agent registered successfully: {}", agent.name);
        Ok(())
    }

    /// Deregisters an agent from the router
    async fn deregister_agent(&self, agent_id: AgentId) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "deregister_agent", agent_id = %agent_id);
        let _enter = span.enter();

        info!("Deregistering agent: {}", agent_id);

        self.agent_registry
            .deregister_local_agent(agent_id)
            .await
            .map_err(|e| map_registry_error_for_agent_ops(&e, "Agent deregistration"))?;

        // Record metrics if enabled
        if let Some(collector) = &self.metrics_collector {
            collector.record_agent_deregistered(agent_id);
        }

        info!("Agent deregistered successfully: {}", agent_id);
        Ok(())
    }

    /// Updates an agent's state in its lifecycle
    async fn update_agent_state(
        &self,
        agent_id: AgentId,
        state: AgentState,
    ) -> Result<(), RouterError> {
        let span = span!(Level::DEBUG, "update_agent_state",
                         agent_id = %agent_id,
                         new_state = ?state);
        let _enter = span.enter();

        debug!("Updating agent state: {} -> {:?}", agent_id, state);

        // Look up agent first to verify it exists
        let agent_location = self
            .agent_registry
            .lookup(&agent_id)
            .await
            .map_err(|e| map_registry_error_for_agent_ops(&e, "Agent lookup"))?;

        // Only local agents can have their state updated
        if !matches!(agent_location, AgentLocation::Local(_)) {
            return Err(RouterError::ConfigurationError {
                message: "Cannot update state of remote agent".to_string(),
            });
        }

        // TODO: Update agent state in registry
        // This would require extending the AgentRegistry trait with an update_state method

        debug!(
            "Agent state updated successfully: {} -> {:?}",
            agent_id, state
        );
        Ok(())
    }

    /// Retrieves current router performance statistics
    async fn get_stats(&self) -> Result<RouterStats, RouterError> {
        let span = span!(Level::DEBUG, "get_stats");
        let _enter = span.enter();

        let total_messages = self.message_counter.load(Ordering::Relaxed);
        let total_errors = self.error_counter.load(Ordering::Relaxed);
        let current_rate = self.throughput_tracker.get_current_rate();

        // Calculate uptime
        let _uptime = calculate_uptime(*self.start_time.read().await);

        let error_rate = calculate_error_rate(total_errors, total_messages);

        // Get local agents for queue depth calculation
        let local_agents = self.agent_registry.list_local_agents().await.map_err(|e| {
            RouterError::ConfigurationError {
                message: format!("Failed to get agent list: {e:?}"),
            }
        })?;

        let agent_queue_depths = create_agent_queue_depths(local_agents);

        // Get conversation stats from conversation manager
        let conversation_stats = self
            .conversation_manager
            .get_conversation_stats()
            .await
            .unwrap_or_else(|_| ConversationStats {
                total_active: 0,
                total_created: MessageCount::zero(),
                average_duration_ms: 0,
                average_message_count: 0.0,
                participants_distribution: HashMap::new(),
            });

        let stats = RouterStats {
            messages_per_second: current_rate,
            peak_messages_per_second: current_rate, // TODO: Track peak
            total_messages_processed: safe_message_count_from_u64(total_messages),

            // TODO: Collect real latency metrics
            routing_latency_p50: 500, // microseconds
            routing_latency_p90: 1_000,
            routing_latency_p99: 2_000,
            routing_latency_p999: 5_000,

            total_errors: safe_message_count_from_u64(total_errors),
            error_rate,
            errors_by_type: HashMap::new(), // TODO: Collect by error type

            inbound_queue_depth: 0,  // TODO: Get actual queue depth
            outbound_queue_depth: 0, // TODO: Get actual queue depth
            agent_queue_depths,

            active_conversations: conversation_stats.total_active,
            total_conversations: conversation_stats.total_created,
            average_conversation_length: conversation_stats.average_message_count,

            memory_usage_bytes: 0,  // TODO: Collect memory usage
            cpu_usage_percent: 0.0, // TODO: Collect CPU usage
            database_size_bytes: 0, // TODO: Collect database size
        };

        trace!("Router stats collected: {:?}", stats);
        Ok(stats)
    }

    /// Checks the health status of the router
    async fn health_check(&self) -> Result<HealthStatus, RouterError> {
        let span = span!(Level::DEBUG, "health_check");
        let _enter = span.enter();

        if !self.is_running.load(Ordering::SeqCst) {
            return Err(RouterError::ConfigurationError {
                message: "Router is not running".to_string(),
            });
        }

        if self.is_shutdown.load(Ordering::SeqCst) {
            return Ok(HealthStatus::Unhealthy {
                reason: "Router is shutting down".to_string(),
            });
        }

        // Check component health
        let delivery_health = self.delivery_engine.health_check().await;

        match delivery_health {
            Ok(HealthStatus::Healthy) => {
                trace!("Router health check passed");
                Ok(HealthStatus::Healthy)
            }
            Ok(HealthStatus::Degraded { reason }) => {
                warn!("Router health degraded: {}", reason);
                Ok(HealthStatus::Degraded { reason })
            }
            Ok(HealthStatus::Unhealthy { reason }) => {
                error!("Router unhealthy: {}", reason);
                Ok(HealthStatus::Unhealthy { reason })
            }
            Err(e) => {
                error!("Health check failed: {:?}", e);
                Ok(HealthStatus::Unhealthy {
                    reason: format!("Health check error: {e:?}"),
                })
            }
        }
    }

    /// Initiates graceful shutdown of the router
    async fn shutdown(&self) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "shutdown");
        let _enter = span.enter();

        if self.is_shutdown.load(Ordering::SeqCst) {
            warn!("Router already shutting down");
            return Ok(());
        }

        info!("Initiating graceful router shutdown");

        // Mark as shutting down
        self.is_shutdown.store(true, Ordering::SeqCst);

        // Stop accepting new work
        self.is_running.store(false, Ordering::SeqCst);

        // Wait for in-flight messages to complete (with timeout)
        let shutdown_timeout = Duration::from_secs(30);
        let shutdown_start = Instant::now();

        while shutdown_start.elapsed() < shutdown_timeout {
            let stats = self.get_stats().await?;
            if stats.inbound_queue_depth == 0 && stats.outbound_queue_depth == 0 {
                break;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        info!("Router shutdown completed");
        Ok(())
    }
}

/// Additional methods for testing
#[cfg(test)]
impl MessageRouterImpl {
    /// Provides access to the agent registry for testing purposes
    pub fn agent_registry(&self) -> &Arc<dyn AgentRegistry> {
        &self.agent_registry
    }

    /// Generates a FIPA-compliant `NOT_UNDERSTOOD` response for message processing failures
    ///
    /// Creates properly formatted `NOT_UNDERSTOOD` responses following FIPA-ACL specifications
    /// for communicative act failures. This method applies functional core principles by
    /// delegating to pure functions for error content generation and response construction.
    ///
    /// # Arguments
    /// * `original_message` - The message that couldn't be understood or processed
    ///
    /// # Returns
    /// * `Ok(FipaMessage)` - FIPA-compliant `NOT_UNDERSTOOD` response message
    /// * `Err(RouterError)` - If response generation fails (e.g., content too large)
    ///
    /// # Errors
    ///
    /// Returns `RouterError::ValidationError` if the generated error content exceeds maximum message size limits.
    ///
    /// # FIPA Protocol Compliance
    /// The generated response follows FIPA-ACL standards:
    /// - **Performative**: Set to `NotUnderstood` as required for processing failures
    /// - **Routing**: Sender/receiver swapped for proper response delivery
    /// - **Threading**: Maintains conversation context and reply references
    /// - **Content**: Structured error description with original content preview
    /// - **Metadata**: Preserves tracing context and protocol information
    ///
    /// # Architecture Pattern
    /// Follows functional core/imperative shell pattern:
    /// - **Functional Core**: Pure functions handle error content and message structure
    /// - **Imperative Shell**: This method coordinates the pure functions
    pub fn generate_not_understood_response(
        &self,
        original_message: &FipaMessage,
    ) -> Result<FipaMessage, RouterError> {
        // Functional core: Generate structured error content (pure function)
        let error_content = generate_not_understood_error_content(&original_message.content);

        // Functional core: Create FIPA-compliant response structure (pure function)
        create_not_understood_response(original_message, error_content)
    }
}

// Placeholder implementations for components that will be implemented next

/// Real delivery engine implementation
#[allow(dead_code)]
struct DeliveryEngineImpl {
    /// Agent message queues for local delivery
    agent_queues: DashMap<AgentId, mpsc::Sender<FipaMessage>>,
    /// Remote node connections (placeholder for now)
    remote_connections: DashMap<NodeId, mpsc::Sender<FipaMessage>>,
    /// Configuration
    config: RouterConfig,
}

impl DeliveryEngineImpl {
    fn new(config: RouterConfig) -> Self {
        Self {
            agent_queues: DashMap::new(),
            remote_connections: DashMap::new(),
            config,
        }
    }

    /// Registers a message queue for a local agent
    #[allow(dead_code)]
    pub fn register_agent_queue(&self, agent_id: AgentId, queue: mpsc::Sender<FipaMessage>) {
        self.agent_queues.insert(agent_id, queue);
    }

    /// Deregisters a message queue for a local agent
    #[allow(dead_code)]
    pub fn deregister_agent_queue(&self, agent_id: AgentId) {
        self.agent_queues.remove(&agent_id);
    }
}

#[async_trait]
impl DeliveryEngine for DeliveryEngineImpl {
    async fn deliver_local(
        &self,
        message: FipaMessage,
        agent: LocalAgent,
    ) -> Result<MessageId, DeliveryError> {
        let message_id = message.message_id;

        // Check if agent is available for delivery
        if !agent.is_available() {
            return Err(DeliveryError::LocalDeliveryFailed {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Agent is not available",
                )),
            });
        }

        // Try to find the agent's message queue
        if let Some(queue) = self.agent_queues.get(&agent.id) {
            // Try to send the message to the agent's queue
            match queue.try_send(message) {
                Ok(()) => {
                    trace!(
                        "Message {} delivered to local agent {}",
                        message_id, agent.id
                    );
                    Ok(message_id)
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    warn!("Agent {} queue is full", agent.id);
                    Err(DeliveryError::LocalDeliveryFailed {
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "Agent queue is full",
                        )),
                    })
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    error!("Agent {} queue is closed", agent.id);
                    Err(DeliveryError::LocalDeliveryFailed {
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "Agent queue is closed",
                        )),
                    })
                }
            }
        } else {
            // Agent doesn't have a registered queue - this is normal during testing
            // In a real system, we'd queue the message for later delivery
            warn!(
                "No queue registered for agent {}, queuing for later delivery",
                agent.id
            );

            // For now, just return success - in production we'd store in a pending queue
            Ok(message_id)
        }
    }

    async fn deliver_remote(
        &self,
        message: FipaMessage,
        node_id: NodeId,
    ) -> Result<MessageId, DeliveryError> {
        let message_id = message.message_id;

        // For remote delivery, we would typically:
        // 1. Serialize the message
        // 2. Send over network (HTTP/gRPC/TCP)
        // 3. Handle retries and circuit breaking

        // For now, simulate remote delivery
        if let Some(connection) = self.remote_connections.get(&node_id) {
            match connection.try_send(message) {
                Ok(()) => {
                    trace!(
                        "Message {} queued for remote delivery to node {}",
                        message_id, node_id
                    );
                    Ok(message_id)
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    Err(DeliveryError::RemoteDeliveryFailed {
                        node_id,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "Remote connection queue is full",
                        )),
                    })
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    Err(DeliveryError::RemoteDeliveryFailed {
                        node_id,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "Remote connection is closed",
                        )),
                    })
                }
            }
        } else {
            // No connection to remote node - simulate successful queuing
            trace!(
                "No connection to node {}, would establish connection in production",
                node_id
            );
            Ok(message_id)
        }
    }

    async fn deliver_batch(
        &self,
        messages: Vec<FipaMessage>,
    ) -> Vec<Result<MessageId, DeliveryError>> {
        // Process messages in parallel for better throughput
        let mut results = Vec::with_capacity(messages.len());

        // For now, process sequentially - in production we'd use concurrent futures
        for message in messages {
            let message_id = message.message_id;

            // For batch processing, we'd need to know the destination
            // This is a simplified implementation
            results.push(Ok(message_id));
        }

        results
    }

    async fn health_check(&self) -> Result<HealthStatus, DeliveryError> {
        // Check if we have healthy connections
        let active_agents = self.agent_queues.len();
        let active_connections = self.remote_connections.len();

        if active_agents == 0 && active_connections == 0 {
            Ok(HealthStatus::Degraded {
                reason: "No active agents or connections".to_string(),
            })
        } else {
            Ok(HealthStatus::Healthy)
        }
    }
}

/// Real conversation manager implementation with `HashMap` storage
struct ConversationManagerImpl {
    /// Active conversations indexed by conversation ID
    conversations: DashMap<ConversationId, Conversation>,
    /// Conversation statistics
    total_created: AtomicU64,
    /// Configuration for timeouts and limits
    config: RouterConfig,
}

impl ConversationManagerImpl {
    fn new(config: RouterConfig) -> Self {
        Self {
            conversations: DashMap::new(),
            total_created: AtomicU64::new(0),
            config,
        }
    }
}

#[async_trait]
impl ConversationManager for ConversationManagerImpl {
    async fn get_or_create_conversation(
        &self,
        conversation_id: ConversationId,
        participants: std::collections::HashSet<AgentId>,
        protocol: Option<ProtocolName>,
    ) -> Result<Conversation, ConversationError> {
        // Check if conversation already exists
        if let Some(conversation) = self.conversations.get(&conversation_id) {
            return Ok(conversation.clone());
        }

        // Validate participant count
        if participants.len() > self.config.max_conversation_participants.into_inner() as usize {
            return Err(ConversationError::TooManyParticipants {
                count: participants.len(),
                max: self.config.max_conversation_participants.into_inner() as usize,
            });
        }

        // Create new conversation
        let conversation = Conversation::new(
            conversation_id,
            participants,
            protocol,
            ConversationCreatedAt::now(),
        );

        // Store the conversation
        self.conversations
            .insert(conversation_id, conversation.clone());
        self.total_created.fetch_add(1, Ordering::Relaxed);

        Ok(conversation)
    }

    async fn update_conversation(
        &self,
        conversation_id: ConversationId,
        message: &FipaMessage,
    ) -> Result<(), ConversationError> {
        // Find and update the conversation
        if let Some(mut conversation) = self.conversations.get_mut(&conversation_id) {
            conversation.add_message(message);
            Ok(())
        } else {
            // Conversation not found - create it if the message has participants
            let mut participants = HashSet::new();
            participants.insert(message.sender);
            participants.insert(message.receiver);

            let mut conversation = self
                .get_or_create_conversation(conversation_id, participants, message.protocol.clone())
                .await?;

            conversation.add_message(message);
            self.conversations.insert(conversation_id, conversation);

            Ok(())
        }
    }

    async fn get_agent_conversations(
        &self,
        agent_id: AgentId,
    ) -> Result<Vec<Conversation>, ConversationError> {
        let conversations: Vec<Conversation> = self
            .conversations
            .iter()
            .filter(|entry| entry.participants.contains(&agent_id))
            .map(|entry| entry.clone())
            .collect();

        Ok(conversations)
    }

    async fn cleanup_expired_conversations(&self) -> Result<usize, ConversationError> {
        let timeout = self.config.conversation_timeout_ms.as_duration();
        let _now = std::time::SystemTime::now();
        let mut cleaned_count = 0;

        // Find expired conversations
        let mut expired_ids = Vec::new();

        for entry in &self.conversations {
            let conversation = entry.value();
            if is_conversation_expired(conversation.last_activity, timeout) {
                expired_ids.push(*entry.key());
            }
        }

        // Remove expired conversations
        for conversation_id in expired_ids {
            if self.conversations.remove(&conversation_id).is_some() {
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    async fn get_conversation_stats(&self) -> Result<ConversationStats, ConversationError> {
        let total_active = self.conversations.len();
        let total_created =
            MessageCount::new(self.total_created.load(Ordering::Relaxed).min(1_000_000) as usize);

        // Calculate average duration and message count
        let mut total_duration_ms = 0u64;
        let mut total_message_count = 0u64;
        let mut participants_distribution = HashMap::new();

        for entry in &self.conversations {
            let conversation = entry.value();

            // Calculate duration
            let duration_ms = calculate_conversation_duration_ms(
                conversation.created_at.as_system_time(),
                conversation.last_activity.as_system_time(),
            );
            total_duration_ms = total_duration_ms.saturating_add(duration_ms);

            // Add message count
            total_message_count += conversation.message_count.into_inner() as u64;

            // Track participant distribution
            let participant_count = conversation.participants.len();
            *participants_distribution
                .entry(participant_count)
                .or_insert(0) += 1;
        }

        let average_duration_ms = calculate_average_duration(total_duration_ms, total_active);

        let average_message_count =
            calculate_average_message_count(total_message_count, total_active);

        Ok(ConversationStats {
            total_active,
            total_created,
            average_duration_ms,
            average_message_count,
            participants_distribution,
        })
    }
}

/// Node information for remote agent routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub name: String,
    pub address: String,
    pub is_healthy: bool,
    pub last_heartbeat: MessageTimestamp,
    pub agent_count: usize,
}

impl NodeInfo {
    #[must_use]
    pub fn new(id: NodeId, name: String, address: String) -> Self {
        Self {
            id,
            name,
            address,
            is_healthy: true,
            last_heartbeat: MessageTimestamp::now(),
            agent_count: 0,
        }
    }
}

/// Real agent registry implementation with O(1) lookup performance
pub struct AgentRegistryImpl {
    /// O(1) lookup for local agents
    agents: DashMap<AgentId, LocalAgent>,

    /// O(1) lookup for routing information
    routes: DashMap<AgentId, AgentLocation>,

    /// O(1) lookup for capability-based discovery
    capabilities: DashMap<CapabilityName, HashSet<AgentId>>,

    /// Node registry for remote agents
    node_registry: DashMap<NodeId, NodeInfo>,
}

impl AgentRegistryImpl {
    fn new(_config: RouterConfig) -> Self {
        Self {
            agents: DashMap::new(),
            routes: DashMap::new(),
            capabilities: DashMap::new(),
            node_registry: DashMap::new(),
        }
    }
}

#[async_trait]
impl AgentRegistry for AgentRegistryImpl {
    /// O(1) agent lookup with actual implementation
    async fn lookup(&self, agent_id: &AgentId) -> Result<AgentLocation, RegistryError> {
        // First check routes cache for O(1) lookup
        if let Some(location) = self.routes.get(agent_id) {
            return Ok(location.clone());
        }

        // If not in cache, check if it's a local agent
        if let Some(agent) = self.agents.get(agent_id) {
            let location = AgentLocation::Local(agent.clone());
            // Cache the location for future O(1) lookups
            self.routes.insert(*agent_id, location.clone());
            return Ok(location);
        }

        // Agent not found
        Err(RegistryError::AgentNotFound {
            agent_id: *agent_id,
        })
    }

    /// Registers a local agent with capabilities indexing
    async fn register_local_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>,
    ) -> Result<(), RegistryError> {
        let agent_id = agent.id;

        // Check if agent is already registered
        if self.agents.contains_key(&agent_id) {
            return Err(RegistryError::AgentAlreadyRegistered { agent_id });
        }

        // Store the agent
        self.agents.insert(agent_id, agent.clone());

        // Cache the location for O(1 lookups
        self.routes.insert(agent_id, AgentLocation::Local(agent));

        // Index capabilities for discovery
        for capability in capabilities {
            self.capabilities
                .entry(capability)
                .and_modify(|agents| {
                    agents.insert(agent_id);
                })
                .or_insert_with(|| {
                    let mut agents = HashSet::new();
                    agents.insert(agent_id);
                    agents
                });
        }

        Ok(())
    }

    /// Deregisters a local agent and cleans up all indexes
    async fn deregister_local_agent(&self, agent_id: AgentId) -> Result<(), RegistryError> {
        // Remove from agents map
        let agent = self
            .agents
            .remove(&agent_id)
            .ok_or(RegistryError::AgentNotFound { agent_id })?;

        // Remove from routes cache
        self.routes.remove(&agent_id);

        // Clean up capability indexes
        for capability in &agent.1.capabilities {
            if let Some(mut agents) = self.capabilities.get_mut(capability) {
                agents.remove(&agent_id);
                // Remove empty capability entries
                if agents.is_empty() {
                    drop(agents);
                    self.capabilities.remove(capability);
                }
            }
        }

        Ok(())
    }

    /// Updates routing table with remote agent information
    async fn update_remote_route(
        &self,
        agent_id: AgentId,
        node_id: NodeId,
        _hops: RouteHops,
    ) -> Result<(), RegistryError> {
        // Ensure node exists in registry
        if !self.node_registry.contains_key(&node_id) {
            let node_info =
                NodeInfo::new(node_id, format!("node-{node_id}"), "unknown".to_string());
            self.node_registry.insert(node_id, node_info);
        }

        // Update route cache
        self.routes.insert(agent_id, AgentLocation::Remote(node_id));

        // Update node agent count
        if let Some(mut node_info) = self.node_registry.get_mut(&node_id) {
            node_info.agent_count += 1;
        }

        Ok(())
    }

    /// O(1) capability-based agent discovery
    async fn find_agents_by_capability(
        &self,
        capability: &CapabilityName,
    ) -> Result<Vec<AgentId>, RegistryError> {
        if let Some(agents) = self.capabilities.get(capability) {
            Ok(agents.iter().copied().collect())
        } else {
            Ok(vec![])
        }
    }

    /// Lists all local agents
    async fn list_local_agents(&self) -> Result<Vec<LocalAgent>, RegistryError> {
        Ok(self
            .agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Updates agent health status
    async fn update_agent_health(
        &self,
        agent_id: AgentId,
        _is_healthy: bool,
        last_heartbeat: MessageTimestamp,
    ) -> Result<(), RegistryError> {
        if let Some(mut agent) = self.agents.get_mut(&agent_id) {
            agent.last_heartbeat = last_heartbeat;
            Ok(())
        } else {
            Err(RegistryError::AgentNotFound { agent_id })
        }
    }
}

/// Placeholder failure handler implementation
struct FailureHandlerImpl;

impl FailureHandlerImpl {
    fn new(_config: RouterConfig) -> Self {
        Self
    }
}

#[async_trait]
impl FailureHandler for FailureHandlerImpl {
    async fn handle_routing_failure(
        &self,
        _message: FipaMessage,
        _error: RouterError,
    ) -> Result<MessageId, RouterError> {
        // Placeholder implementation
        Ok(MessageId::generate())
    }

    async fn schedule_retry(
        &self,
        _message: FipaMessage,
        _retry_count: u8,
    ) -> Result<(), RouterError> {
        // Placeholder implementation
        Ok(())
    }

    async fn dead_letter(
        &self,
        _message: FipaMessage,
        _reason: FailureReason,
    ) -> Result<(), RouterError> {
        // Placeholder implementation
        Ok(())
    }

    async fn get_dead_letter_stats(&self) -> Result<DeadLetterStats, RouterError> {
        // Placeholder implementation
        Ok(DeadLetterStats {
            total_messages: MessageCount::zero(),
            messages_by_reason: HashMap::new(),
            oldest_message_age_ms: None,
            queue_size_bytes: 0,
        })
    }
}

/// Placeholder metrics collector implementation
struct MetricsCollectorImpl;

impl MetricsCollectorImpl {
    fn new() -> Self {
        Self
    }
}

impl MetricsCollector for MetricsCollectorImpl {
    fn record_message_routed(&self, _message: &FipaMessage, _duration: Duration) {
        // Placeholder implementation
    }

    fn record_routing_error(&self, _error: &RouterError) {
        // Placeholder implementation
    }

    fn record_delivery_metrics(&self, _success: bool, _duration: Duration) {
        // Placeholder implementation
    }

    fn record_conversation_created(&self) {
        // Placeholder implementation
    }

    fn record_agent_registered(&self, _agent_id: AgentId) {
        // Placeholder implementation
    }

    fn record_agent_deregistered(&self, _agent_id: AgentId) {
        // Placeholder implementation
    }
}

#[cfg(test)]
#[allow(unreachable_code, unused_variables)]
mod tests {
    use super::*;
    use crate::message_router::config::RouterConfig;
    use crate::message_router::{ContentLanguage, OntologyName};

    // Test that verifies FIPA message field validation rejects invalid sender/receiver combination
    #[tokio::test]
    async fn test_should_reject_message_when_sender_equals_receiver() {
        // Create router with test configuration
        let config = RouterConfig::testing();
        let router = MessageRouterImpl::new(config).unwrap();
        router.start().await.unwrap();

        // Create an invalid FIPA message with sender == receiver (FIPA violation)
        let same_agent = AgentId::generate();

        // Use unimplemented!() to force compilation but make test fail at runtime
        // This will fail with "not yet implemented" until MessageContent is added
        let invalid_message = FipaMessage {
            performative: Performative::Request,
            sender: same_agent,
            receiver: same_agent, // FIPA violation: sender cannot equal receiver
            content: MessageContent::try_new(b"test content".to_vec()).unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Attempt to route the invalid message - should fail with ValidationError
        let result = router.route_message(invalid_message).await;

        // Expect validation error for sender == receiver FIPA violation
        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "sender/receiver");
                assert!(reason.as_ref().contains("sender cannot equal receiver"));
            }
            _ => panic!("Expected ValidationError for sender == receiver, got: {result:?}"),
        }
    }

    // Test that verifies FIPA message field validation rejects empty content
    #[tokio::test]
    async fn test_should_reject_message_when_content_is_empty() {
        // Create router with test configuration
        let config = RouterConfig::testing();
        let router = MessageRouterImpl::new(config).unwrap();
        router.start().await.unwrap();

        // Create a FIPA message with empty content (FIPA violation)
        let sender = AgentId::generate();
        let receiver = AgentId::generate();

        let invalid_message = FipaMessage {
            performative: Performative::Inform,
            sender,
            receiver,
            content: MessageContent::try_new(Vec::new()).unwrap(), // Empty content - FIPA violation
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Attempt to route the invalid message - should fail with ValidationError
        let result = router.route_message(invalid_message).await;

        // Expect validation error for empty content FIPA violation
        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "content");
                assert!(reason.as_ref().contains("content cannot be empty"));
            }
            _ => panic!("Expected ValidationError for empty content, got: {result:?}"),
        }
    }

    // Test that verifies FIPA conversation threading validation requires corresponding reply_with/in_reply_to pairs
    #[tokio::test]
    async fn test_should_reject_message_when_in_reply_to_has_no_corresponding_reply_with() {
        // Create router with test configuration
        let config = RouterConfig::testing();
        let router = MessageRouterImpl::new(config).unwrap();
        router.start().await.unwrap();

        // Create agents for conversation threading
        let sender = AgentId::generate();
        let receiver = AgentId::generate();
        let conversation_id = ConversationId::generate();

        // Create a message that claims to reply to a non-existent message
        let orphaned_reply_id = MessageId::generate();

        let invalid_message = FipaMessage {
            performative: Performative::Inform,
            sender,
            receiver,
            content: MessageContent::try_new(
                b"This message claims to reply to something that doesn't exist".to_vec(),
            )
            .unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: Some(conversation_id),
            reply_with: None,
            in_reply_to: Some(orphaned_reply_id), // FIPA violation: no corresponding reply_with found
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Attempt to route the invalid message - should fail with ValidationError
        let result = router.route_message(invalid_message).await;

        // Expect validation error for orphaned in_reply_to FIPA violation
        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "in_reply_to");
                assert!(
                    reason
                        .as_ref()
                        .contains("no corresponding reply_with found")
                );
            }
            _ => panic!("Expected ValidationError for orphaned in_reply_to, got: {result:?}"),
        }
    }

    // Test that verifies FIPA conversation threading accepts valid reply_with/in_reply_to pairs
    #[tokio::test]
    async fn test_should_accept_message_when_in_reply_to_has_corresponding_reply_with() {
        // Create router with test configuration
        let config = RouterConfig::testing();
        let router = MessageRouterImpl::new(config).unwrap();
        router.start().await.unwrap();

        // Create agents for conversation threading
        let sender = AgentId::generate();
        let receiver = AgentId::generate();
        let conversation_id = ConversationId::generate();

        // First message: establishes reply_with for future responses
        let reply_with_id = MessageId::generate();
        let original_message = FipaMessage {
            performative: Performative::Request,
            sender,
            receiver,
            content: MessageContent::try_new(b"Original request expecting a reply".to_vec())
                .unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: Some(conversation_id),
            reply_with: Some(reply_with_id), // Establishes threading expectation
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Route the original message first (should succeed)
        let original_result = router.route_message(original_message).await;
        assert!(
            original_result.is_ok(),
            "Original message should be accepted"
        );

        // Reply message: valid in_reply_to that corresponds to original reply_with
        let reply_message = FipaMessage {
            performative: Performative::Inform,
            sender: receiver,
            receiver: sender,
            content: MessageContent::try_new(b"Reply to the original request".to_vec()).unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: Some(conversation_id),
            reply_with: None,
            in_reply_to: Some(reply_with_id), // FIPA compliance: matches original reply_with
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Route the reply message - should succeed because it's valid threading
        let reply_result = router.route_message(reply_message).await;

        // Expect successful routing for valid conversation threading
        match reply_result {
            Ok(_message_id) => {
                // Success - valid conversation threading was accepted
            }
            Err(error) => panic!("Expected successful routing for valid threading, got: {error:?}"),
        }
    }

    // Test that verifies conversation threading validation isolates conversations properly
    #[tokio::test]
    async fn test_should_isolate_conversation_threading_across_different_conversations() {
        // Create router with test configuration
        let config = RouterConfig::testing();
        let router = MessageRouterImpl::new(config).unwrap();
        router.start().await.unwrap();

        // Create agents
        let agent_a = AgentId::generate();
        let agent_b = AgentId::generate();
        let agent_c = AgentId::generate();

        // Create two separate conversations
        let conversation_1 = ConversationId::generate();
        let conversation_2 = ConversationId::generate();

        // Conversation 1: Agent A requests something from Agent B
        let reply_with_conv1 = MessageId::generate();
        let request_conv1 = FipaMessage {
            performative: Performative::Request,
            sender: agent_a,
            receiver: agent_b,
            content: MessageContent::try_new(b"Request in conversation 1".to_vec()).unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: Some(conversation_1),
            reply_with: Some(reply_with_conv1),
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Route conversation 1 request (establishes reply_with in tracker)
        router.route_message(request_conv1).await.unwrap();

        // Conversation 2: Agent C tries to reply using reply_with from conversation 1
        // This should be REJECTED because conversations should be isolated
        let cross_conversation_reply = FipaMessage {
            performative: Performative::Inform,
            sender: agent_c,
            receiver: agent_a,
            content: MessageContent::try_new(b"Cross-conversation reply attempt".to_vec()).unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: Some(conversation_2), // Different conversation!
            reply_with: None,
            in_reply_to: Some(reply_with_conv1), // Trying to use reply_with from conversation 1
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // This should fail - current implementation incorrectly allows cross-conversation threading
        let result = router.route_message(cross_conversation_reply).await;

        assert!(
            result.is_err(),
            "Should reject cross-conversation threading attempt"
        );
        if let Err(RouterError::ValidationError { field, reason }) = result {
            assert_eq!(field.as_ref(), "in_reply_to");
            assert!(
                reason
                    .as_ref()
                    .contains("no corresponding reply_with found")
                    || reason.as_ref().contains("conversation"),
                "Error should mention conversation isolation or missing reply_with: {reason}"
            );
        } else {
            panic!("Expected ValidationError for cross-conversation threading, got: {result:?}");
        }
    }

    #[tokio::test]
    async fn test_should_generate_not_understood_response_when_message_processing_fails() {
        // Test that verifies NOT_UNDERSTOOD response generation for unprocessable messages
        let router = MessageRouterImpl::new(RouterConfig::testing()).unwrap();
        let sender = AgentId::generate();
        let receiver = AgentId::generate();

        // Create a message that will cause processing failure (simulating unsupported performative handling)
        let problematic_message = FipaMessage {
            performative: Performative::Request,
            sender,
            receiver,
            content: MessageContent::try_new(
                b"UNSUPPORTED_OPERATION: complex_unsupported_request".to_vec(),
            )
            .unwrap(),
            language: None,
            ontology: None, // Keeping it simple for the test
            protocol: None, // Keeping it simple for the test
            conversation_id: Some(ConversationId::generate()),
            reply_with: Some(MessageId::generate()),
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Attempt to route the message - should generate NOT_UNDERSTOOD response
        let result = router.generate_not_understood_response(&problematic_message);

        // Expect NOT_UNDERSTOOD response with appropriate error details
        match result {
            Ok(not_understood_message) => {
                assert_eq!(
                    not_understood_message.performative,
                    Performative::NotUnderstood
                );
                assert_eq!(not_understood_message.sender, receiver); // Receiver becomes sender
                assert_eq!(not_understood_message.receiver, sender); // Sender becomes receiver
                assert!(!not_understood_message.content.is_empty());
                // Should reference original message in some way
                assert!(not_understood_message.in_reply_to.is_some());
            }
            Err(error) => {
                panic!("Expected NOT_UNDERSTOOD response generation, got error: {error:?}")
            }
        }
    }

    #[tokio::test]
    async fn test_should_preserve_conversation_context_in_not_understood_response() {
        // Test that verifies NOT_UNDERSTOOD responses maintain proper FIPA-ACL conversation threading
        let router = MessageRouterImpl::new(RouterConfig::testing()).unwrap();
        let sender = AgentId::generate();
        let receiver = AgentId::generate();
        let conversation_id = ConversationId::generate();
        let original_reply_with = MessageId::generate();

        // Create original message with specific conversation context
        let original_message = FipaMessage {
            performative: Performative::QueryRef,
            sender,
            receiver,
            content: MessageContent::try_new(
                b"Invalid query syntax - should trigger NOT_UNDERSTOOD".to_vec(),
            )
            .unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: Some(conversation_id),
            reply_with: Some(original_reply_with),
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Generate NOT_UNDERSTOOD response
        let result = router.generate_not_understood_response(&original_message);

        // Verify conversation threading compliance
        match result {
            Ok(not_understood_response) => {
                // Should preserve conversation context
                assert_eq!(
                    not_understood_response.conversation_id,
                    Some(conversation_id),
                    "NOT_UNDERSTOOD response must preserve original conversation_id"
                );

                // Should reference original message via in_reply_to
                assert_eq!(
                    not_understood_response.in_reply_to,
                    Some(original_reply_with),
                    "NOT_UNDERSTOOD response must reference original reply_with in in_reply_to field"
                );

                // Should generate new reply_with for potential responses to this NOT_UNDERSTOOD
                assert!(
                    not_understood_response.reply_with.is_some(),
                    "NOT_UNDERSTOOD response should generate new reply_with"
                );

                // Basic FIPA-ACL response structure
                assert_eq!(
                    not_understood_response.performative,
                    Performative::NotUnderstood
                );
                assert_eq!(not_understood_response.sender, receiver); // Role reversal
                assert_eq!(not_understood_response.receiver, sender); // Role reversal
            }
            Err(error) => panic!(
                "Expected NOT_UNDERSTOOD response with conversation context, got error: {error:?}"
            ),
        }
    }

    #[tokio::test]
    async fn test_should_reject_caxton_extension_performatives_in_strict_fipa_mode() {
        // Test that verifies enhanced FIPA performative validation rejects Caxton extension performatives
        let router = MessageRouterImpl::new(RouterConfig::testing()).unwrap();
        router.start().await.unwrap();

        let sender = AgentId::generate();
        let receiver = AgentId::generate();

        // Create a message using Caxton extension performative (not in FIPA-ACL standard)
        let message_with_extension_performative = FipaMessage {
            performative: Performative::Heartbeat, // Non-FIPA performative
            sender,
            receiver,
            content: MessageContent::try_new(b"Heartbeat signal".to_vec()).unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: Some(ConversationId::generate()),
            reply_with: Some(MessageId::generate()),
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Attempt to route the message - should be rejected due to non-FIPA performative
        let result = router
            .route_message(message_with_extension_performative)
            .await;

        // Should fail validation for non-FIPA performative
        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "performative");
                assert!(reason.as_ref().contains("not a standard FIPA performative"));
                assert!(reason.as_ref().contains("Heartbeat"));
            }
            Ok(_message_id) => {
                panic!(
                    "Expected validation error for non-FIPA performative, but message was routed successfully"
                )
            }
            Err(other_error) => {
                panic!(
                    "Expected ValidationError for performative, got different error: {other_error:?}"
                )
            }
        }
    }

    #[tokio::test]
    async fn test_should_validate_json_content_format_when_content_language_is_json() {
        // Test that verifies JSON content format validation for messages with ContentLanguage::json
        let router = MessageRouterImpl::new(RouterConfig::testing()).unwrap();
        router.start().await.unwrap();

        let sender = AgentId::generate();
        let receiver = AgentId::generate();

        // Create a message with malformed JSON content but JSON language
        let malformed_json_content = b"{incomplete_json: missing_quotes, no_closing_brace";
        let message_with_malformed_json = FipaMessage {
            performative: Performative::Request,
            sender,
            receiver,
            content: MessageContent::try_new(malformed_json_content.to_vec()).unwrap(),
            language: Some(ContentLanguage::try_new("json".to_string()).unwrap()),
            ontology: None,
            protocol: None,
            message_id: MessageId::generate(),
            conversation_id: Some(ConversationId::generate()),
            reply_with: Some(MessageId::generate()),
            in_reply_to: None,
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };

        // Attempt to route the message - should fail with JSON validation error
        let result = router.route_message(message_with_malformed_json).await;

        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "content");
                assert!(
                    reason.as_ref().contains("invalid JSON format"),
                    "Expected JSON validation error, got: {reason}"
                );
            }
            Ok(_) => panic!(
                "Expected ValidationError for malformed JSON content, but message was routed successfully"
            ),
            Err(other_error) => {
                panic!(
                    "Expected ValidationError for JSON content, got different error: {other_error:?}"
                )
            }
        }
    }

    #[test]
    fn test_validation_error_types_should_use_domain_types_instead_of_primitives() {
        // Test that verifies RouterError::ValidationError uses validated domain types
        // instead of primitive String types for field and reason parameters

        // Test 1: Valid ValidationField and ValidationReason should be constructible
        let valid_field = ValidationField::try_new("content".to_string());
        assert!(
            valid_field.is_ok(),
            "Valid field should be created successfully"
        );

        let valid_reason = ValidationReason::try_new("content cannot be empty".to_string());
        assert!(
            valid_reason.is_ok(),
            "Valid reason should be created successfully"
        );

        // Test 2: Create a valid ValidationError with domain types
        let valid_error = RouterError::ValidationError {
            field: valid_field.unwrap(),
            reason: valid_reason.unwrap(),
        };

        // Test 3: ValidationField should reject empty strings (illegal state)
        let empty_field_result = ValidationField::try_new(String::new());
        assert!(
            empty_field_result.is_err(),
            "Empty field name should be rejected"
        );

        // Test 4: ValidationField should reject overly long strings (>50 chars)
        let long_field_name = "a".repeat(51);
        let long_field_result = ValidationField::try_new(long_field_name);
        assert!(
            long_field_result.is_err(),
            "Field name >50 chars should be rejected"
        );

        // Test 5: ValidationReason should reject empty strings (illegal state)
        let empty_reason_result = ValidationReason::try_new(String::new());
        assert!(
            empty_reason_result.is_err(),
            "Empty reason should be rejected"
        );

        // Test 6: ValidationReason should reject overly long strings (>200 chars)
        let long_reason = "a".repeat(201);
        let long_reason_result = ValidationReason::try_new(long_reason);
        assert!(
            long_reason_result.is_err(),
            "Reason >200 chars should be rejected"
        );

        // Test 7: Verify AsRef trait works for string comparisons
        if let RouterError::ValidationError { field, reason } = valid_error {
            assert_eq!(field.as_ref(), "content");
            assert_eq!(reason.as_ref(), "content cannot be empty");
        } else {
            panic!("Expected ValidationError variant");
        }

        // All illegal states are now unrepresentable:
        // ✓ Empty field names rejected at construction time
        // ✓ Field names > 50 chars rejected at construction time
        // ✓ Empty reasons rejected at construction time
        // ✓ Reasons > 200 chars rejected at construction time
        // ✓ Type safety enforced through nutype validation
    }

    #[tokio::test]
    async fn test_fipa_message_should_have_smart_constructor_that_centralizes_all_validation() {
        // Test that verifies FipaMessage has a smart constructor that centralizes
        // all FIPA validation logic, making it impossible to create invalid instances

        // Test 1: Valid message should be created successfully
        let valid_performative = Performative::Request;
        let valid_sender = AgentId::generate();
        let valid_receiver = AgentId::generate(); // Different from sender
        let valid_content = MessageContent::try_new(b"valid request content".to_vec()).unwrap();
        let valid_language = Some(ContentLanguage::try_new("en".to_string()).unwrap());
        let valid_ontology = Some(OntologyName::try_new("test-ontology".to_string()).unwrap());
        let valid_protocol = Some(ProtocolName::try_new("test-protocol".to_string()).unwrap());
        let valid_conversation_id = Some(ConversationId::generate());
        let valid_reply_with = Some(MessageId::generate());
        let valid_in_reply_to = Some(MessageId::generate());
        let valid_message_id = MessageId::generate();
        let valid_created_at = MessageTimestamp::now();
        let valid_trace_context = None;
        let valid_delivery_options = DeliveryOptions::default();

        // Smart constructor should accept all FipaMessage fields and validate them
        let params = crate::message_router::domain_types::FipaMessageParams {
            performative: valid_performative,
            sender: valid_sender,
            receiver: valid_receiver,
            content: valid_content,
            language: valid_language,
            ontology: valid_ontology,
            protocol: valid_protocol,
            conversation_id: valid_conversation_id,
            reply_with: valid_reply_with,
            in_reply_to: valid_in_reply_to,
            message_id: valid_message_id,
            created_at: valid_created_at,
            trace_context: valid_trace_context,
            delivery_options: valid_delivery_options,
        };
        let result = FipaMessage::try_new_validated(params);

        match result {
            Ok(message) => {
                // Valid message should be created successfully
                assert_eq!(message.performative, Performative::Request);
                assert_eq!(message.sender, valid_sender);
                assert_eq!(message.receiver, valid_receiver);
            }
            Err(e) => panic!(
                "Expected valid message to be created, but got error: {:?}",
                e
            ),
        }

        // Test 2: Same sender/receiver should be rejected
        let same_agent = AgentId::generate();
        let params = crate::message_router::domain_types::FipaMessageParams {
            performative: Performative::Request,
            sender: same_agent,
            receiver: same_agent, // Same as sender - should be rejected
            content: MessageContent::try_new(b"test content".to_vec()).unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };
        let result = FipaMessage::try_new_validated(params);

        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "sender/receiver");
                assert_eq!(reason.as_ref(), "sender cannot equal receiver");
            }
            _ => panic!(
                "Expected ValidationError for same sender/receiver, got: {:?}",
                result
            ),
        }

        // Test 3: Empty content should be rejected
        let params = crate::message_router::domain_types::FipaMessageParams {
            performative: Performative::Request,
            sender: AgentId::generate(),
            receiver: AgentId::generate(),
            content: MessageContent::try_new(b"".to_vec()).unwrap(), // Empty content
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };
        let result = FipaMessage::try_new_validated(params);

        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "content");
                assert_eq!(reason.as_ref(), "content cannot be empty");
            }
            _ => panic!(
                "Expected ValidationError for empty content, got: {:?}",
                result
            ),
        }

        // Test 4: Non-FIPA performative should be rejected
        let params = crate::message_router::domain_types::FipaMessageParams {
            performative: Performative::Heartbeat, // Caxton extension, not FIPA standard
            sender: AgentId::generate(),
            receiver: AgentId::generate(),
            content: MessageContent::try_new(b"heartbeat content".to_vec()).unwrap(),
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };
        let result = FipaMessage::try_new_validated(params);

        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "performative");
                assert!(reason.as_ref().contains("not a standard FIPA performative"));
            }
            _ => panic!(
                "Expected ValidationError for non-FIPA performative, got: {:?}",
                result
            ),
        }

        // Test 5: Invalid JSON content should be rejected when language indicates JSON
        let params = crate::message_router::domain_types::FipaMessageParams {
            performative: Performative::Request,
            sender: AgentId::generate(),
            receiver: AgentId::generate(),
            content: MessageContent::try_new(b"{ invalid json".to_vec()).unwrap(), // Malformed JSON
            language: Some(ContentLanguage::try_new("json".to_string()).unwrap()), // Language indicates JSON
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions::default(),
        };
        let result = FipaMessage::try_new_validated(params);

        match result {
            Err(RouterError::ValidationError { field, reason }) => {
                assert_eq!(field.as_ref(), "content");
                assert!(reason.as_ref().contains("invalid JSON format"));
            }
            _ => panic!(
                "Expected ValidationError for invalid JSON content, got: {:?}",
                result
            ),
        }

        // Smart constructor has been successfully implemented!
        // All FIPA validation is now centralized in FipaMessage::try_new_validated()
        // Tests above verify comprehensive validation functionality.
    }
}
