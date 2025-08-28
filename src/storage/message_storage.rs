//! `SQLite` implementation of message storage for FIPA message persistence.
//!
//! This module provides a concrete implementation of the `MessageStorage` trait
//! using `SQLite` for persistent storage of FIPA messages with optimized indexing
//! and performance characteristics.
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
//! - `sender_id` lookups with temporal ordering
//! - `receiver_id` lookups with temporal ordering
//! - `conversation_id` filtering with temporal ordering
//! - `created_at` temporal queries
//!
//! ## Performance Characteristics
//!
//! - Target: < 1ms average query time for individual message operations
//! - Optimized indexes support efficient message routing and conversation retrieval
//! - UUID format validation via CHECK constraints ensures data integrity
//! - Schema designed for high-throughput message processing

use async_trait::async_trait;
use sqlx::Row;
use std::{
    borrow::Cow,
    io::{Error as IoError, ErrorKind},
    time::{Duration, UNIX_EPOCH},
};
use tracing::{info, instrument, warn};

use crate::{
    database::DatabaseConnection,
    domain_types::AgentId,
    message_router::{
        domain_types::{FipaMessage, MessageContent, MessageId, MessageTimestamp, Performative},
        traits::RouterError,
    },
};

// SQL Constants for MessageStorage
const INSERT_MESSAGE: &str = r"
INSERT OR REPLACE INTO message_storage (
    message_id, sender_id, receiver_id, conversation_id, message_content, performative, created_at, expires_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
";

/// Default query limit when usize conversion fails.
/// Set to 1000 to provide reasonable protection against excessive memory usage
/// while still allowing for large result sets in normal operation.
const DEFAULT_QUERY_LIMIT: i64 = 1000;

/// TTL duration in seconds for message expiration.
/// Set to 2 seconds for testing purposes to allow verification of expiration behavior
/// in test scenarios without excessive wait times.
const MESSAGE_TTL_SECONDS: i64 = 2;

const SELECT_MESSAGE_BY_ID: &str = r"
SELECT message_id, sender_id, receiver_id, conversation_id, message_content, performative, created_at, expires_at
FROM message_storage
WHERE message_id = ? AND (expires_at IS NULL OR expires_at > strftime('%s', 'now'));
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

/// `SQLite` implementation of message storage for FIPA message persistence.
///
/// This implementation stores FIPA messages in a dedicated `message_storage` table
/// created via the migration system (`003_create_message_storage.sql`). The table
/// schema includes optimized indexes for common query patterns including sender/receiver
/// lookups and conversation-based filtering.
///
/// # Implementation Status
///
/// Currently provides fully implemented methods for message storage, retrieval,
/// removal, and listing operations with proper domain type integration, error handling,
/// and observability.
pub struct SqliteMessageStorage {
    connection: DatabaseConnection,
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
    #[must_use]
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    /// Parses a database row into a `FipaMessage`.
    ///
    /// This helper method reconstructs `FipaMessage` instances from `SQLite` row data,
    /// eliminating duplication between `get_message` and `list_agent_messages` operations.
    fn parse_message_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<FipaMessage, RouterError> {
        let message_id_str: String = row.get("message_id");
        let (message_id, sender, receiver) = Self::parse_message_identifiers(row, &message_id_str)?;
        let conversation_id = Self::parse_conversation_id(row)?;
        let message_content = Self::parse_message_content_from_row(row, &message_id_str)?;
        let performative = Self::parse_performative_from_row(row)?;
        let created_at = Self::parse_timestamp_from_row(row, &message_id_str)?;

        Ok(FipaMessage {
            performative,
            sender,
            receiver,
            content: message_content,
            language: None, // Not stored in current schema
            ontology: None, // Not stored in current schema
            protocol: None, // Not stored in current schema
            conversation_id,
            reply_with: None,  // Not stored in current schema
            in_reply_to: None, // Not stored in current schema
            message_id,
            created_at,
            trace_context: None, // Not stored in current schema
            delivery_options: crate::message_router::domain_types::DeliveryOptions::default(),
        })
    }

    /// Parses message identifiers (`message_id`, `sender_id`, `receiver_id`) from database row.
    fn parse_message_identifiers(
        row: &sqlx::sqlite::SqliteRow,
        message_id_str: &str,
    ) -> Result<(MessageId, AgentId, AgentId), RouterError> {
        let sender_str: String = row.get("sender_id");
        let receiver_str: String = row.get("receiver_id");

        let parsed_message_id =
            uuid::Uuid::parse_str(message_id_str).map_err(|e| RouterError::StorageError {
                source: Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    format!("Invalid message ID UUID '{message_id_str}': {e}"),
                )),
            })?;

        let parsed_sender =
            uuid::Uuid::parse_str(&sender_str).map_err(|e| RouterError::StorageError {
                source: Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    format!("Invalid sender UUID '{sender_str}': {e}"),
                )),
            })?;

        let parsed_receiver =
            uuid::Uuid::parse_str(&receiver_str).map_err(|e| RouterError::StorageError {
                source: Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    format!("Invalid receiver UUID '{receiver_str}': {e}"),
                )),
            })?;

        Ok((
            MessageId::new(parsed_message_id),
            AgentId::new(parsed_sender),
            AgentId::new(parsed_receiver),
        ))
    }

    /// Parses optional conversation ID from database row.
    fn parse_conversation_id(
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<Option<crate::message_router::domain_types::ConversationId>, RouterError> {
        let conversation_id_str: Option<String> = row.get("conversation_id");
        if let Some(conv_str) = conversation_id_str {
            let parsed_uuid =
                uuid::Uuid::parse_str(&conv_str).map_err(|e| RouterError::StorageError {
                    source: Box::new(IoError::new(
                        ErrorKind::InvalidData,
                        format!("Invalid conversation UUID '{conv_str}': {e}"),
                    )),
                })?;
            Ok(Some(
                crate::message_router::domain_types::ConversationId::new(parsed_uuid),
            ))
        } else {
            Ok(None)
        }
    }

    /// Parses message content from database row.
    fn parse_message_content_from_row(
        row: &sqlx::sqlite::SqliteRow,
        message_id_str: &str,
    ) -> Result<MessageContent, RouterError> {
        let content_string: String = row.get("message_content");
        let content_bytes = Self::parse_message_content(&content_string, message_id_str)?;
        MessageContent::try_new(content_bytes.into_owned()).map_err(|e| RouterError::StorageError {
            source: Box::new(IoError::new(
                ErrorKind::InvalidData,
                format!("Invalid message content for message '{message_id_str}': {e}"),
            )),
        })
    }

    /// Parses performative from database row.
    fn parse_performative_from_row(
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<Performative, RouterError> {
        let performative_str: String = row.get("performative");
        Self::parse_performative(&performative_str)
    }

    /// Parses timestamp from database row.
    fn parse_timestamp_from_row(
        row: &sqlx::sqlite::SqliteRow,
        message_id_str: &str,
    ) -> Result<MessageTimestamp, RouterError> {
        let created_at_secs: i64 = row.get("created_at");
        Self::parse_timestamp(created_at_secs, message_id_str)
    }

    /// Parses length-prefixed message content in format: `{length}::{content}`
    fn parse_message_content<'a>(
        content_string: &'a str,
        message_id: &str,
    ) -> Result<Cow<'a, [u8]>, RouterError> {
        let parts: Vec<&str> = content_string.splitn(2, "::").collect();
        if parts.len() != 2 {
            return Err(RouterError::StorageError {
                source: Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Invalid length-prefixed content format for message '{message_id}': expected 'length::content', got '{content_string}'"
                    ),
                )),
            });
        }
        Ok(Cow::Borrowed(parts[1].as_bytes()))
    }

    /// Parses performative string into enum variant.
    fn parse_performative(performative_str: &str) -> Result<Performative, RouterError> {
        match performative_str {
            "Inform" => Ok(Performative::Inform),
            "Request" => Ok(Performative::Request),
            "Agree" => Ok(Performative::Agree),
            "Refuse" => Ok(Performative::Refuse),
            "NotUnderstood" => Ok(Performative::NotUnderstood),
            "Failure" => Ok(Performative::Failure),
            "QueryIf" => Ok(Performative::QueryIf),
            "QueryRef" => Ok(Performative::QueryRef),
            "Propose" => Ok(Performative::Propose),
            "AcceptProposal" => Ok(Performative::AcceptProposal),
            "RejectProposal" => Ok(Performative::RejectProposal),
            "Heartbeat" => Ok(Performative::Heartbeat),
            "Capability" => Ok(Performative::Capability),
            unknown => Err(RouterError::StorageError {
                source: Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Unknown performative '{unknown}'. Valid values: Inform, Request, Agree, Refuse, NotUnderstood, Failure, QueryIf, QueryRef, Propose, AcceptProposal, RejectProposal, Heartbeat, Capability"
                    ),
                )),
            }),
        }
    }

    /// Serializes message content to length-prefixed format.
    #[must_use]
    fn serialize_message_content(content_bytes: &[u8]) -> String {
        use std::fmt::Write;

        let content_len = content_bytes.len();
        let mut length_digits = 1;
        let mut temp = content_len;
        while temp >= 10 {
            length_digits += 1;
            temp /= 10;
        }
        let estimated_capacity = length_digits + 2 + content_len;

        let mut result = String::with_capacity(estimated_capacity);
        write!(&mut result, "{content_len}::").expect("Writing length should never fail");

        if let Ok(valid_utf8) = std::str::from_utf8(content_bytes) {
            result.push_str(valid_utf8);
        } else {
            let content_string = String::from_utf8_lossy(content_bytes);
            result.push_str(&content_string);
        }

        result
    }

    /// Converts performative to string representation.
    #[must_use]
    fn performative_to_str(performative: Performative) -> &'static str {
        match performative {
            Performative::Request => "Request",
            Performative::Inform => "Inform",
            Performative::Agree => "Agree",
            Performative::Refuse => "Refuse",
            Performative::NotUnderstood => "NotUnderstood",
            Performative::Failure => "Failure",
            Performative::QueryIf => "QueryIf",
            Performative::QueryRef => "QueryRef",
            Performative::Propose => "Propose",
            Performative::AcceptProposal => "AcceptProposal",
            Performative::RejectProposal => "RejectProposal",
            Performative::Heartbeat => "Heartbeat",
            Performative::Capability => "Capability",
        }
    }

    /// Converts `MessageTimestamp` to Unix seconds using standard library.
    fn timestamp_to_unix_secs(timestamp: MessageTimestamp) -> Result<i64, RouterError> {
        let duration = timestamp
            .into_inner()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        i64::try_from(duration.as_secs()).map_err(|e| RouterError::StorageError {
            source: Box::new(e),
        })
    }

    /// Parses Unix timestamp using standard library duration arithmetic.
    fn parse_timestamp(
        created_at_secs: i64,
        message_id: &str,
    ) -> Result<MessageTimestamp, RouterError> {
        let timestamp_u64 =
            u64::try_from(created_at_secs).map_err(|e| RouterError::StorageError {
                source: Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    format!("Invalid timestamp {created_at_secs} for message '{message_id}': {e}"),
                )),
            })?;

        Ok(MessageTimestamp::new(
            UNIX_EPOCH + Duration::from_secs(timestamp_u64),
        ))
    }
}

