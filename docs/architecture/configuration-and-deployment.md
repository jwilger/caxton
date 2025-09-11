---
title: "Configuration and Deployment Architecture"
date: 2025-01-14
layout: page
categories: [architecture, deployment]
---

## Overview

This document specifies the comprehensive configuration architecture and
deployment patterns for the Caxton Message Router. The design supports multiple
deployment scenarios from development to large-scale production with
hot-reloading and operational flexibility.

## Configuration Architecture

### Hierarchical Configuration System

```rust
/// Complete configuration hierarchy for message router
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Core performance and throughput settings
    pub performance: PerformanceConfig,

    /// Message delivery configuration
    pub delivery: DeliveryConfig,

    /// Conversation management settings
    pub conversation: ConversationConfig,

    /// Agent registry configuration
    pub registry: RegistryConfig,

    /// Observability and monitoring
    pub observability: ObservabilityConfig,

    /// WASM runtime integration
    pub wasm_integration: WasmIntegrationConfig,

    /// Local storage settings
    pub storage: StorageConfig,

    /// Network and clustering
    pub network: NetworkConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Feature flags and experiments
    pub features: FeatureConfig,
}

/// Performance and throughput configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    // Queue configurations
    pub inbound_queue_size: ChannelCapacity,
    pub outbound_queue_size: ChannelCapacity,
    pub agent_queue_size: AgentQueueSize,

    // Threading and parallelism
    pub worker_thread_count: WorkerThreadCount,
    pub pipeline_count: PipelineCount,
    pub batch_size: MessageBatchSize,

    // Timeout settings
    pub message_timeout: MessageTimeoutMs,
    pub processing_timeout: ProcessingTimeoutMs,
    pub health_check_timeout: HealthCheckTimeoutMs,

    // Resource limits
    pub max_concurrent_messages: MaxConcurrentMessages,
    pub memory_limit: MemoryBytes,
    pub cpu_limit_percent: CpuLimitPercent,

    // Cache settings
    pub positive_cache_size: CacheSize,
    pub negative_cache_size: CacheSize,
    pub capability_cache_size: CacheSize,

    // Advanced optimizations
    pub enable_simd_optimizations: bool,
    pub enable_zero_copy: bool,
    pub prefetch_distance: PrefetchDistance,
}

/// Message delivery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryConfig {
    // Connection management
    pub connection_pool_size: ConnectionPoolSize,
    pub connection_timeout: ConnectionTimeoutMs,
    pub keep_alive_interval: KeepAliveIntervalMs,
    pub idle_connection_timeout: IdleConnectionTimeoutMs,

    // Retry configuration
    pub max_retries: MaxRetries,
    pub base_retry_delay: RetryDelayMs,
    pub retry_backoff_factor: RetryBackoffFactor,
    pub retry_jitter_ms: RetryJitterMs,
    pub retry_max_delay: RetryMaxDelayMs,

    // Circuit breaker settings
    pub circuit_breaker_threshold: CircuitBreakerThreshold,
    pub circuit_breaker_timeout: CircuitBreakerTimeoutMs,
    pub circuit_breaker_half_open_max_calls: HalfOpenMaxCalls,
    pub circuit_breaker_recovery_timeout: RecoveryTimeoutMs,

    // Dead letter queue
    pub dead_letter_queue_size: DeadLetterQueueSize,
    pub dead_letter_retention: DeadLetterRetentionMs,
    pub dead_letter_max_retries: MaxRetries,

    // Batch processing
    pub enable_batch_delivery: bool,
    pub batch_timeout: BatchTimeoutMs,
    pub max_batch_size: MessageBatchSize,

    // Compression and serialization
    pub enable_compression: bool,
    pub compression_algorithm: CompressionAlgorithm,
    pub compression_level: CompressionLevel,
}
```

### Configuration Validation and Constraints

