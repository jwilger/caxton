//! Basic integration test for message router
//!
//! Tests the core message router functionality with a minimal working example.

use caxton::message_router::*;
use tokio::time::Duration;

#[tokio::test]
async fn test_router_creation_and_basic_operations() {
    // Test router creation with development config
    let config = RouterConfig::development();
    assert!(config.validate().is_ok());

    let router = MessageRouterImpl::new(config).await;
    assert!(router.is_ok());

    let router = router.unwrap();

    // Test starting the router
    let start_result = router.start().await;
    assert!(start_result.is_ok());

    // Test health check
    let health = router.health_check().await;
    assert!(health.is_ok());

    // Test getting stats
    let stats = router.get_stats().await;
    assert!(stats.is_ok());
    let stats = stats.unwrap();
    assert_eq!(stats.total_messages_processed.as_usize(), 0);

    // Test shutdown
    let shutdown_result = router.shutdown().await;
    assert!(shutdown_result.is_ok());
}

#[tokio::test]
async fn test_router_configuration_variants() {
    // Test development config
    let dev_config = RouterConfig::development();
    assert!(dev_config.validate().is_ok());
    assert!(dev_config.enable_detailed_logs());
    assert!(dev_config.trace_sampling_ratio().as_f64() > 0.5); // High sampling for dev

    // Test production config
    let prod_config = RouterConfig::production();
    assert!(prod_config.validate().is_ok());
    assert!(!prod_config.enable_detailed_logs());
    assert!(prod_config.trace_sampling_ratio().as_f64() < 0.1); // Low sampling for prod
    assert!(prod_config.inbound_queue_size.as_usize() > dev_config.inbound_queue_size.as_usize());

    // Test testing config
    let test_config = RouterConfig::testing();
    assert!(test_config.validate().is_ok());
    // Testing config now has larger queue for TDD tests
    assert!(test_config.inbound_queue_size.as_usize() >= dev_config.inbound_queue_size.as_usize());
}

#[tokio::test]
async fn test_router_config_builder() {
    let config = RouterConfig::builder()
        .inbound_queue_size(ChannelCapacity::try_new(5000).unwrap())
        .message_timeout_ms(MessageTimeoutMs::try_new(15000).unwrap())
        .enable_persistence(false)
        .enable_metrics(true)
        .build();

    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.inbound_queue_size.as_usize(), 5000);
    assert_eq!(config.message_timeout_ms.as_u64(), 15000);
    assert!(!config.enable_persistence());
    assert!(config.enable_metrics());
}

#[tokio::test]
async fn test_domain_types_validation() {
    // Test successful domain type creation
    let agent_name = AgentName::try_new("test-agent".to_string());
    assert!(agent_name.is_ok());

    let capability = CapabilityName::try_new("compute".to_string());
    assert!(capability.is_ok());

    let message_content = MessageContent::try_new(b"Hello, World!".to_vec());
    assert!(message_content.is_ok());
    let content = message_content.unwrap();
    assert_eq!(content.len(), 13);
    assert!(!content.is_empty());

    // Test domain type validation failures
    let empty_name = AgentName::try_new(String::new());
    assert!(empty_name.is_err());

    let too_long_name = AgentName::try_new("x".repeat(300));
    assert!(too_long_name.is_err());

    let too_large_content = MessageContent::try_new(vec![0u8; 11_000_000]); // > 10MB
    assert!(too_large_content.is_err());
}

#[tokio::test]
async fn test_message_creation_and_validation() {
    let sender = AgentId::generate();
    let receiver = AgentId::generate();
    let conversation_id = ConversationId::generate();
    let content = MessageContent::try_new(b"Test message content".to_vec()).unwrap();

    let message = FipaMessage {
        performative: Performative::Request,
        sender,
        receiver,
        content,
        language: Some(ContentLanguage::try_new("en".to_string()).unwrap()),
        ontology: Some(OntologyName::try_new("test-ontology".to_string()).unwrap()),
        protocol: Some(ProtocolName::try_new("FIPA-REQUEST".to_string()).unwrap()),
        conversation_id: Some(conversation_id),
        reply_with: Some(MessageId::generate()),
        in_reply_to: None,
        message_id: MessageId::generate(),
        created_at: MessageTimestamp::now(),
        trace_context: None,
        delivery_options: DeliveryOptions::default(),
    };

    // Message should be valid
    assert_ne!(message.message_id.into_inner(), uuid::Uuid::nil());
    assert_ne!(message.sender.into_inner(), uuid::Uuid::nil());
    assert_ne!(message.receiver.into_inner(), uuid::Uuid::nil());
    assert!(!message.content.is_empty());
    assert_eq!(message.delivery_options.priority, MessagePriority::Normal);
    assert_eq!(message.delivery_options.max_retries.as_u8(), 3);
}

