---
title: "LLM Provider Integration"
date: 2025-09-10
layout: guide
categories: [Configuration, Integration]
---

> **🚧 Implementation Status**
>
> The pluggable LLM provider system represents core architecture from ADR-28.
> This documentation serves as the technical specification for the multi-provider
> LLM integration currently under development.
>
> **Target**: Seamless integration with multiple LLM providers
> **Status**: Provider abstraction layer and OpenAI reference implementation
> in progress

## Overview

Caxton's pluggable LLM provider system enables configuration agents to
integrate with any LLM/SLM API through configurable providers. The system is
designed with no vendor lock-in, allowing users to choose their preferred model
provider while maintaining consistent agent behavior.

## Core Architecture

### Provider Abstraction Layer

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete_chat(
        &self,
        request: ChatRequest
    ) -> Result<ChatResponse, LlmError>;
    async fn stream_chat(
        &self,
        request: ChatRequest
    ) -> Result<ChatStream, LlmError>;
    async fn get_model_info(&self) -> Result<ModelInfo, LlmError>;
    fn supports_function_calling(&self) -> bool;
    fn supports_streaming(&self) -> bool;
}
```

### Context Management

Configuration agents require intelligent context management since they cannot
programmatically control context like WASM agents. The runtime provides context
through a multi-layered architecture (ADR-0031) that automatically gathers and
formats relevant information for optimal agent performance.

#### Context Sources Framework

The context management system gathers information from four primary sources:

- **Conversation History**: Agent message threads provide request context and
  prior exchanges
- **Memory System**: Semantic search from embedded memory system provides
  historical patterns and knowledge
- **Capability Registry**: Information about available capabilities and their
  relationships
- **MCP Tool Context**: Tool-specific context requirements and runtime data

#### Enhanced MCP Tool Context Requirements

MCP tools declare their context needs using YAML specifications, eliminating
the need for manual prompt engineering:

```yaml
# Example: Data Analysis Tool Context Specification
context_requirements:
  conversation_depth: 10        # Include last 10 conversation turns
  memory_search:
    query_template: "data analysis {{request_type}} similar to {{user_request}}"
    max_results: 15             # Up to 15 relevant memory entries
    min_similarity: 0.7         # Only highly relevant memories
  capability_context: true      # Include related capability information
  tool_data:                    # Specific tool data to include
    - "user_preferences"
    - "recent_analysis_results"
    - "data_schema_cache"
  provider_optimization:        # LLM-specific settings
    openai:
      max_context_tokens: 8000
      function_calling_format: true
    anthropic:
      max_context_tokens: 12000
      structured_output: true
```

#### Context Filtering Pipeline

The system applies multi-stage filtering to optimize signal-to-noise ratio:

1. **Relevance Scoring**: Semantic similarity scoring for all context candidates
2. **Hierarchical Selection**: Priority-based context selection with token
   budget management
3. **Redundancy Elimination**: Remove duplicate or highly similar information
4. **Provider Formatting**: Adapt context structure for target LLM provider

#### Runtime Context Intelligence

The Context Router provides runtime intelligence that analyzes each request and
dynamically determines optimal context:

```rust
// Conceptual context preparation flow
async fn prepare_context(
    request: &AgentMessage,
    agent_config: &AgentConfig,
    provider: &dyn LlmProvider
) -> Result<FormattedContext, ContextError> {
    // 1. Analyze request to determine context requirements
    let requirements = analyze_context_requirements(request, agent_config)?;

    // 2. Gather context from multiple sources in parallel
    let context_sources = gather_context_sources(requirements).await?;

    // 3. Filter and rank context for relevance
    let filtered_context = filter_context(context_sources, requirements)?;

    // 4. Format for specific LLM provider
    provider.format_context(filtered_context).await
}
```

## Supported LLM Providers

### OpenAI (Reference Implementation)

**Models Supported**:

- GPT-4o (recommended for production)
- GPT-4o-mini (recommended for development)
- GPT-4 Turbo
- GPT-3.5 Turbo

**Configuration**:

```yaml
llm:
  provider: "openai"
  model: "gpt-4o"
  api_key: "${OPENAI_API_KEY}"
  base_url: "https://api.openai.com/v1"  # Optional: for proxies
  max_tokens: 4096
  temperature: 0.7
  timeout_seconds: 30
  retry_attempts: 3
  retry_delay_ms: 1000
