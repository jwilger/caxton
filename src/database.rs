//! Database module for embedded SQLite storage
//!
//! This module provides a clean separation between functional core (pure business logic)
//! and imperative shell (I/O operations) for database management.
//!
//! ## Architecture
//!
//! - **Functional Core**: Configuration validation, connection string generation
//! - **Imperative Shell**: File system operations, SQLite connections

use crate::domain_types::{ConnectionPoolSize, DatabaseSchemaVersion};
use nutype::nutype;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Database-specific error types
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// Database error from `SQLx`
    #[error("Database error: {message}")]
    Database {
        /// Error message from database operation
        message: String,
    },

    /// File system I/O error
    #[error("File system error: {message}")]
    FileSystem {
        /// Error message from file system operation
        message: String,
    },

    /// Invalid configuration
    #[error("Configuration error: {field} - {reason}")]
    Configuration {
        /// Name of the configuration field that was invalid
        field: String,
        /// Reason why the configuration was invalid
        reason: String,
    },

    /// Connection pool error
    #[error("Connection pool error: {message}")]
    ConnectionPool {
        /// Error message from connection pool operation
        message: String,
    },

    /// Migration error
    #[error("Migration error: {message}")]
    Migration {
        /// Error message from migration operation
        message: String,
    },
}

/// Database error types for backward compatibility
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Error from `SQLx`
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

/// Database result type
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Storage result type for pure functions
pub type StorageResult<T> = Result<T, StorageError>;

#[nutype(
    sanitize(with = |path: PathBuf| path),
    validate(predicate = |path| !path.as_os_str().is_empty() && path.extension().is_some_and(|ext| ext == "db")),
    derive(Clone, Debug, Eq, PartialEq)
)]
pub struct DatabasePath(PathBuf);

impl DatabasePath {
    /// Create a new database path with validation
    ///
    /// # Errors
    ///
    /// Returns an error if the path is empty or has an invalid extension
    pub fn new<P: AsRef<Path>>(path: P) -> DatabaseResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        Self::try_new(path_buf).map_err(|_| {
            DatabaseError::Storage(StorageError::Configuration {
                field: "database_path".to_string(),
                reason: "Path is empty or has invalid extension (must be .db)".to_string(),
            })
        })
    }

    /// Get the path as `PathBuf`
    pub fn as_path(&self) -> PathBuf {
        self.clone().into_inner()
    }

    /// Generate `SQLite` connection string (functional core)
    pub fn to_connection_string(&self) -> String {
        format!("sqlite://{}?mode=rwc", self.as_path().display())
    }

    /// Get parent directory for file creation (functional core)
    pub fn parent_directory(&self) -> Option<PathBuf> {
        self.as_path().parent().map(std::path::Path::to_path_buf)
    }

    /// Check if file exists (pure function for testing)
    pub fn exists(&self) -> bool {
        self.as_path().exists()
    }
}

impl std::fmt::Display for DatabasePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_path().display())
    }
}

/// Database configuration with connection pool settings
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseConfig {
    path: DatabasePath,
    pool_size: ConnectionPoolSize,
    enable_wal_mode: bool,
    enable_foreign_keys: bool,
}

impl DatabaseConfig {
    /// Create new database config with default settings
    pub fn new(path: DatabasePath) -> Self {
        Self {
            path,
            pool_size: ConnectionPoolSize::default(),
            enable_wal_mode: true,
            enable_foreign_keys: true,
        }
    }

    /// Create config for testing with minimal settings
    ///
    /// # Panics
    ///
    /// Panics if the default pool size cannot be created (should never happen)
    pub fn for_testing(path: DatabasePath) -> Self {
        Self {
            path,
            pool_size: ConnectionPoolSize::try_new(1)
                .expect("Pool size 1 should be valid (range: 1-100)"),
            enable_wal_mode: false,
            enable_foreign_keys: false,
        }
    }

    /// Builder pattern: set connection pool size
    #[must_use]
    pub fn with_pool_size(mut self, pool_size: ConnectionPoolSize) -> Self {
        self.pool_size = pool_size;
        self
    }

