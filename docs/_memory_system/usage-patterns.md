---
title: "Memory System Usage Patterns"
description: "Comprehensive guide to integrating memory capabilities into Caxton agents with practical patterns and examples"
date: 2025-09-10
categories: [Architecture, Usage, Memory, Agents]
---

## Agent Memory Integration

The Caxton memory system seamlessly integrates with configuration-driven
agents to provide context-aware responses and continuous learning. This guide
covers practical patterns for memory-enabled agents.

## Memory-Enabled Agent Configuration

### Basic Memory Configuration

```yaml
# agent-config.yaml
name: "research-assistant"
description: "AI agent that learns from research interactions"
capabilities: ["research", "analysis", "summarization"]

memory:
  enabled: true
  scope: "workspace"
  backend: "embedded"
  settings:
    auto_store: true
    search_threshold: 0.7
    max_results: 10
    retention_days: 365
```

### Advanced Memory Settings

```yaml
memory:
  enabled: true
  scope: "agent-only"  # or "workspace", "global"
  backend: "embedded"

  settings:
    # Search configuration
    auto_store: true
    search_threshold: 0.7
    max_results: 10
    hybrid_search: true
    semantic_weight: 0.6

    # Content filtering
    content_types: ["solution", "pattern", "error", "preference"]
    exclude_patterns: ["password", "api_key", "secret"]

    # Temporal settings
    retention_days: 365
    auto_cleanup: true
    version_limit: 5

    # Performance tuning
    batch_size: 32
    cache_enabled: true
    preload_context: true
```

## Memory Scopes and Patterns

### Agent-Only Memory

**Use Case**: Specialized agents that need private learning contexts

```yaml
# specialist-agent.yaml
name: "code-reviewer"
memory:
  scope: "agent-only"
  settings:
    # Private patterns and preferences
    auto_store: true
    content_types: ["code_pattern", "review_feedback", "style_preference"]
```

**Storage Patterns**:

```rust
// Automatic storage during agent operation
impl CodeReviewerAgent {
    async fn review_code(&self, code: &str) -> Result<ReviewResult, AgentError> {
        // Search for similar code patterns
        let similar_patterns = self.memory.semantic_search(
            &format!("code review patterns {}", extract_language(code)),
            SearchOptions::default()
        ).await?;

        // Generate review using context
        let review = self.generate_review(code, &similar_patterns).await?;

        // Store successful review patterns
        if review.confidence > 0.8 {
            self.memory.store_entity(Entity {
                name: format!("successful_review_{}", generate_id()),
                entity_type: "code_pattern".to_string(),
                observations: vec![
                    format!("Language: {}", extract_language(code)),
                    format!("Issues found: {}", review.issues.len()),
                    format!("Review approach: {}", review.approach),
                    format!("Success indicators: {}", review.success_factors.join(", ")),
                ],
            }).await?;
        }

        Ok(review)
    }
}
```

### Workspace Memory

**Use Case**: Team collaboration and shared project context

```yaml
# project-agents.yaml
research_agent:
  memory:
    scope: "workspace"
    settings:
      content_types: ["research_finding", "source", "fact"]

planning_agent:
  memory:
    scope: "workspace"
    settings:
      content_types: ["plan", "task", "dependency"]

implementation_agent:
  memory:
    scope: "workspace"
    settings:
      content_types: ["implementation", "solution", "bug_fix"]
```

**Collaboration Pattern**:

```rust
impl WorkspaceMemoryPattern {
    async fn collaborative_task(&self, task: Task) -> Result<TaskResult, AgentError> {
        // Research agent stores findings
        let research_findings = self.research_agent.investigate(&task).await?;
        self.workspace_memory.store_entities(research_findings).await?;

        // Planning agent uses research to create plan
        let context = self.workspace_memory.semantic_search(
            &format!("research findings {}", task.domain),
            SearchOptions { limit: Some(20), ..Default::default() }
        ).await?;

        let plan = self.planning_agent.create_plan(&task, &context).await?;
        self.workspace_memory.store_entity(plan.to_entity()).await?;

        // Implementation agent uses both research and plan
        let implementation_context = self.workspace_memory.semantic_search(
            &format!("implementation approach {}", task.technology),
            SearchOptions { limit: Some(15), ..Default::default() }
        ).await?;

        self.implementation_agent.execute(&task, &implementation_context).await
    }
}
```

