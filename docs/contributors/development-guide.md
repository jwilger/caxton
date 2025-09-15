---
title: "Development Guide"
date: 2025-01-14
layout: page
categories: [Contributors]
---

This comprehensive guide provides best practices, tools, and workflows for
developing with Caxton. Whether you're building agents, extending the
orchestrator, or integrating with external systems, this guide will help you be
productive and successful.

## Quick Start

### Development Environment Setup

#### Prerequisites

```bash
# Required tools
rustc >= 1.70.0
cargo >= 1.70.0
docker >= 20.10
docker-compose >= 2.0
node >= 18.0 (for WebAssembly tooling)

# Optional but recommended
just            # Command runner
watchexec       # File watcher
cargo-watch     # Rust file watcher
cargo-expand    # Macro expansion
cargo-flamegraph # Performance profiling
```

#### Initial Setup

```bash
# Clone the repository
git clone https://github.com/your-org/caxton.git
cd caxton

# Install dependencies
cargo build --all-features

# Install development tools
cargo install cargo-watch cargo-expand cargo-flamegraph

# Setup pre-commit hooks
./scripts/setup-hooks.sh

# Run tests to verify setup
cargo test --all

# Start development environment
docker-compose up -d
```

### Your First Agent

#### Agent Scaffolding

```bash
# Use the agent generator
cargo run --bin caxton -- agent new my-agent \
  --language rust \
  --template basic \
  --capabilities process,communicate

# This creates:
# agents/my-agent/
# ├── Cargo.toml
# ├── src/
# │   ├── lib.rs
# │   └── main.rs
# ├── tests/
# │   └── integration.rs
# └── README.md
```

#### Basic Agent Implementation

```rust
use caxton_sdk::prelude::*;

#[derive(Agent)]
pub struct MyAgent {
    state: AgentState,
}

#[agent_capability]
impl MyAgent {
    #[message_handler(performative = "request")]
    async fn handle_request(&mut self, msg: Message) -> Result<Message> {
        // Process the request
        let result = self.process(msg.content)?;

        // Return response
        Ok(Message::inform(result))
    }

    #[message_handler(performative = "query")]
    async fn handle_query(&mut self, msg: Message) -> Result<Message> {
        let data = self.query_internal(msg.content)?;
        Ok(Message::inform(data))
    }
}

// Entry point for WebAssembly
#[no_mangle]
pub extern "C" fn init() -> *mut dyn Agent {
    Box::into_raw(Box::new(MyAgent::default()))
}
```

## Development Workflows

### 1. Test-Driven Development (TDD)

#### Writing Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use caxton_test::prelude::*;

    #[tokio::test]
    async fn test_agent_processes_request() {
        // Arrange
        let mut agent = MyAgent::new();
        let request = Message::request("calculate", json!({
            "operation": "add",
            "a": 5,
            "b": 3
        }));

        // Act
        let response = agent.handle_request(request).await.unwrap();

        // Assert
        assert_eq!(response.performative, Performative::Inform);
        assert_eq!(response.content["result"], 8);
    }
}
```

#### Test Categories

- **Unit Tests**: Test individual components
- **Integration Tests**: Test agent interactions
- **Property Tests**: Test invariants with random data
- **Snapshot Tests**: Test output consistency

### 2. Debugging Agents

#### Local Debugging

```rust
// Enable debug logging
use tracing::{debug, info, warn, error};

impl MyAgent {
    async fn process(&mut self, data: Value) -> Result<Value> {
        debug!("Processing data: {:?}", data);

        let result = match self.internal_process(data) {
            Ok(r) => {
                info!("Processing successful");
                r
            }
            Err(e) => {
                error!("Processing failed: {}", e);
                return Err(e);
            }
        };

        debug!("Result: {:?}", result);
        Ok(result)
    }
}
```

#### Remote Debugging

```bash
# Enable remote debugging
cargo run --features debug -- \
  --debug-port 9229 \
  --debug-wait

# Connect with debugger
lldb target/debug/my-agent
(lldb) process connect connect://localhost:9229
```

#### Tracing and Observability

```rust
use tracing::instrument;