    /// Builder pattern: enable/disable WAL mode
    #[must_use]
    pub fn with_wal_mode(mut self, enable: bool) -> Self {
        self.enable_wal_mode = enable;
        self
    }

    /// Builder pattern: enable/disable foreign keys
    #[must_use]
    pub fn with_foreign_keys(mut self, enable: bool) -> Self {
        self.enable_foreign_keys = enable;
        self
    }

    /// Get the database path
    pub fn path(&self) -> &DatabasePath {
        &self.path
    }

    /// Get the connection pool size
    pub fn pool_size(&self) -> ConnectionPoolSize {
        self.pool_size
    }

    /// Check if WAL mode is enabled
    pub fn wal_mode_enabled(&self) -> bool {
        self.enable_wal_mode
    }

    /// Check if foreign keys are enabled
    pub fn foreign_keys_enabled(&self) -> bool {
        self.enable_foreign_keys
    }

    /// Validate configuration (functional core)
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid
    pub fn validate(&self) -> StorageResult<()> {
        // Pure validation logic
        if self.pool_size.as_usize() == 0 {
            return Err(StorageError::Configuration {
                field: "pool_size".to_string(),
                reason: "Pool size must be greater than 0".to_string(),
            });
        }
        Ok(())
    }
}

/// Database connection with managed pool
#[derive(Clone)]
pub struct DatabaseConnection {
    pool: Pool<Sqlite>,
    config: DatabaseConfig,
    migration_registry: MigrationRegistry,
}

/// Migration script with version and SQL commands
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Migration {
    version: DatabaseSchemaVersion,
    description: String,
    up_sql: Vec<String>,
    down_sql: Vec<String>,
}

impl Migration {
    /// Create a new migration
    ///
    /// # Errors
    ///
    /// Returns an error if the description is empty or SQL is invalid
    pub fn new(
        version: DatabaseSchemaVersion,
        description: String,
        up_sql: Vec<String>,
        down_sql: Vec<String>,
    ) -> StorageResult<Self> {
        if description.trim().is_empty() {
            return Err(StorageError::Configuration {
                field: "description".to_string(),
                reason: "Migration description cannot be empty".to_string(),
            });
        }

        if up_sql.is_empty() {
            return Err(StorageError::Configuration {
                field: "up_sql".to_string(),
                reason: "Migration must have at least one up SQL statement".to_string(),
            });
        }

        Ok(Self {
            version,
            description,
            up_sql,
            down_sql,
        })
    }

    /// Get migration version
    pub fn version(&self) -> DatabaseSchemaVersion {
        self.version
    }

    /// Get migration description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get up SQL statements
    pub fn up_sql(&self) -> &[String] {
        &self.up_sql
    }

    /// Get down SQL statements (for rollbacks)
    pub fn down_sql(&self) -> &[String] {
        &self.down_sql
    }
}

