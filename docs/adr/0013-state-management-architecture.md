---
title: "ADR-0013: State Management Architecture"
date: 2025-08-09
status: superseded
superseded_by: ADR-0014
layout: adr
categories: [Architecture]
---


## Status

Superseded by
[ADR-0014: Coordination-First Architecture](0014-coordination-first-architecture.md)

## Supersession Notice

This ADR has been superseded because:

- The PostgreSQL dependency violates the minimal core philosophy
- Shared state creates unnecessary operational complexity
- Agent state management should be a business domain concern
- Coordination protocols provide sufficient functionality without shared state

See [ADR-0014](0014-coordination-first-architecture.md) for the current
approach.

## Context

Caxton needs a robust strategy for managing agent state, orchestrator state, and
conversation history. While the architecture emphasizes stateless WebAssembly
agents, practical production systems require state persistence for:

- Agent crash recovery
- Conversation history and context
- Long-running task coordination
- Audit trails and compliance
- Debugging and observability

The state management system must balance consistency, performance, and
operational simplicity while maintaining the minimal core philosophy.

## Decision

Caxton implements a hybrid state management architecture using event sourcing
for critical state transitions and snapshot strategies for performance
optimization.

### Core State Management Principles

#### 1. Event Sourcing for Audit and Recovery

All state changes are captured as immutable events:

```rust
pub enum StateEvent {
    AgentRegistered { id: AgentId, capabilities: Vec<Capability>, timestamp: Instant },
    MessageSent { from: AgentId, to: AgentId, message: Message, timestamp: Instant },
    TaskAssigned { task_id: TaskId, agent_id: AgentId, timestamp: Instant },
    TaskCompleted { task_id: TaskId, result: TaskResult, timestamp: Instant },
    AgentFailed { id: AgentId, reason: String, timestamp: Instant },
}
```

#### 2. Snapshot Strategies

To prevent unbounded event log growth:

- **Time-based snapshots**: Every 1 hour for active agents
- **Event-count snapshots**: Every 1000 events per conversation
- **Size-based snapshots**: When event log exceeds 10MB
- **On-demand snapshots**: Before maintenance operations

#### 3. State Partitioning

State is partitioned by concern:

- **Orchestrator State**: Agent registry, routing tables, health metrics
- **Conversation State**: Message history, correlation contexts
- **Agent State**: Minimal checkpoint data for recovery
- **Task State**: Assignment, progress, results

### Implementation Architecture

#### Storage Backend

Primary storage uses PostgreSQL with JSONB for flexibility:

