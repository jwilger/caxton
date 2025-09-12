---
title: "Memory System Migration Guide"
description: "Complete guide for migrating between memory backends and scaling from embedded to external systems"
date: 2025-09-10
categories: [Architecture, Migration, Memory, Operations]
---

## Migration Overview

As Caxton deployments grow, the embedded SQLite+Candle backend may need to
be replaced with external systems for improved performance, scalability, or
specialized features. This guide covers migration strategies, tooling, and
best practices.

## When to Migrate

### Embedded Backend Limitations

**Performance Indicators**:

- Semantic search queries consistently >100ms
- Entity count approaching 100K limit
- Memory usage exceeding 2GB
- Database file size >10GB
- High CPU usage from embedding generation

**Scalability Indicators**:

- Multiple Caxton instances needed
- Cross-instance memory sharing required
- Advanced analytics and reporting needs
- Multi-tenant isolation requirements
- Geographic distribution requirements

### Migration Decision Matrix

| Requirement | Embedded | Neo4j | Qdrant | Hybrid |
|-------------|----------|-------|---------|---------|
| <100K entities | ✅ | ⚠️ | ⚠️ | ✅ |
| >1M entities | ❌ | ✅ | ✅ | ✅ |
| Complex graph queries | ⚠️ | ✅ | ❌ | ✅ |
| High-speed semantic search | ⚠️ | ❌ | ✅ | ✅ |
| Zero configuration | ✅ | ❌ | ❌ | ❌ |
| Multi-node deployment | ❌ | ✅ | ✅ | ✅ |
| Advanced analytics | ❌ | ✅ | ⚠️ | ✅ |
| Cost optimization | ✅ | ❌ | ❌ | ⚠️ |

## Migration Strategies

### 1. Blue-Green Migration

Zero-downtime migration using parallel systems:

```yaml
# migration-config.yaml
migration:
  strategy: "blue-green"
  source:
    backend: "embedded"
    config:
      database_path: "./data/memory.db"
  destination:
    backend: "neo4j"
    config:
      uri: "bolt://neo4j-new:7687"
      username: "caxton"
      password: "${NEO4J_PASSWORD}"
  validation:
    sample_queries: 100
    consistency_checks: true
    performance_comparison: true
```

**Implementation Process**:

```rust
pub struct BlueGreenMigrator {
    source_backend: Arc<dyn MemoryBackend>,
    destination_backend: Arc<dyn MemoryBackend>,
    validator: MigrationValidator,
}

impl BlueGreenMigrator {
    pub async fn execute_migration(&self) -> Result<MigrationReport, MigrationError> {
        let start_time = Instant::now();

        // Phase 1: Export from source
        tracing::info!("Phase 1: Exporting data from source backend");
        let export_data = self.export_source_data().await?;

        // Phase 2: Import to destination
        tracing::info!("Phase 2: Importing data to destination backend");
        self.import_destination_data(&export_data).await?;

        // Phase 3: Validation
        tracing::info!("Phase 3: Validating migration consistency");
        let validation_result = self.validator.validate_migration(
            &*self.source_backend,
            &*self.destination_backend
        ).await?;

        if !validation_result.is_valid {
            return Err(MigrationError::ValidationFailed(validation_result.errors));
        }

        // Phase 4: Switch traffic
        tracing::info!("Phase 4: Migration completed successfully");

        Ok(MigrationReport {
            duration: start_time.elapsed(),
            entities_migrated: export_data.entities.len(),
            relations_migrated: export_data.relations.len(),
            validation_result,
        })
    }

    async fn export_source_data(&self) -> Result<ExportData, MigrationError> {
        // Export all entities and relations
        let entities = self.source_backend.get_all_entities().await?;
        let relations = self.source_backend.get_all_relations().await?;

        Ok(ExportData {
            entities,
            relations,
            metadata: ExportMetadata {
                version: "1.0".to_string(),
                exported_at: SystemTime::now(),
                source_backend: "embedded".to_string(),
                entity_count: entities.len(),
                relation_count: relations.len(),
            },
        })
    }
}
```