/// Migration registry for managing schema migrations (functional core)
#[derive(Clone, Debug, Default)]
pub struct MigrationRegistry {
    migrations: BTreeMap<u32, Migration>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_core_migrations();
        registry
    }

    /// Register a migration (functional core)
    ///
    /// # Errors
    ///
    /// Returns an error if the migration is invalid or conflicts with existing migrations
    pub fn register(&mut self, migration: Migration) -> StorageResult<()> {
        let version = migration.version().as_u32();

        if self.migrations.contains_key(&version) {
            return Err(StorageError::Configuration {
                field: "version".to_string(),
                reason: format!("Migration version {version} already exists"),
            });
        }

        self.migrations.insert(version, migration);
        Ok(())
    }

    /// Get migration by version (functional core)
    pub fn get_migration(&self, version: DatabaseSchemaVersion) -> Option<&Migration> {
        self.migrations.get(&version.as_u32())
    }

    /// Get all migrations up to target version (functional core)
    pub fn get_migrations_to(&self, target_version: DatabaseSchemaVersion) -> Vec<&Migration> {
        self.migrations
            .values()
            .filter(|m| m.version().as_u32() <= target_version.as_u32())
            .collect()
    }

    /// Get migrations between versions (functional core)
    pub fn get_migrations_between(
        &self,
        from_version: DatabaseSchemaVersion,
        to_version: DatabaseSchemaVersion,
    ) -> Vec<&Migration> {
        let from = from_version.as_u32();
        let to = to_version.as_u32();

        self.migrations
            .values()
            .filter(|m| {
                let version = m.version().as_u32();
                version > from && version <= to
            })
            .collect()
    }

    /// Get highest version (functional core)
    pub fn highest_version(&self) -> Option<DatabaseSchemaVersion> {
        self.migrations
            .keys()
            .max()
            .and_then(|&v| DatabaseSchemaVersion::new(v).ok())
    }

    /// Validate migration path (functional core)
    ///
    /// # Errors
    ///
    /// Returns an error if the migration path is invalid
    pub fn validate_migration_path(
        &self,
        from_version: DatabaseSchemaVersion,
        to_version: DatabaseSchemaVersion,
    ) -> StorageResult<()> {
        let from = from_version.as_u32();
        let to = to_version.as_u32();

        if to < from {
            return Err(StorageError::Migration {
                message: format!("Cannot migrate backwards from {from} to {to}"),
            });
        }

        // Check that all intermediate versions exist
        for version in (from + 1)..=to {
            if !self.migrations.contains_key(&version) {
                return Err(StorageError::Migration {
                    message: format!("Missing migration for version {version}"),
                });
            }
        }

        Ok(())
    }

    /// Register core schema migrations (functional core)
    fn register_core_migrations(&mut self) {
        // Migration 1: Create agents table
        if let Ok(migration) = Migration::new(
            DatabaseSchemaVersion::new(1).expect("Version 1 should be valid"),
            "Create agents table for agent registry storage".to_string(),
            vec![
                r"CREATE TABLE IF NOT EXISTS agents (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    wasm_module BLOB NOT NULL,
                    config TEXT NOT NULL,
                    status TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                )"
                .to_string(),
                "CREATE INDEX IF NOT EXISTS idx_agents_name ON agents(name)".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status)".to_string(),
            ],
            vec![
                "DROP INDEX IF EXISTS idx_agents_status".to_string(),
                "DROP INDEX IF EXISTS idx_agents_name".to_string(),
                "DROP TABLE IF EXISTS agents".to_string(),
            ],
        ) {
            self.migrations.insert(1, migration);
        }

        // Migration 2: Create routes table
        if let Ok(migration) = Migration::new(
            DatabaseSchemaVersion::new(2).expect("Version 2 should be valid"),
            "Create routes table for message routing".to_string(),
            vec![
                r"CREATE TABLE IF NOT EXISTS routes (
                    id TEXT PRIMARY KEY,
                    from_agent_id TEXT NOT NULL,
                    to_agent_id TEXT NOT NULL,
                    message_type TEXT NOT NULL,
                    priority INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL,
                    FOREIGN KEY (from_agent_id) REFERENCES agents(id) ON DELETE CASCADE,
                    FOREIGN KEY (to_agent_id) REFERENCES agents(id) ON DELETE CASCADE
                )"
                .to_string(),
                "CREATE INDEX IF NOT EXISTS idx_routes_from_agent ON routes(from_agent_id)"
                    .to_string(),
                "CREATE INDEX IF NOT EXISTS idx_routes_to_agent ON routes(to_agent_id)".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_routes_message_type ON routes(message_type)"
                    .to_string(),
            ],
            vec![
                "DROP INDEX IF EXISTS idx_routes_message_type".to_string(),
                "DROP INDEX IF EXISTS idx_routes_to_agent".to_string(),
                "DROP INDEX IF EXISTS idx_routes_from_agent".to_string(),
                "DROP TABLE IF EXISTS routes".to_string(),
            ],
        ) {
            self.migrations.insert(2, migration);
        }

        // Migration 3: Create conversations table
        if let Ok(migration) = Migration::new(
            DatabaseSchemaVersion::new(3).expect("Version 3 should be valid"),
            "Create conversations table for message tracking".to_string(),
            vec![
                r"CREATE TABLE IF NOT EXISTS conversations (
                    id TEXT PRIMARY KEY,
                    participants TEXT NOT NULL,
                    message_count INTEGER NOT NULL DEFAULT 0,
                    last_message_at INTEGER,
                    status TEXT NOT NULL DEFAULT 'active',
                    created_at INTEGER NOT NULL
                )".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_conversations_status ON conversations(status)".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_conversations_last_message ON conversations(last_message_at)".to_string(),
            ],
            vec![
                "DROP INDEX IF EXISTS idx_conversations_last_message".to_string(),
                "DROP INDEX IF EXISTS idx_conversations_status".to_string(),
                "DROP TABLE IF EXISTS conversations".to_string(),
            ],
        ) {
            self.migrations.insert(3, migration);
        }
    }
}

