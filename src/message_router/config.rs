//! Router configuration for development and production environments
//!
//! Provides pre-configured settings optimized for different deployment scenarios
//! with validation and builder pattern support.

use crate::message_router::domain_types::{
    ChannelCapacity, CircuitBreakerThreshold, CircuitBreakerTimeoutMs, ConversationTimeoutMs,
    DeadLetterQueueSize, HealthCheckIntervalMs, MaxConversationParticipants, MaxRetries,
    MessageBatchSize, MessageTimeoutMs, RetryBackoffFactor, RetryDelayMs, TraceSamplingRatio,
    WorkerThreadCount,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid configuration: {field} - {reason}")]
    ValidationError { field: String, reason: String },

    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
}

/// Observability configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Trace sampling ratio (0.0 to 1.0)
    pub trace_sampling_ratio: TraceSamplingRatio,
    /// Enable Prometheus metrics collection
    pub enable_metrics: bool,
    /// Enable detailed structured logging
    pub enable_detailed_logs: bool,
}

impl ObservabilityConfig {
    /// High observability for development and debugging
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded configuration values are invalid (should never happen in practice)
    pub fn development() -> Self {
        Self {
            trace_sampling_ratio: TraceSamplingRatio::try_new(1.0).unwrap(),
            enable_metrics: true,
            enable_detailed_logs: true,
        }
    }

    /// Production observability with sampling
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded configuration values are invalid (should never happen in practice)
    pub fn production() -> Self {
        Self {
            trace_sampling_ratio: TraceSamplingRatio::try_new(0.01).unwrap(),
            enable_metrics: true,
            enable_detailed_logs: false,
        }
    }

    /// Minimal observability for testing
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded configuration values are invalid (should never happen in practice)
    pub fn testing() -> Self {
        Self {
            trace_sampling_ratio: TraceSamplingRatio::try_new(0.0).unwrap(),
            enable_metrics: false,
            enable_detailed_logs: false,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self::development()
    }
}

/// Performance optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable message batching for throughput
    pub enable_batching: bool,
    /// Enable connection pooling for efficiency
    pub enable_connection_pooling: bool,
    /// Number of connections to pool
    pub connection_pool_size: usize,
    /// Enable message compression
    pub enable_compression: bool,
}

impl PerformanceConfig {
    /// Development performance settings
    pub fn development() -> Self {
        Self {
            enable_batching: true,
            enable_connection_pooling: false,
            connection_pool_size: 5,
            enable_compression: false,
        }
    }

    /// Production performance settings
    pub fn production() -> Self {
        Self {
            enable_batching: true,
            enable_connection_pooling: true,
            connection_pool_size: 50,
            enable_compression: true,
        }
    }

    /// Testing performance settings
    pub fn testing() -> Self {
        Self {
            enable_batching: false,
            enable_connection_pooling: false,
            connection_pool_size: 1,
            enable_compression: false,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self::development()
    }
}

/// Security and validation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable message validation
    pub enable_message_validation: bool,
    /// Maximum message size in bytes
    pub max_message_size_bytes: usize,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Rate limit messages per second
    pub rate_limit_messages_per_second: usize,
}

impl SecurityConfig {
    /// Development security settings (relaxed)
    pub fn development() -> Self {
        Self {
            enable_message_validation: true,
            max_message_size_bytes: 1_048_576, // 1MB
            enable_rate_limiting: false,
            rate_limit_messages_per_second: 1000,
        }
    }

    /// Production security settings (strict)
    pub fn production() -> Self {
        Self {
            enable_message_validation: true,
            max_message_size_bytes: 10_485_760, // 10MB
            enable_rate_limiting: true,
            rate_limit_messages_per_second: 10_000,
        }
    }

