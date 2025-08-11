---
allowed-tools: Bash(git status:*), Bash(cargo:*), Bash(rustfmt:*), Edit, Write, Read, Grep, Glob
description: Execute the approved plan step-by-step using the implementer subagent with Rust TDD discipline
---

# Implement

Use the implementer subagent to implement the current approved plan with strict Rust TDD discipline:

- RED: Create `.claude/tdd.red` and write one failing test
- GREEN: Create `.claude/tdd.green` and implement minimal fix
- Use `cargo nextest run --nocapture` for testing
- Use `cargo clippy -- -D warnings` for linting
- Use `cargo fmt` for formatting
- Commit when green with descriptive messages