```rust
impl RouterConfig {
    /// Validates configuration for internal consistency and resource feasibility
    pub fn validate(&self) -> Result<ValidationReport, ConfigError> {
        let mut report = ValidationReport::new();

        // Performance validation
        self.validate_performance_config(&mut report)?;

        // Resource limit validation
        self.validate_resource_limits(&mut report)?;

        // Timeout relationship validation
        self.validate_timeout_relationships(&mut report)?;

        // Network configuration validation
        self.validate_network_config(&mut report)?;

        // Security configuration validation
        self.validate_security_config(&mut report)?;

        if report.has_errors() {
            Err(ConfigError::ValidationFailed(report))
        } else {
            Ok(report)
        }
    }

    /// Validates performance configuration consistency
    fn validate_performance_config(&self, report: &mut ValidationReport) -> Result<(), ConfigError> {
        // Queue size relationships
        if self.performance.inbound_queue_size.as_usize() < self.performance.batch_size.as_usize() {
            report.add_error(
                "performance.inbound_queue_size",
                "Queue size must be at least 2x batch size for optimal performance"
            );
        }

        // Thread count vs CPU cores
        let available_cores = num_cpus::get();
        if self.performance.worker_thread_count.as_usize() > available_cores * 2 {
            report.add_warning(
                "performance.worker_thread_count",
                format!("Thread count ({}) exceeds 2x CPU cores ({})",
                       self.performance.worker_thread_count.as_usize(), available_cores)
            );
        }

        // Memory limits
        let estimated_memory = self.estimate_memory_usage()?;
        if estimated_memory > self.performance.memory_limit {
            report.add_error(
                "performance.memory_limit",
                format!("Estimated memory usage ({}) exceeds limit ({})",
                       estimated_memory, self.performance.memory_limit)
            );
        }

        Ok(())
    }

    /// Estimates memory usage based on configuration
    fn estimate_memory_usage(&self) -> Result<MemoryBytes, ConfigError> {
        let queue_memory = self.estimate_queue_memory()?;
        let cache_memory = self.estimate_cache_memory()?;
        let connection_memory = self.estimate_connection_memory()?;
        let overhead_memory = MemoryBytes::from_mb(100)?; // Base overhead

        Ok(MemoryBytes::try_new(
            queue_memory.as_usize() +
            cache_memory.as_usize() +
            connection_memory.as_usize() +
            overhead_memory.as_usize()
        )?)
    }

    /// Provides configuration recommendations based on deployment profile
    pub fn recommend_optimizations(&self, profile: DeploymentProfile) -> ConfigRecommendations {
        let mut recommendations = ConfigRecommendations::new();

        match profile {
            DeploymentProfile::Development => {
                self.add_development_recommendations(&mut recommendations);
            }
            DeploymentProfile::Testing => {
                self.add_testing_recommendations(&mut recommendations);
            }
            DeploymentProfile::Staging => {
                self.add_staging_recommendations(&mut recommendations);
            }
            DeploymentProfile::Production => {
                self.add_production_recommendations(&mut recommendations);
            }
            DeploymentProfile::HighThroughput => {
                self.add_high_throughput_recommendations(&mut recommendations);
            }
        }

        recommendations
    }
}
```

### Environment-Specific Configurations

