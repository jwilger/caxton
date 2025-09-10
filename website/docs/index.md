______________________________________________________________________

## layout: documentation title: Documentation permalink: /docs/

<div align="center">
  <img src="{{ '/assets/img/logo.svg' | relative_url }}"
       alt="Caxton Logo" width="150" height="150">
</div>

Welcome to the comprehensive documentation for Caxton, the production-ready
multi-agent application server.

## What is Caxton?

Caxton is a specialized server for coordinating smart software components called
"agents" - like Redis for caching or Nginx for web serving, but for managing
intelligent automation. You install Caxton, deploy your agents (written in any
language that compiles to WebAssembly), and it handles all the complex
distributed systems challenges: message routing, fault tolerance, observability,
and scaling - all without requiring any external databases or dependencies.

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

- **Customer Service**: Route inquiries between specialist agents (billing,
  technical, sales)
- **Content Processing**: Coordinate agents that analyze, summarize, and
  moderate user-generated content
- **E-commerce**: Have agents handle inventory, recommendations, and fraud
  detection working together
- **Data Analysis**: Pipeline where agents clean data, run analysis, and
  generate reports

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

### Agents

Think of agents as small, focused programs that handle specific tasks. For
example:

- A **billing agent** that processes payments and invoices
- A **notification agent** that sends emails and texts
- A **data agent** that reads from databases and APIs

Each agent runs in its own secure sandbox with configurable memory and CPU
limits, like lightweight containers but even safer.

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

Caxton uses a simplified version of industry-standard agent messaging protocols,
keeping the useful parts (reliable delivery, request tracking) while discarding
academic complexity. See \[ADR-0012\]({{ '/adr/0012-pragmatic-fipa-subset/' |
relative_url }}) for our pragmatic approach.

### Observability

When your agents are working together, you need to see what's happening. Caxton
provides:

- **Distributed tracing**: Follow a customer request across multiple agents
- **Performance metrics**: See which agents are slow or failing
- **Structured logging**: Debug issues with detailed, searchable logs

All built-in from day one using industry-standard OpenTelemetry.

### Tools (MCP Integration)

Agents often need to interact with external systems. Rather than giving agents
direct database access (risky), Caxton provides controlled "tools" they can use:

- Database queries through secure connections
- API calls with rate limiting and error handling
- File system access with proper permissions
- Integration with services like Slack, GitHub, or your internal APIs

This uses the Model Context Protocol (MCP) standard for safe, observable
external interactions.

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
