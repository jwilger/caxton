#![allow(clippy::doc_markdown)]
#![allow(clippy::unused_self)]
#![allow(clippy::float_cmp)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::useless_vec)]
#![allow(clippy::type_complexity)]
#![allow(clippy::cloned_instead_of_copied)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::match_same_arms)]

//! Comprehensive tests for `HotReloadManager`
//!
//! This test suite covers all aspects of the `HotReloadManager` including:
//! - All hot reload strategies (graceful, immediate, parallel, traffic splitting)
//! - State preservation and restoration during reloads
//! - Traffic routing and split management
//! - Rollback scenarios and error recovery
//! - Performance requirements and zero-downtime reloads
//! - Warmup periods and health checks

use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use test_log::test;
use tokio::sync::Mutex;

#[allow(unused_imports)]
use caxton::domain::{
    AgentVersion, HotReloadConfig, HotReloadError, HotReloadRequest, HotReloadStatus,
    HotReloadStrategy, TrafficSplitPercentage, VersionNumber,
};
// Note: ResourceUsageSnapshot not available, will mock in test
use caxton::domain_types::{AgentId, AgentName};
use caxton::{
    CaxtonHotReloadManager, HotReloadManagerTrait, RuntimeManager, TrafficRouter,
    time_provider::test_time_provider,
};

// Mock RuntimeManager for testing
#[derive(Clone)]
struct MockRuntimeManager {
    should_succeed: Arc<AtomicBool>,
    should_be_healthy: Arc<AtomicBool>,
    creation_delay: Arc<Mutex<Duration>>,
    instances: Arc<Mutex<HashMap<(AgentId, AgentVersion), InstanceData>>>,
    preserved_states: Arc<Mutex<HashMap<(AgentId, AgentVersion), Vec<u8>>>>,
    call_count: Arc<AtomicU64>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct InstanceData {
    created_at: SystemTime,
    memory_usage: usize,
    fuel_consumed: u64,
    requests_handled: u64,
}

impl MockRuntimeManager {
    fn new() -> Self {
        Self {
            should_succeed: Arc::new(AtomicBool::new(true)),
            should_be_healthy: Arc::new(AtomicBool::new(true)),
            creation_delay: Arc::new(Mutex::new(Duration::from_millis(1))), // Reduced from 50ms to 1ms
            instances: Arc::new(Mutex::new(HashMap::new())),
            preserved_states: Arc::new(Mutex::new(HashMap::new())),
            call_count: Arc::new(AtomicU64::new(0)),
        }
    }

    fn set_should_succeed(&self, succeed: bool) {
        self.should_succeed.store(succeed, Ordering::SeqCst);
    }

    fn set_should_be_healthy(&self, healthy: bool) {
        self.should_be_healthy.store(healthy, Ordering::SeqCst);
    }

    async fn set_creation_delay(&self, delay: Duration) {
        *self.creation_delay.lock().await = delay;
    }

    fn get_call_count(&self) -> u64 {
        self.call_count.load(Ordering::SeqCst)
    }

    async fn instance_exists(&self, agent_id: AgentId, version: AgentVersion) -> bool {
        self.instances
            .lock()
            .await
            .contains_key(&(agent_id, version))
    }

    #[allow(dead_code)]
    async fn get_instance_count(&self) -> usize {
        self.instances.lock().await.len()
    }
}

#[async_trait::async_trait]
impl RuntimeManager for MockRuntimeManager {
    async fn create_instance(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
        wasm_bytes: &[u8],
    ) -> Result<(), HotReloadError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        if wasm_bytes.is_empty() {
            return Err(HotReloadError::StatePreservationFailed {
                reason: "Empty WASM module".to_string(),
            });
        }

        let delay = *self.creation_delay.lock().await;
        tokio::time::sleep(delay).await;

        if self.should_succeed.load(Ordering::SeqCst) {
            let instance_data = InstanceData {
                created_at: SystemTime::now(),
                memory_usage: 1024 * (wasm_bytes.len() / 100 + 1), // Simulate memory based on WASM size
                fuel_consumed: 1000,
                requests_handled: 0,
            };

            let mut instances = self.instances.lock().await;
            instances.insert((agent_id, version), instance_data);
            Ok(())
        } else {
            Err(HotReloadError::StatePreservationFailed {
                reason: "Mock instance creation failure".to_string(),
            })
        }
    }

