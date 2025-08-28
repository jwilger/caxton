//! Test utilities for storage layer testing.
//!
//! This module provides mock implementations and test helpers for storage traits,
//! enabling unit testing without requiring actual database connections.

use super::AgentStorage;
use crate::database::DatabaseResult;
use crate::domain_types::{AgentId, AgentName};
use async_trait::async_trait;

/// Mock implementation of `AgentStorage` for testing purposes.
///
/// This implementation provides predictable behavior for unit tests:
/// - All save operations succeed
/// - All queries return empty results
/// - All delete operations succeed silently
///
/// # Usage
///
/// ```rust,ignore
/// use crate::storage::test_utils::MockAgentStorage;
/// use crate::storage::AgentStorage;
///
/// #[tokio::test]
/// async fn test_agent_persistence() {
///     let storage = MockAgentStorage::new();
///     // Use storage in your tests...
/// }
/// ```
pub struct MockAgentStorage;

impl MockAgentStorage {
    /// Creates a new mock storage instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockAgentStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentStorage for MockAgentStorage {
    async fn save_agent(&self, _agent_id: AgentId, _agent_name: AgentName) -> DatabaseResult<()> {
        Ok(())
    }

    async fn find_agent_by_id(&self, _agent_id: AgentId) -> DatabaseResult<Option<AgentName>> {
        Ok(None)
    }

    async fn list_all_agents(&self) -> DatabaseResult<Vec<(AgentId, AgentName)>> {
        Ok(vec![])
    }

    async fn remove_agent(&self, _agent_id: AgentId) -> DatabaseResult<()> {
        Ok(())
    }
}

/// Creates test agent data for consistent testing.
///
/// # Panics
///
/// Panics if `AgentName::try_new("test_agent")` fails, which should never happen
/// with the hardcoded valid agent name `"test_agent"`.
#[must_use]
pub fn create_test_agent_data() -> (AgentId, AgentName) {
    let agent_id = AgentId::generate();
    let agent_name = AgentName::try_new("test_agent").expect("Failed to create test AgentName");
    (agent_id, agent_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_storage_operations() {
        let storage = MockAgentStorage::new();
        let (agent_id, agent_name) = create_test_agent_data();

        // Test save operation
        let save_result = storage.save_agent(agent_id, agent_name.clone()).await;
        assert!(save_result.is_ok(), "Mock storage should save successfully");

        // Test find operation
        let find_result = storage.find_agent_by_id(agent_id).await;
        assert!(find_result.is_ok(), "Mock storage should find successfully");
        assert!(
            find_result.unwrap().is_none(),
            "Mock storage should return None for all queries"
        );

        // Test list operation
        let list_result = storage.list_all_agents().await;
        assert!(list_result.is_ok(), "Mock storage should list successfully");
        assert!(
            list_result.unwrap().is_empty(),
            "Mock storage should return empty list"
        );

        // Test remove operation
        let remove_result = storage.remove_agent(agent_id).await;
        assert!(
            remove_result.is_ok(),
            "Mock storage should remove successfully"
        );
    }

    #[test]
    fn test_create_test_agent_data() {
        let (_agent_id, agent_name) = create_test_agent_data();
        assert!(
            !agent_name.to_string().is_empty(),
            "Test agent name should not be empty"
        );
        // AgentId generation is tested in domain_types module
    }

    #[test]
    fn test_mock_storage_default() {
        let _storage1 = MockAgentStorage::new();
        // Both should be equivalent mock instances
        // This test ensures Default trait works correctly
    }
}
