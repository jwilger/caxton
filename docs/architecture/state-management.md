---
title: "State Management Architecture"
date: 2025-01-15
layout: page
categories: [Architecture]
---

## Executive Summary

Based on extensive research, Caxton adopts a **coordination-only approach**
rather than shared state, with agent state management delegated to business
domains via MCP tools. This aligns with the minimal core philosophy and
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
        db.execute("CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            capabilities TEXT,
            last_seen INTEGER
        )", [])?;
        Ok(Self { db })
    }

    pub fn register_agent(&self, id: &str, capabilities: &[String]) -> Result<()> {
        self.db.execute(
            "INSERT OR REPLACE INTO agents (id, capabilities, last_seen)
             VALUES (?1, ?2, ?3)",
            (id, serde_json::to_string(capabilities)?, Utc::now().timestamp())
        )?;
        Ok(())
    }
}
```

### 2. Gossip Protocol for Coordination

For cluster coordination, use **SWIM (Scalable Weakly-consistent Infection-style
Process Group Membership)** protocol:

```rust
pub struct GossipNode {
    node_id: String,
    local_state: LocalState,
    peers: HashMap<String, PeerInfo>,
    gossip_interval: Duration,
}

impl GossipNode {
    pub async fn sync_with_peers(&mut self) {
        let random_peer = self.select_random_peer();
        if let Some(peer) = random_peer {
            self.exchange_state(&peer).await;
        }
    }

    async fn exchange_state(&mut self, peer: &PeerInfo) {
        let local_agents = self.local_state.list_agents();
        let update = StateUpdate {
            node_id: self.node_id.clone(),
            agents: local_agents,
            timestamp: Utc::now(),
        };

        // Send our state and receive peer's state
        if let Ok(peer_update) = self.send_gossip(peer, update).await {
            self.merge_peer_state(peer_update);
        }
    }
}
```

### 3. Agent State as Business Concern

Agent-specific state is handled by **MCP State Tools** provided by the business:

```rust
// Agent delegates state to business-provided tool
pub struct ConfigAgent {
    id: AgentId,
    state_tool: Option<Box<dyn McpStateTool>>, // Optional business state
}

impl ConfigAgent {
    pub async fn save_state(&self, state: Value) -> Result<()> {
        if let Some(tool) = &self.state_tool {
            tool.store(format!("agent:{}", self.id), state).await?;
        }
        // No state tool = stateless agent (perfectly fine!)
        Ok(())
    }
}
```

## Implementation Strategy

### Phase 1: Embedded Foundation

1. **Local SQLite State**: Each instance tracks its own agents
2. **In-Memory Coordination**: Start with single-node deployment
3. **Zero Dependencies**: No external state stores required

### Phase 2: Gossip Coordination

1. **SWIM Protocol**: Add peer discovery and failure detection
2. **Eventually Consistent**: Agent registry syncs across nodes
3. **Partition Tolerance**: Nodes operate independently when isolated

### Phase 3: MCP State Integration

1. **Optional State Tools**: Business provides persistent storage
2. **Multiple Backends**: Redis, PostgreSQL, S3, filesystem, etc.
3. **Agent Choice**: Each agent chooses its own state strategy

## Benefits of This Approach

### Operational Simplicity

- **Zero Dependencies**: Works out-of-the-box without external services
- **Linear Scaling**: Add nodes without complex coordination
- **Graceful Degradation**: Continues operating during partitions

### Business Flexibility

- **State Choice**: Each domain chooses appropriate persistence
- **No Lock-in**: Can change state backends without affecting Caxton
- **Cost Control**: Only pay for state storage you actually need

### Development Speed

- **Local Development**: No setup of external services
- **Fast Iteration**: Embedded SQLite is immediate
- **Simple Testing**: No external dependencies to mock

## Comparison with Alternatives

| Approach | Complexity | Dependencies | Consistency | Performance |
|----------|------------|--------------|-------------|-------------|
| **Embedded SQLite + Gossip** | Low | Zero | Eventual | High |
| Shared PostgreSQL | Medium | PostgreSQL | Strong | Medium |
| Redis Cluster | High | Redis | Strong | High |
| Kubernetes State | High | K8s + etcd | Strong | Medium |
| Consul/etcd | High | Consul/etcd | Strong | Medium |

## Implementation Details

### SQLite Schema

```sql
-- Local agent registry
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    capabilities TEXT NOT NULL, -- JSON array
    last_seen INTEGER NOT NULL,
    status TEXT DEFAULT 'active'
);

