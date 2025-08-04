//! Integration tests for agent coordination and lifecycle management
//! 
//! Tests agent spawning, state transitions, message routing, and termination
//! using testcontainers for isolated testing environments.

use caxton::*;
use serial_test::serial;
use std::time::Duration;
use testcontainers::*;
use tokio::time::timeout;
use tracing_test::traced_test;

/// Test agent lifecycle from creation to termination
#[tokio::test]
#[traced_test]
#[serial]
async fn test_agent_lifecycle_complete_flow() {
    let runtime = setup_test_runtime().await;
    
    // Spawn a coordinator agent
    let agent_config = AgentConfig {
        name: "test-coordinator".to_string(),
        agent_type: AgentType::Coordinator,
        capabilities: vec!["routing".to_string(), "monitoring".to_string()],
        max_memory: Some(64 * 1024 * 1024), // 64MB
        timeout: Some(Duration::from_secs(30)),
    };
    
    let agent_id = runtime.spawn_agent(agent_config).await
        .expect("Failed to spawn agent");
    
    // Verify agent state progression
    assert_eq!(runtime.get_agent_state(&agent_id).await.unwrap(), AgentState::Initializing);
    
    // Wait for agent to become ready
    timeout(Duration::from_secs(5), async {
        loop {
            if runtime.get_agent_state(&agent_id).await.unwrap() == AgentState::Ready {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await.expect("Agent never became ready");
    
    // Send a health check message
    let health_request = create_fipa_request(&agent_id, "health_check", serde_json::json!({}));
    let response = runtime.send_message(health_request).await
        .expect("Failed to send health check");
    
    assert_eq!(response.performative, FipaPerformative::Inform);
    assert!(response.content.get("healthy").unwrap().as_bool().unwrap());
    
    // Terminate agent gracefully
    runtime.terminate_agent(&agent_id, Duration::from_secs(5)).await
        .expect("Failed to terminate agent");
    
    assert_eq!(runtime.get_agent_state(&agent_id).await.unwrap(), AgentState::Terminated);
    
    // Verify observability events were emitted
    let events = runtime.get_agent_events(&agent_id).await;
    assert!(events.iter().any(|e| matches!(e.event_type, AgentEventType::StateChange { 
        from: AgentState::Initializing, 
        to: AgentState::Ready 
    })));
    assert!(events.iter().any(|e| matches!(e.event_type, AgentEventType::MessageReceived(_))));
    assert!(events.iter().any(|e| matches!(e.event_type, AgentEventType::StateChange { 
        from: AgentState::Ready, 
        to: AgentState::Terminated 
    })));
}

/// Test multi-agent coordination with message passing
#[tokio::test]
#[traced_test]
#[serial]
async fn test_multi_agent_coordination() {
    let runtime = setup_test_runtime().await;
    
    // Spawn coordinator and worker agents
    let coordinator_id = runtime.spawn_agent(AgentConfig {
        name: "coordinator".to_string(),
        agent_type: AgentType::Coordinator,
        capabilities: vec!["task_assignment".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn coordinator");
    
    let worker_ids: Vec<_> = (0..3).map(|i| {
        runtime.spawn_agent(AgentConfig {
            name: format!("worker-{}", i),
            agent_type: AgentType::Worker,
            capabilities: vec!["task_execution".to_string()],
            max_memory: Some(16 * 1024 * 1024),
            timeout: Some(Duration::from_secs(10)),
        })
    }).collect::<Vec<_>>();
    
    // Wait for all agents to be ready
    for agent_id in std::iter::once(&coordinator_id).chain(worker_ids.iter()) {
        wait_for_agent_ready(&runtime, agent_id).await;
    }
    
    // Coordinator assigns tasks to workers
    let task_assignments = worker_ids.iter().enumerate().map(|(i, worker_id)| {
        create_fipa_request(worker_id, "execute_task", serde_json::json!({
            "task_id": format!("task-{}", i),
            "payload": format!("Process data batch {}", i)
        }))
    }).collect::<Vec<_>>();
    
    // Send all task assignments
    let mut responses = Vec::new();
    for assignment in task_assignments {
        let response = runtime.send_message(assignment).await
            .expect("Failed to send task assignment");
        responses.push(response);
    }
    
    // Verify all workers accepted tasks
    for response in responses {
        assert_eq!(response.performative, FipaPerformative::Agree);
    }
    
    // Wait for task completion notifications
    let completion_messages = timeout(Duration::from_secs(10), async {
        let mut completions = Vec::new();
        while completions.len() < worker_ids.len() {
            if let Some(msg) = runtime.receive_message().await {
                if msg.performative == FipaPerformative::Inform && 
                   msg.content.get("task_completed").is_some() {
                    completions.push(msg);
                }
            }
        }
        completions
    }).await.expect("Tasks never completed");
    
    assert_eq!(completion_messages.len(), worker_ids.len());
    
    // Verify coordination metrics
    let metrics = runtime.get_coordination_metrics().await;
    assert_eq!(metrics.active_agents, 4); // 1 coordinator + 3 workers
    assert_eq!(metrics.messages_sent, 6); // 3 assignments + 3 completions
    assert_eq!(metrics.successful_coordinations, 3);
}

/// Test agent fault tolerance and recovery
#[tokio::test]
#[traced_test] 
#[serial]
async fn test_agent_fault_tolerance() {
    let runtime = setup_test_runtime().await;
    
    // Spawn an agent that will crash
    let agent_id = runtime.spawn_agent(AgentConfig {
        name: "fault-test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["crash_simulation".to_string()],
        max_memory: Some(16 * 1024 * 1024),
        timeout: Some(Duration::from_secs(5)),
    }).await.expect("Failed to spawn agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Send a message that causes the agent to crash
    let crash_request = create_fipa_request(&agent_id, "simulate_crash", serde_json::json!({}));
    
    // This should result in agent termination
    let result = runtime.send_message(crash_request).await;
    assert!(result.is_err()); // Message should fail due to crash
    
    // Verify agent state is updated
    timeout(Duration::from_secs(3), async {
        loop {
            let state = runtime.get_agent_state(&agent_id).await.unwrap();
            if state == AgentState::Terminated {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await.expect("Agent never marked as terminated");
    
    // Verify fault tolerance metrics
    let metrics = runtime.get_fault_metrics().await;
    assert_eq!(metrics.agent_crashes, 1);
    assert_eq!(metrics.recovery_attempts, 0); // No auto-recovery in test
    
    // Verify observability captured the crash
    let events = runtime.get_agent_events(&agent_id).await;
    assert!(events.iter().any(|e| matches!(e.event_type, AgentEventType::Crashed(_))));
}

/// Test resource isolation between agents
#[tokio::test]
#[traced_test]
#[serial]
async fn test_agent_resource_isolation() {
    let runtime = setup_test_runtime().await;
    
    // Spawn two agents with different memory limits
    let high_mem_agent = runtime.spawn_agent(AgentConfig {
        name: "high-memory-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["memory_intensive".to_string()],
        max_memory: Some(64 * 1024 * 1024), // 64MB
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn high memory agent");
    
    let low_mem_agent = runtime.spawn_agent(AgentConfig {
        name: "low-memory-agent".to_string(), 
        agent_type: AgentType::Worker,
        capabilities: vec!["lightweight".to_string()],
        max_memory: Some(8 * 1024 * 1024), // 8MB
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn low memory agent");
    
    wait_for_agent_ready(&runtime, &high_mem_agent).await;
    wait_for_agent_ready(&runtime, &low_mem_agent).await;
    
    // Have high memory agent allocate memory
    let alloc_request = create_fipa_request(&high_mem_agent, "allocate_memory", 
        serde_json::json!({"size_mb": 32}));
    
    let response = runtime.send_message(alloc_request).await
        .expect("Failed to send allocation request");
    assert_eq!(response.performative, FipaPerformative::Inform);
    
    // Verify low memory agent is unaffected
    let health_request = create_fipa_request(&low_mem_agent, "health_check", serde_json::json!({}));
    let health_response = runtime.send_message(health_request).await
        .expect("Failed to check low memory agent health");
    assert_eq!(health_response.performative, FipaPerformative::Inform);
    assert!(health_response.content.get("healthy").unwrap().as_bool().unwrap());
    
    // Verify resource metrics show isolation
    let high_mem_metrics = runtime.get_agent_resource_usage(&high_mem_agent).await.unwrap();
    let low_mem_metrics = runtime.get_agent_resource_usage(&low_mem_agent).await.unwrap();
    
    assert!(high_mem_metrics.memory_used > 32 * 1024 * 1024);
    assert!(low_mem_metrics.memory_used < 2 * 1024 * 1024);
    assert_ne!(high_mem_metrics.wasm_instance_id, low_mem_metrics.wasm_instance_id);
}

// Helper functions

async fn setup_test_runtime() -> CaxtonRuntime {
    CaxtonRuntime::new(CaxtonConfig {
        max_agents: 10,
        default_timeout: Duration::from_secs(30),
        observability_enabled: true,
        resource_limits: ResourceLimits {
            max_memory_per_agent: 128 * 1024 * 1024,
            max_cpu_time_per_agent: Duration::from_secs(60),
        },
    }).await.expect("Failed to create test runtime")
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

fn create_fipa_request(receiver: &AgentId, action: &str, content: serde_json::Value) -> FipaMessage {
    FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: receiver.clone(),
        content,
        conversation_id: Some(ConversationId::new()),
        reply_to: None,
        language: Some("json".to_string()),
        ontology: Some("caxton-core".to_string()),
        protocol: Some("request-response".to_string()),
        reply_with: Some(format!("req-{}", uuid::Uuid::new_v4())),
        reply_by: Some(chrono::Utc::now() + chrono::Duration::seconds(10)),
    }
}