---
description: Orchestrate SPARC for the next unfinished story in @PLANNING.md (or the specified one) using strict Rust TDD + type-driven design.
argument-hint: [optional-instructions or explicit story]
model: opus
allowed-tools: Read, Write, Edit, MultiEdit, Grep, Glob, WebSearch, WebFetch, Bash(cargo:*), Bash(rustfmt*), Bash(git status:*), Bash(git branch:*), Bash(git diff:*), Bash(rg:*)
---

# SPARC Orchestration (Rust)

**Use the `sparc-orchestrator` subagent for this entire command. Keep all actions inside that subagent’s context.**
If the orchestrator doesn’t exist, ask me to create it via `/agents`.

## Inputs

User instruction (if provided): **$ARGUMENTS**

## Planning file

We will use PLANNING.md as the source of stories.

## Fast context (optional)

- Git status: !`git status --porcelain`
- Branch: !`git branch --show-current`

## Task

Drive the full flow for a SINGLE story with Rust constraints:

A) STORY SELECTION

- If $ARGUMENTS names a story, use it; else pick the next unfinished (`- [ ]` / TODO).
- Echo the exact story text/IDs.

A.5) BRANCH SETUP → (delegate to `pr-manager`)

- Create feature branch: `story-{zero-padded-id}-{kebab-case-slug}`
- Switch to feature branch
- Record story/branch mapping in `.claude/branch.info`

B) RESEARCH → (delegate to `researcher`)

- Open only authoritative sources for Rust tools (cargo-nextest, clippy, proptest, nutype).
- Return a “Research Brief” with linked sources.

C) PLAN → (delegate to `planner`)

- Plan a Kent-Beck Red→Green→Refactor loop (one failing test).
- List new/updated domain types (nutype) and function signatures.
- Specify pure functions vs shell boundaries and the error railway (Result/thiserror).
- Acceptance checks.

D) APPROVAL

- Wait for approval. On approval, write `.claude/plan.approved` with ONLY the plan.

E) IMPLEMENT → (delegate to `implementer`)

- RED: add one failing test (`#[test]` that fails, e.g. `unimplemented!()`).
  - Run: !`cargo nextest run --nocapture`
- GREEN: minimal change to pass.
- REFACTOR: remove duplication; keep core pure; shell minimal.
- TYPE PASS → (delegate partly to `type-architect`)
  - Replace primitives with domain types using `nutype` (derive/validate/sanitize).
  - Tighten function signatures and states (phantom types if needed).
- LINT/FORMAT:
  - !`cargo clippy -- -D warnings`
  - !`cargo fmt`
- TEST: !`cargo nextest run --nocapture`
- COMMIT with a descriptive message.

F) TEST-HARDENING → (delegate to `test-hardener`)

- For each test added/changed in this story, propose a type/API change that would make such failures impossible at compile time.
- If safe, implement type changes (small diffs), update call sites, and re-run clippy + nextest.

G) (optional) EXPERT CHECK → (delegate to `expert`)

- Ask for a brief review on type-state soundness and error pipeline.

H) PR CREATION → (delegate to `pr-manager`)

- Create draft PR with comprehensive description
- Link story acceptance criteria and implementation summary
- Update `.claude/branch.info` with PR number
- Never mark PR ready-for-review (human only)

I) PR REVIEW LOOP (if feedback received) → (delegate to `pr-manager`)

- Monitor for PR comments via `gh pr view`
- Respond with Claude Code attribution
- Address requested changes using TDD discipline
- Create follow-up commits if needed

J) DONE

- Remove `.claude/plan.approved`
- Summarize files changed and commits
- Leave PR in draft status for human review
