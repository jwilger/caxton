---
title: "ADR-0029: FIPA-ACL Lightweight Messaging"
date: 2025-09-09
status: accepted
layout: adr
categories: [Architecture]
---

## Status

Accepted

## Context

With the shift to configuration-driven agents (ADR-0028), we need to reconsider
how agents communicate with each other. The previous FIPA-ACL implementation was
designed for compiled WASM agents, but config agents have different capabilities
and constraints.

### Current State Analysis

- **ADR-0003** established FIPA messaging protocol for WASM agents
- **ADR-0012** defined a pragmatic FIPA subset for performance
- Configuration-driven agents need a simpler, more accessible messaging model
- Contract Net Protocol (CNP) adds significant complexity that may not be needed
  for 1.0

### Key Requirements

1. **Capability-based routing**: Agents should request capabilities, not
   specific agents
2. **Conversation tracking**: Maintain thread context across message exchanges
3. **Standard compliance**: Follow FIPA-ACL where practical for interoperability
4. **Config agent friendly**: Work naturally with prompt-based agent logic
5. **Future extensible**: Allow advanced features without breaking simple cases

## Decision

We will implement a **lightweight FIPA-ACL messaging system** optimized for
configuration-driven agents, with capability-based routing and deferred Contract
Net Protocol implementation.

### Core Message Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FipaMessage {
    /// FIPA performative (intent of the message)
    pub performative: Performative,

    /// Sending agent identifier
    pub sender: AgentId,

    /// Target capability (not specific agent)
    pub capability: Capability,

    /// Message content (natural language or structured data)
    pub content: MessageContent,

    /// Conversation tracking
    pub conversation_id: ConversationId,
    pub reply_with: Option<MessageId>,
    pub in_reply_to: Option<MessageId>,

    /// Protocol and routing metadata
    pub protocol: Protocol,
    pub language: String,        // Default: "en"
    pub ontology: Option<String>,
    pub reply_by: Option<Instant>,
}
```

### Supported Performatives (1.0 Scope)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Performative {
    // Basic communication
    REQUEST,        // Ask agent to perform action
    INFORM,         // Share information
    QUERY,          // Ask for information

    // Error handling
    FAILURE,        // Indicate action failed
    NOT_UNDERSTOOD, // Message not comprehensible

    // Simple negotiation (no bidding)
    PROPOSE,        // Suggest action/value
    ACCEPT_PROPOSAL, // Agree to proposal
    REJECT_PROPOSAL, // Decline proposal
}
```

### Deferred Performatives (Post-1.0)

```rust
// Contract Net Protocol - deferred to post-1.0
pub enum DeferredPerformatives {
    CFP,            // Call for Proposals (bidding)
    CONFIRM,        // Confirm understanding
    DISCONFIRM,     // Disconfirm understanding
    CANCEL,         // Cancel previous request
    SUBSCRIBE,      // Subscribe to notifications
    // ... other advanced performatives
}
```

### Capability-Based Routing

Instead of addressing specific agents, messages target capabilities:

```yaml
# Example config agent declaring capabilities
---
name: DataAnalyzer
capabilities:
  - data-analysis      # Can analyze datasets
  - report-generation  # Can create reports
  - chart-creation     # Can make visualizations
---
```

```rust
// Message routing based on capabilities
pub struct MessageRouter {
    capability_registry: CapabilityRegistry,
    conversation_manager: ConversationManager,
}

impl MessageRouter {
    pub async fn route_message(&self, msg: FipaMessage) -> Result<Vec<AgentId>> {
        // Find agents that provide the requested capability
        let candidates = self.capability_registry
            .get_agents_with_capability(&msg.capability)
            .await?;

        // Route to single agent or multiple based on protocol
        match msg.protocol {
            Protocol::SingleRecipient => Ok(vec![self.select_best_match(candidates)?]),
            Protocol::Broadcast => Ok(candidates),
        }
    }
}
```

### Configuration Agent Integration

Config agents participate in FIPA messaging through the runtime:

```rust
impl ConfigAgentRuntime {
    pub async fn handle_fipa_message(
        &self,
        agent: &ConfigAgent,
        msg: FipaMessage
    ) -> Result<Option<FipaMessage>> {
        // Create context-aware prompt
        let prompt = self.format_fipa_prompt(agent, &msg)?;

        // Get LLM response
        let response = self.llm_client.complete(prompt).await?;

        // Parse response into FIPA message (if any)
        self.parse_fipa_response(response, &msg.conversation_id)
    }

    fn format_fipa_prompt(&self, agent: &ConfigAgent, msg: &FipaMessage) -> String {
        format!(
            "{system_prompt}\n\n\
            You received a {performative} message about {capability}:\n\
            Content: {content}\n\n\
            Respond appropriately using one of: INFORM, REQUEST, QUERY, \
            PROPOSE, ACCEPT_PROPOSAL, REJECT_PROPOSAL, FAILURE, NOT_UNDERSTOOD\n\n\
            If you cannot help, respond with FAILURE and explain why.\n\
            If the message is unclear, respond with NOT_UNDERSTOOD.",

            system_prompt = agent.prompts.system_prompt,
            performative = msg.performative,
            capability = msg.capability,
            content = msg.content
        )
    }
}
```

