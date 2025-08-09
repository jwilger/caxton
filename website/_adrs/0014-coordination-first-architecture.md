---
layout: adr
title: "0014. Coordination-First Architecture"
status: proposed
date: 2025-08-09
---

# ADR-0014: Coordination-First Architecture

## Status
Proposed (Supersedes ADR-0013)

## Context
ADR-0013 proposed using PostgreSQL for state management with event sourcing and snapshots. However, after careful analysis, this approach:
- Violates the minimal core philosophy by adding heavyweight dependencies
- Makes Caxton responsible for business domain concerns (agent state)
- Creates operational complexity (backups, migrations, replication)
- Introduces a shared state bottleneck that limits scalability

Further research revealed that Caxton's actual needs are **coordination** rather than **shared state**:
- Agent discovery and registry
- Health monitoring and failure detection
- Message routing information
- Cluster membership

## Decision
Caxton adopts a **coordination-first architecture** that eliminates shared state in favor of lightweight coordination protocols. Agent state management becomes a business domain responsibility through MCP tools.

### Core Principles

#### 1. No Shared State
Each Caxton instance maintains only local state. No external database dependencies.

#### 2. Coordination Through Gossip
Use SWIM protocol for cluster coordination:
- Scalable membership protocol
- Built-in failure detection
- Eventually consistent
- No single point of failure

#### 3. Agent State via MCP Tools
Agents requiring persistence use business-provided MCP tools:
```rust
// Standard interface for state persistence
#[async_trait]
pub trait McpStateTool: Send + Sync {
    async fn store(&self, key: String, value: Value) -> Result<()>;
    async fn retrieve(&self, key: String) -> Result<Option<Value>>;
    async fn delete(&self, key: String) -> Result<()>;
    async fn list(&self, prefix: String) -> Result<Vec<String>>;
}
```

### Architecture Components

#### Local State Storage
Each instance uses embedded SQLite for local state:
```rust
pub struct LocalState {
    db: rusqlite::Connection,
}

impl LocalState {
    pub fn new(path: &str) -> Result<Self> {
        let db = Connection::open(path)?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                capabilities TEXT NOT NULL,
                metadata TEXT,
                last_seen INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS routes (
                agent_id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );"
        )?;
        Ok(Self { db })
    }
}
```

#### Cluster Coordination
SWIM protocol for distributed coordination:
```rust
use memberlist::{Config, Memberlist, Node};

pub struct ClusterCoordinator {
    memberlist: Memberlist,
    local_agents: Arc<RwLock<HashMap<AgentId, AgentInfo>>>,
}

impl ClusterCoordinator {
    pub async fn start(&mut self, bind_addr: &str, seeds: Vec<String>) -> Result<()> {
        let config = Config::default()
            .with_bind_addr(bind_addr)
            .with_gossip_interval(Duration::from_millis(200));

        self.memberlist = Memberlist::new(config)?;

        if !seeds.is_empty() {
            self.memberlist.join(seeds).await?;
        }

        self.start_gossip_loop().await;
        Ok(())
    }

    async fn start_gossip_loop(&self) {
        // Periodically gossip local agent registry
        tokio::spawn(async move {
            loop {
                self.broadcast_agents().await;
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    }
}
```

#### Message Routing
Routing without shared state:
```rust
pub struct MessageRouter {
    local_routes: HashMap<AgentId, NodeId>,
    gossip: Arc<ClusterCoordinator>,
}

impl MessageRouter {
    pub async fn route(&self, msg: Message) -> Result<()> {
        // Try local agents first
        if let Some(agent) = self.local_agents.get(&msg.receiver) {
            return agent.handle(msg).await;
        }

        // Check gossip-learned routes
        if let Some(node_id) = self.local_routes.get(&msg.receiver) {
            return self.forward_to_node(node_id, msg).await;
        }

        // Broadcast query if unknown
        self.gossip.query_agent_location(&msg.receiver).await
    }
}
```

### State Categories

#### Caxton-Managed (Coordination)
- **Agent Registry**: Which agents exist and their capabilities
- **Cluster Membership**: Which Caxton instances are alive
- **Routing Table**: Which node hosts which agents
- **Health Status**: Liveness and readiness information

#### Business-Managed (State)
- **Agent Checkpoints**: Persistent agent state
- **Conversation History**: Message logs and context
- **Task State**: Long-running operation status
- **Audit Logs**: Compliance and debugging
- **Business Data**: Domain-specific information

### Implementation Example

