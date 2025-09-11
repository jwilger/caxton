---
title: "Development Environment Setup"
description: "Complete guide for setting up Caxton development environment"
audience: contributors
categories: [Development, Setup]
layout: page
---

## Prerequisites

### Required Tools

**Rust Toolchain:**

- Rust stable (latest version via rustup)
- Cargo with standard extensions
- rustfmt for code formatting
- clippy for linting

**Development Tools:**

- cargo-nextest for testing (preferred over cargo test)
- cargo-edit for dependency management
- cargo-expand for macro inspection (optional)

**System Dependencies:**

- Git for version control
- A Unix-like environment (Linux, macOS, or WSL)
- Docker (optional, for containerized development)

### Nix Development Environment (Recommended)

Caxton includes a complete Nix development environment for reproducible
setup:

```bash
# Install Nix (if not already installed)
curl -L https://nixos.org/nix/install | sh

# Enter the development shell
nix develop

# Or for a one-off command
nix shell
```

**What the Nix environment provides:**

- Rust toolchain with all required components
- cargo-nextest, cargo-watch, cargo-expand, cargo-edit
- Development utilities (just, direnv support)
- Consistent versions across all contributors

### Manual Setup

If you prefer manual tool installation:

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required tools
cargo install cargo-nextest --locked
cargo install cargo-edit
cargo install cargo-expand  # optional

# Verify installation
cargo --version
cargo nextest --version
```

## Project Structure

```text
caxton/
├── src/                    # Core Rust source code
│   ├── lib.rs             # Library root
│   ├── domain_types.rs    # Domain types (nutype-based)
│   ├── domain/            # Domain model implementation
│   ├── sandbox.rs         # WebAssembly sandboxing
│   ├── security.rs        # Security policies
│   ├── resource_manager.rs # Resource management
│   ├── message_router/    # FIPA message routing
│   ├── runtime/           # Agent runtime environment
│   └── host_functions.rs  # Safe host function registry
├── tests/                 # Integration tests
│   ├── fixtures/          # WASM test modules
│   └── *.rs              # Test files
├── docs/                  # Documentation source
├── website/               # Jekyll website content
├── flake.nix             # Nix development environment
└── Cargo.toml            # Rust project configuration
```

## First-Time Setup

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/caxton.git
cd caxton
```

### 2. Enter Development Environment

```bash
# Using Nix (recommended)
nix develop

# Or source the environment manually
source ~/.cargo/env
```

### 3. Verify Everything Works

```bash
# Build the project
cargo build

# Run tests with nextest (mandatory)
cargo nextest run

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy
```

### 4. Optional: Install Development Helpers

```bash
# For continuous testing during development
cargo install cargo-watch

# For dependency management (if not using Nix)
cargo install cargo-edit
```

## Development Workflow

### Core Commands

**Building:**

```bash
cargo build                 # Debug build
cargo build --release      # Release build
cargo check                # Fast syntax/type checking
```

**Testing (Critical: Use nextest only):**

```bash
cargo nextest run           # Run all tests
cargo nextest run --lib     # Unit tests only
cargo nextest run --tests  # Integration tests only
cargo nextest run --nocapture  # Show test output
RUST_BACKTRACE=1 cargo nextest run  # With backtrace
```

**Code Quality:**

```bash
cargo fmt                   # Format code
cargo clippy               # Lint code (strict rules)
cargo clippy --fix         # Auto-fix lint issues
```

### Continuous Development

Use cargo-watch for automatic rebuilding:

```bash
# Auto-run tests on file changes
cargo watch -x nextest run

# Auto-check on changes
cargo watch -x check

# Auto-format and check
cargo watch -x fmt -x clippy
```

### Dependency Management

**Always use cargo-edit tools instead of manually editing Cargo.toml:**

```bash
# Add a new dependency (latest compatible version)
cargo add serde
cargo add tokio --features full

# Add development dependency
cargo add --dev proptest

# Remove dependency
cargo remove old-crate

# Upgrade dependencies
cargo upgrade
```

## Architecture Understanding

### Domain-Driven Design

Caxton follows Scott Wlaschin's "Domain Modeling Made Functional"
principles:

- **Core Philosophy**: "Make illegal states unrepresentable"
- **Type-driven design**: Use Rust's type system to encode business
  rules
- **Parse, don't validate**: Transform data into structured types at
  boundaries
- **Railway-oriented programming**: Model workflows as Result chains

### Key Domain Concepts

**Agent Types:**

- **Configuration Agents**: YAML+Markdown files (90% of use cases)
- **WASM Agents**: Compiled WebAssembly modules (10% of use cases)

**Core Architecture:**

- **Hybrid Runtime**: Config agents for ease, WASM for performance
- **FIPA Messaging**: Capability-based message routing
- **Embedded Memory**: SQLite+Candle for zero dependencies
- **Security Isolation**: WASM sandboxes for tools, not config agents

### Type System Patterns

