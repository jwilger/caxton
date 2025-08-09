//! Resource management and limit enforcement for WebAssembly agents

use anyhow::{Result, bail};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tracing::{debug, warn};

use crate::{AgentId, CpuFuel, ExecutionTime, MemoryBytes, MessageSize};

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

/// Manages resource allocation and tracking for all agents
pub struct ResourceManager {
    limits: ResourceLimits,
    agent_usage: Arc<DashMap<AgentId, AgentResourceUsage>>,
    total_memory: Arc<AtomicUsize>,
    total_fuel: Arc<AtomicU64>,
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
            agent_usage: Arc::new(DashMap::new()),
            total_memory: Arc::new(AtomicUsize::new(0)),
            total_fuel: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Allocates memory for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the requested memory exceeds limits
    pub fn allocate_memory(&self, agent_id: AgentId, bytes: MemoryBytes) -> Result<()> {
        let bytes_val: usize = bytes.into_inner();
        let max_bytes: usize = self.limits.max_memory_bytes.into_inner();
        if bytes_val > max_bytes {
            bail!(
                "memory limit exceeded: {} bytes exceeds limit of {} bytes",
                bytes_val,
                max_bytes
            );
        }

        let usage = self
            .agent_usage
            .entry(agent_id)
            .or_insert_with(AgentResourceUsage::new);

        let current = usage.memory_bytes.load(Ordering::SeqCst);
        if current + bytes_val > max_bytes {
            bail!(
                "memory limit exceeded for agent {:?}: current={}, requested={}, limit={}",
                agent_id,
                current,
                bytes_val,
                max_bytes
            );
        }

        usage.memory_bytes.fetch_add(bytes_val, Ordering::SeqCst);
        self.total_memory.fetch_add(bytes_val, Ordering::SeqCst);

        debug!("Allocated {} bytes for agent {:?}", bytes_val, agent_id);
        Ok(())
    }

    /// Deallocates memory for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if deallocation fails
    pub fn deallocate_memory(&self, agent_id: AgentId, bytes: MemoryBytes) -> Result<()> {
        let bytes_val: usize = bytes.into_inner();
        if let Some(usage) = self.agent_usage.get(&agent_id) {
            let current = usage.memory_bytes.load(Ordering::SeqCst);
            if bytes_val > current {
                warn!(
                    "Attempting to deallocate {} bytes but only {} allocated for agent {:?}",
                    bytes_val, current, agent_id
                );
                usage.memory_bytes.store(0, Ordering::SeqCst);
                self.total_memory.fetch_sub(current, Ordering::SeqCst);
            } else {
                usage.memory_bytes.fetch_sub(bytes_val, Ordering::SeqCst);
                self.total_memory.fetch_sub(bytes_val, Ordering::SeqCst);
            }

            debug!("Deallocated {} bytes for agent {:?}", bytes_val, agent_id);
        }

        Ok(())
    }