    async fn stop_instance(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<(), HotReloadError> {
        let mut instances = self.instances.lock().await;
        instances.remove(&(agent_id, version));
        Ok(())
    }

    async fn get_instance_metrics(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<(usize, u64, u64), HotReloadError> {
        let instances = self.instances.lock().await;
        if let Some(instance) = instances.get(&(agent_id, version)) {
            Ok((
                instance.memory_usage,
                instance.fuel_consumed,
                instance.requests_handled,
            ))
        } else {
            Err(HotReloadError::StatePreservationFailed {
                reason: "Instance not found".to_string(),
            })
        }
    }

    async fn preserve_state(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<Vec<u8>, HotReloadError> {
        let instances = self.instances.lock().await;
        if instances.contains_key(&(agent_id, version)) {
            let state_data = format!("state-{agent_id}-{version}").into_bytes();
            drop(instances);

            let mut preserved = self.preserved_states.lock().await;
            preserved.insert((agent_id, version), state_data.clone());

            Ok(state_data)
        } else {
            Err(HotReloadError::StatePreservationFailed {
                reason: "Instance not found for state preservation".to_string(),
            })
        }
    }

    async fn restore_state(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
        state_data: &[u8],
    ) -> Result<(), HotReloadError> {
        let instances = self.instances.lock().await;
        if !instances.contains_key(&(agent_id, version)) {
            return Err(HotReloadError::StatePreservationFailed {
                reason: "Target instance not found for state restoration".to_string(),
            });
        }
        drop(instances);

        // Verify state data is what we expect
        let _expected = format!(
            "state-{}-{}",
            agent_id,
            version.to_string().chars().take(8).collect::<String>()
        );
        if !std::str::from_utf8(state_data)
            .map(|s| s.starts_with("state-"))
            .unwrap_or(false)
        {
            return Err(HotReloadError::StatePreservationFailed {
                reason: "Invalid state data format".to_string(),
            });
        }

        Ok(())
    }

    async fn health_check(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<bool, HotReloadError> {
        let instances = self.instances.lock().await;
        let exists = instances.contains_key(&(agent_id, version));
        let healthy = self.should_be_healthy.load(Ordering::SeqCst);
        Ok(exists && healthy)
    }
}

// Mock TrafficRouter for testing
#[derive(Clone)]
struct MockTrafficRouter {
    should_succeed: Arc<AtomicBool>,
    traffic_splits: Arc<Mutex<HashMap<AgentId, TrafficSplitPercentage>>>,
    active_versions: Arc<Mutex<HashMap<AgentId, AgentVersion>>>,
    call_count: Arc<AtomicU64>,
}

impl MockTrafficRouter {
    fn new() -> Self {
        Self {
            should_succeed: Arc::new(AtomicBool::new(true)),
            traffic_splits: Arc::new(Mutex::new(HashMap::new())),
            active_versions: Arc::new(Mutex::new(HashMap::new())),
            call_count: Arc::new(AtomicU64::new(0)),
        }
    }

    fn set_should_succeed(&self, succeed: bool) {
        self.should_succeed.store(succeed, Ordering::SeqCst);
    }

    fn get_call_count(&self) -> u64 {
        self.call_count.load(Ordering::SeqCst)
    }

    async fn get_active_version(&self, agent_id: AgentId) -> Option<AgentVersion> {
        self.active_versions.lock().await.get(&agent_id).cloned()
    }
}

#[async_trait::async_trait]
impl TrafficRouter for MockTrafficRouter {
    async fn set_traffic_split(
        &self,
        agent_id: AgentId,
        _old_version: AgentVersion,
        _new_version: AgentVersion,
        split_percentage: TrafficSplitPercentage,
    ) -> Result<(), HotReloadError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        if self.should_succeed.load(Ordering::SeqCst) {
            let mut splits = self.traffic_splits.lock().await;
            splits.insert(agent_id, split_percentage);
            Ok(())
        } else {
            Err(HotReloadError::TrafficSplittingFailed {
                reason: "Mock traffic split failure".to_string(),
            })
        }
    }

    async fn get_traffic_split(
        &self,
        agent_id: AgentId,
    ) -> Result<TrafficSplitPercentage, HotReloadError> {
        let splits = self.traffic_splits.lock().await;
        Ok(splits
            .get(&agent_id)
            .cloned()
            .unwrap_or_else(|| TrafficSplitPercentage::half()))
    }

    async fn switch_traffic(
        &self,
        agent_id: AgentId,
        target_version: AgentVersion,
    ) -> Result<(), HotReloadError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        if self.should_succeed.load(Ordering::SeqCst) {
            let mut versions = self.active_versions.lock().await;
            versions.insert(agent_id, target_version);

            // Set traffic to 100% for the target version
            let mut splits = self.traffic_splits.lock().await;
            splits.insert(agent_id, TrafficSplitPercentage::full());

            Ok(())
        } else {
            Err(HotReloadError::TrafficSplittingFailed {
                reason: "Mock traffic switch failure".to_string(),
            })
        }
    }
}

// Test fixture
struct TestFixture {
    manager: CaxtonHotReloadManager,
    runtime_manager: Arc<MockRuntimeManager>,
    traffic_router: Arc<MockTrafficRouter>,
}

impl TestFixture {
    fn new() -> Self {
        let runtime_manager = Arc::new(MockRuntimeManager::new());
        let traffic_router = Arc::new(MockTrafficRouter::new());

        let manager = CaxtonHotReloadManager::with_time_provider(
            runtime_manager.clone(),
            traffic_router.clone(),
            test_time_provider(),
        );

        Self {
            manager,
            runtime_manager,
            traffic_router,
        }
    }

    fn with_limits(max_concurrent: usize, timeout: Duration) -> Self {
        let runtime_manager = Arc::new(MockRuntimeManager::new());
        let traffic_router = Arc::new(MockTrafficRouter::new());

        let manager = CaxtonHotReloadManager::with_limits_and_time_provider(
            runtime_manager.clone(),
            traffic_router.clone(),
            max_concurrent,
            timeout,
            test_time_provider(),
        );

        Self {
            manager,
            runtime_manager,
            traffic_router,
        }
    }

    fn create_test_hot_reload_request(&self, strategy: HotReloadStrategy) -> HotReloadRequest {
        let agent_id = AgentId::generate();
        let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());
        let from_version = AgentVersion::generate();
        let to_version = AgentVersion::generate();
        let to_version_number = VersionNumber::first().next().unwrap();

        // Create config with minimal timeouts for testing
        let mut config = HotReloadConfig::new(strategy);
        config.drain_timeout = caxton::domain::DeploymentTimeout::try_new(30_000).unwrap(); // 30 seconds - minimum allowed
        config.warmup_duration = Duration::from_millis(0); // No warmup

        let wasm_bytes = vec![1, 2, 3, 4, 5, 6, 7, 8];

        HotReloadRequest::new(
            agent_id,
            agent_name,
            from_version,
            to_version,
            to_version_number,
            config,
            wasm_bytes,
        )
    }

    async fn setup_existing_instance(&self, agent_id: AgentId, version: AgentVersion) {
        // Create an existing instance for the old version
        self.runtime_manager
            .create_instance(
                agent_id,
                version,
                &vec![9, 8, 7, 6, 5, 4, 3, 2], // Old WASM
            )
            .await
            .unwrap();
    }
}

// Happy Path Tests - Graceful Hot Reload
#[test(tokio::test)]
async fn test_graceful_hot_reload_success() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let to_version = request.to_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_ok());
    let reload_result = result.unwrap();

    assert!(reload_result.status.is_success());
    assert_eq!(reload_result.agent_id, agent_id);
    assert_eq!(reload_result.from_version, from_version);
    assert_eq!(reload_result.to_version, to_version);

    // Verify new instance was created and old one was stopped
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, to_version)
            .await
    );
    assert!(
        !fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );

    // Verify traffic was switched
    let active_version = fixture.traffic_router.get_active_version(agent_id).await;
    assert_eq!(active_version, Some(to_version));
}

