//! Agent storage module for SQLite persistence
//!
//! This module provides agent storage and retrieval functionality following
//! functional core / imperative shell architecture:
//!
//! ## Architecture
//!
//! - **Functional Core**: SQL generation, data mapping, validation
//! - **Imperative Shell**: Database I/O operations, connection management
//!
//! ## Performance Requirements
//!
//! - Agent storage operations: < 1ms
//! - Agent retrieval operations: < 1ms

use crate::database::{DatabaseConnection, DatabaseResult, StorageError, StorageResult};
use crate::domain_types::{AgentId, AgentName, MessageCount};
use crate::message_router::domain_types::{
    ConversationCreatedAt, ConversationId, MessageTimestamp,
};
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

// =============================================================================
// FUNCTIONAL CORE - Pure functions with no I/O
// =============================================================================

/// Pure SQL generation functions for agent and conversation storage operations
mod sql {

    /// Generate SQL for creating agents table (functional core)
    pub(super) fn create_agents_table() -> &'static str {
        "CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        )"
    }

    /// Generate SQL for inserting/updating agent (functional core)
    pub(super) fn upsert_agent() -> &'static str {
        "INSERT OR REPLACE INTO agents (id, name) VALUES (?, ?)"
    }

    /// Generate SQL for selecting agent by ID (functional core)
    pub(super) fn select_agent_by_id() -> &'static str {
        "SELECT id, name FROM agents WHERE id = ?"
    }

    /// Generate SQL for creating conversations table (functional core)
    pub(super) fn create_conversations_table() -> &'static str {
        "CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL,
            last_activity INTEGER NOT NULL,
            message_count INTEGER NOT NULL
        )"
    }

    /// Generate SQL for creating `conversation_participants` table (functional core)
    pub(super) fn create_conversation_participants_table() -> &'static str {
        "CREATE TABLE IF NOT EXISTS conversation_participants (
            conversation_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            PRIMARY KEY (conversation_id, agent_id),
            FOREIGN KEY (conversation_id) REFERENCES conversations(id)
        )"
    }

    /// Generate SQL for inserting/updating conversation (functional core)
    pub(super) fn upsert_conversation() -> &'static str {
        "INSERT OR REPLACE INTO conversations (id, created_at, last_activity, message_count) VALUES (?, ?, ?, ?)"
    }

    /// Generate SQL for inserting conversation participant (functional core)
    pub(super) fn insert_conversation_participant() -> &'static str {
        "INSERT OR IGNORE INTO conversation_participants (conversation_id, agent_id) VALUES (?, ?)"
    }

    /// Generate SQL for selecting conversation by ID (functional core)
    pub(super) fn select_conversation_by_id() -> &'static str {
        "SELECT id, created_at, last_activity, message_count FROM conversations WHERE id = ?"
    }

    /// Generate SQL for selecting conversation participants (functional core)
    pub(super) fn select_conversation_participants() -> &'static str {
        "SELECT agent_id FROM conversation_participants WHERE conversation_id = ?"
    }

    /// Generate SQL for deleting conversation participants (functional core)
    pub(super) fn delete_conversation_participants() -> &'static str {
        "DELETE FROM conversation_participants WHERE conversation_id = ?"
    }
}

/// Pure data mapping functions for converting between types
mod mapping {
    use super::{
        AgentId, AgentName, ConversationCreatedAt, ConversationId, ConversationState, MessageCount,
        MessageTimestamp, StorageError, StorageResult, StoredAgent, Uuid,
    };
    use std::collections::HashSet;

    /// Convert `AgentId` to string for database storage (functional core)
    pub(super) fn agent_id_to_string(id: AgentId) -> String {
        id.to_string()
    }

    /// Convert `AgentName` to string for database storage (functional core)
    pub(super) fn agent_name_to_string(name: &AgentName) -> String {
        name.to_string()
    }

    /// Parse agent ID from database string (functional core)
    pub(super) fn parse_agent_id(id_str: &str) -> StorageResult<AgentId> {
        let uuid = Uuid::parse_str(id_str).map_err(|e| StorageError::Database {
            message: format!("Invalid agent ID format: {e}"),
        })?;

        Ok(AgentId::from(uuid))
    }