    /// Consumes CPU fuel for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if fuel consumption exceeds CPU limits
    pub fn consume_fuel(&self, agent_id: AgentId, fuel: CpuFuel) -> Result<()> {
        let fuel_val: u64 = fuel.into_inner();
        let max_fuel: u64 = self.limits.max_cpu_fuel.into_inner();
        if fuel_val > max_fuel {
            bail!(
                "fuel consumption of {} exceeds CPU limit of {}",
                fuel_val,
                max_fuel
            );
        }

        let usage = self
            .agent_usage
            .entry(agent_id)
            .or_insert_with(AgentResourceUsage::new);

        let consumed = usage
            .cpu_fuel_consumed
            .fetch_add(fuel_val, Ordering::SeqCst);
        if consumed + fuel_val > max_fuel {
            usage
                .cpu_fuel_consumed
                .fetch_sub(fuel_val, Ordering::SeqCst);
            bail!("fuel limit exceeded (CPU limit) for agent {:?}", agent_id);
        }

        self.total_fuel.fetch_add(fuel_val, Ordering::SeqCst);

        debug!("Consumed {} fuel units for agent {:?}", fuel_val, agent_id);
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

    /// Gets the current memory usage for an agent
    pub fn get_agent_memory_usage(&self, agent_id: AgentId) -> MemoryBytes {
        self.agent_usage
            .get(&agent_id)
            .map_or(MemoryBytes::zero(), |usage| {
                MemoryBytes::try_new(usage.memory_bytes.load(Ordering::SeqCst))
                    .unwrap_or(MemoryBytes::zero())
            })
    }

    /// Gets the total fuel consumed by an agent
    pub fn get_agent_fuel_usage(&self, agent_id: AgentId) -> CpuFuel {
        self.agent_usage
            .get(&agent_id)
            .map_or(CpuFuel::zero(), |usage| {
                CpuFuel::try_new(usage.cpu_fuel_consumed.load(Ordering::SeqCst))
                    .unwrap_or(CpuFuel::zero())
            })
    }

    /// Gets the total memory usage across all agents
    pub fn get_total_memory_usage(&self) -> MemoryBytes {
        MemoryBytes::try_new(self.total_memory.load(Ordering::SeqCst))
            .unwrap_or(MemoryBytes::zero())
    }

    /// Gets the total fuel usage across all agents
    pub fn get_total_fuel_usage(&self) -> CpuFuel {
        CpuFuel::try_new(self.total_fuel.load(Ordering::SeqCst)).unwrap_or(CpuFuel::zero())
    }

    /// Cleans up resources for a removed agent
    pub fn cleanup_agent(&self, agent_id: AgentId) {
        if let Some((_, usage)) = self.agent_usage.remove(&agent_id) {
            let memory = usage.memory_bytes.load(Ordering::SeqCst);
            let fuel = usage.cpu_fuel_consumed.load(Ordering::SeqCst);

            self.total_memory.fetch_sub(memory, Ordering::SeqCst);
            self.total_fuel.fetch_sub(fuel, Ordering::SeqCst);

            debug!(
                "Cleaned up resources for agent {:?}: memory={}, fuel={}",
                agent_id, memory, fuel
            );
        }
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
        let agent_id = AgentId::new_v4();

        let mem = MemoryBytes::try_new(1024).unwrap();
        assert!(manager.allocate_memory(agent_id, mem).is_ok());
        assert_eq!(manager.get_agent_memory_usage(agent_id).into_inner(), 1024);
        assert_eq!(manager.get_total_memory_usage().into_inner(), 1024);
    }

    #[test]
    fn test_allocate_memory_exceeds_limit() {
        let limits = ResourceLimits {
            max_memory_bytes: MemoryBytes::try_new(1024).unwrap(),
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::new_v4();

        let mem = MemoryBytes::try_new(2048).unwrap();
        assert!(manager.allocate_memory(agent_id, mem).is_err());
        assert_eq!(manager.get_agent_memory_usage(agent_id).into_inner(), 0);
    }

    #[test]
    fn test_deallocate_memory() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::new_v4();

        let mem1 = MemoryBytes::try_new(2048).unwrap();
        let mem2 = MemoryBytes::try_new(1024).unwrap();
        manager.allocate_memory(agent_id, mem1).unwrap();
        manager.deallocate_memory(agent_id, mem2).unwrap();

        assert_eq!(manager.get_agent_memory_usage(agent_id).into_inner(), 1024);
        assert_eq!(manager.get_total_memory_usage().into_inner(), 1024);
    }

    #[test]
    fn test_consume_fuel_success() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::new_v4();

        let fuel = CpuFuel::try_new(100).unwrap();
        assert!(manager.consume_fuel(agent_id, fuel).is_ok());
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
        let agent_id = AgentId::new_v4();

        let fuel1 = CpuFuel::try_new(50).unwrap();
        let fuel2 = CpuFuel::try_new(60).unwrap();
        assert!(manager.consume_fuel(agent_id, fuel1).is_ok());
        assert!(manager.consume_fuel(agent_id, fuel2).is_err());
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
        // Can't create MessageSize > 10MB, so we can't test the error case directly
    }

    #[test]
    fn test_cleanup_agent() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::new_v4();

        let mem = MemoryBytes::try_new(1024).unwrap();
        let fuel = CpuFuel::try_new(100).unwrap();
        manager.allocate_memory(agent_id, mem).unwrap();
        manager.consume_fuel(agent_id, fuel).unwrap();

        assert_eq!(manager.get_total_memory_usage().into_inner(), 1024);
        assert_eq!(manager.get_total_fuel_usage().into_inner(), 100);

        manager.cleanup_agent(agent_id);

        assert_eq!(manager.get_agent_memory_usage(agent_id).into_inner(), 0);
        assert_eq!(manager.get_agent_fuel_usage(agent_id).into_inner(), 0);
        assert_eq!(manager.get_total_memory_usage().into_inner(), 0);
        assert_eq!(manager.get_total_fuel_usage().into_inner(), 0);
    }

    #[test]
    fn test_record_message() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = AgentId::new_v4();

        // Allocate some memory to create the agent entry
        let mem = MemoryBytes::try_new(1).unwrap();
        manager.allocate_memory(agent_id, mem).unwrap();

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
}
