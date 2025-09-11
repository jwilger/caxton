---
layout: adr-index
title: "Architecture Decision Records"
permalink: /adr/
date: 2025-09-10
categories: [Website]
---

This is a log of all architecture decisions made for the Caxton project. These
ADRs document the key architectural choices, their context, and the reasoning
behind them.

## What are ADRs?

An Architecture Decision Record (ADR) is a document that captures an important
architectural decision made along with its context and consequences. ADRs help
future developers understand the reasons behind architectural choices.

## Current ADRs

The following ADRs document the architectural decisions for Caxton:

## Configuration-First Architecture (Latest - September 2025)

**NEW: These ADRs establish the configuration-driven agent platform:**

- **[ADR-0028: Configuration-Driven Agent Architecture](0028-configuration-driven-agent-architecture)**
  ✨ **ACCEPTED** - Markdown + YAML agents as primary UX (5-10 minute
  onboarding)
- **[ADR-0029: Agent Messaging Protocol](0029-fipa-acl-lightweight-messaging)**
  ✨ **ACCEPTED** - Capability-based routing optimized for config agents
- **[ADR-0030: Embedded Memory System](0030-embedded-memory-system)**
  ✨ **ACCEPTED** - SQLite + local embeddings with zero dependencies

## Foundation Architecture

- **[ADR-0001: Observability-First Architecture](0001-observability-first-architecture)**
  - **ACCEPTED** - Comprehensive tracing and monitoring
- **[ADR-0002: WebAssembly for Agent Isolation](0002-webassembly-for-agent-isolation)**
  - **ACCEPTED** - WASM sandboxing for security (power users)
- **[ADR-0003: Agent Messaging Protocol](0003-fipa-messaging-protocol)**
  - **SUPERSEDED** by ADR-0029 - Original messaging implementation
- **[ADR-0004: Minimal Core Philosophy](0004-minimal-core-philosophy)**
  - **ACCEPTED** - Lightweight design principles
- **[ADR-0005: MCP for External Tools](0005-mcp-for-external-tools)**
  - **PROPOSED** - Model Context Protocol integration

## Application Architecture

- **[ADR-0006: Application Server Architecture](0006-application-server-architecture)**
  - **ACCEPTED** - Standalone server design
- **[ADR-0007: Management API Design](0007-management-api-design)**
  - **SUPERSEDED** by ADR-0026 - Original API design
- **[ADR-0008: Agent Deployment Model](0008-agent-deployment-model)**
  - **PROPOSED** - Agent lifecycle management
- **[ADR-0009: CLI Tool Design](0009-cli-tool-design)**
  - **PROPOSED** - Command-line interface
- **[ADR-0010: External Agent Routing API](0010-external-agent-routing-api)**
  - **PROPOSED** - External system integration

## Communication and Coordination

- **[ADR-0011: Capability Registration in Code](0011-capability-registration-in-code)**
  - **ACCEPTED** - Code-based capability declaration
- **[ADR-0012: Pragmatic Agent Messaging Subset](0012-pragmatic-fipa-subset)**
  - **SUPERSEDED** by ADR-0029 - Original messaging simplification
- **[ADR-0013: State Management Architecture](0013-state-management-architecture)**
  - **ACCEPTED** - Distributed state handling
- **[ADR-0014: Coordination-First Architecture](0014-coordination-first-architecture)**
  - **ACCEPTED** - Lightweight coordination patterns
- **[ADR-0015: Distributed Protocol Architecture](0015-distributed-protocol-architecture)**
  - **ACCEPTED** - Multi-node communication

## Type Safety and Domain Modeling

- **[ADR-0016: Security Architecture](0016-security-architecture)**
  - **ACCEPTED** - Security boundaries and policies
- **[ADR-0018: Domain Types with nutype](0018-domain-types-nutype)**
  - **ACCEPTED** - Type-safe domain primitives
- **[ADR-0019: Primitives at Boundaries](0019-primitives-at-boundaries)**
  - **ACCEPTED** - Boundary validation patterns
- **[ADR-0020: Parse Don't Validate](0020-parse-dont-validate)**
  - **ACCEPTED** - Parse to domain types
- **[ADR-0021: Atomic Primitives Exception](0021-atomic-primitives-exception)**
  - **ACCEPTED** - When to use primitive types

## Recent Simplifications

- **[ADR-0026: Simplified Management API Protocol](0026-simplified-management-api-protocol)**
  - **ACCEPTED** - Single REST/HTTP protocol
- **[ADR-0027: Single Codebase Architecture](0027-single-codebase-architecture)**
  - **ACCEPTED** - Monorepo organization
- **[ADR-0031: Context Management Architecture](0031-context-management-architecture)**
  - **ACCEPTED** - Intelligent context management for configuration agents

## ADR Status Legend

- **✨ NEW** - Recently added (September 2025)
- **ACCEPTED** - Active architectural decision
- **PROPOSED** - Under consideration
- **SUPERSEDED** - Replaced by newer ADR
- **DEPRECATED** - No longer recommended
