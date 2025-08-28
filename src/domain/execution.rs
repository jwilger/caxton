use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
pub struct FuelConsumed(u64);

impl FuelConsumed {
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::try_new(self.into_inner() + other.into_inner()).unwrap_or_default()
    }
}

#[nutype(
    validate(predicate = |v: &Vec<u8>| v.len() <= 10_485_760),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default),
    default = Vec::new()
)]
pub struct ExecutionOutput(Vec<u8>);

impl ExecutionOutput {
    #[must_use]
    pub fn empty() -> Self {
        Self::try_new(Vec::new()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failure { reason: FailureReason },
    Timeout { elapsed: ElapsedTime },
}

#[nutype(
    validate(len_char_min = 1, len_char_max = 1000),
    derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)
)]
pub struct FailureReason(String);

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
pub struct ElapsedTime(u64);

impl ElapsedTime {
    #[must_use]
    pub fn from_duration(duration: Duration) -> Self {
        let millis = duration.as_millis();
        // Safe conversion: as_millis() returns u128, we need u64
        // If millis exceeds u64::MAX, we use the maximum representable value
        let millis_u64 = u64::try_from(millis).unwrap_or(u64::MAX);
        Self::try_new(millis_u64).unwrap_or_default()
    }

    #[must_use]
    pub fn meets_minimum(&self, minimum_ms: u64) -> bool {
        self.into_inner() >= minimum_ms
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub status: ExecutionStatus,
    pub fuel_consumed: FuelConsumed,
    pub output: Option<ExecutionOutput>,
}

impl ExecutionResult {
    #[must_use]
    pub fn success(fuel_consumed: FuelConsumed, output: Option<ExecutionOutput>) -> Self {
        Self {
            status: ExecutionStatus::Success,
            fuel_consumed,
            output,
        }
    }

    #[must_use]
    pub fn failure(reason: FailureReason, fuel_consumed: FuelConsumed) -> Self {
        Self {
            status: ExecutionStatus::Failure { reason },
            fuel_consumed,
            output: None,
        }
    }

    #[must_use]
    pub fn timeout(elapsed: ElapsedTime, fuel_consumed: FuelConsumed) -> Self {
        Self {
            status: ExecutionStatus::Timeout { elapsed },
            fuel_consumed,
            output: None,
        }
    }
}
