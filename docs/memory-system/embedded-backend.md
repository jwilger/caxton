---
title: "Embedded Memory Backend Technical Guide"
description: "Deep technical implementation guide for Caxton's SQLite+Candle embedded memory backend"
date: 2025-09-10
categories: [Architecture, Implementation, Memory]
---

## Architecture Overview

The embedded memory backend combines SQLite for structured data storage with
Candle for local ML inference, providing zero-dependency semantic search
capabilities. This architecture eliminates external database requirements while
maintaining production-quality performance for single-node deployments.

## Technology Stack

### SQLite + sqlite-vec

**SQLite Database**:

- ACID-compliant storage for entities and relations
- WAL (Write-Ahead Logging) mode for concurrent access
- Foreign key constraints for data integrity
- JSON support for flexible metadata storage

**sqlite-vec Extension**:

- Vector similarity search within SQLite
- Efficient k-nearest-neighbor queries
- Cosine similarity calculations
- Index optimization for large vector collections

### Candle ML Framework

**Local Inference Engine**:

- Pure Rust implementation with no Python dependencies
- ONNX model loading and execution
- CPU and GPU acceleration support
- Memory-efficient batch processing

**All-MiniLM-L6-v2 Model**:

- 384-dimensional sentence embeddings
- ~23MB ONNX model size
- ~1000 embeddings/second on modern CPUs
- Optimized for semantic similarity tasks

## Database Schema

### Core Tables

```sql
-- Entities table
CREATE TABLE entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    entity_type TEXT NOT NULL,
    observations TEXT NOT NULL, -- JSON array
    confidence REAL DEFAULT 1.0,
    strength REAL DEFAULT 1.0,
    version INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    valid_from INTEGER,
    valid_to INTEGER,
    metadata TEXT -- JSON object
);

-- Relations table
CREATE TABLE relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_entity_id INTEGER NOT NULL,
    to_entity_id INTEGER NOT NULL,
    relation_type TEXT NOT NULL,
    strength REAL DEFAULT 1.0,
    confidence REAL DEFAULT 1.0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    valid_from INTEGER,
    valid_to INTEGER,
    metadata TEXT, -- JSON object
    FOREIGN KEY (from_entity_id) REFERENCES entities(id) ON DELETE CASCADE,
    FOREIGN KEY (to_entity_id) REFERENCES entities(id) ON DELETE CASCADE,
    UNIQUE(from_entity_id, to_entity_id, relation_type)
);

-- Embeddings table for vector search
CREATE TABLE embeddings (
    entity_id INTEGER PRIMARY KEY,
    embedding BLOB NOT NULL, -- 384 float32 values
    model_version TEXT NOT NULL DEFAULT 'all-MiniLM-L6-v2',
    created_at INTEGER NOT NULL,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

-- Vector search virtual table using sqlite-vec
CREATE VIRTUAL TABLE vec_embeddings USING vec0(
    entity_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]
);
```

### Indexes and Constraints

```sql
-- Performance indexes
CREATE INDEX idx_entities_name ON entities(name);
CREATE INDEX idx_entities_type ON entities(entity_type);
CREATE INDEX idx_entities_created_at ON entities(created_at);
CREATE INDEX idx_relations_from_to ON relations(from_entity_id, to_entity_id);
CREATE INDEX idx_relations_type ON relations(relation_type);

-- Temporal indexes for time-based queries
CREATE INDEX idx_entities_validity ON entities(valid_from, valid_to);
CREATE INDEX idx_relations_validity ON relations(valid_from, valid_to);

-- Composite indexes for common query patterns
CREATE INDEX idx_entities_type_confidence ON entities(entity_type, confidence DESC);
CREATE INDEX idx_relations_type_strength ON relations(relation_type, strength DESC);
```

## Implementation Architecture

### Memory Backend Implementation

