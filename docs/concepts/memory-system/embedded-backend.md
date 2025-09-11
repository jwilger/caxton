---
title: "Embedded Memory Backend Concepts"
description: "Understanding the technical architecture and design principles
  behind Caxton's zero-dependency SQLite+Candle memory implementation"
date: 2025-01-13
categories: [Concepts, Memory, Implementation]
layout: concept
level: intermediate
---

## What is the Embedded Backend?

The **embedded memory backend** is Caxton's default, zero-dependency solution
for agent knowledge storage that combines **SQLite** for structured data with
**Candle** for local machine learning inference. This architecture eliminates
the need for external databases or cloud services while providing professional-
grade semantic search capabilities.

Think of it as a **self-contained brain** that agents can use immediately
without any setup complexity—similar to how a desktop application includes
everything it needs to run without requiring separate database installation.

## Core Architectural Concepts

### Zero-Dependency Philosophy

**The Problem**: Traditional agent platforms require complex infrastructure

- External vector databases (Pinecone, Weaviate, Qdrant)
- Separate embedding API services (OpenAI, Cohere)
- Database administration and maintenance
- Network dependencies and failure modes

**The Solution**: Embedded architecture with local capabilities

- **SQLite**: Battle-tested embedded database (used in browsers, phones)
- **Candle**: Pure Rust ML inference (no Python dependencies)
- **Local Models**: ~23MB embedding model runs on CPU
- **Immediate Startup**: No external services to configure or wait for

This enables **5-minute deployment** vs **hours of infrastructure setup**.

### Hybrid Storage Architecture

The embedded backend combines two complementary storage approaches:

**Structured Storage (SQLite)**:

- Entities, relationships, metadata
- ACID transactions and data integrity
- Complex queries and graph traversal
- JSON support for flexible schemas

**Vector Storage (sqlite-vec extension)**:

- 384-dimensional semantic embeddings
- Cosine similarity search
- k-nearest-neighbor queries
- Efficient vector indexing

This hybrid approach provides both **exact matching** and **semantic understanding**
in a single, cohesive system.

## Technical Design Principles

### Performance Through Simplicity

Rather than complex distributed systems, the embedded backend achieves
performance through:

**Local Data Access**: No network latency for memory operations

- Semantic search: 10-50ms vs 100-500ms for external APIs
- Graph traversal: 5-20ms vs 50-200ms for external databases
- Zero network failures or timeout issues

**Optimized Data Structures**:

- SQLite Write-Ahead Logging (WAL) for concurrent access
- Vector indexes co-located with entity data
- Efficient batch processing for embeddings
- Connection pooling for multi-agent workloads

**Resource Efficiency**:

- ~200MB baseline memory usage (embedding model)
- ~2.5KB per entity including embedding vectors
- Scales to 100K+ entities before requiring external backends

### Context-Optimized Design

The embedded backend is specifically optimized for **context management**
integration (ADR-0031), prioritizing:

**Low Latency Context Retrieval**:

- <50ms target for context preparation
- Semantic search optimized for agent queries
- Graph traversal limited to 2-3 degrees for speed
- Temporal boosting for recent knowledge

**Token Budget Management**:

- Adaptive result sizing based on LLM context windows
- Provider-specific formatting (OpenAI, Anthropic, local)
- Content estimation and truncation strategies
- Smart result ranking by relevance and recency

### Scalability Through Pluggability

The embedded backend provides a **growth path** without architectural rewrites:

**Start Simple**: Single-node embedded for development and small deployments
**Scale Up**: External backends (Neo4j, Qdrant) for larger requirements
**Migrate Seamlessly**: Standard export/import maintains data continuity

This enables **linear scaling** as requirements grow, from prototype to
enterprise deployment.

## Core Components Deep Dive

### SQLite Integration Patterns

**Database Schema Design**:

```sql
-- Entities: Core knowledge items
entities(id, name, entity_type, observations, confidence, strength,
         version, created_at, updated_at, valid_from, valid_to, metadata)

-- Relations: Typed connections between entities
relations(from_entity_id, to_entity_id, relation_type, strength,
          confidence, temporal_constraints, metadata)

-- Embeddings: Vector representations for semantic search
embeddings(entity_id, embedding_blob, model_version, created_at)
```

**Why This Design**:

- **Normalized structure** prevents data duplication and inconsistency
- **Foreign key constraints** maintain referential integrity
- **Flexible metadata** (JSON) allows schema evolution without migrations
- **Temporal columns** support time-based queries and knowledge decay
- **Version tracking** enables entity evolution and change management

### Candle ML Integration

**Local Inference Architecture**:

- **No Python Dependencies**: Pure Rust implementation eliminates runtime issues
- **ONNX Model Loading**: Standard format supports multiple model types
- **CPU Optimization**: Efficient inference on commodity hardware
- **Batch Processing**: Groups text encoding for better throughput
- **Memory Management**: Controlled resource usage for embedding generation

**Model Selection Rationale (All-MiniLM-L6-v2)**:

- **Size Efficiency**: 23MB vs 500MB+ for larger models
- **Quality Balance**: 384 dimensions provide good semantic understanding
- **Speed Optimization**: ~1000 embeddings/second on modern CPUs
- **Compatibility**: Works well across different text types and domains
- **Proven Performance**: Widely used in semantic search applications

### Vector Search Implementation

**sqlite-vec Extension Benefits**:

- **Native Integration**: Vector search directly within SQLite queries
- **Performance**: Avoids data movement between separate systems
- **Consistency**: ACID properties apply to vector operations
- **Simplicity**: No separate vector database administration

**Search Algorithm**:

1. **Query Embedding**: Text query converted to 384-dimensional vector
2. **Similarity Calculation**: Cosine similarity against stored embeddings
3. **Result Ranking**: Combined semantic score with metadata factors
4. **Filtering**: Minimum similarity thresholds and result limits
5. **Context Enhancement**: Temporal boosting and relationship expansion

## Context Management Integration

### Why Context Management Matters

Modern agents need **relevant background information** to provide accurate,
helpful responses. The embedded backend serves as a primary **context source**
alongside conversation history and tool capabilities.

**Context Query Flow**:

1. **Agent Request**: User asks for help with specific capability
2. **Template Expansion**: Query template filled with request context
3. **Semantic Search**: Memory system finds related patterns/solutions
4. **Graph Expansion**: Relationships add connected knowledge
5. **Result Formatting**: Context adapted for specific LLM provider
6. **Token Management**: Results sized to fit available context window

### Performance Optimization for Context

**Query Pattern Optimization**:

```sql
-- Context queries prioritize recent, high-confidence entities
SELECT entities.*, similarity_score
FROM entities
JOIN semantic_search_results ON entities.id = semantic_search_results.entity_id
WHERE similarity_score >= 0.7
  AND (created_at > recent_threshold OR confidence > 0.8)
ORDER BY
  CASE WHEN created_at > recent_threshold
       THEN similarity_score * 1.2
       ELSE similarity_score END DESC
LIMIT adaptive_limit_based_on_token_budget
```

**Caching Strategy**:

- **Query Result Caching**: LRU cache for frequent context patterns
- **Embedding Caching**: Avoid re-encoding common queries
- **Graph Traversal Caching**: Pre-computed relationship paths
- **>70% hit rate** for typical agent workloads

### Multi-LLM Provider Support

Different LLM providers have different context requirements:

**OpenAI Integration**:

- Token estimation: ~4 characters per token
- Context formatting: Structured with clear sections
- Maximum context: 8K-128K tokens depending on model

**Anthropic Integration**:

- Token estimation: ~4.5 characters per token (slightly different)
- Context formatting: Conversational style preferred
- Maximum context: 32K-200K tokens depending on model

**Local Model Integration**:

- Token estimation: Provider-specific tokenization
- Context formatting: Simplified structure for smaller models
- Maximum context: 2K-8K tokens typically

## Performance Characteristics

### Benchmark Results (Typical Hardware)

**Entity Operations**:

- Create single entity: 5-15ms (including embedding generation)
- Bulk create 1000 entities: 2-5 seconds (batch processing)
- Retrieve entity by ID: <1ms (indexed lookup)
- Update entity: 5-15ms (re-embedding if observations changed)

**Search Operations**:

- Semantic search (1K entities): 10-20ms
- Semantic search (10K entities): 20-35ms
- Semantic search (100K entities): 35-50ms
- Graph traversal (depth 2): 5-15ms
- Graph traversal (depth 3): 10-25ms

**Resource Usage**:

- Base memory: ~200MB (All-MiniLM-L6-v2 model)
- Per-entity memory: ~2.5KB (including 384-dim embedding)
- Database file growth: Linear with entity count
- CPU usage: <5% background, 20-40% during batch operations

### Scaling Characteristics

**Recommended Usage Patterns**:

- **Development**: Perfect for any size during development
- **Single-node production**: <10K entities, <100K relationships
- **Multi-agent systems**: <100K entities, <1M relationships
- **Enterprise single-node**: 100K+ entities with SSD storage

**Performance Degradation Points**:

- **50K entities**: Search latency increases to 30-40ms
- **100K entities**: Consider SSD storage for optimal performance
- **500K entities**: Approach recommended scaling limit
- **1M+ entities**: External backend migration recommended

