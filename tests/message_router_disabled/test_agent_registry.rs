//! Unit tests for AgentRegistry
//!
//! Tests O(1) agent lookup, registration, capability indexing, health tracking,
//! and remote route management.

use caxton::message_router::*;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

/// Test helper to create agent registry
async fn create_test_agent_registry() -> Box<dyn AgentRegistry> {
    let config = RouterConfig::development();
    AgentRegistryImpl::new(config).await.unwrap()
}

/// Test helper to create a test local agent
fn create_test_local_agent(name: &str, capabilities: Vec<CapabilityName>) -> LocalAgent {
    LocalAgent::new(
        AgentId::generate(),
        AgentName::try_new(name.to_string()).unwrap(),
        AgentState::Running,
        capabilities,
        MessageTimestamp::now(),
        AgentQueueSize::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test O(1) agent lookup performance
    #[tokio::test]
    async fn test_lookup_performance() {
        let registry = create_test_agent_registry().await;

        // Register a test agent
        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];
        let agent = create_test_local_agent("test-agent", capabilities.clone());
        let agent_id = agent.id;

        registry.register_local_agent(agent, capabilities).await.unwrap();

        // Measure lookup time
        let start = Instant::now();
        let result = registry.lookup(&agent_id).await;
        let duration = start.elapsed();

        assert!(result.is_ok());

        // Should complete in < 100μs for O(1) performance requirement
        assert!(duration < Duration::from_micros(100),
                "Lookup took {:?}, expected < 100μs", duration);

        if let Ok(AgentLocation::Local(local_agent)) = result {
            assert_eq!(local_agent.id, agent_id);
        } else {
            panic!("Expected local agent location");
        }
    }

    /// Test agent lookup for non-existent agent
    #[tokio::test]
    async fn test_lookup_nonexistent_agent() {
        let registry = create_test_agent_registry().await;

        let non_existent_id = AgentId::generate();
        let result = registry.lookup(&non_existent_id).await;

        assert!(result.is_err());
        if let Err(RegistryError::AgentNotFound { agent_id }) = result {
            assert_eq!(agent_id, non_existent_id);
        } else {
            panic!("Expected AgentNotFound error");
        }
    }

    /// Test local agent registration
    #[tokio::test]
    async fn test_register_local_agent() {
        let registry = create_test_agent_registry().await;

        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
            CapabilityName::try_new("storage".to_string()).unwrap(),
        ];

        let agent = create_test_local_agent("test-agent", capabilities.clone());
        let agent_id = agent.id;

        let result = registry.register_local_agent(agent.clone(), capabilities).await;
        assert!(result.is_ok());

        // Verify agent can be looked up
        let lookup_result = registry.lookup(&agent_id).await;
        assert!(lookup_result.is_ok());

        if let Ok(AgentLocation::Local(local_agent)) = lookup_result {
            assert_eq!(local_agent.id, agent_id);
            assert_eq!(local_agent.name, agent.name);
            assert_eq!(local_agent.state, agent.state);
        } else {
            panic!("Expected local agent location after registration");
        }
    }

    /// Test duplicate agent registration
    #[tokio::test]
    async fn test_duplicate_agent_registration() {
        let registry = create_test_agent_registry().await;

        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];
        let agent = create_test_local_agent("test-agent", capabilities.clone());

        // First registration should succeed
        let result1 = registry.register_local_agent(agent.clone(), capabilities.clone()).await;
        assert!(result1.is_ok());

        // Second registration of same agent should fail
        let result2 = registry.register_local_agent(agent, capabilities).await;
        assert!(result2.is_err());

        if let Err(RegistryError::AgentAlreadyRegistered { agent_id }) = result2 {
            assert_eq!(agent_id, agent.id);
        } else {
            panic!("Expected AgentAlreadyRegistered error");
        }
    }

    /// Test local agent deregistration
    #[tokio::test]
    async fn test_deregister_local_agent() {
        let registry = create_test_agent_registry().await;

        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];
        let agent = create_test_local_agent("test-agent", capabilities.clone());
        let agent_id = agent.id;

        // Register then deregister
        registry.register_local_agent(agent, capabilities).await.unwrap();

        let deregister_result = registry.deregister_local_agent(agent_id).await;
        assert!(deregister_result.is_ok());

        // Agent should no longer be found
        let lookup_result = registry.lookup(&agent_id).await;
        assert!(lookup_result.is_err());
        assert!(matches!(lookup_result.unwrap_err(), RegistryError::AgentNotFound { .. }));
    }

    /// Test deregistering non-existent agent
    #[tokio::test]
    async fn test_deregister_nonexistent_agent() {
        let registry = create_test_agent_registry().await;

        let non_existent_id = AgentId::generate();
        let result = registry.deregister_local_agent(non_existent_id).await;

        assert!(result.is_err());
        if let Err(RegistryError::AgentNotFound { agent_id }) = result {
            assert_eq!(agent_id, non_existent_id);
        } else {
            panic!("Expected AgentNotFound error");
        }
    }

    /// Test remote route management
    #[tokio::test]
    async fn test_update_remote_route() {
        let registry = create_test_agent_registry().await;

        let agent_id = AgentId::generate();
        let node_id = NodeId::generate();
        let hops = RouteHops::try_new(3).unwrap();

        // Update route for remote agent
        let result = registry.update_remote_route(agent_id, node_id, hops).await;
        assert!(result.is_ok());

        // Lookup should now return remote location
        let lookup_result = registry.lookup(&agent_id).await;
        assert!(lookup_result.is_ok());

        if let Ok(AgentLocation::Remote(remote_node_id)) = lookup_result {
            assert_eq!(remote_node_id, node_id);
        } else {
            panic!("Expected remote agent location after route update");
        }
    }

    /// Test capability-based agent discovery
    #[tokio::test]
    async fn test_find_agents_by_capability() {
        let registry = create_test_agent_registry().await;

        let compute_capability = CapabilityName::try_new("compute".to_string()).unwrap();
        let storage_capability = CapabilityName::try_new("storage".to_string()).unwrap();
        let network_capability = CapabilityName::try_new("network".to_string()).unwrap();

        // Register agents with different capabilities
        let agent1 = create_test_local_agent("compute-agent", vec![compute_capability.clone()]);
        let agent2 = create_test_local_agent("storage-agent", vec![storage_capability.clone()]);
        let agent3 = create_test_local_agent("multi-agent", vec![
            compute_capability.clone(),
            storage_capability.clone(),
        ]);

        let agent1_id = agent1.id;
        let agent2_id = agent2.id;
        let agent3_id = agent3.id;

        registry.register_local_agent(agent1, vec![compute_capability.clone()]).await.unwrap();
        registry.register_local_agent(agent2, vec![storage_capability.clone()]).await.unwrap();
        registry.register_local_agent(agent3, vec![
            compute_capability.clone(),
            storage_capability.clone(),
        ]).await.unwrap();

        // Find agents by compute capability
        let compute_agents = registry.find_agents_by_capability(&compute_capability).await;
        assert!(compute_agents.is_ok());
        let compute_agents = compute_agents.unwrap();
        assert_eq!(compute_agents.len(), 2);
        assert!(compute_agents.contains(&agent1_id));
        assert!(compute_agents.contains(&agent3_id));

        // Find agents by storage capability
        let storage_agents = registry.find_agents_by_capability(&storage_capability).await;
        assert!(storage_agents.is_ok());
        let storage_agents = storage_agents.unwrap();
        assert_eq!(storage_agents.len(), 2);
        assert!(storage_agents.contains(&agent2_id));
        assert!(storage_agents.contains(&agent3_id));

        // Find agents by non-existent capability
        let network_agents = registry.find_agents_by_capability(&network_capability).await;
        assert!(network_agents.is_ok());
        let network_agents = network_agents.unwrap();
        assert_eq!(network_agents.len(), 0);
    }

    /// Test listing local agents
    #[tokio::test]
    async fn test_list_local_agents() {
        let registry = create_test_agent_registry().await;

        // Initially should be empty
        let initial_agents = registry.list_local_agents().await;
        assert!(initial_agents.is_ok());
        assert_eq!(initial_agents.unwrap().len(), 0);

        // Register some agents
        let agent1 = create_test_local_agent("agent1", vec![]);
        let agent2 = create_test_local_agent("agent2", vec![]);

        registry.register_local_agent(agent1.clone(), vec![]).await.unwrap();
        registry.register_local_agent(agent2.clone(), vec![]).await.unwrap();

        // Should now have 2 agents
        let agents = registry.list_local_agents().await;
        assert!(agents.is_ok());
        let agents = agents.unwrap();
        assert_eq!(agents.len(), 2);

        let agent_ids: std::collections::HashSet<AgentId> = agents.iter().map(|a| a.id).collect();
        assert!(agent_ids.contains(&agent1.id));
        assert!(agent_ids.contains(&agent2.id));
    }

    /// Test agent health monitoring
    #[tokio::test]
    async fn test_update_agent_health() {
        let registry = create_test_agent_registry().await;

        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];
        let agent = create_test_local_agent("test-agent", capabilities.clone());
        let agent_id = agent.id;

        registry.register_local_agent(agent, capabilities).await.unwrap();

        // Update health status
        let heartbeat_time = MessageTimestamp::now();
        let result = registry.update_agent_health(agent_id, true, heartbeat_time).await;
        assert!(result.is_ok());

        // Update with unhealthy status
        let unhealthy_result = registry.update_agent_health(agent_id, false, heartbeat_time).await;
        assert!(unhealthy_result.is_ok());

        // Update health for non-existent agent
        let missing_result = registry.update_agent_health(AgentId::generate(), true, heartbeat_time).await;
        assert!(missing_result.is_err());
    }

    /// Test concurrent agent operations
    #[tokio::test]
    async fn test_concurrent_agent_operations() {
        let registry = create_test_agent_registry().await;

        // Register multiple agents concurrently
        let mut registration_handles = vec![];
        let mut agent_ids = vec![];

        for i in 0..20 {
            let registry_clone = registry.clone(); // Registry should implement Clone or use Arc
            let agent = create_test_local_agent(&format!("agent-{}", i), vec![]);
            let agent_id = agent.id;
            agent_ids.push(agent_id);

            let handle = tokio::spawn(async move {
                registry_clone.register_local_agent(agent, vec![]).await
            });
            registration_handles.push(handle);
        }

        // Wait for all registrations
        let registration_results = futures::future::join_all(registration_handles).await;
        for result in registration_results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        // Perform concurrent lookups
        let mut lookup_handles = vec![];
        for agent_id in &agent_ids {
            let registry_clone = registry.clone();
            let agent_id = *agent_id;

            let handle = tokio::spawn(async move {
                registry_clone.lookup(&agent_id).await
            });
            lookup_handles.push(handle);
        }

        // All lookups should succeed
        let lookup_results = futures::future::join_all(lookup_handles).await;
        for result in lookup_results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        // Verify final count
        let final_agents = registry.list_local_agents().await.unwrap();
        assert_eq!(final_agents.len(), 20);
    }

    /// Test lookup performance under load
    #[tokio::test]
    async fn test_lookup_performance_under_load() {
        let registry = create_test_agent_registry().await;

        // Register many agents
        let mut agent_ids = vec![];
        for i in 0..1000 {
            let agent = create_test_local_agent(&format!("agent-{}", i), vec![]);
            let agent_id = agent.id;
            agent_ids.push(agent_id);

            registry.register_local_agent(agent, vec![]).await.unwrap();
        }

        // Measure lookup performance with many agents
        let lookup_count = 100;
        let start = Instant::now();

        for i in 0..lookup_count {
            let agent_id = &agent_ids[i % agent_ids.len()];
            let result = registry.lookup(agent_id).await;
            assert!(result.is_ok());
        }

        let total_duration = start.elapsed();
        let average_lookup_time = total_duration / lookup_count;

        // Should maintain O(1) performance even with many registered agents
        assert!(average_lookup_time < Duration::from_micros(100),
                "Average lookup time {:?} exceeds 100μs with {} agents",
                average_lookup_time, agent_ids.len());
    }

    /// Test capability index performance
    #[tokio::test]
    async fn test_capability_index_performance() {
        let registry = create_test_agent_registry().await;

        let compute_capability = CapabilityName::try_new("compute".to_string()).unwrap();

        // Register many agents with the same capability
        for i in 0..1000 {
            let agent = create_test_local_agent(&format!("compute-agent-{}", i), vec![compute_capability.clone()]);
            registry.register_local_agent(agent, vec![compute_capability.clone()]).await.unwrap();
        }

        // Measure capability lookup performance
        let start = Instant::now();
        let agents = registry.find_agents_by_capability(&compute_capability).await;
        let duration = start.elapsed();

        assert!(agents.is_ok());
        let agents = agents.unwrap();
        assert_eq!(agents.len(), 1000);

        // Capability index should provide O(1) lookup
        assert!(duration < Duration::from_millis(1),
                "Capability lookup took {:?}, expected < 1ms", duration);
    }

    /// Test route expiration and cleanup
    #[tokio::test]
    async fn test_route_expiration() {
        let registry = create_test_agent_registry().await;

        let agent_id = AgentId::generate();
        let node_id = NodeId::generate();
        let hops = RouteHops::try_new(2).unwrap();

        // Update remote route
        registry.update_remote_route(agent_id, node_id, hops).await.unwrap();

        // Should find remote agent immediately
        let lookup_result = registry.lookup(&agent_id).await;
        assert!(lookup_result.is_ok());
        assert!(matches!(lookup_result.unwrap(), AgentLocation::Remote(_)));

        // Routes should have expiration mechanisms in production implementation
        // This test validates the interface exists
    }

    /// Test agent state transitions in registry
    #[tokio::test]
    async fn test_agent_state_in_registry() {
        let registry = create_test_agent_registry().await;

        let capabilities = vec![
            CapabilityName::try_new("compute".to_string()).unwrap(),
        ];
        let mut agent = create_test_local_agent("test-agent", capabilities.clone());
        let agent_id = agent.id;

        // Register agent as running
        registry.register_local_agent(agent.clone(), capabilities).await.unwrap();

        // Lookup should return running agent
        let lookup_result = registry.lookup(&agent_id).await;
        assert!(lookup_result.is_ok());

        if let Ok(AgentLocation::Local(local_agent)) = lookup_result {
            assert_eq!(local_agent.state, AgentState::Running);
            assert!(local_agent.is_available());
        } else {
            panic!("Expected local agent");
        }
    }

    /// Test registry with mixed local and remote agents
    #[tokio::test]
    async fn test_mixed_local_remote_agents() {
        let registry = create_test_agent_registry().await;

        // Register local agent
        let local_agent = create_test_local_agent("local-agent", vec![]);
        let local_agent_id = local_agent.id;
        registry.register_local_agent(local_agent, vec![]).await.unwrap();

        // Register remote agent route
        let remote_agent_id = AgentId::generate();
        let remote_node_id = NodeId::generate();
        let hops = RouteHops::try_new(1).unwrap();
        registry.update_remote_route(remote_agent_id, remote_node_id, hops).await.unwrap();

        // Lookup local agent
        let local_lookup = registry.lookup(&local_agent_id).await;
        assert!(local_lookup.is_ok());
        assert!(matches!(local_lookup.unwrap(), AgentLocation::Local(_)));

        // Lookup remote agent
        let remote_lookup = registry.lookup(&remote_agent_id).await;
        assert!(remote_lookup.is_ok());
        assert!(matches!(remote_lookup.unwrap(), AgentLocation::Remote(_)));

        // List local agents should only return local ones
        let local_agents = registry.list_local_agents().await.unwrap();
        assert_eq!(local_agents.len(), 1);
        assert_eq!(local_agents[0].id, local_agent_id);
    }
}
