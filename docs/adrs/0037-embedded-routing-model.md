---
title: "ADR-0037: Embedded Routing Model"
date: 2025-01-17
status: accepted
layout: adr
categories: [Architecture, Routing, AI, Performance]
---

## Status

Accepted

## Context

Caxton's multi-agent system requires intelligent routing to select the best
agent when multiple agents share the same capability. This decision impacts
how the system determines which agent should handle a given request when
concurrent execution is possible and multiple agents are qualified.

### Core Challenge

The routing problem in Caxton differs from traditional load balancing:

- **Concurrent Capability**: Multiple agents can have the same capability and
  run concurrently
- **Quality Over Distribution**: Need to select the BEST agent, not just any
  available agent
- **Semantic Understanding**: Routing decisions require understanding request
  context and agent strengths
- **Zero-Dependency Promise**: Solution must maintain Caxton's zero external
  dependency commitment
- **Sub-10ms Performance**: Routing decisions cannot become a bottleneck

### Requirements

The routing solution must satisfy:

- **Intelligent Selection**: Choose the most appropriate agent based on
  semantic understanding
- **Fast Decision Making**: <10ms routing decisions for production viability
- **Zero External Dependencies**: Work out of the box without external
  services
- **Graceful Degradation**: Fall back to simpler strategies if intelligent
  routing fails
- **Progressive Enhancement**: Allow optional upgrade to more sophisticated
  routing

### Alternative Approaches Considered

**External LLM API**: Rejected due to external dependency requirement and
50-200ms latency.

**Simple Heuristics**: Round-robin or random selection lacks intelligence for
optimal agent selection.

**External Embedding Service**: Violates zero-dependency principle and adds
operational complexity.

**Python Sidecar Process**: Adds deployment complexity and inter-process
communication overhead.

## Decision

We will **embed a DistilBERT model directly in the Caxton binary** for
intelligent routing, using Candle.rs for pure Rust inference without Python
dependencies.

### Architecture Overview

The routing system implements a three-tier architecture:

1. **Cache Layer** (<1ms): LRU cache with 5-minute TTL for repeated routing
   decisions
2. **Embedded Model** (<5ms): DistilBERT inference for semantic similarity
   routing
3. **Optional External** (50-200ms): MCP tool integration for advanced LLM
   routing

### Technical Approach

**Model Selection**: DistilBERT provides the optimal balance of size (10-50MB)
and capability for semantic similarity tasks.

**Inference Engine**: Candle.rs enables pure Rust inference without Python
runtime or external dependencies.

**Binary Integration**: Model weights embedded directly in the binary at
compile time, similar to how SQLite embeds its database engine.

**Fallback Strategy**: Automatic fallback to round-robin routing if model
inference fails, ensuring system resilience.

## Consequences

### Positive Consequences

- **True Zero-Dependency**: System works intelligently out of the box with no
  external services
- **Industry Differentiation**: First truly zero-dependency intelligent
  multi-agent system
- **Predictable Performance**: <5ms routing decisions enable production use
  cases
- **Progressive Enhancement**: Can upgrade to external LLM via MCP tools when
  needed
- **Operational Simplicity**: No external services to manage, monitor, or
  scale

### Negative Consequences

- **Binary Size Increase**: Total binary grows to 80-120MB with embedded
  models
- **Initial Download**: Larger binary means longer initial download time
- **Model Updates**: Updating the routing model requires binary replacement
- **Limited Sophistication**: DistilBERT less capable than larger language
  models

### Performance Characteristics

- **Cache Hit**: <1ms for cached routing decisions
- **Model Inference**: <5ms for new routing decisions
- **Memory Usage**: ~100MB additional RAM for model in memory
- **CPU Usage**: Brief spike during inference, negligible otherwise

## Implementation Approach

The implementation follows a phased approach:

1. **Candle.rs Integration**: Add Candle dependency and DistilBERT model support
2. **Model Embedding**: Include model weights in binary at compile time
3. **Cache Layer**: Implement LRU cache with configurable TTL
4. **Fallback Logic**: Round-robin fallback for model failures
5. **MCP Extension**: Optional external LLM routing via MCP tools

## Alignment with Existing ADRs

This decision aligns with and reinforces:

- **ADR-0004 (Minimal Core)**: Embedded model maintains minimal external
  dependencies
- **ADR-0027 (Single Codebase)**: All routing logic contained within main
  binary
- **ADR-0028 (Config Agents)**: Enables intelligent selection among config
  agents
- **ADR-0036 (Trust Model)**: Routing decisions occur within trusted process
  boundary

## Industry Precedent

The embedded model approach follows established patterns:

- **SQLite**: Embeds full database engine in applications
- **DuckDB**: Embeds analytical database in process
- **LanceDB**: Embeds vector database for similarity search
- **Tantivy**: Embeds full-text search engine in Rust applications

These successful projects demonstrate that embedding sophisticated
capabilities directly in binaries is both practical and valuable.

## Future Considerations

- **Model Swapping**: Support for loading alternative models at runtime
- **Fine-Tuning**: Agent-specific model fine-tuning for improved routing
- **Telemetry**: Routing decision telemetry for continuous improvement
- **Multi-Model**: Different models for different capability domains

## References

- [Candle.rs - Rust-native deep learning
  framework](https://github.com/huggingface/candle)
- [DistilBERT - Distilled version of
  BERT](https://huggingface.co/docs/transformers/model_doc/distilbert)
- [SQLite - Embedded database
  precedent](https://www.sqlite.org/selfcontained.html)
- ADR-0028: Configuration-Driven Agent Architecture
- ADR-0036: Config Agent Trust Model