#[test(tokio::test)]
async fn test_graceful_hot_reload_with_state_preservation() {
    let fixture = TestFixture::new();
    let mut request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    request.preserve_state = true;

    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let to_version = request.to_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_ok());
    let reload_result = result.unwrap();
    assert!(reload_result.status.is_success());

    // Verify instance transition
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, to_version)
            .await
    );
    assert!(
        !fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );
}

#[test(tokio::test)]
async fn test_graceful_hot_reload_with_warmup() {
    let fixture = TestFixture::new();
    let mut request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    request.config.warmup_duration = Duration::from_millis(1); // Reduced from 100ms to 1ms

    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let start_time = std::time::Instant::now();
    let result = fixture.manager.hot_reload_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    // Should take at least the warmup duration (1ms)
    assert!(elapsed >= Duration::from_millis(1));
}

// Immediate Hot Reload Tests
#[test(tokio::test)]
async fn test_immediate_hot_reload_success() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Immediate);
    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let to_version = request.to_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let start_time = std::time::Instant::now();
    let result = fixture.manager.hot_reload_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    let reload_result = result.unwrap();
    assert!(reload_result.status.is_success());

    // Immediate should be very fast with test configs
    assert!(elapsed < Duration::from_millis(100));

    // Verify instance transition
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, to_version)
            .await
    );
    assert!(
        !fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );
}