    /// Testing security settings
    pub fn testing() -> Self {
        Self {
            enable_message_validation: true,
            max_message_size_bytes: 1024,
            enable_rate_limiting: false,
            rate_limit_messages_per_second: 100,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self::development()
    }
}

/// Storage and persistence settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Path for persistent storage
    pub storage_path: Option<PathBuf>,
    /// Enable message persistence
    pub enable_persistence: bool,
    /// Storage cleanup interval in milliseconds
    pub storage_cleanup_interval_ms: u64,
}

impl StorageConfig {
    /// Development storage settings (in-memory)
    pub fn development() -> Self {
        Self {
            storage_path: None,
            enable_persistence: false,
            storage_cleanup_interval_ms: 60_000,
        }
    }

    /// Production storage settings (persistent)
    pub fn production() -> Self {
        Self {
            storage_path: Some(PathBuf::from("./data/message_router")),
            enable_persistence: true,
            storage_cleanup_interval_ms: 3_600_000, // 1 hour
        }
    }

    /// Testing storage settings
    pub fn testing() -> Self {
        Self {
            storage_path: None,
            enable_persistence: false,
            storage_cleanup_interval_ms: 30_000,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::development()
    }
}

/// Complete router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    // Core routing settings
    pub inbound_queue_size: ChannelCapacity,
    pub outbound_queue_size: ChannelCapacity,
    pub message_timeout_ms: MessageTimeoutMs,
    pub message_batch_size: MessageBatchSize,
    pub worker_thread_count: WorkerThreadCount,

    // Retry and failure handling
    pub max_retries: MaxRetries,
    pub retry_delay_ms: RetryDelayMs,
    pub retry_backoff_factor: RetryBackoffFactor,
    pub dead_letter_queue_size: DeadLetterQueueSize,

    // Circuit breaker settings
    pub circuit_breaker_threshold: CircuitBreakerThreshold,
    pub circuit_breaker_timeout_ms: CircuitBreakerTimeoutMs,

    // Conversation management
    pub conversation_timeout_ms: ConversationTimeoutMs,
    pub max_conversation_participants: MaxConversationParticipants,

    // Health monitoring
    pub health_check_interval_ms: HealthCheckIntervalMs,

    // Grouped configuration settings
    pub observability: ObservabilityConfig,
    pub storage: StorageConfig,
    pub performance: PerformanceConfig,
    pub security: SecurityConfig,
}

/// Helper function to get worker thread count from environment or compute default
///
/// Reads `CAXTON_WORKER_THREADS` environment variable and validates it against
/// `WorkerThreadCount` constraints (1-32). If the environment variable is missing,
/// invalid, or out of range, falls back to computed default: `(num_cpus::get() * 2).clamp(2, 8)`.
///
/// This provides configurable thread counts for different deployment scenarios while
/// maintaining sensible defaults for most environments.
fn get_worker_thread_count_with_env_fallback() -> WorkerThreadCount {
    // Try to read from environment variable first
    if let Ok(env_value) = std::env::var("CAXTON_WORKER_THREADS")
        && let Ok(thread_count) = env_value.parse::<usize>()
    {
        // Validate the parsed value is within acceptable range (1-32)
        if let Ok(validated_count) = WorkerThreadCount::try_new(thread_count) {
            return validated_count;
        }
    }

    // Fall back to computed default if env var is missing, invalid, or out of range
    let computed_default = (num_cpus::get() * 2).clamp(2, 8);
    WorkerThreadCount::try_new(computed_default).unwrap()
}

impl RouterConfig {
    // Backward compatibility methods for deprecated fields

    /// Backward compatibility: get trace sampling ratio
    pub fn trace_sampling_ratio(&self) -> TraceSamplingRatio {
        self.observability.trace_sampling_ratio
    }

    /// Backward compatibility: check if metrics are enabled
    pub fn enable_metrics(&self) -> bool {
        self.observability.enable_metrics
    }

    /// Backward compatibility: check if detailed logs are enabled
    pub fn enable_detailed_logs(&self) -> bool {
        self.observability.enable_detailed_logs
    }

    /// Backward compatibility: get storage path
    pub fn storage_path(&self) -> Option<&PathBuf> {
        self.storage.storage_path.as_ref()
    }

