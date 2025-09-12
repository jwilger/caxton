---
title: "ADR-0028: Config Agents"
date: 2025-09-09
status: accepted
layout: adr
categories: [Architecture]
---


## Status

Accepted

## Context

The current Caxton architecture requires agents to be compiled WebAssembly
modules, which creates several challenges:

### Current State Problems

- **High barrier to entry**: 2-4 hours to create first working agent (including
  toolchain setup)
- **Complex development workflow**: Write code → Compile to WASM → Deploy →
  Debug cycle
- **Documentation burden**: Must support 5+ compilation languages and their
  toolchains
- **Debugging difficulties**: WASM debugging tools are still maturing
- **Community friction**: Binary distribution makes sharing and contributing
  agents difficult

### Market Validation

Claude Code has proven that configuration-driven agents can be highly
successful:

- 100+ community-contributed agents
- 5-10 minute onboarding experience
- Text-based, version-controllable agent definitions
- Easy sharing and reuse patterns

### User Research Findings

Through expert analysis, we identified that for 90% of use cases, agents are
primarily:

1. **Orchestration logic**: Combining prompts with tool calls
2. **No custom algorithms**: Most logic can be handled by LLMs + tools
3. **Rapid iteration needs**: Users want to modify behavior quickly during
   development

## Decision

We will **replace compiled WebAssembly agents with configuration-driven agents**
as the primary user experience, while maintaining WebAssembly for advanced use
cases.

### Configuration-Driven Agent Definition

Agents will be defined as TOML configuration files:

```toml
name = "DataAnalyzer"
version = "1.0.0"
capabilities = ["data-analysis", "report-generation"]
tools = ["http_client", "csv_parser", "chart_generator"]

[parameters]
max_file_size = "10MB"
supported_formats = ["csv", "json", "xlsx"]

system_prompt = '''
You are a data analysis expert who helps users understand their data.
You can fetch data from URLs, parse various formats, and create visualizations.
'''

user_prompt_template = '''
Analyze the following data request: {{request}}

Available data: {{context}}
User requirements: {{requirements}}
'''

documentation = '''
# DataAnalyzer Agent

This agent specializes in data analysis tasks and can:
- Fetch data from HTTP endpoints
- Parse CSV, JSON, and Excel files
- Generate charts and visualizations
- Provide statistical summaries

## Usage Examples

Ask me to:
- "Analyze the sales data at https://example.com/sales.csv"
- "Create a chart showing monthly trends"
- "Summarize the key metrics in this dataset"
'''
```

### Architecture Overview

The system will support two types of agents:

- **Configuration agents**: Defined in TOML configuration files,
  executed in the host runtime through LLM orchestration
- **WASM agents**: Compiled modules for power users requiring custom algorithms,
  executed in sandboxed WebAssembly runtime

**Security model**: Configuration agents orchestrate through LLM calls while
actual functionality is provided by WebAssembly MCP servers running in isolated
sandboxes. This maintains security isolation where it matters most while
enabling rapid agent development.

## Consequences

### Positive

- **Dramatically lower barrier to entry**: 5-10 minutes to first working agent
- **Rapid iteration**: Edit config file and immediately test changes
- **Community-friendly**: Text-based, shareable, version-controllable agents
- **Simplified documentation**: Single configuration format vs multiple
  compilation paths
- **Better debugging**: Clear visibility into agent behavior and tool calls
- **Proven pattern**: Validated by Claude Code's success in the market

### Negative

- **Limited to orchestration**: Custom algorithms require WASM agents
- **Runtime dependency**: Config agents need the host runtime for execution
- **Prompt engineering required**: Users must understand how to craft effective
  prompts

### Migration Strategy

- Existing WASM agents remain supported for backward compatibility
- Provide tooling to migrate common WASM patterns to config format
- Create template library covering 80% of common agent patterns
- Document clear guidelines for when to use config vs WASM

## Implementation Approach

The implementation will focus on three core areas:

1. **Configuration Runtime**: TOML schema validation, prompt templating, and
   tool permission systems
2. **Developer Experience**: Agent templates, hot-reload development, and
   migration utilities from WASM patterns
3. **Community Features**: A/B testing, performance monitoring, and marketplace
   integration

## Alignment with Existing ADRs

- **ADR-0004 (Minimal Core Philosophy)**: Config agents reduce core complexity
  while preserving power user options
- **ADR-0002 (WebAssembly Isolation)**: MCP servers maintain WASM sandboxing
  where it matters most
- **ADR-0001 (Observability First)**: Config agents provide better visibility
  into agent behavior
- **ADR-0005 (MCP for External Tools)**: Enhanced by making MCP the primary tool
  integration method

## Related Decisions

- ADR-0029: Lightweight Agent Messaging (defines how config agents
  communicate)
- ADR-0030: Embedded Memory System (provides memory capabilities to config
  agents)

## References

- [Claude Code Agent Documentation](https://docs.anthropic.com/en/docs/claude-code/agents)
- Expert analysis from technical-architect, product-manager, domain-expert, and
  documentation-writer
- Market research on agent platform adoption patterns
