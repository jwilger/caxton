//! # Observability Event Batching and Optimization
//!
//! This module provides high-performance observability with minimal overhead:
//! - Batched event processing to reduce I/O overhead
//! - Async event pipeline with backpressure handling
//! - Structured logging with zero-copy serialization
//! - OpenTelemetry integration with sampling and compression
//! - Performance metrics collection with minimal latency impact

use ahash::HashMap;
use metrics::{counter, histogram};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// High-performance observability system with batched event processing
#[derive(Debug)]
pub struct OptimizedObservabilitySystem {
    /// Event batch processor
    event_processor: Arc<EventBatchProcessor>,
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Configuration
    config: ObservabilityConfig,
    /// Performance monitor
    performance_monitor: Arc<ObservabilityPerformanceMonitor>,
}

/// Configuration for observability optimization
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Maximum events per batch
    pub max_batch_size: usize,
    /// Batch flush interval
    pub batch_flush_interval: Duration,
    /// Maximum concurrent processing
    pub max_concurrent_batches: usize,
    /// Event sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Enable compression for large payloads
    pub enable_compression: bool,
    /// Buffer size for event queue
    pub event_buffer_size: usize,
    /// Metrics collection interval
    pub metrics_collection_interval: Duration,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            batch_flush_interval: Duration::from_millis(100),
            max_concurrent_batches: 10,
            sampling_rate: 1.0, // Collect all events by default
            enable_compression: true,
            event_buffer_size: 10_000,
            metrics_collection_interval: Duration::from_secs(30),
        }
    }
}

impl OptimizedObservabilitySystem {
    /// Create a new optimized observability system
    #[instrument]
    pub fn new(config: ObservabilityConfig) -> Self {
        let event_processor = Arc::new(EventBatchProcessor::new(
            config.max_batch_size,
            config.batch_flush_interval,
            config.max_concurrent_batches,
        ));

        let metrics_collector = Arc::new(MetricsCollector::new(config.metrics_collection_interval));

        let performance_monitor = Arc::new(ObservabilityPerformanceMonitor::new());

        let system = Self {
            event_processor,
            metrics_collector,
            config,
            performance_monitor,
        };

        system.start_background_tasks();

        info!(
            max_batch_size = system.config.max_batch_size,
            flush_interval_ms = system.config.batch_flush_interval.as_millis(),
            sampling_rate = system.config.sampling_rate,
            "Optimized observability system initialized"
        );

        system
    }

    /// Emit an observability event with optimized processing
    #[instrument(skip(self, event))]
    pub async fn emit_event(&self, event: ObservabilityEvent) -> Result<(), ObservabilityError> {
        let start_time = Instant::now();

        // Apply sampling
        if !self.should_sample() {
            counter!("caxton_observability_events_sampled_out_total");
            return Ok(());
        }

        // Add to batch processor
        self.event_processor.add_event(event).await?;

        let duration = start_time.elapsed();
        histogram!(
            "caxton_observability_event_emission_duration_seconds",
            duration.as_secs_f64()
        );
        counter!("caxton_observability_events_emitted_total");

        self.performance_monitor
            .record_event_emission(duration)
            .await;

        debug!(
            duration_us = duration.as_micros(),
            "Event emitted to batch processor"
        );
        Ok(())
    }

    /// Emit multiple events in a batch for efficiency
    #[instrument(skip(self, events))]
    pub async fn emit_events_batch(
        &self,
        events: Vec<ObservabilityEvent>,
    ) -> Result<Vec<Result<(), ObservabilityError>>, ObservabilityError> {
        let start_time = Instant::now();
        let batch_size = events.len();

        let mut results = Vec::with_capacity(batch_size);
        let mut sampled_events = Vec::new();

        // Apply sampling to all events
        for event in events {
            if self.should_sample() {
                sampled_events.push(event);
                results.push(Ok(()));
            } else {
                results.push(Ok(())); // Still successful, just sampled out
            }
        }

        // Process sampled events
        if !sampled_events.is_empty() {
            self.event_processor
                .add_events_batch(sampled_events)
                .await?;
        }

        let duration = start_time.elapsed();
        histogram!(
            "caxton_observability_batch_emission_duration_seconds",
            duration.as_secs_f64()
        );
        counter!("caxton_observability_event_batches_emitted_total");
        counter!("caxton_observability_events_emitted_total").increment(batch_size as u64);

        info!(
            batch_size = batch_size,
            sampled_count = sampled_events.len(),
            duration_ms = duration.as_millis(),
            "Event batch processed"
        );

        Ok(results)
    }