```rust
pub struct EmbeddedMemoryBackend {
    db_pool: Arc<SqlitePool>,
    embedding_engine: Arc<EmbeddingEngine>,
    config: EmbeddedBackendConfig,
    metrics: Arc<MemoryMetrics>,
}

#[derive(Debug, Clone)]
pub struct EmbeddedBackendConfig {
    pub database_path: PathBuf,
    pub model_cache_path: PathBuf,
    pub embedding_model: String,
    pub batch_size: usize,
    pub max_connections: u32,
    pub wal_mode: bool,
    pub foreign_keys: bool,
}

impl EmbeddedMemoryBackend {
    pub async fn new(config: EmbeddedBackendConfig) -> Result<Self, BackendError> {
        // Initialize SQLite connection pool
        let db_pool = Self::create_database_pool(&config).await?;

        // Run migrations
        Self::run_migrations(&db_pool).await?;

        // Initialize embedding engine
        let embedding_engine = Arc::new(
            EmbeddingEngine::new(&config.model_cache_path, &config.embedding_model).await?
        );

        // Initialize metrics
        let metrics = Arc::new(MemoryMetrics::new());

        Ok(Self {
            db_pool,
            embedding_engine,
            config,
            metrics,
        })
    }

    async fn create_database_pool(config: &EmbeddedBackendConfig) -> Result<SqlitePool, BackendError> {
        let mut options = SqliteConnectOptions::from_str(&format!("sqlite:{}", config.database_path.display()))?;

        // Configure SQLite for performance and safety
        options = options
            .journal_mode(if config.wal_mode { SqliteJournalMode::Wal } else { SqliteJournalMode::Delete })
            .foreign_keys(config.foreign_keys)
            .busy_timeout(Duration::from_secs(30))
            .pragma("temp_store", "memory")
            .pragma("cache_size", "-64000") // 64MB cache
            .pragma("synchronous", "normal"); // Balance safety/performance

        let pool = SqlitePool::connect_with(options).await?;
        pool.set_max_connections(config.max_connections);

        Ok(pool)
    }
}
```

### Embedding Engine

```rust
pub struct EmbeddingEngine {
    model: Arc<SentenceTransformerModel>,
    tokenizer: Arc<Tokenizer>,
    device: Device,
    batch_size: usize,
}

impl EmbeddingEngine {
    pub async fn new(cache_path: &Path, model_name: &str) -> Result<Self, EmbeddingError> {
        let device = Device::Cpu; // TODO: GPU detection

        // Download model if not cached
        let model_path = Self::ensure_model_cached(cache_path, model_name).await?;

        // Load ONNX model with Candle
        let model = Arc::new(SentenceTransformerModel::load(&model_path, &device)?);

        // Load tokenizer
        let tokenizer_path = model_path.join("tokenizer.json");
        let tokenizer = Arc::new(Tokenizer::from_file(&tokenizer_path)?);

        Ok(Self {
            model,
            tokenizer,
            device,
            batch_size: 32,
        })
    }

    #[instrument(skip(self, texts))]
    pub async fn encode_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let start_time = Instant::now();

        // Process in batches to manage memory
        let mut all_embeddings = Vec::new();

        for batch in texts.chunks(self.batch_size) {
            let batch_embeddings = self.encode_batch_internal(batch).await?;
            all_embeddings.extend(batch_embeddings);
        }

        let duration = start_time.elapsed();
        tracing::info!(
            text_count = texts.len(),
            duration_ms = duration.as_millis(),
            throughput = (texts.len() as f64) / duration.as_secs_f64(),
            "Batch embedding completed"
        );

        Ok(all_embeddings)
    }

    async fn encode_batch_internal(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // Tokenize inputs
        let encodings = self.tokenizer.encode_batch(texts, true)?;

        // Convert to tensors
        let input_ids = self.encodings_to_tensor(&encodings, "input_ids")?;
        let attention_mask = self.encodings_to_tensor(&encodings, "attention_mask")?;

        // Run inference
        let embeddings = self.model.forward(&input_ids, &attention_mask)?;

        // Convert to Vec<Vec<f32>> and normalize
        let embeddings = self.tensor_to_embeddings(embeddings)?;

        Ok(embeddings)
    }

    fn normalize_embedding(&self, embedding: &mut [f32]) {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in embedding.iter_mut() {
                *x /= norm;
            }
        }
    }
}
```

## CRUD Operations

### Entity Management

