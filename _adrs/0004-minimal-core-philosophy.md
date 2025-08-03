---
layout: adr
title: "0004. Minimal Core Philosophy"
date: 2025-01-31T00:00:00Z
status: proposed
categories: [Architecture, Philosophy]
tags: [minimalism, modularity, extensibility]
---

# 0004. Minimal Core Philosophy

Date: 2025-01-31

## Status

Proposed

## Context

Multi-agent frameworks often become bloated with features that serve specific use cases but add complexity for everyone. We need to balance:
- Core functionality that every agent system needs
- Extensibility for specialized requirements
- Maintainability and comprehensibility
- Performance and resource efficiency

The question is what belongs in the core versus what should be extensions.

## Decision

Caxton will follow a minimal core philosophy with a small, focused core and rich extension ecosystem.

Core responsibilities:
- Agent lifecycle management (spawn, stop, restart)
- Message routing and delivery
- WebAssembly runtime and sandboxing
- Basic observability (tracing, metrics)
- FIPA protocol implementation
- MCP tool integration

Extensions handle:
- Specific agent behaviors and logic
- Custom message routing strategies
- Additional observability backends
- Alternative isolation mechanisms
- Domain-specific protocols

## Consequences

### Positive

- **Simplicity**: Core is easy to understand, debug, and maintain
- **Performance**: Minimal overhead from unused features
- **Flexibility**: Extensions can evolve independently
- **Testing**: Smaller core is easier to test thoroughly
- **Security**: Smaller attack surface in the core
- **Adoption**: Lower barrier to entry for new users

### Negative

- **Fragmentation**: Risk of incompatible extensions
- **Documentation burden**: Need docs for core + all extensions
- **Discovery**: Users may not find relevant extensions
- **Integration complexity**: Combining multiple extensions

### Mitigations

- Clear extension APIs and stability guarantees
- Centralized registry of vetted extensions
- Integration testing across common extension combinations
- Extension compatibility matrix
- Rich examples showing extension patterns

## Core Boundaries

### In Core
- Agent process management
- Message transport and delivery
- WASM runtime integration
- Basic FIPA performatives
- OpenTelemetry integration
- MCP protocol support
- Security and sandboxing

### As Extensions
- Complex interaction protocols
- State persistence mechanisms
- Load balancing strategies
- Service discovery
- Agent deployment tools
- Monitoring dashboards
- Language-specific SDKs

## Implementation Strategy

```rust
// Core trait definitions
pub trait Agent {
    async fn handle_message(&self, msg: FipaMessage) -> Result<(), Error>;
}

pub trait MessageRouter {
    async fn route(&self, msg: FipaMessage) -> Result<(), Error>;
}

pub trait ObservabilityProvider {
    fn record_metric(&self, name: &str, value: f64);
    fn start_span(&self, name: &str) -> Span;
}

// Extension points
pub trait AgentExtension {
    fn before_message(&self, msg: &FipaMessage) -> Result<(), Error>;
    fn after_message(&self, msg: &FipaMessage, result: &Result<(), Error>);
}
```

## Extension Guidelines

1. **Stability**: Extensions should use stable core APIs only
2. **Documentation**: Clear docs for installation and configuration
3. **Testing**: Comprehensive test suites including integration tests
4. **Versioning**: Semantic versioning with compatibility matrix
5. **Performance**: Extensions should document their performance impact

## References

- Unix Philosophy: "Do one thing and do it well"
- "The Art of Unix Programming" by Eric Raymond
- Kubernetes extension model
- VSCode extension architecture