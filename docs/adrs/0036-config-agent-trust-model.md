---
title: "ADR-0036: Config Agent Trust Model"
date: 2025-01-17
status: accepted
layout: adr
categories: [Architecture, Security, Config Agents, Trust Model]
---

## Status

Accepted

## Context

The Caxton platform's configuration-driven agents (ADR-0028) require a
security and trust model that balances developer productivity with operational
safety. This decision impacts how config agents execute, what isolation
boundaries exist, and how security controls are enforced.

### Core Requirements

**5-10 Minute Setup Promise**: Caxton must maintain its commitment to rapid
deployment without complex security configuration or infrastructure setup.

**Security Without Theater**: Security controls must provide actual value,
not performative isolation that adds complexity without meaningful protection.

**Developer Productivity**: The trust model must enable rapid development and
iteration on agent capabilities without bureaucratic overhead.

**Tool Boundary Security**: Security enforcement must occur at
tool/capability boundaries where actual risks exist, not at arbitrary
process boundaries.

**Market Alignment**: The model must align with industry-standard approaches
used by successful platforms (AutoGen, LangChain, CrewAI).

### Design Considerations

The trust model decision impacts several critical aspects:

**Industry Precedent**: Leading AI orchestration frameworks (AutoGen v0.4,
LangChain, CrewAI) run agents in shared processes with security at the tool
layer, validating this approach's practicality and market acceptance.

**Cloud Provider Guidance**: AWS, Google Cloud, and Azure explicitly state
that containers are NOT security boundaries, reinforcing that process
isolation alone doesn't provide meaningful security.

**User Mental Model**: Users conceptualize security in terms of what agents
can DO (capabilities), not HOW they run (process boundaries).

**Resource Efficiency**: Shared process execution eliminates the overhead of
inter-process communication, serialization, and context switching.

**Debugging Experience**: Shared process model enables straightforward
debugging, logging, and observability without complex distributed tracing.

### Current Implementation Context

The configuration-driven agent architecture (ADR-0028) establishes agents as
TOML configurations that orchestrate tool usage. The MCP integration
(ADR-0005) provides the tool layer where actual capabilities exist. The
single-codebase architecture (ADR-0027) emphasizes simplicity and
maintainability.

## Decision

We will **run config agents in the shared Caxton process without
isolation**, with security boundaries enforced at the MCP tool layer where
actual capabilities exist.

### Trust Model Architecture

Config agents operate as trusted orchestrators within the Caxton process:

- **Shared Process Execution**: All config agents run in the main Caxton process
- **No Arbitrary Code Execution**: Config agents cannot run arbitrary code,
  only orchestrate tools
- **Tool-Level Security**: Security boundaries enforced at MCP tool invocations
- **WASM for Tool Isolation**: MCP tools run in WASM sandboxes when security is required
- **Capability-Based Security**: Security model based on what tools agents can access

### Security Control Points

Security enforcement occurs at meaningful boundaries:

- **Tool Allowlists**: Agents explicitly declare which MCP tools they can access
- **Rate Limiting**: Per-agent and per-tool rate limits prevent resource exhaustion
- **Audit Logging**: All tool invocations logged with agent identity and parameters
- **Resource Governance**: Memory and CPU limits enforced at tool execution level
- **Input Validation**: Tool parameters validated before execution

### Configuration Trust Model

Config agent TOML files are treated as configuration, not code:

- **Static Declaration**: Agents declare capabilities at configuration time
- **No Dynamic Code**: TOML cannot contain executable code or scripts
- **Prompt Templates**: LLM prompts are data, not code
- **Tool References**: Only reference pre-registered MCP tools

### Progressive Security Model

Security can be enhanced based on deployment context:

- **Development Mode**: Minimal restrictions for rapid iteration
- **Production Mode**: Full audit logging, rate limiting, and tool restrictions
- **Enterprise Mode**: Optional external tool sandboxing for regulatory
  compliance
- **Future Isolation**: Container/VM isolation available as opt-in for
  specific use cases

### Market Positioning

Position the trust model as pragmatic and industry-aligned:

- **"Security Where It Matters"**: Focus security on actual capability
  boundaries
- **"Industry Standard Approach"**: Align with AutoGen, LangChain, and CrewAI
- **"Progressive Enhancement"**: Start simple, add security controls as needed
- **"No Security Theater"**: Reject performative isolation that doesn't
  improve actual security

