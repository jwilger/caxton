#[allow(unused_imports)]
use crate::domain_types::{AgentId, MemoryBytes};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use thiserror::Error;

pub const AGENT_MEMORY_LIMIT: usize = 10_485_760;
pub const TOTAL_MEMORY_LIMIT: usize = 104_857_600;

#[nutype(
    validate(greater = 0, less_or_equal = AGENT_MEMORY_LIMIT),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)
)]
pub struct AgentMemoryRequest(usize);

impl AgentMemoryRequest {
    pub fn from_mb(mb: usize) -> Result<Self, AgentMemoryRequestError> {
        Self::try_new(mb * 1024 * 1024)
    }
}

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = TOTAL_MEMORY_LIMIT),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default),
    default = 0
)]
pub struct TotalMemoryAllocated(usize);

impl TotalMemoryAllocated {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn add(&self, amount: AgentMemoryRequest) -> Result<Self, MemoryError> {
        let new_total = self.into_inner() + amount.into_inner();
        Self::try_new(new_total).map_err(|_| MemoryError::TotalLimitExceeded {
            requested: amount.into_inner(),
            current: self.into_inner(),
            limit: TOTAL_MEMORY_LIMIT,
        })
    }

    pub fn subtract(&self, amount: usize) -> Self {
        let current = self.into_inner();
        if amount > current {
            Self::zero()
        } else {
            Self::try_new(current - amount).unwrap_or_default()
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum MemoryError {
    #[error("Agent memory limit exceeded: requested {requested}, limit {limit}")]
    AgentLimitExceeded { requested: usize, limit: usize },

    #[error("Total memory limit exceeded: requested {requested}, current {current}, limit {limit}")]
    TotalLimitExceeded {
        requested: usize,
        current: usize,
        limit: usize,
    },

    #[error("Agent {agent:?} not found")]
    AgentNotFound { agent: AgentId },

    #[error("Agent {agent:?} already has allocation")]
    AgentAlreadyAllocated { agent: AgentId },
}

pub struct BoundedMemoryPool {
    allocations: HashMap<AgentId, AgentMemoryRequest>,
    total_allocated: TotalMemoryAllocated,
}

impl BoundedMemoryPool {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            total_allocated: TotalMemoryAllocated::zero(),
        }
    }

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

    pub fn deallocate(&mut self, agent_id: AgentId) -> Result<AgentMemoryRequest, MemoryError> {
        let allocation = self
            .allocations
            .remove(&agent_id)
            .ok_or(MemoryError::AgentNotFound { agent: agent_id })?;

        self.total_allocated = self.total_allocated.subtract(allocation.into_inner());

        Ok(allocation)
    }

    pub fn get_allocation(&self, agent_id: &AgentId) -> Option<AgentMemoryRequest> {
        self.allocations.get(agent_id).copied()
    }

    pub fn total_allocated(&self) -> TotalMemoryAllocated {
        self.total_allocated
    }

    pub fn available_memory(&self) -> usize {
        TOTAL_MEMORY_LIMIT - self.total_allocated.into_inner()
    }
}

pub struct ResourceLimits<const MAX_MEM: usize, const MAX_FUEL: u64> {
    max_memory_bytes: PhantomData<[u8; MAX_MEM]>,
    max_cpu_fuel: PhantomData<u64>,
}

impl<const MAX_MEM: usize, const MAX_FUEL: u64> ResourceLimits<MAX_MEM, MAX_FUEL> {
    pub const fn new() -> Self {
        Self {
            max_memory_bytes: PhantomData,
            max_cpu_fuel: PhantomData,
        }
    }

    pub const fn max_memory() -> usize {
        MAX_MEM
    }

    pub const fn max_fuel() -> u64 {
        MAX_FUEL
    }
}

pub type DefaultResourceLimits = ResourceLimits<{ 10 * 1024 * 1024 }, 1_000_000>;
pub type TestResourceLimits = ResourceLimits<{ 1024 * 1024 }, 10_000>;

pub struct WasmRuntimeConfig<L> {
    resource_limits: L,
}

impl<const MAX_MEM: usize, const MAX_FUEL: u64>
    WasmRuntimeConfig<ResourceLimits<MAX_MEM, MAX_FUEL>>
{
    pub fn new() -> Self {
        Self {
            resource_limits: ResourceLimits::new(),
        }
    }

    pub fn max_memory(&self) -> usize {
        MAX_MEM
    }

    pub fn max_fuel(&self) -> u64 {
        MAX_FUEL
    }
}
