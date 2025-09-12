---
title: "ADR-0039: LLM Failure Handling Strategy"
date: 2025-01-17
status: accepted
layout: adr
categories: [Architecture, User Experience, LLM Integration, Error Handling]
---

## Status

Accepted

## Context

Caxton's configuration-driven agents rely heavily on Large Language Model
(LLM) interactions for intelligent behavior. However, LLM services are
inherently unreliable, exhibiting various failure modes including timeouts,
rate limits, service outages, and quality degradation. The system must handle
these failures in a way that maintains both user trust and developer
productivity.

### Core Challenge

The LLM failure handling challenge involves balancing multiple concerns:

- **User Agency**: Developers expect control over their tools and processes
- **Response Time**: Extended timeouts (120+ seconds) are perceived as system
  failures
- **Quality Transparency**: Silent fallbacks to lower-quality models break
  user trust
- **Cost Awareness**: Token usage and API costs need visibility before retry
  decisions
- **System Reliability**: Automatic recovery mechanisms can mask underlying
  problems
- **Developer Experience**: Failure handling should not interrupt development
  flow

### Current Industry Approaches

**Automatic Failover**: Systems like LangChain automatically switch between
providers, but this creates unpredictable quality variations and unexpected
costs.

**Infinite Retry**: Some systems retry indefinitely with exponential backoff,
leading to multi-minute waits that frustrate users.

**Silent Degradation**: Many tools silently fall back to lower-quality
models, producing subtly incorrect results that erode trust.

**Hard Failure**: Some systems immediately fail on any LLM error, forcing
manual intervention for transient issues.

### Requirements

The solution must provide:

- **Fast Feedback**: Users know within seconds if something is wrong
- **User Control**: Explicit choice over retry, failover, or cancellation
- **Cost Transparency**: Clear visibility into token usage before decisions
- **Quality Preservation**: No silent quality degradation
- **Progress Indication**: Visual feedback during longer operations
- **Clear Communication**: Specific error messages that enable informed
  decisions

## Decision

We will implement a **fail-fast with user control** strategy for LLM
failures, prioritizing developer agency and transparency over automatic
reliability.

### Failure Handling Strategy

**Single Retry Policy**: One automatic retry with a 5-second timeout per
attempt (15 seconds maximum total).

**Progress Indication**: Visual progress indicator appears after 3 seconds of
waiting.

**User Choice on Failure**: Present three explicit options:

1. Try Again - Retry with the same provider
2. Use Backup - Switch to alternative provider (if configured)
3. Cancel - Abort the operation

**Cost Transparency**: Display token usage and estimated cost before retry or
failover decisions.

**No Silent Failover**: Never automatically switch between providers of
different quality tiers.

**Clear Error Messages**: Specific failure reasons (timeout, rate limit,
authentication, service error) guide user decisions.

### User Experience Flow

```text
Request → LLM Provider
  ├─ Success (< 3s) → Continue
  ├─ Waiting (3-5s) → Show progress indicator
  └─ Failure → Retry once
      ├─ Success → Continue
      └─ Failure → Present options:
          ├─ [Try Again] → Manual retry
          ├─ [Use Backup: Ollama] → Switch provider
          └─ [Cancel] → Abort operation
```

### Error Message Format

```text
❌ OpenAI API timeout after 15 seconds
   Used: 1,247 tokens (~$0.02)

   Options:
   [1] Try Again with OpenAI
   [2] Use Backup (Ollama - Local, Free, Lower Quality)
   [3] Cancel Operation

   Choice:
```

## Consequences

### Positive Consequences

- **Developer Trust**: Transparent failures maintain confidence in the system
- **Fast Feedback Loop**: 15-second maximum wait preserves development flow
- **Informed Decisions**: Users understand costs and quality trade-offs
- **Predictable Behavior**: No surprising quality degradation or unexpected
  costs
- **Debugging Clarity**: Explicit failures make problems easier to diagnose
- **User Empowerment**: Developers control their tools rather than being
  controlled by them

### Negative Consequences

- **Manual Intervention Required**: Users must actively choose recovery
  strategies
- **Reduced Automation**: Less "magical" automatic recovery compared to
  competitors
- **Potentially Lower Uptime**: System won't hide provider issues through
  automatic failover
- **Learning Curve**: Users must understand provider differences and
  trade-offs

### Trade-off Analysis

We explicitly trade automatic reliability for user control because:

1. **Developers value agency**: Being forced to wait or accept degraded
   quality frustrates power users
2. **Quality matters**: Silent GPT-4 to Ollama failover can produce subtle
   errors that waste more time than explicit failures
3. **Cost control**: Automatic retries can lead to unexpected API charges
4. **Trust through transparency**: Seeing and controlling failures builds confidence

## Implementation Approach

Implementation follows these principles:

1. **Timeout Configuration**: 5-second per-attempt timeout, configurable per
   provider
2. **Progress Indication**: Standard progress indicator component after 3
   seconds
3. **Error Classification**: Distinguish transient (retry-worthy) from
   permanent failures
4. **Cost Tracking**: Accumulate token usage across attempts for display
5. **Provider Abstraction**: Consistent failure handling across all LLM
   providers

## Alignment with Existing ADRs

This decision reinforces:

- **ADR-0001 (Observability First)**: Explicit failures provide clear
  observability
- **ADR-0034 (OpenAI Compatible LLM)**: Consistent error handling across
  providers
- **ADR-0004 (Minimal Core)**: Simple retry logic over complex failure
  orchestration
- **ADR-0028 (Config Agents)**: User-configured fallback strategies in agent
  TOML

## Industry Precedent

Our approach aligns with tools that prioritize developer control:

- **GitHub Copilot**: Shows explicit "Copilot unavailable" rather than silent
  degradation
- **Vercel**: Displays deployment failures immediately with clear retry options
- **Stripe CLI**: Fast failures with explicit retry commands rather than
  automatic recovery
- **Docker**: Quick timeout on registry pulls with manual retry control

These tools demonstrate that developers prefer fast, transparent failures
over slow, automatic recovery.

## Future Considerations

- **Circuit Breaker Pattern**: Skip providers that repeatedly fail for a
  cooldown period
- **Health Dashboard**: Real-time provider status visibility
- **Fallback Chains**: User-defined provider preference lists
- **Retry Budgets**: Configurable retry limits per time window
- **Quality Metrics**: Track and display quality differences between
  providers

## Security Considerations

- **API Key Protection**: Never log or display API keys in error messages
- **Token Limit Enforcement**: Prevent retry loops from consuming excessive
  tokens
- **Rate Limit Respect**: Honor provider rate limits without aggressive
  retry
- **Error Sanitization**: Remove sensitive data from error messages

## References

- [Fail-Fast Principle](https://en.wikipedia.org/wiki/Fail-fast)
- [User Agency in Developer Tools](https://www.nngroup.com/articles/user-control-freedom/)
- ADR-0034: OpenAI Compatible LLM Abstraction
- ADR-0001: Observability First Architecture
- ADR-0028: Configuration-Driven Agent Architecture
