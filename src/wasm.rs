//! WebAssembly integration and isolation
//!
//! Provides secure, isolated execution environment for agents using WebAssembly.
//! Implements type-driven approach with phantom types for compile-time safety.

use crate::core::wasm::runtime::{WasmRuntimeConfig, WasmRuntimeEngine};
use crate::performance::wasm_runtime::{OptimizedWasmRuntime, WasmRuntimeStats};
use crate::performance::PerformanceMonitor;
use crate::*;
use ::tracing::{debug, error, info, instrument, warn};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use wasmtime::{AsContextMut, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::p2::{WasiCtx, WasiCtxBuilder};

/// Agent lifecycle states with phantom types for compile-time guarantees
pub struct Unloaded;
pub struct Loaded;
pub struct Running;
pub struct Terminated;

/// WASM agent configuration
#[derive(Debug, Clone)]
pub struct WasmAgentConfig {
    pub name: String,
    pub wasm_module: Vec<u8>,
    pub max_memory_pages: u32,
    pub max_execution_time: Duration,
    pub capabilities: Vec<String>,
}

/// Type-safe WASM agent with phantom state tracking
#[derive(Debug)]
pub struct WasmAgent<State> {
    pub id: AgentId,
    config: WasmAgentConfig,
    _state: PhantomData<State>,
}

/// WASM value types for type-safe parameter passing
#[derive(Debug, Clone)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl WasmValue {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            WasmValue::I32(v) => serde_json::json!({"type": "i32", "value": v}),
            WasmValue::I64(v) => serde_json::json!({"type": "i64", "value": v}),
            WasmValue::F32(v) => serde_json::json!({"type": "f32", "value": v}),
            WasmValue::F64(v) => serde_json::json!({"type": "f64", "value": v}),
        }
    }
}

/// WASM execution result with error details
#[derive(Debug, Clone)]
pub struct WasmExecutionResult {
    pub success: bool,
    pub return_values: Vec<WasmValue>,
    pub execution_time: Duration,
    pub memory_used: usize,
    pub error: Option<String>,
}

/// Core WASM runtime for agent execution
pub struct WasmRuntime {
    optimized_runtime: Arc<OptimizedWasmRuntime>,
    core_engine: Arc<WasmRuntimeEngine>,
    agents: Arc<RwLock<std::collections::HashMap<AgentId, WasmAgentMetadata>>>,
    performance_monitor: Arc<PerformanceMonitor>,
}

/// Metadata for active WASM agents
#[derive(Debug, Clone)]
struct WasmAgentMetadata {
    config: WasmAgentConfig,
    module_name: String,
    agent_type: String,
    created_at: std::time::SystemTime,
    execution_count: u64,
}

impl WasmRuntime {
    /// Create a new WASM runtime with optimized configuration
    #[instrument]
    pub async fn new() -> Result<Self, CaxtonError> {
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        let optimized_runtime = Arc::new(
            OptimizedWasmRuntime::new(performance_monitor.clone())
                .map_err(|e| CaxtonError::WasmRuntimeError(e.to_string()))?,
        );

        // Initialize core engine with secure configuration
        let core_config = WasmRuntimeConfig::default();
        let core_engine = Arc::new(
            WasmRuntimeEngine::new(core_config)
                .map_err(|e| CaxtonError::WasmRuntimeError(e.to_string()))?,
        );

        info!("WASM runtime initialized with optimized configuration and core engine");

        Ok(Self {
            optimized_runtime,
            core_engine,
            agents: Arc::new(RwLock::new(std::collections::HashMap::new())),
            performance_monitor,
        })
    }

    /// Load a WASM agent from unloaded to loaded state
    #[instrument(skip(self, agent))]
    pub async fn load_agent(
        &self,
        agent: WasmAgent<Unloaded>,
    ) -> Result<WasmAgent<Loaded>, CaxtonError> {
        let module_name = format!("agent_{}", agent.id);

        // Use the core engine to load and validate the agent with security controls
        let loaded_agent = self.core_engine.load_agent(agent).await?;

        // Also load in optimized runtime for performance
        let _module = self
            .optimized_runtime
            .load_module(&module_name, &loaded_agent.config.wasm_module)
            .await
            .map_err(|e| CaxtonError::WasmRuntimeError(e.to_string()))?;

        // Store agent metadata for tracking
        let metadata = WasmAgentMetadata {
            config: loaded_agent.config.clone(),
            module_name: module_name.clone(),
            agent_type: loaded_agent.config.name.clone(),
            created_at: std::time::SystemTime::now(),
            execution_count: 0,
        };

        {
            let mut agents = self.agents.write().await;
            agents.insert(loaded_agent.id, metadata);
        }

        info!(agent_id = %loaded_agent.id, module_name = %module_name, "WASM agent loaded successfully in both engines");

        Ok(loaded_agent)
    }

