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
    NodeId, RouteHops, RouteInfo,
};

/// Comprehensive error types for message routing operations
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Agent not found: {agent_id}")]
    /// The specified agent could not be found in the registry
    AgentNotFound {
        /// ID of the agent that could not be found
        agent_id: AgentId,
    },

    #[error("Message too large: {size} bytes (max: {max_size} bytes)")]
    /// Message exceeds the maximum allowed size limit
    MessageTooLarge {
        /// Actual size of the message in bytes
        size: usize,
        /// Maximum allowed message size in bytes
        max_size: usize,
    },

    #[error("Queue full: {queue_type}")]
    /// Message queue has reached capacity and cannot accept new messages
    QueueFull {
        /// Type of queue that is full (e.g., "inbound", "outbound")
        queue_type: String,
    },

    #[error("Network error: {source}")]
    /// Network communication failure occurred
    NetworkError {
        #[source]
        /// Underlying network error that caused the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Serialization error: {source}")]
    /// Message serialization or deserialization failed
    SerializationError {
        #[source]
        /// Underlying JSON serialization error
        source: serde_json::Error,
    },

    #[error("Timeout: operation took longer than {timeout_ms}ms")]
    /// Operation exceeded the configured timeout duration
    Timeout {
        /// Timeout duration that was exceeded in milliseconds
        timeout_ms: u64,
    },

    #[error("Configuration error: {message}")]
    /// Invalid configuration settings detected
    ConfigurationError {
        /// Description of the configuration problem
        message: String,
    },

    #[error("Storage error: {source}")]
    /// Database or storage system operation failed
    StorageError {
        #[source]
        /// Underlying storage error that caused the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Circuit breaker open for node: {node_id}")]
    /// Circuit breaker protection activated due to repeated failures
    CircuitBreakerOpen {
        /// ID of the node that has circuit breaker protection active
        node_id: NodeId,
    },

    #[error("Message validation error: {message}")]
    /// General FIPA message validation failed
    MessageValidationError {
        /// Description of the validation failure
        message: String,
    },

    #[error("Conversation threading error: {message}")]
    /// FIPA conversation threading validation failed
    ConversationThreadingError {
        /// Description of the threading validation failure
        message: String,
    },

    #[error("Content size error: {message}")]
    /// Message content exceeds size limits
    ContentSizeError {
        /// Description of the content size problem
        message: String,
    },

    #[error("Resource exhausted: {resource}")]
    /// System resource limit has been reached or exceeded
    ResourceExhausted {
        /// Type of resource that was exhausted (e.g., "memory", "connections")
        resource: String,
    },
}

/// Delivery-specific errors
#[derive(Debug, Error)]
pub enum DeliveryError {
    #[error("Local delivery failed: {source}")]
    /// Failed to deliver message locally to agent
    LocalDeliveryFailed {
        #[source]
        /// Underlying error that caused the delivery failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Remote delivery failed to node {node_id}: {source}")]
    /// Failed to deliver message to remote node
    RemoteDeliveryFailed {
        /// ID of the remote node that failed to receive the message
        node_id: NodeId,
        #[source]
        /// Underlying network or communication error
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Circuit breaker open for node: {node_id}")]
    /// Circuit breaker protection activated for remote node
    CircuitBreakerOpen {
        /// ID of the node with active circuit breaker protection
        node_id: NodeId,
    },

    #[error("Connection pool exhausted for node: {node_id}")]
    /// Connection pool to remote node has no available connections
    ConnectionPoolExhausted {
        /// ID of the node with exhausted connection pool
        node_id: NodeId,
    },

    #[error("Serialization failed: {source}")]
    /// Message serialization failed during delivery preparation
    SerializationFailed {
        #[source]
        /// Underlying JSON serialization error
        source: serde_json::Error,
    },

    #[error("Retry limit exceeded: {max_retries}")]
    /// Maximum number of delivery retries has been exceeded
    RetryLimitExceeded {
        /// Maximum retry limit that was exceeded
        max_retries: MaxRetries,
    },
}

/// Conversation management errors
#[derive(Debug, Error)]
pub enum ConversationError {
    #[error("Conversation not found: {conversation_id}")]
    /// Specified conversation could not be found in storage
    ConversationNotFound {
        /// ID of the conversation that was not found
        conversation_id: ConversationId,
    },

    #[error("Too many participants: {count} (max: {max})")]
    /// Conversation has exceeded the maximum allowed participants
    TooManyParticipants {
        /// Current number of participants
        count: usize,
        /// Maximum allowed participant count
        max: usize,
    },

    #[error("Storage error: {source}")]
    /// Database or storage operation failed for conversation
    StorageError {
        #[source]
        /// Underlying storage error that caused the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Conversation timeout: {conversation_id}")]
    /// Conversation has exceeded its configured timeout duration
    ConversationTimeout {
        /// ID of the conversation that timed out
        conversation_id: ConversationId,
    },
}

/// Agent registry errors
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Agent already registered: {agent_id}")]
    /// Attempted to register an agent that is already registered
    AgentAlreadyRegistered {
        /// ID of the agent that is already registered
        agent_id: AgentId,
    },

    #[error("Agent not found: {agent_id}")]
    /// Requested agent could not be found in the registry
    AgentNotFound {
        /// ID of the agent that was not found
        agent_id: AgentId,
    },

    #[error("Invalid agent state transition: {from:?} -> {to:?}")]
    /// Attempted invalid agent state transition
    InvalidStateTransition {
        /// Current agent state
        from: AgentState,
        /// Attempted target state
        to: AgentState,
    },

    #[error("Storage error: {source}")]
    /// Database or storage operation failed for agent registry
    StorageError {
        #[source]
        /// Underlying storage error that caused the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Health check failed for agent: {agent_id}")]
    /// Agent health check operation failed
    HealthCheckFailed {
        /// ID of the agent that failed health check
        agent_id: AgentId,
    },
}

/// Router health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Router is operating normally with no issues
    Healthy,
    /// Router is operational but with reduced performance
    Degraded {
        /// Description of the performance degradation
        reason: String,
    },
    /// Router is experiencing serious problems affecting functionality
    Unhealthy {
        /// Description of the health problems
        reason: String,
    },
}

/// Router performance statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    // Throughput metrics
    /// Current message processing rate in messages per second
    pub messages_per_second: f64,
    /// Highest observed message processing rate since startup
    pub peak_messages_per_second: f64,
    /// Total number of messages processed since startup
    pub total_messages_processed: MessageCount,

    // Latency metrics (in microseconds)
    /// 50th percentile routing latency in microseconds
    pub routing_latency_p50: u64,
    /// 90th percentile routing latency in microseconds
    pub routing_latency_p90: u64,
    /// 99th percentile routing latency in microseconds
    pub routing_latency_p99: u64,
    /// 99.9th percentile routing latency in microseconds
    pub routing_latency_p999: u64,

    // Error metrics
    /// Total number of routing errors since startup
    pub total_errors: MessageCount,
    /// Current error rate as a percentage of total messages
    pub error_rate: f64,
    /// Count of errors grouped by error type
    pub errors_by_type: HashMap<String, MessageCount>,

    // Queue metrics
    /// Number of messages waiting in the inbound queue
    pub inbound_queue_depth: usize,
    /// Number of messages waiting in the outbound queue
    pub outbound_queue_depth: usize,
    /// Message queue depth for each registered agent
    pub agent_queue_depths: HashMap<AgentId, usize>,

    // Conversation metrics
    /// Number of currently active conversations
    pub active_conversations: usize,
    /// Total number of conversations created since startup
    pub total_conversations: MessageCount,
    /// Average number of messages per conversation
    pub average_conversation_length: f64,

    // Resource metrics
    /// Current memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Current CPU usage as a percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Current database storage size in bytes
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
    /// Number of currently active conversations
    pub total_active: usize,
    /// Total number of conversations created since startup
    pub total_created: MessageCount,
    /// Average conversation duration in milliseconds
    pub average_duration_ms: u64,
    /// Average number of messages per conversation
    pub average_message_count: f64,
    /// Distribution of conversations by participant count
    pub participants_distribution: HashMap<usize, usize>, // participant_count -> conversation_count
}

/// Dead letter queue statistics
#[derive(Debug, Clone)]
pub struct DeadLetterStats {
    /// Total number of messages in the dead letter queue
    pub total_messages: MessageCount,
    /// Count of dead letter messages grouped by failure reason
    pub messages_by_reason: HashMap<FailureReason, MessageCount>,
    /// Age of the oldest message in the queue in milliseconds
    pub oldest_message_age_ms: Option<u64>,
    /// Total storage size of dead letter queue in bytes
    pub queue_size_bytes: usize,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit breaker is closed, allowing normal operation
    Closed,
    /// Circuit breaker is half-open, testing if service has recovered
    HalfOpen,
    /// Circuit breaker is open, blocking requests due to failures
    Open,
}

/// Retry reasons for failure classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetryReason {
    /// Network communication failure occurred
    NetworkError,
    /// Message queue reached capacity
    QueueFull,
    /// Target agent is not available to receive messages
    AgentUnavailable,
    /// System resource limit has been reached
    ResourceExhausted,
    /// Operation exceeded configured timeout duration
    Timeout,
}

/// Health check results with detailed information
#[derive(Debug, Clone)]
pub struct DetailedHealthStatus {
    /// Overall health status of the system
    pub overall: HealthStatus,
    /// Health status of individual system components
    pub components: HashMap<String, ComponentHealth>,
    /// System uptime in milliseconds
    pub uptime_ms: u64,
    /// Most recent error message if any
    pub last_error: Option<String>,
}

/// Individual component health
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Current health status of this component
    pub status: HealthStatus,
    /// Timestamp of the most recent health check
    pub last_check: MessageTimestamp,
    /// Number of errors since last reset
    pub error_count: u32,
    /// Component-specific performance metrics
    pub metrics: HashMap<String, f64>,
}
