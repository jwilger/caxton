//! Agent management types and lifecycle operations
//! 
//! This module provides type-safe agent management with phantom types
//! to make illegal states unrepresentable at compile time.

use crate::*;
use std::marker::PhantomData;

/// Unique identifier for agents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_string(s: &str) -> Self {
        Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, s.as_bytes()))
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
    
    pub fn system() -> Self {
        Self(Uuid::nil())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Agent type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Coordinator,
    Worker,
    Monitor,
    Proxy,
    Custom(String),
}

/// Agent state machine
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Initializing,
    Ready,
    Processing,
    Suspended,
    Terminating,
    Terminated,
}

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub max_memory: Option<u64>,
    pub timeout: Option<Duration>,
}

/// Agent metadata with type-safe state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub name: String,
    pub agent_type: AgentType,
    pub state: AgentState,
    pub capabilities: Vec<String>,
    pub properties: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AgentMetadata {
    pub fn new(name: String, agent_type: AgentType) -> Self {
        let now = Utc::now();
        Self {
            name,
            agent_type,
            state: AgentState::Initializing,
            capabilities: Vec::new(),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
        self.updated_at = Utc::now();
    }
    
    pub fn add_capability(&mut self, capability: &str) {
        self.capabilities.push(capability.to_string());
        self.updated_at = Utc::now();
    }
    
    pub fn set_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
    pub metrics: HashMap<String, f64>,
}

/// Agent with phantom type state tracking
pub struct Agent<State> {
    pub id: AgentId,
    pub metadata: AgentMetadata,
    _state: PhantomData<State>,
}

/// Phantom types for agent states
pub struct Unloaded;
pub struct Loaded;  
pub struct Running;

impl Agent<Unloaded> {
    pub fn new(config: AgentConfig) -> Self {
        let id = AgentId::new();
        let metadata = AgentMetadata::new(config.name, config.agent_type);
        
        Self {
            id,
            metadata,
            _state: PhantomData,
        }
    }
}

impl Agent<Loaded> {
    pub fn start(mut self) -> Result<Agent<Running>, CaxtonError> {
        self.metadata.set_state(AgentState::Ready);
        
        Ok(Agent {
            id: self.id,
            metadata: self.metadata,
            _state: PhantomData,
        })
    }
}

impl Agent<Running> {
    pub fn stop(mut self) -> Agent<Loaded> {
        self.metadata.set_state(AgentState::Terminated);
        
        Agent {
            id: self.id,
            metadata: self.metadata,
            _state: PhantomData,  
        }
    }
}