    /// Execute a function in a loaded WASM agent using parameters as Vec<wasmtime::Val>
    #[instrument(skip(self, agent))]
    pub async fn execute_agent(
        &self,
        agent: &WasmAgent<Loaded>,
        function_name: &str,
        params: Vec<wasmtime::Val>,
    ) -> Result<WasmExecutionResult, CaxtonError> {
        let start_time = std::time::Instant::now();

        // Get agent metadata
        let metadata = {
            let agents = self.agents.read().await;
            agents
                .get(&agent.id)
                .ok_or_else(|| CaxtonError::AgentNotFound(agent.id))?
                .clone()
        };

        // Use the core engine to execute with proper resource limits and security
        let result = self
            .core_engine
            .execute_agent(agent, function_name, params)
            .await?;

        // Update execution count in our metadata
        {
            let mut agents = self.agents.write().await;
            if let Some(agent_meta) = agents.get_mut(&agent.id) {
                agent_meta.execution_count += 1;
            }
        }

        let execution_time = start_time.elapsed();

        debug!(
            agent_id = %agent.id,
            function = function_name,
            execution_time_ms = execution_time.as_millis(),
            success = result.success,
            "WASM function execution completed"
        );

        Ok(result)
    }

    /// Execute a function with typed parameters (convenience method)
    #[instrument(skip(self, agent))]
    pub async fn execute_agent_typed<P, R>(
        &self,
        agent: &WasmAgent<Loaded>,
        function_name: &str,
        params: P,
    ) -> Result<WasmExecutionResult, CaxtonError>
    where
        P: wasmtime::WasmParams + Send,
        R: wasmtime::WasmResults,
    {
        // Convert typed parameters to Vec<wasmtime::Val>
        // This is a simplified conversion - in practice you'd need proper parameter conversion
        let vals = vec![]; // TODO: Implement proper parameter conversion

        self.execute_agent(agent, function_name, vals).await
    }

    /// Get runtime performance statistics
    #[instrument(skip(self))]
    pub async fn get_performance_stats(&self) -> WasmRuntimeStats {
        self.optimized_runtime.get_performance_stats().await
    }

    /// Get agent execution metrics
    #[instrument(skip(self))]
    pub async fn get_agent_metrics(&self, agent_id: &AgentId) -> Option<WasmAgentMetadata> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Terminate an agent and clean up resources
    #[instrument(skip(self, agent))]
    pub async fn terminate_agent(
        &self,
        agent: WasmAgent<Loaded>,
    ) -> Result<WasmAgent<Terminated>, CaxtonError> {
        // Use core engine to properly terminate with resource cleanup
        let terminated_agent = self.core_engine.terminate_agent(agent).await?;

        // Remove agent metadata from our tracking
        {
            let mut agents = self.agents.write().await;
            agents.remove(&terminated_agent.id);
        }

        info!(agent_id = %terminated_agent.id, "WASM agent terminated and resources cleaned up");

        Ok(terminated_agent)
    }
}

/// Factory methods for creating agents in different states
impl WasmAgent<Unloaded> {
    /// Create a new unloaded WASM agent
    pub fn new(config: WasmAgentConfig) -> Self {
        Self {
            id: AgentId::new(),
            config,
            _state: PhantomData,
        }
    }
}

impl WasmAgent<Loaded> {
    /// Transition to running state (conceptual, for future state management)
    pub fn start_running(self) -> WasmAgent<Running> {
        WasmAgent {
            id: self.id,
            config: self.config,
            _state: PhantomData,
        }
    }
}

impl WasmAgent<Running> {
    /// Stop running and return to loaded state
    pub fn stop_running(self) -> WasmAgent<Loaded> {
        WasmAgent {
            id: self.id,
            config: self.config,
            _state: PhantomData,
        }
    }
}
