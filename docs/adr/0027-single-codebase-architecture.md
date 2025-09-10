---
title: "ADR-0027: Single Codebase"
date: 2025-09-08
status: accepted
layout: adr
categories: [Architecture, Organization]
---


## Status

Accepted

## Context

The Caxton system consists of two primary components: the server process that
hosts and orchestrates agents, and the CLI client that administrators use to
manage the system. A fundamental architectural decision is whether these
components should be maintained in separate repositories or combined in a single
codebase.

This decision impacts:

- Development velocity and coordination overhead
- Type safety and API contract enforcement
- Build complexity and deployment pipelines
- Code reuse and maintenance burden
- Version compatibility management

The traditional approach often separates client and server to enforce API
boundaries and enable independent evolution. However, our specific context—a
single client type with shared Rust implementation—presents an opportunity to
reconsider this convention.

## Decision

We will maintain both the Caxton server and CLI client in a single codebase as a
unified Rust project with multiple binary targets.

This monorepo approach prioritizes development velocity, type safety, and
operational simplicity over the theoretical benefits of separation that do not
apply to our single-client scenario.

## Decision Drivers

### Type Safety and Contract Enforcement

- Shared domain types ensure compile-time validation of API contracts
- Breaking changes detected immediately during compilation
- No version drift between type definitions
- Refactoring tools work across entire codebase

### Development Velocity

- Atomic commits can update both server and client simultaneously
- No cross-repository pull request coordination
- Single test suite validates end-to-end functionality
- Faster iteration without version negotiation overhead

### Operational Simplicity

- Single build pipeline produces all artifacts
- Unified versioning eliminates compatibility matrices
- One repository to monitor, backup, and secure
- Simplified dependency management with single lock file

### Code Reuse

- Domain types shared without duplication or synchronization
- Common utilities available to both components
- Validation logic written once, used everywhere
- Consistent error handling across system boundaries

## Alternatives Considered

### Separate Repositories

- Advantages: Clear boundaries, independent versioning, enforced API contracts
- Rejected because: Unnecessary overhead for single-client scenario, type
  synchronization burden

### Monorepo with Workspaces

- Advantages: Some separation while maintaining single repository
- Rejected because: Added complexity without clear benefits for our use case

### Client as External Crate

- Advantages: Could be published separately to crates.io
- Rejected because: No current requirement for external distribution

### Generated Client from OpenAPI

- Advantages: Language-agnostic client generation possible
- Rejected because: Loses Rust type safety benefits, adds generation complexity

## Consequences

### Positive

- **Guaranteed Type Safety**: Server and client cannot drift apart
- **Atomic Changes**: Features can be implemented completely in single commits
- **Reduced Cognitive Load**: Developers work in single repository context
- **Simplified CI/CD**: One pipeline serves all components
- **Faster Onboarding**: New developers learn single codebase structure
- **Consistent Tooling**: Single set of lints, formatters, and development tools
- **Efficient Testing**: Integration tests can directly test both components

### Negative

- **Coupling Risk**: Harder to enforce API boundaries through code organization
- **Build Time**: Changes to shared code trigger rebuilds of both binaries
- **Version Lock-step**: Cannot deploy server and client independently
- **Potential Bloat**: Client binary includes server dependencies and vice versa
- **Future Splitting Cost**: Separating later would require significant
  refactoring

### Risk Mitigation

### API Boundary Enforcement

- Clear module separation between server and client code
- Shared types in dedicated module with no business logic
- Code review discipline to maintain boundaries
- Architecture tests to verify dependency directions

### Build Optimization

- Conditional compilation to exclude unnecessary dependencies
- Incremental compilation reduces rebuild impact
- Binary size optimization through release profiles

### Version Coordination

- Server maintains backward compatibility for at least one version
- Version negotiation protocol in initial handshake
- Clear deprecation warnings before breaking changes

### Future Separation Path

- Maintain clean module boundaries from start
- Document which types are truly shared vs. internal
- Regular review of coupling to identify issues early

## Related Decisions

- ADR-0026: Simplified Management API Protocol - REST approach benefits from
  type sharing
- ADR-0009: CLI Tool Design - Assumes integrated client implementation
- ADR-0006: Application Server Architecture - Defines server responsibilities

## Implementation Notes

The architecture will organize code into clear modules:

- Shared domain types and API contracts in common module
- Server implementation in dedicated server module
- CLI implementation in dedicated client module
- Binary targets reference appropriate modules

This structure maintains logical separation while enabling physical co-location.

## References

- Monorepo advantages in "Trunk Based Development" by Paul Hammant
- Google's monorepo approach and tooling investments
- Benefits of type safety in "Domain Modeling Made Functional" by Scott Wlaschin