### 2. Gradual Migration

Incremental data movement with validation at each step:

```rust
pub struct GradualMigrator {
    source_backend: Arc<dyn MemoryBackend>,
    destination_backend: Arc<dyn MemoryBackend>,
    batch_size: usize,
    validation_threshold: f64,
}

impl GradualMigrator {
    pub async fn execute_gradual_migration(&self) -> Result<MigrationReport, MigrationError> {
        let mut migrated_entities = 0;
        let mut migrated_relations = 0;
        let start_time = Instant::now();

        // Get total counts for progress tracking
        let total_entities = self.source_backend.count_entities().await?;
        let total_relations = self.source_backend.count_relations().await?;

        // Migrate entities in batches
        let mut offset = 0;
        loop {
            let entity_batch = self.source_backend.get_entities_batch(offset, self.batch_size).await?;
            if entity_batch.is_empty() {
                break;
            }

            // Import batch to destination
            let create_requests: Vec<CreateEntityRequest> = entity_batch.into_iter()
                .map(|entity| entity.into())
                .collect();
            self.destination_backend.create_entities(create_requests).await?;

            migrated_entities += entity_batch.len();
            offset += self.batch_size;

            // Progress reporting
            let progress = (migrated_entities as f64) / (total_entities as f64);
            tracing::info!(
                migrated_entities,
                total_entities,
                progress_percent = (progress * 100.0) as u32,
                "Entity migration progress"
            );

            // Validate batch if threshold reached
            if progress >= self.validation_threshold {
                self.validate_partial_migration().await?;
            }
        }

        // Migrate relations in batches
        offset = 0;
        loop {
            let relation_batch = self.source_backend.get_relations_batch(offset, self.batch_size).await?;
            if relation_batch.is_empty() {
                break;
            }

            let create_requests: Vec<CreateRelationRequest> = relation_batch.into_iter()
                .map(|relation| relation.into())
                .collect();
            self.destination_backend.create_relations(create_requests).await?;

            migrated_relations += relation_batch.len();
            offset += self.batch_size;
        }

        Ok(MigrationReport {
            duration: start_time.elapsed(),
            entities_migrated: migrated_entities,
            relations_migrated: migrated_relations,
            validation_result: ValidationResult::default(),
        })
    }
}
```

### 3. Hot Migration

Live migration with dual writes during transition:

```rust
pub struct HotMigrationBackend {
    primary_backend: Arc<dyn MemoryBackend>,
    secondary_backend: Arc<dyn MemoryBackend>,
    migration_state: Arc<RwLock<MigrationState>>,
    dual_write_enabled: AtomicBool,
}

#[derive(Debug, Clone)]
pub enum MigrationState {
    NotStarted,
    DataMigration { progress: f64 },
    DualWrite { cutover_time: SystemTime },
    Completed,
}

impl HotMigrationBackend {
    pub async fn start_migration(&self) -> Result<(), MigrationError> {
        // Phase 1: Begin background data migration
        self.update_migration_state(MigrationState::DataMigration { progress: 0.0 }).await;

        let migrator = BackgroundMigrator::new(
            Arc::clone(&self.primary_backend),
            Arc::clone(&self.secondary_backend),
        );

        tokio::spawn(async move {
            if let Err(e) = migrator.migrate_existing_data().await {
                tracing::error!(error = ?e, "Background migration failed");
            }
        });

        // Phase 2: Enable dual writes once data migration is complete
        // This is handled by the background migrator
        Ok(())
    }

    async fn enable_dual_writes(&self) {
        self.dual_write_enabled.store(true, Ordering::SeqCst);
        self.update_migration_state(MigrationState::DualWrite {
            cutover_time: SystemTime::now() + Duration::from_hours(1)
        }).await;
    }
}

#[async_trait]
impl MemoryBackend for HotMigrationBackend {
    async fn create_entities(&self, entities: Vec<CreateEntityRequest>) -> Result<Vec<EntityId>, MemoryError> {
        let primary_result = self.primary_backend.create_entities(entities.clone()).await;

        // If dual writes are enabled, also write to secondary
        if self.dual_write_enabled.load(Ordering::SeqCst) {
            if let Err(e) = self.secondary_backend.create_entities(entities).await {
                tracing::warn!(error = ?e, "Secondary backend write failed during dual write");
            }
        }

        primary_result
    }

    async fn semantic_search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>, MemoryError> {
        // Always read from primary during migration
        self.primary_backend.semantic_search(query, options).await
    }
}
```

