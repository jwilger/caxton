---
title: "ADR-0040: LLM Providers as MCP Tools"
date: 2025-01-17
status: accepted
layout: adr
categories: [Architecture, LLM Integration, MCP, Tool Architecture]
---

## Status

Accepted

## Context

Caxton's configuration-driven agents require Large Language Model (LLM)
integration for intelligent behavior. Currently, LLM providers are accessed
through direct API integrations, creating coupling between the agent runtime
and specific provider implementations. This approach limits flexibility,
complicates provider management, and prevents runtime provider switching.

### Core Challenge

The LLM provider integration challenge involves multiple architectural concerns:

- **Provider Proliferation**: New LLM providers emerge constantly with
  different APIs and capabilities
- **Runtime Flexibility**: Agents need to switch providers dynamically based
  on availability and cost
- **Custom Models**: Organizations deploy proprietary models requiring custom
  integration
- **Cost Management**: Different tasks require different cost/quality trade-offs
- **Vendor Lock-in**: Direct API integration creates dependencies on specific
  providers
- **Testing Complexity**: Mocking LLM responses for testing requires
  intercepting API calls

### Current Approach Limitations

The existing direct integration approach has several limitations:

**Static Configuration**: Provider selection happens at configuration time,
not runtime.

**Code Coupling**: Each new provider requires code changes to the core platform.

**Limited Observability**: Direct API calls bypass the platform's tool
observation layer.

**No Hot-Swapping**: Changing providers requires agent restart or
reconfiguration.

**Testing Difficulty**: No clean abstraction point for test doubles and mocks.

### Requirements

The solution must provide:

- **Unified Interface**: Single abstraction for all LLM operations
- **Runtime Flexibility**: Dynamic provider selection and switching
- **Custom Provider Support**: Easy integration of proprietary models
- **Cost Tracking**: Per-operation cost visibility and limits
- **Provider Isolation**: WASM sandboxing for untrusted providers
- **Testing Support**: Clean mocking and stubbing for tests

## Decision

We will implement LLM providers as pluggable Model Context Protocol (MCP)
tools, treating LLM operations as tool invocations rather than direct API
calls.

### Architecture Overview

**MCP Tool Interface**: All LLM operations go through the standard MCP tool
invocation protocol.

**Provider Registration**: LLM providers register as MCP tools with
standardized schemas.

**Configuration Format**: Agents reference providers using MCP URIs
(e.g., `mcp://openai-gpt-4`).

**Tool Categories**: Separate tools for different operations (chat,
embeddings, completion).

**WASM Sandboxing**: Custom providers run in WASM sandboxes for security
isolation.

### Provider Tool Schema

LLM provider tools expose standardized operations through MCP:

```yaml
Tool: mcp://openai/chat
Input: messages[], temperature, max_tokens, ...
Output: response, usage, cost

Tool: mcp://anthropic/chat
Input: messages[], temperature, max_tokens, ...
Output: response, usage, cost

Tool: mcp://custom-model/chat
Input: messages[], temperature, max_tokens, ...
Output: response, usage, cost
```

### Multi-Model Architecture

Different agent components use different models optimized for their purposes:

**Routing Model**: Fast, cheap model for capability matching and routing
decisions.

**Execution Model**: Powerful model for actual agent task execution.

**Summarization Model**: Efficient model for context compression and
summarization.

**Embedding Model**: Specialized model for semantic search and similarity.

### Fallback Chain Configuration

Agents configure fallback chains for resilience:

```toml
primary_provider = "mcp://openai-gpt-4"
secondary_provider = "mcp://anthropic-claude"
fallback_provider = "mcp://ollama-local"
```

## Consequences

### Positive Consequences

- **Provider Independence**: No vendor lock-in, easy provider switching
- **Custom Model Support**: Organizations can integrate proprietary models
  as MCP tools
- **Unified Observability**: All LLM operations visible through tool
  invocation tracing
- **Cost Management**: Per-invocation cost tracking and budget enforcement
- **Testing Simplicity**: Mock MCP tools for deterministic testing
- **Security Isolation**: Untrusted providers run in WASM sandboxes
- **Runtime Flexibility**: Hot-swap providers without agent restarts

### Negative Consequences

- **Additional Abstraction Layer**: MCP tool interface adds complexity
- **Performance Overhead**: Tool invocation protocol adds latency
- **Provider Adaptation**: Existing providers need MCP tool wrappers
- **Schema Maintenance**: Standardized schemas must evolve with provider
  capabilities

### Trade-off Analysis

We accept the abstraction overhead because:

1. **Flexibility Value**: Runtime provider switching enables cost optimization
2. **Vendor Independence**: Avoiding lock-in justifies wrapper complexity
3. **Testing Benefits**: Clean test doubles improve development velocity
4. **Security Gains**: WASM isolation protects against malicious providers

## Implementation Approach

Implementation follows a phased approach:

1. **Define MCP Tool Schemas**: Standardize chat, embedding, and completion
   operations
2. **Create Provider Wrappers**: Build MCP tools for major providers
   (OpenAI, Anthropic, Ollama)
3. **Update Agent Runtime**: Modify configuration agents to use MCP tool
   invocations
4. **Add Cost Tracking**: Implement usage and cost reporting per invocation
5. **Enable Hot-Swapping**: Support runtime provider changes without restarts

## Alignment with Existing ADRs

This decision reinforces:

- **ADR-0005 (MCP for External Tools)**: Extends MCP usage to LLM providers
- **ADR-0028 (Config Agents)**: Enhances configuration-driven agents with
  flexible LLM access
- **ADR-0002 (WebAssembly Isolation)**: Uses WASM sandboxing for custom
  providers
- **ADR-0034 (OpenAI Compatible LLM)**: MCP tools use OpenAI-compatible
  schemas
- **ADR-0039 (LLM Failure Handling)**: Fallback chains implement failure
  strategies

## Industry Precedent

Similar approaches in the industry:

- **LangChain**: Abstracts LLM providers through common interfaces
- **LiteLLM**: Provides unified API for multiple LLM providers
- **Semantic Kernel**: Microsoft's pluggable AI orchestration framework
- **AutoGen**: Treats LLMs as pluggable components in agent systems

## Future Considerations

- **Provider Discovery**: Registry for discovering available MCP LLM tools
- **Capability Negotiation**: Runtime detection of provider capabilities
- **Load Balancing**: Distribute requests across multiple provider instances
- **Caching Layer**: Cache responses for identical requests
- **Provider Metrics**: Track latency, reliability, and quality per provider

## Security Considerations

- **API Key Management**: Secure storage and injection of provider credentials
- **Rate Limiting**: Prevent abuse through per-tool rate limits
- **Cost Limits**: Enforce budget constraints at the MCP tool layer
- **Audit Logging**: Track all LLM invocations for compliance
- **WASM Isolation**: Untrusted providers cannot access host resources

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- ADR-0005: MCP for External Tools
- ADR-0028: Configuration-Driven Agent Architecture
- ADR-0034: OpenAI-Compatible LLM Abstraction
- ADR-0039: LLM Failure Handling Strategy
