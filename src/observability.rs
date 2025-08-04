//! Observability and telemetry implementation

use crate::*;

/// Agent event types for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEventType {
    StateChange { from: AgentState, to: AgentState },
    MessageReceived(FipaMessage),
    MessageSent(FipaMessage),
    Crashed(String),
}

/// Structured agent event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub agent_id: AgentId,
    pub timestamp: std::time::SystemTime,
    pub event_type: AgentEventType,
    pub trace_id: Option<String>,
}