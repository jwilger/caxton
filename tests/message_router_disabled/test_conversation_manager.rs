//! Unit tests for ConversationManager
//!
//! Tests conversation creation, state management, participant tracking,
//! cleanup, and multi-turn dialogue support.

use caxton::message_router::*;
use std::collections::HashSet;
use tokio::time::{Duration, sleep};

/// Test helper to create a conversation manager
async fn create_test_conversation_manager() -> Box<dyn ConversationManager> {
    let config = RouterConfig::development();
    ConversationManagerImpl::new(config).await.unwrap()
}

/// Test helper to create a test message in a conversation
fn create_conversation_message(
    sender: AgentId,
    receiver: AgentId,
    conversation_id: ConversationId,
    content: &str,
    in_reply_to: Option<MessageId>
) -> FipaMessage {
    FipaMessage {
        performative: Performative::Request,
        sender,
        receiver,
        content: MessageContent::try_new(content.as_bytes().to_vec()).unwrap(),
        language: Some(ContentLanguage::try_new("en".to_string()).unwrap()),
        ontology: None,
        protocol: Some(ProtocolName::try_new("FIPA-REQUEST".to_string()).unwrap()),
        conversation_id: Some(conversation_id),
        reply_with: Some(MessageId::generate()),
        in_reply_to,
        message_id: MessageId::generate(),
        created_at: MessageTimestamp::now(),
        trace_context: None,
        delivery_options: DeliveryOptions::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test conversation creation
    #[tokio::test]
    async fn test_create_conversation() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        let protocol = Some(ProtocolName::try_new("FIPA-REQUEST".to_string()).unwrap());

        let result = manager.get_or_create_conversation(
            conversation_id,
            participants.clone(),
            protocol.clone()
        ).await;

        assert!(result.is_ok());
        let conversation = result.unwrap();

        assert_eq!(conversation.id, conversation_id);
        assert_eq!(conversation.participants, participants);
        assert_eq!(conversation.protocol, protocol);
        assert_eq!(conversation.message_count.as_usize(), 0);
    }

    /// Test retrieving existing conversation
    #[tokio::test]
    async fn test_get_existing_conversation() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        // Create conversation
        let result1 = manager.get_or_create_conversation(
            conversation_id,
            participants.clone(),
            None
        ).await;
        assert!(result1.is_ok());

        // Get same conversation again
        let result2 = manager.get_or_create_conversation(
            conversation_id,
            participants.clone(),
            None
        ).await;
        assert!(result2.is_ok());

        let conv1 = result1.unwrap();
        let conv2 = result2.unwrap();

        assert_eq!(conv1.id, conv2.id);
        assert_eq!(conv1.created_at.as_system_time(), conv2.created_at.as_system_time());
    }

    /// Test conversation with too many participants
    #[tokio::test]
    async fn test_conversation_too_many_participants() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let mut participants = HashSet::new();

        // Add more participants than the limit allows
        for i in 0..101 {  // Assuming max is 100
            participants.insert(AgentId::generate());
        }

        let result = manager.get_or_create_conversation(
            conversation_id,
            participants,
            None
        ).await;

        assert!(result.is_err());
        if let Err(ConversationError::TooManyParticipants { count, max }) = result {
            assert_eq!(count, 101);
            assert_eq!(max, 100);
        } else {
            panic!("Expected TooManyParticipants error");
        }
    }

    /// Test updating conversation with message
    #[tokio::test]
    async fn test_update_conversation_with_message() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        // Create conversation
        manager.get_or_create_conversation(
            conversation_id,
            participants.clone(),
            None
        ).await.unwrap();

        // Create and send first message
        let message1 = create_conversation_message(
            agent1, agent2, conversation_id, "Hello!", None
        );

        let result = manager.update_conversation(conversation_id, &message1).await;
        assert!(result.is_ok());

        // Verify conversation was updated
        let updated_conv = manager.get_or_create_conversation(
            conversation_id,
            participants,
            None
        ).await.unwrap();

        assert_eq!(updated_conv.message_count.as_usize(), 1);
    }

    /// Test multi-turn conversation
    #[tokio::test]
    async fn test_multi_turn_conversation() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        // Create conversation
        manager.get_or_create_conversation(
            conversation_id,
            participants.clone(),
            Some(ProtocolName::try_new("FIPA-REQUEST".to_string()).unwrap())
        ).await.unwrap();

        // First message: Request
        let message1 = create_conversation_message(
            agent1, agent2, conversation_id, "Can you help me?", None
        );
        manager.update_conversation(conversation_id, &message1).await.unwrap();

        // Second message: Response
        let message2 = create_conversation_message(
            agent2, agent1, conversation_id, "Yes, I can help!", Some(message1.message_id)
        );
        manager.update_conversation(conversation_id, &message2).await.unwrap();

        // Third message: Follow-up
        let message3 = create_conversation_message(
            agent1, agent2, conversation_id, "Great! How do I start?", Some(message2.message_id)
        );
        manager.update_conversation(conversation_id, &message3).await.unwrap();

        // Verify conversation has correct message count
        let final_conv = manager.get_or_create_conversation(
            conversation_id,
            participants,
            None
        ).await.unwrap();

        assert_eq!(final_conv.message_count.as_usize(), 3);
    }

    /// Test get agent conversations
    #[tokio::test]
    async fn test_get_agent_conversations() {
        let manager = create_test_conversation_manager().await;

        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();
        let agent3 = AgentId::generate();

        // Create multiple conversations involving agent1
        let conv1_id = ConversationId::generate();
        let mut participants1 = HashSet::new();
        participants1.insert(agent1);
        participants1.insert(agent2);

        let conv2_id = ConversationId::generate();
        let mut participants2 = HashSet::new();
        participants2.insert(agent1);
        participants2.insert(agent3);

        // Create conversations
        manager.get_or_create_conversation(conv1_id, participants1, None).await.unwrap();
        manager.get_or_create_conversation(conv2_id, participants2, None).await.unwrap();

        // Get conversations for agent1
        let conversations = manager.get_agent_conversations(agent1).await;
        assert!(conversations.is_ok());

        let conversations = conversations.unwrap();
        assert_eq!(conversations.len(), 2);

        let conv_ids: HashSet<ConversationId> = conversations.iter().map(|c| c.id).collect();
        assert!(conv_ids.contains(&conv1_id));
        assert!(conv_ids.contains(&conv2_id));
    }

    /// Test get agent conversations for non-participant
    #[tokio::test]
    async fn test_get_agent_conversations_non_participant() {
        let manager = create_test_conversation_manager().await;

        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();
        let agent3 = AgentId::generate();

        // Create conversation not involving agent3
        let conv_id = ConversationId::generate();
        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        manager.get_or_create_conversation(conv_id, participants, None).await.unwrap();

        // Get conversations for agent3 (not a participant)
        let conversations = manager.get_agent_conversations(agent3).await;
        assert!(conversations.is_ok());

        let conversations = conversations.unwrap();
        assert_eq!(conversations.len(), 0);
    }

    /// Test cleanup expired conversations
    #[tokio::test]
    async fn test_cleanup_expired_conversations() {
        let mut config = RouterConfig::development();
        config.conversation_timeout_ms = ConversationTimeoutMs::try_new(1000).unwrap(); // 1 second timeout

        let manager = ConversationManagerImpl::new(config).await.unwrap();

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        // Create conversation
        manager.get_or_create_conversation(conversation_id, participants, None).await.unwrap();

        // Wait for conversation to expire
        sleep(Duration::from_millis(1100)).await;

        // Clean up expired conversations
        let cleanup_result = manager.cleanup_expired_conversations().await;
        assert!(cleanup_result.is_ok());

        let cleaned_count = cleanup_result.unwrap();
        assert!(cleaned_count >= 1);
    }

    /// Test conversation statistics
    #[tokio::test]
    async fn test_conversation_statistics() {
        let manager = create_test_conversation_manager().await;

        // Get initial stats
        let stats = manager.get_conversation_stats().await;
        assert!(stats.is_ok());
        let initial_stats = stats.unwrap();

        // Create some conversations
        for i in 0..5 {
            let conversation_id = ConversationId::generate();
            let agent1 = AgentId::generate();
            let agent2 = AgentId::generate();

            let mut participants = HashSet::new();
            participants.insert(agent1);
            participants.insert(agent2);

            manager.get_or_create_conversation(conversation_id, participants, None).await.unwrap();

            // Add messages to some conversations
            if i < 3 {
                let message = create_conversation_message(
                    agent1, agent2, conversation_id, &format!("Message {}", i), None
                );
                manager.update_conversation(conversation_id, &message).await.unwrap();
            }
        }

        // Get updated stats
        let updated_stats = manager.get_conversation_stats().await.unwrap();

        assert!(updated_stats.total_active >= initial_stats.total_active);
        assert!(updated_stats.total_created.as_usize() >= initial_stats.total_created.as_usize());
    }

    /// Test conversation not found error
    #[tokio::test]
    async fn test_update_nonexistent_conversation() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let message = create_conversation_message(
            agent1, agent2, conversation_id, "Hello to void!", None
        );

        let result = manager.update_conversation(conversation_id, &message).await;
        assert!(result.is_err());

        if let Err(ConversationError::ConversationNotFound { conversation_id: missing_id }) = result {
            assert_eq!(missing_id, conversation_id);
        } else {
            panic!("Expected ConversationNotFound error");
        }
    }

    /// Test concurrent conversation operations
    #[tokio::test]
    async fn test_concurrent_conversation_operations() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        // Create conversation
        manager.get_or_create_conversation(
            conversation_id,
            participants.clone(),
            None
        ).await.unwrap();

        // Concurrent message updates
        let mut handles = vec![];
        for i in 0..10 {
            let manager_clone = manager.clone(); // Manager should implement Clone or use Arc
            let message = create_conversation_message(
                agent1, agent2, conversation_id, &format!("Concurrent message {}", i), None
            );

            let handle = tokio::spawn(async move {
                manager_clone.update_conversation(conversation_id, &message).await
            });
            handles.push(handle);
        }

        // Wait for all updates
        let results = futures::future::join_all(handles).await;

        // All updates should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        // Verify final message count
        let final_conv = manager.get_or_create_conversation(
            conversation_id,
            participants,
            None
        ).await.unwrap();

        assert_eq!(final_conv.message_count.as_usize(), 10);
    }

    /// Test conversation with different protocols
    #[tokio::test]
    async fn test_conversations_with_different_protocols() {
        let manager = create_test_conversation_manager().await;

        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        // Create conversations with different protocols
        let protocols = vec![
            "FIPA-REQUEST",
            "FIPA-QUERY",
            "FIPA-CONTRACT-NET",
            "FIPA-AUCTION",
        ];

        let mut conversation_ids = vec![];

        for protocol_name in protocols {
            let conversation_id = ConversationId::generate();
            let protocol = Some(ProtocolName::try_new(protocol_name.to_string()).unwrap());

            let result = manager.get_or_create_conversation(
                conversation_id,
                participants.clone(),
                protocol.clone()
            ).await;

            assert!(result.is_ok());
            let conversation = result.unwrap();
            assert_eq!(conversation.protocol, protocol);

            conversation_ids.push(conversation_id);
        }

        // Verify all conversations exist and have correct protocols
        assert_eq!(conversation_ids.len(), 4);
    }

    /// Test conversation last activity tracking
    #[tokio::test]
    async fn test_conversation_last_activity_tracking() {
        let manager = create_test_conversation_manager().await;

        let conversation_id = ConversationId::generate();
        let agent1 = AgentId::generate();
        let agent2 = AgentId::generate();

        let mut participants = HashSet::new();
        participants.insert(agent1);
        participants.insert(agent2);

        // Create conversation
        let initial_conv = manager.get_or_create_conversation(
            conversation_id,
            participants.clone(),
            None
        ).await.unwrap();

        let initial_activity = initial_conv.last_activity;

        // Wait a bit
        sleep(Duration::from_millis(10)).await;

        // Add a message
        let message = create_conversation_message(
            agent1, agent2, conversation_id, "Activity update", None
        );
        manager.update_conversation(conversation_id, &message).await.unwrap();

        // Check that last activity was updated
        let updated_conv = manager.get_or_create_conversation(
            conversation_id,
            participants,
            None
        ).await.unwrap();

        assert!(updated_conv.last_activity.as_system_time() > initial_activity.as_system_time());
    }
}
