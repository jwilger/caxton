# Message Router Component Interaction Diagrams

## Overview

This document presents detailed component interaction diagrams showing how MessageRouter, DeliveryEngine, ConversationManager, and AgentRegistry components collaborate to provide high-performance message routing in the Caxton system.

## Message Flow Diagrams

### 1. Happy Path: Local Agent Message Routing

```mermaid
sequenceDiagram
    participant C as Client
    participant MR as MessageRouter
    participant AR as AgentRegistry
    participant CM as ConversationManager
    participant DE as DeliveryEngine
    participant LA as LocalAgent
    participant OT as OpenTelemetry

    Note over C,OT: Fast Path: Local Agent Message (< 1ms target)

    C->>+MR: route_message(FipaMessage)
    MR->>OT: start_span("message_route")
    MR->>MR: validate_message()

    MR->>+AR: lookup(receiver_agent_id)
    Note over AR: O(1) HashMap lookup
    AR-->>-MR: AgentLocation::Local(agent)

    par Conversation Management
        MR->>+CM: get_or_create_conversation(conv_id, participants)
        Note over CM: O(1) DashMap lookup or creation
        CM-->>-MR: Conversation
        MR->>CM: update_conversation(conv_id, message)
    and Message Delivery
        MR->>+DE: deliver_local(message, agent)
        DE->>DE: check_agent_availability()
        DE->>+LA: message_queue.send(message)
        LA-->>-DE: Ok(())
        DE-->>-MR: MessageId
    end

    MR->>OT: record_metrics(success, duration)
    MR-->>-C: Ok(MessageId)

    Note over C,OT: Total time: ~200-800μs for local delivery
```

### 2. Remote Agent Message Routing

```mermaid
sequenceDiagram
    participant C as Client
    participant MR as MessageRouter
    participant AR as AgentRegistry
    participant DE as DeliveryEngine
    participant CP as ConnectionPool
    participant RN as RemoteNode
    participant CB as CircuitBreaker
    participant OT as OpenTelemetry

    Note over C,OT: Remote Agent Routing (< 5ms target)

    C->>+MR: route_message(FipaMessage)
    MR->>OT: start_span("message_route")

    MR->>+AR: lookup(receiver_agent_id)
    Note over AR: Check local, then remote routing table
    AR-->>-MR: AgentLocation::Remote(node_id)

    MR->>+DE: deliver_remote(message, node_id)

    DE->>+CB: is_open(node_id)
    CB-->>-DE: false (circuit closed)

    DE->>+CP: get_connection(node_id)
    alt Connection Available
        CP-->>-DE: PooledConnection
    else No Connection
        CP->>CP: create_new_connection(node_id)
        CP-->>DE: PooledConnection
    end

    DE->>DE: serialize_message(message)
    DE->>+RN: send(serialized_message)

    alt Success
        RN-->>-DE: Ok(ack)
        DE->>CB: record_success()
        DE-->>MR: Ok(MessageId)
    else Network Error
        RN-->>DE: Err(network_error)
        DE->>CB: record_failure()
        DE->>DE: schedule_retry(message)
        DE-->>MR: Err(DeliveryError)
    end

    MR->>OT: record_metrics(result, duration)
    MR-->>-C: Result<MessageId>

    Note over C,OT: Total time: ~1-5ms for remote delivery
```

### 3. Agent Discovery Flow (Unknown Agent)

```mermaid
sequenceDiagram
    participant C as Client
    participant MR as MessageRouter
    participant AR as AgentRegistry
    participant GC as GossipCoordinator
    participant RC as RouteCache
    participant NC as NegativeCache
    participant RN as RemoteNodes
    participant DLQ as DeadLetterQueue

    Note over C,RN: Agent Discovery via SWIM Gossip Protocol

    C->>+MR: route_message(FipaMessage)

    MR->>+AR: lookup(unknown_agent_id)
    AR->>RC: check_positive_cache(agent_id)
    RC-->>AR: None (cache miss)

    AR->>NC: check_negative_cache(agent_id)
    NC-->>AR: None (not recently failed)

    AR->>AR: check_local_agents(agent_id)
    AR->>AR: check_remote_routes(agent_id)
    AR-->>-MR: AgentLocation::Unknown

    MR->>NC: put(agent_id, negative_entry)

    MR->>+GC: trigger_agent_discovery(agent_id)
    Note over GC: Asynchronous gossip query

    par Gossip Discovery
        GC->>+RN: gossip_query(agent_id)
        alt Agent Found
            RN-->>-GC: agent_location(node_id)
            GC->>AR: update_remote_route(agent_id, node_id)
            GC->>NC: remove(agent_id)
        else Agent Not Found
            RN-->>GC: not_found
            Note over GC: Continue gossip to other nodes
        end
    and Immediate Response
        MR->>+DLQ: queue_for_retry(message, reason="agent_discovery")
        DLQ-->>-MR: Ok(queued)
        MR-->>-C: Err(RouterError::AgentNotFound)
    end

    Note over C,RN: Discovery continues asynchronously
    Note over DLQ: Retry will succeed after discovery completes
```

