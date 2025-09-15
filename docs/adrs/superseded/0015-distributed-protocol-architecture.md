---
title: "ADR-0015: Distributed Protocol"
date: 2025-08-09
status: superseded
superseded_by: 0030
layout: adr
categories: [Architecture]
---

## Status

Superseded by ADR-0030 (Embedded Memory System)

## Context

With the coordination-first architecture (ADR-0014), Caxton uses SWIM for
cluster coordination and agent messaging protocols for agent communication.
This ADR clarifies how these
protocols interact and addresses distributed systems concerns including network
partitioning, consistency, and fault tolerance.

## Decision

### Protocol Layer Separation

Caxton implements a clear separation between coordination (SWIM) and
communication (agent messaging) protocols:

#### SWIM Protocol (Infrastructure Layer)

- **Responsibility**: Cluster membership and failure detection
- **Scope**: Caxton instance coordination
- **Data**: Instance liveness, agent registry, routing tables
- **Consistency**: Eventually consistent via gossip
- **Failure Model**: Crash-stop failures

#### Agent Messaging Protocol (Application Layer)

- **Responsibility**: Agent-to-agent semantic messaging
- **Scope**: Business logic communication
- **Data**: Application messages, conversation state
- **Consistency**: Message ordering per conversation
- **Failure Model**: Handled by application

### Cross-Cluster Agent Communication

The distributed message router handles agent-to-agent communication across
cluster boundaries by:

- Attempting local delivery first
- Using SWIM-learned routes for remote agents
- Verifying node liveness before forwarding
- Initiating discovery for unknown agent locations
- Tracking message delivery status

### Network Partition Handling

#### Detection Strategy

Partition detection uses quorum-based evaluation of cluster membership to
determine operational mode:

- **Majority partition**: Continue normal operations, mark missing nodes as
  failed
- **Minority partition**: Degrade to read-only mode, queue writes for later
  replay
- **Isolated nodes**: Enter local-only mode, disable remote routing

#### Healing After Partition

When partitions heal, the system exchanges vector clocks to detect divergence,
merges agent registries, replays queued messages, and resumes normal operations.

### Consistency Models

#### Agent Registry (Eventually Consistent)

The agent registry uses vector clocks for conflict resolution during merging,
with tombstones for handling deletions. Peer registries are merged by comparing
vector clocks to determine the most recent updates.

#### Message Ordering (Per-Conversation)

Messages within conversations are processed in sequence order to maintain
conversation state consistency. Out-of-order messages are queued until they can
be processed in the correct sequence.

### Fault Tolerance Mechanisms

#### Circuit Breaker for Remote Calls

Circuit breaker patterns protect against cascading failures by opening circuits
after failure thresholds are exceeded, with half-open testing for recovery.

#### Supervisor Trees for Agents

Agent supervision uses configurable restart strategies:

- **OneForOne**: Restart only the failed agent
- **OneForAll**: Restart all supervised agents
- **RestForOne**: Restart the failed agent and its dependents

### Message Delivery Guarantees

The system supports configurable delivery semantics:

- **AtMostOnce**: Fire-and-forget delivery (default)
- **AtLeastOnce**: Retry with deduplication
- **ExactlyOnce**: Idempotent delivery with sequence numbers

## Consequences

### Positive

- **Clear separation of concerns**: SWIM handles infrastructure, agent
  messaging handles
  application
- **Graceful degradation**: System continues functioning during partitions
- **Flexible consistency**: Eventually consistent for coordination, stronger
  guarantees available when needed
- **Fault isolation**: Agent failures don't affect cluster coordination
- **Scalable design**: Can handle thousands of agents across dozens of instances

### Negative

- **Complexity**: Two protocols to understand and maintain
- **Eventual consistency**: Agent registry may be temporarily inconsistent
- **Network overhead**: Gossip protocol generates background traffic
- **Partition handling**: Requires careful consideration of business
  requirements

### Neutral

- Standard distributed systems patterns apply
- Similar complexity to other distributed agent systems
- Trade-offs are well-understood in the industry

## Implementation Priorities

1. **Phase 1**: Basic SWIM integration for membership
2. **Phase 2**: Agent registry gossip and routing
3. **Phase 3**: Partition detection and handling
4. **Phase 4**: Advanced features (consensus, exactly-once delivery)

## Implementation Approach

The implementation uses SWIM protocol libraries (such as memberlist-rs) for
cluster coordination, MessagePack for efficient message serialization, and
configurable network transports (TCP/QUIC). Technology choices emphasize
operational simplicity while supporting the distributed protocol architecture
requirements.

## References

- [SWIM Protocol Paper]
  (https://www.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
- [Distributed Systems: Principles and Paradigms]
  (https://www.distributed-systems.net/index.php/books/ds3/)
- [memberlist-rs](https://github.com/vectordotdev/memberlist-rs)
- [MessagePack Specification](https://msgpack.org/)
- [QUIC RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)
- [ADR-0014: Coordination-First Architecture]
  (0014-coordination-first-architecture.md)
- [ADR-0012: Pragmatic Agent Messaging](0012-pragmatic-fipa-subset.md)
- [ADR-0016: Security Architecture](0016-security-architecture.md)
