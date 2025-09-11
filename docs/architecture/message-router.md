---
title: "Message Router Architecture"
date: 2025-01-15
layout: page
categories: [Architecture]
---

## Executive Summary

The Caxton Message Router implements the core routing infrastructure for
agent messages between agents. This consolidated specification defines
the requirements, design, and implementation for the high-performance,
async message routing system with conversation tracking and comprehensive
observability.

**Key Requirements:**

- Async message processing without blocking
- Agent-ID based message routing with O(1) lookup
- Agent registration/deregistration lifecycle
- Conversation context management
- Observability with trace/span IDs
- Performance target: 100,000 messages/second
- Zero message loss under normal operation

## System Architecture

### Core Components

The Message Router consists of five integrated components:

#### 1. MessageRouterImpl

Central orchestration hub that:

- Accepts messages for routing via async channels
- Coordinates with other components for parallel processing
- Manages worker threads with configurable concurrency
- Tracks performance metrics and health status

#### 2. AgentRegistryImpl

O(1) agent lookup system using concurrent data structures:

- `DashMap<AgentId, LocalAgent>` for agent storage
- `DashMap<AgentId, AgentLocation>` for routing cache
- `DashMap<CapabilityName, HashSet<AgentId>>` for capability discovery
- Thread-safe concurrent access without locks

#### 3. DeliveryEngineImpl

Handles actual message delivery with fault tolerance:

- Local delivery via agent message queues
- Remote delivery for distributed deployments
- Message batching for high throughput scenarios
- Circuit breaker patterns for cascade failure prevention

#### 4. ConversationManagerImpl

Manages multi-turn conversations with state persistence:

- Tracks conversation state and participant lists
- Enforces timeout limits and participant constraints
- Maintains message correlation via reply_with/in_reply_to
- Provides conversation history and context

#### 5. FailureHandler (Trait)

Comprehensive error handling with graceful degradation:

- Retry logic with exponential backoff
- Circuit breakers to prevent cascading failures
- Dead letter queue for undeliverable messages
- Graceful degradation strategies during high load

### Message Flow Architecture

```text
Client → MessageRouter → AgentRegistry → DeliveryEngine → Agent
             |               |                    |
             v               v                    v
        ConversationMgr  Capability Index   Local/Remote
             |               |              Delivery
             v               v
         SQLite Storage  Gossip Protocol
```

## Domain Model

All types use strong typing with `nutype` to eliminate primitive obsession:

### Core Identity Types

```rust
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Hash,
                Serialize, Deserialize, Display))]
pub struct MessageId(Uuid);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Hash,
                Serialize, Deserialize, Display))]
pub struct ConversationId(Uuid);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, Hash,
                Serialize, Deserialize, Display))]
pub struct NodeId(Uuid);
```

### Message Structure

Agent message format with Caxton extensions:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    // Standard agent message fields
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
```

### Configuration Types

```rust
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1_000_000),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
           Serialize, Deserialize, Display, Default),
    default = 1000
)]
pub struct ChannelCapacity(usize);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 32),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
           Serialize, Deserialize, Display, Default),
    default = 4
)]
pub struct WorkerThreadCount(usize);
```

## Performance Specifications

### Throughput and Latency Targets

- **Message Routing**: 100,000+ messages/second sustained
- **Local Routing Latency**: <1ms P99 for same-node agents
- **Remote Routing Latency**: <5ms P99 for cross-node routing
- **Agent Lookup**: O(1) time complexity with HashMap indexing
- **Conversation Lookup**: <5ms P95 for active conversations
- **Capability Resolution**: <3ms P95 for cached capabilities

### Memory and Resource Usage

- **Agent Registry**: ~10MB for 10,000 registered agents
- **Active Conversations**: ~50KB per conversation state
- **Message Queue**: Configurable buffer (default: 100MB)
- **Performance Metrics**: ~5MB rolling window for telemetry

## Routing Strategies

### Capability-Based Routing

Messages route to agents based on advertised capabilities rather than
direct agent addressing:

```rust
impl RoutingStrategy for CapabilityRouter {
    fn route_message(
        &self,
        message: &FipaMessage,
        capabilities: &[Capability],
    ) -> Result<Vec<AgentId>, RoutingError> {
        let candidates = self.routing_table
            .find_agents_with_capabilities(capabilities)
            .filter(|agent| agent.health_status == HealthStatus::Healthy)
            .collect::<Vec<_>>();

        match candidates.len() {
            0 => Err(RoutingError::NoCapableAgents),
            1 => Ok(vec![candidates[0].agent_id.clone()]),
            _ => Ok(self.load_balance(candidates)),
        }
    }
}
```

### Load Balancing Algorithms

1. **Round Robin**: Simple rotation through capable agents
2. **Least Connections**: Route to agent with fewest active conversations
3. **Response Time**: Route based on historical response latency
4. **Weighted**: Route based on agent capacity and performance metrics

### Conversation Continuity

Maintain conversation context across multiple message exchanges:

```rust
pub fn route_with_continuity(
    &self,
    message: &FipaMessage,
) -> Result<AgentId, RoutingError> {
    if let Some(conv_id) = message.conversation_id {
        if let Some(conversation) = self.get_conversation(&conv_id) {
            // Prefer routing to the most recent participant
            if let Some(last_agent) = conversation.get_last_participant() {
                if self.is_agent_available(&last_agent) {
                    return Ok(last_agent);
                }
            }
        }
    }

    // Fall back to capability-based routing
    self.route_by_capability(message)
}
```

## Storage Specifications

### Local SQLite Schema

Based on ADR-0014's coordination-first architecture:

```sql
-- Agent registry with capabilities
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

