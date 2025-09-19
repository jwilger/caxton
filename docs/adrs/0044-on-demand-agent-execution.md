---
title: "ADR-0044: On-Demand Agent Execution Model"
date: 2025-09-17
status: proposed
layout: adr
categories: [Architecture, Performance]
---

## Status

Proposed

## Context

Current agent architecture relies on persistent processes for agent execution, creating significant lifecycle management complexity and resource overhead. Field experience reveals several critical problems with the persistent process model:

### Persistent Process Problems

**Lifecycle Management Complexity**: Managing long-running agent processes requires complex supervision systems, health checks, and restart logic. Process crashes must be detected and handled gracefully.

**Resource Waste**: Idle agents consume memory and system resources between requests. With potentially hundreds of configured agents, idle resource consumption becomes problematic.

**Fault Isolation Issues**: Persistent processes can accumulate state corruption over time. One agent's memory corruption or resource leaks can affect other agents sharing the same process space.

**Deployment Complexity**: Rolling updates, configuration changes, and scaling require coordinated process management across the entire agent fleet.

### Market Validation

Analysis of agent usage patterns shows:

- **Bursty Request Patterns**: Most agents receive requests in short bursts with long idle periods
- **Stateless Operations**: 90% of agent operations are stateless orchestration that doesn't benefit from persistent state
- **Fast LLM Response Times**: Modern LLM APIs respond in 100-500ms, making 10-20ms cold start overhead negligible

### Alternative Models Considered

**Persistent Process Pools**: Pre-spawned agents reduce cold starts but still consume idle resources and require complex pool management.

**Container-Based Execution**: Docker containers add 100-500ms overhead and significant resource consumption for simple configuration-based agents.

**Serverless Functions**: External services like AWS Lambda add network latency and vendor lock-in for a fundamentally local operation.

## Decision

We will adopt a **pure on-demand agent execution model** where agents are spawned as fresh processes per request and exit naturally after completion.

### Core Architecture

**Process Lifecycle**:

1. Agent request arrives via CLI or management API
2. Spawn new process/thread with agent configuration
3. Load TOML configuration and initialize external memory connection
4. Execute agent logic with LLM provider and MCP tools
5. Return result to caller
6. Process exits naturally (no cleanup required)

**State Management**:

- **External Memory System**: All agent state stored in external memory system (database, file system)
- **Configuration Loading**: TOML files loaded fresh on each execution
- **No Process State**: Zero persistent state within agent processes

**Fault Isolation**:

- **Perfect Isolation**: Each request executes in completely isolated process space
- **Natural Cleanup**: Process exit automatically cleans up all resources
- **No State Corruption**: Fresh memory space prevents accumulation of corruption

### Performance Characteristics

**Cold Start Overhead**: ~10-20ms for process spawn, configuration load, and memory system connection

**Memory Efficiency**: Zero idle memory consumption; only active requests consume resources

**Fault Recovery**: Instantaneous - failed processes don't affect subsequent requests

## Decision Drivers

### Simplicity Benefits

**Elimination of Lifecycle Management**: No need for process supervision, health checks, restart logic, or graceful shutdown handling.

**Zero Configuration Complexity**: No pool sizing, worker management, or resource allocation decisions required.

**Natural Resource Cleanup**: Process exit automatically reclaims all memory, file handles, and system resources.

### Operational Excellence

**Perfect Fault Isolation**: Process failures cannot affect other agent executions or accumulate state corruption.

**Instant Fault Recovery**: No recovery time or restart sequences - next request gets fresh process.

**Simplified Monitoring**: Process-level metrics naturally provide request-level observability.

### Performance Acceptability

**Cold Start Analysis**: 10-20ms overhead is <5% of typical LLM response times (200-500ms)

**Resource Efficiency**: Eliminates idle resource consumption for hundreds of potentially configured agents

**Scaling Characteristics**: Linear scaling with request load rather than configured agent count

## Alternatives Considered

### Persistent Agent Processes

- **Advantages**: Zero cold start overhead, potential for stateful optimizations
- **Rejected**: Lifecycle complexity outweighs marginal performance benefits; 90% of agents are stateless

### Hybrid Model (Persistent + On-Demand)

- **Advantages**: Best of both worlds for different agent types
- **Rejected**: Adds architectural complexity; unclear decision criteria for when to use which model

### Process Pools

- **Advantages**: Reduced cold starts with some resource efficiency
- **Rejected**: Still requires complex pool management; doesn't eliminate idle resource consumption

## Consequences

### Positive Outcomes

**Dramatic Simplification**: Eliminates entire categories of operational complexity around process lifecycle management

**Resource Efficiency**: Perfect resource utilization - only active requests consume memory/CPU

**Fault Tolerance**: Inherent fault isolation prevents cascading failures and state corruption

**Development Velocity**: Simpler mental model accelerates feature development and debugging

**Deployment Simplicity**: Configuration changes take effect immediately without process coordination

### Implementation Requirements

**Robust External Memory System**: All agent state must persist outside process boundaries (database, file system, external services)

**Fast Configuration Loading**: TOML parsing and memory system connection must be optimized for sub-20ms initialization

**Multi-Step Conversation Orchestration**: External orchestration layer required for conversations spanning multiple agent invocations

**Process Spawn Optimization**: Minimize startup overhead through optimized binary size and initialization paths

### Acceptable Trade-offs

**Cold Start Overhead**: 10-20ms latency increase per request is acceptable given LLM response time context

**No Process-Level Optimizations**: Cannot maintain warm LLM connections or cached computations between requests

**External Dependency Requirements**: Requires reliable external memory system for state persistence

### Risk Mitigation

**Performance Monitoring**: Measure actual cold start times and LLM response time ratios in production

**Configuration Caching**: Optimize TOML loading and memory connection establishment for minimal overhead

**Orchestration Reliability**: Ensure external orchestration layer can handle multi-step conversation failures gracefully

## Implementation Strategy

### Phase 1: Core Execution Framework

1. Implement process spawn mechanism with configuration loading
2. Establish external memory system connection protocol
3. Add basic performance monitoring for cold start times

### Phase 2: Optimization

1. Optimize binary size and initialization paths
2. Implement configuration caching strategies
3. Add detailed performance metrics and alerting

### Phase 3: Advanced Features

1. Multi-step conversation orchestration layer
2. Request batching for efficiency optimizations
3. Advanced monitoring and observability features

## Alignment with Strategic Goals

**ADR-0041 Configuration-First**: On-demand execution perfectly complements configuration-driven agents by eliminating deployment complexity

**ADR-0030 Embedded Memory**: External memory system requirement aligns with existing embedded memory architecture

**ADR-0001 Observability**: Process-level isolation naturally provides request-level observability and metrics

**ADR-0004 Minimal Core**: Eliminates complex process management from core system, maintaining minimal complexity principle

## Measurement Criteria

**Performance Success**: Cold start overhead <5% of total request time in 95th percentile

**Reliability Success**: Zero cross-request state corruption or resource leaks

**Operational Success**: Elimination of process-related incident categories (crashes, hangs, memory leaks)

**Resource Success**: >50% reduction in idle resource consumption compared to persistent model

## References

- [Serverless Computing Performance Analysis](https://aws.amazon.com/lambda/faqs/)
- [Process Spawning Performance in Rust](https://doc.rust-lang.org/std/process/index.html)
- [UNIX Process Model Best Practices](https://www.kernel.org/doc/html/latest/admin-guide/sysctl/kernel.html)
- [LLM API Response Time Benchmarks](https://platform.openai.com/docs/guides/rate-limits)
