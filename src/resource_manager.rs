//! Resource management and limit enforcement for WebAssembly agents

use anyhow::{Result, bail};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;

/// Resource limits for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory allocation in bytes
    pub max_memory_bytes: usize,
    /// Maximum CPU fuel units
    pub max_cpu_fuel: u64,
    /// Maximum execution time per operation
    pub max_execution_time: Duration,
    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            max_cpu_fuel: 1_000_000,
            max_execution_time: Duration::from_secs(5),
            max_message_size: 100 * 1024, // 100KB
        }
    }
}

/// Manages resource allocation and tracking for all agents
pub struct ResourceManager {
    limits: ResourceLimits,
    agent_usage: Arc<DashMap<Uuid, AgentResourceUsage>>,
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
    pub fn allocate_memory(&self, agent_id: Uuid, bytes: usize) -> Result<()> {
        if bytes > self.limits.max_memory_bytes {
            bail!(
                "memory limit exceeded: {} bytes exceeds limit of {} bytes",
                bytes,
                self.limits.max_memory_bytes
            );
        }

        let usage = self
            .agent_usage
            .entry(agent_id)
            .or_insert_with(AgentResourceUsage::new);

        let current = usage.memory_bytes.load(Ordering::SeqCst);
        if current + bytes > self.limits.max_memory_bytes {
            bail!(
                "memory limit exceeded for agent {}: current={}, requested={}, limit={}",
                agent_id,
                current,
                bytes,
                self.limits.max_memory_bytes
            );
        }

        usage.memory_bytes.fetch_add(bytes, Ordering::SeqCst);
        self.total_memory.fetch_add(bytes, Ordering::SeqCst);

        debug!("Allocated {} bytes for agent {}", bytes, agent_id);
        Ok(())
    }

    /// Deallocates memory for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if deallocation fails
    pub fn deallocate_memory(&self, agent_id: Uuid, bytes: usize) -> Result<()> {
        if let Some(usage) = self.agent_usage.get(&agent_id) {
            let current = usage.memory_bytes.load(Ordering::SeqCst);
            if bytes > current {
                warn!(
                    "Attempting to deallocate {} bytes but only {} allocated for agent {}",
                    bytes, current, agent_id
                );
                usage.memory_bytes.store(0, Ordering::SeqCst);
                self.total_memory.fetch_sub(current, Ordering::SeqCst);
            } else {
                usage.memory_bytes.fetch_sub(bytes, Ordering::SeqCst);
                self.total_memory.fetch_sub(bytes, Ordering::SeqCst);
            }

            debug!("Deallocated {} bytes for agent {}", bytes, agent_id);
        }

        Ok(())
    }

    /// Consumes CPU fuel for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if fuel consumption exceeds CPU limits
    pub fn consume_fuel(&self, agent_id: Uuid, fuel: u64) -> Result<()> {
        if fuel > self.limits.max_cpu_fuel {
            bail!(
                "fuel consumption of {} exceeds CPU limit of {}",
                fuel,
                self.limits.max_cpu_fuel
            );
        }

        let usage = self
            .agent_usage
            .entry(agent_id)
            .or_insert_with(AgentResourceUsage::new);

        let consumed = usage.cpu_fuel_consumed.fetch_add(fuel, Ordering::SeqCst);
        if consumed + fuel > self.limits.max_cpu_fuel {
            usage.cpu_fuel_consumed.fetch_sub(fuel, Ordering::SeqCst);
            bail!("fuel limit exceeded (CPU limit) for agent {}", agent_id);
        }

        self.total_fuel.fetch_add(fuel, Ordering::SeqCst);

        debug!("Consumed {} fuel units for agent {}", fuel, agent_id);
        Ok(())
    }

    /// Checks if a message size is within limits
    ///
    /// # Errors
    ///
    /// Returns an error if the message size exceeds the configured limit
    pub fn check_message_size(&self, size: usize) -> Result<()> {
        if size > self.limits.max_message_size {
            bail!(
                "Message size {} exceeds limit of {} bytes",
                size,
                self.limits.max_message_size
            );
        }
        Ok(())
    }

    /// Gets the current memory usage for an agent
    pub fn get_agent_memory_usage(&self, agent_id: Uuid) -> usize {
        self.agent_usage
            .get(&agent_id)
            .map_or(0, |usage| usage.memory_bytes.load(Ordering::SeqCst))
    }

    /// Gets the total fuel consumed by an agent
    pub fn get_agent_fuel_usage(&self, agent_id: Uuid) -> u64 {
        self.agent_usage
            .get(&agent_id)
            .map_or(0, |usage| usage.cpu_fuel_consumed.load(Ordering::SeqCst))
    }

    /// Gets the total memory usage across all agents
    pub fn get_total_memory_usage(&self) -> usize {
        self.total_memory.load(Ordering::SeqCst)
    }

