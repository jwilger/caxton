---
title: "0003. FIPA Messaging Protocol"
date: 2025-07-31
status: proposed
layout: adr
categories: [Architecture, Technology]
---


Date: 2025-01-31

## Status

Accepted

## Context

Agent communication requires a standardized message format that:

- Supports various interaction patterns (request-reply,
  publish-subscribe, negotiations)
- Includes semantic information about message intent
- Enables interoperability between different agent implementations
- Has existing tooling and specifications

While we could design a custom protocol, this would require significant design
effort and limit interoperability with existing agent systems.

## Decision

We will use FIPA ACL (Agent Communication Language) as the messaging protocol
for agent communication in Caxton.

Key aspects:

- Messages use FIPA performatives (request, inform, propose, etc.)
- Standard fields: sender, receiver, content, language, ontology
- Support for conversation protocols and correlation
- Well-defined semantics for each message type
- Extensible for custom performatives if needed

## Consequences

### Positive

- **Industry standard**: FIPA has 20+ years of research and real-world use
- **Rich semantics**: Performatives clearly express agent intentions
- **Interoperability**: Can communicate with other FIPA-compliant systems
- **Proven patterns**: Established patterns for auctions, negotiations, etc.
- **Extensible**: Can add custom fields while maintaining compatibility

### Negative

- **Complexity**: FIPA ACL is more complex than simple JSON messages
- **Learning curve**: Developers must understand performatives
  and protocols
- **Overhead**: More verbose than minimal protocols
- **Legacy aspects**: Some FIPA concepts feel dated in modern systems

### Mitigations

- Provide high-level builders that hide FIPA complexity for
  simple cases
- Create comprehensive documentation with modern examples
- Support efficient binary encoding (not just XML/string formats)
- Focus on the subset of FIPA that provides clear value
- Allow raw message passing for scenarios that don't fit FIPA

## Alternatives Considered

### Custom JSON Protocol

- **Pros**: Simple, familiar to developers
- **Cons**: Would need to reinvent interaction patterns, no standards

### Protocol Buffers / gRPC

- **Pros**: Efficient, good tooling
- **Cons**: RPC-focused, lacks semantic richness for agent
  interactions

### Actor Model Messages (Erlang-style)

- **Pros**: Simple, proven in production
- **Cons**: Too low-level, no semantic information

### GraphQL Subscriptions

- **Pros**: Modern, good for pub-sub
- **Cons**: Designed for client-server, not peer-to-peer agents

## Implementation Example

```rust
// FIPA message structure
pub struct FipaMessage {
    performative: Performative,
    sender: AgentId,
    receiver: AgentId,
    content: Vec<u8>,
    language: Option<String>,
    ontology: Option<String>,
    protocol: Option<String>,
    conversation_id: Option<ConversationId>,
    reply_with: Option<MessageId>,
    in_reply_to: Option<MessageId>,
}

pub enum Performative {
    Request,
    Inform,
    QueryIf,
    Subscribe,
    Propose,
    AcceptProposal,
    RejectProposal,
    // ... other FIPA performatives
}

// High-level builder API
let msg = FipaMessage::request()
    .sender(agent_id)
    .receiver(other_agent)
    .content("What is the weather?")
    .build();
```

## References

- FIPA ACL Message Structure Specification
- FIPA Communicative Act Library Specification
- "Multi-Agent Systems" by Wooldridge
- JADE (Java Agent DEvelopment framework) documentation