## Troubleshooting Concepts

### Common Performance Issues

**Slow Semantic Search**:

- **Cause**: Large entity count without proper indexing
- **Solution**: Ensure vector indexes are built, consider result limits
- **Prevention**: Monitor entity growth and plan external migration

**High Memory Usage**:

- **Cause**: Large embedding cache or many concurrent connections
- **Solution**: Reduce cache size, limit connection pool
- **Prevention**: Set resource limits in configuration

**Database Locking**:

- **Cause**: High concurrent write load without WAL mode
- **Solution**: Enable WAL mode, increase busy timeout
- **Prevention**: Use batch operations for bulk writes

### Monitoring and Observability

**Key Metrics to Track**:

- `caxton_memory_entity_count`: Total stored entities
- `caxton_memory_search_duration_seconds`: Query performance
- `caxton_memory_cache_hit_ratio`: Cache effectiveness
- `caxton_memory_database_size_bytes`: Storage growth

**Health Check Indicators**:

- Database connectivity and response time
- Embedding model loading and inference capability
- Vector search functionality and accuracy
- Resource utilization within expected bounds

## Migration and Growth Path

### When to Consider External Backends

**Neo4j Migration Indicators**:

- Complex relationship queries requiring Cypher
- Need for graph analytics and advanced algorithms
- Multi-node deployment requirements
- >1M entities with heavy relationship traversal

**Qdrant Migration Indicators**:

- High-volume semantic search requirements
- Need for distributed vector search
- Advanced filtering and hybrid search needs
- >500K entities with search-heavy workloads

### Migration Process

**Data Export**:

1. **Full Export**: JSON format with all entities, relationships, metadata
2. **Incremental Export**: Changes since last export for ongoing sync
3. **Validation**: Verify data integrity and completeness

**Backend Setup**:

1. **External System**: Install and configure Neo4j or Qdrant
2. **Schema Creation**: Set up equivalent structure in new backend
3. **Performance Tuning**: Optimize for expected workload patterns

**Data Import**:

1. **Batch Import**: Bulk load exported data into new backend
2. **Relationship Reconstruction**: Rebuild connections and indexes
3. **Validation**: Verify search accuracy and performance

**Cutover**:

1. **Parallel Testing**: Run both backends during transition period
2. **Performance Validation**: Ensure new backend meets requirements
3. **Configuration Update**: Switch agents to new backend
4. **Monitoring**: Watch for issues during initial production usage

## Security and Privacy Concepts

### Data Protection Model

**Local Data Sovereignty**:

- All data remains on local systems (no cloud dependencies)
- File-level encryption supported through filesystem
- Memory-only processing for sensitive queries
- No external API calls for embedding generation

**Access Control**:

- File system permissions control database access
- SQLite supports user-defined functions for custom filtering
- Memory scope isolation (agent/workspace/global)
- Audit logging for knowledge access patterns

### Privacy Considerations

**Content Filtering Options**:

- Automatic PII detection and redaction
- Custom content filters per memory scope
- Configurable sensitivity levels
- Hash-based storage for ultra-sensitive content

**Data Minimization**:

- Configurable entity retention periods
- Automatic cleanup of expired knowledge
- Selective export excluding sensitive entities
- Memory scope boundaries prevent unauthorized access

## Learning Path

### For Developers

1. **Architecture Understanding**: SQLite + vector search integration
2. **Performance Optimization**: Indexing, caching, batch operations
3. **Context Integration**: How memory serves agent context needs
4. **Migration Planning**: When and how to scale to external backends

### For Operators

1. **Deployment Simplicity**: Zero-configuration startup process
2. **Performance Monitoring**: Key metrics and health indicators
3. **Scaling Decisions**: When to migrate to external backends
4. **Backup and Recovery**: Data export/import procedures

### For Security Teams

1. **Data Sovereignty**: Local storage and processing guarantees
2. **Access Controls**: File system and application-level security
3. **Privacy Controls**: Content filtering and data minimization
4. **Audit Capabilities**: Knowledge access and modification tracking

## Related Concepts

- **[Memory System Overview](/docs/concepts/memory-system/overview.md)**:
  Foundational concepts and use cases
- **[Usage Patterns](/docs/concepts/memory-system/usage-patterns.md)**:
  Common integration and deployment patterns
- **[Migration Strategies](/docs/concepts/memory-system/migration.md)**:
  Planning growth to external backends
- **[Architecture Concepts](/docs/concepts/architecture/)**:
  Overall system design and hybrid runtime
- **[API Concepts](/docs/concepts/api/)**:
  Memory integration patterns and specifications