```rust
impl EmbeddedMemoryBackend {
    #[instrument(skip(self, entities))]
    pub async fn create_entities(&self, entities: Vec<CreateEntityRequest>) -> Result<Vec<EntityId>, BackendError> {
        let mut transaction = self.db_pool.begin().await?;
        let mut created_ids = Vec::new();

        // Generate embeddings for all entities
        let texts: Vec<String> = entities.iter()
            .map(|e| self.entity_to_embedding_text(e))
            .collect();

        let embeddings = self.embedding_engine.encode_batch(&texts).await?;

        // Insert entities and embeddings
        for (entity_req, embedding) in entities.iter().zip(embeddings.iter()) {
            // Insert entity
            let entity_id = sqlx::query!(
                r#"
                INSERT INTO entities (name, entity_type, observations, confidence, strength,
                                    version, created_at, updated_at, valid_from, valid_to, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING id
                "#,
                entity_req.name,
                entity_req.entity_type,
                serde_json::to_string(&entity_req.observations)?,
                entity_req.confidence.unwrap_or(1.0),
                entity_req.strength.unwrap_or(1.0),
                entity_req.version.unwrap_or(1),
                entity_req.created_at.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()),
                entity_req.updated_at.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()),
                entity_req.valid_from,
                entity_req.valid_to,
                entity_req.metadata.as_ref().map(|m| serde_json::to_string(m)).transpose()?
            )
            .fetch_one(&mut *transaction)
            .await?;

            let id = EntityId(entity_id.id);

            // Insert embedding
            let embedding_bytes = self.embedding_to_bytes(embedding)?;
            sqlx::query!(
                r#"
                INSERT INTO embeddings (entity_id, embedding, model_version, created_at)
                VALUES (?, ?, ?, ?)
                "#,
                id.0,
                embedding_bytes,
                "all-MiniLM-L6-v2",
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
            )
            .execute(&mut *transaction)
            .await?;

            // Insert into vector search table
            sqlx::query!(
                "INSERT INTO vec_embeddings (entity_id, embedding) VALUES (?, ?)",
                id.0,
                embedding_bytes
            )
            .execute(&mut *transaction)
            .await?;

            created_ids.push(id);
        }

        transaction.commit().await?;

        // Update metrics
        self.metrics.entities_created.inc_by(created_ids.len() as u64);
        self.metrics.entity_count.inc_by(created_ids.len() as i64);

        Ok(created_ids)
    }

    fn entity_to_embedding_text(&self, entity: &CreateEntityRequest) -> String {
        // Combine entity name, type, and observations for embedding
        let observations_text = entity.observations.join(" ");
        format!("{} {} {}", entity.name, entity.entity_type, observations_text)
    }

    fn embedding_to_bytes(&self, embedding: &[f32]) -> Result<Vec<u8>, BackendError> {
        // Convert f32 vector to bytes for SQLite storage
        let mut bytes = Vec::with_capacity(embedding.len() * 4);
        for &value in embedding {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }
}
```

### Semantic Search Implementation

```rust
impl EmbeddedMemoryBackend {
    #[instrument(skip(self, query))]
    pub async fn semantic_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>, BackendError> {
        let start_time = Instant::now();

        // Generate embedding for query
        let query_embedding = self.embedding_engine.encode_batch(&[query.to_string()]).await?;
        let query_vec = &query_embedding[0];

        // Perform vector similarity search
        let query_bytes = self.embedding_to_bytes(query_vec)?;

        let results = sqlx::query_as!(
            SearchResultRow,
            r#"
            SELECT
                e.id,
                e.name,
                e.entity_type,
                e.observations,
                e.confidence,
                e.strength,
                vec.distance
            FROM entities e
            JOIN (
                SELECT entity_id, distance
                FROM vec_embeddings
                WHERE embedding MATCH ?
                ORDER BY distance
                LIMIT ?
            ) vec ON e.id = vec.entity_id
            WHERE vec.distance >= ?
            ORDER BY vec.distance DESC
            "#,
            query_bytes,
            options.limit.unwrap_or(10),
            options.min_similarity.unwrap_or(0.6)
        )
        .fetch_all(&self.db_pool)
        .await?;

        let search_results: Vec<SearchResult> = results.into_iter()
            .map(|row| SearchResult {
                entity: Entity {
                    id: EntityId(row.id),
                    name: row.name,
                    entity_type: row.entity_type,
                    observations: serde_json::from_str(&row.observations).unwrap_or_default(),
                    confidence: row.confidence,
                    strength: row.strength,
                },
                similarity: row.distance,
            })
            .collect();

        let duration = start_time.elapsed();

        // Update metrics
        self.metrics.search_queries.inc();
        self.metrics.search_duration.observe(duration.as_secs_f64());
        self.metrics.search_results.observe(search_results.len() as f64);

        tracing::info!(
            query_length = query.len(),
            result_count = search_results.len(),
            duration_ms = duration.as_millis(),
            min_similarity = options.min_similarity.unwrap_or(0.6),
            "Semantic search completed"
        );

        Ok(search_results)
    }
}
```