### 4. High-Throughput Batch Processing

```mermaid
sequenceDiagram
    participant BP as BatchProcessor
    participant MR as MessageRouter
    participant AR as AgentRegistry
    participant DE as DeliveryEngine
    participant TP as ThreadPool
    participant M1 as Metrics

    Note over BP,M1: Optimized for 100K+ messages/second

    BP->>+MR: process_batch(Vec<FipaMessage>)
    Note over MR: Batch size: 1000 messages

    MR->>MR: validate_batch(messages)

    par Parallel Agent Lookups
        MR->>+AR: batch_lookup(agent_ids)
        AR->>AR: parallel_hashmap_lookups()
        AR->>AR: group_by_location_type()
        AR-->>-MR: HashMap<AgentId, AgentLocation>
    and Conversation Preprocessing
        MR->>MR: group_by_conversation()
        MR->>MR: prepare_conversation_updates()
    end

    MR->>MR: partition_by_delivery_type()

    par Local Delivery Pipeline
        MR->>+DE: batch_deliver_local(local_messages)
        DE->>+TP: parallel_local_delivery()
        loop For each local message
            TP->>TP: agent.queue.try_send(message)
        end
        TP-->>-DE: Vec<Result<MessageId>>
        DE-->>-MR: local_results
    and Remote Delivery Pipeline
        MR->>+DE: batch_deliver_remote(remote_messages)
        DE->>+TP: parallel_remote_delivery()
        DE->>DE: group_by_target_node()
        DE->>DE: reuse_connections()
        loop For each remote batch
            TP->>TP: connection.send_batch(batch)
        end
        TP-->>-DE: Vec<Result<MessageId>>
        DE-->>-MR: remote_results
    end

    MR->>MR: combine_results_maintaining_order()
    MR->>+M1: record_batch_metrics(batch_size, duration, success_rate)
    M1-->>-MR: recorded

    MR-->>-BP: Vec<Result<MessageId>>

    Note over BP,M1: Batch processing: ~10-50ms for 1000 messages
    Note over BP,M1: Throughput: 20K-100K messages/second
```

## Component State Management

### 5. Agent Registration Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Unloaded: Agent Created

    Unloaded --> Loaded: register_agent()
    note right of Loaded
        Agent metadata stored
        Capabilities indexed
        Health monitoring started
    end note

    Loaded --> Running: update_agent_state(Running)
    note right of Running
        Message delivery enabled
        Queue processing active
        Heartbeat monitoring
    end note

    Running --> Draining: update_agent_state(Draining)
    note right of Draining
        No new conversations
        Finish existing messages
        Prepare for shutdown
    end note

    Running --> Running: Normal Operation
    Running --> Loaded: Temporary Pause

    Draining --> Stopped: Complete Graceful Shutdown
    Loaded --> Stopped: Force Stop
    Running --> Stopped: Emergency Stop

    Stopped --> [*]: deregister_agent()
    note right of Stopped
        Clean up indexes
        Archive conversation state
        Stop health monitoring
    end note

    Running --> Failed: Health Check Timeout
    Draining --> Failed: Shutdown Timeout
    Failed --> Stopped: Cleanup

    note right of Failed
        Move messages to DLQ
        Trigger alerts
        Update routing table
    end note
```

### 6. Circuit Breaker State Transitions

```mermaid
stateDiagram-v2
    [*] --> Closed: Initial State

    Closed --> Open: failure_count >= threshold
    note right of Open
        Block all requests
        Return fast failures
        Start recovery timer
    end note

    Open --> HalfOpen: recovery_timeout expired
    note right of HalfOpen
        Allow limited requests
        Test service health
        Collect success metrics
    end note

    HalfOpen --> Closed: success_count >= threshold
    HalfOpen --> Open: any failure detected

    Closed --> Closed: successful_request()
    Open --> Open: blocked_request()

    note right of Closed
        Normal operation
        Track failure rate
        Allow all requests
    end note
