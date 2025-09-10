---
title: "Comprehensive Error Handling Guide"
date: 2025-09-10
layout: page
categories: [operations, error-handling]
---

> **🚧 Implementation Status**
>
> Comprehensive error handling is a critical operational requirement for
> production Caxton deployments. This guide serves as the technical
> specification for error handling patterns, recovery procedures, and resilience
> mechanisms currently under development.
>
> **Target**: Comprehensive error handling and recovery patterns
> **Status**: Error taxonomy and recovery procedures in development

## Overview

Caxton's error handling system provides comprehensive coverage of failure
scenarios across all system components, with automatic recovery mechanisms,
graceful degradation patterns, and clear operational procedures for manual
intervention when needed.

## Error Taxonomy

### System-Level Errors

#### Configuration Agent Errors

**Agent Deployment Failures**:

```rust
pub enum AgentDeploymentError {
    InvalidConfiguration {
        field: String,
        message: String,
        suggested_fix: Option<String>,
    },
    NameConflict {
        agent_name: String,
        existing_id: String,
        workspace: String,
    },
    ResourceLimitsExceeded {
        requested: ResourceLimits,
        available: ResourceLimits,
        component: String,
    },
    CapabilityValidationFailed {
        capability: String,
        reason: String,
        available_capabilities: Vec<String>,
    },
    TemplateNotFound {
        template_id: String,
        available_templates: Vec<String>,
    },
}
```

**Runtime Execution Errors**:

```rust
pub enum AgentExecutionError {
    LlmProviderUnavailable {
        provider: String,
        last_error: String,
        fallback_attempted: bool,
    },
    ContextTooLarge {
        context_size_tokens: usize,
        max_tokens: usize,
        truncation_strategy: ContextTruncationStrategy,
    },
    ToolExecutionFailed {
        tool_name: String,
        error_code: String,
        error_message: String,
        retry_possible: bool,
    },
    MemorySystemUnavailable {
        backend: String,
        last_error: String,
        fallback_mode: bool,
    },
    PermissionDenied {
        operation: String,
        required_permission: String,
        agent_permissions: Vec<String>,
    },
}
```

#### LLM Provider Errors

**Connection and Authentication**:

```rust
pub enum LlmProviderError {
    ConnectionTimeout {
        provider: String,
        timeout_seconds: u64,
        retry_attempt: usize,
        max_retries: usize,
    },
    AuthenticationFailed {
        provider: String,
        auth_type: String,
        error_code: Option<String>,
        token_expired: bool,
    },
    RateLimitExceeded {
        provider: String,
        limit_type: RateLimitType,
        retry_after_seconds: Option<u64>,
        current_usage: u64,
        limit: u64,
    },
    ModelUnavailable {
        provider: String,
        model: String,
        available_models: Vec<String>,
        fallback_model: Option<String>,
    },
    InsufficientQuota {
        provider: String,
        operation: String,
        required_tokens: usize,
        available_tokens: usize,
        quota_reset_time: Option<SystemTime>,
    },
}

pub enum RateLimitType {
    RequestsPerMinute,
    TokensPerMinute,
    ConcurrentRequests,
    DailyUsage,
}
```

**Response and Processing Errors**:

```rust
pub enum LlmResponseError {
    InvalidResponse {
        provider: String,
        response_body: String,
        parsing_error: String,
    },
    ContentFiltering {
        provider: String,
        filter_reason: String,
        content_type: String,
        can_retry: bool,
    },
    ContextLengthExceeded {
        provider: String,
        requested_tokens: usize,
        max_tokens: usize,
        truncation_applied: bool,
    },
    FunctionCallError {
        provider: String,
        function_name: String,
        error_message: String,
        arguments: String,
    },
    StreamingInterrupted {
        provider: String,
        bytes_received: usize,
        expected_bytes: Option<usize>,
        recovery_possible: bool,
    },
}
```

#### Memory System Errors

**Storage and Retrieval**:

```rust
pub enum MemoryError {
    BackendUnavailable {
        backend: String,
        error: String,
        fallback_available: bool,
    },
    EmbeddingGenerationFailed {
        model: String,
        input_text: String,
        error: String,
        retry_with_fallback: bool,
    },
    StorageQuotaExceeded {
        backend: String,
        current_usage: u64,
        quota_limit: u64,
        cleanup_suggestions: Vec<String>,
    },
    SearchTimeout {
        backend: String,
        query: String,
        timeout_seconds: u64,
        partial_results_available: bool,
    },
    IndexCorruption {
        backend: String,
        index_type: String,
        corruption_extent: CorruptionExtent,
        rebuild_required: bool,
    },
}

pub enum CorruptionExtent {
    Minor,      // Affects few entities, automatic recovery possible
    Major,      // Affects many entities, manual intervention required
    Complete,   // Complete index rebuild necessary
}
```

