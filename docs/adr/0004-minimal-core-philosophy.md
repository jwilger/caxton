# 0004. Minimal Core Philosophy

Date: 2025-01-31

## Status

Proposed

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

Caxton will maintain a strictly minimal core, providing only three primitives:

1. **Event Log**: Append-only record of what happened
2. **Agent Runner**: Executes WASM modules in isolation
3. **Message Router**: Delivers messages between agents

Everything else must be built as libraries on top of these primitives.

This means explicitly NOT including:
- Workflow orchestration languages
- Built-in agent hierarchies or permissions
- Message transformation or routing rules
- Distributed consensus protocols
- State management abstractions
- Retry/circuit breaker policies

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

- Provide official pattern libraries (but not in core)
- Create comprehensive examples showing how to build common patterns
- Foster community ecosystem for sharing solutions
- Document architectural guidance without enforcing it
- Make it easy to contribute and discover community libraries

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
- **Erlang/OTP**: Simple primitives, rich ecosystem
- **HTTP**: Simple protocol, endless applications

## Implementation Guidelines

When evaluating new features, ask:
1. Can this be built as a library on top of the three primitives?
2. Would adding this prevent other valid use cases?
3. Is this essential for the primitives to function?

If the answer to #1 is yes, it doesn't belong in core.

## Example: Workflow Orchestration

Users want workflow orchestration. Instead of building it into core:

```rust
// This is a library, not core
pub struct WorkflowEngine {
    caxton: Arc<dyn AgentHost>,
}

impl WorkflowEngine {
    pub async fn execute(&self, workflow: Workflow) {
        // Uses Caxton's primitives to:
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