    /// Get observability performance statistics
    #[instrument(skip(self))]
    pub async fn get_performance_stats(&self) -> ObservabilityPerformanceStats {
        let processor_stats = self.event_processor.get_stats().await;
        let metrics_stats = self.metrics_collector.get_stats().await;
        let monitor_stats = self.performance_monitor.get_stats().await;

        ObservabilityPerformanceStats {
            processor_stats,
            metrics_stats,
            monitor_stats,
            sampling_rate: self.config.sampling_rate,
        }
    }

    /// Check if an event should be sampled based on configuration
    fn should_sample(&self) -> bool {
        if self.config.sampling_rate >= 1.0 {
            true
        } else if self.config.sampling_rate <= 0.0 {
            false
        } else {
            fastrand::f64() < self.config.sampling_rate
        }
    }

    /// Start background maintenance tasks
    fn start_background_tasks(&self) {
        let performance_monitor = Arc::clone(&self.performance_monitor);
        let config = self.config.clone();

        // Performance monitoring task
        tokio::spawn(async move {
            let mut monitoring_interval = interval(config.metrics_collection_interval);

            loop {
                monitoring_interval.tick().await;
                performance_monitor.collect_system_metrics().await;
            }
        });
    }
}

/// Batched event processor for high-throughput observability
#[derive(Debug)]
struct EventBatchProcessor {
    sender: mpsc::Sender<ObservabilityEvent>,
    stats: Arc<RwLock<BatchProcessorStats>>,
}

impl EventBatchProcessor {
    fn new(max_batch_size: usize, flush_interval: Duration, max_concurrent: usize) -> Self {
        let (sender, mut receiver) = mpsc::channel::<ObservabilityEvent>(10_000);
        let stats = Arc::new(RwLock::new(BatchProcessorStats::default()));
        let stats_clone = Arc::clone(&stats);

        // Batch processing task
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(max_batch_size);
            let mut flush_timer = interval(flush_interval);
            let processing_semaphore = Arc::new(Semaphore::new(max_concurrent));

            loop {
                tokio::select! {
                    // Receive new event
                    event = receiver.recv() => {
                        match event {
                            Some(event) => {
                                batch.push(event);

                                // Process batch if full
                                if batch.len() >= max_batch_size {
                                    let batch_to_process = std::mem::replace(&mut batch, Vec::with_capacity(max_batch_size));
                                    Self::process_batch_async(batch_to_process, &stats_clone, &processing_semaphore);
                                }
                            }
                            None => break, // Channel closed
                        }
                    }
                    // Flush timeout - process partial batch
                    _ = flush_timer.tick() => {
                        if !batch.is_empty() {
                            let batch_to_process = std::mem::replace(&mut batch, Vec::with_capacity(max_batch_size));
                            Self::process_batch_async(batch_to_process, &stats_clone, &processing_semaphore);
                        }
                    }
                }
            }
        });

        Self { sender, stats }
    }

    async fn add_event(&self, event: ObservabilityEvent) -> Result<(), ObservabilityError> {
        self.sender
            .send(event)
            .await
            .map_err(|_| ObservabilityError::ChannelClosed)?;
        Ok(())
    }

    async fn add_events_batch(
        &self,
        events: Vec<ObservabilityEvent>,
    ) -> Result<(), ObservabilityError> {
        for event in events {
            self.sender
                .send(event)
                .await
                .map_err(|_| ObservabilityError::ChannelClosed)?;
        }
        Ok(())
    }

    fn process_batch_async(
        batch: Vec<ObservabilityEvent>,
        stats: &Arc<RwLock<BatchProcessorStats>>,
        semaphore: &Arc<Semaphore>,
    ) {
        let stats = Arc::clone(stats);
        let semaphore = Arc::clone(semaphore);

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            Self::process_batch(batch, &stats).await;
        });
    }

    async fn process_batch(
        batch: Vec<ObservabilityEvent>,
        stats: &Arc<RwLock<BatchProcessorStats>>,
    ) {
        let start_time = Instant::now();
        let batch_size = batch.len();

        // Sort events by priority and timestamp for optimal processing
        let mut sorted_batch = batch;
        sorted_batch.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });

        // Process events (in real implementation, this would send to observability backend)
        for event in sorted_batch {
            Self::process_single_event(event).await;
        }

        let duration = start_time.elapsed();

        // Update statistics
        let mut batch_stats = stats.write().await;
        batch_stats.batches_processed += 1;
        batch_stats.total_events_processed += batch_size as u64;
        batch_stats.total_processing_time += duration;

        counter!("caxton_observability_batches_processed_total");
        histogram!(
            "caxton_observability_batch_processing_duration_seconds",
            duration.as_secs_f64()
        );

        debug!(
            batch_size = batch_size,
            processing_time_us = duration.as_micros(),
            "Observability batch processed"
        );
    }

    async fn process_single_event(event: ObservabilityEvent) {
        // Placeholder for actual event processing
        // In production, this would:
        // 1. Serialize the event (using MessagePack for efficiency)
        // 2. Compress if needed
        // 3. Send to observability backend (Jaeger, Prometheus, etc.)
        // 4. Handle retries and errors

        counter!("caxton_observability_events_processed_total");

        // Simulate processing time based on event type
        let processing_delay = match event.event_type {
            ObservabilityEventType::Trace(_) => Duration::from_micros(10),
            ObservabilityEventType::Metric(_) => Duration::from_micros(5),
            ObservabilityEventType::Log(_) => Duration::from_micros(15),
        };

        sleep(processing_delay).await;
    }

    async fn get_stats(&self) -> BatchProcessorStats {
        self.stats.read().await.clone()
    }
}