    /// Parse agent name from database string (functional core)
    pub(super) fn parse_agent_name(name_str: &str) -> StorageResult<AgentName> {
        AgentName::try_new(name_str.to_string()).map_err(|e| StorageError::Database {
            message: format!("Invalid agent name: {e}"),
        })
    }

    /// Convert `ConversationId` to string for database storage (functional core)
    pub(super) fn conversation_id_to_string(id: ConversationId) -> String {
        id.to_string()
    }

    /// Convert `ConversationCreatedAt` to timestamp for database storage (functional core)
    pub(super) fn created_at_to_timestamp(created_at: ConversationCreatedAt) -> i64 {
        let secs = created_at
            .as_system_time()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // SQLite timestamps are i64, so we need to handle the conversion carefully
        i64::try_from(secs).unwrap_or(i64::MAX)
    }

    /// Convert `MessageTimestamp` to timestamp for database storage (functional core)
    pub(super) fn message_timestamp_to_timestamp(timestamp: MessageTimestamp) -> i64 {
        let secs = timestamp
            .as_system_time()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // SQLite timestamps are i64, so we need to handle the conversion carefully
        i64::try_from(secs).unwrap_or(i64::MAX)
    }

    /// Convert `MessageCount` to integer for database storage (functional core)
    pub(super) fn message_count_to_i64(count: MessageCount) -> i64 {
        // Message counts should be reasonable, so this conversion is safe
        i64::try_from(count.into_inner()).unwrap_or(i64::MAX)
    }

    /// Parse timestamp to `ConversationCreatedAt` (functional core)
    pub(super) fn parse_created_at(timestamp: i64) -> ConversationCreatedAt {
        let secs = u64::try_from(timestamp).unwrap_or(0);
        let duration = std::time::Duration::from_secs(secs);
        let system_time = std::time::UNIX_EPOCH + duration;
        ConversationCreatedAt::new(system_time)
    }

    /// Parse timestamp to `MessageTimestamp` (functional core)
    pub(super) fn parse_message_timestamp(timestamp: i64) -> MessageTimestamp {
        let secs = u64::try_from(timestamp).unwrap_or(0);
        let duration = std::time::Duration::from_secs(secs);
        let system_time = std::time::UNIX_EPOCH + duration;
        MessageTimestamp::new(system_time)
    }

    /// Parse integer to `MessageCount` (functional core)
    pub(super) fn parse_message_count(count: i64) -> MessageCount {
        let usize_count = usize::try_from(count).unwrap_or(0);
        MessageCount::new(usize_count)
    }

    /// Create `StoredAgent` from parsed components (functional core)
    pub(super) fn create_stored_agent(id: AgentId, name: AgentName) -> StoredAgent {
        StoredAgent { id, name }
    }

    /// Create `ConversationState` from parsed components (functional core)
    pub(super) fn create_conversation_state(
        conversation_id: ConversationId,
        participants: HashSet<AgentId>,
        created_at: ConversationCreatedAt,
        last_activity: MessageTimestamp,
        message_count: MessageCount,
    ) -> ConversationState {
        ConversationState::new(
            conversation_id,
            participants,
            created_at,
            last_activity,
            message_count,
        )
    }
}

// =============================================================================
// DOMAIN TYPES
// =============================================================================

/// Agent representation for storage operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAgent {
    id: AgentId,
    name: AgentName,
}

impl StoredAgent {
    /// Create a new stored agent
    pub fn new(id: AgentId, name: AgentName) -> Self {
        Self { id, name }
    }

    /// Get the agent ID
    pub fn id(&self) -> AgentId {
        self.id
    }

    /// Get the agent name
    pub fn name(&self) -> &AgentName {
        &self.name
    }
}

// =============================================================================
// IMPERATIVE SHELL - I/O operations and database interactions
// =============================================================================

/// Agent storage interface for `SQLite` database operations
///
/// This struct follows the imperative shell pattern, handling all I/O operations
/// while delegating business logic to the functional core.
pub struct AgentStorage {
    connection: DatabaseConnection,
}

