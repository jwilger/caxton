---
title: "Configuration-Driven Agents: Overview"
date: 2025-09-10
layout: guide
categories: [Agent Developers, Configuration, Agents]
difficulty: beginner
audience: agent-developers
---

> **🚧 Implementation Status**
>
> Configuration-driven agents represent the primary user experience designed
> in ADR-28. This documentation serves as the specification and acceptance
> criteria for the agent configuration system currently under development.
>
> **Target**: TOML agent definitions with 5-10 minute creation time
> **Current Status**: Configuration schema and runtime implementation in
> progress

## What Are Configuration-Driven Agents? - **Beginner**

Configuration-driven agents are AI agents defined as TOML configuration files,
designed to dramatically lower the barrier to entry for agent development.
Instead of compiling WebAssembly modules, you create agents by writing TOML
files that combine prompts, capabilities, and tool permissions in a clean,
readable format.

## Key Benefits

**Rapid Development**: Create working agents in 5-10 minutes instead of 2-4
hours required for WASM compilation.

**Easy Sharing**: Text-based definitions are version-controllable, shareable,
and community-friendly.

**Simple Debugging**: Clear visibility into agent behavior through readable
configuration and tool calls.

**Template-Driven**: 80% of use cases covered by pre-built templates that you
can customize.

## Architecture Overview - **Intermediate**

Configuration agents operate through a four-layer architecture with intelligent
context management:

```text
┌─────────────────────┐
│ Configuration Layer │  ← TOML configuration with embedded docs
├─────────────────────┤
│ Context Layer       │  ← Intelligent context management (ADR-0031)
├─────────────────────┤
│ Orchestration Layer │  ← LLM-powered prompt execution
├─────────────────────┤
│ Tool Layer          │  ← WebAssembly tool implementations
└─────────────────────┘
```

### Configuration Layer

The top layer where you define agent behavior using TOML configuration with
embedded documentation:

```toml
name = "DataAnalyzer"
version = "1.0.0"
description = "Analyzes CSV data and generates insights"

capabilities = [
  "data_analysis",
  "visualization"
]

tools = [
  "csv_reader",
  "chart_generator"
]

system_prompt = '''
You are an expert data analyst. Your job is to analyze CSV data and provide
actionable insights.

Instructions:
1. When given CSV data, examine the structure first
2. Identify key patterns and trends
3. Generate appropriate visualizations
4. Provide clear, actionable recommendations
'''

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.1

documentation = '''
# Data Analysis Agent

This agent specializes in analyzing CSV data and generating actionable insights
through statistical analysis and visualizations.
'''
```

### Context Layer - **Advanced**

The context layer automatically manages context for your agent, implementing
the patterns from ADR-0031:

- **Automatic Context Preparation**: Relevant context is prepared before each
  LLM call
- **Memory Integration**: Previous conversations and learned patterns are
  included
- **Tool Context**: Information about available tools and their usage patterns
- **Domain Knowledge**: Relevant domain-specific information

### Orchestration Layer

The orchestration layer handles:

- **Prompt Execution**: Your markdown instructions are processed by the LLM
- **Tool Invocation**: When the LLM decides to use tools, they're called
  automatically
- **Response Generation**: Structured responses are generated and returned
- **Error Handling**: Graceful handling of tool failures and LLM errors

### Tool Layer

Tools are implemented as WebAssembly modules for security and performance:

- **Sandboxed Execution**: Each tool runs in an isolated environment
- **Standardized Interface**: All tools use the same calling convention
- **Resource Limits**: Memory and CPU usage is controlled
- **Permission System**: Tools only have access to explicitly granted
  resources

## Agent Development Workflow - **Beginner**

### 1. Choose a Template

Start with a pre-built template for common use cases:

```bash
# List available templates
curl http://localhost:3000/api/templates

# Get template content
curl http://localhost:3000/api/templates/data-analyzer > my-agent.toml
```

Available templates:

- **data-analyzer**: CSV processing and visualization
- **content-writer**: Content generation and editing
- **api-integrator**: External API integration
- **workflow-coordinator**: Multi-step process management
- **monitoring-agent**: System monitoring and alerting

### 2. Customize Configuration

Edit the TOML configuration to match your needs:

```toml
name = "MyCustomAnalyzer"
version = "1.0.0"
description = "Custom data analyzer for sales reports"

capabilities = [
  "data_analysis",
  "report_generation"
]

tools = [
  "csv_reader",
  "excel_reader",        # Added Excel support
  "chart_generator",
  "pdf_generator"        # Added PDF output
]

[llm]
provider = "openai"
model = "gpt-4"
temperature = 0.2       # More deterministic
max_tokens = 2000

[schedule]
cron = "0 9 * * MON"    # Run every Monday at 9 AM
timezone = "UTC"
```

### 3. Define System Prompt

Provide clear, specific instructions in the TOML system_prompt field:

```toml
system_prompt = '''
You are a sales data analyst specializing in weekly reports.

Instructions:
1. **Data Loading**: Accept CSV or Excel files with sales data
2. **Data Validation**: Check for required columns: date, product, amount,
   region
3. **Analysis**: Calculate:
   - Total sales by region
   - Week-over-week growth
   - Top performing products
   - Trend analysis
4. **Visualization**: Create:
   - Regional sales chart
   - Growth trend line
   - Product performance bar chart
5. **Report Generation**: Generate PDF report with findings and recommendations

Example Interaction:
**User**: "Analyze the Q4 sales data"
**Agent**:
1. I'll load your sales data and validate the structure
2. Analyze regional performance and growth trends
3. Create visualizations showing key insights
4. Generate a comprehensive PDF report

Would you like me to start with a specific data file?
'''

documentation = '''
# Sales Data Analyzer

This agent specializes in weekly sales report generation with comprehensive
analysis and visualization capabilities.
'''
```