    /// Backward compatibility: check if persistence is enabled
    pub fn enable_persistence(&self) -> bool {
        self.storage.enable_persistence
    }

    /// Backward compatibility: get storage cleanup interval
    pub fn storage_cleanup_interval_ms(&self) -> u64 {
        self.storage.storage_cleanup_interval_ms
    }

    /// Backward compatibility: check if batching is enabled
    pub fn enable_batching(&self) -> bool {
        self.performance.enable_batching
    }

    /// Backward compatibility: check if connection pooling is enabled
    pub fn enable_connection_pooling(&self) -> bool {
        self.performance.enable_connection_pooling
    }

    /// Backward compatibility: get connection pool size
    pub fn connection_pool_size(&self) -> usize {
        self.performance.connection_pool_size
    }

    /// Backward compatibility: check if compression is enabled
    pub fn enable_compression(&self) -> bool {
        self.performance.enable_compression
    }

    /// Backward compatibility: check if message validation is enabled
    pub fn enable_message_validation(&self) -> bool {
        self.security.enable_message_validation
    }

    /// Backward compatibility: get max message size
    pub fn max_message_size_bytes(&self) -> usize {
        self.security.max_message_size_bytes
    }

    /// Backward compatibility: check if rate limiting is enabled
    pub fn enable_rate_limiting(&self) -> bool {
        self.security.enable_rate_limiting
    }

    /// Backward compatibility: get rate limit
    pub fn rate_limit_messages_per_second(&self) -> usize {
        self.security.rate_limit_messages_per_second
    }
}

impl RouterConfig {
    /// Creates a development configuration optimized for debugging and testing
    ///
    /// Development settings prioritize:
    /// - High observability (detailed logs, high trace sampling)
    /// - Smaller queues for faster debugging
    /// - Shorter timeouts for faster feedback
    /// - In-memory storage for simplicity
    ///
    /// # Panics
    /// Panics if any of the hardcoded values are out of range for their domain types
    pub fn development() -> Self {
        Self {
            // Core routing - smaller queues for dev
            inbound_queue_size: ChannelCapacity::try_new(1_000).unwrap(),
            outbound_queue_size: ChannelCapacity::try_new(1_000).unwrap(),
            message_timeout_ms: MessageTimeoutMs::try_new(10_000).unwrap(), // 10 seconds
            message_batch_size: MessageBatchSize::try_new(10).unwrap(),
            worker_thread_count: WorkerThreadCount::try_new(2).unwrap(),

            // Retry settings - more aggressive for faster feedback
            max_retries: MaxRetries::try_new(2).unwrap(),
            retry_delay_ms: RetryDelayMs::try_new(500).unwrap(),
            retry_backoff_factor: RetryBackoffFactor::try_new(1.5).unwrap(),
            dead_letter_queue_size: DeadLetterQueueSize::try_new(10_000).unwrap(),

            // Circuit breaker - more sensitive in dev
            circuit_breaker_threshold: CircuitBreakerThreshold::try_new(3).unwrap(),
            circuit_breaker_timeout_ms: CircuitBreakerTimeoutMs::try_new(30_000).unwrap(),

            // Conversation management - shorter timeouts
            conversation_timeout_ms: ConversationTimeoutMs::try_new(600_000).unwrap(), // 10 minutes
            max_conversation_participants: MaxConversationParticipants::try_new(5).unwrap(),

            // Health monitoring - frequent checks
            health_check_interval_ms: HealthCheckIntervalMs::try_new(10_000).unwrap(),

            // Grouped configurations
            observability: ObservabilityConfig::development(),
            storage: StorageConfig::development(),
            performance: PerformanceConfig::development(),
            security: SecurityConfig::development(),
        }
    }

