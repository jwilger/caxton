# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Memory System Usage (IMPORTANT)

### Qdrant Memory System

**USE THROUGHOUT ALL CONVERSATIONS**: The qdrant memory system should be
actively used in ALL interactions, not just during agent operations or SPARC
workflows.

#### When to Store Memories

Store knowledge whenever you:

- Learn something new about the codebase architecture or patterns
- Discover user preferences or project conventions
- Understand a complex technical concept or solution
- Find important relationships between components
- Identify recurring patterns or anti-patterns
- Debug and solve non-trivial issues
- Make architectural or design decisions

#### Memory Storage Protocol (For All Conversations)

**Store in Qdrant**: Use `mcp__qdrant__qdrant-store` with descriptive content
including:

- Context of the discovery
- Technical details
- Relationships to other concepts
- Clear, searchable descriptions

#### Search Before Acting

Before diving into any task, search for relevant knowledge:

**Semantic Search**: Use `mcp__qdrant__qdrant-find` to find relevant memories
by content, context, or topic. The tool returns the most relevant stored
knowledge based on semantic similarity.

#### Example Memory Storage Scenarios

- **Bug Fix**: Store the bug pattern, root cause, and solution for future
  reference
- **Code Pattern**: Store effective patterns discovered during implementation
- **User Preference**: Store coding style preferences, tool choices, workflow
  preferences
- **Architecture Decision**: Store rationale, trade-offs, and implementation
  details
- **Performance Optimization**: Store before/after metrics and optimization
  techniques

**Remember**: Every conversation is a learning opportunity. Knowledge not stored
is knowledge lost.

## Development Commands

### Bacon Continuous Testing Integration

**CRITICAL**: Always use bacon for continuous testing instead of manual test
commands. Bacon provides real-time feedback and eliminates the need for manual
test execution.

#### Bacon Setup and Usage (MANDATORY)

```bash
# MANDATORY: Start bacon in headless mode at beginning of ANY work session
bacon --headless                     # Start bacon in background for continuous testing
# Use run_in_background: true when starting via Bash tool

# Optional interactive modes (for human use only):
bacon nextest                       # Explicitly run nextest job
bacon clippy                        # Run clippy continuously
bacon check                         # Run cargo check continuously
```

#### Bacon Integration Workflow (NON-NEGOTIABLE)

1. **MANDATORY STARTUP**: ALWAYS check if bacon is running, start with
   `bacon --headless` if not
   - Use: `ps aux | grep "bacon --headless" | grep -v grep` to check
   - Start with: `bacon --headless` using `run_in_background: true`
   - **CRITICAL**: TDD cycle CANNOT function without bacon running
2. **Monitor bacon output**: Use BashOutput tool to check test results and
   compilation feedback
3. **React to failures immediately**: Address compilation errors and test
   failures as they occur
4. **Look for expected failures**: During TDD, expect to see specific test
   failures in bacon output
5. **Verify success**: Confirm all tests pass before committing changes
6. **NEVER use manual test commands**: No `cargo test`, `cargo nextest run`,
   etc. - bacon only!

#### Manual Testing (Only When Bacon Unavailable)

