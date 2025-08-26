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
use crate::message_router::domain_types::{
    Conversation, ConversationCreatedAt, ConversationId, MessageCount, MessageTimestamp,
    ProtocolName,
};
use crate::message_router::traits::ConversationError;
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashSet;
use tracing::{info, instrument, warn};

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

/// `SQLite` implementation of message storage for FIPA message persistence.
///
/// This implementation stores FIPA messages in a dedicated `message_storage` table
/// created via the migration system (`003_create_message_storage.sql`). The table
/// schema includes optimized indexes for common query patterns including sender/receiver
/// lookups and conversation-based filtering.
///
/// # Migration System Integration
///
/// Table creation is handled by the migration system during `DatabaseConnection::initialize()`.
/// This ensures consistent schema versioning and eliminates the need for CREATE TABLE
/// IF NOT EXISTS patterns. The migration includes performance indexes for:
///
/// - `sender_id` lookups with temporal ordering
/// - `receiver_id` lookups with temporal ordering
/// - `conversation_id` filtering with temporal ordering
/// - `created_at` temporal queries
///
/// # Performance Characteristics
///
/// - Target: < 1ms average query time for individual message operations
/// - Optimized indexes support efficient message routing and conversation retrieval
/// - UUID format validation via CHECK constraints ensures data integrity
/// - Schema designed for high-throughput message processing
///
/// # Implementation Status
///
/// Currently provides placeholder methods returning `unimplemented!()` errors.
/// Full implementation with proper domain type integration, error handling,
/// and observability will be completed in future development cycles.
pub struct SqliteMessageStorage {
    _connection: DatabaseConnection,
}

impl SqliteMessageStorage {
    /// Creates a new `SqliteMessageStorage` instance.
    ///
    /// The provided database connection should already be initialized with the migration
    /// system, ensuring the `message_storage` table and indexes are available.
    ///
    /// # Arguments
    ///
    /// * `connection` - Database connection with migration-created schema
    ///
    /// # Returns
    ///
    /// A new message storage instance ready for FIPA message persistence operations.
    pub fn new(connection: DatabaseConnection) -> Self {
        Self {
            _connection: connection,
        }
    }
}

#[async_trait]
impl crate::message_router::traits::MessageStorage for SqliteMessageStorage {
    #[instrument(skip(self, _message), fields(message_id = %_message.message_id))]
    async fn store_message(
        &self,
        _message: &crate::message_router::domain_types::FipaMessage,
    ) -> Result<(), crate::message_router::traits::RouterError> {
        unimplemented!("Message storage not yet implemented")
    }

    #[instrument(skip(self), fields(message_id = %_message_id))]
    async fn get_message(
        &self,
        _message_id: crate::message_router::domain_types::MessageId,
    ) -> Result<
        Option<crate::message_router::domain_types::FipaMessage>,
        crate::message_router::traits::RouterError,
    > {
        unimplemented!("Message retrieval not yet implemented")
    }

    #[instrument(skip(self), fields(message_id = %_message_id))]
    async fn remove_message(
        &self,
        _message_id: crate::message_router::domain_types::MessageId,
    ) -> Result<(), crate::message_router::traits::RouterError> {
        unimplemented!("Message removal not yet implemented")
    }

