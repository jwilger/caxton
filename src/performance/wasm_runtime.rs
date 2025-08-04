//! # WebAssembly Runtime Performance Optimization
//!
//! This module provides high-performance WebAssembly runtime management with:
//! - Instance pooling for reduced instantiation overhead
//! - Optimized wasmtime configuration for agent workloads
//! - Memory management and resource limiting
//! - Performance monitoring and profiling

use crate::performance::PerformanceMonitor;
use ahash::HashMap;
use metrics::{counter, gauge, histogram};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, instrument, warn};
use wasmtime::{
    AsContextMut, Config, Engine, Instance, Linker, Module, PoolingAllocationConfig,
    ResourceLimiter, Store, StoreLimitsBuilder,
};
use wasmtime_wasi::p2::{WasiCtx, WasiCtxBuilder};

/// High-performance WebAssembly runtime optimized for agent execution
#[derive(Debug)]
pub struct OptimizedWasmRuntime {
    /// Wasmtime engine with optimized configuration
    engine: Engine,
    /// Module cache for fast agent instantiation
    module_cache: Arc<RwLock<HashMap<String, Arc<Module>>>>,
    /// Pre-warmed instance pool
    instance_pool: Arc<InstancePool>,
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
}

impl OptimizedWasmRuntime {
    /// Create a new optimized WASM runtime
    #[instrument]
    pub fn new(performance_monitor: Arc<PerformanceMonitor>) -> wasmtime::Result<Self> {
        let mut config = Config::new();

        // Enable optimizations for agent workloads
        config.wasm_simd(true); // Enable SIMD for computational agents
        config.wasm_multi_value(true); // Enable multi-value returns
        config.wasm_bulk_memory(true); // Enable bulk memory operations
        config.wasm_reference_types(true); // Enable reference types
        config.cranelift_opt_level(wasmtime::OptLevel::Speed); // Optimize for speed

        // Configure async support for I/O bound agents
        config.async_support(true);

        // Enable pooling allocation for reduced memory overhead
        let mut pooling_config = PoolingAllocationConfig::default();
        pooling_config.total_component_instances(1000); // Support up to 1000 agents
        pooling_config.total_core_instances(1000);
        pooling_config.total_memories(1000);
        pooling_config.total_tables(1000);
        pooling_config.max_memories_per_module(1);
        pooling_config.max_tables_per_module(1);
        pooling_config.max_memory_size(64 * 1024 * 1024); // 64MB per agent max

        config.allocation_strategy(wasmtime::InstanceAllocationStrategy::Pooling(
            pooling_config,
        ));

        let engine = Engine::new(&config)?;

        info!(
            simd_enabled = true,
            async_enabled = true,
            pooling_enabled = true,
            max_instances = 1000,
            max_memory_mb = 64,
            "Optimized WASM runtime initialized"
        );

        Ok(Self {
            engine,
            module_cache: Arc::new(RwLock::new(HashMap::default())),
            instance_pool: Arc::new(InstancePool::new(100, 10)), // 100 instances, 10 per type
            performance_monitor,
        })
    }

    /// Load and cache a WASM module for fast instantiation
    #[instrument(skip(self, wasm_bytes))]
    pub async fn load_module(
        &self,
        module_name: &str,
        wasm_bytes: &[u8],
    ) -> wasmtime::Result<Arc<Module>> {
        let start_time = Instant::now();

        // Check cache first
        {
            let cache = self.module_cache.read().await;
            if let Some(module) = cache.get(module_name) {
                debug!(module_name = module_name, "Module loaded from cache");
                return Ok(module.clone());
            }
        }

        // Compile module
        let module = Module::new(&self.engine, wasm_bytes)?;
        let module = Arc::new(module);

        // Cache for future use
        {
            let mut cache = self.module_cache.write().await;
            cache.insert(module_name.to_string(), module.clone());
        }

        let duration = start_time.elapsed();
        histogram!("caxton_wasm_module_compilation_duration_seconds")
            .record(duration.as_secs_f64());
        counter!("caxton_wasm_modules_compiled_total");

        info!(
            module_name = module_name,
            compilation_time_ms = duration.as_millis(),
            "WASM module compiled and cached"
        );

        Ok(module)
    }

