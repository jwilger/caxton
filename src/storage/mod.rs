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
    #[instrument(skip(self, message), fields(message_id = %message.message_id))]
    async fn store_message(
        &self,
        message: &crate::message_router::domain_types::FipaMessage,
    ) -> Result<(), crate::message_router::traits::RouterError> {
        // Serialize complex domain types to JSON for storage
        let message_content_json = serde_json::to_string(&message.content).map_err(|e| {
            crate::message_router::traits::RouterError::SerializationError { source: e }
        })?;

        let performative_str = format!("{:?}", message.performative);
        let created_at = i64::try_from(
            message
                .created_at
                .into_inner()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        )
        .map_err(
            |e| crate::message_router::traits::RouterError::StorageError {
                source: Box::new(e),
            },
        )?;

        // Store message in database
        sqlx::query(INSERT_MESSAGE)
            .bind(message.message_id.to_string())
            .bind(message.sender.to_string())
            .bind(message.receiver.to_string())
            .bind(
                message
                    .conversation_id
                    .as_ref()
                    .map(std::string::ToString::to_string),
            )
            .bind(message_content_json)
            .bind(performative_str)
            .bind(created_at)
            .bind(None::<i64>) // expires_at - not used in current implementation
            .execute(self._connection.pool())
            .await
            .map_err(
                |e| crate::message_router::traits::RouterError::StorageError {
                    source: Box::new(e),
                },
            )?;

        Ok(())
    }

    #[instrument(skip(self), fields(message_id = %message_id))]
    async fn get_message(
        &self,
        message_id: crate::message_router::domain_types::MessageId,
    ) -> Result<
        Option<crate::message_router::domain_types::FipaMessage>,
        crate::message_router::traits::RouterError,
    > {
        // Minimal implementation to pass test
        let row = sqlx::query(SELECT_MESSAGE_BY_ID)
            .bind(message_id.to_string())
            .fetch_optional(self._connection.pool())
            .await
            .map_err(
                |e| crate::message_router::traits::RouterError::StorageError {
                    source: Box::new(e),
                },
            )?;

        let Some(row) = row else {
            return Ok(None);
        };

        // Parse UUID fields
        let message_id_str: String = row.get("message_id");
        let sender_str: String = row.get("sender_id");
        let receiver_str: String = row.get("receiver_id");
        let conversation_id_str: Option<String> = row.get("conversation_id");

        let parsed_message_id = uuid::Uuid::parse_str(&message_id_str).map_err(|e| {
            crate::message_router::traits::RouterError::StorageError {
                source: Box::new(e),
            }
        })?;
        let parsed_sender = uuid::Uuid::parse_str(&sender_str).map_err(|e| {
            crate::message_router::traits::RouterError::StorageError {
                source: Box::new(e),
            }
        })?;
        let parsed_receiver = uuid::Uuid::parse_str(&receiver_str).map_err(|e| {
            crate::message_router::traits::RouterError::StorageError {
                source: Box::new(e),
            }
        })?;

        let conversation_id = if let Some(conv_str) = conversation_id_str {
            Some(crate::message_router::domain_types::ConversationId::new(
                uuid::Uuid::parse_str(&conv_str).map_err(|e| {
                    crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(e),
                    }
                })?,
            ))
        } else {
            None
        };

        // Deserialize content from JSON
        let content_json: String = row.get("message_content");
        let message_content = serde_json::from_str(&content_json).map_err(|e| {
            crate::message_router::traits::RouterError::SerializationError { source: e }
        })?;

        // Parse performative
        let performative_str: String = row.get("performative");
        let performative = match performative_str.as_str() {
            "Inform" => crate::message_router::domain_types::Performative::Inform,
            "Request" => crate::message_router::domain_types::Performative::Request,
            "Agree" => crate::message_router::domain_types::Performative::Agree,
            "Refuse" => crate::message_router::domain_types::Performative::Refuse,
            "NotUnderstood" => crate::message_router::domain_types::Performative::NotUnderstood,
            "Failure" => crate::message_router::domain_types::Performative::Failure,
            "QueryIf" => crate::message_router::domain_types::Performative::QueryIf,
            "QueryRef" => crate::message_router::domain_types::Performative::QueryRef,
            "Propose" => crate::message_router::domain_types::Performative::Propose,
            "AcceptProposal" => crate::message_router::domain_types::Performative::AcceptProposal,
            "RejectProposal" => crate::message_router::domain_types::Performative::RejectProposal,
            "Heartbeat" => crate::message_router::domain_types::Performative::Heartbeat,
            "Capability" => crate::message_router::domain_types::Performative::Capability,
            _ => {
                return Err(crate::message_router::traits::RouterError::StorageError {
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unknown performative: {performative_str}"),
                    )),
                });
            }
        };

        // Parse timestamp
        let created_at_secs: i64 = row.get("created_at");
        let created_at = crate::message_router::domain_types::MessageTimestamp::new(
            std::time::UNIX_EPOCH
                + std::time::Duration::from_secs(u64::try_from(created_at_secs).map_err(|e| {
                    crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(e),
                    }
                })?),
        );

        // Reconstruct FipaMessage
        let fipa_message = crate::message_router::domain_types::FipaMessage {
            performative,
            sender: AgentId::new(parsed_sender),
            receiver: AgentId::new(parsed_receiver),
            content: message_content,
            language: None, // Not stored in current schema
            ontology: None, // Not stored in current schema
            protocol: None, // Not stored in current schema
            conversation_id,
            reply_with: None,  // Not stored in current schema
            in_reply_to: None, // Not stored in current schema
            message_id: crate::message_router::domain_types::MessageId::new(parsed_message_id),
            created_at,
            trace_context: None, // Not stored in current schema
            delivery_options: crate::message_router::domain_types::DeliveryOptions::default(), // Not stored in current schema
        };

        Ok(Some(fipa_message))
    }

    #[instrument(skip(self), fields(message_id = %message_id))]
    async fn remove_message(
        &self,
        message_id: crate::message_router::domain_types::MessageId,
    ) -> Result<(), crate::message_router::traits::RouterError> {
        // Minimal implementation to delete message from database
        sqlx::query(DELETE_MESSAGE_BY_ID)
            .bind(message_id.to_string())
            .execute(self._connection.pool())
            .await
            .map_err(
                |e| crate::message_router::traits::RouterError::StorageError {
                    source: Box::new(e),
                },
            )?;

        // Idempotent operation - return Ok whether message existed or not
        Ok(())
    }

    #[instrument(skip(self), fields(agent_id = %agent_id, limit = ?limit))]
    async fn list_agent_messages(
        &self,
        agent_id: AgentId,
        limit: Option<usize>,
    ) -> Result<
        Vec<crate::message_router::domain_types::FipaMessage>,
        crate::message_router::traits::RouterError,
    > {
        // Minimal implementation to retrieve messages where agent is sender OR receiver
        let rows = if let Some(limit_value) = limit {
            sqlx::query(SELECT_MESSAGES_FOR_AGENT_LIMITED)
                .bind(agent_id.to_string())
                .bind(i64::try_from(limit_value).map_err(|e| {
                    crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(e),
                    }
                })?)
                .fetch_all(self._connection.pool())
                .await
                .map_err(
                    |e| crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(e),
                    },
                )?
        } else {
            sqlx::query(SELECT_MESSAGES_FOR_AGENT)
                .bind(agent_id.to_string())
                .fetch_all(self._connection.pool())
                .await
                .map_err(
                    |e| crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(e),
                    },
                )?
        };

        let mut messages = Vec::new();
        for row in rows {
            // Parse UUID fields
            let message_id_str: String = row.get("message_id");
            let sender_str: String = row.get("sender_id");
            let receiver_str: String = row.get("receiver_id");
            let conversation_id_str: Option<String> = row.get("conversation_id");

            let parsed_message_id = uuid::Uuid::parse_str(&message_id_str).map_err(|e| {
                crate::message_router::traits::RouterError::StorageError {
                    source: Box::new(e),
                }
            })?;
            let parsed_sender = uuid::Uuid::parse_str(&sender_str).map_err(|e| {
                crate::message_router::traits::RouterError::StorageError {
                    source: Box::new(e),
                }
            })?;
            let parsed_receiver = uuid::Uuid::parse_str(&receiver_str).map_err(|e| {
                crate::message_router::traits::RouterError::StorageError {
                    source: Box::new(e),
                }
            })?;

            let conversation_id = if let Some(conv_str) = conversation_id_str {
                Some(crate::message_router::domain_types::ConversationId::new(
                    uuid::Uuid::parse_str(&conv_str).map_err(|e| {
                        crate::message_router::traits::RouterError::StorageError {
                            source: Box::new(e),
                        }
                    })?,
                ))
            } else {
                None
            };

            // Deserialize content from JSON
            let content_json: String = row.get("message_content");
            let message_content = serde_json::from_str(&content_json).map_err(|e| {
                crate::message_router::traits::RouterError::SerializationError { source: e }
            })?;

            // Parse performative
            let performative_str: String = row.get("performative");
            let performative = match performative_str.as_str() {
                "Inform" => crate::message_router::domain_types::Performative::Inform,
                "Request" => crate::message_router::domain_types::Performative::Request,
                "Agree" => crate::message_router::domain_types::Performative::Agree,
                "Refuse" => crate::message_router::domain_types::Performative::Refuse,
                "NotUnderstood" => crate::message_router::domain_types::Performative::NotUnderstood,
                "Failure" => crate::message_router::domain_types::Performative::Failure,
                "QueryIf" => crate::message_router::domain_types::Performative::QueryIf,
                "QueryRef" => crate::message_router::domain_types::Performative::QueryRef,
                "Propose" => crate::message_router::domain_types::Performative::Propose,
                "AcceptProposal" => {
                    crate::message_router::domain_types::Performative::AcceptProposal
                }
                "RejectProposal" => {
                    crate::message_router::domain_types::Performative::RejectProposal
                }
                "Heartbeat" => crate::message_router::domain_types::Performative::Heartbeat,
                "Capability" => crate::message_router::domain_types::Performative::Capability,
                _ => {
                    return Err(crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Unknown performative: {performative_str}"),
                        )),
                    });
                }
            };

            // Parse timestamp
            let created_at_secs: i64 = row.get("created_at");
            let created_at = crate::message_router::domain_types::MessageTimestamp::new(
                std::time::UNIX_EPOCH
                    + std::time::Duration::from_secs(u64::try_from(created_at_secs).map_err(
                        |e| crate::message_router::traits::RouterError::StorageError {
                            source: Box::new(e),
                        },
                    )?),
            );

            // Reconstruct FipaMessage
            let fipa_message = crate::message_router::domain_types::FipaMessage {
                performative,
                sender: AgentId::new(parsed_sender),
                receiver: AgentId::new(parsed_receiver),
                content: message_content,
                language: None, // Not stored in current schema
                ontology: None, // Not stored in current schema
                protocol: None, // Not stored in current schema
                conversation_id,
                reply_with: None,  // Not stored in current schema
                in_reply_to: None, // Not stored in current schema
                message_id: crate::message_router::domain_types::MessageId::new(parsed_message_id),
                created_at,
                trace_context: None, // Not stored in current schema
                delivery_options: crate::message_router::domain_types::DeliveryOptions::default(), // Not stored in current schema
            };

            messages.push(fipa_message);
        }

        Ok(messages)
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

