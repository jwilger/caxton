<!--
Sync Impact Report:
- Version change: 1.0.0 → 1.0.2
- Modified principles: I. Configuration-First Development (clarified platform vs end-user scope)
- Added sections: None
- Removed sections: None
- Templates requiring updates:
  ✅ .specify/templates/plan-template.md - Updated Constitution Check gates and version reference
  ✅ .specify/templates/spec-template.md - No updates needed (no constitution references)
  ✅ .specify/templates/tasks-template.md - No updates needed (already TDD-aligned)
  ✅ .specify/templates/agent-file-template.md - No updates needed (generic template)
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

### V. Zero External Dependencies

Caxton MUST function immediately after single binary installation. Embedded SQLite + Candle memory system by default. External backends (Neo4j, Qdrant) are optional scaling enhancements, not requirements.

## Development Workflow

All Caxton platform development MUST follow Test-Driven Development (TDD) with strict Red-Green-Refactor cycles. Platform feature specifications MUST be written before implementation. Type-driven design principles guide all architectural decisions. The platform serves end-users who deploy configuration agents - the platform itself should minimize compilation complexity.

## Security Requirements

Hybrid security model: The platform runs end-user configuration agents in host runtime for rapid deployment. MCP servers providing system access MUST execute in WebAssembly sandboxes with resource limits and capability allowlists. The platform MUST enforce that agent tool access is explicitly declared in configurations.

## Governance

Constitution supersedes all other platform development practices. All pull requests MUST verify compliance with constitutional principles. Complexity additions MUST be justified against minimal core philosophy. Use `.claude/docs/` files for runtime development guidance. Amendments require documented justification, community discussion, and migration plan for affected code.

**Version**: 1.0.2 | **Ratified**: 2025-09-22 | **Last Amended**: 2025-09-22
