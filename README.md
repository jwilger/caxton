# Caxton

Observable multi-agent orchestration in Rust.

## Why Caxton?

- 🚀 **Fast**: 100K+ messages/second on a single core
- 🔍 **Observable**: Debug any issue in < 5 minutes  
- 📦 **Small**: < 10 public APIs to learn
- 🛡️ **Safe**: Zero message loss guarantee

Caxton enables developers to build reliable, observable multi-agent systems by providing a minimal, composable framework that makes agent communication transparent and debuggable.

## The Problem

Most agent frameworks either:
- Lock you into specific AI/LLM providers
- Hide communication complexity (making debugging impossible)
- Impose rigid architectural patterns
- Require complex distributed systems knowledge

Caxton takes a different approach:
- **Agent-agnostic**: Works with any agent implementation
- **Observable by design**: Comprehensive logging and OpenTelemetry tracing
- **Minimal core**: Just enough to be useful, not enough to be constraining
- **Progressive complexity**: Start simple, add sophistication as needed

## Core Concepts

Caxton provides just three things:

1. **Agent Runtime**: WebAssembly-based isolation and execution
2. **Message Router**: FIPA protocol implementation for agent communication
3. **Observability Layer**: Structured logging and OpenTelemetry integration

That's it. Everything else is a library built on top. We don't tell you how to use these primitives - that's your job.

## Quick Start

```rust
use caxton::{Caxton, AgentHost, FipaMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Caxton host
    let host = Caxton::new().await?;
    
    // Load your agents (WebAssembly modules)
    let alice = host.spawn_agent(include_bytes!("alice.wasm")).await?;
    let bob = host.spawn_agent(include_bytes!("bob.wasm")).await?;
    
    // Send a message (using FIPA ACL format)
    let msg = FipaMessage::request()
        .sender(alice)
        .receiver(bob)
        .content("Hello, Bob!")
        .build();
    
    host.send_message(msg).await?;
    
    // Access traces for debugging
    let trace_id = msg.trace_id();
    println!("Message sent with trace_id: {}", trace_id);
    
    // Logs and traces are automatically collected via OpenTelemetry
    
    Ok(())
}
```

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Agent Alice   │     │   Agent Bob     │     │   Agent Carol   │
│   (WASM)        │     │   (WASM)        │     │   (WASM)        │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         │     FIPA Messages     │                       │
         ▼                       ▼                       ▼
┌────────────────────────────────────────────────────────────────┐
│                        Message Router                           │
│                   (Async, Zero-Copy, Fast)                     │
└────────────────────────────────┬───────────────────────────────┘
                                 │
                                 ▼
┌────────────────────────────────────────────────────────────────┐
│                     Observability Layer                         │
│        (OpenTelemetry Traces, Metrics, Structured Logs)        │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
                        Debug with Your Favorite Tools
```

## Built-in Observability

Caxton provides comprehensive observability through OpenTelemetry, giving you complete visibility into your multi-agent systems. Every message, tool invocation, and agent interaction is traced and logged with rich context.

```rust
// Rich context in every log entry
info!(
    agent_id = %agent.id,
    message_type = "fipa.request",
    correlation_id = %msg.correlation_id,
    "Agent received message"
);

// Automatic distributed tracing
let span = info_span!("agent.handle_message",
    agent_id = %agent.id,
    message_id = %msg.id
);
```

## Type Safety Without Complexity

Caxton uses Rust's type system to prevent common errors at compile time. Start with simple types and add sophistication as your agents grow more complex.

```rust
// Agent states encoded in types
pub struct Agent<State> {
    id: AgentId,
    wasm_module: Module,
    _state: PhantomData<State>,
}

// Only initialized agents can receive messages
impl Agent<Initialized> {
    pub fn receive(&mut self, msg: FipaMessage) -> Result<(), AgentError> {
        // Type system ensures this is only called on ready agents
    }
}
```

## Performance

Caxton is fast by default. The simple API hides a sophisticated async runtime that handles millions of messages without breaking a sweat:

- Work-stealing executors for CPU efficiency
- Bounded channels with automatic back-pressure
- Zero-copy message passing where possible
- WASM instance pooling for rapid agent spawning

## Building Agents

Agents are WebAssembly modules that implement a simple interface:

```rust
// Your agent code (compiled to WASM)
#[no_mangle]
pub extern "C" fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    // Deserialize FIPA message
    let message = FipaMessage::from_bytes(msg_ptr, msg_len);
    
    // Your agent logic here
    match message.performative() {
        Performative::Request => handle_request(message),
        Performative::Inform => handle_inform(message),
        _ => Ok(()),
    }
}
```

## External Tools via MCP

Agents can access external tools through the Model Context Protocol:

```rust
// In your agent
let result = mcp_call("web_search", json!({
    "query": "latest news on quantum computing"
})).await?;
```

## What Caxton is NOT

## What's In Scope

Caxton provides:
- **FIPA agent interaction protocols** including Contract Net Protocol (CNP) for task delegation
- **Agent negotiation and coordination** through typed message passing
- **Observable agent communications** with full tracing support
- **WebAssembly isolation** for secure multi-tenant agent hosting

## What's Out of Scope

We're intentionally NOT building:
- Complex orchestration languages
- Built-in workflow engines  
- Agent hierarchies or permissions systems
- **Infrastructure-level consensus protocols** (Raft, Paxos, PBFT) - use etcd/Consul for distributed state
- Message transformation pipelines

These can all be built as libraries on top of Caxton's simple primitives.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Key areas where we need help:
- Example agents and patterns
- Performance optimizations
- Language bindings for agent development
- Debugging and visualization tools

## License

Caxton is dual-licensed under Apache 2.0 and MIT licenses.

## Acknowledgments

Caxton is inspired by the Actor model, the BEAM VM's approach to fault tolerance, and decades of research in multi-agent systems. Special thanks to the WebAssembly and Rust async communities.