//! Statistical calculation utilities with explicit precision loss handling
//!
//! This module provides functions for common statistical calculations where precision loss
//! is acceptable and expected. All precision loss is explicitly documented and controlled.
//!
//! Note: Precision loss warnings are explicitly suppressed in this module as it is designed
//! to handle statistical conversions where precision loss is acceptable for display purposes.

/// Calculate a percentage from two u64 values, returning f32 for display purposes.
///
/// Precision loss is acceptable for percentage display values.
/// Values are computed using f64 precision and then truncated to f32.
pub fn calculate_percentage_f32(numerator: u64, denominator: u64) -> f32 {
    if denominator == 0 {
        return 0.0;
    }

    // Use f64 for calculation to maximize precision before final conversion
    let num_f64 = numerator as f64; // Accept precision loss for statistical display
    let den_f64 = denominator as f64; // Accept precision loss for statistical display
    let result = (num_f64 / den_f64) * 100.0;

    // Clamp to f32 range and cast - acceptable truncation for percentage display
    result.clamp(f64::from(f32::MIN), f64::from(f32::MAX)) as f32
}

/// Convert timing values from u128 milliseconds to f64, handling very large values.
///
/// Precision loss is acceptable for timing statistics and metrics.
pub fn millis_to_f64_for_stats(millis: u128) -> f64 {
    if let Ok(millis_u64) = u64::try_from(millis) {
        // For values that fit in u64, precision loss in f64 conversion is acceptable for timing statistics
        millis_u64 as f64
    } else {
        // For extremely long times, precision loss is acceptable for statistical purposes
        millis as f64
    }
}

/// Convert u64 to f64 for statistical calculations.
///
/// Precision loss is explicitly acceptable for statistical metrics and displays.
pub fn u64_to_f64_for_stats(value: u64) -> f64 {
    // For statistical calculations, precision loss in u64->f64 conversion is acceptable
    value as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentage_f32() {
        assert_eq!(calculate_percentage_f32(1, 4), 25.0);
        assert_eq!(calculate_percentage_f32(0, 100), 0.0);
        assert_eq!(calculate_percentage_f32(100, 0), 0.0); // Handle division by zero
    }

    #[test]
    fn test_millis_to_f64_for_stats() {
        // Small values should be exact
        assert_eq!(millis_to_f64_for_stats(1000), 1000.0);

        // Large values should not panic
        let large_value = u128::MAX;
        let result = millis_to_f64_for_stats(large_value);
        assert!(result.is_finite());
    }

    #[test]
    fn test_u64_to_f64_for_stats() {
        assert_eq!(u64_to_f64_for_stats(42), 42.0);

        // Large values should not panic
        let large_value = u64::MAX;
        let result = u64_to_f64_for_stats(large_value);
        assert!(result.is_finite());
    }
}
