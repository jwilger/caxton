# Message Router Architecture

## Overview

The Message Router is the core communication infrastructure for the Caxton multi-agent system, providing high-performance, asynchronous message routing between agents. It achieves 100,000+ messages/second throughput through lock-free data structures and async Rust with Tokio.

## Architecture

### System Context

The Message Router sits at the heart of each Caxton instance, coordinating message flow between agents running in isolated WebAssembly sandboxes. It integrates with:

- **WasmRuntime**: Manages agent lifecycle and execution
- **Sandbox**: Provides isolated execution environments
- **OpenTelemetry**: Distributed tracing and metrics
- **SQLite**: Local coordination state storage
- **SWIM Protocol**: Gossip-based cluster membership

### Core Components

#### 1. MessageRouterImpl
Central orchestration hub that:
- Accepts messages for routing via async channels
- Coordinates with other components
- Manages worker threads for parallel processing
- Tracks performance metrics

#### 2. AgentRegistryImpl
O(1) agent lookup system using:
- `DashMap<AgentId, LocalAgent>` for agent storage
- `DashMap<AgentId, AgentLocation>` for routing cache
- `DashMap<CapabilityName, HashSet<AgentId>>` for capability discovery
- Thread-safe concurrent access without locks

#### 3. DeliveryEngineImpl
Handles actual message delivery:
- Local delivery via agent message queues
- Remote delivery preparation (for future cluster support)
- Message batching for high throughput
- Circuit breaker patterns for fault tolerance

#### 4. ConversationManagerImpl
Manages multi-turn conversations:
- Tracks conversation state and participants
- Enforces timeouts and participant limits
- Maintains message correlation
- Provides conversation history

#### 5. FailureHandler (Trait)
Comprehensive error handling:
- Retry logic with exponential backoff
- Circuit breakers to prevent cascading failures
- Dead letter queue for undeliverable messages
- Graceful degradation strategies

## Message Flow

```
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

### Core Types
- `AgentId`: Unique agent identifier
- `MessageId`: Unique message identifier
- `ConversationId`: Conversation correlation ID
- `NodeId`: Cluster node identifier

### Message Types
- `FipaMessage`: FIPA-ACL compliant message structure
- `Performative`: FIPA message types (REQUEST, INFORM, etc.)
- `MessageContent`: Validated message payload
- `DeliveryOptions`: Reliability and priority settings

### Configuration Types
- `RouterConfig`: Environment-specific configurations
- `ChannelCapacity`: Queue size limits
- `WorkerThreadCount`: Parallelism control
- `MessageTimeoutMs`: Timeout specifications

## Performance Characteristics

- **Throughput**: 236,000+ messages/second (measured)
- **Local Routing Latency**: < 1ms P99
- **Remote Routing Latency**: < 5ms P99 (target)
- **Memory Usage**: O(agents + conversations)
- **Agent Lookup**: O(1) time complexity
- **Capability Discovery**: O(1) with hash indexing

## Configuration

Three pre-configured environments:

### Development
- Small queues for quick failure detection
- Detailed logging enabled
- Short timeouts for rapid iteration

### Testing
- Large queues to handle test loads
- Balanced timeouts
- Metrics collection enabled

### Production
- Optimized queue sizes
- Minimal logging overhead
- Extended timeouts for reliability
- Full observability integration

## Observability

Complete observability through OpenTelemetry:

- **Traces**: End-to-end message flow with correlation
- **Metrics**: Throughput, latency, error rates, queue depths
- **Logs**: Structured logging with trace correlation
- **Health Checks**: Component health and performance monitoring

## Thread Safety

All components are thread-safe and optimized for concurrent access:

- Lock-free data structures (DashMap, atomic operations)
- Message passing via async channels
- Immutable message objects
- Actor-model inspired design

## Testing

Comprehensive test coverage:

- 96 unit tests for domain types and components
- 9 integration tests for end-to-end flows
- 12 TDD tests covering all acceptance criteria
- Performance benchmarks validating throughput targets
- Property-based testing for domain type validation

## Future Enhancements

- Distributed routing across cluster nodes
- Persistent message queuing for durability
- Advanced routing strategies (content-based, priority)
- Message compression for network efficiency
- Plugin architecture for custom routing logic