### 4. Test and Deploy

```bash
# Validate configuration
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{"definition": "'$(cat my-agent.toml)'"}'"}

# Deploy agent
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "type": "configuration",
    "definition": "'$(cat my-agent.toml)'"
  }'

# Test with sample data
curl -X POST http://localhost:3000/api/agents/my-custom-analyzer/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Analyze the attached sales data",
    "conversation_id": "test-conv"
  }'
```

## Configuration Options - **Intermediate**

### Core Configuration

```toml
# Required fields
name = "AgentName"                  # Unique identifier
version = "1.0.0"                  # Semantic version
description = "Brief description"  # One-line summary

# Agent capabilities
capabilities = [                   # List of high-level capabilities
  "capability_name"
]

tools = [                          # Specific tools to use
  "tool_name"
]

system_prompt = '''
Agent behavior and instructions go here.
'''

# LLM configuration
[llm]
provider = "openai"               # openai|anthropic|local
model = "model_name"               # Specific model
temperature = 1.0                  # Response randomness (0.0-2.0)
max_tokens = 1000                  # Max response length

# Optional advanced configuration
[permissions]
file_access = "readonly"           # readonly|readwrite|none
network_access = "restricted"      # restricted|full|none
memory_limit = "100MB"

[schedule]
cron = "0 9 * * MON-FRI"          # Cron expression
timezone = "UTC"                   # Timezone

[audit]
log_operations = true
sensitive_data = false

documentation = '''
# Agent Documentation

Agent description and usage instructions.
'''
```

### Tool Configuration

Tools can have specific configuration:

```toml
tools = [
  "csv_reader",
  "api_client",
  "chart_generator"
]

[tool_config.csv_reader]
encoding = "utf-8"
delimiter = ","
max_file_size = "10MB"

[tool_config.api_client]
base_url = "https://api.example.com"
timeout = "30s"
retry_attempts = 3
rate_limit = "100/minute"

[tool_config.chart_generator]
default_format = "png"
max_width = 1024
max_height = 768
dpi = 300
```

### Memory Configuration

Control how the agent uses memory:

```toml
[memory]
enabled = true
retention_period = "30d"           # How long to keep memories
max_entries = 10000                # Maximum memory entries

[memory.context_preparation]
enabled = true
max_context_length = 8000          # Max tokens for context
relevance_threshold = 0.7          # Minimum relevance score

[memory.learning]
enabled = true
learning_rate = 0.1                # How quickly to adapt
confidence_threshold = 0.8         # Minimum confidence for learning
```

## Best Practices - **Intermediate**

### Configuration Best Practices

1. **Clear Names**: Use descriptive, unique agent names
2. **Semantic Versioning**: Follow semver for version numbers
3. **Minimal Permissions**: Only request necessary tools and permissions
4. **Resource Limits**: Set appropriate memory and CPU limits
5. **Testing**: Always validate before deployment

### Instruction Best Practices

1. **Be Specific**: Provide clear, step-by-step instructions
2. **Include Examples**: Show expected input/output patterns
3. **Error Handling**: Explain how to handle common error cases
4. **Context**: Provide relevant background information
5. **Constraints**: Clearly specify limitations and boundaries

### Security Best Practices

1. **Principle of Least Privilege**: Minimal necessary permissions
2. **Input Validation**: Always validate user inputs
3. **Output Sanitization**: Clean outputs before presentation
4. **Audit Logging**: Enable comprehensive logging
5. **Regular Updates**: Keep agent configurations current

## Troubleshooting - **Intermediate**

### Common Configuration Issues

**Agent Won't Deploy**:

- Check TOML syntax with a validator
- Verify all required fields are present
- Ensure capabilities and tools exist

**Tool Access Denied**:

- Check tool permissions in configuration
- Verify agent has necessary capabilities
- Review security policies

**Poor Performance**:

- Reduce context length or complexity
- Optimize tool usage patterns
- Consider caching frequently used data

**Unexpected Behavior**:

- Review agent instructions for clarity
- Check tool configuration
- Examine conversation history for patterns

### Debugging Tools

```bash
# Check agent status
curl http://localhost:3000/api/agents/my-agent

# View agent logs
curl http://localhost:3000/api/agents/my-agent/logs

# Get performance metrics
curl http://localhost:3000/api/agents/my-agent/metrics

# Test specific capability
curl -X POST http://localhost:3000/api/agents/my-agent/test \
  -d '{"capability": "data_analysis", "input": "sample data"}'
```

## Related Documentation

- [Agent Format Specification](agent-format.md) - **Beginner**
- [Best Practices Guide](best-practices.md) - **Intermediate**
- [LLM Provider Configuration](llm-providers.md) - **Beginner**
- [Configuration Examples](examples.md) - **Beginner**
- [Building Agents Guide](../building-agents.md) - **Beginner**
- [Security Guide](../security.md) - **Intermediate**

## Quick Start Checklist

- [ ] Choose appropriate template
- [ ] Configure TOML settings
- [ ] Write clear instructions
- [ ] Set minimal permissions
- [ ] Validate configuration
- [ ] Deploy and test
- [ ] Monitor performance
- [ ] Iterate based on feedback
