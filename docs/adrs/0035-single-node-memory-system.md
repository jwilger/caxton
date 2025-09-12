---
title: "ADR-0035: Single-Node Memory System Architecture"
date: 2025-01-14
status: accepted
layout: adr
categories: [Architecture, Memory System, Performance, Deployment]
---

## Status

Accepted

## Context

The Caxton platform requires a memory system for agents to store and retrieve
contextual information, maintain conversation history, and build knowledge over
time. This system must balance several competing requirements: performance,
scalability, operational simplicity, and alignment with Caxton's
zero-dependency deployment philosophy.

### Core Requirements

**Zero-Dependency Deployment**: Caxton promises single-binary deployment
without external services, databases, or infrastructure dependencies.

**Performance Targets**: Sub-100ms response times for memory operations to
maintain interactive agent experiences.

**Durability**: Agent memory must survive process restarts and system failures
without data loss.

**Scalability Path**: While serving single-node needs initially, the
architecture must provide a clear migration path to distributed systems.

**Operational Simplicity**: System administrators should be able to deploy and
maintain Caxton without specialized database expertise.

### Design Considerations

The memory system architecture impacts several critical aspects:

**Market Reality**: Analysis shows 90%+ of initial use cases involve personal
automation, development workflows, and small team coordination - all
well-served by single-node systems.

**Performance Overhead**: Distributed consensus protocols add 2-5x latency
overhead compared to local operations, significantly impacting user experience.

**Operational Complexity**: Multi-node systems require specialized knowledge
for deployment, monitoring, and troubleshooting that contradicts the simplicity
promise.

**Resource Efficiency**: Single-node systems can leverage OS-level caching,
memory-mapped files, and CPU cache locality for superior performance.

**Migration Patterns**: Users typically start small and grow gradually, making
immediate clustering support premature optimization.

### Current Implementation Context

The embedded memory system (ADR-0030) establishes SQLite as the storage
backend with in-memory caching. The configuration-driven agents (ADR-0028)
require fast memory access for context management. The system needs to provide
excellent single-node performance while preserving future scalability options.

## Decision

We will **optimize for single-node performance with SQLite and Candle** for
the v1.0 release, explicitly deferring clustering and replication capabilities
to post-1.0 development.

### Single-Node Architecture

The memory system will focus exclusively on single-node optimization:

- **Local SQLite Database**: All persistent state stored in a local SQLite
  database file
- **Candle Vector Search**: Embeddings computed and searched locally using
  Candle for CPU/GPU acceleration
- **Memory-Mapped Caching**: Leverage OS page cache for transparent
  performance optimization
- **Write-Ahead Logging**: SQLite WAL mode for concurrent reads with
  consistent writes
- **In-Process Execution**: No network overhead, IPC, or serialization costs

### Durability Strategy

Data durability achieved through periodic snapshots rather than real-time
replication:

- **Automated Hourly Snapshots**: SQLite backup API creates consistent
  point-in-time snapshots
- **Configurable Retention**: Keep daily, weekly, and monthly snapshots based
  on storage availability
- **Export/Import Tools**: Built-in utilities for data migration and backup
  restoration
- **Recovery Testing**: Automated verification of snapshot integrity and
  recoverability

### Performance Optimization

Single-node focus enables aggressive performance optimizations:

- **100K Entity Target**: Support up to 100,000 entities with sub-50ms search
  latency
- **Connection Pooling**: Reuse SQLite connections across agent invocations
- **Prepared Statements**: Cache and reuse query plans for common operations
- **Batch Operations**: Group writes into transactions for 10-100x throughput
  improvement
- **Index Optimization**: Carefully tuned indexes for common query patterns

### Monitoring and Limits

Built-in monitoring to identify when scaling is needed:

- **Entity Count Metrics**: Track total entities, growth rate, and storage usage
- **Performance Metrics**: Measure query latency, write throughput, and cache
  hit rates
- **Capacity Warnings**: Alert when approaching 80% of recommended limits
- **Migration Readiness**: Export tools activate when capacity thresholds are
  reached

### Migration Path

Clear upgrade path when single-node limits are reached:

1. **Export Phase**: Use built-in tools to export memory to portable format
2. **Service Selection**: Choose appropriate distributed backend (Neo4j,
   Qdrant, PostgreSQL)
