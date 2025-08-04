//! Performance metrics collection module

use metrics::{counter, gauge, histogram};
use std::time::Duration;

/// Record agent spawn metrics
pub fn record_agent_spawn(duration: Duration) {
    counter!("caxton_agent_spawns_total");
    histogram!("caxton_agent_spawn_duration_seconds").record(duration);
}

/// Record message routing metrics
pub fn record_message_routing(duration: Duration, success: bool) {
    counter!("caxton_messages_routed_total");
    histogram!("caxton_message_routing_duration_seconds").record(duration);
    if !success {
        counter!("caxton_message_routing_errors_total");
    }
}

/// Update active agent count
pub fn update_active_agents(count: usize) {
    gauge!("caxton_active_agents").set(count as f64);
}
