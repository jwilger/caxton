//! Trait definitions for message router components
//!
//! This module defines the core interfaces that all message router components
//! must implement, enabling loose coupling and testability.

use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

use super::domain_types::{
    AgentId, AgentLocation, AgentState, CapabilityName, Conversation, ConversationId,
    FailureReason, FipaMessage, LocalAgent, MaxRetries, MessageCount, MessageId, MessageTimestamp,
    NodeId, ProtocolName, RouteHops, RouteInfo, ValidationField, ValidationReason,
};

/// Comprehensive error types for message routing operations
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },

    #[error("Message too large: {size} bytes (max: {max_size} bytes)")]
    MessageTooLarge { size: usize, max_size: usize },

    #[error("Queue full: {queue_type}")]
    QueueFull { queue_type: String },

    #[error("Network error: {source}")]
    NetworkError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[source]
        source: serde_json::Error,
    },

    #[error("Timeout: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Storage error: {source}")]
    StorageError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Circuit breaker open for node: {node_id}")]
    CircuitBreakerOpen { node_id: NodeId },

    #[error("Validation error: {field} - {reason}")]
    ValidationError {
        field: ValidationField,
        reason: ValidationReason,
    },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

/// Delivery-specific errors
#[derive(Debug, Error)]
pub enum DeliveryError {
    #[error("Local delivery failed: {source}")]
    LocalDeliveryFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Remote delivery failed to node {node_id}: {source}")]
    RemoteDeliveryFailed {
        node_id: NodeId,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Circuit breaker open for node: {node_id}")]
    CircuitBreakerOpen { node_id: NodeId },

    #[error("Connection pool exhausted for node: {node_id}")]
    ConnectionPoolExhausted { node_id: NodeId },

    #[error("Serialization failed: {source}")]
    SerializationFailed {
        #[source]
        source: serde_json::Error,
    },

    #[error("Retry limit exceeded: {max_retries}")]
    RetryLimitExceeded { max_retries: MaxRetries },
}

/// Conversation management errors
#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("Conversation not found: {conversation_id}")]
    ConversationNotFound { conversation_id: ConversationId },

    #[error("Too many participants: {count} (max: {max})")]
    TooManyParticipants { count: usize, max: usize },

    #[error("Storage error: {source}")]
    StorageError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Conversation timeout: {conversation_id}")]
    ConversationTimeout { conversation_id: ConversationId },
}

/// Agent registry errors
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Agent already registered: {agent_id}")]
    AgentAlreadyRegistered { agent_id: AgentId },

    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },

    #[error("Invalid agent state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: AgentState, to: AgentState },

    #[error("Storage error: {source}")]
    StorageError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Health check failed for agent: {agent_id}")]
    HealthCheckFailed { agent_id: AgentId },
}

/// Router health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Router performance statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    // Throughput metrics
    pub messages_per_second: f64,
    pub peak_messages_per_second: f64,
    pub total_messages_processed: MessageCount,

    // Latency metrics (in microseconds)
    pub routing_latency_p50: u64,
    pub routing_latency_p90: u64,
    pub routing_latency_p99: u64,
    pub routing_latency_p999: u64,

    // Error metrics
    pub total_errors: MessageCount,
    pub error_rate: f64,
    pub errors_by_type: HashMap<String, MessageCount>,

    // Queue metrics
    pub inbound_queue_depth: usize,
    pub outbound_queue_depth: usize,
    pub agent_queue_depths: HashMap<AgentId, usize>,

    // Conversation metrics
    pub active_conversations: usize,
    pub total_conversations: MessageCount,
    pub average_conversation_length: f64,

    // Resource metrics
    pub memory_usage_bytes: usize,
    pub cpu_usage_percent: f64,
    pub database_size_bytes: usize,
}

/// Main message router interface
#[async_trait]
pub trait MessageRouter: Send + Sync {
    /// Routes a message to its destination agent
    ///
    /// Returns the message ID for correlation and tracking.
    /// This operation should complete quickly (< 1ms for local agents)
    /// and handle all error conditions gracefully.
    async fn route_message(&self, message: FipaMessage) -> Result<MessageId, RouterError>;

    /// Registers a new local agent with the router
    ///
    /// The agent must be registered before it can receive messages.
    /// Capabilities are used for agent discovery and routing optimization.
    async fn register_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>,
    ) -> Result<(), RouterError>;

    /// Deregisters an agent from the router
    ///
    /// Any queued messages for this agent will be moved to the dead letter queue
    /// after appropriate retry attempts.
    async fn deregister_agent(&self, agent_id: AgentId) -> Result<(), RouterError>;

    /// Updates an agent's state in its lifecycle
    ///
    /// State transitions affect message delivery behavior:
    /// - Running: Normal message delivery
    /// - Draining: No new conversations, finish existing ones
    /// - Stopped: Move messages to dead letter queue
    async fn update_agent_state(
        &self,
        agent_id: AgentId,
        state: AgentState,
    ) -> Result<(), RouterError>;

    /// Retrieves current router performance statistics
    ///
    /// Used for monitoring, alerting, and capacity planning.
    async fn get_stats(&self) -> Result<RouterStats, RouterError>;

    /// Checks the health status of the router
    ///
    /// Returns detailed health information for operational monitoring.
    async fn health_check(&self) -> Result<HealthStatus, RouterError>;

    /// Initiates graceful shutdown of the router
    ///
    /// Allows in-flight messages to complete while refusing new ones.
    async fn shutdown(&self) -> Result<(), RouterError>;
}