#### Multi-Instance Deployment
```rust
// Instance 1 (Primary datacenter)
let instance1 = Caxton::builder()
    .with_local_db("instance1.db")
    .with_bind_addr("10.0.1.10:7946")
    .with_seeds(vec!["10.0.2.10:7946"])
    .build()?;

// Instance 2 (Secondary datacenter)
let instance2 = Caxton::builder()
    .with_local_db("instance2.db")
    .with_bind_addr("10.0.2.10:7946")
    .with_seeds(vec!["10.0.1.10:7946"])
    .build()?;

// They automatically:
// - Discover each other via SWIM
// - Share agent registries via gossip
// - Route messages without shared state
```

#### Agent with Business State
```rust
pub struct StatefulAgent {
    id: AgentId,
    state_tool: Box<dyn McpStateTool>,
}

impl StatefulAgent {
    pub async fn checkpoint(&self) -> Result<()> {
        let state = self.serialize_state()?;
        self.state_tool.store(
            format!("checkpoints/{}", self.id),
            state
        ).await
    }

    pub async fn restore(&mut self) -> Result<()> {
        if let Some(state) = self.state_tool.retrieve(
            format!("checkpoints/{}", self.id)
        ).await? {
            self.deserialize_state(state)?;
        }
        Ok(())
    }
}
```

## Consequences

### Positive
- **No external dependencies**: SQLite is embedded, SWIM is a library
- **Linear scalability**: No shared state bottleneck
- **Operational simplicity**: No database administration
- **Fault isolation**: Node failures don't affect others
- **Geographic distribution**: Works naturally across regions
- **Business flexibility**: Choose any state backend via MCP
- **Minimal core maintained**: Caxton remains a message router

### Negative
- **Eventual consistency**: Agent registry may be temporarily inconsistent
- **No strong consistency**: Cannot guarantee global ordering
- **Learning curve**: SWIM protocol less familiar than databases

### Neutral
- **Different mental model**: Think coordination, not shared state
- **MCP tool requirement**: Businesses must provide state tools if needed
- **Migration complexity**: Existing systems expecting shared state need updates

## Migration Path

### Phase 1: Local State (Week 1)
- Implement SQLite for local storage
- No breaking changes to external API

### Phase 2: SWIM Protocol (Week 2-3)
- Add memberlist dependency
- Implement gossip for agent registry
- Maintain backward compatibility

### Phase 3: Remove Shared State (Week 4)
- Deprecate PostgreSQL backend
- Provide migration tools
- Document MCP state tool interface

### Phase 4: MCP Tools (Week 5-6)
- Publish MCP StateTool trait
- Provide reference implementations
- Create migration guides

## Alternatives Considered

### Keep PostgreSQL (ADR-0013)
- **Pros**: Strong consistency, familiar tooling
- **Cons**: Heavy dependency, operational complexity, scalability limits
- **Decision**: Rejected due to minimal core violation

### Embedded etcd
- **Pros**: Strong consistency, proven in Kubernetes
- **Cons**: Still requires consensus, complex for our needs
- **Decision**: Overkill for coordination-only needs

### Redis with Clustering
- **Pros**: Fast, supports pub/sub
- **Cons**: External dependency, complex cluster setup
- **Decision**: Still violates zero-dependency goal

## Comparison with Industry Systems

### HashiCorp Consul
- Uses SWIM for membership (like our proposal)
- Raft only for critical config (we avoid entirely)
- Proves gossip scales to thousands of nodes

### Apache Cassandra
- Gossip protocol for cluster state
- No central coordinator
- Validates our approach at scale

### Kubernetes
- etcd for config, local state in kubelet
- Similar hybrid model
- Shows pattern works in production

## Guidelines

1. **Think coordination, not consistency**: Design for eventual consistency
2. **Local first**: Prefer local state over distributed state
3. **Gossip sparingly**: Only share essential information
4. **Business owns state**: Let MCP tools handle persistence
5. **Fail independently**: Design for partition tolerance

## References
- [SWIM: Scalable Weakly-consistent Infection-style Process Group Membership Protocol](https://www.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
- [Gossip Protocol (Wikipedia)](https://en.wikipedia.org/wiki/Gossip_protocol)
- [Lightweight State Alternatives Research](../research/lightweight-state-alternatives.md)
- [ADR-0004: Minimal Core Philosophy](0004-minimal-core-philosophy.md)
- [ADR-0013: State Management Architecture (Superseded)](0013-state-management-architecture.md)

## Notes
This architecture makes Caxton truly lightweight and cloud-native. By eliminating shared state, we remove the primary scaling bottleneck and operational burden. The coordination-first approach aligns perfectly with the minimal core philosophy while providing all necessary functionality through intelligent architectural choices.
