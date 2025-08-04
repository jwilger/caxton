//! Comprehensive tests for agent lifecycle management
//!
//! Tests cover the complete agent lifecycle from creation to termination,
//! including edge cases, error handling, and resource management.

use crate::lifecycle::AgentLifecycleManager;
use crate::*;
use std::sync::Arc;
use tokio_test;

#[tokio::test]
async fn test_complete_agent_lifecycle() {
    let config = CaxtonConfig::default();
    let runtime = Arc::new(CaxtonRuntime::new(config).await.unwrap());
    let lifecycle_manager = AgentLifecycleManager::new(runtime.clone()).await.unwrap();

    // Create agent
    let agent_config = AgentConfig {
        name: "test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["processing".to_string(), "computation".to_string()],
        max_memory: Some(64 * 1024 * 1024), // 64MB
        timeout: Some(Duration::from_secs(30)),
    };

    let agent_id = lifecycle_manager.create_agent(agent_config).await.unwrap();
    assert!(!agent_id.to_string().is_empty());

    // Verify agent is created and in correct state
    let status = lifecycle_manager.get_agent_status(&agent_id).await.unwrap();
    assert_eq!(status.state, AgentState::Ready);
    assert_eq!(status.metadata.capabilities.len(), 2);

    // Suspend agent
    lifecycle_manager
        .suspend_agent(&agent_id, "Testing suspension".to_string())
        .await
        .unwrap();
    let status = lifecycle_manager.get_agent_status(&agent_id).await.unwrap();
    assert_eq!(status.state, AgentState::Suspended);

    // Resume agent
    lifecycle_manager.resume_agent(&agent_id).await.unwrap();
    let status = lifecycle_manager.get_agent_status(&agent_id).await.unwrap();
    assert_eq!(status.state, AgentState::Ready);

    // Stop agent gracefully
    lifecycle_manager
        .stop_agent(&agent_id, Some("Test complete".to_string()))
        .await
        .unwrap();

    // Verify agent is no longer in registry
    let agents = lifecycle_manager.list_managed_agents().await;
    assert!(!agents.iter().any(|(id, _)| id == &agent_id));
}

#[tokio::test]
async fn test_agent_state_transitions() {
    let config = CaxtonConfig::default();
    let runtime = Arc::new(CaxtonRuntime::new(config).await.unwrap());

    // Test phantom type state transitions
    let agent_config = AgentConfig {
        name: "state-test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["testing".to_string()],
        max_memory: None,
        timeout: None,
    };

    // Create unloaded agent
    let unloaded_agent = Agent::<Unloaded>::new(agent_config);
    assert_eq!(unloaded_agent.metadata.state, AgentState::Initializing);

    // Load WASM module (simulated)
    let loaded_agent = unloaded_agent.load_wasm_module(&[]).unwrap();
    assert_eq!(loaded_agent.metadata.state, AgentState::Ready);
    assert_eq!(
        loaded_agent.metadata.properties.get("wasm_loaded"),
        Some(&"true".to_string())
    );

    // Start agent
    let mut running_agent = loaded_agent.start().unwrap();
    assert_eq!(running_agent.metadata.state, AgentState::Processing);

    // Process message
    let message = FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: running_agent.id.clone(),
        content: serde_json::json!({"test": "message"}),
        conversation_id: Some(ConversationId::new()),
        ..Default::default()
    };

    running_agent.process_message(&message).unwrap();
    assert!(running_agent
        .metadata
        .properties
        .contains_key("last_message_at"));

    // Get performance metrics
    let metrics = running_agent.get_performance_metrics();
    assert!(metrics.contains_key("capabilities_count"));
    assert_eq!(metrics.get("capabilities_count"), Some(&"1".to_string()));

    // Stop agent
    let loaded_agent = running_agent.stop();
    assert_eq!(loaded_agent.metadata.state, AgentState::Ready);

    // Unload agent
    let unloaded_agent = loaded_agent.unload();
    assert_eq!(unloaded_agent.metadata.state, AgentState::Initializing);
    assert!(!unloaded_agent
        .metadata
        .properties
        .contains_key("wasm_loaded"));
}

