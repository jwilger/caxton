---
description: Orchestrate SPARC for the next unfinished story in @PLANNING.md (or the specified one) using strict Rust TDD + type-driven design.
argument-hint: [optional-instructions or explicit story]
model: claude-opus-4-1-20250805
allowed-tools: Read, Write, Edit, MultiEdit, Grep, Glob, WebSearch, WebFetch, Bash, Task
---

# SPARC Orchestration (Rust)

Execute the complete SPARC workflow directly, using specialized agents for each phase.

## Context
- User input: **$ARGUMENTS**
- Planning file: PLANNING.md
- Git status: !`git status --porcelain`
- Current branch: !`git branch --show-current`

## SPARC Workflow

Execute each phase using specialized agents:

### A) STORY SELECTION
1. Read PLANNING.md to identify stories
2. If $ARGUMENTS specifies a story, use it; otherwise pick next unfinished (`- [ ]`)
3. Echo chosen story text and ID for confirmation

### A.5) BRANCH SETUP
Use Task tool with `pr-manager` agent:
- Create feature branch: `story-{zero-padded-id}-{kebab-case-slug}`
- Switch to feature branch
- Record story/branch mapping in `.claude/branch.info`

### B) RESEARCH
Use Task tool with `researcher` agent:
- Research external dependencies (Rust tools: cargo-nextest, clippy, proptest, nutype)
- Gather authoritative sources and documentation
- Return "Research Brief" with assumptions, key facts, and open questions

### C) PLAN
Use Task tool with `planner` agent:
- Plan Kent Beck Red→Green→Refactor loop (one failing test)
- List new/updated domain types (nutype) and function signatures
- Specify pure functions vs shell boundaries and error railway (Result/thiserror)
- Define acceptance checks and rollback plan

### D) APPROVAL GATE
- Present plan to user for approval
- On approval, write `.claude/plan.approved` with ONLY the plan content
- Block further progress until approved

### E) IMPLEMENT
Use Task tool with `implementer` agent:
- **RED**: Add one failing test (`#[test]` that fails with `unimplemented!()`)
- **GREEN**: Minimal change to pass the test
- **REFACTOR**: Remove duplication; keep core pure, shell minimal
- **TYPE PASS**: Use Task tool with `type-architect` to replace primitives with nutype domain types
- **LINT/FORMAT**: Run `cargo clippy -- -D warnings` and `cargo fmt`
- **TEST**: Run `cargo nextest run --nocapture`
- **COMMIT**: Create descriptive commit with Claude Code attribution

### F) TEST-HARDENING
Use Task tool with `test-hardener` agent:
- For each test added/changed, propose type/API changes that make failures impossible at compile time
- If safe, implement type changes with small diffs
- Update call sites and re-run clippy + nextest

### G) EXPERT CHECK (Optional)
Use Task tool with `expert` agent:
- Request brief review on type-state soundness and error pipeline
- Get validation of implementation approach

### H) PR CREATION
Use Task tool with `pr-manager` agent:
- Create draft PR with comprehensive description
- Link story acceptance criteria and implementation summary
- Update `.claude/branch.info` with PR number
- Never mark PR ready-for-review (human only)

### I) PR REVIEW LOOP (if feedback exists)
Use Task tool with `pr-manager` agent:
- Monitor for PR comments via `gh pr view`
- Respond with Claude Code attribution
- Address requested changes using TDD discipline
- Create follow-up commits as needed

### J) COMPLETION
- Remove `.claude/plan.approved`
- Summarize files changed and commits made
- Leave PR in draft status for human review and merge

## Critical Rules
- Follow Kent Beck TDD discipline strictly: Red→Green→Refactor
- Use `cargo nextest run` for all testing
- Treat clippy warnings as errors (`-- -D warnings`)
- **NEVER** add clippy allow attributes without explicit team approval
- All new domain types must use nutype with sanitize/validate
- Maintain functional core / imperative shell boundaries
- Use Result/thiserror for error handling railway
- All commits include Claude Code attribution
