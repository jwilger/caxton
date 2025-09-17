//! Domain types for Caxton
//!
//! This module defines the core domain types following Domain-Driven Design principles
//! and the "Parse, Don't Validate" philosophy. All types make illegal states unrepresentable.

use serde::Deserialize;
use std::path::PathBuf;

/// Server configuration domain types
pub mod config {
    use super::{Deserialize, PathBuf};
    use ::config::{Config, File, FileFormat};

    /// A validated port number (1-65535)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
    #[serde(try_from = "u16")]
    pub struct Port(u16);

    impl Port {
        /// Create a new Port with validation
        ///
        /// # Errors
        ///
        /// Returns an error if the port value is 0 (invalid).
        pub fn new(value: u16) -> Result<Self, String> {
            if value == 0 {
                return Err(format!("Port must be between 1 and 65535, got {value}"));
            }
            Ok(Port(value))
        }

        /// Get the inner value
        #[must_use]
        pub fn into_inner(self) -> u16 {
            self.0
        }
    }

    impl TryFrom<u16> for Port {
        type Error = String;

        fn try_from(value: u16) -> Result<Self, Self::Error> {
            Port::new(value)
        }
    }

    impl std::fmt::Display for Port {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Default for Port {
        fn default() -> Self {
            // Safe because 8080 is within valid range
            Port::new(8080).unwrap()
        }
    }

    /// Server configuration with validated fields
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
    pub struct ServerConfig {
        /// Port to bind the server to
        #[serde(default)]
        pub port: Port,
    }

    /// Complete application configuration
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
    pub struct AppConfig {
        /// Server configuration section
        #[serde(default)]
        pub server: ServerConfig,
    }

    /// Source of configuration values
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ConfigSource {
        /// Configuration from file
        File(PathBuf),
        /// Configuration from command-line arguments
        Cli,
        /// Default configuration
        Default,
    }

    /// Domain-specific configuration errors
    #[derive(Debug)]
    pub enum ConfigError {
        /// Configuration file not found
        FileNotFound {
            /// Path to the missing file
            path: PathBuf,
        },

        /// Failed to read configuration file
        ReadError {
            /// Path to the file that couldn't be read
            path: PathBuf,
            /// The underlying IO error
            source: std::io::Error,
        },

        /// Failed to parse TOML configuration
        ParseError {
            /// Error message describing the parse failure
            message: String,
        },

        /// Invalid configuration values
        ValidationError {
            /// Error message describing the validation failure
            message: String,
        },
    }

    /// Result type for configuration operations
    pub type ConfigResult<T> = Result<T, ConfigError>;

    impl std::fmt::Display for ConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ConfigError::FileNotFound { path } => {
                    write!(f, "Configuration file not found: {}", path.display())
                }
                ConfigError::ReadError { path, source } => write!(
                    f,
                    "Failed to read configuration file {}: {}",
                    path.display(),
                    source
                ),
                ConfigError::ParseError { message } => {
                    write!(f, "Invalid TOML configuration: {message}")
                }
                ConfigError::ValidationError { message } => {
                    write!(f, "Invalid configuration: {message}")
                }
            }
        }
    }

    impl std::error::Error for ConfigError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                ConfigError::ReadError { source, .. } => Some(source),
                _ => None,
            }
        }
    }

    /// Load configuration from file, with defaults for missing values
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read or parsed.
    pub fn load_config(path: Option<PathBuf>) -> ConfigResult<AppConfig> {
        let config_path = path.unwrap_or_else(|| PathBuf::from("caxton.toml"));

        if !config_path.exists() {
            // Return default configuration if file doesn't exist
            return Ok(AppConfig::default());
        }

        let content =
            std::fs::read_to_string(&config_path).map_err(|source| ConfigError::ReadError {
                path: config_path.clone(),
                source,
            })?;

        parse_toml_config(&content)
    }

    /// Merge configurations with precedence: CLI > File > Default
    #[must_use]
    pub fn merge_configs(
        _default: AppConfig,
        _file: Option<AppConfig>,
        _cli: Option<AppConfig>,
    ) -> AppConfig {
        unimplemented!("Configuration merging will be implemented by green-implementer")
    }

    /// Parse configuration from TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML string cannot be parsed or deserialized.
    pub fn parse_toml_config(content: &str) -> ConfigResult<AppConfig> {
        let settings = Config::builder()
            .add_source(File::from_str(content, FileFormat::Toml))
            .build()
            .map_err(|e| ConfigError::ParseError {
                message: e.to_string(),
            })?;

        settings
            .try_deserialize()
            .map_err(|e| ConfigError::ParseError {
                message: e.to_string(),
            })
    }
}