```

**Features**:

- Function calling for MCP tool integration
- Streaming responses for real-time feedback
- Context length optimization
- Automatic retry with exponential backoff

### Anthropic

**Models Supported**:

- Claude 3.5 Sonnet (recommended for reasoning tasks)
- Claude 3.5 Haiku (recommended for speed)
- Claude 3 Opus (recommended for complex analysis)

**Configuration**:

```yaml
llm:
  provider: "anthropic"
  model: "claude-3-5-sonnet-20241022"
  api_key: "${ANTHROPIC_API_KEY}"
  max_tokens: 4096
  temperature: 0.7
  timeout_seconds: 45
  system_context_limit: 200000  # Claude's large context window
```

**Features**:

- Large context windows (200K+ tokens)
- Strong reasoning capabilities
- Built-in safety and constitutional AI
- Structured output generation

### Azure OpenAI

**Configuration**:

```yaml
llm:
  provider: "azure"
  deployment_name: "gpt-4o-deployment"
  api_key: "${AZURE_OPENAI_API_KEY}"
  endpoint: "https://your-resource.openai.azure.com"
  api_version: "2024-05-01-preview"
  max_tokens: 4096
  temperature: 0.7
```

**Features**:

- Enterprise security and compliance
- Regional data residency
- Private network integration
- Custom fine-tuned models

### Local LLM Integration (Ollama)

**Models Supported**:

- Llama 3.1/3.2 variants
- Gemma 2 variants
- Qwen 2.5 variants
- Phi 3.5 variants
- Code-specific models (CodeLlama, DeepSeek Coder)

**Configuration**:

```yaml
llm:
  provider: "ollama"
  model: "llama3.1:8b"
  base_url: "http://localhost:11434"
  max_tokens: 2048
  temperature: 0.7
  context_window: 8192
  gpu_acceleration: true
```

**Features**:

- Complete privacy and data residency
- No API usage costs
- Custom model fine-tuning
- Offline operation capability

### Local LLM Integration (vLLM)

**Configuration**:

```yaml
llm:
  provider: "vllm"
  model: "microsoft/Phi-3.5-mini-instruct"
  base_url: "http://localhost:8000/v1"
  api_key: "optional-auth-token"
  max_tokens: 4096
  temperature: 0.7
  tensor_parallel_size: 2  # Multi-GPU support
```

**Features**:

- High-throughput inference serving
- Multi-GPU scaling
- OpenAI-compatible API
- Advanced batching and caching

### Custom Provider Integration

For proprietary or specialized LLM APIs:

```yaml
llm:
  provider: "custom"
  provider_config:
    base_url: "https://api.yourcompany.com/llm"
    auth_type: "bearer"  # bearer, api_key, oauth2
    auth_token: "${CUSTOM_LLM_TOKEN}"
    model_field: "model_name"
    prompt_field: "input_text"
    response_field: "generated_text"
  model: "custom-model-v1"
  max_tokens: 4096
  temperature: 0.7
```

## Multi-Provider Configurations

### Provider Fallback Chain

Configure multiple providers for reliability:

```yaml
llm:
  providers:
    - name: "primary"
      provider: "openai"
      model: "gpt-4o"
      api_key: "${OPENAI_API_KEY}"
      priority: 100
    - name: "fallback"
      provider: "anthropic"
      model: "claude-3-5-haiku-20241022"
      api_key: "${ANTHROPIC_API_KEY}"
      priority: 50
    - name: "local"
      provider: "ollama"
      model: "llama3.1:8b"
      base_url: "http://localhost:11434"
      priority: 10
  fallback_strategy: "priority"  # priority, round_robin, load_balance
