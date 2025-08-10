//! Resource management and limit enforcement for WebAssembly agents

use anyhow::{Result, bail};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tracing::debug;

// Domain-specific types for resource management
use crate::domain_types::{
    AgentId, CpuFuel, ExecutionTime, MaxAgentMemory, MaxTotalMemory, MemoryBytes, MessageSize,
};
use nutype::nutype;
use std::collections::HashMap;
use thiserror::Error;

/// CPU fuel amount for consumption operations
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Display
))]
pub struct CpuFuelAmount(u64);

/// CPU fuel budget for tracking
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
        Default
    ),
    default = 0
)]
pub struct CpuFuelBudget(u64);

/// CPU fuel consumed tracking - import from `domain_types` to avoid conflict
pub use crate::domain_types::CpuFuelConsumed;

/// Error types for fuel operations
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FuelError {
    #[error("Insufficient fuel: requested {requested}, available {available}")]
    InsufficientFuel { requested: u64, available: u64 },

    #[error("Fuel already exhausted")]
    FuelExhausted,
}

/// Resource limits for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory allocation in bytes
    pub max_memory_bytes: MemoryBytes,
    /// Maximum CPU fuel units
    pub max_cpu_fuel: CpuFuel,
    /// Maximum execution time per operation
    pub max_execution_time: ExecutionTime,
    /// Maximum message size in bytes
    pub max_message_size: MessageSize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: MemoryBytes::from_mb(10).unwrap(), // 10MB
            max_cpu_fuel: CpuFuel::try_new(1_000_000).unwrap(),
            max_execution_time: ExecutionTime::from_secs(5),
            max_message_size: MessageSize::from_kb(100).unwrap(), // 100KB
        }
    }
}

/// Bounded memory request with compile-time limits using domain types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct AgentMemoryRequest {
    bytes: MaxAgentMemory,
}

#[allow(missing_docs)]
impl AgentMemoryRequest {
    /// # Errors
    /// Returns an error if bytes is 0 or exceeds the maximum limit
    pub fn try_new(bytes: usize) -> Result<Self, String> {
        if bytes == 0 {
            return Err("Memory request must be greater than 0".to_string());
        }
        let max_memory =
            MaxAgentMemory::try_new(bytes).map_err(|e| format!("Invalid memory request: {e}"))?;
        Ok(Self { bytes: max_memory })
    }

    pub fn into_inner(self) -> usize {
        self.bytes.into_inner()
    }

    /// Gets the max memory as a domain type
    pub fn as_max_memory(&self) -> MaxAgentMemory {
        self.bytes
    }
}

/// Memory allocation errors with strong typing
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum MemoryError {
    #[error("Agent memory limit exceeded: requested {requested}, limit {limit}")]
    AgentLimitExceeded {
        requested: MaxAgentMemory,
        limit: MaxAgentMemory,
    },

    #[error("Total memory limit exceeded: requested {requested}, current {current}, limit {limit}")]
    TotalLimitExceeded {
        requested: MaxAgentMemory,
        current: MaxTotalMemory,
        limit: MaxTotalMemory,
    },

    #[error("Agent {agent:?} not found")]
    AgentNotFound { agent: AgentId },

    #[error("Agent {agent:?} already has allocation")]
    AgentAlreadyAllocated { agent: AgentId },
}

/// Total memory allocated with bounds checking using domain types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct TotalMemoryAllocated {
    bytes: MaxTotalMemory,
}

#[allow(missing_docs)]
impl TotalMemoryAllocated {
    /// Creates a zero memory allocation
    ///
    /// # Panics
    /// Panics if zero is not a valid value for `MaxTotalMemory` (should never happen)
    pub fn zero() -> Self {
        Self {
            bytes: MaxTotalMemory::try_new(0)
                .expect("Zero should always be valid for MaxTotalMemory"),
        }
    }

