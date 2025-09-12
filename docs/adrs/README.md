# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the Caxton
project. ADRs document important architectural decisions made during the
development of the system, including the context, decision rationale, and
consequences.

## Directory Structure

The ADRs are organized by status to improve navigation and clarity:

```text
docs/adrs/
├── README.md                          # This file
├── superseded/                        # Historical decisions that have been replaced
│   ├── 0003-fipa-messaging-protocol.md
│   ├── 0007-management-api-design.md
│   ├── 0008-agent-deployment-model.md
│   ├── 0010-external-agent-routing-api.md
│   ├── 0012-pragmatic-fipa-subset.md
│   ├── 0013-state-management-architecture.md
│   ├── 0014-coordination-first-architecture.md
│   └── 0015-distributed-protocol-architecture.md
├── 0001-observability-first-architecture.md    # Active decisions (accepted status)
├── 0002-webassembly-for-agent-isolation.md
├── 0004-minimal-core-philosophy.md
└── ... (all other accepted ADRs)
```

## ADR Status Definitions

### **Accepted** (Top Level)

- **Location**: `docs/adrs/*.md`
- **Meaning**: Current architectural decisions that are actively guiding the
  system design
- **Count**: 19 ADRs
- **Examples**: Observability-First Architecture, WebAssembly Isolation,
  Domain Types with nutype

### **Superseded** (superseded/ subdirectory)

- **Location**: `docs/adrs/superseded/*.md`
- **Meaning**: Historical decisions that have been replaced by newer ADRs
- **Count**: 8 ADRs
- **Examples**: Original FIPA Messaging Protocol, Management API Design,
  Agent Deployment Model

### **Proposed** (proposed/ subdirectory)

- **Location**: `docs/adrs/proposed/*.md` (when created)
- **Meaning**: Decisions under consideration but not yet implemented
- **Count**: 0 ADRs currently

### **Deprecated** (deprecated/ subdirectory)

- **Location**: `docs/adrs/deprecated/*.md` (when created)
- **Meaning**: Decisions that are no longer recommended but haven't been replaced
- **Count**: 0 ADRs currently

## Status Lifecycle

ADRs typically follow this lifecycle:

```text
proposed → accepted → superseded
         ↘ deprecated
```

- **Proposed**: New architectural decision under consideration
- **Accepted**: Decision has been made and is being implemented
- **Superseded**: Decision has been replaced by a newer ADR (with
  `superseded_by` field)
- **Deprecated**: Decision is no longer recommended but hasn't been formally replaced

## Navigation Guide

### Finding Current Architecture

- **Start here**: Browse the top-level `*.md` files for all current
  architectural decisions
- **Quick overview**: All accepted ADRs are immediately visible in the main directory
- **GitHub browsing**: The main directory shows only active decisions, reducing clutter

### Understanding Evolution

- **Historical context**: Check `superseded/` to understand how the
  architecture evolved
- **Decision relationships**: Look for `superseded_by` fields in superseded ADRs
- **Timeline**: ADR numbers generally indicate chronological order of original decisions

### Contributing New ADRs

1. **Draft**: Create new ADR with `status: proposed` in main directory
2. **Review**: Team reviews and discusses the proposal
3. **Accept**: Change status to `accepted` if approved
4. **Supersede**: When replacing an existing ADR:
   - Set old ADR status to `superseded`
   - Add `superseded_by: ADR-NNNN` field to old ADR
   - Move old ADR to `superseded/` directory

## ADR Format

Each ADR follows this YAML frontmatter format:

```yaml
---
title: "ADR-NNNN: Brief Decision Title"
date: YYYY-MM-DD
status: accepted|proposed|superseded|deprecated
layout: adr
categories: [Architecture]
superseded_by: ADR-NNNN  # Only for superseded ADRs
---
```

## Key Architectural Areas

The current accepted ADRs cover these major architectural areas:

- **Infrastructure**: Observability, WebAssembly isolation, minimal core philosophy
- **Communication**: FIPA messaging, capability registration
- **Data Management**: Domain types, primitive boundaries, memory systems
- **Configuration**: TOML-based agent configuration, context management
- **Security**: Security architecture and validation approaches

## Related Documentation

- [Architecture Overview](../architecture/README.md) - High-level system architecture
- [Developer Guide](../developer/README.md) - Implementation guidance
- [Contributing Guidelines](../../CONTRIBUTING.md) - How to contribute to the project

---

*This organization improves the developer experience by keeping current
architectural decisions prominently visible while preserving historical
context for understanding the system's evolution.*