/// Server runtime domain types
pub mod server {
    use crate::domain::config::Port;

    /// Server state with phantom types for compile-time state machine
    #[derive(Debug)]
    pub struct Server<State> {
        port: Port,
        _state: std::marker::PhantomData<State>,
    }

    /// Server is not yet started
    #[derive(Debug)]
    pub struct NotStarted;

    /// Server is running
    #[derive(Debug)]
    pub struct Running;

    /// Server has been stopped
    #[derive(Debug)]
    pub struct Stopped;

    impl Server<NotStarted> {
        /// Create a new server instance
        #[must_use]
        pub fn new(port: Port) -> Self {
            Server {
                port,
                _state: std::marker::PhantomData,
            }
        }

        /// Start the server (state transition: `NotStarted` -> `Running`)
        ///
        /// # Errors
        ///
        /// Returns an error if the server fails to start or bind to the port.
        pub fn start(self) -> Result<Server<Running>, ServerError> {
            unimplemented!("Server starting will be implemented by green-implementer")
        }
    }

    impl Server<Running> {
        /// Get the port the server is running on
        #[must_use]
        pub fn port(&self) -> Port {
            self.port
        }

        /// Stop the server (state transition: `Running` -> `Stopped`)
        #[must_use]
        pub fn stop(self) -> Server<Stopped> {
            unimplemented!("Server stopping will be implemented by green-implementer")
        }
    }

    /// Domain-specific server errors
    #[derive(Debug)]
    pub enum ServerError {
        /// Failed to bind to port
        BindError {
            /// The port that failed to bind
            port: Port,
            /// The underlying IO error
            source: std::io::Error,
        },

        /// Server startup failed
        StartupError {
            /// Error message describing the startup failure
            message: String,
        },

        /// Server is already running
        AlreadyRunning {
            /// The port the server is already running on
            port: Port,
        },
    }

    impl std::fmt::Display for ServerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ServerError::BindError { port, source } => {
                    write!(f, "Failed to bind to port {port}: {source}")
                }
                ServerError::StartupError { message } => {
                    write!(f, "Server startup failed: {message}")
                }
                ServerError::AlreadyRunning { port } => {
                    write!(f, "Server is already running on port {port}")
                }
            }
        }
    }

    impl std::error::Error for ServerError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                ServerError::BindError { source, .. } => Some(source),
                _ => None,
            }
        }
    }
}

/// Agent configuration and management domain types
pub mod agent {
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    /// A validated agent name following kebab-case pattern
    /// Must match: ^[a-z][a-z0-9-]*[a-z0-9]$
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
    #[serde(try_from = "String")]
    pub struct AgentName(String);

    impl AgentName {
        /// Create a new `AgentName` with validation
        ///
        /// # Errors
        ///
        /// Returns an error if the name doesn't match the required pattern.
        pub fn new(value: String) -> Result<Self, String> {
            // Validate kebab-case pattern
            if value.is_empty() {
                return Err("Agent name cannot be empty".to_string());
            }

            let chars: Vec<char> = value.chars().collect();

            // Must start with lowercase letter
            if !chars[0].is_ascii_lowercase() {
                return Err("Agent name must start with a lowercase letter".to_string());
            }

            // Must end with lowercase letter or digit
            let last = chars[chars.len() - 1];
            if !last.is_ascii_lowercase() && !last.is_ascii_digit() {
                return Err("Agent name must end with a lowercase letter or digit".to_string());
            }

            // Middle characters must be lowercase, digit, or hyphen
            for c in &chars {
                if !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-' {
                    return Err(format!("Agent name contains invalid character '{c}'"));
                }
            }

            // No consecutive hyphens
            if value.contains("--") {
                return Err("Agent name cannot contain consecutive hyphens".to_string());
            }

            Ok(AgentName(value))
        }

