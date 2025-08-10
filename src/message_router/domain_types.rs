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
    TryFrom,
    Into
))]
pub struct MessageId(Uuid);

impl MessageId {
    /// Creates a new random message ID
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
    TryFrom,
    Into
))]
pub struct ConversationId(Uuid);

impl ConversationId {
    /// Creates a new random conversation ID
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
    TryFrom,
    Into
))]
pub struct NodeId(Uuid);

impl NodeId {
    /// Creates a new random node ID
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
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
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
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
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
    pub fn now() -> Self {
        Self::new(SystemTime::now())
    }

    /// Gets the inner `SystemTime`
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
    pub fn now() -> Self {
        Self::new(SystemTime::now())
    }

    /// Gets the inner `SystemTime`
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
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
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
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Checks if content is empty
    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    /// Gets the content as bytes
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
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
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
    pub fn now() -> Self {
        Self::new(SystemTime::now())
    }

    /// Gets the inner `SystemTime`
    pub fn as_system_time(&self) -> SystemTime {
        self.into_inner()
    }
}

impl RouteInfo {
    /// Creates new route information
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
    pub fn is_fresh(&self, ttl: std::time::Duration) -> bool {
        match self.updated_at.as_system_time().elapsed() {
            Ok(elapsed) => elapsed < ttl,
            Err(_) => false, // Clock moved backwards, consider stale
        }
    }
}
