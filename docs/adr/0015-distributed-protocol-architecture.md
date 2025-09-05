## Status

Superseded by ADR-0025: Single-Instance Architecture

## Context

With the coordination-first architecture (ADR-0014), Caxton uses SWIM
for
cluster coordination and FIPA for agent messaging. This ADR clarifies
how
these protocols interact and addresses distributed systems concerns
including
network partitioning, consistency, and fault tolerance.

## Decision

### Protocol Layer Separation

Caxton implements a clear separation between coordination (SWIM)
and
communication (FIPA) protocols:

#### SWIM Protocol (Infrastructure Layer)

- **Responsibility**: Cluster membership and failure detection
- **Scope**: Caxton instance coordination
- **Data**: Instance liveness, agent registry, routing tables
- **Consistency**: Eventually consistent via gossip
- **Failure Model**: Crash-stop failures

#### FIPA Protocol (Application Layer)

- **Responsibility**: Agent-to-agent semantic messaging
- **Scope**: Business logic communication
- **Data**: Application messages, conversation state
- **Consistency**: Message ordering per conversation
- **Failure Model**: Handled by application

### Cross-Cluster Agent Communication

```rust
pub struct DistributedMessageRouter {
    // Local agent registry
    local_agents: Arc<RwLock<HashMap<AgentId, LocalAgent>>>,

    // Remote agent locations (learned via SWIM gossip)
    remote_routes: Arc<RwLock<HashMap<AgentId, NodeId>>>,

    // Cluster membership (SWIM)
    cluster: SwimCluster,

    // Message delivery tracking
    in_flight: Arc<RwLock<HashMap<MessageId, InFlightMessage>>>,
}

impl DistributedMessageRouter {
    pub async fn route_fipa_message(
        &self,
        msg: FipaMessage,
    ) -> Result<DeliveryStatus> {
        let target = &msg.receiver;

        // Try local delivery first
        if let Some(agent) = self.local_agents.read().await.get(target) {
            return agent.deliver(msg).await;
        }

        // Check remote routing table
        if let Some(node_id) = self.remote_routes.read().await.get(target) {
            // Verify node is still alive (SWIM)
            if self.cluster.is_alive(node_id) {
                return self.forward_to_remote(node_id, msg).await;
            }
        }

        // Agent location unknown - initiate discovery
        self.discover_and_route(msg).await
    }

    async fn forward_to_remote(
        &self,
        node: &NodeId,
        msg: FipaMessage,
    ) -> Result<DeliveryStatus> {
        // Track in-flight message
        self.track_message(&msg).await;

        // Forward via cluster network
        let result = self.cluster
            .send_to_node(node, Payload::FipaMessage(msg))
            .await;

        // Update delivery status
        self.update_delivery_status(&msg.message_id, result).await
    }
}
```

### Network Partition Handling

#### Detection Strategy

```rust
pub struct PartitionManager {
    membership: SwimMembership,
    config: PartitionConfig,
}

impl PartitionManager {
    pub fn evaluate_partition_state(&self) -> PartitionState {
        let alive = self.membership.alive_nodes();
        let total = self.config.expected_cluster_size;
        let quorum = total / 2 + 1;

        match alive.len() {
            n if n >= quorum => PartitionState::Majority {
                missing: total - n,
                can_write: true,
            },
            n if n > 0 => PartitionState::Minority {
                available: n,
                can_write: false,
            },
            _ => PartitionState::Isolated,
        }
    }

    pub async fn handle_partition(&self, state: PartitionState) -> Result<()> {
        match state {
            PartitionState::Majority { .. } => {
                // Continue normal operations
                // Mark missing nodes as failed
                self.mark_unreachable_nodes().await?;
            }
            PartitionState::Minority { .. } => {
                // Degrade to read-only mode
                self.enter_degraded_mode().await?;
                // Queue writes for later
                self.enable_write_queue().await?;
            }
            PartitionState::Isolated => {
                // Local-only mode
                self.disable_remote_routing().await?;
            }
        }
        Ok(())
    }
}
```

#### Healing After Partition

```rust
pub struct PartitionHealer {
    detector: PartitionDetector,
    synchronizer: StateSynchronizer,
}

impl PartitionHealer {
    pub async fn heal_partition(&self) -> Result<()> {
        // Detect partition healing
        if self.detector.is_healed() {
            // Exchange vector clocks
            let divergence = self.synchronizer.detect_divergence().await?;

            // Merge agent registries
            self.merge_registries(divergence.registries).await?;

            // Replay queued messages
            self.replay_queued_messages().await?;

            // Resume normal operations
            self.exit_degraded_mode().await?;
        }
        Ok(())
    }
}
```

