use nutype::nutype;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[nutype(
    validate(greater = 0),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize
    )
)]
pub struct NonZeroCpuFuel(u64);

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
        Default
    ),
    default = 0
)]
pub struct CpuFuelBudget(u64);

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
        Default
    ),
    default = 0
)]
pub struct CpuFuelRemaining(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuelState {
    Available { remaining: NonZeroCpuFuel },
    Exhausted,
}

impl FuelState {
    #[must_use]
    pub fn from_remaining(remaining: u64) -> Self {
        NonZeroCpuFuel::try_new(remaining)
            .map(|fuel| FuelState::Available { remaining: fuel })
            .unwrap_or(FuelState::Exhausted)
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        matches!(self, FuelState::Available { .. })
    }

    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        matches!(self, FuelState::Exhausted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuelTracker {
    budget: CpuFuelBudget,
    remaining: CpuFuelRemaining,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum FuelError {
    #[error("Insufficient fuel: requested {requested}, available {available}")]
    InsufficientFuel { requested: u64, available: u64 },

    #[error("Fuel already exhausted")]
    FuelExhausted,
}

impl FuelTracker {
    #[must_use]
    pub fn new(budget: CpuFuelBudget) -> Self {
        let remaining = CpuFuelRemaining::try_new(budget.into_inner()).unwrap_or_default();
        Self { budget, remaining }
    }

    /// Consumes fuel from the remaining budget
    ///
    /// # Errors
    ///
    /// Returns `FuelError` if there is insufficient fuel or if fuel is already exhausted.
    pub fn consume(&mut self, amount: u64) -> Result<CpuFuelRemaining, FuelError> {
        let current = self.remaining.into_inner();

        if current == 0 {
            return Err(FuelError::FuelExhausted);
        }

        if amount > current {
            return Err(FuelError::InsufficientFuel {
                requested: amount,
                available: current,
            });
        }

        self.remaining = CpuFuelRemaining::try_new(current - amount).unwrap_or_default();
        Ok(self.remaining)
    }

    #[must_use]
    pub fn consumed(&self) -> u64 {
        self.budget.into_inner() - self.remaining.into_inner()
    }

    #[must_use]
    pub fn remaining(&self) -> CpuFuelRemaining {
        self.remaining
    }

    #[must_use]
    pub fn budget(&self) -> CpuFuelBudget {
        self.budget
    }

    #[must_use]
    pub fn state(&self) -> FuelState {
        FuelState::from_remaining(self.remaining.into_inner())
    }
}

pub struct ExecutionContext<F> {
    pub fuel_state: F,
}

impl ExecutionContext<FuelState> {
    #[must_use]
    pub fn new(fuel: u64) -> Self {
        Self {
            fuel_state: FuelState::from_remaining(fuel),
        }
    }
}

impl ExecutionContext<NonZeroCpuFuel> {
    /// Executes a function with the given fuel cost
    ///
    /// # Errors
    ///
    /// Returns `FuelError` if there is insufficient fuel or if fuel is exhausted.
    pub fn execute_with_fuel(&mut self, _function: &str, cost: u64) -> Result<(), FuelError> {
        let current = self.fuel_state.into_inner();
        if cost > current {
            return Err(FuelError::InsufficientFuel {
                requested: cost,
                available: current,
            });
        }

        let remaining = current - cost;
        self.fuel_state =
            NonZeroCpuFuel::try_new(remaining).map_err(|_| FuelError::FuelExhausted)?;

        Ok(())
    }
}
