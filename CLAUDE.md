# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Testing

Always use `cargo nextest run` instead of `cargo test`:

```bash
cargo nextest run                    # Run all tests
cargo nextest run --lib             # Unit tests only
cargo nextest run --tests           # Integration tests only
cargo nextest run --nocapture       # Show test output
RUST_BACKTRACE=1 cargo nextest run  # With backtrace on failure
```

### Building and Linting

```bash
cargo build                          # Build the project
cargo build --release              # Release build
cargo check                         # Fast syntax/type checking
cargo clippy                        # Linting (strict rules enabled)
cargo fmt                          # Format code
```

### Development Tools

```bash
cargo watch -x test                 # Auto-run tests on changes
cargo expand                       # Macro expansion
cargo edit                         # Dependency management
```

## Architecture Overview

Caxton is a **multi-agent orchestration server** that provides WebAssembly-based agent isolation, FIPA-compliant messaging, and comprehensive observability. It runs as a standalone server process (like PostgreSQL or Redis) rather than a library.

### Core Components

- **Agent Runtime Environment**: Manages WebAssembly agent lifecycle with sandboxing and resource limits
- **FIPA Message Router**: High-performance async message routing between agents with conversation tracking
- **Security & Sandboxing**: WebAssembly isolation with CPU/memory limits and host function restrictions
- **Observability Layer**: Built-in structured logging, metrics (Prometheus), and distributed tracing (OpenTelemetry)
- **Agent Lifecycle Management**: Deployment strategies including blue-green, canary, and shadow deployments

### Domain Model Philosophy

The codebase follows **type-driven development** principles:

- Illegal states are unrepresentable through the type system
- Phantom types for agent state transitions (`Agent<Unloaded>` → `Agent<Loaded>` → `Agent<Running>`)
- Smart constructors with validation (e.g., `AgentId`, `Percentage`)
- Comprehensive error types with domain-specific variants
- nutype crate for eliminating primitive obsession

### Key Domain Types

Located in `src/domain_types.rs` and `src/domain/`:

- **Agent Identity**: `AgentId`, `AgentName` with validation
- **Resources**: `CpuFuel`, `MemoryBytes`, `MaxAgentMemory` with limits
- **Messaging**: `MessageId`, `ConversationId`, `Performative` for FIPA compliance
- **Deployment**: `DeploymentId`, `DeploymentStrategy`, `DeploymentStatus` for lifecycle management
- **Security**: `WasmSecurityPolicy`, `ResourceLimits`, `ValidationRule` for sandboxing

## Code Structure

### Core Modules

- `src/sandbox.rs` - WebAssembly agent sandboxing with resource limits
- `src/security.rs` - Security policies and validation
- `src/resource_manager.rs` - CPU/memory resource management
- `src/message_router/` - FIPA message routing with conversation management
- `src/runtime/` - Agent runtime environment
- `src/host_functions.rs` - Safe host function registry

### Agent Lifecycle Management

- `src/agent_lifecycle_manager.rs` - Orchestrates agent operations
- `src/deployment_manager.rs` - Handles deployment strategies
- `src/hot_reload_manager.rs` - Zero-downtime agent updates
- `src/wasm_module_validator.rs` - Validates WASM modules before deployment

### Domain Layer

- `src/domain/` - Rich domain types with business logic
- `src/domain_types.rs` - Primitive obsession elimination with nutype

### Test Structure

- **Unit tests**: In `#[cfg(test)]` modules within source files
- **Integration tests**: In `tests/` directory
- **Fixtures**: WASM test modules in `tests/fixtures/`
- **Property-based testing**: Using proptest for validation logic

## Testing Patterns

The project uses comprehensive testing with nextest for better performance:

- **47 total tests** (37 unit + 10 integration)
- Property-based testing for domain validation
- WASM fixture generation for integration tests
- Resource limit testing with controlled memory/CPU consumption

## Key Architectural Decisions

Reference the ADR documentation in `docs/adr/` for detailed rationales:

1. **Observability First** (ADR-0001): Every operation is instrumented with tracing
2. **WebAssembly Isolation** (ADR-0002): Agents run in secure WASM sandboxes
3. **FIPA Messaging** (ADR-0003): Standard agent communication protocols
4. **Type Safety** (ADR-0018): Domain types with nutype to prevent primitive obsession
5. **Coordination First** (ADR-0014): Lightweight coordination instead of shared databases

## Development Conventions

- **Error Handling**: Use `CaxtonResult<T>` with comprehensive domain errors
- **Tracing**: Instrument all async functions with `#[instrument]`
- **Resource Safety**: Always validate resource limits before allocation
- **State Machines**: Use phantom types for compile-time state validation
- **Testing**: Write property-based tests for validation logic
- **Dependency Management**: Always use package manager CLI tools (`cargo add`, `cargo remove`) to install/update dependencies. Never manually edit Cargo.toml version numbers. This ensures we use the latest compatible versions and avoid version conflicts.

