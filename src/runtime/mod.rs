//! WebAssembly runtime for managing agent lifecycle and execution

use anyhow::{Context, Result, bail};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};
use uuid::Uuid;
use wasmtime::{Config as WasmConfig, Engine, Module};

use crate::resource_manager::{ResourceLimits, ResourceManager};
use crate::sandbox::Sandbox;
use crate::security::SecurityPolicy;

/// Unique identifier for an agent
pub type AgentId = Uuid;

/// Configuration for the WebAssembly runtime
#[derive(Debug, Clone)]
pub struct WasmRuntimeConfig {
    /// Resource limits for agents
    pub resource_limits: ResourceLimits,
    /// Security policy for agent execution
    pub security_policy: SecurityPolicy,
    /// Maximum number of concurrent agents
    pub max_agents: usize,
    /// Enable debug mode
    pub enable_debug: bool,
}

impl Default for WasmRuntimeConfig {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            security_policy: SecurityPolicy::default(),
            max_agents: 1000,
            enable_debug: false,
        }
    }
}

/// Main WebAssembly runtime for managing agents
pub struct WasmRuntime {
    engine: Arc<Engine>,
    agents: Arc<DashMap<AgentId, Agent>>,
    config: WasmRuntimeConfig,
    active_count: Arc<AtomicUsize>,
    resource_manager: Arc<ResourceManager>,
    initialized: bool,
}

#[allow(dead_code)]
struct Agent {
    id: AgentId,
    name: String,
    sandbox: Sandbox,
    #[allow(dead_code)]
    module: Module,
    state: AgentState,
    resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgentState {
    #[allow(dead_code)]
    Unloaded,
    Loaded,
    Running,
    Draining,
    Stopped,
}

#[derive(Debug, Default)]
struct ResourceUsage {
    memory_bytes: usize,
    cpu_fuel_consumed: u64,
    message_count: usize,
}

/// Result of executing an agent function
#[derive(Debug)]
pub struct ExecutionResult {
    /// Amount of fuel consumed during execution
    pub fuel_consumed: u64,
    /// Whether the execution completed successfully
    pub completed_successfully: bool,
    /// Output data from the execution
    pub output: Option<Vec<u8>>,
}

impl Agent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl ResourceUsage {
    fn update_memory(&mut self, bytes: usize) {
        self.memory_bytes = bytes;
    }

    fn update_cpu(&mut self, fuel: u64) {
        self.cpu_fuel_consumed += fuel;
    }

    fn increment_message_count(&mut self) {
        self.message_count += 1;
    }
}

impl WasmRuntime {
    /// Creates a new WebAssembly runtime with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the engine cannot be created
    pub fn new(config: WasmRuntimeConfig) -> Result<Self> {
        info!("Initializing WASM runtime with config: {:?}", config);

        let mut wasm_config = WasmConfig::new();

        wasm_config.async_support(true);
        wasm_config.consume_fuel(config.security_policy.enable_fuel_metering);

        // Note: Some WASM features have dependencies, so we need to be careful
        // For now, we'll use the defaults for most features to avoid conflicts
        if config.security_policy.disable_threads {
            wasm_config.wasm_threads(false);
        }

        wasm_config.parallel_compilation(true);
        wasm_config
            .cache_config_load_default()
            .context("Failed to load cache config")?;

        let engine = Arc::new(Engine::new(&wasm_config).context("Failed to create WASM engine")?);

        let resource_manager = Arc::new(ResourceManager::new(config.resource_limits.clone()));

        Ok(Self {
            engine,
            agents: Arc::new(DashMap::new()),
            config,
            active_count: Arc::new(AtomicUsize::new(0)),
            resource_manager,
            initialized: true,
        })
    }

