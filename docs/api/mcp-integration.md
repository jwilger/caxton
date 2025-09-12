---
title: "MCP Integration Guide"
date: 2025-09-10
layout: page
categories: [API]
---

This guide documents MCP (Model Context Protocol) integration patterns and
specifications for Caxton, including the standard StateTool interface for
agent state persistence.

## Overview

Caxton integrates with MCP tools to provide extended capabilities while
maintaining its minimal core philosophy. By delegating state management and
external integrations to business-provided MCP tools, Caxton maintains clean
architectural boundaries.

## Core Principle

**Agent state is a business domain concern, not Caxton's responsibility.**

Caxton provides message routing and coordination. If agents need persistent
state, the business domain provides appropriate MCP tools implementing
standardized interfaces.

## MCP StateTool Interface Specification

### Rust Trait

```rust
use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;

/// Standard interface for MCP state persistence tools
#[async_trait]
pub trait McpStateTool: Send + Sync {
    /// Store a value with the given key
    async fn store(&self, key: String, value: Value) -> Result<(), Box<dyn Error>>;

    /// Retrieve a value by key
    async fn retrieve(&self, key: String) -> Result<Option<Value>, Box<dyn Error>>;

    /// Delete a value by key
    async fn delete(&self, key: String) -> Result<(), Box<dyn Error>>;

    /// List all keys matching a prefix
    async fn list(&self, prefix: String) -> Result<Vec<String>, Box<dyn Error>>;

    /// Check if a key exists
    async fn exists(&self, key: String) -> Result<bool, Box<dyn Error>> {
        Ok(self.retrieve(key).await?.is_some())
    }

    /// Store multiple key-value pairs atomically (optional)
    async fn batch_store(&self, items: Vec<(String, Value)>) -> Result<(), Box<dyn Error>> {
        for (key, value) in items {
            self.store(key, value).await?;
        }
        Ok(())
    }

    /// Retrieve multiple values by keys (optional)
    async fn batch_retrieve(&self, keys: Vec<String>) -> Result<Vec<Option<Value>>, Box<dyn Error>> {
        let mut results = Vec::new();
        for key in keys {
            results.push(self.retrieve(key).await?);
        }
        Ok(results)
    }
}
```

### TypeScript/JavaScript Interface

```typescript
/**
 * MCP StateTool interface for JavaScript/TypeScript agents
 */
export interface McpStateTool {
    /**
     * Store a value with the given key
     */
    store(key: string, value: any): Promise<void>;

    /**
     * Retrieve a value by key
     */
    retrieve(key: string): Promise<any | null>;

    /**
     * Delete a value by key
     */
    delete(key: string): Promise<void>;

    /**
     * List all keys matching a prefix
     */
    list(prefix: string): Promise<string[]>;

    /**
     * Check if a key exists
     */
    exists?(key: string): Promise<boolean>;

    /**
     * Store multiple key-value pairs atomically
     */
    batchStore?(items: Array<[string, any]>): Promise<void>;

    /**
     * Retrieve multiple values by keys
     */
    batchRetrieve?(keys: string[]): Promise<Array<any | null>>;
}
```

## Implementation Examples

### Redis Implementation

```rust
use redis::{Client, AsyncCommands};

pub struct RedisStateTool {
    client: Client,
    prefix: String,
}

#[async_trait]
impl McpStateTool for RedisStateTool {
    async fn store(&self, key: String, value: Value) -> Result<(), Box<dyn Error>> {
        let mut conn = self.client.get_async_connection().await?;
        let full_key = format!("{}{}", self.prefix, key);
        let serialized = serde_json::to_string(&value)?;
        conn.set(full_key, serialized).await?;
        Ok(())
    }

    async fn retrieve(&self, key: String) -> Result<Option<Value>, Box<dyn Error>> {
        let mut conn = self.client.get_async_connection().await?;
        let full_key = format!("{}{}", self.prefix, key);
        let data: Option<String> = conn.get(full_key).await?;
        match data {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, key: String) -> Result<(), Box<dyn Error>> {
        let mut conn = self.client.get_async_connection().await?;
        let full_key = format!("{}{}", self.prefix, key);
        conn.del(full_key).await?;
        Ok(())
    }

    async fn list(&self, prefix: String) -> Result<Vec<String>, Box<dyn Error>> {
        let mut conn = self.client.get_async_connection().await?;
        let pattern = format!("{}{}*", self.prefix, prefix);
        let keys: Vec<String> = conn.keys(pattern).await?;
        Ok(keys.into_iter()
            .map(|k| k.strip_prefix(&self.prefix).unwrap_or(&k).to_string())
            .collect())
    }
}
```

### PostgreSQL Implementation

