---
layout: adr
title: "0005. MCP for External Tools"
date: 2025-01-31
status: proposed
categories: [Architecture, Integration]
tags: [mcp, tools, integration, external-systems]
---

# 0005. MCP for External Tools

Date: 2025-01-31

## Status

Proposed

## Context

Agents need to interact with external systems to be useful:
- Databases for data persistence and retrieval
- File systems for reading and writing files
- HTTP APIs for web service integration
- Command-line tools for system operations

We need a standardized way to provide these capabilities while maintaining:
- Security through controlled access
- Observability of all external interactions
- Consistent APIs across different tool types
- Easy extensibility for new tools

## Decision

We will use the Model Context Protocol (MCP) as the standard interface for all external tool access from agents.

Key aspects:
- All external system access goes through MCP servers
- Agents request tools via standardized MCP protocol
- Host controls which tools are available to which agents
- MCP servers handle authentication and authorization to external systems
- Full observability of all tool usage through OpenTelemetry

## Consequences

### Positive

- **Standardized interface**: Consistent API for all external tools
- **Security**: Controlled access through MCP server permissions
- **Observability**: All tool usage is traced and monitored
- **Extensibility**: Easy to add new tools via new MCP servers
- **Language agnostic**: MCP servers can be written in any language
- **Reusability**: MCP servers can be shared across projects

### Negative

- **Additional complexity**: Extra layer between agents and external systems
- **Performance overhead**: Network calls for all tool access
- **Dependency**: Relies on MCP server availability
- **Learning curve**: Developers need to understand MCP protocol

### Mitigations

- Provide high-performance local MCP servers for common tools
- Cache frequently accessed data where appropriate
- Implement circuit breakers for MCP server failures
- Excellent documentation and examples for MCP integration

## Tool Categories

### Core Tools (Built-in MCP Servers)
- File system operations (read, write, list)
- HTTP client for API calls
- Database connections (SQL, NoSQL)
- Command execution (controlled)

### Extended Tools (Community MCP Servers)
- Cloud provider APIs (AWS, GCP, Azure)
- Version control (Git operations)
- Message queues (RabbitMQ, Kafka)
- Container orchestration (Docker, Kubernetes)

## Implementation Notes

```rust
// Agent requests tool via MCP
#[derive(Serialize)]
struct ToolRequest {
    tool: String,
    parameters: Value,
}

impl Agent {
    async fn use_tool(&self, request: ToolRequest) -> Result<Value, Error> {
        // Automatically traced and permission-checked
        self.mcp_client.call_tool(request).await
    }
    
    async fn read_file(&self, path: &str) -> Result<String, Error> {
        let request = ToolRequest {
            tool: "filesystem/read".to_string(),
            parameters: json!({ "path": path }),
        };
        
        let response = self.use_tool(request).await?;
        Ok(response["content"].as_str().unwrap().to_string())
    }
}

// Host configuration for agent tools
struct AgentConfig {
    allowed_tools: Vec<String>,
    mcp_servers: HashMap<String, McpServerConfig>,
}
```

## Security Model

1. **Agent permissions**: Each agent has explicit tool allowlist
2. **MCP server auth**: Servers handle authentication to external systems
3. **Request validation**: All tool requests are validated and logged
4. **Resource limits**: Agents can be limited in tool usage (rate limits, quotas)

## Alternatives Considered

### Direct System Calls
- **Pros**: Maximum performance, simple integration
- **Cons**: No security isolation, difficult to trace and control

### Custom Plugin System
- **Pros**: Tailored to our needs, tight integration
- **Cons**: Reinventing the wheel, limited ecosystem

### Language-Specific FFI
- **Pros**: Native performance, existing libraries
- **Cons**: Language lock-in, security challenges

### REST APIs Only
- **Pros**: Universal standard, good tooling
- **Cons**: Not designed for tool abstraction, limited standardization

## References

- Model Context Protocol Specification
- Anthropic MCP documentation
- "Security Patterns for Microservice Architectures"