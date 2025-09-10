---
title: "Caxton Memory System Overview"
description: "Comprehensive guide to Caxton's embedded memory system with
  SQLite+Candle backend and semantic search capabilities"
date: 2025-09-10
categories: [Architecture, Memory]
---

> **🚧 Implementation Status**
>
> The embedded memory system represents a core component from ADR-30. This
> documentation serves as the technical specification for the SQLite + Candle
> backend implementation currently under development.
>
> **Target**: Zero-configuration embedded memory with semantic search
> **Status**: Architecture defined, SQLite + embedding model integration in
> progress

## System Architecture

Caxton's memory system provides persistent context-aware storage for agents
through a hybrid architecture that combines zero-configuration embedded
backends with optional external systems for scale. The design prioritizes
ease of deployment while maintaining a clear upgrade path for production
systems.

## Core Capabilities

### Entity-Relationship Storage

The memory system stores knowledge as entities and their relationships,
enabling agents to:

- **Store structured knowledge** as named entities with typed observations
- **Create semantic relationships** between concepts with strength and
  confidence scores
- **Track temporal changes** through versioned knowledge and validity periods
- **Maintain conversation context** across multiple interactions

### Semantic Search

Vector-based similarity search enables agents to:

- **Find relevant context** using natural language queries
- **Discover related concepts** through semantic similarity
- **Retrieve contextual information** for real-time agent responses
- **Surface patterns** from historical interactions

### Graph Traversal

Relationship-based navigation allows agents to:

- **Follow knowledge connections** between related entities
- **Understand concept hierarchies** and dependencies
- **Build contextual understanding** from interconnected information
- **Trace reasoning paths** through knowledge networks

### Context Management Integration

The memory system serves as a primary context source for configuration
agents (ADR-0031), providing:

- **Intelligent context retrieval** through semantic search with <50ms
  performance targets
- **Conversation-aware memory queries** that integrate with FIPA messaging
  threads
- **Template-based context gathering** using patterns like
  `{{capability}} tasks similar to {{request}}`
- **Graph-enhanced context expansion** through relationship traversal
- **Temporal context filtering** to prioritize recent and relevant knowledge
- **Multi-LLM provider context formatting** for different model requirements

## Architecture Components

### Memory Backend Interface

```rust
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    async fn create_entities(&self, entities: Vec<Entity>)
        -> Result<Vec<EntityId>, MemoryError>;
    async fn create_relations(&self, relations: Vec<Relation>)
        -> Result<Vec<RelationId>, MemoryError>;
    async fn semantic_search(&self, query: &str, limit: usize)
        -> Result<Vec<SearchResult>, MemoryError>;
    async fn open_nodes(&self, names: &[String])
        -> Result<Vec<Entity>, MemoryError>;
    async fn get_graph(&self) -> Result<Graph, MemoryError>;
}
```

### Entity Model

Entities represent discrete knowledge items with rich metadata:

```yaml
entity:
  name: "String identifier for the entity"
  entityType: "Categorization (e.g., 'concept', 'pattern', 'solution')"
  observations: ["Array of content strings describing the entity"]
  metadata:
    confidence: 0.95  # Confidence level (0.0-1.0)
    strength: 0.85    # Importance weighting (0.0-1.0)
    validFrom: timestamp
    validTo: timestamp
    version: 1
```

### Relation Model

Relations define typed connections between entities:

```yaml
relation:
  from: "source entity name"
  to: "target entity name"
  relationType: "implements|relates_to|depends_on|contains|etc"
  strength: 0.9      # Connection strength (0.0-1.0)
  confidence: 0.85   # Confidence in relation (0.0-1.0)
  metadata:
    context: "Additional relation context"
    source: "Where this relation was discovered"
```

## Backend Implementations

### Embedded Backend (Default)

**Technology Stack**:

- **SQLite**: Structured storage for entities, relations, and metadata
- **sqlite-vec**: Vector search extension for semantic similarity
- **Candle**: Rust-native ML inference for local embeddings
- **All-MiniLM-L6-v2**: 384-dimensional embedding model (~23MB)

**Performance Characteristics**:

- **Semantic search**: 10-50ms for 100K entities
- **Graph traversal**: 5-20ms for typical queries
- **Memory usage**: ~200MB baseline (embedding model)
- **Storage**: ~2.5KB per entity (including embedding)
- **Throughput**: ~1000 embeddings/second on CPU

**Scaling Limits**:

- **Embedded backend scales**: Up to 100K+ entities, 1M relations
- **Migration path**: Optional external backends for larger deployments
- **Single-node**: Embedded deployment only

### Pluggable External Backends (Optional)

**Neo4j Backend**:

