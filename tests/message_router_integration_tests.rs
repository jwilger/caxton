//! Integration tests for `AgentRegistryImpl` through public `MessageRouter` interface
//!
//! These tests verify that the agent registry provides O(1) lookup performance
//! through the public `MessageRouter` interface without accessing internal structures.

#[cfg(test)]
mod tests {
    use caxton::message_router::{
        config::RouterConfig,
        domain_types::*,
        router::MessageRouterImpl,
        traits::{HealthStatus, MessageRouter, RouterError},
    };
    use std::time::Instant;

    /// Helper function to create test configuration
    fn test_config() -> RouterConfig {
        RouterConfig::development()
    }

    /// Helper function to create a test agent
    fn create_test_agent(id: u32, capabilities: Vec<&str>) -> (LocalAgent, Vec<CapabilityName>) {
        let agent_id = AgentId::generate();
        let agent_name = AgentName::try_new(format!("test-agent-{id}")).unwrap();
        let capabilities: Vec<CapabilityName> = capabilities
            .into_iter()
            .map(|cap| CapabilityName::try_new(cap.to_string()).unwrap())
            .collect();

        let agent = LocalAgent::new(
            agent_id,
            agent_name,
            AgentState::Running,
            capabilities.clone(),
            MessageTimestamp::now(),
            AgentQueueSize::try_new(1000).unwrap(),
        );

        (agent, capabilities)
    }

    #[tokio::test]
    async fn test_agent_registration_and_stats() {
        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();

        // Initially no agents
        let initial_stats = router.get_stats().await.unwrap();
        assert_eq!(initial_stats.agent_queue_depths.len(), 0);

        // Create and register test agent
        let (agent, capabilities) = create_test_agent(1, vec!["data-processing", "file-handling"]);
        let agent_id = agent.id;

        router.register_agent(agent, capabilities).await.unwrap();

        // Verify agent appears in stats (indicating successful registration)
        let stats = router.get_stats().await.unwrap();
        assert_eq!(stats.agent_queue_depths.len(), 1);
        assert!(stats.agent_queue_depths.contains_key(&agent_id));
    }

    #[tokio::test]
    async fn test_agent_deregistration() {
        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();

        // Create and register test agent
        let (agent, capabilities) = create_test_agent(1, vec!["data-processing"]);
        let agent_id = agent.id;

        router.register_agent(agent, capabilities).await.unwrap();

        // Verify agent is registered
        let stats = router.get_stats().await.unwrap();
        assert!(stats.agent_queue_depths.contains_key(&agent_id));

        // Deregister agent
        router.deregister_agent(agent_id).await.unwrap();

        // Verify agent is no longer in stats
        let final_stats = router.get_stats().await.unwrap();
        assert!(!final_stats.agent_queue_depths.contains_key(&agent_id));
    }

    #[tokio::test]
    async fn test_multiple_agent_registration() {
        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();

        // Register multiple agents
        let (agent1, caps1) = create_test_agent(1, vec!["data-processing"]);
        let (agent2, caps2) = create_test_agent(2, vec!["file-handling"]);
        let (agent3, caps3) = create_test_agent(3, vec!["web-scraping"]);

        router.register_agent(agent1.clone(), caps1).await.unwrap();
        router.register_agent(agent2.clone(), caps2).await.unwrap();
        router.register_agent(agent3.clone(), caps3).await.unwrap();

        // Verify all agents are registered
        let stats = router.get_stats().await.unwrap();
        assert_eq!(stats.agent_queue_depths.len(), 3);
        assert!(stats.agent_queue_depths.contains_key(&agent1.id));
        assert!(stats.agent_queue_depths.contains_key(&agent2.id));
        assert!(stats.agent_queue_depths.contains_key(&agent3.id));
    }