**Model Management Errors**:

```rust
pub enum ModelError {
    DownloadFailed {
        model_id: String,
        error: DownloadError,
        fallback_model: Option<String>,
    },
    ValidationFailed {
        model_id: String,
        validation_errors: Vec<ValidationError>,
        severity: ValidationSeverity,
    },
    LoadingFailed {
        model_id: String,
        error: String,
        memory_available: u64,
        memory_required: u64,
    },
    IncompatibleVersion {
        model_id: String,
        model_version: String,
        required_version: String,
        migration_available: bool,
    },
}
```

#### Messaging System Errors

**FIPA Message Routing**:

```rust
pub enum MessageRoutingError {
    AgentNotFound {
        agent_id: String,
        suggested_agents: Vec<String>,
    },
    CapabilityNotAvailable {
        capability: String,
        available_capabilities: Vec<String>,
        similar_capabilities: Vec<String>,
    },
    DeliveryTimeout {
        target_agent: String,
        message_id: String,
        timeout_seconds: u64,
        retry_possible: bool,
    },
    MessageTooLarge {
        size_bytes: usize,
        max_size_bytes: usize,
        compression_attempted: bool,
    },
    InvalidPerformative {
        performative: String,
        valid_performatives: Vec<String>,
        context: MessageContext,
    },
}

pub enum MessageContext {
    Request,
    Response,
    Conversation,
    Broadcast,
}
```

## Error Handling Patterns

### Timeout Handling

#### LLM API Call Timeouts

**Timeout Configuration**:

```yaml
llm:
  timeouts:
    connect_seconds: 10      # Connection establishment
    read_seconds: 60         # Response reading (streaming)
    total_seconds: 120       # Total request timeout
    retry_multiplier: 1.5    # Timeout increase per retry
```

**Timeout Recovery Pattern**:

```rust
async fn handle_llm_timeout(
    provider: &str,
    request: &ChatRequest,
    attempt: usize
) -> Result<ChatResponse, LlmError> {
    match attempt {
        1..=2 => {
            // Short retries with original timeout
            warn!("LLM timeout on attempt {}, retrying", attempt);
            tokio::time::sleep(Duration::from_millis(1000 * attempt)).await;
            retry_request(request, original_timeout()).await
        }
        3..=4 => {
            // Extended timeout retries
            warn!("LLM timeout, trying with extended timeout");
            retry_request(request, extended_timeout()).await
        }
        _ => {
            // Fallback provider or degraded mode
            error!("LLM provider {} repeatedly timing out, trying fallback", provider);
            try_fallback_provider(request).await
        }
    }
}
```

#### Memory System Timeouts

**Search Timeout Handling**:

```rust
async fn handle_search_timeout(
    query: &str,
    timeout: Duration
) -> Result<Vec<SearchResult>, MemoryError> {
    let search_future = semantic_search(query);
    let timeout_future = tokio::time::sleep(timeout);

    match tokio::select! {
        result = search_future => result,
        _ = timeout_future => {
            warn!("Search timeout for query: {}", query);
            // Return cached results if available
            get_cached_search_results(query).await
                .or_else(|_| {
                    // Fallback to keyword-only search
                    keyword_search(query).await
                })
        }
    }
}
```

### Fallback Behavior

#### LLM Provider Fallback Chain

**Provider Selection Strategy**:

```rust
pub struct FallbackChain {
    providers: Vec<LlmProvider>,
    current_index: AtomicUsize,
    failure_counts: HashMap<String, usize>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

impl FallbackChain {
    async fn execute_with_fallback(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        for provider in &self.providers {
            if self.is_provider_available(provider).await {
                match provider.complete_chat(request.clone()).await {
                    Ok(response) => {
                        self.record_success(provider).await;
                        return Ok(response);
                    }
                    Err(error) => {
                        self.record_failure(provider, &error).await;

                        // Determine if error is retryable with different provider
                        if should_try_fallback(&error) {
                            continue;
                        } else {
                            return Err(error);
                        }
                    }
                }
            }
        }

        Err(LlmError::AllProvidersFailed)
    }
}
```