## Alternatives Considered

### Alternative 1: Full Process Isolation

Run each config agent in a separate OS process with IPC communication.

**Rejected** because it would:

- Add 10-100ms latency to every agent interaction
- Complicate debugging and observability significantly
- Violate the 5-10 minute setup promise
- Provide no meaningful security benefit (processes aren't security boundaries)

### Alternative 2: Container Isolation

Run each config agent in a container with orchestration layer.

**Rejected** because it would:

- Require Docker/Podman installation and configuration
- Add significant operational complexity
- Contradict cloud provider guidance on container security
- Create resource overhead without security benefit

### Alternative 3: WASM for Config Agents

Compile config agents to WASM for sandboxed execution.

**Rejected** because it would:

- Require ahead-of-time compilation of configurations
- Eliminate dynamic prompt engineering capabilities
- Add unnecessary complexity for orchestration-only agents
- Solve a non-existent security problem (agents can't execute code)

### Alternative 4: Mandatory Capability Review

Require manual security review for all agent configurations.

**Rejected** because it would:

- Create bureaucratic overhead killing developer productivity
- Violate rapid iteration promise
- Provide false sense of security (configurations aren't code)
- Misalign with market expectations

## Consequences

### Positive

**Maintains Simplicity Promise**: 5-10 minute setup preserved without
security configuration complexity.

**Industry Alignment**: Matches successful patterns from AutoGen, LangChain,
and CrewAI.

**Performance Optimization**: Zero overhead for agent orchestration and
communication.

**Developer Experience**: Simple debugging, logging, and observability
without distributed complexity.

**Progressive Security**: Can add security controls incrementally based on
actual needs.

**Resource Efficiency**: Minimal memory and CPU overhead for agent execution.

### Negative

**Perception Risk**: Some users may expect process isolation despite
industry precedent.

**Shared Failure Domain**: Misbehaving agent could impact other agents in same process.

**Limited Resource Isolation**: Cannot enforce strict per-agent memory
limits at process level.

**Audit Complexity**: Must carefully track agent identity through shared execution.

### Security Mitigations

Address potential concerns through tool-layer controls:

1. **Tool Sandboxing**: MCP tools run in WASM when handling untrusted data
2. **Rate Limiting**: Prevent resource exhaustion through configurable limits
3. **Audit Trail**: Comprehensive logging of all tool invocations
4. **Input Validation**: Strict parameter validation at tool boundaries
5. **Capability Principle**: Agents only access explicitly granted tools

## Implementation Approach

The implementation focuses on security at the right layer:

1. **Tool Registry**: Centralized registry of available MCP tools with security metadata

2. **Capability Grants**: Explicit allowlist of tools per agent configuration

3. **Audit System**: Structured logging of all tool invocations with context

4. **Rate Limiter**: Token bucket implementation for per-agent and per-tool limits

5. **Progressive Controls**: Configuration flags for enabling additional
   security layers

## Alignment with Existing ADRs

- **ADR-0028 (Configuration-Driven Agents)**: Reinforces agents as
  orchestrators, not code executors
- **ADR-0005 (MCP for External Tools)**: Establishes tool layer as security boundary
- **ADR-0027 (Single Codebase)**: Maintains architectural simplicity without
  isolation complexity
- **ADR-0004 (Minimal Core Philosophy)**: Avoids unnecessary security theater

## Related Decisions

- ADR-0028: Configuration-Driven Agent Architecture (establishes config
  agent model)
- ADR-0005: MCP for External Tools (defines tool security boundary)
- ADR-0002: WebAssembly for Agent Isolation (WASM for compiled agents, not
  config)
- ADR-0016: Security Architecture (overall security model)

## References

- AutoGen v0.4 architecture documentation on shared process execution
- LangChain security model and tool sandboxing approach
- AWS/Google Cloud guidance on container security boundaries
- Industry analysis of AI orchestration framework security models

---

**Implementation Status**: This ADR documents the decision to run config
agents in a shared process with security enforced at the tool layer. This
approach maintains the 5-10 minute setup promise while providing meaningful
security controls where they matter, aligning with industry best practices
from AutoGen, LangChain, and cloud provider guidance.
