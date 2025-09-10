# ADR-0030: Embedded Memory System with Pluggable Backends

## Status

Accepted

## Date

2025-09-09

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

**Core Architecture:**

```rust
pub trait MemoryBackend: Send + Sync {
    async fn store_entity(&self, entity: Entity) -> Result<EntityId>;
    async fn create_relation(&self, relation: Relation) -> Result<RelationId>;
    async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<Entity>>;
    async fn graph_traversal(&self, start: EntityId, depth: u32) -> Result<Graph>;
    async fn cleanup_stale(&self, max_age: Duration) -> Result<u32>;
}

#[derive(Clone)]
pub struct EmbeddedMemoryBackend {
    db: Arc<SqlitePool>,
    embeddings: Arc<RwLock<EmbeddingIndex>>,
    encoder: Arc<SentenceTransformer>,
}
```

**Storage Schema:**

```sql
-- Core entities table
CREATE TABLE entities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    observations TEXT NOT NULL,  -- JSON array of strings
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    valid_from INTEGER,
    valid_to INTEGER,
    version INTEGER DEFAULT 1,
    changed_by TEXT
);

-- Entity relationships
CREATE TABLE relations (
    id TEXT PRIMARY KEY,
    from_entity TEXT NOT NULL REFERENCES entities(id),
    to_entity TEXT NOT NULL REFERENCES entities(id),
    relation_type TEXT NOT NULL,
    strength REAL DEFAULT 1.0,
    confidence REAL DEFAULT 1.0,
    metadata TEXT,  -- JSON object
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    valid_from INTEGER,
    valid_to INTEGER,
    UNIQUE(from_entity, to_entity, relation_type)
);

-- Vector embeddings (using sqlite-vec extension)
CREATE VIRTUAL TABLE entity_embeddings USING vec0(
    entity_id TEXT PRIMARY KEY,
    embedding FLOAT[384]  -- All-MiniLM-L6-v2 dimensions
);

-- Indexes for performance
CREATE INDEX idx_entities_type ON entities(entity_type);
CREATE INDEX idx_entities_created ON entities(created_at);
CREATE INDEX idx_relations_from ON relations(from_entity);
CREATE INDEX idx_relations_to ON relations(to_entity);
CREATE INDEX idx_relations_type ON relations(relation_type);
```

### Local Embedding Model

#### Model Selection: All-MiniLM-L6-v2

- Size: ~23MB ONNX model
- Dimensions: 384
- Performance: ~1000 embeddings/second on CPU
- Quality: Excellent for semantic similarity

```rust
pub struct SentenceTransformer {
    session: OrtSession,
    tokenizer: Tokenizer,
    max_length: usize,
}

impl SentenceTransformer {
    pub fn new() -> Result<Self> {
        let model_bytes = include_bytes!("../models/all-MiniLM-L6-v2.onnx");
        let session = OrtSession::from_bytes(model_bytes)?;
        let tokenizer = Tokenizer::from_pretrained("sentence-transformers/all-MiniLM-L6-v2")?;

        Ok(Self {
            session,
            tokenizer,
            max_length: 512,
        })
    }

    pub async fn encode(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Tokenize input texts
        let inputs = self.tokenizer.encode_batch(texts, true)?;

        // Run inference
        let outputs = self.session.run(inputs).await?;

        // Extract embeddings with pooling
        Ok(self.mean_pooling(outputs))
    }
}
```

### Agent Memory Integration

Config agents access memory through their runtime context:

```yaml
# Agent configuration
---
name: CustomerSupportAgent
memory_enabled: true
memory_scope: workspace  # 'global', 'workspace', or 'agent-only'
memory_cleanup: "30d"    # Cleanup entities older than 30 days
system_prompt: |
  You are a customer support agent. Before responding:

  1. Search your memory for similar customer issues
  2. Use past solutions to inform your response
  3. Store new solutions for future reference

  Your memory contains customer interactions, solutions, and patterns.
---
```

**Runtime Integration:**

```rust
impl ConfigAgentRuntime {
    pub async fn execute_with_memory(
        &self,
        agent: &ConfigAgent,
        message: Message
    ) -> Result<Response> {
        // 1. Search memory for relevant context
        let context = self.memory
            .semantic_search(&message.content, 5)
            .await?;

        // 2. Format prompt with memory context
        let prompt = self.format_prompt_with_memory(agent, &message, &context)?;

        // 3. Get LLM response
        let response = self.llm_client.complete(prompt).await?;

        // 4. Extract and store new knowledge
        if let Some(knowledge) = self.extract_knowledge(&response)? {
            self.memory.store_entity(knowledge).await?;
        }

        Ok(response)
    }

    async fn extract_knowledge(&self, response: &str) -> Result<Option<Entity>> {
        // Use LLM to extract structured knowledge from responses
        let extraction_prompt = format!(
            "Extract key facts and solutions from this response as structured data:\n\n{}",
            response
        );

        let extracted = self.llm_client.complete(extraction_prompt).await?;
        self.parse_knowledge_extraction(extracted)
    }
}
```

