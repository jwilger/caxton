//! FIPA Message Protocol Compliance Tests
//! 
//! Tests that verify Caxton's implementation complies with FIPA standards:
//! - Message structure validation
//! - Performative semantics
//! - Conversation protocols
//! - Ontology and language handling

use caxton::*;
use proptest::prelude::*;
use quickcheck::quickcheck;
use serial_test::serial;
use std::collections::HashMap;
use std::time::Duration;
use tracing_test::traced_test;

/// Test FIPA message structure compliance
#[tokio::test]
#[traced_test]
async fn test_fipa_message_structure_compliance() {
    let runtime = setup_test_runtime().await;
    
    // Create a properly structured FIPA message
    let message = FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::from_string("sender-agent"),
        receiver: AgentId::from_string("receiver-agent"), 
        content: serde_json::json!({
            "action": "get-weather",
            "location": "San Francisco"
        }),
        conversation_id: Some(ConversationId::new()),
        reply_to: None,
        language: Some("json".to_string()),
        ontology: Some("weather-service".to_string()),
        protocol: Some("fipa-request".to_string()),
        reply_with: Some("req-12345".to_string()),
        reply_by: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
    };
    
    // Validate message structure
    assert!(validate_fipa_message(&message).is_ok());
    
    // Test serialization preserves all fields
    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: FipaMessage = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(message.performative, deserialized.performative);
    assert_eq!(message.sender, deserialized.sender);
    assert_eq!(message.receiver, deserialized.receiver);
    assert_eq!(message.conversation_id, deserialized.conversation_id);
    assert_eq!(message.language, deserialized.language);
    assert_eq!(message.ontology, deserialized.ontology);
    assert_eq!(message.protocol, deserialized.protocol);
}

