//! Property-based tests for message router domain types and invariants
//!
//! Uses proptest to generate random inputs and verify that domain type
//! validations, conversions, and business logic invariants hold.

use caxton::message_router::*;
use proptest::prelude::*;
use std::collections::HashSet;

/// Property test strategies for generating valid domain values

prop_compose! {
    fn arb_agent_name()(name in "[a-zA-Z][a-zA-Z0-9_-]{0,254}") -> AgentName {
        AgentName::try_new(name).unwrap()
    }
}

prop_compose! {
    fn arb_capability_name()(name in "[a-zA-Z][a-zA-Z0-9_-]{0,99}") -> CapabilityName {
        CapabilityName::try_new(name).unwrap()
    }
}

prop_compose! {
    fn arb_content_language()(lang in "[a-z]{2,5}") -> ContentLanguage {
        ContentLanguage::try_new(lang).unwrap()
    }
}

prop_compose! {
    fn arb_protocol_name()(protocol in "(FIPA-REQUEST|FIPA-QUERY|FIPA-CONTRACT-NET|FIPA-AUCTION|CUSTOM-[A-Z]+)") -> ProtocolName {
        ProtocolName::try_new(protocol).unwrap()
    }
}

prop_compose! {
    fn arb_message_content()(content_bytes in prop::collection::vec(any::<u8>(), 1..1024)) -> MessageContent {
        MessageContent::try_new(content_bytes).unwrap()
    }
}

prop_compose! {
    fn arb_channel_capacity()(capacity in 1usize..=1_000_000) -> ChannelCapacity {
        ChannelCapacity::try_new(capacity).unwrap()
    }
}

prop_compose! {
    fn arb_max_retries()(retries in 1u8..=10) -> MaxRetries {
        MaxRetries::try_new(retries).unwrap()
    }
}

prop_compose! {
    fn arb_retry_delay_ms()(delay in 100u64..=300_000) -> RetryDelayMs {
        RetryDelayMs::try_new(delay).unwrap()
    }
}

prop_compose! {
    fn arb_message_timeout_ms()(timeout in 1000u64..=300_000) -> MessageTimeoutMs {
        MessageTimeoutMs::try_new(timeout).unwrap()
    }
}

prop_compose! {
    fn arb_agent_queue_size()(size in 1usize..=100_000) -> AgentQueueSize {
        AgentQueueSize::try_new(size).unwrap()
    }
}

prop_compose! {
    fn arb_worker_thread_count()(count in 1usize..=32) -> WorkerThreadCount {
        WorkerThreadCount::try_new(count).unwrap()
    }
}

prop_compose! {
    fn arb_message_batch_size()(size in 1usize..=10_000) -> MessageBatchSize {
        MessageBatchSize::try_new(size).unwrap()
    }
}

prop_compose! {
    fn arb_retry_backoff_factor()(factor in 1.1f64..=5.0) -> RetryBackoffFactor {
        RetryBackoffFactor::try_new(factor).unwrap()
    }
}

prop_compose! {
    fn arb_trace_sampling_ratio()(ratio in 0.0f64..=1.0) -> TraceSamplingRatio {
        TraceSamplingRatio::try_new(ratio).unwrap()
    }
}

prop_compose! {
    fn arb_route_hops()(hops in 0u8..=255) -> RouteHops {
        RouteHops::try_new(hops).unwrap()
    }
}

prop_compose! {
    fn arb_local_agent()(
        name in arb_agent_name(),
        capabilities in prop::collection::vec(arb_capability_name(), 0..10),
        queue_size in arb_agent_queue_size(),
    ) -> LocalAgent {
        LocalAgent::new(
            AgentId::generate(),
            name,
            AgentState::Running,
            capabilities,
            MessageTimestamp::now(),
            queue_size,
        )
    }
}