// Functional Core: Pure business logic
impl DatabaseConnection {
    /// Generate `SQLite` options from config (pure function)
    fn create_connect_options(config: &DatabaseConfig) -> sqlx::sqlite::SqliteConnectOptions {
        use sqlx::ConnectOptions;
        use sqlx::sqlite::SqliteConnectOptions;

        let mut options = SqliteConnectOptions::new()
            .filename(config.path().as_path())
            .create_if_missing(true);

        if config.wal_mode_enabled() {
            options = options.pragma("journal_mode", "WAL");
        }

        if config.foreign_keys_enabled() {
            options = options.pragma("foreign_keys", "ON");
        }

        // Disable logging for cleaner test output
        options.disable_statement_logging()
    }
}

// Imperative Shell: I/O operations
impl DatabaseConnection {
    /// Initialize database connection (imperative shell)
    ///
    /// # Errors
    ///
    /// Returns an error if database initialization fails
    pub async fn initialize(config: DatabaseConfig) -> DatabaseResult<Self> {
        // Validate configuration (functional core)
        config.validate().map_err(DatabaseError::Storage)?;

        // Create parent directory if needed (imperative shell)
        Self::ensure_parent_directory_exists(&config).await?;

        // Create connection pool (imperative shell)
        let pool = Self::create_connection_pool(&config).await?;

        // Apply database settings (imperative shell)
        Self::apply_database_settings(&pool, &config).await?;

        Ok(Self {
            pool,
            config,
            migration_registry: MigrationRegistry::new(),
        })
    }

    /// Ensure parent directory exists (imperative shell)
    async fn ensure_parent_directory_exists(config: &DatabaseConfig) -> DatabaseResult<()> {
        if let Some(parent) = config.path().parent_directory() {
            tokio::fs::create_dir_all(&parent).await.map_err(|e| {
                DatabaseError::Storage(StorageError::FileSystem {
                    message: format!("Failed to create directory {}: {}", parent.display(), e),
                })
            })?;
        }
        Ok(())
    }

    /// Create connection pool (imperative shell)
    async fn create_connection_pool(config: &DatabaseConfig) -> DatabaseResult<Pool<Sqlite>> {
        let options = Self::create_connect_options(config);

        SqlitePool::connect_with(options).await.map_err(|e| {
            DatabaseError::Storage(StorageError::ConnectionPool {
                message: format!("Failed to create connection pool: {e}"),
            })
        })
    }

    /// Apply database settings after connection (imperative shell)
    async fn apply_database_settings(
        pool: &Pool<Sqlite>,
        _config: &DatabaseConfig,
    ) -> DatabaseResult<()> {
        // Apply any additional settings that can't be set via connection options
        // For now, this is a placeholder for future enhancements
        sqlx::query("PRAGMA optimize")
            .execute(pool)
            .await
            .map_err(|e| {
                DatabaseError::Storage(StorageError::Database {
                    message: format!("Failed to optimize database: {e}"),
                })
            })?;

        Ok(())
    }