### Global Memory

**Use Case**: Organization-wide knowledge base and best practices

```yaml
# global-knowledge-agents.yaml
best_practices_agent:
  memory:
    scope: "global"
    settings:
      content_types: ["best_practice", "standard", "guideline"]

compliance_agent:
  memory:
    scope: "global"
    settings:
      content_types: ["regulation", "policy", "requirement"]

learning_agent:
  memory:
    scope: "global"
    settings:
      content_types: ["lesson_learned", "case_study", "pattern"]
```

## Automatic Knowledge Management

### Pattern Recognition and Storage

```rust
pub struct AutomaticKnowledgeManager {
    memory_backend: Arc<dyn MemoryBackend>,
    pattern_detector: PatternDetector,
    knowledge_extractor: KnowledgeExtractor,
}

impl AutomaticKnowledgeManager {
    pub async fn process_agent_interaction(
        &self,
        interaction: &AgentInteraction
    ) -> Result<Vec<Entity>, KnowledgeError> {
        let mut extracted_entities = Vec::new();

        // Detect successful solution patterns
        if interaction.outcome.is_successful() {
            let solution_pattern = self.extract_solution_pattern(interaction).await?;
            extracted_entities.push(solution_pattern);
        }

        // Extract user preferences
        if let Some(preferences) = self.extract_user_preferences(interaction).await? {
            extracted_entities.push(preferences);
        }

        // Identify error recovery patterns
        if let Some(error_recovery) = self.extract_error_recovery(interaction).await? {
            extracted_entities.push(error_recovery);
        }

        // Store domain knowledge
        if let Some(domain_knowledge) = self.extract_domain_knowledge(interaction).await? {
            extracted_entities.push(domain_knowledge);
        }

        // Store in memory
        if !extracted_entities.is_empty() {
            self.memory_backend.create_entities(
                extracted_entities.clone().into_iter()
                    .map(CreateEntityRequest::from)
                    .collect()
            ).await?;
        }

        Ok(extracted_entities)
    }

    async fn extract_solution_pattern(&self, interaction: &AgentInteraction) -> Result<Entity, KnowledgeError> {
        Ok(Entity {
            name: format!("solution_pattern_{}", generate_id()),
            entity_type: "solution_pattern".to_string(),
            observations: vec![
                format!("Problem type: {}", interaction.problem_type),
                format!("Solution approach: {}", interaction.solution_approach),
                format!("Success factors: {}", interaction.success_factors.join(", ")),
                format!("Tools used: {}", interaction.tools_used.join(", ")),
                format!("Context: {}", interaction.context),
                format!("Outcome: {}", interaction.outcome.description),
            ],
            confidence: Some(interaction.outcome.confidence),
            strength: Some(calculate_solution_strength(&interaction.outcome)),
            metadata: Some(json!({
                "domain": interaction.domain,
                "complexity": interaction.complexity,
                "duration_ms": interaction.duration.as_millis(),
                "user_satisfaction": interaction.user_feedback.satisfaction_score,
            })),
        })
    }
}
```

### Context-Aware Response Generation

```rust
impl MemoryEnabledAgent {
    async fn generate_contextual_response(&self, query: &str) -> Result<Response, AgentError> {
        // Search for relevant context
        let context_entities = self.memory.semantic_search(
            &format!("relevant context {}", query),
            SearchOptions {
                limit: Some(10),
                min_similarity: Some(0.7),
                hybrid_search: Some(true),
                ..Default::default()
            }
        ).await?;

        // Search for similar past queries
        let similar_queries = self.memory.semantic_search(
            &format!("similar query {}", query),
            SearchOptions {
                limit: Some(5),
                min_similarity: Some(0.8),
                entity_types: Some(vec!["user_query".to_string(), "solved_problem".to_string()]),
                ..Default::default()
            }
        ).await?;

        // Build context for response generation
        let context = ResponseContext {
            current_query: query.to_string(),
            relevant_knowledge: context_entities,
            similar_solutions: similar_queries,
            user_preferences: self.get_user_preferences().await?,
        };

        // Generate response with context
        let response = self.llm_client.generate_response(&context).await?;

        // Store interaction for learning
        self.store_interaction(&query, &response, &context).await?;

        Ok(response)
    }

    async fn get_user_preferences(&self) -> Result<Vec<Entity>, AgentError> {
        self.memory.semantic_search(
            "user preferences settings style",
            SearchOptions {
                limit: Some(20),
                entity_types: Some(vec!["user_preference".to_string()]),
                ..Default::default()
            }
        ).await.map_err(Into::into)
    }
}
```

