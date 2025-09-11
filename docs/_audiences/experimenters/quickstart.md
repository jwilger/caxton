---
title: "Quick Start Guide"
date: 2025-09-10
layout: page
audience: experimenters
navigation_order: 1
categories: [Experimenters, Getting Started]
---

> **🚧 Implementation Status**
>
> This quick start guide represents the target user experience and serves
> as acceptance criteria for development. The configuration-driven agent
> workflow shown here is being implemented according to ADR-28 architectural
> decisions.
>
> **Target Experience**: 5-10 minute agent creation and deployment
> **Current State**: Domain modeling and architecture foundation in progress

## Get a multi-agent system running in under 10 minutes

This guide uses **configuration agents** - the simplest way to create
intelligent agents in Caxton. No compilation, no complex toolchains, just
markdown files with YAML configuration.

Perfect for researchers, hobbyists, and anyone wanting to experiment with
multi-agent systems without infrastructure complexity.

## Prerequisites

- Caxton installed ([Installation Guide](../operators/installation.md))
- Text editor
- Basic command-line familiarity

## 1. Start Caxton Server

```bash
# Start the server with zero configuration
caxton server start

# Verify it's running
curl http://localhost:8080/api/v1/health
```

You should see:

```json
{"status":"healthy","memory_backend":"embedded","agents_count":0}
```

> **Zero Dependencies**: Caxton runs with no external dependencies. The
> embedded memory system uses SQLite + local embeddings for semantic search,
> and scales to 100K+ entities without additional setup.

## 2. Create Your First Configuration Agent

Create a file called `data-analyzer.md`:

```yaml
---
name: DataAnalyzer
version: "1.0.0"
capabilities:
  - data-analysis
  - report-generation
tools:
  - http_client
  - csv_parser
  - chart_generator
memory:
  enabled: true
  scope: agent
parameters:
  max_file_size: "10MB"
  supported_formats: ["csv", "json", "xlsx"]
system_prompt: |
  You are a data analysis expert who helps users understand their data.
  You can fetch data from URLs, parse various formats, and create
  visualizations.

  When you receive data analysis requests:
  1. Check your memory for similar past analyses
  2. Use appropriate tools to fetch and parse data
  3. Generate clear insights and recommendations
  4. Store successful patterns in memory for future use

user_prompt_template: |
  Analyze the following data request: {{request}}

  Available context from memory: {{memory_context}}
  Data source: {{data_source}}
  Requirements: {{requirements}}
---

## DataAnalyzer Agent

This agent specializes in data analysis and can:

- Fetch data from HTTP endpoints
- Parse CSV, JSON, and Excel files
- Generate charts and visualizations
- Learn from past analyses using embedded memory

## Usage Examples

Send capability-based messages like:
- "Analyze the sales trends in Q3 data"
- "Create a visualization showing monthly growth"
- "Compare this dataset with similar analyses you've done before"

The agent automatically learns from successful analyses and improves over
time.
```

## 3. Deploy the Configuration Agent

```bash
# Deploy from your config file
caxton agent deploy data-analyzer.md

# Verify deployment
caxton agent list
```

You should see:

```text
NAME           TYPE     STATUS    CAPABILITIES
DataAnalyzer   config   running   data-analysis, report-generation
```

## 4. Send a Capability-Based Message

Instead of addressing specific agents, send messages to **capabilities**:

```bash
# Request data analysis capability
caxton message send \
  --capability "data-analysis" \
  --performative request \
  --content '{
    "request": "Analyze monthly sales trends",
    "data_source": "https://example.com/sales.csv",
    "requirements": "Focus on growth patterns and seasonality"
  }'
```

The message router finds agents with `data-analysis` capability and routes
your request appropriately.

## 5. Watch Agent Learn and Improve

```bash
# Follow agent logs to see memory usage
caxton logs DataAnalyzer --follow
```

You'll see output showing how the agent:

- Searches memory for relevant past analyses
- Stores new patterns and insights
- Improves responses using learned context

```text
[DataAnalyzer] Searching memory for "sales trends analysis"...
[DataAnalyzer] Found 2 similar analyses from memory
[DataAnalyzer] Using pattern: "seasonal_analysis_template"
[DataAnalyzer] Analysis complete, storing insights in memory
[DataAnalyzer] Memory updated: added "sales_pattern_q3_2025"
```

## 6. Create a Multi-Agent Workflow

Create a report generator agent in `report-generator.md`:

