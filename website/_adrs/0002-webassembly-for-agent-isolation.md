______________________________________________________________________

## title: "0002. WebAssembly for Agent Isolation" date: 2025-07-31 status: proposed layout: adr

## categories: [Architecture, Technology]

# 0002. WebAssembly for Agent Isolation

Date: 2025-01-31

## Status

Proposed

## Context

Multi-agent systems require strong isolation between agents to ensure:

- Security: One agent cannot access another's memory or state
- Fault tolerance: Agent crashes don't affect the host or other agents
- Resource control: Agents can be limited in CPU, memory, and I/O
- Portability: Agents can be written in any language

Traditional approaches like OS processes are too heavyweight, while
language-level isolation (e.g., JavaScript isolates) ties us to specific
runtimes.

## Decision

We will use WebAssembly (WASM) as the execution environment for all agents in
Caxton.

Key aspects:

- Agents are compiled to WASM modules
- Each agent runs in its own WASM instance with isolated memory
- Communication happens only through well-defined message passing interfaces
- The host controls all resource access through WASM imports
- Agents can be written in any language that compiles to WASM

## Consequences

### Positive

- **True isolation**: WASM's sandboxing ensures agents cannot interfere with
  each other
- **Language agnostic**: Write agents in Rust, Go, AssemblyScript, C++, etc.
- **Deterministic execution**: WASM's semantics enable reproducible behavior
- **Fast instantiation**: WASM modules start in milliseconds
- **Small footprint**: WASM modules are compact and efficient
- **Industry standard**: Growing ecosystem and tooling support

### Negative

- **Limited system access**: Agents can only do what the host explicitly allows
- **Performance overhead**: ~10-50% slower than native code
- **Debugging challenges**: WASM debugging tools are still maturing
- **No threading**: WASM doesn't support shared memory parallelism (yet)
- **Binary size**: WASM modules can be larger than native code

### Mitigations

- Provide rich host functions for common operations (via MCP)
- Use WASM optimizers (wasm-opt) to improve performance
- Invest in debugging tools and source map support
- Use WASM instance pooling to amortize startup costs
- Document clear patterns for async operations without threads

## Alternatives Considered

### OS Processes

- **Pros**: Maximum isolation, existing tooling
- **Cons**: High overhead, slow startup, complex IPC

### Docker/Containers

- **Pros**: Good isolation, rich ecosystem
- **Cons**: Heavyweight, requires container runtime, slow startup

### Language-specific Isolation (V8 Isolates, Erlang Processes)

- **Pros**: Fast, efficient for specific languages
- **Cons**: Locks us into specific language/runtime

### Shared Memory Threads

- **Pros**: Maximum performance
- **Cons**: No isolation, complex concurrency bugs

### Native Plugins

- **Pros**: Full system access, maximum performance
- **Cons**: No isolation, security risks, platform-specific

## Implementation Notes

```rust
// Example agent interface
#[no_mangle]
pub extern "C" fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    // Agent processes message and returns status
}

#[no_mangle]
pub extern "C" fn get_response(buf_ptr: *mut u8, buf_len: usize) -> i32 {
    // Agent writes response to provided buffer
}
```

The host will:

1. Load WASM modules using wasmtime or similar runtime
2. Provide imports for MCP tool access
3. Marshal messages in/out of WASM memory
4. Enforce resource limits (memory, CPU time)

## References

- WebAssembly specification
- Wasmtime documentation
- "WebAssembly: A New Hope" by Lin Clark
- WASI (WebAssembly System Interface) proposal