3. **Import Phase**: Load exported data into chosen service
4. **Configuration Update**: Point Caxton at external memory service
5. **Validation Phase**: Verify data integrity and query performance

### Product Positioning

Position single-node architecture as intentional design choice:

- **"Personal Scale First"**: Optimize for individual and small team
  productivity
- **"Zero Dependencies"**: Maintain deployment simplicity as core value
  proposition
- **"Graduate When Ready"**: Natural progression path as needs grow
- **"Performance Over Complexity"**: 2-5x better latency than distributed
  alternatives

## Alternatives Considered

### Alternative 1: Distributed by Default

Start with a distributed memory system using Raft consensus or similar
protocols.

**Rejected** because it would:

- Violate zero-dependency deployment promise
- Add 2-5x latency overhead for all operations
- Require complex operational expertise
- Serve less than 10% of actual use cases

### Alternative 2: Embedded Database Cluster

Use embedded databases like DuckDB or RocksDB with built-in clustering support.

**Rejected** because it would:

- Still require network configuration and discovery
- Add significant complexity to deployment
- Provide minimal benefits for single-node deployments
- Increase binary size and resource requirements

### Alternative 3: Cloud-Native Services

Default to cloud memory services (AWS DynamoDB, Google Firestore, etc.).

**Rejected** because it would:

- Create vendor lock-in and ongoing costs
- Require internet connectivity for basic operation
- Violate privacy expectations for personal automation
- Contradict self-hosted deployment philosophy

## Consequences

### Positive

**Superior Performance**: 2-5x better latency than distributed systems for
target workloads.

**Operational Simplicity**: No database administration, network
configuration, or cluster management required.

**Cost Efficiency**: No external service costs, minimal resource
requirements, efficient hardware utilization.

**Data Sovereignty**: All data remains local, addressing privacy and
compliance concerns.

**Predictable Behavior**: No network partitions, split-brain scenarios, or
consistency complications.

### Negative

**Scale Limitations**: Hard limit at approximately 100K entities before
performance degrades.

**Single Point of Failure**: No automatic failover or high availability
without external tools.

**Manual Scaling**: Requires explicit migration when outgrowing single-node
capacity.

**Feature Limitations**: Some advanced features (geo-distribution, read
replicas) not available.

### Migration Triggers

Clear indicators for when to migrate to distributed system:

1. **Entity Count**: Approaching 80K entities with growth trajectory
2. **Query Latency**: p99 latency exceeding 100ms consistently
3. **Write Throughput**: Sustained writes exceeding 1000 ops/second
4. **High Availability**: Business requirement for zero-downtime operations
5. **Multi-Region**: Need for geo-distributed data access

## Implementation Approach

The implementation prioritizes performance and reliability:

1. **Performance Baseline**: Establish benchmarks for 100K entities with 50ms
   query target

2. **Monitoring Integration**: Built-in metrics for capacity planning and
   migration decisions

3. **Snapshot System**: Automated hourly snapshots with configurable retention

4. **Export Tools**: Standardized export format compatible with major graph
   databases

5. **Migration Documentation**: Clear guides for upgrading to distributed
   systems when needed

## Alignment with Existing ADRs

- **ADR-0030 (Embedded Memory System)**: Extends embedded approach with
  explicit single-node optimization
- **ADR-0028 (Configuration-Driven Agents)**: Maintains simple configuration
  without cluster complexity
- **ADR-0004 (Minimal Core Philosophy)**: Reduces complexity by deferring
  distributed features
- **ADR-0027 (Single Codebase)**: Avoids separate clustering codebase until
  actually needed

## Related Decisions

- ADR-0030: Embedded Memory System (establishes SQLite foundation)
- ADR-0031: Context Management Architecture (defines memory access patterns)
- ADR-0004: Minimal Core Philosophy (influences architectural simplicity)
- ADR-0028: Configuration-Driven Agent Architecture (primary memory consumer)

## References

- SQLite performance characteristics and scaling limits
- Analysis of multi-agent system deployment patterns
- Industry data on clustering adoption rates and timing
- Performance comparison of local vs distributed databases

---

**Implementation Status**: This ADR documents the decision to optimize for
single-node performance in v1.0, with clustering and replication deferred to
post-1.0 releases. The architecture provides excellent performance for 90%+ of
use cases while maintaining a clear migration path for growth.
