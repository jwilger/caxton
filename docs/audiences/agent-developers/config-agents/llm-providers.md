---
title: "LLM Provider Integration"
date: 2025-09-10
layout: guide
categories: [Agent Developers, Configuration, Integration]
difficulty: beginner
audience: agent-developers
---

> **🚧 Implementation Status**
>
> The pluggable LLM provider system represents core architecture from ADR-28.
> This documentation serves as the technical specification for the
> multi-provider
> LLM integration currently under development.
>
> **Target**: Seamless integration with multiple LLM providers
> **Status**: Provider abstraction layer and OpenAI reference implementation
> in progress

## Overview - **Beginner**

Caxton's pluggable LLM provider system enables configuration agents to
integrate with any LLM/SLM API through configurable providers. The system is
designed with no vendor lock-in, allowing users to choose their preferred model
provider while maintaining consistent agent behavior.

## Supported Providers

### OpenAI - **Beginner**

Full-featured integration with OpenAI's API:

```yaml
llm:
  provider: openai
  model: gpt-4                    # or gpt-4-turbo, gpt-3.5-turbo
  temperature: 0.7
  max_tokens: 2000
  top_p: 1.0
  frequency_penalty: 0.0
  presence_penalty: 0.0

  # OpenAI-specific configuration
  openai:
    api_key: ${OPENAI_API_KEY}    # Environment variable
    organization: ${OPENAI_ORG}   # Optional
    base_url: https://api.openai.com/v1  # Default
    timeout: 30s
    max_retries: 3
```

**Available Models**:

- `gpt-4`: Most capable, higher cost
- `gpt-4-turbo`: Fast and capable, good balance
- `gpt-3.5-turbo`: Fast and cost-effective

### Anthropic - **Beginner**

Integration with Claude models:

```yaml
llm:
  provider: anthropic
  model: claude-3-sonnet          # or claude-3-opus, claude-3-haiku
  temperature: 0.5
  max_tokens: 1500

  # Anthropic-specific configuration
  anthropic:
    api_key: ${ANTHROPIC_API_KEY}
    base_url: https://api.anthropic.com
    timeout: 45s
    max_retries: 3
    version: "2023-06-01"         # API version
```

**Available Models**:

- `claude-3-opus`: Most capable, highest cost
- `claude-3-sonnet`: Balanced performance and cost
- `claude-3-haiku`: Fast and cost-effective

### Local Models - **Intermediate**

Integration with locally hosted models via Ollama, vLLM, or similar:

```yaml
llm:
  provider: local
  model: llama2                   # or mistral, codellama, etc.
  temperature: 0.8
  max_tokens: 1000

  # Local provider configuration
  local:
    base_url: http://localhost:11434  # Ollama default
    timeout: 60s
    max_retries: 1

    # Model-specific parameters
    model_params:
      num_ctx: 4096               # Context window
      num_predict: 512            # Max prediction tokens
      repeat_penalty: 1.1
      top_k: 40
      top_p: 0.9
```

**Popular Local Models**:

- `llama2`: General purpose, good performance
- `mistral`: Efficient and capable
- `codellama`: Specialized for code tasks
- `phi`: Lightweight, fast inference

## Provider Configuration - **Intermediate**

### Environment-Based Configuration

Configure providers using environment variables for security:

```bash
# OpenAI configuration
export OPENAI_API_KEY="sk-..."
export OPENAI_ORGANIZATION="org-..."

# Anthropic configuration
export ANTHROPIC_API_KEY="sk-ant-..."

# Local model configuration
export LOCAL_MODEL_BASE_URL="http://localhost:11434"
```

### Multi-Provider Setup

Use different providers for different agents:

```yaml
# High-performance agent using GPT-4
---
name: complex-analyzer
llm:
  provider: openai
  model: gpt-4
  temperature: 0.1
---

# Cost-effective agent using local model
---
name: simple-responder
llm:
  provider: local
  model: llama2
  temperature: 0.7
---

# Balanced agent using Claude
---
name: content-writer
llm:
  provider: anthropic
  model: claude-3-sonnet
  temperature: 0.6
---
```

## Model Selection Guide - **Beginner**

### By Use Case

**Data Analysis & Technical Tasks**:

- OpenAI GPT-4: Best accuracy for complex analysis
- Claude-3-Opus: Strong analytical capabilities
- Local CodeLlama: Good for code-related analysis

**Content Creation**:

- Claude-3-Sonnet: Excellent writing capabilities
- GPT-4-Turbo: Creative and versatile
- Mistral: Good local alternative

**Simple Q&A & Support**:

- GPT-3.5-Turbo: Fast and cost-effective
- Claude-3-Haiku: Quick responses
- Local Llama2: Privacy-focused

**Code Generation**:

- GPT-4: Best for complex code tasks
- Claude-3-Sonnet: Good code understanding
- Local CodeLlama: Specialized for coding

### By Performance Requirements

**Latency-Critical Applications**:

```yaml
llm:
  provider: openai
  model: gpt-3.5-turbo    # Fastest commercial option
  max_tokens: 500         # Limit response length
```

**Quality-Critical Applications**:

```yaml
llm:
  provider: openai
  model: gpt-4            # Highest quality
  temperature: 0.1        # More deterministic
```

**Cost-Optimized Applications**:

```yaml
llm:
  provider: local
  model: llama2           # No per-token costs
  max_tokens: 1000        # Reasonable limits
```

## Advanced Configuration - **Advanced**

### Custom Provider Implementation

For specialized or proprietary models:

```yaml
llm:
  provider: custom
  model: proprietary-model

  custom:
    endpoint: https://api.yourcompany.com/llm
    authentication:
      type: bearer
      token: ${CUSTOM_API_TOKEN}
    headers:
      X-Model-Version: "2.1"
      X-Customer-ID: ${CUSTOMER_ID}

    request_format: openai_compatible  # or anthropic_compatible, custom
    response_format: openai_compatible

    # Custom parameters
    parameters:
      custom_param: value
      another_param: ${CUSTOM_VALUE}
```

### Provider Fallback

Configure fallback providers for reliability:

```yaml
llm:
  primary_provider: openai
  fallback_providers:
    - provider: anthropic
      conditions:
        - rate_limit_exceeded
        - service_unavailable
    - provider: local
      conditions:
        - all_external_providers_down

  # Primary configuration
  provider: openai
  model: gpt-4

  # Fallback configurations
  fallback_configs:
    anthropic:
      model: claude-3-sonnet
      temperature: 0.7
    local:
      model: llama2
      temperature: 0.8
```

### Load Balancing

Distribute requests across multiple providers:

```yaml
llm:
  strategy: load_balance
  providers:
    - provider: openai
      model: gpt-4
      weight: 60              # 60% of requests
      max_requests_per_minute: 100

    - provider: anthropic
      model: claude-3-sonnet
      weight: 40              # 40% of requests
      max_requests_per_minute: 80

  load_balance:
    algorithm: weighted_round_robin
    health_check_interval: 30s
    failover_threshold: 3
```

## Performance Optimization - **Intermediate**

### Caching Configuration

Cache responses to reduce costs and improve performance:

```yaml
llm:
  provider: openai
  model: gpt-4

  caching:
    enabled: true
    ttl: 3600                     # 1 hour cache
    key_strategy: semantic        # or exact, fuzzy
    max_cache_size: 1000         # Number of cached responses

    # Cache conditions
    cache_when:
      - temperature: "< 0.3"     # Only cache deterministic responses
      - max_tokens: "< 1000"     # Don't cache very long responses
```

### Request Optimization

Optimize requests for specific providers:

```yaml
llm:
  provider: openai
  model: gpt-4-turbo

  optimization:
    batch_requests: true          # Batch multiple requests
    compress_context: true        # Compress large contexts
    async_processing: true        # Use async where possible

    # OpenAI-specific optimizations
    openai:
      use_streaming: true         # Stream responses
      parallel_requests: 3        # Max concurrent requests
      adaptive_timeout:           # Adjust timeout based on request size
        enabled: true
        # Example formula
        formula: "timeout = base_timeout + 0.5s * (request_tokens / 1000)"
        min_timeout: 10s           # Minimum timeout
        max_timeout: 120s          # Maximum timeout
```

## Monitoring and Debugging - **Intermediate**

### Provider Metrics

Monitor provider performance and costs:

```yaml
monitoring:
  providers:
    enabled: true
    metrics:
      - request_count
      - response_time
      - error_rate
      - token_usage
      - cost_tracking

  alerts:
    high_error_rate:
      threshold: 5%
      window: "5m"
    high_cost:
      threshold: $100
      window: "1d"
    slow_responses:
      threshold: 10s
      window: "1m"
```

### Debug Configuration

Enable detailed logging for troubleshooting:

```yaml
llm:
  provider: openai
  model: gpt-4

  debug:
    log_requests: true           # Log all requests (careful with PII)
    log_responses: false         # Usually too verbose
    log_errors: true            # Always log errors
    log_performance: true       # Track timing metrics

    # Debug levels
    request_logging:
      include_headers: false     # Security concern
      include_full_context: false  # Can be very large
      include_parameters: true
```

## Security Considerations - **Intermediate**

### API Key Management

Secure handling of provider credentials:

```yaml
# ✅ Good: Use environment variables
llm:
  provider: openai
  openai:
    api_key: ${OPENAI_API_KEY}

# ❌ Bad: Hardcoded keys
llm:
  provider: openai
  openai:
    api_key: "sk-hardcoded-key-here"
```

### Request Filtering

Filter sensitive data before sending to providers:

```yaml
llm:
  provider: openai
  model: gpt-4

  security:
    data_filtering:
      enabled: true
      remove_patterns:
        - "\\b\\d{3}-\\d{2}-\\d{4}\\b"  # SSN pattern
        - "\\b\\d{16}\\b"               # Credit card pattern
        - "password|secret|key"         # Sensitive keywords

    compliance:
      gdpr_mode: true              # EU compliance
      hipaa_mode: false           # Healthcare compliance
      pci_mode: false             # Payment compliance
```

## Migration Guide - **Advanced**

### Switching Providers

Migrate from one provider to another:

```bash
# 1. Test new provider configuration
curl -X POST http://localhost:3000/api/agents/validate \
  -d '{"definition": "'$(cat agent-with-new-provider.md)'"}'

# 2. Deploy with canary strategy
curl -X POST http://localhost:3000/api/agents/my-agent/deploy \
  -d '{
    "strategy": "canary",
    "traffic_percentage": 10,
    "definition": "'$(cat agent-with-new-provider.md)'"
  }'

# 3. Monitor performance
curl http://localhost:3000/api/agents/my-agent/metrics | jq .provider_stats

# 4. Gradually increase traffic
curl -X PUT http://localhost:3000/api/agents/my-agent/canary \
  -d '{"traffic_percentage": 50}'

# 5. Complete migration
curl -X POST http://localhost:3000/api/agents/my-agent/promote \
  -d '{"version": "canary"}'
```

### Provider Comparison

Compare providers for specific workloads:

```bash
# Run comparison test
curl -X POST http://localhost:3000/api/agents/test/compare-providers \
  -d '{
    "providers": ["openai", "anthropic", "local"],
    "test_cases": [
      {"prompt": "Analyze this data...", "expected_capabilities": ["analysis"]},
      {"prompt": "Write a summary...", "expected_capabilities": ["writing"]}
    ],
    "metrics": ["response_time", "quality_score", "cost"]
  }'
```

## Troubleshooting - **Intermediate**

### Common Issues

**Authentication Errors**:

```bash
# Check API key format
echo $OPENAI_API_KEY | wc -c  # Should be ~51 characters for OpenAI

# Test authentication
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
  https://api.openai.com/v1/models
```

**Rate Limiting**:

```yaml
# Add retry configuration
llm:
  provider: openai
  openai:
    max_retries: 5
    retry_delay: 1s
    exponential_backoff: true
```

**Context Length Errors**:

```yaml
# Limit context and response length
llm:
  max_tokens: 1000

memory:
  context_preparation:
    max_context_length: 6000    # Leave room for response
```

### Provider Health Checks

Monitor provider availability:

```bash
# Check provider status
curl http://localhost:3000/api/providers/openai/health
curl http://localhost:3000/api/providers/anthropic/health
curl http://localhost:3000/api/providers/local/health

# Get provider capabilities
curl http://localhost:3000/api/providers/openai/capabilities
```

## Related Documentation

- [Agent Format](agent-format.md) - **Beginner**
- [Best Practices](best-practices.md) - **Intermediate**
- [Configuration Examples](examples.md) - **Beginner**
- [Overview](overview.md) - **Beginner**
- [Building Agents Guide](../building-agents.md) - **Beginner**
- [Security Guide](../security.md) - **Intermediate**

## Provider Comparison Table

| Provider | Strengths | Best For | Cost | Setup Difficulty |
|----------|-----------|----------|------|------------------|
| OpenAI | Most capable, fast | Complex analysis, coding | High | Easy |
| Anthropic | Excellent writing, safe | Content creation | Medium | Easy |
| Local | Privacy, no per-token cost | High-volume | Hardware | Advanced |
