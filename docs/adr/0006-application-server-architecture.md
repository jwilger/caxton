---
title: "0006. Application Server Architecture"
date: 2025-08-03
status: proposed
layout: adr
categories: [Architecture]
---


Date: 2025-08-03

## Status

Superseded by ADR-0025: Single-Instance Architecture

## Context

Caxton was initially conceived and documented as a Rust library that
developers would integrate into their applications. However, this approach has
fundamental limitations:

- **High barrier to entry**: Requires Rust knowledge, limiting adoption to ~3%
  of developers
- **Integration complexity**: Users must understand Rust's type system and
  async runtime
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

- **Adoption barrier**: Current approach limits users to Rust developers (~3%
  of market)
- **Operational complexity**: Each integration handles infrastructure
  differently
- **Industry patterns**: Successful platforms (Docker, Kubernetes, PostgreSQL)
  follow server model
- **User feedback**: Early adopters struggle with Rust integration requirements
- **Language diversity**: Teams want to write agents in JavaScript, Python,
  Go, etc.

## Decision

We will pivot Caxton from a Rust library to a standalone application server that:

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
- **Operational clarity**: Standard server deployment patterns (systemd,
  Docker, Kubernetes)
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

- **Performance**:
  - Use gRPC for efficient binary protocol
  - Implement connection pooling and multiplexing
  - Target < 1ms API overhead for local deployments
  - Benchmark: 100K+ messages/second on single core

- **Deployment**:
  - Provide Docker images, Helm charts within 30 days
  - Package managers: brew, apt, yum support
  - systemd unit files with proper service management
  - One-line installation scripts

- **Migration**:
  - Clear documentation and automated migration tools
  - Maintain library adapter for 6-month transition period
  - Direct support for early adopters

- **Resources**:
  - Implement cgroup-based resource isolation
  - Publish sizing guidelines: 100 agents/GB RAM baseline
  - CPU quotas and memory limits per agent
  - Automatic resource recommendation engine

- **API stability**:
  - Semantic versioning from v1.0.0
  - 12-month deprecation policy
  - Generated SDKs for major languages
  - gRPC backward/forward compatibility

## Operational Requirements

### State Management

- **Persistence**: Event sourcing for agent state with snapshots
- **Recovery**: Automatic state restoration after crashes
- **Migration**: Zero-downtime state migration during upgrades
- **Backup**: Point-in-time recovery capabilities

### Capacity Planning

- **Scaling metrics**: Agents per server, memory per agent, messages/second
- **Resource isolation**: cgroup v2 integration for hard limits
- **Horizontal scaling**: Consistent hashing for agent distribution
- **Vertical scaling**: Hot-reload configuration for resource adjustments

### High Availability

- **Active-passive**: Automatic failover with < 30s RTO
- **Health checks**: L4 (TCP), L7 (HTTP), and business logic health
- **Load balancing**: Built-in support for HAProxy, nginx, cloud LBs
- **Split-brain prevention**: Consensus-based leader election

### Security Operations

- **Authentication**: mTLS, API keys, OIDC/OAuth2 integration
- **Authorization**: RBAC with per-agent permissions
- **Audit logging**: Structured logs for all API access
- **Secrets management**: Integration with Vault, K8s secrets

## Deployment Models

### systemd Service

```ini
[Unit]
Description=Caxton Multi-Agent Orchestration Server
After=network.target

[Service]
Type=notify
ExecStart=/usr/bin/caxton server --config /etc/caxton/config.yaml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### Docker Container

```dockerfile
FROM caxton/caxton:latest
EXPOSE 8080 9090
HEALTHCHECK CMD caxton health
```

### Kubernetes Deployment

- StatefulSet for persistent agent state
- Service for load balancing
- ConfigMap for configuration
- PersistentVolumeClaim for state storage

## Comparison with Familiar Servers

- **Like PostgreSQL**: Persistent state, backup/restore, replication,
  connection pooling
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