## Backend-Specific Migration

### Embedded to Neo4j

**Prerequisites**:

```bash
# Install Neo4j
docker run -d \
  --name neo4j \
  -p 7474:7474 -p 7687:7687 \
  -e NEO4J_AUTH=caxton/password \
  -e NEO4J_PLUGINS='["graph-data-science"]' \
  neo4j:5.15
```

**Migration Configuration**:

```yaml
# neo4j-migration.yaml
migration:
  source:
    backend: "embedded"
    config:
      database_path: "./data/memory.db"

  destination:
    backend: "neo4j"
    config:
      uri: "bolt://localhost:7687"
      username: "caxton"
      password: "password"
      database: "caxton_memory"

  mapping:
    entity_label: "Entity"
    relation_type_property: "type"
    confidence_property: "confidence"
    strength_property: "strength"
```

**Migration Implementation**:

```rust
pub struct EmbeddedToNeo4jMigrator {
    source: Arc<EmbeddedMemoryBackend>,
    destination: Arc<Neo4jMemoryBackend>,
}

impl EmbeddedToNeo4jMigrator {
    async fn migrate_entities(&self, entities: Vec<Entity>) -> Result<(), MigrationError> {
        // Convert entities to Cypher statements
        let mut transaction = self.destination.begin_transaction().await?;

        for entity in entities {
            let cypher = format!(
                r#"
                CREATE (e:Entity {{
                    name: $name,
                    entity_type: $entity_type,
                    observations: $observations,
                    confidence: $confidence,
                    strength: $strength,
                    version: $version,
                    created_at: $created_at,
                    metadata: $metadata
                }})
                "#
            );

            transaction.run(cypher, hashmap! {
                "name" => entity.name.into(),
                "entity_type" => entity.entity_type.into(),
                "observations" => serde_json::to_string(&entity.observations)?.into(),
                "confidence" => entity.confidence.unwrap_or(1.0).into(),
                "strength" => entity.strength.unwrap_or(1.0).into(),
                "version" => entity.version.unwrap_or(1).into(),
                "created_at" => entity.created_at.unwrap_or_else(SystemTime::now).into(),
                "metadata" => entity.metadata.map(|m| serde_json::to_string(&m).unwrap()).into(),
            }).await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    async fn migrate_relations(&self, relations: Vec<Relation>) -> Result<(), MigrationError> {
        let mut transaction = self.destination.begin_transaction().await?;

        for relation in relations {
            let cypher = format!(
                r#"
                MATCH (from:Entity {{name: $from_name}})
                MATCH (to:Entity {{name: $to_name}})
                CREATE (from)-[r:{} {{
                    strength: $strength,
                    confidence: $confidence,
                    created_at: $created_at,
                    metadata: $metadata
                }}]->(to)
                "#,
                relation.relation_type.to_uppercase()
            );

            transaction.run(cypher, hashmap! {
                "from_name" => relation.from.into(),
                "to_name" => relation.to.into(),
                "strength" => relation.strength.unwrap_or(1.0).into(),
                "confidence" => relation.confidence.unwrap_or(1.0).into(),
                "created_at" => relation.created_at.unwrap_or_else(SystemTime::now).into(),
                "metadata" => relation.metadata.map(|m| serde_json::to_string(&m).unwrap()).into(),
            }).await?;
        }

        transaction.commit().await?;
        Ok(())
    }
}
```

### Embedded to Qdrant

**Prerequisites**:

```bash
# Install Qdrant
docker run -d \
  --name qdrant \
  -p 6333:6333 \
  qdrant/qdrant:v1.7.0
```

**Migration Configuration**:

```yaml
# qdrant-migration.yaml
migration:
  source:
    backend: "embedded"
    config:
      database_path: "./data/memory.db"

  destination:
    backend: "qdrant"
    config:
      url: "http://localhost:6333"
      collection: "caxton_embeddings"
      vector_size: 384
      distance: "Cosine"

  settings:
    batch_size: 1000
    parallel_uploads: 4
    recreate_collection: true
```

**Migration Implementation**:

```rust
pub struct EmbeddedToQdrantMigrator {
    source: Arc<EmbeddedMemoryBackend>,
    destination: Arc<QdrantMemoryBackend>,
    embedding_engine: Arc<EmbeddingEngine>,
}

impl EmbeddedToQdrantMigrator {
    pub async fn migrate_entities_with_embeddings(&self, entities: Vec<Entity>) -> Result<(), MigrationError> {
        // Create collection if it doesn't exist
        self.destination.ensure_collection_exists().await?;

        // Process in batches for memory efficiency
        for entity_batch in entities.chunks(1000) {
            // Generate embeddings for batch
            let texts: Vec<String> = entity_batch.iter()
                .map(|e| self.entity_to_text(e))
                .collect();

            let embeddings = self.embedding_engine.encode_batch(&texts).await?;

            // Create Qdrant points
            let points: Vec<PointStruct> = entity_batch.iter()
                .zip(embeddings.iter())
                .enumerate()
                .map(|(i, (entity, embedding))| {
                    PointStruct::new(
                        (entity.id.0 as u64),
                        embedding.clone(),
                        json!({
                            "name": entity.name,
                            "entity_type": entity.entity_type,
                            "observations": entity.observations,
                            "confidence": entity.confidence.unwrap_or(1.0),
                            "strength": entity.strength.unwrap_or(1.0),
                            "metadata": entity.metadata,
                        }).into()
                    )
                })
                .collect();

            // Upload points to Qdrant
            self.destination.upsert_points(points).await?;
        }

        Ok(())
    }

    fn entity_to_text(&self, entity: &Entity) -> String {
        format!("{} {} {}",
            entity.name,
            entity.entity_type,
            entity.observations.join(" ")
        )
    }
}
```

### Hybrid Backend Migration

For complex scenarios requiring both graph queries and high-performance
semantic search:

```yaml
# hybrid-migration.yaml
migration:
  source:
    backend: "embedded"

  destination:
    backend: "hybrid"
    config:
      graph_backend:
        type: "neo4j"
        uri: "bolt://localhost:7687"
        username: "caxton"
        password: "password"

      vector_backend:
        type: "qdrant"
        url: "http://localhost:6333"
        collection: "caxton_vectors"

  distribution:
    entities: "both"      # Store in both backends
    relations: "neo4j"    # Only in graph backend
    embeddings: "qdrant"  # Only in vector backend
```

## Migration Tools and CLI

### Command Line Interface

```bash
# Export from embedded backend
caxton memory export \
  --backend embedded \
  --database ./data/memory.db \
  --format json \
  --output backup.json

# Import to Neo4j
caxton memory import \
  --backend neo4j \
  --uri bolt://localhost:7687 \
  --username caxton \
  --password password \
  --input backup.json \
  --batch-size 1000

# Migrate directly between backends
caxton memory migrate \
  --source-backend embedded \
  --source-database ./data/memory.db \
  --destination-backend neo4j \
  --destination-uri bolt://localhost:7687 \
  --strategy blue-green \
  --validate

# Check migration status
caxton memory migration-status \
  --migration-id 12345

# Rollback migration
caxton memory rollback \
  --migration-id 12345 \
  --reason "validation_failed"
```

### Migration Validation

