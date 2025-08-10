use super::fuel::FuelTracker;
use crate::domain_types::{AgentId, MemoryBytes};
use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uninitialized;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Initialized {
    pub memory_allocated: MemoryBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Running {
    pub memory_allocated: MemoryBytes,
    pub fuel_tracker: FuelTracker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draining {
    pub memory_allocated: MemoryBytes,
    pub messages_remaining: MessageCount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stopped;

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 1000),
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
        Default
    ),
    default = 0
)]
pub struct MessageCount(u32);

impl MessageCount {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn decrement(&self) -> Option<Self> {
        let current = self.into_inner();
        if current > 0 {
            Some(Self::try_new(current - 1).unwrap_or_default())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sandbox<S> {
    id: AgentId,
    state: S,
    _phantom: PhantomData<S>,
}

impl Sandbox<Uninitialized> {
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            state: Uninitialized,
            _phantom: PhantomData,
        }
    }

    pub fn get_memory_usage(&self) -> MemoryBytes {
        MemoryBytes::zero()
    }

    pub fn initialize(self, memory: MemoryBytes) -> Sandbox<Initialized> {
        Sandbox {
            id: self.id,
            state: Initialized {
                memory_allocated: memory,
            },
            _phantom: PhantomData,
        }
    }
}

impl Sandbox<Initialized> {
    pub fn get_memory_usage(&self) -> MemoryBytes {
        self.state.memory_allocated
    }

    pub fn start(self, fuel_tracker: FuelTracker) -> Sandbox<Running> {
        Sandbox {
            id: self.id,
            state: Running {
                memory_allocated: self.state.memory_allocated,
                fuel_tracker,
            },
            _phantom: PhantomData,
        }
    }
}

impl Sandbox<Running> {
    pub fn get_memory_usage(&self) -> MemoryBytes {
        self.state.memory_allocated
    }

    pub fn consume_fuel(&mut self, amount: u64) -> Result<(), super::fuel::FuelError> {
        self.state.fuel_tracker.consume(amount)?;
        Ok(())
    }

    pub fn start_draining(self, message_count: MessageCount) -> Sandbox<Draining> {
        Sandbox {
            id: self.id,
            state: Draining {
                memory_allocated: self.state.memory_allocated,
                messages_remaining: message_count,
            },
            _phantom: PhantomData,
        }
    }

    pub fn stop(self) -> Sandbox<Stopped> {
        Sandbox {
            id: self.id,
            state: Stopped,
            _phantom: PhantomData,
        }
    }
}

impl Sandbox<Draining> {
    pub fn get_memory_usage(&self) -> MemoryBytes {
        self.state.memory_allocated
    }

    pub fn process_message(&mut self) -> Option<MessageCount> {
        self.state.messages_remaining.decrement().map(|count| {
            self.state.messages_remaining = count;
            count
        })
    }

    pub fn is_drained(&self) -> bool {
        self.state.messages_remaining.into_inner() == 0
    }

    pub fn stop(self) -> Sandbox<Stopped> {
        Sandbox {
            id: self.id,
            state: Stopped,
            _phantom: PhantomData,
        }
    }
}

impl Sandbox<Stopped> {
    pub fn get_memory_usage(&self) -> MemoryBytes {
        MemoryBytes::zero()
    }

    pub fn get_id(&self) -> AgentId {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxState {
    Uninitialized(Sandbox<Uninitialized>),
    Initialized(Sandbox<Initialized>),
    Running(Sandbox<Running>),
    Draining(Sandbox<Draining>),
    Stopped(Sandbox<Stopped>),
}

impl SandboxState {
    pub fn get_memory_usage(&self) -> MemoryBytes {
        match self {
            SandboxState::Uninitialized(s) => s.get_memory_usage(),
            SandboxState::Initialized(s) => s.get_memory_usage(),
            SandboxState::Running(s) => s.get_memory_usage(),
            SandboxState::Draining(s) => s.get_memory_usage(),
            SandboxState::Stopped(s) => s.get_memory_usage(),
        }
    }
}
