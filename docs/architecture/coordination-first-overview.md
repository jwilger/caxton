# Coordination-First Architecture Overview

This document provides a comprehensive overview of Caxton's coordination-first architecture, which eliminates external dependencies while providing distributed systems capabilities.

## Key Architectural Shift

Caxton has evolved from requiring external state stores (PostgreSQL) to a **coordination-first architecture** that requires no external dependencies. This fundamental shift provides:

- **Zero external dependencies** - No databases, message queues, or coordination services required
- **Simplified operations** - Deploy Caxton binaries and run
- **Better scalability** - No shared state bottlenecks
- **Improved resilience** - Graceful degradation during failures

## Core Design Principles

### 1. Minimal Core Philosophy
Following [ADR-0004](../adr/0004-minimal-core-philosophy.md), Caxton maintains a minimal core that handles only:
- Agent lifecycle management
- Message routing
- Cluster coordination
- Observability

Business logic and state persistence are delegated to agents and MCP tools.

### 2. Coordination Over Consensus
Using [SWIM protocol](../adr/0014-coordination-first-architecture.md) for:
- Membership management
- Failure detection
- Agent registry gossip
- Eventually consistent coordination

No consensus required for normal operations.

### 3. Local State with Global Coordination
Each Caxton instance:
- Maintains local state in embedded SQLite
- Shares coordination data via gossip
- Routes messages across cluster boundaries
- Handles partitions gracefully

## Architecture Layers

```
┌─────────────────────────────────────────────┐
│           Business Applications              │
├─────────────────────────────────────────────┤
│              Agent Layer                     │
│  (WebAssembly Isolated Agents)              │
├─────────────────────────────────────────────┤
│         FIPA Messaging Protocol              │
│    (Semantic Agent Communication)           │
├─────────────────────────────────────────────┤
│        Coordination Layer (SWIM)             │
│   (Membership, Routing, Discovery)          │
├─────────────────────────────────────────────┤
│          Local State (SQLite)               │
│     (Agent State, Message Queues)           │
├─────────────────────────────────────────────┤
│         Infrastructure (Network)             │
└─────────────────────────────────────────────┘
```

## Key Components

### SWIM Protocol Layer
Handles infrastructure-level coordination:
- **Membership**: Track alive/dead nodes
- **Failure Detection**: Detect node failures in < 5 seconds
- **Gossip Dissemination**: Share agent registry updates
- **Partition Handling**: Graceful degradation in minority partitions

See [ADR-0015](../adr/0015-distributed-protocol-architecture.md) for protocol details.

### FIPA Messaging Layer
Handles application-level communication:
- **Semantic Messages**: Structured agent communication
- **Cross-Cluster Routing**: Automatic message routing
- **Conversation Management**: Track multi-message conversations
- **Protocol Support**: Request/Reply, Contract Net, etc.

See [ADR-0012](../adr/0012-pragmatic-fipa-subset.md) for protocol subset.

### Local State Management
Each instance maintains:
- **Agent Registry**: Local agents and capabilities
- **Message Queues**: Pending and in-flight messages
- **Conversation State**: Active conversation tracking
- **Metrics & Logs**: Local observability data

No shared database required.

### MCP State Tools
For business state persistence:
- **External Interface**: Agents access state via MCP tools
- **Provider Agnostic**: Use any database, API, or storage
- **Business Owned**: State management owned by business domain
- **Clean Separation**: Caxton doesn't manage business state

See [State Tool Specification](../mcp/state-tool-specification.md).

## Operational Benefits

### Deployment Simplicity
```bash
# That's it - no database setup, no external services
caxton server start --cluster
```

### High Availability
- Nodes automatically discover each other
- Agents redistribute on node failure
- No single point of failure
- Graceful degradation during partitions

### Scalability
- Linear scaling by adding nodes
- No shared state bottlenecks
- Efficient gossip protocol
- Distributed message routing

### Observability
- Built-in distributed tracing
- Metrics at every layer
- Structured logging
- Real-time cluster visibility

## Security Architecture

Comprehensive security at every layer:

### Inter-Node Security
- **mTLS**: Mutual TLS between nodes
- **Gossip Encryption**: Encrypted cluster communication
- **Certificate Rotation**: Automatic certificate management

### Agent Security
- **WebAssembly Isolation**: Complete sandboxing
- **Capability-Based**: Fine-grained permissions
- **Resource Limits**: Prevent resource exhaustion

