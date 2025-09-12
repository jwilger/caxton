---
title: "Memory System Concepts"
description: "Core concepts for understanding Caxton's embedded memory system
  with entity-relationship storage and semantic search"
date: 2025-01-13
categories: [Concepts, Memory, Architecture]
layout: concept
level: foundational
---

> **🚧 Implementation Status**
>
> The embedded memory system represents a core component from ADR-30. This
> documentation serves as the conceptual foundation for understanding how agents
> store, retrieve, and share knowledge through Caxton's memory architecture.
>
> **Target**: Zero-configuration embedded memory with semantic search
> **Status**: Foundational concepts defined, implementation in progress

## What is the Memory System?

Caxton's memory system is a **knowledge persistence layer** that enables agents
to store, retrieve, and share information across conversations and sessions.
It combines **entity-relationship storage** with **semantic search** to create
an intelligent memory that understands context and meaning, not just exact
matches.

Think of it as giving agents a **persistent brain** that remembers successful
solutions, learns from past interactions, and can find relevant information
when needed, similar to how humans recall related experiences to solve new
problems.

## Core Concepts

### Knowledge as Entities and Relationships

The memory system models knowledge using two fundamental building blocks:

**Entities**: Discrete pieces of information with rich context

- A solution pattern that worked well
- A user preference discovered during interaction
- A domain concept learned from documentation
- A performance optimization technique

**Relationships**: Typed connections between entities

- "Solution A *implements* Pattern B"
- "Concept X *relates_to* Problem Y"
- "User *prefers* Approach Z"
- "Error *caused_by* Configuration Issue"

This graph-like structure enables agents to follow conceptual connections and
build contextual understanding from interconnected knowledge.

### Semantic Search vs Keyword Matching

Traditional search requires exact keyword matches. Semantic search understands
**meaning and context**:

**Keyword Search**: `"database connection error"` only finds exact phrase
**Semantic Search**: Finds related concepts like "SQL timeout", "connection
pool exhaustion", "database authentication failure"

This enables agents to find relevant solutions even when problems are described
differently than previously encountered.

### Memory Scopes for Knowledge Sharing

Memory operates at three different scopes to balance privacy with collaboration:

**Agent-Only Memory**: Private knowledge for specialized roles

- Personal learning patterns and preferences
- Agent-specific optimizations and techniques
- Confidential or sensitive information handling

**Workspace Memory**: Shared knowledge within project teams

- Team conventions and best practices
- Project-specific domain knowledge
- Collaborative problem-solving patterns

**Global Memory**: Organization-wide knowledge base

- Company standards and policies
- Cross-project solutions and patterns
- Organizational learning and expertise

## Why This Architecture Matters

### For Developers

- **Context-Aware APIs**: Memory queries return relevant patterns and solutions
- **Learning Systems**: Agents improve performance through accumulated experience
- **Debugging Support**: Historical problem-solution pairs aid troubleshooting

### For Operators

- **Zero Configuration**: Embedded backend works immediately without external setup
- **Scaling Flexibility**: Optional migration to external backends (Neo4j, Qdrant)
- **Performance Monitoring**: Built-in metrics track memory usage and query performance

### For End Users

- **Consistent Experience**: Agents remember preferences and successful approaches
- **Improved Accuracy**: Relevant historical context leads to better responses
- **Knowledge Continuity**: Learning persists across conversations and sessions

### For Stakeholders

- **Organizational Learning**: Knowledge accumulates and compounds over time
- **Cost Efficiency**: Embedded backend eliminates external infrastructure costs
- **Growth Path**: Clear upgrade options as memory requirements scale

## Architecture Philosophy

### Zero Dependencies by Default

The embedded backend (SQLite + Candle) provides a complete memory system
without external dependencies:

- **SQLite**: Reliable, embedded database for structured storage
- **Candle**: Rust-native ML inference for local embeddings
- **All-MiniLM-L6-v2**: Lightweight embedding model (~23MB)

This eliminates deployment complexity while maintaining professional capabilities.

### Progressive Scalability

The architecture supports growth through pluggable backends:

- **Start Simple**: Embedded backend for <100K entities
- **Scale Up**: External backends (Neo4j, Qdrant) for larger deployments
- **Data Portability**: Standard JSON export/import for migrations

### Context-Driven Design

Memory integrates deeply with Caxton's context management (ADR-31):

- **Intelligent Retrieval**: Semantic queries based on agent capabilities
- **Performance Targets**: <50ms context preparation from memory
- **Multi-Provider Support**: Context formatting for different LLM providers

## Key Performance Characteristics

### Embedded Backend (Default)

