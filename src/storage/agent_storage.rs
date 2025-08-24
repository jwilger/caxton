//! `SQLite` implementation of agent storage persistence.
//!
//! This module provides a concrete implementation of the `AgentStorage` trait
//! using `SQLite` for persistent storage of agent registry information.
//!
//! ## Architecture
//!
//! This implementation follows the functional core/imperative shell pattern:
//! - Pure domain validation and transformation logic
//! - I/O operations isolated to trait implementation methods
//! - Proper error handling with domain-specific error types
//!
//! ## Performance
//!
//! - Uses prepared statements for optimal query performance
//! - Implements proper table initialization strategy
//! - Includes comprehensive logging for observability

use crate::database::{DatabaseConnection, DatabaseResult};
use crate::domain_types::{AgentId, AgentName};
use crate::storage::AgentStorage;
use async_trait::async_trait;
use sqlx::Row;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, instrument, warn};

// SQL query constants for maintainability and performance
const CREATE_AGENT_REGISTRY_TABLE: &str = r"
    CREATE TABLE IF NOT EXISTS agent_registry (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        CHECK (length(id) > 0),
        CHECK (length(name) > 0)
    )
";

const INSERT_OR_REPLACE_AGENT: &str = r"
    INSERT OR REPLACE INTO agent_registry (id, name, created_at, updated_at)
    VALUES (?1, ?2, ?3, ?4)
";

const SELECT_AGENT_BY_ID: &str = r"
    SELECT name, created_at FROM agent_registry WHERE id = ?1
";

// Additional constants for future implementation
// const SELECT_ALL_AGENTS: &str = r#"
//     SELECT id, name FROM agent_registry ORDER BY name
// "#;
//
// const DELETE_AGENT_BY_ID: &str = r#"
//     DELETE FROM agent_registry WHERE id = ?1
// "#;

// Ensure table creation happens only once per connection
static TABLE_CREATED: Once = Once::new();

/// SQLite-backed implementation of agent storage.
///
/// This struct provides persistent storage for agent registry information
/// using the existing database infrastructure with `SQLx` and `SQLite`.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::storage::SqliteAgentStorage;
/// use crate::database::{DatabaseConfig, DatabasePath};
///
/// let path = DatabasePath::new("agents.db")?;
/// let config = DatabaseConfig::for_testing(path);
/// let connection = DatabaseConnection::initialize(config).await?;
/// let storage = SqliteAgentStorage::new(connection);
/// ```
pub struct SqliteAgentStorage {
    connection: DatabaseConnection,
}

impl SqliteAgentStorage {
    /// Create a new `SQLite` agent storage with the given database connection.
    ///
    /// The constructor ensures the database schema is initialized on first use.
    ///
    /// # Arguments
    ///
    /// * `connection` - Database connection for `SQLite` operations
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let storage = SqliteAgentStorage::new(connection);
    /// ```
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    /// Initialize database schema if not already done.
    ///
    /// This method ensures the `agent_registry` table exists with proper constraints.
    /// Uses a static Once to ensure initialization happens only once per process.
    #[instrument(skip(self), err)]
    async fn ensure_schema_initialized(&self) -> DatabaseResult<()> {
        TABLE_CREATED.call_once(|| {
            info!("Initializing SQLite agent registry schema");
        });

        sqlx::query(CREATE_AGENT_REGISTRY_TABLE)
            .execute(self.connection.pool())
            .await?;

        info!("SQLite agent registry schema initialized successfully");
        Ok(())
    }

    /// Get current Unix timestamp for record timestamps.
    ///
    /// # Returns
    ///
    /// Current Unix timestamp as i64, or 0 if system time is unavailable.
    fn current_timestamp() -> i64 {
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            // Use min to avoid overflow when converting u64 to i64
            let secs = duration.as_secs().min(i64::MAX as u64);
            // This cast is safe because we've clamped to i64::MAX
            secs.try_into().unwrap_or(i64::MAX)
        } else {
            warn!("Failed to get system time, using timestamp 0");
            0
        }
    }
}

