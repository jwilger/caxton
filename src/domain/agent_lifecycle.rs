//! Agent Lifecycle Management domain types
//!
//! This module defines the core state machine and types for managing agent lifecycles
//! from deployment through termination, including state transitions and validation.

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

use crate::domain_types::{AgentId, AgentName};

/// Unique identifier for an agent version
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
pub struct AgentVersion(Uuid);

impl AgentVersion {
    /// Creates a new random agent version ID
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }

    /// Creates a version from a string representation
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid UUID format.
    pub fn parse(s: &str) -> Result<Self, String> {
        let uuid = Uuid::parse_str(s).map_err(|e| format!("Invalid UUID format: {e}"))?;
        Ok(Self::new(uuid))
    }
}

/// Sequential version number for agent releases
#[nutype(
    validate(greater_or_equal = 1),
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
pub struct VersionNumber(u64);

impl VersionNumber {
    /// First version number
    ///
    /// # Panics
    ///
    /// This function panics if version 1 is not valid, which should never happen.
    pub fn first() -> Self {
        Self::try_new(1).expect("Version 1 should always be valid")
    }

    /// Increment to next version
    ///
    /// # Errors
    ///
    /// Returns an error if the next version number would overflow or be invalid.
    pub fn next(&self) -> Result<Self, VersionNumberError> {
        Self::try_new(self.into_inner() + 1)
    }

    /// Gets the value as u64
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }
}

/// Agent lifecycle state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AgentLifecycleState {
    /// Agent module not yet loaded
    Unloaded,
    /// Agent module loaded but not instantiated
    Loaded,
    /// Agent instantiated and ready to run
    Ready,
    /// Agent actively executing
    Running,
    /// Agent draining existing requests before shutdown
    Draining,
    /// Agent has stopped execution
    Stopped,
    /// Agent encountered a fatal error
    Failed,
}

impl AgentLifecycleState {
    /// Checks if the state allows starting execution
    pub fn can_start(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Checks if the state allows draining
    pub fn can_drain(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Checks if the state allows stopping
    pub fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Draining | Self::Ready)
    }

    /// Checks if the state is terminal (cannot transition further)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }

    /// Checks if the state is active (consuming resources)
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Draining)
    }

    /// Gets all valid next states from current state
    pub fn valid_transitions(&self) -> Vec<Self> {
        match self {
            Self::Unloaded => vec![Self::Loaded, Self::Failed],
            Self::Loaded => vec![Self::Ready, Self::Failed, Self::Unloaded],
            Self::Ready => vec![Self::Running, Self::Stopped, Self::Failed],
            Self::Running => vec![Self::Draining, Self::Stopped, Self::Failed],
            Self::Draining => vec![Self::Stopped, Self::Failed],
            Self::Stopped | Self::Failed => vec![], // Terminal states
        }
    }

    /// Validates if transition to new state is allowed
    pub fn can_transition_to(&self, next: Self) -> bool {
        self.valid_transitions().contains(&next)
    }
}

impl fmt::Display for AgentLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match self {
            Self::Unloaded => "unloaded",
            Self::Loaded => "loaded",
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        };
        write!(f, "{state_str}")
    }
}

/// Timeout for agent state transitions
#[nutype(
    validate(greater_or_equal = 1000, less_or_equal = 300_000), // 1 second to 5 minutes
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
    default = 30_000 // 30 seconds
)]
pub struct TransitionTimeout(u64);

impl TransitionTimeout {
    /// Creates timeout from seconds
    ///
    /// # Errors
    ///
    /// Returns an error if the timeout value is invalid or too large.
    pub fn from_secs(secs: u64) -> Result<Self, TransitionTimeoutError> {
        Self::try_new(secs * 1000)
    }

    /// Gets the value as milliseconds
    pub fn as_millis(&self) -> u64 {
        self.into_inner()
    }

    /// Gets the value as seconds
    pub fn as_secs(&self) -> u64 {
        self.into_inner() / 1000
    }
}

/// Timeout for draining operations
#[nutype(
    validate(greater_or_equal = 5_000, less_or_equal = 600_000), // 5 seconds to 10 minutes
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
    default = 60_000 // 60 seconds
)]
pub struct DrainTimeout(u64);

impl DrainTimeout {
    /// Creates timeout from seconds
    ///
    /// # Errors
    ///
    /// Returns an error if the timeout value is invalid or too large.
    pub fn from_secs(secs: u64) -> Result<Self, DrainTimeoutError> {
        Self::try_new(secs * 1000)
    }

