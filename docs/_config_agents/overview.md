---
title: "Configuration-Driven Agents: Overview"
date: 2025-09-10
layout: guide
categories: [Configuration, Agents]
---

> **🚧 Implementation Status**
>
> Configuration-driven agents represent the primary user experience designed
> in ADR-28. This documentation serves as the specification and acceptance
> criteria for the agent configuration system currently under development.
>
> **Target**: Markdown + YAML agent definitions with 5-10 minute creation time
> **Current Status**: Configuration schema and runtime implementation in
> progress

## What Are Configuration-Driven Agents?

Configuration-driven agents are AI agents defined as markdown files with YAML
frontmatter, designed to dramatically lower the barrier to entry for agent
development. Instead of compiling WebAssembly modules, you create agents by
writing configuration files that combine prompts with tool permissions.

## Key Benefits

**Rapid Development**: Create working agents in 5-10 minutes instead of 2-4
hours required for WASM compilation.

**Easy Sharing**: Text-based definitions are version-controllable, shareable,
and community-friendly.

**Simple Debugging**: Clear visibility into agent behavior through readable
configuration and tool calls.

**Template-Driven**: 80% of use cases covered by pre-built templates that you
can customize.

## Architecture Overview

Configuration agents operate through a four-layer architecture with intelligent
context management:

```text
┌─────────────────────┐
│ Configuration Layer │  ← YAML frontmatter + markdown
├─────────────────────┤
│ Context Layer       │  ← Intelligent context management (ADR-0031)
├─────────────────────┤
│ Orchestration Layer │  ← LLM-powered prompt execution
├─────────────────────┤
│ Tool Execution Layer│  ← WebAssembly MCP servers
└─────────────────────┘
```

**Configuration Layer**: Defines agent capabilities, tools, and prompts
**Context Layer**: Automatically gathers and formats relevant context
**Orchestration Layer**: LLM interprets prompts and orchestrates tool calls
**Tool Execution Layer**: Secure WebAssembly sandboxes execute actual work

### Intelligent Context Management

Configuration agents benefit from automatic context management that eliminates
the need for manual prompt engineering. The context system provides:

- **Conversation-aware context**: Maintains relevant history across multi-turn
  interactions
- **Memory-augmented insights**: Leverages embedded memory system for
  historical patterns
- **Tool-specific requirements**: MCP tools declare exactly what context they
  need
- **Multi-LLM optimization**: Formats context appropriately for different
  providers

This architecture ensures agents receive optimal context without user
configuration, achieving >90% task completion rates with <100ms context
preparation latency.

## Security Model

Configuration agents maintain security through **capability-based isolation**:

- **Agent Configuration**: Declares required tools and permissions
- **Tool Sandboxing**: Actual tools run in WebAssembly sandboxes
- **Permission Verification**: Runtime verifies agent can access requested tools
- **Resource Limits**: Memory and CPU constraints enforced at tool level

## When to Use Configuration vs WebAssembly

**Use Configuration Agents for**:

- Data analysis and transformation
- API integration and orchestration
- Content generation and processing
- Workflow automation
- Most business logic scenarios

**Use WebAssembly Agents for**:

- Custom algorithms and mathematical computations
- High-performance data processing
- Specialized security requirements
- Integration with existing compiled libraries

## Agent Lifecycle

1. **Definition**: Write markdown file with YAML frontmatter configuration
2. **Validation**: Runtime validates schema and tool permissions
3. **Loading**: Agent registered with capability system
4. **Context Preparation**: Runtime gathers relevant context from multiple
   sources automatically
5. **Execution**: LLM processes prompts with optimized context and orchestrates
   tool calls
6. **Hot Reload**: Changes detected and reloaded automatically

### Context Management in Execution

During execution, the context management system automatically:

- **Analyzes requests**: Determines what context is needed based on the
  requested capability and agent configuration
- **Gathers multi-source context**: Combines conversation history, memory
  system insights, and tool-specific data
- **Filters for relevance**: Applies hierarchical filtering to optimize
  signal-to-noise ratio
- **Formats for provider**: Adapts context structure for the target LLM
  provider's optimal format

This eliminates the need for manual context engineering while ensuring optimal
agent performance.

## Template Categories

Configuration agents support common patterns through templates:

- **Data Processors**: CSV/JSON analysis, transformation, reporting
- **API Integrators**: REST API calls, authentication, data mapping
- **Content Generators**: Documentation, reports, summaries
- **Workflow Coordinators**: Multi-step business processes
- **Monitoring Agents**: System health checks, alert processing

## Getting Started

1. Choose an appropriate template from the library
2. Customize the YAML configuration for your needs
3. Modify prompts to match your specific requirements
4. Test the agent in the development environment
5. Deploy to production with hot-reload capabilities

## Comparison with Traditional Development

| Aspect | Config Agents | WASM Agents |
|--------|---------------|-------------|
| Setup Time | 5-10 minutes | 2-4 hours |
| Toolchain | Text editor | Rust/Go/C++ compiler |
| Context Management | Automatic runtime intelligence | Manual programmatic |
|                    |                                | control             |
| Debugging | Readable logs | WASM debugging tools |
| Sharing | Copy text file | Binary distribution |
| Iteration | Edit and reload | Compile and redeploy |
| Security | Tool-level sandboxing | Module-level isolation |

### Context Management Benefits

**Configuration Agents**:

- Context automatically gathered from conversation, memory, and tools
- Multi-LLM provider optimization without configuration
- Performance targets: <100ms preparation, >85% token utilization
- Zero manual prompt engineering required

**WASM Agents**:

- Full programmatic control over context management
- Custom optimization strategies possible
- Direct integration with agent logic
- Requires implementation expertise

## Next Steps

- **Agent Format**: Learn the YAML schema and configuration options
- **Examples**: Explore practical configuration patterns
- **Best Practices**: Development guidelines and optimization tips
- **Migration**: Convert existing WASM agents to configuration format
