//! Performance Benchmarks for Caxton Multi-Agent Platform
//! 
//! Comprehensive performance testing covering:
//! - Message throughput and latency
//! - Agent spawning and termination performance
//! - Memory usage and garbage collection
//! - Resource contention under load
//! - Scaling characteristics

use caxton::*;
use criterion::{black_box, Criterion, Throughput};
use serial_test::serial;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Semaphore};
use tracing_test::traced_test;

/// Benchmark message throughput between agents
#[tokio::test]
#[traced_test]
#[serial]
async fn benchmark_message_throughput() {
    let runtime = setup_benchmark_runtime().await;
    let message_counts = vec![100, 1000, 5000, 10000];
    
    for message_count in message_counts {
        let sender_id = runtime.spawn_agent(AgentConfig {
            name: format!("throughput-sender-{}", message_count),
            agent_type: AgentType::Worker,
            capabilities: vec!["bulk-messaging".to_string()],
            max_memory: Some(64 * 1024 * 1024),
            timeout: Some(Duration::from_secs(30)),
        }).await.expect("Failed to spawn sender");
        
        let receiver_id = runtime.spawn_agent(AgentConfig {
            name: format!("throughput-receiver-{}", message_count),
            agent_type: AgentType::Worker,
            capabilities: vec!["bulk-processing".to_string()],
            max_memory: Some(64 * 1024 * 1024),
            timeout: Some(Duration::from_secs(30)),
        }).await.expect("Failed to spawn receiver");
        
        wait_for_agent_ready(&runtime, &sender_id).await;
        wait_for_agent_ready(&runtime, &receiver_id).await;
        
        // Measure throughput
        let start_time = Instant::now();
        
        for i in 0..message_count {
            let message = create_benchmark_message(&sender_id, &receiver_id, i);
            runtime.send_message(message).await
                .expect("Failed to send benchmark message");
        }
        
        // Wait for all messages to be processed
        let completion_timeout = Duration::from_secs(60);
        let received_count = timeout(completion_timeout, async {
            let mut count = 0;
            while count < message_count {
                if let Some(_) = runtime.receive_message().await {
                    count += 1;
                }
            }
            count
        }).await.expect("Messages not processed in time");
        
        let elapsed = start_time.elapsed();
        let throughput = message_count as f64 / elapsed.as_secs_f64();
        
        println!("Messages: {}, Time: {:?}, Throughput: {:.2} msgs/sec", 
                message_count, elapsed, throughput);
        
        assert_eq!(received_count, message_count);
        assert!(throughput > 100.0, "Throughput too low: {:.2} msgs/sec", throughput);
        
        // Clean up agents
        runtime.terminate_agent(&sender_id, Duration::from_secs(5)).await
            .expect("Failed to terminate sender");
        runtime.terminate_agent(&receiver_id, Duration::from_secs(5)).await
            .expect("Failed to terminate receiver");
    }
}

/// Benchmark agent spawning and termination performance
#[tokio::test]
#[traced_test]
#[serial]
async fn benchmark_agent_lifecycle_performance() {
    let runtime = setup_benchmark_runtime().await;
    let agent_counts = vec![1, 5, 10, 25, 50];
    
    for agent_count in agent_counts {
        // Benchmark spawning
        let spawn_start = Instant::now();
        let mut agent_ids = Vec::with_capacity(agent_count);
        
        for i in 0..agent_count {
            let agent_id = runtime.spawn_agent(AgentConfig {
                name: format!("perf-agent-{}", i),
                agent_type: AgentType::Worker,
                capabilities: vec!["performance-testing".to_string()],
                max_memory: Some(32 * 1024 * 1024),
                timeout: Some(Duration::from_secs(10)),
            }).await.expect("Failed to spawn performance agent");
            
            agent_ids.push(agent_id);
        }
        
        let spawn_elapsed = spawn_start.elapsed();
        
        // Wait for all agents to be ready
        let ready_start = Instant::now();
        for agent_id in &agent_ids {
            wait_for_agent_ready(&runtime, agent_id).await;
        }
        let ready_elapsed = ready_start.elapsed();
        
        // Benchmark termination
        let terminate_start = Instant::now();
        for agent_id in &agent_ids {
            runtime.terminate_agent(agent_id, Duration::from_secs(5)).await
                .expect("Failed to terminate agent");
        }
        let terminate_elapsed = terminate_start.elapsed();
        
        let spawn_rate = agent_count as f64 / spawn_elapsed.as_secs_f64();
        let ready_rate = agent_count as f64 / ready_elapsed.as_secs_f64();
        let terminate_rate = agent_count as f64 / terminate_elapsed.as_secs_f64();
        
        println!("Agent Lifecycle Performance (n={})", agent_count);
        println!("  Spawn: {:?} ({:.2} agents/sec)", spawn_elapsed, spawn_rate);
        println!("  Ready: {:?} ({:.2} agents/sec)", ready_elapsed, ready_rate);
        println!("  Terminate: {:?} ({:.2} agents/sec)", terminate_elapsed, terminate_rate);
        
        // Performance requirements
        assert!(spawn_rate > 1.0, "Agent spawn rate too low: {:.2}", spawn_rate);
        assert!(ready_rate > 2.0, "Agent ready rate too low: {:.2}", ready_rate);
        assert!(terminate_rate > 5.0, "Agent terminate rate too low: {:.2}", terminate_rate);
        
        // Verify clean shutdown
        let active_agents = runtime.get_active_agent_count().await;
        assert_eq!(active_agents, 0, "Agents not properly terminated");
    }
}