    /// Creates a production configuration optimized for performance and reliability
    ///
    /// Production settings prioritize:
    /// - High throughput (large queues, batching)
    /// - Reliability (persistence, longer timeouts)
    /// - Efficient resource usage (connection pooling, compression)
    /// - Appropriate observability (sampled tracing)
    ///
    /// # Environment Variables
    /// - `CAXTON_WORKER_THREADS`: Override worker thread count (1-32). Falls back to
    ///   `(num_cpus::get() * 2).clamp(2, 8)` if unset, invalid, or out of range.
    ///
    /// # Panics
    /// Panics if any of the hardcoded values are out of range for their domain types
    pub fn production() -> Self {
        Self {
            // Core routing - optimized for throughput
            inbound_queue_size: ChannelCapacity::try_new(100_000).unwrap(),
            outbound_queue_size: ChannelCapacity::try_new(50_000).unwrap(),
            message_timeout_ms: MessageTimeoutMs::try_new(30_000).unwrap(), // 30 seconds
            message_batch_size: MessageBatchSize::try_new(1000).unwrap(),
            worker_thread_count: get_worker_thread_count_with_env_fallback(),

            // Retry settings - balanced for reliability
            max_retries: MaxRetries::try_new(3).unwrap(),
            retry_delay_ms: RetryDelayMs::try_new(1000).unwrap(),
            retry_backoff_factor: RetryBackoffFactor::try_new(2.0).unwrap(),
            dead_letter_queue_size: DeadLetterQueueSize::try_new(1_000_000).unwrap(),

            // Circuit breaker - production resilience
            circuit_breaker_threshold: CircuitBreakerThreshold::try_new(10).unwrap(),
            circuit_breaker_timeout_ms: CircuitBreakerTimeoutMs::try_new(60_000).unwrap(),

            // Conversation management - longer timeouts for production workflows
            conversation_timeout_ms: ConversationTimeoutMs::default(), // 30 minutes
            max_conversation_participants: MaxConversationParticipants::try_new(20).unwrap(),

            // Health monitoring - less frequent to reduce overhead
            health_check_interval_ms: HealthCheckIntervalMs::try_new(60_000).unwrap(),

            // Grouped configurations
            observability: ObservabilityConfig::production(),
            storage: StorageConfig::production(),
            performance: PerformanceConfig::production(),
            security: SecurityConfig::production(),
        }
    }

    /// Creates a configuration builder for custom settings
    pub fn builder() -> RouterConfigBuilder {
        RouterConfigBuilder::new()
    }

    /// Validates the configuration for consistency and reasonable values
    ///
    /// # Errors
    /// Returns `ConfigError` if any configuration values are inconsistent or invalid
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate queue sizes are reasonable
        if self.inbound_queue_size.as_usize() < 10 {
            return Err(ConfigError::ValidationError {
                field: "inbound_queue_size".to_string(),
                reason: "Must be at least 10".to_string(),
            });
        }

        if self.outbound_queue_size.as_usize() < 10 {
            return Err(ConfigError::ValidationError {
                field: "outbound_queue_size".to_string(),
                reason: "Must be at least 10".to_string(),
            });
        }

        // Validate timeouts are reasonable
        if self.message_timeout_ms.as_u64() < 1000 {
            return Err(ConfigError::ValidationError {
                field: "message_timeout_ms".to_string(),
                reason: "Must be at least 1 second".to_string(),
            });
        }

        if self.conversation_timeout_ms.as_u64() < 60_000 {
            return Err(ConfigError::ValidationError {
                field: "conversation_timeout_ms".to_string(),
                reason: "Must be at least 1 minute".to_string(),
            });
        }

        // Validate worker thread count
        if self.worker_thread_count.as_usize() > num_cpus::get() * 2 {
            return Err(ConfigError::ValidationError {
                field: "worker_thread_count".to_string(),
                reason: format!("Should not exceed 2x CPU cores ({})", num_cpus::get() * 2),
            });
        }

