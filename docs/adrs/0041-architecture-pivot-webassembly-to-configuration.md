---
title: "ADR-0041: Architecture Pivot: From WebAssembly to Configuration-First"
date: 2025-01-12
status: accepted
layout: adr
categories: [Architecture, Strategy]
---

## Status

Accepted

## Context

The Caxton project began with a WebAssembly-first architecture (ADR-0002) where
all agents were compiled WASM modules running in isolated sandboxes. This
approach provided strong security and language-agnostic agent development.

However, field experience and market validation revealed critical adoption
barriers:

### Original WebAssembly-First Problems

1. **High barrier to entry**: 2-4 hours to create first working agent
2. **Complex toolchain requirements**: Multiple language compilation pipelines
3. **Debugging difficulties**: WASM debugging tools still maturing
4. **Community friction**: Binary distribution complicates sharing

### Market Validation

Claude Code's success with 100+ community agents using simple configuration files
proved that most agent use cases (90%) are primarily orchestration logic that
can be handled by LLMs + tools, not custom algorithms requiring compilation.

## Decision

We are **pivoting to a configuration-first architecture** while maintaining
WebAssembly for specific use cases. This is not a replacement but a
**fundamental reordering of priorities**.

### New Architecture Hierarchy

1. **Primary UX**: Configuration agents (TOML files) - 5-10 minute onboarding
2. **Advanced Use Cases**: WebAssembly for MCP tool servers requiring sandboxing
3. **Power Users**: Direct WASM agents for custom algorithms (future)

### Role Clarification

- **Configuration Agents**: User-facing orchestration logic
- **WebAssembly**: Infrastructure for deployable MCP tool servers
- **Hybrid Model**: Both approaches coexist with clear boundaries

## Architecture Reconciliation

### How ADR-0002 (WebAssembly) and ADR-0028 (Configuration) Work Together

ADR-0002 is **NOT superseded** but **repurposed**:

- WebAssembly provides sandboxing for MCP tool servers
- Configuration agents orchestrate these sandboxed tools
- Security isolation is maintained where it matters most (external tool access)
- User-facing simplicity doesn't compromise system security

### Updated Component Responsibilities

```text
User Experience Layer:
  Configuration Agents (ADR-0028) → Simple TOML files
    ↓ orchestrates
Infrastructure Layer:
  MCP Tool Servers (ADR-0002) → WebAssembly sandboxes
    ↓ provides
Functionality Layer:
  HTTP clients, databases, file systems → Isolated execution
```

## Consequences

### Documentation Updates Required

The following documentation must be updated to reflect this pivot:

#### Immediate Priority

1. **README.md** - Already updated to emphasize config-first approach ✓
2. **ARCHITECTURE.md** - Already updated with hybrid model ✓
3. **docs/README.md** - Needs update to remove "WebAssembly-based agent isolation"
   as primary feature

#### ADRs to Clean Up

**Remove duplicates** (exist in both main and superseded directories):

- ADR-0003, 0007, 0008, 0010, 0012, 0013, 0014, 0015

**Mark as context/background** (not superseded, but deprioritized):

- ADR-0015 (Distributed Protocol) - Future scaling consideration
- ADR-0016 (Security Architecture) - Needs update for single-node reality

#### User-Facing Documentation

All getting-started guides, tutorials, and examples must:

- Lead with configuration agent examples
- Mention WebAssembly only for MCP tool development
- Remove compilation toolchain from basic setup

### Positive Outcomes

- **Dramatic adoption improvement**: 5-10 minute first agent experience
- **Community growth**: Text-based agents are shareable and forkable
- **Clearer mental model**: Users understand agents as configurations, not binaries
- **Maintained security**: WebAssembly still isolates dangerous operations

### Risks and Mitigations

- **Risk**: Confusion about when to use config vs WASM
  - **Mitigation**: Clear decision tree in documentation
- **Risk**: Performance concerns with LLM-based orchestration
  - **Mitigation**: MCP tools handle compute-intensive tasks in WASM
- **Risk**: Security perception with config agents
  - **Mitigation**: Emphasize WebAssembly isolation for actual tool execution

## Implementation Strategy

### Phase 1: Documentation Alignment (Immediate)

1. Update all references to prioritize configuration agents
2. Clean up duplicate ADRs
3. Create migration guides for existing WASM agent patterns

### Phase 2: Developer Experience (Next Sprint)

1. Agent template library covering 80% of use cases
2. Hot-reload development for configuration agents
3. Clear examples showing config + MCP tool integration

### Phase 3: Community Features (Future)

1. Agent marketplace for sharing configurations
2. A/B testing framework for agent behaviors
3. Performance monitoring specific to LLM orchestration

## Alignment with Core Philosophy

This pivot **strengthens** our core principles:

- **ADR-0001 (Observability First)**: Config agents provide better visibility
- **ADR-0004 (Minimal Core)**: Simpler core with optional complexity
- **ADR-0005 (MCP for Tools)**: MCP becomes the primary extension mechanism
- **ADR-0030 (Embedded Memory)**: Zero-dependency remains achievable

## Decision Rationale

The configuration-first approach is the right architecture because:

1. **User needs validation**: 90% of agents are orchestration, not algorithms
2. **Market proof**: Claude Code's success with similar model
3. **Adoption trajectory**: 5-10 minutes vs 2-4 hours to first success
4. **Security maintained**: WebAssembly still isolates dangerous operations
5. **Future-proof**: Can always add more WASM capabilities as needed

## References

- ADR-0002: WebAssembly for Agent Isolation (repurposed, not superseded)
- ADR-0028: Configuration-Driven Agent Architecture (primary UX)
- ADR-0029: FIPA-ACL Lightweight Messaging (simplified protocol)
- ADR-0030: Embedded Memory System (zero-dependency approach)
- Claude Code documentation and community adoption metrics
