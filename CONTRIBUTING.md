# Contributing to Caxton

Thank you for your interest in contributing to Caxton! This guide will help you get started with contributing to the Caxton application server.

## Getting Started

Caxton is an application server for multi-agent systems, not a library. Contributors work on the server implementation, CLI tools, and deployment infrastructure - not on end-user agent code.

### Prerequisites

- **Rust** (latest stable) - for server development
- **Protocol Buffers compiler** - for API definitions
- **Docker** - for running dependencies
- **Nix** (optional) - for reproducible development environment

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/caxton.git
cd caxton

# Option 1: Use Nix for automatic setup
nix develop

# Option 2: Manual setup
cargo install cargo-nextest cargo-watch
docker-compose up -d  # PostgreSQL for development

# Verify setup
cargo test --workspace
cargo run -- server --dev
```

## Architecture Overview

Caxton follows a clear architectural pattern:

```
caxton/
├── caxton-server/       # Main server binary
├── caxton-cli/          # CLI tool
├── caxton-api/          # gRPC API definitions
├── caxton-runtime/      # WebAssembly runtime
├── caxton-router/       # Message routing engine
└── caxton-observability/# Logging, tracing, metrics
```

**Key Principles**:
- **Type-driven development**: Types first, implementation second
- **Test-driven development**: Write tests before implementation
- **Observable by default**: Every operation must be traceable

## Making Contributions

### 1. Find Something to Work On

- Check [GitHub Issues](https://github.com/yourusername/caxton/issues) for `good-first-issue` labels
- Review the [ROADMAP.md](ROADMAP.md) for upcoming features
- Join discussions about design decisions

### 2. Design First

Before implementing, discuss your approach:

1. **Comment on the issue** with your proposed solution
2. **Create an ADR** for significant changes (see `docs/adr/template.md`)
3. **Get feedback** from maintainers

### 3. Implementation Guidelines

#### Type-Driven Development

```rust
// GOOD: Define types that make illegal states unrepresentable
enum AgentState {
    Initializing { started_at: Instant },
    Running { capabilities: Vec<Capability> },
    Failed { error: AgentError, failed_at: Instant },
}

// BAD: Stringly-typed, nullable fields
struct Agent {
    state: String,  // "init", "running", "failed"
    error: Option<String>,
    capabilities: Option<Vec<String>>,
}
```

#### Test-Driven Development

Follow the red-green-refactor cycle:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn agent_should_start_in_initializing_state() {
        // Red: Write failing test first
        let agent = Agent::new("test-agent");
        assert!(matches!(agent.state(), AgentState::Initializing { .. }));
    }
}

// Green: Implement minimal code to pass
// Refactor: Improve design while keeping tests green
```

#### Observability

Every operation must include:

```rust
#[instrument(skip(wasm_module))]
async fn deploy_agent(
    name: &str,
    wasm_module: &[u8],
) -> Result<AgentId, DeployError> {
    info!("Deploying agent", agent_name = name);
    
    // Implementation with structured logging
    let agent_id = AgentId::new();
    
    info!(
        "Agent deployed successfully",
        agent_id = %agent_id,
        deployment_time_ms = timer.elapsed().as_millis()
    );
    
    Ok(agent_id)
}
```

### 4. Submitting Changes

1. **Create a feature branch**: `git checkout -b issue-42-descriptive-name`
2. **Write clear commits**: Follow [Conventional Commits](https://www.conventionalcommits.org/)
3. **Run all checks**: `cargo test && cargo clippy && cargo fmt`
4. **Update documentation**: Include any API or behavior changes
5. **Submit PR**: Reference the issue and provide context

#### PR Template

```markdown
## Summary
Brief description of changes

## Related Issue
Closes #42

## Changes
- Added X to improve Y
- Refactored Z for better performance

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Documentation
- [ ] API documentation updated
- [ ] ADR created (if applicable)
```

## Code Review Process

Expect feedback on:

1. **Type safety**: Are illegal states prevented?
2. **Test coverage**: Are edge cases tested?
3. **Performance**: Any benchmarks for critical paths?
4. **Observability**: Can we debug this in production?
5. **API design**: Is it consistent with existing patterns?

## Development Workflow

### Running Tests

```bash
# All tests
cargo nextest run --workspace

# Specific module
cargo nextest run -p caxton-runtime

# With coverage
cargo llvm-cov nextest --workspace
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=caxton=debug cargo run -- server

# Trace specific operation
RUST_LOG=caxton::router=trace cargo run
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Profile with flamegraph
cargo flamegraph --bench router_bench
```

## Release Process

Releases follow semantic versioning:

1. **Feature branches** merge to `main`
2. **Release candidates** tagged as `v1.0.0-rc.1`
3. **Final releases** tagged as `v1.0.0`
4. **Binaries** built and published via GitHub Actions

## Community

- **Discord**: Real-time discussions
- **GitHub Discussions**: Design decisions
- **Monthly calls**: First Thursday of each month

## Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Be kind, considerate, and respectful.

## Recognition

Contributors are recognized in:
- Release notes
- Project README
- Annual contributor spotlight

Thank you for helping make Caxton better! 🚀