impl AgentStorage {
    /// Create new agent storage with database connection
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    /// Store an agent in the database (imperative shell)
    ///
    /// This method orchestrates the storage operation by:
    /// 1. Ensuring the agents table exists
    /// 2. Converting domain types to database format using functional core
    /// 3. Executing the database operation
    ///
    /// # Performance
    ///
    /// Target: < 1ms for agent storage operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database table creation fails
    /// - Agent data conversion fails
    /// - Database insertion fails
    pub async fn store_agent(
        &self,
        agent_id: AgentId,
        agent_name: AgentName,
    ) -> DatabaseResult<()> {
        // Ensure table exists (imperative shell)
        self.ensure_agents_table_exists().await?;

        // Convert domain types to database format (functional core)
        let id_str = mapping::agent_id_to_string(agent_id);
        let name_str = mapping::agent_name_to_string(&agent_name);

        // Execute database operation (imperative shell)
        sqlx::query(sql::upsert_agent())
            .bind(id_str)
            .bind(name_str)
            .execute(self.connection.pool())
            .await?;

        Ok(())
    }

    /// Get an agent from the database by ID (imperative shell)
    ///
    /// This method orchestrates the retrieval operation by:
    /// 1. Executing the database query
    /// 2. Extracting raw data from the result
    /// 3. Converting database format to domain types using functional core
    ///
    /// # Performance
    ///
    /// Target: < 1ms for agent retrieval operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database query fails
    /// - Agent is not found
    /// - Data conversion from database format fails
    pub async fn get_agent(&self, agent_id: AgentId) -> DatabaseResult<StoredAgent> {
        // Convert agent ID for query (functional core)
        let id_str = mapping::agent_id_to_string(agent_id);

        // Execute database query (imperative shell)
        let row = sqlx::query(sql::select_agent_by_id())
            .bind(id_str)
            .fetch_one(self.connection.pool())
            .await?;

        // Extract raw data from database row (imperative shell)
        let id_str: String = row.get("id");
        let name_str: String = row.get("name");

        // Convert database format to domain types (functional core)
        let id = mapping::parse_agent_id(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let name = mapping::parse_agent_name(&name_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "name".to_string(),
            source: Box::new(e),
        })?;

        // Create domain object (functional core)
        Ok(mapping::create_stored_agent(id, name))
    }

    /// Ensure agents table exists in database (imperative shell)
    async fn ensure_agents_table_exists(&self) -> DatabaseResult<()> {
        sqlx::query(sql::create_agents_table())
            .execute(self.connection.pool())
            .await?;
        Ok(())
    }
}

// =============================================================================
// CONVERSATION STORAGE IMPLEMENTATION
// =============================================================================

/// Conversation state representation for storage operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationState {
    conversation_id: ConversationId,
    participants: HashSet<AgentId>,
    created_at: ConversationCreatedAt,
    last_activity: MessageTimestamp,
    message_count: MessageCount,
}

impl ConversationState {
    /// Create a new conversation state
    pub fn new(
        conversation_id: ConversationId,
        participants: HashSet<AgentId>,
        created_at: ConversationCreatedAt,
        last_activity: MessageTimestamp,
        message_count: MessageCount,
    ) -> Self {
        Self {
            conversation_id,
            participants,
            created_at,
            last_activity,
            message_count,
        }
    }

    /// Get the conversation ID
    pub fn conversation_id(&self) -> ConversationId {
        self.conversation_id
    }

    /// Get the participants
    pub fn participants(&self) -> &HashSet<AgentId> {
        &self.participants
    }

    /// Get the created at timestamp
    pub fn created_at(&self) -> ConversationCreatedAt {
        self.created_at
    }

    /// Get the last activity timestamp
    pub fn last_activity(&self) -> MessageTimestamp {
        self.last_activity
    }

    /// Get the message count
    pub fn message_count(&self) -> MessageCount {
        self.message_count
    }
}

/// Conversation storage interface for `SQLite` database operations
///
/// This struct follows the imperative shell pattern, handling all I/O operations
/// while delegating business logic to the functional core.
pub struct ConversationStorage {
    connection: DatabaseConnection,
}

