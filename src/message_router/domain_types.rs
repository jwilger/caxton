//! Domain types for the message router module
//!
//! This module defines strongly-typed domain values specifically for message routing
//! functionality to prevent primitive obsession and improve type safety.

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::SystemTime;
use uuid::Uuid;

// Re-export base domain types that are used in message routing
pub use crate::domain_types::{AgentId, AgentName, MessageCount, MessageSize};

/// Unique identifier for a message
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    From,
    Into
))]
pub struct MessageId(Uuid);

impl MessageId {
    /// Creates a new random message ID
    #[must_use]
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// Unique identifier for a conversation
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    From,
    Into
))]
pub struct ConversationId(Uuid);

impl ConversationId {
    /// Creates a new random conversation ID
    #[must_use]
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// Unique identifier for a node in the cluster
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    From,
    Into
))]
pub struct NodeId(Uuid);

impl NodeId {
    /// Creates a new random node ID
    #[must_use]
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// Channel capacity for bounded queues
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1_000_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 1000
)]
pub struct ChannelCapacity(usize);

impl ChannelCapacity {
    /// Gets the value as usize for use with tokio channels
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Maximum retry attempts for failed operations
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 10),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 3
)]
pub struct MaxRetries(u8);

impl MaxRetries {
    /// Gets the value as u8
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        self.into_inner()
    }
}

/// Retry delay in milliseconds
#[nutype(
    validate(greater_or_equal = 100, less_or_equal = 300_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 1000
)]
pub struct RetryDelayMs(u64);

impl RetryDelayMs {
    /// Converts to Duration
    #[must_use]
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }
}

/// Circuit breaker failure threshold
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 10
)]
pub struct CircuitBreakerThreshold(u32);

/// Dead letter queue maximum size
#[nutype(
    validate(greater_or_equal = 10_000, less_or_equal = 10_000_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 100_000
)]
pub struct DeadLetterQueueSize(usize);

impl DeadLetterQueueSize {
    /// Gets the value as usize
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Conversation timeout in milliseconds
#[nutype(
    validate(greater_or_equal = 300_000, less_or_equal = 86_400_000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default, TryFrom, Into),
    default = 1_800_000  // 30 minutes
)]
pub struct ConversationTimeoutMs(u64);

impl ConversationTimeoutMs {
    /// Converts to Duration
    #[must_use]
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }
}

/// Maximum participants in a conversation
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 10
)]
pub struct MaxConversationParticipants(u8);

/// Timestamp for when conversation was created
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize
))]
pub struct ConversationCreatedAt(SystemTime);

impl ConversationCreatedAt {
    /// Creates timestamp for current time
    #[must_use]
    pub fn now() -> Self {
        Self::new(SystemTime::now())
    }

    /// Gets the inner `SystemTime`
    #[must_use]
    pub fn as_system_time(&self) -> SystemTime {
        self.into_inner()
    }
}

/// OpenTelemetry trace ID
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct TraceId(String);

/// OpenTelemetry span ID
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct SpanId(String);

/// Trace sampling ratio (0.0 to 1.0)
#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        PartialOrd,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0.1
)]
pub struct TraceSamplingRatio(f64);

impl TraceSamplingRatio {
    /// Gets the value as f64
    #[must_use]
    pub fn as_f64(&self) -> f64 {
        self.into_inner()
    }
}

/// Agent capability name
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct CapabilityName(String);

/// Agent capability description
#[nutype(
    validate(len_char_min = 1, len_char_max = 1000),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct CapabilityDescription(String);

/// Health check interval in milliseconds
#[nutype(
    validate(greater_or_equal = 1000, less_or_equal = 300_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 30_000
)]
pub struct HealthCheckIntervalMs(u64);

impl HealthCheckIntervalMs {
    /// Converts to Duration
    #[must_use]
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }
}

/// Message timestamp
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize
))]
pub struct MessageTimestamp(SystemTime);

impl MessageTimestamp {
    /// Creates timestamp for current time
    #[must_use]
    pub fn now() -> Self {
        Self::new(SystemTime::now())
    }