prop_compose! {
    fn arb_fipa_message()(
        content in arb_message_content(),
        language in prop::option::of(arb_content_language()),
        protocol in prop::option::of(arb_protocol_name()),
        priority in prop::sample::select(vec![
            MessagePriority::Low,
            MessagePriority::Normal,
            MessagePriority::High,
            MessagePriority::Critical,
        ]),
        timeout in prop::option::of(arb_message_timeout_ms()),
        max_retries in arb_max_retries(),
        require_receipt in any::<bool>(),
    ) -> FipaMessage {
        FipaMessage {
            performative: Performative::Request,
            sender: AgentId::generate(),
            receiver: AgentId::generate(),
            content,
            language,
            ontology: None,
            protocol,
            conversation_id: Some(ConversationId::generate()),
            reply_with: Some(MessageId::generate()),
            in_reply_to: None,
            message_id: MessageId::generate(),
            created_at: MessageTimestamp::now(),
            trace_context: None,
            delivery_options: DeliveryOptions {
                priority,
                timeout,
                require_receipt,
                max_retries,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        /// Property: Domain types should preserve values through round-trip conversions
        #[test]
        fn test_domain_type_round_trip_conversions(
            capacity in arb_channel_capacity(),
            retries in arb_max_retries(),
            delay in arb_retry_delay_ms(),
            timeout in arb_message_timeout_ms(),
            queue_size in arb_agent_queue_size(),
            thread_count in arb_worker_thread_count(),
            batch_size in arb_message_batch_size(),
            backoff in arb_retry_backoff_factor(),
            sampling in arb_trace_sampling_ratio(),
            hops in arb_route_hops(),
        ) {
            // Test round-trip conversions preserve values
            assert_eq!(capacity.as_usize(), ChannelCapacity::try_new(capacity.as_usize()).unwrap().as_usize());
            assert_eq!(retries.as_u8(), MaxRetries::try_new(retries.as_u8()).unwrap().as_u8());
            assert_eq!(delay.as_u64(), RetryDelayMs::try_new(delay.as_u64()).unwrap().as_u64());
            assert_eq!(timeout.as_u64(), MessageTimeoutMs::try_new(timeout.as_u64()).unwrap().as_u64());
            assert_eq!(queue_size.as_usize(), AgentQueueSize::try_new(queue_size.as_usize()).unwrap().as_usize());
            assert_eq!(thread_count.as_usize(), WorkerThreadCount::try_new(thread_count.as_usize()).unwrap().as_usize());
            assert_eq!(batch_size.as_usize(), MessageBatchSize::try_new(batch_size.as_usize()).unwrap().as_usize());
            assert_eq!(backoff.as_f64(), RetryBackoffFactor::try_new(backoff.as_f64()).unwrap().as_f64());
            assert_eq!(sampling.as_f64(), TraceSamplingRatio::try_new(sampling.as_f64()).unwrap().as_f64());
            assert_eq!(hops.into_inner(), RouteHops::try_new(hops.into_inner()).unwrap().into_inner());
        }
    }

    proptest! {
        /// Property: Duration conversions should be consistent and valid
        #[test]
        fn test_duration_conversions(
            delay in arb_retry_delay_ms(),
            timeout in arb_message_timeout_ms(),
        ) {
            let delay_duration = delay.as_duration();
            let timeout_duration = timeout.as_duration();

            // Duration should match the millisecond value
            assert_eq!(delay_duration.as_millis() as u64, delay.as_u64());
            assert_eq!(timeout_duration.as_millis() as u64, timeout.as_u64());

            // Duration should be positive
            assert!(delay_duration > std::time::Duration::ZERO);
            assert!(timeout_duration > std::time::Duration::ZERO);
        }
    }

    proptest! {
        /// Property: String domain types should preserve content and validate length
        #[test]
        fn test_string_domain_types(
            agent_name in arb_agent_name(),
            capability in arb_capability_name(),
            language in arb_content_language(),
            protocol in arb_protocol_name(),
        ) {
            // All string types should have non-empty display representation
            assert!(!agent_name.to_string().is_empty());
            assert!(!capability.to_string().is_empty());
            assert!(!language.to_string().is_empty());
            assert!(!protocol.to_string().is_empty());

            // Display representation should match inner value
            assert_eq!(agent_name.to_string(), agent_name.into_inner());
            assert_eq!(capability.to_string(), capability.into_inner());
            assert_eq!(language.to_string(), language.into_inner());
            assert_eq!(protocol.to_string(), protocol.into_inner());

            // Should respect length constraints
            assert!(agent_name.into_inner().len() <= 255);
            assert!(capability.into_inner().len() <= 100);
            assert!(language.into_inner().len() <= 50);
            assert!(protocol.into_inner().len() <= 100);
        }
    }

    proptest! {
        /// Property: Message content should respect size limits
        #[test]
        fn test_message_content_size_limits(content in arb_message_content()) {
            // Content should not exceed maximum size (10MB)
            assert!(content.len() <= 10_485_760);

            // Content length methods should be consistent
            assert_eq!(content.len(), content.as_bytes().len());
            assert_eq!(content.is_empty(), content.len() == 0);

            // AsRef should work correctly
            assert_eq!(content.as_ref(), content.as_bytes());
        }
    }

    proptest! {
        /// Property: Local agents should maintain consistent state
        #[test]
        fn test_local_agent_invariants(agent in arb_local_agent()) {
            // Agent ID should not be nil
            assert_ne!(agent.id.into_inner(), uuid::Uuid::nil());

            // Agent name should not be empty
            assert!(!agent.name.to_string().is_empty());

            // Queue size should be positive
            assert!(agent.queue_size.as_usize() > 0);

            // Availability should match state
            let is_running = matches!(agent.state, AgentState::Running);
            assert_eq!(agent.is_available(), is_running);
        }
    }

    proptest! {
        /// Property: FIPA messages should maintain structural invariants
        #[test]
        fn test_fipa_message_invariants(message in arb_fipa_message()) {
            // Message ID should not be nil
            assert_ne!(message.message_id.into_inner(), uuid::Uuid::nil());

            // Sender and receiver should not be nil
            assert_ne!(message.sender.into_inner(), uuid::Uuid::nil());
            assert_ne!(message.receiver.into_inner(), uuid::Uuid::nil());

            // Sender and receiver should be different
            assert_ne!(message.sender, message.receiver);

            // Content should exist and respect size limits
            assert!(message.content.len() > 0);
            assert!(message.content.len() <= 10_485_760);

            // Created timestamp should be reasonable (within last hour for test)
            let now = std::time::SystemTime::now();
            let created = message.created_at.as_system_time();
            let time_diff = now.duration_since(created).unwrap_or_default();
            assert!(time_diff < std::time::Duration::from_secs(3600));

            // If conversation ID exists, should not be nil
            if let Some(conv_id) = message.conversation_id {
                assert_ne!(conv_id.into_inner(), uuid::Uuid::nil());
            }

            // If reply_with exists, should not be nil
            if let Some(reply_id) = message.reply_with {
                assert_ne!(reply_id.into_inner(), uuid::Uuid::nil());
            }

            // Delivery options should be valid
            assert!(message.delivery_options.max_retries.as_u8() >= 1);
            assert!(message.delivery_options.max_retries.as_u8() <= 10);
        }
    }

    proptest! {
        /// Property: Route information should maintain temporal consistency
        #[test]
        fn test_route_info_temporal_consistency(
            hops in arb_route_hops(),
        ) {
            let node_id = NodeId::generate();
            let updated_at = MessageTimestamp::now();
            let route = RouteInfo::new(node_id, hops, updated_at);

            // Expiration should be after update time
            assert!(route.expires_at.as_system_time() > route.updated_at.as_system_time());

            // Should be fresh initially (within default TTL)
            let default_ttl = std::time::Duration::from_secs(300);
            assert!(route.is_fresh(default_ttl));

            // Should respect hop count limits
            assert!(route.hops.into_inner() <= 255);

            // Node ID should not be nil
            assert_ne!(route.node_id.into_inner(), uuid::Uuid::nil());
        }
    }

    proptest! {
        /// Property: Conversation participants should be unique
        #[test]
        fn test_conversation_participant_uniqueness(
            participant_count in 1usize..=10,
        ) {
            let mut participants = HashSet::new();

            // Generate unique agent IDs
            for _ in 0..participant_count {
                participants.insert(AgentId::generate());
            }

            let conversation_id = ConversationId::generate();
            let created_at = ConversationCreatedAt::now();

            let conversation = Conversation::new(
                conversation_id,
                participants.clone(),
                None,
                created_at,
            );

            // Participants should match what was provided
            assert_eq!(conversation.participants.len(), participants.len());
            assert_eq!(conversation.participants, participants);

            // Initial message count should be zero
            assert_eq!(conversation.message_count.as_usize(), 0);

            // Conversation ID should not be nil
            assert_ne!(conversation.id.into_inner(), uuid::Uuid::nil());
        }
    }

    proptest! {
        /// Property: Numeric domain types should respect bounds
        #[test]
        fn test_numeric_bounds_validation(
            channel_capacity in 1usize..=1_000_000,
            max_retries in 1u8..=10,
            retry_delay in 100u64..=300_000,
            timeout in 1000u64..=300_000,
            queue_size in 1usize..=100_000,
            thread_count in 1usize..=32,
            batch_size in 1usize..=10_000,
            backoff_factor in 1.1f64..=5.0,
            sampling_ratio in 0.0f64..=1.0,
            route_hops in 0u8..=255,
        ) {
            // All domain type creations should succeed with valid bounds
            let capacity = ChannelCapacity::try_new(channel_capacity);
            assert!(capacity.is_ok());

            let retries = MaxRetries::try_new(max_retries);
            assert!(retries.is_ok());

            let delay = RetryDelayMs::try_new(retry_delay);
            assert!(delay.is_ok());

            let timeout_val = MessageTimeoutMs::try_new(timeout);
            assert!(timeout_val.is_ok());

            let queue = AgentQueueSize::try_new(queue_size);
            assert!(queue.is_ok());

            let threads = WorkerThreadCount::try_new(thread_count);
            assert!(threads.is_ok());

            let batch = MessageBatchSize::try_new(batch_size);
            assert!(batch.is_ok());

            let backoff = RetryBackoffFactor::try_new(backoff_factor);
            assert!(backoff.is_ok());

            let sampling = TraceSamplingRatio::try_new(sampling_ratio);
            assert!(sampling.is_ok());

            let hops = RouteHops::try_new(route_hops);
            assert!(hops.is_ok());
        }
    }

    proptest! {
        /// Property: Invalid values should be rejected consistently
        #[test]
        fn test_invalid_value_rejection(
            invalid_capacity in prop::sample::select(vec![0usize, 1_000_001]),
            invalid_retries in prop::sample::select(vec![0u8, 11]),
            invalid_delay in prop::sample::select(vec![99u64, 300_001]),
            invalid_timeout in prop::sample::select(vec![999u64, 300_001]),
            invalid_backoff in prop::sample::select(vec![1.0f64, 5.1]),
            invalid_sampling in prop::sample::select(vec![-0.1f64, 1.1]),
        ) {
            // All invalid values should be rejected
            assert!(ChannelCapacity::try_new(invalid_capacity).is_err());
            assert!(MaxRetries::try_new(invalid_retries).is_err());
            assert!(RetryDelayMs::try_new(invalid_delay).is_err());
            assert!(MessageTimeoutMs::try_new(invalid_timeout).is_err());
            assert!(RetryBackoffFactor::try_new(invalid_backoff).is_err());
            assert!(TraceSamplingRatio::try_new(invalid_sampling).is_err());
        }
    }

    proptest! {
        /// Property: Default values should be valid and reasonable
        #[test]
        fn test_default_values_are_valid(_unit in prop::sample::Just(())) {
            // All default values should be valid
            let default_capacity = ChannelCapacity::default();
            assert!(default_capacity.as_usize() > 0);
            assert!(default_capacity.as_usize() <= 1_000_000);

            let default_retries = MaxRetries::default();
            assert!(default_retries.as_u8() >= 1);
            assert!(default_retries.as_u8() <= 10);

            let default_delay = RetryDelayMs::default();
            assert!(default_delay.as_u64() >= 100);
            assert!(default_delay.as_u64() <= 300_000);

            let default_timeout = MessageTimeoutMs::default();
            assert!(default_timeout.as_u64() >= 1000);
            assert!(default_timeout.as_u64() <= 300_000);

            let default_queue_size = AgentQueueSize::default();
            assert!(default_queue_size.as_usize() > 0);
            assert!(default_queue_size.as_usize() <= 100_000);

            let default_threads = WorkerThreadCount::default();
            assert!(default_threads.as_usize() >= 1);
            assert!(default_threads.as_usize() <= 32);

            let default_batch = MessageBatchSize::default();
            assert!(default_batch.as_usize() >= 1);
            assert!(default_batch.as_usize() <= 10_000);

            let default_backoff = RetryBackoffFactor::default();
            assert!(default_backoff.as_f64() >= 1.1);
            assert!(default_backoff.as_f64() <= 5.0);

            let default_sampling = TraceSamplingRatio::default();
            assert!(default_sampling.as_f64() >= 0.0);
            assert!(default_sampling.as_f64() <= 1.0);

            // Default message priority should be Normal
            let default_priority = MessagePriority::default();
            assert_eq!(default_priority, MessagePriority::Normal);
        }
    }

    proptest! {
        /// Property: Serialization/deserialization should be consistent
        #[test]
        fn test_serialization_round_trip(message in arb_fipa_message()) {
            // Serialize to JSON
            let serialized = serde_json::to_string(&message);
            assert!(serialized.is_ok());

            // Deserialize from JSON
            let deserialized: Result<FipaMessage, _> = serde_json::from_str(&serialized.unwrap());
            assert!(deserialized.is_ok());

            let restored_message = deserialized.unwrap();

            // Key fields should be preserved
            assert_eq!(message.message_id, restored_message.message_id);
            assert_eq!(message.sender, restored_message.sender);
            assert_eq!(message.receiver, restored_message.receiver);
            assert_eq!(message.performative, restored_message.performative);
            assert_eq!(message.content.as_bytes(), restored_message.content.as_bytes());
            assert_eq!(message.conversation_id, restored_message.conversation_id);
            assert_eq!(message.delivery_options.priority, restored_message.delivery_options.priority);
        }
    }
}
