//! Unit tests for DeliveryEngine
//!
//! Tests message delivery to local and remote agents, batch processing,
//! connection pooling, and error handling.

use caxton::message_router::*;
use tokio::time::{Duration, Instant};

/// Test helper to create a test delivery engine
async fn create_test_delivery_engine() -> Box<dyn DeliveryEngine> {
    let config = RouterConfig::development();
    DeliveryEngineImpl::new(config).await.unwrap()
}

/// Test helper to create a test message
fn create_test_message(sender: AgentId, receiver: AgentId, content: &str) -> FipaMessage {
    FipaMessage {
        performative: Performative::Request,
        sender,
        receiver,
        content: MessageContent::try_new(content.as_bytes().to_vec()).unwrap(),
        language: Some(ContentLanguage::try_new("en".to_string()).unwrap()),
        ontology: None,
        protocol: None,
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
    LocalAgent::new(
        AgentId::generate(),
        AgentName::try_new(name.to_string()).unwrap(),
        AgentState::Running,
        vec![],
        MessageTimestamp::now(),
        AgentQueueSize::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test local message delivery to running agent
    #[tokio::test]
    async fn test_deliver_local_to_running_agent() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");
        let message = create_test_message(sender.id, receiver.id, "Hello locally!");

        let start = Instant::now();
        let result = engine.deliver_local(message, receiver).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        let message_id = result.unwrap();
        assert_ne!(message_id.into_inner(), uuid::Uuid::nil());

        // Local delivery should complete in < 1ms for available agents
        assert!(duration < Duration::from_millis(1));
    }

    /// Test local message delivery to stopped agent
    #[tokio::test]
    async fn test_deliver_local_to_stopped_agent() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let mut receiver = create_test_local_agent("receiver");
        receiver.state = AgentState::Stopped;

        let message = create_test_message(sender.id, receiver.id, "Hello stopped agent!");

        let result = engine.deliver_local(message, receiver).await;
        // Should either queue the message or return an error depending on implementation
        // For stopped agents, messages should typically be moved to dead letter queue
        if result.is_err() {
            // Error is acceptable for stopped agents
            assert!(matches!(result.unwrap_err(), DeliveryError::LocalDeliveryFailed { .. }));
        }
    }

    /// Test local message delivery to draining agent
    #[tokio::test]
    async fn test_deliver_local_to_draining_agent() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let mut receiver = create_test_local_agent("receiver");
        receiver.state = AgentState::Draining;

        let message = create_test_message(sender.id, receiver.id, "Hello draining agent!");

        let result = engine.deliver_local(message, receiver).await;
        // Draining agents should still accept messages for existing conversations
        // but may reject new conversation initiating messages
        assert!(result.is_ok() || matches!(result.unwrap_err(), DeliveryError::LocalDeliveryFailed { .. }));
    }

    /// Test remote message delivery
    #[tokio::test]
    async fn test_deliver_remote() {
        let engine = create_test_delivery_engine().await;

        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();
        let remote_node = NodeId::generate();
        let message = create_test_message(sender_id, receiver_id, "Hello remotely!");

        let start = Instant::now();
        let result = engine.deliver_remote(message, remote_node).await;
        let duration = start.elapsed();

        // Remote delivery may fail in test environment due to no actual remote node
        // But should complete quickly and handle the error gracefully
        if result.is_err() {
            assert!(matches!(result.unwrap_err(), DeliveryError::RemoteDeliveryFailed { .. }));
        } else {
            // If successful, should complete in < 5ms for healthy remote nodes
            assert!(duration < Duration::from_millis(5));
        }
    }

    /// Test batch message delivery
    #[tokio::test]
    async fn test_deliver_batch() {
        let engine = create_test_delivery_engine().await;

        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();

        // Create a batch of messages
        let mut messages = vec![];
        for i in 0..10 {
            let message = create_test_message(sender_id, receiver_id, &format!("Batch message {}", i));
            messages.push(message);
        }

        let start = Instant::now();
        let results = engine.deliver_batch(messages).await;
        let duration = start.elapsed();

        assert_eq!(results.len(), 10);

        // Batch processing should be more efficient than individual deliveries
        // Even with failures, should complete reasonably quickly
        assert!(duration < Duration::from_millis(100));

        // Check that each result is either success or a proper error
        for result in results {
            match result {
                Ok(message_id) => assert_ne!(message_id.into_inner(), uuid::Uuid::nil()),
                Err(error) => assert!(matches!(error, DeliveryError::LocalDeliveryFailed { .. } |
                                                     DeliveryError::RemoteDeliveryFailed { .. })),
            }
        }
    }

    /// Test empty batch delivery
    #[tokio::test]
    async fn test_deliver_empty_batch() {
        let engine = create_test_delivery_engine().await;

        let results = engine.deliver_batch(vec![]).await;
        assert_eq!(results.len(), 0);
    }

    /// Test large batch delivery performance
    #[tokio::test]
    async fn test_deliver_large_batch_performance() {
        let engine = create_test_delivery_engine().await;

        let sender_id = AgentId::generate();
        let receiver_id = AgentId::generate();

        // Create a large batch to test performance
        let batch_size = 1000;
        let mut messages = vec![];
        for i in 0..batch_size {
            let message = create_test_message(sender_id, receiver_id, &format!("Large batch message {}", i));
            messages.push(message);
        }

        let start = Instant::now();
        let results = engine.deliver_batch(messages).await;
        let duration = start.elapsed();

        assert_eq!(results.len(), batch_size);

        // Should process 1000 messages in reasonable time for performance testing
        // This is a key requirement for achieving 100K+ msg/sec
        let messages_per_second = (batch_size as f64) / duration.as_secs_f64();
        println!("Batch processing rate: {:.2} messages/second", messages_per_second);

        // Should achieve significant throughput even in test environment
        assert!(messages_per_second > 10_000.0,
                "Batch processing rate too low: {:.2} msg/sec", messages_per_second);
    }

    /// Test delivery engine health check
    #[tokio::test]
    async fn test_delivery_engine_health_check() {
        let engine = create_test_delivery_engine().await;

        let health = engine.health_check().await;
        assert!(health.is_ok());

        let status = health.unwrap();
        assert_eq!(status, HealthStatus::Healthy);
    }

    /// Test message delivery with high priority
    #[tokio::test]
    async fn test_deliver_high_priority_message() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");

        let mut message = create_test_message(sender.id, receiver.id, "High priority message");
        message.delivery_options.priority = MessagePriority::High;

        let result = engine.deliver_local(message, receiver).await;
        assert!(result.is_ok());

        // High priority messages should still deliver successfully
        // Implementation may prioritize them in queuing
        let message_id = result.unwrap();
        assert_ne!(message_id.into_inner(), uuid::Uuid::nil());
    }

    /// Test message delivery with critical priority
    #[tokio::test]
    async fn test_deliver_critical_priority_message() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");

        let mut message = create_test_message(sender.id, receiver.id, "Critical message");
        message.delivery_options.priority = MessagePriority::Critical;

        let result = engine.deliver_local(message, receiver).await;
        assert!(result.is_ok());

        let message_id = result.unwrap();
        assert_ne!(message_id.into_inner(), uuid::Uuid::nil());
    }

    /// Test message delivery with custom timeout
    #[tokio::test]
    async fn test_deliver_with_custom_timeout() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");

        let mut message = create_test_message(sender.id, receiver.id, "Timeout test message");
        message.delivery_options.timeout = Some(MessageTimeoutMs::try_new(5000).unwrap());

        let result = engine.deliver_local(message, receiver).await;
        assert!(result.is_ok());
    }

    /// Test message delivery requiring receipt
    #[tokio::test]
    async fn test_deliver_with_receipt_required() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");

        let mut message = create_test_message(sender.id, receiver.id, "Receipt required message");
        message.delivery_options.require_receipt = true;

        let result = engine.deliver_local(message, receiver).await;
        assert!(result.is_ok());

        // Message requiring receipt should still deliver
        // Implementation would track receipt requirements
        let message_id = result.unwrap();
        assert_ne!(message_id.into_inner(), uuid::Uuid::nil());
    }

    /// Test concurrent local deliveries
    #[tokio::test]
    async fn test_concurrent_local_deliveries() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");

        // Start multiple concurrent deliveries
        let mut handles = vec![];
        for i in 0..50 {
            let engine_clone = engine.clone(); // Engine should implement Clone or use Arc
            let message = create_test_message(sender.id, receiver.id, &format!("Concurrent message {}", i));
            let receiver_clone = receiver.clone();

            let handle = tokio::spawn(async move {
                engine_clone.deliver_local(message, receiver_clone).await
            });
            handles.push(handle);
        }

        // Wait for all deliveries to complete
        let results = futures::future::join_all(handles).await;

        // All deliveries should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }

    /// Test delivery with tracing context
    #[tokio::test]
    async fn test_deliver_with_trace_context() {
        let engine = create_test_delivery_engine().await;

        let sender = create_test_local_agent("sender");
        let receiver = create_test_local_agent("receiver");

        let mut message = create_test_message(sender.id, receiver.id, "Traced message");
        message.trace_context = Some(TraceContext {
            trace_id: TraceId::try_new("trace-123".to_string()).unwrap(),
            span_id: SpanId::try_new("span-456".to_string()).unwrap(),
            trace_flags: 1,
            trace_state: None,
        });

        let result = engine.deliver_local(message, receiver).await;
        assert!(result.is_ok());

        // Traced messages should deliver successfully and preserve trace context
        let message_id = result.unwrap();
        assert_ne!(message_id.into_inner(), uuid::Uuid::nil());
    }
}
