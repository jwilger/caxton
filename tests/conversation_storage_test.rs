//! Conversation Storage Tests - Story 004: Local State Storage  
//!
//! This test suite covers conversation state persistence using embedded `SQLite` storage.
//! Tests verify that conversation metadata can be persisted and retrieved across
//! Caxton instance restarts to maintain message threading and context.

use caxton::ConversationStorage;
use caxton::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
use caxton::message_router::domain_types::{
    AgentId, ConversationCreatedAt, ConversationId, MessageCount, MessageTimestamp,
};
use std::collections::HashSet;
use tempfile::tempdir;

/// Test that verifies conversation state can be persisted and retrieved from `SQLite` database
#[tokio::test]
async fn test_should_persist_conversation_state_when_storing_conversation() {
    // Arrange: Set up temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("conversations.db");
    let database_path = DatabasePath::new(db_path).unwrap();
    let config = DatabaseConfig::for_testing(database_path);
    let connection = DatabaseConnection::initialize(config).await.unwrap();

    // Create test conversation data
    let conversation_id = ConversationId::generate();
    let agent1 = AgentId::generate();
    let agent2 = AgentId::generate();
    let mut participants = HashSet::new();
    participants.insert(agent1);
    participants.insert(agent2);

    let created_at = ConversationCreatedAt::now();
    let last_activity = MessageTimestamp::now();
    let message_count = MessageCount::new(5);

    // Create conversation storage interface (this will fail - missing type)
    let conversation_storage = ConversationStorage::new(connection);

    // Act: Store conversation state (this should fail - unimplemented)
    let store_result = conversation_storage
        .store_conversation_state(
            conversation_id,
            participants.clone(),
            created_at,
            last_activity,
            message_count,
        )
        .await;
    assert!(store_result.is_ok());

    // Act: Retrieve conversation state (this should fail - unimplemented)
    let retrieved_state = conversation_storage
        .get_conversation_state(conversation_id)
        .await;

    // Assert: Conversation state should be retrieved successfully
    assert!(retrieved_state.is_ok());
    let state = retrieved_state.unwrap();
    assert_eq!(state.conversation_id(), conversation_id);
    assert_eq!(state.participants(), &participants);
    assert_eq!(state.message_count(), message_count);
    assert!(state.created_at().as_system_time() <= state.last_activity().as_system_time());
}
