//! Integration tests for the Caxton WebAssembly runtime

use anyhow::Result;
use caxton::{CpuFuel, ExecutionTime, MemoryBytes, MessageSize};
use caxton::{ResourceLimits, SecurityPolicy, WasmRuntime, WasmRuntimeConfig};
use std::time::Duration;

#[tokio::test]
async fn test_runtime_initialization() -> Result<()> {
    let config = WasmRuntimeConfig::default();
    let runtime = WasmRuntime::new(config)?;

    assert!(runtime.is_initialized());
    assert_eq!(runtime.active_agent_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_agent_sandbox_isolation() -> Result<()> {
    let config = WasmRuntimeConfig::default();
    let mut runtime = WasmRuntime::new(config)?;

    let wasm_module = include_bytes!("../tests/fixtures/test_agent.wasm");

    let agent1_id = runtime.deploy_agent("agent1", wasm_module).await?;
    let agent2_id = runtime.deploy_agent("agent2", wasm_module).await?;

    assert_ne!(agent1_id, agent2_id);
    assert_eq!(runtime.active_agent_count(), 2);

    let agent1_memory = runtime.get_agent_memory_usage(agent1_id)?;
    let agent2_memory = runtime.get_agent_memory_usage(agent2_id)?;

    assert!(agent1_memory.as_usize() > 0);
    assert!(agent2_memory.as_usize() > 0);

    Ok(())
}

#[tokio::test]
async fn test_memory_limits_enforced() -> Result<()> {
    let config = WasmRuntimeConfig {
        resource_limits: ResourceLimits {
            max_memory_bytes: MemoryBytes::try_new(1024 * 1024).unwrap(), // 1MB
            max_cpu_fuel: CpuFuel::try_new(1_000_000).unwrap(),
            max_execution_time: ExecutionTime::from_secs(1),
            max_message_size: MessageSize::try_new(1024 * 10).unwrap(), // 10KB
        },
        ..Default::default()
    };

    let mut runtime = WasmRuntime::new(config)?;
    let wasm_module = include_bytes!("../tests/fixtures/memory_hog.wasm");

    let result = runtime.deploy_agent("memory_hog", wasm_module).await;

    match result {
        Ok(agent_id) => {
            let exec_result = runtime
                .execute_agent(agent_id, "allocate_memory", &[])
                .await;
            assert!(exec_result.is_err());

            let err = exec_result.unwrap_err();
            assert!(err.to_string().contains("memory limit exceeded"));
        }
        Err(e) => {
            assert!(e.to_string().contains("memory") || e.to_string().contains("resource"));
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_cpu_limits_with_fuel() -> Result<()> {
    let config = WasmRuntimeConfig {
        resource_limits: ResourceLimits {
            max_memory_bytes: MemoryBytes::from_mb(10).unwrap(),
            max_cpu_fuel: CpuFuel::try_new(1000).unwrap(), // Very low fuel to trigger limit
            max_execution_time: ExecutionTime::from_secs(10),
            max_message_size: MessageSize::try_new(1024 * 10).unwrap(),
        },
        ..Default::default()
    };

    let mut runtime = WasmRuntime::new(config)?;
    let wasm_module = include_bytes!("../tests/fixtures/infinite_loop.wasm");

    let agent_id = runtime.deploy_agent("cpu_hog", wasm_module).await?;
    runtime.start_agent(agent_id)?;
    let result = runtime.execute_agent(agent_id, "infinite_loop", &[]).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("fuel") || err.to_string().contains("CPU limit"));

    Ok(())
}

#[tokio::test]
async fn test_wasm_module_loading() -> Result<()> {
    let config = WasmRuntimeConfig::default();
    let mut runtime = WasmRuntime::new(config)?;

    let valid_wasm = include_bytes!("../tests/fixtures/valid_agent.wasm");
    let invalid_wasm = b"not a valid wasm module";

    let valid_result = runtime.deploy_agent("valid", valid_wasm).await;
    assert!(valid_result.is_ok());

    let invalid_result = runtime.deploy_agent("invalid", invalid_wasm).await;
    assert!(invalid_result.is_err());
    assert!(invalid_result.unwrap_err().to_string().contains("invalid"));

    Ok(())
}

#[tokio::test]
async fn test_host_function_exposure() -> Result<()> {
    let config = WasmRuntimeConfig::default();
    let mut runtime = WasmRuntime::new(config)?;

    let wasm_module = include_bytes!("../tests/fixtures/host_function_test.wasm");
    let agent_id = runtime.deploy_agent("host_test", wasm_module).await?;

    let allowed_functions = runtime.get_exposed_host_functions(agent_id)?;

    assert!(allowed_functions.contains(&"log".to_string()));
    assert!(allowed_functions.contains(&"get_time".to_string()));
    assert!(allowed_functions.contains(&"send_message".to_string()));

    assert!(!allowed_functions.contains(&"file_system_access".to_string()));
    assert!(!allowed_functions.contains(&"network_raw_socket".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_security_features_disabled() -> Result<()> {
    let config = WasmRuntimeConfig {
        security_policy: SecurityPolicy::strict(),
        ..Default::default()
    };

    let runtime = WasmRuntime::new(config)?;
    let policy = runtime.get_security_policy();

    assert!(policy.disable_simd);
    assert!(policy.disable_reference_types);
    assert!(policy.disable_bulk_memory);
    assert!(policy.disable_threads);
    assert!(policy.enable_fuel_metering);

    Ok(())
}

#[tokio::test]
async fn test_agent_startup_performance() -> Result<()> {
    let config = WasmRuntimeConfig::default();
    let mut runtime = WasmRuntime::new(config)?;

    let wasm_module = include_bytes!("../tests/fixtures/minimal_agent.wasm");

    let start = std::time::Instant::now();
    let agent_id = runtime.deploy_agent("perf_test", wasm_module).await?;
    runtime.start_agent(agent_id)?;
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(100),
        "Agent startup took {duration:?}, expected < 100ms"
    );

    Ok(())
}

#[tokio::test]
async fn test_cooperative_scheduling_with_fuel() -> Result<()> {
    let config = WasmRuntimeConfig {
        resource_limits: ResourceLimits {
            max_cpu_fuel: CpuFuel::try_new(10000).unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut runtime = WasmRuntime::new(config)?;
    let wasm_module = include_bytes!("../tests/fixtures/cooperative_agent.wasm");

    let agent_id = runtime.deploy_agent("cooperative", wasm_module).await?;

    let result = runtime
        .execute_agent_with_fuel_tracking(agent_id, "long_computation", &[])
        .await?;

    assert!(result.fuel_consumed > 0);
    assert!(result.fuel_consumed <= 10000);
    assert!(result.completed_successfully);

    Ok(())
}

#[tokio::test]
async fn test_multiple_agent_resource_isolation() -> Result<()> {
    let config = WasmRuntimeConfig {
        resource_limits: ResourceLimits {
            max_memory_bytes: MemoryBytes::from_mb(5).unwrap(), // 5MB per agent
            max_cpu_fuel: CpuFuel::try_new(100_000).unwrap(),
            max_execution_time: ExecutionTime::from_secs(5),
            max_message_size: MessageSize::try_new(1024 * 10).unwrap(),
        },
        ..Default::default()
    };

    let mut runtime = WasmRuntime::new(config)?;
    let wasm_module = include_bytes!("../tests/fixtures/resource_test.wasm");

    let mut agent_ids = vec![];
    for i in 0..5 {
        let agent_id = runtime
            .deploy_agent(&format!("agent_{i}"), wasm_module)
            .await?;
        agent_ids.push(agent_id);
    }

    for agent_id in &agent_ids {
        let memory = runtime.get_agent_memory_usage(*agent_id)?;
        assert!(memory.as_usize() <= 5 * 1024 * 1024);

        let cpu_usage = runtime.get_agent_cpu_usage(*agent_id)?;
        assert!(cpu_usage.as_u64() <= 100_000);
    }

    assert_eq!(runtime.active_agent_count(), 5);

    Ok(())
}