        // Validate batch size is reasonable
        if self.message_batch_size.as_usize() > self.inbound_queue_size.as_usize() / 10 {
            return Err(ConfigError::ValidationError {
                field: "message_batch_size".to_string(),
                reason: "Should not exceed 10% of inbound queue size".to_string(),
            });
        }

        // Validate retry settings
        if self.retry_delay_ms.as_u64() >= self.message_timeout_ms.as_u64() {
            return Err(ConfigError::ValidationError {
                field: "retry_delay_ms".to_string(),
                reason: "Should be less than message timeout".to_string(),
            });
        }

        // Validate circuit breaker settings
        if self.circuit_breaker_timeout_ms.as_u64()
            < self.retry_delay_ms.as_u64() * u64::from(self.max_retries.as_u8())
        {
            return Err(ConfigError::ValidationError {
                field: "circuit_breaker_timeout_ms".to_string(),
                reason: "Should be longer than total retry time".to_string(),
            });
        }

        // Validate storage path if persistence enabled
        if self.storage.enable_persistence && self.storage.storage_path.is_none() {
            return Err(ConfigError::ValidationError {
                field: "storage_path".to_string(),
                reason: "Must specify storage path when persistence is enabled".to_string(),
            });
        }

        // Validate rate limiting settings
        if self.security.enable_rate_limiting && self.security.rate_limit_messages_per_second == 0 {
            return Err(ConfigError::ValidationError {
                field: "rate_limit_messages_per_second".to_string(),
                reason: "Must be greater than 0 when rate limiting is enabled".to_string(),
            });
        }

        Ok(())
    }

    /// Saves configuration to JSON file
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if JSON serialization fails or file write fails
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads configuration from JSON file
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if file read fails or JSON deserialization fails
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        let json = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&json)?;
        config.validate()?;
        Ok(config)
    }

    /// Creates a configuration suitable for testing with minimal resources
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded configuration values are invalid (should never happen in practice)
    pub fn testing() -> Self {
        Self {
            inbound_queue_size: ChannelCapacity::try_new(10000).unwrap(),
            outbound_queue_size: ChannelCapacity::try_new(10000).unwrap(),
            message_timeout_ms: MessageTimeoutMs::try_new(5_000).unwrap(),
            message_batch_size: MessageBatchSize::try_new(5).unwrap(),
            worker_thread_count: WorkerThreadCount::try_new(1).unwrap(),

            max_retries: MaxRetries::try_new(1).unwrap(),
            retry_delay_ms: RetryDelayMs::try_new(100).unwrap(),
            retry_backoff_factor: RetryBackoffFactor::try_new(1.1).unwrap(),
            dead_letter_queue_size: DeadLetterQueueSize::try_new(10_000).unwrap(),

            circuit_breaker_threshold: CircuitBreakerThreshold::try_new(1).unwrap(),
            circuit_breaker_timeout_ms: CircuitBreakerTimeoutMs::try_new(5_000).unwrap(),

            conversation_timeout_ms: ConversationTimeoutMs::try_new(300_000).unwrap(), // 5 minutes
            max_conversation_participants: MaxConversationParticipants::try_new(3).unwrap(),

            health_check_interval_ms: HealthCheckIntervalMs::try_new(5_000).unwrap(),

            // Grouped configurations
            observability: ObservabilityConfig::testing(),
            storage: StorageConfig::testing(),
            performance: PerformanceConfig::testing(),
            security: SecurityConfig::testing(),
        }
    }
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self::development()
    }
}

/// Builder for custom router configurations
pub struct RouterConfigBuilder {
    config: RouterConfig,
}

impl RouterConfigBuilder {
    /// Creates a new builder starting with development defaults
    pub fn new() -> Self {
        Self {
            config: RouterConfig::development(),
        }
    }

    /// Sets the inbound queue size
    #[must_use]
    pub fn inbound_queue_size(mut self, size: ChannelCapacity) -> Self {
        self.config.inbound_queue_size = size;
        self
    }

    /// Sets the outbound queue size
    #[must_use]
    pub fn outbound_queue_size(mut self, size: ChannelCapacity) -> Self {
        self.config.outbound_queue_size = size;
        self
    }

