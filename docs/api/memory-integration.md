---
title: "Memory System Integration API"
date: 2025-09-10
layout: page
categories: [api, memory]
---

## Overview

The Memory System Integration API provides agents with persistent memory
capabilities through an embedded SQLite + Candle backend (ADR-0030). Agents
can store entities, create relationships, and perform semantic search to
provide context-aware responses and learn from interactions over time.

**Architecture**: Hybrid memory system with embedded default (SQLite +
All-MiniLM-L6-v2) for zero-configuration deployment and optional external
backends (Neo4j, Qdrant) for scale.

**Memory Model**: Entity-relationship storage with semantic embeddings,
temporal tracking, and configurable scopes (agent-only, workspace, global).

## Core Concepts

### Entities

**Entity**: A named knowledge item with typed observations and temporal validity

- **Name**: Unique identifier within scope (e.g., "Customer_ABC",
  "Sales_Q3_2024")
- **Type**: Classification of entity (e.g., "customer", "product", "report")
- **Observations**: Array of text content associated with the entity
- **Temporal tracking**: Created/updated timestamps and validity periods

### Relations

**Relation**: Typed connection between entities with strength and confidence
scores

- **From/To**: Source and target entity names
- **Type**: Relationship classification (e.g., "purchased", "analyzed",
  "contains")
- **Strength**: Numerical strength of relationship (0.0 to 1.0)
- **Confidence**: Certainty in relationship accuracy (0.0 to 1.0)

### Memory Scopes

- **Agent-only**: Private memory per agent instance
- **Workspace**: Shared memory within workspace or project
- **Global**: System-wide shared knowledge base

### Semantic Search

Vector-based similarity search using All-MiniLM-L6-v2 embeddings (384
dimensions) for retrieving relevant context based on semantic meaning rather
than exact keyword matching.

## API Endpoints

### Store Entity

**POST** `/api/v1/memory/entities`

Store or update an entity in agent memory.

#### Request Body

```json
{
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "entity_name": "Sales_Report_Q3_2024",
  "entity_type": "report",
  "observations": [
    "Q3 2024 sales increased 15% compared to Q3 2023",
    "Top performing product category was electronics with $2.3M revenue",
    "Customer satisfaction scores averaged 4.2/5.0 across all regions"
  ],
  "memory_scope": "workspace",
  "metadata": {
    "quarter": "Q3",
    "year": 2024,
    "report_type": "sales_summary",
    "author": "DataAnalyzer",
    "confidence": 0.95
  },
  "valid_from": "2024-07-01T00:00:00Z",
  "valid_to": "2024-12-31T23:59:59Z"
}
```

#### Request Fields

- `agent_id` (string, required): Agent storing the entity
- `entity_name` (string, required): Unique entity identifier within scope
- `entity_type` (string, required): Entity classification
- `observations` (array, required): Array of text observations about the entity
- `memory_scope` (string, optional): Memory sharing scope (default: agent
  scope from config)
- `metadata` (object, optional): Structured metadata for filtering and search
- `valid_from` (string, optional): ISO 8601 validity start timestamp
- `valid_to` (string, optional): ISO 8601 validity end timestamp

#### Response (201 Created)

```json
{
  "entity_id": "ent_abc123def456",
  "entity_name": "Sales_Report_Q3_2024",
  "entity_type": "report",
  "observation_count": 3,
  "memory_scope": "workspace",
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2025-09-10T14:30:00Z",
  "updated_at": "2025-09-10T14:30:00Z",
  "embedding_generated": true,
  "version": 1
}
```

#### Error Responses

- **400 Bad Request**: Invalid entity data or missing required fields
- **404 Not Found**: Agent does not exist
- **409 Conflict**: Entity name already exists in scope with different type
- **413 Payload Too Large**: Observations exceed size limit (10MB total)

### Semantic Search

**GET** `/api/v1/memory/search`

Perform semantic search across entities based on meaning and context.

#### Query Parameters