### Graph Traversal

```rust
impl EmbeddedMemoryBackend {
    #[instrument(skip(self, start_entities))]
    pub async fn traverse_graph(&self, start_entities: &[EntityId], options: TraversalOptions) -> Result<GraphTraversalResult, BackendError> {
        let max_depth = options.max_depth.unwrap_or(3);
        let mut visited = HashSet::new();
        let mut current_level = start_entities.to_vec();
        let mut all_entities = Vec::new();
        let mut all_relations = Vec::new();

        for depth in 0..max_depth {
            if current_level.is_empty() {
                break;
            }

            // Get entities at current level
            let entity_placeholders = format!("?{}", ", ?".repeat(current_level.len().saturating_sub(1)));
            let entity_params: Vec<i64> = current_level.iter().map(|id| id.0).collect();

            let entities = sqlx::query_as!(
                EntityRow,
                &format!(
                    r#"
                    SELECT id, name, entity_type, observations, confidence, strength,
                           version, created_at, updated_at, valid_from, valid_to, metadata
                    FROM entities
                    WHERE id IN ({})
                    "#,
                    entity_placeholders
                ),
            )
            .bind_all(entity_params.clone())
            .fetch_all(&self.db_pool)
            .await?;

            // Add to results
            all_entities.extend(entities.into_iter().map(Entity::from));

            // Find outgoing relations from current level
            let relations = sqlx::query_as!(
                RelationRow,
                &format!(
                    r#"
                    SELECT r.id, r.from_entity_id, r.to_entity_id, r.relation_type,
                           r.strength, r.confidence, r.created_at, r.updated_at,
                           r.valid_from, r.valid_to, r.metadata,
                           from_e.name as from_name, to_e.name as to_name
                    FROM relations r
                    JOIN entities from_e ON r.from_entity_id = from_e.id
                    JOIN entities to_e ON r.to_entity_id = to_e.id
                    WHERE r.from_entity_id IN ({})
                    AND r.strength >= ?
                    "#,
                    entity_placeholders
                ),
                options.min_strength.unwrap_or(0.1)
            )
            .bind_all(entity_params)
            .fetch_all(&self.db_pool)
            .await?;

            // Prepare next level and track relations
            let mut next_level = Vec::new();
            for relation in relations {
                let to_entity_id = EntityId(relation.to_entity_id);
                if !visited.contains(&to_entity_id) {
                    next_level.push(to_entity_id);
                    visited.insert(to_entity_id);
                }
                all_relations.push(Relation::from(relation));
            }

            // Mark current level as visited
            for entity_id in &current_level {
                visited.insert(*entity_id);
            }

            current_level = next_level;
        }

        Ok(GraphTraversalResult {
            entities: all_entities,
            relations: all_relations,
            max_depth_reached: depth == max_depth,
        })
    }
}
```

## Context Management Integration

The embedded memory backend serves as a critical context source for
configuration agents in ADR-0031's context management architecture. This
integration requires specific performance characteristics and query patterns
optimized for real-time context preparation.

### Context Query Performance Targets

The embedded backend must meet strict performance requirements for context management:

- **Context Search Latency**: <50ms (P95) for semantic search queries
- **Graph Traversal**: <20ms for context expansion through relationships
- **Result Formatting**: <5ms for LLM provider-specific context formatting
- **Cache Hit Rate**: >70% for frequently requested context patterns
- **Token Budget Compliance**: Adaptive result sizing based on LLM context windows

### Context-Optimized Query Patterns

#### Semantic Context Search

