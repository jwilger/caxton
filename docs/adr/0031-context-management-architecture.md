---
title: "ADR-0031: Context Management Architecture"
date: 2025-09-10
status: accepted
layout: adr
categories: [Architecture]
---


## Status

Accepted

## Context

Configuration-driven agents (ADR-0028) require intelligent context management
to
provide effective responses during agent collaboration. Unlike compiled WASM
agents that can programmatically manage their context, configuration agents
operate through LLM API calls and need the runtime to provide appropriate
context.

### Core Challenge

When Agent A sends a FIPA message requesting the "data-analysis" capability,
what
context should Agent B (the receiving data analyzer) get in its LLM prompt? The
context must be:

- **Relevant and focused**: Include only information that helps complete the
  task
- **Well-formatted**: Appropriate for the target LLM provider
- **Performance-optimized**: Prepared in <100ms with >85% token utilization
- **Signal-to-noise optimized**: Filter out irrelevant information that confuses
  agents

### Integration Requirements

The context management system must integrate seamlessly with:

- **FIPA-ACL Messaging** (ADR-0029): Context derived from conversation threads
- **Embedded Memory System** (ADR-0030): Historical knowledge and patterns
- **MCP Tools**: Tool-specific context requirements and capabilities
- **Multi-LLM Providers**: Support for different context formatting needs

### Current State Problems

- No systematic approach to context preparation for configuration
  agents
- Risk of context overflow that degrades agent performance
- No standardized way for tools to declare their context needs
- Lack of conversation-aware context management across multi-turn interactions

## Decision

We will implement a **multi-layered context management architecture** that
automatically gathers, filters, and formats context for configuration agents
based on their current task and conversation context.

### Architecture Components

**Context Router**: Central orchestration component that coordinates context
gathering and formatting for each agent interaction.

**Context Sources Framework**: Pluggable system for gathering context from
multiple
sources:

- Conversation history from FIPA message threads
- Semantic search from embedded memory system (ADR-0030)
- Capability registry information
- MCP tool specifications and data

**Context Filtering Pipeline**: Multi-stage refinement system that optimizes
signal-to-noise ratio through hierarchical context windows and adaptive token
budgeting.

**Context Specification Engine**: Processes MCP tool context requirements and
agent prompt templates to determine exactly what context each interaction needs.

### Primary Strategy: Enhanced MCP Tool Context Requirements

MCP tools declare their context needs in YAML specifications:

```yaml
context_requirements:
  conversation_depth: 5  # Last 5 messages
  memory_search:
    query_template: "{{capability}} tasks similar to {{request}}"
    max_results: 10
  capability_info: true
  tool_data: ["user_preferences", "recent_results"]
```

The runtime uses these specifications to automatically gather appropriate
context without requiring prompt engineering from users.

### Multi-LLM Provider Support

The system adapts context formatting for different LLM providers:

- Provider-specific token limits and context window management
- Optimized formatting for each model's prompt structure
- Adaptive context selection based on provider capabilities

### Performance Targets

- **Context preparation latency**: <100ms (P95)
- **Token utilization efficiency**: >85% of available context window
- **Task completion success rate**: >90% for configuration agents

### Context Flow Architecture

1. **Request Analysis**: Parse incoming FIPA message and identify target
   capability
2. **Context Specification**: Determine context requirements from MCP tools and
   agent configuration
3. **Multi-Source Gathering**: Collect context from conversation history,
   memory, and tool data
4. **Filtering and Ranking**: Apply hierarchical filtering with semantic
   clustering
5. **Provider Formatting**: Format context appropriately for target LLM provider
6. **Context Injection**: Provide formatted context in agent's LLM API call

## Consequences

### Positive

- **Automatic context management**: Configuration agents get appropriate context
  without manual prompt engineering
- **Performance optimized**: Context preparation meets <100ms latency targets
- **Provider agnostic**: Works across multiple LLM providers with appropriate
  formatting
- **Tool integration**: MCP tools can declare exactly what context they need
- **Conversation aware**: Maintains context across multi-turn FIPA message
  exchanges
- **Memory enhanced**: Leverages embedded memory system for historical context
- **Signal-to-noise optimized**: Filtering pipeline prevents context overflow

### Negative

- **Complexity overhead**: Adds significant runtime complexity for context
  management
- **Resource usage**: Context processing requires CPU and memory resources
- **Latency introduction**: <100ms context preparation adds to response time
- **Configuration burden**: MCP tools must specify context requirements
  correctly

### Risk Mitigation

- **Performance monitoring**: Built-in metrics track context preparation
  latency and success rates
- **Graceful degradation**: System continues working with reduced context if
  preparation fails
- **Context debugging**: Development tools help optimize context specifications
- **Incremental rollout**: Context management can be enabled per-agent for
  testing

## Implementation Approach

The implementation focuses on four core areas:

1. **Context Router**: Central orchestration with pluggable context source
   integration
2. **MCP Tool Integration**: Enhanced tool specifications with context
   requirement declarations
3. **Multi-Provider Support**: LLM provider abstraction with provider-specific
   formatting
4. **Performance Optimization**: Caching, parallel context gathering, and
   adaptive filtering

## Alignment with Existing ADRs

- **ADR-0028 (Configuration-Driven Agents)**: Provides intelligent context
  management for config agents
- **ADR-0029 (FIPA-ACL Messaging)**: Context derived from conversation threads
  and message history
- **ADR-0030 (Embedded Memory System)**: Semantic search provides historical
  context and patterns
- **ADR-0005 (MCP for External Tools)**: Enhanced with context requirement
  specifications
- **ADR-0001 (Observability First)**: Context operations are fully instrumented
  and observable

## Related Decisions

- ADR-0028: Configuration-Driven Agent Architecture (defines agents that use
  this context management)
- ADR-0029: FIPA-ACL Lightweight Messaging (provides conversation context
  source)
- ADR-0030: Embedded Memory System (provides semantic search context source)
- ADR-0005: MCP for External Tools (enhanced with context requirements)

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io) - MCP
  tool integration patterns
- Context management patterns from expert technical architecture analysis
- Multi-LLM provider context formatting research
- Performance optimization techniques for real-time context preparation

---

**Implementation Status**: This ADR documents an architectural decision being
made. The context management architecture will be implemented as part of the
configuration-driven agent system development, with the runtime taking
responsibility for intelligent context gathering and formatting rather than
requiring users to manage context manually.
