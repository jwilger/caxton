//! Utility functions

use crate::*;

/// Utility functions for Caxton
pub fn format_duration(duration: Duration) -> String {
    format!("{:.2}s", duration.as_secs_f64())
}