### Consistency Models

#### Agent Registry (Eventually Consistent)

```rust
pub struct AgentRegistry {
    local: HashMap<AgentId, AgentMetadata>,
    vector_clock: VectorClock,
    tombstones: HashMap<AgentId, Timestamp>,
}

impl AgentRegistry {
    pub async fn merge_with_peer(&mut self, peer_registry: &AgentRegistry) {
        for (agent_id, peer_meta) in &peer_registry.local {
            match self.local.get(agent_id) {
                Some(local_meta) => {
                    // Resolve conflict using vector clocks
                    if peer_registry.vector_clock
                        .happens_after(&self.vector_clock, agent_id) {
                        self.local.insert(agent_id.clone(), peer_meta.clone());
                    }
                }
                None => {
                    // Check if we have a tombstone
                    if !self.is_tombstoned(agent_id) {
                        self.local.insert(agent_id.clone(), peer_meta.clone());
                    }
                }
            }
        }

        // Merge vector clocks
        self.vector_clock.merge(&peer_registry.vector_clock);
    }
}
```

#### Message Ordering (Per-Conversation)

```rust
pub struct ConversationManager {
    conversations: HashMap<ConversationId, Conversation>,
}

impl ConversationManager {
    pub async fn handle_message(&mut self, msg: FipaMessage) -> Result<()> {
        let conv_id = msg.conversation_id.as_ref()
            .ok_or(Error::NoConversationId)?;

        let conversation = self.conversations.entry(conv_id.clone())
            .or_insert_with(|| Conversation::new(conv_id.clone()));

        // Ensure message ordering within conversation
        conversation.add_message(msg).await?;

        // Process in order
        while let Some(next_msg) = conversation.next_in_sequence() {
            self.process_message(next_msg).await?;
        }

        Ok(())
    }
}
```

### Fault Tolerance Mechanisms

#### Circuit Breaker for Remote Calls

```rust
pub struct RemoteCallCircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: usize,
    timeout: Duration,
}

impl RemoteCallCircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let state = self.state.read().await;

        match *state {
            CircuitState::Open => {
                Err(Error::CircuitOpen)
            }
            CircuitState::HalfOpen => {
                // Try one request
                match timeout(self.timeout, f).await {
                    Ok(Ok(result)) => {
                        self.reset().await;
                        Ok(result)
                    }
                    _ => {
                        self.trip().await;
                        Err(Error::CircuitTripped)
                    }
                }
            }
            CircuitState::Closed => {
                match timeout(self.timeout, f).await {
                    Ok(Ok(result)) => Ok(result),
                    _ => {
                        self.record_failure().await;
                        Err(Error::RemoteCallFailed)
                    }
                }
            }
        }
    }
}
```

#### Supervisor Trees for Agents

```rust
pub struct AgentSupervisor {
    strategy: SupervisionStrategy,
    children: HashMap<AgentId, AgentHandle>,
}

impl AgentSupervisor {
    pub async fn supervise(&mut self) {
        loop {
            tokio::select! {
                Some(failure) = self.receive_failure() => {
                    match self.strategy {
                        SupervisionStrategy::OneForOne => {
                            self.restart_agent(failure.agent_id).await;
                        }
                        SupervisionStrategy::OneForAll => {
                            self.restart_all_agents().await;
                        }
                        SupervisionStrategy::RestForOne => {
                            self.restart_dependent_agents(failure.agent_id)
                                .await;
                        }
                    }
                }
            }
        }
    }
}
```

### Message Delivery Guarantees

#### Configurable Delivery Semantics

```rust
pub enum DeliveryGuarantee {
    AtMostOnce,   // Fire and forget (default)
    AtLeastOnce,  // Retry with deduplication
    ExactlyOnce,  // Idempotent with sequence numbers
}

pub struct MessageDelivery {
    guarantee: DeliveryGuarantee,
    dedup_cache: LruCache<MessageId, DeliveryStatus>,
}

impl MessageDelivery {
    pub async fn deliver(
        &mut self,
        msg: FipaMessage,
    ) -> Result<DeliveryStatus> {
        match self.guarantee {
            DeliveryGuarantee::AtMostOnce => {
                self.send_once(msg).await
            }
            DeliveryGuarantee::AtLeastOnce => {
                // Check dedup cache
                if let Some(status) = self.dedup_cache.get(&msg.message_id) {
                    return Ok(status.clone());
                }

                let status = self.send_with_retry(msg).await?;
                self.dedup_cache.put(msg.message_id, status.clone());
                Ok(status)
            }
            DeliveryGuarantee::ExactlyOnce => {
                self.send_idempotent(msg).await
            }
        }
    }
}
```

