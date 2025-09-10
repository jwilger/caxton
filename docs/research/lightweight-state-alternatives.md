---
title: "Lightweight State Management Alternatives"
date: 2025-01-15
layout: page
categories: [Research]
---


## Executive Summary

Based on extensive research, Caxton should adopt a **coordination-only
approach** rather than shared state, with agent state management delegated to
business domains via MCP tools. This aligns with the minimal core philosophy and
significantly reduces operational complexity.

## Key Recommendation: Coordination Over Shared State

### Why Caxton Doesn't Need Shared State

After analyzing Caxton's actual requirements, most "state" needs are actually
**coordination concerns**:

1. **Agent Registry**: Which agents are available and their capabilities
2. **Routing Information**: How to reach specific agents
3. **Health Status**: Liveness and readiness of agents
4. **Message Correlation**: Tracking conversation contexts

These can be managed through **gossip protocols** and **eventual consistency**
rather than strongly consistent shared state.

## Proposed Architecture: Hybrid Coordination Model

### 1. Embedded SQLite for Local State

Each Caxton instance maintains its own local state using **embedded SQLite**:

- Zero external dependencies
- Excellent performance for local queries
- Mature, battle-tested technology
- Small footprint (~500KB)

```rust
// Local state storage per instance
pub struct LocalState {
    db: rusqlite::Connection,
}

impl LocalState {
    pub fn new() -> Result<Self> {
        let db = Connection::open("caxton_local.db")?;
        // Store instance-specific data
        Ok(Self { db })
    }
}
```

### 2. SWIM Protocol for Cluster Coordination

Use the **SWIM protocol** for lightweight cluster coordination:

- No shared state required
- Scales to thousands of nodes
- Failure detection built-in
- Eventually consistent membership

```rust
// Using memberlist (Rust SWIM implementation)
use memberlist::{Config, Memberlist};

pub struct ClusterCoordinator {
    memberlist: Memberlist,
    local_registry: HashMap<AgentId, AgentInfo>,
}

impl ClusterCoordinator {
    pub async fn join_cluster(&mut self, seeds: Vec<String>) -> Result<()> {
        self.memberlist.join(seeds).await?;
        // Gossip local agent registry to peers
        self.broadcast_local_agents().await
    }
}
```

### 3. Agent State as Business Domain Concern

**Critical Insight**: Agent state should NOT be Caxton's responsibility.

#### Current Problem with ADR-0013

The proposed PostgreSQL-based state management violates the minimal core
philosophy by making Caxton responsible for:

- Agent checkpointing
- State recovery
- Event sourcing
- Snapshot management

#### Proposed Solution: MCP State Tools

Agents requiring state persistence should use MCP tools provided by the business
domain:

```rust
// Example: Agent uses MCP tool for state
pub struct StatefulAgent {
    state_tool: Box<dyn McpStateTool>,
}

impl StatefulAgent {
    pub async fn save_state(&self, key: &str, value: Value) -> Result<()> {
        // Delegate to business-provided MCP tool
        self.state_tool.store(key, value).await
    }

    pub async fn load_state(&self, key: &str) -> Result<Value> {
        // Business decides storage backend
        self.state_tool.retrieve(key).await
    }
}
```

This allows businesses to choose their own state backends:

- Redis for caching
- PostgreSQL for transactions
- S3 for blob storage
- DynamoDB for serverless

## Lightweight Storage Options Comparison

### For Caxton's Internal Needs Only

| Solution | Pros | Cons | Use Case | |----------|------|------|----------| |
**SQLite** | Zero deps, mature, SQL support | Single-writer limitation | ✅ Local
instance state | | **sled** | Pure Rust, lock-free | Unstable, space inefficient
| ❌ Too immature | | **RocksDB** | High performance, LSM-tree | C++ dependency,
complex | ⚠️ If performance critical | | **LMDB** | Memory-mapped, multi-process
| Read-optimized | ❌ Wrong access pattern |

### Recommendation: SQLite for Local State

- Each Caxton instance has its own SQLite database
- No coordination needed for local operations
- Gossip protocol shares necessary information

## Implementation Strategy

### Phase 1: Remove Shared State Requirements

```rust
// Before: Shared state in PostgreSQL
pub struct SharedOrchestrator {
    postgres: PostgresPool,
    // Complex event sourcing...
}

// After: Coordination-only
pub struct CoordinatedOrchestrator {
    local_db: SQLite,
    gossip: SwimProtocol,
    // No shared state!
}
```

### Phase 2: Implement SWIM Protocol

