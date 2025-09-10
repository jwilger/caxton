# Caxton Roadmap

This document outlines the development phases for Caxton, with concrete
milestones and explicit non-goals. It is aligned with the comprehensive user
story backlog in [PLANNING.md](PLANNING.md).

## Vision

Build the simplest possible server for multi-agent systems, then let the
community build amazing agents and workflows.

## Development Phases

### Phase 1: Minimal Core (V1.0) - Q1 2025

**Goal**: Ship a production-ready orchestration server with baseline
performance.

**User Stories**: P0 (Stories 001-004) + P1 (Stories 005-010)

- Critical foundation components
- Essential features for MVP

**Deliverables**:

- [ ] WASM agent runtime with isolation guarantees (Story 001)
- [ ] Async message router with FIPA ACL support (Stories 002, 005)
- [ ] Agent lifecycle management (Story 003)
- [ ] Local SQLite state storage (Story 004)
- [ ] External agent routing API (REST/HTTP) (Story 006)
- [ ] CLI tool for agent deployment and management (Story 008)
- [ ] OpenTelemetry integration (traces, metrics, logs) (Story 009)
- [ ] Basic MCP tool integration (Story 010)
- [ ] Health checks and readiness probes (Story 017)

**Success Metrics**:

- Time to first agent: < 10 minutes
- External API latency: < 1ms overhead for local calls
- Message throughput: 100K/sec minimum
- Memory per agent: < 10MB
- Zero message loss under normal operation
- API error responses include actionable debugging information

**Explicitly NOT in V1**:

- Distributed clustering
- Complex routing patterns
- Built-in agent templates
- Graphical debugging tools
- Performance optimizations beyond baseline

### Phase 2: Patterns & Performance (V2.0) - Q3 2025

**Goal**: Double performance and provide pattern libraries (not in core).

**User Stories**: P2 (Stories 011-017) + Selected P3 (Stories 018-025)

- Standard production features
- Enhanced security and operations

**Deliverables**:

- [ ] Contract Net Protocol implementation (Story 011)
- [ ] Multi-stage deployment validation (Story 012)
- [ ] Blue-green deployment strategy (Story 013)
- [ ] External agent router enhancements (Story 014)
- [ ] Capability-based discovery (Story 015)
- [ ] Resource management and limits (Story 016)
- [ ] SWIM cluster membership (Story 018)
- [ ] Cross-instance message routing (Story 019)
- [ ] Canary and shadow deployments (Stories 020, 021)
- [ ] Security framework:
  - mTLS inter-node security (Story 022)
  - API authentication methods (Story 023)
  - Role-based access control (Story 024)
- [ ] Agent capability registration (Story 025)
- [ ] Performance improvements:
  - Message batching (Story 031)
  - Agent instance pooling (Story 032)
  - Circuit breakers (Story 029)
  - Rate limiting (Story 030)

**Success Metrics**:

- 2x performance improvement (200K messages/sec)
- 50% reduction in memory usage
- External API supports streaming and batch patterns
- Pattern library covers 80% of use cases
- Agent development possible in 4+ languages
- Production-ready security and auth integrations
- < 30 second cluster convergence
- Zero-downtime deployments

**Still NOT in scope**:

- Infrastructure-level consensus (Raft, Paxos, PBFT)
- Built-in workflow engine
- Agent hierarchies
- Automatic scaling

### Phase 3: Scale & Ecosystem (V3.0) - Q1 2026

**Goal**: Enable planet-scale agent systems through composability.

**User Stories**: Remaining P3 + P4 (Stories 026-040)

- Advanced scaling features
- Enterprise capabilities
- Ecosystem enablement

**Deliverables**:

- [ ] Distributed agent registry (Story 026)
- [ ] Performance monitoring dashboard (Story 027)
- [ ] Automated backup and recovery (Story 028)
- [ ] Cluster auto-scaling (Story 033)
- [ ] Debug tracing interface (Story 034)
- [ ] Chaos engineering support (Story 035)
- [ ] Load testing framework (Story 036)
- [ ] Compliance audit logging (Story 037)
- [ ] Multi-tenancy support (Story 038)
- [ ] Plugin architecture (Story 039)
- [ ] GraphQL API layer (Story 040)
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
- Complete multi-tenant isolation
- Plugin ecosystem thriving
- Enterprise compliance certified

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
4. **Composition over features** - Simple capabilities that combine well
5. **Honest about tradeoffs** - No distributed systems magic

## Implementation Planning

For detailed implementation planning, see [PLANNING.md](PLANNING.md) which
contains:

- 40 comprehensive user stories with full acceptance criteria
- Complete Definition of Done for each story
- Priority-based backlog (P0-P4)
- Coverage matrix mapping all ADRs to stories
- Success metrics alignment

Each story in PLANNING.md represents a complete vertical slice that can be
independently developed, tested, and released.

## Questions?

Join our discussions:

- GitHub Discussions for design decisions
- Discord for real-time chat
- Monthly community calls

Remember: The best server is one that gets out of your way. Let's build
that\\ntogether.
