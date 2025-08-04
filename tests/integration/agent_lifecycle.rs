//! Integration tests for agent lifecycle management
//!
//! Tests the complete agent lifecycle from spawning through termination,
//! including state transitions, resource monitoring, and error handling.

use caxton::*;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_agent_spawn_and_terminate() {
    // Initialize runtime
    let runtime = CaxtonRuntime::new(CaxtonConfig::default())
        .await
        .expect("Failed to create runtime");

    // Spawn an agent
    let agent_config = AgentConfig {
        name: "test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["testing".to_string(), "echo".to_string()],
        max_memory: Some(32 * 1024 * 1024), // 32MB
        timeout: Some(Duration::from_secs(10)),
    };

    let agent_id = runtime
        .spawn_agent(agent_config)
        .await
        .expect("Failed to spawn agent");

    // Verify agent is in correct state
    let state = runtime
        .get_agent_state(&agent_id)
        .await
        .expect("Failed to get agent state");
    assert_eq!(state, AgentState::Ready);

    // Get agent metadata
    let (metadata, resource_usage) = runtime
        .get_agent_metadata(&agent_id)
        .await
        .expect("Failed to get agent metadata");

    assert_eq!(metadata.name, "test-agent");
    assert_eq!(metadata.agent_type, AgentType::Worker);
    assert_eq!(metadata.capabilities, vec!["testing", "echo"]);
    assert_eq!(resource_usage.message_count, 0);

    // Verify agent appears in listings
    let agents = runtime.list_agents().await;
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].0, agent_id);
    assert_eq!(agents[0].1, vec!["testing", "echo"]);

    // Terminate the agent
    runtime
        .terminate_agent(&agent_id, Duration::from_secs(5))
        .await
        .expect("Failed to terminate agent");

    // Verify agent is no longer listed
    let agents = runtime.list_agents().await;
    assert_eq!(agents.len(), 0);

    // Shutdown runtime
    runtime
        .shutdown(Duration::from_secs(5))
        .await
        .expect("Failed to shutdown runtime");
}