    /// Gets the inner `SystemTime`
    #[must_use]
    pub fn as_system_time(&self) -> SystemTime {
        self.into_inner()
    }
}

/// Content language identifier
#[nutype(
    validate(len_char_min = 1, len_char_max = 50),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct ContentLanguage(String);

/// Ontology name
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct OntologyName(String);

/// Protocol name for conversation protocols
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct ProtocolName(String);

/// Message timeout in milliseconds
#[nutype(
    validate(greater_or_equal = 1000, less_or_equal = 300_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 30_000
)]
pub struct MessageTimeoutMs(u64);

impl MessageTimeoutMs {
    /// Converts to Duration
    #[must_use]
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }
}

/// Message content as validated bytes
#[nutype(
    validate(predicate = |content| content.len() <= 10_485_760), // 10MB max
    derive(Debug, Clone, Serialize, Deserialize, AsRef, Deref)
)]
pub struct MessageContent(Vec<u8>);

impl MessageContent {
    /// Gets the length of the content
    #[must_use]
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Checks if content is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    /// Gets the content as bytes
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

/// Agent queue size
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 1000
)]
pub struct AgentQueueSize(usize);

impl AgentQueueSize {
    /// Gets the value as usize
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Worker thread count for parallel processing
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 32),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 4
)]
pub struct WorkerThreadCount(usize);

impl WorkerThreadCount {
    /// Gets the value as usize
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Message batch size for high-throughput processing
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 10_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 100
)]
pub struct MessageBatchSize(usize);

impl MessageBatchSize {
    /// Gets the value as usize
    #[must_use]
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Retry backoff multiplication factor
#[nutype(
    validate(greater_or_equal = 1.1, less_or_equal = 5.0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        PartialOrd,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 2.0
)]
pub struct RetryBackoffFactor(f64);

impl RetryBackoffFactor {
    /// Gets the value as f64
    #[must_use]
    pub fn as_f64(&self) -> f64 {
        self.into_inner()
    }
}

/// Circuit breaker timeout in milliseconds
#[nutype(
    validate(greater_or_equal = 5_000, less_or_equal = 300_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 60_000
)]
pub struct CircuitBreakerTimeoutMs(u64);

impl CircuitBreakerTimeoutMs {
    /// Converts to Duration
    #[must_use]
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }
}

/// FIPA message performatives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Performative {
    // Core FIPA performatives
    Request,
    Inform,
    QueryIf,
    QueryRef,
    Propose,
    AcceptProposal,
    RejectProposal,
    Agree,
    Refuse,
    Failure,
    NotUnderstood,
    // Caxton extensions
    Heartbeat,
    Capability,
}

/// Message delivery priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 1,
    Normal = 5,
    High = 8,
    Critical = 10,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Reasons for message delivery failure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureReason {
    AgentNotFound,
    AgentNotResponding,
    NetworkError,
    ResourceExhausted,
    MessageTooLarge,
    InvalidMessage,
    CircuitBreakerOpen,
    QueueFull,
    Timeout,
}

/// Agent state in its lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Unloaded,
    Loaded,
    Running,
    Draining,
    Stopped,
}

/// Agent location information
#[derive(Debug, Clone)]
pub enum AgentLocation {
    Local(LocalAgent),
    Remote(NodeId),
    Unknown,
}

/// Route information for remote agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub node_id: NodeId,
    pub hops: RouteHops,
    pub updated_at: MessageTimestamp,
    pub expires_at: MessageTimestamp,
}

/// Number of hops to reach an agent
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 255),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct RouteHops(u8);

/// Local agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAgent {
    pub id: AgentId,
    pub name: AgentName,
    pub state: AgentState,
    pub capabilities: Vec<CapabilityName>,
    pub last_heartbeat: MessageTimestamp,
    pub queue_size: AgentQueueSize,
}