-- Routing table for cross-node agents
CREATE TABLE IF NOT EXISTS routes (
    agent_id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    hops INTEGER DEFAULT 0,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Active conversations with participant tracking
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    participants TEXT NOT NULL, -- JSON array of agent IDs
    protocol TEXT,
    created_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL,
    message_count INTEGER DEFAULT 0
);

-- Message queue for offline/busy agents
CREATE TABLE IF NOT EXISTS message_queue (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    message BLOB NOT NULL, -- Serialized FipaMessage
    priority INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    retry_count INTEGER DEFAULT 0,
    next_retry INTEGER,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_agents_state ON agents(state);
CREATE INDEX IF NOT EXISTS idx_routes_node_id ON routes(node_id);
CREATE INDEX IF NOT EXISTS idx_conversations_last_activity
  ON conversations(last_activity);
CREATE INDEX IF NOT EXISTS idx_message_queue_agent_priority
  ON message_queue(agent_id, priority);
```

## Error Handling and Recovery

### Error Classification

```rust
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum RoutingError {
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },

    #[error("Message too large: {size} bytes (max: {max_size})")]
    MessageTooLarge { size: MessageSize, max_size: MessageSize },

    #[error("Queue full: {queue_type}")]
    QueueFull { queue_type: String },

    #[error("Network error: {source}")]
    NetworkError { source: anyhow::Error },

    #[error("Timeout: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: MessageTimeoutMs },
}
```

### Recovery Strategies

1. **Automatic Retry**: Transient failures with exponential backoff
2. **Alternative Routing**: Route to different capable agent on failure
3. **Circuit Breaking**: Prevent cascade failures during overload
4. **Dead Letter Handling**: Store undeliverable messages for analysis
5. **Graceful Degradation**: Reduce functionality under high load

## Monitoring and Observability

### Metrics Collection

Comprehensive telemetry with Prometheus-compatible metrics:

```rust
lazy_static! {
    static ref MESSAGE_ROUTING_DURATION: Histogram = register_histogram!(
        "caxton_message_routing_duration_seconds",
        "Time spent routing messages"
    ).unwrap();

    static ref ACTIVE_CONVERSATIONS: Gauge = register_gauge!(
        "caxton_active_conversations",
        "Number of active conversations"
    ).unwrap();

    static ref MESSAGE_QUEUE_SIZE: Gauge = register_gauge!(
        "caxton_message_queue_size",
        "Current message queue size"
    ).unwrap();
}
```

### Distributed Tracing

OpenTelemetry integration for end-to-end message flow visibility:

```rust
#[instrument(skip(self, message))]
pub async fn route_message(
    &self,
    message: FipaMessage,
) -> Result<RouteResult, RoutingError> {
    let span = tracing::info_span!(
        "message_routing",
        message_id = %message.message_id,
        sender = %message.sender,
        performative = %message.performative
    );

    async move {
        // Routing logic with automatic span context
    }.instrument(span).await
}
```

### Health Endpoints

```rust
// GET /health/router
pub async fn router_health() -> Json<RouterHealth> {
    Json(RouterHealth {
        queue_size: message_queue.size(),
        active_conversations: conversation_manager.count(),
        routing_latency_p95: metrics.routing_latency_p95(),
        agent_registry_size: routing_table.size(),
        status: determine_health_status(),
    })
}
```

## Testing Strategy

### Performance Benchmarking

```rust
#[cfg(test)]
mod benchmarks {
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