    /// Gets the value as milliseconds
    pub fn as_millis(&self) -> u64 {
        self.into_inner()
    }

    /// Gets the value as seconds
    pub fn as_secs(&self) -> u64 {
        self.into_inner() / 1000
    }
}

/// Number of pending requests during draining
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 10_000),
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
pub struct PendingRequestCount(u32);

impl PendingRequestCount {
    /// Zero pending requests
    pub fn zero() -> Self {
        Self::default()
    }

    /// Increment count by one
    ///
    /// # Errors
    ///
    /// Returns an error if incrementing would overflow the count.
    pub fn increment(&self) -> Result<Self, PendingRequestCountError> {
        Self::try_new(self.into_inner() + 1)
    }

    /// Decrement count by one, saturating at zero
    #[must_use]
    pub fn decrement(&self) -> Self {
        let current = self.into_inner();
        if current > 0 {
            Self::try_new(current - 1).unwrap_or_default()
        } else {
            Self::zero()
        }
    }

    /// Gets the value as u32
    pub fn as_u32(&self) -> u32 {
        self.into_inner()
    }

    /// Check if there are pending requests
    pub fn has_pending(&self) -> bool {
        self.into_inner() > 0
    }
}

/// Reason for agent failure
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
pub struct FailureReason(String);

impl FailureReason {
    /// Creates failure reason from error
    /// Creates failure reason from error
    ///
    /// # Errors
    ///
    /// Returns an error if the error message is too long or invalid.
    pub fn from_error<E: std::error::Error>(error: &E) -> Result<Self, FailureReasonError> {
        Self::try_new(error.to_string())
    }

    /// Creates failure reason from string
    ///
    /// # Errors
    ///
    /// Returns an error if the reason string is invalid.
    pub fn from_reason(reason: &str) -> Result<Self, FailureReasonError> {
        Self::try_new(reason.to_string())
    }
}

/// Agent lifecycle state with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLifecycle {
    pub agent_id: AgentId,
    pub agent_name: Option<AgentName>,
    pub version: AgentVersion,
    pub version_number: VersionNumber,
    pub current_state: AgentLifecycleState,
    pub previous_state: Option<AgentLifecycleState>,
    pub transition_timeout: TransitionTimeout,
    pub drain_timeout: DrainTimeout,
    pub pending_requests: PendingRequestCount,
    pub failure_reason: Option<FailureReason>,
}

impl AgentLifecycle {
    /// Creates a new agent lifecycle in unloaded state
    pub fn new(
        agent_id: AgentId,
        agent_name: Option<AgentName>,
        version: AgentVersion,
        version_number: VersionNumber,
    ) -> Self {
        Self {
            agent_id,
            agent_name,
            version,
            version_number,
            current_state: AgentLifecycleState::Unloaded,
            previous_state: None,
            transition_timeout: TransitionTimeout::default(),
            drain_timeout: DrainTimeout::default(),
            pending_requests: PendingRequestCount::zero(),
            failure_reason: None,
        }
    }

    /// Attempts to transition to a new state
    ///
    /// # Errors
    ///
    /// Returns an error if the state transition is invalid.
    pub fn transition_to(
        &mut self,
        new_state: AgentLifecycleState,
        failure_reason: Option<FailureReason>,
    ) -> Result<(), StateTransitionError> {
        if !self.current_state.can_transition_to(new_state) {
            return Err(StateTransitionError::InvalidTransition {
                from: self.current_state,
                to: new_state,
            });
        }

        self.previous_state = Some(self.current_state);
        self.current_state = new_state;
        self.failure_reason = failure_reason;

        Ok(())
    }

    /// Starts the agent (Ready -> Running)
    ///
    /// # Errors
    ///
    /// Returns an error if the agent cannot transition to Running state.
    pub fn start(&mut self) -> Result<(), StateTransitionError> {
        self.transition_to(AgentLifecycleState::Running, None)
    }

    /// Begins draining the agent (Running -> Draining)
    ///
    /// # Errors
    ///
    /// Returns an error if the agent cannot transition to Draining state.
    pub fn start_draining(&mut self) -> Result<(), StateTransitionError> {
        self.transition_to(AgentLifecycleState::Draining, None)
    }

    /// Stops the agent (Running/Draining/Ready -> Stopped)
    ///
    /// # Errors
    ///
    /// Returns an error if the agent cannot transition to Stopped state.
    pub fn stop(&mut self) -> Result<(), StateTransitionError> {
        self.transition_to(AgentLifecycleState::Stopped, None)
    }