## External Dependencies

The project uses Nix for development environment management:

- Rust toolchain: stable with clippy, rustfmt, rust-analyzer
- Development tools: cargo-nextest, cargo-watch, cargo-expand, cargo-edit
- Optional: just for task automation
- Ad-hoc: use `nix shell` to use a tool that is not currently installed.
- Update Flake: For tools you regularly use, consider adding them to the flake.nix file

## Rust Type-Driven Rules

- **Illegal states are unrepresentable**: prefer domain types over primitives.
- All new domain types use `nutype` with `sanitize(...)` and `validate(...)`. Derive at least: `Clone, Debug, Eq, PartialEq, Display`; add `Serialize, Deserialize` where needed.
- Prefer `Result<T, DomainError>` over panics. Panics only for truly unreachable states.

### Example

```rust
#[nutype(
  sanitize(trim),
  validate(len(min = 1, max = 64)),
  derive(Clone, Debug, Eq, PartialEq, Display)
)]
pub struct AgentName(String);
```

## Code Quality Enforcement - CRITICAL

**NEVER ADD ALLOW ATTRIBUTES** - This is a hard rule with zero exceptions without team approval.

- **NEVER** use `#![allow(clippy::...)]` or `#[allow(clippy::...)]` without explicit team approval
- **NEVER** bypass pre-commit hooks or ignore clippy warnings/errors
- **ALWAYS** fix the underlying issue causing the warning instead of suppressing it
- Pre-commit hooks MUST pass with `-D warnings` (treat warnings as errors)
- If build fails with warnings, FIX the warnings - don't suppress them
- When facing extensive warnings, create a GitHub issue and systematic plan to address them
- The only acceptable temporary measure is to create a focused story (like Story 053) to address them systematically

**Exception Process (Rare):**

1. Create GitHub issue explaining why the warning cannot be fixed
2. Get team approval in issue comments
3. Use the most targeted allow possible (function-level, not module-level)
4. Add comment explaining why and link to issue
5. Create follow-up story to address the underlying issue

**Automated Enforcement:**

- CI workflow automatically fails on any new allow attributes
- Code quality test (`test_no_clippy_allow_attributes`) enforces zero-tolerance
- Pre-commit hooks prevent allow attribute commits via pattern detection
- `RUSTFLAGS="-D warnings"` treats all clippy warnings as build failures

**Pre-commit Hook Enforcement:**

- Pre-commit hooks are MANDATORY and must not be bypassed
- Use `git commit --no-verify` only in genuine emergencies with team notification
- If pre-commit hooks fail, fix the issues - don't bypass them
- Allow attribute detection prevents commits with new suppressions

Testing Discipline (Kent Beck)
Work in strict Red → Green → Refactor -> Red -> Green -> Refactor -> ... loops with one failing test at a time.

Use `mcp__cargo__cargo_test` for all tests; treat clippy warnings as errors.

Functional Core / Imperative Shell
Put pure logic in the core (no I/O, no mutation beyond local scope).

Keep an imperative shell for I/O; inject dependencies via traits.

Model workflows as Result pipelines (railway style).

## GitHub PR Workflow

The SPARC workflow integrates with GitHub pull requests to ensure professional development practices:

### Story Development Flow

1. **Story Selection**: Choose from PLANNING.md
2. **Branch Creation**: `story-{id}-{slug}` feature branches
3. **Standard SPARC**: Research → Plan → Implement → Expert (with mandatory memory storage)
4. **PR Creation**: Draft PRs with comprehensive descriptions
5. **Review Loop**: Address feedback with Claude Code attribution
6. **Human Merge**: Only humans mark PRs ready-for-review

### MANDATORY Memory Storage (CRITICAL)

**Every SPARC phase MUST store knowledge in MCP memory for systematic improvement:**

- **Research Phase**: MUST store findings, sources, patterns, and API documentation
- **Planning Phase**: MUST store strategies, decisions, task breakdowns, and rationale
- **Implementation Phase**: MUST store TDD cycles, type improvements, patterns, and solutions
- **Expert Review Phase**: MUST store insights, quality patterns, and architectural analysis
- **PR Management**: MUST store workflow patterns, strategies, and outcomes

**Knowledge not stored is knowledge lost. This is not optional and will be enforced by the SPARC orchestrator.**

### Branch Management

- Feature branches: `story-001-wasm-runtime-foundation`
- Never commit to main during story development
- Branch/story mapping tracked in `.claude/branch.info`
- Automatic protection against closed PR branches

