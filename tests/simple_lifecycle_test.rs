//! Simple Agent Lifecycle Test
//!
//! A minimal test to validate the basic integration works

use std::sync::Arc;
use std::time::Duration;
use test_log::test;

#[allow(unused_imports)]
use caxton::domain_types::{AgentId, AgentName, CpuFuel, MemoryBytes};
#[allow(unused_imports)]
use caxton::{
    AgentLifecycleManager, AgentVersion, DeploymentConfig, DeploymentError, DeploymentId,
    DeploymentManagerTrait, DeploymentRequest, DeploymentResult, DeploymentStatus,
    DeploymentStrategy, HotReloadConfig, HotReloadError, HotReloadId, HotReloadManagerTrait,
    HotReloadRequest, HotReloadResult, HotReloadStatus, HotReloadStrategy, ResourceRequirements,
    TrafficSplitPercentage, ValidationResult, VersionNumber, WasmModule, WasmModuleValidatorTrait,
    WasmSecurityPolicy, WasmValidationError,
};

// Simple mock implementations
struct SimpleMockDeploymentManager;

#[async_trait::async_trait]
impl DeploymentManagerTrait for SimpleMockDeploymentManager {
    async fn deploy_agent(
        &self,
        request: DeploymentRequest,
    ) -> std::result::Result<DeploymentResult, DeploymentError> {
        Ok(DeploymentResult::success(
            request.deployment_id,
            request.agent_id,
            std::time::SystemTime::now(),
            std::time::SystemTime::now(),
            None,
        ))
    }

    async fn get_deployment_status(
        &self,
        _deployment_id: DeploymentId,
    ) -> std::result::Result<DeploymentStatus, DeploymentError> {
        Ok(DeploymentStatus::Completed)
    }

    async fn cancel_deployment(
        &self,
        _deployment_id: DeploymentId,
    ) -> std::result::Result<(), DeploymentError> {
        Ok(())
    }

    async fn rollback_deployment(
        &self,
        deployment_id: DeploymentId,
        target_version: AgentVersion,
    ) -> std::result::Result<DeploymentResult, DeploymentError> {
        Ok(DeploymentResult::failure(
            deployment_id,
            AgentId::generate(),
            Some(std::time::SystemTime::now()),
            format!("Rolled back to version {}", target_version),
            Some(target_version),
        ))
    }

    async fn cleanup_agent(&self, _agent_id: AgentId) -> std::result::Result<(), DeploymentError> {
        // Mock implementation - always succeeds
        Ok(())
    }
}

struct SimpleMockHotReloadManager;

#[async_trait::async_trait]
impl HotReloadManagerTrait for SimpleMockHotReloadManager {
    async fn hot_reload_agent(
        &self,
        request: HotReloadRequest,
    ) -> std::result::Result<HotReloadResult, HotReloadError> {
        Ok(HotReloadResult::success(
            request.reload_id,
            request.agent_id,
            request.from_version,
            request.to_version,
            std::time::SystemTime::now(),
            None,
            vec![],
        ))
    }

    async fn get_hot_reload_status(
        &self,
        _reload_id: HotReloadId,
    ) -> std::result::Result<HotReloadStatus, HotReloadError> {
        Ok(HotReloadStatus::Completed)
    }

    async fn cancel_hot_reload(
        &self,
        _reload_id: HotReloadId,
    ) -> std::result::Result<(), HotReloadError> {
        Ok(())
    }

    async fn rollback_hot_reload(
        &self,
        reload_id: HotReloadId,
        target_version: AgentVersion,
    ) -> std::result::Result<HotReloadResult, HotReloadError> {
        Ok(HotReloadResult::rollback(
            reload_id,
            AgentId::generate(),
            AgentVersion::generate(),
            AgentVersion::generate(),
            Some(std::time::SystemTime::now()),
            format!("Rolled back to version {}", target_version),
            None,
        ))
    }
}

struct SimpleMockWasmModuleValidator;

#[async_trait::async_trait]
impl WasmModuleValidatorTrait for SimpleMockWasmModuleValidator {
    async fn validate_module(
        &self,
        wasm_bytes: &[u8],
        agent_name: Option<AgentName>,
    ) -> std::result::Result<WasmModule, WasmValidationError> {
        if wasm_bytes.is_empty() {
            return Err(WasmValidationError::EmptyModule);
        }

        WasmModule::from_bytes(
            AgentVersion::generate(),
            VersionNumber::first(),
            None,
            agent_name,
            wasm_bytes,
            WasmSecurityPolicy::testing(),
        )
    }

    async fn validate_security(
        &self,
        _module: &WasmModule,
    ) -> std::result::Result<ValidationResult, WasmValidationError> {
        Ok(ValidationResult::Valid)
    }

    async fn extract_metadata(
        &self,
        _wasm_bytes: &[u8],
    ) -> std::result::Result<std::collections::HashMap<String, String>, WasmValidationError> {
        Ok(std::collections::HashMap::new())
    }
}

#[test(tokio::test)]
async fn test_simple_agent_deployment() {
    let deployment_manager = Arc::new(SimpleMockDeploymentManager);
    let hot_reload_manager = Arc::new(SimpleMockHotReloadManager);
    let module_validator = Arc::new(SimpleMockWasmModuleValidator);

    let lifecycle_manager =
        AgentLifecycleManager::new(deployment_manager, hot_reload_manager, module_validator);

    let agent_id = AgentId::generate();
    let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());
    let version = AgentVersion::generate();
    let version_number = VersionNumber::first();
    let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
    let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // Valid WASM header

    let result = lifecycle_manager
        .deploy_agent(
            agent_id,
            agent_name,
            version,
            version_number,
            config,
            wasm_bytes,
        )
        .await;

    assert!(result.is_ok());
    let deployment_result = result.unwrap();
    assert!(deployment_result.status.is_success());
}