If bacon is not available, fall back to manual nextest commands:

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
cargo check                         # Fast syntax/type checking (use bacon)
cargo clippy                        # Linting (strict rules) (use bacon)
cargo fmt                          # Format code
```

### Development Tools

```bash
bacon --headless                    # Continuous testing with nextest (PREFERRED)
cargo expand                       # Macro expansion
cargo edit                         # Dependency management
```

## Architecture Overview

Caxton is a **multi-agent orchestration server** that provides WebAssembly-based
agent isolation, FIPA-compliant messaging, and comprehensive observability. It
runs as a standalone server process (like PostgreSQL or Redis) rather than a
library.

### Core Components

- **Agent Runtime Environment**: Manages WebAssembly agent lifecycle with
  sandboxing and resource limits
- **FIPA Message Router**: High-performance async message routing between agents
  with conversation tracking
- **Security & Sandboxing**: WebAssembly isolation with CPU/memory limits and
  host function restrictions
- **Observability Layer**: Built-in structured logging, metrics (Prometheus),
  and distributed tracing (OpenTelemetry)
- **Agent Lifecycle Management**: Deployment strategies including blue-green,
  canary, and shadow deployments

### Domain Model Philosophy (Scott Wlaschin Approach)

The codebase follows **Domain Modeling Made Functional** principles inspired by
Scott Wlaschin:

#### Core Philosophy: "Make Illegal States Unrepresentable"

- **Type-driven domain design**: Use Rust's type system to encode business rules
- **Parse, don't validate**: Transform unstructured data into structured types
  at boundaries
- **Algebraic data types**: Sum types (enums) for OR, Product types (structs)
  for AND
- **Total functions over partial**: Prefer functions that work for all inputs of
  their type
- **Railway-oriented programming**: Model workflows as Result chains

#### Implementation Patterns

- **Domain primitives with nutype**: Eliminate primitive obsession with
  validated newtypes
- **Phantom types for state machines**: (`Agent<Unloaded>` → `Agent<Loaded>` →
  `Agent<Running>`)
- **Smart constructors with validation**: Ensure only valid data can exist
- **Comprehensive error types**: Domain-specific error variants that guide
  handling
- **Workflow signatures without implementations**: Define domain operations as
  function signatures

#### Domain-First Development

Use `/sparc/model` command for pure domain modeling:

- Create domain types that make illegal states unrepresentable
- Model state machines and workflows as types
- Define trait-based capabilities
- Focus on **what** the domain is, not **how** it works

#### When to Use Domain Modeling vs Full SPARC

**Use `/sparc/model` when:**

- Starting a new feature area and need to establish domain types first
- Complex business rules need to be encoded in the type system
- You want to explore the problem space before implementing solutions
- The domain is unclear and needs type-driven exploration

**Use `/sparc` (full workflow) when:**

- Domain types already exist and you need to implement functionality
- Building on existing domain foundation
- Ready for test-driven development cycles
- Implementation work needs to be done

### Key Domain Types

Located in `src/domain_types.rs` and `src/domain/`:

- **Agent Identity**: `AgentId`, `AgentName` with validation
- **Resources**: `CpuFuel`, `MemoryBytes`, `MaxAgentMemory` with limits
- **Messaging**: `MessageId`, `ConversationId`, `Performative` for FIPA
  compliance
- **Deployment**: `DeploymentId`, `DeploymentStrategy`, `DeploymentStatus` for
  lifecycle management
- **Security**: `WasmSecurityPolicy`, `ResourceLimits`, `ValidationRule` for
  sandboxing

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

The project uses comprehensive testing with nextest via bacon for continuous
feedback:

- Property-based testing for domain validation with immediate feedback
- WASM fixture generation for integration tests with continuous validation
- Resource limit testing with controlled memory/CPU consumption
- **Bacon Integration**: Continuous test monitoring eliminates manual test
  execution
- **Expected Failure Detection**: During TDD, bacon shows expected test failures
  in real-time
- **Immediate Error Response**: Compilation errors and test failures appear
  instantly in bacon output

## Key Architectural Decisions

Reference the ADR documentation in `docs/adr/` for detailed rationales:

1. **Observability First** (ADR-0001): Every operation is instrumented with
   tracing
2. **WebAssembly Isolation** (ADR-0002): Agents run in secure WASM sandboxes
3. **FIPA Messaging** (ADR-0003): Standard agent communication protocols
4. **Type Safety** (ADR-0018): Domain types with nutype to prevent primitive
   obsession
5. **Coordination First** (ADR-0014): Lightweight coordination instead of shared
   databases

## Development Conventions

- **Error Handling**: Use `CaxtonResult<T>` with comprehensive domain errors
- **Tracing**: Instrument all async functions with `#[instrument]`
- **Resource Safety**: Always validate resource limits before allocation
- **State Machines**: Use phantom types for compile-time state validation
- **Testing**: Write property-based tests for validation logic
- **Dependency Management**: Always use package manager CLI tools (`cargo add`,
  `cargo remove`) to install/update dependencies. Never manually edit Cargo.toml
  version numbers. This ensures we use the latest compatible versions and avoid
  version conflicts.

## External Dependencies

The project uses Nix for development environment management:

- Rust toolchain: stable with clippy, rustfmt, rust-analyzer
- Development tools: cargo-nextest, cargo-watch, cargo-expand, cargo-edit
- Optional: just for task automation
- Ad-hoc: use `nix shell` to use a tool that is not currently installed.
- Update Flake: For tools you regularly use, consider adding them to the
  flake.nix file

## Rust Type-Driven Rules

- **Illegal states are unrepresentable**: prefer domain types over primitives.
- All new domain types use `nutype` with `sanitize(...)` and `validate(...)`.
  Derive at least: `Clone, Debug, Eq, PartialEq, Display`; add
  `Serialize, Deserialize` where needed.
- Prefer `Result<T, DomainError>` over panics. Panics only for truly unreachable
  states.

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

**NEVER ADD ALLOW ATTRIBUTES** - This is a hard rule with zero exceptions
without team approval.

- **NEVER** use `#![allow(clippy::...)]` or `#[allow(clippy::...)]` without
  explicit team approval
- **NEVER** bypass pre-commit hooks or ignore clippy warnings/errors
- **ALWAYS** fix the underlying issue causing the warning instead of suppressing
  it
- Pre-commit hooks MUST pass with `-D warnings` (treat warnings as errors)
- If build fails with warnings, FIX the warnings - don't suppress them
- When facing extensive warnings, create a GitHub issue and systematic plan to
  address them
- The only acceptable temporary measure is to create a focused story (like
  Story 053) to address them systematically

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
- Use `git commit --no-verify` only in genuine emergencies with team
  notification
- If pre-commit hooks fail, fix the issues - don't bypass them
- Allow attribute detection prevents commits with new suppressions

Testing Discipline (Kent Beck) Work in strict Red → Green → Refactor -> Red ->
Green -> Refactor -> ... loops with one failing test at a time.

Use `mcp__cargo__cargo_test` for all tests; treat clippy warnings as errors.

Functional Core / Imperative Shell Put pure logic in the core (no I/O, no
mutation beyond local scope).

Keep an imperative shell for I/O; inject dependencies via traits.

Model workflows as Result pipelines (railway style).

## GitHub PR Workflow

The SPARC workflow integrates with GitHub pull requests to ensure professional
development practices:

### Story Development Flow

1. **Story Selection**: Choose from PLANNING.md
2. **Branch Creation**: `story-{id}-{slug}` feature branches
3. **Standard SPARC**: Research → Plan → Implement → Expert (with mandatory
   memory storage)
4. **PR Creation**: Draft PRs with comprehensive descriptions
5. **Story Completion**: MANDATORY update of PLANNING.md to mark story as
   completed (included in same PR)
6. **Review Loop**: Address feedback with Claude Code attribution
7. **Human Merge**: Only humans mark PRs ready-for-review

### MANDATORY Memory Storage (CRITICAL)

**Every SPARC phase MUST store knowledge using the qdrant memory system:**

#### Memory Storage Protocol

**Store in Qdrant**: Use `mcp__qdrant__qdrant-store` with descriptive content
that includes:

- Clear description of the finding or pattern
- Context and relevance
- Technical details and implementation notes
- Related concepts or dependencies

#### Required Storage by Phase

- **Research Phase**: MUST store findings, sources, patterns, and API
  documentation
- **Planning Phase**: MUST store strategies, decisions, task breakdowns, and
  rationale
- **Implementation Phase**: MUST store TDD cycles, type improvements, patterns,
  and solutions
- **Expert Review Phase**: MUST store insights, quality patterns, and
  architectural analysis
- **PR Management**: MUST store workflow patterns, strategies, and outcomes

#### Search Strategy

**Semantic Search**: Use `mcp__qdrant__qdrant-find` to find memories by
meaning, context, or topic. The search returns relevant stored knowledge
based on semantic similarity.

**Knowledge not stored is knowledge lost. This is not optional and will be
enforced by the SPARC orchestrator.**

### MANDATORY PLANNING.md Updates (CRITICAL)

**Every story completion MUST update PLANNING.md in the same PR:**