/// Message delivery engine interface
#[async_trait]
pub trait DeliveryEngine: Send + Sync {
    /// Delivers a message to a local agent
    ///
    /// Should complete in < 1ms for available agents.
    /// Queues messages for unavailable agents.
    async fn deliver_local(
        &self,
        message: FipaMessage,
        agent: LocalAgent,
    ) -> Result<MessageId, DeliveryError>;

    /// Delivers a message to a remote agent via network
    ///
    /// Handles connection pooling, retries, and circuit breaking.
    /// Should complete in < 5ms for healthy remote nodes.
    async fn deliver_remote(
        &self,
        message: FipaMessage,
        node_id: NodeId,
    ) -> Result<MessageId, DeliveryError>;

    /// Processes a batch of messages for high-throughput scenarios
    ///
    /// Optimizes throughput by batching operations and using parallelism.
    /// Critical for achieving 100K+ messages/second.
    async fn deliver_batch(
        &self,
        messages: Vec<FipaMessage>,
    ) -> Vec<Result<MessageId, DeliveryError>>;

    /// Checks delivery engine health and performance
    async fn health_check(&self) -> Result<HealthStatus, DeliveryError>;
}

/// Conversation management interface
#[async_trait]
pub trait ConversationManager: Send + Sync {
    /// Creates or retrieves an existing conversation
    ///
    /// Conversations enable multi-turn dialogues between agents
    /// with proper context and state management.
    async fn get_or_create_conversation(
        &self,
        conversation_id: ConversationId,
        participants: std::collections::HashSet<AgentId>,
        protocol: Option<ProtocolName>,
    ) -> Result<Conversation, ConversationError>;

    /// Updates conversation state with a new message
    ///
    /// Maintains conversation activity timestamps and message counts.
    async fn update_conversation(
        &self,
        conversation_id: ConversationId,
        message: &FipaMessage,
    ) -> Result<(), ConversationError>;

    /// Retrieves all active conversations for an agent
    ///
    /// Used for context-aware message routing and agent state management.
    async fn get_agent_conversations(
        &self,
        agent_id: AgentId,
    ) -> Result<Vec<Conversation>, ConversationError>;

    /// Cleans up expired conversations
    ///
    /// Returns the number of conversations that were cleaned up.
    /// Should be called periodically to prevent memory leaks.
    async fn cleanup_expired_conversations(&self) -> Result<usize, ConversationError>;

    /// Retrieves conversation statistics
    async fn get_conversation_stats(&self) -> Result<ConversationStats, ConversationError>;
}

/// Agent registry interface with O(1) lookup performance
#[async_trait]
pub trait AgentRegistry: Send + Sync {
    /// Performs O(1) agent lookup returning location information
    ///
    /// This is the most performance-critical operation in the system.
    /// Must complete in < 100μs for local agents.
    async fn lookup(&self, agent_id: &AgentId) -> Result<AgentLocation, RegistryError>;

    /// Registers a new local agent with capabilities
    ///
    /// Updates both the agent registry and capability indexes
    /// for optimal lookup performance.
    async fn register_local_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>,
    ) -> Result<(), RegistryError>;

    /// Deregisters a local agent
    ///
    /// Cleans up all associated indexes and routing information.
    async fn deregister_local_agent(&self, agent_id: AgentId) -> Result<(), RegistryError>;

    /// Updates routing table with remote agent information
    ///
    /// Used by the gossip protocol to maintain distributed agent awareness.
    async fn update_remote_route(
        &self,
        agent_id: AgentId,
        node_id: NodeId,
        hops: RouteHops,
    ) -> Result<(), RegistryError>;

    /// Finds agents by capability for discovery
    ///
    /// Returns agents that match the specified capability.
    /// Used for capability-based routing and service discovery.
    async fn find_agents_by_capability(
        &self,
        capability: &CapabilityName,
    ) -> Result<Vec<AgentId>, RegistryError>;

    /// Lists all local agents with their current states
    async fn list_local_agents(&self) -> Result<Vec<LocalAgent>, RegistryError>;

    /// Updates agent health status
    ///
    /// Called by health monitoring subsystem to track agent liveness.
    async fn update_agent_health(
        &self,
        agent_id: AgentId,
        is_healthy: bool,
        last_heartbeat: MessageTimestamp,
    ) -> Result<(), RegistryError>;
}

/// Storage interface for persistence operations
#[async_trait]
pub trait MessageStorage: Send + Sync {
    /// Persists a message for delivery guarantees
    async fn store_message(&self, message: &FipaMessage) -> Result<(), RouterError>;

    /// Retrieves a stored message by ID
    async fn get_message(&self, message_id: MessageId) -> Result<Option<FipaMessage>, RouterError>;