- Graph database optimized for complex relationship queries
- Cypher query language for advanced graph operations
- Horizontal scaling and clustering support
- Recommended for relationship-heavy workloads requiring >100K+ entities

**Qdrant Backend**:

- Dedicated vector database for high-performance semantic search
- Distributed architecture with sharding and replication
- Advanced filtering and hybrid search capabilities
- Recommended for large-scale semantic search workloads requiring
  >100K+ entities

## Agent Integration

### Memory-Enabled Agents

Configuration agents can be enhanced with memory capabilities through YAML
configuration:

```yaml
# agent-config.yaml
memory:
  enabled: true
  scope: "workspace"  # agent-only, workspace, global
  backend: "embedded" # embedded, neo4j, qdrant
  settings:
    auto_store: true
    search_threshold: 0.7
    max_results: 10
```

### Memory Scopes

**Agent-Only Memory**:

- Private knowledge base per agent instance
- Isolated learning and context storage
- Suitable for specialized agent roles

**Workspace Memory**:

- Shared knowledge within a project or workspace
- Cross-agent learning and collaboration
- Team-wide context and patterns

**Global Memory**:

- System-wide shared knowledge base
- Organization-wide best practices and solutions
- Cross-project learning and standardization

### Automatic Knowledge Management

The runtime automatically extracts and stores valuable information:

1. **Successful Solutions**: Patterns that led to successful outcomes
2. **Error Recovery**: How problems were diagnosed and resolved
3. **User Preferences**: Learned patterns from user interactions
4. **Domain Knowledge**: Concepts and relationships discovered during operation
5. **Performance Patterns**: What approaches work well in different contexts

## Context Management Integration

The memory system plays a critical role in Caxton's context management
architecture (ADR-0031), serving as one of four primary context sources for
configuration agents. This integration enables intelligent, automatic context
preparation without requiring manual prompt engineering.

### Context Router Integration

The **Context Router** queries the memory system using semantic search to
find relevant historical knowledge:

- **Performance Target**: <50ms semantic search latency for context preparation
- **Query Templates**: Uses patterns like
  `{{capability}} tasks similar to {{request}}` for contextual relevance
- **Result Filtering**: Applies minimum similarity thresholds (typically
  0.6+) to ensure quality
- **Token Budget Management**: Adapts result count based on available
  context window

### Memory-Driven Context Flow

1. **Agent Request Analysis**: FIPA message parsed to extract capability
   and task requirements
2. **Memory Query Generation**: Context Router creates semantic search
   queries based on request context
3. **Semantic Retrieval**: Memory system returns relevant entities and
   relationships within performance targets
4. **Graph Expansion**: Related concepts discovered through relationship
   traversal to enrich context
5. **Temporal Filtering**: Recent knowledge prioritized over historical
   patterns for relevance
6. **Context Formatting**: Memory results formatted for specific LLM
   provider requirements

### Context Query Patterns

The memory system supports several context query patterns:

**Capability-Driven Queries**:

```yaml
memory_search:
  query_template: "{{capability}} expertise and best practices"
  max_results: 8
  min_similarity: 0.7
```

**Task-Specific Context**:

```yaml
memory_search:
  query_template: "{{request_type}} similar to {{user_request}}"
  max_results: 12
  min_similarity: 0.6
  include_relationships: true
```

**Conversation-Aware Search**:

```yaml
memory_search:
  query_template: "{{conversation_context}} related patterns and solutions"
  max_results: 10
  temporal_boost: "recent"
```

### Multi-Source Context Integration

The memory system works alongside other context sources in the Context Router:

- **Conversation History**: Memory search enhanced with current
  conversation context
- **Capability Registry**: Memory queries filtered by agent capability relevance
- **MCP Tool Data**: Tool-specific memory patterns and user preferences
- **Performance Optimization**: Memory cache maintains >70% hit rate for
  frequent queries

### Performance Characteristics for Context

**Context Query Performance**:

- Semantic search: 10-50ms (target <50ms for context preparation)
- Graph traversal: 5-20ms for context expansion
- Result formatting: <5ms for LLM provider adaptation
- Cache hit rate: >70% for frequent context patterns

**Context Quality Metrics**:

- Signal-to-noise ratio optimization through hierarchical filtering
- Context relevance scoring based on semantic similarity
- Temporal decay modeling for knowledge freshness
- Multi-provider context format validation

### Context-Aware Storage

The memory system learns from successful context usage:

- **Context Effectiveness Tracking**: Records which memory results led to
  successful agent responses
- **Pattern Recognition**: Identifies frequently useful context patterns
  for specific capabilities
- **User Preference Learning**: Stores patterns from user interactions to
  improve future context