#[tokio::test]
async fn test_local_agent_creation() {
    let agent_id = AgentId::generate();
    let agent_name = AgentName::try_new("test-agent".to_string()).unwrap();
    let capabilities = vec![
        CapabilityName::try_new("compute".to_string()).unwrap(),
        CapabilityName::try_new("storage".to_string()).unwrap(),
    ];

    let agent = LocalAgent::new(
        agent_id,
        agent_name.clone(),
        AgentState::Running,
        capabilities.clone(),
        MessageTimestamp::now(),
        AgentQueueSize::try_new(1000).unwrap(),
    );

    assert_eq!(agent.id, agent_id);
    assert_eq!(agent.name, agent_name);
    assert_eq!(agent.state, AgentState::Running);
    assert_eq!(agent.capabilities.len(), 2);
    assert!(agent.is_available());
    assert_eq!(agent.queue_size.as_usize(), 1000);
}

#[tokio::test]
async fn test_conversation_creation() {
    let conversation_id = ConversationId::generate();
    let mut participants = std::collections::HashSet::new();
    participants.insert(AgentId::generate());
    participants.insert(AgentId::generate());

    let protocol = Some(ProtocolName::try_new("FIPA-REQUEST".to_string()).unwrap());
    let created_at = ConversationCreatedAt::now();

    let conversation = Conversation::new(
        conversation_id,
        participants.clone(),
        protocol.clone(),
        created_at,
    );

    assert_eq!(conversation.id, conversation_id);
    assert_eq!(conversation.participants, participants);
    assert_eq!(conversation.protocol, protocol);
    assert_eq!(conversation.message_count.as_usize(), 0);
}

#[tokio::test]
async fn test_performance_types() {
    // Test throughput and performance-related domain types
    let batch_size = MessageBatchSize::try_new(1000).unwrap();
    assert_eq!(batch_size.as_usize(), 1000);

    let worker_threads = WorkerThreadCount::try_new(8).unwrap();
    assert_eq!(worker_threads.as_usize(), 8);

    let retry_backoff = RetryBackoffFactor::try_new(2.0).unwrap();
    assert!((retry_backoff.as_f64() - 2.0).abs() < f64::EPSILON);

    let sampling_ratio = TraceSamplingRatio::try_new(0.1).unwrap();
    assert!((sampling_ratio.as_f64() - 0.1).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_timeout_and_duration_types() {
    let message_timeout = MessageTimeoutMs::try_new(30000).unwrap();
    assert_eq!(message_timeout.as_u64(), 30000);
    assert_eq!(message_timeout.as_duration(), Duration::from_millis(30000));

    let retry_delay = RetryDelayMs::try_new(1000).unwrap();
    assert_eq!(retry_delay.as_u64(), 1000);
    assert_eq!(retry_delay.as_duration(), Duration::from_millis(1000));

    let conversation_timeout = ConversationTimeoutMs::try_new(1_800_000).unwrap(); // 30 minutes
    assert_eq!(conversation_timeout.as_u64(), 1_800_000);
    assert_eq!(
        conversation_timeout.as_duration(),
        Duration::from_millis(1_800_000)
    );
}

#[test]
fn test_message_router_architecture() {
    // Test that the architecture is properly structured

    // Domain types should be available
    let _agent_id = AgentId::generate();
    let _message_id = MessageId::generate();
    let _conversation_id = ConversationId::generate();
    let _node_id = NodeId::generate();

    // Enums should have expected values
    assert_eq!(format!("{:?}", Performative::Request), "Request");
    assert_eq!(format!("{:?}", AgentState::Running), "Running");
    assert_eq!(format!("{:?}", MessagePriority::High), "High");

    // Default values should be reasonable
    assert_eq!(MessagePriority::default(), MessagePriority::Normal);
    assert_eq!(ChannelCapacity::default().as_usize(), 1000);
    assert_eq!(MaxRetries::default().as_u8(), 3);
}
