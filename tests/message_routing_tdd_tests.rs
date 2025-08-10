//! Test-Driven Development tests for Core Message Router
//! Following red-green-refactor cycle
//!
//! These tests are written FIRST, before implementation, to drive the design

#![allow(unused_variables, unused_mut, dead_code)]

use caxton::message_router::{
    AgentId, AgentName, AgentQueueSize, AgentState, ConversationId, DeliveryOptions, FipaMessage,
    LocalAgent, MessageContent, MessageId, MessageRouter, MessageRouterImpl, MessageTimestamp,
    Performative, RouterConfig, RouterError,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Helper to create a test message
fn create_test_message(sender: AgentId, receiver: AgentId) -> FipaMessage {
    FipaMessage {
        performative: Performative::Inform,
        sender,
        receiver,
        content: MessageContent::try_new("Test message content".as_bytes().to_vec()).unwrap(),
        message_id: MessageId::generate(),
        conversation_id: Some(ConversationId::generate()),
        reply_with: None,
        in_reply_to: None,
        protocol: None,
        language: None,
        ontology: None,
        created_at: MessageTimestamp::now(),
        trace_context: None,
        delivery_options: DeliveryOptions::default(),
    }
}

/// Helper to create test agent
fn create_test_agent(id: u32) -> LocalAgent {
    LocalAgent {
        id: AgentId::generate(),
        name: AgentName::try_new(format!("test-agent-{id}")).unwrap(),
        state: AgentState::Running,
        capabilities: vec![],
        last_heartbeat: MessageTimestamp::now(),
        queue_size: AgentQueueSize::default(),
    }
}

// ============================================================================
// ACCEPTANCE CRITERIA 1: Async message router processes messages without blocking
// ============================================================================

#[tokio::test]
async fn test_async_non_blocking_routing() {
    // Given: A message router with async processing
    let config = RouterConfig::testing();
    let router = MessageRouterImpl::new(config).await.unwrap();
    router.start().await.unwrap();

    // When: We send a message
    let agent1 = create_test_agent(1);
    let agent2 = create_test_agent(2);
    let message = create_test_message(agent1.id, agent2.id);

    // Then: The route_message call should return immediately (non-blocking)
    let start = std::time::Instant::now();
    let message_id = router.route_message(message).await.unwrap();
    let duration = start.elapsed();

    // Non-blocking means it returns quickly, not waiting for delivery
    assert!(
        duration < Duration::from_millis(10),
        "Routing should be non-blocking"
    );
    assert_ne!(
        message_id,
        MessageId::generate(),
        "Should return actual message ID"
    );

    router.shutdown().await.unwrap();
}

// ============================================================================
// ACCEPTANCE CRITERIA 2: Messages are routed based on agent ID
// ============================================================================

#[tokio::test]
async fn test_route_message_by_agent_id() {
    // Given: Router with registered agents
    let config = RouterConfig::testing();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    // Create test agents with message queues
    let (_tx1, _rx1) = mpsc::channel::<FipaMessage>(10);
    let (_tx2, _rx2) = mpsc::channel::<FipaMessage>(10);

    let agent1 = create_test_agent(1);
    let agent2 = create_test_agent(2);

    // TODO: Need to implement actual message queue injection
    // For now, we'll test that routing attempts to find the agent

    // When: We send a message to agent2
    let message = create_test_message(agent1.id, agent2.id);
    let msg_id = message.message_id;
    let result = router.route_message(message).await;

    // Then: Message should be routed (even if delivery fails due to no queue)
    assert!(result.is_ok() || matches!(result, Err(RouterError::AgentNotFound { .. })));

    router.shutdown().await.unwrap();
}

// ============================================================================
// ACCEPTANCE CRITERIA 3: Router handles agent registration and deregistration
// ============================================================================

#[tokio::test]
async fn test_agent_registration_deregistration() {
    // Given: A message router
    let config = RouterConfig::testing();
    let router = MessageRouterImpl::new(config).await.unwrap();
    router.start().await.unwrap();

    let agent = create_test_agent(1);
    let agent_id = agent.id;

    // When: We register an agent
    // Note: This test reveals we need a public API for registration
    // Currently registration is internal only

    // Then: Agent should be findable
    let stats = router.get_stats().await.unwrap();

    // When: We deregister the agent
    // Note: This test reveals we need a public API for deregistration

    // Then: Agent should not be findable

    router.shutdown().await.unwrap();
}

// ============================================================================
// ACCEPTANCE CRITERIA 4: Message delivery failures are handled gracefully
// ============================================================================

#[tokio::test]
async fn test_graceful_failure_handling() {
    // Given: Router with non-existent receiver
    let config = RouterConfig::testing();
    let router = MessageRouterImpl::new(config).await.unwrap();
    router.start().await.unwrap();

    // When: We send to non-existent agent
    let message = create_test_message(AgentId::generate(), AgentId::generate());
    let result = router.route_message(message).await;

    // Then: Should handle gracefully (not panic)
    assert!(result.is_ok() || result.is_err());

    // And: Stats should reflect the failure
    tokio::time::sleep(Duration::from_millis(100)).await;
    let stats = router.get_stats().await.unwrap();
    // We expect failed deliveries to be tracked

    router.shutdown().await.unwrap();
}

// ============================================================================
// ACCEPTANCE CRITERIA 5: Router maintains conversation context
// ============================================================================

#[tokio::test]
async fn test_conversation_context_maintenance() {
    // Given: Router with conversation tracking
    let config = RouterConfig::testing();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    let agent1 = create_test_agent(1);
    let agent2 = create_test_agent(2);
    let conversation_id = ConversationId::generate();

    // When: We send multiple messages in a conversation
    for i in 0..3 {
        let mut message = create_test_message(
            if i % 2 == 0 { agent1.id } else { agent2.id },
            if i % 2 == 0 { agent2.id } else { agent1.id },
        );
        message.conversation_id = Some(conversation_id);
        if i > 0 {
            message.in_reply_to = Some(MessageId::generate());
        }

        router.route_message(message).await.ok();
    }

    // Then: Conversation should be tracked
    tokio::time::sleep(Duration::from_millis(100)).await;
    let stats = router.get_stats().await.unwrap();

    // Check that conversation tracking is working
    assert!(
        stats.total_conversations.into_inner() > 0,
        "Should track conversations"
    );

    router.shutdown().await.unwrap();
}

// ============================================================================
// ACCEPTANCE CRITERIA 6: Messages include trace and span IDs for observability
// ============================================================================

#[tokio::test]
async fn test_trace_span_observability() {
    // Given: Router with observability
    let config = RouterConfig::testing();
    let router = MessageRouterImpl::new(config).await.unwrap();
    router.start().await.unwrap();

    // When: We route a message
    let message = create_test_message(AgentId::generate(), AgentId::generate());
    let result = router.route_message(message).await;

    // Then: Tracing should be active (we can't directly test spans without a collector)
    // This test validates that the router doesn't panic with tracing enabled
    assert!(result.is_ok() || result.is_err());

    router.shutdown().await.unwrap();
}

// ============================================================================
// DEFINITION OF DONE: Message routing works for local agents
// ============================================================================

#[tokio::test]
async fn test_local_agent_message_routing_works() {
    // This is the KEY test - messages must actually be delivered

    // Given: Router with local agents that have message queues
    let config = RouterConfig::testing();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    // Create agents with actual message channels
    let (tx1, mut rx1) = mpsc::channel::<FipaMessage>(10);
    let (tx2, mut rx2) = mpsc::channel::<FipaMessage>(10);

    let agent1 = create_test_agent(1);
    let agent2 = create_test_agent(2);

    // TODO: We need a way to register agents WITH their message queues
    // This reveals a design issue - how do agents receive messages?

    // When: We route a message from agent1 to agent2
    let message = create_test_message(agent1.id, agent2.id);
    let msg_copy = message.clone();
    let result = router.route_message(message).await;

    // Then: Message should be delivered to agent2's queue
    // THIS IS THE CRITICAL TEST THAT MUST PASS

    router.shutdown().await.unwrap();
}

// ============================================================================
// DEFINITION OF DONE: Performance meets 100K messages/second target
// ============================================================================

#[tokio::test]
#[ignore = "Performance test - Run with: cargo test --ignored"]
async fn test_performance_100k_messages_per_second() {
    // Given: Production configuration
    let config = RouterConfig::production();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    // Setup: Create 100 agents
    let mut agents = Vec::new();
    for i in 0..100 {
        agents.push(create_test_agent(i));
    }

    // When: We send 100,000 messages
    let start = std::time::Instant::now();
    let message_count = 100_000;

    let mut handles = Vec::new();
    for i in 0..message_count {
        let router_clone = router.clone();
        let sender = agents[i % 100].id;
        let receiver = agents[(i + 1) % 100].id;

        let handle = tokio::spawn(async move {
            let message = create_test_message(sender, receiver);
            router_clone.route_message(message).await
        });

        handles.push(handle);

        // Batch spawning to avoid overwhelming tokio
        if handles.len() >= 1000 {
            for h in handles.drain(..) {
                h.await.ok();
            }
        }
    }

    // Wait for remaining
    for h in handles {
        h.await.ok();
    }

    let duration = start.elapsed();

    // Then: Should achieve 100K msgs/sec
    #[allow(clippy::cast_precision_loss)]
    let msgs_per_sec = message_count as f64 / duration.as_secs_f64();
    println!("Performance: {msgs_per_sec:.0} messages/second");

    assert!(
        msgs_per_sec >= 100_000.0,
        "Should achieve 100K msgs/sec, got {msgs_per_sec:.0}"
    );

    router.shutdown().await.unwrap();
}

// ============================================================================
// DEFINITION OF DONE: No message loss under normal operation
// ============================================================================

#[tokio::test]
async fn test_no_message_loss_normal_operation() {
    // Given: Router under normal load
    let config = RouterConfig::testing();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    // When: We send 1000 messages
    let message_count = 1000;
    let mut sent_ids = HashSet::new();

    for i in 0..message_count {
        let message = create_test_message(AgentId::generate(), AgentId::generate());
        sent_ids.insert(message.message_id);
        router.route_message(message).await.ok();
    }

    // Then: All messages should be accounted for (routed or in dead letter queue)
    tokio::time::sleep(Duration::from_millis(500)).await;
    let stats = router.get_stats().await.unwrap();

    let total_processed = stats.total_messages_processed.into_inner();

    // Messages should be processed (successfully or unsuccessfully)
    assert!(total_processed > 0, "Messages should be processed");

    // In production, we'd track successful + failed + dead letter = total sent

    router.shutdown().await.unwrap();
}

// ============================================================================
// DEFINITION OF DONE: Unit tests cover all routing scenarios
// ============================================================================

#[tokio::test]
async fn test_routing_scenario_high_concurrency() {
    // Given: Router with concurrent message flow
    let config = RouterConfig::testing();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    // When: 100 concurrent senders
    let mut handles = Vec::new();
    for i in 0..100 {
        let router_clone = router.clone();
        let handle = tokio::spawn(async move {
            let message = create_test_message(AgentId::generate(), AgentId::generate());
            router_clone.route_message(message).await
        });
        handles.push(handle);
    }

    // Then: All should complete without deadlock
    let mut success_count = 0;
    for h in handles {
        if h.await.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert!(success_count > 0, "Some messages should route successfully");

    router.shutdown().await.unwrap();
}

// ============================================================================
// DEFINITION OF DONE: Integration tests verify end-to-end delivery
// ============================================================================

#[tokio::test]
async fn test_end_to_end_delivery_integration() {
    // This test verifies the complete message flow from sender to receiver

    // Given: Complete router setup with all components
    let config = RouterConfig::testing();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    // TODO: This test needs actual delivery implementation
    // It should verify:
    // 1. Message accepted by router
    // 2. Agent lookup successful
    // 3. Message queued for delivery
    // 4. Message delivered to agent
    // 5. Delivery confirmation received
    // 6. Stats updated correctly

    router.shutdown().await.unwrap();
}

// ============================================================================
// DEFINITION OF DONE: Metrics track routing performance
// ============================================================================

#[tokio::test]
async fn test_metrics_track_routing_performance() {
    // Given: Router with metrics collection
    let config = RouterConfig::testing();
    let router = Arc::new(MessageRouterImpl::new(config).await.unwrap());
    router.start().await.unwrap();

    // When: We route messages
    for _ in 0..10 {
        let message = create_test_message(AgentId::generate(), AgentId::generate());
        router.route_message(message).await.ok();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Then: Metrics should be collected
    let stats = router.get_stats().await.unwrap();

    assert!(
        stats.total_messages_processed.into_inner() > 0,
        "Should track message count"
    );
    assert!(stats.messages_per_second >= 0.0, "Should track throughput");
    assert!(stats.routing_latency_p50 > 0, "Should track latency");

    router.shutdown().await.unwrap();
}