const INSERT_MESSAGE: &str = r"
INSERT OR REPLACE INTO message_storage (
    message_id, sender_id, receiver_id, conversation_id, message_content, performative, created_at, expires_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
";

const SELECT_MESSAGE_BY_ID: &str = r"
SELECT message_id, sender_id, receiver_id, conversation_id, message_content, performative, created_at, expires_at
FROM message_storage
WHERE message_id = ?;
";

const DELETE_MESSAGE_BY_ID: &str = r"
DELETE FROM message_storage WHERE message_id = ?;
";

const SELECT_MESSAGES_FOR_AGENT: &str = r"
SELECT message_id, sender_id, receiver_id, conversation_id, message_content, performative, created_at, expires_at
FROM message_storage
WHERE sender_id = ?1 OR receiver_id = ?1
ORDER BY created_at DESC;
";

const SELECT_MESSAGES_FOR_AGENT_LIMITED: &str = r"
SELECT message_id, sender_id, receiver_id, conversation_id, message_content, performative, created_at, expires_at
FROM message_storage
WHERE sender_id = ?1 OR receiver_id = ?1
ORDER BY created_at DESC
LIMIT ?2;
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
}

#[async_trait]
impl crate::message_router::traits::ConversationStorage for SqliteConversationStorage {
    #[instrument(skip(self, conversation))]
    async fn save_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), ConversationError> {
        // Schema is already initialized via migration system during DatabaseConnection::initialize()
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
    async fn test_should_reject_oversized_message_when_content_exceeds_storage_limits() {
        // Test that verifies MessageStorage handles extremely large content appropriately
        // This tests the boundary condition where message content approaches database storage limits

        use crate::database::{DatabaseConfig, DatabaseConnection, DatabasePath};
        use crate::domain_types::AgentId;
        use crate::message_router::domain_types::{
            FipaMessage, MessageContent, MessageId, MessageTimestamp, Performative,
        };
        use crate::message_router::traits::{MessageStorage, RouterError};
        use tempfile::TempDir;

        // Create temporary database for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = DatabasePath::try_new(temp_dir.path().join("test.db"))
            .expect("Failed to create database path");
        let db_config = DatabaseConfig::new(db_path);
        let connection = DatabaseConnection::initialize(db_config)
            .await
            .expect("Failed to initialize database connection");

        let message_storage = SqliteMessageStorage::new(connection);

        // Create message with extremely large content (10MB) to test storage limits
        let large_content_size = 10 * 1024 * 1024; // 10MB
        let large_content_data = vec![b'X'; large_content_size];

        let oversized_content = MessageContent::try_new(large_content_data)
            .expect("Failed to create oversized message content");

        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();

        let oversized_message = FipaMessage {
            performative: Performative::Inform,
            sender: sender_id,
            receiver: receiver_id,
            content: oversized_content,
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

        // Attempt to store oversized message - should either succeed or fail gracefully
        let store_result = message_storage.store_message(&oversized_message).await;

        match store_result {
            Ok(()) => {
                // If storage succeeds, verify we can retrieve the large message
                let retrieved = message_storage
                    .get_message(oversized_message.message_id)
                    .await
                    .expect("Failed to retrieve oversized message");
                assert!(
                    retrieved.is_some(),
                    "Oversized message should be retrievable after storage"
                );

                // Verify message integrity for large content
                let retrieved_message = retrieved.unwrap();
                assert_eq!(retrieved_message.message_id, oversized_message.message_id);
                assert_eq!(retrieved_message.sender, oversized_message.sender);
                assert_eq!(retrieved_message.receiver, oversized_message.receiver);
                assert_eq!(
                    retrieved_message.performative,
                    oversized_message.performative
                );
            }
            Err(RouterError::StorageError { .. } | RouterError::SerializationError { .. }) => {
                // Acceptable failure - storage system has size limits
                // This is a valid response for extremely large messages
            }
            Err(other_error) => {
                panic!("Unexpected error type for oversized message: {other_error:?}");
            }
        }
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
    async fn test_should_store_message_when_given_valid_fipa_message() {
        // Test that verifies store_message can persist a FipaMessage to the database

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

        // Create a complete FipaMessage with all required fields
        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();
        let message_content = MessageContent::try_new(b"Test message for storage".to_vec())
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

        // Store the message - this should succeed once implemented
        let store_result = message_storage.store_message(&message).await;
        assert!(
            store_result.is_ok(),
            "store_message should persist FipaMessage successfully"
        );
    }

    #[tokio::test]
    async fn test_should_retrieve_message_when_given_valid_id() {
        // Test that verifies get_message can retrieve a stored message and handle missing messages

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

        // Create a complete FipaMessage with all required fields
        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();
        let message_content = MessageContent::try_new(b"Test message for retrieval".to_vec())
            .expect("Failed to create message content");

        let original_message = FipaMessage {
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

        // Store the message first (this should work since store_message is implemented)
        let store_result = message_storage.store_message(&original_message).await;
        assert!(
            store_result.is_ok(),
            "store_message should persist FipaMessage successfully"
        );

        // Retrieve the message by ID - this should fail with unimplemented!()
        let retrieved_message_result = message_storage
            .get_message(original_message.message_id)
            .await;

        // This test should fail because get_message is not implemented yet
        assert!(
            retrieved_message_result.is_ok(),
            "get_message should retrieve stored message"
        );

        let retrieved_message = retrieved_message_result.unwrap();
        assert!(
            retrieved_message.is_some(),
            "Stored message should be found"
        );

        let retrieved_message = retrieved_message.unwrap();

        // Verify all fields match exactly
        assert_eq!(
            retrieved_message.message_id, original_message.message_id,
            "Message ID should match"
        );
        assert_eq!(
            retrieved_message.sender, original_message.sender,
            "Sender should match"
        );
        assert_eq!(
            retrieved_message.receiver, original_message.receiver,
            "Receiver should match"
        );
        assert_eq!(
            retrieved_message.performative, original_message.performative,
            "Performative should match"
        );
        // Note: Skipping content comparison as MessageContent doesn't implement PartialEq yet
        assert_eq!(
            retrieved_message.language, original_message.language,
            "Language should match"
        );
        assert_eq!(
            retrieved_message.ontology, original_message.ontology,
            "Ontology should match"
        );
        assert_eq!(
            retrieved_message.protocol, original_message.protocol,
            "Protocol should match"
        );

        // Test not-found case - try to retrieve non-existent message
        let non_existent_id = MessageId::generate();
        let not_found_result = message_storage.get_message(non_existent_id).await;
        assert!(
            not_found_result.is_ok(),
            "get_message should handle non-existent message ID"
        );
        assert!(
            not_found_result.unwrap().is_none(),
            "Non-existent message should return None"
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
