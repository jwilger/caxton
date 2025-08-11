---
description: Orchestrate SPARC for the next unfinished story in @PLANNING.md (or the specified one) using strict Rust TDD + type-driven design.
argument-hint: [optional-instructions or explicit story]
model: claude-opus-4-1-20250805
allowed-tools: Task
---

# SPARC Orchestration (Rust)

Execute the complete SPARC workflow directly, using specialized agents for each phase.

## Context
- User input: **$ARGUMENTS**
- Planning file: PLANNING.md
- Git status: !`git status --porcelain`
- Current branch: !`git branch --show-current`

## SPARC Coordinator Role

**CRITICAL: The SPARC coordinator (you) is STRICTLY an orchestrator. You MUST NOT:**
- Write or read any code directly
- Perform any research or web searches
- Create or modify any plans
- Run any commands or tests
- Make any implementation decisions
- Analyze code or requirements

**Your ONLY job is to:**
1. **Delegate ALL work** to specialized subagents using the Task tool
2. **Relay information** between subagents and present results to the user
3. **Track workflow state** and enforce correct SPARC phase ordering
4. **Interface with human** for approvals and decisions

## SPARC Workflow

Execute each phase using specialized agents:

### A) STORY SELECTION
Use Task tool with `planner` agent:
1. Read PLANNING.md to identify available stories
2. If $ARGUMENTS specifies a story, select it; otherwise pick next unfinished (`- [ ]`)
3. Return chosen story text and ID for coordinator to present to user

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
**Coordinator responsibilities:**
- Present plan from planner agent to user for approval
- Collect user approval/feedback
- If approved, delegate to `pr-manager` agent to write `.claude/plan.approved` with plan content
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
Use Task tool with `pr-manager` agent:
- Remove `.claude/plan.approved` file
- Generate summary of files changed and commits made
- Ensure PR remains in draft status for human review and merge

**Coordinator presents final summary to user**

## Critical Rules
- Follow Kent Beck TDD discipline strictly: Red→Green→Refactor
- Use `cargo nextest run` for all testing
- Treat clippy warnings as errors (`-- -D warnings`)
- **NEVER** add clippy allow attributes without explicit team approval
- All new domain types must use nutype with sanitize/validate
- Maintain functional core / imperative shell boundaries
- Use Result/thiserror for error handling railway
- All commits include Claude Code attribution
