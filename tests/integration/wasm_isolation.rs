//! WebAssembly Isolation Boundary Tests
//! 
//! Tests that verify WebAssembly-based agent isolation:
//! - Memory isolation between agents
//! - CPU time limits and enforcement
//! - System call restrictions
//! - Resource exhaustion protection
//! - Security boundary validation

use caxton::*;
use serial_test::serial;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::Barrier;
use tracing_test::traced_test;
use wasmtime::*;

/// Test memory isolation between WASM agents
#[tokio::test]
#[traced_test]
#[serial]
async fn test_wasm_memory_isolation() {
    let runtime = setup_wasm_test_runtime().await;
    
    // Create two agents with separate WASM instances
    let agent1_id = runtime.spawn_wasm_agent(WasmAgentConfig {
        name: "memory-agent-1".to_string(),
        wasm_module: create_memory_test_module(),
        max_memory_pages: 10, // 640KB
        max_execution_time: Duration::from_secs(5),
        capabilities: vec!["memory-operations".to_string()],
    }).await.expect("Failed to spawn agent 1");
    
    let agent2_id = runtime.spawn_wasm_agent(WasmAgentConfig {
        name: "memory-agent-2".to_string(),
        wasm_module: create_memory_test_module(),
        max_memory_pages: 10, // 640KB
        max_execution_time: Duration::from_secs(5),
        capabilities: vec!["memory-operations".to_string()],
    }).await.expect("Failed to spawn agent 2");
    
    wait_for_agent_ready(&runtime, &agent1_id).await;
    wait_for_agent_ready(&runtime, &agent2_id).await;
    
    // Agent 1 allocates and writes to memory
    let allocate_msg1 = create_wasm_call_message(&agent1_id, "allocate_and_write", vec![
        WasmValue::I32(1024), // size
        WasmValue::I32(0xDEADBEEF), // pattern
    ]);
    
    let response1 = runtime.send_message(allocate_msg1).await
        .expect("Failed to send allocation message to agent 1");
    
    let memory_addr1 = extract_i32_result(&response1).expect("Failed to get memory address");
    
    // Agent 2 allocates and writes different pattern
    let allocate_msg2 = create_wasm_call_message(&agent2_id, "allocate_and_write", vec![
        WasmValue::I32(1024), // size  
        WasmValue::I32(0xCAFEBABE), // different pattern
    ]);
    
    let response2 = runtime.send_message(allocate_msg2).await
        .expect("Failed to send allocation message to agent 2");
    
    let memory_addr2 = extract_i32_result(&response2).expect("Failed to get memory address");
    
    // Verify agents can read their own memory
    let read_msg1 = create_wasm_call_message(&agent1_id, "read_memory", vec![
        WasmValue::I32(memory_addr1),
        WasmValue::I32(4), // read 4 bytes
    ]);
    
    let read_result1 = runtime.send_message(read_msg1).await
        .expect("Failed to read memory from agent 1");
    
    assert_eq!(extract_i32_result(&read_result1).unwrap(), 0xDEADBEEF);
    
    let read_msg2 = create_wasm_call_message(&agent2_id, "read_memory", vec![
        WasmValue::I32(memory_addr2),
        WasmValue::I32(4),
    ]);
    
    let read_result2 = runtime.send_message(read_msg2).await
        .expect("Failed to read memory from agent 2");
    
    assert_eq!(extract_i32_result(&read_result2).unwrap(), 0xCAFEBABE);
    
    // Verify agents cannot access each other's memory addresses
    let cross_read_msg = create_wasm_call_message(&agent1_id, "read_memory", vec![
        WasmValue::I32(memory_addr2), // Try to read agent2's memory
        WasmValue::I32(4),
    ]);
    
    let cross_read_result = runtime.send_message(cross_read_msg).await;
    
    // Should either fail or return garbage (not agent2's pattern)
    match cross_read_result {
        Ok(response) => {
            let value = extract_i32_result(&response).unwrap_or(0);
            assert_ne!(value, 0xCAFEBABE, "Cross-agent memory access succeeded!");
        }
        Err(_) => {
            // Expected - memory access should fail
        }
    }
    
    // Verify memory usage metrics show isolation
    let metrics1 = runtime.get_wasm_metrics(&agent1_id).await.unwrap();
    let metrics2 = runtime.get_wasm_metrics(&agent2_id).await.unwrap();
    
    assert_ne!(metrics1.instance_id, metrics2.instance_id);
    assert!(metrics1.memory_used > 0);
    assert!(metrics2.memory_used > 0);
    assert_ne!(metrics1.memory_base_addr, metrics2.memory_base_addr);
}

