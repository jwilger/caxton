# API Documentation

This section covers Caxton's REST API, config agent patterns, and MCP integration.

## Quick Start

- **[Config Agents Overview](overview.md)** - Start here for config-driven
  agents
- **[Implementation Status](implementation-status.md)** - Current
  implementation state

## Config Agents

Config agents are TOML-based agents that provide a 5-10 minute onboarding
experience:

- **[Agent Format](agent-format.md)** - TOML configuration format for agents
- **[Examples](examples.md)** - Example config agents for common use cases
- **[Best Practices](best-practices.md)** - Config agent development patterns
- **[LLM Providers](llm-providers.md)** - Integration with language model providers

## Advanced Configuration

- **[Config Agent Patterns](config-agent-patterns.md)** - Advanced patterns
  and workflows
- **[Configuration Validation](configuration-validation.md)** - Validation
  rules and error handling
- **[Performance Specifications](performance-specifications.md)** -
  Performance requirements and optimization

## REST API Integration

- **[Capability Registration](capability-registration.md)** - Register and
  manage agent capabilities
- **[Memory Integration](memory-integration.md)** - Integrate with Caxton's
  embedded memory system

## MCP (Model Context Protocol) Integration

- **[MCP Integration Guide](mcp-integration.md)** - Integrate MCP tools with
  Caxton agents
- **[MCP State Tool Specification](mcp-state-tool-specification.md)** - State
  tool implementation details

## Related Documentation

- **[Getting Started](../getting-started/)** - Quick setup and first agent
- **[Architecture](../architecture/)** - System design and ADRs
- **[Operations](../operations/)** - Production deployment guides
- **[Concepts](../concepts/)** - Deep dives into messaging and memory
- **[Main Documentation](../README.md)** - Back to docs overview

## Implementation Note

This documentation serves as acceptance criteria and architectural
specification for development. Some features may be in planning or development
phases - check [implementation-status.md](implementation-status.md) for current
state.
