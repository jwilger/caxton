//! # Performance Monitoring and Optimization Module
//!
//! This module provides comprehensive performance monitoring, profiling, and optimization
//! capabilities for the Caxton multi-agent orchestration platform. It implements:
//!
//! - Zero-cost performance monitoring using metrics
//! - WebAssembly runtime optimization and pooling
//! - FIPA message routing performance tracking
//! - Memory allocation monitoring and optimization
//! - Agent coordination overhead analysis
//! - Observability event batching for reduced overhead

pub mod benchmarks;
pub mod memory_tracking;
pub mod message_routing;
pub mod metrics;
pub mod observability;
pub mod wasm_runtime;

use std::time::{Duration, Instant};
// Remove the direct import - macros are already imported in the individual modules that need them
use ::tracing::{debug, info, instrument};
use ahash::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance metrics collector for the entire Caxton system
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    /// Start time of the monitoring session
    start_time: Instant,
    /// Active performance counters
    counters: Arc<RwLock<HashMap<String, u64>>>,
    /// Active gauges for real-time metrics
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    /// Histogram data for latency tracking
    histograms: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor instance
    pub fn new() -> Self {
        // Metrics are registered on first use with the metrics! macros
        // No need for explicit registration in the constructor

        Self {
            start_time: Instant::now(),
            counters: Arc::new(RwLock::new(HashMap::default())),
            gauges: Arc::new(RwLock::new(HashMap::default())),
            histograms: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    /// Record a counter increment
    #[instrument(skip(self))]
    pub async fn increment_counter(&self, name: &str, value: u64) {
        // Use the metrics helper functions instead of direct macro calls
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += value;

        debug!(counter = name, value = value, "Counter incremented");
    }

    /// Set a gauge value
    #[instrument(skip(self))]
    pub async fn set_gauge(&self, name: &str, value: f64) {
        // Use the metrics helper functions instead of direct macro calls
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);

        debug!(gauge = name, value = value, "Gauge updated");
    }

    /// Record a histogram value
    #[instrument(skip(self))]
    pub async fn record_histogram(&self, name: &str, duration: Duration) {
        // Use the metrics helper functions instead of direct macro calls
        let mut histograms = self.histograms.write().await;
        histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);

        debug!(
            histogram = name,
            duration_ms = duration.as_millis(),
            "Histogram recorded"
        );
    }

    /// Get current performance summary
    #[instrument(skip(self))]
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;

        let uptime = self.start_time.elapsed();

        // Calculate histogram statistics
        let mut histogram_stats = HashMap::default();
        for (name, values) in histograms.iter() {
            if !values.is_empty() {
                let mut sorted_values = values.clone();
                sorted_values.sort();

                let count = sorted_values.len();
                let sum: Duration = sorted_values.iter().sum();
                let mean = sum / count as u32;
                let p50 = sorted_values[count / 2];
                let p95 = sorted_values[(count as f64 * 0.95) as usize];
                let p99 = sorted_values[(count as f64 * 0.99) as usize];

                histogram_stats.insert(
                    name.clone(),
                    HistogramStats {
                        count,
                        mean,
                        p50,
                        p95,
                        p99,
                    },
                );
            }
        }

        PerformanceSummary {
            uptime,
            counters: counters.clone(),
            gauges: gauges.clone(),
            histogram_stats,
        }
    }

    /// Identify performance bottlenecks based on current metrics
    #[instrument(skip(self))]
    pub async fn identify_bottlenecks(&self) -> Vec<PerformanceBottleneck> {
        let summary = self.get_performance_summary().await;
        let mut bottlenecks = Vec::new();

        // Check for high agent spawn latency
        if let Some(spawn_stats) = summary
            .histogram_stats
            .get("caxton_agent_spawn_duration_seconds")
        {
            if spawn_stats.p95.as_millis() > 100 {
                bottlenecks.push(PerformanceBottleneck {
                    area: "agent_spawn".to_string(),
                    description: format!(
                        "Agent spawn P95 latency is {}ms (target: <100ms)",
                        spawn_stats.p95.as_millis()
                    ),
                    severity: BottleneckSeverity::High,
                    recommended_actions: vec![
                        "Consider pre-warming WASM instances".to_string(),
                        "Optimize agent initialization code".to_string(),
                        "Implement agent instance pooling".to_string(),
                    ],
                });
            }
        }

        // Check for high message routing latency
        if let Some(routing_stats) = summary
            .histogram_stats
            .get("caxton_message_routing_duration_seconds")
        {
            if routing_stats.p95.as_millis() > 10 {
                bottlenecks.push(PerformanceBottleneck {
                    area: "message_routing".to_string(),
                    description: format!(
                        "Message routing P95 latency is {}ms (target: <10ms)",
                        routing_stats.p95.as_millis()
                    ),
                    severity: BottleneckSeverity::Medium,
                    recommended_actions: vec![
                        "Implement message batching".to_string(),
                        "Optimize serialization format".to_string(),
                        "Consider async message routing".to_string(),
                    ],
                });
            }
        }

        // Check memory usage
        if let Some(&memory_usage) = summary.gauges.get("caxton_memory_usage_bytes") {
            if memory_usage > 1_000_000_000.0 {
                // 1GB
                bottlenecks.push(PerformanceBottleneck {
                    area: "memory_usage".to_string(),
                    description: format!(
                        "Memory usage is {:.1}MB (consider optimization above 1GB)",
                        memory_usage / 1_000_000.0
                    ),
                    severity: BottleneckSeverity::Low,
                    recommended_actions: vec![
                        "Profile memory allocations".to_string(),
                        "Implement object pooling".to_string(),
                        "Review agent lifecycle management".to_string(),
                    ],
                });
            }
        }

        info!(
            bottleneck_count = bottlenecks.len(),
            "Performance bottlenecks identified"
        );
        bottlenecks
    }

    /// Generate optimization recommendations based on current performance data
    #[instrument(skip(self))]
    pub async fn generate_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let summary = self.get_performance_summary().await;
        let bottlenecks = self.identify_bottlenecks().await;

        let mut recommendations = Vec::new();

        // High-impact optimizations
        recommendations.push(OptimizationRecommendation {
            title: "Implement WASM Instance Pooling".to_string(),
            description: "Pre-instantiate and pool WASM modules to reduce agent spawn latency"
                .to_string(),
            impact: OptimizationImpact::High,
            effort: OptimizationEffort::Medium,
            implementation_notes: vec![
                "Use wasmtime's pooling allocator".to_string(),
                "Configure pool size based on expected agent count".to_string(),
                "Implement warm-up strategy for critical agent types".to_string(),
            ],
        });

        recommendations.push(OptimizationRecommendation {
            title: "Optimize FIPA Message Serialization".to_string(),
            description: "Use MessagePack instead of JSON for FIPA message serialization"
                .to_string(),
            impact: OptimizationImpact::Medium,
            effort: OptimizationEffort::Low,
            implementation_notes: vec![
                "Switch from serde_json to rmp-serde for message payloads".to_string(),
                "Maintain JSON support for debugging/external APIs".to_string(),
                "Implement compression for large message payloads".to_string(),
            ],
        });

        recommendations.push(OptimizationRecommendation {
            title: "Implement Observability Event Batching".to_string(),
            description: "Batch observability events to reduce I/O overhead".to_string(),
            impact: OptimizationImpact::Medium,
            effort: OptimizationEffort::Medium,
            implementation_notes: vec![
                "Buffer events for 100ms or 1000 events, whichever comes first".to_string(),
                "Use dedicated background task for event flushing".to_string(),
                "Implement backpressure handling for high event rates".to_string(),
            ],
        });

        if bottlenecks.is_empty() {
            recommendations.push(OptimizationRecommendation {
                title: "Performance Monitoring is Healthy".to_string(),
                description: "All key performance metrics are within acceptable ranges".to_string(),
                impact: OptimizationImpact::Low,
                effort: OptimizationEffort::Low,
                implementation_notes: vec![
                    "Continue monitoring for performance regressions".to_string(),
                    "Consider setting up automated performance alerts".to_string(),
                ],
            });
        }

        info!(
            recommendation_count = recommendations.len(),
            "Optimization recommendations generated"
        );
        recommendations
    }
}