/// Test CPU time limits and enforcement
#[tokio::test]
#[traced_test]
#[serial]
async fn test_wasm_cpu_time_limits() {
    let runtime = setup_wasm_test_runtime().await;
    
    // Create agent with strict CPU time limit
    let agent_id = runtime.spawn_wasm_agent(WasmAgentConfig {
        name: "cpu-limited-agent".to_string(),
        wasm_module: create_cpu_intensive_module(),
        max_memory_pages: 5,
        max_execution_time: Duration::from_millis(100), // Very short limit
        capabilities: vec!["cpu-intensive".to_string()],
    }).await.expect("Failed to spawn CPU limited agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Send CPU-intensive task that should exceed time limit
    let intensive_task = create_wasm_call_message(&agent_id, "fibonacci", vec![
        WasmValue::I32(40), // Computationally expensive
    ]);
    
    let start_time = Instant::now();
    let result = runtime.send_message(intensive_task).await;
    let elapsed = start_time.elapsed();
    
    // Should either timeout or be terminated quickly
    match result {
        Ok(response) => {
            // If it completed, should be very fast (not the full computation)
            assert!(elapsed < Duration::from_millis(200));
            // Response should indicate early termination
            assert!(response.content.get("terminated").is_some());
        }
        Err(e) => {
            // Expected - should timeout
            assert!(e.to_string().contains("timeout") || e.to_string().contains("terminated"));
            assert!(elapsed < Duration::from_millis(200));
        }
    }
    
    // Verify CPU metrics show enforcement
    let metrics = runtime.get_wasm_metrics(&agent_id).await.unwrap();
    assert!(metrics.cpu_time_used <= Duration::from_millis(150));
    assert!(metrics.time_limit_exceeded);
}

/// Test system call restrictions
#[tokio::test]
#[traced_test]
#[serial]
async fn test_wasm_syscall_restrictions() {
    let runtime = setup_wasm_test_runtime().await;
    
    let agent_id = runtime.spawn_wasm_agent(WasmAgentConfig {
        name: "restricted-agent".to_string(),
        wasm_module: create_syscall_test_module(),
        max_memory_pages: 5,
        max_execution_time: Duration::from_secs(5),
        capabilities: vec!["file-operations".to_string()],
    }).await.expect("Failed to spawn restricted agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Test file system access (should be restricted)
    let file_access_msg = create_wasm_call_message(&agent_id, "try_file_access", vec![
        WasmValue::I32(0), // flag for read attempt
    ]);
    
    let file_result = runtime.send_message(file_access_msg).await
        .expect("Failed to send file access message");
    
    // Should indicate access denied
    assert_eq!(extract_i32_result(&file_result).unwrap(), -1); // EACCES
    
    // Test network access (should be restricted)
    let network_msg = create_wasm_call_message(&agent_id, "try_network_access", vec![]);
    
    let network_result = runtime.send_message(network_msg).await
        .expect("Failed to send network access message");
    
    // Should indicate network access denied
    assert_eq!(extract_i32_result(&network_result).unwrap(), -1);
    
    // Test allowed operations (memory allocation)
    let alloc_msg = create_wasm_call_message(&agent_id, "safe_allocation", vec![
        WasmValue::I32(1024),
    ]);
    
    let alloc_result = runtime.send_message(alloc_msg).await
        .expect("Failed to send allocation message");
    
    // Should succeed
    assert!(extract_i32_result(&alloc_result).unwrap() > 0);
    
    // Verify security metrics
    let security_metrics = runtime.get_security_metrics(&agent_id).await.unwrap();
    assert_eq!(security_metrics.syscall_violations, 2); // file + network
    assert_eq!(security_metrics.allowed_operations, 1); // allocation
}

/// Test resource exhaustion protection
#[tokio::test]
#[traced_test]
#[serial]
async fn test_wasm_resource_exhaustion_protection() {
    let runtime = setup_wasm_test_runtime().await;
    
    // Create agent with limited resources
    let agent_id = runtime.spawn_wasm_agent(WasmAgentConfig {
        name: "resource-limited-agent".to_string(),
        wasm_module: create_resource_exhaustion_module(),
        max_memory_pages: 3, // Very limited memory (192KB)
        max_execution_time: Duration::from_secs(2),
        capabilities: vec!["memory-stress".to_string()],
    }).await.expect("Failed to spawn resource limited agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Try to allocate more memory than allowed
    let over_alloc_msg = create_wasm_call_message(&agent_id, "allocate_large", vec![
        WasmValue::I32(1024 * 1024), // 1MB - exceeds limit
    ]);
    
    let over_alloc_result = runtime.send_message(over_alloc_msg).await;
    
    // Should fail due to memory limit
    match over_alloc_result {
        Ok(response) => {
            // Should return error code
            assert_eq!(extract_i32_result(&response).unwrap(), -1);
        }
        Err(e) => {
            // Expected - out of memory error
            assert!(e.to_string().contains("memory") || e.to_string().contains("limit"));
        }
    }
    
    // Verify agent is still responsive after failed allocation
    let health_msg = create_wasm_call_message(&agent_id, "health_check", vec![]);
    let health_result = runtime.send_message(health_msg).await
        .expect("Agent became unresponsive after resource exhaustion");
    
    assert_eq!(extract_i32_result(&health_result).unwrap(), 1); // healthy
    
    // Try memory bomb (many small allocations)
    let bomb_msg = create_wasm_call_message(&agent_id, "memory_bomb", vec![
        WasmValue::I32(10000), // Try to allocate 10000 small blocks
    ]);
    
    let bomb_result = runtime.send_message(bomb_msg).await;
    
    // Should be prevented by resource limits
    match bomb_result {
        Ok(response) => {
            let allocated = extract_i32_result(&response).unwrap();
            assert!(allocated < 1000, "Memory bomb succeeded too much: {}", allocated);
        }
        Err(_) => {
            // Expected - resource exhaustion protection kicked in
        }
    }
    
    // Verify resource protection metrics
    let resource_metrics = runtime.get_resource_metrics(&agent_id).await.unwrap();
    assert!(resource_metrics.memory_limit_hits > 0);
    assert!(resource_metrics.allocation_failures > 0);
}

/// Test concurrent WASM agent isolation
#[tokio::test]
#[traced_test]
#[serial]
async fn test_concurrent_wasm_isolation() {
    let runtime = setup_wasm_test_runtime().await;
    let num_agents = 5;
    let barrier = Arc::new(Barrier::new(num_agents + 1));
    
    // Spawn multiple agents concurrently
    let mut agent_ids = Vec::new();
    let mut handles = Vec::new();
    
    for i in 0..num_agents {
        let agent_id = runtime.spawn_wasm_agent(WasmAgentConfig {
            name: format!("concurrent-agent-{}", i),
            wasm_module: create_concurrent_test_module(),
            max_memory_pages: 8,
            max_execution_time: Duration::from_secs(10),
            capabilities: vec!["concurrent-ops".to_string()],
        }).await.expect("Failed to spawn concurrent agent");
        
        agent_ids.push(agent_id.clone());
        
        let runtime_clone = runtime.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            wait_for_agent_ready(&runtime_clone, &agent_id).await;
            
            // Wait for all agents to be ready
            barrier_clone.wait().await;
            
            // Each agent performs intensive operations
            let work_msg = create_wasm_call_message(&agent_id, "intensive_work", vec![
                WasmValue::I32(i as i32), // unique work pattern
            ]);
            
            runtime_clone.send_message(work_msg).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all agents to be ready
    barrier.wait().await;
    
    // Collect results
    let results = futures::future::join_all(handles).await;
    
    // Verify all agents completed successfully and independently
    for (i, result) in results.into_iter().enumerate() {
        let message_result = result.expect("Task panicked")
            .expect("Message failed");
        
        let work_result = extract_i32_result(&message_result).unwrap();
        assert_eq!(work_result, i as i32 * 42); // Expected pattern
    }
    
    // Verify isolation metrics
    let mut instance_ids = std::collections::HashSet::new();
    let mut memory_addrs = std::collections::HashSet::new();
    
    for agent_id in &agent_ids {
        let metrics = runtime.get_wasm_metrics(agent_id).await.unwrap();
        
        // Each agent should have unique WASM instance
        assert!(instance_ids.insert(metrics.instance_id));
        assert!(memory_addrs.insert(metrics.memory_base_addr));
        
        // All should have completed work
        assert!(metrics.function_calls > 0);
        assert_eq!(metrics.runtime_errors, 0);
    }
    
    assert_eq!(instance_ids.len(), num_agents);
    assert_eq!(memory_addrs.len(), num_agents);
}

/// Test WASM module validation and sandboxing
#[tokio::test]
#[traced_test]
#[serial]
async fn test_wasm_module_validation() {
    let runtime = setup_wasm_test_runtime().await;
    
    // Test invalid WASM module
    let invalid_module = vec![0x00, 0x61, 0x73]; // Invalid WASM magic bytes
    
    let result = runtime.spawn_wasm_agent(WasmAgentConfig {
        name: "invalid-wasm-agent".to_string(),
        wasm_module: invalid_module,
        max_memory_pages: 5,
        max_execution_time: Duration::from_secs(5),
        capabilities: vec![],
    }).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid") || 
            result.unwrap_err().to_string().contains("module"));
    
    // Test malicious WASM module (if we can detect it)
    let malicious_module = create_malicious_wasm_module();
    
    let result = runtime.spawn_wasm_agent(WasmAgentConfig {
        name: "malicious-wasm-agent".to_string(),
        wasm_module: malicious_module,
        max_memory_pages: 5,
        max_execution_time: Duration::from_secs(5),
        capabilities: vec![],
    }).await;
    
    // Should either reject the module or sandbox it safely
    match result {
        Ok(agent_id) => {
            // If accepted, verify it's properly sandboxed
            wait_for_agent_ready(&runtime, &agent_id).await;
            
            let malicious_msg = create_wasm_call_message(&agent_id, "malicious_function", vec![]);
            let malicious_result = runtime.send_message(malicious_msg).await;
            
            // Should not cause system damage
            match malicious_result {
                Ok(response) => {
                    // Should return safe result or error
                    assert!(response.content.get("error").is_some());
                }
                Err(_) => {
                    // Expected - malicious operation blocked
                }
            }
        }
        Err(_) => {
            // Expected - malicious module rejected
        }
    }
}

// Helper functions for creating test WASM modules

fn create_memory_test_module() -> Vec<u8> {
    // Simple WASM module with memory allocation and read/write functions
    // This would typically be generated from WAT or Rust->WASM compilation
    include_bytes!("../fixtures/memory_test.wasm").to_vec()
}

fn create_cpu_intensive_module() -> Vec<u8> {
    include_bytes!("../fixtures/cpu_intensive.wasm").to_vec()
}

fn create_syscall_test_module() -> Vec<u8> {
    include_bytes!("../fixtures/syscall_test.wasm").to_vec()
}

fn create_resource_exhaustion_module() -> Vec<u8> {
    include_bytes!("../fixtures/resource_exhaustion.wasm").to_vec()
}

fn create_concurrent_test_module() -> Vec<u8> {
    include_bytes!("../fixtures/concurrent_test.wasm").to_vec()
}

fn create_malicious_wasm_module() -> Vec<u8> {
    include_bytes!("../fixtures/malicious_test.wasm").to_vec()
}

fn create_wasm_call_message(agent_id: &AgentId, function: &str, args: Vec<WasmValue>) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({
            "wasm_call": {
                "function": function,
                "args": args.into_iter().map(|v| v.to_json()).collect::<Vec<_>>()
            }
        }),
        protocol: Some("wasm-execution".to_string()),
        ..Default::default()
    }
}

fn extract_i32_result(response: &FipaMessage) -> Result<i32, String> {
    response.content
        .get("result")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| "No i32 result found".to_string())
}

async fn setup_wasm_test_runtime() -> CaxtonRuntime {
    CaxtonRuntime::new(CaxtonConfig {
        max_agents: 10,
        default_timeout: Duration::from_secs(30),
        observability_enabled: true,
        wasm_isolation_enabled: true,
        resource_limits: ResourceLimits {
            max_memory_per_agent: 64 * 1024 * 1024, // 64MB
            max_cpu_time_per_agent: Duration::from_secs(10),
        },
    }).await.expect("Failed to create WASM test runtime")
}

async fn wait_for_agent_ready(runtime: &CaxtonRuntime, agent_id: &AgentId) {
    timeout(Duration::from_secs(5), async {
        loop {
            if runtime.get_agent_state(agent_id).await.unwrap() == AgentState::Ready {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await.expect("Agent never became ready");
}