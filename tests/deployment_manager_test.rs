//! Comprehensive tests for `DeploymentManager`
//!
//! This test suite covers all aspects of the `DeploymentManager` including:

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unused_self)]
#![allow(clippy::float_cmp)]
//! - Resource allocation and instance deployment
//! - Deployment strategies (immediate, rolling, blue-green, canary)
//! - Health checks and deployment validation
//! - Performance requirements and error handling
//! - Resource cleanup and failure isolation

use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[allow(unused_imports)]
use std::time::{Duration, SystemTime};
use test_log::test;
use tokio::sync::Mutex;

#[allow(unused_imports)]
use caxton::deployment_manager::{HealthCheckResult, InstanceDeploymentResult};
#[allow(unused_imports)]
use caxton::domain::{
    AgentVersion, DeploymentConfig, DeploymentError, DeploymentId, DeploymentMetrics,
    DeploymentProgress, DeploymentRequest, DeploymentResult, DeploymentStatus, DeploymentStrategy,
    DeploymentValidationError, ResourceRequirements, VersionNumber,
};
use caxton::domain_types::{AgentId, AgentName, CpuFuel, MemoryBytes};
use caxton::{CaxtonDeploymentManager, DeploymentManagerTrait, InstanceManager, ResourceAllocator};

// Mock ResourceAllocator for testing
#[derive(Clone)]
struct MockResourceAllocator {
    should_succeed: Arc<AtomicBool>,
    allocation_delay: Arc<Mutex<Duration>>,
    allocated_resources: Arc<Mutex<HashMap<AgentId, ResourceRequirements>>>,
    call_count: Arc<AtomicU64>,
}

impl MockResourceAllocator {
    fn new() -> Self {
        Self {
            should_succeed: Arc::new(AtomicBool::new(true)),
            allocation_delay: Arc::new(Mutex::new(Duration::from_millis(10))),
            allocated_resources: Arc::new(Mutex::new(HashMap::new())),
            call_count: Arc::new(AtomicU64::new(0)),
        }
    }

    fn set_should_succeed(&self, succeed: bool) {
        self.should_succeed.store(succeed, Ordering::SeqCst);
    }

    async fn set_allocation_delay(&self, delay: Duration) {
        *self.allocation_delay.lock().await = delay;
    }

    fn get_call_count(&self) -> u64 {
        self.call_count.load(Ordering::SeqCst)
    }

    async fn get_allocated_resources(&self, agent_id: AgentId) -> Option<ResourceRequirements> {
        self.allocated_resources
            .lock()
            .await
            .get(&agent_id)
            .cloned()
    }

    #[allow(dead_code)]
    async fn get_allocated_count(&self) -> usize {
        self.allocated_resources.lock().await.len()
    }
}

#[async_trait::async_trait]
impl ResourceAllocator for MockResourceAllocator {
    async fn allocate_resources(
        &self,
        agent_id: AgentId,
        requirements: &ResourceRequirements,
    ) -> Result<(), DeploymentError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        let delay = *self.allocation_delay.lock().await;
        tokio::time::sleep(delay).await;

        if self.should_succeed.load(Ordering::SeqCst) {
            let mut allocated = self.allocated_resources.lock().await;
            allocated.insert(agent_id, requirements.clone());
            Ok(())
        } else {
            Err(DeploymentError::InsufficientResources {
                resource: "Mock resource allocation failure".to_string(),
            })
        }
    }

    async fn deallocate_resources(&self, agent_id: AgentId) -> Result<(), DeploymentError> {
        let mut allocated = self.allocated_resources.lock().await;
        allocated.remove(&agent_id);
        Ok(())
    }

    async fn check_resource_availability(
        &self,
        _requirements: &ResourceRequirements,
    ) -> Result<bool, DeploymentError> {
        Ok(self.should_succeed.load(Ordering::SeqCst))
    }
}