    /// Get a pre-warmed instance from the pool or create a new one
    #[instrument(skip(self, module))]
    pub async fn get_instance(
        &self,
        module: &Arc<Module>,
        agent_type: &str,
    ) -> wasmtime::Result<PooledInstance> {
        let start_time = Instant::now();

        // Try to get instance from pool first
        if let Some(instance) = self.instance_pool.get_instance(agent_type).await {
            let duration = start_time.elapsed();
            histogram!("caxton_wasm_instance_acquisition_duration_seconds")
                .record(duration.as_secs_f64());
            counter!("caxton_wasm_instances_from_pool_total");

            debug!(
                agent_type = agent_type,
                acquisition_time_us = duration.as_micros(),
                source = "pool",
                "WASM instance acquired"
            );

            return Ok(instance);
        }

        // Create new instance if pool is empty
        let instance = self.create_instance(module, agent_type).await?;

        let duration = start_time.elapsed();
        histogram!("caxton_wasm_instance_acquisition_duration_seconds")
            .record(duration.as_secs_f64());
        counter!("caxton_wasm_instances_created_total");

        info!(
            agent_type = agent_type,
            acquisition_time_ms = duration.as_millis(),
            source = "new",
            "WASM instance created"
        );

        Ok(instance)
    }

    /// Create a new WASM instance with optimized configuration
    #[instrument(skip(self, module))]
    async fn create_instance(
        &self,
        module: &Arc<Module>,
        agent_type: &str,
    ) -> wasmtime::Result<PooledInstance> {
        let mut store = Store::new(&self.engine, WasmRuntimeContext::new(agent_type));

        // Configure resource limits
        let limits = StoreLimitsBuilder::new()
            .memory_size(64 * 1024 * 1024) // 64MB memory limit
            .table_elements(10_000) // 10K table elements
            .instances(1) // Only 1 instance per store
            .tables(10) // Max 10 tables
            .memories(1) // Max 1 memory
            .build();
        store.limiter(|ctx| &mut ctx.limiter);
        store.set_limits(limits);

        // Set up WASI context
        let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_env().build();
        store.data_mut().wasi = Some(wasi);

        // Create linker with WASI imports
        let mut linker = Linker::new(&self.engine);
        // TODO: Update to use proper wasmtime_wasi API when implementing full WASI support
        // For now, skip WASI imports to avoid API compatibility issues

        // Instantiate the module
        let instance = linker.instantiate_async(&mut store, module).await?;

        counter!("caxton_wasm_instances_instantiated_total");
        gauge!("caxton_wasm_instances_active").increment(1.0);

        Ok(PooledInstance {
            store,
            instance,
            agent_type: agent_type.to_string(),
            pool: Arc::downgrade(&self.instance_pool),
        })
    }

    /// Return an instance to the pool for reuse
    #[instrument(skip(self, instance))]
    pub async fn return_instance(&self, instance: PooledInstance) {
        let agent_type = instance.agent_type.clone();

        // Reset instance state before returning to pool
        if let Err(e) = self.reset_instance_state(&instance).await {
            warn!(
                agent_type = agent_type,
                error = %e,
                "Failed to reset instance state, discarding"
            );
            return;
        }

        // Return to pool
        self.instance_pool.return_instance(instance).await;

        counter!("caxton_wasm_instances_returned_to_pool_total");
        debug!(agent_type = agent_type, "Instance returned to pool");
    }

    /// Reset instance state for reuse
    #[instrument(skip(self, instance))]
    async fn reset_instance_state(&self, instance: &PooledInstance) -> wasmtime::Result<()> {
        // Call reset function if available
        if let Ok(reset_func) = instance
            .instance
            .get_typed_func::<(), ()>(instance.store.as_context_mut(), "reset")
        {
            reset_func
                .call_async(instance.store.as_context_mut(), ())
                .await?;
        }

        // Clear any global state if needed
        // This would be agent-specific and might require custom reset logic

        Ok(())
    }

    /// Get runtime performance statistics
    #[instrument(skip(self))]
    pub async fn get_performance_stats(&self) -> WasmRuntimeStats {
        let module_cache = self.module_cache.read().await;
        let pool_stats = self.instance_pool.get_stats().await;

        WasmRuntimeStats {
            cached_modules: module_cache.len(),
            pooled_instances: pool_stats.total_pooled,
            active_instances: pool_stats.total_active,
            pool_hit_rate: pool_stats.hit_rate,
        }
    }
}

/// WebAssembly runtime context for each agent instance
struct WasmRuntimeContext {
    agent_type: String,
    wasi: Option<WasiCtx>,
    limiter: ResourceLimiterImpl,
}

impl std::fmt::Debug for WasmRuntimeContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmRuntimeContext")
            .field("agent_type", &self.agent_type)
            .field("wasi", &"<WasiCtx>")
            .field("limiter", &self.limiter)
            .finish()
    }
}

impl WasmRuntimeContext {
    fn new(agent_type: &str) -> Self {
        Self {
            agent_type: agent_type.to_string(),
            wasi: None,
            limiter: ResourceLimiterImpl::new(),
        }
    }
}

/// Resource limiter implementation for WASM instances
#[derive(Debug)]
struct ResourceLimiterImpl {
    memory_used: usize,
    table_elements_used: usize,
}

impl ResourceLimiterImpl {
    fn new() -> Self {
        Self {
            memory_used: 0,
            table_elements_used: 0,
        }
    }
}