```sql
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(50) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    INDEX idx_aggregate (aggregate_id, created_at),
    INDEX idx_type_time (aggregate_type, created_at)
);

CREATE TABLE snapshots (
    aggregate_id UUID PRIMARY KEY,
    aggregate_type VARCHAR(50) NOT NULL,
    snapshot_data JSONB NOT NULL,
    event_version BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

#### Recovery Procedures

##### Agent Recovery

1. Load latest snapshot for agent
2. Replay events since snapshot
3. Reconstruct agent state
4. Resume operation from last checkpoint

```rust
pub async fn recover_agent(agent_id: AgentId) -> Result<AgentState> {
    let snapshot = load_snapshot(agent_id).await?;
    let events = load_events_since(agent_id, snapshot.version).await?;

    let mut state = snapshot.state;
    for event in events {
        state = apply_event(state, event)?;
    }

    Ok(state)
}
```

##### Conversation Recovery

1. Load conversation snapshot
2. Replay message events
3. Restore correlation contexts
4. Resume message processing

##### Orchestrator Recovery

1. Load orchestrator snapshot
2. Replay registration and routing events
3. Rebuild agent registry
4. Restore health metrics
5. Resume normal operation

### State Consistency Guarantees

#### Eventually Consistent Views

- Agent registry updates propagate within 100ms
- Message history available within 500ms
- Task status updates within 1 second

#### Strong Consistency for Critical Operations

- Task assignment uses distributed locks
- Agent registration uses compare-and-swap
- Message ordering preserved per conversation

### Performance Optimizations

#### Write-Through Cache

- Redis for hot state (active conversations)
- 5-minute TTL with refresh on access
- Write-through to PostgreSQL

#### Read Replicas

- Separate read replicas for queries
- Lag monitoring with alerts at >1 second
- Automatic failover to primary if lag exceeds threshold

#### Batch Processing

- Event writes batched per 100ms window
- Snapshot generation in background workers
- Vacuum operations during low-traffic periods

## Consequences

### Positive

- **Complete audit trail** - Every state change is recorded
- **Point-in-time recovery** - Can restore to any previous state
- **Debugging capability** - Can replay scenarios exactly
- **Horizontal scalability** - Event log can be partitioned
- **Compliance ready** - Immutable audit log for regulations

### Negative

- **Storage overhead** - Events and snapshots require significant space
- **Complexity** - Event sourcing adds conceptual overhead
- **Eventual consistency** - Some operations see stale data
- **Operational burden** - Requires snapshot management and cleanup

### Neutral

- Standard PostgreSQL operations knowledge required
- Event sourcing patterns well-understood in industry
- Existing tooling (Kafka, EventStore) could replace if needed

## Migration Path

### Phase 1: Basic Event Logging (Week 1-2)

- Implement event schema
- Add event logging to critical paths
- Deploy PostgreSQL infrastructure

### Phase 2: Snapshot Implementation (Week 3-4)

- Implement snapshot generation
- Add snapshot-based recovery
- Test recovery procedures

### Phase 3: Performance Optimization (Week 5-6)

- Add Redis caching layer
- Implement read replicas
- Optimize query patterns

### Phase 4: Production Hardening (Week 7-8)

- Add monitoring and alerts
- Implement backup strategies
- Document operational procedures

## Alternatives Considered

### Pure Event Streaming (Kafka)

- **Pros**: Proven scale, existing ecosystem
- **Cons**: Operational complexity, requires Kafka expertise
- **Decision**: PostgreSQL simpler for initial implementation

### Document Store (MongoDB)

- **Pros**: Flexible schema, good developer experience
- **Cons**: Weaker consistency guarantees, less operational maturity
- **Decision**: PostgreSQL JSONB provides similar flexibility

### Key-Value Store (DynamoDB/Cassandra)

- **Pros**: Massive scale, predictable performance
- **Cons**: Complex data modeling, expensive at small scale
- **Decision**: Overkill for initial requirements

## Guidelines for State Management

1. **Minimize State**: Agents should be as stateless as possible
2. **Immutable Events**: Never modify past events
3. **Idempotent Operations**: Handle duplicate events gracefully
4. **Bounded Contexts**: Don't share state across boundaries
5. **Explicit Schemas**: Version all event and snapshot formats

## Monitoring and Alerts

Key metrics to track:

- Event write latency (target: \<10ms p99)
- Snapshot generation time (target: \<1 second)
- Recovery time objective (target: \<30 seconds)
- Storage growth rate (alert at >1GB/day)
- Replication lag (alert at >1 second)

## Security Considerations

- Encrypt events at rest using PostgreSQL TDE
- Audit log access with row-level security
- Separate encryption keys for PII data
- Regular backup encryption and testing
- GDPR compliance via event anonymization

## References

- [Event Sourcing Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/event-sourcing)
- [CQRS and Event Sourcing](https://martinfowler.com/bliki/CQRS.html)
- [PostgreSQL JSONB Performance](https://www.postgresql.org/docs/current/datatype-json.html)
- [Observability-First Architecture (ADR-0001)](0001-observability-first-architecture.md)
- [Minimal Core Philosophy (ADR-0004)](0004-minimal-core-philosophy.md)

## Notes

This state management architecture provides the foundation for reliable agent
coordination while maintaining operational simplicity. The event sourcing
approach ensures we never lose critical data, while snapshots keep performance
acceptable. As the system grows, we can migrate to specialized event stores
without changing the conceptual model.