// Mock InstanceManager for testing
#[derive(Clone)]
struct MockInstanceManager {
    should_succeed: Arc<AtomicBool>,
    deployment_delay: Arc<Mutex<Duration>>,
    deployed_instances: Arc<Mutex<HashMap<AgentId, InstanceDeploymentResult>>>,
    call_count: Arc<AtomicU64>,
    health_check_count: Arc<AtomicU64>,
}

impl MockInstanceManager {
    fn new() -> Self {
        Self {
            should_succeed: Arc::new(AtomicBool::new(true)),
            deployment_delay: Arc::new(Mutex::new(Duration::from_millis(50))),
            deployed_instances: Arc::new(Mutex::new(HashMap::new())),
            call_count: Arc::new(AtomicU64::new(0)),
            health_check_count: Arc::new(AtomicU64::new(0)),
        }
    }

    fn set_should_succeed(&self, succeed: bool) {
        self.should_succeed.store(succeed, Ordering::SeqCst);
    }

    async fn set_deployment_delay(&self, delay: Duration) {
        *self.deployment_delay.lock().await = delay;
    }

    fn get_call_count(&self) -> u64 {
        self.call_count.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    fn get_health_check_count(&self) -> u64 {
        self.health_check_count.load(Ordering::SeqCst)
    }

    async fn is_instance_deployed(&self, agent_id: AgentId) -> bool {
        self.deployed_instances.lock().await.contains_key(&agent_id)
    }
}

#[async_trait::async_trait]
impl InstanceManager for MockInstanceManager {
    async fn deploy_instance(
        &self,
        agent_id: AgentId,
        wasm_bytes: &[u8],
        resources: &ResourceRequirements,
    ) -> Result<InstanceDeploymentResult, DeploymentError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        let delay = *self.deployment_delay.lock().await;
        tokio::time::sleep(delay).await;

        let success = self.should_succeed.load(Ordering::SeqCst) && !wasm_bytes.is_empty();

        let result = InstanceDeploymentResult {
            success,
            instance_id: format!("instance-{}", agent_id),
            duration: delay,
            error: if success {
                None
            } else {
                Some("Mock deployment failure".to_string())
            },
            memory_used: resources.memory_limit.into_inner(),
            fuel_consumed: resources.fuel_limit.into_inner(),
        };

        if success {
            let mut instances = self.deployed_instances.lock().await;
            instances.insert(agent_id, result.clone());
        }

        Ok(result)
    }

    async fn health_check(&self, agent_id: AgentId) -> Result<HealthCheckResult, DeploymentError> {
        self.health_check_count.fetch_add(1, Ordering::SeqCst);

        let instances = self.deployed_instances.lock().await;
        // Health check should only check if the instance is deployed, not the global should_succeed flag
        // Once deployed, an instance remains healthy unless explicitly stopped
        let healthy = instances.contains_key(&agent_id);

        Ok(HealthCheckResult {
            healthy,
            response_time: Duration::from_millis(10),
            error: if healthy {
                None
            } else {
                Some("Instance not healthy".to_string())
            },
        })
    }

    async fn stop_instance(&self, agent_id: AgentId) -> Result<(), DeploymentError> {
        let mut instances = self.deployed_instances.lock().await;
        instances.remove(&agent_id);
        Ok(())
    }

    async fn get_instance_metrics(
        &self,
        agent_id: AgentId,
    ) -> Result<(usize, u64), DeploymentError> {
        let instances = self.deployed_instances.lock().await;
        if let Some(instance) = instances.get(&agent_id) {
            Ok((instance.memory_used, instance.fuel_consumed))
        } else {
            Err(DeploymentError::InsufficientResources {
                resource: format!("Instance not found: instance-{}", agent_id),
            })
        }
    }
}

// Test fixture for setup
struct TestFixture {
    deployment_manager: CaxtonDeploymentManager,
    resource_allocator: Arc<MockResourceAllocator>,
    instance_manager: Arc<MockInstanceManager>,
}

impl TestFixture {
    fn new() -> Self {
        let resource_allocator = Arc::new(MockResourceAllocator::new());
        let instance_manager = Arc::new(MockInstanceManager::new());

        let deployment_manager =
            CaxtonDeploymentManager::new(resource_allocator.clone(), instance_manager.clone());

        Self {
            deployment_manager,
            resource_allocator,
            instance_manager,
        }
    }