/// Benchmark memory usage and garbage collection
#[tokio::test]
#[traced_test]
#[serial]
async fn benchmark_memory_performance() {
    let runtime = setup_benchmark_runtime().await;
    
    // Create memory-intensive agent
    let agent_id = runtime.spawn_agent(AgentConfig {
        name: "memory-benchmark-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["memory-intensive".to_string()],
        max_memory: Some(128 * 1024 * 1024), // 128MB
        timeout: Some(Duration::from_secs(30)),
    }).await.expect("Failed to spawn memory benchmark agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    let allocation_sizes = vec![1024, 10240, 102400, 1048576]; // 1KB to 1MB
    let iterations = 1000;
    
    for alloc_size in allocation_sizes {
        let start_memory = runtime.get_agent_memory_usage(&agent_id).await.unwrap();
        let start_time = Instant::now();
        
        // Perform many allocations
        for i in 0..iterations {
            let alloc_message = create_memory_benchmark_message(&agent_id, alloc_size, i);
            runtime.send_message(alloc_message).await
                .expect("Failed to send allocation message");
            
            // Every 100 iterations, check memory usage
            if i % 100 == 0 {
                let current_memory = runtime.get_agent_memory_usage(&agent_id).await.unwrap();
                
                // Ensure memory usage doesn't grow unboundedly
                let memory_growth = current_memory - start_memory;
                let expected_max_growth = (alloc_size * 200) as u64; // Allow some overhead
                
                if memory_growth > expected_max_growth {
                    // Trigger garbage collection
                    let gc_message = create_gc_message(&agent_id);
                    runtime.send_message(gc_message).await
                        .expect("Failed to send GC message");
                    
                    // Wait for GC to complete
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
        
        let elapsed = start_time.elapsed();
        let end_memory = runtime.get_agent_memory_usage(&agent_id).await.unwrap();
        
        let allocation_rate = iterations as f64 / elapsed.as_secs_f64();
        let memory_efficiency = (end_memory - start_memory) as f64 / (alloc_size * iterations) as f64;
        
        println!("Memory Benchmark - Size: {} bytes", alloc_size);
        println!("  Iterations: {}, Time: {:?}", iterations, elapsed);
        println!("  Allocation rate: {:.2} allocs/sec", allocation_rate);
        println!("  Memory efficiency: {:.2}", memory_efficiency);
        println!("  Memory growth: {} bytes", end_memory - start_memory);
        
        // Performance requirements
        assert!(allocation_rate > 50.0, "Allocation rate too low: {:.2}", allocation_rate);
        assert!(memory_efficiency < 2.0, "Memory efficiency too poor: {:.2}", memory_efficiency);
        
        // Ensure memory is properly managed
        let max_expected_memory = start_memory + (alloc_size * 50) as u64; // Allow some retained memory
        assert!(end_memory < max_expected_memory, 
               "Memory leak detected: {} > {}", end_memory, max_expected_memory);
    }
}

/// Benchmark resource contention under high load
#[tokio::test]
#[traced_test]
#[serial]
async fn benchmark_resource_contention() {
    let runtime = setup_benchmark_runtime().await;
    let num_agents = 20;
    let messages_per_agent = 100;
    
    // Spawn many agents concurrently
    let mut agent_ids = Vec::with_capacity(num_agents);
    let barrier = Arc::new(Barrier::new(num_agents + 1));
    let semaphore = Arc::new(Semaphore::new(num_agents));
    
    for i in 0..num_agents {
        let agent_id = runtime.spawn_agent(AgentConfig {
            name: format!("contention-agent-{}", i),
            agent_type: AgentType::Worker,
            capabilities: vec!["high-contention".to_string()],
            max_memory: Some(32 * 1024 * 1024),
            timeout: Some(Duration::from_secs(15)),
        }).await.expect("Failed to spawn contention agent");
        
        agent_ids.push(agent_id);
    }
    
    // Wait for all agents to be ready
    for agent_id in &agent_ids {
        wait_for_agent_ready(&runtime, agent_id).await;
    }
    
    // Create high contention scenario
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for (i, agent_id) in agent_ids.iter().enumerate() {
        let agent_id = agent_id.clone();
        let runtime = runtime.clone();
        let barrier = barrier.clone();
        let semaphore = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            // Wait for all agents to start simultaneously
            barrier.wait().await;
            
            let mut latencies = Vec::with_capacity(messages_per_agent);
            
            for j in 0..messages_per_agent {
                let msg_start = Instant::now();
                
                let message = create_contention_message(&agent_id, i, j);
                runtime.send_message(message).await
                    .expect("Failed to send contention message");
                
                let latency = msg_start.elapsed();
                latencies.push(latency);
            }
            
            (i, latencies)
        });
        
        handles.push(handle);
    }
    
    // Start the benchmark
    barrier.wait().await;
    
    // Collect results
    let results = futures::future::join_all(handles).await;
    let total_elapsed = start_time.elapsed();
    
    // Analyze contention performance
    let mut all_latencies = Vec::new();
    let mut per_agent_stats = Vec::new();
    
    for result in results {
        let (agent_idx, latencies) = result.expect("Agent task failed");
        
        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let max_latency = *latencies.iter().max().unwrap();
        let min_latency = *latencies.iter().min().unwrap();
        
        per_agent_stats.push((agent_idx, avg_latency, min_latency, max_latency));
        all_latencies.extend(latencies);
    }
    
    // Calculate overall statistics
    all_latencies.sort();
    let total_messages = num_agents * messages_per_agent;
    let overall_throughput = total_messages as f64 / total_elapsed.as_secs_f64();
    
    let avg_latency = all_latencies.iter().sum::<Duration>() / all_latencies.len() as u32;
    let p50_latency = all_latencies[all_latencies.len() / 2];
    let p95_latency = all_latencies[all_latencies.len() * 95 / 100];
    let p99_latency = all_latencies[all_latencies.len() * 99 / 100];
    
    println!("Resource Contention Benchmark");
    println!("  Agents: {}, Messages per agent: {}", num_agents, messages_per_agent);
    println!("  Total time: {:?}", total_elapsed);
    println!("  Overall throughput: {:.2} msgs/sec", overall_throughput);
    println!("  Average latency: {:?}", avg_latency);
    println!("  P50 latency: {:?}", p50_latency);
    println!("  P95 latency: {:?}", p95_latency);
    println!("  P99 latency: {:?}", p99_latency);
    
    // Performance requirements under contention
    assert!(overall_throughput > 200.0, "Throughput under contention too low: {:.2}", overall_throughput);
    assert!(avg_latency < Duration::from_millis(100), "Average latency too high: {:?}", avg_latency);
    assert!(p95_latency < Duration::from_millis(500), "P95 latency too high: {:?}", p95_latency);
    
    // Verify fairness - no agent should be starved
    for (agent_idx, avg_lat, min_lat, max_lat) in &per_agent_stats {
        println!("  Agent {}: avg={:?}, min={:?}, max={:?}", agent_idx, avg_lat, min_lat, max_lat);
        
        // No agent should have extremely poor performance
        assert!(*avg_lat < Duration::from_millis(200), 
               "Agent {} starved with avg latency {:?}", agent_idx, avg_lat);
    }
    
    // Clean up
    for agent_id in &agent_ids {
        runtime.terminate_agent(agent_id, Duration::from_secs(5)).await
            .expect("Failed to terminate contention agent");
    }
}

/// Benchmark scaling characteristics
#[tokio::test]
#[traced_test]
#[serial]
async fn benchmark_scaling_characteristics() {
    let runtime = setup_benchmark_runtime().await;
    let agent_scales = vec![1, 2, 5, 10, 20, 50];
    let messages_per_scale = 1000;
    
    let mut scaling_results = Vec::new();
    
    for num_agents in agent_scales {
        println!("Testing scale: {} agents", num_agents);
        
        // Spawn agents for this scale
        let mut agent_ids = Vec::with_capacity(num_agents);
        let spawn_start = Instant::now();
        
        for i in 0..num_agents {
            let agent_id = runtime.spawn_agent(AgentConfig {
                name: format!("scale-agent-{}-{}", num_agents, i),
                agent_type: AgentType::Worker,
                capabilities: vec!["scaling-test".to_string()],
                max_memory: Some(32 * 1024 * 1024),
                timeout: Some(Duration::from_secs(20)),
            }).await.expect("Failed to spawn scaling agent");
            
            agent_ids.push(agent_id);
        }
        
        // Wait for all to be ready
        for agent_id in &agent_ids {
            wait_for_agent_ready(&runtime, agent_id).await;
        }
        
        let ready_time = spawn_start.elapsed();
        
        // Measure message processing performance
        let msg_start = Instant::now();
        let mut message_handles = Vec::new();
        
        for (i, agent_id) in agent_ids.iter().enumerate() {
            let agent_id = agent_id.clone();
            let runtime = runtime.clone();
            
            let handle = tokio::spawn(async move {
                let mut sent = 0;
                for j in 0..(messages_per_scale / num_agents) {
                    let message = create_scaling_message(&agent_id, i, j);
                    if runtime.send_message(message).await.is_ok() {
                        sent += 1;
                    }
                }
                sent
            });
            
            message_handles.push(handle);
        }
        
        let message_results = futures::future::join_all(message_handles).await;
        let msg_elapsed = msg_start.elapsed();
        
        let total_sent: usize = message_results.iter()
            .map(|r| r.as_ref().unwrap_or(&0))
            .sum();
        
        let throughput = total_sent as f64 / msg_elapsed.as_secs_f64();
        let throughput_per_agent = throughput / num_agents as f64;
        
        // Measure resource usage
        let mut total_memory = 0u64;
        let mut total_cpu_time = Duration::from_nanos(0);
        
        for agent_id in &agent_ids {
            let memory = runtime.get_agent_memory_usage(agent_id).await.unwrap_or(0);
            let cpu_time = runtime.get_agent_cpu_time(agent_id).await.unwrap_or(Duration::from_nanos(0));
            
            total_memory += memory;
            total_cpu_time += cpu_time;
        }
        
        let avg_memory_per_agent = total_memory / num_agents as u64;
        let avg_cpu_per_agent = total_cpu_time / num_agents as u32;
        
        // Record scaling result
        let scaling_result = ScalingResult {
            num_agents,
            ready_time,
            throughput,
            throughput_per_agent,
            avg_memory_per_agent,
            avg_cpu_per_agent,
            messages_sent: total_sent,
        };
        
        scaling_results.push(scaling_result.clone());
        
        println!("Scale {} results:", num_agents);
        println!("  Ready time: {:?}", ready_time);
        println!("  Throughput: {:.2} msgs/sec", throughput);
        println!("  Throughput per agent: {:.2} msgs/sec", throughput_per_agent);
        println!("  Avg memory per agent: {} bytes", avg_memory_per_agent);
        println!("  Avg CPU per agent: {:?}", avg_cpu_per_agent);
        
        // Clean up agents
        for agent_id in &agent_ids {
            runtime.terminate_agent(agent_id, Duration::from_secs(5)).await
                .expect("Failed to terminate scaling agent");
        }
        
        // Brief pause between scales
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Analyze scaling characteristics
    analyze_scaling_results(&scaling_results);
}

// Helper functions and types

#[derive(Clone, Debug)]
struct ScalingResult {
    num_agents: usize,
    ready_time: Duration,
    throughput: f64,
    throughput_per_agent: f64,
    avg_memory_per_agent: u64,
    avg_cpu_per_agent: Duration,
    messages_sent: usize,
}

fn analyze_scaling_results(results: &[ScalingResult]) {
    println!("\nScaling Analysis:");
    
    // Check if throughput scales linearly
    let base_throughput = results[0].throughput;
    let base_agents = results[0].num_agents;
    
    for result in results.iter().skip(1) {
        let expected_throughput = base_throughput * (result.num_agents as f64 / base_agents as f64);
        let efficiency = result.throughput / expected_throughput;
        
        println!("  {} agents: {:.2}% efficiency", result.num_agents, efficiency * 100.0);
        
        // Scaling should be at least 70% efficient
        assert!(efficiency > 0.7, 
               "Poor scaling efficiency at {} agents: {:.2}%", 
               result.num_agents, efficiency * 100.0);
    }
    
    // Memory usage should be roughly linear
    let base_total_memory = results[0].avg_memory_per_agent * results[0].num_agents as u64;
    
    for result in results.iter().skip(1) {
        let current_total_memory = result.avg_memory_per_agent * result.num_agents as u64;
        let memory_ratio = current_total_memory as f64 / base_total_memory as f64;
        let agent_ratio = result.num_agents as f64 / base_agents as f64;
        let memory_efficiency = agent_ratio / memory_ratio;
        
        println!("  {} agents memory efficiency: {:.2}%", result.num_agents, memory_efficiency * 100.0);
        
        // Memory scaling should be reasonable (within 50% of linear)
        assert!(memory_efficiency > 0.5 && memory_efficiency < 2.0,
               "Poor memory scaling at {} agents: {:.2}%", 
               result.num_agents, memory_efficiency * 100.0);
    }
}

async fn setup_benchmark_runtime() -> CaxtonRuntime {
    CaxtonRuntime::new(CaxtonConfig {
        max_agents: 100,
        default_timeout: Duration::from_secs(60),
        observability_enabled: false, // Disable for cleaner benchmarks
        performance_mode: true,
        resource_limits: ResourceLimits {
            max_memory_per_agent: 128 * 1024 * 1024,
            max_cpu_time_per_agent: Duration::from_secs(30),
        },
    }).await.expect("Failed to create benchmark runtime")
}

fn create_benchmark_message(sender: &AgentId, receiver: &AgentId, sequence: usize) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Inform,
        sender: sender.clone(),
        receiver: receiver.clone(),
        content: serde_json::json!({
            "benchmark": true,
            "sequence": sequence,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "payload": format!("benchmark-data-{}", sequence)
        }),
        protocol: Some("benchmark".to_string()),
        ..Default::default()
    }
}

fn create_memory_benchmark_message(agent_id: &AgentId, size: usize, iteration: usize) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({
            "action": "allocate_memory",
            "size": size,
            "iteration": iteration
        }),
        protocol: Some("memory-benchmark".to_string()),
        ..Default::default()
    }
}

fn create_gc_message(agent_id: &AgentId) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({"action": "garbage_collect"}),
        protocol: Some("memory-management".to_string()),
        ..Default::default()
    }
}

fn create_contention_message(agent_id: &AgentId, agent_idx: usize, msg_idx: usize) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({
            "action": "cpu_intensive_task",
            "agent_index": agent_idx,
            "message_index": msg_idx,
            "complexity": 1000 // Operations to perform
        }),
        protocol: Some("contention-test".to_string()),
        ..Default::default()
    }
}

fn create_scaling_message(agent_id: &AgentId, agent_idx: usize, msg_idx: usize) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({
            "action": "simple_task",
            "agent_index": agent_idx,
            "message_index": msg_idx
        }),
        protocol: Some("scaling-test".to_string()),
        ..Default::default()
    }
}

async fn wait_for_agent_ready(runtime: &CaxtonRuntime, agent_id: &AgentId) {
    timeout(Duration::from_secs(10), async {
        loop {
            if runtime.get_agent_state(agent_id).await.unwrap() == AgentState::Ready {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }).await.expect("Agent never became ready");
}