```rust
#[instrument(skip(self, query_template))]
pub async fn context_search(&self, query_template: &str, context_vars: &HashMap<String, String>, options: ContextSearchOptions) -> Result<Vec<ContextResult>, BackendError> {
    let start_time = Instant::now();

    // Expand query template with context variables
    let expanded_query = self.expand_query_template(query_template, context_vars)?;

    // Generate embedding for contextualized query
    let query_embedding = self.embedding_engine.encode_batch(&[expanded_query]).await?;
    let query_vec = &query_embedding[0];

    // Perform optimized context search with temporal weighting
    let results = sqlx::query_as!(
        ContextResultRow,
        r#"
        SELECT
            e.id,
            e.name,
            e.entity_type,
            e.observations,
            e.confidence,
            e.strength,
            e.created_at,
            e.updated_at,
            vec.distance,
            CASE
                WHEN e.created_at > ? THEN vec.distance * 1.2  -- Boost recent entries
                ELSE vec.distance
            END as context_score
        FROM entities e
        JOIN (
            SELECT entity_id, distance
            FROM vec_embeddings
            WHERE embedding MATCH ?
            ORDER BY distance
            LIMIT ?
        ) vec ON e.id = vec.entity_id
        WHERE vec.distance >= ?
        ORDER BY context_score DESC
        LIMIT ?
        "#,
        options.temporal_boost_threshold.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - 86400000), // 24h ago
        self.embedding_to_bytes(query_vec)?,
        options.max_candidates.unwrap_or(50),
        options.min_similarity.unwrap_or(0.6),
        options.max_results.unwrap_or(10)
    )
    .fetch_all(&self.db_pool)
    .await?;

    let duration = start_time.elapsed();

    // Update context-specific metrics
    self.metrics.context_queries.inc();
    self.metrics.context_query_duration.observe(duration.as_secs_f64());

    // Ensure we meet the <50ms target
    if duration.as_millis() > 50 {
        tracing::warn!(
            duration_ms = duration.as_millis(),
            query = %expanded_query,
            "Context query exceeded 50ms target"
        );
    }

    Ok(results.into_iter().map(ContextResult::from).collect())
}

fn expand_query_template(&self, template: &str, vars: &HashMap<String, String>) -> Result<String, BackendError> {
    let mut query = template.to_string();

    // Replace template variables like {{capability}}, {{request_type}}, etc.
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        query = query.replace(&placeholder, value);
    }

    Ok(query)
}
```

#### Graph-Enhanced Context Retrieval

```rust
#[instrument(skip(self, entity_ids))]
pub async fn expand_context_graph(&self, entity_ids: &[EntityId], options: GraphContextOptions) -> Result<ContextGraph, BackendError> {
    let max_depth = options.max_depth.unwrap_or(2); // Shallow for context speed
    let min_strength = options.min_strength.unwrap_or(0.7); // High threshold for context quality

    let mut context_entities = HashSet::new();
    let mut context_relations = Vec::new();
    let mut current_level = entity_ids.to_vec();

    for depth in 0..max_depth {
        if current_level.is_empty() {
            break;
        }

        // Find high-strength relationships for context expansion
        let entity_placeholders = format!("?{}", ", ?".repeat(current_level.len().saturating_sub(1)));
        let entity_params: Vec<i64> = current_level.iter().map(|id| id.0).collect();

        let relations = sqlx::query_as!(
            ContextRelationRow,
            &format!(
                r#"
                SELECT r.from_entity_id, r.to_entity_id, r.relation_type, r.strength,
                       from_e.name as from_name, to_e.name as to_name
                FROM relations r
                JOIN entities from_e ON r.from_entity_id = from_e.id
                JOIN entities to_e ON r.to_entity_id = to_e.id
                WHERE r.from_entity_id IN ({})
                AND r.strength >= ?
                ORDER BY r.strength DESC
                LIMIT ?
                "#,
                entity_placeholders
            ),
            min_strength,
            options.max_relations.unwrap_or(20)
        )
        .bind_all(entity_params)
        .fetch_all(&self.db_pool)
        .await?;

        // Collect next level entities and relations
        let mut next_level = Vec::new();
        for relation in relations {
            let to_entity_id = EntityId(relation.to_entity_id);
            if !context_entities.contains(&to_entity_id) {
                next_level.push(to_entity_id);
                context_entities.insert(to_entity_id);
            }
            context_relations.push(ContextRelation::from(relation));
        }

        current_level = next_level;
    }

    Ok(ContextGraph {
        entities: context_entities.into_iter().collect(),
        relations: context_relations,
        max_depth_reached: depth == max_depth,
    })
}
```

### Context-Specific Performance Optimizations

#### Context Query Caching