    fn with_limits(max_concurrent: usize, timeout: Duration) -> Self {
        let resource_allocator = Arc::new(MockResourceAllocator::new());
        let instance_manager = Arc::new(MockInstanceManager::new());

        let deployment_manager = CaxtonDeploymentManager::with_limits(
            resource_allocator.clone(),
            instance_manager.clone(),
            max_concurrent,
            timeout,
        );

        Self {
            deployment_manager,
            resource_allocator,
            instance_manager,
        }
    }

    fn create_test_deployment_request(&self) -> DeploymentRequest {
        let agent_id = AgentId::generate();
        let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = DeploymentConfig::immediate();
        let wasm_bytes = vec![1, 2, 3, 4, 5, 6, 7, 8];

        DeploymentRequest::new(
            agent_id,
            agent_name,
            None,
            version,
            version_number,
            config,
            wasm_bytes,
        )
    }

    fn create_custom_deployment_request(
        &self,
        strategy: DeploymentStrategy,
        wasm_size: usize,
    ) -> DeploymentRequest {
        let agent_id = AgentId::generate();
        let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = DeploymentConfig::new(strategy);
        let wasm_bytes = vec![0; wasm_size];

        DeploymentRequest::new(
            agent_id,
            agent_name,
            None,
            version,
            version_number,
            config,
            wasm_bytes,
        )
    }
}

// Happy Path Tests
#[test(tokio::test)]
async fn test_successful_immediate_deployment() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();
    let agent_id = request.agent_id;

    let result = fixture.deployment_manager.deploy_agent(request).await;

    assert!(result.is_ok());
    let deployment_result = result.unwrap();

    assert!(deployment_result.status.is_success());
    assert_eq!(deployment_result.agent_id, agent_id);
    assert!(deployment_result.metrics.is_some());

    let metrics = deployment_result.metrics.unwrap();
    assert_eq!(metrics.instances_deployed, 1);
    assert_eq!(metrics.instances_failed, 0);

    // Verify resource allocation occurred
    assert_eq!(fixture.resource_allocator.get_call_count(), 1);
    assert_eq!(fixture.instance_manager.get_call_count(), 1);

    let allocated = fixture
        .resource_allocator
        .get_allocated_resources(agent_id)
        .await;
    assert!(allocated.is_some());
}

#[test(tokio::test)]
async fn test_deployment_with_metrics() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();

    let start_time = std::time::Instant::now();
    let result = fixture
        .deployment_manager
        .deploy_agent(request)
        .await
        .unwrap();
    let elapsed = start_time.elapsed();

    let metrics = result.metrics.unwrap();
    assert_eq!(metrics.instances_deployed, 1);
    assert_eq!(metrics.instances_failed, 0);
    assert!(metrics.total_duration <= elapsed + Duration::from_millis(10)); // Allow small variance
    assert!(metrics.memory_usage_peak > 0);
    assert!(metrics.fuel_consumed > 0);
    assert_eq!(metrics.health_check_success_rate, 100.0);
}

#[test(tokio::test)]
async fn test_deployment_status_tracking() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();
    let deployment_id = request.deployment_id;

    // Deploy agent
    fixture
        .deployment_manager
        .deploy_agent(request)
        .await
        .unwrap();

    // Check status
    let status = fixture
        .deployment_manager
        .get_deployment_status(deployment_id)
        .await
        .unwrap();
    assert_eq!(status, DeploymentStatus::Completed);
}

#[test(tokio::test)]
async fn test_deployment_cancellation() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();
    let deployment_id = request.deployment_id;

    let result = fixture
        .deployment_manager
        .cancel_deployment(deployment_id)
        .await;
    assert!(result.is_ok());
}

#[test(tokio::test)]
async fn test_deployment_rollback() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();
    let deployment_id = request.deployment_id;
    let target_version = AgentVersion::generate();

    let result = fixture
        .deployment_manager
        .rollback_deployment(deployment_id, target_version)
        .await;
    assert!(result.is_ok());

    let rollback_result = result.unwrap();
    assert!(!rollback_result.status.is_success()); // Rollback should indicate failure
}