impl ConversationStorage {
    /// Create new conversation storage with database connection
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    /// Store conversation state in the database (imperative shell)
    ///
    /// This method orchestrates the storage operation by:
    /// 1. Ensuring the conversation tables exist
    /// 2. Converting domain types to database format using functional core
    /// 3. Executing the database operations in a transaction
    ///
    /// # Performance
    ///
    /// Target: < 1ms for conversation storage operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database table creation fails
    /// - Conversation data conversion fails
    /// - Database insertion fails
    pub async fn store_conversation_state(
        &self,
        conversation_id: ConversationId,
        participants: HashSet<AgentId>,
        created_at: ConversationCreatedAt,
        last_activity: MessageTimestamp,
        message_count: MessageCount,
    ) -> DatabaseResult<()> {
        // Ensure tables exist (imperative shell)
        self.ensure_conversation_tables_exist().await?;

        // Convert domain types to database format (functional core)
        let id_str = mapping::conversation_id_to_string(conversation_id);
        let created_at_timestamp = mapping::created_at_to_timestamp(created_at);
        let last_activity_timestamp = mapping::message_timestamp_to_timestamp(last_activity);
        let message_count_int = mapping::message_count_to_i64(message_count);

        // Begin transaction (imperative shell)
        let mut tx = self.connection.pool().begin().await?;

        // Store conversation metadata (imperative shell)
        sqlx::query(sql::upsert_conversation())
            .bind(&id_str)
            .bind(created_at_timestamp)
            .bind(last_activity_timestamp)
            .bind(message_count_int)
            .execute(&mut *tx)
            .await?;

        // Clear existing participants (imperative shell)
        sqlx::query(sql::delete_conversation_participants())
            .bind(&id_str)
            .execute(&mut *tx)
            .await?;

        // Store participants (imperative shell)
        for participant in participants {
            let participant_str = mapping::agent_id_to_string(participant);
            sqlx::query(sql::insert_conversation_participant())
                .bind(&id_str)
                .bind(participant_str)
                .execute(&mut *tx)
                .await?;
        }

        // Commit transaction (imperative shell)
        tx.commit().await?;

        Ok(())
    }

    /// Get conversation state from the database (imperative shell)
    ///
    /// This method orchestrates the retrieval operation by:
    /// 1. Executing the database queries to get conversation and participants
    /// 2. Extracting raw data from the results
    /// 3. Converting database format to domain types using functional core
    ///
    /// # Performance
    ///
    /// Target: < 1ms for conversation retrieval operations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database query fails
    /// - Conversation is not found
    /// - Data conversion from database format fails
    pub async fn get_conversation_state(
        &self,
        conversation_id: ConversationId,
    ) -> DatabaseResult<ConversationState> {
        // Convert conversation ID for query (functional core)
        let id_str = mapping::conversation_id_to_string(conversation_id);

        // Execute conversation query (imperative shell)
        let row = sqlx::query(sql::select_conversation_by_id())
            .bind(&id_str)
            .fetch_one(self.connection.pool())
            .await?;

        // Extract conversation data from database row (imperative shell)
        let created_at_timestamp: i64 = row.get("created_at");
        let last_activity_timestamp: i64 = row.get("last_activity");
        let message_count_int: i64 = row.get("message_count");

        // Execute participants query (imperative shell)
        let participant_rows = sqlx::query(sql::select_conversation_participants())
            .bind(&id_str)
            .fetch_all(self.connection.pool())
            .await?;

        // Extract participants from database rows (imperative shell)
        let mut participants = HashSet::new();
        for participant_row in participant_rows {
            let agent_id_str: String = participant_row.get("agent_id");
            let agent_id =
                mapping::parse_agent_id(&agent_id_str).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "agent_id".to_string(),
                    source: Box::new(e),
                })?;
            participants.insert(agent_id);
        }

        // Convert database format to domain types (functional core)
        let created_at = mapping::parse_created_at(created_at_timestamp);

        let last_activity = mapping::parse_message_timestamp(last_activity_timestamp);

        let message_count = mapping::parse_message_count(message_count_int);

        // Create domain object (functional core)
        Ok(mapping::create_conversation_state(
            conversation_id,
            participants,
            created_at,
            last_activity,
            message_count,
        ))
    }

    /// Ensure conversation tables exist in database (imperative shell)
    async fn ensure_conversation_tables_exist(&self) -> DatabaseResult<()> {
        sqlx::query(sql::create_conversations_table())
            .execute(self.connection.pool())
            .await?;

        sqlx::query(sql::create_conversation_participants_table())
            .execute(self.connection.pool())
            .await?;

        Ok(())
    }
}
