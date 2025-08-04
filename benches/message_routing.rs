//! # FIPA Message Routing Performance Benchmarks
//!
//! Benchmarks for measuring FIPA message routing performance, including
//! serialization, batching, and routing table operations.

use bytes::Bytes;
use caxton::performance::message_routing::{
    AgentId, FipaMessage, MessagePriority, OptimizedMessageRouter, Performative, RouterConfig,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Generate test FIPA messages
fn generate_test_message(sender: AgentId, receiver: AgentId, content_size: usize) -> FipaMessage {
    FipaMessage {
        performative: Performative::Inform,
        sender,
        receiver,
        conversation_id: Uuid::new_v4(),
        reply_with: Some("test_reply".to_string()),
        in_reply_to: None,
        content: Bytes::from(vec![0u8; content_size]),
        ontology: Some("test_ontology".to_string()),
        language: Some("english".to_string()),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        priority: MessagePriority::Normal,
    }
}

/// Benchmark message routing performance
fn bench_message_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("message_routing");
    group.measurement_time(Duration::from_secs(10));

    // Test with different message sizes
    for message_size in [64, 512, 4096, 32768].iter() {
        group.throughput(Throughput::Bytes(*message_size as u64));

        group.bench_with_input(
            BenchmarkId::new("route_single_message", message_size),
            message_size,
            |b, &message_size| {
                b.to_async(&rt).iter(|| async {
                    let config = RouterConfig::default();
                    let router = OptimizedMessageRouter::new(config);

                    // Register test agents
                    let sender = AgentId::new();
                    let receiver = AgentId::new();

                    let (tx, _rx) = mpsc::unbounded_channel();
                    router
                        .register_agent(
                            receiver,
                            "localhost:8080".to_string(),
                            vec!["test_capability".to_string()],
                            tx,
                        )
                        .await;

                    let message = generate_test_message(sender, receiver, message_size);

                    let result = router.route_message(message).await;
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch message routing
fn bench_batch_message_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_message_routing");
    group.measurement_time(Duration::from_secs(10));

    // Test with different batch sizes
    for batch_size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("route_message_batch", batch_size),
            batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async {
                    let config = RouterConfig {
                        max_batch_size: batch_size * 2, // Ensure batch doesn't get split
                        ..Default::default()
                    };
                    let router = OptimizedMessageRouter::new(config);

                    // Register test agents
                    let sender = AgentId::new();
                    let receiver = AgentId::new();

                    let (tx, _rx) = mpsc::unbounded_channel();
                    router
                        .register_agent(
                            receiver,
                            "localhost:8080".to_string(),
                            vec!["test_capability".to_string()],
                            tx,
                        )
                        .await;

                    // Generate batch of messages
                    let messages: Vec<FipaMessage> = (0..batch_size)
                        .map(|_| generate_test_message(sender, receiver, 512))
                        .collect();

                    let result = router.route_messages_batch(messages).await;
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark agent registration and lookup
fn bench_agent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_operations");
    group.measurement_time(Duration::from_secs(5));

    // Test agent registration performance
    group.bench_function("register_agent", |b| {
        b.to_async(&rt).iter(|| async {
            let config = RouterConfig::default();
            let router = OptimizedMessageRouter::new(config);
            let agent_id = AgentId::new();

            let (tx, _rx) = mpsc::unbounded_channel();

            let result = router
                .register_agent(
                    agent_id,
                    "localhost:8080".to_string(),
                    vec!["capability1".to_string(), "capability2".to_string()],
                    tx,
                )
                .await;

            black_box(result);
        });
    });

    // Test capability-based lookup with different agent counts
    for agent_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("capability_lookup", agent_count),
            agent_count,
            |b, &agent_count| {
                b.to_async(&rt).iter_batched(
                    || {
                        let config = RouterConfig::default();
                        let router = OptimizedMessageRouter::new(config);

                        rt.block_on(async {
                            // Register agents with various capabilities
                            for i in 0..agent_count {
                                let agent_id = AgentId::new();
                                let (tx, _rx) = mpsc::unbounded_channel();

                                let capabilities = vec![
                                    format!("capability_{}", i % 5), // 5 different capabilities
                                    "common_capability".to_string(),
                                ];

                                router
                                    .register_agent(
                                        agent_id,
                                        format!("localhost:{}", 8080 + i),
                                        capabilities,
                                        tx,
                                    )
                                    .await;
                            }

                            router
                        })
                    },
                    |router| async move {
                        let agents = router.find_agents_by_capability("common_capability").await;
                        black_box(agents);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark message serialization formats
fn bench_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");
    group.measurement_time(Duration::from_secs(5));

    let message = generate_test_message(AgentId::new(), AgentId::new(), 1024);

    // JSON serialization (baseline)
    group.bench_function("json_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_vec(&message).unwrap();
            let _deserialized: FipaMessage = serde_json::from_slice(&serialized).unwrap();
            black_box(serialized);
        });
    });

    // MessagePack serialization (optimized)
    group.bench_function("messagepack_serialization", |b| {
        b.iter(|| {
            let serialized = rmp_serde::to_vec(&message).unwrap();
            let _deserialized: FipaMessage = rmp_serde::from_slice(&serialized).unwrap();
            black_box(serialized);
        });
    });

    // Bincode serialization (fastest but largest)
    group.bench_function("bincode_serialization", |b| {
        b.iter(|| {
            let serialized = bincode::serialize(&message).unwrap();
            let _deserialized: FipaMessage = bincode::deserialize(&serialized).unwrap();
            black_box(serialized);
        });
    });

    group.finish();
}

/// Benchmark routing table operations under concurrent load
fn bench_concurrent_routing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_routing");
    group.measurement_time(Duration::from_secs(10));

    // Test with different concurrency levels
    for concurrency in [1, 4, 16, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_message_routing", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let config = RouterConfig {
                        max_concurrent_messages: concurrency * 10,
                        ..Default::default()
                    };
                    let router = OptimizedMessageRouter::new(config);

                    // Register test agents
                    let agents: Vec<_> = (0..10).map(|_| AgentId::new()).collect();

                    for &agent_id in &agents {
                        let (tx, _rx) = mpsc::unbounded_channel();
                        router
                            .register_agent(
                                agent_id,
                                "localhost:8080".to_string(),
                                vec!["test_capability".to_string()],
                                tx,
                            )
                            .await;
                    }

                    // Spawn concurrent routing tasks
                    let mut handles = Vec::new();

                    for _ in 0..concurrency {
                        let router = &router;
                        let agents = &agents;

                        let handle = tokio::spawn(async move {
                            for _ in 0..100 {
                                let sender = agents[fastrand::usize(..agents.len())];
                                let receiver = agents[fastrand::usize(..agents.len())];

                                if sender != receiver {
                                    let message = generate_test_message(sender, receiver, 256);
                                    let _ = router.route_message(message).await;
                                }
                            }
                        });

                        handles.push(handle);
                    }

                    // Wait for all tasks to complete
                    for handle in handles {
                        let _ = handle.await;
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different routing configurations
fn bench_routing_configurations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("routing_configurations");
    group.measurement_time(Duration::from_secs(5));

    let agents: Vec<AgentId> = (0..100).map(|_| AgentId::new()).collect();
    let messages: Vec<FipaMessage> = (0..1000)
        .map(|_| {
            let sender = agents[fastrand::usize(..agents.len())];
            let receiver = agents[fastrand::usize(..agents.len())];
            generate_test_message(sender, receiver, 512)
        })
        .collect();

    // Small batches, frequent flushes
    group.bench_function("small_batches_frequent_flush", |b| {
        b.to_async(&rt).iter(|| async {
            let config = RouterConfig {
                max_batch_size: 10,
                batch_timeout: Duration::from_millis(1),
                ..Default::default()
            };

            let router = OptimizedMessageRouter::new(config);

            // Register agents
            for &agent_id in &agents {
                let (tx, _rx) = mpsc::unbounded_channel();
                router
                    .register_agent(
                        agent_id,
                        "localhost:8080".to_string(),
                        vec!["test_capability".to_string()],
                        tx,
                    )
                    .await;
            }

            // Route messages
            for message in &messages[..100] {
                // Subset for benchmarking
                let _ = router.route_message(message.clone()).await;
            }
        });
    });

    // Large batches, infrequent flushes
    group.bench_function("large_batches_infrequent_flush", |b| {
        b.to_async(&rt).iter(|| async {
            let config = RouterConfig {
                max_batch_size: 1000,
                batch_timeout: Duration::from_millis(100),
                ..Default::default()
            };

            let router = OptimizedMessageRouter::new(config);

            // Register agents
            for &agent_id in &agents {
                let (tx, _rx) = mpsc::unbounded_channel();
                router
                    .register_agent(
                        agent_id,
                        "localhost:8080".to_string(),
                        vec!["test_capability".to_string()],
                        tx,
                    )
                    .await;
            }

            // Route messages in batch
            let _ = router.route_messages_batch(messages[..100].to_vec()).await;
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_message_routing,
    bench_batch_message_routing,
    bench_agent_operations,
    bench_message_serialization,
    bench_concurrent_routing,
    bench_routing_configurations
);

criterion_main!(benches);
