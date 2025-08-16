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
    sanitize(with = |path: PathBuf| path.canonicalize().unwrap_or(path)),
    validate(predicate = |path| !path.as_os_str().is_empty() && path.extension().is_none_or(|ext| ext == "db")),
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
            pool_size: ConnectionPoolSize::try_new(1).expect("Pool size 1 should be valid"),
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
