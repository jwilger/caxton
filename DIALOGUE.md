# Communication Patterns Design Dialogue

## Summary of Expert Consensus

The expert team has reached strong consensus on the fundamental approach to communication patterns in the Caxton framework.

### Core Agreement: Minimal Event-Based Foundation

All experts agree that the framework should provide:

1. **Simple Event Storage Primitive**
   ```rust
   pub trait EventLog {
       fn append(&self, stream_id: StreamId, events: Vec<RawEvent>, expected_version: Version) -> Result<(), Error>;
       fn read(&self, stream_id: StreamId, from: Version) -> Result<Vec<RawEvent>, Error>;
   }
   ```

2. **Essential Event Metadata**
   ```rust
   pub struct EventMetadata {
       pub event_id: EventId,           // For idempotency
       pub correlation_id: CorrelationId, // For tracing conversations
       pub causation_id: Option<EventId>, // For understanding cause/effect
       pub timestamp: Timestamp,          // For human debugging (not ordering!)
   }
   ```

3. **Communication Patterns as Emergent Behavior**
   - Commands, Events, and Queries are just data types
   - Request/Reply emerges from correlation IDs
   - Pub/Sub emerges from event projections
   - Sagas emerge from event-triggered commands
   - No built-in routing, orchestration, or coordination

### Key Principles Agreed Upon

1. **Mechanism, Not Policy**: Framework provides tools, users decide how to use them
2. **Make the Implicit Explicit**: Time, failures, and causation should be visible events
3. **Type Safety Without Coupling**: Strong types for correctness, but generic framework
4. **Simplicity First**: Start minimal, let complexity emerge from real usage

### Open Questions Requiring Resolution

Before finalizing the design, the team needs to resolve:

1. **Vector Clocks for Causality Detection**
   - Greg Young strongly advocates for including vector clocks in v1
   - Would help detect concurrent events in distributed agent systems
   - Question: Essential for v1 or can be added later?

2. **Global Event Ordering**
   - Greg Young insists on `read_all()` for debugging distributed flows
   - Rich Hickey might view this as unnecessary complexity
   - Question: Core framework feature or optional extension?

3. **Async Architecture Documentation**
   - Yoshua Wuyts provided detailed async/await patterns with back-pressure
   - Others focused on logical patterns rather than implementation
   - Question: Document async patterns now or defer to implementation phase?

### Next Steps

The Project Manager requests the expert team to:
1. Vote on the three open questions
2. Provide brief rationale for their positions
3. Suggest compromise approaches if there's disagreement

Once these are resolved, we can create the final technical specification for communication patterns.

### Communication Patterns Emerging from Consensus

Based on the current agreement, these patterns would naturally emerge:

```rust
// Request-Reply (using correlation)
let command = Command { 
    id: EventId::new(),
    correlation_id: CorrelationId::new(),
    payload: GetBalance { account_id }
};
// ... events with same correlation_id contain the reply

// Pub-Sub (through projections)
struct NotificationProjection;
impl Projection for NotificationProjection {
    fn handle_event(&mut self, event: &Event) {
        match event {
            OrderPlaced { .. } => self.notify_subscribers(event),
            _ => {}
        }
    }
}

// Sagas (event-driven state machines)
struct OrderSaga;
impl Projection for OrderSaga {
    fn handle_event(&mut self, event: &Event) -> Vec<Command> {
        match (&self.state, event) {
            (WaitingForPayment, PaymentReceived { .. }) => {
                vec![ShipOrder { .. }]
            }
            _ => vec![]
        }
    }
}
```

All without framework-level support beyond basic event storage!

---

## Question 9: Ready to Document?

The expert team has successfully reached consensus on all technical aspects of the Caxton framework. They've designed:

**Core Architecture:**
- Minimal event sourcing framework with essential metadata
- WebAssembly agents with FIPA messaging
- MCP tools for external communication
- Event-based debugging and traceability

**Key Decisions:**
- Start with minimal V1 (just event storage + metadata)
- Defer complex features (vector clocks, detailed async) to V2
- Focus on providing mechanisms, not policies
- Let communication patterns emerge from event choreography

**Phased Approach:**
- V1: Core event sourcing, basic agent lifecycle, debugging
- V2: Clustering, advanced patterns, performance optimizations
- V3+: Based on real usage and community feedback

The experts are ready to create:
1. **README.md** - Framework vision and getting started guide
2. **ROADMAP.md** - Detailed phased development plan
3. **Initial ADRs** - Document key architectural decisions

Do you approve proceeding with documentation creation based on this consensus?

---

*Please provide your response below:*

## User Response - 2025-07-31T11:30:00

Approved!