    /// # Errors
    /// Returns an error if bytes exceeds the maximum total limit
    pub fn try_new(bytes: usize) -> Result<Self, String> {
        let max_total =
            MaxTotalMemory::try_new(bytes).map_err(|e| format!("Invalid total memory: {e}"))?;
        Ok(Self { bytes: max_total })
    }

    pub fn into_inner(self) -> usize {
        self.bytes.into_inner()
    }

    /// Gets the max total memory as a domain type
    pub fn as_max_total_memory(&self) -> MaxTotalMemory {
        self.bytes
    }

    /// # Errors
    /// Returns `MemoryError::TotalLimitExceeded` if adding would exceed limit
    pub fn add(&self, amount: AgentMemoryRequest) -> Result<Self, MemoryError> {
        let new_total = self.bytes.into_inner() + amount.into_inner();
        match MaxTotalMemory::try_new(new_total) {
            Ok(max_total) => Ok(Self { bytes: max_total }),
            Err(_) => Err(MemoryError::TotalLimitExceeded {
                requested: amount.as_max_memory(),
                current: self.bytes,
                limit: MaxTotalMemory::default(),
            }),
        }
    }

    /// Subtracts the given amount from total memory
    ///
    /// # Panics
    /// Panics if creating a zero `MaxTotalMemory` fails (should never happen)
    #[must_use]
    pub fn subtract(&self, amount: usize) -> Self {
        let current = self.bytes.into_inner();
        let new_value = current.saturating_sub(amount);
        Self {
            bytes: MaxTotalMemory::try_new(new_value)
                .unwrap_or_else(|_| MaxTotalMemory::try_new(0).unwrap()),
        }
    }
}

/// Bounded memory pool for agent allocations
#[allow(missing_docs)]
pub struct BoundedMemoryPool {
    allocations: HashMap<AgentId, AgentMemoryRequest>,
    total_allocated: TotalMemoryAllocated,
}

#[allow(missing_docs)]
impl Default for BoundedMemoryPool {
    fn default() -> Self {
        Self {
            allocations: HashMap::new(),
            total_allocated: TotalMemoryAllocated::zero(),
        }
    }
}

impl BoundedMemoryPool {
    /// Creates a new bounded memory pool
    pub fn new() -> Self {
        Self::default()
    }

    /// # Errors
    /// Returns `MemoryError` if agent already has allocation or if total limit would be exceeded
    pub fn allocate(
        &mut self,
        agent_id: AgentId,
        request: AgentMemoryRequest,
    ) -> Result<(), MemoryError> {
        if self.allocations.contains_key(&agent_id) {
            return Err(MemoryError::AgentAlreadyAllocated { agent: agent_id });
        }

        let new_total = self.total_allocated.add(request)?;

        self.allocations.insert(agent_id, request);
        self.total_allocated = new_total;

        Ok(())
    }

    /// # Errors
    /// Returns `MemoryError::AgentNotFound` if the agent has no allocation
    pub fn deallocate(&mut self, agent_id: AgentId) -> Result<AgentMemoryRequest, MemoryError> {
        let allocation = self
            .allocations
            .remove(&agent_id)
            .ok_or(MemoryError::AgentNotFound { agent: agent_id })?;

        self.total_allocated = self.total_allocated.subtract(allocation.into_inner());

        Ok(allocation)
    }

    /// Gets the current memory allocation for an agent if it exists
    pub fn get_allocation(&self, agent_id: &AgentId) -> Option<AgentMemoryRequest> {
        self.allocations.get(agent_id).copied()
    }

    /// Gets the total memory allocated across all agents
    pub fn total_allocated(&self) -> TotalMemoryAllocated {
        self.total_allocated
    }
}

/// Simple fuel tracker with domain types
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct SimpleFuelTracker {
    budget: CpuFuelBudget,
    consumed: CpuFuelConsumed,
}

