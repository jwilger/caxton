---
layout: documentation
title: Documentation
permalink: /docs/
---

# Caxton Documentation

Welcome to the comprehensive documentation for Caxton, the production-ready multi-agent application server.

## What is Caxton?

Caxton is a multi-agent orchestration server - like Redis for caching or PostgreSQL for data, but for coordinating intelligent agents. You install Caxton, deploy your agents (written in any language that compiles to WebAssembly), and it handles all the complex distributed systems challenges: message routing, fault tolerance, observability, and scaling.

## Quick Links

<div class="docs-quick-links">
    <a href="{{ '/docs/getting-started/quickstart/' | relative_url }}" class="quick-link-card">
        <span class="icon">🚀</span>
        <h3>Quick Start</h3>
        <p>Get up and running with Caxton in 5 minutes</p>
    </a>

    <a href="{{ '/docs/getting-started/first-agent/' | relative_url }}" class="quick-link-card">
        <span class="icon">🤖</span>
        <h3>Build Your First Agent</h3>
        <p>Step-by-step guide to creating and deploying agents</p>
    </a>

    <a href="{{ '/docs/developer-guide/api-reference/' | relative_url }}" class="quick-link-card">
        <span class="icon">📖</span>
        <h3>API Reference</h3>
        <p>Complete REST and gRPC API documentation</p>
    </a>

    <a href="{{ '/adr/' | relative_url }}" class="quick-link-card">
        <span class="icon">🏛️</span>
        <h3>Architecture Decisions</h3>
        <p>Understand the design choices behind Caxton</p>
    </a>
</div>

## Core Concepts

### Agents
WebAssembly-isolated components that process messages and make decisions. Each agent runs in its own sandbox with configurable resource limits.

### Messages
FIPA-compliant messages that agents exchange. Caxton handles routing, delivery guarantees, and protocol state machines.

### Observability
Built-in OpenTelemetry support provides distributed tracing, metrics, and structured logging from day one.

### Tools
MCP (Model Context Protocol) bridges allow agents to interact with external systems, databases, and APIs.

## Documentation Structure

### Getting Started
- **[Quick Start]({{ '/docs/getting-started/quickstart/' | relative_url }})** - Get Caxton running in minutes
- **[Installation]({{ '/docs/getting-started/installation/' | relative_url }})** - Detailed installation instructions for all platforms
- **[Your First Agent]({{ '/docs/getting-started/first-agent/' | relative_url }})** - Build and deploy your first agent
- **[Configuration]({{ '/docs/getting-started/configuration/' | relative_url }})** - Server and agent configuration reference

### Developer Resources
- **[API Reference]({{ '/docs/developer-guide/api-reference/' | relative_url }})** - REST and gRPC API documentation
- **[Rust API Docs](https://docs.rs/caxton/latest/caxton/)** - Complete Rust API reference on docs.rs

### Operations & Production
- **[DevOps & Security Guide]({{ '/docs/operations/devops-security-guide/' | relative_url }})** - Production deployment, monitoring, and security best practices

### Architecture & Design
- **[Architecture Overview]({{ '/docs/ARCHITECTURE/' | relative_url }})** - System architecture and design principles
- **[Architecture Decision Records]({{ '/adr/' | relative_url }})** - Detailed documentation of architectural choices
- **[Roadmap]({{ '/docs/ROADMAP/' | relative_url }})** - Project roadmap and planned features

### Contributing
- **[Contributing Guide]({{ '/docs/CONTRIBUTING/' | relative_url }})** - How to contribute to Caxton
- **[Security Policy]({{ '/docs/SECURITY/' | relative_url }})** - Security guidelines and vulnerability reporting

## Need Help?

<div class="help-section">
    <div class="help-card">
        <h3>💬 Community Support</h3>
        <p>Join our <a href="{{ site.social.github }}/discussions" target="_blank" rel="noopener">GitHub Discussions</a> to ask questions and share experiences</p>
    </div>

    <div class="help-card">
        <h3>🐛 Report Issues</h3>
        <p>Found a bug? Report it on our <a href="{{ site.social.github }}/issues" target="_blank" rel="noopener">Issue Tracker</a></p>
    </div>

    <div class="help-card">
        <h3>📺 Examples</h3>
        <p>Check out example agents and use cases in our <a href="{{ site.social.github }}/tree/main/examples" target="_blank" rel="noopener">examples directory</a></p>
    </div>
</div>