- `agent_id` (string, required): Agent performing the search
- `query` (string, required): Search query text
- `memory_scope` (string, optional): Scope to search within (default:
  agent's default scope)
- `entity_types` (string, optional): Comma-separated entity types to include
- `limit` (integer, optional): Maximum results (default: 10, max: 100)
- `min_similarity` (number, optional): Minimum similarity threshold 0.0-1.0
  (default: 0.6)
- `include_relations` (boolean, optional): Include related entities
  (default: false)
- `temporal_filter` (string, optional): Temporal constraint (`current`,
  `all`, `date_range`)
- `start_date` (string, optional): Start date for date range filter (ISO 8601)
- `end_date` (string, optional): End date for date range filter (ISO 8601)

#### Response (200 OK)

```json
{
  "query": "sales performance Q3 2024",
  "results": [
    {
      "entity_id": "ent_abc123def456",
      "entity_name": "Sales_Report_Q3_2024",
      "entity_type": "report",
      "similarity_score": 0.89,
      "observations": [
        "Q3 2024 sales increased 15% compared to Q3 2023",
        "Top performing product category was electronics with $2.3M revenue"
      ],
      "metadata": {
        "quarter": "Q3",
        "year": 2024,
        "report_type": "sales_summary"
      },
      "created_at": "2025-09-10T14:30:00Z",
      "agent_id": "config-550e8400-e29b-41d4-a716-446655440000"
    },
    {
      "entity_id": "ent_def789ghi012",
      "entity_name": "Electronics_Category_Analysis",
      "entity_type": "analysis",
      "similarity_score": 0.76,
      "observations": [
        "Electronics category drove 40% of Q3 growth",
        "Smart phone sales particularly strong in September"
      ],
      "metadata": {
        "category": "electronics",
        "analysis_type": "category_performance"
      },
      "created_at": "2025-09-10T12:15:00Z",
      "agent_id": "config-550e8400-e29b-41d4-a716-446655440000"
    }
  ],
  "total_results": 2,
  "search_time_ms": 45,
  "memory_scope": "workspace"
}
```

#### With Relations (include_relations=true)

```json
{
  "query": "sales performance Q3 2024",
  "results": [
    {
      "entity_id": "ent_abc123def456",
      "entity_name": "Sales_Report_Q3_2024",
      "entity_type": "report",
      "similarity_score": 0.89,
      "observations": ["..."],
      "relations": [
        {
          "relation_type": "analyzed_by",
          "to_entity": "DataAnalyzer_Agent",
          "strength": 1.0,
          "confidence": 0.95
        },
        {
          "relation_type": "contains",
          "to_entity": "Electronics_Category_Analysis",
          "strength": 0.8,
          "confidence": 0.9
        }
      ]
    }
  ]
}
```

### Create Entity Relationship

**POST** `/api/v1/memory/relations`

Create a typed relationship between two entities.

#### Request Body

```json
{
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "from_entity": "Sales_Report_Q3_2024",
  "to_entity": "Electronics_Category_Analysis",
  "relation_type": "contains",
  "strength": 0.8,
  "confidence": 0.9,
  "memory_scope": "workspace",
  "metadata": {
    "created_by": "DataAnalyzer",
    "analysis_date": "2025-09-10",
    "relationship_basis": "report_section_reference"
  },
  "bidirectional": false
}
```

#### Request Fields

- `agent_id` (string, required): Agent creating the relationship
- `from_entity` (string, required): Source entity name
- `to_entity` (string, required): Target entity name
- `relation_type` (string, required): Relationship classification
- `strength` (number, optional): Relationship strength 0.0-1.0 (default: 0.5)
- `confidence` (number, optional): Confidence in relationship 0.0-1.0
  (default: 1.0)
- `memory_scope` (string, optional): Memory scope for relationship
- `metadata` (object, optional): Additional relationship metadata
- `bidirectional` (boolean, optional): Create reverse relationship
  (default: false)

#### Response (201 Created)

```json
{
  "relation_id": "rel_ghi789jkl012",
  "from_entity": "Sales_Report_Q3_2024",
  "to_entity": "Electronics_Category_Analysis",
  "relation_type": "contains",
  "strength": 0.8,
  "confidence": 0.9,
  "memory_scope": "workspace",
  "created_at": "2025-09-10T15:30:00Z",
  "created_by": "config-550e8400-e29b-41d4-a716-446655440000",
  "bidirectional": false
}
```

#### Error Responses

- **400 Bad Request**: Invalid relationship data or entity names
- **404 Not Found**: One or both entities do not exist
- **409 Conflict**: Relationship already exists with different properties

### Traverse Memory Graph

**GET** `/api/v1/memory/graph/{entity_name}`

Traverse the memory graph starting from a specific entity.

#### Path Parameters

- `entity_name` (string, required): Starting entity for graph traversal

#### Query Parameters

- `agent_id` (string, required): Agent performing traversal
- `memory_scope` (string, optional): Memory scope to traverse
- `max_depth` (integer, optional): Maximum traversal depth (default: 3, max: 10)
- `relation_types` (string, optional): Comma-separated relation types to follow
- `min_strength` (number, optional): Minimum relationship strength to
  follow (default: 0.0)
- `include_observations` (boolean, optional): Include entity observations
  (default: true)
- `traversal_strategy` (string, optional): `breadth_first` or `depth_first`
  (default: breadth_first)

#### Response (200 OK)

```json
{
  "root_entity": "Sales_Report_Q3_2024",
  "graph": {
    "nodes": [
      {
        "entity_name": "Sales_Report_Q3_2024",
        "entity_type": "report",
        "depth": 0,
        "observations": ["Q3 2024 sales increased 15% compared to Q3 2023"]
      },
      {
        "entity_name": "Electronics_Category_Analysis",
        "entity_type": "analysis",
        "depth": 1,
        "observations": ["Electronics category drove 40% of Q3 growth"]
      },
      {
        "entity_name": "Customer_Feedback_Q3",
        "entity_type": "feedback",
        "depth": 2,
        "observations": [
          "Customer satisfaction with electronics improved significantly"
        ]
      }
    ],
    "edges": [
      {
        "from": "Sales_Report_Q3_2024",
        "to": "Electronics_Category_Analysis",
        "relation_type": "contains",
        "strength": 0.8,
        "confidence": 0.9
      },
      {
        "from": "Electronics_Category_Analysis",
        "to": "Customer_Feedback_Q3",
        "relation_type": "supported_by",
        "strength": 0.7,
        "confidence": 0.85
      }
    ]
  },
  "traversal_stats": {
    "max_depth": 3,
    "nodes_found": 3,
    "edges_found": 2,
    "traversal_time_ms": 25
  }
}
```

### Clear Agent Memory

**DELETE** `/api/v1/memory/{agent_id}`

Clear all memory for a specific agent.

#### Path Parameters

- `agent_id` (string, required): Agent whose memory to clear

#### Query Parameters

- `memory_scope` (string, optional): Specific scope to clear (default: all
  scopes for agent)
- `entity_types` (string, optional): Comma-separated entity types to remove
- `confirm` (boolean, required): Confirmation flag must be true

#### Response (200 OK)

```json
{
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "memory_scope": "workspace",
  "entities_deleted": 15,
  "relations_deleted": 23,
  "cleared_at": "2025-09-10T16:45:00Z"
}
```

#### Error Responses

- **400 Bad Request**: Missing confirmation or invalid parameters
- **404 Not Found**: Agent does not exist

## Advanced Memory Operations

### Batch Entity Storage

**POST** `/api/v1/memory/entities/batch`

Store multiple entities in a single operation for improved performance.

#### Request Body

```json
{
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "memory_scope": "workspace",
  "entities": [
    {
      "entity_name": "Product_A",
      "entity_type": "product",
      "observations": ["High-performance widget with 5-year warranty"]
    },
    {
      "entity_name": "Product_B",
      "entity_type": "product",
      "observations": ["Budget-friendly option popular with students"]
    }
  ]
}
```

#### Response (201 Created)

```json
{
  "batch_id": "batch_xyz789abc123",
  "entities_created": 2,
  "entities_updated": 0,
  "failed_entities": 0,
  "processing_time_ms": 150,
  "results": [
    {
      "entity_name": "Product_A",
      "status": "created",
      "entity_id": "ent_pqr456stu789"
    },
    {
      "entity_name": "Product_B",
      "status": "created",
      "entity_id": "ent_vwx012yzb345"
    }
  ]
}
```

### Memory Export

**GET** `/api/v1/memory/export`

Export agent memory for backup or migration.

#### Query Parameters

- `agent_id` (string, required): Agent whose memory to export
- `memory_scope` (string, optional): Specific scope to export
- `format` (string, optional): Export format (`json`, `jsonl`) (default: json)
- `include_embeddings` (boolean, optional): Include vector embeddings
  (default: false)

#### Response (200 OK)

```json
{
  "export_metadata": {
    "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
    "memory_scope": "workspace",
    "exported_at": "2025-09-10T17:00:00Z",
    "entity_count": 15,
    "relation_count": 23,
    "format": "json"
  },
  "entities": [
    {
      "entity_name": "Sales_Report_Q3_2024",
      "entity_type": "report",
      "observations": ["..."],
      "metadata": {"..."},
      "created_at": "2025-09-10T14:30:00Z",
      "embedding": [0.123, -0.456, 0.789] // only if include_embeddings=true
    }
  ],
  "relations": [
    {
      "from_entity": "Sales_Report_Q3_2024",
      "to_entity": "Electronics_Category_Analysis",
      "relation_type": "contains",
      "strength": 0.8,
      "confidence": 0.9,
      "created_at": "2025-09-10T15:30:00Z"
    }
  ]
}
```

### Memory Import

**POST** `/api/v1/memory/import`

Import memory data from export or external source.

#### Request Body

```json
{
  "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
  "memory_scope": "workspace",
  "merge_strategy": "update_existing",
  "data": {
    "entities": [...],
    "relations": [...]
  }
}
```

#### Request Fields

- `agent_id` (string, required): Target agent for import
- `memory_scope` (string, required): Target memory scope
- `merge_strategy` (string, optional): How to handle conflicts
  (`update_existing`, `skip_existing`, `error_on_conflict`)
- `data` (object, required): Memory data in export format

#### Response (201 Created)

```json
{
  "import_id": "import_mno345pqr678",
  "entities_imported": 12,
  "entities_updated": 3,
  "entities_skipped": 0,
  "relations_imported": 18,
  "relations_updated": 5,
  "processing_time_ms": 350,
  "conflicts_resolved": 8
}
```

## Memory Analytics

### Memory Statistics

**GET** `/api/v1/memory/stats`

Get memory usage and performance statistics.

#### Query Parameters

- `agent_id` (string, optional): Filter by specific agent
- `memory_scope` (string, optional): Filter by memory scope

#### Response (200 OK)

```json
{
  "global_stats": {
    "total_entities": 1247,
    "total_relations": 2843,
    "total_agents": 15,
    "storage_size_mb": 45.7,
    "embedding_model": "all-MiniLM-L6-v2",
    "embedding_dimensions": 384
  },
  "scope_breakdown": [
    {
      "memory_scope": "workspace",
      "entity_count": 856,
      "relation_count": 1923,
      "active_agents": 8
    },
    {
      "memory_scope": "global",
      "entity_count": 391,
      "relation_count": 920,
      "active_agents": 15
    }
  ],
  "performance_metrics": {
    "avg_search_time_ms": 42,
    "avg_storage_time_ms": 18,
    "cache_hit_rate": 0.76,
    "embeddings_per_second": 950
  }
}
```

### Memory Health Check

**GET** `/api/v1/memory/health`

Check memory system health and consistency.

#### Response (200 OK)

```json
{
  "status": "healthy",
  "checks": [
    {
      "check": "database_connectivity",
      "status": "pass",
      "response_time_ms": 5
    },
    {
      "check": "embedding_model_loaded",
      "status": "pass",
      "model": "all-MiniLM-L6-v2",
      "memory_usage_mb": 23.4
    },
    {
      "check": "index_integrity",
      "status": "pass",
      "indexed_entities": 1247,
      "index_size_mb": 12.8
    },
    {
      "check": "orphaned_relations",
      "status": "warning",
      "orphaned_count": 3,
      "auto_cleanup": "scheduled"
    }
  ],
  "last_check": "2025-09-10T17:15:00Z",
  "next_check": "2025-09-10T18:15:00Z"
}
```

## Memory Configuration

### Get Memory Configuration

**GET** `/api/v1/memory/config`

Retrieve current memory system configuration.

#### Response (200 OK)

```json
{
  "backend": "embedded_sqlite",
  "embedding_model": "all-MiniLM-L6-v2",
  "embedding_dimensions": 384,
  "default_memory_scope": "workspace",
  "retention_policy": {
    "max_entities_per_agent": 10000,
    "max_age_days": 365,
    "cleanup_frequency": "daily"
  },
  "performance_settings": {
    "batch_size": 100,
    "max_concurrent_searches": 10,
    "cache_size_mb": 50
  },
  "external_backends": {
    "neo4j": {
      "enabled": false,
      "connection_url": null
    },
    "qdrant": {
      "enabled": false,
      "connection_url": null
    }
  }
}
```

### Update Memory Configuration

**PUT** `/api/v1/memory/config`

Update memory system configuration settings.

#### Request Body

```json
{
  "retention_policy": {
    "max_entities_per_agent": 15000,
    "max_age_days": 730
  },
  "performance_settings": {
    "cache_size_mb": 100,
    "max_concurrent_searches": 20
  }
}
```

#### Response (200 OK)

```json
{
  "updated": true,
  "changes": [
    "retention_policy.max_entities_per_agent",
    "retention_policy.max_age_days",
    "performance_settings.cache_size_mb",
    "performance_settings.max_concurrent_searches"
  ],
  "restart_required": false,
  "applied_at": "2025-09-10T17:30:00Z"
}
```

## WebSocket Integration

Real-time memory events:

**Connection**: `ws://localhost:8080/ws/memory`

**Event Types**:

```json
{
  "type": "entity_created",
  "data": {
    "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
    "entity_name": "New_Customer_Analysis",
    "entity_type": "analysis",
    "memory_scope": "workspace"
  },
  "timestamp": "2025-09-10T17:45:00Z"
}
```

```json
{
  "type": "search_performed",
  "data": {
    "agent_id": "config-550e8400-e29b-41d4-a716-446655440000",
    "query": "customer satisfaction trends",
    "results_count": 5,
    "search_time_ms": 38
  },
  "timestamp": "2025-09-10T17:45:00Z"
}
```

**Event Types**: `entity_created`, `entity_updated`, `relation_created`,
`search_performed`, `memory_cleared`, `import_completed`

## Domain Types

### MemoryEntity

```typescript
interface MemoryEntity {
  entity_id: string; // Unique entity identifier
  entity_name: string; // Entity name within scope
  entity_type: string; // Entity classification
  observations: string[]; // Text observations about entity
  metadata?: Record<string, any>; // Structured metadata
  memory_scope: MemoryScope; // Memory sharing scope
  agent_id: string; // Agent that created entity
  created_at: string; // ISO 8601 creation timestamp
  updated_at: string; // ISO 8601 last update timestamp
  valid_from?: string; // ISO 8601 validity start
  valid_to?: string; // ISO 8601 validity end
  version: number; // Entity version number
  embedding?: number[]; // Vector embedding (if requested)
}
```

### MemoryRelation

```typescript
interface MemoryRelation {
  relation_id: string; // Unique relation identifier
  from_entity: string; // Source entity name
  to_entity: string; // Target entity name
  relation_type: string; // Relationship classification
  strength: number; // Relationship strength (0.0-1.0)
  confidence: number; // Confidence in relationship (0.0-1.0)
  metadata?: Record<string, any>; // Additional relation metadata
  memory_scope: MemoryScope; // Memory sharing scope
  created_by: string; // Agent that created relation
  created_at: string; // ISO 8601 creation timestamp
  updated_at: string; // ISO 8601 last update timestamp
  bidirectional: boolean; // Whether relation works both ways
}
```

### SearchResult

```typescript
interface SearchResult {
  entity_id: string; // Entity identifier
  entity_name: string; // Entity name
  entity_type: string; // Entity classification
  similarity_score: number; // Semantic similarity (0.0-1.0)
  observations: string[]; // Matching observations
  metadata?: Record<string, any>; // Entity metadata
  relations?: RelationInfo[]; // Related entities (if requested)
  created_at: string; // Entity creation timestamp
  agent_id: string; // Entity creator
}
```

### MemoryScope

Memory sharing configuration:

- `agent-only` - Private memory per agent instance
- `workspace` - Shared memory within workspace
- `global` - System-wide shared knowledge base

## Performance Characteristics

### Embedded Backend (SQLite + Candle)

- **Entity storage**: 5-20ms per entity
- **Semantic search**: 10-50ms for 100K entities
- **Graph traversal**: 5-20ms for typical queries
- **Memory usage**: ~200MB baseline + 2.5KB per entity
- **Concurrent operations**: 10-20 simultaneous searches
- **Effective scale**: Up to 100K entities, 1M relations

### Scaling Guidelines

**Embedded Backend Suitable For**:

- Development and testing environments
- Small to medium deployments (< 100K entities)
- Single-node Caxton instances
- Zero-configuration requirements

**External Backend Recommended For**:

- Production environments with > 1M entities
- Multi-node Caxton clusters
- High-concurrent search workloads (> 100 searches/sec)
- Advanced graph analytics requirements

## Integration Examples

### Configuration Agent with Memory

```yaml
---
name: CustomerInsightAnalyzer
memory_enabled: true
memory_scope: "workspace"
capabilities:
  - customer-analysis
  - insight-generation
---
# Customer Insight Analyzer

I analyze customer data and maintain insights over time.
```

**Automatic Memory Usage**: Agent automatically stores analysis results and
retrieves relevant historical insights for context.

### Manual Memory Operations

```javascript
// Store analysis results in memory
const response = await fetch("/api/v1/memory/entities", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    agent_id: "config-customer-analyzer",
    entity_name: "Customer_Segment_Analysis_Q3",
    entity_type: "analysis",
    observations: [
      "Premium customers show 23% higher retention than standard tier",
      "Mobile app usage correlates strongly with customer satisfaction",
      "Support ticket volume decreased 15% after onboarding improvements",
    ],
    memory_scope: "workspace",
  }),
});

// Search for relevant context before responding
const searchResponse = await fetch(
  `/api/v1/memory/search?agent_id=config-customer-analyzer&query=${encodeURIComponent(
    "customer retention mobile app",
  )}&limit=5`,
);
const { results } = await searchResponse.json();
```

### Relationship Building

```javascript
// Create relationship between analysis and supporting data
await fetch("/api/v1/memory/relations", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    agent_id: "config-customer-analyzer",
    from_entity: "Customer_Segment_Analysis_Q3",
    to_entity: "Mobile_App_Usage_Data",
    relation_type: "supported_by",
    strength: 0.9,
    confidence: 0.95,
    memory_scope: "workspace",
  }),
});
```

## Error Handling

### Common Error Responses

```json
{
  "error": "Entity storage failed",
  "code": "STORAGE_ERROR",
  "details": {
    "entity_name": "Sales_Report_Q3_2024",
    "reason": "Observation content exceeds 1MB limit",
    "max_size_mb": 1
  },
  "timestamp": "2025-09-10T17:45:00Z",
  "request_id": "req_mem_abc123"
}
```

### Error Codes

- `ENTITY_NOT_FOUND` - Referenced entity does not exist
- `STORAGE_ERROR` - Failed to store entity or relation
- `SEARCH_ERROR` - Semantic search operation failed
- `EMBEDDING_ERROR` - Vector embedding generation failed
- `SCOPE_PERMISSION_DENIED` - Insufficient access to memory scope
- `MEMORY_LIMIT_EXCEEDED` - Agent exceeded memory allocation
- `INVALID_ENTITY_DATA` - Entity data validation failed
- `GRAPH_TRAVERSAL_ERROR` - Graph traversal operation failed

## Migration and Backup

### Database Migration

When migrating from embedded to external backends:

1. **Export current data**: Use `/api/v1/memory/export`
2. **Configure external backend**: Update `caxton.yaml`
3. **Import data**: Use `/api/v1/memory/import`
4. **Verify integrity**: Run health checks
5. **Update agent configurations**: Point to new backend

### Backup Strategy

- **Automated exports**: Daily JSON exports to external storage
- **Incremental backups**: Export only entities modified since last backup
- **Cross-backend replication**: Sync between embedded and external for
  redundancy
- **Point-in-time recovery**: Restore memory state to specific timestamp

## Related Documentation

- [Configuration Agent API](config-agents.md) - Agents that use memory
  automatically
- [Capability Registration API](capability-registration.md) - Store
  capability usage patterns
- [Agent Messaging API](fipa-messaging.md) - Store conversation context
- [ADR-0030](../adrs/0030-embedded-memory-system.md) - Memory system
  architecture decision
- [ADR-0013](../adrs/0013-state-management-architecture.md) - Agent state
  management approach
