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
//! # Submodules
//!
//! - `agent_storage` - SQLite implementation for agent registry persistence
//! - `message_storage` - SQLite implementation for FIPA message persistence
//! - `conversation_storage` - SQLite implementation for conversation state persistence
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::storage::{AgentStorage, SqliteAgentStorage};
//! use crate::domain_types::{AgentId, AgentName};
//!
//! async fn persist_agent(storage: &dyn AgentStorage, id: AgentId, name: AgentName) {
//!     storage.save_agent(id, name).await.expect("Failed to save agent");
//! }
//! ```

use async_trait::async_trait;

use crate::{
    database::DatabaseResult,
    domain_types::{AgentId, AgentName},
};

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

// Storage implementation submodules
pub mod agent_storage;
pub mod conversation_storage;
pub mod message_storage;

// Re-export storage implementations for convenient access
pub use agent_storage::SqliteAgentStorage;
pub use conversation_storage::SqliteConversationStorage;
pub use message_storage::SqliteMessageStorage;

// Re-export test utilities for convenient access
#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod tests {
    use super::test_utils::MockAgentStorage;
    use super::*;
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

    #[tokio::test]
    async fn test_should_persist_message_to_sqlite_and_retrieve_when_storing() {
        // Test that verifies FipaMessage can be stored and retrieved with content preserved

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
        use crate::message_router::domain_types::{
            FipaMessage, MessageContent, MessageId, MessageTimestamp, Performative,
        };
        use crate::message_router::traits::MessageStorage;
        use tempfile::TempDir;

        // Create temporary database for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = DatabasePath::try_new(temp_dir.path().join("test.db"))
            .expect("Failed to create database path");
        let db_config = DatabaseConfig::new(db_path);
        let db_connection = DatabaseConnection::initialize(db_config)
            .await
            .expect("Failed to initialize database connection");

        // Create storage instance
        let message_storage = SqliteMessageStorage::new(db_connection);

        // Create a valid FipaMessage with all required fields
        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();
        let message_content = MessageContent::try_new(b"Test message content".to_vec())
            .expect("Failed to create message content");

        let message = FipaMessage {
            performative: Performative::Request,
            sender: sender_id,
            receiver: receiver_id,
            content: message_content,
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: crate::message_router::domain_types::DeliveryOptions::default(),
        };

        // Store the message - this should now work because we have full implementation
        let store_result = message_storage.store_message(&message).await;
        assert!(
            store_result.is_ok(),
            "Message should be stored successfully"
        );

        // Retrieve the message by ID - this should also work
        let retrieve_result = message_storage.get_message(message.message_id).await;
        assert!(
            retrieve_result.is_ok(),
            "Message should be retrieved successfully"
        );
        assert!(
            retrieve_result.unwrap().is_some(),
            "Stored message should be found"
        );
    }

    #[tokio::test]
    async fn test_should_delete_message_from_sqlite_when_removing() {
        // Test that verifies message deletion removes message from storage completely

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
        use crate::message_router::domain_types::{
            FipaMessage, MessageContent, MessageId, MessageTimestamp, Performative,
        };
        use crate::message_router::traits::MessageStorage;
        use tempfile::TempDir;

        // Create temporary database for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = DatabasePath::try_new(temp_dir.path().join("test.db"))
            .expect("Failed to create database path");
        let db_config = DatabaseConfig::new(db_path);
        let db_connection = DatabaseConnection::initialize(db_config)
            .await
            .expect("Failed to initialize database connection");

        // Create storage instance
        let message_storage = SqliteMessageStorage::new(db_connection);

        // Create a valid FipaMessage with all required fields
        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();
        let message_content = MessageContent::try_new(b"Message to be deleted".to_vec())
            .expect("Failed to create message content");

        let message = FipaMessage {
            performative: Performative::Inform,
            sender: sender_id,
            receiver: receiver_id,
            content: message_content,
            language: None,
            ontology: None,
            protocol: None,
            conversation_id: None,
            reply_with: None,
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: crate::message_router::domain_types::DeliveryOptions::default(),
        };

        let message_id = message.message_id;

        // Store the message first
        let store_result = message_storage.store_message(&message).await;
        assert!(
            store_result.is_ok(),
            "Message should be stored successfully"
        );

        // Delete the message - this should now work
        let delete_result = message_storage.remove_message(message_id).await;
        assert!(
            delete_result.is_ok(),
            "Message should be deleted successfully"
        );

        // Verify message is gone
        let retrieve_result = message_storage.get_message(message_id).await;
        assert!(retrieve_result.is_ok(), "Retrieval should not error");
        assert!(
            retrieve_result.unwrap().is_none(),
            "Deleted message should not be found"
        );
    }

    #[tokio::test]
    async fn test_should_create_conversation_storage_via_migration_not_create_table() {
        // Test that verifies conversations and conversation_participants tables are created via migrations with proper foreign key relationships

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
        use crate::message_router::domain_types::{
            Conversation, ConversationCreatedAt, ConversationId, MessageCount, MessageTimestamp,
            ProtocolName,
        };
        use crate::message_router::traits::ConversationStorage;
        use sqlx::Row;
        use std::collections::HashSet;
        use tempfile::TempDir;

        // Create temporary database for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = DatabasePath::try_new(temp_dir.path().join("test.db"))
            .expect("Failed to create database path");
        let db_config = DatabaseConfig::new(db_path);

        // Initialize database connection - this should run migrations automatically
        let db_connection = DatabaseConnection::initialize(db_config)
            .await
            .expect("Failed to initialize database connection");

        // Verify conversations table exists via sqlite_master query
        let conversations_table_query =
            "SELECT name FROM sqlite_master WHERE type='table' AND name='conversations'";
        let conversations_result = sqlx::query(conversations_table_query)
            .fetch_optional(db_connection.pool())
            .await
            .expect("Failed to query sqlite_master for conversations table");

        assert!(
            conversations_result.is_some(),
            "conversations table should exist after database initialization via migrations"
        );

        // Verify conversation_participants table exists via sqlite_master query
        let participants_table_query = "SELECT name FROM sqlite_master WHERE type='table' AND name='conversation_participants'";
        let participants_result = sqlx::query(participants_table_query)
            .fetch_optional(db_connection.pool())
            .await
            .expect("Failed to query sqlite_master for conversation_participants table");

        assert!(
            participants_result.is_some(),
            "conversation_participants table should exist after database initialization via migrations"
        );

        // Verify foreign key constraint exists in conversation_participants table
        let foreign_key_query =
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='conversation_participants'";
        let schema_result = sqlx::query(foreign_key_query)
            .fetch_optional(db_connection.pool())
            .await
            .expect("Failed to query table schema");

        if let Some(row) = schema_result {
            let schema_sql: String = row.get("sql");
            assert!(
                schema_sql.contains("FOREIGN KEY")
                    && schema_sql.contains("REFERENCES conversations"),
                "conversation_participants table should have proper foreign key constraint referencing conversations table"
            );
        }

        // Create conversation storage and attempt conversation operations with migration-created schema
        let conversation_storage = SqliteConversationStorage::new(db_connection);

        // Create a realistic multi-participant conversation for testing
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();
        let agent3 = AgentId::generate();
        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);
        participants.insert(agent3);

        let protocol =
            ProtocolName::try_new("contract-net").expect("Failed to create protocol name");

        let conversation = Conversation {
            id: ConversationId::generate(),
            participants,
            protocol: Some(protocol),
            created_at: ConversationCreatedAt::now(),
            last_activity: MessageTimestamp::now(),
            message_count: MessageCount::new(7),
        };

        // Attempt to save conversation - this should now work because migration files exist
        // The migration system has created the tables via 004_create_conversations.sql and 005_create_conversation_participants.sql
        let save_result = conversation_storage.save_conversation(&conversation).await;
        assert!(
            save_result.is_ok(),
            "save_conversation should succeed because migration system has created conversation schema via migrations"
        );
    }

    #[tokio::test]
    async fn test_should_create_message_storage_via_migration_not_create_table() {
        // Test that verifies message_storage table is created via migration system, not CREATE TABLE IF NOT EXISTS

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use tempfile::TempDir;

        // Create temporary database for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = DatabasePath::try_new(temp_dir.path().join("test.db"))
            .expect("Failed to create database path");
        let db_config = DatabaseConfig::new(db_path);

        // Initialize database connection - this should run migrations automatically
        let db_connection = DatabaseConnection::initialize(db_config)
            .await
            .expect("Failed to initialize database connection");

        // Verify message_storage table exists via sqlite_master query
        let table_exists_query =
            "SELECT name FROM sqlite_master WHERE type='table' AND name='message_storage'";
        let result = sqlx::query(table_exists_query)
            .fetch_optional(db_connection.pool())
            .await
            .expect("Failed to query sqlite_master");

        assert!(
            result.is_some(),
            "message_storage table should exist after database initialization via migrations"
        );

        // Create message storage instance to verify migration-created schema compatibility
        let _message_storage = SqliteMessageStorage::new(db_connection);

        // Migration test complete - table exists and SqliteMessageStorage can be constructed
        // Note: Message storage operations are now fully implemented
        // This test specifically verifies migration system creates the required schema
    }
}
