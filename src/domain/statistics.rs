//! Statistical calculation utilities using high-precision decimal arithmetic
//!
//! This module provides functions for statistical calculations using `rust_decimal`
//! to avoid floating-point precision issues entirely. Results are converted to
//! f32/f64 only at the final step for display purposes.

use rust_decimal::prelude::*;

/// Calculate a percentage from two u64 values, returning f32 for display purposes.
///
/// Uses Decimal arithmetic internally to maintain precision, converting to f32
/// only for the final display value.
#[must_use]
pub fn calculate_percentage_f32(numerator: u64, denominator: u64) -> f32 {
    if denominator == 0 {
        return 0.0;
    }

    // Use Decimal for precise calculation
    let num_decimal = Decimal::from(numerator);
    let den_decimal = Decimal::from(denominator);
    let hundred = Decimal::from(100);

    let result = (num_decimal / den_decimal) * hundred;

    // Convert to f32 for display - this is the only place we lose precision,
    // and it's acceptable for display purposes
    result.to_f32().unwrap_or(0.0)
}

/// Convert timing values from u128 milliseconds to f64, handling very large values.
///
/// Uses Decimal for large value handling, converting to f64 only for display.
#[must_use]
pub fn millis_to_f64_for_stats(millis: u128) -> f64 {
    // Try to create a Decimal from u128
    // For extremely large values that don't fit, use f64::MAX as a sentinel
    Decimal::from_u128(millis)
        .and_then(|d| d.to_f64())
        .unwrap_or(f64::MAX)
}

/// Convert u64 to f64 for statistical calculations.
///
/// Uses Decimal as intermediate to maintain precision as long as possible.
#[must_use]
pub fn u64_to_f64_for_stats(value: u64) -> f64 {
    Decimal::from(value).to_f64().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_calculate_percentage_f32() {
        assert_relative_eq!(calculate_percentage_f32(1, 4), 25.0, epsilon = 0.0001);
        assert_relative_eq!(calculate_percentage_f32(0, 100), 0.0, epsilon = 0.0001);
        assert_relative_eq!(calculate_percentage_f32(100, 0), 0.0, epsilon = 0.0001); // Handle division by zero
    }

    #[test]
    fn test_millis_to_f64_for_stats() {
        // Small values should be exact within epsilon
        assert_relative_eq!(millis_to_f64_for_stats(1000), 1000.0, epsilon = 0.0001);

        // Large values should not panic
        let large_value = u128::MAX;
        let result = millis_to_f64_for_stats(large_value);
        assert!(result.is_finite());
    }

    #[test]
    fn test_u64_to_f64_for_stats() {
        assert_relative_eq!(u64_to_f64_for_stats(42), 42.0, epsilon = 0.0001);

        // Large values should not panic
        let large_value = u64::MAX;
        let result = u64_to_f64_for_stats(large_value);
        assert!(result.is_finite());
    }
}