    /// Gets the total fuel usage across all agents
    pub fn get_total_fuel_usage(&self) -> u64 {
        self.total_fuel.load(Ordering::SeqCst)
    }

    /// Cleans up resources for a removed agent
    pub fn cleanup_agent(&self, agent_id: Uuid) {
        if let Some((_, usage)) = self.agent_usage.remove(&agent_id) {
            let memory = usage.memory_bytes.load(Ordering::SeqCst);
            let fuel = usage.cpu_fuel_consumed.load(Ordering::SeqCst);

            self.total_memory.fetch_sub(memory, Ordering::SeqCst);
            self.total_fuel.fetch_sub(fuel, Ordering::SeqCst);

            debug!(
                "Cleaned up resources for agent {}: memory={}, fuel={}",
                agent_id, memory, fuel
            );
        }
    }

    /// Gets the configured resource limits
    pub fn get_limits(&self) -> &ResourceLimits {
        &self.limits
    }

    /// Records a message sent by an agent
    pub fn record_message(&self, agent_id: Uuid) {
        if let Some(usage) = self.agent_usage.get(&agent_id) {
            usage.increment_message_count();
            debug!(
                "Agent {} sent message #{}, last update was {:?} ago",
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

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 10 * 1024 * 1024);
        assert_eq!(limits.max_cpu_fuel, 1_000_000);
        assert_eq!(limits.max_execution_time, Duration::from_secs(5));
        assert_eq!(limits.max_message_size, 100 * 1024);
    }

    #[test]
    fn test_resource_manager_new() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        assert_eq!(manager.get_total_memory_usage(), 0);
        assert_eq!(manager.get_total_fuel_usage(), 0);
    }

    #[test]
    fn test_allocate_memory_success() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = Uuid::new_v4();

        assert!(manager.allocate_memory(agent_id, 1024).is_ok());
        assert_eq!(manager.get_agent_memory_usage(agent_id), 1024);
        assert_eq!(manager.get_total_memory_usage(), 1024);
    }

    #[test]
    fn test_allocate_memory_exceeds_limit() {
        let limits = ResourceLimits {
            max_memory_bytes: 1024,
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);
        let agent_id = Uuid::new_v4();

        assert!(manager.allocate_memory(agent_id, 2048).is_err());
        assert_eq!(manager.get_agent_memory_usage(agent_id), 0);
    }

    #[test]
    fn test_deallocate_memory() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = Uuid::new_v4();

        manager.allocate_memory(agent_id, 2048).unwrap();
        manager.deallocate_memory(agent_id, 1024).unwrap();

        assert_eq!(manager.get_agent_memory_usage(agent_id), 1024);
        assert_eq!(manager.get_total_memory_usage(), 1024);
    }

    #[test]
    fn test_consume_fuel_success() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = Uuid::new_v4();

        assert!(manager.consume_fuel(agent_id, 100).is_ok());
        assert_eq!(manager.get_agent_fuel_usage(agent_id), 100);
        assert_eq!(manager.get_total_fuel_usage(), 100);
    }

    #[test]
    fn test_consume_fuel_exceeds_limit() {
        let limits = ResourceLimits {
            max_cpu_fuel: 100,
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);
        let agent_id = Uuid::new_v4();

        assert!(manager.consume_fuel(agent_id, 50).is_ok());
        assert!(manager.consume_fuel(agent_id, 60).is_err());
        assert_eq!(manager.get_agent_fuel_usage(agent_id), 50);
    }

    #[test]
    fn test_check_message_size() {
        let limits = ResourceLimits {
            max_message_size: 1024,
            ..Default::default()
        };
        let manager = ResourceManager::new(limits);

        assert!(manager.check_message_size(512).is_ok());
        assert!(manager.check_message_size(1024).is_ok());
        assert!(manager.check_message_size(2048).is_err());
    }

    #[test]
    fn test_cleanup_agent() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = Uuid::new_v4();

        manager.allocate_memory(agent_id, 1024).unwrap();
        manager.consume_fuel(agent_id, 100).unwrap();

        assert_eq!(manager.get_total_memory_usage(), 1024);
        assert_eq!(manager.get_total_fuel_usage(), 100);

        manager.cleanup_agent(agent_id);

        assert_eq!(manager.get_agent_memory_usage(agent_id), 0);
        assert_eq!(manager.get_agent_fuel_usage(agent_id), 0);
        assert_eq!(manager.get_total_memory_usage(), 0);
        assert_eq!(manager.get_total_fuel_usage(), 0);
    }

    #[test]
    fn test_record_message() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        let agent_id = Uuid::new_v4();

        // Allocate some memory to create the agent entry
        manager.allocate_memory(agent_id, 1).unwrap();

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