```rust
pub struct MigrationValidator {
    sample_size: usize,
    tolerance: f64,
}

impl MigrationValidator {
    pub async fn validate_migration(
        &self,
        source: &dyn MemoryBackend,
        destination: &dyn MemoryBackend,
    ) -> Result<ValidationResult, ValidationError> {
        let mut validation_result = ValidationResult::default();

        // 1. Count validation
        let source_count = source.count_entities().await?;
        let dest_count = destination.count_entities().await?;

        if source_count != dest_count {
            validation_result.errors.push(
                ValidationError::EntityCountMismatch { source_count, dest_count }
            );
        }

        // 2. Random sample validation
        let sample_entities = source.get_random_entities(self.sample_size).await?;

        for entity in sample_entities {
            let dest_entity = destination.open_nodes(&[entity.name.clone()]).await?;

            if dest_entity.is_empty() {
                validation_result.errors.push(
                    ValidationError::MissingEntity(entity.name)
                );
                continue;
            }

            // Compare entity content
            let similarity = self.compare_entities(&entity, &dest_entity[0]);
            if similarity < self.tolerance {
                validation_result.errors.push(
                    ValidationError::EntityMismatch {
                        entity_name: entity.name,
                        similarity,
                        threshold: self.tolerance,
                    }
                );
            }
        }

        // 3. Search consistency validation
        validation_result.search_consistency = self.validate_search_consistency(
            source, destination
        ).await?;

        validation_result.is_valid = validation_result.errors.is_empty()
            && validation_result.search_consistency > 0.95;

        Ok(validation_result)
    }

    async fn validate_search_consistency(
        &self,
        source: &dyn MemoryBackend,
        destination: &dyn MemoryBackend,
    ) -> Result<f64, ValidationError> {
        let test_queries = vec![
            "machine learning algorithms",
            "database optimization",
            "software architecture patterns",
            "error handling strategies",
            "performance optimization",
        ];

        let mut consistent_queries = 0;

        for query in test_queries {
            let source_results = source.semantic_search(query, SearchOptions::default()).await?;
            let dest_results = destination.semantic_search(query, SearchOptions::default()).await?;

            let similarity = self.compare_search_results(&source_results, &dest_results);
            if similarity > 0.8 {
                consistent_queries += 1;
            }
        }

        Ok(consistent_queries as f64 / test_queries.len() as f64)
    }
}
```

## Performance Optimization During Migration

### Parallel Processing

```rust
pub struct ParallelMigrator {
    source: Arc<dyn MemoryBackend>,
    destination: Arc<dyn MemoryBackend>,
    worker_count: usize,
    batch_size: usize,
}

impl ParallelMigrator {
    pub async fn migrate_with_parallel_workers(&self) -> Result<MigrationReport, MigrationError> {
        let total_entities = self.source.count_entities().await?;
        let batches_per_worker = (total_entities + self.batch_size - 1) / self.batch_size / self.worker_count;

        let mut tasks = Vec::new();

        for worker_id in 0..self.worker_count {
            let start_offset = worker_id * batches_per_worker * self.batch_size;
            let end_offset = min(start_offset + batches_per_worker * self.batch_size, total_entities);

            let source = Arc::clone(&self.source);
            let destination = Arc::clone(&self.destination);
            let batch_size = self.batch_size;

            let task = tokio::spawn(async move {
                Self::migrate_range(source, destination, start_offset, end_offset, batch_size).await
            });

            tasks.push(task);
        }

        // Wait for all workers to complete
        let results = futures::future::try_join_all(tasks).await?;

        // Aggregate results
        let total_migrated = results.iter().sum::<usize>();

        Ok(MigrationReport {
            entities_migrated: total_migrated,
            // ... other fields
        })
    }

    async fn migrate_range(
        source: Arc<dyn MemoryBackend>,
        destination: Arc<dyn MemoryBackend>,
        start_offset: usize,
        end_offset: usize,
        batch_size: usize,
    ) -> Result<usize, MigrationError> {
        let mut migrated_count = 0;
        let mut offset = start_offset;

        while offset < end_offset {
            let current_batch_size = min(batch_size, end_offset - offset);
            let entities = source.get_entities_batch(offset, current_batch_size).await?;

            if entities.is_empty() {
                break;
            }

            let create_requests: Vec<CreateEntityRequest> = entities.into_iter()
                .map(|entity| entity.into())
                .collect();

            destination.create_entities(create_requests).await?;

            migrated_count += current_batch_size;
            offset += current_batch_size;
        }

        Ok(migrated_count)
    }
}
```