### API Security
- **Multiple Auth Methods**: API keys, JWT, OAuth2, mTLS
- **RBAC**: Role-based access control
- **Rate Limiting**: Prevent abuse
- **Audit Logging**: Complete security trail

See [ADR-0016](../adr/0016-security-architecture.md) for details.

## Performance Characteristics

### Latency Targets
- **Local routing**: < 100μs P50, < 1ms P99
- **Remote routing**: < 5ms P50, < 50ms P99
- **Agent startup**: < 10ms P50, < 100ms P99
- **Gossip convergence**: < 5 seconds

### Throughput
- **Per instance**: 100,000 messages/second
- **Per cluster**: 1,000,000+ messages/second
- **Concurrent agents**: 10,000 per instance
- **Cluster size**: Up to 100 nodes

See [ADR-0017](../adr/0017-performance-requirements.md) for requirements.

## Operational Procedures

### Cluster Bootstrap
```bash
# First node
caxton server start --bootstrap

# Additional nodes
caxton server start --join seed-node:7946
```

### Rolling Upgrades
```bash
# Zero-downtime upgrade
caxton cluster upgrade --version v1.2.0
```

### Monitoring
```bash
# Cluster health
caxton cluster status

# Performance metrics
caxton cluster performance

# Agent distribution
caxton agents list --by-node
```

See [ADR-0018](../adr/0018-operational-procedures.md) for procedures.

## Migration Path

For existing deployments using PostgreSQL:

### Phase 1: Parallel Operation
1. Deploy new Caxton with coordination-first
2. Run parallel to existing deployment
3. Compare behavior and performance

### Phase 2: State Migration
1. Export agent state from PostgreSQL
2. Import into MCP state tools
3. Verify state consistency

### Phase 3: Cutover
1. Route traffic to new deployment
2. Monitor for issues
3. Decommission old deployment

## Comparison with Previous Architecture

| Aspect | PostgreSQL-Based | Coordination-First |
|--------|------------------|-------------------|
| **External Dependencies** | PostgreSQL required | None |
| **Setup Complexity** | Database setup needed | Just run binary |
| **Scalability** | Database bottleneck | Linear scaling |
| **Failure Handling** | Database SPOF | Graceful degradation |
| **Operational Overhead** | Database administration | Minimal |
| **State Consistency** | Strong consistency | Eventually consistent |
| **Network Partitions** | Complete failure | Degraded operation |

## Best Practices

### Deployment
- Use odd number of nodes (3, 5, 7) for quorum
- Distribute across availability zones
- Enable mTLS in production
- Monitor key metrics continuously

### Configuration
- Tune SWIM parameters for cluster size
- Use QUIC transport for better performance
- Enable message compression for WAN
- Set appropriate resource limits

### Operations
- Regular certificate rotation
- Automated backups
- Capacity planning with headroom
- Practice failure scenarios

## Getting Started

1. **Read the ADRs**: Understand architectural decisions
   - [ADR-0014: Coordination-First Architecture](../adr/0014-coordination-first-architecture.md)
   - [ADR-0015: Distributed Protocol Architecture](../adr/0015-distributed-protocol-architecture.md)

2. **Follow the Guides**:
   - [Clustering Guide](../user-guide/clustering.md) - Set up multi-node cluster
   - [Operational Runbook](../operations/operational-runbook.md) - Day-to-day operations
   - [Performance Tuning](../operations/performance-tuning.md) - Optimize performance

3. **Implement Security**:
   - [Security Guide](../developer-guide/security-guide.md) - Security best practices
   - [ADR-0016: Security Architecture](../adr/0016-security-architecture.md) - Security design

4. **Test Thoroughly**:
   - [Testing Strategy](../development/testing-strategy.md) - Comprehensive testing
   - Include chaos testing for partition scenarios

## Summary

Caxton's coordination-first architecture represents a significant simplification while maintaining distributed systems capabilities. By eliminating external dependencies and using proven protocols like SWIM and FIPA, Caxton provides:

- **Operational simplicity** without sacrificing features
- **Production reliability** with graceful failure handling
- **Linear scalability** without bottlenecks
- **Strong security** at every layer
- **Observable behavior** for debugging

This architecture makes Caxton suitable for production multi-agent systems that need to be reliable, scalable, and maintainable.