```rust
use sqlx::{PgPool, Row};

pub struct PostgresStateTool {
    pool: PgPool,
    table: String,
}

#[async_trait]
impl McpStateTool for PostgresStateTool {
    async fn store(&self, key: String, value: Value) -> Result<(), Box<dyn Error>> {
        sqlx::query(&format!(
            "INSERT INTO {} (key, value) VALUES ($1, $2)
             ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()",
            self.table
        ))
        .bind(&key)
        .bind(&value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn retrieve(&self, key: String) -> Result<Option<Value>, Box<dyn Error>> {
        let row = sqlx::query(&format!(
            "SELECT value FROM {} WHERE key = $1",
            self.table
        ))
        .bind(&key)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.get("value"))),
            None => Ok(None),
        }
    }

    // Additional methods...
}
```

### S3 Implementation

```rust
use aws_sdk_s3::{Client as S3Client, ByteStream};

pub struct S3StateTool {
    client: S3Client,
    bucket: String,
    prefix: String,
}

#[async_trait]
impl McpStateTool for S3StateTool {
    async fn store(&self, key: String, value: Value) -> Result<(), Box<dyn Error>> {
        let full_key = format!("{}{}", self.prefix, key);
        let body = ByteStream::from(serde_json::to_vec(&value)?);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(full_key)
            .body(body)
            .send()
            .await?;

        Ok(())
    }

    async fn retrieve(&self, key: String) -> Result<Option<Value>, Box<dyn Error>> {
        let full_key = format!("{}{}", self.prefix, key);

        match self.client
            .get_object()
            .bucket(&self.bucket)
            .key(full_key)
            .send()
            .await
        {
            Ok(output) => {
                let data = output.body.collect().await?.into_bytes();
                Ok(Some(serde_json::from_slice(&data)?))
            }
            Err(_) => Ok(None),
        }
    }

    // Additional methods...
}
```

### Local File System Implementation

```rust
use tokio::fs;
use std::path::PathBuf;

pub struct FileStateTool {
    base_path: PathBuf,
}

#[async_trait]
impl McpStateTool for FileStateTool {
    async fn store(&self, key: String, value: Value) -> Result<(), Box<dyn Error>> {
        let path = self.base_path.join(&key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let data = serde_json::to_string_pretty(&value)?;
        fs::write(path, data).await?;
        Ok(())
    }

    async fn retrieve(&self, key: String) -> Result<Option<Value>, Box<dyn Error>> {
        let path = self.base_path.join(&key);
        match fs::read_to_string(path).await {
            Ok(data) => Ok(Some(serde_json::from_str(&data)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    // Additional methods...
}
```

## Usage in Agents

### Basic Usage

```rust
pub struct MyAgent {
    id: AgentId,
    state_tool: Box<dyn McpStateTool>,
}

impl MyAgent {
    pub async fn save_checkpoint(&self) -> Result<()> {
        let state = json!({
            "id": self.id,
            "timestamp": chrono::Utc::now(),
            "data": self.get_internal_state(),
        });

        self.state_tool.store(
            format!("checkpoints/{}/latest", self.id),
            state
        ).await
    }

    pub async fn restore_checkpoint(&mut self) -> Result<()> {
        if let Some(state) = self.state_tool.retrieve(
            format!("checkpoints/{}/latest", self.id)
        ).await? {
            self.restore_internal_state(state)?;
        }
        Ok(())
    }
}
```

### Advanced Patterns

#### Partition-Aware State Management

```rust
pub struct PartitionAwareAgent {
    id: AgentId,
    state_tool: Box<dyn McpStateTool>,
    partition_detector: PartitionDetector,
}

impl PartitionAwareAgent {
    pub async fn save_with_partition_check(&self) -> Result<()> {
        // Check partition state before saving
        match self.partition_detector.current_state() {
            PartitionState::Healthy | PartitionState::Majority => {
                // Normal save operation
                self.state_tool.store(
                    format!("states/{}", self.id),
                    self.serialize()?
                ).await
            }
            PartitionState::Minority => {
                // Queue for later or save locally
                self.queue_state_update().await
            }
            PartitionState::Isolated => {
                // Local-only save
                self.save_to_local_cache().await
            }
        }
    }

    pub async fn restore_with_conflict_resolution(&mut self) -> Result<()> {
        // Try primary state source
        if let Some(state) = self.state_tool.retrieve(
            format!("states/{}", self.id)
        ).await? {
            // Check for conflicts after partition healing
            if self.detect_conflict(&state)? {
                self.resolve_conflict(state).await?;
            } else {
                self.restore_internal_state(state)?;
            }
        }
        Ok(())
    }
}
```

#### Versioned State

```rust
impl MyAgent {
    pub async fn save_versioned(&self, version: u64) -> Result<()> {
        let key = format!("states/{}/v{}", self.id, version);
        self.state_tool.store(key, self.serialize()?).await
    }

    pub async fn list_versions(&self) -> Result<Vec<u64>> {
        let keys = self.state_tool.list(
            format!("states/{}/v", self.id)
        ).await?;

        let mut versions = Vec::new();
        for key in keys {
            if let Some(v) = key.strip_prefix(&format!("states/{}/v", self.id)) {
                if let Ok(version) = v.parse::<u64>() {
                    versions.push(version);
                }
            }
        }
        versions.sort();
        Ok(versions)
    }
}
```

