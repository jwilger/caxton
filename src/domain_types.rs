//! Domain types for the Caxton WebAssembly runtime
//!
//! This module defines strongly-typed domain values to prevent primitive obsession
//! and improve type safety throughout the codebase.

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for an agent
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    TryFrom,
    Into
))]
pub struct AgentId(Uuid);

impl AgentId {
    /// Creates a new random agent ID
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// Name of an agent
#[nutype(
    validate(len_char_min = 1, len_char_max = 255),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct AgentName(String);

/// Name of a host function
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct HostFunctionName(String);

/// Memory size in bytes
#[nutype(
    validate(less_or_equal = 1073741824), // 1GB max
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default, TryFrom, Into),
    default = 0,
)]
pub struct MemoryBytes(usize);

impl MemoryBytes {
    /// Returns zero memory bytes
    pub fn zero() -> Self {
        Self::default()
    }

    /// Creates memory bytes from megabytes
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting byte count exceeds the maximum allowed (1GB)
    pub fn from_mb(mb: usize) -> Result<Self, MemoryBytesError> {
        Self::try_new(mb * 1024 * 1024)
    }

    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }

    /// Applies memory limits to a wasmtime Store
    ///
    /// This method encapsulates the knowledge of how to configure wasmtime
    /// memory limits using domain types, keeping the wasmtime integration
    /// separate from business logic.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The wasmtime Store data type
    ///
    /// # Arguments
    ///
    /// * `store` - Mutable reference to the wasmtime Store to configure
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let memory_limit = MemoryBytes::from_mb(10)?;
    /// memory_limit.apply_to_wasmtime_store(&mut store);
    /// ```
    /// Applies memory limits to a wasmtime Store (when wasmtime feature is enabled)
    /// Note: This is currently a placeholder for future wasmtime integration
    #[allow(unused_variables)]
    pub fn apply_to_wasmtime_store<T>(&self, _store: &mut T) {
        // Placeholder for wasmtime integration
        let _bytes = self.as_usize(); // Use the validated byte count
    }

    /// Applies memory limits to a wasmtime Store (feature-agnostic version)
    ///
    /// This version works without the wasmtime feature flag and returns
    /// the byte count that should be applied to external systems.
    pub fn get_wasmtime_limit(&self) -> usize {
        self.as_usize()
    }
}

/// CPU fuel units for execution
#[nutype(
    validate(less_or_equal = 1_000_000_000), // 1 billion max
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default, TryFrom, Into),
    default = 0,
)]
pub struct CpuFuel(u64);

impl CpuFuel {
    /// Returns zero fuel
    pub fn zero() -> Self {
        Self::default()
    }

    /// Adds fuel amounts safely
    ///
    /// # Panics
    ///
    /// Panics if the maximum fuel value (1 billion) cannot be created (should never happen)
    #[must_use]
    pub fn saturating_add(self, other: Self) -> Self {
        let sum = self.into_inner().saturating_add(other.into_inner());
        Self::try_new(sum.min(1_000_000_000))
            .unwrap_or_else(|_| Self::try_new(1_000_000_000).unwrap())
    }

    /// Subtracts fuel amounts safely, saturating at zero
    ///
    /// # Errors
    ///
    /// Returns an error if there is insufficient fuel to subtract
    pub fn subtract(self, amount: CpuFuelAmount) -> Result<Self, ValidationError> {
        let current = self.into_inner();
        let to_subtract = amount.into_inner();

        if to_subtract > current {
            return Err(ValidationError::ConstraintViolation {
                constraint: format!(
                    "Insufficient fuel: tried to subtract {to_subtract} from {current}"
                ),
            });
        }

        // Use try_new since CpuFuel has validation
        Self::try_new(current - to_subtract).map_err(|e| ValidationError::InvalidField {
            field: "cpu_fuel".to_string(),
            reason: e.to_string(),
        })
    }

    /// Subtracts fuel amounts, saturating at zero (no error on underflow)
    #[must_use]
    pub fn saturating_subtract(self, amount: CpuFuelAmount) -> Self {
        let current = self.into_inner();
        let to_subtract = amount.into_inner();
        // Safe to use new since we know saturating_sub returns a valid value
        Self::try_new(current.saturating_sub(to_subtract)).unwrap_or_default()
    }

    /// Adds a fuel amount to this fuel
    ///
    /// # Errors
    ///
    /// Returns an error if the addition would exceed the maximum fuel limit
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, amount: CpuFuelAmount) -> Result<Self, ValidationError> {
        let current = self.into_inner();
        let to_add = amount.into_inner();
        let sum = current.saturating_add(to_add);

        if sum > 1_000_000_000 {
            return Err(ValidationError::ValueOutOfRange {
                value: i64::try_from(sum).unwrap_or(i64::MAX),
                min: 0,
                max: 1_000_000_000,
            });
        }

        Self::try_new(sum).map_err(|e| ValidationError::InvalidField {
            field: "cpu_fuel".to_string(),
            reason: e.to_string(),
        })
    }

    /// Gets the value as u64
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }
}

