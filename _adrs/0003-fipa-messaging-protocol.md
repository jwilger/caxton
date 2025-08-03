---
layout: adr
title: "0003. FIPA Messaging Protocol"
date: 2025-01-31T00:00:00Z
status: proposed
categories: [Architecture, Protocol]
tags: [fipa, messaging, communication, standards]
---

# 0003. FIPA Messaging Protocol

Date: 2025-01-31

## Status

Proposed

## Context

Multi-agent systems need standardized communication protocols to enable:
- Semantic messaging beyond simple request/response
- Interoperability between different agent implementations
- Well-defined conversation patterns and state machines
- Contract negotiation and complex multi-party interactions

Ad-hoc messaging formats lead to incompatible agents and complex integration challenges.

## Decision

We will implement FIPA (Foundation for Intelligent Physical Agents) messaging protocols as the standard communication mechanism for Caxton agents.

Key aspects:
- FIPA ACL (Agent Communication Language) message structure
- Support for core FIPA performatives (inform, request, propose, accept-proposal, etc.)
- FIPA interaction protocols (Contract Net, Request-Reply, Subscribe-Notify)
- Conversation tracking with unique conversation IDs
- Ontology support for semantic message content

## Consequences

### Positive

- **Industry standard**: FIPA is the established standard for agent communication
- **Rich semantics**: Performatives express intent, not just data transfer
- **Proven patterns**: Well-defined interaction protocols for common scenarios
- **Interoperability**: Potential compatibility with other FIPA-compliant systems
- **Conversation management**: Built-in support for tracking multi-turn interactions

### Negative

- **Learning curve**: Developers must understand FIPA concepts and protocols
- **Complexity**: More complex than simple request/response patterns
- **Overhead**: FIPA messages carry more metadata than minimal protocols
- **Limited adoption**: FIPA is not widely used in modern systems

### Mitigations

- Provide high-level APIs that abstract FIPA complexity
- Implement common patterns as reusable templates
- Excellent documentation and examples
- Support for simplified messaging modes when FIPA semantics aren't needed

## Alternatives Considered

### gRPC
- **Pros**: High performance, strong typing, wide adoption
- **Cons**: RPC-style, limited semantic messaging capabilities

### HTTP/REST
- **Pros**: Universal adoption, simple tooling
- **Cons**: Stateless, no conversation management, limited semantics

### Message Queues (AMQP, Kafka)
- **Pros**: Reliable delivery, scalability
- **Cons**: Infrastructure complexity, no semantic messaging

### Custom Protocol
- **Pros**: Tailored to our needs, maximum control
- **Cons**: No interoperability, reinventing the wheel

## Implementation Notes

```rust
// FIPA message structure
pub struct FipaMessage {
    pub performative: Performative,
    pub sender: AgentId,
    pub receiver: AgentId,
    pub conversation_id: ConversationId,
    pub reply_with: Option<String>,
    pub in_reply_to: Option<String>,
    pub content: MessageContent,
    pub ontology: Option<String>,
    pub language: Option<String>,
}

// Example Contract Net Protocol
async fn initiate_contract_net(&self, task: Task) -> Result<ContractResult, Error> {
    // Send call-for-proposals
    let cfp = FipaMessage::new(
        Performative::Cfp,
        self.id(),
        recipients,
        conversation_id,
    );
    
    // Collect proposals
    let proposals = self.send_and_collect_responses(cfp, timeout).await?;
    
    // Select best proposal
    let winner = self.evaluate_proposals(proposals)?;
    
    // Send accept-proposal
    let accept = FipaMessage::new(
        Performative::AcceptProposal,
        self.id(),
        winner.sender,
        conversation_id,
    );
    
    self.send(accept).await
}
```

## References

- FIPA Agent Communication Language Specification
- FIPA Interaction Protocol Library Specification
- "Multi-Agent Systems: A Modern Approach" by Gerhard Weiss