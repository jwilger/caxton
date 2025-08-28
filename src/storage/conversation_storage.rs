//! `SQLite` implementation of conversation storage for FIPA conversation persistence.
//!
//! This module provides a concrete implementation of the `ConversationStorage` trait
//! using `SQLite` for persistent storage of conversation state and participants with
//! optimized indexing and referential integrity.
//!
//! ## Architecture
//!
//! This implementation follows the functional core/imperative shell pattern:
//! - Pure domain validation and transformation logic in helper functions
//! - I/O operations isolated to trait implementation methods  
//! - Proper error handling with domain-specific error types
//!
//! ## Migration System Integration
//!
//! Table creation is handled by the migration system during `DatabaseConnection::initialize()`.
//! This ensures consistent schema versioning and eliminates the need for CREATE TABLE
//! IF NOT EXISTS patterns. The migration includes performance indexes for:
//!
//! - conversation archival status filtering
//! - temporal ordering for last activity queries
//! - protocol-based conversation filtering
//! - efficient participant lookups
//!
//! ## Performance Characteristics
//!
//! - Target: < 1ms average query time for individual conversation operations
//! - Optimized indexes support efficient conversation and participant queries
//! - Foreign key constraints ensure referential integrity
//! - Schema designed for multi-participant conversation scenarios

use async_trait::async_trait;
use sqlx::Row;
use std::{
    collections::HashSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::{info, instrument, warn};

use crate::{
    database::DatabaseConnection,
    domain_types::AgentId,
    message_router::{
        domain_types::{
            Conversation, ConversationCreatedAt, ConversationId, MessageCount, MessageTimestamp,
            ProtocolName,
        },
        traits::ConversationError,
    },
};

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

    /// Converts `SystemTime` to Unix seconds using standard library.
    fn system_time_to_unix_secs(time: SystemTime) -> Result<i64, ConversationError> {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
        i64::try_from(duration.as_secs()).map_err(|e| ConversationError::StorageError {
            source: Box::new(e),
        })
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
        let created_at = Self::system_time_to_unix_secs(conversation.created_at.into_inner())
            .map_err(|e| {
                warn!(
                    "Failed to convert created_at timestamp for conversation {}: {}",
                    conversation.id, e
                );
                e
            })?;
        let last_activity = Self::system_time_to_unix_secs(conversation.last_activity.into_inner())
            .map_err(|e| {
                warn!(
                    "Failed to convert last_activity timestamp for conversation {}: {}",
                    conversation.id, e
                );
                e
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
            UNIX_EPOCH
                + Duration::from_secs(u64::try_from(created_at_secs).map_err(|e| {
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
            UNIX_EPOCH
                + Duration::from_secs(u64::try_from(last_activity_secs).map_err(|e| {
                    warn!(
                        "Failed to convert last_activity timestamp for conversation {}: {}",
                        conversation_id, e
                    );
                    ConversationError::StorageError {
                        source: Box::new(e),
                    }
                })?),
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