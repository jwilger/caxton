---
title: "ADR-0014: Coordination Architecture"
date: 2025-08-09
status: proposed
layout: adr
categories: [Architecture]
---


## Status

Proposed (Supersedes ADR-0013)

## Context

ADR-0013 proposed using PostgreSQL for state management with event sourcing and
snapshots. However, after careful analysis, this approach:

- Violates the minimal core philosophy by adding heavyweight dependencies
- Makes Caxton responsible for business domain concerns (agent state)
- Creates operational complexity (backups, migrations, replication)
- Introduces a shared state bottleneck that limits scalability

Further research revealed that Caxton's actual needs are **coordination** rather
than **shared state**:

- Agent discovery and registry
- Health monitoring and failure detection
- Message routing information
- Cluster membership

## Decision

Caxton adopts a **coordination-first architecture** that eliminates shared state
in favor of lightweight coordination protocols. Agent state management becomes a
business domain responsibility through MCP tools.

**Protocol Layering**:

- **SWIM Protocol**: Infrastructure layer for cluster coordination and
  membership
- **Agent Messaging Protocol**: Application layer for semantic agent-to-agent messaging
- **Clear Separation**: These protocols complement rather than compete with each
  other

### Core Principles

#### 1. No Shared State

Each Caxton instance maintains only local state. No external database
dependencies.

#### 2. Coordination Through Gossip

Use SWIM protocol for cluster coordination:

- Scalable membership protocol
- Built-in failure detection
- Eventually consistent
- No single point of failure

#### 3. Agent State via MCP Tools

Agents requiring persistence would use business-provided MCP tools, separating
coordination concerns from business domain state management.

### Architecture Components

#### Local State Storage

Each instance uses embedded SQLite for local coordination state:

- Agent registry cache
- Routing tables
- Message queues during partitions
- Conversation state tracking

#### Cluster Coordination

SWIM protocol provides distributed coordination through gossip-based membership,
failure detection, and agent registry synchronization.

#### Message Routing

Messages are routed by checking local agents first, then consulting
gossip-learned routes, with fallback to location discovery via the cluster
coordination layer.

### State Categories

#### Caxton-Managed (Coordination)

- **Agent Registry**: Which agents exist and their capabilities
- **Cluster Membership**: Which Caxton instances are alive
- **Routing Table**: Which node hosts which agents
- **Health Status**: Liveness and readiness information

#### Business-Managed (State)

- **Agent Checkpoints**: Persistent agent state
- **Conversation History**: Message logs and context
- **Task State**: Long-running operation status
- **Audit Logs**: Compliance and debugging
- **Business Data**: Domain-specific information

### Implementation Approach

Multiple Caxton instances automatically discover each other via SWIM gossip,
share agent registries, and route messages without requiring shared state.
Agents requiring persistent state use business-provided MCP tools for
checkpointing and restoration.

## Consequences

### Positive

- **No external dependencies**: SQLite is embedded, SWIM is a library
- **Linear scalability**: No shared state bottleneck
- **Operational simplicity**: No database administration
- **Fault isolation**: Node failures don't affect others
- **Geographic distribution**: Works naturally across regions
- **Business flexibility**: Choose any state backend via MCP
- **Minimal core maintained**: Caxton remains a message router
- **Partition tolerance**: Graceful degradation during network splits
- **Cross-cluster communication**: Agents can communicate across instance
  boundaries

### Negative

- **Eventual consistency**: Agent registry may be temporarily inconsistent
- **No strong consistency**: Cannot guarantee global ordering
- **Learning curve**: SWIM protocol less familiar than databases
- **Network partitions**: Require careful handling and degraded modes
- **Gossip overhead**: Background network traffic for coordination

### Neutral

- **Different mental model**: Think coordination, not shared state
- **MCP tool requirement**: Businesses must provide state tools if needed
- **Migration complexity**: Existing systems expecting shared state need updates

## Migration Path

The transition would be phased: implement local SQLite storage, integrate SWIM
protocol for clustering, deprecate shared state backends, and establish MCP
state tool interfaces for business domain state management.

## Alternatives Considered

### Keep PostgreSQL (ADR-0013)

- **Pros**: Strong consistency, familiar tooling
- **Cons**: Heavy dependency, operational complexity, scalability limits
- **Decision**: Rejected due to minimal core violation

### Embedded etcd

- **Pros**: Strong consistency, proven in Kubernetes
- **Cons**: Still requires consensus, complex for our needs
- **Decision**: Overkill for coordination-only needs

### Redis with Clustering

- **Pros**: Fast, supports pub/sub
- **Cons**: External dependency, complex cluster setup
- **Decision**: Still violates zero-dependency goal

## Comparison with Industry Systems

### HashiCorp Consul

- Uses SWIM for membership (like our proposal)
- Raft only for critical config (we avoid entirely)
- Proves gossip scales to thousands of nodes

### Apache Cassandra

- Gossip protocol for cluster state
- No central coordinator
- Validates our approach at scale

### Kubernetes

- etcd for config, local state in kubelet
- Similar hybrid model
- Shows pattern works in production

## Guidelines

1. **Think coordination, not consistency**: Design for eventual consistency
2. **Local first**: Prefer local state over distributed state
3. **Gossip sparingly**: Only share essential information
4. **Business owns state**: Let MCP tools handle persistence
5. **Fail independently**: Design for partition tolerance

## Related ADRs

- [ADR-0015: Distributed Protocol Architecture](0015-distributed-protocol-architecture.md)
  \- Details agent messaging/SWIM integration and partition handling
- [ADR-0012: Pragmatic Agent Messaging](0012-pragmatic-fipa-subset.md) - Agent
  communication protocol
- [ADR-0013: State Management Architecture (Superseded)](0013-state-management-architecture.md)
  \- Previous approach
- [ADR-0004: Minimal Core Philosophy](0004-minimal-core-philosophy.md) - Core
  design principle

## References

- [SWIM: Scalable Weakly-consistent Infection-style Process Group Membership Protocol](https://www.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
- [Gossip Protocol (Wikipedia)](https://en.wikipedia.org/wiki/Gossip_protocol)
- [Lightweight State Alternatives Research](../research/lightweight-state-alternatives.md)
- [MCP State Tool Specification](../mcp/state-tool-specification.md)

## Notes

This architecture makes Caxton truly lightweight and cloud-native. By
eliminating shared state, we remove the primary scaling bottleneck and
operational burden. The coordination-first approach aligns perfectly with the
minimal core philosophy while providing all necessary functionality through
intelligent architectural choices.