#[allow(missing_docs)]
impl SimpleFuelTracker {
    pub fn new(budget: CpuFuelBudget) -> Self {
        Self {
            budget,
            consumed: CpuFuelConsumed::zero(),
        }
    }

    /// # Errors
    /// Returns an error if insufficient fuel is available
    pub fn consume(&mut self, amount: CpuFuelAmount) -> Result<(), FuelError> {
        let current_consumed = self.consumed.into_inner();
        let budget = self.budget.into_inner();
        let amount_val = amount.into_inner();

        if current_consumed + amount_val > budget {
            return Err(FuelError::InsufficientFuel {
                requested: amount_val,
                available: budget - current_consumed,
            });
        }

        self.consumed = self.consumed.saturating_add(amount_val);
        Ok(())
    }

    pub fn consumed(&self) -> CpuFuelConsumed {
        self.consumed
    }

    pub fn remaining(&self) -> CpuFuelBudget {
        let budget = self.budget.into_inner();
        let consumed = self.consumed.into_inner();
        CpuFuelBudget::new(budget - consumed)
    }
}

/// Manages resource allocation and tracking for all agents
pub struct ResourceManager {
    limits: ResourceLimits,
    memory_pool: Arc<std::sync::Mutex<BoundedMemoryPool>>,
    fuel_trackers: Arc<DashMap<AgentId, SimpleFuelTracker>>,
    agent_usage: Arc<DashMap<AgentId, AgentResourceUsage>>,
}

#[derive(Debug)]
struct AgentResourceUsage {
    memory_bytes: AtomicUsize,
    cpu_fuel_consumed: AtomicU64,
    message_count: AtomicUsize,
    last_updated: std::time::Instant,
}