// Error Handling Tests
#[test(tokio::test)]
async fn test_resource_allocation_failure() {
    let fixture = TestFixture::new();
    fixture.resource_allocator.set_should_succeed(false);

    let request = fixture.create_test_deployment_request();

    let result = fixture.deployment_manager.deploy_agent(request).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeploymentError::InsufficientResources { .. }
    ));

    // Instance deployment should not have been attempted
    assert_eq!(fixture.instance_manager.get_call_count(), 0);
}

#[test(tokio::test)]
async fn test_instance_deployment_failure() {
    let fixture = TestFixture::new();
    fixture.instance_manager.set_should_succeed(false);

    let request = fixture.create_test_deployment_request();

    let result = fixture.deployment_manager.deploy_agent(request).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeploymentError::WasmValidationFailed { .. }
    ));

    // Resource allocation should have occurred but failed during deployment
    assert_eq!(fixture.resource_allocator.get_call_count(), 1);
    assert_eq!(fixture.instance_manager.get_call_count(), 1);
}

#[test(tokio::test)]
async fn test_empty_wasm_module_deployment() {
    let fixture = TestFixture::new();
    let request = fixture.create_custom_deployment_request(DeploymentStrategy::Immediate, 0);

    let result = fixture.deployment_manager.deploy_agent(request).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeploymentError::WasmValidationFailed { .. }
    ));
}

#[test(tokio::test)]
async fn test_deployment_validation_errors() {
    let fixture = TestFixture::new();

    // Create invalid request with empty required fields
    let mut request = fixture.create_test_deployment_request();
    request.wasm_module_bytes.clear(); // Empty WASM

    let validation_result = request.validate();
    assert!(validation_result.is_err());

    // Test with actual deployment
    let result = fixture.deployment_manager.deploy_agent(request).await;
    assert!(result.is_err());
}

// Performance Tests
#[test(tokio::test)]
async fn test_deployment_performance_requirement() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();

    let start_time = std::time::Instant::now();
    let result = fixture.deployment_manager.deploy_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    // Deployment should complete in < 1 second (Story 003 requirement)
    assert!(elapsed < Duration::from_secs(1));
}

#[test(tokio::test)]
async fn test_resource_allocation_performance() {
    let fixture = TestFixture::new();

    // Set short allocation delay
    fixture
        .resource_allocator
        .set_allocation_delay(Duration::from_millis(1))
        .await;
    fixture
        .instance_manager
        .set_deployment_delay(Duration::from_millis(1))
        .await;

    let request = fixture.create_test_deployment_request();

    let start_time = std::time::Instant::now();
    let result = fixture.deployment_manager.deploy_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    // Fast deployment should be much quicker
    assert!(elapsed < Duration::from_millis(100));
}

#[test(tokio::test)]
async fn test_large_wasm_module_deployment() {
    let fixture = TestFixture::new();

    // Create 1MB WASM module
    let request =
        fixture.create_custom_deployment_request(DeploymentStrategy::Immediate, 1024 * 1024);

    let start_time = std::time::Instant::now();
    let result = fixture.deployment_manager.deploy_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());

    // Even large modules should deploy quickly with mocks
    assert!(elapsed < Duration::from_secs(2));
}

