<!--
Sync Impact Report:
- Version change: 1.2.1 → 1.3.0
- Modified principles: None renamed
- Added sections:
  * XII. Outside-In Black-Box Testing Methodology (new principle)
- Removed sections: None
- Templates requiring updates:
  ✅ .specify/templates/plan-template.md - Constitution Check section updated
  ✅ .specify/templates/tasks-template.md - Testing workflow refined
- Follow-up TODOs: None
-->

# Caxton Constitution

## Core Principles

### I. Configuration-First Platform Design

Caxton MUST provide a platform where end-users can deploy agents via TOML configuration files. The platform MUST support 5-10 minute agent deployment time from configuration to running agent. Platform APIs MUST prioritize configuration-driven deployment over programmatic complexity.

### II. Minimal Core Architecture

Caxton core provides only: Agent Runtime, Message Router, and Observability Layer. All other functionality MUST be implemented as deployed agents or MCP servers. Feature requests for core expansion require justification against minimal philosophy.

### III. Type-Driven Safety (NON-NEGOTIABLE)

Illegal states MUST be unrepresentable in the type system. All external inputs MUST be parsed at boundaries with comprehensive validation. Internal operations MUST assume validated types. Railway-oriented programming required for error handling.

### IV. Observability First

Every operation MUST be traced with OpenTelemetry. Structured logging required for all components. Metrics collection mandatory for performance monitoring. Debug information MUST be available without recompilation.

### V. Zero External Service Dependencies

Caxton MUST function immediately after single binary installation without requiring external services. End-users MUST NOT be forced to set up databases, vector stores, event stores, or other external services for basic functionality. Embedded SQLite + Candle memory system by default. External services (Neo4j, Qdrant) are optional scaling enhancements, not requirements. Third-party Rust crates are permitted and encouraged for implementation quality.

### VI. Architectural Decision Records (ADRs)

ALL major architectural decisions affecting this project MUST be recorded using ADRs kept as markdown files in the docs/adrs directory. ADRs MUST follow the same format/template and describe the what and why, but not how or when. ADRs MUST provide an accounting of what alternatives were considered and why the chosen decision was taken. ADRs are the authoritative source of architectural reasoning.

### VII. Comprehensive User Documentation

Comprehensive user documentation MUST be provided for the product and kept up to date as changes are made. Documentation MUST target three distinct audiences: contributors (people working on Caxton itself), system administrators (people responsible for installing, running, securing, and maintaining installations of Caxton), and developers (people responsible for developing agents and MCP servers that will be deployed to Caxton). Each audience's documentation MUST be complete, accurate, and maintained in parallel with product changes.

### VIII. Professional Website Standards

The professional landing page website in the website directory MUST be consistent with the features and documentation for the product. The website MUST accurately represent current capabilities and roadmap. Website content MUST be updated whenever product features change to maintain consistency and professional appearance.

### IX. GitHub Pull Request Workflow

GitHub PR flow MUST be used for working on features. GitHub Actions checks MUST be maintained that must pass before a PR can be merged. All feature work MUST go through pull requests with appropriate review processes. The main branch MUST be protected and only accept changes through approved pull requests.

### X. Pre-commit Hook Compliance

Pre-commit hooks MUST be used to verify code and documentation quality prior to committing changes. ALL pre-commit hooks MUST pass before a commit is allowed. The `--no-verify` flag or any other means to circumvent pre-commit hooks MUST NOT be used. Code quality and documentation standards are enforced at commit time to maintain consistency.

### XI. Mandatory Research Agent Delegation

ALL research, analysis, and planning tasks MUST be delegated to specialized research agents to preserve main context for implementation. Main agents MUST NOT perform research that specialized agents can handle. Research agents operate in separate context windows and save findings to `.claude/docs/` for main agent consumption. MANDATORY agent delegation rules:

- Requirements clarification → requirements-analyst agent
- Codebase analysis → codebase-scanner agent
- Implementation design options → implementation-planner agent
- Pattern and best practice research → documentation-expert agent
- Test strategy planning → test-designer agent

Violation of delegation requirements constitutes inefficient context usage and MUST be avoided. Main agents MUST read agent outputs from `.claude/docs/[agent-name]-plan.md` files before proceeding with implementation decisions.

### XII. Outside-In Black-Box Testing Methodology (NON-NEGOTIABLE)

ALL feature implementation MUST begin with outside-in black-box integration tests that verify only externally visible system behavior. Tests MUST be placed in the `tests/` directory and MUST NOT require changes when internal implementation details change. Integration tests MUST be written BEFORE any types or implementation code.

**MANDATORY 11-Step Process**:

1. Write black-box integration test testing only externally visible behavior
2. Run test immediately to ensure it fails as expected
3. If compilation errors exist, fix with smallest possible change, repeat until clean compilation
4. If linting errors exist, fix with smallest change using automatic tools (`cargo fmt`, `cargo clippy`), repeat until clean
5. If unexpected logic error, verify test correctness and problem understanding before proceeding
6. If test failure makes obvious fix clear, skip step 7 and proceed to step 8
7. If test failure unclear, write deeper unit test, mark previous test as skipped, maintain single failing test
8. Make smallest implementation change to make failing test pass
9. Remove hard-coded values from test, ensure total functions, make smallest change for updated test to pass
10. Leverage type system to make test failure impossible if possible, delete test if compilation prevents failure
11. Commit progress, remove skipped markers, continue with next test

**Single Failing Test Rule**: Only ONE test MUST be failing at any time. All other tests MUST pass or be explicitly skipped.

**Black-Box Requirement**: Tests MUST verify system behavior through public interfaces only (HTTP endpoints, CLI commands, file outputs) without knowledge of internal implementation structure.

## Development Workflow

All Caxton platform development MUST follow Test-Driven Development (TDD) with strict Red-Green-Refactor cycles. Platform feature specifications MUST be written before implementation. Type-driven design principles guide all architectural decisions. The platform serves end-users who deploy configuration agents - the platform itself should minimize compilation complexity.

Research phases MUST utilize appropriate specialized agents rather than inline analysis to preserve main context for implementation work. Agent delegation is mandatory for efficient token usage and context preservation.

## Security Requirements

Hybrid security model: The platform runs end-user configuration agents in host runtime for rapid deployment. MCP servers providing system access MUST execute in WebAssembly sandboxes with resource limits and capability allowlists. The platform MUST enforce that agent tool access is explicitly declared in configurations.

## Governance

Constitution supersedes all other platform development practices. All pull requests MUST verify compliance with constitutional principles. Complexity additions MUST be justified against minimal core philosophy. Use `.claude/docs/` files for runtime development guidance. Amendments require documented justification, community discussion, and migration plan for affected code.

**Version**: 1.3.0 | **Ratified**: 2025-09-22 | **Last Amended**: 2025-09-22
