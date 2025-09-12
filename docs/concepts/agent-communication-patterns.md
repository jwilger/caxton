---
title: "Agent Communication Pattern Catalog"
date: 2024-01-01
layout: page
categories: [Patterns]
---


## Introduction

This catalog documents common agent interaction patterns in Caxton, providing
developers with proven solutions for typical multi-agent coordination scenarios.
Each pattern includes implementation examples, use cases, and best practices.

## Pattern Categories

1. **Basic Patterns** - Fundamental communication primitives
2. **Coordination Patterns** - Multi-agent synchronization
3. **Negotiation Patterns** - Resource allocation and bidding
4. **Workflow Patterns** - Complex task orchestration
5. **Resilience Patterns** - Error handling and recovery

______________________________________________________________________

## 1. Basic Patterns

### 1.1 Request-Response Pattern

**Purpose**: Simple synchronous communication between two agents.

**Structure**:

```json
// Request
{
  "performative": "request",
  "sender": "agent-a",
  "receiver": "agent-b",
  "content": {
    "action": "calculate",
    "params": {"x": 10, "y": 20}
  },
  "reply_with": "req-123",
  "conversation_id": "conv-456"
}

// Response
{
  "performative": "inform",
  "sender": "agent-b",
  "receiver": "agent-a",
  "content": {
    "result": 30
  },
  "in_reply_to": "req-123",
  "conversation_id": "conv-456"
}
```

**Use Cases**:

- Service invocation
- Data retrieval
- Computation requests

**Best Practices**:

- Always include `reply_with` and `conversation_id`
- Set reasonable timeouts (default: 30s)
- Handle `failure` and `not_understood` responses

### 1.2 Fire-and-Forget Pattern

**Purpose**: Asynchronous notification without expecting response.

**Structure**:

```json
{
  "performative": "inform",
  "sender": "monitor-agent",
  "receiver": "logger-agent",
  "content": {
    "event": "threshold_exceeded",
    "metric": "cpu_usage",
    "value": 85.2
  },
  "conversation_id": "monitoring-stream"
}
```

**Use Cases**:

- Event notifications
- Logging
- Metrics collection

**Best Practices**:

- Use for non-critical notifications
- Implement at-least-once delivery
- Consider batching for high-volume events

### 1.3 Query Pattern

**Purpose**: Information retrieval from knowledge-holding agents.

**Structure**:

```json
// Query
{
  "performative": "query",
  "sender": "analyst-agent",
  "receiver": "database-agent",
  "content": {
    "query": "SELECT * FROM orders WHERE status = 'pending'"
  },
  "reply_with": "query-789"
}

// Result
{
  "performative": "inform",
  "sender": "database-agent",
  "receiver": "analyst-agent",
  "content": {
    "results": [
      {"id": 1, "status": "pending", "amount": 100},
      {"id": 2, "status": "pending", "amount": 200}
    ]
  },
  "in_reply_to": "query-789"
}
```

**Use Cases**:

- Database queries
- Knowledge base lookups
- Status inquiries

______________________________________________________________________

## 2. Coordination Patterns

### 2.1 Publish-Subscribe Pattern

**Purpose**: One-to-many event distribution.

**Implementation**:

```rust
// Publisher
pub fn publish_event(event: Event) {
    let message = Message {
        performative: Performative::Inform,
        sender: AgentId::from("publisher"),
        receiver: AgentId::from("topic://events"),
        content: json!({"event": event}),
        conversation_id: format!("pub-{}", Uuid::new_v4()),
    };
    orchestrator.route_message(message);
}

// Subscriber Registration
pub fn subscribe_to_topic(topic: &str, agent_id: AgentId) {
    orchestrator.register_subscription(topic, agent_id);
}
```

**Use Cases**:

- Event streaming
- Broadcast notifications
- Market data distribution

**Best Practices**:

- Use topic hierarchies (e.g., `events/orders/created`)
- Implement subscription filtering
- Handle subscriber disconnections gracefully

### 2.2 Contract Net Protocol

**Purpose**: Task distribution through bidding.

**Flow**:

1. **Call for Proposals (CFP)**

```json
{
  "performative": "cfp",
  "sender": "manager",
  "receiver": "broadcast",
  "content": {
    "task": "image_processing",
    "requirements": {
      "format": "jpeg",
      "operations": ["resize", "compress"],
      "deadline": "2024-01-01T10:00:00Z"
    }
  },
  "reply_by": "2024-01-01T09:00:00Z"
}
```

1. **Proposals from Agents**

```json
{
  "performative": "propose",
  "sender": "worker-1",
  "receiver": "manager",
  "content": {
    "bid": {
      "cost": 10,
      "estimated_time": 300,
      "quality_score": 0.95
    }
  }
}
```

1. **Accept/Reject Proposals**

```json
{
  "performative": "accept_proposal",
  "sender": "manager",
  "receiver": "worker-1",
  "content": {
    "task_id": "task-123",
    "start_time": "2024-01-01T09:05:00Z"
  }
}
```

