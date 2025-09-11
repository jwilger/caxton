---
title: "ADR-0012: Pragmatic Agent Messaging"
date: 2025-08-08
status: superseded
superseded_by: "ADR-0029: Lightweight Agent Messaging"
layout: adr
categories: [Architecture]
---


## Status

**Superseded** by
[ADR-0029: Lightweight Agent Messaging](0029-fipa-acl-lightweight-messaging.md)

This ADR defined a pragmatic agent messaging approach for compiled WASM
agents. ADR-0029
provides an updated lightweight messaging implementation specifically optimized
for configuration-driven agents with capability-based routing and simplified
interaction patterns.

## Relationship to ADR-0003

This ADR refines and supersedes certain aspects of
[ADR-0003: Agent Messaging Protocol](0003-fipa-messaging-protocol.md). While
ADR-0003 established agent messaging as our foundation, this ADR pragmatically
adapts messaging patterns for production use by:

- Keeping only the valuable core patterns
- Replacing academic complexity with modern alternatives
- Focusing on developer experience and operational simplicity

ADR-0003 remains valid for understanding why we chose these messaging
patterns. This ADR
defines **how** we implement them practically.

## Context

Traditional agent messaging specifications were developed
in the late 1990s by academic researchers working on multi-agent systems. While
these specifications provide useful patterns for agent communication, they
include significant
complexity that doesn't add value in modern production systems.

Caxton needs reliable agent coordination, not academic purity.

### Relationship to SWIM Protocol

Agent messaging operates at the **application layer** for semantic agent
messaging, while
SWIM operates at the **infrastructure layer** for cluster coordination. They are
complementary:

- **SWIM**: Manages which Caxton instances are alive and where agents are
  located
- **Agent Messaging**: Defines how agents communicate once SWIM has
  established routing
- **Clear Separation**: SWIM handles infrastructure concerns, agent
  messaging handles
  business logic

See
[ADR-0015: Distributed Protocol Architecture]
(0015-distributed-protocol-architecture.md)
for detailed protocol interaction.

## Decision

Caxton implements a pragmatic agent messaging approach, keeping what's
useful and
discarding academic baggage.

### What We Keep from Traditional Agent Messaging

#### 1. Core Message Performatives

The basic speech acts that are genuinely useful:

- `REQUEST` - Ask an agent to perform an action
- `INFORM` - Share information
- `QUERY` - Ask for information
- `PROPOSE` / `ACCEPT_PROPOSAL` / `REJECT_PROPOSAL` - Negotiation
- `FAILURE` - Report inability to complete request
- `NOT_UNDERSTOOD` - Message parsing/comprehension failure

#### 2. Message Structure

Basic fields that enable routing and correlation:

- `performative` - Message type
- `sender` / `receiver` - Routing
- `content` - Payload (JSON, not SL or KIF)
- `conversation_id` - Correlation across messages
- `reply_with` / `in_reply_to` - Request/response pairing

#### 3. Interaction Patterns

- Request-Response protocol
- Contract Net for task distribution
- Basic publish-subscribe

### What We Explicitly Reject

#### 1. Ontologies

**Traditional Approach**: Define formal ontologies in OWL/RDF for semantic
interoperability. **We Do**: Use JSON schemas and TypeScript/Rust types.
**Why**: Modern type systems and JSON Schema provide better developer experience
and tooling.

#### 2. Content Languages (SL, KIF)

**Traditional Approach**: Use semantic languages for content. **We Do**: Use
JSON exclusively. **Why**: JSON has won. Every language has excellent JSON
support. Semantic languages add complexity without practical benefit.

#### 3. Protocol Negotiation

**Traditional Approach**: Agents negotiate which protocols to use. **We Do**:
Agent messaging
only, no negotiation. **Why**: One protocol reduces complexity. Agents either
speak our agent messaging protocol or they don't belong in Caxton.

#### 4. Directory Facilitator / Agent Management System

**Traditional Approach**: Complex service discovery with yellow pages,
white pages, etc.
**We Do**: Direct capability registration with the orchestrator. **Why**:
Kubernetes/cloud-native patterns handle service discovery better.

#### 5. Agent Communication Language Representations

**Traditional Approach**: Support bit-efficient, XML, and string encodings.
**We Do**: JSON
over HTTP/WebSocket only. **Why**: One serialization format. Modern networks
make bit-efficiency irrelevant.

#### 6. Complex Performatives

Rarely-used performatives that add cognitive overhead:

- `PROPAGATE` - Just use pub-sub
- `PROXY` - Handle at infrastructure layer
- `CFP` with complex auction protocols - YAGNI
- `DISCONFIRM` / `CONFIRM` - Use INFORM with success/failure

#### 7. Mobility Specifications

**Traditional Approach**: Agents can move between platforms. **We Do**:
Agents are
stateless WebAssembly modules. **Why**: Container orchestration handles
"mobility" better.

## Consequences

### Positive

- **Dramatically simpler implementation** - Less code, fewer bugs
- **Better developer experience** - JSON and types instead of ontologies
- **Modern tooling** - Standard HTTP/JSON tools work
- **Clearer mental model** - No academic abstractions
- **Faster onboarding** - Developers understand JSON APIs

### Negative

- **No academic standards compliance certification** - We don't care
- **Can't interoperate with "pure" academic agent systems** - They don't
  exist in
  production
- **Less semantic richness** - Solved with good API design

### Neutral

- Still agent-based architecture
- Still message-passing coordination
- Still using proven interaction patterns

## Conceptual Comparison

Our pragmatic approach chooses JSON over traditional semantic languages
(SL, KIF) and
complex ontologies. This results in significantly simpler message formats that
developers can immediately understand and work with using standard tooling,
while preserving the core communication patterns that make agent messaging
valuable.

## Guidelines for Future Decisions

When evaluating agent messaging specifications:

1. Does it solve a real production problem?
2. Is there a simpler modern alternative?
3. Will developers understand it immediately?
4. Can we implement it with standard tools?

If any answer is "no", we don't need it.

## References

- [Original Agent Message Protocol (ADR-0003)](0003-fipa-messaging-protocol.md)
- [Capability Registration in Code (ADR-0011)]
  (0011-capability-registration-in-code.md)

## Notes

Agent messaging standards were groundbreaking for their time (1996-2005),
but software engineering has
evolved. We honor their core insights while adapting to modern realities.
Academic
purity is not a goal; production reliability is.
