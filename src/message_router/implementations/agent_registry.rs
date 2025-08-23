//! Agent registry implementation for message router
//!
//! Provides O(1) agent lookup and registration with capability indexing.

use crate::{
    domain_types::AgentId,
    message_router::{
        config::RouterConfig,
        domain_types::{
            AgentLocation, CapabilityName, LocalAgent, MessageTimestamp, NodeId, RouteHops,
        },
        traits::{AgentRegistry, RegistryError},
    },
};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashSet;

/// Node information for remote agent routing
pub struct NodeInfo {
    pub id: NodeId,
    pub name: String,
    pub address: String,
    pub is_healthy: bool,
    pub last_heartbeat: MessageTimestamp,
    pub agent_count: usize,
}

impl NodeInfo {
    pub fn new(id: NodeId, name: String, address: String) -> Self {
        Self {
            id,
            name,
            address,
            is_healthy: true,
            last_heartbeat: MessageTimestamp::now(),
            agent_count: 0,
        }
    }
}

/// Real agent registry implementation with O(1) lookup performance
pub struct AgentRegistryImpl {
    /// O(1) lookup for local agents
    agents: DashMap<AgentId, LocalAgent>,

    /// O(1) lookup for routing information
    routes: DashMap<AgentId, AgentLocation>,

    /// O(1) lookup for capability-based discovery
    capabilities: DashMap<CapabilityName, HashSet<AgentId>>,

    /// Node registry for remote agents
    node_registry: DashMap<NodeId, NodeInfo>,
}

impl AgentRegistryImpl {
    pub fn new(_config: RouterConfig) -> Self {
        Self {
            agents: DashMap::new(),
            routes: DashMap::new(),
            capabilities: DashMap::new(),
            node_registry: DashMap::new(),
        }
    }
}

#[async_trait]
impl AgentRegistry for AgentRegistryImpl {
    /// O(1) agent lookup with actual implementation
    async fn lookup(&self, agent_id: &AgentId) -> Result<AgentLocation, RegistryError> {
        // First check routes cache for O(1) lookup
        if let Some(location) = self.routes.get(agent_id) {
            return Ok(location.clone());
        }

        // If not in cache, check if it's a local agent
        if let Some(agent) = self.agents.get(agent_id) {
            let location = AgentLocation::Local(agent.clone());
            // Cache the location for future O(1) lookups
            self.routes.insert(*agent_id, location.clone());
            return Ok(location);
        }

        // Agent not found
        Err(RegistryError::AgentNotFound {
            agent_id: *agent_id,
        })
    }

    /// Registers a local agent with capabilities indexing
    async fn register_local_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>,
    ) -> Result<(), RegistryError> {
        let agent_id = agent.id;

        // Check if agent is already registered
        if self.agents.contains_key(&agent_id) {
            return Err(RegistryError::AgentAlreadyRegistered { agent_id });
        }

        // Store the agent
        self.agents.insert(agent_id, agent.clone());

        // Cache the location for O(1 lookups
        self.routes.insert(agent_id, AgentLocation::Local(agent));

        // Index capabilities for discovery
        for capability in capabilities {
            self.capabilities
                .entry(capability)
                .and_modify(|agents| {
                    agents.insert(agent_id);
                })
                .or_insert_with(|| {
                    let mut agents = HashSet::new();
                    agents.insert(agent_id);
                    agents
                });
        }

        Ok(())
    }

    /// Deregisters a local agent and cleans up all indexes
    async fn deregister_local_agent(&self, agent_id: AgentId) -> Result<(), RegistryError> {
        // Remove from agents map
        let agent = self
            .agents
            .remove(&agent_id)
            .ok_or(RegistryError::AgentNotFound { agent_id })?;

        // Remove from routes cache
        self.routes.remove(&agent_id);

        // Clean up capability indexes
        for capability in &agent.1.capabilities {
            if let Some(mut agents) = self.capabilities.get_mut(capability) {
                agents.remove(&agent_id);
                // Remove empty capability entries
                if agents.is_empty() {
                    drop(agents);
                    self.capabilities.remove(capability);
                }
            }
        }

        Ok(())
    }

    /// Updates routing table with remote agent information
    async fn update_remote_route(
        &self,
        agent_id: AgentId,
        node_id: NodeId,
        _hops: RouteHops,
    ) -> Result<(), RegistryError> {
        // Ensure node exists in registry
        if !self.node_registry.contains_key(&node_id) {
            let node_info =
                NodeInfo::new(node_id, format!("node-{node_id}"), "unknown".to_string());
            self.node_registry.insert(node_id, node_info);
        }

        // Update route cache
        self.routes.insert(agent_id, AgentLocation::Remote(node_id));

        // Update node agent count
        if let Some(mut node_info) = self.node_registry.get_mut(&node_id) {
            node_info.agent_count += 1;
        }

        Ok(())
    }

    /// O(1) capability-based agent discovery
    async fn find_agents_by_capability(
        &self,
        capability: &CapabilityName,
    ) -> Result<Vec<AgentId>, RegistryError> {
        if let Some(agents) = self.capabilities.get(capability) {
            Ok(agents.iter().copied().collect())
        } else {
            Ok(vec![])
        }
    }

    /// Lists all local agents
    async fn list_local_agents(&self) -> Result<Vec<LocalAgent>, RegistryError> {
        Ok(self
            .agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Updates agent health status
    async fn update_agent_health(
        &self,
        agent_id: AgentId,
        _is_healthy: bool,
        last_heartbeat: MessageTimestamp,
    ) -> Result<(), RegistryError> {
        if let Some(mut agent) = self.agents.get_mut(&agent_id) {
            agent.last_heartbeat = last_heartbeat;
            Ok(())
        } else {
            Err(RegistryError::AgentNotFound { agent_id })
        }
    }
}