```

### Capability-Specific Providers

Different providers for different capabilities:

```yaml
agents:
  - name: CodeReviewer
    capabilities: ["code-analysis"]
    llm:
      provider: "openai"
      model: "gpt-4o"  # Strong reasoning for code
  - name: DataAnalyzer
    capabilities: ["data-processing"]
    llm:
      provider: "anthropic"
      model: "claude-3-5-sonnet-20241022"  # Large context for data
  - name: ContentGenerator
    capabilities: ["content-generation"]
    llm:
      provider: "ollama"
      model: "llama3.1:8b"  # Cost-effective for content
```

## Context Management Strategies

### Capability-Driven Context Formatting

When Agent A requests a capability from Agent B, the context management system
automatically gathers and formats relevant context. The system uses MCP tool
context specifications to determine exactly what information to include:

```json
{
  "request_context": {
    "requesting_agent": "DataProcessor",
    "requested_capability": "report-generation",
    "request_parameters": {
      "format": "pdf",
      "template": "quarterly-summary"
    },
    "context_requirements": {
      "conversation_depth": 5,
      "memory_similarity_threshold": 0.8,
      "tool_context_sources": ["user_preferences", "report_templates"]
    }
  },
  "conversation_history": [
    {"role": "user", "content": "Analyze Q3 sales data"},
    {"role": "assistant", "content": "I'll process the sales data and generate the quarterly summary..."},
    {"role": "user", "content": "Include YoY comparison charts"},
    {"role": "assistant", "content": "I'll add year-over-year comparison visualizations to the report..."}
  ],
  "memory_context": {
    "search_query": "report generation quarterly summary similar to pdf template",
    "relevant_entities": [
      {
        "name": "Q3 Sales Analysis Pattern",
        "similarity": 0.89,
        "observations": ["Standard quarterly reports include revenue trends, regional breakdowns, YoY comparisons"]
      },
      {
        "name": "PDF Report Template Preferences",
        "similarity": 0.85,
        "observations": ["Users prefer executive summary on page 1, detailed charts on page 2-3"]
      }
    ],
    "total_memory_entries_searched": 150,
    "entries_above_threshold": 12
  },
  "capability_context": {
    "related_capabilities": ["data-visualization", "chart-generation", "pdf-creation"],
    "capability_requirements": {
      "input_formats": ["json", "csv"],
      "output_formats": ["pdf"],
      "performance_sla": "< 30 seconds for standard reports"
    }
  },
  "available_tools": [
    {
      "name": "pdf_generator",
      "description": "Generate PDF reports with templating support",
      "context_data": {
        "available_templates": ["quarterly-summary", "executive-brief", "detailed-analysis"],
        "supported_chart_types": ["bar", "line", "pie", "trend"]
      }
    },
    {
      "name": "chart_creator",
      "description": "Create charts and visualizations",
      "context_data": {
        "recent_chart_preferences": ["YoY comparison bars", "regional pie charts"],
        "performance_data": "average generation time: 2.3s"
      }
    }
  ],
  "context_metadata": {
    "preparation_time_ms": 87,
    "total_context_tokens": 2847,
    "token_utilization_rate": 0.89,
    "context_sources_used": ["conversation", "memory", "tools", "capabilities"]
  }
}
```

### MCP Tool Context Integration

Tools specify their context needs in their MCP specifications, allowing the
runtime to gather exactly the right information:

```yaml
# pdf_generator tool context specification
name: "pdf_generator"
description: "Advanced PDF report generation with templating"
context_requirements:
  conversation_depth: 3  # Focus on recent report requirements
  memory_search:
    query_template: "PDF report {{report_type}} {{business_context}}"
    max_results: 8
    focus_areas: ["report_templates", "user_preferences", "formatting_standards"]
  tool_data:
    - "template_library"      # Available report templates
    - "user_style_preferences" # Preferred fonts, colors, layouts
    - "recent_generation_stats" # Performance and usage data
  provider_optimization:
    openai:
      structured_output: true   # Use structured function calling
      max_context_tokens: 6000
    anthropic:
      tool_use_format: true     # Use Anthropic's tool format
      max_context_tokens: 8000
