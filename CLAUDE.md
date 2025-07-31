# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) for orchestrating agent-based development in this repository.

**Purpose**: This file defines HOW Claude Code facilitates agent collaboration. Technical details and domain expertise reside in individual agent files (`/.claude/agents/`).

## 🚨 CRITICAL RULES - ALWAYS APPLY

**These rules must NEVER be violated under any circumstances:**

1. **NEVER use the `--no-verify` flag when committing code**
2. **ALWAYS stop and ask for help rather than taking shortcuts** - When faced with obstacles, ask the user for guidance
3. **ALWAYS follow the exact todo list structure** - This prevents process drift
4. **Use GitHub Issues for all task tracking** - All work items are tracked in GitHub Issues, not PLANNING.md
5. **ALWAYS consult expert agents before ANY changes** - Claude Code MUST launch expert agents and obtain consensus through WORK.md before making ANY modifications to code, documentation, configuration files, or any other project artifacts. This includes every step of the TDD cycle (writing tests, making them pass, refactoring) and every git operation that changes content. NO EXCEPTIONS - even typo fixes, formatting changes, and comment updates require expert consensus. Changes made without expert consensus must NEVER be committed.

**🚨 CRITICAL REMINDER**: Because you are forgetful, remind yourself of these most important rules by including them in every single reply:
1. DO NOT USE THE --no-verify FLAG TO COMMIT CODE. EVER.
2. ALWAYS CONSULT EXPERT AGENTS BEFORE ANY CHANGES - No exceptions, even for typos.
(and also include these directions to include them in every single reply in every single reply.)

## 📋 QUICK REFERENCE

