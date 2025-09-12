---
title: "Configuration Agent Format Specification"
date: 2025-09-10
layout: guide
categories: [Configuration, Agents, Reference]
---

## File Structure

Configuration agents are TOML files (`.toml`) that define agent behavior,
capabilities, and documentation in a single structured configuration file:

```text
agent-name.toml
├── Agent metadata
├── Capability declarations
├── Tool permissions
├── Prompt templates
├── Configuration parameters
└── Embedded documentation (in 'documentation' field)
    ├── Agent description
    ├── Usage examples
    └── Implementation notes
```

## TOML Schema Reference

### Required Fields

```toml
name = "string"                 # Agent identifier (required)
version = "string"              # Semantic version (required)
capabilities = ["string"]       # Capability declarations (required)
tools = ["string"]             # Tool permission list (required)
system_prompt = '''             # Agent behavior definition (required)
Multi-line agent behavior and instructions
'''
```

### Complete Schema

```toml
# Core Identity
name = "string"                 # Agent name (kebab-case recommended)
version = "string"              # Semantic versioning (e.g., "1.0.0")
description = "string"          # Brief agent purpose

# Capability System
capabilities = ["string"]       # List of capabilities this agent provides
requires = ["string"]          # Capabilities needed from other agents

# Tool Access Control
tools = ["string"]             # Allowed tool names

# Prompting System
system_prompt = '''
Multi-line prompt text defining agent personality,
constraints, and operating instructions.
'''

user_prompt_template = '''
Template with {{variable}} substitutions for dynamic content.
'''

# Agent Parameters
[parameters]
custom_param = "value"         # Agent-specific configuration values

# Tool Configuration
[tool_config.tool_name]
parameter = "value"

# Conversation Management
[conversation]
max_turns = 50                 # Maximum conversation length
context_window = 4000          # Token limit for context
memory_strategy = "sliding"    # "sliding" | "summarize" | "persist"

# Performance Settings
[performance]
max_execution_time = "60s"     # Maximum runtime (e.g., "30s", "5m")
max_memory_usage = "256MB"     # Memory limit (e.g., "100MB", "1GB")
concurrent_tools = 3           # Max simultaneous tool calls

# Memory Configuration
[memory]
enabled = true                 # Enable embedded memory system
scope = "agent"                # Memory scope: "agent" | "global" | "session"

# Metadata
author = "string"              # Agent author
license = "string"             # License identifier
tags = ["string"]             # Searchable tags
created = "2025-01-21T00:00:00Z" # Creation date (ISO 8601)
updated = "2025-01-21T00:00:00Z" # Last modification date

# Embedded Documentation
documentation = '''
# Agent Documentation

Markdown content describing the agent, usage examples,
and implementation notes.
'''
```

## Field Specifications

### Core Identity Fields

**name** (required): Agent identifier used in capability routing and agent
discovery. Must be unique within the deployment.

- Format: kebab-case recommended (e.g., `data-analyzer`)
- Length: 3-64 characters
- Pattern: `^[a-z][a-z0-9-]*[a-z0-9]$`

**version** (required): Semantic version string for agent versioning and
compatibility.

- Format: Semantic versioning (Major.Minor.Patch)
- Example: `"1.0.0"`, `"2.1.3-beta"`

**description** (optional): Brief description of agent purpose and capabilities.

- Length: Maximum 200 characters
- Used in agent discovery and marketplace listings

### Capability System Fields

**capabilities** (required): Array of capabilities this agent provides to other
agents.

```toml
capabilities = [
  "data-analysis",
  "report-generation",
  "chart-creation"
]
```

**requires** (optional): Array of capabilities this agent needs from other
agents.

```toml
requires = [
  "http-client",
  "database-access"
]
```

### Tool Access Control

**tools** (required): Array of tool names this agent is permitted to use.

```toml
tools = [
  "http_client",
  "csv_parser",
  "chart_generator",
  "file_storage"
]
```

