//! Integration tests for Agent Lifecycle Management domain types
//!
//! This test demonstrates the complete lifecycle of an agent using the new domain types.

use caxton::domain::{HashAlgorithm, ResourceRequirements, TrafficSplitPercentage};
use caxton::{
    AgentId, AgentLifecycle, AgentLifecycleState, AgentName, AgentVersion, DeploymentConfig,
    DeploymentRequest, DeploymentStrategy, HotReloadConfig, HotReloadRequest, HotReloadStrategy,
    VersionNumber, WasmModule, WasmSecurityPolicy,
};

#[test]
fn test_complete_agent_lifecycle_integration() {
    // Create agent identifiers
    let agent_id = AgentId::generate();
    let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());
    let version = AgentVersion::generate();
    let version_number = VersionNumber::first();

    // Create new agent lifecycle
    let mut lifecycle = AgentLifecycle::new(agent_id, agent_name.clone(), version, version_number);

    // Verify initial state
    assert_eq!(lifecycle.current_state, AgentLifecycleState::Unloaded);
    assert!(lifecycle.previous_state.is_none());
    assert!(!lifecycle.pending_requests.has_pending());

    // Test state transitions
    assert!(
        lifecycle
            .transition_to(AgentLifecycleState::Loaded, None)
            .is_ok()
    );
    assert_eq!(lifecycle.current_state, AgentLifecycleState::Loaded);
    assert_eq!(
        lifecycle.previous_state,
        Some(AgentLifecycleState::Unloaded)
    );

    assert!(
        lifecycle
            .transition_to(AgentLifecycleState::Ready, None)
            .is_ok()
    );
    assert_eq!(lifecycle.current_state, AgentLifecycleState::Ready);

    // Test starting agent
    assert!(lifecycle.start().is_ok());
    assert_eq!(lifecycle.current_state, AgentLifecycleState::Running);
    assert!(lifecycle.current_state.is_active());

    // Test draining
    assert!(lifecycle.start_draining().is_ok());
    assert_eq!(lifecycle.current_state, AgentLifecycleState::Draining);

    // Test stopping
    assert!(lifecycle.stop().is_ok());
    assert_eq!(lifecycle.current_state, AgentLifecycleState::Stopped);
    assert!(lifecycle.current_state.is_terminal());
}

#[test]
fn test_deployment_request_creation() {
    let agent_id = AgentId::generate();
    let from_version = None; // Initial deployment
    let to_version = AgentVersion::generate();
    let version_number = VersionNumber::first();
    let config = DeploymentConfig::new(DeploymentStrategy::Rolling);
    let wasm_bytes = b"fake wasm module content".to_vec();

    let deployment_request = DeploymentRequest::new(
        agent_id,
        None,
        from_version,
        to_version,
        version_number,
        config,
        wasm_bytes,
    );

    // Verify deployment request properties
    assert!(deployment_request.is_initial_deployment());
    assert!(!deployment_request.is_upgrade());
    assert_eq!(deployment_request.module_size(), 24);
    assert!(deployment_request.validate().is_ok());
}

#[test]
fn test_hot_reload_request_creation() {
    let agent_id = AgentId::generate();
    let from_version = AgentVersion::generate();
    let to_version = AgentVersion::generate();
    let version_number = VersionNumber::first();
    let config = HotReloadConfig::graceful();
    let wasm_bytes = b"new wasm module content".to_vec();

    let reload_request = HotReloadRequest::new(
        agent_id,
        None,
        from_version,
        to_version,
        version_number,
        config,
        wasm_bytes,
    );

    // Verify hot reload request properties
    assert_ne!(reload_request.from_version, reload_request.to_version);
    assert_eq!(reload_request.module_size(), 23);
    assert!(reload_request.validate().is_ok());
    assert_eq!(reload_request.config.strategy, HotReloadStrategy::Graceful);
}

#[test]
fn test_wasm_module_creation_and_validation() {
    let version = AgentVersion::generate();
    let version_number = VersionNumber::first();
    let wasm_bytes = b"fake wasm module for testing";
    let security_policy = WasmSecurityPolicy::testing();

    let wasm_module = WasmModule::from_bytes(
        version,
        version_number,
        None,
        None,
        wasm_bytes,
        &security_policy,
    )
    .unwrap();

    // Verify module properties
    assert!(wasm_module.is_valid());
    assert_eq!(wasm_module.size.as_bytes(), wasm_bytes.len());
    assert_eq!(wasm_module.hash.algorithm(), HashAlgorithm::Sha256);
    assert_eq!(wasm_module.total_function_count(), 0); // No functions in fake module
}

#[test]
fn test_resource_requirements_compatibility() {
    let requirements = ResourceRequirements::minimal();

    // Test compatibility with sufficient resources
    assert!(requirements.is_compatible_with(10_000_000, 100_000));

    // Test incompatibility with insufficient resources
    assert!(!requirements.is_compatible_with(100, 100_000)); // Insufficient memory
    assert!(!requirements.is_compatible_with(10_000_000, 100)); // Insufficient fuel
}

#[test]
fn test_domain_type_integration() {
    // Test that all domain types work together harmoniously
    let _agent_id = AgentId::generate();
    let agent_name = AgentName::try_new("integration-test-agent".to_string()).unwrap();
    let version = AgentVersion::generate();
    let version_number = VersionNumber::first();

    // Create deployment configuration
    let deployment_config = DeploymentConfig::canary();
    assert_eq!(deployment_config.strategy, DeploymentStrategy::Canary);
    assert!(deployment_config.health_check.enabled);

    // Create hot reload configuration
    let hot_reload_config =
        HotReloadConfig::traffic_splitting(TrafficSplitPercentage::try_new(20).unwrap());
    assert_eq!(
        hot_reload_config.strategy,
        HotReloadStrategy::TrafficSplitting
    );
    assert_eq!(hot_reload_config.traffic_split.as_percentage(), 20);

    // Create WASM module with testing security policy (more permissive)
    let wasm_bytes = b"integration test wasm module";
    let testing_policy = WasmSecurityPolicy::testing();
    let wasm_module = WasmModule::from_bytes(
        version,
        version_number,
        None,
        Some(agent_name),
        wasm_bytes,
        &testing_policy,
    )
    .unwrap();

    // Verify everything integrates correctly
    assert!(wasm_module.is_valid());
    assert_eq!(wasm_module.security_policy.name, "testing");
    assert!(deployment_config.auto_rollback);
    assert!(hot_reload_config.enable_metrics_collection);
}