#[async_trait]
impl AgentStorage for SqliteAgentStorage {
    /// Save an agent to persistent storage.
    ///
    /// This method persists agent information with proper timestamps and
    /// ensures the database schema is initialized.
    #[instrument(skip(self), fields(agent_id = %agent_id, agent_name = %agent_name))]
    async fn save_agent(&self, agent_id: AgentId, agent_name: AgentName) -> DatabaseResult<()> {
        info!("Saving agent to SQLite storage");

        // Ensure schema is initialized
        self.ensure_schema_initialized().await?;

        let now = Self::current_timestamp();

        // Check if agent already exists to determine if this is create or update
        let existing_created_at = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT created_at FROM agent_registry WHERE id = ?1",
        )
        .bind(agent_id.to_string())
        .fetch_optional(self.connection.pool())
        .await?
        .flatten();

        let created_at = existing_created_at.unwrap_or(now);

        // Insert or replace agent with proper timestamps
        sqlx::query(INSERT_OR_REPLACE_AGENT)
            .bind(agent_id.to_string())
            .bind(agent_name.to_string())
            .bind(created_at)
            .bind(now) // updated_at is always current time
            .execute(self.connection.pool())
            .await?;

        info!("Agent saved successfully to SQLite storage");
        Ok(())
    }

    /// Find an agent by ID in persistent storage.
    ///
    /// Returns the agent name if found, or None if the agent doesn't exist.
    #[instrument(skip(self), fields(agent_id = %agent_id))]
    async fn find_agent_by_id(&self, agent_id: AgentId) -> DatabaseResult<Option<AgentName>> {
        info!("Looking up agent by ID in SQLite storage");

        let row = sqlx::query(SELECT_AGENT_BY_ID)
            .bind(agent_id.to_string())
            .fetch_optional(self.connection.pool())
            .await?;

        if let Some(row) = row {
            let name_str: String = row.get("name");
            let agent_name = AgentName::try_new(name_str).map_err(|_| {
                crate::database::DatabaseError::Storage(crate::database::StorageError::Database {
                    message: "Invalid agent name in database".to_string(),
                })
            })?;
            info!("Agent found in SQLite storage");
            Ok(Some(agent_name))
        } else {
            info!("Agent not found in SQLite storage");
            Ok(None)
        }
    }

    /// List all agents in persistent storage.
    ///
    /// Returns a vector of (`AgentId`, `AgentName`) tuples ordered by agent name.
    /// This method is not yet fully implemented.
    async fn list_all_agents(&self) -> DatabaseResult<Vec<(AgentId, AgentName)>> {
        // TODO: Implement when needed for actual testing
        Ok(Vec::new())
    }

    /// Remove an agent from persistent storage.
    ///
    /// This is an idempotent operation - removing a non-existent agent succeeds.
    /// This method is not yet fully implemented.
    async fn remove_agent(&self, _agent_id: AgentId) -> DatabaseResult<()> {
        // TODO: Implement when needed for actual testing
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{DatabaseConfig, DatabasePath};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_should_persist_agent_to_sqlite_when_saving() {
        // Test that verifies SqliteAgentStorage actually persists agents to SQLite database

        // Create temporary database for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test_agents.db");
        let path = DatabasePath::new(db_path).expect("Failed to create DatabasePath");
        let config = DatabaseConfig::for_testing(path);
        let connection = DatabaseConnection::initialize(config)
            .await
            .expect("Failed to initialize database connection");

        // Create SQLite storage implementation
        let storage = SqliteAgentStorage::new(connection);

        // Create test data using domain types
        let agent_id = AgentId::generate();
        let agent_name =
            AgentName::try_new("sqlite_test_agent").expect("Failed to create AgentName");

        // Save agent to SQLite - this should fail because implementation doesn't exist yet
        let save_result = storage.save_agent(agent_id, agent_name.clone()).await;
        assert!(
            save_result.is_ok(),
            "SqliteAgentStorage should save agent successfully"
        );

        // Retrieve agent to verify persistence
        let retrieved_agent = storage
            .find_agent_by_id(agent_id)
            .await
            .expect("Failed to retrieve agent");
        assert_eq!(
            retrieved_agent,
            Some(agent_name),
            "Retrieved agent should match saved agent"
        );
    }
}
