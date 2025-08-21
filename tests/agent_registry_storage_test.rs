//! Agent Registry Storage Tests - Story 004: Local State Storage
//!
//! This test suite covers agent persistence using embedded `SQLite` storage.
//! Tests verify that agents can be stored and retrieved across Caxton instance restarts.

use caxton::AgentStorage;
use caxton::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
use caxton::domain_types::{AgentId, AgentName};
use tempfile::tempdir;

/// Test that verifies agent can be stored and retrieved from `SQLite` database
#[tokio::test]
async fn test_should_persist_agent_to_database_when_storing_agent() {
    // Arrange: Set up temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("agents.db");
    let database_path = DatabasePath::new(db_path).unwrap();
    let config = DatabaseConfig::for_testing(database_path);
    let connection = DatabaseConnection::initialize(config).await.unwrap();

    // Create test agent
    let agent_id = AgentId::generate();
    let agent_name = AgentName::try_new("test-agent".to_string()).unwrap();

    // Create agent storage interface (this will fail - missing type)
    let agent_storage = AgentStorage::new(connection);

    // Act: Store agent (this should fail - unimplemented)
    let store_result = agent_storage
        .store_agent(agent_id, agent_name.clone())
        .await;
    assert!(store_result.is_ok());

    // Act: Retrieve agent (this should fail - unimplemented)
    let retrieved_agent = agent_storage.get_agent(agent_id).await;

    // Assert: Agent should be retrieved successfully
    assert!(retrieved_agent.is_ok());
    let agent = retrieved_agent.unwrap();
    assert_eq!(agent.id(), agent_id);
    assert_eq!(agent.name(), &agent_name);
}