```yaml
---
name: ReportGenerator
version: "1.0.0"
capabilities:
  - report-generation
  - document-creation
tools:
  - pdf_generator
  - template_engine
memory:
  enabled: true
  scope: workspace
system_prompt: |
  You are a report generation specialist. You create professional reports
  from analysis results provided by other agents.

  Listen for messages from data-analysis agents and automatically generate
  comprehensive reports from their findings.
---

## ReportGenerator Agent

Automatically creates professional reports from data analysis results.
```

Deploy and test the workflow:

```bash
# Deploy report generator
caxton agent deploy report-generator.md

# Send request that triggers both agents
caxton message send \
  --capability "data-analysis" \
  --performative request \
  --content '{
    "request": "Create a Q3 sales report",
    "data_source": "https://example.com/sales.csv",
    "requirements": "Include charts and executive summary"
  }' \
  --follow-conversation
```

The DataAnalyzer will process the data, then automatically send results to
ReportGenerator via capability-based routing.

## 7. Monitor Agent Conversations

```bash
# View conversation threads
caxton conversations list

# Follow a specific conversation
caxton conversations show <conversation-id> --follow
```

You'll see agent message flows between agents:

```text
CONVERSATION: conv-123
├─ REQUEST data-analysis: "Create Q3 sales report"
├─ INFORM report-generation: "Analysis complete: {results}"
└─ INFORM user: "Report generated: report.pdf"
```

## 8. Explore the Web Dashboard

Open http://localhost:8080/dashboard to see:

- **Agent Status**: Real-time agent health and activity
- **Capability Map**: Visual representation of available capabilities
- **Message Flow**: Agent message routing and conversation threads
- **Memory Insights**: What agents are learning and storing
- **Performance Metrics**: Response times and resource usage

## What Makes This Different

### 5-10 Minute Onboarding

Unlike traditional agent platforms requiring hours of compilation setup:

- **No toolchains**: Just markdown + YAML configuration
- **No compilation**: Agents deploy instantly from text files
- **No external dependencies**: Memory and routing work out-of-the-box

### Capability-Based Architecture

Agents communicate via **capabilities**, not rigid agent names:

- **Flexible routing**: Request "data-analysis" and get the best available
  agent
- **Load balancing**: Distribute work across agents with same capabilities
- **Easy scaling**: Add more agents with shared capabilities

### Built-in Learning

Agents automatically improve through embedded memory:

- **Pattern recognition**: Learn from successful interactions
- **Context building**: Build domain knowledge over time
- **Zero configuration**: SQLite + embedding model included

### Professional Messaging

Capability-based messaging ensures robust agent communication:

- **Standard protocols**: Industry-standard agent messaging
- **Conversation tracking**: Maintain context across multi-turn interactions
- **Error handling**: Proper failure and retry semantics

## Next Steps

Now that you have agents running and communicating:

- **[Create Your First Config Agent](first-agent.md)** - Deep dive into
  configuration options
- **[Configuration Reference](../operators/configuration.md)** - Complete
  YAML schema documentation
- **[API Patterns](../agent-developers/api-quickstart.md)** - Interact via
  REST API
- **[Advanced Patterns](../../developer-guide/agent-patterns.md)** -
  Multi-agent orchestration

## Experimentation Tips

1. **Start with single-capability agents** to understand the basics
2. **Enable agent memory** to see learning patterns emerge
3. **Use the web dashboard** to visualize agent interactions
4. **Try different capability combinations** to discover emergent behaviors
5. **Monitor memory growth** to understand what agents are learning
6. **Experiment with different LLM models** via configuration

## Common Experiments

### Pattern Discovery

Let agents find patterns in your data by enabling memory and running multiple
similar analyses:

```bash
# Run multiple related analyses
caxton message send --capability "data-analysis" --content '{"dataset": "jan-sales.csv"}'
caxton message send --capability "data-analysis" --content '{"dataset": "feb-sales.csv"}'
caxton message send --capability "data-analysis" --content '{"dataset": "mar-sales.csv"}'

# Check what patterns the agent discovered
caxton memory inspect DataAnalyzer --patterns
```

### Emergent Behavior

Create agents with overlapping capabilities and see how they self-organize:

```yaml
# Agent 1: Broad data analysis
capabilities: ["data-analysis", "visualization"]

# Agent 2: Specialized in finance
capabilities: ["data-analysis", "financial-modeling"]

# Agent 3: Report specialist
capabilities: ["visualization", "report-generation"]
```

### Knowledge Building

Use global memory scope to create knowledge-accumulating agents:

```yaml
memory:
  enabled: true
  scope: global
  semantic_search: true
  relationship_tracking: true
```

**Need help?** Run `caxton doctor` to diagnose issues, or check logs with
`caxton logs --all --level debug`.
