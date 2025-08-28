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
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }

    #[must_use]
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
    #[must_use]
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            state: Uninitialized,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn get_memory_usage(&self) -> MemoryBytes {
        MemoryBytes::zero()
    }

    #[must_use]
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
    #[must_use]
    pub fn get_memory_usage(&self) -> MemoryBytes {
        self.state.memory_allocated
    }

    #[must_use]
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
    #[must_use]
    pub fn get_memory_usage(&self) -> MemoryBytes {
        self.state.memory_allocated
    }

    /// Consume fuel from the sandbox's fuel tracker
    ///
    /// # Errors
    ///
    /// Returns `FuelError` if there is insufficient fuel.
    pub fn consume_fuel(&mut self, amount: u64) -> Result<(), super::fuel::FuelError> {
        self.state.fuel_tracker.consume(amount)?;
        Ok(())
    }

    #[must_use]
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

    #[must_use]
    pub fn stop(self) -> Sandbox<Stopped> {
        Sandbox {
            id: self.id,
            state: Stopped,
            _phantom: PhantomData,
        }
    }
}

impl Sandbox<Draining> {
    #[must_use]
    pub fn get_memory_usage(&self) -> MemoryBytes {
        self.state.memory_allocated
    }

    #[must_use]
    pub fn process_message(&mut self) -> Option<MessageCount> {
        self.state.messages_remaining.decrement().inspect(|&count| {
            self.state.messages_remaining = count;
        })
    }

    #[must_use]
    pub fn is_drained(&self) -> bool {
        self.state.messages_remaining.into_inner() == 0
    }

    #[must_use]
    pub fn stop(self) -> Sandbox<Stopped> {
        Sandbox {
            id: self.id,
            state: Stopped,
            _phantom: PhantomData,
        }
    }
}

impl Sandbox<Stopped> {
    #[must_use]
    pub fn get_memory_usage(&self) -> MemoryBytes {
        MemoryBytes::zero()
    }

    #[must_use]
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
    #[must_use]
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
