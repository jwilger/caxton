---
description:
  Model domain using Rust's type system following Scott Wlaschin's principles.
  Makes illegal states unrepresentable without implementing logic.
argument-hint: domain-requirements-or-story
model: claude-opus-4-1-20250805
allowed-tools: Task, Bash, BashOutput
---

# Domain Modeling Command

ULTRATHINK

Model the domain using Rust's type system to make illegal states
unrepresentable, following Scott Wlaschin's "Domain Modeling Made Functional"
principles.

## Context

- User input: **$ARGUMENTS**
- Git status: !`git status --porcelain`
- Current branch: !`git branch --show-current`

## Pre-Workflow Setup

**CRITICAL FIRST STEP**: Before delegating to any agents, the coordinator MUST:

1. **Set Cargo Working Directory**: Call `mcp__cargo__set_working_directory`
   with the absolute path to the project root (where Cargo.toml exists)
2. **Start Bacon Continuous Testing**: Launch `bacon --headless` in background
   for real-time type checking and compilation monitoring
3. **Verify Setup**: Ensure the working directory is set correctly and bacon is
   running for all subsequent operations

This ensures all agents have proper access to cargo commands and continuous
compilation feedback.

## SPARC Coordinator Role

**CRITICAL: The SPARC coordinator (you) is STRICTLY an orchestrator. You MUST
NOT:**

- Write or read any code directly
- Design any domain types yourself
- Make any modeling decisions
- Run any commands or tests

**Your ONLY job is to:**

1. **Delegate ALL work** to specialized subagents using the Task tool
2. **Relay information** between subagents as needed
3. **Track workflow state** and enforce correct phase ordering
4. **ENFORCE MEMORY STORAGE** - Verify each agent stores knowledge in qdrant
5. **Monitor bacon output** via BashOutput tool for compilation status

## Purpose

This command focuses purely on domain modeling - creating types, traits, and
function signatures that encode business rules in the type system. No
implementation logic, just pure domain design.

## Workflow Phases

### 1. Requirements Analysis

If $ARGUMENTS references a story from PLANNING.md, use Task tool with `planner`
agent to:

- Extract the story requirements
- Identify domain concepts mentioned
- Return domain boundaries and key entities

Otherwise, treat $ARGUMENTS as the domain requirements directly.

### 2. Domain Research (Optional)

If domain concepts need clarification, use Task tool with `researcher` agent to:

- Research domain terminology and patterns
- Find similar domain modeling examples in Rust
- Gather information about business rules and constraints
- Return a brief with domain insights
- **MANDATORY**: Store research findings in MCP memory

**Memory Verification**: Confirm researcher used `mcp__qdrant__qdrant-store`

### 3. Domain Modeling

Use Task tool with `domain-modeler` agent (Scott Wlaschin persona) to:

- **Search existing patterns**: Use `mcp__qdrant__qdrant-find` for relevant
  domain patterns
- Analyze domain requirements thoroughly
- Identify entities, value objects, aggregates, events, and commands
- Create nutype-based domain primitives
- Design sum and product types for the domain
- Model state machines with phantom types where appropriate
- Define workflow signatures (without implementations)
- Create trait definitions for domain capabilities
- **MANDATORY**: Store domain patterns in MCP memory

**Memory Storage Requirements**: Domain-modeler MUST store:

- Domain type design decisions
- Business rule encodings
- State machine patterns
- Workflow signatures
- Anti-patterns to avoid

**ENFORCEMENT**: If domain-modeler fails to store knowledge, immediately request
they do so before proceeding.

The domain-modeler will create:

- `src/domain/types.rs` - Core domain types
- `src/domain/states.rs` - State machine definitions (if needed)
- `src/domain/workflows.rs` - Workflow signatures
- `src/domain/traits.rs` - Domain capability traits
- `src/domain/mod.rs` - Module exports

### 4. Type Safety Review (Optional)

If complex type machinery is used, use Task tool with `expert` agent to:

- Review the domain model for soundness
- Verify illegal states are truly unrepresentable
- Check for potential runtime panics in the type design
- Suggest improvements to the type model
- **MANDATORY**: Store insights about type safety in MCP memory