```rust
/// Predefined configurations for different deployment environments
impl RouterConfig {
    /// Development configuration: High observability, debugging features
    pub fn development() -> Self {
        Self {
            performance: PerformanceConfig {
                inbound_queue_size: ChannelCapacity::try_new(1_000).unwrap(),
                outbound_queue_size: ChannelCapacity::try_new(1_000).unwrap(),
                agent_queue_size: AgentQueueSize::try_new(100).unwrap(),
                worker_thread_count: WorkerThreadCount::try_new(2).unwrap(),
                pipeline_count: PipelineCount::try_new(2).unwrap(),
                batch_size: MessageBatchSize::try_new(10).unwrap(),
                message_timeout: MessageTimeoutMs::try_new(30_000).unwrap(),
                processing_timeout: ProcessingTimeoutMs::try_new(5_000).unwrap(),
                max_concurrent_messages: MaxConcurrentMessages::try_new(1_000).unwrap(),
                memory_limit: MemoryBytes::from_mb(512).unwrap(),
                positive_cache_size: CacheSize::try_new(1_000).unwrap(),
                negative_cache_size: CacheSize::try_new(500).unwrap(),
                capability_cache_size: CacheSize::try_new(100).unwrap(),
                enable_simd_optimizations: false, // Easier debugging
                enable_zero_copy: false,
                prefetch_distance: PrefetchDistance::try_new(1).unwrap(),
                ..Default::default()
            },
            delivery: DeliveryConfig {
                max_retries: MaxRetries::try_new(2).unwrap(),
                base_retry_delay: RetryDelayMs::try_new(1_000).unwrap(),
                circuit_breaker_threshold: CircuitBreakerThreshold::try_new(3).unwrap(),
                dead_letter_queue_size: DeadLetterQueueSize::try_new(1_000).unwrap(),
                enable_batch_delivery: false, // Simpler debugging
                enable_compression: false,
                ..Default::default()
            },
            observability: ObservabilityConfig {
                tracing_enabled: true,
                metrics_enabled: true,
                sampling_ratio: TraceSamplingRatio::try_new(1.0).unwrap(), // 100% sampling
                export_interval: MetricsExportIntervalMs::try_new(5_000).unwrap(),
                log_level: LogLevel::Debug,
                structured_logging: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Production configuration: Optimized for throughput and reliability
    pub fn production() -> Self {
        Self {
            performance: PerformanceConfig {
                inbound_queue_size: ChannelCapacity::try_new(100_000).unwrap(),
                outbound_queue_size: ChannelCapacity::try_new(100_000).unwrap(),
                agent_queue_size: AgentQueueSize::try_new(10_000).unwrap(),
                worker_thread_count: WorkerThreadCount::try_new(16).unwrap(),
                pipeline_count: PipelineCount::try_new(8).unwrap(),
                batch_size: MessageBatchSize::try_new(1_000).unwrap(),
                message_timeout: MessageTimeoutMs::try_new(10_000).unwrap(),
                processing_timeout: ProcessingTimeoutMs::try_new(2_000).unwrap(),
                max_concurrent_messages: MaxConcurrentMessages::try_new(1_000_000).unwrap(),
                memory_limit: MemoryBytes::from_mb(8_192).unwrap(), // 8GB
                positive_cache_size: CacheSize::try_new(100_000).unwrap(),
                negative_cache_size: CacheSize::try_new(10_000).unwrap(),
                capability_cache_size: CacheSize::try_new(1_000).unwrap(),
                enable_simd_optimizations: true,
                enable_zero_copy: true,
                prefetch_distance: PrefetchDistance::try_new(8).unwrap(),
                ..Default::default()
            },
            delivery: DeliveryConfig {
                connection_pool_size: ConnectionPoolSize::try_new(100).unwrap(),
                max_retries: MaxRetries::try_new(5).unwrap(),
                base_retry_delay: RetryDelayMs::try_new(500).unwrap(),
                circuit_breaker_threshold: CircuitBreakerThreshold::try_new(20).unwrap(),
                dead_letter_queue_size: DeadLetterQueueSize::try_new(1_000_000).unwrap(),
                enable_batch_delivery: true,
                enable_compression: true,
                compression_algorithm: CompressionAlgorithm::Lz4,
                ..Default::default()
            },
            observability: ObservabilityConfig {
                tracing_enabled: true,
                metrics_enabled: true,
                sampling_ratio: TraceSamplingRatio::try_new(0.01).unwrap(), // 1% sampling
                export_interval: MetricsExportIntervalMs::try_new(60_000).unwrap(),
                log_level: LogLevel::Info,
                structured_logging: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// High-throughput configuration: Maximum performance for large deployments
    pub fn high_throughput() -> Self {
        let mut config = Self::production();

        // Maximize throughput settings
        config.performance.inbound_queue_size = ChannelCapacity::try_new(1_000_000).unwrap();
        config.performance.outbound_queue_size = ChannelCapacity::try_new(1_000_000).unwrap();
        config.performance.batch_size = MessageBatchSize::try_new(10_000).unwrap();
        config.performance.worker_thread_count = WorkerThreadCount::try_new(32).unwrap();
        config.performance.pipeline_count = PipelineCount::try_new(16).unwrap();
        config.performance.memory_limit = MemoryBytes::from_mb(32_768).unwrap(); // 32GB

        // Aggressive caching
        config.performance.positive_cache_size = CacheSize::try_new(1_000_000).unwrap();
        config.performance.negative_cache_size = CacheSize::try_new(100_000).unwrap();

        // Optimized delivery
        config.delivery.enable_batch_delivery = true;
        config.delivery.max_batch_size = MessageBatchSize::try_new(10_000).unwrap();
        config.delivery.batch_timeout = BatchTimeoutMs::try_new(10).unwrap(); // Very low latency

        // Minimal observability overhead
        config.observability.sampling_ratio = TraceSamplingRatio::try_new(0.001).unwrap(); // 0.1%
        config.observability.export_interval = MetricsExportIntervalMs::try_new(300_000).unwrap();

        config
    }
}
```

