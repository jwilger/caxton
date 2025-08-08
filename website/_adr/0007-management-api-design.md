---
title: "0007. Management API Design"
date: 2025-08-03
status: proposed
layout: adr
categories: [Architecture, Technology]
---

# 0007. Management API Design

Date: 2025-08-03

## Status

Proposed

## Context

With Caxton's pivot to an application server architecture (ADR-0006), we need a well-designed management API that enables programmatic control of the multi-agent system. This API must be accessible to developers regardless of their language choice while maintaining the performance and type safety that Rust provides internally.

The API design must balance several concerns:
- **Language agnostic**: Accessible from any programming language
- **Performance**: Minimal overhead for high-frequency operations
- **Type safety**: Preserve Rust's guarantees across the API boundary
- **Observability**: Built-in instrumentation for debugging distributed systems
- **Evolution**: Ability to extend without breaking existing clients

## Decision Drivers

- **Industry standards**: gRPC is the de facto standard for high-performance APIs
- **REST familiarity**: Many developers expect REST APIs for tooling integration
- **Type safety**: Need to preserve Rust's type guarantees across API boundaries
- **Performance requirements**: < 1ms overhead for local API calls
- **Debugging needs**: Must support distributed tracing and structured logging

## Decision

We will implement a dual-protocol API architecture:

### 1. gRPC as Primary Protocol

```protobuf
service CaxtonManagement {
  // Agent lifecycle management
  rpc DeployAgent(DeployAgentRequest) returns (DeployAgentResponse);
  rpc UndeployAgent(UndeployAgentRequest) returns (UndeployAgentResponse);
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);

  // Message operations
  rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);
  rpc SubscribeMessages(SubscribeRequest) returns (stream Message);

  // Health and monitoring
  rpc Health(HealthRequest) returns (HealthResponse);
  rpc Metrics(MetricsRequest) returns (MetricsResponse);
}
```

### 2. REST Gateway via gRPC-Gateway

- Auto-generated from gRPC definitions
- OpenAPI/Swagger documentation
- JSON request/response format
- WebSocket support for streaming operations

### 3. API Design Principles

**Resource-Oriented Design**:
```
/api/v1/agents                    # Agent collection
/api/v1/agents/{id}              # Individual agent
/api/v1/agents/{id}/messages     # Agent's messages
/api/v1/messages                 # System-wide message stream
```

**Structured Error Handling**:
```protobuf
message Error {
  string code = 1;        // Machine-readable error code
  string message = 2;     // Human-readable description
  string trace_id = 3;    // Correlation ID for debugging
  map<string, string> metadata = 4;  // Additional context
}
```

**OpenTelemetry Integration**:
- Every API call creates a trace span
- Propagate trace context via headers
- Structured logging with trace correlation
- Prometheus metrics for all operations

## Consequences

### Positive

- **Language agnostic**: Any language with gRPC support can use Caxton
- **Type safety**: Protocol buffers provide schema validation
- **Performance**: Binary protocol with streaming support
- **REST compatibility**: Gateway provides familiar HTTP/JSON interface
- **Future proof**: gRPC supports backward/forward compatibility
- **Generated SDKs**: Automatic client generation for all languages
- **Built-in observability**: Tracing and metrics from day one

### Negative

- **Complexity**: Two protocols to maintain
- **Learning curve**: gRPC less familiar than REST
- **Tooling requirements**: Need protoc compiler for development
- **Debugging**: Binary protocol harder to inspect than JSON

### Mitigation Strategies

**Complexity**:
- Single source of truth (protobuf definitions)
- Automated gateway generation
- Comprehensive testing of both protocols

**Learning Curve**:
- Excellent documentation with examples
- Pre-built SDKs for popular languages
- REST gateway for initial exploration

**Debugging**:
- gRPC reflection for runtime introspection
- Request/response logging in development
- Trace-based debugging tools

## API Examples

### Deploy Agent (gRPC)
```rust
let request = DeployAgentRequest {
    name: "processor".to_string(),
    wasm_module: module_bytes,
    capabilities: vec!["messaging", "mcp-tools"],
    resources: Some(Resources {
        memory_limit: 100 * 1024 * 1024, // 100MB
        cpu_shares: 1024,
    }),
};

let response = client.deploy_agent(request).await?;
println!("Agent deployed: {}", response.agent_id);
```

### Deploy Agent (REST)
```bash
curl -X POST https://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "processor",
    "wasm_module": "base64...",
    "capabilities": ["messaging", "mcp-tools"],
    "resources": {
      "memory_limit": 104857600,
      "cpu_shares": 1024
    }
  }'
```

### Subscribe to Messages (gRPC Streaming)
```rust
let request = SubscribeRequest {
    filter: Some(MessageFilter {
        agent_id: Some("processor".to_string()),
        message_types: vec!["task", "result"],
    }),
};

let mut stream = client.subscribe_messages(request).await?;
while let Some(message) = stream.message().await? {
    println!("Received: {:?}", message);
}
```

### Health Check with Tracing
```bash
curl -X GET https://localhost:8080/api/v1/health \
  -H "X-Trace-Id: 550e8400-e29b-41d4-a716-446655440000"

{
  "status": "healthy",
  "version": "1.0.0",
  "agents": {
    "running": 42,
    "capacity": 100
  },
  "trace_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Observability Integration

Every API operation:
1. Creates a trace span with operation details
2. Logs structured data with trace correlation
3. Updates Prometheus metrics
4. Propagates context to downstream operations

Example trace structure:
```
caxton.api.deploy_agent (1.2ms)
├── caxton.wasm.validate (0.3ms)
├── caxton.runtime.create (0.5ms)
├── caxton.registry.register (0.2ms)
└── caxton.events.emit (0.1ms)
```

## Related Decisions

- ADR-0001: Observability-First Architecture - Defines tracing/metrics strategy
- ADR-0006: Application Server Architecture - Established need for management API
- ADR-0008: Agent Deployment Model - Uses this API for deployment operations
- ADR-0009: CLI Tool Design - Uses the gRPC API

## References

- [gRPC Best Practices](https://grpc.io/docs/guides/performance/)
- [Google API Design Guide](https://cloud.google.com/apis/design)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/reference/specification/)
- Bryan Cantrill's talks on API observability