### Memory Management

```rust
pub struct MemoryEfficientMigrator {
    source: Arc<dyn MemoryBackend>,
    destination: Arc<dyn MemoryBackend>,
    max_memory_usage: usize,
}

impl MemoryEfficientMigrator {
    pub async fn migrate_with_memory_limit(&self) -> Result<MigrationReport, MigrationError> {
        let mut migrated_entities = 0;
        let mut current_memory_usage = 0;
        let mut entity_buffer = Vec::new();

        let entity_stream = self.source.stream_entities().await?;

        tokio::pin!(entity_stream);

        while let Some(entity) = entity_stream.next().await {
            let entity = entity?;
            let entity_size = self.estimate_entity_size(&entity);

            // Check if adding this entity would exceed memory limit
            if current_memory_usage + entity_size > self.max_memory_usage && !entity_buffer.is_empty() {
                // Flush current buffer
                self.flush_entity_buffer(&mut entity_buffer, &mut current_memory_usage).await?;
                migrated_entities += entity_buffer.len();
                entity_buffer.clear();
                current_memory_usage = 0;
            }

            entity_buffer.push(entity);
            current_memory_usage += entity_size;
        }

        // Flush remaining entities
        if !entity_buffer.is_empty() {
            self.flush_entity_buffer(&mut entity_buffer, &mut current_memory_usage).await?;
            migrated_entities += entity_buffer.len();
        }

        Ok(MigrationReport {
            entities_migrated: migrated_entities,
            // ... other fields
        })
    }
}
```

## Error Handling and Recovery

### Migration Error Types

```rust
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Source backend error: {0}")]
    SourceBackend(#[from] SourceBackendError),

    #[error("Destination backend error: {0}")]
    DestinationBackend(#[from] DestinationBackendError),

    #[error("Validation failed: {errors:?}")]
    ValidationFailed { errors: Vec<ValidationError> },

    #[error("Migration timeout after {duration:?}")]
    Timeout { duration: Duration },

    #[error("Insufficient disk space: needed {needed}, available {available}")]
    InsufficientSpace { needed: u64, available: u64 },

    #[error("Memory limit exceeded: {current} > {limit}")]
    MemoryLimitExceeded { current: usize, limit: usize },

    #[error("Network connectivity issue: {details}")]
    NetworkError { details: String },
}
```

### Recovery Mechanisms