## Dynamic Configuration Management

### Hot-Reload Architecture

```rust
/// Configuration manager supporting hot-reload and validation
pub struct ConfigManager {
    current_config: Arc<RwLock<RouterConfig>>,
    config_source: Box<dyn ConfigSource>,
    validation_rules: ValidationRules,
    change_listeners: Vec<Box<dyn ConfigChangeListener>>,
    reload_strategy: ReloadStrategy,
}

impl ConfigManager {
    /// Attempts to reload configuration with validation
    pub async fn reload_config(&self) -> Result<ReloadResult, ConfigError> {
        // Load new configuration
        let new_config = self.config_source.load().await?;

        // Validate new configuration
        let validation_result = new_config.validate()?;
        if validation_result.has_errors() {
            return Err(ConfigError::ValidationFailed(validation_result));
        }

        // Check compatibility with current config
        let current_config = self.current_config.read().await;
        let compatibility = self.check_compatibility(&current_config, &new_config)?;

        match compatibility {
            CompatibilityResult::HotReloadable(changes) => {
                self.apply_hot_reload(&new_config, changes).await
            }
            CompatibilityResult::RequiresRestart(reason) => {
                self.schedule_restart(new_config, reason).await
            }
            CompatibilityResult::Incompatible(errors) => {
                Err(ConfigError::IncompatibleConfiguration(errors))
            }
        }
    }

    /// Applies configuration changes that can be hot-reloaded
    async fn apply_hot_reload(
        &self,
        new_config: &RouterConfig,
        changes: Vec<ConfigChange>
    ) -> Result<ReloadResult, ConfigError> {
        // Validate that hot-reload is safe
        self.validate_hot_reload_safety(&changes)?;

        // Apply changes in order of dependency
        let ordered_changes = self.order_changes_by_dependency(changes)?;

        let mut applied_changes = Vec::new();
        let mut rollback_actions = Vec::new();

        for change in ordered_changes {
            match self.apply_single_change(&change).await {
                Ok(rollback) => {
                    applied_changes.push(change);
                    rollback_actions.push(rollback);
                }
                Err(e) => {
                    // Rollback all previously applied changes
                    self.rollback_changes(rollback_actions).await?;
                    return Err(ConfigError::HotReloadFailed {
                        applied_changes,
                        failed_change: change,
                        error: e
                    });
                }
            }
        }

        // Update current configuration
        *self.current_config.write().await = new_config.clone();

        // Notify listeners of successful reload
        self.notify_config_reloaded(new_config, &applied_changes).await;

        Ok(ReloadResult::HotReloadSuccessful {
            applied_changes,
            validation_warnings: validation_result.warnings
        })
    }

    /// Validates that hot-reload changes are safe to apply
    fn validate_hot_reload_safety(&self, changes: &[ConfigChange]) -> Result<(), ConfigError> {
        for change in changes {
            match change {
                ConfigChange::QueueSizeIncrease { .. } => {
                    // Safe - can increase queue sizes without data loss
                    continue;
                }
                ConfigChange::QueueSizeDecrease { from, to } => {
                    // Check if current queue usage allows decrease
                    let current_usage = self.get_current_queue_usage()?;
                    if current_usage > to.as_usize() {
                        return Err(ConfigError::UnsafeHotReload {
                            change: change.clone(),
                            reason: format!("Queue usage ({}) exceeds new size ({})",
                                          current_usage, to.as_usize())
                        });
                    }
                }
                ConfigChange::TimeoutDecrease { .. } => {
                    // Check if there are operations that would timeout
                    let long_running_ops = self.get_long_running_operations()?;
                    if !long_running_ops.is_empty() {
                        return Err(ConfigError::UnsafeHotReload {
                            change: change.clone(),
                            reason: format!("{} long-running operations would timeout",
                                          long_running_ops.len())
                        });
                    }
                }
                ConfigChange::ObservabilitySettings { .. } => {
                    // Always safe to change observability settings
                    continue;
                }
                ConfigChange::SecuritySettings { .. } => {
                    // Security changes require restart for safety
                    return Err(ConfigError::RequiresRestart {
                        reason: "Security configuration changes require restart".to_string()
                    });
                }
            }
        }

        Ok(())
    }
}
```

