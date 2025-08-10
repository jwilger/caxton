//! Domain types for the Caxton WebAssembly runtime
//!
//! This module defines strongly-typed domain values to prevent primitive obsession
//! and improve type safety throughout the codebase.

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::time::Duration;
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