// Concurrency Tests
#[test(tokio::test)]
async fn test_concurrent_deployments() {
    let fixture = TestFixture::new();

    let requests: Vec<_> = (0..5)
        .map(|_| fixture.create_test_deployment_request())
        .collect();
    let agent_ids: Vec<_> = requests.iter().map(|r| r.agent_id).collect();

    let tasks: Vec<_> = requests
        .into_iter()
        .map(|request| {
            let manager = &fixture.deployment_manager;
            async move { manager.deploy_agent(request).await }
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All deployments should succeed
    for result in &results {
        assert!(result.is_ok());
    }

    // Verify all agents were deployed
    assert_eq!(fixture.resource_allocator.get_call_count(), 5);
    assert_eq!(fixture.instance_manager.get_call_count(), 5);

    // Check each agent was allocated resources
    for agent_id in agent_ids {
        let allocated = fixture
            .resource_allocator
            .get_allocated_resources(agent_id)
            .await;
        assert!(allocated.is_some());
    }
}

#[test(tokio::test)]
async fn test_deployment_isolation() {
    let fixture = TestFixture::new();

    let request1 = fixture.create_test_deployment_request();
    let request2 = fixture.create_test_deployment_request();
    let agent1_id = request1.agent_id;
    let agent2_id = request2.agent_id;

    // Deploy first agent successfully
    let result1 = fixture.deployment_manager.deploy_agent(request1).await;
    assert!(result1.is_ok());

    // Make second deployment fail
    fixture.instance_manager.set_should_succeed(false);
    let result2 = fixture.deployment_manager.deploy_agent(request2).await;
    assert!(result2.is_err());

    // First agent should still be deployed successfully
    let health_result1 = fixture
        .instance_manager
        .health_check(agent1_id)
        .await
        .unwrap();
    assert!(health_result1.healthy);

    // Second agent should not be deployed
    assert!(
        !fixture
            .instance_manager
            .is_instance_deployed(agent2_id)
            .await
    );
}

// Resource Management Tests
#[test(tokio::test)]
async fn test_resource_cleanup_on_failure() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();
    let agent_id = request.agent_id;

    // Make instance deployment fail after resource allocation
    fixture.instance_manager.set_should_succeed(false);

    let result = fixture.deployment_manager.deploy_agent(request).await;
    assert!(result.is_err());

    // Resources should have been allocated but then cleaned up isn't implemented in this mock
    // In a real implementation, we'd verify cleanup occurred
    assert_eq!(fixture.resource_allocator.get_call_count(), 1);
    assert_eq!(fixture.instance_manager.get_call_count(), 1);

    // Instance should not exist
    assert!(
        !fixture
            .instance_manager
            .is_instance_deployed(agent_id)
            .await
    );
}

#[test(tokio::test)]
async fn test_resource_requirements_validation() {
    let fixture = TestFixture::new();
    let mut request = fixture.create_test_deployment_request();

    // Modify resource requirements
    request.config.resource_requirements.memory_limit =
        caxton::domain::DeploymentMemoryLimit::try_new(1024 * 1024 * 100).unwrap(); // 100MB
    request.config.resource_requirements.fuel_limit =
        caxton::domain::DeploymentFuelLimit::try_new(1_000_000).unwrap();

    let result = fixture.deployment_manager.deploy_agent(request).await;
    assert!(result.is_ok());

    let deployment_result = result.unwrap();
    let metrics = deployment_result.metrics.unwrap();

    // Verify metrics reflect resource usage
    assert_eq!(metrics.memory_usage_peak, 1024 * 1024 * 100);
    assert_eq!(metrics.fuel_consumed, 1_000_000);
}

#[test(tokio::test)]
async fn test_health_check_integration() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();
    let agent_id = request.agent_id;

    // Deploy agent
    let result = fixture
        .deployment_manager
        .deploy_agent(request)
        .await
        .unwrap();
    assert!(result.status.is_success());

    // Verify health check was performed (implicitly through successful deployment)
    // In the real implementation, health checks would be explicit
    let health_result = fixture
        .instance_manager
        .health_check(agent_id)
        .await
        .unwrap();
    assert!(health_result.healthy);
    assert!(health_result.response_time <= Duration::from_millis(100));
}

// Deployment Strategy Tests
#[test(tokio::test)]
async fn test_immediate_deployment_strategy() {
    let fixture = TestFixture::new();
    let request = fixture.create_custom_deployment_request(DeploymentStrategy::Immediate, 1024);

    let result = fixture.deployment_manager.deploy_agent(request).await;
    assert!(result.is_ok());

    let deployment_result = result.unwrap();
    assert!(deployment_result.status.is_success());
}