## Consequences

### Positive

- **Clear separation of concerns**: SWIM handles infrastructure, FIPA handles
  application
- **Graceful degradation**: System continues functioning during partitions
- **Flexible consistency**: Eventually consistent for coordination, stronger
  guarantees available when needed
- **Fault isolation**: Agent failures don't affect cluster coordination
- **Scalable design**: Can handle thousands of agents across dozens of instances

### Negative

- **Complexity**: Two protocols to understand and maintain
- **Eventual consistency**: Agent registry may be temporarily inconsistent
- **Network overhead**: Gossip protocol generates background traffic
- **Partition handling**: Requires careful consideration of business
  requirements

### Neutral

- Standard distributed systems patterns apply
- Similar complexity to other distributed agent systems
- Trade-offs are well-understood in the industry

## Implementation Priorities

1. **Phase 1**: Basic SWIM integration for membership
2. **Phase 2**: Agent registry gossip and routing
3. **Phase 3**: Partition detection and handling
4. **Phase 4**: Advanced features (consensus, exactly-once delivery)

## Implementation Details

### Technology Selection

#### SWIM Implementation

```toml
[dependencies]
# Primary choice: memberlist-rs (Rust port of HashiCorp's memberlist)
memberlist = { git = "https://github.com/vectordotdev/memberlist-rs" }

# Alternative: Build on swimmer (pure Rust SWIM)
# swimmer = "0.1"

# Network transport
quinn = "0.10"  # QUIC for better performance
tokio = { version = "1.0", features = ["full"] }
```

#### Message Serialization

```toml
# MessagePack for efficiency and schema evolution
rmp-serde = "1.1"  # MessagePack serialization
serde = { version = "1.0", features = ["derive"] }

# Alternative: Protocol Buffers for stricter schemas
# prost = "0.12"
# prost-build = "0.12"
```

#### Network Transport

```rust
pub enum TransportLayer {
    // TCP for reliability (default)
    Tcp(TcpConfig),

    // QUIC for performance (recommended)
    Quic(QuicConfig),

    // Unix sockets for local testing
    Unix(UnixConfig),
}

pub struct TransportConfig {
    // TCP Configuration
    tcp: TcpConfig {
        nodelay: true,        // Disable Nagle's algorithm
        keepalive: Some(30s), // TCP keepalive
        buffer_size: 64KB,    // Socket buffer size
    },

    // QUIC Configuration (recommended for production)
    quic: QuicConfig {
        max_streams: 100,
        idle_timeout: 30s,
        congestion_control: CongestionControl::Bbr,
    },
}
```

### SWIM Library Integration

```rust
use memberlist::{Memberlist, Config, Node, NodeState};

pub struct SwimCluster {
    memberlist: Arc<Memberlist>,
    delegate: Arc<CaxtonDelegate>,
}

impl SwimCluster {
    pub async fn new(config: ClusterConfig) -> Result<Self> {
        let mut ml_config = Config::default();

        // Configure SWIM parameters
        ml_config.gossip_interval = config.gossip_interval;
        ml_config.gossip_nodes = config.gossip_fanout;
        ml_config.probe_interval = config.probe_interval;
        ml_config.probe_timeout = config.probe_timeout;

        // Set up delegates for custom behavior
        let delegate = Arc::new(CaxtonDelegate::new());
        ml_config.delegate = Some(delegate.clone());

        // Initialize memberlist
        let memberlist = Memberlist::create(ml_config).await?;

        Ok(Self {
            memberlist: Arc::new(memberlist),
            delegate,
        })
    }

    pub async fn join(&self, seeds: Vec<String>) -> Result<()> {
        self.memberlist.join(seeds).await?;
        Ok(())
    }
}

// Custom delegate for Caxton-specific behavior
struct CaxtonDelegate {
    agent_registry: Arc<RwLock<AgentRegistry>>,
    event_handler: Arc<EventHandler>,
}

impl memberlist::Delegate for CaxtonDelegate {
    fn node_meta(&self, limit: usize) -> Vec<u8> {
        // Include agent count and capabilities in metadata
        let meta = NodeMetadata {
            agent_count: self.agent_registry.read().len(),
            capabilities: self.capabilities(),
            version: env!("CARGO_PKG_VERSION"),
        };
        rmp_serde::to_vec(&meta).unwrap()
    }

    fn notify_msg(&self, msg: &[u8]) {
        // Handle custom messages (agent updates, routing info)
        if let Ok(update) = rmp_serde::from_slice::<AgentUpdate>(msg) {
            self.handle_agent_update(update);
        }
    }

    fn get_broadcast(&self, overhead: usize, limit: usize) -> Option<Vec<u8>> {
        // Broadcast agent registry changes
        self.agent_registry.read().get_pending_broadcasts(limit)
    }
}
```