```

## Error Handling Flows

### 7. Comprehensive Error Handling

```mermaid
flowchart TD
    Start([Message Received]) --> Validate{Validate Message}

    Validate -->|Invalid| ValidationError[ValidationError]
    ValidationError --> LogError[Log Error & Metrics]
    LogError --> ReturnError([Return Error])

    Validate -->|Valid| Lookup{Agent Lookup}

    Lookup -->|Local Agent| CheckAvailable{Agent Available?}
    Lookup -->|Remote Agent| CheckCircuit{Circuit Breaker Open?}
    Lookup -->|Unknown| TriggerDiscovery[Trigger Discovery]

    CheckAvailable -->|Available| LocalDelivery[Local Delivery]
    CheckAvailable -->|Unavailable| QueueMessage[Queue for Later]

    CheckCircuit -->|Closed| RemoteDelivery[Remote Delivery]
    CheckCircuit -->|Open| CircuitError[Circuit Breaker Error]

    LocalDelivery -->|Success| RecordSuccess[Record Success Metrics]
    LocalDelivery -->|Queue Full| RetryScheduler[Schedule Retry]
    LocalDelivery -->|Agent Error| AgentErrorHandling[Agent Error Handling]

    RemoteDelivery -->|Success| RecordSuccess
    RemoteDelivery -->|Network Error| NetworkErrorHandling[Network Error Handling]
    RemoteDelivery -->|Timeout| TimeoutHandling[Timeout Handling]

    QueueMessage --> RetryScheduler
    CircuitError --> RetryScheduler
    TriggerDiscovery --> DeadLetterQueue[Dead Letter Queue]

    RetryScheduler -->|Max Retries| DeadLetterQueue
    RetryScheduler -->|Retry Available| ExponentialBackoff[Exponential Backoff]

    ExponentialBackoff --> DelayedRetry[Delayed Retry]
    DelayedRetry --> Lookup

    AgentErrorHandling -->|Recoverable| RetryScheduler
    AgentErrorHandling -->|Fatal| DeadLetterQueue

    NetworkErrorHandling -->|Transient| RetryScheduler
    NetworkErrorHandling -->|Permanent| DeadLetterQueue

    TimeoutHandling --> RetryScheduler

    RecordSuccess --> ReturnSuccess([Return MessageId])
    DeadLetterQueue --> AlertOperators[Alert Operators]
    AlertOperators --> ReturnError

    style Start fill:#e1f5fe
    style ReturnSuccess fill:#c8e6c9
    style ReturnError fill:#ffcdd2
    style DeadLetterQueue fill:#fff3e0
    style ValidationError fill:#ffcdd2
    style CircuitError fill:#ffcdd2
```

## Performance Optimization Flows

### 8. Cache Management Strategy

```mermaid
flowchart TD
    Request([Lookup Request]) --> CheckPositive{Positive Cache Hit?}

    CheckPositive -->|Hit & Valid| CacheHit[Return Cached Result]
    CheckPositive -->|Hit & Expired| InvalidatePositive[Invalidate Entry]
    CheckPositive -->|Miss| CheckNegative{Negative Cache Hit?}

    CheckNegative -->|Hit & Valid| NegativeHit[Return Unknown]
    CheckNegative -->|Hit & Expired| InvalidateNegative[Invalidate Entry]
    CheckNegative -->|Miss| PerformLookup[Perform Actual Lookup]

    InvalidatePositive --> PerformLookup
    InvalidateNegative --> PerformLookup

    PerformLookup --> LocalCheck{Check Local Agents}
    LocalCheck -->|Found| CacheLocal[Cache Local Result]
    LocalCheck -->|Not Found| RemoteCheck{Check Remote Routes}

    RemoteCheck -->|Found & Fresh| CacheRemote[Cache Remote Result]
    RemoteCheck -->|Found & Stale| RemoveStale[Remove Stale Route]
    RemoteCheck -->|Not Found| CacheNegative[Cache Negative Result]

    RemoveStale --> TriggerDiscovery[Trigger Discovery]
    TriggerDiscovery --> CacheNegative

    CacheLocal --> AdaptiveTTL[Calculate Adaptive TTL]
    CacheRemote --> AdaptiveTTL
    CacheNegative --> ShortTTL[Short TTL for Negative]

    AdaptiveTTL --> UpdateMetrics[Update Cache Metrics]
    ShortTTL --> UpdateMetrics

    UpdateMetrics --> ReturnResult[Return Result]
    CacheHit --> UpdateMetrics
    NegativeHit --> UpdateMetrics

    ReturnResult --> PredictiveWarm{Predictive Warming Needed?}
    PredictiveWarm -->|Yes| WarmCache[Warm Related Entries]
    PredictiveWarm -->|No| Complete([Complete])

    WarmCache --> Complete

    style Request fill:#e1f5fe
    style Complete fill:#c8e6c9
    style CacheHit fill:#c8e6c9
    style NegativeHit fill:#fff3e0
    style TriggerDiscovery fill:#ffecb3
