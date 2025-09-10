______________________________________________________________________

## title: "0004. Minimal Core Philosophy" date: 2025-07-31 status: accepted layout: adr categories: [Architecture]

Date: 2025-01-31

## Status

Accepted

## Context

Framework design faces constant pressure to add features:

- Users request built-in workflow orchestration
- Developers want automatic scaling and load balancing
- Enterprises need complex permission systems
- Everyone wants their specific use case supported in core

This feature creep leads to:

- Bloated APIs that are hard to learn
- Rigid abstractions that don't fit all use cases
- Performance overhead from unused features
- Maintenance burden that slows innovation
- Lock-in to specific architectural patterns

## Decision

Caxton will maintain a strictly minimal server design, providing only three core
capabilities:

1. **Agent Runtime**: WebAssembly-based isolation and execution
2. **Message Router**: FIPA protocol implementation including Contract Net
   Protocol (CNP) for agent coordination
3. **Observability Layer**: Structured logging and OpenTelemetry integration

Everything else must be implemented as agents deployed to the server.

This means explicitly NOT including:

- Workflow orchestration languages
- Built-in agent hierarchies or permissions
- Message transformation or routing rules
- Infrastructure-level consensus protocols (Raft, Paxos, PBFT) for distributed
  state agreement
- State management or persistence layers
- Retry/circuit breaker policies
- Event storage or databases

### Important Distinction: Agent Coordination vs Infrastructure Consensus

Caxton DOES include FIPA agent coordination protocols like Contract Net Protocol
(CNP) because:

- CNP is about task delegation and negotiation between agents
- It's essential for multi-agent coordination
- It operates at the application/business logic layer
- It doesn't require distributed state agreement

Caxton does NOT include infrastructure consensus protocols because:

- These solve distributed state agreement problems
- They add operational complexity
- They're better handled by specialized systems (etcd, Consul)
- They operate at the infrastructure layer

## Consequences

### Positive

- **Simplicity**: New users can understand the entire system quickly
- **Flexibility**: Users can build exactly what they need
- **Performance**: No overhead from unused abstractions
- **Maintainability**: Small codebase is easier to maintain and evolve
- **Composability**: Libraries can be mixed and matched
- **Innovation**: Community can experiment without framework constraints

### Negative

- **More work for users**: Must build or find libraries for common patterns
- **Potential fragmentation**: Multiple competing solutions for same problems
- **Steeper learning curve**: No "rails" to guide architecture decisions
- **Missing features**: Some users expect batteries-included frameworks

### Mitigations

- Provide official example agents and patterns (deployed to server)
- Create comprehensive examples showing agent development patterns
- Foster community ecosystem for sharing agent implementations
- Document architectural guidance without enforcing it
- Make it easy to contribute and discover community agents

## Alternatives Considered

### Batteries-Included Framework

- **Pros**: Everything works out of the box
- **Cons**: Bloated, opinionated, hard to customize

### Plugin Architecture

- **Pros**: Core stays small, extensions are modular
- **Cons**: Plugin APIs become another thing to maintain

### Layered Architecture

- **Pros**: Progressive disclosure of complexity
- **Cons**: Layers create artificial boundaries

### Microkernel Pattern

- **Pros**: Minimal core with services
- **Cons**: Still prescribes an architecture

## Philosophical Alignment

This decision aligns with successful minimal systems:

- **Unix philosophy**: Small tools that do one thing well
- **Plan 9**: Everything is a file (everything is an event)
- **Erlang/OTP**: Simple server model, rich ecosystem
- **HTTP**: Simple protocol, endless applications

## Implementation Guidelines

When evaluating new features, ask:

1. Can this be implemented as an agent using the server's capabilities?
2. Would adding this prevent other valid use cases?
3. Is this essential for the server to function?

If the answer to #1 is yes, it doesn't belong in core.

## Example: Workflow Orchestration

Users want workflow orchestration. Instead of building it into core:

```rust
// This is an agent pattern, not core
pub struct WorkflowEngine {
    caxton: Arc<dyn AgentHost>,
}

impl WorkflowEngine {
    pub async fn execute(&self, workflow: Workflow) {
        // Uses Caxton server APIs to:
        // - Spawn coordinator agent
        // - Route messages based on workflow
        // - Track state through events
    }
}
```

## References

- "The Unix Philosophy" by Mike Gancarz
- "Simplicity Matters" by Rich Hickey
- "Worse is Better" by Richard Gabriel
- "The Architecture of Open Source Applications"
