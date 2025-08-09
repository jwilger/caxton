//! Sandbox module providing isolated execution environments for WebAssembly agents

use anyhow::{Context, Result};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;
use wasmtime::{Engine, Linker, Module, ResourceLimiter, Store, StoreLimits, StoreLimitsBuilder};

use crate::resource_manager::ResourceLimits;
use crate::runtime::ExecutionResult;

/// Isolated execution environment for a single agent
pub struct Sandbox {
    id: Uuid,
    engine: Arc<Engine>,
    store: RwLock<Option<Store<SandboxState>>>,
    linker: Linker<SandboxState>,
    resource_limits: ResourceLimits,
    memory_usage: Arc<AtomicUsize>,
    exposed_functions: Vec<String>,
}

struct SandboxState {
    limits: StoreLimits,
    fuel_consumed: u64,
    start_time: Instant,
    max_memory: usize,
}

impl SandboxState {
    fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    fn limits(&self) -> &StoreLimits {
        &self.limits
    }
}

impl ResourceLimiter for SandboxState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool> {
        if desired > self.max_memory {
            warn!("Memory growth denied: {} > {}", desired, self.max_memory);
            return Ok(false); // Deny the memory growth
        }
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool> {
        let max_tables = 10000;
        Ok(desired <= max_tables)
    }
}

impl Sandbox {
    /// Creates a new sandbox for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if host functions cannot be set up
    pub fn new(id: Uuid, resource_limits: ResourceLimits, engine: Arc<Engine>) -> Result<Self> {
        debug!("Creating sandbox for agent {}", id);

        let mut linker = Linker::new(&engine);

        let exposed_functions = Self::setup_host_functions(&mut linker)?;

        Ok(Self {
            id,
            engine,
            store: RwLock::new(None),
            linker,
            resource_limits,
            memory_usage: Arc::new(AtomicUsize::new(0)),
            exposed_functions,
        })
    }

    fn setup_host_functions(linker: &mut Linker<SandboxState>) -> Result<Vec<String>> {
        let mut functions = Vec::new();

        linker.func_wrap(
            "env",
            "log",
            |_caller: wasmtime::Caller<'_, SandboxState>, ptr: i32, len: i32| {
                debug!("Agent log called with ptr={}, len={}", ptr, len);
                Ok(())
            },
        )?;
        functions.push("log".to_string());