#[tokio::test]
async fn test_resource_monitoring() {
    let config = CaxtonConfig::default();
    let runtime = Arc::new(CaxtonRuntime::new(config).await.unwrap());

    let agent_config = AgentConfig {
        name: "resource-test-agent".to_string(),
        agent_type: AgentType::Monitor,
        capabilities: vec!["monitoring".to_string()],
        max_memory: Some(32 * 1024 * 1024), // 32MB
        timeout: Some(Duration::from_secs(60)),
    };

    let agent_id = runtime.spawn_agent(agent_config).await.unwrap();

    // Test resource usage tracking
    let resource_usage = runtime.get_agent_resource_usage(&agent_id).await.unwrap();
    assert_eq!(resource_usage.memory_bytes, 0); // Initially zero
    assert_eq!(resource_usage.message_count, 0);

    // Send message to update activity
    let message = FipaMessage {
        performative: FipaPerformative::Inform,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({"status": "active"}),
        conversation_id: Some(ConversationId::new()),
        ..Default::default()
    };

    runtime.send_message(message).await.unwrap();

    // Check updated resource usage
    let updated_usage = runtime.get_agent_resource_usage(&agent_id).await.unwrap();
    assert_eq!(updated_usage.message_count, 1);

    // Test resource limit updates
    runtime
        .update_agent_limits(
            &agent_id,
            Some(128 * 1024 * 1024), // 128MB
            Some(Duration::from_secs(120)),
        )
        .await
        .unwrap();

    let (metadata, _) = runtime.get_agent_metadata(&agent_id).await.unwrap();
    assert_eq!(
        metadata.properties.get("max_memory"),
        Some(&(128 * 1024 * 1024).to_string())
    );
    assert_eq!(
        metadata.properties.get("max_cpu_time_ms"),
        Some(&(120 * 1000).to_string())
    );

    // Test health check
    let health_status = runtime.health_check_agent(&agent_id).await.unwrap();
    assert!(health_status.healthy);
    assert!(health_status.metrics.contains_key("message_count"));

    // Clean up
    runtime
        .terminate_agent(&agent_id, Duration::from_secs(5))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_agent_registry_operations() {
    let registry = AgentRegistry::new();

    // Create test agent metadata
    let agent_id1 = AgentId::new();
    let mut metadata1 = AgentMetadata::new("agent1".to_string(), AgentType::Worker);
    metadata1.add_capability("processing");
    metadata1.add_capability("storage");

    let agent_id2 = AgentId::new();
    let mut metadata2 = AgentMetadata::new("agent2".to_string(), AgentType::Coordinator);
    metadata2.add_capability("coordination");
    metadata2.add_capability("processing");

    // Register agents
    registry.register(agent_id1.clone(), metadata1.clone());
    registry.register(agent_id2.clone(), metadata2.clone());

    // Test registry operations
    assert_eq!(registry.total_count(), 2);

    // Test capability-based search
    let processing_agents = registry.find_by_capability("processing");
    assert_eq!(processing_agents.len(), 2);
    assert!(processing_agents.contains(&agent_id1));
    assert!(processing_agents.contains(&agent_id2));

    let storage_agents = registry.find_by_capability("storage");
    assert_eq!(storage_agents.len(), 1);
    assert!(storage_agents.contains(&agent_id1));

    // Test type-based search
    let worker_agents = registry.find_by_type(&AgentType::Worker);
    assert_eq!(worker_agents.len(), 1);
    assert_eq!(worker_agents[0].0, agent_id1);

    // Test capability distribution
    let capability_dist = registry.capability_distribution();
    assert_eq!(capability_dist.get("processing"), Some(&2));
    assert_eq!(capability_dist.get("storage"), Some(&1));
    assert_eq!(capability_dist.get("coordination"), Some(&1));

    // Test metadata update
    let mut updated_metadata = metadata1.clone();
    updated_metadata.add_capability("new_capability");
    registry
        .update_metadata(&agent_id1, updated_metadata)
        .unwrap();

    let retrieved_metadata = registry.get_metadata(&agent_id1).unwrap();
    assert!(retrieved_metadata
        .capabilities
        .contains(&"new_capability".to_string()));

    // Test unregister
    let removed_metadata = registry.unregister(&agent_id1).unwrap();
    assert_eq!(removed_metadata.name, "agent1");
    assert_eq!(registry.total_count(), 1);

    // Verify capability index cleanup
    let storage_agents_after = registry.find_by_capability("storage");
    assert_eq!(storage_agents_after.len(), 0);
}

#[tokio::test]
async fn test_error_handling() {
    let config = CaxtonConfig::default();
    let runtime = Arc::new(CaxtonRuntime::new(config).await.unwrap());

    // Test operations on non-existent agent
    let non_existent_id = AgentId::new();

    let result = runtime.get_agent_state(&non_existent_id).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        CaxtonError::Agent(msg) => assert!(msg.contains("Agent not found")),
        _ => panic!("Expected Agent error"),
    }

    // Test invalid state transitions
    let result = runtime.suspend_agent(&non_existent_id).await;
    assert!(result.is_err());

    let result = runtime.resume_agent(&non_existent_id).await;
    assert!(result.is_err());

    // Test resource limit updates on non-existent agent
    let result = runtime
        .update_agent_limits(&non_existent_id, Some(1024), None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let config = CaxtonConfig {
        max_agents: 10,
        ..Default::default()
    };
    let runtime = Arc::new(CaxtonRuntime::new(config).await.unwrap());
    let lifecycle_manager = AgentLifecycleManager::new(runtime.clone()).await.unwrap();

    // Create multiple agents concurrently
    let mut handles = Vec::new();
    for i in 0..5 {
        let manager = lifecycle_manager.clone();
        let handle = tokio::spawn(async move {
            let config = AgentConfig {
                name: format!("concurrent-agent-{}", i),
                agent_type: AgentType::Worker,
                capabilities: vec![format!("capability-{}", i)],
                max_memory: Some(16 * 1024 * 1024),
                timeout: Some(Duration::from_secs(30)),
            };
            manager.create_agent(config).await
        });
        handles.push(handle);
    }

    // Wait for all agents to be created
    let mut agent_ids = Vec::new();
    for handle in handles {
        let agent_id = handle.await.unwrap().unwrap();
        agent_ids.push(agent_id);
    }

    assert_eq!(agent_ids.len(), 5);

    // Verify all agents are managed
    let managed_agents = lifecycle_manager.list_managed_agents().await;
    assert_eq!(managed_agents.len(), 5);

    // Clean up all agents concurrently
    let mut cleanup_handles = Vec::new();
    for agent_id in agent_ids {
        let manager = lifecycle_manager.clone();
        let handle = tokio::spawn(async move {
            manager
                .stop_agent(&agent_id, Some("Concurrent test cleanup".to_string()))
                .await
        });
        cleanup_handles.push(handle);
    }

    // Wait for all cleanup operations
    for handle in cleanup_handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all agents are cleaned up
    let remaining_agents = lifecycle_manager.list_managed_agents().await;
    assert_eq!(remaining_agents.len(), 0);
}

#[tokio::test]
async fn test_lifecycle_statistics() {
    let config = CaxtonConfig::default();
    let runtime = Arc::new(CaxtonRuntime::new(config).await.unwrap());
    let lifecycle_manager = AgentLifecycleManager::new(runtime.clone()).await.unwrap();

    // Create agents with different types and capabilities
    let configs = vec![
        AgentConfig {
            name: "worker1".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec!["processing".to_string(), "storage".to_string()],
            max_memory: None,
            timeout: None,
        },
        AgentConfig {
            name: "coordinator1".to_string(),
            agent_type: AgentType::Coordinator,
            capabilities: vec!["coordination".to_string(), "monitoring".to_string()],
            max_memory: None,
            timeout: None,
        },
        AgentConfig {
            name: "monitor1".to_string(),
            agent_type: AgentType::Monitor,
            capabilities: vec!["monitoring".to_string(), "logging".to_string()],
            max_memory: None,
            timeout: None,
        },
    ];

    let mut agent_ids = Vec::new();
    for config in configs {
        let agent_id = lifecycle_manager.create_agent(config).await.unwrap();
        agent_ids.push(agent_id);
    }

    // Get lifecycle statistics
    let stats = lifecycle_manager.get_lifecycle_stats().await;
    assert_eq!(stats.total_agents, 3);

    // Check state distribution (all should be ready)
    assert_eq!(stats.agents_by_state.get(&AgentState::Ready), Some(&3));

    // Check capability distribution
    assert_eq!(stats.capability_distribution.get("monitoring"), Some(&2));
    assert_eq!(stats.capability_distribution.get("processing"), Some(&1));
    assert_eq!(stats.capability_distribution.get("storage"), Some(&1));
    assert_eq!(stats.capability_distribution.get("coordination"), Some(&1));
    assert_eq!(stats.capability_distribution.get("logging"), Some(&1));

    // Test recovery settings
    assert!(stats.recovery_enabled); // Should be enabled by default

    lifecycle_manager.set_recovery_enabled(false).await;
    let updated_stats = lifecycle_manager.get_lifecycle_stats().await;
    assert!(!updated_stats.recovery_enabled);

    // Clean up
    for agent_id in agent_ids {
        lifecycle_manager.stop_agent(&agent_id, None).await.unwrap();
    }
}
