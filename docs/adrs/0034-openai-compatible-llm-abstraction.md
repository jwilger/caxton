---
title: "ADR-0034: OpenAI-Compatible LLM Abstraction"
date: 2025-01-12
status: accepted
layout: adr
categories: [Architecture, Agent Architecture, LLM Integration]
---

## Status

Accepted

## Context

The Caxton platform's configuration-driven agents (as defined in ADR-0028)
need to communicate with various Large Language Model (LLM) providers to
execute their logic. Each provider has different API formats, authentication
methods, and capabilities, creating integration complexity and vendor lock-in
risks.

### Core Requirements

**Provider Flexibility**: Agents must be able to use different LLM providers
(OpenAI, Anthropic, Google, Ollama, etc.) without code changes.

**Zero Learning Curve**: Developers familiar with LLM APIs should be able to
configure agents immediately without learning proprietary formats.

**Progressive Disclosure**: Basic functionality should work with minimal
configuration while advanced features remain accessible.

**Fast Onboarding**: New users should achieve a working agent in under 60
seconds with zero external dependencies.

**Provider Switching**: Changing LLM providers should require only
configuration updates, not agent rewrites.

### Design Considerations

The choice of abstraction format impacts several aspects:

**Industry Standards**: Over 80% of developers have experience with OpenAI's
chat completion format, making it a de facto standard.

**Documentation Burden**: Using a well-known format reduces documentation
needs as developers can leverage existing knowledge.

**Provider Ecosystem**: Most LLM providers already offer OpenAI-compatible
endpoints or translation layers.

**Feature Parity**: Different providers offer unique capabilities that
shouldn't be hidden by the abstraction.

### Current Implementation Context

The existing architecture (ADR-0028 for configuration-driven agents and
ADR-0032 for TOML configuration) requires a standard way to configure LLM
interactions. The system needs to balance simplicity for common cases with
flexibility for provider-specific features.

## Decision

We will **adopt the OpenAI-compatible chat completion format** as the
standard LLM abstraction for all configuration-driven agents in Caxton.

### Abstraction Model

All LLM interactions will use the OpenAI chat completion format as the
common interface:

- **Message Format**: Standard roles (system, user, assistant) with
  content fields
- **Request Parameters**: Temperature, max_tokens, top_p, and other
  common parameters
- **Response Format**: Choices array with message content and finish
  reasons
- **Tool Calling**: Function/tool calling using OpenAI's format
- **Streaming**: Optional streaming responses following OpenAI's SSE
  format

### Provider Translation

Each LLM provider integration translates between their native format and
the OpenAI-compatible format:

- Input messages converted from OpenAI format to provider-specific
  format
- Provider responses translated back to OpenAI format
- Common parameters mapped to provider equivalents
- Provider-specific features exposed through extensions

### Progressive Disclosure Strategy

Configuration follows a progressive disclosure pattern:

- **Basic Level**: Only model name and optional API key required
- **Standard Level**: Common parameters like temperature and max_tokens
- **Advanced Level**: Provider-specific extensions for unique features
- **Expert Level**: Custom request transformers for complete control

### Mock Provider for Onboarding

A built-in mock LLM provider enables immediate experimentation:

- No API keys or external services required
- Deterministic responses for predictable demos
- Configurable response patterns for different scenarios
- Under 60-second first experience from install to working agent

### Rationale for OpenAI-Compatible Format

**Universal Familiarity**: The OpenAI format has become the industry
standard with the highest developer recognition.

**Zero Training Required**: Developers can immediately use their existing
knowledge without learning Caxton-specific formats.

**Ecosystem Compatibility**: Many tools, libraries, and services already
support this format, enabling easy integration.

**Provider Support**: Most LLM providers offer OpenAI-compatible endpoints
or clear migration guides.

**Future-Proof**: As the de facto standard, this format will likely remain
relevant even as new providers emerge.

## Alternatives Considered

### Alternative 1: Custom Caxton Format

Design a proprietary format optimized for Caxton's specific needs and
architectural patterns.

**Rejected** because it would:

- Create unnecessary learning curve for developers
- Require extensive documentation and examples
- Reduce community adoption due to unfamiliarity
- Increase maintenance burden for format evolution

### Alternative 2: Provider-Native Formats

Support each provider's native format directly without abstraction.

**Rejected** because it would:

- Require agents to be rewritten for different providers
- Increase configuration complexity exponentially
- Make provider switching difficult and error-prone
- Violate the configuration-driven philosophy of ADR-0028

### Alternative 3: GraphQL-Based Abstraction

Use GraphQL as a flexible query language for LLM interactions.

**Rejected** because it would:

- Add unnecessary complexity for simple use cases
- Require GraphQL knowledge in addition to LLM concepts
- Provide minimal benefits over REST-based approaches
- Increase implementation complexity significantly

## Consequences

### Positive

**Immediate Productivity**: Developers can start building agents using
familiar patterns without learning new concepts.

**Provider Portability**: Agents can switch between providers through
configuration changes without logic modifications.

**Reduced Documentation**: Can reference existing OpenAI documentation for
format details, reducing maintenance burden.

**Community Adoption**: Lower barrier to entry increases likelihood of
community contributions and adoption.

**Tooling Compatibility**: Existing OpenAI-compatible tools and libraries
work with minimal adaptation.

### Negative

**Format Limitations**: Some provider-specific features may not map cleanly
to the OpenAI format.

**Translation Overhead**: Converting between formats adds a small
performance overhead (typically <5ms).

**Version Management**: Must track OpenAI format changes and maintain
compatibility across versions.

**Feature Lag**: New provider features may take time to expose through the
abstraction layer.

### Migration Strategy

The OpenAI-compatible format provides clear migration paths:

1. **From OpenAI**: Direct configuration with no changes required
2. **From Other Providers**: Use provider extensions for specific features
3. **Between Providers**: Update provider configuration, adjust extensions
4. **To Custom Formats**: Future support for custom transformers if needed

## Implementation Approach

The implementation focuses on developer experience and extensibility:

1. **Core Abstraction**: Define OpenAI-compatible types and interfaces as
   the standard contract

2. **Provider Adapters**: Implement translation layers for each supported
   provider

3. **Extension System**: Allow provider-specific parameters through a
   `provider_extensions` configuration field

4. **Mock Provider**: Built-in provider with configurable responses for
   testing and demos

5. **Validation Layer**: Ensure configuration validity at startup to
   prevent runtime errors

## Alignment with Existing ADRs

- **ADR-0028 (Configuration-Driven Agents)**: OpenAI format enables
  simple TOML configuration for LLM interactions
- **ADR-0032 (TOML Agent Configuration)**: LLM configuration naturally
  fits in TOML with familiar parameter names
- **ADR-0004 (Minimal Core Philosophy)**: Using an existing standard
  reduces core complexity
- **ADR-0005 (MCP for External Tools)**: Similar philosophy of adopting
  established standards

## Related Decisions

- ADR-0028: Configuration-Driven Agent Architecture (defines
  configuration model)
- ADR-0032: TOML Agent Configuration Format (defines configuration
  syntax)
- ADR-0005: MCP for External Tools (precedent for adopting standards)
- ADR-0004: Minimal Core Philosophy (influences abstraction choices)

## References

- OpenAI Chat Completion API documentation
- Analysis of LLM provider API formats and compatibility
- Community feedback on configuration complexity in multi-agent systems
- Industry surveys on LLM API familiarity and usage patterns

---

**Implementation Status**: This ADR documents an architectural decision for
the LLM abstraction layer in configuration-driven agents. The
OpenAI-compatible format provides immediate familiarity while enabling
provider flexibility through configuration.
