---
title: "ADR-0005: MCP for External Tools"
date: 2025-01-31
status: proposed
layout: adr
categories: [Technology]
---


## Status

Proposed

## Context

Agents need to interact with the external world:

- Query databases and APIs
- Read and write files
- Send emails and notifications
- Call LLM services
- Execute system commands

We need a standardized way for agents to access these capabilities while
maintaining:

- Security: Agents can only access approved tools
- Observability: All tool usage is recorded as events
- Portability: Same agent code works in different environments
- Flexibility: Easy to add new tools without modifying agents

## Decision

We will use the Model Context Protocol (MCP) as the standard interface for
agents to access external tools and resources.

Key aspects:

- Tools are exposed to agents as MCP servers
- Agents make tool requests through a standard API
- The host manages tool authentication and authorization
- All tool invocations are recorded as events
- Tools can be implemented in any language

## Consequences

### Positive

- **Industry standard**: MCP is becoming the de facto standard for LLM tools
- **Language agnostic**: Tools can be written in any language
- **Rich ecosystem**: Growing collection of MCP-compatible tools
- **Security model**: Clear boundary between agents and system resources
- **Composability**: Tools can be shared across different agent systems

### Negative

- **Additional protocol**: Agents must understand MCP in addition to FIPA
- **Performance overhead**: Extra serialization/deserialization step
- **Complexity**: Another moving part in the system
- **Early standard**: MCP is still evolving

### Mitigations

- Provide WASM-friendly MCP client libraries
- Cache tool connections to reduce overhead
- Keep MCP integration optional - agents can work without tools
- Track MCP standard evolution and update accordingly
- Implement efficient binary encoding for MCP messages

## Alternatives Considered

### Direct Function Calls

- **Pros**: Simple, fast
- **Cons**: No isolation, security risks, couples agents to host

### Custom Tool Protocol

- **Pros**: Optimized for our use case
- **Cons**: Another protocol to design and maintain, no ecosystem

### GraphQL/REST APIs

- **Pros**: Familiar to developers
- **Cons**: Not designed for tool discovery and invocation

### WASI (WebAssembly System Interface)

- **Pros**: Standard for WASM system access
- **Cons**: Limited to system calls, not general tools

## Implementation Example

```rust
// Host provides MCP access to agents
impl HostFunctions for CaxtonHost {
    async fn mcp_call(
        &self,
        agent_id: AgentId,
        tool_name: &str,
        params: Value,
    ) -> Result<Value, ToolError> {
        // Record tool invocation event
        self.events.record(Event::ToolInvoked {
            agent: agent_id,
            tool: tool_name.to_string(),
            params: params.clone(),
        });

        // Check permissions
        if !self.can_access_tool(agent_id, tool_name) {
            return Err(ToolError::Unauthorized);
        }

        // Execute tool
        let result = self.mcp_client.call(tool_name, params).await?;

        // Record completion event
        self.events.record(Event::ToolCompleted {
            agent: agent_id,
            tool: tool_name.to_string(),
            result: result.clone(),
        });

        Ok(result)
    }
}

// Agent-side usage
let weather = mcp_call("weather", json!({
    "location": "Seattle"
})).await?;
```

## Security Model

1. Tools are registered with the host, not agents
2. Host maintains ACL of which agents can access which tools
3. All tool access is logged for audit
4. Tools run in separate processes for isolation
5. Resource limits can be applied per-tool

## References

- Model Context Protocol specification
- MCP SDK documentation
- WebAssembly System Interface (WASI) proposal
- "Capability-Based Security" papers