#[tokio::test]
async fn test_agent_message_routing() {
    let runtime = CaxtonRuntime::new(CaxtonConfig::default())
        .await
        .expect("Failed to create runtime");

    // Spawn two agents
    let agent1_id = runtime
        .spawn_agent(AgentConfig {
            name: "agent-1".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec!["ping".to_string()],
            max_memory: None,
            timeout: None,
        })
        .await
        .expect("Failed to spawn agent 1");

    let agent2_id = runtime
        .spawn_agent(AgentConfig {
            name: "agent-2".to_string(),
            agent_type: AgentType::Coordinator,
            capabilities: vec!["pong".to_string()],
            max_memory: None,
            timeout: None,
        })
        .await
        .expect("Failed to spawn agent 2");

    // Send message from agent 1 to agent 2
    let message = FipaMessage {
        performative: FipaPerformative::Request,
        sender: agent1_id.clone(),
        receiver: agent2_id.clone(),
        content: serde_json::json!({"action": "ping", "data": "hello"}),
        conversation_id: Some(ConversationId::new()),
        protocol: Some("ping-pong".to_string()),
        ..Default::default()
    };

    let response = runtime
        .send_message(message)
        .await
        .expect("Failed to send message");

    // Verify response
    assert_eq!(response.performative, FipaPerformative::Inform);
    assert_eq!(response.sender, agent2_id);
    assert_eq!(response.receiver, agent1_id);

    // Check that message count increased
    let (_, resource_usage) = runtime
        .get_agent_metadata(&agent2_id)
        .await
        .expect("Failed to get metadata");
    assert_eq!(resource_usage.message_count, 1);

    // Check runtime metrics
    let (spawned, active, messages, _memory) = runtime.get_metrics();
    assert_eq!(spawned, 2);
    assert_eq!(active, 2);
    assert_eq!(messages, 1);

    // Cleanup
    runtime.terminate_agent(&agent1_id, Duration::from_secs(1)).await.ok();
    runtime.terminate_agent(&agent2_id, Duration::from_secs(1)).await.ok();
    runtime.shutdown(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
async fn test_agent_registry_functionality() {
    let registry = AgentRegistry::new();

    // Create test agents
    let agent1_id = AgentId::new();
    let mut metadata1 = AgentMetadata::new("worker-1".to_string(), AgentType::Worker);
    metadata1.add_capability("compute");
    metadata1.add_capability("data-processing");

    let agent2_id = AgentId::new();
    let mut metadata2 = AgentMetadata::new("coord-1".to_string(), AgentType::Coordinator);
    metadata2.add_capability("coordination");
    metadata2.add_capability("monitoring");

    let agent3_id = AgentId::new();
    let mut metadata3 = AgentMetadata::new("worker-2".to_string(), AgentType::Worker);
    metadata3.add_capability("compute");
    metadata3.set_state(AgentState::Processing);

    // Register agents
    registry.register(agent1_id.clone(), metadata1.clone());
    registry.register(agent2_id.clone(), metadata2.clone());
    registry.register(agent3_id.clone(), metadata3.clone());

    // Test total count
    assert_eq!(registry.total_count(), 3);

    // Test find by capability
    let compute_agents = registry.find_by_capability("compute");
    assert_eq!(compute_agents.len(), 2);
    assert!(compute_agents.contains(&agent1_id));
    assert!(compute_agents.contains(&agent3_id));

    let coord_agents = registry.find_by_capability("coordination");
    assert_eq!(coord_agents.len(), 1);
    assert!(coord_agents.contains(&agent2_id));

    // Test find by type
    let workers = registry.find_by_type(&AgentType::Worker);
    assert_eq!(workers.len(), 2);

    let coordinators = registry.find_by_type(&AgentType::Coordinator);
    assert_eq!(coordinators.len(), 1);

    // Test find by state
    let ready_agents = registry.find_by_state(&AgentState::Ready);
    assert_eq!(ready_agents.len(), 2);

    let processing_agents = registry.find_by_state(&AgentState::Processing);
    assert_eq!(processing_agents.len(), 1);
    assert_eq!(processing_agents[0].0, agent3_id);

    // Test count by state
    let state_counts = registry.count_by_state();
    assert_eq!(state_counts.get(&AgentState::Ready), Some(&2));
    assert_eq!(state_counts.get(&AgentState::Processing), Some(&1));

    // Test capability distribution
    let cap_dist = registry.capability_distribution();
    assert_eq!(cap_dist.get("compute"), Some(&2));
    assert_eq!(cap_dist.get("coordination"), Some(&1));
    assert_eq!(cap_dist.get("monitoring"), Some(&1));
    assert_eq!(cap_dist.get("data-processing"), Some(&1));

    // Test unregister
    let removed = registry.unregister(&agent1_id);
    assert!(removed.is_some());
    assert_eq!(registry.total_count(), 2);

    // Verify capability index is cleaned up
    let compute_agents = registry.find_by_capability("compute");
    assert_eq!(compute_agents.len(), 1);
    assert!(compute_agents.contains(&agent3_id));

    let data_proc_agents = registry.find_by_capability("data-processing");
    assert_eq!(data_proc_agents.len(), 0);
}

#[tokio::test]
async fn test_agent_monitoring() {
    let monitor = AgentMonitor::new();
    let agent_id = AgentId::new();

    // Test initial health check (should fail - agent not monitored)
    let health = monitor.health_check(&agent_id).await;
    assert!(!health.healthy);
    assert!(!health.errors.is_empty());

    // Record some events
    let health_status = HealthStatus {
        healthy: true,
        timestamp: Utc::now(),
        details: Some("All systems operational".to_string()),
        metrics: HashMap::new(),
    };

    let health_event = AgentEvent {
        agent_id: agent_id.clone(),
        timestamp: std::time::SystemTime::now(),
        event_type: AgentEventType::HealthCheck { status: health_status },
        trace_id: None,
    };

    monitor.record_event(&health_event);

    // Record resource usage
    let resource_event = AgentEvent {
        agent_id: agent_id.clone(),
        timestamp: std::time::SystemTime::now(),
        event_type: AgentEventType::ResourceUsage {
            memory_mb: 64,
            cpu_percent: 25.5,
        },
        trace_id: None,
    };

    monitor.record_event(&resource_event);

    // Record message events
    let msg_event = AgentEvent {
        agent_id: agent_id.clone(),
        timestamp: std::time::SystemTime::now(),
        event_type: AgentEventType::MessageReceived(FipaMessage::default()),
        trace_id: None,
    };

    monitor.record_event(&msg_event);

    // Test health check (should now pass)
    let health = monitor.health_check(&agent_id).await;
    assert!(health.healthy);
    assert!(health.errors.is_empty());
    assert!(health.details.contains_key("cpu_usage"));
    assert!(health.details.contains_key("memory_usage"));

    // Test performance metrics
    let metrics = monitor.get_performance_metrics(&agent_id);
    assert!(metrics.is_some());
    let metrics = metrics.unwrap();
    assert_eq!(metrics.memory_usage_bytes, 64 * 1024 * 1024);
    assert_eq!(metrics.cpu_usage_percent, 25.5);

    // Test system stats
    let stats = monitor.get_system_stats();
    assert_eq!(stats.get("monitored_agents"), Some(&"1".to_string()));
    assert_eq!(stats.get("healthy_agents"), Some(&"1".to_string()));
    assert_eq!(stats.get("unhealthy_agents"), Some(&"0".to_string()));

    // Test cleanup
    monitor.cleanup_agent(&agent_id);
    let health = monitor.health_check(&agent_id).await;
    assert!(!health.healthy);
}

#[tokio::test]
async fn test_runtime_shutdown_handling() {
    let runtime = CaxtonRuntime::new(CaxtonConfig::default())
        .await
        .expect("Failed to create runtime");

    // Spawn multiple agents
    let mut agent_ids = Vec::new();
    for i in 0..5 {
        let agent_id = runtime
            .spawn_agent(AgentConfig {
                name: format!("agent-{}", i),
                agent_type: AgentType::Worker,
                capabilities: vec![format!("task-{}", i)],
                max_memory: None,
                timeout: None,
            })
            .await
            .expect("Failed to spawn agent");
        agent_ids.push(agent_id);
    }

    // Verify all agents are running
    let (spawned, active, _messages, _memory) = runtime.get_metrics();
    assert_eq!(spawned, 5);
    assert_eq!(active, 5);

    // Shutdown runtime (should terminate all agents)
    let shutdown_result = timeout(
        Duration::from_secs(10),
        runtime.shutdown(Duration::from_secs(2))
    ).await;

    assert!(shutdown_result.is_ok());
    assert!(shutdown_result.unwrap().is_ok());

    // Verify agents are no longer listed
    let agents = runtime.list_agents().await;
    assert_eq!(agents.len(), 0);
}

#[tokio::test]
async fn test_agent_resource_limits() {
    let mut config = CaxtonConfig::default();
    config.max_agents = 2; // Limit to 2 agents

    let runtime = CaxtonRuntime::new(config)
        .await
        .expect("Failed to create runtime");

    // Spawn first agent (should succeed)
    let agent1 = runtime
        .spawn_agent(AgentConfig {
            name: "agent-1".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec!["test".to_string()],
            max_memory: None,
            timeout: None,
        })
        .await
        .expect("Failed to spawn first agent");

    // Spawn second agent (should succeed)
    let agent2 = runtime
        .spawn_agent(AgentConfig {
            name: "agent-2".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec!["test".to_string()],
            max_memory: None,
            timeout: None,
        })
        .await
        .expect("Failed to spawn second agent");

    // Try to spawn third agent (should block or fail due to semaphore)
    let spawn_future = runtime.spawn_agent(AgentConfig {
        name: "agent-3".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["test".to_string()],
        max_memory: None,
        timeout: None,
    });

    // This should timeout because we've reached the agent limit
    let result = timeout(Duration::from_millis(100), spawn_future).await;
    assert!(result.is_err()); // Should timeout

    // Terminate one agent to free up space
    runtime.terminate_agent(&agent1, Duration::from_secs(1)).await.ok();

    // Now third agent should be able to spawn
    let agent3 = runtime
        .spawn_agent(AgentConfig {
            name: "agent-3".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec!["test".to_string()],
            max_memory: None,
            timeout: None,
        })
        .await
        .expect("Failed to spawn third agent after freeing space");

    // Cleanup
    runtime.terminate_agent(&agent2, Duration::from_secs(1)).await.ok();
    runtime.terminate_agent(&agent3, Duration::from_secs(1)).await.ok();
    runtime.shutdown(Duration::from_secs(5)).await.ok();
}

#[tokio::test]
async fn test_agent_state_transitions() {
    let runtime = CaxtonRuntime::new(CaxtonConfig::default())
        .await
        .expect("Failed to create runtime");

    let agent_id = runtime
        .spawn_agent(AgentConfig {
            name: "state-test-agent".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec!["state-testing".to_string()],
            max_memory: None,
            timeout: None,
        })
        .await
        .expect("Failed to spawn agent");

    // Agent should start in Ready state
    let state = runtime.get_agent_state(&agent_id).await.unwrap();
    assert_eq!(state, AgentState::Ready);

    // Send a message to trigger Processing state
    let message = FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({"action": "process"}),
        ..Default::default()
    };

    runtime.send_message(message).await.expect("Failed to send message");

    // Give some time for message processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Agent should be back to Ready state after processing
    let state = runtime.get_agent_state(&agent_id).await.unwrap();
    assert_eq!(state, AgentState::Ready);

    // Start termination
    let terminate_future = runtime.terminate_agent(&agent_id, Duration::from_secs(5));

    // During termination, state should eventually become Terminated
    let result = terminate_future.await;
    assert!(result.is_ok());

    // Final verification through metadata
    if let Ok((metadata, _)) = runtime.get_agent_metadata(&agent_id).await {
        assert_eq!(metadata.state, AgentState::Terminated);
    }

    runtime.shutdown(Duration::from_secs(5)).await.ok();
}
