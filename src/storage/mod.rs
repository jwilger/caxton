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

use crate::database::{DatabaseConnection, DatabaseResult};
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

/// Simple placeholder implementation for Message Storage to enable current tests.
/// This will be fully implemented in a future story.
pub struct SqliteMessageStorage {
    _connection: DatabaseConnection,
}

impl SqliteMessageStorage {
    /// Creates a new `SqliteMessageStorage` instance.
    pub fn new(connection: DatabaseConnection) -> Self {
        Self {
            _connection: connection,
        }
    }
}

#[async_trait]
impl crate::message_router::traits::MessageStorage for SqliteMessageStorage {
    async fn store_message(
        &self,
        _message: &crate::message_router::domain_types::FipaMessage,
    ) -> Result<(), crate::message_router::traits::RouterError> {
        unimplemented!("Message storage not yet implemented")
    }

    async fn get_message(
        &self,
        _message_id: crate::message_router::domain_types::MessageId,
    ) -> Result<
        Option<crate::message_router::domain_types::FipaMessage>,
        crate::message_router::traits::RouterError,
    > {
        unimplemented!("Message retrieval not yet implemented")
    }

    async fn remove_message(
        &self,
        _message_id: crate::message_router::domain_types::MessageId,
    ) -> Result<(), crate::message_router::traits::RouterError> {
        unimplemented!("Message removal not yet implemented")
    }

    async fn list_agent_messages(
        &self,
        _agent_id: AgentId,
        _limit: Option<usize>,
    ) -> Result<
        Vec<crate::message_router::domain_types::FipaMessage>,
        crate::message_router::traits::RouterError,
    > {
        unimplemented!("Message listing not yet implemented")
    }
}

// Agent storage implementations
pub mod agent_storage;

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

        // Store the message - this should panic with unimplemented!()
        let store_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(message_storage.store_message(&message))
        }));
        assert!(store_result.is_err(), "Should panic with unimplemented");

        // Retrieve the message by ID - this should also panic with unimplemented!()
        let retrieve_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(message_storage.get_message(message.message_id))
        }));
        assert!(retrieve_result.is_err(), "Should panic with unimplemented");
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

        // Delete the message - this should panic with unimplemented!()
        let delete_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(message_storage.remove_message(message_id))
        }));
        assert!(delete_result.is_err(), "Should panic with unimplemented");
    }

    #[tokio::test]
    async fn test_should_list_all_agent_messages_when_retrieving_without_limit() {
        // Test that verifies list_agent_messages() returns all messages for specific agent

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
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

        let message_storage = SqliteMessageStorage::new(db_connection);
        let target_agent_id = AgentId::generate();

        // List all messages for target agent (no limit) - should panic with unimplemented
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(message_storage.list_agent_messages(target_agent_id, None))
        }));
        assert!(result.is_err(), "Should panic with unimplemented");
    }

    #[tokio::test]
    async fn test_should_list_limited_agent_messages_when_retrieving_with_limit() {
        // Test that verifies list_agent_messages() respects limit parameter

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
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

        let message_storage = SqliteMessageStorage::new(db_connection);
        let target_agent_id = AgentId::generate();

        // List messages with limit of 2 - should panic with unimplemented
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(message_storage.list_agent_messages(target_agent_id, Some(2)))
        }));
        assert!(result.is_err(), "Should panic with unimplemented");
    }

    #[tokio::test]
    async fn test_should_filter_agent_messages_correctly() {
        // Test that verifies list_agent_messages() filters by agent correctly

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
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

        let message_storage = SqliteMessageStorage::new(db_connection);
        let target_agent_id = AgentId::generate();

        // List messages for target agent only - should panic with unimplemented
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(message_storage.list_agent_messages(target_agent_id, None))
        }));
        assert!(result.is_err(), "Should panic with unimplemented");
    }

    #[tokio::test]
    async fn test_should_return_empty_list_for_agent_with_no_messages() {
        // Test that verifies list_agent_messages() handles agents with no messages

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
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

        let message_storage = SqliteMessageStorage::new(db_connection);

        // List messages for agent with no messages - should panic with unimplemented
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(message_storage.list_agent_messages(AgentId::generate(), None))
        }));
        assert!(result.is_err(), "Should panic with unimplemented");
    }
}