## Memory Query Patterns

### Semantic Search Patterns

```rust
// Pattern 1: Contextual Information Retrieval
async fn get_contextual_info(&self, topic: &str) -> Result<Vec<Entity>, MemoryError> {
    self.memory.semantic_search(
        &format!("context information about {}", topic),
        SearchOptions {
            limit: Some(15),
            min_similarity: Some(0.6),
            hybrid_search: Some(true),
            semantic_weight: Some(0.7),
        }
    ).await
}

// Pattern 2: Solution Pattern Discovery
async fn find_solution_patterns(&self, problem: &str) -> Result<Vec<Entity>, MemoryError> {
    self.memory.semantic_search(
        &format!("solution patterns for {}", problem),
        SearchOptions {
            limit: Some(10),
            min_similarity: Some(0.75),
            entity_types: Some(vec!["solution_pattern".to_string(), "successful_approach".to_string()]),
        }
    ).await
}

// Pattern 3: Error Resolution Lookup
async fn find_error_resolutions(&self, error: &str) -> Result<Vec<Entity>, MemoryError> {
    self.memory.semantic_search(
        &format!("error resolution fix for {}", error),
        SearchOptions {
            limit: Some(8),
            min_similarity: Some(0.8),
            entity_types: Some(vec!["error_fix".to_string(), "troubleshooting".to_string()]),
        }
    ).await
}

// Pattern 4: Historical Context Building
async fn build_historical_context(&self, project: &str) -> Result<Vec<Entity>, MemoryError> {
    let recent_entities = self.memory.search_nodes(
        &format!("project:{} recent activity", project)
    ).await?;

    let mut context_entities = Vec::new();

    // Get related entities through graph traversal
    if !recent_entities.is_empty() {
        let entity_ids: Vec<EntityId> = recent_entities.iter().map(|e| e.id).collect();
        let graph_result = self.memory.traverse_graph(
            &entity_ids,
            TraversalOptions {
                max_depth: Some(2),
                min_strength: Some(0.5),
                relation_types: Some(vec!["relates_to".to_string(), "builds_on".to_string()]),
            }
        ).await?;

        context_entities.extend(graph_result.entities);
    }

    Ok(context_entities)
}
```

### Graph Traversal Patterns

```rust
// Pattern 1: Knowledge Chain Discovery
impl KnowledgeChainPattern {
    async fn discover_knowledge_chain(&self, start_concept: &str) -> Result<KnowledgeChain, MemoryError> {
        // Find the starting entity
        let start_entities = self.memory.search_nodes(start_concept).await?;
        if start_entities.is_empty() {
            return Err(MemoryError::EntityNotFound(start_concept.to_string()));
        }

        // Traverse knowledge dependencies
        let chain = self.memory.traverse_graph(
            &[start_entities[0].id],
            TraversalOptions {
                max_depth: Some(5),
                min_strength: Some(0.6),
                relation_types: Some(vec!["depends_on".to_string(), "builds_on".to_string()]),
                direction: Some(TraversalDirection::Outgoing),
            }
        ).await?;

        Ok(KnowledgeChain {
            root_concept: start_entities[0].clone(),
            dependencies: chain.entities,
            relationships: chain.relations,
        })
    }
}

// Pattern 2: Impact Analysis
impl ImpactAnalysisPattern {
    async fn analyze_change_impact(&self, changed_entity: &str) -> Result<ImpactAnalysis, MemoryError> {
        let entity = self.memory.open_nodes(&[changed_entity.to_string()]).await?;
        if entity.is_empty() {
            return Err(MemoryError::EntityNotFound(changed_entity.to_string()));
        }

        // Find all entities that depend on this one
        let impact_graph = self.memory.traverse_graph(
            &[entity[0].id],
            TraversalOptions {
                max_depth: Some(3),
                min_strength: Some(0.4),
                relation_types: Some(vec!["depends_on".to_string(), "uses".to_string()]),
                direction: Some(TraversalDirection::Incoming),
            }
        ).await?;

        Ok(ImpactAnalysis {
            changed_entity: entity[0].clone(),
            affected_entities: impact_graph.entities,
            impact_relationships: impact_graph.relations,
            severity_score: self.calculate_impact_severity(&impact_graph),
        })
    }
}

// Pattern 3: Concept Clustering
impl ConceptClusteringPattern {
    async fn find_related_concepts(&self, central_concept: &str, radius: usize) -> Result<ConceptCluster, MemoryError> {
        let center_entities = self.memory.search_nodes(central_concept).await?;
        if center_entities.is_empty() {
            return Err(MemoryError::EntityNotFound(central_concept.to_string()));
        }

        let cluster = self.memory.traverse_graph(
            &[center_entities[0].id],
            TraversalOptions {
                max_depth: Some(radius),
                min_strength: Some(0.3),
                relation_types: Some(vec!["relates_to".to_string(), "similar_to".to_string()]),
                direction: Some(TraversalDirection::Bidirectional),
            }
        ).await?;

        Ok(ConceptCluster {
            center: center_entities[0].clone(),
            related_concepts: cluster.entities,
            concept_relations: cluster.relations,
            cluster_coherence: self.calculate_cluster_coherence(&cluster),
        })
    }
}
```