#[instrument(skip(self))]
async fn complex_operation(&mut self, input: String) -> Result<Output> {
    let span = tracing::span!(Level::INFO, "processing");
    let _enter = span.enter();

    // Operations are automatically traced
    let parsed = self.parse(input)?;
    let processed = self.process(parsed)?;
    let output = self.format(processed)?;

    Ok(output)
}
```

### 3. Performance Optimization

#### Profiling Tools

```bash
# CPU profiling with flamegraph
cargo flamegraph --bin my-agent -- --bench

# Memory profiling with valgrind
valgrind --tool=massif target/debug/my-agent
ms_print massif.out.<pid>

# Heap profiling with jemallocator
MALLOC_CONF=prof:true,prof_prefix:jeprof.out \
  cargo run --release --features jemalloc
```

#### Common Optimizations

```rust
// 1. Use efficient data structures
use rustc_hash::FxHashMap; // Faster than std::HashMap for small keys

// 2. Minimize allocations
use smallvec::SmallVec;
type SmallString = SmallVec<[u8; 32]>; // Stack-allocated for small strings

// 3. Batch operations
pub async fn batch_process(items: Vec<Item>) -> Vec<Result<Output>> {
    // Process in parallel with controlled concurrency
    futures::stream::iter(items)
        .map(|item| async move { process_item(item).await })
        .buffer_unordered(10) // Process 10 items concurrently
        .collect()
        .await
}

// 4. Use zero-copy where possible
use bytes::Bytes;
pub fn process_bytes(data: Bytes) -> Result<Bytes> {
    // Avoid copying data
    Ok(data.slice(10..20))
}
```

## Best Practices

### 1. Code Organization

#### Project Structure

```text
caxton/
├── src/
│   ├── orchestrator/     # Core orchestrator
│   ├── agents/          # Built-in agents
│   ├── api/            # API definitions
│   └── common/         # Shared utilities
├── agents/             # External agents
├── tests/             # Integration tests
├── benches/           # Benchmarks
├── examples/          # Usage examples
└── docs/             # Documentation
```

#### Module Guidelines

- Keep modules focused and < 500 lines
- Use clear, descriptive names
- Group related functionality
- Minimize public API surface

### 2. Error Handling

#### Error Design

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Processing failed: {0}")]
    ProcessingError(#[from] ProcessingError),

    #[error("Resource limit exceeded: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}

// Use Result type alias for consistency
pub type Result<T> = std::result::Result<T, AgentError>;
```

#### Error Handling Patterns

```rust
// 1. Early returns for cleaner code
pub fn process(input: Input) -> Result<Output> {
    let validated = validate(input)?;
    let transformed = transform(validated)?;
    let output = finalize(transformed)?;
    Ok(output)
}

// 2. Context wrapping for better errors
use anyhow::{Context, Result};

pub async fn load_config(path: &str) -> Result<Config> {
    let content = tokio::fs::read_to_string(path)
        .await
        .context("Failed to read config file")?;

    let config = serde_json::from_str(&content)
        .context("Failed to parse config JSON")?;

    Ok(config)
}

// 3. Custom error recovery
pub async fn resilient_operation() -> Result<Value> {
    let result = match primary_operation().await {
        Ok(v) => v,
        Err(e) if e.is_retriable() => {
            warn!("Primary failed, trying fallback: {}", e);
            fallback_operation().await?
        }
        Err(e) => return Err(e),
    };
    Ok(result)
}
```

### 3. Testing Strategies

#### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod unit {
        use super::*;

        #[test]
        fn test_parsing() {
            // Unit tests
        }
    }

    mod integration {
        use super::*;

        #[tokio::test]
        async fn test_full_flow() {
            // Integration tests
        }
    }

    mod property {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_invariants(input in any::<String>()) {
                // Property-based tests
            }
        }
    }
}
```

#### Test Utilities

```rust
// Test fixture builder
pub struct AgentTestBuilder {
    config: Config,
    state: State,
}

impl AgentTestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn with_state(mut self, state: State) -> Self {
        self.state = state;
        self
    }

    pub fn build(self) -> MyAgent {
        MyAgent::new(self.config, self.state)
    }
}

// Usage
let agent = AgentTestBuilder::new()
    .with_config(test_config())
    .with_state(initial_state())
    .build();