### Message Serialization Format

```rust
use serde::{Serialize, Deserialize};
use rmp_serde;

// MessagePack serialization for FIPA messages
#[derive(Serialize, Deserialize)]
pub struct WireFipaMessage {
    // Header fields (compact representation)
    #[serde(rename = "p")]
    performative: u8,  // Enum as u8 for compactness

    #[serde(rename = "s")]
    sender: CompactAgentId,

    #[serde(rename = "r")]
    receiver: CompactAgentId,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    conversation_id: Option<Uuid>,

    // Payload
    #[serde(rename = "b")]
    body: Vec<u8>,  // Pre-serialized content
}

// Compact agent ID representation
#[derive(Serialize, Deserialize)]
pub struct CompactAgentId {
    #[serde(rename = "n")]
    node: u32,  // Node index in cluster

    #[serde(rename = "a")]
    agent: u32, // Agent index on node
}

impl WireFipaMessage {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec_named(self)
            .map_err(|e| Error::Serialization(e))
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data)
            .map_err(|e| Error::Deserialization(e))
    }
}
```

### Network Transport Details

```rust
// QUIC transport for better performance
pub struct QuicTransport {
    endpoint: quinn::Endpoint,
    connections: Arc<RwLock<HashMap<NodeId, quinn::Connection>>>,
}

impl QuicTransport {
    pub async fn new(config: QuicConfig) -> Result<Self> {
        let mut endpoint_config = quinn::ServerConfig::with_crypto(
            Arc::new(rustls::ServerConfig::builder()
                .with_safe_defaults()
                .with_no_client_auth()
                .with_single_cert(cert_chain, key)?)
        );

        // Configure transport parameters
        let mut transport_config = quinn::TransportConfig::default();
        transport_config.max_concurrent_uni_streams(0_u8.into());
        transport_config.max_concurrent_bidi_streams(100_u8.into());
        transport_config
            .max_idle_timeout(Some(Duration::from_secs(30).try_into()?));

        endpoint_config.transport = Arc::new(transport_config);

        let endpoint = quinn::Endpoint::server(endpoint_config, addr)?;

        Ok(Self {
            endpoint,
            connections: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn send(&self, node: &NodeId, data: Vec<u8>) -> Result<()> {
        let conn = self.get_or_create_connection(node).await?;
        let mut stream = conn.open_uni().await?;
        stream.write_all(&data).await?;
        stream.finish().await?;
        Ok(())
    }
}
```

### Bootstrap Configuration

```yaml
# Cluster bootstrap configuration
coordination:
  cluster:
    # SWIM protocol settings
    swim:
      # Use memberlist-rs library
      implementation: memberlist-rs

      # Gossip parameters
      gossip_interval: 200ms
      gossip_fanout: 3
      gossip_to_dead: 3

      # Failure detection
      probe_interval: 1s
      probe_timeout: 500ms
      suspicion_multiplier: 4

    # Network transport
    transport:
      type: quic  # tcp | quic | unix
      bind_addr: 0.0.0.0:7946
      advertise_addr: auto  # auto-detect or specify

    # Message serialization
    serialization:
      format: messagepack  # messagepack | protobuf
      compression: lz4     # none | lz4 | zstd

    # Security
    security:
      encryption: true
      auth_key: ${CLUSTER_AUTH_KEY}
```

## References

- [SWIM Protocol Paper](
  https://www.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
- [FIPA Agent Communication Language](
  http://www.fipa.org/specs/fipa00061/SC00061G.html)
- [Distributed Systems: Principles and Paradigms](
  https://www.distributed-systems.net/index.php/books/ds3/)
- [memberlist-rs](https://github.com/vectordotdev/memberlist-rs)
- [MessagePack Specification](https://msgpack.org/)
- [QUIC RFC 9000](https://datatracker.ietf.org/doc/html/rfc9000)
- [ADR-0014: Coordination-First Architecture](
  0014-coordination-first-architecture.md)
- [ADR-0012: Pragmatic FIPA Subset](0012-pragmatic-fipa-subset.md)
- [ADR-0016: Security Architecture](0016-security-architecture.md)
- [ADR-0017: Performance Requirements](0017-performance-requirements.md)
