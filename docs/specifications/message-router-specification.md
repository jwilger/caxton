---
title: "Core Message Router - Detailed Specification"
date: 2025-01-15
layout: page
categories: [Specifications]
---

## Executive Summary

This specification defines the requirements and design for the Core Message
Router, a critical foundation component of the Caxton multi-agent system. The
router enables asynchronous message routing between agents without requiring
them to know infrastructure details, while maintaining conversation context and
providing comprehensive observability.

**Key Requirements:**

- Async message processing without blocking
- Agent-ID based message routing
- Agent registration/deregistration lifecycle
- Conversation context management
- Observability with trace/span IDs
- Performance target: 100,000 messages/second
- Zero message loss under normal operation

## Architecture Analysis

### Current Caxton Architecture

Based on analysis of the existing codebase:

1. **WebAssembly Runtime Foundation**: Established in Story 001 with
   `WasmRuntime`, `Sandbox`, and agent lifecycle management
2. **Domain Types**: Strong type safety using `nutype` for preventing primitive
   obsession
3. **Coordination-First Architecture**: ADR-0014 establishes lightweight
   coordination via SWIM protocol and agent messaging
4. **Agent Lifecycle States**: Unloaded → Loaded → Running → Draining → Stopped
5. **Resource Management**: CPU fuel, memory limits, and message counting
   already implemented

### Domain-Driven Design Analysis

The message router must handle multiple domain concepts:

- **Agent Communication**: Agent message structure and performatives
- **Routing Information**: Agent location and reachability
- **Conversation Management**: Multi-turn dialog state
- **Observability Context**: Trace correlation across system boundaries
- **Performance Metrics**: Throughput, latency, and error rates

## Functional Requirements

### FR1: Asynchronous Message Processing

**Requirement**: The message router MUST process messages asynchronously without
blocking caller threads.

**Acceptance Criteria:**

- Message submission returns immediately with correlation ID
- Processing occurs on background tokio tasks
- Back-pressure is applied through bounded channels
- No caller is blocked waiting for message delivery
- Message ordering is preserved per conversation

**Domain Types Required:**

```rust
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display))]
pub struct MessageId(Uuid);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display))]
pub struct ConversationId(Uuid);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1_000_000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 1000
)]
pub struct ChannelCapacity(usize);
```

### FR2: Agent-ID Based Routing

**Requirement**: Messages MUST be routed to agents based on their unique AgentId
without requiring senders to know agent locations.

**Acceptance Criteria:**

- Local agent lookup occurs in O(1) time using HashMap
- Unknown agents trigger discovery via gossip protocol
- Routing table is updated as agents move between nodes
- Failed routes are retried with exponential backoff
- Dead letter queue captures undeliverable messages

**Domain Types Required:**

```rust
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display))]
pub struct NodeId(Uuid);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 10),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 3
)]
pub struct MaxRetries(u8);

#[nutype(
    validate(greater_or_equal = 100, less_or_equal = 30000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 1000
)]
pub struct RetryDelayMs(u64);
```

### FR3: Agent Registration and Deregistration

**Requirement**: The router MUST handle dynamic agent registration and
deregistration throughout agent lifecycle.

**Acceptance Criteria:**

- Agent registration occurs during deployment (Unloaded → Loaded transition)
- Agent activation updates routing table (Loaded → Running transition)
- Graceful deregistration during shutdown (Running → Draining → Stopped)
- Failed agents are detected and removed from routing table
- Registration state is persisted to local SQLite storage
- Capability metadata is included in registration

**Domain Types Required:**

```rust
#[nutype(
    validate(len_char_min = 1, len_char_max = 1000),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)
)]
pub struct CapabilityDescription(String);

#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display)
)]
pub struct CapabilityName(String);

#[nutype(
    validate(greater_or_equal = 1000, less_or_equal = 300000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 30000
)]
pub struct HealthCheckIntervalMs(u64);
```

### FR4: Message Delivery Failure Handling

**Requirement**: The router MUST handle message delivery failures gracefully
without losing messages.

**Acceptance Criteria:**

- Failed deliveries generate agent FAILURE messages back to sender
- Temporary failures are retried with exponential backoff
- Permanent failures (agent not found) are reported immediately
- Circuit breaker pattern prevents cascade failures
- Dead letter queue stores undeliverable messages for analysis
- Delivery receipts are supported for critical messages

