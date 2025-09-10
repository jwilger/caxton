# ADR-0015: Distributed Protocol Architecture - FIPA and SWIM Integration

## Status

Proposed

## Context

With the coordination-first architecture (ADR-0014), Caxton uses SWIM for
cluster coordination and FIPA for agent messaging. This ADR clarifies how these
protocols interact and addresses distributed systems concerns including network
partitioning, consistency, and fault tolerance.

## Decision

### Protocol Layer Separation

Caxton implements a clear separation between coordination (SWIM) and
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
    pub async fn route_fipa_message(&self, msg: FipaMessage) -> Result<DeliveryStatus> {
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

    async fn forward_to_remote(&self, node: &NodeId, msg: FipaMessage) -> Result<DeliveryStatus> {
        // Track in-flight message
        self.track_message(&msg).await;

        // Forward via cluster network
        let result = self.cluster.send_to_node(node, Payload::FipaMessage(msg)).await;

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
                    if peer_registry.vector_clock.happens_after(&self.vector_clock, agent_id) {
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
                            self.restart_dependent_agents(failure.agent_id).await;
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
    pub async fn deliver(&mut self, msg: FipaMessage) -> Result<DeliveryStatus> {
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

## References

- [SWIM Protocol Paper](https://www.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
- [FIPA Agent Communication Language](http://www.fipa.org/specs/fipa00061/SC00061G.html)
- [Distributed Systems: Principles and Paradigms](https://www.distributed-systems.net/index.php/books/ds3/)
- [ADR-0014: Coordination-First Architecture](0014-coordination-first-architecture.md)
- [ADR-0012: Pragmatic FIPA Subset](0012-pragmatic-fipa-subset.md)