```

### 4. Documentation

#### Code Documentation

````rust
/// Processes incoming messages according to business rules.
///
/// # Arguments
/// * `message` - The message to process
///
/// # Returns
/// * `Ok(Response)` - Successfully processed response
/// * `Err(ProcessingError)` - If processing fails
///
/// # Examples
/// ```
/// let msg = Message::request("action", json!({"key": "value"}));
/// let response = agent.process_message(msg).await?;
/// assert_eq!(response.performative, Performative::Inform);
/// ```
///
/// # Panics
/// Panics if the agent is not initialized
pub async fn process_message(&mut self, message: Message) -> Result<Response> {
    // Implementation
}
````

#### README Template

````markdown
# Agent Name

Brief description of what this agent does.

## Features

- Feature 1
- Feature 2

## Installation

```bash
cargo install agent-name
```

## Usage

```rust
use agent_name::Agent;

let agent = Agent::new();
agent.start().await?;
```

## Configuration

| Option  | Description     | Default |
| ------- | --------------- | ------- |
| timeout | Request timeout | 30s     |

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md)
````

## Development Tools

### 1. CLI Commands

```bash
# Development commands
caxton agent new <name>          # Create new agent
caxton agent test <name>         # Test agent
caxton agent build <name>        # Build agent
caxton agent deploy <name>       # Deploy agent

# Debugging commands
caxton debug trace <agent-id>    # Trace agent execution
caxton debug logs <agent-id>     # Show agent logs
caxton debug metrics <agent-id>  # Show agent metrics

# Management commands
caxton list agents               # List all agents
caxton describe <agent-id>       # Show agent details
caxton restart <agent-id>        # Restart agent
```

### 2. IDE Integration

#### VS Code Extensions

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb",
    "mutantdino.resourcemonitor"
  ]
}
```

#### Settings

```json
{
  "rust-analyzer.cargo.features": ["all"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.enable": true,
  "editor.formatOnSave": true
}
```

### 3. Continuous Integration

#### GitHub Actions Workflow

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Format Check
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Test
        run: cargo test --all-features

      - name: Build
        run: cargo build --release
```

## Troubleshooting

### Common Issues

#### Issue: Agent Won't Start

```bash
# Check logs
caxton logs <agent-id> --tail 100

# Verify configuration
caxton validate config agents/<agent-name>/config.toml

# Check resource limits
caxton describe <agent-id> --resources
```

#### Issue: Performance Problems

```bash
# Profile CPU usage
cargo flamegraph --bin <agent-name>

# Check memory usage
valgrind --tool=massif target/debug/<agent-name>

# Monitor metrics
caxton metrics <agent-id> --interval 1s
```

#### Issue: Message Not Received

```rust
// Add debug logging
#[instrument]
async fn handle_message(&mut self, msg: Message) -> Result<()> {
    debug!("Received message: {:?}", msg);
    // Process message
}

// Check message routing
caxton trace message <conversation-id>
```

## Learning Resources

### Tutorials

1. [Building Your First Agent](../tutorials/first-agent.md)
2. [Agent Communication Patterns](../concepts/communication-patterns.md)
3. [Advanced WebAssembly Agents](../tutorials/wasm-agents.md)

### Examples

- [Simple Echo Agent](../../examples/echo-agent)
- [Database Agent](../../examples/database-agent)
- [ML Inference Agent](../../examples/ml-agent)
- [Workflow Orchestration](../../examples/workflow)

### API Documentation

- [Rust SDK Docs](https://docs.rs/caxton-sdk)
- [REST API Reference](../api/rest-api.md)

## Community and Support

### Getting Help

- **Documentation**: [docs.caxton.io](https://docs.caxton.io)
- **Discord**: [discord.gg/caxton](https://discord.gg/caxton)
- **GitHub Issues**:
  [github.com/caxton/caxton/issues](https://github.com/caxton/caxton/issues)
- **Stack Overflow**: Tag questions with `caxton`

### Contributing

We welcome contributions! See [CONTRIBUTING.md](../../CONTRIBUTING.md) for
guidelines.

### Code of Conduct

Please read our [Code of Conduct](../../CODE_OF_CONDUCT.md) before
participating.

## References

- [Architecture Decision Records](../adrs/)
- [Security Guidelines](../operations/devops-security-guide.md)
- [Performance Benchmarks](../operations/performance-tuning.md)
- [State Recovery Patterns](../operations/state-recovery-patterns.md)
