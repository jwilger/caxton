//! FIPA messaging protocol implementation
//!
//! Standards-compliant agent communication following FIPA specifications

use crate::*;

/// FIPA conversation identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationId(Uuid);

impl ConversationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for ConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// FIPA performatives (message types)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FipaPerformative {
    // Query performatives
    QueryIf,
    QueryRef,

    // Response performatives
    Inform,
    NotUnderstood,
    Refuse,
    Failure,

    // Action performatives
    Request,
    Agree,
    Cancel,

    // Contract net performatives
    Cfp, // Call for proposals
    Propose,
    AcceptProposal,
    RejectProposal,
}

/// FIPA message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FipaMessage {
    pub performative: FipaPerformative,
    pub sender: AgentId,
    pub receiver: AgentId,
    pub content: serde_json::Value,
    pub conversation_id: Option<ConversationId>,
    pub reply_to: Option<ConversationId>,
    pub in_reply_to: Option<String>,
    pub reply_with: Option<String>,
    pub reply_by: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub ontology: Option<String>,
    pub protocol: Option<String>,
}

impl Default for FipaMessage {
    fn default() -> Self {
        Self {
            performative: FipaPerformative::Inform,
            sender: AgentId::system(),
            receiver: AgentId::system(),
            content: serde_json::Value::Null,
            conversation_id: None,
            reply_to: None,
            in_reply_to: None,
            reply_with: None,
            reply_by: None,
            language: None,
            ontology: None,
            protocol: None,
        }
    }
}

/// Validate FIPA message structure
pub fn validate_fipa_message(message: &FipaMessage) -> Result<(), CaxtonError> {
    // Check required fields
    if message.sender.to_string().is_empty() {
        return Err(CaxtonError::InvalidMessage("Empty sender".to_string()));
    }

    if message.receiver.to_string().is_empty() {
        return Err(CaxtonError::InvalidMessage("Empty receiver".to_string()));
    }

    if message.content.is_null() {
        return Err(CaxtonError::InvalidMessage("Null content".to_string()));
    }

    Ok(())
}