    criterion_group!(benches, bench_local_routing,
                     bench_conversation_management);
    criterion_main!(benches);
}
```

### Integration Test Coverage

- **High-Load Scenario**: 100K messages/second sustained for 5 minutes
- **Fault Tolerance**: Network partitions, node failures, agent crashes
- **Memory Pressure**: Limited memory with growing conversation count
- **Cold Start**: Router startup with existing conversation state
- **Hot Restart**: Graceful shutdown and restart with message preservation

### Acceptance Testing

```rust
#[tokio::test]
async fn acceptance_test_100k_messages_per_second() {
    let router = MessageRouter::new(RouterConfig::production()).await?;
    let message_count = 500_000; // 5 seconds at 100K msg/sec
    let start_time = Instant::now();

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

    // Verify performance targets
    let throughput = message_count as f64 / elapsed.as_secs_f64();
    assert!(throughput >= 100_000.0,
            "Throughput: {:.0} msg/sec", throughput);

    // Verify no message loss
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, message_count, "Message loss detected");

    // Verify latency requirements
    let stats = router.get_stats().await?;
    assert!(stats.routing_latency_p99 < 1000,
            "P99 latency: {}μs", stats.routing_latency_p99);
}
```

## Configuration

### Environment-Specific Settings

```yaml
# Development Environment
router:
  inbound_queue_size: 1000
  outbound_queue_size: 1000
  worker_thread_count: 2
  message_timeout_ms: 30000
  conversation_timeout_ms: 1800000  # 30 minutes
  max_retries: 3
  trace_sampling_ratio: 1.0  # 100% sampling

# Production Environment
router:
  inbound_queue_size: 100000
  outbound_queue_size: 100000
  worker_thread_count: 16
  message_timeout_ms: 10000
  conversation_timeout_ms: 3600000  # 1 hour
  max_retries: 5
  trace_sampling_ratio: 0.01  # 1% sampling
```

### Resource Requirements

- **CPU**: 4 cores minimum, 16 cores recommended for 100K msg/sec
- **Memory**: 8GB minimum, 32GB recommended for large conversation counts
- **Storage**: 100GB minimum for queues and conversation history
- **Network**: 10Gbps NIC for cross-node message routing

## Deployment Considerations

### Single Node Deployment

- In-memory routing table and conversation state
- Local message queue with optional persistence
- Direct agent communication without network overhead
- Suitable for development and small-scale production

### Multi-Node Deployment

- Distributed routing table with gossip synchronization
- External message queue (Redis, RabbitMQ) for durability
- Service discovery integration for agent registry
- Load balancer integration for high availability

## Future Enhancements

### Planned Features

1. **Message Transformation**: Protocol adaptation between agents
2. **Content-Based Routing**: Route based on message content analysis
3. **Predictive Routing**: ML-based routing optimization
4. **Multi-Tenant Support**: Isolated routing per tenant
5. **Geographic Routing**: Location-aware agent selection

### Performance Optimizations

1. **Zero-Copy Message Passing**: Reduce memory allocation overhead
2. **SIMD-Optimized Matching**: Faster capability matching algorithms
3. **Persistent Connections**: Connection pooling for agent communication
4. **Adaptive Batching**: Dynamic batch size based on load patterns

## References

- [ADR-0003: Agent Messaging Protocol](../adr/0003-fipa-messaging-protocol.md)
- [ADR-0012: Pragmatic Agent Messaging Subset](../adr/0012-pragmatic-fipa-subset.md)
- [ADR-0031: Context Management Architecture](../adr/0031-context-management-architecture.md)
- [Performance Tuning Guide](../operations/performance-tuning.md)
- [Security Guide](../operations/devops-security-guide.md)