### Configuration Sources

```rust
/// Trait for configuration sources (files, databases, remote configs)
#[async_trait]
pub trait ConfigSource: Send + Sync {
    /// Loads configuration from the source
    async fn load(&self) -> Result<RouterConfig, ConfigError>;

    /// Watches for configuration changes
    async fn watch(&self) -> Result<ConfigWatcher, ConfigError>;

    /// Saves configuration to the source (if writable)
    async fn save(&self, config: &RouterConfig) -> Result<(), ConfigError>;
}

/// File-based configuration source
pub struct FileConfigSource {
    file_path: PathBuf,
    format: ConfigFormat,
    watcher: Option<notify::RecommendedWatcher>,
}

impl FileConfigSource {
    /// Creates a new file-based configuration source
    pub fn new(file_path: impl Into<PathBuf>, format: ConfigFormat) -> Self {
        Self {
            file_path: file_path.into(),
            format,
            watcher: None,
        }
    }

    /// Enables file watching for automatic reloads
    pub async fn enable_watching(&mut self) -> Result<ConfigWatcher, ConfigError> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        let watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
            match result {
                Ok(event) => {
                    if event.kind.is_modify() {
                        let _ = tx.blocking_send(ConfigChangeEvent::FileModified);
                    }
                }
                Err(e) => {
                    let _ = tx.blocking_send(ConfigChangeEvent::WatchError(e.to_string()));
                }
            }
        })?;

        watcher.watch(&self.file_path, notify::RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);

        Ok(ConfigWatcher::new(rx))
    }
}

#[async_trait]
impl ConfigSource for FileConfigSource {
    async fn load(&self) -> Result<RouterConfig, ConfigError> {
        let content = tokio::fs::read_to_string(&self.file_path).await
            .map_err(|e| ConfigError::FileReadError {
                path: self.file_path.clone(),
                error: e
            })?;

        match self.format {
            ConfigFormat::Toml => {
                toml::from_str(&content)
                    .map_err(|e| ConfigError::ParseError {
                        format: self.format,
                        error: e.to_string()
                    })
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(&content)
                    .map_err(|e| ConfigError::ParseError {
                        format: self.format,
                        error: e.to_string()
                    })
            }
            ConfigFormat::Json => {
                serde_json::from_str(&content)
                    .map_err(|e| ConfigError::ParseError {
                        format: self.format,
                        error: e.to_string()
                    })
            }
        }
    }

    async fn save(&self, config: &RouterConfig) -> Result<(), ConfigError> {
        let content = match self.format {
            ConfigFormat::Toml => toml::to_string_pretty(config)
                .map_err(|e| ConfigError::SerializeError {
                    format: self.format,
                    error: e.to_string()
                })?,
            ConfigFormat::Yaml => serde_yaml::to_string(config)
                .map_err(|e| ConfigError::SerializeError {
                    format: self.format,
                    error: e.to_string()
                })?,
            ConfigFormat::Json => serde_json::to_string_pretty(config)
                .map_err(|e| ConfigError::SerializeError {
                    format: self.format,
                    error: e.to_string()
                })?,
        };

        tokio::fs::write(&self.file_path, content).await
            .map_err(|e| ConfigError::FileWriteError {
                path: self.file_path.clone(),
                error: e
            })
    }
}
```