impl LocalAgent {
    /// Creates a new local agent
    #[must_use]
    pub fn new(
        id: AgentId,
        name: AgentName,
        state: AgentState,
        capabilities: Vec<CapabilityName>,
        last_heartbeat: MessageTimestamp,
        queue_size: AgentQueueSize,
    ) -> Self {
        Self {
            id,
            name,
            state,
            capabilities,
            last_heartbeat,
            queue_size,
        }
    }

    /// Checks if agent is available for message delivery
    #[must_use]
    pub fn is_available(&self) -> bool {
        matches!(self.state, AgentState::Running)
    }
}

/// Conversation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: ConversationId,
    pub participants: HashSet<AgentId>,
    pub protocol: Option<ProtocolName>,
    pub created_at: ConversationCreatedAt,
    pub last_activity: MessageTimestamp,
    pub message_count: MessageCount,
}

impl Conversation {
    /// Creates a new conversation
    #[must_use]
    pub fn new(
        id: ConversationId,
        participants: HashSet<AgentId>,
        protocol: Option<ProtocolName>,
        created_at: ConversationCreatedAt,
    ) -> Self {
        Self {
            id,
            participants,
            protocol,
            created_at,
            last_activity: MessageTimestamp::now(),
            message_count: MessageCount::zero(),
        }
    }

    /// Updates conversation with new message activity
    pub fn add_message(&mut self, _message: &FipaMessage) {
        self.message_count = self.message_count.increment();
        self.last_activity = MessageTimestamp::now();
    }

    /// Updates the last activity timestamp
    pub fn update_last_activity(&mut self, timestamp: MessageTimestamp) {
        self.last_activity = timestamp;
    }
}

/// FIPA-ACL message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FipaMessage {
    // Standard FIPA fields
    pub performative: Performative,
    pub sender: AgentId,
    pub receiver: AgentId,
    pub content: MessageContent,
    pub language: Option<ContentLanguage>,
    pub ontology: Option<OntologyName>,
    pub protocol: Option<ProtocolName>,

    // Conversation management
    pub conversation_id: Option<ConversationId>,
    pub reply_with: Option<MessageId>,
    pub in_reply_to: Option<MessageId>,

    // Caxton extensions
    pub message_id: MessageId,
    pub created_at: MessageTimestamp,
    pub trace_context: Option<TraceContext>,
    pub delivery_options: DeliveryOptions,
}

