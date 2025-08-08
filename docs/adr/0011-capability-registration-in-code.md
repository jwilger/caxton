---
layout: adr
title: "0011. Capability Registration in Code"
status: accepted
date: 2025-08-08
---

# ADR-0011: Capability Registration in Code

## Status
Accepted

## Context
Agent systems need to advertise their capabilities so the orchestration layer can route messages and tasks appropriately. There are two main approaches:
1. Declarative configuration (JSON/YAML manifests)
2. Programmatic registration (in agent initialization code)

Initially, documentation showed capabilities in both places, creating redundancy and potential conflicts.

## Decision
Capabilities MUST be registered programmatically in agent initialization code. Agent manifests are strictly limited to deployment configuration (resources, environment, scaling).

### Manifest Schema
Agent manifests contain only:
- `name`: Agent identifier
- `resources`: Memory and CPU limits
- `environment`: Runtime environment variables
- `scaling`: Instance count and autoscaling rules

The manifest schema enforces this with `additionalProperties: false`.

## Consequences

### Positive
- **Single source of truth**: Agent code defines behavior and capabilities
- **Type safety**: Compile-time checking in strongly-typed languages
- **Dynamic capabilities**: Can register based on runtime conditions
- **Cleaner testing**: Easy to mock/stub capabilities in tests
- **Self-documenting code**: Capabilities visible in implementation
- **Simpler manifests**: Focus purely on deployment concerns
- **Clear separation**: Code = behavior, Manifest = deployment

### Negative
- **No static analysis**: Can't determine capabilities without running code
- **Deployment complexity**: Can't filter deployments by capability at manifest level
- **Runtime discovery**: Capabilities only known after agent initialization

### Neutral
- Version information also moves to code constants
- Protocol is fixed (FIPA-ACL) rather than configurable
- No ontology configuration (use JSON schemas in code)

## Alternatives Considered

### 1. Capabilities in Manifest Only
- ❌ No dynamic registration
- ❌ Redundant with code behavior
- ❌ Can drift from implementation

### 2. Capabilities in Both Places
- ❌ Redundancy and potential conflicts
- ❌ Unclear which is authoritative
- ❌ Maintenance burden

### 3. Capability Discovery Protocol
- ❌ Additional complexity
- ❌ Performance overhead
- ❌ Not needed for known agents

## Implementation
```rust
impl Agent for MyAgent {
    async fn initialize(&mut self, ctx: &AgentContext) -> Result<()> {
        // Capabilities registered here, not in manifest
        ctx.register_capability("data_processing").await?;
        ctx.register_capability("ml_inference").await?;
        Ok(())
    }
}
```

## References
- [Building Agents Documentation](../../website/docs/developer-guide/building-agents.md)
- [Minimal Core Philosophy (ADR-0004)](0004-minimal-core-philosophy.md)
