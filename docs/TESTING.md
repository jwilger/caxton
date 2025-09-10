# Testing Guide

## Running Tests

**IMPORTANT**: Always use `cargo nextest run` instead of `cargo test` for
running tests.

```bash
cargo nextest run
```

If nextest is not installed, install it first:

```bash
cargo install cargo-nextest --locked
```

This will run:

- **Unit tests**: Located in `#[cfg(test)]` modules within source files in
  `src/`
- **Integration tests**: Located in `tests/` directory

## Test Structure

### Unit Tests (37 tests)

Unit tests are embedded in each module using `#[cfg(test)]` blocks:

- `src/sandbox.rs`: Sandbox isolation and memory management tests
- `src/security.rs`: Security policy validation tests
- `src/resource_manager.rs`: Resource allocation and limit enforcement tests
- `src/host_functions.rs`: Host function registry tests
- `src/runtime/mod.rs`: Runtime lifecycle and agent management tests

### Integration Tests (10 tests)

Integration tests are in `tests/wasm_runtime_test.rs` and test the complete
system:

- Runtime initialization
- Agent sandboxing and isolation
- Memory and CPU limits enforcement
- Security features
- WebAssembly module loading
- Host function exposure
- Performance benchmarks

## Running Specific Tests

### Run only unit tests

```bash
cargo nextest run --lib
```

### Run only integration tests

```bash
cargo nextest run --tests
```

### Run tests for a specific module

```bash
cargo nextest run sandbox::tests
cargo nextest run security::tests
cargo nextest run resource_manager::tests
```

### Run a specific test

```bash
cargo nextest run test_sandbox_creation
cargo nextest run test_runtime_initialization
```

### Run tests with output

```bash
cargo nextest run --nocapture
```

### Run tests with backtrace on failure

```bash
RUST_BACKTRACE=1 cargo nextest run
```

### Run tests without stopping on first failure

```bash
cargo nextest run --no-fail-fast
```

## Test Coverage

Currently the project has comprehensive test coverage for:

- Security policy configurations and validation
- Resource limits and management
- Host function safety checks
- Sandbox creation and lifecycle
- Runtime initialization and agent management

## Known Test Issues

Some integration tests may fail due to incomplete implementation of async
runtime features. These are expected failures during development and will be
resolved as the implementation progresses.

## Continuous Testing

For development, you can use cargo-watch to automatically run tests on file
changes:

```bash
cargo install cargo-watch
cargo watch -x test
```
