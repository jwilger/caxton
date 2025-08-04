//! Performance benchmarking module

use std::time::Duration;
use tracing::{info, instrument};

/// Simple benchmark runner
pub struct BenchmarkRunner {
    name: String,
}

impl BenchmarkRunner {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Run a benchmark function multiple times
    #[instrument(skip(self, f))]
    pub fn run<F>(&self, iterations: usize, f: F) -> BenchmarkResult
    where
        F: Fn() -> Duration,
    {
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            durations.push(f());
        }

        let total: Duration = durations.iter().sum();
        let mean = total / iterations as u32;

        durations.sort();
        let p50 = durations[iterations / 2];
        let p95 = durations[(iterations as f64 * 0.95) as usize];
        let p99 = durations[(iterations as f64 * 0.99) as usize];

        let result = BenchmarkResult {
            name: self.name.clone(),
            iterations,
            mean,
            p50,
            p95,
            p99,
        };

        info!(
            benchmark = %result.name,
            iterations = result.iterations,
            mean_ms = result.mean.as_millis(),
            p95_ms = result.p95.as_millis(),
            "Benchmark completed"
        );

        result
    }
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}
