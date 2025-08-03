# Caxton Roadmap

This document outlines the development phases for Caxton, with concrete milestones and explicit non-goals.

## Vision

Build the simplest possible server for multi-agent systems, then let the community build amazing agents and workflows.

## Development Phases

### Phase 1: Minimal Core (V1.0) - Q1 2025

**Goal**: Ship a production-ready orchestration server with baseline performance.

**Deliverables**:
- [ ] WASM agent runtime with isolation guarantees  
- [ ] Async message router with FIPA ACL support
- [ ] FIPA Contract Net Protocol (CNP) implementation for task delegation
- [ ] OpenTelemetry integration (traces, metrics, logs)
- [ ] Structured logging with correlation IDs
- [ ] Basic MCP tool integration
- [ ] Health checks and readiness probes

**Success Metrics**:
- Time to first agent: < 10 minutes
- Core API surface: < 10 public types
- Message throughput: 100K/sec minimum
- Memory per agent: < 10MB
- Zero message loss under normal operation

**Explicitly NOT in V1**:
- Distributed clustering
- Complex routing patterns
- Built-in agent templates
- Graphical debugging tools
- Performance optimizations beyond baseline

### Phase 2: Patterns & Performance (V2.0) - Q3 2025

**Goal**: Double performance and provide pattern libraries (not in core).

**Deliverables**:
- [ ] Example pattern library:
  - Request-reply patterns
  - Pub-sub patterns
  - Basic workflow orchestration
  - Circuit breakers
- [ ] Performance improvements:
  - WASM instance pooling
  - Zero-copy optimizations
  - Message batching strategies
  - Parallel message processing
- [ ] Developer experience:
  - Agent testing framework
  - Local debugging tools
  - Trace visualization
- [ ] Language bindings for agents:
  - JavaScript/TypeScript
  - Python
  - Go

**Success Metrics**:
- 2x performance improvement (200K messages/sec)
- 50% reduction in memory usage
- Pattern library covers 80% of use cases
- Agent development possible in 4+ languages

**Still NOT in scope**:
- Infrastructure-level consensus (Raft, Paxos, PBFT)
- Built-in workflow engine
- Agent hierarchies
- Automatic scaling

### Phase 3: Scale & Ecosystem (V3.0) - Q1 2026

**Goal**: Enable planet-scale agent systems through composability.

**Deliverables**:
- [ ] Clustering support (as optional module):
  - Node discovery
  - Message routing across nodes
  - Cross-node messaging
- [ ] Advanced patterns:
  - Distributed workflows
  - State management patterns
  - Persistence adapters
- [ ] Ecosystem tools:
  - Cloud-native operators
  - Monitoring integrations
  - Security scanners
- [ ] Performance at scale:
  - 1M+ messages/sec (clustered)
  - Sub-millisecond p99 latency
  - Automatic backpressure

**Success Metrics**:
- 10x scale increase from V1
- 99.99% availability in production
- 100+ community-contributed patterns
- Major cloud providers offer Caxton

## Permanent Non-Goals

We will **NEVER** add these to Caxton core:

### Complex Orchestration
- ❌ Workflow definition languages
- ❌ BPMN/BPEL support
- ❌ Visual flow designers
- ✅ Users can implement these as agents

### Agent Management
- ❌ Built-in permission systems
- ❌ Agent lifecycle management
- ❌ Resource quotas and limits
- ✅ Cloud platforms can add these

### Message Transformation
- ❌ Message routing rules engine
- ❌ Content-based routing
- ❌ Protocol translation
- ✅ Agents can implement these

### Distributed Systems Magic
- ❌ Consensus protocols (Raft/Paxos)
- ❌ Distributed transactions
- ❌ Exactly-once guarantees
- ✅ Be honest about distributed reality

## How to Contribute

### Phase 1 Priorities
1. **Performance optimizations** - Help us hit throughput targets
2. **WASM security** - Ensure true isolation
3. **Documentation** - Examples and tutorials
4. **Testing** - Chaos testing, benchmarks

### Phase 2 Opportunities  
1. **Pattern development** - Share your agent patterns
2. **Language bindings** - Make agents easy to build
3. **Debugging tools** - Trace visualization and analysis
4. **Performance** - Profile and optimize

### Phase 3 Challenges
1. **Distributed systems** - Clustering and partitioning
2. **Ecosystem** - Integrations and tools
3. **Production hardening** - Real-world testing
4. **Standards** - Work with FIPA and MCP communities

## Principles

1. **Simplicity wins** - Every feature must pay for its complexity
2. **Performance matters** - Fast by default, optimize later
3. **Observability first** - If you can't debug it, it's broken
4. **Composition over features** - Primitives that combine well
5. **Honest about tradeoffs** - No distributed systems magic

## Questions?

Join our discussions:
- GitHub Discussions for design decisions
- Discord for real-time chat
- Monthly community calls

Remember: The best framework is one that gets out of your way. Let's build that together.