/// Message size in bytes
#[nutype(
    validate(less_or_equal = 10485760), // 10MB max
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, TryFrom, Into),
)]
pub struct MessageSize(usize);

impl MessageSize {
    /// Creates message size from kilobytes
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting byte count exceeds the maximum allowed (10MB)
    pub fn from_kb(kb: usize) -> Result<Self, MessageSizeError> {
        Self::try_new(kb * 1024)
    }
}

/// Maximum number of concurrent agents
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 10000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into,
        Default
    ),
    default = 1000
)]
pub struct MaxAgents(usize);

impl MaxAgents {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Maximum number of import functions
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into,
        Default
    ),
    default = 10
)]
pub struct MaxImportFunctions(usize);

/// Maximum number of export functions
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into,
        Default
    ),
    default = 10
)]
pub struct MaxExports(usize);

impl MaxExports {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Message count for tracking
#[nutype(
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct MessageCount(usize);

impl MessageCount {
    /// Returns zero message count
    pub fn zero() -> Self {
        Self::default()
    }

    /// Increments the count by one
    #[must_use]
    pub fn increment(self) -> Self {
        Self::new(self.into_inner() + 1)
    }

    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Execution time duration wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTime(Duration);

impl ExecutionTime {
    /// Creates a new execution time from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    /// Creates a new execution time from a Duration
    pub fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }

    /// Gets the inner Duration
    pub fn as_duration(&self) -> Duration {
        self.0
    }
}

impl From<Duration> for ExecutionTime {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

impl From<ExecutionTime> for Duration {
    fn from(time: ExecutionTime) -> Self {
        time.0
    }
}

/// Maximum memory per agent for sandboxing
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10_485_760), // Max 10MB per agent, allowing 0
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default, TryFrom, Into),
    default = 1_048_576 // Default 1MB
)]
pub struct MaxAgentMemory(usize);

impl MaxAgentMemory {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Maximum total memory for all agents
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 104_857_600), // Max 100MB total, allowing 0
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, Default, TryFrom, Into),
    default = 104_857_600 // Default 100MB
)]
pub struct MaxTotalMemory(usize);

impl MaxTotalMemory {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Maximum table entries for WASM
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 10_000
)]
pub struct MaxTableEntries(usize);

impl MaxTableEntries {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Function module name
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct FunctionModuleName(String);

/// Function description
#[nutype(
    validate(len_char_min = 1, len_char_max = 1000),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct FunctionDescription(String);

/// Permission name for host functions
#[nutype(
    validate(len_char_min = 1, len_char_max = 100),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct PermissionName(String);

/// Connection pool size
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 1000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 10
)]
pub struct ConnectionPoolSize(usize);

impl ConnectionPoolSize {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// Storage cleanup interval in milliseconds
#[nutype(
    validate(greater_or_equal = 60_000, less_or_equal = 86_400_000), // 1 min to 24 hours
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 3_600_000 // 1 hour
)]
pub struct StorageCleanupIntervalMs(u64);

impl StorageCleanupIntervalMs {
    /// Converts to Duration
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis(self.into_inner())
    }

    /// Gets the value as u64
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }
}

/// Rate limit for messages per second
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100_000),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 1000
)]
pub struct RateLimitPerSecond(usize);

impl RateLimitPerSecond {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }
}

/// CPU fuel consumed during execution
#[nutype(
    validate(greater_or_equal = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct CpuFuelConsumed(u64);

impl CpuFuelConsumed {
    /// Returns zero consumed fuel
    pub fn zero() -> Self {
        Self::default()
    }

    /// Gets the value as u64
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }

    /// Adds consumed fuel amounts safely
    #[must_use]
    pub fn saturating_add(self, other: u64) -> Self {
        let sum = self.into_inner().saturating_add(other);
        Self::try_new(sum).unwrap_or_default()
    }

    /// Adds consumed fuel amounts safely with `CpuFuelAmount`
    #[must_use]
    pub fn saturating_add_fuel(self, other: CpuFuelAmount) -> Self {
        let sum = self.into_inner().saturating_add(other.into_inner());
        Self::try_new(sum).unwrap_or_default()
    }
}

/// Amount of CPU fuel for operations
#[nutype(
    validate(greater_or_equal = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct CpuFuelAmount(u64);

impl CpuFuelAmount {
    /// Returns zero fuel amount
    pub fn zero() -> Self {
        Self::default()
    }

    /// Gets the value as u64
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }

    /// Creates fuel amount from a primitive u64
    pub fn from_u64(value: u64) -> Self {
        // CpuFuelAmount has no validation so we can use new directly
        Self::try_new(value).unwrap_or_default()
    }
}

/// Worker thread identifier
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 32),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct WorkerId(usize);