```rust
pub struct ContextCache {
    query_cache: Arc<Mutex<LruCache<String, Vec<ContextResult>>>>,
    graph_cache: Arc<Mutex<LruCache<Vec<EntityId>, ContextGraph>>>,
    cache_ttl: Duration,
}

impl ContextCache {
    pub fn new(max_queries: usize, max_graphs: usize, ttl: Duration) -> Self {
        Self {
            query_cache: Arc::new(Mutex::new(LruCache::new(max_queries))),
            graph_cache: Arc::new(Mutex::new(LruCache::new(max_graphs))),
            cache_ttl: ttl,
        }
    }

    pub async fn get_cached_context(&self, query_key: &str) -> Option<Vec<ContextResult>> {
        let cache = self.query_cache.lock().await;
        cache.get(query_key).cloned()
    }

    pub async fn cache_context(&self, query_key: String, results: Vec<ContextResult>) {
        let mut cache = self.query_cache.lock().await;
        cache.put(query_key, results);
    }
}
```

#### Token Budget Management

```rust
impl EmbeddedMemoryBackend {
    pub async fn context_search_with_budget(&self, query: &str, token_budget: usize, provider: LLMProvider) -> Result<FormattedContext, BackendError> {
        let base_results = self.context_search(query, &HashMap::new(), Default::default()).await?;

        // Estimate token usage based on provider characteristics
        let estimated_tokens = self.estimate_context_tokens(&base_results, provider);

        // Adjust result count to fit within budget
        let adjusted_results = if estimated_tokens > token_budget {
            let reduction_ratio = token_budget as f32 / estimated_tokens as f32;
            let target_count = (base_results.len() as f32 * reduction_ratio) as usize;
            base_results.into_iter().take(target_count).collect()
        } else {
            base_results
        };

        // Format for specific LLM provider
        Ok(self.format_context_for_provider(adjusted_results, provider).await?)
    }

    fn estimate_context_tokens(&self, results: &[ContextResult], provider: LLMProvider) -> usize {
        let base_tokens: usize = results.iter()
            .map(|r| r.observations.join(" ").len() / 4) // Rough token estimation
            .sum();

        match provider {
            LLMProvider::OpenAI => base_tokens,
            LLMProvider::Anthropic => (base_tokens as f32 * 1.1) as usize, // Slightly different tokenization
            LLMProvider::Local => base_tokens,
        }
    }
}
```

### Context Quality Metrics

The embedded backend tracks context-specific metrics to ensure quality:

```rust
pub struct ContextMetrics {
    pub context_queries: IntCounter,
    pub context_query_duration: Histogram,
    pub context_cache_hits: IntCounter,
    pub context_cache_misses: IntCounter,
    pub context_results_count: Histogram,
    pub context_token_usage: Histogram,
    pub context_quality_score: Histogram,
}
```

### MCP Tool Context Integration

The backend supports MCP tool context specifications:

```yaml
# Example context specification in MCP tool
context_requirements:
  conversation_depth: 5
  memory_search:
    query_template: "{{capability}} expertise for {{request_type}}"
    max_results: 10
    min_similarity: 0.7
    include_relationships: true
  temporal_boost: "recent"  # Boost entities from last 24h
  provider_optimization:
    openai:
      token_budget: 4000
      format: "detailed"
    anthropic:
      token_budget: 8000
      format: "structured"
```

This integration enables automatic, intelligent context preparation that adapts
to agent needs without manual configuration.

## Performance Optimizations

### Connection Pooling

```rust
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub idle_timeout: Duration,
    pub connection_timeout: Duration,
    pub test_before_acquire: bool,
}

impl EmbeddedMemoryBackend {
    async fn optimize_database_settings(&self) -> Result<(), BackendError> {
        // Set optimal SQLite pragmas
        sqlx::query("PRAGMA journal_mode = WAL").execute(&self.db_pool).await?;
        sqlx::query("PRAGMA synchronous = normal").execute(&self.db_pool).await?;
        sqlx::query("PRAGMA cache_size = -64000").execute(&self.db_pool).await?; // 64MB
        sqlx::query("PRAGMA temp_store = memory").execute(&self.db_pool).await?;
        sqlx::query("PRAGMA mmap_size = 268435456").execute(&self.db_pool).await?; // 256MB mmap

        Ok(())
    }
}
```

### Batch Processing