```rust
use async_std::sync::RwLock;

pub struct SwimCluster {
    members: RwLock<HashMap<NodeId, NodeInfo>>,
    failure_detector: FailureDetector,
}

impl SwimCluster {
    pub async fn detect_failures(&self) {
        // SWIM's scalable failure detection
        let target = self.select_random_member().await;
        if !self.ping(target).await {
            self.request_ping_from_others(target).await;
        }
    }
}
```

### Phase 3: MCP State Tool Specification

```rust
// Standard interface for state persistence
#[async_trait]
pub trait McpStateTool: Send + Sync {
    async fn store(&self, key: String, value: Value) -> Result<()>;
    async fn retrieve(&self, key: String) -> Result<Option<Value>>;
    async fn delete(&self, key: String) -> Result<()>;
    async fn list(&self, prefix: String) -> Result<Vec<String>>;
}

// Businesses implement their preferred backend
pub struct RedisStateTool { /* ... */ }
pub struct S3StateTool { /* ... */ }
pub struct PostgresStateTool { /* ... */ }
```

## Benefits of This Approach

### 1. Operational Simplicity

- **No PostgreSQL required**: Eliminates heavy dependency
- **No backup management**: Each instance is disposable
- **No migration complexity**: Schema-less coordination

### 2. Better Scalability

- **Linear scaling**: Add nodes without shared state bottleneck
- **Geographic distribution**: Works across regions
- **Fault isolation**: Node failures don't affect others

### 3. Alignment with Minimal Core

- **Core remains simple**: Just message routing
- **Flexibility for users**: Choose their own state backend
- **Clear boundaries**: Caxton handles coordination, not business state

### 4. Reduced Complexity

- **No event sourcing**: Eliminates complex replay logic
- **No snapshots**: No snapshot management overhead
- **No consensus**: SWIM provides eventual consistency

## Migration Path from ADR-0013

### Step 1: Redefine State Categories

```yaml
# What Caxton manages (coordination)
coordination:
  - agent_registry    # Via gossip
  - health_status    # Via SWIM
  - routing_info     # Via gossip

# What businesses manage (state)
business_state:
  - agent_checkpoints  # Via MCP tools
  - conversation_history  # Via MCP tools
  - task_state  # Via MCP tools
  - audit_logs  # Via MCP tools
```

### Step 2: Update ADR-0013

Create ADR-0014 that supersedes ADR-0013:

- Title: "Coordination-First Architecture"
- Explicitly reject shared state
- Define MCP state tool interface
- Document SWIM protocol usage

### Step 3: Implement Gradually

1. Start with SQLite for local state
2. Add SWIM for cluster membership
3. Define MCP state tool interface
4. Migrate shared state to coordination

## Example: Multi-Instance Deployment

```rust
// Instance 1 (Primary DC)
let instance1 = Caxton::new()
    .with_local_db("instance1.db")
    .with_swim_seeds(vec!["instance2:7946"]);

// Instance 2 (Secondary DC)
let instance2 = Caxton::new()
    .with_local_db("instance2.db")
    .with_swim_seeds(vec!["instance1:7946"]);

// They discover each other via SWIM
// Share agent registry via gossip
// No shared database needed!
```

## Comparison with Other Systems

### HashiCorp Consul

- Uses SWIM for membership
- Raft only for critical configuration
- Proves gossip scales to thousands of nodes

### Apache Cassandra

- Uses gossip for cluster state
- No central coordinator
- Scales to hundreds of nodes

### Kubernetes

- etcd only for critical config
- Kubelet has local state
- Proves hybrid model works

## Risks and Mitigations

### Risk: Eventual Consistency

**Mitigation**: Only use for non-critical data like agent discovery. Critical
operations use local state.

### Risk: Network Partitions

**Mitigation**: SWIM handles partitions gracefully. Each partition continues
operating independently.

### Risk: Missing Features

**Mitigation**: MCP tools provide flexibility. Businesses can add any state
management they need.

## Conclusion

Caxton should:

1. **Abandon shared state** in favor of coordination protocols
2. **Use SQLite** for local instance state
3. **Implement SWIM** for cluster coordination
4. **Delegate agent state** to MCP tools

This approach:

- Eliminates PostgreSQL dependency
- Reduces operational complexity
- Improves scalability
- Aligns with minimal core philosophy
- Provides maximum flexibility

The key insight: **Caxton is a message router, not a database**. Let it excel at
routing while businesses handle their own state requirements through MCP tools.

## Recommended Next Steps

1. **Revise ADR-0013** to remove PostgreSQL dependency
2. **Create new ADR** for coordination-first architecture
3. **Define MCP StateTool interface** specification
4. **Prototype SWIM integration** using memberlist-rs
5. **Update architecture docs** to reflect this approach

This lightweight approach will make Caxton easier to deploy, operate, and scale
while maintaining all necessary functionality through intelligent architectural
choices.
