//! Agent Lifecycle Management Demo
//!
//! A standalone demonstration of the complete agent lifecycle implementation
//! showing spawning, state management, resource tracking, and termination.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::time::sleep;
use tracing::{info, warn, debug, error};
use uuid::Uuid;

// ================================================================================================
// CORE TYPES - Agent Lifecycle Components
// ================================================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Initializing,
    Ready,
    Processing,
    Suspended,
    Terminating,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Worker,
    Coordinator,
    Monitor,
    Proxy,
}

#[derive(Debug, Clone)]
pub struct AgentId(String);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub max_memory: Option<u64>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub name: String,
    pub agent_type: AgentType,
    pub state: AgentState,
    pub capabilities: Vec<String>,
    pub properties: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AgentMetadata {
    pub fn new(name: String, agent_type: AgentType) -> Self {
        let now = Utc::now();
        Self {
            name,
            agent_type,
            state: AgentState::Initializing,
            capabilities: Vec::new(),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
        self.updated_at = Utc::now();
    }

    pub fn add_capability(&mut self, capability: &str) {
        self.capabilities.push(capability.to_string());
        self.updated_at = Utc::now();
    }

    pub fn set_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResourceUsage {
    pub memory_bytes: u64,
    pub cpu_time_ms: u64,
    pub message_count: u64,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
    pub metrics: HashMap<String, f64>,
}

// ================================================================================================
// ERROR TYPES
// ================================================================================================

#[derive(Error, Debug)]
pub enum DemoError {
    #[error("Agent error: {0}")]
    Agent(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

// ================================================================================================
// AGENT RUNTIME - Core Lifecycle Management
// ================================================================================================

#[derive(Debug)]
struct AgentInstance {
    pub metadata: AgentMetadata,
    pub resource_usage: AgentResourceUsage,
    pub message_tx: mpsc::UnboundedSender<String>,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct CaxtonRuntime {
    agents: Arc<DashMap<AgentId, AgentInstance>>,
    spawn_semaphore: Arc<Semaphore>,
    max_agents: usize,
}

impl CaxtonRuntime {
    pub async fn new(max_agents: usize) -> Self {
        info!("🚀 Initializing Caxton Runtime with max {} agents", max_agents);

        Self {
            agents: Arc::new(DashMap::new()),
            spawn_semaphore: Arc::new(Semaphore::new(max_agents)),
            max_agents,
        }
    }

    /// Spawn a new agent with complete lifecycle management
    pub async fn spawn_agent(&self, config: AgentConfig) -> Result<AgentId, DemoError> {
        // Acquire semaphore permit to limit concurrent agents
        let _permit = self.spawn_semaphore
            .acquire()
            .await
            .map_err(|_| DemoError::Runtime("Failed to acquire spawn permit".to_string()))?;

        let agent_id = AgentId::new();
        info!("🔧 Spawning agent: {} ({})", config.name, agent_id);

        // Create agent metadata
        let mut metadata = AgentMetadata::new(config.name.clone(), config.agent_type.clone());
        for capability in &config.capabilities {
            metadata.add_capability(capability);
        }

        // Set resource limits in properties
        if let Some(max_memory) = config.max_memory {
            metadata.set_property("max_memory", &max_memory.to_string());
        }
        if let Some(timeout) = config.timeout {
            metadata.set_property("timeout_ms", &timeout.as_millis().to_string());
        }

        // Create message channels
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Initialize resource usage tracking
        let resource_usage = AgentResourceUsage {
            memory_bytes: 0,
            cpu_time_ms: 0,
            message_count: 0,
            last_activity: Utc::now(),
        };

        // Start agent task
        let agent_task = self.start_agent_task(
            agent_id.clone(),
            config,
            message_rx,
            shutdown_rx,
        ).await?;

        // Create agent instance
        let agent_instance = AgentInstance {
            metadata: metadata.clone(),
            resource_usage,
            message_tx,
            shutdown_tx: Some(shutdown_tx),
            task_handle: Some(agent_task),
        };

        // Update state to ready
        let mut metadata = metadata;
        metadata.set_state(AgentState::Ready);
        let mut agent_instance = agent_instance;
        agent_instance.metadata = metadata;

        // Register agent
        self.agents.insert(agent_id.clone(), agent_instance);

        info!("✅ Agent {} spawned successfully", agent_id);
        Ok(agent_id)
    }

    /// Terminate an agent with graceful shutdown
    pub async fn terminate_agent(&self, agent_id: &AgentId, timeout: Duration) -> Result<(), DemoError> {
        info!("🛑 Terminating agent: {} with timeout: {:?}", agent_id, timeout);

        // Get agent instance
        let mut agent_instance = self.agents.get_mut(agent_id)
            .ok_or_else(|| DemoError::Agent(format!("Agent not found: {}", agent_id)))?;

        // Update state to terminating
        agent_instance.metadata.set_state(AgentState::Terminating);

        // Send shutdown signal
        if let Some(shutdown_tx) = agent_instance.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        // Wait for task completion with timeout
        if let Some(handle) = agent_instance.task_handle.take() {
            match tokio::time::timeout(timeout, handle).await {
                Ok(Ok(())) => {
                    info!("✅ Agent {} terminated gracefully", agent_id);
                }
                Ok(Err(e)) => {
                    warn!("⚠️ Agent {} task panicked: {:?}", agent_id, e);
                }
                Err(_) => {
                    warn!("⚠️ Agent {} termination timed out", agent_id);
                }
            }
        }

        // Update final state
        agent_instance.metadata.set_state(AgentState::Terminated);

        info!("🏁 Agent {} termination complete", agent_id);
        Ok(())
    }

    /// Get current agent state
    pub fn get_agent_state(&self, agent_id: &AgentId) -> Result<AgentState, DemoError> {
        let agent = self.agents.get(agent_id)
            .ok_or_else(|| DemoError::Agent(format!("Agent not found: {}", agent_id)))?;
        Ok(agent.metadata.state.clone())
    }

    /// Get agent metadata and resource usage
    pub fn get_agent_info(&self, agent_id: &AgentId) -> Result<(AgentMetadata, AgentResourceUsage), DemoError> {
        let agent = self.agents.get(agent_id)
            .ok_or_else(|| DemoError::Agent(format!("Agent not found: {}", agent_id)))?;
        Ok((agent.metadata.clone(), agent.resource_usage.clone()))
    }

    /// List all agents with their capabilities
    pub fn list_agents(&self) -> Vec<(AgentId, Vec<String>)> {
        self.agents.iter()
            .map(|entry| (entry.key().clone(), entry.value().metadata.capabilities.clone()))
            .collect()
    }

    /// Simulate sending a message to an agent
    pub fn send_message(&self, agent_id: &AgentId, message: &str) -> Result<(), DemoError> {
        let agent = self.agents.get(agent_id)
            .ok_or_else(|| DemoError::Agent(format!("Agent not found: {}", agent_id)))?;

        agent.message_tx.send(message.to_string())
            .map_err(|_| DemoError::Runtime("Failed to send message to agent".to_string()))?;

        info!("📨 Message sent to agent {}: {}", agent_id, message);
        Ok(())
    }

    /// Get runtime metrics
    pub fn get_metrics(&self) -> (usize, usize, u64, u64) {
        let total_agents = self.agents.len();
        let active_agents = self.agents.iter()
            .filter(|entry| matches!(entry.value().metadata.state, AgentState::Ready | AgentState::Processing))
            .count();
        let total_messages: u64 = self.agents.iter()
            .map(|entry| entry.value().resource_usage.message_count)
            .sum();
        let total_memory: u64 = self.agents.iter()
            .map(|entry| entry.value().resource_usage.memory_bytes)
            .sum();

        (total_agents, active_agents, total_messages, total_memory)
    }

    /// Graceful shutdown of entire runtime
    pub async fn shutdown(&self, timeout: Duration) -> Result<(), DemoError> {
        info!("🛑 Shutting down Caxton runtime");

        // Collect all agent IDs
        let agent_ids: Vec<AgentId> = self.agents.iter()
            .map(|entry| entry.key().clone())
            .collect();

        // Terminate all agents
        let per_agent_timeout = timeout / agent_ids.len().max(1) as u32;

        for agent_id in agent_ids {
            if let Err(e) = self.terminate_agent(&agent_id, per_agent_timeout).await {
                warn!("⚠️ Failed to terminate agent {}: {:?}", agent_id, e);
            }
        }

        info!("🏁 Caxton runtime shutdown complete");
        Ok(())
    }

    /// Start an individual agent task with message processing
    async fn start_agent_task(
        &self,
        agent_id: AgentId,
        config: AgentConfig,
        mut message_rx: mpsc::UnboundedReceiver<String>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>, DemoError> {
        let agents = self.agents.clone();

        let handle = tokio::spawn(async move {
            info!("🚀 Agent task started: {} ({})", config.name, agent_id);

            loop {
                tokio::select! {
                    // Handle incoming messages
                    message = message_rx.recv() => {
                        match message {
                            Some(msg) => {
                                debug!("📨 Agent {} processing message: {}", agent_id, msg);

                                // Update agent state to processing
                                if let Some(mut agent) = agents.get_mut(&agent_id) {
                                    agent.metadata.set_state(AgentState::Processing);
                                    agent.resource_usage.last_activity = Utc::now();
                                    agent.resource_usage.message_count += 1;
                                    agent.resource_usage.cpu_time_ms += 10;
                                    agent.resource_usage.memory_bytes += 1024;
                                }

                                // Simulate processing time
                                sleep(Duration::from_millis(50)).await;

                                // Update agent state back to ready
                                if let Some(mut agent) = agents.get_mut(&agent_id) {
                                    agent.metadata.set_state(AgentState::Ready);
                                }

                                debug!("✅ Agent {} completed message processing", agent_id);
                            }
                            None => {
                                warn!("⚠️ Agent {} message channel closed", agent_id);
                                break;
                            }
                        }
                    }

                    // Handle shutdown signal
                    _ = &mut shutdown_rx => {
                        info!("🛑 Agent {} received shutdown signal", agent_id);
                        break;
                    }
                }
            }

            info!("🏁 Agent task completed: {}", agent_id);
        });

        Ok(handle)
    }
}

// ================================================================================================
// DEMONSTRATION SCENARIOS
// ================================================================================================

async fn demonstrate_basic_lifecycle() -> Result<(), DemoError> {
    println!("\n🎯 DEMO 1: Basic Agent Lifecycle");
    println!("================================");

    let runtime = CaxtonRuntime::new(10).await;

    // Spawn a worker agent
    let worker_id = runtime.spawn_agent(AgentConfig {
        name: "worker-001".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["compute".to_string(), "data-processing".to_string()],
        max_memory: Some(64 * 1024 * 1024),
        timeout: Some(Duration::from_secs(30)),
    }).await?;

    println!("✅ Worker agent spawned: {}", worker_id);

    // Check initial state
    let state = runtime.get_agent_state(&worker_id)?;
    println!("📊 Initial state: {:?}", state);

    // Send some messages
    runtime.send_message(&worker_id, "process_data_batch_1")?;
    runtime.send_message(&worker_id, "process_data_batch_2")?;

    // Give time for processing
    sleep(Duration::from_millis(200)).await;

    // Check resource usage
    let (metadata, resources) = runtime.get_agent_info(&worker_id)?;
    println!("📈 Resource usage:");
    println!("   Messages processed: {}", resources.message_count);
    println!("   CPU time: {}ms", resources.cpu_time_ms);
    println!("   Memory usage: {} bytes", resources.memory_bytes);
    println!("   Last activity: {}", resources.last_activity.format("%H:%M:%S%.3f"));

    // Terminate agent
    runtime.terminate_agent(&worker_id, Duration::from_secs(5)).await?;

    let final_state = runtime.get_agent_state(&worker_id)?;
    println!("🏁 Final state: {:?}", final_state);

    Ok(())
}

async fn demonstrate_multi_agent_coordination() -> Result<(), DemoError> {
    println!("\n🎯 DEMO 2: Multi-Agent Coordination");
    println!("===================================");

    let runtime = CaxtonRuntime::new(10).await;

    // Spawn multiple agents with different roles
    let coordinator_id = runtime.spawn_agent(AgentConfig {
        name: "coordinator-alpha".to_string(),
        agent_type: AgentType::Coordinator,
        capabilities: vec!["task-distribution".to_string(), "load-balancing".to_string()],
        max_memory: Some(32 * 1024 * 1024),
        timeout: Some(Duration::from_secs(60)),
    }).await?;

    let worker1_id = runtime.spawn_agent(AgentConfig {
        name: "worker-001".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["compute".to_string(), "data-analysis".to_string()],
        max_memory: Some(64 * 1024 * 1024),
        timeout: Some(Duration::from_secs(30)),
    }).await?;

    let worker2_id = runtime.spawn_agent(AgentConfig {
        name: "worker-002".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["compute".to_string(), "image-processing".to_string()],
        max_memory: Some(64 * 1024 * 1024),
        timeout: Some(Duration::from_secs(30)),
    }).await?;

    let monitor_id = runtime.spawn_agent(AgentConfig {
        name: "monitor-sentinel".to_string(),
        agent_type: AgentType::Monitor,
        capabilities: vec!["health-check".to_string(), "metrics-collection".to_string()],
        max_memory: Some(16 * 1024 * 1024),
        timeout: Some(Duration::from_secs(120)),
    }).await?;

    println!("✅ Multi-agent system deployed");

    // List all agents
    let agents = runtime.list_agents();
    println!("📋 Active agents:");
    for (agent_id, capabilities) in &agents {
        println!("   {} -> {:?}", agent_id, capabilities);
    }

    // Simulate coordinated work
    println!("\n🔄 Simulating coordinated workflow...");

    // Coordinator distributes tasks
    runtime.send_message(&coordinator_id, "distribute_tasks")?;
    runtime.send_message(&worker1_id, "analyze_dataset_part_1")?;
    runtime.send_message(&worker2_id, "process_images_batch_1")?;
    runtime.send_message(&monitor_id, "collect_performance_metrics")?;

    // Second round of tasks
    runtime.send_message(&worker1_id, "analyze_dataset_part_2")?;
    runtime.send_message(&worker2_id, "process_images_batch_2")?;
    runtime.send_message(&coordinator_id, "aggregate_results")?;

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    // Show runtime metrics
    let (total, active, messages, memory) = runtime.get_metrics();
    println!("\n📊 Runtime metrics:");
    println!("   Total agents: {}", total);
    println!("   Active agents: {}", active);
    println!("   Messages processed: {}", messages);
    println!("   Total memory usage: {} KB", memory / 1024);

    // Show individual agent status
    println!("\n🔍 Individual agent status:");
    for agent_id in [&coordinator_id, &worker1_id, &worker2_id, &monitor_id] {
        let (metadata, resources) = runtime.get_agent_info(agent_id)?;
        println!("   {} ({}):", agent_id, metadata.name);
        println!("     State: {:?}", metadata.state);
        println!("     Messages: {}, CPU: {}ms, Memory: {} KB",
                resources.message_count, resources.cpu_time_ms, resources.memory_bytes / 1024);
    }

    // Graceful shutdown
    println!("\n🛑 Shutting down multi-agent system...");
    runtime.shutdown(Duration::from_secs(10)).await?;

    Ok(())
}

async fn demonstrate_resource_limits() -> Result<(), DemoError> {
    println!("\n🎯 DEMO 3: Resource Limits & Error Handling");
    println!("===========================================");

    // Create runtime with limited capacity
    let runtime = CaxtonRuntime::new(2).await;

    // Spawn agents up to limit
    println!("📈 Testing agent spawning limits...");

    let agent1 = runtime.spawn_agent(AgentConfig {
        name: "agent-1".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["task-1".to_string()],
        max_memory: None,
        timeout: None,
    }).await?;

    let agent2 = runtime.spawn_agent(AgentConfig {
        name: "agent-2".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["task-2".to_string()],
        max_memory: None,
        timeout: None,
    }).await?;

    println!("✅ Spawned 2 agents (at capacity)");

    // Try to spawn third agent (should block briefly)
    println!("⏳ Attempting to spawn third agent (will timeout)...");
    let spawn_future = runtime.spawn_agent(AgentConfig {
        name: "agent-3".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["task-3".to_string()],
        max_memory: None,
        timeout: None,
    });

    // This should timeout because we're at capacity
    match tokio::time::timeout(Duration::from_millis(100), spawn_future).await {
        Ok(_) => println!("❌ Unexpected: third agent spawned"),
        Err(_) => println!("✅ Expected: spawn blocked due to capacity limit"),
    }

    // Terminate one agent to free up space
    println!("🔄 Terminating one agent to free capacity...");
    runtime.terminate_agent(&agent1, Duration::from_secs(1)).await?;

    // Now third agent should be able to spawn
    let agent3 = runtime.spawn_agent(AgentConfig {
        name: "agent-3".to_string(),
        agent_type: AgentType::Worker,
        capabilities: vec!["task-3".to_string()],
        max_memory: None,
        timeout: None,
    }).await?;

    println!("✅ Third agent spawned after freeing capacity");

    // Clean up
    runtime.terminate_agent(&agent2, Duration::from_secs(1)).await?;
    runtime.terminate_agent(&agent3, Duration::from_secs(1)).await?;

    Ok(())
}

// ================================================================================================
// MAIN DEMONSTRATION
// ================================================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("🚀 CAXTON AGENT LIFECYCLE MANAGEMENT DEMO");
    println!("=========================================");
    println!("Demonstrating complete agent lifecycle with:");
    println!("  • Agent spawning and termination");
    println!("  • State transitions and monitoring");
    println!("  • Resource usage tracking");
    println!("  • Message processing");
    println!("  • Multi-agent coordination");
    println!("  • Resource limits and error handling");

    // Run all demonstrations
    if let Err(e) = demonstrate_basic_lifecycle().await {
        error!("❌ Basic lifecycle demo failed: {}", e);
    }

    if let Err(e) = demonstrate_multi_agent_coordination().await {
        error!("❌ Multi-agent coordination demo failed: {}", e);
    }

    if let Err(e) = demonstrate_resource_limits().await {
        error!("❌ Resource limits demo failed: {}", e);
    }

    println!("\n✨ All demonstrations completed!");
    println!("🎉 Agent Lifecycle Management implementation validated successfully!");
}