**tool_config** (optional): Tool-specific configuration parameters.

```toml
[tool_config.http_client]
timeout = "30s"
max_redirects = 5

[tool_config.csv_parser]
delimiter = ","
encoding = "utf-8"
```

### Prompting System

**system_prompt** (required): Multi-line string defining agent behavior,
personality, and constraints.

```toml
system_prompt = '''
You are a data analysis expert who helps users understand their data.

Guidelines:
- Always validate data before processing
- Provide clear explanations of your analysis
- Use visualizations when they enhance understanding
- Ask clarifying questions when requirements are unclear

Constraints:
- Maximum file size: 10MB
- Supported formats: CSV, JSON, Excel
- Always respect data privacy and security
'''
```

**user_prompt_template** (optional): Template for processing user requests with
variable substitution.

```toml
user_prompt_template = '''
Analyze the following data request:

User Request: {{request}}

Available Data: {{data_source}}
Requirements: {{requirements}}

Please provide:
1. Data summary and validation
2. Analysis results
3. Visualization recommendations
4. Next steps or follow-up questions
'''
```

### Template Variables

User prompt templates support variable substitution using `{{variable}}`
syntax:

- `{{request}}` - Original user request text
- `{{context}}` - Conversation context and history
- `{{data}}` - Structured data passed to the agent
- `{{user_id}}` - Authenticated user identifier
- `{{timestamp}}` - Current timestamp
- `{{custom_param}}` - Values from parameters section

### Conversation Management

**conversation** (optional): Configuration for conversation handling and
context management.

```toml
[conversation]
max_turns = 50               # Maximum conversation length
context_window = 4000        # Token limit for context
memory_strategy = "sliding"  # Context management strategy
```

Memory strategies:

- `"sliding"` - Keep recent messages within context window
- `"summarize"` - Compress old messages into summaries
- `"persist"` - Store full conversation in persistent memory

### Performance Settings

**performance** (optional): Resource limits and execution constraints.

```toml
[performance]
max_execution_time = "60s"   # Maximum runtime per request
max_memory_usage = "256MB"   # Memory limit for agent execution
concurrent_tools = 3         # Maximum simultaneous tool calls
```

## Validation Rules

### Schema Validation

The runtime validates TOML configuration against the schema:

1. **Required field presence**: name, version, capabilities, tools,
   system_prompt
2. **Type checking**: Strings, arrays, objects match expected types
3. **Format validation**: Semantic versioning, kebab-case names
4. **Constraint checking**: String lengths, array sizes, numeric ranges

### Tool Permission Validation

Tool access is validated at runtime:

1. **Tool existence**: Requested tools must be available in the environment
2. **Permission grants**: Agent tool list matches requested permissions
3. **Configuration validity**: Tool-specific parameters are valid
4. **Capability alignment**: Required capabilities are available

### Prompt Template Validation

Template syntax and variables are validated:

1. **Variable syntax**: `{{variable}}` format is correct
2. **Balanced braces**: Opening and closing braces match
3. **Reserved variables**: System variables are not redefined
4. **Template compilation**: Templates compile successfully

## Example Minimal Agent

```toml
name = "echo-agent"
version = "1.0.0"
capabilities = ["message-echo"]
tools = ["text_processor"]

system_prompt = '''
You are a simple echo agent that repeats user messages with a friendly
greeting. Always be polite and add "Hello!" before echoing the message.
'''

documentation = '''
# Echo Agent

This is a minimal example agent that demonstrates the basic configuration
format. The agent simply echoes user messages with a greeting.

## Usage

Send any message to this agent and it will respond with "Hello! [your message]".
'''
```

## Next Steps

- **Examples**: Review practical configuration patterns and use cases
- **Best Practices**: Learn development guidelines and optimization techniques
- **Tools Integration**: Understand tool permission and configuration patterns
- **Templates**: Explore the template library for rapid development