```rust
pub struct MigrationRecoveryManager {
    checkpoint_storage: Arc<dyn CheckpointStorage>,
    retry_policy: RetryPolicy,
}

impl MigrationRecoveryManager {
    pub async fn migrate_with_recovery(
        &self,
        migration_config: MigrationConfig
    ) -> Result<MigrationReport, MigrationError> {
        let migration_id = Uuid::new_v4();

        // Check for existing checkpoint
        if let Some(checkpoint) = self.checkpoint_storage.load_checkpoint(migration_id).await? {
            tracing::info!("Resuming migration from checkpoint: {:?}", checkpoint);
            return self.resume_from_checkpoint(checkpoint).await;
        }

        // Start new migration with checkpointing
        self.execute_migration_with_checkpoints(migration_id, migration_config).await
    }

    async fn execute_migration_with_checkpoints(
        &self,
        migration_id: Uuid,
        config: MigrationConfig
    ) -> Result<MigrationReport, MigrationError> {
        let mut checkpoint = MigrationCheckpoint {
            migration_id,
            entities_completed: 0,
            relations_completed: 0,
            last_entity_id: None,
            last_relation_id: None,
            phase: MigrationPhase::Entities,
        };

        // Migrate entities with checkpointing
        while checkpoint.phase == MigrationPhase::Entities {
            match self.migrate_entity_batch(&mut checkpoint, &config).await {
                Ok(completed) => {
                    if completed {
                        checkpoint.phase = MigrationPhase::Relations;
                    }
                    self.checkpoint_storage.save_checkpoint(&checkpoint).await?;
                },
                Err(e) if self.retry_policy.should_retry(&e) => {
                    tracing::warn!("Migration error, retrying: {:?}", e);
                    tokio::time::sleep(self.retry_policy.delay()).await;
                    continue;
                },
                Err(e) => return Err(e),
            }
        }

        // Migrate relations with checkpointing
        while checkpoint.phase == MigrationPhase::Relations {
            match self.migrate_relation_batch(&mut checkpoint, &config).await {
                Ok(completed) => {
                    if completed {
                        checkpoint.phase = MigrationPhase::Validation;
                    }
                    self.checkpoint_storage.save_checkpoint(&checkpoint).await?;
                },
                Err(e) if self.retry_policy.should_retry(&e) => {
                    tracing::warn!("Migration error, retrying: {:?}", e);
                    tokio::time::sleep(self.retry_policy.delay()).await;
                    continue;
                },
                Err(e) => return Err(e),
            }
        }

        // Final validation
        // ... validation logic

        // Clean up checkpoint
        self.checkpoint_storage.delete_checkpoint(migration_id).await?;

        Ok(MigrationReport {
            entities_migrated: checkpoint.entities_completed,
            relations_migrated: checkpoint.relations_completed,
            // ... other fields
        })
    }
}
```

## Post-Migration Operations

### Configuration Updates

After successful migration, update Caxton configuration:

```yaml
# Updated caxton.yaml
memory:
  # Switch from embedded to new backend
  backend: "neo4j"  # or "qdrant", "hybrid"

  neo4j:
    uri: "bolt://neo4j:7687"
    username: "caxton"
    password: "${NEO4J_PASSWORD}"
    database: "caxton_memory"

  # Keep embedded config commented for rollback
  # embedded:
  #   database_path: "./data/memory.db.backup"
  #   model_cache_path: "./data/models"
```

### Performance Optimization

```rust
// Post-migration optimization for Neo4j
impl Neo4jPostMigrationOptimizer {
    pub async fn optimize_after_migration(&self) -> Result<(), OptimizationError> {
        // Create indexes for better performance
        self.create_performance_indexes().await?;

        // Update statistics for query planner
        self.update_statistics().await?;

        // Configure memory settings
        self.optimize_memory_settings().await?;

        Ok(())
    }

    async fn create_performance_indexes(&self) -> Result<(), OptimizationError> {
        let indexes = vec![
            "CREATE INDEX entity_name_idx IF NOT EXISTS FOR (e:Entity) ON (e.name)",
            "CREATE INDEX entity_type_idx IF NOT EXISTS FOR (e:Entity) ON (e.entity_type)",
            "CREATE INDEX relation_strength_idx IF NOT EXISTS FOR ()-[r]-() ON (r.strength)",
            "CREATE INDEX entity_confidence_idx IF NOT EXISTS FOR (e:Entity) ON (e.confidence)",
        ];

        for index_query in indexes {
            self.execute_cypher(index_query).await?;
        }

        Ok(())
    }
}
```

### Monitoring and Alerting

```rust
pub struct PostMigrationMonitor {
    source_metrics: Arc<BackendMetrics>,
    destination_metrics: Arc<BackendMetrics>,
    alert_manager: Arc<AlertManager>,
}

impl PostMigrationMonitor {
    pub async fn monitor_post_migration(&self, duration: Duration) -> Result<MonitoringReport, MonitoringError> {
        let start_time = Instant::now();
        let mut report = MonitoringReport::default();

        while start_time.elapsed() < duration {
            // Compare query performance
            let performance_comparison = self.compare_query_performance().await?;
            report.performance_samples.push(performance_comparison);

            // Check for errors
            let error_rate = self.destination_metrics.get_error_rate().await?;
            if error_rate > 0.01 {  // 1% error threshold
                self.alert_manager.send_alert(
                    AlertType::HighErrorRate,
                    format!("Post-migration error rate: {:.2}%", error_rate * 100.0)
                ).await?;
            }

            // Check response times
            let avg_response_time = self.destination_metrics.get_avg_response_time().await?;
            if avg_response_time > Duration::from_millis(100) {
                self.alert_manager.send_alert(
                    AlertType::SlowResponse,
                    format!("Average response time: {:?}", avg_response_time)
                ).await?;
            }

            tokio::time::sleep(Duration::from_minutes(5)).await;
        }

        Ok(report)
    }
}
```

