---
title: "ADR-0026: Management API"
date: 2025-09-08
status: accepted
layout: adr
categories: [Architecture, Technology]
---


## Status

Accepted

## Context

Our initial management API design (ADR-0007) proposed a dual-protocol
architecture with gRPC as the primary protocol and a REST gateway for
compatibility. This decision was based on assumptions about multiple client
types, polyglot requirements, and the need for maximum performance across
network boundaries.

Recent analysis of actual requirements reveals a significantly simpler scenario:

- Single client type: caxton-cli command-line tool
- Pure Rust environment: both server and client implemented in Rust
- Local deployment pattern: CLI typically runs on same machine as server
- No browser compatibility requirement: no web dashboard planned
- No third-party integration needs: no webhooks or external API consumers

The complexity of maintaining dual protocols and protobuf compilation
infrastructure now appears unnecessary for our actual use case.

## Decision

We will adopt a single REST/HTTP protocol for the management API, replacing the
dual-protocol approach of ADR-0007.

This decision prioritizes operational simplicity and debugging transparency over
theoretical performance gains and polyglot support that our single-client
scenario does not require.

## Decision Drivers

### Operational Concerns

- Incident response capability: ability to debug production issues quickly
- Tool ecosystem maturity: leverage existing HTTP debugging and monitoring tools
- Deployment simplicity: avoid protobuf compiler dependencies in build pipeline
- Learning curve: REST is universally understood by operations teams

### Development Velocity

- Reduced cognitive load: single protocol to understand and maintain
- Faster iteration: changes don't require protobuf recompilation
- Simpler testing: standard HTTP testing tools and patterns apply
- Direct debugging: JSON payloads readable without special tooling

### Type Safety Preservation

- Single crate architecture enables shared type definitions
- Compile-time validation remains intact between client and server
- No type information lost compared to gRPC approach

### Future Flexibility

- REST API can be versioned through standard HTTP mechanisms
- Additional protocols can be added later if requirements change
- Browser-based tools become possible without gateway complexity
- Webhook integrations straightforward with standard HTTP

## Alternatives Considered

### gRPC Only

- Advantages: High performance, streaming support, polyglot code generation
- Rejected because: Unnecessary complexity for single Rust client scenario

### tarpc (Rust-native RPC)

- Advantages: Elegant trait-based RPC, no protobuf compilation, excellent type
  safety
- Rejected because: Limited production evidence, binary protocol hinders
  debugging, smaller ecosystem

### Dual Protocol (Original ADR-0007)

- Advantages: Maximum flexibility, best of both worlds
- Rejected because: Maintenance overhead unjustified for single client type

### GraphQL

- Advantages: Flexible querying, self-documenting schema
- Rejected because: Overcomplicated for fixed command-line operations

## Consequences

### Positive

- **Operational Excellence**: Standard HTTP tools enable rapid incident response
- **Debugging Transparency**: JSON payloads inspectable with curl, browser dev
  tools, proxies
- **Ecosystem Compatibility**: Works with existing load balancers, API gateways,
  monitoring tools
- **Reduced Dependencies**: No protobuf compiler, no code generation pipeline
- **Lower Barrier to Entry**: Any developer can understand and debug REST APIs
- **Future Extensibility**: Natural path to browser dashboards or third-party
  integrations

### Negative

- **Performance Overhead**: JSON serialization slower than binary protocols
- **Message Size**: Text-based format larger than binary alternatives
- **Streaming Complexity**: Server-sent events or WebSockets required for
  real-time updates
- **Schema Evolution**: Less formal than protobuf for backward compatibility

### Risk Mitigation

### Performance Concerns

- Single-client scenario makes performance less critical
- Local deployment pattern minimizes network latency
- Can add binary protocol later if performance becomes critical

### Type Safety

- Shared types in single crate preserve compile-time guarantees
- JSON schema validation at runtime catches deserialization errors
- Integration tests verify client-server contract

### Schema Evolution

- Semantic versioning through URL paths or headers
- Careful addition of optional fields maintains compatibility
- Deprecation warnings before breaking changes

## Related Decisions

- ADR-0007: Management API Design - Superseded by this decision
- ADR-0009: CLI Tool Design - Will use REST client instead of gRPC
- ADR-0027: Single Codebase Architecture - Enables type sharing strategy

## References

- Industry best practices favor boring technology for critical infrastructure
- Bryan Cantrill: "The Paradox of Choice in Technology" - simpler is often
  better
- Martin Fowler: "Richardson Maturity Model" - REST API design principles
