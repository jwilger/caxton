---
title: "Documentation"
layout: documentation
permalink: /docs/
date: 2025-09-10
categories: [Website]
---

<div align="center">
  <img src="{{ '/assets/img/logo.svg' | relative_url }}"
       alt="Caxton Logo" width="150" height="150">
</div>

Welcome to the comprehensive documentation for Caxton, the configuration-driven
agent platform.

## What is Caxton?

Caxton is a hybrid agent platform designed for rapid development and
production deployment. Unlike traditional platforms that require hours of
compilation setup, Caxton lets you **create and deploy agents in 5-10
minutes** using simple markdown configuration files.

**Two Agent Types, One Platform:**

- **Configuration agents**: Define behavior in markdown files with YAML
  frontmatter (90% of use cases)
- **WebAssembly agents**: Compile custom algorithms when you need maximum
  performance (10% of use cases)

**Zero External Dependencies:** Embedded SQLite memory, local embedding models,
and single-binary deployment mean Caxton works immediately out of the box - no
PostgreSQL, Neo4j, or complex infrastructure required.

## What problems does Caxton solve?

**Building intelligent automation is hard.** Here are common challenges Caxton
addresses:

### Coordinating Multiple AI Systems

- **Problem**: You have different AI models (LLMs, vision models, specialized
  algorithms) that need to work together
- **Solution**: Caxton routes messages between agents, so your GPT-4 agent can
  request image analysis from your vision agent

### Scaling Smart Applications

- **Problem**: Your AI application works great for one user, but breaks under
  load or when processing multiple requests
- **Solution**: Caxton handles resource management, queuing, and scaling so your
  agents stay responsive

### Integrating with Existing Systems

- **Problem**: Your AI agents need to read databases, call APIs, send emails, or
  interact with other services
- **Solution**: Caxton's tool system (MCP bridges) lets agents safely interact
  with external systems

### Making AI Systems Observable

- **Problem**: When your AI application misbehaves, you can't see what went
  wrong or why
- **Solution**: Caxton provides built-in tracing and monitoring so you can debug
  distributed AI workflows

### Real-World Examples

**Configuration Agent Use Cases** (5-10 minute setup):

- **Customer Service**: Natural language routing between specialist capabilities
- **Content Processing**: Orchestrate analysis, summarization, and moderation workflows
- **Data Analysis**: Chain CSV parsing, statistical analysis, and report
  generation
- **Team Automation**: Coordinate Slack notifications, GitHub updates, and email
  workflows

**WebAssembly Agent Use Cases** (for power users):

- **Custom ML Models**: Deploy proprietary algorithms with security isolation
- **High-Performance Computing**: CPU-intensive mathematical operations
- **Legacy Integration**: Wrap existing C/C++/Rust libraries as agents

## Quick Links

<div class="docs-quick-links">
    <a href="{{ '/docs/getting-started/quickstart/' | relative_url }}" class="quick-link-card">
        <span class="icon">🚀</span>
        <h3>Quick Start</h3>
        <p>Get up and running with Caxton in 5 minutes</p>
    </a>

```html
<a href="{{ '/docs/getting-started/first-agent/' | relative_url }}" class="quick-link-card">
    <span class="icon">🤖</span>
    <h3>Build Your First Agent</h3>
    <p>Step-by-step guide to creating and deploying agents</p>
</a>

<a href="{{ '/docs/developer-guide/api-reference/' | relative_url }}" class="quick-link-card">
    <span class="icon">📖</span>
    <h3>API Reference</h3>
    <p>Complete REST API documentation</p>
</a>

<a href="{{ '/adr/' | relative_url }}" class="quick-link-card">
    <span class="icon">🏛️</span>
    <h3>Architecture Decisions</h3>
    <p>Understand the design choices behind Caxton</p>
</a>
```

</div>

## Core Concepts

### Configuration Agents

**Primary Agent Type (90% of use cases)**: Define agent behavior in markdown
files with YAML frontmatter. No compilation required.

```yaml
---
name: BillingAgent
capabilities: [billing, invoices]
tools: [database, email]
system_prompt: |
  You process payments and generate invoices.
---
```