    /// Removes a message after successful delivery
    async fn remove_message(&self, message_id: MessageId) -> Result<(), RouterError>;

    /// Lists messages for a specific agent (for queue management)
    async fn list_agent_messages(
        &self,
        agent_id: AgentId,
        limit: Option<usize>,
    ) -> Result<Vec<FipaMessage>, RouterError>;
}

/// Conversation storage interface
#[async_trait]
pub trait ConversationStorage: Send + Sync {
    /// Saves conversation state
    async fn save_conversation(&self, conversation: &Conversation)
    -> Result<(), ConversationError>;

    /// Loads conversation by ID
    async fn load_conversation(
        &self,
        conversation_id: ConversationId,
    ) -> Result<Option<Conversation>, ConversationError>;

    /// Archives expired conversation
    async fn archive_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), ConversationError>;

    /// Lists active conversations for an agent
    async fn list_agent_conversations(
        &self,
        agent_id: AgentId,
    ) -> Result<Vec<ConversationId>, ConversationError>;
}

/// Agent storage interface
#[async_trait]
pub trait AgentStorage: Send + Sync {
    /// Saves agent registration information
    async fn save_agent_registration(
        &self,
        agent: &LocalAgent,
        capabilities: &[CapabilityName],
    ) -> Result<(), RegistryError>;

    /// Loads agent information
    async fn load_agent(&self, agent_id: AgentId) -> Result<Option<LocalAgent>, RegistryError>;

    /// Removes agent registration
    async fn remove_agent(&self, agent_id: AgentId) -> Result<(), RegistryError>;

    /// Saves route information
    async fn save_route(&self, agent_id: AgentId, route: &RouteInfo) -> Result<(), RegistryError>;

    /// Loads route information
    async fn load_route(&self, agent_id: AgentId) -> Result<Option<RouteInfo>, RegistryError>;

    /// Lists all stored agents
    async fn list_agents(&self) -> Result<Vec<LocalAgent>, RegistryError>;
}

/// Failure handling interface
#[async_trait]
pub trait FailureHandler: Send + Sync {
    /// Handles routing failures with appropriate strategies
    ///
    /// Decides whether to retry, dead-letter, or escalate based on
    /// failure type and message characteristics.
    async fn handle_routing_failure(
        &self,
        message: FipaMessage,
        error: RouterError,
    ) -> Result<MessageId, RouterError>;

    /// Schedules a message for retry with exponential backoff
    async fn schedule_retry(
        &self,
        message: FipaMessage,
        retry_count: u8,
    ) -> Result<(), RouterError>;

    /// Moves a message to the dead letter queue
    async fn dead_letter(
        &self,
        message: FipaMessage,
        reason: FailureReason,
    ) -> Result<(), RouterError>;

    /// Retrieves dead letter queue statistics
    async fn get_dead_letter_stats(&self) -> Result<DeadLetterStats, RouterError>;
}

/// Circuit breaker interface for fault tolerance
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    /// Records a successful operation
    async fn record_success(&self);

    /// Records a failed operation
    async fn record_failure(&self);

    /// Checks if the circuit breaker is open
    async fn is_open(&self) -> bool;

    /// Gets current circuit breaker state
    async fn get_state(&self) -> CircuitBreakerState;
}

/// Metrics collection interface
pub trait MetricsCollector: Send + Sync {
    /// Records a message routing event
    fn record_message_routed(&self, message: &FipaMessage, duration: std::time::Duration);

    /// Records a routing error
    fn record_routing_error(&self, error: &RouterError);

    /// Records delivery metrics
    fn record_delivery_metrics(&self, success: bool, duration: std::time::Duration);

    /// Records conversation metrics
    fn record_conversation_created(&self);

    /// Records agent registration event
    fn record_agent_registered(&self, agent_id: AgentId);

    /// Records agent deregistration event
    fn record_agent_deregistered(&self, agent_id: AgentId);
}

/// Supporting data structures
/// Conversation statistics
#[derive(Debug, Clone)]
pub struct ConversationStats {
    pub total_active: usize,
    pub total_created: MessageCount,
    pub average_duration_ms: u64,
    pub average_message_count: f64,
    pub participants_distribution: HashMap<usize, usize>, // participant_count -> conversation_count
}

/// Dead letter queue statistics
#[derive(Debug, Clone)]
pub struct DeadLetterStats {
    pub total_messages: MessageCount,
    pub messages_by_reason: HashMap<FailureReason, MessageCount>,
    pub oldest_message_age_ms: Option<u64>,
    pub queue_size_bytes: usize,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    HalfOpen,
    Open,
}

/// Retry reasons for failure classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetryReason {
    NetworkError,
    QueueFull,
    AgentUnavailable,
    ResourceExhausted,
    Timeout,
}

/// Health check results with detailed information
#[derive(Debug, Clone)]
pub struct DetailedHealthStatus {
    pub overall: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub uptime_ms: u64,
    pub last_error: Option<String>,
}

/// Individual component health
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub last_check: MessageTimestamp,
    pub error_count: u32,
    pub metrics: HashMap<String, f64>,
}