-- Conversation tracking
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    participants TEXT NOT NULL, -- JSON array of agent IDs
    created_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL
);

-- Message correlation (temporary)
CREATE TABLE pending_messages (
    correlation_id TEXT PRIMARY KEY,
    sender_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);
```

### Gossip Message Format

```rust
#[derive(Serialize, Deserialize)]
pub struct GossipMessage {
    pub sender: String,
    pub message_type: GossipType,
    pub payload: GossipPayload,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub enum GossipType {
    StateSync,
    AgentUpdate,
    NodeJoin,
    NodeLeave,
    HealthCheck,
}

#[derive(Serialize, Deserialize)]
pub struct StateSync {
    pub agents: Vec<AgentInfo>,
    pub version: u64,
}
```

### Health Checking

```rust
impl GossipNode {
    pub async fn health_check_loop(&mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;
            self.check_peer_health().await;
            self.cleanup_stale_agents().await;
        }
    }

    async fn check_peer_health(&mut self) {
        for peer in &mut self.peers.values_mut() {
            if peer.last_seen.elapsed() > Duration::from_secs(60) {
                peer.status = PeerStatus::Suspected;
                self.initiate_failure_detection(peer).await;
            }
        }
    }
}
```

## Deployment Patterns

### Single Node (Development)

```yaml
# No configuration needed - embedded SQLite only
caxton_config:
  embedded_only: true
  sqlite_path: "/var/lib/caxton/state.db"
```

### Multi-Node (Production)

```yaml
# Minimal gossip configuration
caxton_config:
  gossip:
    enabled: true
    bind_address: "0.0.0.0:7946"
    seed_nodes:
      - "caxton-1:7946"
      - "caxton-2:7946"
    gossip_interval: "1s"
    failure_detection_timeout: "60s"
```

### Kubernetes Deployment

```yaml
# Uses headless service for peer discovery
apiVersion: v1
kind: Service
metadata:
  name: caxton-gossip
spec:
  clusterIP: None  # Headless service
  selector:
    app: caxton
  ports:
    - port: 7946
      name: gossip

---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: caxton
spec:
  serviceName: caxton-gossip
  replicas: 3
  template:
    spec:
      containers:
        - name: caxton
          env:
            - name: CAXTON_GOSSIP_SEEDS
              value: "caxton-0.caxton-gossip:7946,caxton-1.caxton-gossip:7946"
```

## Monitoring and Observability

### Metrics

```rust
// Gossip protocol metrics
lazy_static! {
    static ref GOSSIP_MESSAGES_SENT: Counter = Counter::new(
        "caxton_gossip_messages_sent_total",
        "Total gossip messages sent"
    );

    static ref PEER_COUNT: Gauge = Gauge::new(
        "caxton_gossip_peers",
        "Number of known peers"
    );

    static ref AGENT_REGISTRY_SIZE: Gauge = Gauge::new(
        "caxton_agent_registry_size",
        "Number of agents in local registry"
    );
}
```

### Health Endpoints

```rust
// GET /health/gossip
pub async fn gossip_health() -> Json<GossipHealth> {
    Json(GossipHealth {
        peer_count: gossip_node.peer_count(),
        last_sync: gossip_node.last_sync_time(),
        agent_count: local_state.agent_count(),
        status: if gossip_node.is_healthy() {
            "healthy"
        } else {
            "degraded"
        },
    })
}
```

## Migration Path

### From External State Stores

If currently using external state stores:

1. **Phase 1**: Deploy embedded SQLite alongside existing state
2. **Phase 2**: Migrate read queries to local state
3. **Phase 3**: Remove external dependencies
4. **Phase 4**: Add gossip for multi-node coordination

### From Stateless Architecture

If currently stateless:

1. **Phase 1**: Add embedded SQLite for local caching
2. **Phase 2**: Enable agent registry persistence
3. **Phase 3**: Add gossip for cluster awareness

## References

- [ADR-0030: Embedded Memory System](../adr/0030-embedded-memory-system.md)
- [MCP Integration Guide](../api/mcp-integration.md)
- [Performance Tuning](../operations/performance-tuning.md)
- [SWIM Protocol Paper](https://www.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