## Memory Lifecycle Management

### Automated Cleanup Patterns

```rust
pub struct MemoryLifecycleManager {
    memory_backend: Arc<dyn MemoryBackend>,
    cleanup_policies: Vec<CleanupPolicy>,
}

#[derive(Debug, Clone)]
pub struct CleanupPolicy {
    pub name: String,
    pub condition: CleanupCondition,
    pub action: CleanupAction,
    pub schedule: CleanupSchedule,
}

#[derive(Debug, Clone)]
pub enum CleanupCondition {
    Age(Duration),
    LowConfidence(f64),
    LowStrength(f64),
    EntityType(String),
    Custom(String), // Custom query
}

impl MemoryLifecycleManager {
    pub async fn run_cleanup(&self) -> Result<CleanupReport, MemoryError> {
        let mut report = CleanupReport::default();

        for policy in &self.cleanup_policies {
            let cleanup_result = self.apply_cleanup_policy(policy).await?;
            report.merge(cleanup_result);
        }

        Ok(report)
    }

    async fn apply_cleanup_policy(&self, policy: &CleanupPolicy) -> Result<CleanupResult, MemoryError> {
        match &policy.condition {
            CleanupCondition::Age(max_age) => {
                let cutoff_time = SystemTime::now() - *max_age;
                let old_entities = self.find_entities_older_than(cutoff_time).await?;

                match &policy.action {
                    CleanupAction::Delete => {
                        let entity_names: Vec<String> = old_entities.iter().map(|e| e.name.clone()).collect();
                        self.memory_backend.delete_entities(entity_names).await?;
                        Ok(CleanupResult {
                            policy_name: policy.name.clone(),
                            entities_deleted: old_entities.len(),
                            entities_archived: 0,
                        })
                    },
                    CleanupAction::Archive => {
                        self.archive_entities(&old_entities).await?;
                        Ok(CleanupResult {
                            policy_name: policy.name.clone(),
                            entities_deleted: 0,
                            entities_archived: old_entities.len(),
                        })
                    },
                }
            },
            CleanupCondition::LowConfidence(threshold) => {
                let low_confidence_entities = self.find_low_confidence_entities(*threshold).await?;
                // Apply cleanup action...
                Ok(CleanupResult::default())
            },
            // Handle other conditions...
            _ => Ok(CleanupResult::default()),
        }
    }
}
```

### Version Management

```rust
impl MemoryVersionManager {
    pub async fn update_entity_with_versioning(
        &self,
        entity_name: &str,
        new_observations: Vec<String>
    ) -> Result<Entity, MemoryError> {
        // Get current entity
        let current_entities = self.memory_backend.open_nodes(&[entity_name.to_string()]).await?;
        if current_entities.is_empty() {
            return Err(MemoryError::EntityNotFound(entity_name.to_string()));
        }

        let current_entity = &current_entities[0];

        // Create new version
        let new_version = Entity {
            name: current_entity.name.clone(),
            entity_type: current_entity.entity_type.clone(),
            observations: new_observations,
            confidence: current_entity.confidence,
            strength: current_entity.strength,
            version: Some(current_entity.version.unwrap_or(1) + 1),
            valid_from: Some(SystemTime::now()),
            metadata: current_entity.metadata.clone(),
        };

        // Archive current version by setting valid_to
        self.archive_entity_version(current_entity).await?;

        // Store new version
        self.memory_backend.create_entities(vec![new_version.into()]).await?;

        Ok(new_version)
    }

    pub async fn get_entity_history(&self, entity_name: &str) -> Result<Vec<Entity>, MemoryError> {
        // Implementation depends on backend support for temporal queries
        self.memory_backend.get_entity_history(entity_name).await
    }
}
```