## Best Practices

### Pre-Migration Checklist

- [ ] **Backup all data** - Create full export before starting
- [ ] **Test migration in staging** - Never migrate production directly
- [ ] **Verify resource requirements** - Disk, memory, network capacity
- [ ] **Plan downtime window** - Even "zero-downtime" migrations need
  maintenance windows
- [ ] **Prepare rollback plan** - Know how to revert if migration fails
- [ ] **Monitor destination backend** - Ensure target system is healthy
- [ ] **Update application configuration** - Prepare config changes
- [ ] **Validate data consistency** - Run consistency checks on source data

### During Migration

- [ ] **Monitor progress continuously** - Track entity and relation counts
- [ ] **Watch resource usage** - CPU, memory, disk, network utilization
- [ ] **Validate sample data** - Check random samples for consistency
- [ ] **Maintain logs** - Detailed logging for troubleshooting
- [ ] **Be prepared to pause/rollback** - Stop if issues are detected
- [ ] **Communicate status** - Keep stakeholders informed

### Post-Migration

- [ ] **Run full validation** - Comprehensive consistency checks
- [ ] **Performance testing** - Verify query performance meets expectations
- [ ] **Monitor error rates** - Watch for increased errors or timeouts
- [ ] **Update documentation** - Document new backend configuration
- [ ] **Train team members** - Ensure operational knowledge transfer
- [ ] **Archive old backend** - Keep backup but stop using embedded system
- [ ] **Plan capacity scaling** - Monitor growth and plan future scaling

## Troubleshooting Common Issues

### Migration Failures

**Entity Count Mismatch**:

```bash
# Check source count
caxton memory stats --backend embedded --database ./data/memory.db

# Check destination count
caxton memory stats --backend neo4j --uri bolt://localhost:7687

# Find missing entities
caxton memory diff \
  --source embedded:./data/memory.db \
  --destination neo4j:bolt://localhost:7687
```

**Performance Degradation**:

```cypher
-- Neo4j: Check for missing indexes
CALL db.indexes()

-- Create missing performance indexes
CREATE INDEX entity_search_idx IF NOT EXISTS
FOR (e:Entity) ON (e.name, e.entity_type)
```

**Memory Issues During Migration**:

```yaml
# Reduce batch size
migration:
  batch_size: 100  # Reduce from default 1000
  parallel_workers: 2  # Reduce parallelism
  memory_limit_mb: 1024
```

### Data Consistency Issues

**Embedding Mismatches**:

```rust
// Re-generate embeddings for consistency
pub async fn regenerate_embeddings(&self, entity_names: Vec<String>) -> Result<(), RegenerationError> {
    for entity_name in entity_names {
        let entity = self.backend.open_nodes(&[entity_name]).await?;
        let text = self.entity_to_text(&entity[0]);
        let embedding = self.embedding_engine.encode(&text).await?;
        self.backend.update_embedding(entity[0].id, embedding).await?;
    }
    Ok(())
}
```

## Related Documentation

- [Memory System Overview](/docs/memory-system/overview.md)
- [Embedded Backend Guide](/docs/memory-system/embedded-backend.md)
- [Usage Patterns](/docs/memory-system/usage-patterns.md)
- [ADR-0030: Embedded Memory System](/docs/_adrs/0030-embedded-memory-system.md)
- [Operational Runbook](/docs/operations/operational-runbook.md)