impl FipaMessage {
    /// Creates a validated FIPA message with centralized validation.
    ///
    /// This smart constructor centralizes all FIPA-ACL validation logic,
    /// ensuring that only valid messages can be created. It performs
    /// comprehensive validation including:
    /// - Sender and receiver must be different agents (FIPA requirement)
    /// - Message content must not be empty (FIPA requirement)
    /// - Performative must be a standard FIPA performative
    /// - JSON content format validation when language indicates JSON
    ///
    /// # Arguments
    ///
    /// All required fields for a FIPA message:
    /// - `performative`: The FIPA performative indicating message intent
    /// - `sender`: The agent sending the message
    /// - `receiver`: The agent receiving the message
    /// - `content`: The message content
    /// - `language`: Optional content language specification
    /// - `ontology`: Optional ontology name for message semantics
    /// - `protocol`: Optional protocol name for interaction pattern
    /// - `conversation_id`: Optional conversation identifier for message threading
    /// - `reply_with`: Optional identifier for expecting replies
    /// - `in_reply_to`: Optional identifier referencing previous message
    /// - `message_id`: Unique message identifier
    /// - `created_at`: Timestamp when message was created
    /// - `trace_context`: Optional OpenTelemetry trace context
    /// - `delivery_options`: Message delivery configuration
    ///
    /// # Returns
    ///
    /// `Result<FipaMessage, RouterError>` - Returns the validated message or validation error
    ///
    /// # Errors
    ///
    /// Returns `RouterError::ValidationError` if:
    /// - Sender equals receiver (field: "sender/receiver", reason: "sender cannot equal receiver")
    /// - Content is empty (field: "content", reason: "content cannot be empty")
    /// - Performative is not a standard FIPA performative (field: "performative", reason: "not a standard FIPA performative")
    /// - Content is invalid JSON when language indicates JSON (field: "content", reason: "invalid JSON format")
    ///
    /// # Example
    ///
    /// ```rust
    /// use caxton::message_router::{FipaMessage, FipaMessageParams, Performative, MessageContent, MessageId, MessageTimestamp, DeliveryOptions};
    /// use caxton::domain_types::AgentId;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    ///
    /// // Create required parameters
    /// let sender_id = AgentId::generate();
    /// let receiver_id = AgentId::generate();
    /// let content = MessageContent::try_new("Hello, world!".as_bytes().to_vec())?;
    /// let message_id = MessageId::generate();
    /// let created_at = MessageTimestamp::now();
    /// let delivery_options = DeliveryOptions::default();
    ///
    /// let params = FipaMessageParams {
    ///     performative: Performative::Request,
    ///     sender: sender_id,
    ///     receiver: receiver_id,
    ///     content,
    ///     language: None,
    ///     ontology: None,
    ///     protocol: None,
    ///     conversation_id: None,
    ///     reply_with: None,
    ///     in_reply_to: None,
    ///     message_id,
    ///     created_at,
    ///     trace_context: None,
    ///     delivery_options,
    /// };
    /// let message = FipaMessage::try_new_validated(params)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_new_validated(
        params: FipaMessageParams,
    ) -> Result<Self, crate::message_router::traits::RouterError> {
        // Create message instance first
        let message = Self {
            performative: params.performative,
            sender: params.sender,
            receiver: params.receiver,
            content: params.content,
            language: params.language,
            ontology: params.ontology,
            protocol: params.protocol,
            conversation_id: params.conversation_id,
            reply_with: params.reply_with,
            in_reply_to: params.in_reply_to,
            message_id: params.message_id,
            created_at: params.created_at,
            trace_context: params.trace_context,
            delivery_options: params.delivery_options,
        };

        // Apply all FIPA validation rules
        validate_sender_receiver_different(&message)?;
        validate_content_not_empty(&message)?;
        validate_performative_fipa_compliance(&message)?;
        validate_json_content_format(&message)?;

        Ok(message)
    }
}

// ============================================================================
// FIPA Message Validation Functions - Centralized for Smart Constructor
// ============================================================================

/// Field name constants for validation errors
const FIELD_CONTENT: &str = "content";
const FIELD_PERFORMATIVE: &str = "performative";
const FIELD_SENDER_RECEIVER: &str = "sender/receiver";

/// Validation reason constants
const REASON_CONTENT_EMPTY: &str = "content cannot be empty";
const REASON_SENDER_EQUALS_RECEIVER: &str = "sender cannot equal receiver";

/// Standard FIPA-ACL performatives as defined in FIPA specification
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

/// JSON content language identifier
const JSON_CONTENT_LANGUAGE: &str = "json";

/// Creates a validation error with proper domain types
fn create_validation_error(
    field: &str,
    reason: &str,
) -> crate::message_router::traits::RouterError {
    crate::message_router::traits::RouterError::ValidationError {
        field: ValidationField::try_new(field.to_string())
            .expect("Field name should meet validation requirements"),
        reason: ValidationReason::try_new(reason.to_string())
            .expect("Reason should meet validation requirements"),
    }
}