        /// Get the inner value
        #[must_use]
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl TryFrom<String> for AgentName {
        type Error = String;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            AgentName::new(value)
        }
    }

    impl std::fmt::Display for AgentName {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    /// Semantic version for agent versioning
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(try_from = "String")]
    pub struct Version(String);

    impl Version {
        /// Create a new Version with validation
        ///
        /// # Errors
        ///
        /// Returns an error if the version doesn't follow semver format.
        pub fn new(value: String) -> Result<Self, String> {
            // Basic semver validation (MAJOR.MINOR.PATCH)
            let parts: Vec<&str> = value.split('.').collect();
            if parts.len() != 3 {
                return Err("Version must be in MAJOR.MINOR.PATCH format".to_string());
            }

            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    return Err(format!("Version part {} cannot be empty", i + 1));
                }
                if part.parse::<u32>().is_err() {
                    return Err(format!("Version part '{part}' must be a number"));
                }
            }

            Ok(Version(value))
        }

        /// Get the inner value
        #[must_use]
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl TryFrom<String> for Version {
        type Error = String;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Version::new(value)
        }
    }

    impl std::fmt::Display for Version {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    /// A validated capability identifier
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
    #[serde(try_from = "String")]
    pub struct Capability(String);

    impl Capability {
        /// Create a new Capability with validation
        ///
        /// # Errors
        ///
        /// Returns an error if the capability name is invalid.
        pub fn new(value: String) -> Result<Self, String> {
            if value.is_empty() {
                return Err("Capability cannot be empty".to_string());
            }

            // Capability must be snake_case or contain valid characters
            for c in value.chars() {
                if !c.is_ascii_alphanumeric() && c != '_' {
                    return Err(format!("Capability contains invalid character '{c}'"));
                }
            }

            Ok(Capability(value))
        }

        /// Get the inner value
        #[must_use]
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl TryFrom<String> for Capability {
        type Error = String;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Capability::new(value)
        }
    }

    /// A validated tool identifier (MCP tool reference)
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
    #[serde(try_from = "String")]
    pub struct ToolName(String);

    impl ToolName {
        /// Create a new `ToolName` with validation
        ///
        /// # Errors
        ///
        /// Returns an error if the tool name is invalid.
        pub fn new(value: String) -> Result<Self, String> {
            if value.is_empty() {
                return Err("Tool name cannot be empty".to_string());
            }

            // Tool names can contain alphanumeric, underscore, double underscore (for MCP)
            // Example: mcp__git__git_diff
            for c in value.chars() {
                if !c.is_ascii_alphanumeric() && c != '_' {
                    return Err(format!("Tool name contains invalid character '{c}'"));
                }
            }

            Ok(ToolName(value))
        }

        /// Get the inner value
        #[must_use]
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl TryFrom<String> for ToolName {
        type Error = String;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            ToolName::new(value)
        }
    }

    /// LLM provider type
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum LlmProvider {
        /// `OpenAI` provider (GPT models)
        OpenAI,
        /// Anthropic provider (Claude models)
        Anthropic,
        /// Local provider (self-hosted models)
        Local,
    }

    /// LLM configuration for agent
    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct LlmConfig {
        /// LLM provider
        pub provider: LlmProvider,
        /// Model identifier
        pub model: String,
        /// Temperature (0.0-2.0)
        #[serde(default = "default_temperature")]
        pub temperature: f32,
        /// Max tokens for response
        #[serde(default)]
        pub max_tokens: Option<u32>,
        /// Top-p nucleus sampling
        #[serde(default)]
        pub top_p: Option<f32>,
        /// Frequency penalty
        #[serde(default)]
        pub frequency_penalty: Option<f32>,
        /// Presence penalty
        #[serde(default)]
        pub presence_penalty: Option<f32>,
    }

    fn default_temperature() -> f32 {
        0.7
    }

    /// Memory configuration for agent
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub struct MemoryConfig {
        /// Whether memory is enabled
        #[serde(default = "default_true")]
        pub enabled: bool,
        /// Context window size
        #[serde(default = "default_context_window")]
        pub context_window: u32,
        /// Memory scope
        #[serde(default)]
        pub scope: MemoryScope,
        /// Retention period
        #[serde(default)]
        pub retention_period: Option<String>,
        /// Max entries to store
        #[serde(default)]
        pub max_entries: Option<u32>,
    }

    fn default_true() -> bool {
        true
    }

    fn default_context_window() -> u32 {
        4000
    }

    /// Memory scope for agent
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
    #[serde(rename_all = "lowercase")]
    pub enum MemoryScope {
        /// Agent-specific memory
        #[default]
        Agent,
        /// Conversation-specific memory
        Conversation,
        /// Global memory shared across agents
        Global,
    }

    /// Conversation configuration
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub struct ConversationConfig {
        /// Maximum conversation turns
        #[serde(default = "default_max_turns")]
        pub max_turns: u32,
        /// Timeout in seconds
        #[serde(default = "default_timeout")]
        pub timeout_seconds: u32,
    }

    fn default_max_turns() -> u32 {
        50
    }

    fn default_timeout() -> u32 {
        300
    }

    /// Tools configuration section
    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub struct ToolsConfig {
        /// List of available tools
        pub available: Vec<ToolName>,
    }

    /// Complete agent configuration
    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub struct AgentConfig {
        /// Agent name
        pub name: AgentName,
        /// Agent version
        pub version: Version,
        /// Agent description
        #[serde(default)]
        pub description: Option<String>,
        /// Agent capabilities
        pub capabilities: Vec<Capability>,
        /// Tool configuration
        pub tools: ToolsConfig,
        /// LLM configuration
        #[serde(default)]
        pub llm: Option<LlmConfig>,
        /// Memory configuration
        #[serde(default)]
        pub memory: Option<MemoryConfig>,
        /// Conversation configuration
        #[serde(default)]
        pub conversation: Option<ConversationConfig>,
        /// System prompt
        pub system_prompt: String,
        /// User prompt template
        pub user_prompt_template: String,
        /// Agent documentation
        #[serde(default)]
        pub documentation: Option<String>,
        /// Additional parameters
        #[serde(default)]
        pub parameters: HashMap<String, toml::Value>,
    }

    impl AgentConfig {
        /// Get the agent name
        #[must_use]
        pub fn name(&self) -> &AgentName {
            &self.name
        }

        /// Get the agent version
        #[must_use]
        pub fn version(&self) -> &Version {
            &self.version
        }

        /// Check if agent has a specific capability
        #[must_use]
        pub fn has_capability(&self, capability: &str) -> bool {
            self.capabilities.iter().any(|c| c.as_str() == capability)
        }

        /// Check if agent has access to a specific tool
        #[must_use]
        pub fn has_tool(&self, tool: &str) -> bool {
            self.tools.available.iter().any(|t| t.as_str() == tool)
        }
    }

    /// Common TOML error types that can provide helpful suggestions
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TomlErrorKind {
        /// Unclosed string literal
        UnclosedString,
        /// Invalid key format
        InvalidKey,
        /// Missing value for key
        MissingValue,
        /// Invalid table syntax
        InvalidTable,
        /// Type mismatch
        TypeMismatch {
            /// The expected type
            expected: String,
            /// The type that was found
            found: String,
        },
        /// General syntax error
        SyntaxError,
    }

    impl TomlErrorKind {
        /// Get a helpful suggestion for fixing this error
        #[must_use]
        pub fn suggestion(&self) -> String {
            match self {
                TomlErrorKind::UnclosedString => {
                    "Suggestion: Ensure all string literals are properly closed with matching quotes. Use triple quotes (''') for multi-line strings.".to_string()
                }
                TomlErrorKind::InvalidKey => {
                    "Suggestion: Keys must be bare (alphanumeric+underscore), quoted strings, or dotted. Check for special characters or spaces.".to_string()
                }
                TomlErrorKind::MissingValue => {
                    "Suggestion: Every key must have a value. Add = followed by a valid TOML value (string, number, boolean, array, or table).".to_string()
                }
                TomlErrorKind::InvalidTable => {
                    "Suggestion: Table headers must be in [brackets] or [[double brackets]] for arrays. Ensure no syntax errors in table definition.".to_string()
                }
                TomlErrorKind::TypeMismatch { expected, found } => {
                    format!("Suggestion: Expected {expected} but found {found}. Check the value type matches the field requirements.")
                }
                TomlErrorKind::SyntaxError => {
                    "Suggestion: Check for common syntax issues: unclosed quotes, missing commas in arrays, or invalid escape sequences.".to_string()
                }
            }
        }

        /// Detect error kind from TOML parse error message
        #[must_use]
        pub fn from_parse_error(error_msg: &str) -> Self {
            let lower = error_msg.to_lowercase();
            if lower.contains("unterminated")
                || lower.contains("unclosed")
                || lower.contains("eof while parsing")
            {
                TomlErrorKind::UnclosedString
            } else if lower.contains("invalid key") || lower.contains("bare key") {
                TomlErrorKind::InvalidKey
            } else if lower.contains("missing value") || lower.contains("expected value") {
                TomlErrorKind::MissingValue
            } else if lower.contains("table") || lower.contains("header") {
                TomlErrorKind::InvalidTable
            } else if lower.contains("expected") && lower.contains("found") {
                // Try to extract types from error message
                TomlErrorKind::TypeMismatch {
                    expected: "the correct type".to_string(),
                    found: "an incorrect type".to_string(),
                }
            } else {
                TomlErrorKind::SyntaxError
            }
        }
    }

    /// Agent configuration errors
    #[derive(Debug)]
    pub enum AgentConfigError {
        /// TOML parsing failed with structured information
        ParseError {
            /// Error message
            message: String,
            /// Line number where error occurred (if available)
            line: Option<usize>,
            /// Column number where error occurred (if available)
            column: Option<usize>,
            /// Type of TOML error for generating suggestions
            kind: TomlErrorKind,
        },
        /// Invalid agent name
        InvalidName {
            /// The invalid name
            name: String,
            /// Reason for invalidity
            reason: String,
        },
        /// Invalid version
        InvalidVersion {
            /// The invalid version
            version: String,
            /// Reason for invalidity
            reason: String,
        },
        /// Missing required field
        MissingField {
            /// Name of the missing field
            field: String,
        },
        /// Invalid capability
        InvalidCapability {
            /// The invalid capability
            capability: String,
            /// Reason for invalidity
            reason: String,
        },
        /// Invalid tool reference
        InvalidTool {
            /// The invalid tool
            tool: String,
            /// Reason for invalidity
            reason: String,
        },
    }

    impl std::fmt::Display for AgentConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AgentConfigError::ParseError {
                    message,
                    line,
                    column,
                    kind,
                } => {
                    let location = match (line, column) {
                        (Some(l), Some(c)) => format!(" at line {l}, column {c}"),
                        (Some(l), None) => format!(" at line {l}"),
                        _ => String::new(),
                    };
                    write!(
                        f,
                        "Failed to parse agent configuration{location}: {message}. {}",
                        kind.suggestion()
                    )
                }
                AgentConfigError::InvalidName { name, reason } => {
                    write!(f, "Invalid agent name '{name}': {reason}")
                }
                AgentConfigError::InvalidVersion { version, reason } => {
                    write!(f, "Invalid version '{version}': {reason}")
                }
                AgentConfigError::MissingField { field } => {
                    write!(f, "Missing required field: {field}")
                }
                AgentConfigError::InvalidCapability { capability, reason } => {
                    write!(f, "Invalid capability '{capability}': {reason}")
                }
                AgentConfigError::InvalidTool { tool, reason } => {
                    write!(f, "Invalid tool '{tool}': {reason}")
                }
            }
        }
    }

    impl std::error::Error for AgentConfigError {}

    /// Result type for agent operations
    pub type AgentResult<T> = Result<T, AgentConfigError>;

    /// Load agent configuration from TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML cannot be parsed or validated.
    pub fn load_agent_config_from_toml(content: &str) -> AgentResult<AgentConfig> {
        // Trim leading/trailing whitespace from TOML content
        let trimmed_content = content.trim();
        toml::from_str(trimmed_content).map_err(|e| {
            let error_str = e.to_string();

            // Extract line and column from error message if available
            // TOML errors typically include "at line X, column Y" in the message
            let mut line = None;
            let mut column = None;

            if let Some(pos) = error_str.find(" at line ") {
                let substr = &error_str[pos + 9..];
                if let Some(comma_pos) = substr.find(", column ") {
                    // Parse line number
                    if let Ok(l) = substr[..comma_pos].parse::<usize>() {
                        line = Some(l);
                    }
                    // Parse column number
                    let col_substr = &substr[comma_pos + 9..];
                    if let Some(end_pos) = col_substr.find(|c: char| !c.is_ascii_digit()) {
                        if let Ok(c) = col_substr[..end_pos].parse::<usize>() {
                            column = Some(c);
                        }
                    } else if let Ok(c) = col_substr.parse::<usize>() {
                        column = Some(c);
                    }
                } else {
                    // Only line number, no column
                    if let Some(end_pos) = substr.find(|c: char| !c.is_ascii_digit())
                        && let Ok(l) = substr[..end_pos].parse::<usize>()
                    {
                        line = Some(l);
                    }
                }
            }

            // Detect the kind of error for appropriate suggestion
            let kind = TomlErrorKind::from_parse_error(&error_str);

            AgentConfigError::ParseError {
                message: error_str,
                line,
                column,
                kind,
            }
        })
    }

    /// Validate agent configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate_agent_config(_config: &AgentConfig) -> AgentResult<()> {
        unimplemented!("Agent configuration validation will be implemented by green-implementer")
    }

    /// Deploy agent with configuration
    ///
    /// # Errors
    ///
    /// Returns an error if deployment fails.
    pub fn deploy_agent(_config: AgentConfig) -> AgentResult<AgentId> {
        unimplemented!("Agent deployment will be implemented by green-implementer")
    }

    /// Configuration watcher for hot reload functionality
    #[derive(Debug)]
    pub struct ConfigWatcher {
        /// Path to the configuration file being watched
        path: std::path::PathBuf,
        /// Current configuration snapshot (thread-safe access)
        current: Arc<Mutex<AgentConfig>>,
        /// Last modification time for change detection
        last_modified: Arc<Mutex<SystemTime>>,
        /// Watcher state (phantom type could be added for state machine)
        _watcher_handle: WatcherHandle,
    }

    /// Handle to the file system watcher
    #[derive(Debug)]
    #[allow(dead_code)]
    struct WatcherHandle {
        /// Path being watched
        path: std::path::PathBuf,
        /// Background task handle for watching
        _task_handle: tokio::task::JoinHandle<()>,
    }

    impl ConfigWatcher {
        /// Get the current configuration snapshot
        ///
        /// # Errors
        ///
        /// Returns an error if the configuration cannot be read.
        ///
        /// # Panics
        ///
        /// Panics if the internal mutex is poisoned (rare programming error).
        pub fn current_config(&self) -> AgentResult<AgentConfig> {
            // Check if file has been modified and reload if necessary
            if let Ok(metadata) = std::fs::metadata(&self.path)
                && let Ok(modified) = metadata.modified()
            {
                let mut last_modified = self.last_modified.lock().unwrap();
                if modified > *last_modified {
                    // File has been modified, reload configuration
                    let content = std::fs::read_to_string(&self.path).map_err(|e| {
                        AgentConfigError::ParseError {
                            message: format!("Failed to read file: {e}"),
                            line: None,
                            column: None,
                            kind: TomlErrorKind::SyntaxError,
                        }
                    })?;

                    let new_config = load_agent_config_from_toml(&content)?;
                    let mut current = self.current.lock().unwrap();
                    *current = new_config;
                    *last_modified = modified;
                }
            }

            // Return current configuration
            let current = self.current.lock().unwrap();
            Ok(current.clone())
        }

        /// Get the path being watched
        #[must_use]
        pub fn path(&self) -> &std::path::Path {
            &self.path
        }

        /// Check if the watcher is still active
        #[must_use]
        pub fn is_active(&self) -> bool {
            // Implementation will check if file watcher is still running
            unimplemented!("Watcher activity check will be implemented by green-implementer")
        }

        /// Stop watching for configuration changes
        pub fn stop(self) {
            // Clean shutdown of file watcher
            unimplemented!("Watcher shutdown will be implemented by green-implementer")
        }
    }

    /// Hot reload trigger types
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum HotReloadTrigger {
        /// File system change detected
        FileChange {
            /// Path that changed
            path: std::path::PathBuf,
            /// Type of change
            change_type: FileChangeType,
        },
        /// Manual reload requested via signal
        Signal {
            /// Signal type (e.g., SIGHUP)
            signal: i32,
        },
        /// Reload requested via API
        ApiRequest {
            /// Request ID for tracking
            request_id: String,
        },
    }

    /// Types of file system changes
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FileChangeType {
        /// File was modified
        Modified,
        /// File was created
        Created,
        /// File was deleted
        Deleted,
        /// File was renamed
        Renamed,
    }

    /// Hot reload errors
    #[derive(Debug)]
    pub enum HotReloadError {
        /// Failed to start file watcher
        WatcherStartError {
            /// Path that couldn't be watched
            path: std::path::PathBuf,
            /// Error message
            message: String,
        },
        /// Configuration reload failed
        ReloadError {
            /// Path to the configuration file
            path: std::path::PathBuf,
            /// The underlying error
            source: Box<dyn std::error::Error + Send + Sync>,
        },
        /// File system error
        FileSystemError {
            /// Error message
            message: String,
            /// The underlying IO error
            source: std::io::Error,
        },
    }

    impl std::fmt::Display for HotReloadError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                HotReloadError::WatcherStartError { path, message } => {
                    write!(
                        f,
                        "Failed to start watcher for {}: {}",
                        path.display(),
                        message
                    )
                }
                HotReloadError::ReloadError { path, source } => {
                    write!(
                        f,
                        "Failed to reload configuration from {}: {}",
                        path.display(),
                        source
                    )
                }
                HotReloadError::FileSystemError { message, source } => {
                    write!(f, "File system error: {message} ({source})")
                }
            }
        }
    }

    impl std::error::Error for HotReloadError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                HotReloadError::FileSystemError { source, .. } => Some(source),
                _ => None,
            }
        }
    }

    /// Start a hot reload watcher for an agent configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the watcher cannot be started or the initial configuration cannot be loaded.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (rare programming error).
    pub fn start_hot_reload_watcher(
        path: impl AsRef<std::path::Path>,
    ) -> Result<ConfigWatcher, HotReloadError> {
        let path = path.as_ref().to_path_buf();

        // Load initial configuration
        let content =
            std::fs::read_to_string(&path).map_err(|e| HotReloadError::FileSystemError {
                message: "Failed to read initial configuration".to_string(),
                source: e,
            })?;

        let config =
            load_agent_config_from_toml(&content).map_err(|e| HotReloadError::ReloadError {
                path: path.clone(),
                source: Box::new(e),
            })?;

        // Get initial modification time
        let metadata = std::fs::metadata(&path).map_err(|e| HotReloadError::FileSystemError {
            message: "Failed to get file metadata".to_string(),
            source: e,
        })?;
        let last_modified = metadata
            .modified()
            .map_err(|e| HotReloadError::FileSystemError {
                message: "Failed to get modification time".to_string(),
                source: e,
            })?;

        // Create shared state
        let current = Arc::new(Mutex::new(config));
        let last_modified_shared = Arc::new(Mutex::new(last_modified));

        // Start background task for periodic checking (minimal polling approach)
        let path_clone = path.clone();
        let current_clone = current.clone();
        let last_modified_clone = last_modified_shared.clone();

        let task_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
            loop {
                interval.tick().await;

                // Check if file has been modified
                if let Ok(metadata) = std::fs::metadata(&path_clone)
                    && let Ok(modified) = metadata.modified()
                {
                    let mut last_modified = last_modified_clone.lock().unwrap();
                    if modified > *last_modified {
                        // File has been modified, try to reload
                        if let Ok(content) = std::fs::read_to_string(&path_clone)
                            && let Ok(new_config) = load_agent_config_from_toml(&content)
                        {
                            let mut current = current_clone.lock().unwrap();
                            *current = new_config;
                            *last_modified = modified;
                        }
                    }
                }
            }
        });

        let watcher_handle = WatcherHandle {
            path: path.clone(),
            _task_handle: task_handle,
        };

        Ok(ConfigWatcher {
            path,
            current,
            last_modified: last_modified_shared,
            _watcher_handle: watcher_handle,
        })
    }

    /// Agent identifier (generated UUID)
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct AgentId(String);

    impl AgentId {
        /// Generate a new agent ID
        #[must_use]
        pub fn generate() -> Self {
            unimplemented!("Agent ID generation will be implemented by green-implementer")
        }

        /// Create from existing ID string
        ///
        /// # Errors
        ///
        /// Returns an error if the ID is invalid.
        pub fn from_string(id: String) -> Result<Self, String> {
            // Validate UUID format
            if id.len() != 36 {
                return Err("Agent ID must be a valid UUID".to_string());
            }
            Ok(AgentId(id))
        }

        /// Get the inner value
        #[must_use]
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl std::fmt::Display for AgentId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

/// Application initialization workflow
pub mod init {
    use crate::domain::config::ConfigResult;
    use crate::domain::server::Server;
    use std::path::PathBuf;

    /// Initialize application from configuration sources
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or is invalid.
    pub fn initialize_app(
        _config_path: Option<PathBuf>,
    ) -> ConfigResult<Server<crate::domain::server::NotStarted>> {
        unimplemented!("Application initialization will be implemented by green-implementer")
    }

    /// Bootstrap the complete application
    ///
    /// # Errors
    ///
    /// Returns an error if the application cannot be initialized or started.
    pub fn bootstrap() -> Result<(), Box<dyn std::error::Error>> {
        unimplemented!("Application bootstrap will be implemented by green-implementer")
    }
}
