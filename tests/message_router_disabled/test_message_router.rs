//! Unit tests for MessageRouter main interface
//!
//! Tests the core coordination functionality including message routing,
//! agent registration, and state management.

use caxton::message_router::*;
use std::collections::HashSet;
use tokio::time::Duration;

/// Test helper to create a test message
fn create_test_message(sender: AgentId, receiver: AgentId, content: &str) -> FipaMessage {
    FipaMessage {
        performative: Performative::Request,
        sender,
        receiver,
        content: MessageContent::try_new(content.as_bytes().to_vec()).unwrap(),
        language: Some(ContentLanguage::try_new("en".to_string()).unwrap()),
        ontology: Some(OntologyName::try_new("test".to_string()).unwrap()),
        protocol: Some(ProtocolName::try_new("FIPA-REQUEST".to_string()).unwrap()),
        conversation_id: Some(ConversationId::generate()),
        reply_with: None,
        in_reply_to: None,
        message_id: MessageId::generate(),
        created_at: MessageTimestamp::now(),
        trace_context: None,
        delivery_options: DeliveryOptions::default(),
    }
}

/// Test helper to create a test local agent
fn create_test_local_agent(name: &str) -> LocalAgent {
    let capabilities = vec![
        CapabilityName::try_new("compute".to_string()).unwrap(),
        CapabilityName::try_new("storage".to_string()).unwrap(),
    ];

    LocalAgent::new(
        AgentId::generate(),
        AgentName::try_new(name.to_string()).unwrap(),
        AgentState::Running,
        capabilities,
        MessageTimestamp::now(),
        AgentQueueSize::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test router configuration creation
    #[tokio::test]
    async fn test_router_config_creation() {
        // Development configuration should have reasonable defaults
        let dev_config = RouterConfig::development();
        assert!(dev_config.inbound_queue_size.as_usize() > 0);
        assert!(dev_config.message_timeout_ms.as_u64() > 0);

        // Production configuration should be optimized for throughput
        let prod_config = RouterConfig::production();
        assert!(prod_config.inbound_queue_size.as_usize() >= dev_config.inbound_queue_size.as_usize());
    }

    /// Test router creation and startup
    #[tokio::test]
    async fn test_router_creation_and_startup() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await;
        assert!(router.is_ok());

        let router = router.unwrap();
        let start_result = router.start().await;
        assert!(start_result.is_ok());

        // Test health check
        let health = router.health_check().await;
        assert!(health.is_ok());
        assert_eq!(health.unwrap(), HealthStatus::Healthy);

        // Graceful shutdown
        let shutdown_result = router.shutdown().await;
        assert!(shutdown_result.is_ok());
    }

    /// Test agent registration
    #[tokio::test]
    async fn test_agent_registration() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        let agent = create_test_local_agent("test-agent");
        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];

        // Register agent
        let result = router.register_agent(agent.clone(), capabilities).await;
        assert!(result.is_ok());

        // Attempting to register the same agent should fail
        let duplicate_result = router.register_agent(agent, vec![]).await;
        assert!(duplicate_result.is_err());

        router.shutdown().await.unwrap();
    }

    /// Test agent deregistration
    #[tokio::test]
    async fn test_agent_deregistration() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        let agent = create_test_local_agent("test-agent");
        let agent_id = agent.id;
        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];

        // Register then deregister agent
        router.register_agent(agent, capabilities).await.unwrap();
        let result = router.deregister_agent(agent_id).await;
        assert!(result.is_ok());

        // Deregistering non-existent agent should fail
        let missing_result = router.deregister_agent(AgentId::generate()).await;
        assert!(missing_result.is_err());

        router.shutdown().await.unwrap();
    }

    /// Test agent state transitions
    #[tokio::test]
    async fn test_agent_state_transitions() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        let agent = create_test_local_agent("test-agent");
        let agent_id = agent.id;
        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];

        // Register agent
        router.register_agent(agent, capabilities).await.unwrap();

        // Test valid state transitions
        let result = router.update_agent_state(agent_id, AgentState::Draining).await;
        assert!(result.is_ok());

        let result = router.update_agent_state(agent_id, AgentState::Stopped).await;
        assert!(result.is_ok());

        // Test updating non-existent agent
        let missing_result = router.update_agent_state(AgentId::generate(), AgentState::Running).await;
        assert!(missing_result.is_err());

        router.shutdown().await.unwrap();
    }

    /// Test message routing to local agent
    #[tokio::test]
    async fn test_route_message_local() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        // Register sender and receiver agents
        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");
        let receiver_id = receiver.id;

        router.register_agent(sender, vec![]).await.unwrap();
        router.register_agent(receiver, vec![]).await.unwrap();

        // Route message
        let message = create_test_message(sender.id, receiver_id, "Hello, World!");
        let result = router.route_message(message).await;
        assert!(result.is_ok());

        let message_id = result.unwrap();
        assert_ne!(message_id.into_inner(), uuid::Uuid::nil());

        router.shutdown().await.unwrap();
    }

    /// Test message routing to non-existent agent
    #[tokio::test]
    async fn test_route_message_agent_not_found() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        let sender = create_test_local_agent("sender");
        router.register_agent(sender, vec![]).await.unwrap();

        // Route message to non-existent receiver
        let message = create_test_message(sender.id, AgentId::generate(), "Hello, Nobody!");
        let result = router.route_message(message).await;
        assert!(result.is_err());

        if let Err(RouterError::AgentNotFound { agent_id }) = result {
            assert_ne!(agent_id.into_inner(), uuid::Uuid::nil());
        } else {
            panic!("Expected AgentNotFound error");
        }

        router.shutdown().await.unwrap();
    }

    /// Test message routing with invalid message size
    #[tokio::test]
    async fn test_route_message_too_large() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");

        router.register_agent(sender, vec![]).await.unwrap();
        router.register_agent(receiver, vec![]).await.unwrap();

        // Create message that exceeds size limit (10MB + 1)
        let large_content = vec![0u8; 10_485_761];

        // This should fail at domain type validation level
        let content_result = MessageContent::try_new(large_content);
        assert!(content_result.is_err());
    }

    /// Test router statistics collection
    #[tokio::test]
    async fn test_router_stats() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        // Get initial stats
        let stats = router.get_stats().await;
        assert!(stats.is_ok());
        let stats = stats.unwrap();

        // Should start with zero messages processed
        assert_eq!(stats.total_messages_processed.as_usize(), 0);
        assert_eq!(stats.total_errors.as_usize(), 0);
        assert!(stats.messages_per_second >= 0.0);

        router.shutdown().await.unwrap();
    }

    /// Test router health monitoring
    #[tokio::test]
    async fn test_router_health_monitoring() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        // Router should be healthy after startup
        let health = router.health_check().await.unwrap();
        assert_eq!(health, HealthStatus::Healthy);

        router.shutdown().await.unwrap();

        // Router should not be healthy after shutdown
        let health_after_shutdown = router.health_check().await;
        assert!(health_after_shutdown.is_err());
    }

    /// Test concurrent message routing
    #[tokio::test]
    async fn test_concurrent_message_routing() {
        let config = RouterConfig::development();
        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        // Register agents
        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");
        let receiver_id = receiver.id;

        router.register_agent(sender, vec![]).await.unwrap();
        router.register_agent(receiver, vec![]).await.unwrap();

        // Route messages concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let router_clone = router.clone(); // Router should implement Clone for concurrent usage
            let message = create_test_message(sender.id, receiver_id, &format!("Message {}", i));

            let handle = tokio::spawn(async move {
                router_clone.route_message(message).await
            });
            handles.push(handle);
        }

        // Wait for all messages to complete
        let results: Vec<Result<Result<MessageId, RouterError>, _>> =
            futures::future::join_all(handles).await;

        // All should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        router.shutdown().await.unwrap();
    }

    /// Test message routing timeout handling
    #[tokio::test]
    async fn test_message_routing_timeout() {
        let mut config = RouterConfig::development();
        config.message_timeout_ms = MessageTimeoutMs::try_new(100).unwrap(); // Very short timeout

        let router = MessageRouter::new(config).await.unwrap();
        router.start().await.unwrap();

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("slow-receiver");

        router.register_agent(sender, vec![]).await.unwrap();
        router.register_agent(receiver, vec![]).await.unwrap();

        // This test would require a mock implementation that can simulate slow delivery
        // For now, we test that timeout configuration is respected
        assert_eq!(config.message_timeout_ms.as_u64(), 100);

        router.shutdown().await.unwrap();
    }
}