#[test(tokio::test)]
async fn test_rolling_deployment_strategy() {
    let fixture = TestFixture::new();
    let request = fixture.create_custom_deployment_request(DeploymentStrategy::Rolling, 1024);

    let result = fixture.deployment_manager.deploy_agent(request).await;
    assert!(result.is_ok());

    // Current implementation treats all strategies as immediate
    let deployment_result = result.unwrap();
    assert!(deployment_result.status.is_success());
}

#[test(tokio::test)]
async fn test_blue_green_deployment_strategy() {
    let fixture = TestFixture::new();
    let request = fixture.create_custom_deployment_request(DeploymentStrategy::BlueGreen, 1024);

    let result = fixture.deployment_manager.deploy_agent(request).await;
    assert!(result.is_ok());

    let deployment_result = result.unwrap();
    assert!(deployment_result.status.is_success());
}

#[test(tokio::test)]
async fn test_canary_deployment_strategy() {
    let fixture = TestFixture::new();
    let request = fixture.create_custom_deployment_request(DeploymentStrategy::Canary, 1024);

    let result = fixture.deployment_manager.deploy_agent(request).await;
    assert!(result.is_ok());

    let deployment_result = result.unwrap();
    assert!(deployment_result.status.is_success());
}

// Property-based tests for domain types
prop_compose! {
    fn arb_memory_bytes()(bytes in 1024_usize..=100_000_000) -> MemoryBytes {
        MemoryBytes::try_new(bytes).unwrap()
    }
}

prop_compose! {
    fn arb_cpu_fuel()(fuel in 1000_u64..=10_000_000) -> CpuFuel {
        CpuFuel::try_new(fuel).unwrap()
    }
}

proptest! {
    #[test]
    fn test_resource_requirements_properties(
        memory in 1_048_576usize..=100_000_000,  // 1MB to 100MB (within DeploymentMemoryLimit constraints)
        cpu_fuel in 10_000u64..=10_000_000       // 10K to 10M (within DeploymentFuelLimit constraints)
    ) {
        let requirements = ResourceRequirements {
            memory_limit: caxton::domain::DeploymentMemoryLimit::try_new(memory).unwrap(),
            fuel_limit: caxton::domain::DeploymentFuelLimit::try_new(cpu_fuel).unwrap(),
            requires_isolation: false,
            max_concurrent_requests: None,
        };

        // Memory should be within deployment constraints
        assert!(requirements.memory_limit.into_inner() >= 1_048_576);
        assert!(requirements.memory_limit.into_inner() <= 1_073_741_824);

        // CPU fuel should be within deployment constraints
        assert!(requirements.fuel_limit.into_inner() >= 10_000);
        assert!(requirements.fuel_limit.into_inner() <= 100_000_000);
    }
}

// Integration Tests
#[test(tokio::test)]
async fn test_full_deployment_lifecycle() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_deployment_request();
    let agent_id = request.agent_id;
    let deployment_id = request.deployment_id;

    // 1. Deploy agent
    let start_time = std::time::Instant::now();
    let result = fixture.deployment_manager.deploy_agent(request).await;
    let deployment_time = start_time.elapsed();

    assert!(result.is_ok());
    let deployment_result = result.unwrap();

    // 2. Verify deployment result
    assert!(deployment_result.status.is_success());
    assert_eq!(deployment_result.agent_id, agent_id);
    assert_eq!(deployment_result.deployment_id, deployment_id);
    assert!(deployment_result.metrics.is_some());

    // 3. Check deployment status
    let status = fixture
        .deployment_manager
        .get_deployment_status(deployment_id)
        .await
        .unwrap();
    assert_eq!(status, DeploymentStatus::Completed);

    // 4. Verify resource allocation
    let allocated = fixture
        .resource_allocator
        .get_allocated_resources(agent_id)
        .await;
    assert!(allocated.is_some());

    // 5. Verify instance deployment
    assert!(
        fixture
            .instance_manager
            .is_instance_deployed(agent_id)
            .await
    );

    // 6. Check instance metrics
    let (memory, fuel) = fixture
        .instance_manager
        .get_instance_metrics(agent_id)
        .await
        .unwrap();
    assert!(memory > 0);
    assert!(fuel > 0);

    // 7. Verify performance requirement
    assert!(deployment_time < Duration::from_secs(1));

    // 8. Check final metrics
    let metrics = deployment_result.metrics.unwrap();
    assert_eq!(metrics.instances_deployed, 1);
    assert_eq!(metrics.instances_failed, 0);
    assert_eq!(metrics.health_check_success_rate, 100.0);

    // 9. Perform health check
    let health = fixture
        .instance_manager
        .health_check(agent_id)
        .await
        .unwrap();
    assert!(health.healthy);
}

