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
- **Observable by design**: Every interaction is recorded as an event
- **Minimal core**: Just enough to be useful, not enough to be constraining
- **Progressive complexity**: Start simple, add sophistication as needed

## Core Concepts

Caxton provides just three things:

1. **Event Log**: Append-only record of what happened
2. **Agent Runner**: Executes WASM modules in isolation  
3. **Message Router**: Delivers messages between agents

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
    
    // Watch events stream for debugging
    let mut events = host.events();
    while let Some(event) = events.next().await {
        println!("Event: {:?}", event);
    }
    
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
│                          Event Log                              │
│              (Every message, decision, action)                  │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
                        Time-Travel Debugging
```

## Event Sourcing for Observability

Caxton records every agent interaction as an event, giving you complete visibility into your multi-agent systems. When something goes wrong (and it will), you can replay the entire conversation to understand exactly what happened.

```rust
// Every interaction becomes a debuggable event
enum AgentEvent {
    MessageSent { from: AgentId, to: AgentId, content: FipaMessage },
    MessageReceived { agent: AgentId, message: FipaMessage },
    ToolInvoked { agent: AgentId, tool: McpTool, params: Value },
    ToolCompleted { agent: AgentId, result: Result<Value, Error> },
    AgentFailed { agent: AgentId, error: String },
}
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

We're intentionally NOT building:
- Complex orchestration languages
- Built-in workflow engines  
- Agent hierarchies or permissions systems
- Distributed consensus protocols
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