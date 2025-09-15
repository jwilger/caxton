---
title: "ADR-0029: Lightweight Agent Messaging"
date: 2025-09-09
status: accepted
layout: adr
categories: [Architecture]
---

## Status

Accepted

## Context

With the shift to configuration-driven agents (ADR-0028), we need to reconsider
how agents communicate with each other. The previous agent messaging
implementation was
designed for compiled WASM agents, but config agents have different capabilities
and constraints.

### Current State Analysis

- **ADR-0003** established agent messaging protocol for WASM agents
- **ADR-0012** defined a pragmatic agent messaging subset for performance
- Configuration-driven agents need a simpler, more accessible messaging model
- Contract Net Protocol (CNP) adds significant complexity that may not be needed
  for 1.0

### Key Requirements

1. **Capability-based routing**: Agents should request capabilities, not
   specific agents
2. **Conversation tracking**: Maintain thread context across message exchanges
3. **Standard compliance**: Follow agent messaging patterns where practical
   for interoperability
4. **Config agent friendly**: Work naturally with prompt-based agent logic
5. **Future extensible**: Allow advanced features without breaking simple cases

## Decision

We will implement a **lightweight agent messaging system** optimized for
configuration-driven agents, with capability-based routing and deferred Contract
Net Protocol implementation.

### Message Structure Principles

Agent messages will include:

- **Performative**: Communicative intent (REQUEST, INFORM, QUERY, etc.)
- **Capability-based addressing**: Messages target capabilities, not specific
  agents
- **Conversation tracking**: Thread context maintained across exchanges
- **Standard metadata**: Protocol, language, ontology fields for
  interoperability

### Supported Communication Patterns (1.0 Scope)

**Basic Communication**:

- REQUEST: Ask agent to perform action
- INFORM: Share information
- QUERY: Ask for information

**Error Handling**:

- FAILURE: Indicate action failed
- NOT_UNDERSTOOD: Message not comprehensible

**Simple Negotiation**:

- PROPOSE: Suggest action/value
- ACCEPT_PROPOSAL: Agree to proposal
- REJECT_PROPOSAL: Decline proposal

**Deferred Features (Post-1.0)**:

- Contract Net Protocol (CFP, bidding)
- Advanced negotiation patterns (CONFIRM, DISCONFIRM, CANCEL)
- Subscription-based messaging

### Capability-Based Routing

Instead of addressing specific agents, messages target capabilities. Agents
declare what they can do (e.g., "data-analysis", "report-generation"), and the
message router finds suitable agents based on requested capabilities rather than
specific agent identities.

**Routing Strategies**:

- **Single recipient**: Route to best-matching agent for the capability
- **Broadcast**: Send to all agents that provide the capability
- **Load balancing**: Distribute requests across capable agents

### Configuration Agent Integration

Configuration-driven agents participate in agent messaging through their runtime
environment. The agent runtime formats incoming FIPA messages into natural
language prompts that config agents can understand, and parses their responses
back into FIPA message format.

**Key Integration Points**:

- **Prompt formatting**: Convert agent messages to natural language context
- **Response parsing**: Extract agent message performatives from agent responses
- **Conversation continuity**: Maintain thread context across exchanges
- **Error handling**: Guide agents to use appropriate failure responses

### Conversation Management

The system maintains conversation context across multi-turn message exchanges.
Each conversation tracks participants, message history, protocol state, and
activity timestamps.

**Core Capabilities**:

- **Thread tracking**: Link messages into conversation threads
- **Participant management**: Track which agents are involved in each
  conversation
- **Reply chain linking**: Connect request/response message pairs
- **Cleanup automation**: Remove stale conversations based on configurable
  timeouts
- **State management**: Track conversation progress and protocol state

### Usage Scenarios

**Request-Response Pattern**: Agent A sends a REQUEST to the "data-analysis"
capability asking for Q3 sales analysis. The system routes this to Agent B (a
data analyzer), which responds with an INFORM message containing the analysis
results. The conversation_id links these messages into a coherent thread.

**Capability Discovery**: Agents can QUERY capabilities to discover what
services are available. For example, querying "image-processing" returns
information about supported formats from all agents that provide image
processing capabilities.

**Simple Negotiation**: An agent can PROPOSE a solution or value, and other
agents can ACCEPT_PROPOSAL or REJECT_PROPOSAL, enabling basic collaborative
decision-making without complex bidding protocols.

## Consequences

### Positive

- **Simplified for config agents**: No complex bidding logic or negotiation
  required
- **Capability-based decoupling**: Agents don't need to know about each other
  specifically
- **Standard compliance**: Follows FIPA-ACL conventions where practical
- **Conversation tracking**: Maintains context across multi-turn interactions
- **Future extensible**: Can add CNP and advanced features in post-1.0 releases
- **Natural language friendly**: Config agents can generate/parse agent messages
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
- **Protocol versioning**: Agent messages include protocol field for future
  compatibility

## Implementation Plan

### Phase 1: Core Messaging (1.0)

1. Basic agent message structure and performatives
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

- **ADR-0003 (Agent Messaging Protocol)**: Updates with new lightweight approach
- **ADR-0012 (Pragmatic Agent Messaging Subset)**: Extends with
  config-agent-optimized
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
- [FIPA Contract Net Protocol]
  (http://www.fipa.org/specs/fipa00029/SC00029H.html)
  \- deferred to post-1.0
- Configuration agent messaging patterns from expert analysis
