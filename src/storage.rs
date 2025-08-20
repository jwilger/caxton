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
use crate::domain_types::{AgentId, AgentName};
use sqlx::Row;
use uuid::Uuid;

// =============================================================================
// FUNCTIONAL CORE - Pure functions with no I/O
// =============================================================================

/// Pure SQL generation functions for agent storage operations
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
}

/// Pure data mapping functions for converting between types
mod mapping {
    use super::{AgentId, AgentName, StorageError, StorageResult, StoredAgent, Uuid};

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

    /// Create `StoredAgent` from parsed components (functional core)
    pub(super) fn create_stored_agent(id: AgentId, name: AgentName) -> StoredAgent {
        StoredAgent { id, name }
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