```rust
// Domain primitives with nutype (eliminating primitive obsession)
#[nutype(
  sanitize(trim),
  validate(len(min = 1, max = 64)),
  derive(Clone, Debug, Eq, PartialEq, Display)
)]
pub struct AgentName(String);

// Phantom types for state machines
impl Agent<Unloaded> {
    pub fn load(self, module: WasmModule) -> Result<Agent<Loaded>, LoadError>
}

// Smart constructors with validation
impl AgentId {
    pub fn new() -> Self  // Always valid
    pub fn parse(s: &str) -> Result<Self, ParseError>  // Validated
}
```

## Testing Philosophy

### Test Types

**Unit Tests** (in `#[cfg(test)]` modules):

- Test individual functions and domain types
- Fast execution (< 1ms each)
- Focus on business logic correctness

**Integration Tests** (in `tests/` directory):

- Test complete workflows and system behavior
- Use WASM fixtures for realistic scenarios
- Cover error handling and edge cases

**Property-Based Tests** (using proptest):

- Generate random inputs for domain validation
- Verify invariants hold across input ranges
- Catch edge cases that manual testing misses

### Test-Driven Development (TDD)

Caxton uses strict Red→Green→Refactor cycles:

1. **Red**: Write a failing test that captures intended behavior
2. **Green**: Implement minimal code to make the test pass
3. **Refactor**: Improve code structure while maintaining tests

**Critical**: Always use `cargo nextest run` instead of `cargo test`.
Nextest provides better parallelization and clearer output.

### Testing Domain Types

Domain types created with `nutype` include automatic validation tests.
Add custom tests for business logic:

```rust
#[test]
fn test_agent_name_business_rules() {
    // Test valid names
    assert!(AgentName::try_new("web-scraper".to_string()).is_ok());

    // Test invalid names
    assert!(AgentName::try_new("".to_string()).is_err());
    assert!(AgentName::try_new("a".repeat(256)).is_err());
}
```

## Common Development Tasks

### Adding New Domain Types

1. **Identify primitive obsession**: Look for raw `String`, `usize`,
   `u64` in business logic
2. **Create domain type**: Use `nutype` with appropriate validation
3. **Update function signatures**: Replace primitives with domain types
4. **Add helper methods**: Provide conversion and utility functions
5. **Update tests**: Use domain types throughout test code

### Adding New Features

1. **Domain modeling first**: Define types that make illegal states
   unrepresentable
2. **Write failing tests**: Start with test-driven development
3. **Implement incrementally**: Small, focused commits
4. **Maintain type safety**: Never compromise on compile-time validation

### Performance Optimization

1. **Measure first**: Use criterion.rs for benchmarking
2. **Domain-aware optimization**: Optimize within domain constraints
3. **Preserve type safety**: Don't sacrifice safety for performance
4. **Document trade-offs**: Explain optimization decisions

## Code Quality Standards

### Mandatory Rules

**Never add allow attributes without team approval:**

```rust
// ❌ NEVER do this without explicit approval
#![allow(clippy::some_warning)]
#[allow(clippy::some_warning)]
```

**Always fix underlying issues instead of suppressing warnings.**

### Pre-commit Hooks

Pre-commit hooks are mandatory and must not be bypassed:

```bash
# ❌ Only in genuine emergencies
git commit --no-verify
```

**If pre-commit hooks fail, fix the issues - don't bypass them.**

### Code Formatting

```bash
# Format code (automatically applied by pre-commit hooks)
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy with project's strict rules
cargo clippy

# Auto-fix where possible
cargo clippy --fix
```

**All clippy warnings must be resolved.** Create GitHub issues for
systematic cleanup if needed.

## Troubleshooting

### Common Issues

**Build fails with dependency conflicts:**

```bash
# Clean and rebuild
cargo clean
cargo build
```

**Tests fail unexpectedly:**

```bash
# Ensure using nextest
cargo install cargo-nextest --locked
cargo nextest run

# Check for environment issues
RUST_BACKTRACE=1 cargo nextest run
```

**Clippy warnings about allow attributes:**

- Never suppress warnings without team approval
- Fix underlying issues instead
- Create systematic cleanup issues if needed

**Pre-commit hooks failing:**

- Fix reported issues instead of bypassing
- Use `cargo fmt` and `cargo clippy --fix` for auto-fixes
- Check for import organization and unused imports

### Getting Help

**Documentation:**

- Read relevant ADRs in `docs/_adrs/` for architectural decisions
- Check existing tests for usage patterns
- Review domain types documentation

**Community:**

- Open GitHub issues for bugs and feature requests
- Follow contribution guidelines in repository
- Ask questions in project discussions

## Next Steps

After completing development setup:

1. **Read the Architecture Overview**: Understand the system design
   and component interactions
2. **Study Domain Modeling Guide**: Learn the type-driven development
   approach
3. **Review Testing Guide**: Understand testing philosophy and
   practices
4. **Check Coding Standards**: Learn project conventions and quality
   requirements
5. **Start Contributing**: Begin with good first issues or
   documentation improvements

Welcome to Caxton development! The project prioritizes type safety,
clear domain modeling, and comprehensive testing to build a reliable
multi-agent orchestration platform.