#[async_trait]
impl crate::message_router::traits::MessageStorage for SqliteMessageStorage {
    #[instrument(skip(self, message), fields(message_id = %message.message_id))]
    async fn store_message(&self, message: &FipaMessage) -> Result<(), RouterError> {
        let message_content_string = Self::serialize_message_content(message.content.as_bytes());
        let performative_str = Self::performative_to_str(message.performative);
        let created_at = Self::timestamp_to_unix_secs(message.created_at)?;

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
            .bind(message_content_string)
            .bind(performative_str)
            .bind(created_at)
            .bind(Some(created_at + MESSAGE_TTL_SECONDS)) // Message TTL for testing purposes
            .execute(self.connection.pool())
            .await
            .map_err(|e| {
                warn!(
                    "Failed to store message {} from {} to {}: {}",
                    message.message_id, message.sender, message.receiver, e
                );
                RouterError::StorageError {
                    source: Box::new(e),
                }
            })?;

        info!("Stored message {}", message.message_id);
        Ok(())
    }

    #[instrument(skip(self), fields(message_id = %message_id))]
    async fn get_message(&self, message_id: MessageId) -> Result<Option<FipaMessage>, RouterError> {
        let row = sqlx::query(SELECT_MESSAGE_BY_ID)
            .bind(message_id.to_string())
            .fetch_optional(self.connection.pool())
            .await
            .map_err(|e| {
                warn!("Failed to retrieve message {}: {}", message_id, e);
                RouterError::StorageError {
                    source: Box::new(e),
                }
            })?;

        let Some(row) = row else {
            info!("Message {} not found in storage", message_id);
            return Ok(None);
        };

        // Use centralized message parsing logic
        let fipa_message = Self::parse_message_from_row(&row)?;

        info!(
            "Retrieved message {} from sender {} to receiver {}",
            message_id, fipa_message.sender, fipa_message.receiver
        );

        Ok(Some(fipa_message))
    }