- **Story marking**: Change `- [ ]` to `- [x]` for completed story
- **Completion status**: Add completion indicator with brief summary (e.g., "✅
  (COMPLETED - All acceptance criteria met)")
- **Same PR requirement**: PLANNING.md update MUST be committed as part of the
  story implementation PR
- **Validation**: Story ID must match current branch and be verified before
  marking complete
- **Enforcement**: pr-manager agent has exclusive authority and responsibility
  for this update

**Example completion format**:

```diff
- [ ] Story 052: Dependency Vulnerability Resolution - Address the
  GitHub-detected vulnerability
+ [x] Story 052: Dependency Vulnerability Resolution - Address the
  GitHub-detected vulnerability ✅ (COMPLETED - All acceptance criteria met)
```

**Automation**: The SPARC workflow will automatically fail if PLANNING.md is not
updated during story completion.

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

_This comment was generated automatically..._
```

PRs created in **draft status only** - humans control ready-for-review.
**NEVER modify PR status back to draft once a human has marked it ready-for-review.**

### Commands & Agents

Primary commands:

- `/sparc` - Full story workflow with PR integration (includes TDD cycles)
- `/sparc/model` - Pure domain modeling using Scott Wlaschin's principles (NO
  implementation)
- `/sparc/pr` - Create draft PR for completed story
- `/sparc/review` - Respond to PR feedback
- `/sparc/status` - Check branch/PR/story status

Subagents: researcher, planner, domain-modeler, implementer, type-architect,
test-hardener, expert, documentation-writer, pr-manager.

After each story: run `mcp__cargo__cargo_clippy`, `mcp__cargo__cargo_fmt_check`,
and `mcp__cargo__cargo_test`.

### SPARC Coordinator Role (CRITICAL)

**When running under the `/sparc` command, the main agent (SPARC coordinator)
has ONE job:**

The SPARC coordinator is STRICTLY an orchestrator and MUST NOT:

- Write or read any code directly
- Perform any research or web searches
- Create or modify any plans
- Run any commands or tests
- Make any implementation decisions
- Analyze code or requirements

The SPARC coordinator's ONLY responsibilities are:

1. **Delegate to subagents** - Use the Task tool to invoke appropriate subagents
   for each phase
2. **Relay information** - Pass outputs from one subagent to another as needed
3. **Interface with human** - Present subagent results to the user and collect
   approvals
4. **Track workflow state** - Know which SPARC phase is active and what comes
   next
5. **Enforce process** - Ensure all SPARC phases execute in the correct order
6. **Enforce TDD discipline** - Ensure proper Red→Green→Refactor cycles with
   agent authority
7. **Verify memory usage** - Ensure all agents search and store knowledge
   appropriately

#### Domain Modeling Integration (CRITICAL)

**When to invoke domain-modeler during SPARC workflow:**

- **During Planning Phase**: If planner identifies missing or inadequate domain
  types
- **During Implementation**: If red/green/refactor agents identify need for
  stronger types
- **Type Escalation Protocol**: When implementers discover illegal states are
  representable
- **Before TDD Cycles**: When story requires new domain concepts not yet modeled

**Domain-modeler handoff protocol:**

1. **Invocation Trigger**: Planner or implementer identifies domain modeling
   need
2. **Context Passing**: Provide domain requirements and business rules to
   domain-modeler
3. **Type Creation**: Domain-modeler creates types with nutype, phantom types,
   traits
4. **NO IMPLEMENTATION**: Domain-modeler provides only types and signatures
5. **Return to TDD**: After domain modeling, resume normal Red→Green→Refactor
   cycles

#### TDD Cycle Authority and Control (CRITICAL)

- **Red-implementer has FINAL authority** on cycle completion - no other agent
  can override their assessment
- **Minimum one complete cycle** required per story (Red→Green→Red→...→Refactor)
- **CRITICAL FLOW**: Green ALWAYS returns to Red (never directly to Refactor)
  - After Green makes tests pass → Red decides: write another test OR proceed to
    Refactor
  - Only Red-implementer can authorize moving to Refactor phase
- **Strict ping-pong enforcement** - Red and Green agents alternate with
  smallest possible changes
- **No test modification in green** - Green-implementer PROHIBITED from changing
  tests; must hand back to red-implementer if needed
- **No test modification in refactor** - Refactor-implementer PROHIBITED from
  changing tests; must hand back to red-implementer if needed
- **Refactor escalation** - If refactor-implementer discovers missing
  functionality, MUST escalate back to coordinator (cannot add features)
- **Domain type escalation** - If any implementer discovers representable
  illegal states, MUST escalate to coordinator for domain-modeler invocation

**Memory Usage Enforcement (MANDATORY):**

- **All agents MUST search for relevant knowledge** when receiving control:
  - Use `mcp__qdrant__qdrant-find` for semantic search
- **All agents MUST store patterns and insights**:
  - Store content in qdrant with clear, searchable descriptions
- **Coordinator tracks compliance** - Agents failing memory requirements will be
  reprimanded

ALL actual work MUST be performed by the specialized subagents:

- `researcher` - Gathers information and creates research briefs
- `planner` - Creates implementation plans following TDD principles
- `domain-modeler` - Creates domain types using Scott Wlaschin's principles (NO
  implementation logic)
- `red-implementer` - Writes failing tests that capture behavioral intent (FINAL
  AUTHORITY on cycle completion) (CAN ONLY modify test code)
- `green-implementer` - Implements minimal code to make tests pass (CANNOT
  modify tests)
- `refactor-implementer` - Improves code structure while preserving behavior
  (CANNOT modify tests)
- `type-architect` - Refines existing types to strengthen guarantees (CANNOT
  modify tests)
- `test-hardener` - Strengthens tests and proposes type improvements
- `expert` - Reviews code for correctness and best practices (CANNOT modify
  code)
- `documentation-writer` - Creates user guides, API docs, and operational
  procedures (ONLY writes documentation)
- `pr-manager` - Handles GitHub PR operations and local git operations

The coordinator is a pure orchestrator - think of it as a project manager who
doesn't code but enforces strict TDD discipline.

## SPARC Coordinator Verification Protocols (CRITICAL)

**MANDATORY VERIFICATION**: The SPARC coordinator MUST verify actual work
completion at each TDD phase to prevent phantom claims.

### Red Phase Verification (MANDATORY)

**After red-implementer claims test creation, coordinator MUST verify:**

1. **File Creation Verification**:

   ```bash
   # Verify test file was actually created/modified
   Read tool: src/lib.rs, tests/*.rs, or relevant test file
   ```

2. **Test Content Verification**:

   ```bash
   # Search for actual test functions
   Grep: pattern="#\[test\]" output_mode="content" -A 3 -B 1
   ```

3. **Test Execution Verification**:

   ```bash
   # Run tests to confirm they fail as expected
   cargo nextest run --nocapture
   ```

4. **Line Count Requirements**:
   - Minimum 3 lines of meaningful test code beyond `#[test]` attribute
   - Must contain actual assertions or panic conditions
   - No empty test bodies or placeholder comments

**Red Phase Failure Protocol**:

- If verification fails: "RED PHASE VERIFICATION FAILED - No actual tests
  detected"
- Immediately return control to red-implementer with specific failure details
- Do NOT proceed to Green phase until actual tests are verified

### Green Phase Verification (MANDATORY)

**After green-implementer claims implementation completion, coordinator MUST
verify:**

1. **Implementation File Verification**:

   ```bash
   # Verify implementation files were modified
   Read tool: src/lib.rs, src/*.rs files claimed to be modified
   ```

2. **Test Passing Verification**:

   ```bash
   # Confirm tests now pass
   cargo nextest run --nocapture
   ```

3. **Implementation Content Verification**:

   ```bash
   # Search for actual implementation code
   Grep: pattern="fn|impl|struct|enum" output_mode="content" -A 5
   ```

4. **No Test Modification Verification**:

   ```bash
   # Verify green-implementer did not modify tests
   Read tool: test files to confirm no unauthorized changes
   ```

**Green Phase Failure Protocol**:

- If tests still fail: "GREEN PHASE VERIFICATION FAILED - Tests still failing"
- If no implementation detected: "GREEN PHASE VERIFICATION FAILED - No
  implementation changes detected"
- If test modifications detected: "GREEN PHASE VIOLATION - Unauthorized test
  modifications detected"
- Return control to appropriate agent based on failure type

### Refactor Phase Verification (MANDATORY)

**After refactor-implementer claims refactoring completion, coordinator MUST
verify:**

1. **Code Structure Verification**:

   ```bash
   # Verify refactoring actually occurred
   Read tool: files claimed to be refactored
   ```

2. **Test Preservation Verification**:

   ```bash
   # Confirm all tests still pass
   cargo nextest run
   ```

3. **No Test Modification Verification**:

   ```bash
   # Verify refactor-implementer did not modify tests
   Read tool: test files to confirm no unauthorized changes
   ```

4. **Quality Improvement Verification**:

   ```bash
   # Check for actual improvements (reduced complexity, better naming, etc.)
   cargo clippy
   ```

**Refactor Phase Failure Protocol**:

- If tests fail: "REFACTOR PHASE VERIFICATION FAILED - Tests broken by
  refactoring"
- If no changes detected: "REFACTOR PHASE VERIFICATION FAILED - No refactoring
  changes detected"
- If test modifications detected: "REFACTOR PHASE VIOLATION - Unauthorized test
  modifications detected"
- Return control to appropriate agent based on failure type

### Phantom Claim Detection (ZERO TOLERANCE)

**The coordinator MUST detect and reject phantom claims:**

1. **Empty Test Detection**:
   - Tests with only `#[test]` and empty body
   - Tests with only `todo!()` or `unimplemented!()`
   - Tests with only comments and no executable code

2. **Phantom Implementation Detection**:
   - Claims of implementation without actual code changes
   - Implementation that doesn't address the failing tests
   - Copy-paste code without meaningful logic

3. **False Refactoring Detection**:
   - Claims of refactoring without structural improvements
   - Cosmetic changes that don't improve code quality
   - Whitespace-only modifications

**Escalation Protocol for Phantom Claims**:

1. **First Detection**: Warning with specific verification failure details
2. **Second Detection**: Formal reprimand with requirement to demonstrate
   understanding
3. **Third Detection**: Agent replacement with expert review of all previous
   work

### Verification Command Reference

**Mandatory verification commands the coordinator MUST use:**

```bash
# File existence and content verification
Read /absolute/path/to/test/file.rs
Read /absolute/path/to/implementation/file.rs

# Test function detection
Grep pattern="#\[test\]" output_mode="content" -A 10 -B 2

# Implementation detection
Grep pattern="fn|impl|struct|enum" output_mode="content" -A 5

# Test execution verification
cargo nextest run --nocapture

# Build verification
cargo check
cargo clippy
```

**Verification is MANDATORY, not optional. Phantom claims are unacceptable and
will result in immediate agent remediation.**

## Code Quality Gates (CRITICAL)

- All clippy warnings MUST be fixed, not suppressed with allow attributes
- Pre-commit hooks MUST pass without `--no-verify` bypasses
- If extensive warnings exist, create a systematic cleanup story (see Story 053)
- Never commit code that adds new allow attributes without explicit team
  approval

## Property-Based Testing

Use proptest for invariants of domain types and parsers.

When a test reveals a representational gap, strengthen types so the failure
becomes impossible.

## Important Instruction Reminders

Do what has been asked; nothing more, nothing less. NEVER create files unless
they're absolutely necessary for achieving your goal. ALWAYS prefer editing an
existing file to creating a new one. NEVER proactively create documentation
files (\*.md) or README files. Only create documentation files if explicitly
requested by the User.

**CRITICAL CODE QUALITY RULES:**

- NEVER add clippy allow attributes (`#[allow(clippy::...)]` or
  `#![allow(clippy::...)]`) without explicit team approval
- NEVER bypass pre-commit hooks with `--no-verify` unless it's a genuine
  emergency with team notification
- ALWAYS fix clippy warnings instead of suppressing them
- If facing many warnings, create a systematic cleanup story and plan - don't
  suppress them

**CRITICAL MEMORY STORAGE RULES:**

- EVERY agent MUST store knowledge after significant actions
- Research findings, planning decisions, implementation patterns, and insights
  MUST be preserved
- The SPARC orchestrator will enforce memory storage compliance
- Knowledge not stored represents wasted learning opportunities and repeated
  mistakes