### Conversation Management

```rust
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: ConversationId,
    pub participants: HashSet<AgentId>,
    pub protocol: Protocol,
    pub messages: Vec<FipaMessage>,
    pub state: ConversationState,
    pub created_at: Instant,
    pub last_activity: Instant,
}

pub struct ConversationManager {
    conversations: DashMap<ConversationId, Conversation>,
    cleanup_interval: Duration,
}

impl ConversationManager {
    pub async fn track_message(&self, msg: &FipaMessage) -> Result<()> {
        let mut conversation = self.get_or_create_conversation(&msg.conversation_id).await?;

        // Add participants
        conversation.participants.insert(msg.sender.clone());

        // Link reply chains
        if let Some(reply_to) = &msg.in_reply_to {
            conversation.link_reply(msg.clone(), reply_to.clone())?;
        }

        // Update activity timestamp
        conversation.last_activity = Instant::now();

        Ok(())
    }

    // Cleanup stale conversations
    pub async fn cleanup_stale_conversations(&self, max_age: Duration) -> Result<u32> {
        let cutoff = Instant::now() - max_age;
        let mut removed = 0;

        self.conversations.retain(|_id, conversation| {
            if conversation.last_activity < cutoff {
                removed += 1;
                false
            } else {
                true
            }
        });

        Ok(removed)
    }
}
```

### Example Usage Patterns

#### Simple Request-Response

```rust
// Agent A requests data analysis
let request = FipaMessage {
    performative: Performative::REQUEST,
    sender: agent_a_id,
    capability: Capability::new("data-analysis"),
    content: MessageContent::text("Please analyze sales data from Q3"),
    conversation_id: ConversationId::new(),
    reply_with: Some(MessageId::new()),
    // ... other fields
};

// Agent B (data analyzer) responds
let response = FipaMessage {
    performative: Performative::INFORM,
    sender: agent_b_id,
    capability: Capability::new("data-analysis"),
    content: MessageContent::structured(analysis_results),
    conversation_id: request.conversation_id,
    in_reply_to: request.reply_with,
    // ... other fields
};
```

#### Capability Discovery

```rust
// Query what agents can handle image processing
let query = FipaMessage {
    performative: Performative::QUERY,
    capability: Capability::new("image-processing"),
    content: MessageContent::text("What image formats do you support?"),
    // ... routing finds all agents with image-processing capability
};
```

## Consequences

### Positive

- **Simplified for config agents**: No complex bidding logic or negotiation
  required
- **Capability-based decoupling**: Agents don't need to know about each other
  specifically
- **Standard compliance**: Follows FIPA-ACL conventions where practical
- **Conversation tracking**: Maintains context across multi-turn interactions
- **Future extensible**: Can add CNP and advanced features in post-1.0 releases
- **Natural language friendly**: Config agents can generate/parse FIPA messages
  through prompts

### Negative

- **No bidding in 1.0**: Contract Net Protocol deferred means no automatic
  resource optimization
- **Limited negotiation**: Only simple propose/accept/reject patterns
- **Prompt engineering required**: Config agents need good prompts to handle
  FIPA messages properly

### Risk Mitigation

- **Clear upgrade path**: CNP can be added without breaking existing message
  patterns
- **Capability evolution**: Capability registry can be enhanced without changing
  message format
- **Protocol versioning**: FIPA messages include protocol field for future
  compatibility

## Implementation Plan

### Phase 1: Core Messaging (1.0)

1. Basic FIPA message structure and performatives
2. Capability registry and routing
3. Conversation management and cleanup
4. Config agent runtime integration

### Phase 2: Developer Experience (1.0)

1. Message debugging and inspection tools
2. Capability discovery APIs
3. Conversation monitoring dashboard
4. Message pattern documentation and examples

### Phase 3: Advanced Features (Post-1.0)

1. Contract Net Protocol implementation
2. Advanced negotiation patterns
3. Message persistence and replay
4. Cross-instance message routing

## Alignment with Existing ADRs

- **ADR-0003 (FIPA Messaging Protocol)**: Updates with new lightweight approach
- **ADR-0012 (Pragmatic FIPA Subset)**: Extends with config-agent-optimized
  subset
- **ADR-0028 (Configuration-Driven Agents)**: Provides messaging layer for
  config agents
- **ADR-0011 (Capability Registration)**: Enhanced with capability-based routing

## Related Decisions

- ADR-0028: Configuration-Driven Agent Architecture (defines agents that use
  this messaging)
- ADR-0030: Embedded Memory System (agents can store conversation context in
  memory)

## References

- [FIPA-ACL Specification](http://www.fipa.org/specs/fipa00061/SC00061G.html)
- [FIPA Contract Net Protocol](http://www.fipa.org/specs/fipa00029/SC00029H.html)
  \- deferred to post-1.0
- Configuration agent messaging patterns from expert analysis
