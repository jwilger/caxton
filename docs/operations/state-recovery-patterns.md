# State Recovery Patterns for Agents

## Overview
This document outlines comprehensive patterns and procedures for recovering agent state after crashes, restarts, or failures. These patterns ensure system resilience and maintain operational continuity in production environments.

## Recovery Scenarios

### 1. Agent Crash Recovery
When an individual agent crashes unexpectedly.

### 2. Orchestrator Restart
When the orchestrator restarts (planned or unplanned).

### 3. Network Partition Recovery
When agents reconnect after network isolation.

### 4. Partial System Failure
When subsystems fail but the core remains operational.

### 5. Complete System Recovery
When recovering from total system failure.

## State Recovery Patterns

### Pattern 1: Checkpoint-Based Recovery

#### Overview
Agents periodically save state checkpoints that can be restored on restart.

#### Implementation
```rust
pub struct AgentCheckpoint {
    agent_id: AgentId,
    state_version: u64,
    timestamp: Instant,
    state_data: Vec<u8>,
    conversation_contexts: HashMap<ConversationId, ConversationState>,
    pending_tasks: Vec<Task>,
}

impl Agent {
    pub async fn create_checkpoint(&self) -> Result<AgentCheckpoint> {
        let checkpoint = AgentCheckpoint {
            agent_id: self.id.clone(),
            state_version: self.version,
            timestamp: Instant::now(),
            state_data: self.serialize_state()?,
            conversation_contexts: self.get_active_conversations(),
            pending_tasks: self.get_pending_tasks(),
        };

        self.store_checkpoint(checkpoint).await
    }

    pub async fn restore_from_checkpoint(
        checkpoint: AgentCheckpoint
    ) -> Result<Self> {
        let mut agent = Self::new(checkpoint.agent_id);
        agent.deserialize_state(checkpoint.state_data)?;
        agent.restore_conversations(checkpoint.conversation_contexts)?;
        agent.requeue_tasks(checkpoint.pending_tasks)?;
        agent.version = checkpoint.state_version + 1;

        Ok(agent)
    }
}
```

#### Checkpoint Strategies

##### Time-Based Checkpointing
```rust
pub struct TimeBasedCheckpointer {
    interval: Duration,
    last_checkpoint: Instant,
}

impl TimeBasedCheckpointer {
    pub fn should_checkpoint(&self) -> bool {
        Instant::now().duration_since(self.last_checkpoint) > self.interval
    }
}
```

##### Event-Based Checkpointing
```rust
pub struct EventBasedCheckpointer {
    event_threshold: usize,
    events_since_checkpoint: AtomicUsize,
}

impl EventBasedCheckpointer {
    pub fn should_checkpoint(&self) -> bool {
        self.events_since_checkpoint.load(Ordering::Relaxed) >= self.event_threshold
    }
}
```

### Pattern 2: Event Sourcing Recovery

#### Overview
Reconstruct state by replaying events from the event log.

#### Implementation
```rust
pub struct EventSourcedRecovery {
    event_store: EventStore,
    snapshot_store: SnapshotStore,
}

impl EventSourcedRecovery {
    pub async fn recover_agent(&self, agent_id: AgentId) -> Result<Agent> {
        // Try to load latest snapshot
        let (mut state, last_event_id) = match self.snapshot_store
            .load_latest(agent_id.clone()).await?
        {
            Some(snapshot) => {
                (snapshot.state, snapshot.last_event_id)
            }
            None => {
                (Agent::initial_state(agent_id.clone()), 0)
            }
        };

        // Replay events since snapshot
        let events = self.event_store
            .load_events_after(agent_id, last_event_id)
            .await?;

        for event in events {
            state = state.apply_event(event)?;
        }

        Ok(state)
    }
}
```

