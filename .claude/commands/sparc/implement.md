---
allowed-tools: Task, Bash, BashOutput
description:
  Delegate implementation to the implementer subagent following Rust TDD
  discipline
---

# Implement

Delegate to the implementer subagent to execute the current approved plan with
strict Rust TDD discipline.

## Pre-Implementation Setup

**CRITICAL FIRST STEP**: Before delegating to any agents, the coordinator MUST:

1. **Set Cargo Working Directory**: Call `mcp__cargo__set_working_directory`
   with the absolute path to the project root (where Cargo.toml exists)
2. **Start Bacon Continuous Testing**: Launch `bacon --headless` in background
   for real-time test monitoring
3. **Verify Setup**: Ensure the working directory is set correctly and bacon is
   running for all subsequent operations

This ensures all agents have proper access to cargo commands and continuous test
feedback without manual test execution.

## SPARC Coordinator Role

**CRITICAL: The SPARC coordinator (you) is STRICTLY an orchestrator. You MUST
NOT:**

- Write or read any code directly
- Perform any implementation yourself
- Run any commands or tests
- Make any technical decisions

**Your ONLY job is to:**

1. **Delegate ALL work** to specialized TDD subagents using the Task tool
2. **Relay information** between subagents as needed
3. **Track TDD cycle state** and enforce correct phase ordering
4. **ENFORCE MEMORY STORAGE** - Verify each agent stores knowledge in qdrant
5. **Monitor bacon output** via BashOutput tool for test status

The implementer agent will follow the Red-Green-Refactor cycle with STRICT TDD
discipline:

## TDD Cycle Enforcement (CRITICAL)

**Red-implementer has FINAL AUTHORITY** on cycle completion. No other agent can
override their decision on whether additional cycles are needed.

**Minimum Requirements:**

- MUST complete at least ONE full Red→Green→Refactor cycle per story
- RED and GREEN agents work in strict ping-pong alternation with smallest
  possible changes
- Planner MUST verify and approve before refactor-implementer can proceed
- Refactor-implementer PROHIBITED from modifying tests - must hand back to
  red-implementer if needed

## Bacon Integration Throughout

**MANDATORY**: All agents must monitor bacon output instead of running manual
test commands:

- **Use BashOutput tool** to check bacon status when tests are expected to
  change
- **Look for expected failures** during RED phase - bacon should show the
  failing test
- **Confirm test passes** during GREEN phase - bacon should show all tests
  passing
- **Verify no regressions** during REFACTOR phase - bacon should maintain green
  status
- **React immediately** to any unexpected compilation errors or test failures
- **NEVER use manual test commands** like `cargo test` - bacon provides
  continuous feedback

**Memory Storage Requirements:**

**MANDATORY**: All agents MUST:

- **Search existing knowledge**: Use `mcp__qdrant__qdrant-find` when taking
  control
- **Store patterns and insights**: Use `mcp__qdrant__qdrant-store` before
  returning control
- **Store with clear context**: Include descriptions, technical details, and
  relationships

**ENFORCEMENT**: If any agent fails to store knowledge, immediately request they
do so before continuing.

## TDD Cycle Steps

**RED Phase** - Use Task tool with `red-implementer` agent:

- Write exactly ONE failing test that captures the next behavior
- **Monitor bacon output** to verify test fails for the right reason
- Create `.claude/tdd.red` state file
- Store test patterns in MCP memory
- **COORDINATOR VALIDATION**: Verify response contains ONLY test code

**GREEN Phase** - Use Task tool with `green-implementer` agent:

- Implement minimal code to make the failing test pass
- **Monitor bacon output** to verify test passes
- Create `.claude/tdd.green` state file
- Store implementation patterns in MCP memory
- **COORDINATOR VALIDATION**: Verify response contains ONLY implementation code

**REFACTOR Phase** - Use Task tool with `refactor-implementer` agent:

- Remove duplication and improve code structure
- **Monitor bacon output** to ensure tests stay green
- Store refactoring patterns in MCP memory
- **COORDINATOR VALIDATION**: Verify no test modifications
- **COMMIT**: Create descriptive commit with Claude Code attribution

**TYPE PASS** - Use Task tool with `type-architect`:

- Replace primitives with domain newtypes
- **Monitor bacon output** for type checking
- Store type design patterns in MCP memory

## TDD Role Validation Protocol

**COORDINATOR MUST VALIDATE** every TDD agent response before acceptance:

### Red-Implementer Validation

- ✅ **Required**: Response begins with role declaration
- ✅ **Required**: Contains test code blocks only
- ✅ **Required**: Includes role compliance statement
- ❌ **Forbidden**: ANY implementation code

### Green-Implementer Validation

- ✅ **Required**: Response begins with role declaration
- ✅ **Required**: Contains implementation code only
- ✅ **Required**: Includes role compliance statement
- ❌ **Forbidden**: ANY test code or test modifications

### Refactor-Implementer Validation

- ✅ **Required**: Response begins with role declaration
- ✅ **Required**: Contains implementation improvements only
- ✅ **Required**: Includes role compliance statement
- ❌ **Forbidden**: ANY test modifications

### Violation Response Protocol

If validation fails:

1. **Re-delegate** to same agent with role reminder
2. **Include validation error** in re-delegation prompt
3. **Do NOT proceed** until validation passes
4. **Escalate to human** after 2 failed attempts

## Information Request Routing

During implementation, agents may include "Information Requests" sections. The
coordinator MUST:

1. **Parse Information Requests** from agent responses
2. **Route to Target Agent** using Task tool
3. **Track Requests** to prevent infinite loops
4. **Relay Responses** back to requesting agent
5. **Never answer requests directly** - always delegate

## Critical Rules

- Follow Kent Beck TDD discipline strictly: Red→Green→Refactor
- **Use bacon for continuous testing** - monitor BashOutput for feedback
- Treat clippy warnings as errors (`-- -D warnings`)
- **NEVER** add clippy allow attributes without team approval
- All commits include Claude Code attribution:

  ```text
  🤖 Generated with [Claude Code](https://claude.ai/code)

  Co-Authored-By: Claude <noreply@anthropic.com>
  ```

- **MANDATORY MEMORY STORAGE** - every agent must store knowledge

## Planning.md Update (When Completing Story)

If this implementation completes a story, delegate to `pr-manager` to:

- Update PLANNING.md to mark story as `[x]` completed
- Add completion status (e.g., "✅ (COMPLETED)")
- Include update in same PR as implementation

This command acts as a coordinator and delegates all actual implementation work
to the specialized implementer agents.