**Memory Verification**: Confirm expert used `mcp__qdrant__qdrant-store`

### 5. Bacon Monitoring

Throughout the domain modeling process:

- **Monitor bacon output** via BashOutput tool for compilation status
- **Check for type errors** as domain types are created
- **Verify nutype validation** compiles correctly
- **Ensure phantom types** are properly constructed
- **React immediately** to compilation errors

### 6. Commit Domain Model

Use Task tool with `pr-manager` agent to:

- Stage all domain modeling files
- Commit with message like:

  ```text
  feat(domain): model [domain area] following DDD principles

  - Create domain types using nutype for validation
  - Model [key concept] with phantom types for state machine
  - Define workflow signatures for [main workflows]
  - Make illegal states unrepresentable at compile time

  Following Scott Wlaschin's Domain Modeling Made Functional approach.
  No implementation logic included - pure type modeling only.
  ```

- Push changes if on a feature branch

## Key Principles (Scott Wlaschin)

The domain-modeler agent follows these principles:

1. **Make Illegal States Unrepresentable** - Use types to prevent invalid data
2. **Parse, Don't Validate** - Transform unstructured data into structured types
   at boundaries
3. **Use Algebraic Data Types** - Sum types for OR, Product types for AND
4. **Total Functions Over Partial** - Prefer functions that work for all inputs
5. **Model the Workflow as a Pipeline** - Domain workflows are transformations
6. **Types as Documentation** - The type system tells the story of the domain

## No Implementation

This command specifically AVOIDS:

- Writing validation logic (just the types with nutype validators)
- Implementing functions (all use `unimplemented!()`)
- Creating tests (that's for TDD cycles)
- Building infrastructure
- Database schemas
- API endpoints

## Example Usage

```bash
/sparc/model Story 003: Agent Communication Protocol - Model the FIPA-compliant message types
```

Results in domain types like:

```rust
// Domain primitives
#[nutype(
    sanitize(trim),
    validate(len(min = 1, max = 64)),
    derive(Clone, Debug, PartialEq, Eq, Display)
)]
pub struct AgentId(String);

// Sum types for choices
pub enum MessagePerformative {
    Request(RequestContent),
    Inform(InformContent),
    Query(QueryContent),
}

// State machines
pub struct Message<State> {
    id: MessageId,
    sender: AgentId,
    _state: PhantomData<State>,
}

// Workflow signatures
pub fn send_message(
    message: UnvalidatedMessage,
) -> Result<MessageSent, SendMessageError> {
    unimplemented!()
}
```

## Information Request Routing

During domain modeling, agents may include "Information Requests" sections. The
coordinator MUST:

1. **Parse Information Requests** from agent responses
2. **Route to Target Agent** using Task tool (researcher, planner, expert)
3. **Track Requests** to prevent infinite loops (max depth: 3)
4. **Relay Responses** back to requesting agent
5. **Never answer requests directly** - always delegate

Common patterns:

- Domain-modeler → Researcher (for domain terminology)
- Domain-modeler → Expert (for type safety validation)
- Domain-modeler → Planner (for requirements clarification)

## Critical Rules

- **Use bacon for continuous compilation** - monitor BashOutput for type checking
- Treat clippy warnings as errors (`-- -D warnings`)
- **NEVER** add clippy allow attributes without team approval
- All new domain types must use nutype with sanitize/validate
- Make illegal states unrepresentable at compile time
- All commits include Claude Code attribution:

  ```text
  🤖 Generated with [Claude Code](https://claude.ai/code)

  Co-Authored-By: Claude <noreply@anthropic.com>
  ```

- **MANDATORY MEMORY STORAGE** - every agent must store knowledge

## Completion

After domain modeling is complete:

1. All domain types are created in `src/domain/`
2. Business rules are encoded in the type system
3. Illegal states are unrepresentable
4. Changes are committed with clear documentation
5. **Memory contains domain patterns** for future reference (VERIFIED)
6. **Bacon shows clean compilation** with no type errors

The implementers can now use these types with confidence that the compiler will
guide them toward correct implementations.
