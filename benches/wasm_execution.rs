//! # WebAssembly Execution Performance Benchmarks
//!
//! Benchmarks for measuring WASM execution performance including
//! instance creation, function calls, memory management, and pooling.

use caxton::performance::{wasm_runtime::OptimizedWasmRuntime, PerformanceMonitor};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark WASM function execution performance
fn bench_wasm_function_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("wasm_function_execution");
    group.measurement_time(Duration::from_secs(10));

    // Simple arithmetic function
    let arithmetic_wasm = wat::parse_str(
        r#"
        (module
            (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (func $multiply (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul
            )
            (func $factorial (param i32) (result i32)
                (local $result i32)
                (local $i i32)
                i32.const 1
                local.set $result
                i32.const 1
                local.set $i
                (loop $loop
                    local.get $i
                    local.get 0
                    i32.gt_s
                    br_if 1
                    local.get $result
                    local.get $i
                    i32.mul
                    local.set $result
                    local.get $i
                    i32.const 1
                    i32.add
                    local.set $i
                    br $loop
                )
                local.get $result
            )
            (export "add" (func $add))
            (export "multiply" (func $multiply))
            (export "factorial" (func $factorial))
        )
    "#,
    )
    .unwrap();

    // Simple function call
    group.bench_function("simple_add_function", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let monitor = Arc::new(PerformanceMonitor::new());
                let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                rt.block_on(async {
                    let module = runtime
                        .load_module("arithmetic", &arithmetic_wasm)
                        .await
                        .unwrap();
                    let mut instance = runtime
                        .get_instance(&module, "arithmetic_agent")
                        .await
                        .unwrap();
                    instance
                })
            },
            |mut instance| async move {
                let result = instance
                    .call_function::<(i32, i32), i32>("add", (42, 13))
                    .await;
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // More complex function with loops
    group.bench_function("factorial_function", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let monitor = Arc::new(PerformanceMonitor::new());
                let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                rt.block_on(async {
                    let module = runtime
                        .load_module("arithmetic", &arithmetic_wasm)
                        .await
                        .unwrap();
                    let mut instance = runtime
                        .get_instance(&module, "arithmetic_agent")
                        .await
                        .unwrap();
                    instance
                })
            },
            |mut instance| async move {
                let result = instance.call_function::<i32, i32>("factorial", 10).await;
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark WASM memory operations
fn bench_wasm_memory_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("wasm_memory_operations");
    group.measurement_time(Duration::from_secs(5));

    // WASM module with memory operations
    let memory_wasm = wat::parse_str(
        r#"
        (module
            (memory 1)
            (func $fill_memory (param $start i32) (param $len i32) (param $value i32)
                (local $i i32)
                (local.set $i (local.get $start))
                (loop $loop
                    (local.get $i)
                    (local.get $start)
                    (local.get $len)
                    i32.add
                    i32.ge_u
                    br_if 1

                    (local.get $i)
                    (local.get $value)
                    i32.store8

                    (local.get $i)
                    i32.const 1
                    i32.add
                    local.set $i
                    br $loop
                )
            )
            (func $sum_memory (param $start i32) (param $len i32) (result i32)
                (local $i i32)
                (local $sum i32)
                (local.set $i (local.get $start))
                (local.set $sum (i32.const 0))
                (loop $loop
                    (local.get $i)
                    (local.get $start)
                    (local.get $len)
                    i32.add
                    i32.ge_u
                    br_if 1

                    (local.get $sum)
                    (local.get $i)
                    i32.load8_u
                    i32.add
                    local.set $sum

                    (local.get $i)
                    i32.const 1
                    i32.add
                    local.set $i
                    br $loop
                )
                local.get $sum
            )
            (export "fill_memory" (func $fill_memory))
            (export "sum_memory" (func $sum_memory))
        )
    "#,
    )
    .unwrap();

    // Test different memory operation sizes
    for memory_size in [1024, 4096, 16384, 65536].iter() {
        group.throughput(Throughput::Bytes(*memory_size as u64));

        group.bench_with_input(
            BenchmarkId::new("fill_memory", memory_size),
            memory_size,
            |b, &memory_size| {
                b.to_async(&rt).iter_batched(
                    || {
                        let monitor = Arc::new(PerformanceMonitor::new());
                        let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                        rt.block_on(async {
                            let module = runtime
                                .load_module("memory_ops", &memory_wasm)
                                .await
                                .unwrap();
                            let mut instance =
                                runtime.get_instance(&module, "memory_agent").await.unwrap();
                            instance
                        })
                    },
                    |mut instance| async move {
                        let result = instance
                            .call_function::<(i32, i32, i32), ()>(
                                "fill_memory",
                                (0, memory_size as i32, 42),
                            )
                            .await;
                        black_box(result);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sum_memory", memory_size),
            memory_size,
            |b, &memory_size| {
                b.to_async(&rt).iter_batched(
                    || {
                        let monitor = Arc::new(PerformanceMonitor::new());
                        let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                        rt.block_on(async {
                            let module = runtime
                                .load_module("memory_ops", &memory_wasm)
                                .await
                                .unwrap();
                            let mut instance =
                                runtime.get_instance(&module, "memory_agent").await.unwrap();

                            // Pre-fill memory for sum test
                            let _ = instance
                                .call_function::<(i32, i32, i32), ()>(
                                    "fill_memory",
                                    (0, memory_size as i32, 1),
                                )
                                .await;

                            instance
                        })
                    },
                    |mut instance| async move {
                        let result = instance
                            .call_function::<(i32, i32), i32>("sum_memory", (0, memory_size as i32))
                            .await;
                        black_box(result);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark instance lifecycle (creation and destruction)
fn bench_instance_lifecycle(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("instance_lifecycle");
    group.measurement_time(Duration::from_secs(5));

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

    // Test instance creation (cold start)
    group.bench_function("create_new_instance", |b| {
        b.to_async(&rt).iter(|| async {
            let monitor = Arc::new(PerformanceMonitor::new());
            let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
            let module = runtime
                .load_module("lifecycle_test", &simple_wasm)
                .await
                .unwrap();

            let instance = runtime.get_instance(&module, "test_agent").await;
            black_box(instance);
        });
    });

    // Test instance reuse from pool (warm start)
    group.bench_function("reuse_pooled_instance", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let monitor = Arc::new(PerformanceMonitor::new());
                let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                rt.block_on(async {
                    let module = runtime
                        .load_module("lifecycle_test", &simple_wasm)
                        .await
                        .unwrap();

                    // Pre-warm the pool
                    let instance = runtime.get_instance(&module, "test_agent").await.unwrap();
                    runtime.return_instance(instance).await;

                    (runtime, module)
                })
            },
            |(runtime, module)| async move {
                let instance = runtime.get_instance(&module, "test_agent").await;
                black_box(instance);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Test multiple instances with different agent types
    group.bench_function("multiple_agent_types", |b| {
        b.to_async(&rt).iter(|| async {
            let monitor = Arc::new(PerformanceMonitor::new());
            let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
            let module = runtime
                .load_module("multi_agent", &simple_wasm)
                .await
                .unwrap();

            let mut instances = Vec::new();

            // Create instances for different agent types
            for i in 0..10 {
                let agent_type = format!("agent_type_{}", i);
                let instance = runtime.get_instance(&module, &agent_type).await.unwrap();
                instances.push(instance);
            }

            black_box(instances);
        });
    });

    group.finish();
}

/// Benchmark concurrent WASM execution
fn bench_concurrent_wasm_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_wasm_execution");
    group.measurement_time(Duration::from_secs(10));

    let concurrent_wasm = wat::parse_str(
        r#"
        (module
            (func $cpu_intensive (param $iterations i32) (result i32)
                (local $i i32)
                (local $result i32)
                (local.set $i (i32.const 0))
                (local.set $result (i32.const 0))
                (loop $loop
                    (local.get $i)
                    (local.get $iterations)
                    i32.ge_s
                    br_if 1

                    (local.get $result)
                    (local.get $i)
                    i32.add
                    local.set $result

                    (local.get $i)
                    i32.const 1
                    i32.add
                    local.set $i
                    br $loop
                )
                local.get $result
            )
            (export "cpu_intensive" (func $cpu_intensive))
        )
    "#,
    )
    .unwrap();

    // Test with different concurrency levels
    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_execution", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let monitor = Arc::new(PerformanceMonitor::new());
                    let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                    let module = runtime
                        .load_module("concurrent_test", &concurrent_wasm)
                        .await
                        .unwrap();

                    let mut handles = Vec::new();

                    // Spawn concurrent executions
                    for i in 0..concurrency {
                        let runtime = &runtime;
                        let module = &module;

                        let handle = tokio::spawn(async move {
                            let mut instance = runtime
                                .get_instance(module, &format!("concurrent_agent_{}", i))
                                .await
                                .unwrap();
                            let result = instance
                                .call_function::<i32, i32>("cpu_intensive", 1000)
                                .await;
                            result
                        });

                        handles.push(handle);
                    }

                    // Wait for all executions to complete
                    let results: Vec<_> = futures::future::join_all(handles).await;
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark WASM module size impact on performance
fn bench_module_size_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("module_size_impact");
    group.measurement_time(Duration::from_secs(5));

    // Generate WASM modules of different sizes
    let small_module = wat::parse_str(
        r#"
        (module
            (func $small (result i32) i32.const 42)
            (export "small" (func $small))
        )
    "#,
    )
    .unwrap();

    let medium_module = wat::parse_str(&format!(
        r#"
        (module
            {}
            (func $main (result i32)
                call $func_50
            )
            (export "main" (func $main))
        )
    "#,
        (0..100)
            .map(|i| format!("(func $func_{} (result i32) i32.const {})", i, i))
            .collect::<Vec<_>>()
            .join("\n")
    ))
    .unwrap();

    // Test module loading performance by size
    group.bench_function("load_small_module", |b| {
        b.to_async(&rt).iter(|| async {
            let monitor = Arc::new(PerformanceMonitor::new());
            let runtime = OptimizedWasmRuntime::new(monitor).unwrap();

            let module = runtime.load_module("small_module", &small_module).await;
            black_box(module);
        });
    });

    group.bench_function("load_medium_module", |b| {
        b.to_async(&rt).iter(|| async {
            let monitor = Arc::new(PerformanceMonitor::new());
            let runtime = OptimizedWasmRuntime::new(monitor).unwrap();

            let module = runtime.load_module("medium_module", &medium_module).await;
            black_box(module);
        });
    });

    // Test instantiation performance by size
    group.bench_function("instantiate_small_module", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let monitor = Arc::new(PerformanceMonitor::new());
                let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                rt.block_on(async {
                    let module = runtime
                        .load_module("small_module", &small_module)
                        .await
                        .unwrap();
                    (runtime, module)
                })
            },
            |(runtime, module)| async move {
                let instance = runtime.get_instance(&module, "small_agent").await;
                black_box(instance);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("instantiate_medium_module", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let monitor = Arc::new(PerformanceMonitor::new());
                let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
                rt.block_on(async {
                    let module = runtime
                        .load_module("medium_module", &medium_module)
                        .await
                        .unwrap();
                    (runtime, module)
                })
            },
            |(runtime, module)| async move {
                let instance = runtime.get_instance(&module, "medium_agent").await;
                black_box(instance);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_wasm_function_execution,
    bench_wasm_memory_operations,
    bench_instance_lifecycle,
    bench_concurrent_wasm_execution,
    bench_module_size_impact
);

criterion_main!(benches);
