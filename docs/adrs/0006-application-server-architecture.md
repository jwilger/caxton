---
title: "ADR-0006: Application Server"
date: 2025-08-03
status: accepted
layout: adr
categories: [Architecture]
---


## Status

Accepted

## Context

Caxton was initially conceived and documented as a Rust library that developers
would integrate into their applications. However, this approach has fundamental
limitations:

- **High barrier to entry**: Requires Rust knowledge, limiting adoption to ~3%
  of developers
- **Integration complexity**: Users must understand Rust's type system and async
  runtime
- **Deployment challenges**: Every application needs to embed agent
  orchestration logic
- **Operational burden**: Each integration handles scaling, monitoring, and
  updates differently
- **Limited language support**: Agent developers are constrained by Rust FFI
  capabilities

The multi-agent systems community needs infrastructure that "just works" -
similar to how developers use PostgreSQL or Redis without needing to understand
their implementation languages.

## Decision Drivers

- **Adoption barrier**: Current approach limits users to Rust developers (~3% of
  market)
- **Operational complexity**: Each integration handles infrastructure
  differently
- **Industry patterns**: Successful platforms (Docker, Kubernetes, PostgreSQL)
  follow server model
- **User feedback**: Early adopters struggle with Rust integration requirements
- **Language diversity**: Teams want to write agents in JavaScript, Python, Go,
  etc.

## Decision

We will pivot Caxton from a Rust library to a standalone application server
that:

1. **Runs as an independent process** - Like PostgreSQL, Redis, or Kubernetes
   API server
2. **Provides management APIs** - gRPC (primary) and REST (gateway) for
   programmatic control
3. **Includes a CLI tool** - For operational tasks and debugging
4. **Requires zero Rust knowledge** - Users never see or write Rust code
5. **Supports any WebAssembly language** - JavaScript, Python, Go, Rust, etc.

## Consequences

### Positive

- **Accessibility**: Any developer can use Caxton regardless of language
  expertise
- **Operational clarity**: Standard server deployment patterns (systemd, Docker,
  Kubernetes)
- **Language freedom**: Write agents in any language that compiles to
  WebAssembly
- **Centralized management**: Single point for monitoring, scaling, and updates
- **Clear boundaries**: Server/client separation simplifies mental model
- **Enterprise ready**: Fits existing infrastructure and deployment pipelines
- **Advanced deployment**: Enables canary deployments, A/B testing, feature
  flags

### Negative

- **Network overhead**: API calls instead of in-process function calls (~1ms
  local latency)
- **Deployment complexity**: Users must run and manage another service
- **Breaking change**: Existing library users must migrate (minimal impact as
  project is early-stage)
- **Resource requirements**: Dedicated server process needs CPU/memory
  allocation
- **API versioning**: Must maintain backward compatibility as we evolve
- **State management**: Need strategy for agent state persistence and recovery

### Mitigation Strategies

- **Performance**: Efficient binary protocols and connection management to
  minimize API overhead
- **Deployment**: Standard packaging and installation methods for common
  platforms
- **Migration**: Transition support and compatibility layers for existing
  integrations
- **Resources**: Isolation and limits to ensure predictable resource usage
- **API stability**: Versioning and backward compatibility policies

## Operational Considerations

As a standalone server, Caxton must address typical enterprise operational
requirements including state persistence, scaling strategies, high availability
patterns, and security policies. These operational aspects will be implemented
following standard server deployment patterns.

## Deployment Models

Caxton will support standard deployment patterns:

- **System service**: Native OS service management (systemd, etc.)
- **Container**: Docker and OCI-compatible container deployment
- **Orchestrated**: Kubernetes and similar container orchestration platforms

Each deployment method follows established patterns from comparable server
applications.

## Comparison with Familiar Servers

- **Like PostgreSQL**: Persistent state, backup/restore, replication, connection
  pooling
- **Like Redis**: In-memory performance, pub/sub patterns, Lua scripting (via
  WASM)
- **Like Kubernetes**: API-driven, declarative configuration, hot reload,
  controllers
- **Like Docker daemon**: REST/gRPC API, CLI tool, SDKs, daemon mode

## Related Decisions

- ADR-0007: Management API Design - Defines the gRPC/REST API architecture
- ADR-0008: Agent Deployment Model - How agents are deployed to the server
- ADR-0009: CLI Tool Design - User interface for server management

## References

- Industry examples: Docker daemon, Kubernetes API server, PostgreSQL, Redis
- CNCF project structures and deployment patterns
- [The Twelve-Factor App](https://12factor.net/) methodology
- Original library-focused documentation (now deprecated)