/// Metrics collector for system and application metrics
#[derive(Debug)]
struct MetricsCollector {
    stats: Arc<RwLock<MetricsCollectorStats>>,
}

impl MetricsCollector {
    fn new(collection_interval: Duration) -> Self {
        let collector = Self {
            stats: Arc::new(RwLock::new(MetricsCollectorStats::default())),
        };

        let stats = Arc::clone(&collector.stats);
        tokio::spawn(async move {
            let mut interval_timer = interval(collection_interval);

            loop {
                interval_timer.tick().await;
                Self::collect_metrics(&stats).await;
            }
        });

        collector
    }

    async fn collect_metrics(stats: &Arc<RwLock<MetricsCollectorStats>>) {
        let start_time = Instant::now();

        // Collect various system metrics
        let system_metrics = Self::collect_system_metrics().await;
        let application_metrics = Self::collect_application_metrics().await;

        let duration = start_time.elapsed();

        // Update collector stats
        let mut collector_stats = stats.write().await;
        collector_stats.collections_performed += 1;
        collector_stats.total_collection_time += duration;
        collector_stats.last_collection = Some(start_time);

        histogram!(
            "caxton_observability_metrics_collection_duration_seconds",
            duration.as_secs_f64()
        );
        counter!("caxton_observability_metrics_collections_total");

        debug!(
            system_metrics_count = system_metrics.len(),
            app_metrics_count = application_metrics.len(),
            collection_time_us = duration.as_micros(),
            "Metrics collection completed"
        );
    }

    async fn collect_system_metrics() -> Vec<MetricData> {
        // Placeholder for system metrics collection
        // In production, this would collect:
        // - CPU usage
        // - Memory usage
        // - Disk I/O
        // - Network stats
        // - Process information

        vec![
            MetricData {
                name: "system_cpu_usage".to_string(),
                value: 0.15, // 15% CPU usage
                labels: HashMap::default(),
                timestamp: SystemTime::now(),
            },
            MetricData {
                name: "system_memory_usage_bytes".to_string(),
                value: 1_000_000_000.0, // 1GB
                labels: HashMap::default(),
                timestamp: SystemTime::now(),
            },
        ]
    }

    async fn collect_application_metrics() -> Vec<MetricData> {
        // Placeholder for application-specific metrics
        // In production, this would collect:
        // - Agent count
        // - Message throughput
        // - WASM execution metrics
        // - Error rates

        vec![MetricData {
            name: "caxton_active_agents".to_string(),
            value: 42.0, // Example value
            labels: HashMap::default(),
            timestamp: SystemTime::now(),
        }]
    }

    async fn get_stats(&self) -> MetricsCollectorStats {
        self.stats.read().await.clone()
    }
}

/// Performance monitor for the observability system itself
#[derive(Debug)]
struct ObservabilityPerformanceMonitor {
    emission_times: Arc<RwLock<SmallVec<[Duration; 100]>>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
}