**Use Cases**:

- Load balancing
- Resource allocation
- Service discovery

### 2.3 Blackboard Pattern

**Purpose**: Shared workspace for collaborative problem-solving.

**Implementation**:

```rust
pub struct Blackboard {
    entries: HashMap<String, BlackboardEntry>,
    subscriptions: HashMap<String, Vec<AgentId>>,
}

pub struct BlackboardEntry {
    key: String,
    value: serde_json::Value,
    author: AgentId,
    timestamp: Instant,
    version: u64,
}

impl Blackboard {
    pub fn write(&mut self, key: String, value: Value, author: AgentId) {
        let entry = BlackboardEntry {
            key: key.clone(),
            value,
            author,
            timestamp: Instant::now(),
            version: self.next_version(&key),
        };

        self.entries.insert(key.clone(), entry);
        self.notify_subscribers(&key);
    }
}
```

**Use Cases**:

- Collaborative planning
- Shared state management
- Knowledge accumulation

______________________________________________________________________

## 3. Negotiation Patterns

### 3.1 Auction Pattern

**Purpose**: Resource allocation through competitive bidding.

**Types**:

- **English Auction**: Ascending price
- **Dutch Auction**: Descending price
- **Vickrey Auction**: Sealed-bid, second-price

**Example (English Auction)**:

```json
// Auctioneer announces
{
  "performative": "inform",
  "content": {
    "auction_type": "english",
    "item": "compute_resource",
    "starting_bid": 100,
    "increment": 10
  }
}

// Bidder responds
{
  "performative": "propose",
  "content": {
    "bid": 110,
    "bidder_id": "agent-x"
  }
}
```

### 3.2 Bargaining Pattern

**Purpose**: Bilateral negotiation for mutual agreement.

**Flow**:

```rust
enum NegotiationState {
    Initial,
    Proposing,
    CounterProposing,
    Agreed,
    Failed,
}

pub fn negotiate(initial_offer: Offer) -> Result<Agreement> {
    let mut state = NegotiationState::Initial;
    let mut current_offer = initial_offer;

    loop {
        match state {
            NegotiationState::Proposing => {
                send_proposal(current_offer);
                state = NegotiationState::CounterProposing;
            }
            NegotiationState::CounterProposing => {
                match receive_response() {
                    Response::Accept => {
                        state = NegotiationState::Agreed;
                        break;
                    }
                    Response::Counter(offer) => {
                        current_offer = evaluate_offer(offer)?;
                        state = NegotiationState::Proposing;
                    }
                    Response::Reject => {
                        state = NegotiationState::Failed;
                        break;
                    }
                }
            }
            _ => break,
        }
    }

    finalize_negotiation(state, current_offer)
}
```

______________________________________________________________________

## 4. Workflow Patterns

### 4.1 Pipeline Pattern

**Purpose**: Sequential processing through agent chain.

**Structure**:

```yaml
pipeline:
  name: "data_processing"
  stages:
    - agent: "validator"
      action: "validate_input"
    - agent: "transformer"
      action: "transform_data"
    - agent: "analyzer"
      action: "analyze_patterns"
    - agent: "reporter"
      action: "generate_report"
```

**Implementation**:

```rust
pub async fn execute_pipeline(data: Data, pipeline: Pipeline) -> Result<Data> {
    let mut result = data;

    for stage in pipeline.stages {
        result = execute_stage(result, stage).await?;
    }

    Ok(result)
}
```

### 4.2 Scatter-Gather Pattern

**Purpose**: Parallel processing with result aggregation.

**Structure**:

```rust
pub async fn scatter_gather<T>(
    task: Task,
    agents: Vec<AgentId>,
    aggregator: fn(Vec<T>) -> T
) -> Result<T> {
    // Scatter phase
    let futures: Vec<_> = agents
        .iter()
        .map(|agent| send_task(agent, task.clone()))
        .collect();

    // Gather phase
    let results = futures::future::join_all(futures).await;

    // Aggregate results
    Ok(aggregator(results?))
}
```

**Use Cases**:

- Map-reduce operations
- Distributed search
- Parallel computation

### 4.3 Saga Pattern

**Purpose**: Distributed transaction with compensation.

**Implementation**:

```rust
pub struct Saga {
    steps: Vec<SagaStep>,
    compensation_stack: Vec<CompensationAction>,
}

pub struct SagaStep {
    agent: AgentId,
    action: Action,
    compensation: CompensationAction,
}

impl Saga {
    pub async fn execute(&mut self) -> Result<()> {
        for step in &self.steps {
            match execute_step(step).await {
                Ok(_) => {
                    self.compensation_stack.push(step.compensation.clone());
                }
                Err(e) => {
                    self.compensate().await?;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    async fn compensate(&mut self) -> Result<()> {
        while let Some(action) = self.compensation_stack.pop() {
            execute_compensation(action).await?;
        }
        Ok(())
    }
}
```