    /// Sets the message timeout
    #[must_use]
    pub fn message_timeout_ms(mut self, timeout: MessageTimeoutMs) -> Self {
        self.config.message_timeout_ms = timeout;
        self
    }

    /// Sets the message batch size
    #[must_use]
    pub fn message_batch_size(mut self, size: MessageBatchSize) -> Self {
        self.config.message_batch_size = size;
        self
    }

    /// Sets the worker thread count
    #[must_use]
    pub fn worker_thread_count(mut self, count: WorkerThreadCount) -> Self {
        self.config.worker_thread_count = count;
        self
    }

    /// Sets the maximum retry attempts
    #[must_use]
    pub fn max_retries(mut self, retries: MaxRetries) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Sets the retry delay
    #[must_use]
    pub fn retry_delay_ms(mut self, delay: RetryDelayMs) -> Self {
        self.config.retry_delay_ms = delay;
        self
    }

    /// Sets the conversation timeout
    #[must_use]
    pub fn conversation_timeout_ms(mut self, timeout: ConversationTimeoutMs) -> Self {
        self.config.conversation_timeout_ms = timeout;
        self
    }

    /// Sets the trace sampling ratio
    #[must_use]
    pub fn trace_sampling_ratio(mut self, ratio: TraceSamplingRatio) -> Self {
        self.config.observability.trace_sampling_ratio = ratio;
        self
    }

    /// Enables or disables persistence
    #[must_use]
    pub fn enable_persistence(mut self, enable: bool) -> Self {
        self.config.storage.enable_persistence = enable;
        self
    }

