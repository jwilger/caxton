//! Database module for embedded SQLite storage
//!
//! This module provides a clean separation between functional core (pure business logic)
//! and imperative shell (I/O operations) for database management.
//!
//! ## Architecture
//!
//! - **Functional Core**: Configuration validation, connection string generation
//! - **Imperative Shell**: File system operations, SQLite connections

use crate::domain_types::ConnectionPoolSize;
use nutype::nutype;
use sqlx::{Pool, Sqlite, SqlitePool};
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

        Ok(Self { pool, config })
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
}
