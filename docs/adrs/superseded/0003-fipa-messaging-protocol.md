---
title: "ADR-0003: Agent Messaging Protocol"
date: 2025-01-31
status: superseded
superseded_by: "ADR-0029: Lightweight Agent Messaging"
layout: adr
categories: [Architecture, Technology]
---


## Status

**Superseded** by
[ADR-0029: Lightweight Agent Messaging](0029-fipa-acl-lightweight-messaging.md)

This ADR established agent messaging protocols for compiled WASM agents.
ADR-0029
updates the approach specifically for configuration-driven agents with a
lightweight, capability-based messaging system.

## Context

Agent communication requires a standardized message format that:

- Supports various interaction patterns (request-reply, publish-subscribe,
  negotiations)
- Includes semantic information about message intent
- Enables interoperability between different agent implementations
- Has existing tooling and specifications

While we could design a custom protocol, this would require significant design
effort and limit interoperability with existing agent systems.

## Decision

We will use a structured agent communication protocol based on performatives
for agent communication in Caxton.

Key aspects:

- Messages use performatives (request, inform, propose, etc.)
- Standard fields: sender, receiver, content, language, ontology
- Support for conversation protocols and correlation
- Well-defined semantics for each message type
- Extensible for custom performatives if needed

## Consequences

### Positive

- **Proven concepts**: Performative-based messaging has 20+ years of
  research and real-world use
- **Rich semantics**: Performatives clearly express agent intentions
- **Interoperability**: Can communicate with other agent systems using
  similar patterns
- **Proven patterns**: Established patterns for auctions, negotiations, etc.
- **Extensible**: Can add custom fields while maintaining compatibility

### Negative

- **Complexity**: Performative-based messaging is more complex than simple
  JSON messages
- **Learning curve**: Developers must understand performatives and protocols
- **Overhead**: More verbose than minimal protocols
- **Legacy aspects**: Some traditional agent communication concepts feel
  dated in modern systems

### Mitigations

- Provide high-level builders that hide protocol complexity for simple cases
- Create comprehensive documentation with modern examples
- Support efficient binary encoding (not just XML/string formats)
- Focus on the subset of agent messaging patterns that provide clear value
- Allow raw message passing for scenarios that don't fit structured protocols

## Alternatives Considered

### Custom JSON Protocol

- **Pros**: Simple, familiar to developers
- **Cons**: Would need to reinvent interaction patterns, no standards

### Protocol Buffers / gRPC

- **Pros**: Efficient, good tooling
- **Cons**: RPC-focused, lacks semantic richness for agent interactions

### Actor Model Messages (Erlang-style)

- **Pros**: Simple, proven in production
- **Cons**: Too low-level, no semantic information

### GraphQL Subscriptions

- **Pros**: Modern, good for pub-sub
- **Cons**: Designed for client-server, not peer-to-peer agents

## Implementation Considerations

The FIPA messaging approach requires:

- **Message structure standardization**: All agent messages must include
  standard fields such as performative, sender, receiver, and conversation
  context
- **Performative semantics**: The system must support standard
  performatives (request, inform, propose, etc.) with their defined meanings
- **Conversation correlation**: Messages must be linkable through conversation
  IDs and reply relationships to support multi-turn interactions
- **Developer experience balance**: While maintaining protocol consistency,
  provide
  simplified APIs for common messaging patterns to reduce complexity

## References

- FIPA ACL Message Structure Specification
- FIPA Communicative Act Library Specification
- "Multi-Agent Systems" by Wooldridge
- JADE (Java Agent DEvelopment framework) documentation