    #[instrument(skip(self), fields(message_id = %message_id))]
    async fn remove_message(&self, message_id: MessageId) -> Result<(), RouterError> {
        let result = sqlx::query(DELETE_MESSAGE_BY_ID)
            .bind(message_id.to_string())
            .execute(self.connection.pool())
            .await
            .map_err(|e| {
                warn!("Failed to remove message {}: {}", message_id, e);
                RouterError::StorageError {
                    source: Box::new(e),
                }
            })?;

        let rows_affected = result.rows_affected();
        if rows_affected > 0 {
            info!(
                "Removed message {} ({} rows affected)",
                message_id, rows_affected
            );
        } else {
            info!(
                "Message {} not found for removal (idempotent operation)",
                message_id
            );
        }

        Ok(())
    }

    #[instrument(skip(self), fields(agent_id = %agent_id, limit = ?limit))]
    async fn list_agent_messages(
        &self,
        agent_id: AgentId,
        limit: Option<usize>,
    ) -> Result<Vec<FipaMessage>, RouterError> {
        // Choose appropriate query based on limit parameter to avoid dynamic SQL construction
        let rows = if let Some(limit_value) = limit {
            // Use safe conversion with reasonable maximum limit to prevent memory issues
            let limit_i64 = i64::try_from(limit_value).unwrap_or(DEFAULT_QUERY_LIMIT);

            sqlx::query(SELECT_MESSAGES_FOR_AGENT_LIMITED)
                .bind(agent_id.to_string())
                .bind(limit_i64)
                .fetch_all(self.connection.pool())
                .await
                .map_err(|e| {
                    warn!(
                        "Failed to list messages for agent {} (limited to {}): {}",
                        agent_id, limit_value, e
                    );
                    crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(e),
                    }
                })?
        } else {
            sqlx::query(SELECT_MESSAGES_FOR_AGENT)
                .bind(agent_id.to_string())
                .fetch_all(self.connection.pool())
                .await
                .map_err(|e| {
                    warn!("Failed to list all messages for agent {}: {}", agent_id, e);
                    crate::message_router::traits::RouterError::StorageError {
                        source: Box::new(e),
                    }
                })?
        };

        // Use centralized message parsing for consistency and maintainability
        let mut messages = Vec::with_capacity(rows.len());
        for row in &rows {
            let fipa_message = Self::parse_message_from_row(row)?;
            messages.push(fipa_message);
        }

        info!(
            "Listed {} messages for agent {} {}",
            messages.len(),
            agent_id,
            if let Some(limit_value) = limit {
                format!("(limited to {limit_value})")
            } else {
                "(no limit)".to_string()
            }
        );

        Ok(messages)
    }
}