### Pluggable Backend Architecture

**Configuration-Based Backend Selection:**

```yaml
# caxton.yaml - Default (embedded)
memory:
  backend: embedded
  settings:
    storage_path: "./caxton-memory.db"
    embedding_model: "all-MiniLM-L6-v2"
    cleanup_interval: "24h"
    max_entities: 100000

---
# caxton.yaml - External Neo4j
memory:
  backend: neo4j
  settings:
    uri: "bolt://localhost:7687"
    username: "neo4j"
    password: "${NEO4J_PASSWORD}"
    database: "caxton"

---
# caxton.yaml - External Qdrant
memory:
  backend: qdrant
  settings:
    url: "http://localhost:6333"
    collection: "caxton-memory"
    vector_size: 384
```

**Backend Implementation:**

```rust
pub enum MemoryBackend {
    Embedded(EmbeddedMemoryBackend),
    Neo4j(Neo4jBackend),
    Qdrant(QdrantBackend),
}

impl MemoryBackend {
    pub async fn from_config(config: MemoryConfig) -> Result<Self> {
        match config.backend.as_str() {
            "embedded" => {
                let backend = EmbeddedMemoryBackend::new(config.settings).await?;
                Ok(MemoryBackend::Embedded(backend))
            }
            "neo4j" => {
                let backend = Neo4jBackend::new(config.settings).await?;
                Ok(MemoryBackend::Neo4j(backend))
            }
            "qdrant" => {
                let backend = QdrantBackend::new(config.settings).await?;
                Ok(MemoryBackend::Qdrant(backend))
            }
            _ => Err(MemoryError::UnsupportedBackend(config.backend)),
        }
    }
}
```

### Performance Characteristics

**Embedded SQLite + Candle:**

- **Semantic search**: 10-50ms for 100K entities
- **Graph traversal**: 5-20ms for typical 3-hop queries
- **Storage overhead**: ~1KB per entity + 1.5KB for embedding
- **Memory usage**: ~200MB for model + active index
- **Startup time**: ~100ms to load embedding model

**Scaling Thresholds:**

- **Embedded works well up to**: 100K entities, 1M relations
- **Consider external backends beyond**: 1M entities, 10M relations

### Migration and Data Portability

**Export/Import Support:**

```rust
pub struct MemoryExporter {
    backend: Arc<dyn MemoryBackend>,
}

impl MemoryExporter {
    pub async fn export_to_json(&self, path: &Path) -> Result<()> {
        let entities = self.backend.get_all_entities().await?;
        let relations = self.backend.get_all_relations().await?;

        let export = MemoryExport {
            version: "1.0",
            created_at: Utc::now(),
            entities,
            relations,
        };

        serde_json::to_writer_pretty(File::create(path)?, &export)?;
        Ok(())
    }

    pub async fn import_from_json(&self, path: &Path) -> Result<ImportResult> {
        let export: MemoryExport = serde_json::from_reader(File::open(path)?)?;

        // Validate compatibility
        self.validate_import(&export)?;

        // Import entities and relations
        for entity in export.entities {
            self.backend.store_entity(entity).await?;
        }

        for relation in export.relations {
            self.backend.create_relation(relation).await?;
        }

        Ok(ImportResult::success(export.entities.len(), export.relations.len()))
    }
}
```

## Consequences

### Positive

- **Zero configuration startup**: Memory works immediately out of the box
- **No external dependencies**: Eliminates database setup complexity
- **Local development friendly**: Everything runs in a single process
- **Production scalable**: Can upgrade to external backends when needed
- **Data portability**: Standard export/import formats for migration
- **Cost effective**: No additional infrastructure costs for small deployments

### Negative

- **Embedded limitations**: Single-node only, limited to ~100K entities
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

### Phase 3: External Backends (Post-1.0)

1. Neo4j backend implementation
2. Qdrant backend implementation
3. Migration tooling and documentation
4. Advanced analytics and reporting

## Alignment with Existing ADRs

- **ADR-0028 (Configuration-Driven Agents)**: Provides memory capabilities for
  config agents
- **ADR-0029 (FIPA-ACL Messaging)**: Agents can store conversation context and
  patterns
- **ADR-0004 (Minimal Core Philosophy)**: Embedded approach minimizes external
  dependencies
- **ADR-0001 (Observability First)**: Memory operations are fully instrumented
  and observable

## Related Decisions

- ADR-0028: Configuration-Driven Agent Architecture (defines agents that use
  memory)
- ADR-0029: FIPA-ACL Lightweight Messaging (conversations can be stored in
  memory)
- ADR-0013: State Management Architecture (memory is part of agent state
  management)

## References

- [Memento MCP Tools](https://github.com/modelcontextprotocol/servers/tree/main/src/memory)
  \- inspiration for entity-relation model
- [SQLite Vector Search](https://alexgarcia.xyz/sqlite-vec/) - sqlite-vec
  extension
- [Sentence Transformers](https://www.sbert.net/) - embedding model approach
- [All-MiniLM-L6-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
  \- specific model selection
- Expert analysis on embedded vs external memory systems