## Deployment Patterns

### Container Deployment

```dockerfile
# Multi-stage Dockerfile for optimal container images

# Build stage
FROM rust:1.75-slim as builder
WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build application
RUN touch src/main.rs && \
    cargo build --release --bin caxton

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false caxton

# Copy binary
COPY --from=builder /build/target/release/caxton /usr/local/bin/caxton

# Copy default configuration
COPY config/production.toml /etc/caxton/config.toml

# Create data directories
RUN mkdir -p /var/lib/caxton /var/log/caxton && \
    chown -R caxton:caxton /var/lib/caxton /var/log/caxton

# Switch to non-root user
USER caxton

# Expose ports
EXPOSE 8080 8081 7946

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

# Command
CMD ["/usr/local/bin/caxton", "--config", "/etc/caxton/config.toml"]
```

### Kubernetes Deployment

```yaml
# Complete Kubernetes deployment for Caxton Message Router
apiVersion: apps/v1
kind: Deployment
metadata:
  name: caxton-message-router
  namespace: caxton-system
  labels:
    app: caxton-message-router
    component: message-router
    version: v0.1.3
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: caxton-message-router
  template:
    metadata:
      labels:
        app: caxton-message-router
        component: message-router
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8081"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: caxton-message-router
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
        fsGroup: 65534
      containers:
      - name: message-router
        image: caxton/message-router:v0.1.3
        imagePullPolicy: IfNotPresent
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        - name: metrics
          containerPort: 8081
          protocol: TCP
        - name: gossip
          containerPort: 7946
          protocol: TCP
        env:
        - name: CAXTON_NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: CAXTON_CLUSTER_SEEDS
          value: "caxton-message-router-0.caxton-message-router:7946,caxton-message-router-1.caxton-message-router:7946"
        - name: RUST_LOG
          value: "caxton=info,caxton::message_router=debug"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://jaeger-collector:14268"
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 2000m
            memory: 4Gi
        livenessProbe:
          httpGet:
            path: /health
            port: metrics
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: metrics
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        volumeMounts:
        - name: config
          mountPath: /etc/caxton
          readOnly: true
        - name: data
          mountPath: /var/lib/caxton
        - name: logs
          mountPath: /var/log/caxton
      volumes:
      - name: config
        configMap:
          name: caxton-config
      - name: data
        persistentVolumeClaim:
          claimName: caxton-data
      - name: logs
        emptyDir: {}
      nodeSelector:
        kubernetes.io/arch: amd64
      tolerations:
      - key: "caxton.ai/message-router"
        operator: "Equal"
        value: "true"
        effect: "NoSchedule"

---
apiVersion: v1
kind: Service
metadata:
  name: caxton-message-router
  namespace: caxton-system
  labels:
    app: caxton-message-router
spec:
  type: ClusterIP
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: metrics
    port: 8081
    targetPort: 8081
  - name: gossip
    port: 7946
    targetPort: 7946
  selector:
    app: caxton-message-router

---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: caxton-message-router
  namespace: caxton-system

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: caxton-config
  namespace: caxton-system
data:
  config.toml: |
    [performance]
    inbound_queue_size = 100000
    outbound_queue_size = 100000
    worker_thread_count = 16
    batch_size = 1000
    message_timeout_ms = 10000
    memory_limit = 3221225472  # 3GB

    [delivery]
    connection_pool_size = 100
    max_retries = 5
    circuit_breaker_threshold = 20
    dead_letter_queue_size = 1000000

    [observability]
    tracing_enabled = true
    metrics_enabled = true
    sampling_ratio = 0.01  # 1% sampling
    export_interval_ms = 60000

    [network]
    bind_address = "0.0.0.0:8080"
    metrics_address = "0.0.0.0:8081"
    gossip_address = "0.0.0.0:7946"

    [storage]
    database_path = "/var/lib/caxton/router.db"
```