```

### Provider-Specific Context Formatting

The context management system automatically formats context for optimal
compatibility with different LLM providers, leveraging their specific strengths
and capabilities.

**OpenAI Function Calling Format**:

```json
{
  "messages": [
    {
      "role": "system",
      "content": "You are a report generation expert specialized in quarterly business analysis.\n\nContext from memory system:\n- Standard quarterly reports include revenue trends, regional breakdowns, YoY comparisons\n- Users prefer executive summary on page 1, detailed charts on page 2-3\n\nAvailable tools have been configured with recent user preferences and template options."
    },
    {
      "role": "user",
      "content": "Based on the Q3 sales data analysis, create a quarterly sales report using the quarterly-summary template with YoY comparison charts."
    },
    {
      "role": "assistant",
      "content": "I'll process the Q3 sales data and generate the quarterly summary report with year-over-year comparison visualizations..."
    },
    {
      "role": "user",
      "content": "Please generate the final PDF report now."
    }
  ],
  "functions": [
    {
      "name": "generate_pdf_report",
      "description": "Generate a PDF report from data with templating support. Available templates: quarterly-summary, executive-brief, detailed-analysis. Supports chart types: bar, line, pie, trend.",
      "parameters": {
        "type": "object",
        "properties": {
          "template": {
            "type": "string",
            "enum": ["quarterly-summary", "executive-brief", "detailed-analysis"],
            "description": "Report template to use"
          },
          "data": {"type": "object"},
          "chart_preferences": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Preferred chart types based on user history"
          }
        }
      }
    }
  ],
  "function_call": "auto",
  "max_tokens": 4000
}
```

**Anthropic Tool Use Format**:

```json
{
  "system": "You are a report generation expert specialized in quarterly business analysis.\n\nRelevant context from conversation and memory:\n- Previous request focused on Q3 sales data analysis with YoY comparisons\n- Historical preference: executive summary first, then detailed visualizations\n- Report template 'quarterly-summary' matches current requirements\n\nYour tools have access to user preferences and recent usage patterns for optimal report generation.",
  "messages": [
    {
      "role": "user",
      "content": "Based on the Q3 sales data analysis, create a quarterly sales report using the quarterly-summary template with YoY comparison charts."
    },
    {
      "role": "assistant",
      "content": "I'll process the Q3 sales data and generate the quarterly summary report with year-over-year comparison visualizations..."
    },
    {
      "role": "user",
      "content": "Please generate the final PDF report now."
    }
  ],
  "tools": [
    {
      "name": "generate_pdf_report",
      "description": "Generate a PDF report from data with advanced templating. Context: Available templates include quarterly-summary (preferred), executive-brief, and detailed-analysis. Recent user preferences favor YoY comparison bars and regional pie charts with average generation time of 2.3s.",
      "input_schema": {
        "type": "object",
        "properties": {
          "template": {
            "type": "string",
            "enum": ["quarterly-summary", "executive-brief", "detailed-analysis"],
            "description": "Report template - quarterly-summary recommended based on context"
          },
          "data": {"type": "object"},
          "chart_preferences": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Chart types - YoY comparison bars recommended"
          }
        }
      }
    }
  ],
  "max_tokens": 4000
}
```

**Local LLM Optimized Format (Ollama/vLLM)**:

```json
{
  "messages": [
    {
      "role": "system",
      "content": "Generate quarterly PDF reports. Templates: quarterly-summary, executive-brief, detailed-analysis. Charts: bar, line, pie, trend. User prefers: executive summary page 1, YoY charts, regional breakdowns."
    },
    {
      "role": "user",
      "content": "Create Q3 sales report, quarterly-summary template, include YoY comparison charts."
    }
  ],
  "tools": [
    {
      "name": "generate_pdf_report",
      "description": "Generate PDF reports with templates",
      "parameters": {
        "template": "quarterly-summary",
        "chart_types": ["bar", "line"],
        "data": "object"
      }
    }
  ],
  "max_tokens": 2048,
  "temperature": 0.1
}
```

### Context Optimization by Provider

**Performance Characteristics**:

- **OpenAI GPT-4o**: Excellent function calling, handles complex context well,
  optimized for 8K context windows
- **Anthropic Claude**: Large context windows (200K tokens), strong reasoning
  with comprehensive context
- **Local Models**: Simplified context formatting, focus on essential
  information only

### Context Length Optimization

**Automatic Context Trimming**:

- Prioritize system instructions and current request
- Preserve recent conversation history (last 5-10 exchanges)
- Include only most relevant memory entities (top 5 by similarity)
- Summarize older context when approaching token limits

**Dynamic Context Selection**:

- Calculate token usage for each context component
- Prioritize based on relevance scores and recency
- Use provider-specific context windows efficiently
- Implement sliding window for long conversations

## Authentication and Security

### API Key Management

**Environment Variables** (Recommended):

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export AZURE_OPENAI_API_KEY="..."
```