#### Event Replay Optimization
```rust
pub struct OptimizedEventReplay {
    batch_size: usize,
    parallel_replay: bool,
}

impl OptimizedEventReplay {
    pub async fn replay_events(
        &self,
        events: Vec<Event>
    ) -> Result<AgentState> {
        if self.parallel_replay && events.len() > self.batch_size {
            self.parallel_replay_events(events).await
        } else {
            self.sequential_replay_events(events).await
        }
    }

    async fn parallel_replay_events(
        &self,
        events: Vec<Event>
    ) -> Result<AgentState> {
        // Partition events by independence
        let partitions = self.partition_independent_events(events);

        // Replay partitions in parallel
        let futures: Vec<_> = partitions
            .into_iter()
            .map(|partition| self.replay_partition(partition))
            .collect();

        // Merge results
        let states = futures::future::join_all(futures).await;
        self.merge_states(states?)
    }
}
```

### Pattern 3: Conversation Recovery

#### Overview
Restore in-progress conversations and message contexts.

#### Implementation
```rust
pub struct ConversationRecovery {
    conversation_store: ConversationStore,
    message_store: MessageStore,
}

impl ConversationRecovery {
    pub async fn recover_conversation(
        &self,
        conversation_id: ConversationId
    ) -> Result<ConversationContext> {
        let conversation = self.conversation_store
            .load(conversation_id.clone())
            .await?;

        let messages = self.message_store
            .load_conversation_messages(conversation_id.clone())
            .await?;

        let mut context = ConversationContext::new(conversation);

        // Rebuild conversation state
        for message in messages {
            context.process_message(message)?;
        }

        // Identify pending responses
        context.identify_pending_responses()?;

        Ok(context)
    }

    pub async fn resume_conversations(
        &self,
        agent_id: AgentId
    ) -> Result<Vec<ConversationContext>> {
        let active_conversations = self.conversation_store
            .find_active_for_agent(agent_id)
            .await?;

        let mut recovered = Vec::new();
        for conv_id in active_conversations {
            match self.recover_conversation(conv_id).await {
                Ok(context) => recovered.push(context),
                Err(e) => {
                    // Log and continue with other conversations
                    error!("Failed to recover conversation: {}", e);
                }
            }
        }

        Ok(recovered)
    }
}
```

### Pattern 4: Task Recovery

#### Overview
Recover and resume incomplete tasks after agent restart.

#### Implementation
```rust
pub struct TaskRecovery {
    task_store: TaskStore,
    retry_policy: RetryPolicy,
}

pub struct Task {
    id: TaskId,
    agent_id: AgentId,
    state: TaskState,
    payload: serde_json::Value,
    attempts: u32,
    last_attempt: Option<Instant>,
    deadline: Option<Instant>,
}

pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed,
    Compensating,
}

impl TaskRecovery {
    pub async fn recover_tasks(
        &self,
        agent_id: AgentId
    ) -> Result<Vec<Task>> {
        let incomplete_tasks = self.task_store
            .find_incomplete_for_agent(agent_id)
            .await?;

        let mut recovered_tasks = Vec::new();

        for mut task in incomplete_tasks {
            match task.state {
                TaskState::Running => {
                    // Was running when crashed, retry based on policy
                    if self.should_retry(&task) {
                        task.state = TaskState::Pending;
                        task.attempts += 1;
                        recovered_tasks.push(task);
                    } else {
                        task.state = TaskState::Failed;
                        self.task_store.update(task).await?;
                    }
                }
                TaskState::Pending => {
                    // Was queued, can resume
                    recovered_tasks.push(task);
                }
                TaskState::Compensating => {
                    // Was compensating, must complete compensation
                    recovered_tasks.push(task);
                }
                _ => {}
            }
        }

        Ok(recovered_tasks)
    }

    fn should_retry(&self, task: &Task) -> bool {
        task.attempts < self.retry_policy.max_attempts &&
        task.deadline.map_or(true, |d| Instant::now() < d)
    }
}
```

### Pattern 5: Distributed State Recovery

#### Overview
Coordinate state recovery across multiple agents and nodes.