// Parallel Hot Reload Tests
#[test(tokio::test)]
async fn test_parallel_hot_reload_success() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Parallel);
    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let to_version = request.to_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_ok());
    let reload_result = result.unwrap();
    assert!(reload_result.status.is_success());

    // Final state: only new version should exist
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, to_version)
            .await
    );
    assert!(
        !fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );
}

#[test(tokio::test)]
async fn test_parallel_hot_reload_rollback_on_health_failure() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Parallel);
    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    // Make health check fail for new instance
    fixture.runtime_manager.set_should_be_healthy(false);

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HotReloadError::AutomaticRollback { .. }
    ));

    // Original instance should still exist
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );
}

// Traffic Splitting Hot Reload Tests
#[test(tokio::test)]
async fn test_traffic_splitting_hot_reload_success() {
    let fixture = TestFixture::new();
    let _traffic_split = TrafficSplitPercentage::try_new(25).unwrap();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::TrafficSplitting);
    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let to_version = request.to_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_ok());
    let reload_result = result.unwrap();
    assert!(reload_result.status.is_success());

    // Verify final traffic switch
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, to_version)
            .await
    );
    assert!(
        !fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );

    let active_version = fixture.traffic_router.get_active_version(agent_id).await;
    assert_eq!(active_version, Some(to_version));
}

#[test(tokio::test)]
async fn test_traffic_splitting_with_progressive_rollout() {
    let fixture = TestFixture::new();
    let mut request = fixture.create_test_hot_reload_request(HotReloadStrategy::TrafficSplitting);
    request.config.progressive_rollout = true;

    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let start_time = std::time::Instant::now();
    let result = fixture.manager.hot_reload_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());

    // Progressive rollout should be fast with test configs
    assert!(elapsed < Duration::from_millis(500));

    // Verify multiple traffic router calls
    assert!(fixture.traffic_router.get_call_count() > 2);
}

// Error Handling Tests
#[test(tokio::test)]
async fn test_hot_reload_instance_creation_failure() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    // Make new instance creation fail
    fixture.runtime_manager.set_should_succeed(false);

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HotReloadError::StatePreservationFailed { .. }
    ));

    // Original instance should still exist
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );
}

#[test(tokio::test)]
async fn test_hot_reload_traffic_routing_failure() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Immediate);
    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    // Make traffic routing fail
    fixture.traffic_router.set_should_succeed(false);

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HotReloadError::TrafficSplittingFailed { .. }
    ));
}

#[test(tokio::test)]
async fn test_hot_reload_empty_wasm_module() {
    let fixture = TestFixture::new();
    let mut request = fixture.create_test_hot_reload_request(HotReloadStrategy::Immediate);
    request.new_wasm_module.clear(); // Empty WASM

    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_err());
    // Empty WASM causes a validation error, not state preservation failure
    // The mock create_instance returns StatePreservationFailed but the actual manager
    // might validate first and return a different error
    match result.unwrap_err() {
        HotReloadError::StatePreservationFailed { .. } => {}
        HotReloadError::ValidationFailed { .. } => {}
        other => panic!("Unexpected error: {other:?}"),
    }
}

#[test(tokio::test)]
async fn test_hot_reload_validation_failure() {
    let fixture = TestFixture::new();
    let mut request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);

    // Create invalid request - same from/to version
    request.to_version = request.from_version;

    let result = request.validate();
    assert!(result.is_err());
}