**Secrets Management Integration**:

```yaml
llm:
  provider: "openai"
  api_key:
    source: "vault"  # vault, aws_secrets, azure_keyvault
    path: "caxton/llm/openai"
    key: "api_key"
```

**Rotation and Monitoring**:

- Automatic API key rotation support
- Usage monitoring and alerting
- Rate limit tracking and management
- Cost monitoring and budgeting

### Request/Response Security

**Content Filtering**:

- Automatic PII detection and redaction
- Custom content filtering rules
- Audit logging for sensitive requests
- Compliance with data protection regulations

**Network Security**:

- TLS encryption for all API communications
- Request/response logging with sanitization
- Network-level access controls
- VPC/private network support

## Error Handling and Resilience

### Provider-Level Error Handling

**Timeout Behavior**:

```yaml
llm:
  provider: "openai"
  timeout_seconds: 30
  read_timeout_seconds: 60  # For streaming responses
  connect_timeout_seconds: 10
```

**Retry Strategies**:

```yaml
llm:
  retry_policy:
    max_attempts: 3
    initial_delay_ms: 1000
    max_delay_ms: 10000
    backoff_multiplier: 2.0
    retry_on_errors: ["timeout", "rate_limit", "server_error"]
```

**Circuit Breaker Patterns**:

- Automatic provider failover on repeated failures
- Health checks for provider availability
- Gradual traffic restoration after failures
- Circuit breaker configuration per provider

### Fallback Strategies

**Provider Unavailable**:

1. Attempt primary provider
2. Fall back to secondary provider with adjusted context
3. Use local model as last resort
4. Graceful degradation with error message

**Context Too Large**:

1. Automatic context trimming
2. Summarization of older context
3. Multi-turn processing for large requests
4. Clear error messaging for unsupported requests

**Rate Limiting**:

1. Automatic backoff and retry
2. Request queuing and throttling
3. Provider switching for different rate limits
4. Usage monitoring and alerting

## Performance Optimization

### Response Caching

**Semantic Caching**:

```yaml
llm:
  caching:
    enabled: true
    ttl_seconds: 3600
    similarity_threshold: 0.95
    cache_backend: "redis"  # redis, memory, disk
```

**Cache Invalidation**:

- Time-based expiration
- Memory system updates trigger invalidation
- Configuration changes clear relevant caches
- Manual cache management API

### Streaming Responses

**Real-time Feedback**:

```yaml
llm:
  streaming:
    enabled: true
    chunk_size: 64
    buffer_timeout_ms: 100
```

**Progressive Processing**:

- Stream tokens to client as generated
- Partial response processing
- Early error detection
- Graceful stream termination

### Batch Processing

**Multi-Request Optimization**:

- Batch similar requests when possible
- Parallel provider requests for different capabilities
- Request deduplication and caching
- Load balancing across provider instances

## Monitoring and Observability

### Provider Metrics

**Performance Tracking**:

```rust
// Prometheus metrics
caxton_llm_request_duration_seconds{provider="openai", model="gpt-4o"}
caxton_llm_requests_total{provider="openai", status="success"}
caxton_llm_token_usage_total{provider="openai", type="input"}
caxton_llm_errors_total{provider="openai", error_type="timeout"}
```

**Cost Tracking**:

- Token usage per provider and model
- Estimated cost calculations
- Budget alerts and limits
- Usage trend analysis

### Health Monitoring

**Provider Health Checks**:

```yaml
llm:
  health_check:
    enabled: true
    interval_seconds: 60
    timeout_seconds: 10
    test_prompt: "Hello, respond with 'OK'"
```

**Alerting**:

- Provider downtime notifications
- High error rate alerts
- Cost threshold warnings
- Performance degradation detection

## Configuration Examples

### Development Configuration

```yaml
# caxton-dev.yaml
llm:
  provider: "ollama"
  model: "llama3.1:8b"
  base_url: "http://localhost:11434"
  temperature: 0.1
  max_tokens: 2048
  caching:
    enabled: true
    ttl_seconds: 1800
```

### Production Configuration

```yaml
# caxton-prod.yaml
llm:
  providers:
    - name: "primary"
      provider: "openai"
      model: "gpt-4o"
      api_key: "${OPENAI_API_KEY}"
      priority: 100
      timeout_seconds: 30
    - name: "fallback"
      provider: "anthropic"
      model: "claude-3-5-haiku-20241022"
      api_key: "${ANTHROPIC_API_KEY}"
      priority: 50
      timeout_seconds: 45
  retry_policy:
    max_attempts: 3
    initial_delay_ms: 1000
    backoff_multiplier: 2.0
  caching:
    enabled: true
    ttl_seconds: 3600
    backend: "redis"
  monitoring:
    metrics_enabled: true
    cost_tracking: true
    budget_limit_usd: 1000
```

### Enterprise Configuration

```yaml
# caxton-enterprise.yaml
llm:
  provider: "azure"
  deployment_name: "gpt-4o-enterprise"
  endpoint: "https://caxton.openai.azure.com"
  api_key:
    source: "azure_keyvault"
    vault_url: "https://caxton-vault.vault.azure.net"
    secret_name: "openai-api-key"
  security:
    content_filtering: true
    pii_redaction: true
    audit_logging: true
  compliance:
    data_residency: "us-east"
    encryption_at_rest: true
    network_isolation: true
```

## Migration and Compatibility

### Provider Migration

**Zero-Downtime Migration**:

1. Configure new provider alongside existing
2. Gradually shift traffic using weighted routing
3. Monitor performance and error rates
4. Complete migration once validated
5. Remove old provider configuration

**Configuration Compatibility**:

- Automatic parameter mapping between providers
- Model capability detection and validation
- Feature support checking
- Clear migration error messages

### Version Management

**Model Versioning**:

```yaml
llm:
  provider: "openai"
  model: "gpt-4o"
  model_version: "gpt-4o-2024-05-13"  # Pin to specific version
  auto_upgrade: false  # Prevent automatic updates
```

**Provider API Versioning**:

- Explicit API version specification
- Backward compatibility maintenance
- Deprecation warnings and migration guides
- Version-specific feature support

## Related Documentation

- [Configuration Agent Overview](overview.md) - Core configuration agent concepts
- [ADR-0028: Configuration-Driven Agents](
  ../adrs/0028-configuration-driven-agent-architecture.md) -
  Architectural rationale
- [Memory System Overview](../memory-system/overview.md) - Memory-augmented context
- [API Reference](../api/config-agents.md) - Configuration agent API
- [Best Practices](best-practices.md) - Development guidelines

## Next Steps

1. **Choose Provider**: Select appropriate LLM provider for your use case
2. **Configure Authentication**: Set up API keys and security
3. **Test Integration**: Validate provider connectivity and functionality
4. **Monitor Performance**: Set up observability and alerting
5. **Optimize Configuration**: Tune parameters for your workload