#### Implementation
```rust
pub struct DistributedRecovery {
    coordinator: RecoveryCoordinator,
    nodes: Vec<NodeId>,
}

impl DistributedRecovery {
    pub async fn coordinate_recovery(&self) -> Result<()> {
        // Phase 1: Discovery
        let alive_nodes = self.discover_alive_nodes().await?;
        let failed_nodes = self.identify_failed_nodes(&alive_nodes)?;

        // Phase 2: Election (if coordinator failed)
        if failed_nodes.contains(&self.coordinator.node_id) {
            self.elect_new_coordinator(&alive_nodes).await?;
        }

        // Phase 3: State reconciliation
        self.reconcile_distributed_state(&alive_nodes).await?;

        // Phase 4: Redistribute work from failed nodes
        self.redistribute_work(&failed_nodes, &alive_nodes).await?;

        // Phase 5: Resume operations
        self.resume_normal_operations().await?;

        Ok(())
    }

    async fn reconcile_distributed_state(
        &self,
        nodes: &[NodeId]
    ) -> Result<()> {
        // Collect state vectors from all nodes
        let state_vectors = self.collect_state_vectors(nodes).await?;

        // Compute consensus state
        let consensus = self.compute_consensus_state(state_vectors)?;

        // Distribute consensus to all nodes
        self.distribute_consensus(consensus, nodes).await?;

        Ok(())
    }
}
```

## Recovery Strategies by Scenario

### Scenario 1: Single Agent Crash

```rust
pub async fn recover_crashed_agent(agent_id: AgentId) -> Result<()> {
    // 1. Load checkpoint
    let checkpoint = load_latest_checkpoint(agent_id.clone()).await?;

    // 2. Restore agent state
    let agent = Agent::restore_from_checkpoint(checkpoint).await?;

    // 3. Recover conversations
    let conversations = ConversationRecovery::new()
        .resume_conversations(agent_id.clone())
        .await?;

    // 4. Recover tasks
    let tasks = TaskRecovery::new()
        .recover_tasks(agent_id.clone())
        .await?;

    // 5. Re-register with orchestrator
    orchestrator.register_agent(agent).await?;

    // 6. Resume processing
    agent.resume_processing(conversations, tasks).await?;

    Ok(())
}
```

### Scenario 2: Orchestrator Restart

```rust
pub async fn recover_orchestrator() -> Result<Orchestrator> {
    // 1. Load orchestrator state
    let state = load_orchestrator_state().await?;

    // 2. Rebuild agent registry
    let registry = rebuild_agent_registry().await?;

    // 3. Restore routing tables
    let routing = restore_routing_tables().await?;

    // 4. Recover message queues
    let queues = recover_message_queues().await?;

    // 5. Re-establish agent connections
    let connections = reestablish_connections(&registry).await?;

    // 6. Resume message processing
    let orchestrator = Orchestrator::new(state, registry, routing, queues);
    orchestrator.resume().await?;

    Ok(orchestrator)
}
```

### Scenario 3: Network Partition Healing

```rust
pub async fn heal_network_partition(
    partition_a: Vec<NodeId>,
    partition_b: Vec<NodeId>
) -> Result<()> {
    // 1. Detect partition healing
    let healed = detect_partition_healing(&partition_a, &partition_b).await?;

    if healed {
        // 2. Exchange state vectors
        let state_a = collect_partition_state(&partition_a).await?;
        let state_b = collect_partition_state(&partition_b).await?;

        // 3. Merge states using CRDTs or vector clocks
        let merged_state = merge_partition_states(state_a, state_b)?;

        // 4. Resolve conflicts
        let resolved_state = resolve_conflicts(merged_state)?;

        // 5. Distribute merged state
        distribute_state(resolved_state, &partition_a).await?;
        distribute_state(resolved_state, &partition_b).await?;

        // 6. Resume normal operations
        resume_unified_operations().await?;
    }

    Ok(())
}
```

## Recovery Testing

### Chaos Engineering Tests