/// Test FIPA performative semantics and valid transitions
#[tokio::test]
#[traced_test]
async fn test_fipa_performative_semantics() {
    let runtime = setup_test_runtime().await;
    
    // Spawn agent for performative testing
    let agent_id = runtime.spawn_agent(AgentConfig {
        name: "fipa-test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["fipa-compliance".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn test agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Test REQUEST -> AGREE/REFUSE performative flow
    let request = create_fipa_message(
        FipaPerformative::Request,
        &agent_id,
        serde_json::json!({"action": "execute-task", "task_id": "123"})
    );
    
    let response = runtime.send_message(request).await
        .expect("Failed to send request");
    
    // Response should be AGREE or REFUSE (not INFORM)
    assert!(matches!(response.performative, 
        FipaPerformative::Agree | FipaPerformative::Refuse));
    
    if response.performative == FipaPerformative::Agree {
        // If agreed, should eventually receive INFORM with result
        let completion = timeout(Duration::from_secs(5), runtime.receive_message()).await
            .expect("No completion message received")
            .expect("Failed to receive message");
        
        assert_eq!(completion.performative, FipaPerformative::Inform);
        assert!(completion.content.get("result").is_some());
    }
}

/// Test FIPA conversation protocols
#[tokio::test]
#[traced_test]
async fn test_fipa_conversation_protocols() {
    let runtime = setup_test_runtime().await;
    
    // Contract Net Protocol test
    let initiator_id = runtime.spawn_agent(AgentConfig {
        name: "contract-initiator".to_string(),
        agent_type: AgentType::Coordinator,
        capabilities: vec!["contract-net".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(15)),
    }).await.expect("Failed to spawn initiator");
    
    let bidder_ids: Vec<_> = (0..3).map(|i| {
        runtime.spawn_agent(AgentConfig {
            name: format!("bidder-{}", i),
            agent_type: AgentType::Worker,
            capabilities: vec!["bidding".to_string()],
            max_memory: Some(16 * 1024 * 1024),
            timeout: Some(Duration::from_secs(15)),
        })
    }).collect::<Vec<_>>();
    
    // Wait for all agents to be ready
    for agent_id in std::iter::once(&initiator_id).chain(bidder_ids.iter()) {
        wait_for_agent_ready(&runtime, agent_id).await;
    }
    
    let conversation_id = ConversationId::new();
    
    // Step 1: Send CFP (Call for Proposals) to all bidders
    for bidder_id in &bidder_ids {
        let cfp = FipaMessage {
            performative: FipaPerformative::Cfp,
            sender: initiator_id.clone(),
            receiver: bidder_id.clone(),
            content: serde_json::json!({
                "task": "data-processing",
                "deadline": "2024-12-31T23:59:59Z",
                "requirements": ["rust", "performance"]
            }),
            conversation_id: Some(conversation_id.clone()),
            protocol: Some("fipa-contract-net".to_string()),
            reply_with: Some(format!("cfp-{}", uuid::Uuid::new_v4())),
            reply_by: Some(chrono::Utc::now() + chrono::Duration::seconds(30)),
            ..Default::default()
        };
        
        runtime.send_message(cfp).await
            .expect("Failed to send CFP");
    }
    
    // Step 2: Collect proposals from bidders
    let mut proposals = Vec::new();
    for _ in 0..bidder_ids.len() {
        let response = timeout(Duration::from_secs(10), runtime.receive_message()).await
            .expect("Proposal timeout")
            .expect("Failed to receive proposal");
        
        assert!(matches!(response.performative, 
            FipaPerformative::Propose | FipaPerformative::Refuse));
        assert_eq!(response.conversation_id, Some(conversation_id.clone()));
        
        if response.performative == FipaPerformative::Propose {
            proposals.push(response);
        }
    }
    
    assert!(!proposals.is_empty(), "No proposals received");
    
    // Step 3: Accept best proposal and reject others
    let best_proposal = proposals.first().unwrap();
    
    let accept_proposal = FipaMessage {
        performative: FipaPerformative::AcceptProposal,
        sender: initiator_id.clone(),
        receiver: best_proposal.sender.clone(),
        content: serde_json::json!({"accepted": true}),
        conversation_id: Some(conversation_id.clone()),
        in_reply_to: best_proposal.reply_with.clone(),
        protocol: Some("fipa-contract-net".to_string()),
        ..Default::default()
    };
    
    runtime.send_message(accept_proposal).await
        .expect("Failed to accept proposal");
    
    // Reject other proposals
    for proposal in &proposals[1..] {
        let reject_proposal = FipaMessage {
            performative: FipaPerformative::RejectProposal,
            sender: initiator_id.clone(),
            receiver: proposal.sender.clone(),
            content: serde_json::json!({"reason": "better-proposal-selected"}),
            conversation_id: Some(conversation_id.clone()),
            in_reply_to: proposal.reply_with.clone(),
            protocol: Some("fipa-contract-net".to_string()),
            ..Default::default()
        };
        
        runtime.send_message(reject_proposal).await
            .expect("Failed to reject proposal");
    }
    
    // Verify protocol compliance
    let protocol_metrics = runtime.get_protocol_metrics("fipa-contract-net").await;
    assert_eq!(protocol_metrics.conversations_started, 1);
    assert_eq!(protocol_metrics.proposals_received, proposals.len());
    assert_eq!(protocol_metrics.successful_negotiations, 1);
}

/// Property: FIPA message validation catches malformed messages
proptest! {
    #[test]
    fn prop_fipa_message_validation(
        sender_valid in any::<bool>(),
        receiver_valid in any::<bool>(),
        performative_valid in any::<bool>(),
        content_valid in any::<bool>()
    ) {
        let message = create_test_fipa_message(
            sender_valid,
            receiver_valid, 
            performative_valid,
            content_valid
        );
        
        let is_valid = validate_fipa_message(&message).is_ok();
        let should_be_valid = sender_valid && receiver_valid && 
                             performative_valid && content_valid;
        
        prop_assert_eq!(is_valid, should_be_valid);
    }
}

/// Property: FIPA conversation IDs maintain consistency
#[quickcheck]
fn prop_conversation_id_consistency(messages: Vec<FipaMessage>) -> bool {
    let conversations = group_by_conversation(&messages);
    
    // All messages in a conversation should have the same conversation_id
    for (conv_id, msgs) in conversations {
        if msgs.iter().any(|m| m.conversation_id != Some(conv_id.clone())) {
            return false;
        }
    }
    
    true
}

/// Test FIPA ontology and language support
#[tokio::test]
#[traced_test]
async fn test_fipa_ontology_language_support() {
    let runtime = setup_test_runtime().await;
    
    let agent_id = runtime.spawn_agent(AgentConfig {
        name: "ontology-test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["multi-ontology".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn ontology test agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Test different ontologies
    let ontologies = vec![
        ("weather-service", "json"),
        ("task-management", "xml"),
        ("financial-data", "rdf"),
    ];
    
    for (ontology, language) in ontologies {
        let message = FipaMessage {
            performative: FipaPerformative::QueryRef,
            sender: AgentId::system(),
            receiver: agent_id.clone(),
            content: create_content_for_ontology(ontology, language),
            ontology: Some(ontology.to_string()),
            language: Some(language.to_string()),
            protocol: Some("fipa-query".to_string()),
            ..Default::default()
        };
        
        let response = runtime.send_message(message).await
            .expect("Failed to send ontology message");
        
        // Agent should understand the ontology and respond appropriately
        assert!(matches!(response.performative, 
            FipaPerformative::Inform | FipaPerformative::NotUnderstood));
        
        if response.performative == FipaPerformative::Inform {
            // Response should use same ontology and language
            assert_eq!(response.ontology, Some(ontology.to_string()));
            assert_eq!(response.language, Some(language.to_string()));
        }
    }
}

/// Test FIPA message ordering and delivery guarantees
#[tokio::test]
#[traced_test]
async fn test_fipa_message_ordering() {
    let runtime = setup_test_runtime().await;
    
    let sender_id = runtime.spawn_agent(AgentConfig {
        name: "message-sender".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["bulk-messaging".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(20)),
    }).await.expect("Failed to spawn sender");
    
    let receiver_id = runtime.spawn_agent(AgentConfig {
        name: "message-receiver".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["message-processing".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(20)),
    }).await.expect("Failed to spawn receiver");
    
    wait_for_agent_ready(&runtime, &sender_id).await;
    wait_for_agent_ready(&runtime, &receiver_id).await;
    
    let conversation_id = ConversationId::new();
    let message_count = 10;
    
    // Send ordered sequence of messages
    for i in 0..message_count {
        let message = FipaMessage {
            performative: FipaPerformative::Inform,
            sender: sender_id.clone(),
            receiver: receiver_id.clone(),
            content: serde_json::json!({
                "sequence_number": i,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            conversation_id: Some(conversation_id.clone()),
            protocol: Some("ordered-messaging".to_string()),
            ..Default::default()
        };
        
        runtime.send_message(message).await
            .expect("Failed to send ordered message");
    }
    
    // Verify messages were received in order
    let received_messages = runtime.get_conversation_messages(&conversation_id).await;
    assert_eq!(received_messages.len(), message_count);
    
    for (i, message) in received_messages.iter().enumerate() {
        let seq_num = message.content.get("sequence_number").unwrap().as_u64().unwrap();
        assert_eq!(seq_num, i as u64);
    }
}

/// Test FIPA error handling and recovery
#[tokio::test]
#[traced_test]
async fn test_fipa_error_handling() {
    let runtime = setup_test_runtime().await;
    
    let agent_id = runtime.spawn_agent(AgentConfig {
        name: "error-test-agent".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["error-simulation".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(10)),
    }).await.expect("Failed to spawn error test agent");
    
    wait_for_agent_ready(&runtime, &agent_id).await;
    
    // Test malformed message handling
    let malformed_message = FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({"invalid": "$$malformed$$"}),
        protocol: Some("test-protocol".to_string()),
        ..Default::default()
    };
    
    let response = runtime.send_message(malformed_message).await
        .expect("Failed to send malformed message");
    
    assert_eq!(response.performative, FipaPerformative::NotUnderstood);
    assert!(response.content.get("error").is_some());
    
    // Test timeout handling
    let timeout_message = FipaMessage {
        performative: FipaPerformative::Request,
        sender: AgentId::system(),
        receiver: agent_id.clone(),
        content: serde_json::json!({"action": "long-running-task"}),
        reply_by: Some(chrono::Utc::now() + chrono::Duration::milliseconds(100)),
        protocol: Some("test-protocol".to_string()),
        ..Default::default()
    };
    
    let result = runtime.send_message(timeout_message).await;
    
    // Should either complete quickly or return timeout error
    match result {
        Ok(response) => {
            assert!(matches!(response.performative, 
                FipaPerformative::Inform | FipaPerformative::Failure));
        }
        Err(e) => {
            assert!(e.to_string().contains("timeout"));
        }
    }
}

// Helper functions

fn create_fipa_message(performative: FipaPerformative, receiver: &AgentId, content: serde_json::Value) -> FipaMessage {
    FipaMessage {
        performative,
        sender: AgentId::system(),
        receiver: receiver.clone(),
        content,
        conversation_id: Some(ConversationId::new()),
        protocol: Some("test-protocol".to_string()),
        ..Default::default()
    }
}

fn create_test_fipa_message(
    sender_valid: bool,
    receiver_valid: bool,
    performative_valid: bool,
    content_valid: bool,
) -> FipaMessage {
    FipaMessage {
        performative: if performative_valid { 
            FipaPerformative::Request 
        } else { 
            // Invalid performative would be caught at type level
            FipaPerformative::Request
        },
        sender: if sender_valid { 
            AgentId::from_string("valid-sender") 
        } else { 
            AgentId::from_string("") // Invalid empty ID
        },
        receiver: if receiver_valid { 
            AgentId::from_string("valid-receiver") 
        } else { 
            AgentId::from_string("") // Invalid empty ID
        },
        content: if content_valid {
            serde_json::json!({"valid": true})
        } else {
            serde_json::Value::Null // Invalid content
        },
        ..Default::default()
    }
}

fn create_content_for_ontology(ontology: &str, language: &str) -> serde_json::Value {
    match (ontology, language) {
        ("weather-service", "json") => serde_json::json!({
            "query": "current_weather",
            "location": "San Francisco, CA"
        }),
        ("task-management", "xml") => serde_json::json!({
            "query": "<task><status>pending</status></task>"
        }),
        ("financial-data", "rdf") => serde_json::json!({
            "query": "@prefix fin: <http://financial.org/> . ?stock fin:price ?value ."
        }),
        _ => serde_json::json!({"generic": "query"})
    }
}

fn group_by_conversation(messages: &[FipaMessage]) -> HashMap<ConversationId, Vec<&FipaMessage>> {
    let mut conversations = HashMap::new();
    
    for message in messages {
        if let Some(conv_id) = &message.conversation_id {
            conversations.entry(conv_id.clone())
                .or_insert_with(Vec::new)
                .push(message);
        }
    }
    
    conversations
}

async fn setup_test_runtime() -> CaxtonRuntime {
    CaxtonRuntime::new(CaxtonConfig {
        max_agents: 20,
        default_timeout: Duration::from_secs(30),
        observability_enabled: true,
        fipa_compliance_strict: true,
        resource_limits: ResourceLimits::default(),
    }).await.expect("Failed to create FIPA test runtime")
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