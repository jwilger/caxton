//! `ConversationManager` implementation with O(1) conversation tracking
//!
//! Provides high-performance conversation management using `DashMap` for concurrent access
//! and atomic counters for statistics.

use crate::message_router::{
    config::RouterConfig,
    domain_types::{
        AgentId, Conversation, ConversationCreatedAt, ConversationId, FipaMessage, MessageCount,
        MessageTimestamp, ProtocolName,
    },
    traits::{ConversationError, ConversationManager, ConversationStats},
};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Real conversation manager implementation with O(1) lookup performance
pub struct ConversationManagerImpl {
    /// Active conversations indexed by conversation ID
    conversations: DashMap<ConversationId, Conversation>,
    /// Conversation statistics
    total_created: AtomicU64,
    /// Configuration for timeouts and limits
    config: RouterConfig,
}

impl ConversationManagerImpl {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            conversations: DashMap::new(),
            total_created: AtomicU64::new(0),
            config,
        }
    }
}

#[async_trait]
impl ConversationManager for ConversationManagerImpl {
    async fn get_or_create_conversation(
        &self,
        conversation_id: ConversationId,
        participants: std::collections::HashSet<AgentId>,
        protocol: Option<ProtocolName>,
    ) -> Result<Conversation, ConversationError> {
        // Check if conversation already exists
        if let Some(conversation) = self.conversations.get(&conversation_id) {
            return Ok(conversation.clone());
        }

        // Validate participant count
        if participants.len() > self.config.max_conversation_participants.into_inner() as usize {
            return Err(ConversationError::TooManyParticipants {
                count: participants.len(),
                max: self.config.max_conversation_participants.into_inner() as usize,
            });
        }

        // Create new conversation
        let conversation = Conversation::new(
            conversation_id,
            participants,
            protocol,
            ConversationCreatedAt::now(),
        );

        // Store the conversation
        self.conversations
            .insert(conversation_id, conversation.clone());
        self.total_created.fetch_add(1, Ordering::Relaxed);

        Ok(conversation)
    }

    async fn update_conversation(
        &self,
        conversation_id: ConversationId,
        message: &FipaMessage,
    ) -> Result<(), ConversationError> {
        // Find and update the conversation
        if let Some(mut conversation) = self.conversations.get_mut(&conversation_id) {
            conversation.add_message(message);
            Ok(())
        } else {
            // Conversation not found - create it if the message has participants
            let mut participants = HashSet::new();
            participants.insert(message.sender);
            participants.insert(message.receiver);

            let mut conversation = self
                .get_or_create_conversation(conversation_id, participants, message.protocol.clone())
                .await?;

            conversation.add_message(message);
            self.conversations.insert(conversation_id, conversation);

            Ok(())
        }
    }

    async fn get_agent_conversations(
        &self,
        agent_id: AgentId,
    ) -> Result<Vec<Conversation>, ConversationError> {
        let conversations: Vec<Conversation> = self
            .conversations
            .iter()
            .filter(|entry| entry.participants.contains(&agent_id))
            .map(|entry| entry.clone())
            .collect();

        Ok(conversations)
    }

    async fn cleanup_expired_conversations(&self) -> Result<usize, ConversationError> {
        let timeout = self.config.conversation_timeout_ms.as_duration();
        let _now = std::time::SystemTime::now();
        let mut cleaned_count = 0;

        // Find expired conversations
        let mut expired_ids = Vec::new();

        for entry in &self.conversations {
            let conversation = entry.value();
            if is_conversation_expired(conversation.last_activity, timeout) {
                expired_ids.push(*entry.key());
            }
        }

        // Remove expired conversations
        for conversation_id in expired_ids {
            if self.conversations.remove(&conversation_id).is_some() {
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    async fn get_conversation_stats(&self) -> Result<ConversationStats, ConversationError> {
        let total_active = self.conversations.len();
        let total_created =
            MessageCount::new(self.total_created.load(Ordering::Relaxed).min(1_000_000) as usize);

        // Calculate average duration and message count
        let mut total_duration_ms = 0u64;
        let mut total_message_count = 0u64;
        let mut participants_distribution = HashMap::new();

        for entry in &self.conversations {
            let conversation = entry.value();

            // Calculate duration
            let duration_ms = calculate_conversation_duration_ms(
                conversation.created_at.as_system_time(),
                conversation.last_activity.as_system_time(),
            );
            total_duration_ms = total_duration_ms.saturating_add(duration_ms);

            // Add message count
            total_message_count += conversation.message_count.into_inner() as u64;

            // Track participant distribution
            let participant_count = conversation.participants.len();
            *participants_distribution
                .entry(participant_count)
                .or_insert(0) += 1;
        }

        let average_duration_ms = calculate_average_duration(total_duration_ms, total_active);

        let average_message_count =
            calculate_average_message_count(total_message_count, total_active);

        Ok(ConversationStats {
            total_active,
            total_created,
            average_duration_ms,
            average_message_count,
            participants_distribution,
        })
    }
}

// Helper functions for conversation management

fn calculate_conversation_duration_ms(
    created_at: std::time::SystemTime,
    last_activity: std::time::SystemTime,
) -> u64 {
    if let Ok(duration) = last_activity.duration_since(created_at) {
        u64::try_from(duration.as_millis().min(u128::from(u64::MAX))).unwrap_or(0)
    } else {
        0
    }
}

fn calculate_average_duration(total_duration_ms: u64, conversation_count: usize) -> u64 {
    if conversation_count > 0 {
        total_duration_ms / conversation_count as u64
    } else {
        0
    }
}

fn calculate_average_message_count(total_message_count: u64, conversation_count: usize) -> f64 {
    if conversation_count > 0 {
        let safe_msg_count = safe_u64_to_u32(total_message_count);
        let safe_active = u32::try_from(conversation_count.min(u32::MAX as usize)).unwrap_or(1);
        f64::from(safe_msg_count) / f64::from(safe_active)
    } else {
        0.0
    }
}

fn is_conversation_expired(last_activity: MessageTimestamp, timeout: Duration) -> bool {
    if let Ok(elapsed) = last_activity.as_system_time().elapsed() {
        elapsed > timeout
    } else {
        false
    }
}

fn safe_u64_to_u32(value: u64) -> u32 {
    u32::try_from(value.min(u64::from(u32::MAX))).unwrap_or(u32::MAX)
}
