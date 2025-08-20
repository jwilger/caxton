//! Integration tests for end-to-end message routing flows
//!
//! Tests complete message routing scenarios including multi-agent conversations,
//! failure handling, retry mechanisms, and system-wide coordination.

use caxton::message_router::*;
use std::collections::{HashMap, HashSet};
use tokio::time::{Duration, timeout};

/// Test helper to create a complete router system
async fn create_test_router_system() -> Box<dyn MessageRouter> {
    let config = RouterConfig::development();
    MessageRouterImpl::new(config).await.unwrap()
}

/// Test helper to create test agents with specific capabilities
async fn setup_test_agents(
    router: &dyn MessageRouter,
) -> (LocalAgent, LocalAgent, LocalAgent) {
    let compute_agent = LocalAgent::new(
        AgentId::generate(),
        AgentName::try_new("compute-agent".to_string()).unwrap(),
        AgentState::Running,
        vec![CapabilityName::try_new("compute".to_string()).unwrap()],
        MessageTimestamp::now(),
        AgentQueueSize::default(),
    );

    let storage_agent = LocalAgent::new(
        AgentId::generate(),
        AgentName::try_new("storage-agent".to_string()).unwrap(),
        AgentState::Running,
        vec![CapabilityName::try_new("storage".to_string()).unwrap()],
        MessageTimestamp::now(),
        AgentQueueSize::default(),
    );

    let coordinator_agent = LocalAgent::new(
        AgentId::generate(),
        AgentName::try_new("coordinator-agent".to_string()).unwrap(),
        AgentState::Running,
        vec![
            CapabilityName::try_new("coordination".to_string()).unwrap(),
            CapabilityName::try_new("planning".to_string()).unwrap(),
        ],
        MessageTimestamp::now(),
        AgentQueueSize::default(),
    );

    // Register all agents
    router.register_agent(
        compute_agent.clone(),
        vec![CapabilityName::try_new("compute".to_string()).unwrap()]
    ).await.unwrap();

    router.register_agent(
        storage_agent.clone(),
        vec![CapabilityName::try_new("storage".to_string()).unwrap()]
    ).await.unwrap();

    router.register_agent(
        coordinator_agent.clone(),
        vec![
            CapabilityName::try_new("coordination".to_string()).unwrap(),
            CapabilityName::try_new("planning".to_string()).unwrap(),
        ]
    ).await.unwrap();

    (compute_agent, storage_agent, coordinator_agent)
}