#### Embedding Model Fallback

**Model Unavailability Handling**:

```rust
async fn generate_embedding_with_fallback(
    text: &str
) -> Result<Vec<f32>, MemoryError> {
    let models = &["all-MiniLM-L6-v2", "all-MiniLM-L12-v2", "bundled-mini"];

    for model in models {
        match load_model_and_embed(model, text).await {
            Ok(embedding) => return Ok(embedding),
            Err(ModelError::LoadingFailed { .. }) => {
                warn!("Model {} failed to load, trying next", model);
                continue;
            }
            Err(ModelError::ValidationFailed { severity, .. }) => {
                match severity {
                    ValidationSeverity::Critical => continue,
                    ValidationSeverity::Warning => {
                        // Use with warning
                        warn!("Model {} has validation warnings but proceeding", model);
                        return generate_embedding_unsafe(model, text).await;
                    }
                }
            }
            Err(error) => return Err(MemoryError::EmbeddingFailed(error)),
        }
    }

    // Final fallback: keyword-only mode
    warn!("All embedding models failed, falling back to keyword-only search");
    Err(MemoryError::NoEmbeddingModel)
}
```

### Circuit Breaker Patterns

#### External Dependency Circuit Breakers

**LLM Provider Circuit Breaker**:

```rust
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: usize,
    failure_threshold: usize,
    success_count: usize,
    success_threshold: usize,
    timeout: Duration,
    last_failure_time: Option<Instant>,
}

pub enum CircuitBreakerState {
    Closed,     // Normal operation
    Open,       // Failing, reject requests
    HalfOpen,   // Testing if service recovered
}

impl CircuitBreaker {
    async fn call<F, T, E>(&mut self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        match self.state {
            CircuitBreakerState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitBreakerState::HalfOpen;
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            }
            CircuitBreakerState::Closed | CircuitBreakerState::HalfOpen => {}
        }

        match operation.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(CircuitBreakerError::Failure(error))
            }
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                    warn!("Circuit breaker opened due to {} failures", self.failure_count);
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                warn!("Circuit breaker re-opened after test failure");
            }
            CircuitBreakerState::Open => {}
        }
    }
}
```

### Recovery Procedures

#### Automatic Recovery Patterns

**Agent Recovery from Failure**:

```rust
pub struct AgentRecoveryManager {
    recovery_strategies: HashMap<ErrorType, RecoveryStrategy>,
    max_recovery_attempts: usize,
    recovery_backoff: ExponentialBackoff,
}

impl AgentRecoveryManager {
    async fn recover_agent(&self, agent_id: &str, error: &AgentError) -> Result<(), RecoveryError> {
        let strategy = self.recovery_strategies
            .get(&error.error_type())
            .unwrap_or(&RecoveryStrategy::Default);

        match strategy {
            RecoveryStrategy::Restart => {
                info!("Restarting agent {} due to recoverable error", agent_id);
                self.restart_agent(agent_id).await
            }
            RecoveryStrategy::ReloadConfiguration => {
                info!("Reloading configuration for agent {}", agent_id);
                self.reload_agent_config(agent_id).await
            }
            RecoveryStrategy::ClearCache => {
                info!("Clearing cache for agent {}", agent_id);
                self.clear_agent_cache(agent_id).await
            }
            RecoveryStrategy::FallbackMode => {
                warn!("Switching agent {} to fallback mode", agent_id);
                self.enable_fallback_mode(agent_id).await
            }
            RecoveryStrategy::ManualIntervention => {
                error!("Agent {} requires manual intervention", agent_id);
                self.trigger_alert(agent_id, error).await;
                Err(RecoveryError::ManualInterventionRequired)
            }
        }
    }
}
```

#### Memory System Recovery

**Index Corruption Recovery**:

```rust
async fn recover_from_index_corruption(
    corruption: &IndexCorruption
) -> Result<(), RecoveryError> {
    match corruption.extent {
        CorruptionExtent::Minor => {
            info!("Minor index corruption detected, attempting automatic repair");

            // Try incremental repair
            if let Ok(_) = repair_index_incremental().await {
                return Ok(());
            }

            // Fall back to partial rebuild
            rebuild_affected_indexes(&corruption.affected_indexes).await
        }

        CorruptionExtent::Major => {
            warn!("Major index corruption detected, rebuilding affected indexes");

            // Backup existing data
            backup_corrupted_indexes(&corruption.affected_indexes).await?;

            // Rebuild from source data
            rebuild_indexes_from_entities(&corruption.affected_indexes).await
        }

        CorruptionExtent::Complete => {
            error!("Complete index corruption detected, full rebuild required");

            // Emergency backup
            emergency_backup_all_data().await?;

            // Full system rebuild
            rebuild_entire_memory_system().await
        }
    }
}
```