```

## Monitoring and Observability

### 9. OpenTelemetry Integration Flow

```mermaid
sequenceDiagram
    participant MR as MessageRouter
    participant Tracer as OpenTelemetry Tracer
    participant Metrics as Metrics Collector
    participant Logger as Structured Logger
    participant Exporter as OTLP Exporter

    Note over MR,Exporter: Comprehensive Observability Integration

    MR->>+Tracer: start_span("message_route")
    Tracer-->>-MR: Span

    MR->>MR: span.set_attributes(message_metadata)

    par Trace Collection
        MR->>Tracer: add_event("agent_lookup_start")
        MR->>Tracer: add_event("agent_lookup_complete", attributes)
        MR->>Tracer: add_event("delivery_start")
        MR->>Tracer: add_event("delivery_complete")
    and Metrics Collection
        MR->>+Metrics: record_counter("messages_received", 1)
        MR->>Metrics: record_histogram("lookup_duration", duration_us)
        MR->>Metrics: record_histogram("delivery_duration", duration_us)
        MR->>Metrics: record_gauge("active_conversations", count)
        Metrics-->>-MR: recorded
    and Structured Logging
        MR->>+Logger: info("Message routed successfully", context)
        Logger->>Logger: correlate_with_trace_id(span.trace_id)
        Logger->>Logger: add_structured_fields(message_metadata)
        Logger-->>-MR: logged
    end

    MR->>Tracer: span.set_status(success)
    MR->>Tracer: span.end()

    par Export to Observability Backends
        Tracer->>+Exporter: export_traces(batch)
        Metrics->>Exporter: export_metrics(batch)
        Logger->>Exporter: export_logs(batch)
        Exporter-->>-Tracer: exported
    end

    Note over MR,Exporter: Real-time observability for operations
```

## Configuration and Deployment Flows

### 10. Dynamic Configuration Updates

```mermaid
flowchart TD
    ConfigUpdate([Configuration Update]) --> Validate{Validate New Config}

    Validate -->|Invalid| RejectUpdate[Reject Update]
    Validate -->|Valid| CheckCompatibility{Backward Compatible?}

    CheckCompatibility -->|Yes| HotReload[Hot Reload]
    CheckCompatibility -->|No| RequireRestart[Require Restart]

    HotReload --> UpdateRouterConfig[Update Router Config]
    UpdateRouterConfig --> UpdateDeliveryConfig[Update Delivery Config]
    UpdateDeliveryConfig --> UpdateCacheConfig[Update Cache Config]
    UpdateCacheConfig --> UpdateObservabilityConfig[Update Observability Config]

    UpdateObservabilityConfig --> ValidateRuntime{Runtime Validation}

    ValidateRuntime -->|Success| ApplyChanges[Apply Changes]
    ValidateRuntime -->|Failure| RollbackConfig[Rollback Configuration]

    ApplyChanges --> NotifyComponents[Notify Components]
    NotifyComponents --> RecordConfigChange[Record Configuration Change]

    RequireRestart --> ScheduleRestart[Schedule Graceful Restart]
    ScheduleRestart --> DrainConnections[Drain Connections]
    DrainConnections --> CompleteInflight[Complete In-flight Messages]
    CompleteInflight --> Restart[Restart with New Config]

    RollbackConfig --> LogError[Log Configuration Error]
    RejectUpdate --> LogError

    Restart --> Success([Configuration Applied])
    RecordConfigChange --> Success
    LogError --> Failure([Configuration Failed])

    style ConfigUpdate fill:#e1f5fe
    style Success fill:#c8e6c9
    style Failure fill:#ffcdd2
    style HotReload fill:#c8e6c9
    style RequireRestart fill:#fff3e0
```

## Summary

These component interaction diagrams illustrate:

1. **Message Routing Flows**: Fast local delivery, remote routing with fault tolerance, and agent discovery
2. **State Management**: Agent lifecycle and circuit breaker state transitions
3. **Error Handling**: Comprehensive error recovery with retries, circuit breaking, and dead letter queues
4. **Performance Optimization**: Multi-level caching with adaptive TTL and predictive warming
5. **Observability**: Complete telemetry integration with traces, metrics, and structured logging
6. **Configuration Management**: Hot-reload capabilities with validation and rollback

The diagrams demonstrate how the components collaborate to achieve:
- **Sub-millisecond local routing** (< 1ms target)
- **Fast remote routing** (< 5ms target)
- **High throughput processing** (100K+ messages/second)
- **Comprehensive fault tolerance** with graceful degradation
- **Complete observability** for operational excellence

This architecture provides the foundation for reliable, scalable message routing in the Caxton multi-agent system.