#### Transaction Log

```rust
impl MyAgent {
    pub async fn append_to_log(&self, event: Event) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_nanos();
        let key = format!("logs/{}/{}", self.id, timestamp);
        self.state_tool.store(key, serde_json::to_value(event)?).await
    }

    pub async fn replay_log(&self, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<Event>> {
        let prefix = format!("logs/{}/", self.id);
        let keys = self.state_tool.list(prefix).await?;

        let mut events = Vec::new();
        for key in keys {
            if let Some(ts_str) = key.strip_prefix(&format!("logs/{}/", self.id)) {
                if let Ok(ts) = ts_str.parse::<i64>() {
                    if ts >= since.timestamp_nanos() {
                        if let Some(value) = self.state_tool.retrieve(key).await? {
                            events.push(serde_json::from_value(value)?);
                        }
                    }
                }
            }
        }
        Ok(events)
    }
}
```

## Configuration

### Agent Configuration with State Tool

```yaml
# agent-config.yaml
agent:
  id: "processor-001"
  type: "data-processor"

state_tool:
  type: "redis"
  config:
    url: "redis://localhost:6379"
    prefix: "caxton:agents:"
    ttl: 3600
```

### Alternative Configurations

```yaml
# PostgreSQL backend
state_tool:
  type: "postgres"
  config:
    url: "postgresql://user:pass@localhost/caxton"
    table: "agent_states"

# S3 backend
state_tool:
  type: "s3"
  config:
    bucket: "caxton-agent-states"
    prefix: "production/"
    region: "us-west-2"

# Local filesystem (development)
state_tool:
  type: "file"
  config:
    path: "/var/lib/caxton/states"
```

## Best Practices

### 1. Key Naming Conventions

Use hierarchical keys for organization:

```text
checkpoints/{agent_id}/latest
checkpoints/{agent_id}/v{version}
logs/{agent_id}/{timestamp}
tasks/{agent_id}/{task_id}
conversations/{conversation_id}/messages/{index}
```

### 2. Error Handling

Always handle state tool failures gracefully:

```rust
match self.state_tool.retrieve(key).await {
    Ok(Some(state)) => self.restore(state),
    Ok(None) => self.initialize_default(),
    Err(e) => {
        warn!("State retrieval failed: {}", e);
        self.initialize_default()
    }
}
```

### 3. Atomic Operations

Use batch operations when available:

```rust
let updates = vec![
    (format!("state/{}", id), state),
    (format!("metadata/{}", id), metadata),
    (format!("timestamp/{}", id), json!(timestamp)),
];
self.state_tool.batch_store(updates).await?;
```

### 4. TTL and Cleanup

Implement cleanup for temporary data:

```rust
// Store with timestamp in key for natural expiration
let key = format!("temp/{}/{}",
    chrono::Utc::now().format("%Y%m%d"),
    temp_id
);
self.state_tool.store(key, data).await?;

// Periodic cleanup job
async fn cleanup_old_temp_data(&self, days: i64) -> Result<()> {
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
    let prefix = format!("temp/{}/", cutoff.format("%Y%m%d"));
    let old_keys = self.state_tool.list(prefix).await?;
    for key in old_keys {
        self.state_tool.delete(key).await?;
    }
    Ok(())
}
```

## Testing

### Mock Implementation for Tests

```rust
use std::collections::HashMap;
use parking_lot::RwLock;

pub struct MockStateTool {
    storage: Arc<RwLock<HashMap<String, Value>>>,
}

#[async_trait]
impl McpStateTool for MockStateTool {
    async fn store(&self, key: String, value: Value) -> Result<(), Box<dyn Error>> {
        self.storage.write().insert(key, value);
        Ok(())
    }

    async fn retrieve(&self, key: String) -> Result<Option<Value>, Box<dyn Error>> {
        Ok(self.storage.read().get(&key).cloned())
    }

    // Additional methods...
}
```

## Migration Guide

### From Shared State to MCP Tools

```rust
// Before: Direct database access
let pool = PgPool::connect("postgresql://...").await?;
sqlx::query("INSERT INTO agent_states...").execute(&pool).await?;

// After: MCP StateTool abstraction
let state_tool: Box<dyn McpStateTool> = create_state_tool(config)?;
state_tool.store("agent_state", state).await?;
```

## Security Considerations

1. **Encryption**: Sensitive data should be encrypted before storage
2. **Access Control**: Implement proper authentication/authorization in
   tool implementations
3. **Key Isolation**: Use prefixes to isolate different agents/tenants
4. **Audit Logging**: Log all state operations for compliance

## Conclusion

The MCP StateTool interface provides a flexible abstraction for state
persistence while maintaining Caxton's minimal core philosophy. Businesses can
choose the most appropriate storage backend for their needs without affecting
Caxton's core architecture.

## References

- [Configuration Validation](configuration-validation.md)
- [Capability Registration](capability-registration.md)
- [Memory Integration](memory-integration.md)
- [Performance Specifications](performance-specifications.md)