### Error Monitoring and Alerting

#### Error Rate Monitoring

**Error Rate Thresholds**:

```yaml
monitoring:
  error_thresholds:
    agent_deployment_failures: 10%    # Per hour
    llm_provider_errors: 5%          # Per minute
    memory_search_timeouts: 2%       # Per minute
    message_delivery_failures: 1%    # Per minute

  alert_conditions:
    critical:
      - agent_startup_failures > 50% for 5 minutes
      - all_llm_providers_failing for 2 minutes
      - memory_system_unavailable for 1 minute
    warning:
      - error_rate_increase > 200% for 10 minutes
      - fallback_usage > 25% for 15 minutes
      - recovery_attempts_failing > 50% for 5 minutes
```

**Prometheus Metrics**:

```rust
// Error tracking metrics
caxton_errors_total{component, error_type, severity}
caxton_error_rate{component, time_window="5m"}
caxton_recovery_attempts_total{component, strategy, success}
caxton_fallback_usage_total{component, fallback_type}
caxton_circuit_breaker_state{component, provider}
```

#### Alert Escalation

**Alert Severity Levels**:

```yaml
alerts:
  severity_levels:
    p0_critical:
      description: "System unavailable or data loss risk"
      response_time: "immediate"
      escalation: "on-call engineer"
      examples:
        - "All LLM providers failing"
        - "Memory system data corruption"
        - "Agent deployment completely broken"

    p1_high:
      description: "Major functionality degraded"
      response_time: "15 minutes"
      escalation: "team slack channel"
      examples:
        - "Single LLM provider failing"
        - "High agent error rates"
        - "Memory search performance degraded"

    p2_medium:
      description: "Minor functionality issues"
      response_time: "2 hours"
      escalation: "email notification"
      examples:
        - "Individual agent failures"
        - "Elevated timeout rates"
        - "Cache hit ratio degraded"

    p3_low:
      description: "Informational or trend alerts"
      response_time: "next business day"
      escalation: "dashboard notification"
      examples:
        - "Resource usage trends"
        - "Model download warnings"
        - "Configuration validation issues"
```

### Manual Recovery Procedures

#### System Recovery Playbooks

**Complete System Recovery**:

```markdown
# Emergency System Recovery Procedure

## Prerequisites
- System administrator access
- Recent backup files
- Network connectivity to external dependencies

## Step 1: Assess System State
```bash
# Check system health
caxton status --detailed
caxton diagnose --all-components

# Check disk space and system resources
df -h
free -m
ps aux | grep caxton
```

## Step 2: Stop All Services

```bash
# Graceful shutdown
caxton shutdown --graceful --timeout 60s

# Force stop if needed
systemctl stop caxton
```

## Step 3: Backup Current State

```bash
# Backup configuration
cp -r /etc/caxton /backup/caxton-config-$(date +%Y%m%d)

# Backup data directory
tar -czf /backup/caxton-data-$(date +%Y%m%d).tar.gz /var/lib/caxton/
```

## Step 4: Restore from Backup

```bash
# Restore configuration
caxton restore-config --backup /backup/caxton-config-YYYYMMDD

# Restore data
caxton restore-data --backup /backup/caxton-data-YYYYMMDD.tar.gz --verify
```

## Step 5: Validate and Restart

```bash
# Validate configuration
caxton validate --config /etc/caxton/caxton.yaml

# Start services
systemctl start caxton
caxton health-check --wait-ready --timeout 300s
```

```text

```

**LLM Provider Recovery**:

```markdown
# LLM Provider Recovery Procedure

## Symptoms
- All LLM providers showing connection errors
- Configuration agents unable to execute
- Timeout errors in logs

## Diagnosis
```bash
# Check provider connectivity
caxton llm test-connection --all-providers
caxton llm check-quotas --provider openai
caxton llm check-models --provider anthropic

# Review error logs
caxton logs --component llm-provider --level error --tail 100
```

## Recovery Steps

### 1. Network Connectivity