    /// Sets the storage path
    #[must_use]
    pub fn storage_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.storage.storage_path = Some(path.into());
        self
    }

    /// Enables or disables batching
    #[must_use]
    pub fn enable_batching(mut self, enable: bool) -> Self {
        self.config.performance.enable_batching = enable;
        self
    }

    /// Enables or disables connection pooling
    #[must_use]
    pub fn enable_connection_pooling(mut self, enable: bool) -> Self {
        self.config.performance.enable_connection_pooling = enable;
        self
    }

    /// Sets the connection pool size
    #[must_use]
    pub fn connection_pool_size(mut self, size: usize) -> Self {
        self.config.performance.connection_pool_size = size;
        self
    }

    /// Enables or disables metrics
    #[must_use]
    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.config.observability.enable_metrics = enable;
        self
    }

    /// Enables or disables detailed logging
    #[must_use]
    pub fn enable_detailed_logs(mut self, enable: bool) -> Self {
        self.config.observability.enable_detailed_logs = enable;
        self
    }

    /// Enables or disables rate limiting
    #[must_use]
    pub fn enable_rate_limiting(mut self, enable: bool) -> Self {
        self.config.security.enable_rate_limiting = enable;
        self
    }

    /// Sets the rate limit
    #[must_use]
    pub fn rate_limit_messages_per_second(mut self, rate: usize) -> Self {
        self.config.security.rate_limit_messages_per_second = rate;
        self
    }

    /// Sets the entire observability configuration
    #[must_use]
    pub fn observability(mut self, observability: ObservabilityConfig) -> Self {
        self.config.observability = observability;
        self
    }

    /// Sets the entire storage configuration
    #[must_use]
    pub fn storage(mut self, storage: StorageConfig) -> Self {
        self.config.storage = storage;
        self
    }

    /// Sets the entire performance configuration
    #[must_use]
    pub fn performance(mut self, performance: PerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }

    /// Sets the entire security configuration
    #[must_use]
    pub fn security(mut self, security: SecurityConfig) -> Self {
        self.config.security = security;
        self
    }

    /// Builds and validates the configuration
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if the configuration validation fails
    pub fn build(self) -> Result<RouterConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for RouterConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_development_config_is_valid() {
        let config = RouterConfig::development();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_production_config_is_valid() {
        let config = RouterConfig::production();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_testing_config_is_valid() {
        let config = RouterConfig::testing();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = RouterConfig::builder()
            .inbound_queue_size(ChannelCapacity::try_new(5000).unwrap())
            .message_timeout_ms(MessageTimeoutMs::try_new(15000).unwrap())
            .enable_persistence(false)
            .build()
            .unwrap();

        assert_eq!(config.inbound_queue_size.as_usize(), 5000);
        assert_eq!(config.message_timeout_ms.as_u64(), 15000);
        assert!(!config.enable_persistence());
    }

    #[test]
    fn test_config_validation_errors() {
        // Test invalid queue size
        let invalid_config = RouterConfig::builder()
            .inbound_queue_size(ChannelCapacity::try_new(5).unwrap()) // Too small
            .build();
        assert!(invalid_config.is_err());

        // Test invalid timeout
        let invalid_config = RouterConfig::builder()
            .message_timeout_ms(MessageTimeoutMs::try_new(1000).unwrap()) // Minimum valid value
            .retry_delay_ms(RetryDelayMs::try_new(2000).unwrap()) // Longer than timeout - should fail validation
            .build();
        assert!(invalid_config.is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = RouterConfig::development();

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RouterConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.inbound_queue_size, deserialized.inbound_queue_size);
        assert_eq!(config.message_timeout_ms, deserialized.message_timeout_ms);
        assert_eq!(
            config.enable_persistence(),
            deserialized.enable_persistence()
        );
    }

    #[test]
    fn test_config_file_operations() {
        let config = RouterConfig::development();
        let temp_file = NamedTempFile::new().unwrap();

        // Save to file
        config.save_to_file(temp_file.path()).unwrap();

        // Load from file
        let loaded_config = RouterConfig::load_from_file(temp_file.path()).unwrap();

        assert_eq!(config.inbound_queue_size, loaded_config.inbound_queue_size);
        assert_eq!(config.message_timeout_ms, loaded_config.message_timeout_ms);
    }

    #[test]
    fn test_configurable_worker_thread_count_from_env() {
        // Test that CAXTON_WORKER_THREADS environment variable overrides default calculation
        unsafe {
            std::env::set_var("CAXTON_WORKER_THREADS", "16");
        }

        // This should use the environment variable value instead of the computed default
        let config = RouterConfig::production();
        assert_eq!(config.worker_thread_count.as_usize(), 16);

        // Clean up
        unsafe {
            std::env::remove_var("CAXTON_WORKER_THREADS");
        }
    }

    #[test]
    fn test_worker_thread_count_falls_back_to_computed_default() {
        // Ensure environment variable is not set
        unsafe {
            std::env::remove_var("CAXTON_WORKER_THREADS");
        }

        let config = RouterConfig::production();
        let expected = (num_cpus::get() * 2).clamp(2, 8);
        assert_eq!(config.worker_thread_count.as_usize(), expected);
    }

    #[test]
    fn test_invalid_worker_thread_count_env_falls_back_to_default() {
        // Test invalid values fall back to computed default
        unsafe {
            std::env::set_var("CAXTON_WORKER_THREADS", "0"); // Invalid: too low
        }
        let config = RouterConfig::production();
        let expected = (num_cpus::get() * 2).clamp(2, 8);
        assert_eq!(config.worker_thread_count.as_usize(), expected);

        unsafe {
            std::env::set_var("CAXTON_WORKER_THREADS", "256"); // Invalid: too high
        }
        let config = RouterConfig::production();
        assert_eq!(config.worker_thread_count.as_usize(), expected);

        unsafe {
            std::env::set_var("CAXTON_WORKER_THREADS", "invalid"); // Invalid: not a number
        }
        let config = RouterConfig::production();
        assert_eq!(config.worker_thread_count.as_usize(), expected);

        // Clean up
        unsafe {
            std::env::remove_var("CAXTON_WORKER_THREADS");
        }
    }
}