/// Validates sender and receiver are different agents (FIPA requirement)
fn validate_sender_receiver_different(
    message: &FipaMessage,
) -> Result<(), crate::message_router::traits::RouterError> {
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
fn validate_content_not_empty(
    message: &FipaMessage,
) -> Result<(), crate::message_router::traits::RouterError> {
    if message.content.is_empty() {
        Err(create_validation_error(FIELD_CONTENT, REASON_CONTENT_EMPTY))
    } else {
        Ok(())
    }
}

/// Validates performative is a standard FIPA performative
fn validate_performative_fipa_compliance(
    message: &FipaMessage,
) -> Result<(), crate::message_router::traits::RouterError> {
    if STANDARD_FIPA_PERFORMATIVES.contains(&message.performative) {
        Ok(())
    } else {
        Err(create_validation_error(
            FIELD_PERFORMATIVE,
            "not a standard FIPA performative",
        ))
    }
}

/// Validates JSON content format when `ContentLanguage` indicates JSON
fn validate_json_content_format(
    message: &FipaMessage,
) -> Result<(), crate::message_router::traits::RouterError> {
    // Only validate JSON when ContentLanguage contains JSON identifier
    if let Some(language) = &message.language {
        let language_str = language.to_string();
        if language_str.to_lowercase().contains(JSON_CONTENT_LANGUAGE) {
            // Validate JSON syntax using serde_json
            if serde_json::from_slice::<serde_json::Value>(message.content.as_ref()).is_err() {
                return Err(create_validation_error(
                    FIELD_CONTENT,
                    "invalid JSON format",
                ));
            }
        }
    }
    Ok(())
}

/// Parameters for creating and validating a FIPA message
///
/// This struct consolidates all the parameters needed for `FipaMessage::try_new_validated()`
/// to avoid `clippy::too_many_arguments` warnings while maintaining type safety.
#[derive(Debug, Clone)]
pub struct FipaMessageParams {
    pub performative: Performative,
    pub sender: AgentId,
    pub receiver: AgentId,
    pub content: MessageContent,
    pub language: Option<ContentLanguage>,
    pub ontology: Option<OntologyName>,
    pub protocol: Option<ProtocolName>,
    pub conversation_id: Option<ConversationId>,
    pub reply_with: Option<MessageId>,
    pub in_reply_to: Option<MessageId>,
    pub message_id: MessageId,
    pub created_at: MessageTimestamp,
    pub trace_context: Option<TraceContext>,
    pub delivery_options: DeliveryOptions,
}

/// OpenTelemetry trace context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub trace_flags: u8,
    pub trace_state: Option<String>,
}

/// Message delivery options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOptions {
    pub priority: MessagePriority,
    pub timeout: Option<MessageTimeoutMs>,
    pub require_receipt: bool,
    pub max_retries: MaxRetries,
}

impl Default for DeliveryOptions {
    fn default() -> Self {
        Self {
            priority: MessagePriority::Normal,
            timeout: None,
            require_receipt: false,
            max_retries: MaxRetries::default(),
        }
    }
}

/// Timestamp wrapper for routing operations
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize
))]
pub struct Timestamp(SystemTime);

impl Timestamp {
    /// Creates timestamp for current time
    #[must_use]
    pub fn now() -> Self {
        Self::new(SystemTime::now())
    }

    /// Gets the inner `SystemTime`
    #[must_use]
    pub fn as_system_time(&self) -> SystemTime {
        self.into_inner()
    }
}

/// Validation field names with length constraints (1-50 characters)
#[nutype(
    sanitize(trim),
    validate(len_char_min = 1, len_char_max = 50),
    derive(Clone, Debug, Eq, PartialEq, Display, AsRef)
)]
pub struct ValidationField(String);

/// Validation reason messages with length constraints (1-200 characters)
#[nutype(
    sanitize(trim),
    validate(len_char_min = 1, len_char_max = 200),
    derive(Clone, Debug, Eq, PartialEq, Display, AsRef)
)]
pub struct ValidationReason(String);

impl RouteInfo {
    /// Creates new route information
    #[must_use]
    pub fn new(node_id: NodeId, hops: RouteHops, updated_at: MessageTimestamp) -> Self {
        let ttl_duration = std::time::Duration::from_secs(300); // 5 minutes default TTL
        let expires_at = MessageTimestamp::new(updated_at.as_system_time() + ttl_duration);

        Self {
            node_id,
            hops,
            updated_at,
            expires_at,
        }
    }

    /// Checks if route is still fresh within TTL
    #[must_use]
    pub fn is_fresh(&self, ttl: std::time::Duration) -> bool {
        match self.updated_at.as_system_time().elapsed() {
            Ok(elapsed) => elapsed < ttl,
            Err(_) => false, // Clock moved backwards, consider stale
        }
    }
}