impl ResourceLimiter for ResourceLimiterImpl {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        if let Some(max) = maximum {
            if desired > max {
                return Ok(false);
            }
        }

        // Allow growth up to 64MB
        if desired > 64 * 1024 * 1024 {
            warn!(
                current = current,
                desired = desired,
                "Memory growth rejected - exceeds limit"
            );
            return Ok(false);
        }

        self.memory_used = desired;
        Ok(true)
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        if let Some(max) = maximum {
            if desired > max {
                return Ok(false);
            }
        }

        // Allow up to 10K table elements
        if desired > 10_000 {
            warn!(
                current = current,
                desired = desired,
                "Table growth rejected - exceeds limit"
            );
            return Ok(false);
        }

        self.table_elements_used = desired;
        Ok(true)
    }
}

/// Pool of pre-warmed WASM instances for fast agent spawning
#[derive(Debug)]
struct InstancePool {
    /// Maximum number of instances per agent type
    max_per_type: usize,
    /// Total maximum instances
    max_total: usize,
    /// Pooled instances by agent type
    pools: Arc<RwLock<HashMap<String, Vec<PooledInstance>>>>,
    /// Semaphore to limit total instances
    semaphore: Arc<Semaphore>,
    /// Pool statistics
    stats: Arc<RwLock<PoolStats>>,
}

impl InstancePool {
    fn new(max_total: usize, max_per_type: usize) -> Self {
        Self {
            max_per_type,
            max_total,
            pools: Arc::new(RwLock::new(HashMap::default())),
            semaphore: Arc::new(Semaphore::new(max_total)),
            stats: Arc::new(RwLock::new(PoolStats::default())),
        }
    }

    async fn get_instance(&self, agent_type: &str) -> Option<PooledInstance> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(agent_type)?;

        if let Some(instance) = pool.pop() {
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            stats.total_active += 1;
            Some(instance)
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            None
        }
    }

    async fn return_instance(&self, instance: PooledInstance) {
        let agent_type = instance.agent_type.clone();
        let mut pools = self.pools.write().await;

        let pool = pools.entry(agent_type).or_insert_with(Vec::new);

        if pool.len() < self.max_per_type {
            pool.push(instance);

            let mut stats = self.stats.write().await;
            stats.total_active -= 1;
        }
        // If pool is full, instance is dropped
    }

    async fn get_stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }
}

/// Pooled WASM instance wrapper
pub struct PooledInstance {
    store: Store<WasmRuntimeContext>,
    instance: Instance,
    agent_type: String,
    pool: std::sync::Weak<InstancePool>,
}

impl std::fmt::Debug for PooledInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledInstance")
            .field("agent_type", &self.agent_type)
            .field("store", &"<Store>")
            .field("instance", &"<Instance>")
            .field("pool", &"<Weak<InstancePool>>")
            .finish()
    }
}

impl PooledInstance {
    /// Execute a function in this instance
    #[instrument(skip(self))]
    pub async fn call_function<P, R>(&mut self, func_name: &str, params: P) -> wasmtime::Result<R>
    where
        P: wasmtime::WasmParams + Send,
        R: wasmtime::WasmResults,
    {
        let start_time = Instant::now();

        let func = self
            .instance
            .get_typed_func::<P, R>(&mut self.store, func_name)?;
        let result = func.call_async(&mut self.store, params).await?;

        let duration = start_time.elapsed();
        histogram!("caxton_wasm_function_execution_duration_seconds")
            .record(duration.as_secs_f64());
        counter!("caxton_wasm_function_calls_total");

        debug!(
            function = func_name,
            agent_type = self.agent_type,
            execution_time_us = duration.as_micros(),
            "WASM function executed"
        );

        Ok(result)
    }

    /// Get the agent type for this instance
    pub fn agent_type(&self) -> &str {
        &self.agent_type
    }
}

impl Drop for PooledInstance {
    fn drop(&mut self) {
        gauge!("caxton_wasm_instances_active").decrement(1.0);
    }
}

/// Pool performance statistics
#[derive(Debug, Clone, Default)]
struct PoolStats {
    hits: u64,
    misses: u64,
    total_pooled: usize,
    total_active: usize,
}

impl PoolStats {
    fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

/// Runtime performance statistics
#[derive(Debug, Clone)]
pub struct WasmRuntimeStats {
    pub cached_modules: usize,
    pub pooled_instances: usize,
    pub active_instances: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_wasm_runtime_creation() {
        let monitor = Arc::new(PerformanceMonitor::new());
        let runtime = OptimizedWasmRuntime::new(monitor).unwrap();
        let stats = runtime.get_performance_stats().await;

        assert_eq!(stats.cached_modules, 0);
        assert_eq!(stats.pooled_instances, 0);
    }

    // Additional tests would require actual WASM modules
    // These would be added as integration tests
}
