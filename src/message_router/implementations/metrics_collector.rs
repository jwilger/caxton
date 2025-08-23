//! Metrics collection implementation for message routing

use crate::domain_types::AgentId;
use crate::message_router::domain_types::FipaMessage;
use crate::message_router::traits::{MetricsCollector, RouterError};
use std::time::Duration;

/// Placeholder metrics collector implementation
pub struct MetricsCollectorImpl;

impl MetricsCollectorImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetricsCollectorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector for MetricsCollectorImpl {
    fn record_message_routed(&self, _message: &FipaMessage, _duration: Duration) {
        // Placeholder implementation
    }

    fn record_routing_error(&self, _error: &RouterError) {
        // Placeholder implementation
    }

    fn record_delivery_metrics(&self, _success: bool, _duration: Duration) {
        // Placeholder implementation
    }

    fn record_conversation_created(&self) {
        // Placeholder implementation
    }

    fn record_agent_registered(&self, _agent_id: AgentId) {
        // Placeholder implementation
    }

    fn record_agent_deregistered(&self, _agent_id: AgentId) {
        // Placeholder implementation
    }
}