## Performance Patterns

### Batch Operations

```rust
impl BatchMemoryOperations {
    pub async fn bulk_knowledge_import(&self, knowledge_items: Vec<KnowledgeItem>) -> Result<ImportResult, MemoryError> {
        const BATCH_SIZE: usize = 100;
        let mut total_created = 0;
        let mut total_relations = 0;

        // Process entities in batches
        for entity_batch in knowledge_items.iter().map(|k| &k.entity).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let create_requests: Vec<CreateEntityRequest> = entity_batch.iter()
                .map(|&entity| entity.clone().into())
                .collect();

            let created_ids = self.memory_backend.create_entities(create_requests).await?;
            total_created += created_ids.len();
        }

        // Process relations in batches
        let all_relations: Vec<Relation> = knowledge_items.iter()
            .flat_map(|k| k.relations.iter().cloned())
            .collect();

        for relation_batch in all_relations.chunks(BATCH_SIZE) {
            let create_requests: Vec<CreateRelationRequest> = relation_batch.iter()
                .map(|relation| relation.clone().into())
                .collect();

            let created_ids = self.memory_backend.create_relations(create_requests).await?;
            total_relations += created_ids.len();
        }

        Ok(ImportResult {
            entities_created: total_created,
            relations_created: total_relations,
            processing_time: start_time.elapsed(),
        })
    }
}
```

### Caching Patterns

```rust
pub struct MemoryCacheLayer {
    memory_backend: Arc<dyn MemoryBackend>,
    search_cache: Arc<Mutex<LruCache<String, Vec<SearchResult>>>>,
    entity_cache: Arc<Mutex<LruCache<String, Entity>>>,
    cache_ttl: Duration,
}

impl MemoryCacheLayer {
    pub async fn cached_semantic_search(
        &self,
        query: &str,
        options: SearchOptions
    ) -> Result<Vec<SearchResult>, MemoryError> {
        let cache_key = format!("{}:{:?}", query, options);

        // Check cache first
        {
            let mut cache = self.search_cache.lock().await;
            if let Some(results) = cache.get(&cache_key) {
                return Ok(results.clone());
            }
        }

        // Perform actual search
        let results = self.memory_backend.semantic_search(query, options).await?;

        // Cache results
        {
            let mut cache = self.search_cache.lock().await;
            cache.put(cache_key, results.clone());
        }

        Ok(results)
    }
}
```

## Testing Memory-Enabled Agents

### Memory Mock for Testing

```rust
pub struct MockMemoryBackend {
    entities: Arc<Mutex<HashMap<String, Entity>>>,
    relations: Arc<Mutex<Vec<Relation>>>,
    search_responses: Arc<Mutex<HashMap<String, Vec<SearchResult>>>>,
}

impl MockMemoryBackend {
    pub fn new() -> Self {
        Self {
            entities: Arc::new(Mutex::new(HashMap::new())),
            relations: Arc::new(Mutex::new(Vec::new())),
            search_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn setup_search_response(&self, query: &str, results: Vec<SearchResult>) {
        let mut responses = self.search_responses.lock().await;
        responses.insert(query.to_string(), results);
    }

    pub async fn verify_entity_stored(&self, entity_name: &str) -> bool {
        let entities = self.entities.lock().await;
        entities.contains_key(entity_name)
    }
}

#[async_trait]
impl MemoryBackend for MockMemoryBackend {
    async fn semantic_search(&self, query: &str, _options: SearchOptions) -> Result<Vec<SearchResult>, MemoryError> {
        let responses = self.search_responses.lock().await;
        Ok(responses.get(query).cloned().unwrap_or_default())
    }

    async fn create_entities(&self, entities: Vec<CreateEntityRequest>) -> Result<Vec<EntityId>, MemoryError> {
        let mut stored_entities = self.entities.lock().await;
        let mut ids = Vec::new();

        for (i, entity_req) in entities.into_iter().enumerate() {
            let id = EntityId(i as i64);
            let entity = Entity {
                id,
                name: entity_req.name.clone(),
                entity_type: entity_req.entity_type,
                observations: entity_req.observations,
                confidence: entity_req.confidence,
                strength: entity_req.strength,
                version: entity_req.version,
                metadata: entity_req.metadata,
            };
            stored_entities.insert(entity_req.name, entity);
            ids.push(id);
        }

        Ok(ids)
    }

    // Implement other MemoryBackend methods...
}
```