- **Cross-Agent Knowledge Sharing**: Enables workspace and global memory
  scopes for collaborative learning

## Configuration and Deployment

### Backend Selection

Memory backends are configured in `caxton.yaml`:

```yaml
# caxton.yaml
memory:
  backend: "embedded"  # embedded (default), neo4j, qdrant

  embedded:
    database_path: "./data/memory.db"
    model_cache_path: "./data/models"
    embedding_model: "all-MiniLM-L6-v2"

  neo4j:
    uri: "bolt://localhost:7687"
    username: "caxton"
    password: "${NEO4J_PASSWORD}"
    database: "caxton_memory"

  qdrant:
    url: "http://localhost:6333"
    collection: "caxton_embeddings"
    api_key: "${QDRANT_API_KEY}"
```

### Zero-Configuration Startup

The embedded backend requires no external setup:

1. **First Run**: System downloads embedding model (~23MB)
2. **Initialization**: SQLite database and vector indexes created automatically
3. **Ready**: Agents can immediately use memory capabilities

### Data Portability

Standard JSON export/import enables:

- **Backup and restore** of knowledge bases
- **Migration** between backend implementations
- **Environment promotion** (dev → staging → production)
- **Multi-tenant** knowledge base management

```bash
# Export memory data
caxton memory export --format json --output backup.json

# Import to different backend
caxton memory import --format json --input backup.json --backend neo4j
```

## Performance and Scaling

### Embedded Backend Performance

**Query Performance**:

- Entity retrieval: 1-5ms
- Semantic search: 10-50ms (100K entities)
- Graph traversal: 5-20ms (typical depth)
- Bulk operations: 1000+ entities/second

**Resource Usage**:

- Base memory: ~200MB (embedding model)
- Per-entity storage: ~2.5KB
- CPU overhead: <5% for background operations
- Disk I/O: Minimal with SQLite WAL mode

### Scaling Guidance

**When to Use Embedded Backend**:

- Development and testing environments
- Single-node deployments
- <100K entities, <1M relations
- Cost-sensitive deployments
- Simple deployment requirements

**When to Migrate to External Backends**:

- Multi-node distributed deployments
- >1M entities or complex graph queries
- High-concurrency semantic search requirements
- Advanced analytics and reporting needs
- Enterprise compliance requirements

## Monitoring and Observability

### Built-in Metrics

The memory system provides comprehensive observability:

```rust
// Performance metrics
caxton_memory_search_duration_seconds
caxton_memory_entity_count
caxton_memory_relation_count
caxton_memory_backend_errors_total

// Usage patterns
caxton_memory_queries_per_second
caxton_memory_cache_hit_ratio
caxton_memory_storage_usage_bytes
```

### Health Checks

Automated health monitoring includes:

- **Backend connectivity** and response times
- **Data consistency** checks and validation
- **Resource utilization** monitoring
- **Performance degradation** detection

### Troubleshooting

Common issues and solutions:

**Slow semantic search**: Check entity count and consider external backend
migration
**High memory usage**: Monitor embedding cache size and consider cleanup
policies
**Data corruption**: Use built-in consistency checks and repair tools
**Backend failures**: Implement graceful degradation and fallback strategies

## Security and Privacy

### Data Protection

**Embedded Backend Security**:

- SQLite database file permissions and encryption
- Local embedding model with no external API calls
- Memory-only processing for sensitive queries

**External Backend Security**:

- TLS encryption for all network communications
- Backend-specific authentication and authorization
- API key and credential management
- Network security and firewall configuration

### Privacy Considerations

**Data Residency**:

- Embedded: All data stored locally
- External: Data location depends on backend deployment
- Export/import: Full control over data movement

**Content Filtering**:

- Configurable content filtering for sensitive information
- Automatic PII detection and redaction options
- Custom privacy policies per memory scope

## Related Documentation

- [ADR-0030: Embedded Memory System](/docs/adr/0030-embedded-memory-system.md)
- [ADR-0031: Context Management Architecture](
  /docs/adr/0031-context-management-architecture.md)
- [Embedded Backend Guide](/docs/memory-system/embedded-backend.md)
- [Usage Patterns](/docs/memory-system/usage-patterns.md)
- [Migration Guide](/docs/memory-system/migration.md)
- [Configuration-Driven Agents](
  /docs/adr/0028-configuration-driven-agent-architecture.md)
- [FIPA-ACL Lightweight Messaging](
  /docs/adr/0029-fipa-acl-lightweight-messaging.md)

## Next Steps

1. **Quick Start**: Follow the embedded backend guide for immediate setup
2. **Integration**: Learn agent memory integration patterns
3. **Production**: Review scaling guidance and external backend options
4. **Migration**: Plan upgrade paths as requirements grow
