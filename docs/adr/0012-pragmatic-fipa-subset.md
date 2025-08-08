---
layout: adr
title: "0012. Pragmatic FIPA Subset"
status: accepted
date: 2025-08-08
---

# ADR-0012: Pragmatic FIPA Subset

## Status
Accepted

## Context
FIPA (Foundation for Intelligent Physical Agents) specifications were developed in the late 1990s by academic researchers working on multi-agent systems. While FIPA provides useful patterns for agent communication, it includes significant complexity that doesn't add value in modern production systems.

Caxton needs reliable agent coordination, not academic purity.

## Decision
Caxton implements a pragmatic subset of FIPA, keeping what's useful and discarding academic baggage.

### What We Keep from FIPA

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
**FIPA Says**: Define formal ontologies in OWL/RDF for semantic interoperability.
**We Do**: Use JSON schemas and TypeScript/Rust types.
**Why**: Modern type systems and JSON Schema provide better developer experience and tooling.

#### 2. Content Languages (SL, KIF, FIPA-RDF)
**FIPA Says**: Use semantic languages like FIPA-SL for content.
**We Do**: Use JSON exclusively.
**Why**: JSON has won. Every language has excellent JSON support. Semantic languages add complexity without practical benefit.

#### 3. Protocol Negotiation
**FIPA Says**: Agents negotiate which protocols to use.
**We Do**: FIPA-ACL only, no negotiation.
**Why**: One protocol reduces complexity. Agents either speak FIPA-ACL or they don't belong in Caxton.

#### 4. Directory Facilitator (DF) / Agent Management System (AMS)
**FIPA Says**: Complex service discovery with yellow pages, white pages, etc.
**We Do**: Direct capability registration with the orchestrator.
**Why**: Kubernetes/cloud-native patterns handle service discovery better.

#### 5. Agent Communication Language (ACL) Representations
**FIPA Says**: Support bit-efficient, XML, and string encodings.
**We Do**: JSON over HTTP/WebSocket only.
**Why**: One serialization format. Modern networks make bit-efficiency irrelevant.

#### 6. Complex Performatives
Rarely-used performatives that add cognitive overhead:
- `PROPAGATE` - Just use pub-sub
- `PROXY` - Handle at infrastructure layer
- `CFP` with complex auction protocols - YAGNI
- `DISCONFIRM` / `CONFIRM` - Use INFORM with success/failure

#### 7. Mobility Specifications
**FIPA Says**: Agents can move between platforms.
**We Do**: Agents are stateless WebAssembly modules.
**Why**: Container orchestration handles "mobility" better.

## Consequences

### Positive
- **Dramatically simpler implementation** - Less code, fewer bugs
- **Better developer experience** - JSON and types instead of ontologies
- **Modern tooling** - Standard HTTP/JSON tools work
- **Clearer mental model** - No academic abstractions
- **Faster onboarding** - Developers understand JSON APIs

### Negative
- **No FIPA compliance certification** - We don't care
- **Can't interoperate with "pure" FIPA systems** - They don't exist in production
- **Less semantic richness** - Solved with good API design

### Neutral
- Still agent-based architecture
- Still message-passing coordination
- Still using proven interaction patterns

## Implementation Example

### What FIPA Wants
```xml
<fipa-message ontology="logistics-ontology" language="fipa-sl">
  <performative>REQUEST</performative>
  <content>
    ((action
      (agent-identifier :name dispatcher@platform)
      (deliver
        :item (package :id pkg-123 :weight 5kg)
        :destination (location :address "123 Main St"))))
  </content>
</fipa-message>
```

### What We Actually Do
```json
{
  "performative": "request",
  "sender": "dispatcher",
  "receiver": "delivery-agent",
  "content": {
    "action": "deliver",
    "package_id": "pkg-123",
    "destination": "123 Main St"
  }
}
```

## Guidelines for Future Decisions

When evaluating FIPA specifications:
1. Does it solve a real production problem?
2. Is there a simpler modern alternative?
3. Will developers understand it immediately?
4. Can we implement it with standard tools?

If any answer is "no", we don't need it.

## References
- [FIPA Specifications](http://www.fipa.org/repository/standardspecs.html)
- [Original FIPA Message Protocol (ADR-0003)](0003-fipa-messaging-protocol.md)
- [Capability Registration in Code (ADR-0011)](0011-capability-registration-in-code.md)

## Notes
FIPA was groundbreaking for its time (1996-2005), but software engineering has evolved. We honor its core insights while adapting to modern realities. Academic purity is not a goal; production reliability is.