______________________________________________________________________

## 5. Resilience Patterns

### 5.1 Circuit Breaker Pattern

**Purpose**: Prevent cascading failures.

**Implementation**:

```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    failure_count: AtomicU32,
    state: AtomicU8, // 0: Closed, 1: Open, 2: Half-Open
    last_failure_time: AtomicU64,
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match self.state.load(Ordering::Relaxed) {
            0 => { // Closed
                match f.await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        self.record_failure();
                        Err(e)
                    }
                }
            }
            1 => { // Open
                if self.should_attempt_reset() {
                    self.state.store(2, Ordering::Relaxed);
                    self.attempt_call(f).await
                } else {
                    Err(Error::CircuitOpen)
                }
            }
            2 => { // Half-Open
                self.attempt_call(f).await
            }
            _ => unreachable!()
        }
    }
}
```

### 5.2 Retry Pattern with Exponential Backoff

**Purpose**: Handle transient failures.

**Implementation**:

```rust
pub async fn retry_with_backoff<F, T>(
    mut f: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> Future<Output = Result<T>>,
{
    let mut retries = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 5.3 Bulkhead Pattern

**Purpose**: Isolate failures to prevent system-wide impact.

**Implementation**:

```rust
pub struct Bulkhead {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl Bulkhead {
    pub async fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let permit = self.semaphore
            .acquire()
            .await
            .map_err(|_| Error::BulkheadFull)?;

        let result = f.await;
        drop(permit);
        result
    }
}
```

______________________________________________________________________

## Pattern Composition

### Combining Patterns

Patterns can be composed for complex scenarios:

```rust
// Combine Contract Net with Pipeline
pub async fn distributed_pipeline(task: Task) -> Result<Output> {
    // Use Contract Net to find best agents for each stage
    let pipeline_agents = negotiate_pipeline_agents(task).await?;

    // Execute pipeline with selected agents
    execute_pipeline_with_agents(task, pipeline_agents).await
}

// Combine Scatter-Gather with Circuit Breaker
pub async fn resilient_scatter_gather(task: Task) -> Result<Vec<Output>> {
    let agents = discover_agents().await?;

    let results = futures::stream::iter(agents)
        .map(|agent| {
            let circuit_breaker = get_circuit_breaker(agent);
            circuit_breaker.call(send_task(agent, task.clone()))
        })
        .buffer_unordered(10)
        .collect()
        .await;

    aggregate_results(results)
}
```

## Anti-Patterns to Avoid

### 1. Chatty Communication

**Problem**: Excessive message exchanges for simple operations. **Solution**:
Batch operations, use coarser-grained interfaces.

### 2. Missing Correlation IDs

**Problem**: Cannot track related messages in conversations. **Solution**:
Always include `conversation_id` and use `reply_with`/`in_reply_to`.

### 3. Unbounded Waits

**Problem**: Waiting indefinitely for responses. **Solution**: Implement
timeouts and fallback strategies.

### 4. Ignoring Failures

**Problem**: Not handling `failure` or `not_understood` messages. **Solution**:
Implement comprehensive error handling.

### 5. Synchronous Chains

**Problem**: Sequential processing when parallelism is possible. **Solution**:
Use Scatter-Gather or parallel patterns where applicable.

## Testing Patterns

### Pattern Test Framework

```rust
#[cfg(test)]
mod pattern_tests {
    use super::*;

    #[tokio::test]
    async fn test_request_response_pattern() {
        let mut orchestrator = MockOrchestrator::new();
        let requester = TestAgent::new("requester");
        let responder = TestAgent::new("responder");

        orchestrator.register_agent(requester.clone());
        orchestrator.register_agent(responder.clone());

        // Setup responder behavior
        responder.on_request("calculate", |params| {
            Ok(json!({"result": params["x"].as_i64()? + params["y"].as_i64()?}))
        });

        // Execute pattern
        let response = requester
            .request(&responder, "calculate", json!({"x": 10, "y": 20}))
            .await
            .unwrap();

        assert_eq!(response["result"], 30);
    }
}
```

## Performance Considerations

### Message Size Limits

- Keep message payloads under 1MB
- Use references for large data (store in shared storage)
- Implement chunking for large transfers

### Throughput Optimization

- Batch small messages
- Use async patterns for I/O-bound operations
- Implement connection pooling

### Latency Reduction

- Minimize message hops
- Cache frequently accessed data
- Use regional agent deployment

## References

- [ADR-0003: Agent Messaging Protocol](../adrs/0003-agent-messaging-protocol.md)
- [ADR-0012: Pragmatic Agent Messaging Subset](../adrs/0012-pragmatic-agent-subset.md)
- [Enterprise Integration Patterns](https://www.enterpriseintegrationpatterns.com/)
- [Reactive Messaging Patterns](https://www.reactivemanifesto.org/)