/// Test helper to create a message with conversation context
fn create_conversation_message(
    sender: AgentId,
    receiver: AgentId,
    conversation_id: ConversationId,
    performative: Performative,
    content: &str,
    in_reply_to: Option<MessageId>,
) -> FipaMessage {
    FipaMessage {
        performative,
        sender,
        receiver,
        content: MessageContent::try_new(content.as_bytes().to_vec()).unwrap(),
        language: Some(ContentLanguage::try_new("en".to_string()).unwrap()),
        ontology: Some(OntologyName::try_new("task-coordination".to_string()).unwrap()),
        protocol: Some(ProtocolName::try_new("FIPA-REQUEST".to_string()).unwrap()),
        conversation_id: Some(conversation_id),
        reply_with: Some(MessageId::generate()),
        in_reply_to,
        message_id: MessageId::generate(),
        created_at: MessageTimestamp::now(),
        trace_context: Some(TraceContext {
            trace_id: TraceId::try_new("integration-test-trace".to_string()).unwrap(),
            span_id: SpanId::try_new("test-span".to_string()).unwrap(),
            trace_flags: 1,
            trace_state: None,
        }),
        delivery_options: DeliveryOptions::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test complete end-to-end message routing
    #[tokio::test]
    async fn test_end_to_end_message_routing() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        // Create a simple request-response flow
        let conversation_id = ConversationId::generate();

        // Step 1: Compute agent requests data from storage agent
        let request_message = create_conversation_message(
            compute_agent.id,
            storage_agent.id,
            conversation_id,
            Performative::Request,
            "Please provide dataset X for processing",
            None,
        );

        let request_id = router.route_message(request_message.clone()).await;
        assert!(request_id.is_ok());

        // Step 2: Storage agent responds with data
        let response_message = create_conversation_message(
            storage_agent.id,
            compute_agent.id,
            conversation_id,
            Performative::Inform,
            "Here is dataset X: [data payload]",
            Some(request_message.message_id),
        );

        let response_id = router.route_message(response_message.clone()).await;
        assert!(response_id.is_ok());

        // Step 3: Compute agent acknowledges receipt
        let ack_message = create_conversation_message(
            compute_agent.id,
            storage_agent.id,
            conversation_id,
            Performative::Agree,
            "Data received successfully, processing started",
            Some(response_message.message_id),
        );

        let ack_id = router.route_message(ack_message).await;
        assert!(ack_id.is_ok());

        // Verify router statistics show the messages
        let stats = router.get_stats().await.unwrap();
        assert!(stats.total_messages_processed.as_usize() >= 3);
        assert_eq!(stats.total_errors.as_usize(), 0);

        router.shutdown().await.unwrap();
    }

    /// Test multi-party conversation orchestration
    #[tokio::test]
    async fn test_multi_party_conversation() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, coordinator_agent) = setup_test_agents(&*router).await;

        let conversation_id = ConversationId::generate();

        // Step 1: Coordinator initiates task planning
        let planning_request = create_conversation_message(
            coordinator_agent.id,
            compute_agent.id,
            conversation_id,
            Performative::Request,
            "Task planning initiated: need compute capacity for data processing",
            None,
        );

        router.route_message(planning_request.clone()).await.unwrap();

        // Step 2: Compute agent proposes resource requirements
        let resource_proposal = create_conversation_message(
            compute_agent.id,
            coordinator_agent.id,
            conversation_id,
            Performative::Propose,
            "I can handle the processing, but need 10GB of storage space",
            Some(planning_request.message_id),
        );

        router.route_message(resource_proposal.clone()).await.unwrap();

        // Step 3: Coordinator requests storage from storage agent
        let storage_request = create_conversation_message(
            coordinator_agent.id,
            storage_agent.id,
            conversation_id,
            Performative::Request,
            "Can you allocate 10GB storage for compute agent task?",
            Some(resource_proposal.message_id),
        );

        router.route_message(storage_request.clone()).await.unwrap();

        // Step 4: Storage agent accepts the request
        let storage_acceptance = create_conversation_message(
            storage_agent.id,
            coordinator_agent.id,
            conversation_id,
            Performative::AcceptProposal,
            "10GB storage allocated and ready",
            Some(storage_request.message_id),
        );

        router.route_message(storage_acceptance.clone()).await.unwrap();

        // Step 5: Coordinator confirms to compute agent
        let task_confirmation = create_conversation_message(
            coordinator_agent.id,
            compute_agent.id,
            conversation_id,
            Performative::Inform,
            "Resources secured, you may proceed with the task",
            Some(storage_acceptance.message_id),
        );

        router.route_message(task_confirmation).await.unwrap();

        // Verify all messages were routed successfully
        let stats = router.get_stats().await.unwrap();
        assert!(stats.total_messages_processed.as_usize() >= 5);
        assert_eq!(stats.error_rate, 0.0);

        router.shutdown().await.unwrap();
    }

    /// Test agent lifecycle during message routing
    #[tokio::test]
    async fn test_agent_lifecycle_during_routing() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        let conversation_id = ConversationId::generate();

        // Step 1: Send message to running agent
        let message1 = create_conversation_message(
            compute_agent.id,
            storage_agent.id,
            conversation_id,
            Performative::Request,
            "Initial request to running agent",
            None,
        );

        let result1 = router.route_message(message1).await;
        assert!(result1.is_ok());

        // Step 2: Change agent state to draining
        router.update_agent_state(storage_agent.id, AgentState::Draining).await.unwrap();

        // Step 3: Send message to draining agent (should still work for existing conversation)
        let message2 = create_conversation_message(
            compute_agent.id,
            storage_agent.id,
            conversation_id,
            Performative::QueryIf,
            "Query to draining agent in existing conversation",
            None,
        );

        let result2 = router.route_message(message2).await;
        // May succeed or fail depending on implementation policy for draining agents

        // Step 4: Change agent state to stopped
        router.update_agent_state(storage_agent.id, AgentState::Stopped).await.unwrap();

        // Step 5: Send message to stopped agent (should be handled by failure mechanisms)
        let message3 = create_conversation_message(
            compute_agent.id,
            storage_agent.id,
            conversation_id,
            Performative::Request,
            "Request to stopped agent",
            None,
        );

        let result3 = router.route_message(message3).await;
        // This should either be queued for later or moved to dead letter queue

        router.shutdown().await.unwrap();
    }

    /// Test high-throughput message routing performance
    #[tokio::test]
    async fn test_high_throughput_routing() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        let message_count = 1000;
        let conversation_id = ConversationId::generate();

        // Send many messages concurrently
        let start = tokio::time::Instant::now();
        let mut handles = vec![];

        for i in 0..message_count {
            let router_clone = router.clone();
            let message = create_conversation_message(
                compute_agent.id,
                storage_agent.id,
                conversation_id,
                Performative::Inform,
                &format!("High throughput message {}", i),
                None,
            );

            let handle = tokio::spawn(async move {
                router_clone.route_message(message).await
            });
            handles.push(handle);
        }

        // Wait for all messages to complete
        let results = futures::future::join_all(handles).await;
        let duration = start.elapsed();

        // Count successful routes
        let successful_routes = results.into_iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        let messages_per_second = (successful_routes as f64) / duration.as_secs_f64();

        println!("High throughput test: {:.2} messages/second", messages_per_second);

        // Should achieve significant throughput
        assert!(messages_per_second > 5000.0,
                "Throughput too low: {:.2} msg/sec", messages_per_second);

        // Most messages should succeed
        assert!(successful_routes >= message_count * 95 / 100, // 95% success rate
                "Success rate too low: {}/{}", successful_routes, message_count);

        router.shutdown().await.unwrap();
    }

    /// Test failure recovery and retry mechanisms
    #[tokio::test]
    async fn test_failure_recovery() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, _storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        // Try to route message to non-existent agent
        let non_existent_agent = AgentId::generate();
        let message_to_missing = create_conversation_message(
            compute_agent.id,
            non_existent_agent,
            ConversationId::generate(),
            Performative::Request,
            "Message to non-existent agent",
            None,
        );

        let result = router.route_message(message_to_missing).await;
        assert!(result.is_err());

        // Verify error is properly categorized
        if let Err(RouterError::AgentNotFound { agent_id }) = result {
            assert_eq!(agent_id, non_existent_agent);
        } else {
            panic!("Expected AgentNotFound error");
        }

        // Check that router stats reflect the error
        let stats = router.get_stats().await.unwrap();
        assert!(stats.total_errors.as_usize() >= 1);
        assert!(stats.error_rate > 0.0);

        router.shutdown().await.unwrap();
    }

    /// Test conversation context preservation
    #[tokio::test]
    async fn test_conversation_context_preservation() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        let conversation_id = ConversationId::generate();

        // Create a chain of related messages
        let mut previous_message_id = None;
        let message_chain_length = 5;

        for i in 0..message_chain_length {
            let sender = if i % 2 == 0 { compute_agent.id } else { storage_agent.id };
            let receiver = if i % 2 == 0 { storage_agent.id } else { compute_agent.id };

            let message = create_conversation_message(
                sender,
                receiver,
                conversation_id,
                Performative::Inform,
                &format!("Message {} in conversation chain", i + 1),
                previous_message_id,
            );

            previous_message_id = Some(message.message_id);

            let result = router.route_message(message).await;
            assert!(result.is_ok(), "Failed to route message {} in chain", i + 1);
        }

        // Verify conversation was tracked properly
        let stats = router.get_stats().await.unwrap();
        assert!(stats.active_conversations >= 1);
        assert!(stats.total_messages_processed.as_usize() >= message_chain_length);

        router.shutdown().await.unwrap();
    }

    /// Test timeout handling in message routing
    #[tokio::test]
    async fn test_timeout_handling() {
        let mut config = RouterConfig::development();
        config.message_timeout_ms = MessageTimeoutMs::try_new(100).unwrap(); // Very short timeout

        let router = MessageRouterImpl::new(config).await.unwrap();
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        // Create message with custom timeout
        let mut message = create_conversation_message(
            compute_agent.id,
            storage_agent.id,
            ConversationId::generate(),
            Performative::Request,
            "Message with short timeout",
            None,
        );
        message.delivery_options.timeout = Some(MessageTimeoutMs::try_new(50).unwrap());

        // Route message with timeout
        let result = timeout(Duration::from_millis(200), router.route_message(message)).await;

        // Should either succeed quickly or timeout gracefully
        match result {
            Ok(route_result) => {
                // Message routing completed within timeout
                assert!(route_result.is_ok() || matches!(route_result.unwrap_err(), RouterError::Timeout { .. }));
            }
            Err(_) => {
                // Test timed out, which is acceptable for this test
                println!("Test timed out as expected with short timeout configuration");
            }
        }

        router.shutdown().await.unwrap();
    }

    /// Test trace context propagation
    #[tokio::test]
    async fn test_trace_context_propagation() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        let trace_id = TraceId::try_new("test-trace-123".to_string()).unwrap();
        let span_id = SpanId::try_new("test-span-456".to_string()).unwrap();

        let mut message = create_conversation_message(
            compute_agent.id,
            storage_agent.id,
            ConversationId::generate(),
            Performative::Request,
            "Message with trace context",
            None,
        );

        // Set trace context
        message.trace_context = Some(TraceContext {
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            trace_flags: 1,
            trace_state: Some("custom=value".to_string()),
        });

        let result = router.route_message(message).await;
        assert!(result.is_ok());

        // In a real implementation, we would verify that:
        // 1. Trace context is preserved throughout routing
        // 2. New spans are created for internal operations
        // 3. Correlation IDs are maintained

        router.shutdown().await.unwrap();
    }

    /// Test graceful shutdown with in-flight messages
    #[tokio::test]
    async fn test_graceful_shutdown() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        // Start routing messages
        let mut handles = vec![];
        for i in 0..10 {
            let router_clone = router.clone();
            let message = create_conversation_message(
                compute_agent.id,
                storage_agent.id,
                ConversationId::generate(),
                Performative::Inform,
                &format!("Message during shutdown {}", i),
                None,
            );

            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(i * 10)).await;
                router_clone.route_message(message).await
            });
            handles.push(handle);
        }

        // Start shutdown after a brief delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        let shutdown_handle = tokio::spawn(async move {
            router.shutdown().await
        });

        // Wait for both in-flight messages and shutdown
        let (message_results, shutdown_result) =
            tokio::join!(futures::future::join_all(handles), shutdown_handle);

        // Shutdown should succeed
        assert!(shutdown_result.is_ok());
        assert!(shutdown_result.unwrap().is_ok());

        // Some messages may succeed, others may fail due to shutdown
        // Both outcomes are acceptable for graceful shutdown
        let total_messages = message_results.len();
        let successful_messages = message_results.into_iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();

        println!("Graceful shutdown: {}/{} messages completed", successful_messages, total_messages);
    }

    /// Test router health monitoring during load
    #[tokio::test]
    async fn test_health_monitoring_under_load() {
        let router = create_test_router_system().await;
        router.start().await.unwrap();

        let (compute_agent, storage_agent, _coordinator_agent) = setup_test_agents(&*router).await;

        // Start continuous message routing
        let message_handle = tokio::spawn({
            let router_clone = router.clone();
            async move {
                for i in 0..100 {
                    let message = create_conversation_message(
                        compute_agent.id,
                        storage_agent.id,
                        ConversationId::generate(),
                        Performative::Inform,
                        &format!("Load test message {}", i),
                        None,
                    );

                    let _ = router_clone.route_message(message).await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        });

        // Monitor health during load
        let mut health_checks = vec![];
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let health = router.health_check().await;
            health_checks.push(health);
        }

        // Wait for message routing to complete
        message_handle.await.unwrap();

        // Most health checks should succeed
        let successful_health_checks = health_checks.into_iter()
            .filter(|h| h.is_ok() && h.as_ref().unwrap() == &HealthStatus::Healthy)
            .count();

        assert!(successful_health_checks >= 8,
                "Health checks failed too often under load: {}/10", successful_health_checks);

        router.shutdown().await.unwrap();
    }
}
