---
title: "ADR-0030: Embedded Memory"
date: 2025-09-09
status: accepted
layout: adr
categories: [Architecture]
---

## Status

Accepted

## Context

Configuration-driven agents (ADR-0028) need persistent memory to provide
context-aware responses and learn from interactions over time. However,
requiring external databases like Neo4j or Qdrant creates deployment complexity
that conflicts with Caxton's "easy to get started" philosophy.

### Memory System Requirements

Based on analysis of modern agent platforms and the Memento MCP tools used in
Claude Code:

1. **Entity-Relationship Storage**: Agents need to store entities and their
   relationships
2. **Semantic Search**: Vector-based similarity search for retrieving relevant
   context
3. **Temporal Tracking**: Understanding how knowledge evolves over time
4. **Graph Traversal**: Finding connections between related concepts
5. **Fast Retrieval**: Sub-second query performance for real-time agent
   responses
6. **Zero Configuration**: Should work immediately without external dependencies

### Current Challenges

- **External dependencies**: Neo4j, Qdrant, etc. require separate installation
  and configuration
- **Deployment complexity**: Production deployments need database expertise
- **Development friction**: Local development requires database setup
- **Resource overhead**: Small projects don't need enterprise-scale database
  systems

## Decision

We will implement a **hybrid memory architecture** with an embedded default
backend and optional external backends for scale.

### Default Implementation: SQLite + Candle

The embedded memory backend combines SQLite for structured storage with local
embedding models for semantic search. This provides a zero-configuration
solution that works immediately without external dependencies.

**Storage Model**:

- **Entities**: Named knowledge items with typed observations and temporal
  validity
- **Relations**: Typed connections between entities with strength and confidence
  scores
- **Embeddings**: Vector representations for semantic similarity search
- **Temporal tracking**: Version history and validity periods for knowledge
  evolution

### Local Embedding Model

**Model Selection**: All-MiniLM-L6-v2 provides the optimal balance of size
(~23MB), performance (~1000 embeddings/second on CPU), and quality for semantic
similarity tasks. The 384-dimensional embeddings enable effective semantic
search while maintaining reasonable resource requirements.

### Agent Memory Integration

Configuration agents can be enabled with memory capabilities through their YAML
configuration. Memory-enabled agents automatically:

1. **Search** memory for relevant context before responding
2. **Incorporate** past solutions and patterns into their responses
3. **Store** new knowledge from successful interactions
4. **Maintain** conversation history and learned patterns

**Memory Scopes**:

- **Agent-only**: Private memory per agent instance
- **Workspace**: Shared memory within a workspace or project
- **Global**: System-wide shared knowledge base

**Automatic Knowledge Management**: The runtime extracts valuable information
from agent interactions and stores it for future reference, enabling agents to
learn and improve over time without manual knowledge engineering.

### Pluggable Backend Architecture

The memory system supports multiple backend implementations through
configuration:

**Embedded Backend (Default)**:

- SQLite + local embedding model
- Zero external dependencies
- Scales to 100K+ entities for single-node deployments

**External Backends (Optional)**:

- **Neo4j**: Graph database for complex relationship queries
- **Qdrant**: Dedicated vector database for high-performance semantic search
- **Custom backends**: Pluggable architecture allows additional implementations

**Backend Selection**: Configured through `caxton.yaml` with backend-specific
settings. System automatically initializes the appropriate backend
implementation at startup.

### Performance Characteristics

**Embedded Performance**:

- Semantic search: 10-50ms for 100K entities
- Graph traversal: 5-20ms for typical queries
- Memory usage: ~200MB baseline for embedding model
- Storage: ~2.5KB per entity (including embedding)

**Scaling Guidance**:

- **Embedded backend**: Scales to 100K+ entities, 1M relations
- **External backends**: Optional migration path for larger deployments
  requiring scale beyond embedded capacity

### Migration and Data Portability

The memory system provides standard JSON export/import functionality for
migrating between backend implementations or creating backups. This ensures data
portability and enables smooth transitions from embedded to external backends as
systems scale.

## Consequences

### Positive

- **Zero configuration startup**: Memory works immediately out of the box
- **No external dependencies**: Eliminates database setup complexity
- **Local development friendly**: Everything runs in a single process
- **Production scalable**: Can upgrade to external backends when needed
- **Data portability**: Standard export/import formats for migration
- **Cost effective**: No additional infrastructure costs for small deployments

### Negative

- **Embedded limitations**: Single-node only, scaling limit at 100K+ entities
- **Model size**: 23MB embedding model increases binary size
- **CPU overhead**: Local embedding generation uses CPU cycles
- **Memory usage**: ~200MB baseline memory usage for embedding model

### Risk Mitigation

- **Clear scaling guidance**: Documentation specifies when to migrate to
  external backends
- **Performance monitoring**: Built-in metrics to track memory system
  performance
- **Graceful degradation**: System continues working even if memory operations
  fail
- **Migration tooling**: Automated export/import for moving between backends

## Implementation Plan

### Phase 1: Embedded Backend (1.0)

1. SQLite schema and basic operations
2. Local embedding model integration (All-MiniLM-L6-v2)
3. Semantic search implementation
4. Agent runtime integration
5. Basic cleanup and maintenance

### Phase 2: Enhanced Features (1.0)

1. Graph traversal algorithms
2. Export/import functionality
3. Performance optimization
4. Memory usage monitoring
5. Configuration management

### Future Enhancement: External Backends

1. Neo4j backend implementation
2. Qdrant backend implementation
3. Migration tooling and documentation
4. Advanced analytics and reporting

## Alignment with Existing ADRs

- **ADR-0028 (Configuration-Driven Agents)**: Provides memory capabilities for
  config agents
- **ADR-0029 (Capability-Based Messaging)**: Agents can store
  conversation context and
  patterns
- **ADR-0004 (Minimal Core Philosophy)**: Embedded approach minimizes external
  dependencies
- **ADR-0001 (Observability First)**: Memory operations are fully instrumented
  and observable

## Related Decisions

- ADR-0028: Configuration-Driven Agent Architecture (defines agents that use
  memory)
- ADR-0029: Capability-Based Messaging (conversations can be stored in
  memory)
- ADR-0013: State Management Architecture (memory is part of agent state
  management)

## References

- [Memento MCP Tools]
  (https://github.com/modelcontextprotocol/servers/tree/main/src/memory)
  \- inspiration for entity-relation model
- [SQLite Vector Search](https://alexgarcia.xyz/sqlite-vec/) - sqlite-vec
  extension
- [Sentence Transformers](https://www.sbert.net/) - embedding model approach
- [All-MiniLM-L6-v2]
  (https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
  \- specific model selection
- Expert analysis on embedded vs external memory systems
