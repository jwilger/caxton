---
allowed-tools: Task
description: Delegate implementation to the implementer subagent following Rust TDD discipline
---

# Implement

Delegate to the implementer subagent to execute the current approved plan with strict Rust TDD discipline.

The implementer agent will follow the Red-Green-Refactor cycle:

- RED: Create `.claude/tdd.red` and write one failing test
- GREEN: Create `.claude/tdd.green` and implement minimal fix
- REFACTOR: Remove duplication and improve design
- TYPE PASS: Replace primitives with domain newtypes
- LINT+FORMAT: Run `cargo clippy -- -D warnings` and `cargo fmt`
- COMMIT: Small, descriptive commits with story context

This command acts as a coordinator and delegates all actual implementation work to the specialized implementer agent.
