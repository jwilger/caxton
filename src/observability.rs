//! Observability and telemetry implementation
//!
//! Comprehensive agent monitoring with structured events, health checks,
//! and performance metrics collection.

use crate::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use dashmap::DashMap;

/// Agent event types for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEventType {
    StateChange { from: AgentState, to: AgentState },
    MessageReceived(FipaMessage),
    MessageSent(FipaMessage),
    Crashed(String),
    HealthCheck { status: HealthStatus },
    ResourceUsage { memory_mb: u64, cpu_percent: f64 },
    WasmModuleLoaded { module_size: usize },
    WasmModuleUnloaded { reason: String },
    CapabilityAdded { capability: String },
    CapabilityRemoved { capability: String },
    PropertyUpdated { key: String, value: String },
}

/// Structured agent event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub agent_id: AgentId,
    pub timestamp: std::time::SystemTime,
    pub event_type: AgentEventType,
    pub trace_id: Option<String>,
}

/// Performance metrics for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceMetrics {
    pub message_processing_time_ms: f64,
    pub messages_per_second: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for AgentPerformanceMetrics {
    fn default() -> Self {
        Self {
            message_processing_time_ms: 0.0,
            messages_per_second: 0.0,
            error_rate: 0.0,
            uptime_seconds: 0,
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Health check result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub agent_id: AgentId,
    pub healthy: bool,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: u64,
    pub details: HashMap<String, String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Agent monitoring service for tracking health and performance
#[derive(Debug, Clone)]
pub struct AgentMonitor {
    performance_metrics: Arc<DashMap<AgentId, AgentPerformanceMetrics>>,
    health_status: Arc<DashMap<AgentId, HealthStatus>>,
    event_counter: Arc<AtomicU64>,
}

impl AgentMonitor {
    pub fn new() -> Self {
        Self {
            performance_metrics: Arc::new(DashMap::new()),
            health_status: Arc::new(DashMap::new()),
            event_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record an agent event and update metrics
    pub fn record_event(&self, event: &AgentEvent) {
        self.event_counter.fetch_add(1, Ordering::Relaxed);

        match &event.event_type {
            AgentEventType::MessageReceived(_) | AgentEventType::MessageSent(_) => {
                self.update_message_metrics(&event.agent_id);
            }
            AgentEventType::HealthCheck { status } => {
                self.health_status.insert(event.agent_id.clone(), status.clone());
            }
            AgentEventType::ResourceUsage { memory_mb, cpu_percent } => {
                self.update_resource_metrics(&event.agent_id, *memory_mb, *cpu_percent);
            }
            AgentEventType::Crashed(reason) => {
                self.record_error(&event.agent_id, reason);
            }
            _ => {}
        }
    }

    /// Get performance metrics for an agent
    pub fn get_performance_metrics(&self, agent_id: &AgentId) -> Option<AgentPerformanceMetrics> {
        self.performance_metrics.get(agent_id).map(|entry| entry.clone())
    }

    /// Get health status for an agent
    pub fn get_health_status(&self, agent_id: &AgentId) -> Option<HealthStatus> {
        self.health_status.get(agent_id).map(|entry| entry.clone())
    }

    /// Perform health check on an agent
    pub async fn health_check(&self, agent_id: &AgentId) -> HealthCheckResult {
        let start_time = std::time::Instant::now();
        let timestamp = Utc::now();

        // Get current health status
        let health_status = self.get_health_status(agent_id);
        let performance_metrics = self.get_performance_metrics(agent_id);

        let mut details = HashMap::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut healthy = true;

        // Check if agent exists in our monitoring
        if health_status.is_none() {
            errors.push("Agent not found in health monitoring".to_string());
            healthy = false;
        } else {
            let status = health_status.unwrap();
            details.insert("last_health_check".to_string(), status.timestamp.to_rfc3339());
            details.insert("healthy".to_string(), status.healthy.to_string());

            if !status.healthy {
                healthy = false;
                if let Some(details_str) = &status.details {
                    errors.push(details_str.clone());
                }
            }
        }

        // Check performance metrics
        if let Some(metrics) = performance_metrics {
            details.insert("cpu_usage".to_string(), format!("{:.2}%", metrics.cpu_usage_percent));
            details.insert("memory_usage".to_string(), format!("{} bytes", metrics.memory_usage_bytes));
            details.insert("uptime".to_string(), format!("{} seconds", metrics.uptime_seconds));
            details.insert("error_rate".to_string(), format!("{:.2}%", metrics.error_rate));

            // Check for performance issues
            if metrics.cpu_usage_percent > 80.0 {
                warnings.push(format!("High CPU usage: {:.2}%", metrics.cpu_usage_percent));
            }

            if metrics.memory_usage_bytes > 100 * 1024 * 1024 { // 100MB
                warnings.push(format!("High memory usage: {} MB", metrics.memory_usage_bytes / 1024 / 1024));
            }

            if metrics.error_rate > 5.0 {
                warnings.push(format!("High error rate: {:.2}%", metrics.error_rate));
                healthy = false;
            }
        }

        let response_time = start_time.elapsed().as_millis() as u64;

        HealthCheckResult {
            agent_id: agent_id.clone(),
            healthy,
            timestamp,
            response_time_ms: response_time,
            details,
            warnings,
            errors,
        }
    }

    /// Get all monitored agents
    pub fn get_monitored_agents(&self) -> Vec<AgentId> {
        self.performance_metrics
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get system-wide monitoring statistics
    pub fn get_system_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();

        stats.insert("total_events".to_string(), self.event_counter.load(Ordering::Relaxed).to_string());
        stats.insert("monitored_agents".to_string(), self.performance_metrics.len().to_string());

        let healthy_agents = self.health_status
            .iter()
            .filter(|entry| entry.value().healthy)
            .count();
        stats.insert("healthy_agents".to_string(), healthy_agents.to_string());

        let unhealthy_agents = self.health_status.len() - healthy_agents;
        stats.insert("unhealthy_agents".to_string(), unhealthy_agents.to_string());

        // Calculate average performance metrics
        let total_cpu: f64 = self.performance_metrics
            .iter()
            .map(|entry| entry.value().cpu_usage_percent)
            .sum();
        let avg_cpu = if self.performance_metrics.len() > 0 {
            total_cpu / self.performance_metrics.len() as f64
        } else {
            0.0
        };
        stats.insert("average_cpu_usage".to_string(), format!("{:.2}%", avg_cpu));

        let total_memory: u64 = self.performance_metrics
            .iter()
            .map(|entry| entry.value().memory_usage_bytes)
            .sum();
        stats.insert("total_memory_usage".to_string(), format!("{} MB", total_memory / 1024 / 1024));

        stats
    }

    /// Clean up metrics for removed agents
    pub fn cleanup_agent(&self, agent_id: &AgentId) {
        self.performance_metrics.remove(agent_id);
        self.health_status.remove(agent_id);
    }

    /// Update message processing metrics
    fn update_message_metrics(&self, agent_id: &AgentId) {
        let mut metrics = self.performance_metrics
            .entry(agent_id.clone())
            .or_insert_with(AgentPerformanceMetrics::default);

        // Simple metric update (in real implementation, this would be more sophisticated)
        metrics.messages_per_second += 0.1;
        metrics.last_updated = Utc::now();
    }

    /// Update resource usage metrics
    fn update_resource_metrics(&self, agent_id: &AgentId, memory_mb: u64, cpu_percent: f64) {
        let mut metrics = self.performance_metrics
            .entry(agent_id.clone())
            .or_insert_with(AgentPerformanceMetrics::default);

        metrics.memory_usage_bytes = memory_mb * 1024 * 1024;
        metrics.cpu_usage_percent = cpu_percent;
        metrics.last_updated = Utc::now();
    }

    /// Record an error for an agent
    fn record_error(&self, agent_id: &AgentId, _reason: &str) {
        let mut metrics = self.performance_metrics
            .entry(agent_id.clone())
            .or_insert_with(AgentPerformanceMetrics::default);

        // Simple error rate calculation
        metrics.error_rate += 1.0;
        metrics.last_updated = Utc::now();
    }
}