**Domain Types Required:**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureReason {
    AgentNotFound,
    AgentNotResponding,
    NetworkError,
    ResourceExhausted,
    MessageTooLarge,
    InvalidMessage,
}

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 10
)]
pub struct CircuitBreakerThreshold(u32);

#[nutype(
    validate(greater_or_equal = 10000, less_or_equal = 1_000_000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 100000
)]
pub struct DeadLetterQueueSize(usize);
```

### FR5: Conversation Context Management

**Requirement**: The router MUST maintain conversation context to enable
multi-turn agent dialogues.

**Acceptance Criteria:**

- Conversations are identified by unique ConversationId
- Message correlation via reply_with and in_reply_to fields
- Conversation state is persisted across system restarts
- Conversation timeouts automatically clean up stale dialogs
- Conversation participants are tracked for delivery optimization
- Context is preserved during agent migration

**Domain Types Required:**

```rust
#[nutype(
    validate(greater_or_equal = 300000, less_or_equal = 86400000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 1800000  // 30 minutes
)]
pub struct ConversationTimeoutMs(u64);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 10
)]
pub struct MaxConversationParticipants(u8);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize))]
pub struct ConversationCreatedAt(std::time::SystemTime);
```

### FR6: OpenTelemetry Integration

**Requirement**: All messages MUST include trace and span IDs for end-to-end
observability.

**Acceptance Criteria:**

- Trace context is automatically injected into agent message headers
- Span is created for each routing operation
- Trace context propagates across node boundaries
- Custom attributes include agent IDs, message types, and conversation IDs
- Sampling is configurable per environment
- Integration with Jaeger, Zipkin, and OTLP exporters

**Domain Types Required:**

```rust
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)
)]
pub struct TraceId(String);

#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)
)]
pub struct SpanId(String);

#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Display, Default),
    default = 0.1
)]
pub struct TraceSamplingRatio(f64);
```

## Non-Functional Requirements

### NFR1: Performance Requirements

**Target**: 100,000 messages per second sustained throughput per instance.

**Acceptance Criteria:**

- P99 message routing latency < 1ms for local agents
- P99 message routing latency < 5ms for remote agents
- Memory usage grows linearly with active conversations
- CPU usage remains below 80% at target throughput
- Network overhead < 10% of message payload size
- Batch processing optimizes high-volume scenarios

**Implementation Notes:**

- Use `tokio::sync::mpsc` with bounded channels for back-pressure
- Implement message batching for high-throughput scenarios
- Use connection pooling for remote node communication
- Apply zero-copy techniques where possible
- Monitor with high-cardinality metrics

### NFR2: Reliability Requirements

**Target**: Zero message loss under normal operation conditions.

**Acceptance Criteria:**

- Messages are persisted before acknowledgment
- Graceful degradation during high load
- Circuit breakers prevent cascade failures
- Automatic recovery from transient failures
- Heartbeat-based failure detection
- Chaos engineering validates fault tolerance

### NFR3: Scalability Requirements

**Target**: Linear scaling with number of agents and nodes.

**Acceptance Criteria:**

- O(1) agent lookup using HashMap indexing
- O(log n) conversation cleanup using priority queues
- Gossip protocol scales to 1000+ nodes
- Memory usage bounded by conversation count
- No shared state bottlenecks
- Horizontal scaling through clustering

### NFR4: Observability Requirements

**Target**: Complete visibility into message flow for debugging and monitoring.

**Acceptance Criteria:**

- Every message generates structured logs with correlation IDs
- Metrics track latency, throughput, errors, and saturation
- Distributed traces show end-to-end message flow
- Dashboard templates for common monitoring scenarios
- Health check endpoints for orchestration platforms
- Automated performance regression detection

## Message Structure Specification

### Agent Message Format

Based on ADR-0003 and ADR-0012, messages follow agent messaging structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    // Standard message fields
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Performative {
    // Core agent messaging performatives
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

#[nutype(
    validate(less_or_equal = 10485760), // 10MB max
    derive(Debug, Clone, Serialize, Deserialize)
)]
pub struct MessageContent(Vec<u8>);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize))]
pub struct MessageTimestamp(std::time::SystemTime);
```

### Domain Types for Message Processing