- **🆕 Starting new work?** → [Development Workflow](#development-workflow), [GitHub Issues](#github-issues-workflow)
- **🔧 Setting up environment?** → [Development Commands](#development-commands)
- **💻 Writing code?** → [Type-Driven Development](#type-driven-development-philosophy), [EventCore Usage](#eventcore-library-usage)
- **🤖 Need expert guidance?** → [Expert Agents](#expert-agent-coordination)
- **📤 Making commits/PRs?** → [Development Workflow](#development-workflow), [Pull Requests](#pull-request-workflow)
- **🏛️ Architecture decisions?** → [ADRs](#architecture-decision-records-adrs)
- **📝 Making ANY change?** → [Change Request Protocol](#change-request-protocol)

## Project Overview

Union Square is a proxy/wire-tap service for making LLM calls and recording everything that happens in a session for later analysis and test-case extraction.

## Agent-Based Development Model

**IMPORTANT**: This project uses an agent-based development model where specialized AI agents collaborate to drive all development work through a structured TDD workflow.

### Communication Protocol

Agents communicate using shared workspace files:

**WORK.md** - For inter-agent discussions:
- Reset when starting a new issue
- Compacted if it grows too large during work
- Used for all inter-agent discussions and consensus building

**DIALOGUE.md** - For expert-to-user communication:
- Used when experts need user input during work
- Contains structured requests and responses
- Cleared when starting new issues
- Claude Code monitors and facilitates the exchange

#### Bidirectional Communication Flow

When expert agents need user input:

1. **Experts identify need** during WORK.md discussions
2. **Project Manager writes to DIALOGUE.md** with:
   - Request type: `[BLOCKING]` or `[INFORMATIONAL]`
   - Topic and context
   - Specific numbered questions
3. **Project Manager adds marker** in WORK.md: `**PM: User input requested in DIALOGUE.md**`
4. **Claude Code detects marker** and:
   - Reads questions from DIALOGUE.md
   - Presents them to user in chat
   - Captures user response
   - Writes response back to DIALOGUE.md
5. **Project Manager detects response** and continues expert discussion

**Request Format in DIALOGUE.md:**
```markdown
## [BLOCKING|INFORMATIONAL] Request from Experts - [Timestamp]
**Topic**: [Brief description]
**Context**: [Why this information is needed]
**Questions**:
1. [Specific question 1]
2. [Specific question 2]

**Response Needed By**: [For BLOCKING only - timestamp]

## User Response - [Timestamp]
[User's response here]
```

**Claude Code Monitoring:**
- Check WORK.md for PM marker after each agent task completion
- Present BLOCKING requests immediately
- Re-prompt for BLOCKING requests every 5 minutes if unanswered
- Clear DIALOGUE.md when starting new issues

### Development Roles

**Expert Agents**: Specialized AI personas that:
1. **Plan work** - Review and prioritize GitHub issues collaboratively
2. **Design solutions** - Create TDD-appropriate incremental steps
3. **Review implementation** - Validate each step meets requirements
4. **Reach consensus** - All agents must agree before proceeding

**Project Manager Agent**: Bridge between experts and Claude Code:
- Monitors WORK.md for expert consensus
- Communicates next TDD steps to Claude Code
- Presents implementation results back to experts
- Escalates if consensus isn't reached after 10 rounds

**Claude Code**: Executes implementation:
- Receives specific TDD instructions from Project Manager
- Implements exactly what is requested (no more, no less)
- Reports results back through Project Manager
- Does NOT participate in planning discussions

## Development Workflow

**🚨 ALWAYS follow this exact agent-based workflow:**

### Workflow Steps

1. **Agent-Based Issue Selection**
   - Launch Project Manager and 2-4 planning agents
   - Agents discuss in WORK.md to select highest priority issue
   - Project Manager presents consensus to Claude Code
   - User confirms the selected issue

2. **Get assigned to selected issue** - Use `mcp__github__update_issue` to assign

3. **Create feature branch** - Use `mcp__github__create_branch` with pattern: `issue-{number}-descriptive-name`

4. **TDD Implementation Loop**
   - Expert agents plan next TDD step in WORK.md
   - Project Manager communicates step to Claude Code:
     - "Write a test that asserts X"
     - "Make minimal change to pass the test"
     - "Refactor while keeping tests green"
   - Claude Code implements and reports results
   - Experts review results and plan next step
   - Continue until issue is complete

### TDD Workflow Management

**Implementation follows strict TDD cycles:**

**Red Phase (Test First):**
- Experts specify exact test to write
- Project Manager communicates to Claude Code
- Claude Code writes failing test and reports output

**Green Phase (Make It Pass):**
- Experts analyze failure and plan minimal fix
- Project Manager communicates implementation step
- Claude Code implements and reports results

**Refactor Phase (Improve Design):**
- Experts review working code for improvements
- Project Manager communicates refactoring steps
- Claude Code refactors and verifies tests still pass

**Commits:** Made at expert-determined boundaries (typically after each complete Red-Green-Refactor cycle)

**Note**: Each phase transition requires expert consensus:
- Before writing any test → Expert approval
- Before implementing to pass test → Expert approval  
- Before any refactoring → Expert approval
- Before committing → Expert verification

### Change Request Protocol

**ALL changes, regardless of size or complexity, MUST follow this protocol:**

1. **Identify the change** - Claude Code identifies what needs to be modified
2. **Launch expert agents** - Select 2-4 relevant experts based on the change type
3. **Present to experts** - Clearly describe the proposed change in WORK.md
4. **Expert discussion** - Facilitate expert discussion until consensus
5. **Implement consensus** - Make ONLY the changes agreed upon by experts
6. **Report back** - Show results to experts for verification

**Examples requiring expert consultation:**
- Writing a new test
- Fixing a failing test
- Adding a new function
- Modifying existing code
- Updating documentation
- Fixing typos
- Adjusting formatting
- Adding or modifying comments
- Changing configuration files
- Updating dependencies

### Commit Requirements

- **Expert consensus required** - NEVER commit changes that weren't reviewed by expert agents
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
| Project Manager    | `project-manager`                                         | Expert team coordination, TDD workflow management, Claude Code communication |

### Core Architectural Principles

When multiple experts are involved in a decision, these principles guide resolution:

1. **Type Safety First**: When conflicts arise, type system recommendations (Simon Peyton Jones/Niko Matsakis) take precedence
2. **Event Sourcing is Non-Negotiable**: Greg Young's event patterns are foundational - other patterns must adapt to this
3. **TDD is the Process**: Kent Beck drives the implementation workflow - no code without tests
4. **Functional Core, Imperative Shell**: Rich Hickey owns the boundary between pure and impure code

### Agent-Driven Development Process

**CRITICAL**: All expert agents must be launched as actual independent AI agents using the Task tool. The Project Manager MUST NOT simulate or impersonate other expert agents.

#### How Claude Code Launches Expert Agents

1. **Launch Multiple Agents Concurrently**: Use the Task tool to launch multiple expert agents in a single message
2. **Each agent gets the same context**: Provide WORK.md contents and current task
3. **Agents write their responses to WORK.md**: Each agent contributes independently
4. **Project Manager synthesizes**: After all agents respond, Project Manager facilitates consensus

#### Issue Selection Phase

Claude Code launches Project Manager + 2-4 planning agents CONCURRENTLY:
- **Product/Business Focus**: Teresa Torres (`product-discovery-coach`), Jared Spool (`ux-research-expert`)
- **Technical Planning**: Nicole Forsgren (`engineering-effectiveness-expert`), Martin Fowler (`refactoring-patterns-architect`)
- **Domain Modeling**: Alberto Brandolini (`event-modeling-expert`), Greg Young (`event-sourcing-architect`)

All agents discuss priorities in WORK.md until consensus.

#### TDD Implementation Phase

Claude Code launches Project Manager + ALL relevant expert agents CONCURRENTLY who:
1. **Plan each TDD step** - Specific test or implementation instruction
2. **Review results** - Validate Claude Code's implementation
3. **Reach consensus** - All must agree before next step
4. **Track progress** - Ensure incremental value delivery

Project Manager ensures smooth communication flow between experts and Claude Code.

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

The Project Manager facilitates expert collaboration through WORK.md:

**Consensus Building:**
- Experts present perspectives in WORK.md
- Discuss until convergence toward agreement
- Maximum 10 rounds before escalation to user
- Document significant decisions in ADRs

**Communication Flow:**
1. Experts discuss and reach consensus in WORK.md
2. Project Manager translates consensus to Claude Code
3. Claude Code implements and reports results
4. Project Manager presents results to experts
5. Cycle continues until feature complete

**Escalation:** If no consensus after 10 rounds with diverging opinions, Project Manager escalates to user.

### Integration with Development Workflow

**Issue Selection:**
1. Claude Code launches Project Manager + planning agents
2. Agents review GitHub issues in WORK.md
3. Reach consensus on priority
4. Project Manager communicates selection to Claude Code
5. User confirms

**TDD Implementation:**
1. Claude Code launches Project Manager + ALL expert agents
2. Experts plan next TDD step in WORK.md
3. Project Manager instructs Claude Code
4. Claude Code implements and reports
5. Experts review and plan next step
6. Repeat until issue complete

**Quality Gates:**
- Every step requires expert consensus
- Tests must pass before proceeding
- Refactoring preserves all tests
- Architecture decisions documented in ADRs


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
   
   **PR Description must include:**
   ```markdown
   ## Expert Review Confirmation
   - [ ] All changes were discussed in WORK.md
   - [ ] Expert consensus was reached before implementation
   - [ ] Link to WORK.md discussion: [provide commit SHA where WORK.md shows consensus]
   ```

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

**Remember: TDD drives everything through expert consensus!**

- **Communication**: All agents use WORK.md for discussions
- **Coordination**: Project Manager bridges experts and Claude Code
- **Implementation**: Claude Code executes only what Project Manager instructs
- **Quality**: Every TDD step reviewed by all experts
- **Consensus**: Required at each step before proceeding

**Claude Code's Role**: Execute implementation steps exactly as instructed by Project Manager. Do not participate in planning or decision-making.

## 🔴 FINAL REMINDERS

**Before ANY task:**

1. **NEVER use `--no-verify`** - Fix issues, don't bypass checks
2. **Agents select GitHub Issues** - Through collaborative discussion
3. **Agents drive implementation** - Through multi-way conversations
4. **Ask for help when stuck** - Don't take shortcuts

**If pre-commit checks fail**: Fix the issues, run again, only commit when all pass. **IF YOU CANNOT FIX**: STOP and ASK FOR HELP.

**These rules are absolute. No exceptions. Ever.**