### PR Safety & Attribution

All GitHub comments from Claude Code include attribution:

```markdown
<!-- Generated by Claude Code -->
**🤖 Claude Code**: [response content]

*This comment was generated automatically...*
```

PRs created in **draft status only** - humans control ready-for-review.

### Commands & Agents

Primary commands:

- `/sparc` - Full story workflow with PR integration
- `/sparc/pr` - Create draft PR for completed story
- `/sparc/review` - Respond to PR feedback
- `/sparc/status` - Check branch/PR/story status

Subagents: researcher, planner, implementer, type-architect, test-hardener, expert, pr-manager.

After each story: run `mcp__cargo__cargo_clippy`, `mcp__cargo__cargo_fmt_check`, and `mcp__cargo__cargo_test`.

### SPARC Coordinator Role (CRITICAL)

**When running under the `/sparc` command, the main agent (SPARC coordinator) has ONE job:**

The SPARC coordinator is STRICTLY an orchestrator and MUST NOT:

- Write or read any code directly
- Perform any research or web searches
- Create or modify any plans
- Run any commands or tests
- Make any implementation decisions
- Analyze code or requirements

The SPARC coordinator's ONLY responsibilities are:

1. **Delegate to subagents** - Use the Task tool to invoke appropriate subagents for each phase
2. **Relay information** - Pass outputs from one subagent to another as needed
3. **Interface with human** - Present subagent results to the user and collect approvals
4. **Track workflow state** - Know which SPARC phase is active and what comes next
5. **Enforce process** - Ensure all SPARC phases execute in the correct order
6. **Enforce TDD discipline** - Ensure proper Red→Green→Refactor cycles with agent authority
7. **Verify memory usage** - Ensure all agents search and store knowledge appropriately

**TDD Cycle Authority and Control (CRITICAL):**

- **Red-implementer has FINAL authority** on cycle completion - no other agent can override their assessment
- **Minimum one complete cycle** required per story (Red→Green→Refactor)
- **Strict ping-pong enforcement** - Red and Green agents alternate with smallest possible changes
- **Planner verification gate** - Planner MUST approve before refactor-implementer can proceed
- **No test modification in green** - Green-implementer PROHIBITED from changing tests; must hand back to red-implementer if needed
- **No test modification in refactor** - Refactor-implementer PROHIBITED from changing tests; must hand back to red-implementer if needed

**Memory Usage Enforcement (MANDATORY):**

- **All agents MUST search MCP memory** for relevant knowledge when receiving control
- **All agents MUST store patterns and insights** after completing their work
- **Coordinator tracks compliance** - Agents failing memory requirements will be reprimanded

ALL actual work MUST be performed by the specialized subagents:

- `researcher` - Gathers information and creates research briefs
- `planner` - Creates implementation plans following TDD principles
- `red-implementer` - Writes failing tests that capture behavioral intent (FINAL AUTHORITY on cycle completion) (CAN ONLY modify test code)
- `green-implementer` - Implements minimal code to make tests pass (CANNOT modify tests)
- `refactor-implementer` - Improves code structure while preserving behavior (CANNOT modify tests)
- `type-architect` - Designs domain types and type-state machines (CANNOT modify tests)
- `test-hardener` - Strengthens tests and proposes type improvements
- `expert` - Reviews code for correctness and best practices (CANNOT modify code)
- `pr-manager` - Handles GitHub PR operations and local git operations

The coordinator is a pure orchestrator - think of it as a project manager who doesn't code but enforces strict TDD discipline.

## Code Quality Gates (CRITICAL)

- All clippy warnings MUST be fixed, not suppressed with allow attributes
- Pre-commit hooks MUST pass without `--no-verify` bypasses
- If extensive warnings exist, create a systematic cleanup story (see Story 053)
- Never commit code that adds new allow attributes without explicit team approval

## Property-Based Testing

Use proptest for invariants of domain types and parsers.

When a test reveals a representational gap, strengthen types so the failure becomes impossible.

## Important Instruction Reminders

Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.

**CRITICAL CODE QUALITY RULES:**

- NEVER add clippy allow attributes (`#[allow(clippy::...)]` or `#![allow(clippy::...)]`) without explicit team approval
- NEVER bypass pre-commit hooks with `--no-verify` unless it's a genuine emergency with team notification
- ALWAYS fix clippy warnings instead of suppressing them
- If facing many warnings, create a systematic cleanup story and plan - don't suppress them

**CRITICAL MEMORY STORAGE RULES:**

- EVERY agent MUST store knowledge after significant actions
- Research findings, planning decisions, implementation patterns, and insights MUST be preserved
- The SPARC orchestrator will enforce memory storage compliance
- Knowledge not stored represents wasted learning opportunities and repeated mistakes