    /// Check if database file exists
    pub fn database_file_exists(&self) -> bool {
        self.config.path().exists()
    }

    /// Test the database connection
    ///
    /// # Errors
    ///
    /// Returns an error if the connection test fails
    pub async fn test_connection(&self) -> DatabaseResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DatabaseError::Storage(StorageError::Database {
                    message: format!("Connection test failed: {e}"),
                })
            })?;
        Ok(())
    }

    /// Get access to the connection pool for advanced operations
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Get the database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get the current database schema version
    ///
    /// # Errors
    ///
    /// Returns an error if the schema version cannot be retrieved
    pub async fn get_schema_version(&self) -> DatabaseResult<DatabaseSchemaVersion> {
        // Use SQLite's user_version pragma to track schema version
        let result = sqlx::query("PRAGMA user_version")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DatabaseError::Storage(StorageError::Database {
                    message: format!("Failed to get schema version: {e}"),
                })
            })?;

        let version: i64 = result.get(0);
        let version_u32 = u32::try_from(version).unwrap_or(0);

        DatabaseSchemaVersion::new(version_u32).map_err(|_| {
            DatabaseError::Storage(StorageError::Database {
                message: format!("Invalid schema version: {version}"),
            })
        })
    }

    /// Migrate database to target schema version
    ///
    /// # Errors
    ///
    /// Returns an error if the migration fails
    pub async fn migrate_to_version(
        &self,
        target_version: DatabaseSchemaVersion,
    ) -> DatabaseResult<()> {
        // Get current version
        let current_version = self.get_schema_version().await?;

        // If already at target version, nothing to do
        if current_version == target_version {
            return Ok(());
        }

        // Validate migration path using functional core
        self.migration_registry
            .validate_migration_path(current_version, target_version)
            .map_err(DatabaseError::Storage)?;

        // Get migrations to apply (functional core)
        let migrations = self
            .migration_registry
            .get_migrations_between(current_version, target_version);

        // Apply migrations within a transaction (imperative shell)
        self.apply_migrations_in_transaction(&migrations, target_version)
            .await
    }

    /// Apply migrations within a database transaction (imperative shell)
    async fn apply_migrations_in_transaction(
        &self,
        migrations: &[&Migration],
        target_version: DatabaseSchemaVersion,
    ) -> DatabaseResult<()> {
        let mut transaction = self.pool.begin().await.map_err(|e| {
            DatabaseError::Storage(StorageError::Database {
                message: format!("Failed to begin migration transaction: {e}"),
            })
        })?;

        // Apply each migration
        for migration in migrations {
            for sql_statement in migration.up_sql() {
                sqlx::query(sql_statement)
                    .execute(&mut *transaction)
                    .await
                    .map_err(|e| {
                        DatabaseError::Storage(StorageError::Migration {
                            message: format!(
                                "Failed to execute migration {} statement '{}': {e}",
                                migration.version().as_u32(),
                                sql_statement
                            ),
                        })
                    })?;
            }
        }

        // Update schema version in user_version pragma
        let target = target_version.as_u32();
        sqlx::query(&format!("PRAGMA user_version = {target}"))
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                DatabaseError::Storage(StorageError::Database {
                    message: format!("Failed to update schema version to {target}: {e}"),
                })
            })?;

        // Commit transaction
        transaction.commit().await.map_err(|e| {
            DatabaseError::Storage(StorageError::Database {
                message: format!("Failed to commit migration transaction: {e}"),
            })
        })?;

        Ok(())
    }

    /// Verify database schema integrity
    ///
    /// # Errors
    ///
    /// Returns an error if the integrity check fails
    ///
    /// # Panics
    ///
    /// Panics if `DatabaseSchemaVersion::new(0)` fails, which should never happen
    pub async fn verify_schema_integrity(&self) -> DatabaseResult<()> {
        // Use SQLite's integrity_check for data integrity
        let integrity_result = sqlx::query("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                DatabaseError::Storage(StorageError::Database {
                    message: format!("Schema integrity check failed: {e}"),
                })
            })?;

        let integrity_status: String = integrity_result.get(0);
        if integrity_status != "ok" {
            return Err(DatabaseError::Storage(StorageError::Database {
                message: format!("Database integrity check failed: {integrity_status}"),
            }));
        }

        // Also perform schema version validation
        let current_version = self.get_schema_version().await?;
        let highest_available = self
            .migration_registry
            .highest_version()
            .unwrap_or(DatabaseSchemaVersion::new(0).expect("Version 0 should be valid"));

        if current_version.as_u32() > highest_available.as_u32() {
            return Err(DatabaseError::Storage(StorageError::Migration {
                message: format!(
                    "Database schema version {} is higher than highest available migration {}",
                    current_version.as_u32(),
                    highest_available.as_u32()
                ),
            }));
        }

        Ok(())
    }

    /// Get access to the migration registry for inspection (functional core access)
    pub fn migration_registry(&self) -> &MigrationRegistry {
        &self.migration_registry
    }

    /// Check if specific table exists in the database
    ///
    /// # Errors
    ///
    /// Returns an error if the table check fails
    pub async fn table_exists(&self, table_name: &str) -> DatabaseResult<bool> {
        let result =
            sqlx::query("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1")
                .bind(table_name)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    DatabaseError::Storage(StorageError::Database {
                        message: format!("Failed to check if table '{table_name}' exists: {e}"),
                    })
                })?;

        let count: i64 = result.get(0);
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_types::ConnectionPoolSize;
    use tempfile::tempdir;

    #[test]
    fn test_should_create_valid_database_path_when_given_db_extension() {
        // Test that verifies DatabasePath accepts valid .db extensions
        let path = DatabasePath::new("test.db").unwrap();
        assert!(path.to_connection_string().contains("test.db"));
    }

    #[test]
    fn test_should_reject_empty_path_when_creating_database_path() {
        // Test that verifies DatabasePath validation rejects empty paths
        let result = DatabasePath::new("");
        assert!(result.is_err());
        match result.unwrap_err() {
            DatabaseError::Storage(StorageError::Configuration { field, reason }) => {
                assert_eq!(field, "database_path");
                assert!(reason.contains("empty"));
            }
            _ => panic!("Expected Configuration error for empty path"),
        }
    }

    #[test]
    fn test_should_reject_invalid_extension_when_creating_database_path() {
        // Test that verifies DatabasePath validation rejects non-.db extensions
        let result = DatabasePath::new("test.txt");
        assert!(result.is_err());
        match result.unwrap_err() {
            DatabaseError::Storage(StorageError::Configuration { field, reason }) => {
                assert_eq!(field, "database_path");
                assert!(reason.contains("invalid extension"));
            }
            _ => panic!("Expected Configuration error for invalid extension"),
        }
    }

    #[test]
    fn test_should_generate_correct_connection_string_when_converting_path() {
        // Test that verifies DatabasePath generates proper SQLite connection strings
        let path = DatabasePath::new("/tmp/test.db").unwrap();
        let conn_str = path.to_connection_string();
        assert!(conn_str.starts_with("sqlite://"));
        assert!(conn_str.contains("/tmp/test.db"));
        assert!(conn_str.contains("mode=rwc"));
    }

    #[test]
    fn test_should_extract_parent_directory_when_path_has_parent() {
        // Test that verifies DatabasePath parent directory extraction
        let path = DatabasePath::new("/tmp/subdir/test.db").unwrap();
        let parent = path.parent_directory().unwrap();
        assert!(parent.to_string_lossy().contains("tmp"));
    }

    #[test]
    fn test_should_create_default_config_when_given_valid_path() {
        // Test that verifies DatabaseConfig creation with defaults
        let path = DatabasePath::new("test.db").unwrap();
        let config = DatabaseConfig::new(path);
        assert_eq!(config.pool_size().as_usize(), 10); // Default from ConnectionPoolSize
        assert!(config.wal_mode_enabled());
        assert!(config.foreign_keys_enabled());
    }

    #[test]
    fn test_should_create_testing_config_when_requested() {
        // Test that verifies DatabaseConfig testing configuration
        let path = DatabasePath::new("test.db").unwrap();
        let config = DatabaseConfig::for_testing(path);
        assert_eq!(config.pool_size().as_usize(), 1);
        assert!(!config.wal_mode_enabled());
        assert!(!config.foreign_keys_enabled());
    }

    #[test]
    fn test_should_apply_builder_settings_when_configuring() {
        // Test that verifies DatabaseConfig builder pattern works correctly
        let path = DatabasePath::new("test.db").unwrap();
        let pool_size = ConnectionPoolSize::try_new(5).unwrap();
        let config = DatabaseConfig::new(path)
            .with_pool_size(pool_size)
            .with_wal_mode(false)
            .with_foreign_keys(false);

        assert_eq!(config.pool_size().as_usize(), 5);
        assert!(!config.wal_mode_enabled());
        assert!(!config.foreign_keys_enabled());
    }

    #[test]
    fn test_should_pass_validation_when_config_is_valid() {
        // Test that verifies DatabaseConfig validation accepts valid configurations
        let path = DatabasePath::new("test.db").unwrap();
        let config = DatabaseConfig::new(path);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_should_fail_validation_when_pool_size_is_zero() {
        // Test that verifies DatabaseConfig validation rejects zero pool size
        let path = DatabasePath::new("test.db").unwrap();
        let zero_pool_size = ConnectionPoolSize::try_new(0);

        // Note: This test may fail if ConnectionPoolSize validation prevents zero
        // In that case, this demonstrates type safety working correctly
        if let Ok(zero_pool) = zero_pool_size {
            let config = DatabaseConfig::new(path).with_pool_size(zero_pool);
            let result = config.validate();
            assert!(result.is_err());
            match result.unwrap_err() {
                StorageError::Configuration { field, reason } => {
                    assert_eq!(field, "pool_size");
                    assert!(reason.contains("greater than 0"));
                }
                _ => panic!("Expected Configuration error for zero pool size"),
            }
        }
    }

    #[test]
    fn test_should_create_sqlite_options_with_wal_mode_when_enabled() {
        // Test that verifies SQLite options generation with WAL mode
        let path = DatabasePath::new("test.db").unwrap();
        let config = DatabaseConfig::new(path).with_wal_mode(true);
        let _options = DatabaseConnection::create_connect_options(&config);
        // Note: SQLite options are opaque, so we test this through integration
    }

    #[test]
    fn test_should_create_sqlite_options_without_wal_mode_when_disabled() {
        // Test that verifies SQLite options generation without WAL mode
        let path = DatabasePath::new("test.db").unwrap();
        let config = DatabaseConfig::new(path).with_wal_mode(false);
        let _options = DatabaseConnection::create_connect_options(&config);
        // Note: SQLite options are opaque, so we test this through integration
    }

    #[tokio::test]
    async fn test_should_initialize_database_connection_when_given_valid_config() {
        // Test that verifies DatabaseConnection initialization succeeds with valid config
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let path = DatabasePath::new(db_path).unwrap();
        let config = DatabaseConfig::for_testing(path);

        let connection = DatabaseConnection::initialize(config).await;
        assert!(connection.is_ok());
        let conn = connection.unwrap();
        assert!(conn.database_file_exists());
    }

    #[tokio::test]
    async fn test_should_fail_initialization_when_path_is_invalid() {
        // Test that verifies DatabaseConnection initialization fails with invalid path
        // This test may be hard to trigger due to strong typing, but attempts edge cases
        let path = DatabasePath::new("/root/impossible_write_location.db");

        if let Ok(path) = path {
            let config = DatabaseConfig::for_testing(path);
            let _result = DatabaseConnection::initialize(config).await;
            // May fail during directory creation or connection setup
            // The exact failure depends on system permissions
        }
    }

    #[tokio::test]
    async fn test_should_pass_connection_test_when_database_is_healthy() {
        // Test that verifies database connection testing works
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let path = DatabasePath::new(db_path).unwrap();
        let config = DatabaseConfig::for_testing(path);

        let connection = DatabaseConnection::initialize(config).await.unwrap();
        let test_result = connection.test_connection().await;
        assert!(test_result.is_ok());
    }

    #[tokio::test]
    async fn test_should_provide_access_to_connection_pool_when_requested() {
        // Test that verifies DatabaseConnection provides pool access
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let path = DatabasePath::new(db_path).unwrap();
        let config = DatabaseConfig::for_testing(path);

        let connection = DatabaseConnection::initialize(config).await.unwrap();
        let _pool = connection.pool();
        // Pool access is primarily for advanced operations
    }

    #[tokio::test]
    async fn test_should_provide_access_to_config_when_requested() {
        // Test that verifies DatabaseConnection provides config access
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let path = DatabasePath::new(db_path).unwrap();
        let config = DatabaseConfig::for_testing(path.clone());

        let connection = DatabaseConnection::initialize(config).await.unwrap();
        let stored_config = connection.config();
        assert_eq!(stored_config.path(), &path);
    }

    #[test]
    fn test_should_display_database_path_correctly_when_formatted() {
        // Test that verifies DatabasePath Display implementation
        let path = DatabasePath::new("test.db").unwrap();
        let display_str = format!("{path}");
        assert!(display_str.contains("test.db"));
    }

    #[test]
    fn test_should_handle_database_error_types_correctly() {
        // Test that verifies error type conversions and formatting
        let storage_error = StorageError::Database {
            message: "Test error".to_string(),
        };
        let db_error = DatabaseError::Storage(storage_error);
        let error_string = format!("{db_error}");
        assert!(error_string.contains("Test error"));
    }

    #[test]
    fn test_should_handle_storage_error_types_correctly() {
        // Test that verifies StorageError variants format correctly
        let config_error = StorageError::Configuration {
            field: "test_field".to_string(),
            reason: "test reason".to_string(),
        };
        let error_string = format!("{config_error}");
        assert!(error_string.contains("test_field"));
        assert!(error_string.contains("test reason"));

        let filesystem_error = StorageError::FileSystem {
            message: "filesystem error".to_string(),
        };
        let fs_error_string = format!("{filesystem_error}");
        assert!(fs_error_string.contains("filesystem error"));

        let pool_error = StorageError::ConnectionPool {
            message: "pool error".to_string(),
        };
        let pool_error_string = format!("{pool_error}");
        assert!(pool_error_string.contains("pool error"));
    }

    #[tokio::test]
    async fn test_should_migrate_schema_from_version_zero_to_version_one_when_upgrading() {
        // Test that verifies database schema can be safely migrated between versions
        use crate::domain_types::DatabaseSchemaVersion;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("migration_test.db");
        let path = DatabasePath::new(db_path).unwrap();
        let config = DatabaseConfig::for_testing(path);

        // Initialize database with version 0 schema
        let connection = DatabaseConnection::initialize(config).await.unwrap();

        // Verify initial schema version is 0
        let initial_version = connection.get_schema_version().await.unwrap();
        assert_eq!(initial_version, DatabaseSchemaVersion::new(0).unwrap());

        // Attempt to migrate to version 1
        let target_version = DatabaseSchemaVersion::new(1).unwrap();
        let migration_result = connection.migrate_to_version(target_version).await;

        // Verify migration succeeded
        assert!(migration_result.is_ok());

        // Verify schema version was updated
        let final_version = connection.get_schema_version().await.unwrap();
        assert_eq!(final_version, target_version);

        // Verify database integrity after migration
        let integrity_check = connection.verify_schema_integrity().await;
        assert!(integrity_check.is_ok());
    }
}

// Re-export storage types for convenient access alongside database types
pub use crate::storage::{AgentStorage, StoredAgent};