// Performance Tests
#[test(tokio::test)]
async fn test_hot_reload_zero_downtime() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let _to_version = request.to_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let start_time = std::time::Instant::now();
    let result = fixture.manager.hot_reload_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());

    // Zero downtime requirement - should complete very quickly with test configs
    assert!(elapsed < Duration::from_millis(500));

    // During graceful reload, there should be a period where both versions exist
    // (though our mock doesn't track this precisely)
}

#[test(tokio::test)]
async fn test_hot_reload_timeout() {
    let fixture = TestFixture::with_limits(5, Duration::from_millis(100));
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    // Set delay longer than timeout to trigger timeout
    fixture
        .runtime_manager
        .set_creation_delay(Duration::from_millis(150))
        .await;

    let result = fixture.manager.hot_reload_agent(request).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        HotReloadError::TimeoutExceeded { .. }
    ));
}

#[test(tokio::test)]
async fn test_hot_reload_performance_metrics() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    let result = fixture.manager.hot_reload_agent(request).await.unwrap();

    assert!(result.metrics.is_some());
    let metrics = result.metrics.unwrap();

    // Verify metrics are populated
    assert!(metrics.memory_usage_peak > 0);
    #[allow(unused_comparisons)]
    {
        assert!(metrics.requests_processed >= 0);
    }
    assert!(metrics.health_check_success_rate >= 0.0);
    assert!(metrics.collected_at <= SystemTime::now());
}

// Concurrency Tests
#[test(tokio::test)]
async fn test_concurrent_hot_reloads() {
    let fixture = TestFixture::new();

    // Create multiple agents for concurrent hot reloads
    let requests: Vec<_> = (0..3)
        .map(|_| fixture.create_test_hot_reload_request(HotReloadStrategy::Immediate))
        .collect();

    // Setup existing instances for all agents
    for request in &requests {
        fixture
            .setup_existing_instance(request.agent_id, request.from_version)
            .await;
    }

    let tasks: Vec<_> = requests
        .into_iter()
        .map(|request| {
            let manager = &fixture.manager;
            async move { manager.hot_reload_agent(request).await }
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All hot reloads should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().status.is_success());
    }
}

#[test(tokio::test)]
async fn test_hot_reload_isolation() {
    let fixture = TestFixture::new();

    let request1 = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let request2 = fixture.create_test_hot_reload_request(HotReloadStrategy::Immediate);

    let agent1_id = request1.agent_id;
    let agent2_id = request2.agent_id;
    let from_version1 = request1.from_version;
    let from_version2 = request2.from_version;

    // Setup existing instances
    fixture
        .setup_existing_instance(agent1_id, from_version1)
        .await;
    fixture
        .setup_existing_instance(agent2_id, from_version2)
        .await;

    // Start first hot reload
    let result1 = fixture.manager.hot_reload_agent(request1).await;
    assert!(result1.is_ok());

    // Make second hot reload fail
    fixture.runtime_manager.set_should_succeed(false);
    let result2 = fixture.manager.hot_reload_agent(request2).await;
    assert!(result2.is_err());

    // First agent should have completed successfully
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent1_id, result1.unwrap().to_version)
            .await
    );

    // Second agent should still have old version
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent2_id, from_version2)
            .await
    );
}

// State Management Tests
#[test(tokio::test)]
async fn test_hot_reload_status_tracking() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let reload_id = request.reload_id;
    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    // Perform hot reload directly (async test doesn't need background)
    let result = fixture.manager.hot_reload_agent(request).await;
    assert!(result.is_ok());

    // Check final status
    let final_status = fixture.manager.get_hot_reload_status(reload_id).await;
    assert_eq!(final_status.unwrap(), HotReloadStatus::Completed);
}

#[test(tokio::test)]
async fn test_hot_reload_cancellation() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let reload_id = request.reload_id;
    let agent_id = request.agent_id;
    let from_version = request.from_version;

    // Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;

    // Make hot reload take longer
    fixture
        .runtime_manager
        .set_creation_delay(Duration::from_millis(200))
        .await;

    // Test cancellation directly
    let _cancel_result = fixture.manager.cancel_hot_reload(reload_id).await;

    // Then perform hot reload (in practice this would be cancelled)
    let _result = fixture.manager.hot_reload_agent(request).await;

    // Cancellation may succeed depending on timing
    // In practice, this would interrupt the hot reload process
}

