---
title: "ADR-0033: Static Capability Registration"
date: 2025-01-11
status: accepted
layout: adr
categories: [Architecture, Agent Architecture]
---

## Status

Accepted

## Context

The Caxton platform needs a mechanism for agents to advertise their
capabilities so that other agents and the message router can discover what
services each
agent provides. This capability registration system is fundamental to enabling
agents to find and communicate with each other based on functional requirements
rather than hard-coded agent names.

### Core Requirements

**Service Discovery**: Agents need to discover which other agents can fulfill
specific capability requirements without maintaining hard-coded agent mappings.

**Routing Efficiency**: The message router needs to efficiently route messages
to agents based on requested capabilities without runtime discovery overhead.

**Predictability**: System operators need predictable agent behavior where
capabilities remain consistent throughout the agent's lifecycle.

**Type Safety**: The Rust implementation requires compile-time guarantees about
capability declarations to prevent runtime errors.

**Simplicity**: The initial implementation should avoid unnecessary complexity
while not precluding future enhancements.

### Design Considerations

The choice between static and dynamic capability registration impacts several
architectural aspects:

**Static Registration**: Agents declare capabilities at startup and cannot
change them during their lifetime. Changes require agent restart.

**Dynamic Registration**: Agents can register, update, or remove capabilities
at runtime based on changing conditions or learned behaviors.

### Current Implementation Context

The existing architecture (as defined in ADR-0028 for configuration-driven
agents and ADR-0011 for capability registration in code) assumes agents have
defined capabilities but doesn't specify whether these can change at runtime.
The FIPA messaging protocol (ADR-0003, ADR-0012) supports capability-based
routing but doesn't mandate dynamic discovery.

## Decision

We will **implement static capability registration** where agents declare their
capabilities at startup and these capabilities remain fixed for the agent's
entire lifetime.

### Static Registration Model

Agents declare capabilities in their configuration or initialization code:

- Configuration-driven agents specify capabilities in their TOML configuration
  files (per ADR-0032)
- WebAssembly agents declare capabilities through their manifest or
  initialization exports
- Capabilities are registered with the message router during agent startup
- Once registered, capabilities cannot be added, removed, or modified
- Capability changes require agent restart with updated configuration

### Rationale for Static Approach

**Simplicity First**: Static registration dramatically reduces implementation
complexity by eliminating runtime state management, capability versioning, and
discovery protocols.

**Predictable Behavior**: Operators and other agents can rely on consistent
capability sets throughout an agent's lifetime, simplifying debugging and
system reasoning.

**Zero Discovery Overhead**: Capability lookups become simple hashtable
operations with no network calls or consensus protocols required.

**Type Safety**: Static capabilities enable compile-time validation and
type-safe routing without runtime type checks or capability verification.

**Migration Path**: Starting with static registration doesn't preclude adding
dynamic capabilities later as an optional enhancement for specific use cases.

## Alternatives Considered

### Alternative 1: Full Dynamic Registration

Implement complete dynamic capability registration from the start, allowing
agents to modify their capabilities at any time.

**Rejected** because it introduces significant complexity around:

- Capability versioning and consistency across the cluster
- Race conditions between capability updates and message routing
- Discovery protocol implementation and maintenance
- Testing complexity for capability state transitions

### Alternative 2: Hybrid Approach

Allow both static base capabilities and dynamic supplemental capabilities,
where core capabilities are fixed but agents can add optional capabilities at
runtime.

**Rejected** because it combines the complexity of both approaches without
clear benefits for the initial implementation. This could be reconsidered
post-1.0.

### Alternative 3: Capability Leasing

Implement time-bounded capability registration where agents must periodically
renew their capability declarations.

**Rejected** because it adds unnecessary complexity for most use cases and
creates failure modes around lease expiration without clear benefits over
static registration.

## Consequences

### Positive

**Implementation Simplicity**: Static registration requires minimal code -
essentially a registration API call during startup and a lookup table in the
router.

**Performance**: Capability lookups are O(1) hashtable operations with no
network overhead or consensus requirements.

**Debugging**: System state is predictable - the capabilities an agent started
with are the capabilities it has throughout its lifetime.

**Testing**: Test scenarios are straightforward without needing to handle
capability state transitions or timing issues.

**Type Safety**: Capabilities can be validated at compile-time or startup,
preventing runtime capability mismatches.

### Negative

**No Runtime Adaptation**: Agents cannot adapt their capabilities based on
runtime conditions without a full restart.

**Scaling Limitations**: Adding new capabilities to handle increased load
requires deploying new agent instances rather than expanding existing ones.

**Learning Systems**: Agents that could theoretically learn new capabilities
through interaction cannot express these without restart.

**Maintenance Windows**: Capability updates require agent downtime, though this
can be mitigated with rolling deployments.

### Migration Strategy

The static approach provides a clear migration path to dynamic capabilities if
needed:

1. Static capabilities become the "base" capability set
2. Dynamic capability registration added as optional agent feature
3. Router enhanced to handle both static and dynamic lookups
4. Gradual migration of specific agents to dynamic model where beneficial

This ensures we can evolve the system without breaking existing agents or
requiring wholesale architectural changes.

## Implementation Approach

The implementation focuses on simplicity and correctness:

1. **Configuration Integration**: Capability declarations in agent TOML files
   are parsed and validated at startup

2. **Registration Protocol**: Simple one-way registration message from agent to
   router during initialization

3. **Router Storage**: In-memory hashtable mapping capabilities to agent IDs,
   populated during agent startup

4. **Lookup Operations**: Synchronous capability-to-agent lookups with no
   external dependencies

## Alignment with Existing ADRs

- **ADR-0028 (Configuration-Driven Agents)**: Static capabilities align with
  the configuration-driven model where agent behavior is defined upfront
- **ADR-0032 (TOML Agent Configuration)**: Capabilities naturally fit as static
  configuration in TOML files
- **ADR-0011 (Capability Registration in Code)**: Static registration
  simplifies the code-based registration patterns
- **ADR-0003 (FIPA Messaging Protocol)**: Static capabilities work well with
  FIPA's service discovery patterns

## Related Decisions

- ADR-0011: Capability Registration in Code (defines capability concept)
- ADR-0028: Configuration-Driven Agent Architecture (defines agent model)
- ADR-0032: TOML Agent Configuration Format (defines configuration format)
- ADR-0012: Pragmatic FIPA Subset (influences capability discovery needs)

## References

- FIPA Agent Communication specifications for service discovery patterns
- Consul and etcd documentation on service registration approaches
- Analysis of dynamic vs static service discovery in distributed systems

---

**Implementation Status**: This ADR documents an architectural decision for the
initial implementation of capability registration. The static approach provides
a solid foundation that can be extended with dynamic capabilities post-1.0 if
specific use cases demonstrate the need.
