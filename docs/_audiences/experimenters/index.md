---
title: "For Experimenters: Try Caxton in 10 Minutes"
date: 2025-01-15
layout: page
categories: [Audiences, Experimenters]
audience: experimenters
description: "New to Caxton? Get a working multi-agent system in 10 minutes with zero dependencies."
---

## Welcome, Experimenter

You're here because you want to **try Caxton quickly** without heavy setup. Good
news: Caxton is designed for exactly this experience. You can have intelligent
agents communicating with each other in **under 10 minutes**.

## What You'll Achieve

By the end of this path, you'll have:

- ✅ A running Caxton server (zero external dependencies)
- ✅ Your first configuration agent deployed from a simple markdown file
- ✅ Agents communicating via capability-based messaging
- ✅ Built-in memory system learning from interactions
- ✅ Understanding of when to use config agents vs WASM agents

## Why Caxton is Perfect for Experimentation

### 5-10 Minute Onboarding

Unlike platforms requiring hours of toolchain setup:

- **No compilation**: Agents are markdown files with YAML configuration
- **Zero dependencies**: SQLite + local embeddings work out-of-the-box
- **Instant deployment**: Hot-reload config agents without restart
- **Visual feedback**: Web dashboard shows agent communication in real-time

### Capability-Based Architecture

Your agents don't need to know about each other:

- Request **"data-analysis"** capability, not a specific agent
- Load balancing and routing happens automatically
- Easy to add more agents with the same capabilities
- Natural multi-agent workflow patterns emerge

## Quick Start Path (Choose Your Speed)

### ⚡ Ultra-Quick (5 minutes)

Perfect for busy engineers who want to see it working immediately.

1. **[Installation](../../getting-started/installation.md)** (2 min)
   - Single binary download, zero configuration needed

2. **[Quick Start Guide](../../getting-started/quickstart.md)** (3 min)
   - One command server start + one agent deployment
   - See agent communication in action

### 🚀 Standard Path (10 minutes)

Best balance of speed and understanding.

1. **[Installation](../../getting-started/installation.md)** (2 min)
2. **[Quick Start Guide](../../getting-started/quickstart.md)** (5 min)
3. **[Create Your First Agent](../../getting-started/first-agent.md)** (3 min)
   - Understand the YAML configuration format
   - See how agents use tools and memory

### 🔬 Deep Exploration (30 minutes)

For when you want to understand the full capabilities.

1. Complete the Standard Path (10 min)
2. **[Configuration Reference](../../getting-started/configuration.md)** (10 min)
   - Full YAML schema and options
   - Memory scopes and tool integration
3. **[REST API Quickstart](../../getting-started/rest-api-quickstart.md)** (10 min)
   - Programmatic interaction with agents
   - Integration patterns for your applications

## Key Concepts (5-Minute Read)

### Configuration Agents

Your agents are **markdown files** with YAML frontmatter:

```yaml
---
name: DataHelper
capabilities: [data-analysis]
tools: [http_client, csv_parser]
memory:
  enabled: true
  scope: workspace
---

# DataHelper Agent

I help analyze data from various sources!
```

That's it. Deploy with `caxton agent deploy data-helper.md`.

### Capability-Based Messaging

Instead of `send_message_to_agent("DataHelper", request)`:

```bash
caxton message send \
  --capability "data-analysis" \
  --content "Analyze sales trends in Q3 data"
```

Caxton finds the right agent automatically.

### Built-in Learning

Agents remember successful interactions:

- Pattern recognition across datasets
- Context building over time
- Zero configuration memory with SQLite + embeddings

### Zero Dependencies by Default

- **Embedded memory**: SQLite + All-MiniLM-L6-v2 embedding model (~23MB)
- **Scales to 100K+ entities** without external databases
- **Migration path available** to Neo4j/Qdrant for larger deployments

## What's Different About Caxton?

### vs. Traditional Agent Frameworks

| Traditional | Caxton |
|-------------|---------|
| Hours of toolchain setup | 5-10 minute onboarding |
| Agent-to-agent coupling | Capability-based routing |
| External databases required | Zero-dependency embedded memory |
| Complex WASM compilation | Markdown + YAML configuration |

### vs. LangChain/CrewAI

| Framework Approach | Caxton Approach |
|-------------------|-----------------|
| Python orchestration scripts | Server process with REST API |
| No built-in persistence | Embedded memory with semantic search |
| Manual agent coordination | FIPA-standard message protocols |
| Development-only tools | Production-ready with observability |