### Testing Patterns

```rust
#[tokio::test]
async fn test_agent_learns_from_successful_interaction() {
    let mock_memory = Arc::new(MockMemoryBackend::new());
    let agent = TestAgent::new(mock_memory.clone());

    // Setup: Agent encounters a problem and finds solution
    let problem = "How to optimize database queries";
    let solution = "Use indexes and query optimization";

    // Execute: Agent processes successful interaction
    agent.handle_query_with_solution(problem, solution).await.unwrap();

    // Verify: Knowledge was stored
    assert!(mock_memory.verify_entity_stored("solution_pattern_db_optimization").await);

    // Setup: Similar problem occurs
    mock_memory.setup_search_response(
        "database optimization patterns",
        vec![SearchResult {
            entity: Entity {
                name: "solution_pattern_db_optimization".to_string(),
                entity_type: "solution_pattern".to_string(),
                observations: vec!["Use indexes", "Query optimization"].into_iter().map(String::from).collect(),
                ..Default::default()
            },
            similarity: 0.9,
        }]
    ).await;

    // Execute: Agent handles similar problem
    let response = agent.handle_query("Database performance issues").await.unwrap();

    // Verify: Agent used learned knowledge
    assert!(response.contains("indexes"));
    assert!(response.contains("optimization"));
}

#[tokio::test]
async fn test_workspace_memory_collaboration() {
    let mock_memory = Arc::new(MockMemoryBackend::new());
    let research_agent = ResearchAgent::new(mock_memory.clone());
    let implementation_agent = ImplementationAgent::new(mock_memory.clone());

    // Research agent stores findings
    research_agent.store_research_finding(
        "microservices",
        vec!["Scalability benefits", "Deployment complexity", "Service communication overhead"]
    ).await.unwrap();

    // Setup search response for implementation agent
    mock_memory.setup_search_response(
        "microservices implementation patterns",
        vec![/* research findings as search results */]
    ).await;

    // Implementation agent uses research findings
    let implementation_plan = implementation_agent.create_implementation_plan("microservices").await.unwrap();

    // Verify implementation plan incorporates research findings
    assert!(implementation_plan.contains("Scalability"));
    assert!(implementation_plan.addresses("Deployment complexity"));
}
```

## Configuration Examples

### Development Environment

```yaml
# dev-memory-config.yaml
memory:
  backend: "embedded"
  embedded:
    database_path: "./dev-data/memory.db"
    model_cache_path: "./dev-data/models"
  settings:
    auto_store: true
    search_threshold: 0.6  # Lower threshold for exploration
    max_results: 15
    retention_days: 30     # Shorter retention in dev
    auto_cleanup: true
```

### Production Environment

```yaml
# prod-memory-config.yaml
memory:
  backend: "embedded"  # or external for large scale
  embedded:
    database_path: "/var/lib/caxton/memory/memory.db"
    model_cache_path: "/var/lib/caxton/models"
    max_connections: 20
    cache_size_mb: 128
  settings:
    auto_store: true
    search_threshold: 0.75  # Higher threshold for production
    max_results: 10
    retention_days: 730     # 2 years
    auto_cleanup: true
    content_filtering:
      exclude_patterns: ["password", "secret", "key", "token"]
      pii_detection: true
```

## Related Documentation

- [Memory System Overview](/docs/memory-system/overview.md)
- [Embedded Backend Guide](/docs/memory-system/embedded-backend.md)
- [Migration Guide](/docs/memory-system/migration.md)
- [ADR-0030: Embedded Memory System](/docs/_adrs/0030-embedded-memory-system.md)
- [Configuration-Driven Agents](/docs/_adrs/0028-configuration-driven-agent-architecture.md)
