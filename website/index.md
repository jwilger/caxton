---
title: "Caxton - Configuration-Driven Agent Platform"
description: "5-10 minute onboarding to multi-agent systems. Zero external dependencies, embedded memory, hybrid agent architecture."
layout: home
permalink: /
date: 2025-09-10
categories: [Website]
---

## Configuration-Driven Agents for Rapid Development

**Get your first agent running in 5-10 minutes** with Caxton's
configuration-first architecture. No compilation, no external dependencies, no
hours of toolchain setup.

### Why Caxton?

#### Configuration-First Experience

- Define agents in markdown files with YAML frontmatter
- Zero compilation required for 90% of use cases
- Edit and test changes immediately
- Version-controllable, shareable agent definitions

#### Zero External Dependencies

- Embedded SQLite + local embedding models for memory
- No PostgreSQL, Neo4j, or Qdrant setup required
- Works immediately out of the box
- Single binary deployment

### Hybrid Architecture for All Use Cases

- **Configuration agents**: Perfect for orchestration and tool coordination
- **WebAssembly agents**: Available when you need custom algorithms
- Security isolation where it matters most
- Clear upgrade path as needs evolve

### Quick Start

Create your first agent in under 5 minutes:

```yaml
---
name: DataAnalyzer
capabilities:
  - data-analysis
  - report-generation
tools:
  - http_client
  - csv_parser
system_prompt: |
  You are a data analysis expert who helps users understand their data.
---

# DataAnalyzer Agent

Ask me to analyze CSV files, create charts, or summarize datasets.
```

Save as `data-analyzer.md`, deploy with one command, and start using immediately.

### Production Ready

- **FIPA-ACL messaging** for interoperable agent communication
- **Capability-based routing** for loose coupling
- **Built-in observability** with structured logging and metrics
- **WebAssembly isolation** for security-critical operations
- **Hot-reload development** for rapid iteration

### For Every Scale

- **Individuals**: Start with embedded memory, zero configuration
- **Teams**: Shared workspaces with collaborative agent development
- **Enterprises**: Scale to external backends (Neo4j, Qdrant) as needed

**Ready to build agents in minutes, not hours?** [Get Started →](docs/)