## Common Experiment Patterns

### Single Agent Exploration

Start with one capability to understand the basics:

```yaml
---
name: WebResearcher
capabilities: [research]
tools: [http_client]
system_prompt: |
  You help users research topics by finding information on the web.
---
```

### Multi-Agent Workflow

Create agents that work together automatically:

```yaml
# Agent 1: Fetches data
name: DataFetcher
capabilities: [data-ingestion]

# Agent 2: Analyzes data
name: DataAnalyzer
capabilities: [data-analysis]

# Agent 3: Creates reports
name: ReportGenerator
capabilities: [report-generation]
```

Send one request: Caxton orchestrates the entire workflow.

### Learning Behavior Exploration

Watch agents improve over time:

```bash
# Send similar requests and watch responses improve
caxton message send --capability "data-analysis" \
  --content "Analyze monthly sales trends"

# Check what the agent learned
caxton memory search "sales trends analysis"
```

## When to Graduate to Advanced Features

### Stick with Config Agents When

- ✅ Orchestrating LLM calls with tools
- ✅ Combining existing services and APIs
- ✅ Building business workflows
- ✅ Rapid prototyping and experimentation
- ✅ 90% of use cases

### Consider WASM Agents When

- 🔧 Custom algorithms needed
- 🔧 Performance-critical computation
- 🔧 Legacy code integration
- 🔧 Proprietary algorithms
- 🔧 Maximum security isolation

### Consider External Backends When

- 📊 More than 100K entities in memory
- 📊 Multi-instance deployments
- 📊 Advanced graph analytics needed
- 📊 Enterprise compliance requirements

## Experiment Ideas

### 🧪 Personal Productivity

- **Email Responder**: Analyzes emails and suggests responses
- **Meeting Summarizer**: Processes meeting transcripts
- **Task Prioritizer**: Sorts your TODO list by importance

### 🧪 Data Exploration

- **CSV Analyzer**: Automatically finds trends in uploaded data
- **API Monitor**: Watches API endpoints and reports changes
- **Log Analyzer**: Processes application logs for insights

### 🧪 Business Workflows

- **Customer Support**: Triages support tickets automatically
- **Content Moderator**: Reviews user-generated content
- **Inventory Tracker**: Monitors stock levels and reorders

### 🧪 Multi-Agent Coordination

- **Research Team**: One agent searches, another summarizes, third creates reports
- **Data Pipeline**: Ingestion → Processing → Analysis → Visualization
- **Customer Journey**: Lead qualification → Follow-up → Conversion tracking

## Troubleshooting Your Experiments

### Agent Won't Deploy

```bash
# Check configuration syntax
caxton validate my-agent.md

# Common issues:
# - Invalid YAML syntax in frontmatter
# - Missing required fields (name, capabilities)
# - Unknown tools in tools list
```

### Agent Not Responding

```bash
# Check agent status
caxton agents list

# Check logs
caxton logs MyAgent --tail 20

# Test capability routing
caxton capabilities list
```

### Performance Issues

```bash
# Check memory system status
caxton memory status

# Optimize if needed
caxton memory optimize
```

### Need Help?

```bash
# Built-in diagnostics
caxton doctor

# Community resources
caxton help --examples
```

## Next Steps After Experimenting

Once you've proven Caxton works for your use case:

### For Building Production Agents

→ **[Agent Developers Landing Page](../agent-developers/index.md)**

- Advanced configuration patterns
- Tool integration and security
- Testing and deployment strategies
- Performance optimization

### For Production Deployment

→ **[Operators Landing Page](../operators/index.md)**

- Production deployment patterns
- Monitoring and observability
- Backup and recovery procedures
- Scaling strategies

### For Contributing to Caxton

→ **[Contributors Landing Page](../contributors/index.md)**

- Development environment setup
- Architecture deep dives
- Testing and CI/CD workflows
- Code contribution guidelines

## Community and Support

- **GitHub Discussions**: Share experiments and get help
- **Examples Repository**: Community-contributed agent configurations
- **Discord**: Real-time chat with other Caxton users
- **Documentation**: Comprehensive guides and API references

---

**Ready to start?** Jump into the **[Quick Start Guide](../../getting-started/quickstart.md)**
and have your first agent running in 5 minutes!