```rust
#[nutype(
    validate(len_char_min = 1, len_char_max = 50),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)
)]
pub struct ContentLanguage(String);

#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)
)]
pub struct OntologyName(String);

#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)
)]
pub struct ProtocolName(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryOptions {
    pub priority: MessagePriority,
    pub timeout: Option<MessageTimeoutMs>,
    pub require_receipt: bool,
    pub max_retries: MaxRetries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 1,
    Normal = 5,
    High = 8,
    Critical = 10,
}

#[nutype(
    validate(greater_or_equal = 1000, less_or_equal = 300000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 30000
)]
pub struct MessageTimeoutMs(u64);
```

## Component Architecture

### Core Components

#### 1. MessageRouter

**Responsibility**: Central hub for all message routing operations.

```rust
pub struct MessageRouter {
    // Local agent registry
    local_agents: Arc<RwLock<HashMap<AgentId, LocalAgent>>>,

    // Routing table for remote agents
    routing_table: Arc<RwLock<HashMap<AgentId, NodeId>>>,

    // Active conversations
    conversations: Arc<RwLock<HashMap<ConversationId, Conversation>>>,

    // Message queues
    inbound_queue: mpsc::Receiver<AgentMessage>,
    outbound_queue: mpsc::Sender<AgentMessage>,

    // Components
    delivery_engine: DeliveryEngine,
    conversation_manager: ConversationManager,
    failure_handler: FailureHandler,

    // Configuration
    config: RouterConfig,

    // Metrics
    metrics: RouterMetrics,
}
```

#### 2. DeliveryEngine

**Responsibility**: Handles actual message delivery to local and remote agents.

```rust
pub struct DeliveryEngine {
    local_delivery: LocalDeliveryService,
    remote_delivery: RemoteDeliveryService,
    dead_letter_queue: DeadLetterQueue,
    circuit_breakers: HashMap<NodeId, CircuitBreaker>,
    retry_scheduler: RetryScheduler,
}
```

#### 3. ConversationManager

**Responsibility**: Manages conversation state and participant tracking.

```rust
pub struct ConversationManager {
    active_conversations: HashMap<ConversationId, Conversation>,
    cleanup_scheduler: ConversationCleanupScheduler,
    persistence: ConversationStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: ConversationId,
    pub participants: HashSet<AgentId>,
    pub created_at: ConversationCreatedAt,
    pub last_activity: MessageTimestamp,
    pub message_count: MessageCount,
    pub protocol: Option<ProtocolName>,
}
```

#### 4. Agent Registry

**Responsibility**: Tracks agent lifecycle and capabilities.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAgent {
    pub id: AgentId,
    pub name: AgentName,
    pub state: AgentState,
    pub capabilities: Vec<CapabilityName>,
    pub last_heartbeat: MessageTimestamp,
    pub message_queue: AgentMessageQueue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Unloaded,
    Loaded,
    Running,
    Draining,
    Stopped,
}

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 10000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 1000
)]
pub struct AgentQueueSize(usize);
```

## API Specification

### MessageRouter Public API

```rust
#[async_trait]
pub trait MessageRouter: Send + Sync {
    /// Submit a message for routing
    async fn route_message(&self, message: AgentMessage) -> Result<MessageId, RouterError>;

    /// Register a new agent
    async fn register_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>
    ) -> Result<(), RouterError>;

    /// Deregister an agent
    async fn deregister_agent(&self, agent_id: AgentId) -> Result<(), RouterError>;

    /// Update agent state
    async fn update_agent_state(
        &self,
        agent_id: AgentId,
        state: AgentState
    ) -> Result<(), RouterError>;

    /// Get routing statistics
    async fn get_stats(&self) -> Result<RouterStats, RouterError>;

    /// Health check
    async fn health_check(&self) -> Result<HealthStatus, RouterError>;
}
```

### Error Types

```rust
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum RouterError {
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },

    #[error("Message too large: {size} bytes (max: {max_size})")]
    MessageTooLarge { size: MessageSize, max_size: MessageSize },

    #[error("Queue full: {queue_type}")]
    QueueFull { queue_type: String },

    #[error("Network error: {source}")]
    NetworkError { source: anyhow::Error },

    #[error("Serialization error: {source}")]
    SerializationError { source: serde_json::Error },

    #[error("Timeout: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: MessageTimeoutMs },
}
```

## Storage Specifications

### Local SQLite Schema

Based on ADR-0014's coordination-first architecture, local state is stored in
SQLite:

```sql
-- Agent registry
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    state TEXT NOT NULL,
    capabilities TEXT NOT NULL, -- JSON array
    node_id TEXT NOT NULL,
    last_heartbeat INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Routing table