```rust
#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_crash_recovery() {
        let mut system = TestSystem::new();
        let agent = system.spawn_agent("test-agent").await;

        // Create some state
        agent.process_messages(generate_test_messages(100)).await;

        // Simulate crash
        system.crash_agent(&agent.id).await;

        // Attempt recovery
        let recovered = recover_crashed_agent(agent.id.clone()).await.unwrap();

        // Verify state consistency
        assert_eq!(recovered.processed_count(), 100);
        assert!(recovered.is_consistent());
    }

    #[tokio::test]
    async fn test_partition_recovery() {
        let mut cluster = TestCluster::new(5);

        // Create partition
        let (partition_a, partition_b) = cluster.create_partition(2, 3).await;

        // Process messages in both partitions
        partition_a.process_messages(100).await;
        partition_b.process_messages(100).await;

        // Heal partition
        cluster.heal_partition().await;

        // Verify eventual consistency
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert!(cluster.is_consistent());
    }
}
```

### Recovery Benchmarks

```rust
#[bench]
fn bench_checkpoint_recovery(b: &mut Bencher) {
    let checkpoint = create_large_checkpoint(1_000_000_events);

    b.iter(|| {
        let _ = Agent::restore_from_checkpoint(checkpoint.clone());
    });
}

#[bench]
fn bench_event_replay(b: &mut Bencher) {
    let events = generate_events(10_000);

    b.iter(|| {
        let _ = replay_events(events.clone());
    });
}
```

## Monitoring Recovery Operations

### Key Metrics

```rust
pub struct RecoveryMetrics {
    pub recovery_time: Histogram,
    pub recovered_agents: Counter,
    pub failed_recoveries: Counter,
    pub data_loss_events: Counter,
    pub checkpoint_size: Histogram,
    pub replay_speed: Gauge,
}
```

### Recovery Dashboards

```yaml
recovery_dashboard:
  panels:
    - title: "Recovery Time (p95)"
      query: "histogram_quantile(0.95, caxton_recovery_time_seconds)"

    - title: "Recovery Success Rate"
      query: "rate(caxton_recovered_agents[5m]) / rate(caxton_recovery_attempts[5m])"

    - title: "Data Loss Events"
      query: "increase(caxton_data_loss_events[1h])"

    - title: "Checkpoint Sizes"
      query: "caxton_checkpoint_size_bytes"
```

## Best Practices

### 1. Checkpoint Frequency
- Balance between recovery time and overhead
- More frequent for critical agents
- Less frequent for stateless agents

### 2. State Minimization
- Keep agent state minimal
- Store only essential data
- Use references for large objects

### 3. Idempotent Operations
- Ensure operations can be safely retried
- Use unique operation IDs
- Check for duplicate processing

### 4. Graceful Degradation
- Continue operating with reduced functionality
- Prioritize critical operations
- Queue non-critical work for later

### 5. Testing Recovery Paths
- Regular disaster recovery drills
- Automated chaos testing
- Monitor recovery metrics

## Troubleshooting Guide

### Common Issues and Solutions

#### Issue: Slow Recovery
**Symptoms**: Recovery takes longer than RTO
**Solutions**:
- Increase checkpoint frequency
- Optimize event replay
- Use parallel recovery
- Add more snapshots

#### Issue: State Inconsistency
**Symptoms**: Agents have different views of state
**Solutions**:
- Implement vector clocks
- Use CRDTs for convergence
- Add state reconciliation phase
- Increase consistency checks

#### Issue: Message Loss
**Symptoms**: Missing messages after recovery
**Solutions**:
- Implement message persistence
- Add acknowledgment tracking
- Use reliable message queues
- Implement replay from source

## References
- [ADR-0013: State Management Architecture](../adr/0013-state-management-architecture.md)
- [Agent Communication Pattern Catalog](../patterns/agent-communication-patterns.md)
- [Disaster Recovery Planning](https://aws.amazon.com/disaster-recovery/)
- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