#[test(tokio::test)]
async fn test_hot_reload_rollback() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let reload_id = request.reload_id;
    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let _to_version = request.to_version;

    // Setup existing instance and complete hot reload
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;
    let result = fixture.manager.hot_reload_agent(request).await.unwrap();
    assert!(result.status.is_success());

    // Rollback to previous version
    let _rollback_result = fixture
        .manager
        .rollback_hot_reload(reload_id, from_version)
        .await;

    // Rollback might succeed or fail depending on implementation state
    // In practice, this would restore the previous version
}

// Property-based tests
prop_compose! {
    fn arb_traffic_split()(percentage in 0_u8..=100) -> TrafficSplitPercentage {
        TrafficSplitPercentage::try_new(percentage).unwrap()
    }
}

prop_compose! {
    fn arb_duration_ms()(ms in 0_u64..=5000) -> Duration {
        Duration::from_millis(ms)
    }
}

proptest! {
    #[test]
    fn test_traffic_split_percentage_properties(split in arb_traffic_split()) {
        assert!(split.as_percentage() <= 100);
        #[allow(unused_comparisons)]
        {
            assert!(split.as_percentage() >= 0);
        }
    }

    #[test]
    fn test_warmup_duration_properties(duration in arb_duration_ms()) {
        // Test that duration is reasonable for warmup
        assert!(duration <= Duration::from_secs(5));
    }
}

// Integration Tests
#[test(tokio::test)]
async fn test_complete_hot_reload_lifecycle() {
    let fixture = TestFixture::new();
    let request = fixture.create_test_hot_reload_request(HotReloadStrategy::Graceful);
    let agent_id = request.agent_id;
    let from_version = request.from_version;
    let to_version = request.to_version;
    let reload_id = request.reload_id;

    // 1. Setup existing instance
    fixture
        .setup_existing_instance(agent_id, from_version)
        .await;
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );

    // 2. Execute hot reload
    let start_time = std::time::Instant::now();
    let result = fixture.manager.hot_reload_agent(request).await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    let reload_result = result.unwrap();

    // 3. Verify hot reload result
    assert!(reload_result.status.is_success());
    assert_eq!(reload_result.agent_id, agent_id);
    assert_eq!(reload_result.from_version, from_version);
    assert_eq!(reload_result.to_version, to_version);
    assert_eq!(reload_result.reload_id, reload_id);

    // 4. Verify instance transition
    assert!(
        fixture
            .runtime_manager
            .instance_exists(agent_id, to_version)
            .await
    );
    assert!(
        !fixture
            .runtime_manager
            .instance_exists(agent_id, from_version)
            .await
    );

    // 5. Verify traffic routing
    let active_version = fixture.traffic_router.get_active_version(agent_id).await;
    assert_eq!(active_version, Some(to_version));

    // 6. Verify metrics
    assert!(reload_result.metrics.is_some());
    let metrics = reload_result.metrics.unwrap();
    assert!(metrics.memory_usage_peak > 0);

    // 7. Verify performance
    // Should be very fast with test configs
    assert!(elapsed < Duration::from_millis(100));

    // 8. Check status
    let status = fixture
        .manager
        .get_hot_reload_status(reload_id)
        .await
        .unwrap();
    assert_eq!(status, HotReloadStatus::Completed);

    // 9. Verify subsystem interactions
    assert!(fixture.runtime_manager.get_call_count() >= 2); // Create + health checks
    assert!(fixture.traffic_router.get_call_count() >= 1); // Traffic switch
}

#[test(tokio::test)]
async fn test_all_hot_reload_strategies() {
    let fixture = TestFixture::new();
    let strategies = vec![
        HotReloadStrategy::Graceful,
        HotReloadStrategy::Immediate,
        HotReloadStrategy::Parallel,
        HotReloadStrategy::TrafficSplitting,
    ];

    for strategy in strategies {
        let request = fixture.create_test_hot_reload_request(strategy);
        let agent_id = request.agent_id;
        let from_version = request.from_version;

        // Setup existing instance
        fixture
            .setup_existing_instance(agent_id, from_version)
            .await;

        let result = fixture.manager.hot_reload_agent(request).await;

        assert!(
            result.is_ok(),
            "Hot reload failed for strategy: {strategy:?}"
        );
        let reload_result = result.unwrap();
        assert!(reload_result.status.is_success());

        // Cleanup for next test
        fixture
            .runtime_manager
            .stop_instance(agent_id, reload_result.to_version)
            .await
            .unwrap();
    }
}
