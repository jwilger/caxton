---
allowed-tools: Task
description: Delegate implementation to the implementer subagent following Rust TDD discipline
---

# Implement

Delegate to the implementer subagent to execute the current approved plan with strict Rust TDD discipline.

The implementer agent will follow the Red-Green-Refactor cycle with STRICT TDD discipline:

## TDD Cycle Enforcement (CRITICAL)

**Red-implementer has FINAL AUTHORITY** on cycle completion. No other agent can override their decision on whether additional cycles are needed.

**Minimum Requirements:**
- MUST complete at least ONE full Red→Green→Refactor cycle per story
- RED and GREEN agents work in strict ping-pong alternation with smallest possible changes
- Planner MUST verify and approve before refactor-implementer can proceed
- Refactor-implementer PROHIBITED from modifying tests - must hand back to red-implementer if needed

**Memory Verification:**
- ALL agents MUST search MCP memory for relevant patterns when taking control
- ALL agents MUST store insights and patterns before returning control

**TDD Cycle Steps:**
- RED: Create `.claude/tdd.red` and write one failing test
- GREEN: Create `.claude/tdd.green` and implement minimal fix
- REFACTOR: Remove duplication and improve design (NO TEST CHANGES)
- TYPE PASS: Replace primitives with domain newtypes
- LINT+FORMAT: Run `cargo clippy -- -D warnings` and `cargo fmt`
- COMMIT: Small, descriptive commits with story context

This command acts as a coordinator and delegates all actual implementation work to the specialized implementer agent.