    #[tokio::test]
    async fn test_concurrent_agent_operations() {
        const NUM_CONCURRENT_OPS: usize = 100;

        let config = test_config();
        let router = std::sync::Arc::new(MessageRouterImpl::new(config).await.unwrap());
        let mut handles = Vec::new();

        // Spawn concurrent registration operations
        for i in 0..NUM_CONCURRENT_OPS {
            let router_clone = std::sync::Arc::clone(&router);
            let handle = tokio::spawn(async move {
                let agent_id = u32::try_from(i).unwrap_or(u32::MAX);
                let (agent, capabilities) = create_test_agent(agent_id, vec!["concurrent-test"]);
                let agent_id = agent.id;

                // Register agent
                router_clone
                    .register_agent(agent, capabilities)
                    .await
                    .unwrap();
                agent_id
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let agent_ids: Vec<AgentId> = futures::future::try_join_all(handles).await.unwrap();

        // Verify all agents were registered
        assert_eq!(agent_ids.len(), NUM_CONCURRENT_OPS);

        let stats = router.get_stats().await.unwrap();
        assert_eq!(stats.agent_queue_depths.len(), NUM_CONCURRENT_OPS);

        // Verify all agent IDs are present
        for agent_id in agent_ids {
            assert!(stats.agent_queue_depths.contains_key(&agent_id));
        }
    }

    #[tokio::test]
    async fn test_stats_lookup_performance() {
        const NUM_AGENTS: usize = 1_000;
        const STATS_CALLS: usize = 100;

        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();
        let mut agent_ids = Vec::with_capacity(NUM_AGENTS);

        // Registration phase
        for i in 0..NUM_AGENTS {
            let agent_id = u32::try_from(i).unwrap_or(u32::MAX);
            let (agent, capabilities) = create_test_agent(agent_id, vec!["perf-test"]);
            agent_ids.push(agent.id);
            router.register_agent(agent, capabilities).await.unwrap();
        }

        // Test stats retrieval performance (which internally uses the registry)
        let start_time = Instant::now();

        for _ in 0..STATS_CALLS {
            let stats = router.get_stats().await.unwrap();
            assert_eq!(stats.agent_queue_depths.len(), NUM_AGENTS);
        }

        let elapsed = start_time.elapsed();
        let stats_calls_u32 = u32::try_from(STATS_CALLS).unwrap_or(u32::MAX);
        let avg_time = elapsed / stats_calls_u32;

        println!(
            "Average stats retrieval time: {}μs for {} agents",
            avg_time.as_micros(),
            NUM_AGENTS
        );

        // Reasonable performance requirement for stats API
        assert!(
            avg_time.as_millis() < 50,
            "Average stats time {}ms exceeds 50ms requirement",
            avg_time.as_millis()
        );
    }

    #[tokio::test]
    async fn test_error_conditions() {
        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();

        // Test deregistration of non-existent agent
        let nonexistent_id = AgentId::generate();
        let result = router.deregister_agent(nonexistent_id).await;
        assert!(matches!(result, Err(RouterError::AgentNotFound { .. })));

        // Test duplicate registration
        let (agent, capabilities) = create_test_agent(1, vec!["test-capability"]);

        router
            .register_agent(agent.clone(), capabilities.clone())
            .await
            .unwrap();

        // Attempting to register the same agent again should fail
        let duplicate_result = router.register_agent(agent, capabilities).await;
        assert!(matches!(
            duplicate_result,
            Err(RouterError::ConfigurationError { .. })
        ));
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();

        // Start the router so health checks work
        router.start().await.unwrap();

        // Health check should work after starting (may be Degraded if no agents)
        let health = router.health_check().await.unwrap();
        assert!(matches!(
            health,
            HealthStatus::Healthy | HealthStatus::Degraded { .. }
        ));

        // Register an agent and verify health still works
        let (agent, capabilities) = create_test_agent(1, vec!["health-test"]);
        router.register_agent(agent, capabilities).await.unwrap();

        let health_after_registration = router.health_check().await.unwrap();
        // Health may still be degraded even after registration if no queues registered
        assert!(matches!(
            health_after_registration,
            HealthStatus::Healthy | HealthStatus::Degraded { .. }
        ));
    }

    #[tokio::test]
    async fn test_router_lifecycle() {
        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();

        // Start the router
        router.start().await.unwrap();

        // Register some agents
        let (agent1, caps1) = create_test_agent(1, vec!["lifecycle-test"]);
        let (agent2, caps2) = create_test_agent(2, vec!["lifecycle-test"]);

        router.register_agent(agent1.clone(), caps1).await.unwrap();
        router.register_agent(agent2.clone(), caps2).await.unwrap();

        // Verify agents are registered
        let stats = router.get_stats().await.unwrap();
        assert_eq!(stats.agent_queue_depths.len(), 2);

        // Shutdown the router
        router.shutdown().await.unwrap();

        // After shutdown, health check should return error because router is not running
        let final_health = router.health_check().await;
        assert!(matches!(
            final_health,
            Err(RouterError::ConfigurationError { .. })
        ));
    }

    #[tokio::test]
    async fn test_agent_state_management() {
        let config = test_config();
        let router = MessageRouterImpl::new(config).await.unwrap();

        // Register an agent
        let (agent, capabilities) = create_test_agent(1, vec!["state-test"]);
        let agent_id = agent.id;

        router.register_agent(agent, capabilities).await.unwrap();

        // Try to update agent state
        let state_update_result = router
            .update_agent_state(agent_id, AgentState::Draining)
            .await;
        assert!(state_update_result.is_ok());

        // Verify agent is still in stats
        let stats = router.get_stats().await.unwrap();
        assert!(stats.agent_queue_depths.contains_key(&agent_id));
    }
}
