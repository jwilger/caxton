//! Agent management types and lifecycle operations
//!
//! This module provides type-safe agent management with phantom types
//! to make illegal states unrepresentable at compile time.

use crate::*;
use std::marker::PhantomData;
use std::sync::Arc;
use dashmap::DashMap;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentState {
    Initializing,
    Ready,
    Processing,
    Suspended,
    Terminating,
    Terminated,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let mut metadata = AgentMetadata::new(config.name, config.agent_type);

        // Add capabilities from config
        for capability in config.capabilities {
            metadata.add_capability(&capability);
        }

        // Set resource limits if provided
        if let Some(max_memory) = config.max_memory {
            metadata.set_property("max_memory_bytes", &max_memory.to_string());
        }

        if let Some(timeout) = config.timeout {
            metadata.set_property("timeout_ms", &timeout.as_millis().to_string());
        }

        Self {
            id,
            metadata,
            _state: PhantomData,
        }
    }

    /// Load agent with WASM module (transitions to Loaded state)
    pub fn load_wasm_module(mut self, _module_data: &[u8]) -> Result<Agent<Loaded>, CaxtonError> {
        // In real implementation, this would validate and load the WASM module
        self.metadata.set_state(AgentState::Ready);
        self.metadata.set_property("wasm_loaded", "true");

        Ok(Agent {
            id: self.id,
            metadata: self.metadata,
            _state: PhantomData,
        })
    }
}

impl Agent<Loaded> {
    pub fn start(mut self) -> Result<Agent<Running>, CaxtonError> {
        // Validate agent is ready to start
        if self.metadata.state != AgentState::Ready {
            return Err(CaxtonError::Agent(format!(
                "Cannot start agent in state: {:?}",
                self.metadata.state
            )));
        }

        self.metadata.set_state(AgentState::Processing);
        self.metadata.set_property("started_at", &Utc::now().to_rfc3339());

        Ok(Agent {
            id: self.id,
            metadata: self.metadata,
            _state: PhantomData,
        })
    }

    /// Unload the agent (transitions back to Unloaded)
    pub fn unload(mut self) -> Agent<Unloaded> {
        self.metadata.set_state(AgentState::Initializing);
        self.metadata.properties.remove("wasm_loaded");

        Agent {
            id: self.id,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }
}

impl Agent<Running> {
    pub fn stop(mut self) -> Agent<Loaded> {
        self.metadata.set_state(AgentState::Ready); // Back to loaded state
        self.metadata.set_property("stopped_at", &Utc::now().to_rfc3339());

        Agent {
            id: self.id,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }

    pub fn suspend(mut self) -> Agent<Loaded> {
        self.metadata.set_state(AgentState::Suspended);
        self.metadata.set_property("suspended_at", &Utc::now().to_rfc3339());

        Agent {
            id: self.id,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }

    /// Terminate the agent permanently (transitions to final state)
    pub fn terminate(mut self) -> Result<(), CaxtonError> {
        self.metadata.set_state(AgentState::Terminated);
        self.metadata.set_property("terminated_at", &Utc::now().to_rfc3339());

        // Agent is consumed and cannot be used again
        Ok(())
    }

    /// Process a message (remains in Running state)
    pub fn process_message(&mut self, _message: &FipaMessage) -> Result<(), CaxtonError> {
        // In real implementation, this would invoke WASM to process the message
        self.metadata.set_property("last_message_at", &Utc::now().to_rfc3339());
        Ok(())
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> HashMap<String, String> {
        let mut metrics = HashMap::new();

        if let Some(started_at) = self.metadata.properties.get("started_at") {
            metrics.insert("started_at".to_string(), started_at.clone());
        }

        if let Some(last_message) = self.metadata.properties.get("last_message_at") {
            metrics.insert("last_message_at".to_string(), last_message.clone());
        }

        metrics.insert("capabilities_count".to_string(), self.metadata.capabilities.len().to_string());
        metrics.insert("properties_count".to_string(), self.metadata.properties.len().to_string());

        metrics
    }
}

/// Thread-safe agent registry for tracking all agents
#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: Arc<DashMap<AgentId, AgentMetadata>>,
    capabilities_index: Arc<DashMap<String, Vec<AgentId>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            capabilities_index: Arc::new(DashMap::new()),
        }
    }

    /// Register an agent in the registry
    pub fn register(&self, agent_id: AgentId, metadata: AgentMetadata) {
        // Add to main registry
        self.agents.insert(agent_id.clone(), metadata.clone());

        // Update capabilities index
        for capability in &metadata.capabilities {
            self.capabilities_index
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(agent_id.clone());
        }
    }

    /// Unregister an agent from the registry
    pub fn unregister(&self, agent_id: &AgentId) -> Option<AgentMetadata> {
        let metadata = self.agents.remove(agent_id)?;

        // Clean up capabilities index
        for capability in &metadata.1.capabilities {
            if let Some(mut agents) = self.capabilities_index.get_mut(capability) {
                agents.retain(|id| id != agent_id);
                if agents.is_empty() {
                    drop(agents);
                    self.capabilities_index.remove(capability);
                }
            }
        }

        Some(metadata.1)
    }

    /// Update agent metadata
    pub fn update_metadata(&self, agent_id: &AgentId, metadata: AgentMetadata) -> Result<(), CaxtonError> {
        if let Some(mut entry) = self.agents.get_mut(agent_id) {
            *entry = metadata;
            Ok(())
        } else {
            Err(CaxtonError::Agent(format!("Agent not found: {}", agent_id)))
        }
    }

    /// Get agent metadata
    pub fn get_metadata(&self, agent_id: &AgentId) -> Option<AgentMetadata> {
        self.agents.get(agent_id).map(|entry| entry.clone())
    }

    /// Find agents by capability
    pub fn find_by_capability(&self, capability: &str) -> Vec<AgentId> {
        self.capabilities_index
            .get(capability)
            .map(|agents| agents.clone())
            .unwrap_or_default()
    }

    /// Find agents by type
    pub fn find_by_type(&self, agent_type: &AgentType) -> Vec<(AgentId, AgentMetadata)> {
        self.agents
            .iter()
            .filter(|entry| &entry.value().agent_type == agent_type)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Find agents by state
    pub fn find_by_state(&self, state: &AgentState) -> Vec<(AgentId, AgentMetadata)> {
        self.agents
            .iter()
            .filter(|entry| &entry.value().state == state)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// List all agents
    pub fn list_all(&self) -> Vec<(AgentId, AgentMetadata)> {
        self.agents
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Get agent count by state
    pub fn count_by_state(&self) -> HashMap<AgentState, usize> {
        let mut counts = HashMap::new();

        for entry in self.agents.iter() {
            *counts.entry(entry.value().state.clone()).or_insert(0) += 1;
        }

        counts
    }

    /// Get total agent count
    pub fn total_count(&self) -> usize {
        self.agents.len()
    }

    /// Get capability distribution
    pub fn capability_distribution(&self) -> HashMap<String, usize> {
        self.capabilities_index
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().len()))
            .collect()
    }
}