/// Performance summary containing key metrics
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub uptime: Duration,
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histogram_stats: HashMap<String, HistogramStats>,
}

/// Statistics for histogram metrics
#[derive(Debug, Clone)]
pub struct HistogramStats {
    pub count: usize,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

/// Performance bottleneck identification
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    pub area: String,
    pub description: String,
    pub severity: BottleneckSeverity,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub title: String,
    pub description: String,
    pub impact: OptimizationImpact,
    pub effort: OptimizationEffort,
    pub implementation_notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum OptimizationImpact {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub enum OptimizationEffort {
    Low,
    Medium,
    High,
}

/// Global performance monitor instance
static PERFORMANCE_MONITOR: once_cell::sync::Lazy<PerformanceMonitor> =
    once_cell::sync::Lazy::new(PerformanceMonitor::new);

/// Get the global performance monitor instance
pub fn performance_monitor() -> &'static PerformanceMonitor {
    &PERFORMANCE_MONITOR
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_performance_monitor_basic_operations() {
        let monitor = PerformanceMonitor::new();

        // Test counter
        monitor.increment_counter("test_counter", 5).await;
        monitor.increment_counter("test_counter", 3).await;

        // Test gauge
        monitor.set_gauge("test_gauge", 42.5).await;

        // Test histogram
        monitor
            .record_histogram("test_histogram", Duration::from_millis(100))
            .await;
        monitor
            .record_histogram("test_histogram", Duration::from_millis(200))
            .await;

        let summary = monitor.get_performance_summary().await;

        assert_eq!(summary.counters.get("test_counter"), Some(&8));
        assert_eq!(summary.gauges.get("test_gauge"), Some(&42.5));
        assert!(summary.histogram_stats.contains_key("test_histogram"));
    }

    #[tokio::test]
    async fn test_bottleneck_identification() {
        let monitor = PerformanceMonitor::new();

        // Simulate high latency
        for _ in 0..10 {
            monitor
                .record_histogram(
                    "caxton_agent_spawn_duration_seconds",
                    Duration::from_millis(150),
                )
                .await;
        }

        let bottlenecks = monitor.identify_bottlenecks().await;
        assert!(!bottlenecks.is_empty());
        assert!(bottlenecks.iter().any(|b| b.area == "agent_spawn"));
    }
}