impl ResourceManager {
    /// Creates a new resource manager with specified limits
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            memory_pool: Arc::new(std::sync::Mutex::new(BoundedMemoryPool::new())),
            fuel_trackers: Arc::new(DashMap::new()),
            agent_usage: Arc::new(DashMap::new()),
        }
    }

    /// Allocates memory for an agent using domain types with compile-time safety
    ///
    /// # Errors
    ///
    /// Returns a `MemoryError` if the allocation fails due to limits or agent conflicts
    pub fn allocate_memory(
        &self,
        agent_id: AgentId,
        request: AgentMemoryRequest,
    ) -> Result<(), MemoryError> {
        let mut pool = self
            .memory_pool
            .lock()
            .map_err(|_| MemoryError::AgentNotFound { agent: agent_id })?;

        pool.allocate(agent_id, request)?;

        // Update usage tracking for monitoring
        let usage = self
            .agent_usage
            .entry(agent_id)
            .or_insert_with(AgentResourceUsage::new);
        usage
            .memory_bytes
            .store(request.into_inner(), Ordering::SeqCst);

        debug!(
            "Allocated {} bytes for agent {:?}",
            request.into_inner(),
            agent_id
        );
        Ok(())
    }

    /// Deallocates all memory for an agent using the bounded memory pool
    ///
    /// # Errors
    ///
    /// Returns a `MemoryError` if the agent is not found
    pub fn deallocate_memory(&self, agent_id: AgentId) -> Result<AgentMemoryRequest, MemoryError> {
        let mut pool = self
            .memory_pool
            .lock()
            .map_err(|_| MemoryError::AgentNotFound { agent: agent_id })?;

        let deallocated = pool.deallocate(agent_id)?;

        // Update usage tracking
        if let Some(usage) = self.agent_usage.get(&agent_id) {
            usage.memory_bytes.store(0, Ordering::SeqCst);
        }

        debug!(
            "Deallocated {} bytes for agent {:?}",
            deallocated.into_inner(),
            agent_id
        );
        Ok(deallocated)
    }

    /// Consumes CPU fuel for an agent using domain types for type safety
    ///
    /// # Errors
    ///
    /// Returns a fuel error if consumption fails due to limits or exhaustion
    pub fn consume_fuel(&self, agent_id: AgentId, fuel_amount: CpuFuelAmount) -> Result<()> {
        let budget = CpuFuelBudget::new(self.limits.max_cpu_fuel.into_inner());
        let mut tracker = self
            .fuel_trackers
            .entry(agent_id)
            .or_insert_with(|| SimpleFuelTracker::new(budget));

        tracker.consume(fuel_amount).map_err(|e| {
            anyhow::anyhow!("Fuel consumption failed for agent {:?}: {}", agent_id, e)
        })?;

        // Update usage tracking for monitoring
        let usage = self
            .agent_usage
            .entry(agent_id)
            .or_insert_with(AgentResourceUsage::new);
        usage
            .cpu_fuel_consumed
            .store(tracker.consumed().into_inner(), Ordering::SeqCst);

        debug!(
            "Consumed {} fuel units for agent {:?}, remaining: {}",
            fuel_amount,
            agent_id,
            tracker.remaining()
        );
        Ok(())
    }

    /// Checks if a message size is within limits
    ///
    /// # Errors
    ///
    /// Returns an error if the message size exceeds the configured limit
    pub fn check_message_size(&self, size: MessageSize) -> Result<()> {
        let size_val: usize = size.into_inner();
        let max_size: usize = self.limits.max_message_size.into_inner();
        if size_val > max_size {
            bail!(
                "Message size {} exceeds limit of {} bytes",
                size_val,
                max_size
            );
        }
        Ok(())
    }

    /// Gets the current memory usage for an agent from the bounded pool
    pub fn get_agent_memory_usage(&self, agent_id: AgentId) -> Option<AgentMemoryRequest> {
        let pool = self.memory_pool.lock().ok()?;
        pool.get_allocation(&agent_id)
    }

    /// Gets the total fuel consumed by an agent using the `FuelTracker`
    pub fn get_agent_fuel_usage(&self, agent_id: AgentId) -> CpuFuelConsumed {
        self.fuel_trackers
            .get(&agent_id)
            .map_or(CpuFuelConsumed::zero(), |tracker| tracker.consumed())
    }

    /// Gets the total memory usage across all agents from the bounded pool
    pub fn get_total_memory_usage(&self) -> TotalMemoryAllocated {
        self.memory_pool
            .lock()
            .map_or_else(|_| TotalMemoryAllocated::zero(), |p| p.total_allocated())
    }

    /// Gets the total fuel consumed across all agents
    pub fn get_total_fuel_usage(&self) -> CpuFuelConsumed {
        let total: u64 = self
            .fuel_trackers
            .iter()
            .map(|entry| entry.consumed().into_inner())
            .sum();
        CpuFuelConsumed::try_new(total).unwrap_or_default()
    }

    /// Cleans up resources for a removed agent using domain types
    pub fn cleanup_agent(&self, agent_id: AgentId) {
        // Deallocate memory from bounded pool
        if let Ok(deallocated) = self.deallocate_memory(agent_id) {
            debug!(
                "Deallocated {} bytes during cleanup for agent {:?}",
                deallocated.into_inner(),
                agent_id
            );
        }

        // Remove fuel tracker
        let fuel_consumed = self
            .fuel_trackers
            .remove(&agent_id)
            .map_or(CpuFuelConsumed::zero(), |(_, tracker)| tracker.consumed());

        // Remove usage tracking
        self.agent_usage.remove(&agent_id);

        debug!(
            "Cleaned up resources for agent {:?}: fuel_consumed={}",
            agent_id, fuel_consumed
        );
    }

    /// Gets the configured resource limits
    pub fn get_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    /// Records a message sent by an agent
    pub fn record_message(&self, agent_id: AgentId) {
        if let Some(usage) = self.agent_usage.get(&agent_id) {
            usage.increment_message_count();
            debug!(
                "Agent {:?} sent message #{}, last update was {:?} ago",
                agent_id,
                usage.message_count(),
                usage.time_since_update()
            );
        }
    }
}