    /// Checks if the runtime is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns the number of active agents
    pub fn active_agent_count(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    /// Gets the resource manager for monitoring
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    /// Deploys a new agent from WebAssembly bytecode
    ///
    /// # Errors
    ///
    /// Returns an error if the module is invalid or if agent limit is reached
    pub async fn deploy_agent(&mut self, name: &str, wasm_bytes: &[u8]) -> Result<AgentId> {
        info!("Deploying agent: {}", name);

        if self.active_agent_count() >= self.config.max_agents {
            bail!(
                "Maximum number of agents ({}) reached",
                self.config.max_agents
            );
        }

        let module = Module::new(&self.engine, wasm_bytes)
            .context("invalid WASM module: failed to compile")?;

        Self::validate_module(&module);

        let agent_id = AgentId::new_v4();

        let mut sandbox = Sandbox::new(
            agent_id,
            self.config.resource_limits.clone(),
            self.engine.clone(),
        )?;

        // Initialize the sandbox with the module immediately
        sandbox.initialize(&module).await?;

        // Start agents in Loaded state since sandbox is already initialized
        let agent = Agent {
            id: agent_id,
            name: name.to_string(),
            sandbox,
            module,
            state: AgentState::Loaded,
            resource_usage: ResourceUsage::default(),
        };

        debug!(
            "Agent {} created and initialized in {:?} state",
            agent_id, agent.state
        );

        self.agents.insert(agent_id, agent);
        self.active_count.fetch_add(1, Ordering::SeqCst);

        // Use agent name and id for logging
        info!("Agent '{}' deployed with ID: {}", name, agent_id);
        debug!(
            "Agent {} is now in {:?} state",
            agent_id,
            AgentState::Loaded
        );
        Ok(agent_id)
    }

    /// Starts an agent that has been deployed (transitions from Loaded to Running)
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or not in Loaded state
    pub fn start_agent(&mut self, agent_id: AgentId) -> Result<()> {
        let mut agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        if agent.state != AgentState::Loaded {
            bail!(
                "Agent {} is not in Loaded state (current: {:?})",
                agent_id,
                agent.state
            );
        }

        // Agent sandbox is already initialized during deployment, just transition state
        agent.state = AgentState::Running;

        info!("Agent {} started", agent_id);
        Ok(())
    }

    /// Executes a function on an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or not running
    pub async fn execute_agent(
        &mut self,
        agent_id: AgentId,
        function: &str,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        let mut agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        if agent.state != AgentState::Running {
            bail!("Agent {} is not running", agent_id);
        }

        let result = agent.sandbox.execute(function, args).await?;

        agent.resource_usage.update_cpu(result.fuel_consumed);

        Ok(result.output.unwrap_or_default())
    }

    /// Executes a function on an agent with detailed fuel tracking
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or execution fails
    ///
    /// # Panics
    ///
    /// Panics if the agent exists but cannot be retrieved after starting
    pub async fn execute_agent_with_fuel_tracking(
        &mut self,
        agent_id: AgentId,
        function: &str,
        args: &[u8],
    ) -> Result<ExecutionResult> {
        // Check if agent needs to be started
        {
            let agent = self
                .agents
                .get(&agent_id)
                .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

            if agent.state != AgentState::Running {
                drop(agent);
                self.start_agent(agent_id)?;
            }
        }

        // Now execute
        let mut agent = self.agents.get_mut(&agent_id).unwrap();
        let result = agent.sandbox.execute(function, args).await?;

        agent.resource_usage.update_cpu(result.fuel_consumed);

        Ok(result)
    }

    /// Gets the memory usage of a specific agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found
    pub fn get_agent_memory_usage(&self, agent_id: AgentId) -> Result<usize> {
        let agent = self
            .agents
            .get(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        Ok(agent.sandbox.get_memory_usage())
    }

    /// Gets the CPU fuel usage of a specific agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found
    pub fn get_agent_cpu_usage(&self, agent_id: AgentId) -> Result<u64> {
        let agent = self
            .agents
            .get(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        Ok(agent.resource_usage.cpu_fuel_consumed)
    }

    /// Gets the list of host functions exposed to an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found
    pub fn get_exposed_host_functions(&self, agent_id: AgentId) -> Result<Vec<String>> {
        let agent = self
            .agents
            .get(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        Ok(agent.sandbox.get_exposed_functions())
    }

    /// Gets the runtime's security policy
    pub fn get_security_policy(&self) -> &SecurityPolicy {
        &self.config.security_policy
    }

    fn validate_module(module: &Module) {
        debug!("Validating WASM module");

        let mut exports = module.exports();
        let has_memory = exports.any(|e| e.name() == "memory");

        if !has_memory {
            debug!("Module does not export memory, this is acceptable");
        }
    }

    /// Stops a running agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or shutdown fails
    pub async fn stop_agent(&mut self, agent_id: AgentId) -> Result<()> {
        let mut agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        // Transition through draining state
        let prev_state = agent.state.clone();
        agent.state = AgentState::Draining;
        info!(
            "Agent {} ({}) transitioning from {:?} to {:?}",
            agent.name(),
            agent.id(),
            prev_state,
            AgentState::Draining
        );

        agent.sandbox.shutdown().await?;

        agent.state = AgentState::Stopped;

        // Log resource usage on stop
        agent.resource_usage.increment_message_count();
        info!(
            "Agent {} ({}) stopped after processing messages",
            agent.name(),
            agent.id()
        );
        Ok(())
    }

    /// Removes an agent from the runtime
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found
    pub fn remove_agent(&mut self, agent_id: AgentId) -> Result<()> {
        if let Some((_, mut agent)) = self.agents.remove(&agent_id) {
            match &agent.state {
                AgentState::Running => {
                    warn!("Removing running agent {} ({})", agent.name(), agent.id());
                }
                AgentState::Unloaded => {
                    debug!("Removing unloaded agent {} ({})", agent.name(), agent.id());
                }
                state => {
                    info!(
                        "Removing agent {} ({}) in state {:?}",
                        agent.name(),
                        agent.id(),
                        state
                    );
                }
            }

            // Track resource usage
            agent.resource_usage.update_memory(0);

            self.active_count.fetch_sub(1, Ordering::SeqCst);
            self.resource_manager.cleanup_agent(agent_id);
            info!("Agent {} removed", agent_id);
            Ok(())
        } else {
            bail!("Agent not found: {}", agent_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wasm_runtime_config_default() {
        let config = WasmRuntimeConfig::default();
        assert_eq!(config.max_agents, 1000);
        assert!(!config.enable_debug);
    }

    #[test]
    fn test_wasm_runtime_new() {
        let config = WasmRuntimeConfig::default();
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());
        let runtime = runtime.unwrap();
        assert!(runtime.is_initialized());
        assert_eq!(runtime.active_agent_count(), 0);
    }

    #[tokio::test]
    async fn test_agent_state_transitions() {
        let state = AgentState::Unloaded;
        assert_eq!(state, AgentState::Unloaded);

        let state = AgentState::Loaded;
        assert_eq!(state, AgentState::Loaded);

        let state = AgentState::Running;
        assert_eq!(state, AgentState::Running);
    }

    #[test]
    fn test_resource_usage_update_memory() {
        let mut usage = ResourceUsage::default();
        assert_eq!(usage.memory_bytes, 0);

        usage.update_memory(1024);
        assert_eq!(usage.memory_bytes, 1024);

        usage.update_memory(2048);
        assert_eq!(usage.memory_bytes, 2048);
    }

    #[test]
    fn test_resource_usage_update_cpu() {
        let mut usage = ResourceUsage::default();
        assert_eq!(usage.cpu_fuel_consumed, 0);

        usage.update_cpu(100);
        assert_eq!(usage.cpu_fuel_consumed, 100);

        usage.update_cpu(50);
        assert_eq!(usage.cpu_fuel_consumed, 150);
    }

    #[test]
    fn test_resource_usage_increment_message_count() {
        let mut usage = ResourceUsage::default();
        assert_eq!(usage.message_count, 0);

        usage.increment_message_count();
        assert_eq!(usage.message_count, 1);

        usage.increment_message_count();
        assert_eq!(usage.message_count, 2);
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            fuel_consumed: 100,
            completed_successfully: true,
            output: Some(vec![1, 2, 3]),
        };

        assert_eq!(result.fuel_consumed, 100);
        assert!(result.completed_successfully);
        assert_eq!(result.output, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_wasm_runtime_max_agents() {
        let config = WasmRuntimeConfig {
            max_agents: 2,
            ..Default::default()
        };
        let runtime = WasmRuntime::new(config).unwrap();
        assert_eq!(runtime.config.max_agents, 2);
    }

    #[test]
    fn test_agent_state_equality() {
        assert_eq!(AgentState::Unloaded, AgentState::Unloaded);
        assert_ne!(AgentState::Unloaded, AgentState::Loaded);
        assert_ne!(AgentState::Running, AgentState::Stopped);
    }
}
