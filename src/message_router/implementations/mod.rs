//! Implementation modules for message router components
//!
//! This module organizes trait implementations into separate files
//! for better maintainability and modularity.

pub mod agent_registry;
pub mod conversation_manager;
pub mod delivery_engine;
pub mod failure_handler;
pub mod message_router_impl;
pub mod metrics_collector;

// Re-export implementations for easy access
pub use agent_registry::AgentRegistryImpl;
pub use conversation_manager::ConversationManagerImpl;
pub use delivery_engine::DeliveryEngineImpl;
pub use failure_handler::FailureHandlerImpl;
pub use metrics_collector::MetricsCollectorImpl;