#[test(tokio::test)]
async fn test_simple_agent_lifecycle_transitions() {
    let deployment_manager = Arc::new(SimpleMockDeploymentManager);
    let hot_reload_manager = Arc::new(SimpleMockHotReloadManager);
    let module_validator = Arc::new(SimpleMockWasmModuleValidator);

    let lifecycle_manager =
        AgentLifecycleManager::new(deployment_manager, hot_reload_manager, module_validator);

    let agent_id = AgentId::generate();
    let agent_name = Some(AgentName::try_new("lifecycle-test-agent".to_string()).unwrap());
    let version = AgentVersion::generate();
    let version_number = VersionNumber::first();
    let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
    let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // Valid WASM header

    // Deploy agent
    lifecycle_manager
        .deploy_agent(
            agent_id,
            agent_name,
            version,
            version_number,
            config,
            wasm_bytes,
        )
        .await
        .unwrap();

    // Start agent
    let start_result = lifecycle_manager.start_agent(agent_id).await.unwrap();
    assert!(start_result.success);

    // Stop agent
    let stop_result = lifecycle_manager
        .stop_agent(agent_id, Some(Duration::from_millis(100)))
        .await
        .unwrap();
    assert!(stop_result.success);

    // Remove agent
    let remove_result = lifecycle_manager.remove_agent(agent_id).await.unwrap();
    assert!(remove_result.success);
}

#[test(tokio::test)]
async fn test_simple_hot_reload() {
    let deployment_manager = Arc::new(SimpleMockDeploymentManager);
    let hot_reload_manager = Arc::new(SimpleMockHotReloadManager);
    let module_validator = Arc::new(SimpleMockWasmModuleValidator);

    let lifecycle_manager =
        AgentLifecycleManager::new(deployment_manager, hot_reload_manager, module_validator);

    let agent_id = AgentId::generate();
    let agent_name = Some(AgentName::try_new("hotreload-test-agent".to_string()).unwrap());
    let from_version = AgentVersion::generate();
    let from_version_number = VersionNumber::first();
    let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
    let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // Valid WASM header

    // Deploy initial version
    lifecycle_manager
        .deploy_agent(
            agent_id,
            agent_name,
            from_version,
            from_version_number,
            config,
            wasm_bytes,
        )
        .await
        .unwrap();
    lifecycle_manager.start_agent(agent_id).await.unwrap();

    // Hot reload to new version
    let to_version = AgentVersion::generate();
    let to_version_number = from_version_number.next().unwrap();
    let reload_config = HotReloadConfig::new(HotReloadStrategy::Graceful);
    let new_wasm_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04]; // Valid WASM

    let result = lifecycle_manager
        .hot_reload_agent(
            agent_id,
            from_version,
            to_version,
            to_version_number,
            reload_config,
            new_wasm_bytes,
        )
        .await;

    assert!(result.is_ok());
    let reload_result = result.unwrap();
    assert!(reload_result.status.is_success());

    // Verify version was updated
    let lifecycle = lifecycle_manager
        .get_agent_lifecycle(agent_id)
        .await
        .unwrap();
    assert_eq!(lifecycle.version, to_version);
    assert_eq!(lifecycle.version_number, to_version_number);
}

#[test(tokio::test)]
async fn test_deployment_performance_requirement() {
    let deployment_manager = Arc::new(SimpleMockDeploymentManager);
    let hot_reload_manager = Arc::new(SimpleMockHotReloadManager);
    let module_validator = Arc::new(SimpleMockWasmModuleValidator);

    let lifecycle_manager =
        AgentLifecycleManager::new(deployment_manager, hot_reload_manager, module_validator);

    let agent_id = AgentId::generate();
    let agent_name = Some(AgentName::try_new("perf-test-agent".to_string()).unwrap());
    let version = AgentVersion::generate();
    let version_number = VersionNumber::first();
    let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
    let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // Valid WASM header

    let start_time = std::time::Instant::now();
    let result = lifecycle_manager
        .deploy_agent(
            agent_id,
            agent_name,
            version,
            version_number,
            config,
            wasm_bytes,
        )
        .await;
    let elapsed = start_time.elapsed();

    assert!(result.is_ok());
    // Story 003 requirement: deployment should complete in < 1 second
    assert!(elapsed < Duration::from_secs(1));
}

#[test(tokio::test)]
async fn test_validation_failure_handling() {
    let deployment_manager = Arc::new(SimpleMockDeploymentManager);
    let hot_reload_manager = Arc::new(SimpleMockHotReloadManager);
    let module_validator = Arc::new(SimpleMockWasmModuleValidator);

    let lifecycle_manager =
        AgentLifecycleManager::new(deployment_manager, hot_reload_manager, module_validator);

    let agent_id = AgentId::generate();
    let agent_name = Some(AgentName::try_new("validation-test-agent".to_string()).unwrap());
    let version = AgentVersion::generate();
    let version_number = VersionNumber::first();
    let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
    let empty_wasm = vec![]; // Empty WASM should fail validation

    let result = lifecycle_manager
        .deploy_agent(
            agent_id,
            agent_name,
            version,
            version_number,
            config,
            empty_wasm,
        )
        .await;

    assert!(result.is_err());
    // Should not be tracked if validation fails
    assert!(lifecycle_manager.get_agent_status(agent_id).await.is_err());
}
