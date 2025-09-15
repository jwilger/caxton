---
title: "ADR-0013: State Management"
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

All state changes would be captured as immutable events to provide complete
audit trails and point-in-time recovery capabilities.

#### 2. Snapshot Strategies

To prevent unbounded event log growth, the system would implement periodic
snapshots based on time, event count, and size thresholds.

#### 3. State Partitioning

State would be partitioned by concern:

- **Orchestrator State**: Agent registry, routing tables, health metrics
- **Conversation State**: Message history, correlation contexts
- **Agent State**: Minimal checkpoint data for recovery
- **Task State**: Assignment, progress, results

### Architectural Approach

The system would use PostgreSQL as primary storage with JSONB for schema
flexibility, implementing event sourcing patterns with snapshot optimization for
performance. Recovery procedures would restore system state by combining
snapshots with event replay.

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

The implementation would be phased: establish event logging infrastructure, add
snapshot capabilities, optimize for performance, and finally harden for
production operations.

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

## Operational Considerations

The system would require monitoring of event processing performance, snapshot
generation, recovery procedures, and storage growth. Security would be
implemented through encryption at rest, access controls, and compliance with
data protection requirements.

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
