//! Storage layer for persisting agent registry, routing tables, and conversation state.
//!
//! This module provides trait definitions and implementations for persistent storage
//! of Caxton's runtime state, enabling recovery after server restarts.
//!
//! # Architecture
//!
//! The storage layer follows the functional core / imperative shell pattern:
//! - Pure functions handle domain validation and transformation
//! - Async trait methods handle I/O operations with the database
//! - All storage operations use existing domain types for type safety
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::storage::AgentStorage;
//! use crate::domain_types::{AgentId, AgentName};
//!
//! async fn persist_agent(storage: &dyn AgentStorage, id: AgentId, name: AgentName) {
//!     storage.save_agent(id, name).await.expect("Failed to save agent");
//! }
//! ```

use crate::database::DatabaseResult;
use crate::domain_types::{AgentId, AgentName};
use async_trait::async_trait;

/// Persistent storage interface for agent registry operations.
///
/// This trait provides CRUD operations for agent persistence, enabling the server
/// to recover agent registry state after restarts. All operations are async-compatible
/// and use domain types to ensure type safety.
///
/// # Implementation Requirements
///
/// - All operations must be atomic and consistent
/// - Agent IDs must be validated as proper UUIDs
/// - Agent names must meet domain validation requirements
/// - Storage errors should be mapped to appropriate `DatabaseResult` variants
///
/// # Performance Considerations
///
/// Implementations should target < 1ms average query time for individual operations.
/// Batch operations are preferred for bulk updates to amortize I/O overhead.
#[async_trait]
pub trait AgentStorage: Send + Sync {
    /// Saves or updates agent information in persistent storage.
    ///
    /// If an agent with the given ID already exists, this operation should update
    /// the stored information. If the agent doesn't exist, it should be created.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Unique identifier for the agent (validated UUID)
    /// * `agent_name` - Display name for the agent (validated string)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful save/update, or a `DatabaseError` if the
    /// operation fails due to storage issues, validation failures, or capacity limits.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let agent_id = AgentId::generate();
    /// let agent_name = AgentName::try_new("web-scraper").unwrap();
    /// storage.save_agent(agent_id, agent_name).await?;
    /// ```
    async fn save_agent(&self, agent_id: AgentId, agent_name: AgentName) -> DatabaseResult<()>;

    /// Retrieves agent information by ID.
    ///
    /// Returns the agent's stored information if found, or `None` if no agent
    /// exists with the given ID.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Unique identifier for the agent to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(AgentName)` if the agent exists, `None` if not found,
    /// or a `DatabaseError` if the query fails.
    async fn find_agent_by_id(&self, agent_id: AgentId) -> DatabaseResult<Option<AgentName>>;

    /// Lists all stored agents.
    ///
    /// Returns a vector of all agent ID/name pairs currently stored.
    /// For large datasets, consider implementing pagination in future versions.
    ///
    /// # Returns
    ///
    /// Returns a vector of `(AgentId, AgentName)` tuples, or a `DatabaseError`
    /// if the query fails.
    async fn list_all_agents(&self) -> DatabaseResult<Vec<(AgentId, AgentName)>>;

    /// Removes an agent from persistent storage.
    ///
    /// This operation is idempotent - removing a non-existent agent succeeds silently.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Unique identifier for the agent to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful removal (or if agent didn't exist),
    /// or a `DatabaseError` if the operation fails.
    async fn remove_agent(&self, agent_id: AgentId) -> DatabaseResult<()>;
}

// Agent storage implementations
pub mod agent_storage;

// Re-export test utilities for convenient access
#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod tests {
    use super::AgentStorage;
    use super::test_utils::MockAgentStorage;
    use crate::domain_types::{AgentId, AgentName};

    #[tokio::test]
    async fn test_should_define_agent_storage_trait_when_implementing_persistence() {
        // Test that verifies AgentStorage trait exists for agent persistence operations

        // This test intentionally tries to use an AgentStorage trait that doesn't exist yet
        // It should fail to compile because the trait is missing
        let agent_id = AgentId::generate();
        let agent_name = AgentName::try_new("test_agent").expect("Failed to create AgentName");

        // Attempt to use AgentStorage trait (now it compiles!)
        let storage: Box<dyn AgentStorage> = Box::new(MockAgentStorage::new());
        let result = storage.save_agent(agent_id, agent_name).await;

        // This assertion will never be reached due to compilation failure
        assert!(
            result.is_ok(),
            "AgentStorage should save agent successfully"
        );
    }
}