```rust
impl EmbeddedMemoryBackend {
    const BATCH_SIZE: usize = 1000;

    pub async fn bulk_insert_entities(&self, entities: Vec<CreateEntityRequest>) -> Result<Vec<EntityId>, BackendError> {
        let mut all_ids = Vec::new();

        // Process in batches to avoid memory issues
        for batch in entities.chunks(Self::BATCH_SIZE) {
            let batch_ids = self.create_entities(batch.to_vec()).await?;
            all_ids.extend(batch_ids);
        }

        Ok(all_ids)
    }
}
```

### Embedding Caching

```rust
pub struct EmbeddingCache {
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    max_size: usize,
}

impl EmbeddingCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(max_size))),
            max_size,
        }
    }

    pub async fn get_or_compute<F, Fut>(&self, key: &str, compute_fn: F) -> Result<Vec<f32>, EmbeddingError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<f32>, EmbeddingError>>,
    {
        // Check cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(embedding) = cache.get(key) {
                return Ok(embedding.clone());
            }
        }

        // Compute and cache
        let embedding = compute_fn().await?;
        {
            let mut cache = self.cache.lock().await;
            cache.put(key.to_string(), embedding.clone());
        }

        Ok(embedding)
    }
}
```

## Monitoring and Metrics

### Performance Metrics

```rust
pub struct MemoryMetrics {
    pub entity_count: IntGauge,
    pub relation_count: IntGauge,
    pub entities_created: IntCounter,
    pub relations_created: IntCounter,
    pub search_queries: IntCounter,
    pub search_duration: Histogram,
    pub search_results: Histogram,
    pub embedding_duration: Histogram,
    pub database_size: IntGauge,
}

impl MemoryMetrics {
    pub fn new() -> Self {
        Self {
            entity_count: register_int_gauge!("caxton_memory_entity_count", "Total number of entities").unwrap(),
            relation_count: register_int_gauge!("caxton_memory_relation_count", "Total number of relations").unwrap(),
            entities_created: register_int_counter!("caxton_memory_entities_created_total", "Total entities created").unwrap(),
            relations_created: register_int_counter!("caxton_memory_relations_created_total", "Total relations created").unwrap(),
            search_queries: register_int_counter!("caxton_memory_search_queries_total", "Total search queries").unwrap(),
            search_duration: register_histogram!("caxton_memory_search_duration_seconds", "Search query duration").unwrap(),
            search_results: register_histogram!("caxton_memory_search_results", "Number of search results returned").unwrap(),
            embedding_duration: register_histogram!("caxton_memory_embedding_duration_seconds", "Embedding generation duration").unwrap(),
            database_size: register_int_gauge!("caxton_memory_database_size_bytes", "Database file size in bytes").unwrap(),
        }
    }
}
```

### Health Checks

```rust
impl EmbeddedMemoryBackend {
    pub async fn health_check(&self) -> Result<HealthStatus, BackendError> {
        let start_time = Instant::now();

        // Check database connectivity
        let entity_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM entities")
            .fetch_one(&self.db_pool)
            .await?;

        // Check embedding engine
        let test_embedding = self.embedding_engine.encode_batch(&["health check".to_string()]).await?;

        let duration = start_time.elapsed();

        Ok(HealthStatus {
            status: "healthy".to_string(),
            entity_count: entity_count as u64,
            embedding_model: "all-MiniLM-L6-v2".to_string(),
            response_time_ms: duration.as_millis() as u64,
            database_file_size: self.get_database_file_size().await?,
        })
    }

    async fn get_database_file_size(&self) -> Result<u64, BackendError> {
        let metadata = tokio::fs::metadata(&self.config.database_path).await?;
        Ok(metadata.len())
    }
}
```

## Configuration and Tuning

### Performance Tuning

```yaml
# caxton.yaml memory configuration
memory:
  backend: "embedded"
  embedded:
    database_path: "./data/memory.db"
    model_cache_path: "./data/models"

    # Performance settings
    max_connections: 10
    batch_size: 32
    embedding_cache_size: 10000

    # SQLite optimizations
    wal_mode: true
    cache_size_mb: 64
    mmap_size_mb: 256
    temp_store: "memory"

    # Vector search settings
    vector_index_type: "flat"  # flat, ivf, hnsw (future)
    similarity_threshold: 0.6
```