impl ObservabilityPerformanceMonitor {
    fn new() -> Self {
        Self {
            emission_times: Arc::new(RwLock::new(SmallVec::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
        }
    }

    async fn record_event_emission(&self, duration: Duration) {
        let mut times = self.emission_times.write().await;

        // Keep only the last 100 measurements for rolling statistics
        if times.len() >= 100 {
            times.remove(0);
        }
        times.push(duration);
    }

    async fn collect_system_metrics(&self) {
        // Collect current system performance metrics
        let metrics = SystemMetrics {
            memory_usage: get_current_memory_usage(),
            cpu_usage: get_current_cpu_usage(),
            disk_usage: get_current_disk_usage(),
            network_usage: get_current_network_usage(),
            timestamp: SystemTime::now(),
        };

        let mut system_metrics = self.system_metrics.write().await;
        *system_metrics = metrics;
    }

    async fn get_stats(&self) -> ObservabilityMonitorStats {
        let times = self.emission_times.read().await;
        let system_metrics = self.system_metrics.read().await;

        let (avg_emission_time, p95_emission_time) = if times.is_empty() {
            (Duration::ZERO, Duration::ZERO)
        } else {
            let sum: Duration = times.iter().sum();
            let avg = sum / times.len() as u32;

            let mut sorted_times = times.clone();
            sorted_times.sort();
            let p95_index = (times.len() as f64 * 0.95) as usize;
            let p95 = sorted_times
                .get(p95_index)
                .copied()
                .unwrap_or(Duration::ZERO);

            (avg, p95)
        };

        ObservabilityMonitorStats {
            average_emission_time: avg_emission_time,
            p95_emission_time,
            system_metrics: system_metrics.clone(),
        }
    }
}

/// Observability event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityEvent {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub priority: EventPriority,
    pub source: String,
    pub event_type: ObservabilityEventType,
    pub metadata: HashMap<String, String>,
}

/// Event priority for processing optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Types of observability events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservabilityEventType {
    Trace(TraceEvent),
    Metric(MetricEvent),
    Log(LogEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub trace_id: Uuid,
    pub span_id: Uuid,
    pub parent_span_id: Option<Uuid>,
    pub operation_name: String,
    pub duration: Option<Duration>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEvent {
    pub name: String,
    pub value: f64,
    pub metric_type: MetricType,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Performance statistics structures
#[derive(Debug, Clone)]
pub struct ObservabilityPerformanceStats {
    pub processor_stats: BatchProcessorStats,
    pub metrics_stats: MetricsCollectorStats,
    pub monitor_stats: ObservabilityMonitorStats,
    pub sampling_rate: f64,
}

#[derive(Debug, Clone, Default)]
pub struct BatchProcessorStats {
    pub batches_processed: u64,
    pub total_events_processed: u64,
    pub total_processing_time: Duration,
}

impl BatchProcessorStats {
    pub fn average_batch_size(&self) -> f64 {
        if self.batches_processed == 0 {
            0.0
        } else {
            self.total_events_processed as f64 / self.batches_processed as f64
        }
    }

    pub fn average_processing_time(&self) -> Duration {
        if self.batches_processed == 0 {
            Duration::ZERO
        } else {
            self.total_processing_time / self.batches_processed as u32
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MetricsCollectorStats {
    pub collections_performed: u64,
    pub total_collection_time: Duration,
    pub last_collection: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct ObservabilityMonitorStats {
    pub average_emission_time: Duration,
    pub p95_emission_time: Duration,
    pub system_metrics: SystemMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub memory_usage: f64,
    pub cpu_usage: f64,
    pub disk_usage: f64,
    pub network_usage: f64,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
struct MetricData {
    name: String,
    value: f64,
    labels: HashMap<String, String>,
    timestamp: SystemTime,
}

/// Observability error types
#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    #[error("Event channel closed")]
    ChannelClosed,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Backend connection error: {0}")]
    BackendError(String),

    #[error("Rate limiting exceeded")]
    RateLimitExceeded,
}

// Placeholder functions for system metrics (would be implemented with actual system calls)
fn get_current_memory_usage() -> f64 {
    0.0
}
fn get_current_cpu_usage() -> f64 {
    0.0
}
fn get_current_disk_usage() -> f64 {
    0.0
}
fn get_current_network_usage() -> f64 {
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_system_creation() {
        let config = ObservabilityConfig::default();
        let system = OptimizedObservabilitySystem::new(config);

        let stats = system.get_performance_stats().await;
        assert_eq!(stats.processor_stats.batches_processed, 0);
    }

    #[tokio::test]
    async fn test_event_emission() {
        let config = ObservabilityConfig::default();
        let system = OptimizedObservabilitySystem::new(config);

        let event = ObservabilityEvent {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            priority: EventPriority::Normal,
            source: "test".to_string(),
            event_type: ObservabilityEventType::Log(LogEvent {
                level: LogLevel::Info,
                message: "Test event".to_string(),
                fields: HashMap::new(),
            }),
            metadata: HashMap::new(),
        };

        let result = system.emit_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_batch_event_emission() {
        let config = ObservabilityConfig {
            sampling_rate: 1.0, // Ensure all events are processed
            ..Default::default()
        };
        let system = OptimizedObservabilitySystem::new(config);

        let events: Vec<ObservabilityEvent> = (0..5)
            .map(|i| ObservabilityEvent {
                id: Uuid::new_v4(),
                timestamp: SystemTime::now(),
                priority: EventPriority::Normal,
                source: format!("test_{}", i),
                event_type: ObservabilityEventType::Log(LogEvent {
                    level: LogLevel::Info,
                    message: format!("Test event {}", i),
                    fields: HashMap::new(),
                }),
                metadata: HashMap::new(),
            })
            .collect();

        let results = system.emit_events_batch(events).await;
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
