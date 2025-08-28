//! Time abstraction layer for testable time-dependent operations
//!
//! This module provides a `TimeProvider` trait that allows for mocking time
//! in tests while using real time in production, without conditional compilation.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::time::sleep as tokio_sleep;

/// Trait for providing time-related functionality
///
/// This allows for dependency injection of time behavior,
/// enabling fast test execution without real-time delays.
#[async_trait]
pub trait TimeProvider: Send + Sync + std::fmt::Debug {
    /// Sleep for the specified duration
    async fn sleep(&self, duration: Duration);

    /// Get the current system time
    #[must_use]
    fn now(&self) -> SystemTime;

    /// Get the current instant for measuring elapsed time
    #[must_use]
    fn instant(&self) -> Instant;

    /// Check if we should skip delays (for testing)
    #[must_use]
    fn should_skip_delays(&self) -> bool {
        false
    }
}

/// Real time provider for production use
#[derive(Debug, Clone, Default)]
pub struct RealTimeProvider;

impl RealTimeProvider {
    /// Creates a new real time provider
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TimeProvider for RealTimeProvider {
    async fn sleep(&self, duration: Duration) {
        tokio_sleep(duration).await;
    }

    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn instant(&self) -> Instant {
        Instant::now()
    }
}

/// Mock time provider for testing
///
/// This provider skips all sleeps and timeouts, allowing tests
/// to run at full speed without waiting for real time to pass.
#[derive(Debug, Clone, Default)]
pub struct MockTimeProvider {
    skip_delays: bool,
}

impl MockTimeProvider {
    /// Creates a new mock time provider that skips delays
    #[must_use]
    pub fn new() -> Self {
        Self { skip_delays: true }
    }

    /// Creates a mock time provider that uses real delays (for integration tests)
    #[must_use]
    pub fn with_real_delays() -> Self {
        Self { skip_delays: false }
    }
}

#[async_trait]
impl TimeProvider for MockTimeProvider {
    async fn sleep(&self, duration: Duration) {
        if !self.skip_delays {
            // For integration tests that need real timing
            tokio_sleep(duration).await;
        } else if duration > Duration::from_millis(1) {
            // Sleep for at most 1ms in tests to ensure async operations can yield
            tokio_sleep(Duration::from_millis(1)).await;
        }
    }

    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn instant(&self) -> Instant {
        Instant::now()
    }

    fn should_skip_delays(&self) -> bool {
        self.skip_delays
    }
}

/// Type alias for shared time provider
pub type SharedTimeProvider = Arc<dyn TimeProvider>;

/// Create a production time provider
#[must_use]
pub fn production_time_provider() -> SharedTimeProvider {
    Arc::new(RealTimeProvider::new())
}

/// Create a test time provider that skips delays
#[must_use]
pub fn test_time_provider() -> SharedTimeProvider {
    Arc::new(MockTimeProvider::new())
}

/// Create a test time provider with real delays (for integration tests)
#[must_use]
pub fn integration_test_time_provider() -> SharedTimeProvider {
    Arc::new(MockTimeProvider::with_real_delays())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_time_provider_skips_delays() {
        let provider = MockTimeProvider::new();
        let start = Instant::now();

        // This should complete almost instantly
        provider.sleep(Duration::from_secs(10)).await;

        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(100),
            "Mock sleep took too long: {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_real_time_provider_actually_sleeps() {
        let provider = RealTimeProvider::new();
        let start = Instant::now();

        // This should take at least 50ms
        provider.sleep(Duration::from_millis(50)).await;

        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(50),
            "Real sleep was too short: {elapsed:?}"
        );
    }
}