    /// Marks agent as failed with reason
    ///
    /// # Errors
    ///
    /// Returns an error if the agent cannot transition to Failed state.
    pub fn fail(&mut self, reason: FailureReason) -> Result<(), StateTransitionError> {
        self.transition_to(AgentLifecycleState::Failed, Some(reason))
    }

    /// Increments pending request count
    ///
    /// # Errors
    ///
    /// Returns an error if too many requests are pending.
    pub fn add_pending_request(&mut self) -> Result<(), StateTransitionError> {
        self.pending_requests = self.pending_requests.increment().map_err(|_| {
            StateTransitionError::TooManyPendingRequests {
                current: self.pending_requests.as_u32(),
            }
        })?;
        Ok(())
    }

    /// Decrements pending request count
    pub fn complete_request(&mut self) {
        self.pending_requests = self.pending_requests.decrement();
    }

    /// Checks if agent is ready to be fully stopped (no pending requests)
    pub fn is_ready_to_stop(&self) -> bool {
        self.current_state == AgentLifecycleState::Draining && !self.pending_requests.has_pending()
    }

    /// Gets time-based constraints for current state
    pub fn get_timeout_for_state(&self) -> u64 {
        match self.current_state {
            AgentLifecycleState::Draining => self.drain_timeout.as_millis(),
            _ => self.transition_timeout.as_millis(),
        }
    }
}

/// Errors related to agent lifecycle management
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum StateTransitionError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition {
        from: AgentLifecycleState,
        to: AgentLifecycleState,
    },

    #[error("Too many pending requests: {current}")]
    TooManyPendingRequests { current: u32 },

    #[error("Agent is in terminal state: {state}")]
    TerminalState { state: AgentLifecycleState },

    #[error("Transition timeout exceeded: {timeout_ms}ms")]
    TimeoutExceeded { timeout_ms: u64 },

    #[error("Agent already in target state: {state}")]
    AlreadyInState { state: AgentLifecycleState },
}

/// Lifecycle operation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleOperationResult {
    pub success: bool,
    pub previous_state: Option<AgentLifecycleState>,
    pub current_state: AgentLifecycleState,
    pub error: Option<String>,
}

impl LifecycleOperationResult {
    /// Creates a successful operation result
    pub fn success(
        previous_state: Option<AgentLifecycleState>,
        current_state: AgentLifecycleState,
    ) -> Self {
        Self {
            success: true,
            previous_state,
            current_state,
            error: None,
        }
    }

    /// Creates a failed operation result
    pub fn failure(
        previous_state: Option<AgentLifecycleState>,
        current_state: AgentLifecycleState,
        error: String,
    ) -> Self {
        Self {
            success: false,
            previous_state,
            current_state,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_lifecycle_state_transitions() {
        // Test valid transitions
        let unloaded = AgentLifecycleState::Unloaded;
        assert!(unloaded.can_transition_to(AgentLifecycleState::Loaded));
        assert!(unloaded.can_transition_to(AgentLifecycleState::Failed));
        assert!(!unloaded.can_transition_to(AgentLifecycleState::Running));

        let loaded = AgentLifecycleState::Loaded;
        assert!(loaded.can_transition_to(AgentLifecycleState::Ready));
        assert!(loaded.can_transition_to(AgentLifecycleState::Failed));
        assert!(loaded.can_transition_to(AgentLifecycleState::Unloaded));

        let running = AgentLifecycleState::Running;
        assert!(running.can_transition_to(AgentLifecycleState::Draining));
        assert!(running.can_transition_to(AgentLifecycleState::Stopped));
        assert!(running.can_transition_to(AgentLifecycleState::Failed));
    }

    #[test]
    fn test_pending_request_count() {
        let mut count = PendingRequestCount::zero();
        assert_eq!(count.as_u32(), 0);
        assert!(!count.has_pending());

        count = count.increment().unwrap();
        assert_eq!(count.as_u32(), 1);
        assert!(count.has_pending());

        count = count.decrement();
        assert_eq!(count.as_u32(), 0);
        assert!(!count.has_pending());
    }

    #[test]
    fn test_agent_lifecycle_new() {
        let agent_id = AgentId::generate();
        let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();

        let lifecycle = AgentLifecycle::new(agent_id, agent_name, version, version_number);

        assert_eq!(lifecycle.current_state, AgentLifecycleState::Unloaded);
        assert_eq!(lifecycle.previous_state, None);
        assert!(!lifecycle.pending_requests.has_pending());
    }
}