```bash
# Test external connectivity
curl -I https://api.openai.com/v1/models
curl -I https://api.anthropic.com/v1/models

# Check DNS resolution
nslookup api.openai.com
```

### 2. Authentication Validation

```bash
# Verify API keys
caxton llm validate-auth --provider openai
caxton llm rotate-keys --provider anthropic --confirm

# Check token expiration
caxton llm token-info --all-providers
```

### 3. Provider Configuration

```bash
# Reset provider configuration
caxton llm reset-provider --provider openai
caxton llm reconfigure --provider anthropic --interactive

# Test with minimal request
caxton llm test-chat --provider openai --prompt "Hello"
```

### 4. Fallback Activation

```bash
# Enable local fallback
caxton llm enable-fallback --provider ollama
caxton llm set-priority --provider ollama --priority 100

# Verify fallback working
caxton agent test --agent-id test-agent --use-fallback
```

```text

#### Data Recovery Procedures

**Memory System Data Recovery**:
```markdown
# Memory System Recovery Procedure

## Corruption Detection
```bash
# Check index integrity
caxton memory validate --all-indexes
caxton memory check-consistency --repair-minor

# Generate corruption report
caxton memory diagnose --output corruption-report.json
```

## Recovery Options

### Option 1: Incremental Repair (Minor Corruption)

```bash
# Attempt automatic repair
caxton memory repair --incremental --backup-first
caxton memory reindex --affected-only

# Validate repair
caxton memory validate --repaired-indexes
caxton memory test-search --query "test query"
```

### Option 2: Partial Rebuild (Major Corruption)

```bash
# Backup corrupted data
caxton memory backup --corrupted-only --output corrupted-backup.json

# Rebuild affected indexes
caxton memory rebuild --indexes vector,relationship --source entities

# Verify rebuild
caxton memory validate --rebuilt-indexes
```

### Option 3: Complete Rebuild (Complete Corruption)

```bash
# Full data export (if possible)
caxton memory export --format json --output full-backup.json

# Complete system rebuild
caxton memory rebuild --complete --confirm-data-loss

# Reimport data
caxton memory import --source full-backup.json --validate
```

```text

### Configuration Examples

#### Error Handling Configuration

```yaml
# caxton-error-handling.yaml
error_handling:
  # Global error policies
  global:
    max_retries: 3
    retry_backoff_ms: [1000, 2000, 5000]
    circuit_breaker_enabled: true
    fallback_enabled: true

  # Component-specific error handling
  components:
    llm_providers:
      timeout_seconds: 30
      max_concurrent_requests: 10
      circuit_breaker:
        failure_threshold: 5
        success_threshold: 3
        timeout_seconds: 60
      fallback_chain: ["openai", "anthropic", "ollama"]

    memory_system:
      search_timeout_ms: 5000
      embedding_timeout_ms: 10000
      corruption_detection: true
      auto_repair_enabled: true
      backup_on_corruption: true

    message_routing:
      delivery_timeout_ms: 2000
      max_queue_size: 1000
      dead_letter_queue: true
      retry_failed_messages: true

  # Monitoring and alerting
  monitoring:
    error_rate_window: "5m"
    alert_thresholds:
      error_rate: 5.0
      timeout_rate: 2.0
      failure_rate: 1.0
    prometheus_enabled: true
    webhook_alerts: "https://alerts.company.com/caxton"

  # Recovery strategies
  recovery:
    automatic_recovery: true
    recovery_timeout_minutes: 15
    max_recovery_attempts: 3
    manual_intervention_required:
      - "data_corruption"
      - "security_breach"
```

## Related Documentation

- [Operational Runbook](operational-runbook.md) - Standard operational
  procedures
- [Performance Specifications](../api/performance-specifications.md) - SLO
  definitions and monitoring
- [LLM Provider Integration](../config-agents/llm-providers.md) - Provider-specific
  error handling
- [Memory System Management](../memory-system/model-management.md) - Model and
  memory error recovery
- [Security Guide](../developer-guide/security-guide.md) - Security-related
  error handling

## Next Steps

1. **Error Monitoring Setup**: Implement comprehensive error tracking and alerting
2. **Recovery Testing**: Validate recovery procedures through chaos engineering
3. **Documentation Updates**: Maintain up-to-date runbooks and procedures
4. **Team Training**: Ensure operations team is familiar with recovery procedures
5. **Automation**: Automate common recovery patterns where appropriate
