# Caxton Architecture

This section contains comprehensive architectural documentation for
Caxton's multi-agent orchestration system.

## Overview

Caxton follows a coordination-first architecture with WebAssembly isolation,
FIPA-compliant messaging, and embedded observability. The system emphasizes
security, performance, and operational simplicity.

## Architecture Documentation

### Core Architecture

- **[Coordination-First Overview](coordination-first-overview.md)** -
  High-level architectural philosophy and design
- **[Component Interaction Diagrams](component-interaction-diagrams.md)** -
  Visual system component relationships
- **[Configuration and Deployment](configuration-and-deployment.md)** -
  System configuration patterns and deployment models

### Message Routing & Communication

- **[Message Router](message-router.md)** - Core message routing implementation
- **[Message Router Specification](message-router-specification.md)** -
  Detailed router specifications and protocols
- **[Routing Algorithm Design](routing-algorithm-design.md)** -
  Routing algorithm implementation details

### State & Memory Management

- **[State Management](state-management.md)** - State handling patterns and implementation
- **[State Alternatives](state-alternatives.md)** -
  Research on lightweight state management alternatives

### WebAssembly & Runtime

- **[WebAssembly Ecosystem](webassembly-ecosystem.md)** -
  WebAssembly integration and ecosystem guide

## Architecture Decision Records (ADRs)

For detailed architectural decisions and their rationale, see the [ADRs directory](../adrs/).

Key architectural decisions include:

- [ADR-0001: Observability First](../adrs/0001-observability-first-architecture.md)
- [ADR-0002: WebAssembly Isolation](../adrs/0002-webassembly-for-agent-isolation.md)
- [ADR-0028: Configuration-Driven Agents](../adrs/0028-configuration-driven-agent-architecture.md)
- [ADR-0029: FIPA-ACL Messaging](../adrs/0029-fipa-acl-lightweight-messaging.md)
- [ADR-0030: Embedded Memory System](../adrs/0030-embedded-memory-system.md)

## Related Documentation

- **[Getting Started](../getting-started/)** - Quick setup and first agent
- **[API Documentation](../api/)** - REST API and integration guides
- **[Operations](../operations/)** - Production deployment and monitoring
- **[Concepts](../concepts/)** - Deep dives into messaging and memory systems
- **[Main Documentation](../README.md)** - Back to docs overview