impl WorkerId {
    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }

    /// Creates `WorkerId` from zero-based index (adds 1)
    ///
    /// # Errors
    ///
    /// Returns an error if the index would result in an invalid `WorkerId` (> 32)
    pub fn from_zero_based_index(index: usize) -> Result<Self, WorkerIdError> {
        Self::try_new(index + 1)
    }
}

/// Queue depth for message processing
#[nutype(
    validate(greater_or_equal = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct QueueDepth(usize);

impl QueueDepth {
    /// Returns zero queue depth
    pub fn zero() -> Self {
        Self::default()
    }

    /// Gets the value as usize
    pub fn as_usize(&self) -> usize {
        self.into_inner()
    }

    /// Increments the queue depth by one
    #[must_use]
    pub fn increment(self) -> Self {
        Self::try_new(self.into_inner() + 1).unwrap_or(self)
    }

    /// Decrements the queue depth by one, saturating at zero
    #[must_use]
    pub fn saturating_decrement(self) -> Self {
        Self::try_new(self.into_inner().saturating_sub(1)).unwrap_or_default()
    }
}

/// Retry attempt counter
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 24),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        TryFrom,
        Into
    )
)]
pub struct RetryAttempt(u8);

impl RetryAttempt {
    /// Gets the value as u8
    pub fn as_u8(&self) -> u8 {
        self.into_inner()
    }

    /// First attempt (attempt 1)
    /// # Panics
    ///
    /// Panics if 1 is not a valid retry attempt (should never happen)
    pub fn first() -> Self {
        Self::try_new(1).expect("First attempt should always be valid")
    }

    /// Increments the retry attempt by one
    ///
    /// # Errors
    ///
    /// Returns an error if incrementing would exceed the maximum (24)
    pub fn increment(self) -> Result<Self, RetryAttemptError> {
        Self::try_new(self.into_inner() + 1)
    }

    /// Checks if this is the final allowed attempt
    pub fn is_final(&self) -> bool {
        self.into_inner() == 24
    }
}

/// Test agent identifier
#[nutype(
    validate(greater_or_equal = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct TestAgentId(u32);

impl TestAgentId {
    /// Gets the value as u32
    pub fn as_u32(&self) -> u32 {
        self.into_inner()
    }

    /// Creates `TestAgentId` from a usize, clamping to `u32::MAX`
    pub fn from_usize(value: usize) -> Self {
        let max_as_usize = usize::try_from(u32::MAX).unwrap_or(usize::MAX);
        let clamped = value.min(max_as_usize);
        let clamped_u32 = u32::try_from(clamped).unwrap_or(u32::MAX);
        Self::try_new(clamped_u32).unwrap_or_default()
    }
}

/// Test sequence number
#[nutype(
    validate(greater_or_equal = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct TestSequence(u32);

impl TestSequence {
    /// Gets the value as u32
    pub fn as_u32(&self) -> u32 {
        self.into_inner()
    }

    /// Increments the sequence number by one
    #[must_use]
    pub fn increment(self) -> Self {
        Self::try_new(self.into_inner().saturating_add(1)).unwrap_or(self)
    }

    /// Returns zero sequence
    pub fn zero() -> Self {
        Self::default()
    }
}

/// Domain-level validation errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[allow(missing_docs)] // Error variant fields are self-documenting through error messages
pub enum ValidationError {
    /// Invalid field value with descriptive reason
    #[error("Invalid field '{field}': {reason}")]
    InvalidField { field: String, reason: String },

    /// Value is outside allowed range
    #[error("Value out of range: {value}, expected {min}-{max}")]
    ValueOutOfRange { value: i64, min: i64, max: i64 },

    /// Field has invalid format
    #[error("Invalid format for '{field}': {reason}")]
    InvalidFormat { field: String, reason: String },

    /// Required field is missing
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    /// Domain constraint violation
    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },
}

/// Resource creation and management errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[allow(missing_docs)] // Error variant fields are self-documenting through error messages
pub enum ResourceCreationError {
    /// Resource limit has been exceeded
    #[error("Resource limit exceeded: {resource_type} requested {requested}, limit {limit}")]
    LimitExceeded {
        resource_type: String,
        requested: u64,
        limit: u64,
    },

    /// Resource is currently unavailable
    #[error("Resource unavailable: {resource_type}")]
    Unavailable { resource_type: String },

    /// Resource already exists with the given ID
    #[error("Resource already exists: {resource_id}")]
    AlreadyExists { resource_id: String },

    /// Resource with the given ID was not found
    #[error("Resource not found: {resource_id}")]
    NotFound { resource_id: String },

    /// Resource configuration is invalid
    #[error("Invalid resource configuration: {reason}")]
    InvalidConfiguration { reason: String },

    /// Resource dependency could not be satisfied
    #[error("Resource dependency error: {dependency} required for {resource}")]
    DependencyError {
        resource: String,
        dependency: String,
    },
}