CREATE TABLE IF NOT EXISTS routes (
    agent_id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    hops INTEGER DEFAULT 0,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Active conversations
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    participants TEXT NOT NULL, -- JSON array of agent IDs
    protocol TEXT,
    created_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL,
    message_count INTEGER DEFAULT 0
);

-- Message queue for offline agents
CREATE TABLE IF NOT EXISTS message_queue (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    message BLOB NOT NULL, -- Serialized AgentMessage
    priority INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    retry_count INTEGER DEFAULT 0,
    next_retry INTEGER,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_agents_state ON agents(state);
CREATE INDEX IF NOT EXISTS idx_agents_updated_at ON agents(updated_at);
CREATE INDEX IF NOT EXISTS idx_routes_node_id ON routes(node_id);
CREATE INDEX IF NOT EXISTS idx_conversations_last_activity ON conversations(last_activity);
CREATE INDEX IF NOT EXISTS idx_message_queue_agent_priority ON message_queue(agent_id, priority);
```

## Configuration Specification

### RouterConfig Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    // Performance settings
    pub inbound_queue_size: ChannelCapacity,
    pub outbound_queue_size: ChannelCapacity,
    pub worker_thread_count: WorkerThreadCount,
    pub batch_size: MessageBatchSize,

    // Timeout settings
    pub message_timeout: MessageTimeoutMs,
    pub conversation_timeout: ConversationTimeoutMs,
    pub health_check_interval: HealthCheckIntervalMs,

    // Retry settings
    pub max_retries: MaxRetries,
    pub base_retry_delay: RetryDelayMs,
    pub retry_backoff_factor: RetryBackoffFactor,

    // Circuit breaker settings
    pub circuit_breaker_threshold: CircuitBreakerThreshold,
    pub circuit_breaker_timeout: CircuitBreakerTimeoutMs,

    // Storage settings
    pub database_path: String,
    pub dead_letter_queue_size: DeadLetterQueueSize,

    // Observability settings
    pub trace_sampling_ratio: TraceSamplingRatio,
    pub metrics_enabled: bool,
    pub structured_logging: bool,
}

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 32),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 4
)]
pub struct WorkerThreadCount(usize);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 100
)]
pub struct MessageBatchSize(usize);

#[nutype(
    validate(greater_or_equal = 1.1, less_or_equal = 5.0),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Display, Default),
    default = 2.0
)]
pub struct RetryBackoffFactor(f64);

#[nutype(
    validate(greater_or_equal = 5000, less_or_equal = 300000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default),
    default = 60000
)]
pub struct CircuitBreakerTimeoutMs(u64);
```

## Metrics and Observability

### Performance Metrics

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterMetrics {
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
```

### Structured Logging Format

```rust
// Example log entry structure
{
  "timestamp": "2025-08-09T10:30:00.123Z",
  "level": "INFO",
  "message": "Message routed successfully",
  "trace_id": "abc123def456",
  "span_id": "789xyz012",
  "agent_id": "agent_001",
  "message_id": "msg_12345",
  "conversation_id": "conv_67890",
  "performative": "REQUEST",
  "routing_time_us": 250,
  "queue_depth": 42,
  "node_id": "node_primary_1"
}
```

## Testing Specifications

### Unit Test Coverage Requirements

- **Message Routing Logic**: 100% path coverage
- **Agent Registration/Deregistration**: 100% state transition coverage
- **Conversation Management**: 100% lifecycle coverage
- **Error Handling**: 100% error path coverage
- **Domain Type Validation**: 100% boundary condition coverage

### Integration Test Scenarios

1. **High-Load Scenario**: 100K messages/second sustained for 5 minutes
2. **Fault Tolerance**: Network partitions, node failures, agent crashes
3. **Memory Pressure**: Limited memory with growing conversation count
4. **Cold Start**: Router startup with existing conversation state
5. **Hot Restart**: Graceful shutdown and restart with message preservation

### Performance Test Benchmarks

```rust
// Benchmark message routing performance
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_local_routing(c: &mut Criterion) {
        c.bench_function("local_message_routing", |b| {
            let router = create_test_router();
            let message = create_test_message();

            b.iter(|| {
                black_box(router.route_message(black_box(message.clone())));
            });
        });
    }

    fn bench_conversation_management(c: &mut Criterion) {
        c.bench_function("conversation_tracking", |b| {
            let manager = ConversationManager::new();

            b.iter(|| {
                let conv_id = ConversationId::generate();
                black_box(manager.create_conversation(black_box(conv_id)));
            });
        });
    }

    criterion_group!(benches, bench_local_routing, bench_conversation_management);
    criterion_main!(benches);
}
```

## Security Specifications

### Message Validation

- **Size Limits**: Messages must not exceed 10MB (configurable)
- **Content Validation**: Agent message structure must be valid
- **Agent Authorization**: Only registered agents can send messages
- **Rate Limiting**: Per-agent message rate limits to prevent flooding
- **Content Sanitization**: Message content is treated as opaque binary data

### Audit Trail

- **Message Flow**: All message routing events are logged
- **Agent Actions**: Registration, deregistration, state changes
- **Performance Events**: Circuit breaker trips, queue overflows
- **Security Events**: Invalid messages, unauthorized attempts

## Deployment Specifications

### Environment Configuration

```yaml
# Development Environment
router:
  inbound_queue_size: 1000
  outbound_queue_size: 1000
  worker_thread_count: 2
  message_timeout_ms: 30000
  conversation_timeout_ms: 1800000  # 30 minutes
  max_retries: 3
  trace_sampling_ratio: 1.0  # 100% sampling for dev

# Production Environment
router:
  inbound_queue_size: 100000
  outbound_queue_size: 100000
  worker_thread_count: 16
  message_timeout_ms: 10000
  conversation_timeout_ms: 3600000  # 1 hour
  max_retries: 5
  trace_sampling_ratio: 0.01  # 1% sampling for prod
```

### Resource Requirements

- **CPU**: 4 cores minimum, 16 cores recommended for 100K msg/sec
- **Memory**: 8GB minimum, 32GB recommended for large conversation counts
- **Storage**: 100GB minimum for message queues and conversation history
- **Network**: 10Gbps NIC for cross-node message routing

## Success Criteria

### Definition of Done

1. **Functional Requirements**: All FR1-FR6 acceptance criteria met
2. **Performance Requirements**: 100K msg/sec sustained throughput achieved
3. **Test Coverage**: >95% code coverage with unit and integration tests
4. **Documentation**: API documentation, deployment guides, troubleshooting
   runbooks
5. **Observability**: Metrics, logging, and tracing working end-to-end
6. **Production Readiness**: Security review passed, chaos testing completed

### Acceptance Testing

```rust
#[tokio::test]
async fn acceptance_test_100k_messages_per_second() {
    let router = MessageRouter::new(RouterConfig::production()).await?;
    let start_time = Instant::now();
    let message_count = 500_000; // 5 seconds at 100K msg/sec

    // Send messages concurrently
    let tasks: Vec<_> = (0..message_count)
        .map(|i| {
            let router = router.clone();
            tokio::spawn(async move {
                let message = create_test_message(i);
                router.route_message(message).await
            })
        })
        .collect();

    // Wait for all messages to be processed
    let results: Vec<_> = futures::future::join_all(tasks).await;
    let elapsed = start_time.elapsed();

    // Verify performance
    let throughput = message_count as f64 / elapsed.as_secs_f64();
    assert!(throughput >= 100_000.0, "Throughput: {:.0} msg/sec", throughput);

    // Verify no message loss
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, message_count, "Message loss detected");

    // Verify latency requirements
    let stats = router.get_stats().await?;
    assert!(stats.routing_latency_p99 < 1000, "P99 latency: {}μs", stats.routing_latency_p99);
}
```

## Implementation Roadmap

### Phase 1: Core Foundation (Week 1)

- Domain types definition with nutype
- Basic MessageRouter structure
- Local agent registration/deregistration
- Simple message routing for local agents
- SQLite storage schema

### Phase 2: Message Processing (Week 2)

- Async message queues with tokio::mpsc
- FIPA message parsing and validation
- Conversation management
- Basic error handling and retries

### Phase 3: Performance Optimization (Week 3)

- Message batching for high throughput
- Connection pooling for remote delivery
- Circuit breaker pattern
- Performance benchmarking

### Phase 4: Observability (Week 4)

- OpenTelemetry integration
- Structured logging
- Metrics collection
- Health check endpoints

### Phase 5: Production Hardening (Week 5)

- Comprehensive error handling
- Dead letter queue
- Graceful shutdown
- Configuration management
- Security hardening

This specification provides the complete foundation for implementing Story 002:
Core Message Router, ensuring type safety through domain types, meeting
performance requirements, and establishing the critical messaging infrastructure
for the Caxton multi-agent system.