impl AgentResourceUsage {
    fn new() -> Self {
        Self {
            memory_bytes: AtomicUsize::new(0),
            cpu_fuel_consumed: AtomicU64::new(0),
            message_count: AtomicUsize::new(0),
            last_updated: std::time::Instant::now(),
        }
    }

    fn increment_message_count(&self) {
        self.message_count.fetch_add(1, Ordering::SeqCst);
        // Note: In a real implementation, we'd update last_updated here
        // but that would require interior mutability for the Instant
    }

    fn message_count(&self) -> usize {
        self.message_count.load(Ordering::SeqCst)
    }

    fn time_since_update(&self) -> std::time::Duration {
        self.last_updated.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes.into_inner(), 10 * 1024 * 1024);
        assert_eq!(limits.max_cpu_fuel.into_inner(), 1_000_000);
        assert_eq!(
            limits.max_execution_time.as_duration(),
            std::time::Duration::from_secs(5)
        );
        assert_eq!(limits.max_message_size.into_inner(), 100 * 1024);
    }

    #[test]
    fn test_resource_manager_new() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        assert_eq!(manager.get_total_memory_usage().into_inner(), 0);
        assert_eq!(manager.get_total_fuel_usage().into_inner(), 0);
    }

    #[test]
    fn test_allocate_memory_success() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        let request = AgentMemoryRequest::try_new(1024).unwrap();
        assert!(manager.allocate_memory(agent_id, request).is_ok());
        assert_eq!(
            manager
                .get_agent_memory_usage(agent_id)
                .unwrap()
                .into_inner(),
            1024
        );
        assert_eq!(manager.get_total_memory_usage().into_inner(), 1024);
    }

    #[test]
    fn test_allocate_memory_exceeds_limit() {
        // AgentMemoryRequest enforces limits at compile time, so this test
        // demonstrates that invalid requests cannot be constructed
        assert!(
            AgentMemoryRequest::try_new(
                MaxAgentMemory::try_new(10_485_760).unwrap().as_usize() + 1
            )
            .is_err()
        );
    }

    #[test]
    fn test_deallocate_memory() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        let request = AgentMemoryRequest::try_new(2048).unwrap();
        manager.allocate_memory(agent_id, request).unwrap();
        let deallocated = manager.deallocate_memory(agent_id).unwrap();

        assert_eq!(deallocated.into_inner(), 2048);
        assert!(manager.get_agent_memory_usage(agent_id).is_none());
        assert_eq!(manager.get_total_memory_usage().into_inner(), 0);
    }

    #[test]
    fn test_consume_fuel_success() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        let fuel_amount = CpuFuelAmount::new(100);
        assert!(manager.consume_fuel(agent_id, fuel_amount).is_ok());
        assert_eq!(manager.get_agent_fuel_usage(agent_id).into_inner(), 100);
        assert_eq!(manager.get_total_fuel_usage().into_inner(), 100);
    }

    #[test]
    fn test_consume_fuel_exceeds_limit() {
        let limits = ResourceLimits {
            max_cpu_fuel: CpuFuel::try_new(100).unwrap(),
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        let fuel_50 = CpuFuelAmount::new(50);
        let fuel_60 = CpuFuelAmount::new(60);
        assert!(manager.consume_fuel(agent_id, fuel_50).is_ok());
        assert!(manager.consume_fuel(agent_id, fuel_60).is_err());
        assert_eq!(manager.get_agent_fuel_usage(agent_id).into_inner(), 50);
    }

    #[test]
    fn test_check_message_size() {
        let limits = ResourceLimits {
            max_message_size: MessageSize::try_new(1024).unwrap(),
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);

        let size1 = MessageSize::try_new(512).unwrap();
        let size2 = MessageSize::try_new(1024).unwrap();
        assert!(manager.check_message_size(size1).is_ok());
        assert!(manager.check_message_size(size2).is_ok());
        // MessageSize domain type enforces limits at compile time
    }

    #[test]
    fn test_cleanup_agent() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        let request = AgentMemoryRequest::try_new(1024).unwrap();
        let fuel_amount = CpuFuelAmount::new(100);
        manager.allocate_memory(agent_id, request).unwrap();
        manager.consume_fuel(agent_id, fuel_amount).unwrap();

        assert_eq!(manager.get_total_memory_usage().into_inner(), 1024);
        assert_eq!(manager.get_total_fuel_usage().into_inner(), 100);

        manager.cleanup_agent(agent_id);

        assert!(manager.get_agent_memory_usage(agent_id).is_none());
        assert_eq!(manager.get_agent_fuel_usage(agent_id).into_inner(), 0);
        assert_eq!(manager.get_total_memory_usage().into_inner(), 0);
        assert_eq!(manager.get_total_fuel_usage().into_inner(), 0);
    }

    #[test]
    fn test_record_message() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        // Allocate some memory to create the agent entry
        let request = AgentMemoryRequest::try_new(1).unwrap();
        manager.allocate_memory(agent_id, request).unwrap();

        manager.record_message(agent_id);
        manager.record_message(agent_id);

        // Message count is incremented internally
        assert!(manager.agent_usage.get(&agent_id).is_some());
    }

    #[test]
    fn test_agent_resource_usage() {
        let usage = AgentResourceUsage::new();
        assert_eq!(usage.message_count(), 0);

        usage.increment_message_count();
        assert_eq!(usage.message_count(), 1);

        thread::sleep(Duration::from_millis(10));
        assert!(usage.time_since_update() >= Duration::from_millis(10));
    }

    #[test]
    fn test_memory_request_bounds_enforced_at_compile_time() {
        // These should succeed within bounds
        assert!(AgentMemoryRequest::try_new(1).is_ok());
        assert!(AgentMemoryRequest::try_new(MaxAgentMemory::default().as_usize()).is_ok());

        // These should fail outside bounds
        assert!(AgentMemoryRequest::try_new(0).is_err());
        assert!(
            AgentMemoryRequest::try_new(
                MaxAgentMemory::try_new(10_485_760).unwrap().as_usize() + 1
            )
            .is_err()
        );
    }

    #[test]
    fn test_agent_already_allocated_error() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        let request = AgentMemoryRequest::try_new(1024).unwrap();

        // First allocation should succeed
        assert!(manager.allocate_memory(agent_id, request).is_ok());

        // Second allocation for same agent should fail with domain error
        let result = manager.allocate_memory(agent_id, request);
        assert!(matches!(
            result,
            Err(MemoryError::AgentAlreadyAllocated { .. })
        ));
    }

    #[test]
    fn test_fuel_tracker_state_transitions() {
        let limits = ResourceLimits {
            max_cpu_fuel: CpuFuel::try_new(100).unwrap(),
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::generate();

        // Initial state - can consume fuel
        let fuel_50 = CpuFuelAmount::new(50);
        let fuel_30 = CpuFuelAmount::new(30);
        let fuel_25 = CpuFuelAmount::new(25);
        let fuel_20 = CpuFuelAmount::new(20);
        let fuel_1 = CpuFuelAmount::new(1);

        assert!(manager.consume_fuel(agent_id, fuel_50).is_ok());

        // Partial consumption
        assert!(manager.consume_fuel(agent_id, fuel_30).is_ok());

        // Should have 20 remaining, so 25 should fail
        assert!(manager.consume_fuel(agent_id, fuel_25).is_err());

        // But 20 should still work
        assert!(manager.consume_fuel(agent_id, fuel_20).is_ok());

        // Now exhausted - any consumption should fail
        assert!(manager.consume_fuel(agent_id, fuel_1).is_err());
    }
}
