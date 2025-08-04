//! # Agent Spawn Performance Benchmarks
//!
//! Benchmarks for measuring agent spawning performance across different
//! optimization strategies and configuration parameters.

use caxton::performance::{wasm_runtime::OptimizedWasmRuntime, PerformanceMonitor};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark agent spawning with different configurations
fn bench_agent_spawn_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_spawn");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Test with different agent counts
    for agent_count in [1, 10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("spawn_agents", agent_count),
            agent_count,
            |b, &agent_count| {
                b.to_async(&rt).iter(|| async {
                    let monitor = Arc::new(PerformanceMonitor::new());
                    let runtime = OptimizedWasmRuntime::new(monitor).unwrap();

                    // Simulate spawning multiple agents
                    for i in 0..agent_count {
                        // In a real benchmark, this would load actual WASM modules
                        // and create instances. For now, we simulate the overhead.
                        let agent_type = format!("test_agent_{}", i);
                        black_box(agent_type);

                        // Simulate WASM module loading time
                        tokio::task::yield_now().await;
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark WASM module loading and compilation
fn bench_wasm_module_loading(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("wasm_module_loading");
    group.measurement_time(Duration::from_secs(5));

    // Simple WASM module for testing (minimal "hello world" module)
    let simple_wasm = wat::parse_str(
        r#"
        (module
            (func $hello (result i32)
                i32.const 42
            )
            (export "hello" (func $hello))
        )
    "#,
    )
    .unwrap();

    group.bench_function("load_simple_module", |b| {
        b.to_async(&rt).iter(|| async {
            let monitor = Arc::new(PerformanceMonitor::new());
            let runtime = OptimizedWasmRuntime::new(monitor).unwrap();

            let result = runtime.load_module("test_module", &simple_wasm).await;
            black_box(result);
        });
    });

    // Test with cached module loading
    group.bench_function("load_cached_module", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let monitor = Arc::new(PerformanceMonitor::new());
                let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                rt.block_on(async {
                    // Pre-load the module to cache it
                    runtime
                        .load_module("cached_module", &simple_wasm)
                        .await
                        .unwrap();
                    runtime
                })
            },
            |runtime| async move {
                // Load the same module again (should hit cache)
                let result = runtime.load_module("cached_module", &simple_wasm).await;
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark instance pooling performance
fn bench_instance_pooling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("instance_pooling");
    group.measurement_time(Duration::from_secs(5));

    let simple_wasm = wat::parse_str(
        r#"
        (module
            (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (export "add" (func $add))
        )
    "#,
    )
    .unwrap();

    group.bench_function("get_new_instance", |b| {
        b.to_async(&rt).iter(|| async {
            let monitor = Arc::new(PerformanceMonitor::new());
            let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
            let module = runtime
                .load_module("pool_test", &simple_wasm)
                .await
                .unwrap();

            // Get instance (will create new since pool is empty)
            let instance = runtime.get_instance(&module, "test_agent").await;
            black_box(instance);
        });
    });

    group.bench_function("get_pooled_instance", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let monitor = Arc::new(PerformanceMonitor::new());
                let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                rt.block_on(async {
                    let module = runtime
                        .load_module("pool_test", &simple_wasm)
                        .await
                        .unwrap();

                    // Pre-warm the pool by creating and returning an instance
                    let instance = runtime.get_instance(&module, "test_agent").await.unwrap();
                    runtime.return_instance(instance).await;

                    (runtime, module)
                })
            },
            |(runtime, module)| async move {
                // Get instance from pool (should be faster)
                let instance = runtime.get_instance(&module, "test_agent").await;
                black_box(instance);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    group.measurement_time(Duration::from_secs(3));

    // Test different allocation patterns
    group.bench_function("frequent_small_allocations", |b| {
        b.iter(|| {
            let mut vectors = Vec::new();
            for i in 0..1000 {
                let vec = vec![i; 10]; // Small allocations
                vectors.push(black_box(vec));
            }
        });
    });

    group.bench_function("infrequent_large_allocations", |b| {
        b.iter(|| {
            let mut vectors = Vec::new();
            for i in 0..10 {
                let vec = vec![i; 10000]; // Large allocations
                vectors.push(black_box(vec));
            }
        });
    });

    group.bench_function("pooled_allocation_simulation", |b| {
        let rt = Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            let pool = caxton::performance::memory_tracking::MemoryPool::new(100);

            // Get and return objects to simulate pooling
            for _ in 0..100 {
                let item = pool.get(|| vec![0u8; 1024]).await;
                pool.return_item(item).await;
            }
        });
    });

    group.finish();
}

/// Benchmark performance monitoring overhead
fn bench_performance_monitoring(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("performance_monitoring");
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("monitor_overhead", |b| {
        b.to_async(&rt).iter(|| async {
            let monitor = PerformanceMonitor::new();

            // Simulate recording various metrics
            for i in 0..100 {
                monitor.increment_counter("test_counter", 1).await;
                monitor.set_gauge("test_gauge", i as f64).await;
                monitor
                    .record_histogram("test_histogram", Duration::from_micros(i * 10))
                    .await;
            }
        });
    });

    group.bench_function("baseline_without_monitoring", |b| {
        b.iter(|| {
            // Same work without monitoring
            for i in 0..100 {
                let _counter = black_box(i);
                let _gauge = black_box(i as f64);
                let _histogram = black_box(Duration::from_micros(i * 10));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_agent_spawn_performance,
    bench_wasm_module_loading,
    bench_instance_pooling,
    bench_memory_allocation,
    bench_performance_monitoring
);

criterion_main!(benches);