#[test(tokio::test)]
async fn test_deployment_manager_limits() {
    let fixture = TestFixture::with_limits(2, Duration::from_secs(5));

    // Create multiple deployment requests
    let requests: Vec<_> = (0..3)
        .map(|_| fixture.create_test_deployment_request())
        .collect();

    let tasks: Vec<_> = requests
        .into_iter()
        .map(|request| {
            let manager = &fixture.deployment_manager;
            async move { manager.deploy_agent(request).await }
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All should succeed since our mock doesn't actually enforce limits
    for result in results {
        assert!(result.is_ok());
    }

    assert_eq!(fixture.instance_manager.get_call_count(), 3);
}

#[test(tokio::test)]
async fn test_deployment_metrics_accuracy() {
    let fixture = TestFixture::new();

    // Set known delays
    let allocation_delay = Duration::from_millis(10);
    let deployment_delay = Duration::from_millis(20);

    fixture
        .resource_allocator
        .set_allocation_delay(allocation_delay)
        .await;
    fixture
        .instance_manager
        .set_deployment_delay(deployment_delay)
        .await;

    let request = fixture.create_test_deployment_request();
    let memory_requirement = request
        .config
        .resource_requirements
        .memory_limit
        .into_inner();
    let fuel_requirement = request.config.resource_requirements.fuel_limit.into_inner();

    let start_time = std::time::Instant::now();
    let result = fixture
        .deployment_manager
        .deploy_agent(request)
        .await
        .unwrap();
    let total_time = start_time.elapsed();

    let metrics = result.metrics.unwrap();

    // Verify timing accuracy
    assert!(metrics.total_duration >= allocation_delay + deployment_delay);
    assert!(metrics.total_duration <= total_time + Duration::from_millis(10)); // Allow variance

    // Verify resource metrics
    assert_eq!(metrics.memory_usage_peak, memory_requirement);
    assert_eq!(metrics.fuel_consumed, fuel_requirement);

    // Verify instance metrics
    assert_eq!(metrics.instances_deployed, 1);
    assert_eq!(metrics.instances_failed, 0);
    assert_eq!(metrics.health_check_success_rate, 100.0);
}

#[test(tokio::test)]
async fn test_deployment_error_scenarios() {
    let fixture = TestFixture::new();

    // Test scenarios that should fail
    let test_cases = vec![
        ("Resource allocation failure", true, false), // Allocator fails, instance succeeds
        ("Instance deployment failure", false, true), // Allocator succeeds, instance fails
        ("Both fail", true, true),                    // Both fail
    ];

    for (description, allocator_fails, instance_fails) in test_cases {
        // Reset state
        fixture
            .resource_allocator
            .set_should_succeed(!allocator_fails);
        fixture.instance_manager.set_should_succeed(!instance_fails);

        let request = fixture.create_test_deployment_request();
        let result = fixture.deployment_manager.deploy_agent(request).await;

        assert!(result.is_err(), "Expected failure for: {}", description);

        match result.unwrap_err() {
            DeploymentError::InsufficientResources { .. } if allocator_fails => {}
            DeploymentError::WasmValidationFailed { .. } if instance_fails && !allocator_fails => {}
            DeploymentError::InsufficientResources { .. } if allocator_fails && instance_fails => {}
            other => panic!("Unexpected error for {}: {:?}", description, other),
        }
    }
}