- **Search Latency**: 10-50ms for 100K entities
- **Memory Usage**: ~200MB baseline (embedding model)
- **Storage Efficiency**: ~2.5KB per entity including embeddings
- **Scaling Limit**: 100K+ entities, 1M+ relationships

### External Backends (Optional)

- **Neo4j**: Optimized for complex relationship queries and graph analytics
- **Qdrant**: High-performance vector search with distributed architecture
- **Migration**: Seamless upgrade path when scaling requirements exceed
  embedded limits

## Memory in Action

### Automatic Knowledge Extraction

The system automatically identifies and stores valuable patterns:

```text
User Request: "Help me debug this API timeout"

Agent Response: Successfully diagnoses connection pool exhaustion

Memory Storage:
Entity: "API timeout debugging pattern"
Observations: ["Check connection pool size", "Monitor active connections",
               "Review timeout configuration"]
Relationship: "API timeout" relates_to "connection pool exhaustion"
```

### Contextual Information Retrieval

When facing similar problems, agents can find relevant solutions:

```text
New Request: "My API is responding slowly"

Memory Query: semantic_search("API performance problems")

Retrieved Context:
- "API timeout debugging pattern" (similarity: 0.85)
- "Database connection optimization" (similarity: 0.72)
- "Load balancer configuration" (similarity: 0.68)

Enhanced Response: Agent provides targeted debugging steps based on
historical successful solutions
```

### Cross-Agent Learning

Workspace memory enables knowledge sharing between agents:

```text
Data Agent: Learns optimal CSV parsing configuration
DevOps Agent: Accesses data parsing patterns for monitoring scripts
Support Agent: Uses both patterns for customer troubleshooting
```

## Integration Patterns

### Configuration Agent Memory

YAML configuration enables memory for any agent:

```yaml
memory:
  enabled: true
  scope: workspace
  settings:
    auto_store: true          # Automatically save successful patterns
    search_threshold: 0.7     # Minimum semantic similarity
    max_results: 10          # Context window management
```

### Context Management Integration

Memory serves as a primary context source alongside:

- **Conversation History**: Current dialogue context
- **Capability Registry**: Agent skill and tool information
- **MCP Tool Data**: Tool-specific patterns and preferences

This multi-source approach ensures agents have comprehensive context for
accurate, relevant responses.

### Performance Optimization

The memory system includes several performance features:

- **Semantic Caching**: >70% hit rate for frequent queries
- **Graph Indexing**: Optimized relationship traversal
- **Temporal Filtering**: Prioritize recent knowledge over historical patterns
- **Token Budget Management**: Adaptive result sizing based on available
  context window

## Security and Privacy Model

### Data Protection

**Embedded Backend**:

- All data stored locally with file-level security
- No external API calls for embedding generation
- Memory-only processing for sensitive queries

**External Backends**:

- TLS encryption for all network communications
- Backend-specific authentication and authorization
- Configurable data residency controls

### Privacy Considerations

**Content Filtering**:

- Automatic PII detection and redaction options
- Configurable privacy policies per memory scope
- Custom filtering rules for sensitive information

**Access Control**:

- Memory scope isolation (agent-only vs workspace vs global)
- Audit logging for knowledge access and modifications
- Export/import controls for data movement

## Learning Path

### Getting Started (Basic)

1. **Concept Understanding**: Entities, relationships, semantic search
2. **Memory Scopes**: Agent-only vs workspace vs global knowledge sharing
3. **Basic Configuration**: Enable memory in configuration agents

### Intermediate

1. **Backend Selection**: Embedded vs external backend trade-offs
2. **Context Integration**: How memory enhances agent responses
3. **Performance Tuning**: Query optimization and caching strategies

### Advanced

1. **Custom Backends**: Implementing additional backend providers
2. **Migration Strategies**: Moving between embedded and external systems
3. **Analytics Patterns**: Leveraging memory data for insights

### Expert

1. **Architecture Customization**: Extending memory model for specific domains
2. **Integration Development**: Building memory-aware applications
3. **Performance Engineering**: Optimizing for high-scale deployments

## Related Concepts

- **[Architecture Concepts](/docs/concepts/architecture/)**: System design
  and hybrid agent runtime
- **[Messaging Concepts](/docs/concepts/messaging/)**: FIPA communication
  and conversation management
- **[API Concepts](/docs/concepts/api/)**: Memory integration patterns
  and performance specifications

## Implementation Guides

- [Embedded Backend Setup](/docs/concepts/memory-system/embedded-backend.md)
- [Usage Patterns](/docs/concepts/memory-system/usage-patterns.md)
- [Migration Strategies](/docs/concepts/memory-system/migration.md)
- [Model Management](/docs/concepts/memory-system/model-management.md)