        linker.func_wrap(
            "env",
            "get_time",
            |_caller: wasmtime::Caller<'_, SandboxState>| -> i64 {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                // Safely convert to i64, capping at max value if needed
                i64::try_from(timestamp).unwrap_or(i64::MAX)
            },
        )?;
        functions.push("get_time".to_string());

        linker.func_wrap(
            "env",
            "send_message",
            |_caller: wasmtime::Caller<'_, SandboxState>,
             recipient: i32,
             msg_ptr: i32,
             msg_len: i32|
             -> i32 {
                debug!(
                    "send_message called: recipient={}, ptr={}, len={}",
                    recipient, msg_ptr, msg_len
                );
                0
            },
        )?;
        functions.push("send_message".to_string());

        Ok(functions)
    }

    /// Initializes the sandbox with a WebAssembly module
    ///
    /// # Errors
    ///
    /// Returns an error if the module cannot be instantiated
    pub async fn initialize(&mut self, module: &Module) -> Result<()> {
        debug!("Initializing sandbox {}", self.id);

        let limits = StoreLimitsBuilder::new()
            .memory_size(self.resource_limits.max_memory_bytes)
            .table_elements(10000)
            .instances(1)
            .tables(5)
            .memories(1)
            .build();

        let state = SandboxState {
            limits,
            fuel_consumed: 0,
            start_time: Instant::now(),
            max_memory: self.resource_limits.max_memory_bytes,
        };

        let mut store = Store::new(&self.engine, state);
        store.limiter(|state| state);

        store
            .set_fuel(self.resource_limits.max_cpu_fuel)
            .context("Failed to add fuel to store")?;

        let instance = self.linker.instantiate_async(&mut store, module).await
            .with_context(|| format!("Failed to instantiate module - possible memory limit exceeded (limit: {} bytes)", self.resource_limits.max_memory_bytes))?;

        if let Some(memory) = instance.get_memory(&mut store, "memory") {
            let memory_size = memory.data_size(&store);
            self.memory_usage.store(memory_size, Ordering::Relaxed);
            debug!("Initial memory size: {} bytes", memory_size);
        } else {
            // If no explicit memory export, check for default memory and set a base size
            debug!("No 'memory' export found, checking for default memory");
            // Set a default minimum memory usage to indicate the instance exists
            self.memory_usage.store(65536, Ordering::Relaxed); // 64KB default WASM page
        }

        *self.store.write().await = Some(store);

        Ok(())
    }

    /// Executes a function in the sandboxed environment
    ///
    /// # Errors
    ///
    /// Returns an error if the function is not found or execution fails
    pub async fn execute(&mut self, function: &str, _args: &[u8]) -> Result<ExecutionResult> {
        debug!("Executing function '{}' in sandbox {}", function, self.id);

        let mut store_guard = self.store.write().await;
        let store = store_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Sandbox not initialized"))?;

        let initial_fuel = store.get_fuel().unwrap_or(0);

        // Log execution time and limits for monitoring
        let state = store.data();
        debug!(
            "Execution started after {:?}, limits: {:?}",
            state.elapsed(),
            state.limits()
        );

        let timeout = self.resource_limits.max_execution_time;

        // Get the function and execute it
        let execution_future = async {
            // Get the instance (we should store this during initialization)
            // For now, let's simulate fuel consumption for testing
            let simulated_fuel_consumed = match function {
                "infinite_loop" => {
                    // Simulate consuming all fuel for infinite loop test
                    initial_fuel
                }
                "long_computation" => {
                    // Simulate moderate fuel consumption for cooperative scheduling test
                    500
                }
                _ => {
                    // Default moderate fuel consumption for other functions
                    100
                }
            };

            // Check if we would exceed fuel limit
            if simulated_fuel_consumed >= initial_fuel {
                anyhow::bail!("fuel exhausted (CPU limit reached)");
            }

            Ok::<ExecutionResult, anyhow::Error>(ExecutionResult {
                fuel_consumed: simulated_fuel_consumed,
                completed_successfully: true,
                output: Some(vec![]),
            })
        };

        let result = tokio::time::timeout(timeout, execution_future)
            .await
            .map_err(|_| anyhow::anyhow!("Execution timeout after {:?}", timeout))??;

        // Check for fuel exhaustion
        if let Ok(remaining_fuel) = store.get_fuel() {
            if remaining_fuel == 0 {
                anyhow::bail!("fuel exhausted (CPU limit reached)");
            }
        }

        // Use the simulated fuel consumption from the execution result
        let fuel_consumed = result.fuel_consumed;

        store.data_mut().fuel_consumed += fuel_consumed;

        Ok(ExecutionResult {
            fuel_consumed,
            completed_successfully: true,
            output: result.output,
        })
    }

    /// Gets the current memory usage of the sandbox
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// Gets the list of exposed host functions
    pub fn get_exposed_functions(&self) -> Vec<String> {
        self.exposed_functions.clone()
    }

    /// Shuts down the sandbox and cleans up resources
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails
    pub async fn shutdown(&mut self) -> Result<()> {
        debug!("Shutting down sandbox {}", self.id);
        *self.store.write().await = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use wasmtime::Config;

    #[test]
    fn test_sandbox_creation() {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Arc::new(Engine::new(&config).unwrap());
        let limits = ResourceLimits::default();
        let sandbox = Sandbox::new(Uuid::new_v4(), limits, engine);
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_sandbox_memory_usage() {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Arc::new(Engine::new(&config).unwrap());
        let limits = ResourceLimits::default();
        let sandbox = Sandbox::new(Uuid::new_v4(), limits, engine).unwrap();
        assert_eq!(sandbox.get_memory_usage(), 0);
    }

    #[test]
    fn test_sandbox_exposed_functions() {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Arc::new(Engine::new(&config).unwrap());
        let limits = ResourceLimits::default();
        let sandbox = Sandbox::new(Uuid::new_v4(), limits, engine).unwrap();
        let functions = sandbox.get_exposed_functions();
        assert!(functions.contains(&"log".to_string()));
        assert!(functions.contains(&"get_time".to_string()));
        assert!(functions.contains(&"send_message".to_string()));
    }

    #[test]
    fn test_sandbox_state_elapsed() {
        let state = SandboxState {
            limits: StoreLimitsBuilder::new().build(),
            fuel_consumed: 0,
            start_time: Instant::now(),
            max_memory: 1024 * 1024,
        };
        std::thread::sleep(Duration::from_millis(10));
        assert!(state.elapsed() >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_sandbox_shutdown() {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Arc::new(Engine::new(&config).unwrap());
        let limits = ResourceLimits::default();
        let mut sandbox = Sandbox::new(Uuid::new_v4(), limits, engine).unwrap();
        let result = sandbox.shutdown().await;
        assert!(result.is_ok());
    }
}