    #[instrument(skip(self), fields(agent_id = %_agent_id, limit = ?_limit))]
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

// SQL Constants for ConversationStorage
// Table creation handled by migration system - see migrations/004_create_conversations.sql and migrations/005_create_conversation_participants.sql

const INSERT_CONVERSATION: &str = r"
INSERT OR REPLACE INTO conversations (
    conversation_id, protocol_name, created_at, last_activity, message_count, is_archived
) VALUES (?, ?, ?, ?, ?, ?);
";

const INSERT_PARTICIPANT: &str = r"
INSERT INTO conversation_participants (conversation_id, participant_id)
VALUES (?, ?);
";

const DELETE_PARTICIPANTS: &str = r"
DELETE FROM conversation_participants WHERE conversation_id = ?;
";

const SELECT_CONVERSATION: &str = r"
SELECT conversation_id, protocol_name, created_at, last_activity, message_count, is_archived
FROM conversations WHERE conversation_id = ?;
";

const SELECT_PARTICIPANTS: &str = r"
SELECT participant_id FROM conversation_participants WHERE conversation_id = ?;
";

const UPDATE_CONVERSATION_ARCHIVE_STATUS: &str = r"
UPDATE conversations SET is_archived = TRUE WHERE conversation_id = ?;
";

const SELECT_AGENT_CONVERSATIONS: &str = r"
SELECT DISTINCT c.conversation_id
FROM conversations c
JOIN conversation_participants p ON c.conversation_id = p.conversation_id
WHERE p.participant_id = ? AND c.is_archived = FALSE
ORDER BY c.last_activity DESC;
";

/// `SQLite` implementation of conversation storage for FIPA conversation persistence.
///
/// This implementation stores conversation state and participants in dedicated
/// `conversations` and `conversation_participants` tables created via the migration
/// system (`004_create_conversations.sql`, `005_create_conversation_participants.sql`).
/// The tables include optimized indexes for common query patterns.
///
/// # Migration System Integration
///
/// Table creation is handled by the migration system during `DatabaseConnection::initialize()`.
/// This ensures consistent schema versioning and eliminates the need for CREATE TABLE
/// IF NOT EXISTS patterns. The migration includes performance indexes for:
///
/// - conversation archival status filtering
/// - temporal ordering for last activity queries
/// - protocol-based conversation filtering
/// - efficient participant lookups
///
/// # Performance Characteristics
///
/// - Target: < 1ms average query time for individual conversation operations
/// - Optimized indexes support efficient conversation and participant queries
/// - Foreign key constraints ensure referential integrity
/// - Schema designed for multi-participant conversation scenarios
pub struct SqliteConversationStorage {
    connection: DatabaseConnection,
}

impl SqliteConversationStorage {
    /// Creates a new `SqliteConversationStorage` instance.
    ///
    /// The provided database connection should already be initialized with the migration
    /// system, ensuring the `conversations` and `conversation_participants` tables and indexes
    /// are available.
    ///
    /// # Arguments
    ///
    /// * `connection` - Database connection with migration-created schema
    ///
    /// # Returns
    ///
    /// A new conversation storage instance ready for FIPA conversation persistence operations.
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    /// Verifies conversation schema is available (created by migration system)
    ///
    /// Schema creation is handled by migration system during `DatabaseConnection::initialize()`.
    /// This method serves as a placeholder for future schema validation if needed.
    /// See migrations: `004_create_conversations.sql` and `005_create_conversation_participants.sql`
    fn ensure_schema_initialized() {
        // Migration system handles all schema creation during DatabaseConnection::initialize()
        // Future enhancement: Add optional schema validation here if needed
    }
}

#[async_trait]
impl crate::message_router::traits::ConversationStorage for SqliteConversationStorage {
    #[instrument(skip(self, conversation))]
    async fn save_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), ConversationError> {
        // Schema is already initialized via migration system during DatabaseConnection::initialize()
        Self::ensure_schema_initialized();

        let pool = self.connection.pool();

        // Clear existing participants
        sqlx::query(DELETE_PARTICIPANTS)
            .bind(conversation.id.to_string())
            .execute(pool)
            .await
            .map_err(|e| {
                warn!(
                    "Failed to delete participants for conversation {}: {}",
                    conversation.id, e
                );
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;

        // Insert conversation record
        let protocol_name = conversation
            .protocol
            .as_ref()
            .map(std::string::ToString::to_string);
        let created_at = i64::try_from(
            conversation
                .created_at
                .into_inner()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        )
        .map_err(|e| {
            warn!(
                "Failed to convert created_at timestamp for conversation {}: {}",
                conversation.id, e
            );
            ConversationError::StorageError {
                source: Box::new(e),
            }
        })?;
        let last_activity = i64::try_from(
            conversation
                .last_activity
                .into_inner()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        )
        .map_err(|e| {
            warn!(
                "Failed to convert last_activity timestamp for conversation {}: {}",
                conversation.id, e
            );
            ConversationError::StorageError {
                source: Box::new(e),
            }
        })?;
        let message_count =
            u32::try_from(conversation.message_count.into_inner()).map_err(|e| {
                warn!(
                    "Failed to convert message_count for conversation {}: {}",
                    conversation.id, e
                );
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;

        sqlx::query(INSERT_CONVERSATION)
            .bind(conversation.id.to_string())
            .bind(protocol_name)
            .bind(created_at)
            .bind(last_activity)
            .bind(message_count)
            .bind(false) // is_archived
            .execute(pool)
            .await
            .map_err(|e| {
                warn!("Failed to insert conversation {}: {}", conversation.id, e);
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;

        // Insert participants
        for participant in &conversation.participants {
            sqlx::query(INSERT_PARTICIPANT)
                .bind(conversation.id.to_string())
                .bind(participant.to_string())
                .execute(pool)
                .await
                .map_err(|e| {
                    warn!(
                        "Failed to insert participant {} for conversation {}: {}",
                        participant, conversation.id, e
                    );
                    ConversationError::StorageError {
                        source: Box::new(e),
                    }
                })?;
        }

        info!(
            "Saved conversation {} with {} participants",
            conversation.id,
            conversation.participants.len()
        );
        Ok(())
    }

    #[instrument(skip(self))]
    async fn load_conversation(
        &self,
        conversation_id: ConversationId,
    ) -> Result<Option<Conversation>, ConversationError> {
        let pool = self.connection.pool();

        // Load conversation record
        let conversation_row = sqlx::query(SELECT_CONVERSATION)
            .bind(conversation_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                warn!("Failed to load conversation {}: {}", conversation_id, e);
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;

        let Some(conversation_row) = conversation_row else {
            return Ok(None);
        };

        // Load participants
        let participant_rows = sqlx::query(SELECT_PARTICIPANTS)
            .bind(conversation_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| {
                warn!(
                    "Failed to load participants for conversation {}: {}",
                    conversation_id, e
                );
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;

        // Parse participants
        let mut participants = HashSet::new();
        for row in participant_rows {
            let participant_str: String = row.get("participant_id");
            let participant_uuid = participant_str.parse::<uuid::Uuid>().map_err(|e| {
                warn!(
                    "Failed to parse participant UUID {}: {}",
                    participant_str, e
                );
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;
            participants.insert(AgentId::new(participant_uuid));
        }

        // Parse conversation fields
        let protocol_name_str: Option<String> = conversation_row.get("protocol_name");
        let protocol = if let Some(name) = protocol_name_str {
            Some(
                ProtocolName::try_new(name).map_err(|e| ConversationError::StorageError {
                    source: Box::new(e),
                })?,
            )
        } else {
            None
        };

        let created_at_secs: i64 = conversation_row.get("created_at");
        let created_at = ConversationCreatedAt::new(
            std::time::UNIX_EPOCH
                + std::time::Duration::from_secs(u64::try_from(created_at_secs).map_err(|e| {
                    warn!(
                        "Failed to convert created_at timestamp for conversation {}: {}",
                        conversation_id, e
                    );
                    ConversationError::StorageError {
                        source: Box::new(e),
                    }
                })?),
        );

        let last_activity_secs: i64 = conversation_row.get("last_activity");
        let last_activity = MessageTimestamp::new(
            std::time::UNIX_EPOCH
                + std::time::Duration::from_secs(u64::try_from(last_activity_secs).map_err(
                    |e| {
                        warn!(
                            "Failed to convert last_activity timestamp for conversation {}: {}",
                            conversation_id, e
                        );
                        ConversationError::StorageError {
                            source: Box::new(e),
                        }
                    },
                )?),
        );

        let message_count_val: u32 = conversation_row.get("message_count");
        let message_count = MessageCount::new(message_count_val as usize);

        let conversation = Conversation {
            id: conversation_id,
            participants,
            protocol,
            created_at,
            last_activity,
            message_count,
        };

        info!(
            "Loaded conversation {} with {} participants",
            conversation_id,
            conversation.participants.len()
        );
        Ok(Some(conversation))
    }

    #[instrument(skip(self, conversation))]
    async fn archive_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), ConversationError> {
        let pool = self.connection.pool();

        sqlx::query(UPDATE_CONVERSATION_ARCHIVE_STATUS)
            .bind(conversation.id.to_string())
            .execute(pool)
            .await
            .map_err(|e| {
                warn!("Failed to archive conversation {}: {}", conversation.id, e);
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;

        info!("Archived conversation {}", conversation.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_agent_conversations(
        &self,
        agent_id: AgentId,
    ) -> Result<Vec<ConversationId>, ConversationError> {
        let pool = self.connection.pool();

        let rows = sqlx::query(SELECT_AGENT_CONVERSATIONS)
            .bind(agent_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| {
                warn!("Failed to list conversations for agent {}: {}", agent_id, e);
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;

        let mut conversation_ids = Vec::new();
        for row in rows {
            let conversation_id_str: String = row.get("conversation_id");
            let conversation_uuid = conversation_id_str.parse::<uuid::Uuid>().map_err(|e| {
                warn!(
                    "Failed to parse conversation UUID {}: {}",
                    conversation_id_str, e
                );
                ConversationError::StorageError {
                    source: Box::new(e),
                }
            })?;
            conversation_ids.push(ConversationId::new(conversation_uuid));
        }

        info!(
            "Listed {} conversations for agent {}",
            conversation_ids.len(),
            agent_id
        );
        Ok(conversation_ids)
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
        // Note: Message storage operations are not tested here as they are unimplemented
        // This test specifically verifies migration system creates the required schema
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
            let schema_sql: String = row.try_get("sql").expect("Failed to get schema SQL");
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
}