### Monitoring and Alerting

```yaml
# Prometheus monitoring rules
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: caxton-message-router-alerts
  namespace: caxton-system
spec:
  groups:
  - name: caxton.message-router
    rules:
    - alert: CaxtonMessageRouterDown
      expr: up{job="caxton-message-router"} == 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "Caxton Message Router is down"
        description: "Message Router instance {{ $labels.instance }} has been down for more than 1 minute."

    - alert: CaxtonHighMessageLatency
      expr: caxton_message_routing_duration_p99 > 0.005  # 5ms
      for: 2m
      labels:
        severity: warning
      annotations:
        summary: "High message routing latency"
        description: "P99 message routing latency is {{ $value }}s on {{ $labels.instance }}"

    - alert: CaxtonHighErrorRate
      expr: rate(caxton_message_routing_errors_total[5m]) > 0.01  # 1% error rate
      for: 2m
      labels:
        severity: critical
      annotations:
        summary: "High message routing error rate"
        description: "Message routing error rate is {{ $value | humanizePercentage }} on {{ $labels.instance }}"

    - alert: CaxtonQueueDepthHigh
      expr: caxton_message_queue_depth > 10000
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High message queue depth"
        description: "Message queue depth is {{ $value }} on {{ $labels.instance }}"

    - alert: CaxtonMemoryUsageHigh
      expr: caxton_memory_usage_bytes / caxton_memory_limit_bytes > 0.9
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "High memory usage"
        description: "Memory usage is {{ $value | humanizePercentage }} of limit on {{ $labels.instance }}"
```

## Operational Runbooks

### Performance Tuning Guide

```bash
#!/bin/bash
# Performance tuning script for Caxton Message Router

set -euo pipefail

echo "=== Caxton Message Router Performance Tuning ==="

# System optimization
echo "1. Optimizing system settings..."
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
sudo sysctl -w net.ipv4.tcp_rmem="4096 16384 134217728"
sudo sysctl -w net.ipv4.tcp_wmem="4096 65536 134217728"
sudo sysctl -w net.core.netdev_max_backlog=30000
sudo sysctl -w net.core.netdev_budget=600

# CPU optimization
echo "2. Setting CPU governor to performance..."
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Memory optimization
echo "3. Configuring memory settings..."
sudo sysctl -w vm.swappiness=1
sudo sysctl -w vm.dirty_ratio=80
sudo sysctl -w vm.dirty_background_ratio=5

# Generate optimized configuration
echo "4. Generating optimized configuration..."
CPU_CORES=$(nproc)
MEMORY_GB=$(free -g | awk '/^Mem:/{print $2}')

cat > /tmp/caxton-optimized.toml <<EOF
[performance]
worker_thread_count = $((CPU_CORES * 2))
inbound_queue_size = 1000000
outbound_queue_size = 1000000
memory_limit = $((MEMORY_GB * 1024 * 1024 * 1024 * 3 / 4))  # 75% of system memory
enable_simd_optimizations = true
enable_zero_copy = true

[delivery]
connection_pool_size = 200
enable_batch_delivery = true
max_batch_size = 10000
batch_timeout_ms = 1

[observability]
sampling_ratio = 0.001  # 0.1% for minimal overhead
export_interval_ms = 300000
EOF

echo "Optimized configuration written to /tmp/caxton-optimized.toml"
echo "Review and deploy the configuration for optimal performance."
```

## Summary

This configuration and deployment architecture provides:

1. **Hierarchical Configuration**: Comprehensive settings for all components
2. **Environment Profiles**: Optimized configs for dev/staging/production
3. **Hot-Reload Support**: Safe runtime configuration updates
4. **Multiple Sources**: File, database, and remote configuration sources
5. **Container Ready**: Optimized Docker images and Kubernetes manifests
6. **Monitoring Integration**: Prometheus metrics and alerting rules
7. **Performance Tuning**: System optimization and configuration recommendations
8. **Operational Tools**: Deployment scripts and performance tuning utilities

The architecture enables smooth deployment and operation of the Caxton Message
Router across diverse environments while maintaining high performance and
reliability.
