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
updates the messaging approach specifically for configuration-driven agents
with a
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

We will use a structured agent messaging protocol based on proven
communication patterns
for agent communication in Caxton.

Key aspects:

- Messages use semantic performatives (request, inform, propose, etc.)
- Standard fields: sender, receiver, content, language, ontology
- Support for conversation protocols and correlation
- Well-defined semantics for each message type
- Extensible for custom performatives if needed

## Consequences

### Positive

- **Research foundation**: Agent messaging has 20+ years of research and
  real-world use
- **Rich semantics**: Performatives clearly express agent intentions
- **Interoperability**: Can communicate with other agent messaging systems
- **Proven patterns**: Established patterns for auctions, negotiations, etc.
- **Extensible**: Can add custom fields while maintaining compatibility

### Negative

- **Complexity**: Structured agent messaging is more complex than simple
  JSON messages
- **Learning curve**: Developers must understand performatives and protocols
- **Overhead**: More verbose than minimal protocols
- **Legacy aspects**: Some traditional concepts feel dated in modern systems

### Mitigations

- Provide high-level builders that hide messaging complexity for simple cases
- Create comprehensive documentation with modern examples
- Support efficient binary encoding (not just XML/string formats)
- Focus on the subset of agent messaging that provides clear value
- Allow raw message passing for scenarios that don't fit structured messaging

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

The agent messaging approach requires:

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

- Agent Communication Message Structure Research
- Communicative Act Library Specifications
- "Multi-Agent Systems" by Wooldridge
- Agent platform documentation and research