### Resource Management

```rust
pub struct ResourceLimits {
    pub max_entities: Option<usize>,
    pub max_database_size_mb: Option<usize>,
    pub max_memory_usage_mb: Option<usize>,
    pub max_embedding_cache_size: Option<usize>,
}

impl EmbeddedMemoryBackend {
    async fn check_resource_limits(&self) -> Result<(), BackendError> {
        if let Some(max_entities) = self.config.resource_limits.max_entities {
            let current_count = self.get_entity_count().await?;
            if current_count >= max_entities {
                return Err(BackendError::ResourceLimitExceeded("entity count"));
            }
        }

        if let Some(max_size) = self.config.resource_limits.max_database_size_mb {
            let current_size_mb = self.get_database_file_size().await? / 1_048_576;
            if current_size_mb >= max_size as u64 {
                return Err(BackendError::ResourceLimitExceeded("database size"));
            }
        }

        Ok(())
    }
}
```

## Troubleshooting Guide

### Common Issues

**Slow Embedding Generation**:

```rust
// Enable batch processing
let batch_size = 64; // Increase for better CPU utilization
let embeddings = embedding_engine.encode_batch_with_size(&texts, batch_size).await?;
```

**High Memory Usage**:

```rust
// Reduce embedding cache size
let cache_size = 5000; // Reduce from default 10000
let cache = EmbeddingCache::new(cache_size);
```

**Database Lock Errors**:

```rust
// Increase busy timeout and use WAL mode
let options = SqliteConnectOptions::new()
    .busy_timeout(Duration::from_secs(60))
    .journal_mode(SqliteJournalMode::Wal);
```

**Vector Search Performance**:

```sql
-- Ensure proper indexes exist
CREATE INDEX IF NOT EXISTS idx_embeddings_model ON embeddings(model_version);
VACUUM; -- Defragment database periodically
```

### Debugging Tools

```bash
# Check database integrity
sqlite3 memory.db "PRAGMA integrity_check;"

# Analyze query performance
sqlite3 memory.db ".timer on" "EXPLAIN QUERY PLAN SELECT ..."

# Monitor resource usage
caxton memory stats --backend embedded
```

## Migration and Backup

### Data Export

```rust
impl EmbeddedMemoryBackend {
    pub async fn export_data(&self, format: ExportFormat) -> Result<Vec<u8>, BackendError> {
        match format {
            ExportFormat::Json => {
                let entities = self.get_all_entities().await?;
                let relations = self.get_all_relations().await?;

                let export_data = ExportData {
                    entities,
                    relations,
                    metadata: ExportMetadata {
                        version: "1.0".to_string(),
                        exported_at: SystemTime::now(),
                        backend_type: "embedded".to_string(),
                        entity_count: entities.len(),
                        relation_count: relations.len(),
                    },
                };

                Ok(serde_json::to_vec_pretty(&export_data)?)
            }
        }
    }
}
```

### Database Maintenance

```rust
impl EmbeddedMemoryBackend {
    pub async fn maintenance(&self) -> Result<MaintenanceReport, BackendError> {
        let start_time = Instant::now();

        // Vacuum database
        sqlx::query("VACUUM").execute(&self.db_pool).await?;

        // Analyze tables for query optimization
        sqlx::query("ANALYZE").execute(&self.db_pool).await?;

        // Clean up expired entities
        let deleted_count = self.cleanup_expired_entities().await?;

        let duration = start_time.elapsed();

        Ok(MaintenanceReport {
            vacuum_completed: true,
            analysis_completed: true,
            expired_entities_deleted: deleted_count,
            duration: duration,
            database_size_after: self.get_database_file_size().await?,
        })
    }
}
```

## Related Documentation

- [Memory System Overview](/docs/memory-system/overview.md)
- [Usage Patterns](/docs/memory-system/usage-patterns.md)
- [Migration Guide](/docs/memory-system/migration.md)
- [ADR-0030: Embedded Memory System](/docs/adr/0030-embedded-memory-system.md)
- [ADR-0031: Context Management Architecture](/docs/adr/0031-context-management-architecture.md)
- [Configuration-Driven Agents](/docs/adr/0028-configuration-driven-agent-architecture.md)
- [FIPA-ACL Lightweight Messaging](/docs/adr/0029-fipa-acl-lightweight-messaging.md)