**WebAssembly Agents**: When you need custom algorithms or maximum
performance, agents can still be compiled modules running in secure sandboxes
with memory/CPU limits.

### Messages

Agents communicate by sending structured messages to each other, similar to REST
API calls but between your own components. For example:

```json
{
  "type": "request",
  "from": "customer-service",
  "to": "billing-agent",
  "content": "Please create invoice for order #12345"
}
```

Caxton uses **capability-based routing** with lightweight FIPA-ACL messaging.
Agents request capabilities (like "data-analysis") rather than specific agents,
enabling loose coupling and easier scaling. See
[ADR-0029]({{ '/adr/0029-fipa-acl-lightweight-messaging/' | relative_url }})
for our configuration-friendly approach.

### Observability

When your agents are working together, you need to see what's happening. Caxton
provides:

- **Distributed tracing**: Follow a customer request across multiple agents
- **Performance metrics**: See which agents are slow or failing
- **Structured logging**: Debug issues with detailed, searchable logs

All built-in from day one using industry-standard OpenTelemetry.

### Memory and Tools

**Embedded Memory System**: Configuration agents get persistent memory out of
the box using embedded SQLite + local embedding models. No external database
setup required. Memory enables agents to:

- Remember past conversations and solutions
- Store and retrieve relevant context automatically
- Learn patterns from successful interactions

**MCP Tools**: Agents interact with external systems through secure, observable tools:

- Database queries with proper permissions
- API calls with built-in rate limiting
- File system access within security boundaries
- Integration with Slack, GitHub, and internal services

Memory scales from embedded (instant setup) to external backends (Neo4j,
Qdrant) as needed. See [ADR-0030]({{ '/adr/0030-embedded-memory-system/' |
relative_url }}) for the hybrid architecture.

## Documentation Structure

### Getting Started

- **\[Quick Start\]({{ '/docs/getting-started/quickstart/' | relative_url }})**
  \- Get Caxton running in minutes
- **\[Installation\]({{ '/docs/getting-started/installation/' | relative_url
  }})** - Detailed installation instructions for all platforms
- **\[Your First Agent\]({{ '/docs/getting-started/first-agent/' | relative_url
  }})** - Build and deploy your first agent
- **\[Configuration\]({{ '/docs/getting-started/configuration/' | relative_url
  }})** - Server and agent configuration reference

### Developer Resources

- **\[API Reference\]({{ '/docs/developer-guide/api-reference/' | relative_url
  }})** - REST API documentation
- **[Rust API Docs](https://docs.rs/caxton/latest/caxton/)** - Complete Rust API
  reference on docs.rs

### Operations & Production

- **\[DevOps & Security Guide\]({{ '/docs/operations/devops-security-guide/' |
  relative_url }})** - Production deployment, monitoring, and security best
  practices

### Architecture & Design

- **\[Architecture Overview\]({{ '/docs/ARCHITECTURE/' | relative_url }})** -
  System architecture and design principles
- **\[Architecture Decision Records\]({{ '/adr/' | relative_url }})** - Detailed
  documentation of architectural choices
- **\[Roadmap\]({{ '/docs/ROADMAP/' | relative_url }})** - Project roadmap and
  planned features

### Contributing

- **\[Contributing Guide\]({{ '/docs/CONTRIBUTING/' | relative_url }})** - How
  to contribute to Caxton
- **\[Security Policy\]({{ '/docs/SECURITY/' | relative_url }})** - Security
  guidelines and vulnerability reporting

## Need Help?

<div class="help-section">
    <div class="help-card">
        <h3>💬 Community Support</h3>
        <p>Join our <a href="{{ site.social.github }}/discussions" target="_blank"
           rel="noopener">GitHub Discussions</a> to ask questions and share
           experiences</p>
    </div>

```html
<div class="help-card">
    <h3>🐛 Report Issues</h3>
    <p>Found a bug? Report it on our <a href="{{ site.social.github }}/issues" target="_blank" rel="noopener">Issue Tracker</a></p>
</div>

<div class="help-card">
    <h3>📺 Examples</h3>
    <p>Check out example agents and use cases in our <a href="{{ site.social.github }}/tree/main/examples" target="_blank" rel="noopener">examples directory</a></p>
</div>
```

</div>
