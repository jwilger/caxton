# Caxton Documentation

Welcome to the Caxton documentation! Caxton is a production-ready server that
orchestrates multi-agent systems.

## Documentation Structure

### 📚 Getting Started

- [Installation Guide](getting-started/installation.md) - Install Caxton on your
  system
- [Quick Start](getting-started/quickstart.md) - Get agents running in 3 minutes
- [First Agent Tutorial](getting-started/first-agent.md) - Build your first
  WebAssembly agent
- [Configuration Guide](getting-started/configuration.md) - Configure Caxton for
  your needs

### 👤 User Guide

- [CLI Reference](user-guide/cli-reference.md) - Complete command-line
  documentation
- [Dashboard Guide](user-guide/dashboard.md) - Web dashboard overview
- [Deployment Guide](user-guide/deployment.md) - Deploy agents and manage
  versions
- [Monitoring Guide](user-guide/monitoring.md) - Monitor agents and system
  health
- [Clustering Guide](user-guide/clustering.md) - Multi-node cluster
  configuration

### ⚙️ Operations Guide

- [Agent Lifecycle Management](operations/agent-lifecycle-management.md) -
  Deploy, manage, and maintain agents
- [Performance Tuning](operations/performance-tuning.md) - Optimize system
  performance
- [Security Guide](operations/devops-security-guide.md) - Security best
  practices
- [Operational Runbook](operations/operational-runbook.md) - Troubleshooting and
  maintenance

### 🛠️ Developer Guide

- [API Reference](developer-guide/api-reference.md) - Complete API documentation
- [Building Agents](developer-guide/building-agents.md) - Agent development in
  depth
- [Message Protocols](developer-guide/message-protocols.md) - Agent messaging protocol
  implementation
- [WebAssembly Integration](developer-guide/wasm-integration.md) - WASM module
  details
- [Security Guide](developer-guide/security-guide.md) - Security features and
  best practices
- [Testing Guide](developer-guide/testing.md) - Testing agents and integrations
- [Testing Strategy](development/testing-strategy.md) - Comprehensive testing
  approach

### 🚀 Operations

- [Production Deployment](operations/production-deployment.md) - Deploy Caxton
  to production
- [DevOps & Security Guide](operations/devops-security-guide.md) - Security best
  practices
- [Operational Runbook](operations/operational-runbook.md) - Step-by-step
  operational procedures
- [Performance Tuning](operations/performance-tuning.md) - Optimize Caxton
  performance
- [Kubernetes Guide](operations/kubernetes.md) - Run Caxton on Kubernetes
- [Docker Guide](operations/docker.md) - Container deployment
- [Troubleshooting](operations/troubleshooting.md) - Common issues and solutions

### 🏗️ Architecture

- [Coordination-First Overview](architecture/coordination-first-overview.md) -
  Complete architecture guide
- [Architecture Decision Records](adr/) - Key architectural decisions
  - [ADR-0001: Observability First](adr/0001-observability-first-architecture.md)
  - [ADR-0002: WebAssembly Isolation](adr/0002-webassembly-for-agent-isolation.md)
  - [ADR-0003: Agent Messaging](adr/0003-fipa-messaging-protocol.md)
  - [ADR-0004: Minimal Core](adr/0004-minimal-core-philosophy.md)
  - [ADR-0005: MCP Integration](adr/0005-mcp-for-external-tools.md)
  - [ADR-0006: Application Server](adr/0006-application-server-architecture.md)
  - [ADR-0007: Management API](adr/0007-management-api-design.md)
  - [ADR-0008: Agent Deployment](adr/0008-agent-deployment-model.md)
  - [ADR-0009: CLI Design](adr/0009-cli-tool-design.md)
  - [ADR-0010: External Routing](adr/0010-external-agent-routing-api.md)
  - [ADR-0011: Capability Registration](adr/0011-capability-registration-in-code.md)
  - [ADR-0012: Pragmatic Agent Messaging](adr/0012-pragmatic-fipa-subset.md)
  - [ADR-0013: State Management](adr/0013-state-management-architecture.md)
    *(Superseded)*
  - [ADR-0014: Coordination-First Architecture](adr/0014-coordination-first-architecture.md)
  - [ADR-0015: Distributed Protocol Architecture](adr/0015-distributed-protocol-architecture.md)
  - [ADR-0016: Security Architecture](adr/0016-security-architecture.md)
  - [ADR-0017: Performance Requirements](adr/0017-performance-requirements.md)
  - [ADR-0018: Operational Procedures](adr/0018-operational-procedures.md)

### 🔧 Development

- [CLAUDE.md](development/CLAUDE.md) - Claude Code integration
- [Coordination](development/coordination.md) - Task coordination
- [Memory Bank](development/memory-bank.md) - Persistent context

## Quick Links

- **Main README**: [/README.md](../README.md)
- **Architecture**: [/ARCHITECTURE.md](../ARCHITECTURE.md)
- **Contributing**: [/CONTRIBUTING.md](../CONTRIBUTING.md)
- **Roadmap**: [/ROADMAP.md](../ROADMAP.md)
- **Security**: [/SECURITY.md](../SECURITY.md)
- **Changelog**: [/CHANGELOG.md](../CHANGELOG.md)

## Getting Help

- **GitHub Issues**:
  [Report bugs or request features](https://github.com/caxton/caxton/issues)
- **Discussions**:
  [Ask questions and share ideas](https://github.com/caxton/caxton/discussions)
- **Discord**: [Join our community](https://discord.gg/caxton)

## Documentation Conventions

### Code Examples

- Examples are provided in multiple languages where applicable
- All examples are tested and working with the latest Caxton version
- Replace placeholder values (like `your-api-key`) with actual values

### Version Compatibility

- Documentation is for Caxton v1.0+ unless otherwise noted
- Breaking changes are clearly marked
- Legacy documentation is available in version branches

### Contributing to Docs

- See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines
- Documentation uses Markdown with GitHub Flavored Markdown extensions
- Run `caxton docs validate` to check documentation
