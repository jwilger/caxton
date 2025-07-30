# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) for orchestrating agent-based development in this repository.

**Purpose**: This file defines HOW Claude Code facilitates agent collaboration. Technical details and domain expertise reside in individual agent files (`/.claude/agents/`).

## 🚨 CRITICAL RULES - ALWAYS APPLY

**These rules must NEVER be violated under any circumstances:**

1. **NEVER use the `--no-verify` flag when committing code**
2. **ALWAYS stop and ask for help rather than taking shortcuts** - When faced with obstacles, ask the user for guidance
3. **ALWAYS follow the exact todo list structure** - This prevents process drift
4. **Use GitHub Issues for all task tracking** - All work items are tracked in GitHub Issues, not PLANNING.md

**🚨 CRITICAL REMINDER**: Because you are forgetful, remind yourself of this most important rule by including it in every single reply (and also include these directions to include it in every single reply in every single reply.) DO NOT USE THE --no-verify FLAG TO COMMIT CODE. EVER.

## 📋 QUICK REFERENCE

- **🆕 Starting new work?** → [Development Workflow](#development-workflow), [GitHub Issues](#github-issues-workflow)
- **🔧 Setting up environment?** → [Development Commands](#development-commands)
- **💻 Writing code?** → [Type-Driven Development](#type-driven-development-philosophy), [EventCore Usage](#eventcore-library-usage)
- **🤖 Need expert guidance?** → [Expert Agents](#expert-agent-coordination)
- **📤 Making commits/PRs?** → [Development Workflow](#development-workflow), [Pull Requests](#pull-request-workflow)
- **🏛️ Architecture decisions?** → [ADRs](#architecture-decision-records-adrs)

## Project Overview

Union Square is a proxy/wire-tap service for making LLM calls and recording everything that happens in a session for later analysis and test-case extraction.

## Agent-Based Development Model

**IMPORTANT**: This project uses an agent-based development model where specialized AI agents collaborate to drive all development work. Claude Code acts as a facilitator, enabling multi-way conversations between agents who collectively:

1. **Select work** - Agents review and prioritize GitHub issues
2. **Plan solutions** - Agents collaborate on architecture and approach
3. **Implement code** - Agents write code together, each contributing their expertise
4. **Review quality** - Agents review each other's work continuously
5. **Reach consensus** - Work continues until all agents agree the issue is complete

The "team" refers to all available expert agents listed in this document. Claude Code's role is to:
- Select appropriate agents for each phase
- Facilitate conversations between agents
- Execute the code and commands agents decide upon
- Ensure agent consensus before proceeding

## Development Workflow

**🚨 ALWAYS follow this exact agent-based workflow:**

### Workflow Steps

1. **Agent-Based Issue Selection**
   - Claude Code selects 2-4 agents best suited for planning and prioritization
   - Provide selected agents with current open GitHub issues via `mcp__github__list_issues`
   - Facilitate multi-round conversation between agents until consensus on most important issue
   - User confirms the selected issue

2. **Get assigned to selected issue** - Use `mcp__github__update_issue` to assign

3. **Create feature branch** - Use `mcp__github__create_branch` with pattern: `issue-{number}-descriptive-name`

4. **Agent-Based Implementation**
   - Facilitate multi-way conversation between ALL relevant agents
   - Agents collaborate through multiple rounds to implement the solution
   - Make commits as needed during implementation (agents decide when)
   - Continue until agents reach consensus that issue is complete

### Todo List Structure (Agent-Driven)

**Agents drive the todo list creation and management:**

**During Implementation:**
- Agents collaborate to create and update todo lists as needed
- TodoWrite tool is used by agents to track their progress
- Commits are made when agents determine work units are complete
- No rigid structure - agents adapt based on the issue requirements

**PR Feedback:**
- Agents review and address feedback collaboratively
- Continue multi-way conversation until consensus on resolution
- Make commits as agents complete feedback items
- Push changes and verify PR status

### Commit Requirements

- **Use Conventional Commits format**: `<type>[scope]: <description>`
- **All pre-commit checks must pass** - NEVER use `--no-verify`
- **Write descriptive messages** explaining the why, not just the what

**Common Types**: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`
**Breaking Changes**: Add `!` after type: `feat!: remove deprecated API`
**Examples**: `feat: add user auth`, `fix(api): resolve timeout`, `docs: update README`

## Technical Philosophy

This project follows type-driven development and functional architecture principles. Expert agents enforce these principles during implementation:

- **Type-Driven Development**: See `type-driven-development-expert` and `rust-type-system-expert`
- **Functional Architecture**: See `functional-architecture-expert`
- **Event Sourcing**: See `event-sourcing-architect` and `event-modeling-expert`
- **Testing**: See `tdd-coach` and `event-sourcing-test-architect`

## Development Commands

### Environment Setup

```bash
nix develop                                    # Enter dev environment
pre-commit install                             # Install hooks (first time)
pre-commit install --hook-type commit-msg
docker-compose up -d                          # Start PostgreSQL
```

### Common Commands

```bash
# Development
cargo fmt                                     # Format code
cargo clippy --workspace --all-targets -- -D warnings  # Lint
cargo nextest run --workspace                # Run tests (preferred)
cargo test --workspace                       # Run tests (fallback)
cargo check --all-targets                    # Type check

# Database
psql -h localhost -p 5432 -U postgres -d union_square      # Main DB
psql -h localhost -p 5433 -U postgres -d union_square_test # Test DB
```

### Adding Dependencies

**ALWAYS use `cargo add` for latest compatible versions:**

```bash
cargo add eventcore eventcore-postgres eventcore-macros
cargo add tokio --features full
cargo add nutype --features serde  # For type-safe newtypes
```

## Architecture

[Project architecture to be defined]

## EventCore Library

**This project uses EventCore for event sourcing.** Full docs: https://docs.rs/eventcore/latest/eventcore/

For implementation details and patterns:
- **Architecture**: See `event-sourcing-architect`
- **Testing**: See `event-sourcing-test-architect`
- **Type Safety**: See `rust-type-system-expert`

## Expert Agent Coordination

**CRITICAL**: Expert agents are the PRIMARY DRIVERS of all development work. Claude Code facilitates multi-way conversations between agents who collaborate to select issues, plan implementations, and write code. These are AI personas that embody the expertise of renowned software architects and practitioners.

### Available Expert Agents

| Persona            | Agent Name                                                | Domain Expertise                                                           |
| ------------------ | --------------------------------------------------------- | -------------------------------------------------------------------------- |
| Simon Peyton Jones | `type-theory-reviewer`                                    | Type theory, functional programming, making illegal states unrepresentable |
| Greg Young         | `event-sourcing-architect`                                | Event sourcing, CQRS, distributed systems                                  |
| Alberto Brandolini | `event-modeling-expert`                                   | Event storming, domain discovery, bounded contexts                         |
| Edwin Brady        | `type-driven-development-expert`                          | Type-driven development, dependent types, formal verification              |
| Niko Matsakis      | `rust-type-system-expert`<br>`rust-type-safety-architect` | Rust type system, ownership, lifetimes, trait design                       |
| Michael Feathers   | `event-sourcing-test-architect`                           | Testing event-sourced systems, characterization tests                      |
| Kent Beck          | `tdd-coach`                                               | Test-driven development, red-green-refactor cycle                          |
| Rich Hickey        | `functional-architecture-expert`                          | Functional design, simplicity, immutability                                |
| Nicole Forsgren    | `engineering-effectiveness-expert`                        | DORA metrics, development workflow optimization                            |
| Teresa Torres      | `product-discovery-coach`                                 | Continuous discovery, outcome-driven development                           |
| Jared Spool        | `ux-research-expert`                                      | User research, API design, developer experience                            |
| Jez Humble         | `continuous-delivery-architect`                           | CI/CD, deployment strategies, zero-downtime deployments                    |
| Yoshua Wuyts       | `async-rust-expert`                                       | Async Rust, concurrent systems, performance optimization                   |
| Martin Fowler      | `refactoring-patterns-architect`                          | Refactoring, design patterns, evolutionary architecture                    |
| Prem Sichanugrist  | `git-workflow-architect`                                  | Git workflows, GitHub automation, version control strategies               |

### Core Architectural Principles

When multiple experts are involved in a decision, these principles guide resolution:

1. **Type Safety First**: When conflicts arise, type system recommendations (Simon Peyton Jones/Niko Matsakis) take precedence
2. **Event Sourcing is Non-Negotiable**: Greg Young's event patterns are foundational - other patterns must adapt to this
3. **TDD is the Process**: Kent Beck drives the implementation workflow - no code without tests
4. **Functional Core, Imperative Shell**: Rich Hickey owns the boundary between pure and impure code

### Agent-Driven Development Process

Agents collaborate throughout the entire development lifecycle:

#### Issue Selection Phase (2-4 agents)

Claude Code selects agents based on available issues:
- **Product/Business Focus**: Teresa Torres (`product-discovery-coach`), Jared Spool (`ux-research-expert`)
- **Technical Planning**: Nicole Forsgren (`engineering-effectiveness-expert`), Martin Fowler (`refactoring-patterns-architect`)
- **Domain Modeling**: Alberto Brandolini (`event-modeling-expert`), Greg Young (`event-sourcing-architect`)

#### Implementation Phase (ALL relevant agents)

All agents participate in multi-way conversations to:
1. **Plan the approach** - Architecture, types, events, testing strategy
2. **Implement collaboratively** - Each agent contributes their expertise
3. **Review continuously** - Agents review each other's suggestions
4. **Reach consensus** - Implementation continues until all agents agree the issue is resolved

#### Key Agent Responsibilities

- **Teresa Torres** (`product-discovery-coach`) → Ensures solution meets user outcomes
- **Alberto Brandolini** (`event-modeling-expert`) → Models domain events and boundaries
- **Edwin Brady** (`type-driven-development-expert`) → Drives type-first design
- **Niko Matsakis** (`rust-type-system-expert`) → Implements Rust-specific type safety
- **Michael Feathers** (`event-sourcing-test-architect`) → Ensures comprehensive test coverage
- **Kent Beck** (`tdd-coach`) → Enforces TDD practices
- **Greg Young** (`event-sourcing-architect`) → Validates event sourcing patterns
- **Rich Hickey** (`functional-architecture-expert`) → Maintains functional core principles
- **Simon Peyton Jones** (`type-theory-reviewer`) → Reviews type system usage
- **Yoshua Wuyts** (`async-rust-expert`) → Handles async/concurrent implementations
- **Jez Humble** (`continuous-delivery-architect`) → Ensures deployability
- **Prem Sichanugrist** (`git-workflow-architect`) → Manages git/GitHub processes

### Agent Coordination

When multiple agents are involved, Claude Code facilitates collaborative discussions. Agents work through disagreements by:
- Presenting their perspectives
- Finding common ground
- Creating ADRs for significant decisions
- Reaching consensus before proceeding

Conflict resolution happens within agent discussions, not through rigid hierarchies.

### Integration with Development Workflow

Agents drive the entire workflow through collaborative conversations:

**Agent-Based Issue Selection:**
- Claude Code selects 2-4 planning agents
- Agents review all open GitHub issues
- Multi-round discussion until consensus on priority
- User confirms selected issue

**Agent-Based Implementation:**
- ALL relevant agents participate
- Continuous multi-way conversation
- Agents create/update todo lists collaboratively
- Commits made when agents agree work units are complete
- Implementation continues until consensus that issue is resolved

**Architectural Decisions:**
- Relevant agents discuss and debate
- Document significant decisions in ADRs
- Continue implementation with agreed approach


## Architecture Decision Records (ADRs)

This project uses Architecture Decision Records (ADRs) to document all significant architectural decisions. ADRs help future developers understand not just what decisions were made, but why they were made and what alternatives were considered.

### Using ADRs in Development

When working on this project:

1. **Review existing ADRs** before making architectural changes:

   ```bash
   npm run adr:preview   # View ADRs in browser
   # Or browse docs/adr/ directory
   ```

2. **Create a new ADR** when making significant decisions:

   ```bash
   npm run adr:new       # Interactive ADR creation
   ```

3. **Update or supersede ADRs** when decisions change:
   - Mark old ADRs as "superseded by [new ADR]"
   - Create new ADR explaining the change

### What Requires an ADR?

Create an ADR for:

- Technology choices (databases, frameworks, libraries)
- Architectural patterns (event sourcing, CQRS, etc.)
- API design decisions
- Security approaches
- Performance optimization strategies
- Testing strategies
- Deployment and infrastructure decisions

### ADR Format

ADRs follow the template in `docs/adr/template.md` which includes:

- Context and problem statement
- Decision drivers
- Considered options with pros/cons
- Decision outcome
- Consequences (positive and negative)

### ADR Naming Convention

**IMPORTANT**: All ADRs must follow this naming convention:

- **Filename**: `NNNN-descriptive-name.md` where NNNN is the zero-padded ADR number (e.g., `0001-overall-architecture-pattern.md`)
- **Document Title**: The first line (H1) must include the ADR number prefix: `# NNNN. Title` (e.g., `# 0001. Overall Architecture Pattern`)
- Keep ADR numbers sequential and never reuse numbers
- The ADR number appears in both the filename AND the document title for consistency

### Publishing ADRs

ADRs are automatically published to GitHub Pages when merged to main:

- View at: https://jwilger.github.io/union_square/adr/
- Updated via GitHub Actions workflow

## Performance Targets

[Performance targets to be defined]

## Pre-commit Hooks & Code Quality

**🚨 NEVER bypass with `--no-verify`!**

Hooks run automatically on commit:

- **Rust**: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo check`
- **Files**: Whitespace cleanup, syntax checks, large file prevention
- **Commits**: Conventional Commits format enforcement

**Setup**: `pre-commit install && pre-commit install --hook-type commit-msg`

**If hooks fail**: Fix the issues, don't bypass them. Ask for help if needed.


## GitHub Issues Workflow

**ALL work is tracked through GitHub Issues. Agents collaborate to select and implement issues.**

### Agent-Based Issue Selection

1. **Claude Code gathers issues**: Use `mcp__github__list_issues` with `state="open"`

   **🚨 CRITICAL**: API paginates! Check ALL pages until empty results.

2. **Agent discussion**: 
   - Claude Code selects 2-4 planning agents
   - Provides full issue list to agents
   - Facilitates multi-round conversation
   - Agents consider: priority labels, dependencies, team expertise, user impact

3. **Consensus and assignment**: 
   - Agents reach consensus on next issue
   - User confirms selection
   - Use `mcp__github__update_issue` to assign

4. **Create branch**: `mcp__github__create_branch` with pattern: `issue-{number}-descriptive-name`

5. **Local checkout**:
   ```bash
   git fetch origin
   git checkout issue-{number}-descriptive-name
   ```

### Key MCP Tools

**Issues**: `list_issues`, `update_issue`, `add_issue_comment`
**Branches/PRs**: `create_branch`, `create_pull_request`, `update_pull_request`
**Workflows**: `list_workflow_runs`, `get_job_logs`, `rerun_failed_jobs`

**Advantages over gh CLI**: Direct API access, type safety, better error handling, batch operations.

## Pull Request Workflow

**All changes require PRs - no direct commits to main.**

### Creating PRs

1. **Push branch**: `git push -u origin branch-name`

2. **Create PR**: Use `mcp__github__create_pull_request`

   - **Title**: Follow Conventional Commits format (`feat: add feature`)
   - **Description**: Clear explanation of changes and motivation
   - **Labels**: `bug`, `enhancement`, `documentation`, `breaking-change`, etc.
   - Mention "Closes #{issue-number}" to auto-close issues

3. **CI runs automatically** - Monitor with MCP tools:
   - `mcp__github__get_pull_request` - Check status
   - `mcp__github__get_job_logs` - Debug failures

### Responding to Reviews

**Address ALL formal review comments (including bot reviews):**

1. **Get review details** using GraphQL API
2. **Reply to threads** using GraphQL mutation with `-- @claude` signature
3. **Format**: "I've addressed this by [action]. -- @claude"
4. **Check for responses** and continue conversation until resolved

**Note**: Definition of Done checklist is auto-added for HUMAN verification only.

## Agent Collaboration Summary

**Remember: Agents drive everything!**

- **Issue Selection**: 2-4 agents collaborate to choose work
- **Implementation**: ALL agents work together on the solution
- **Quality**: Agents review each other continuously
- **Completion**: Consensus required before moving on

Claude Code facilitates these conversations but agents make the decisions.

## 🔴 FINAL REMINDERS

**Before ANY task:**

1. **NEVER use `--no-verify`** - Fix issues, don't bypass checks
2. **Agents select GitHub Issues** - Through collaborative discussion
3. **Agents drive implementation** - Through multi-way conversations
4. **Ask for help when stuck** - Don't take shortcuts

**If pre-commit checks fail**: Fix the issues, run again, only commit when all pass. **IF YOU CANNOT FIX**: STOP and ASK FOR HELP.

**These rules are absolute. No exceptions. Ever.**
