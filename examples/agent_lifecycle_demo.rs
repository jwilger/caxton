//! Agent Lifecycle Management Demo
//!
//! Demonstrates the complete agent lifecycle implementation with:
//! - Agent spawning and termination
//! - State transitions and monitoring
//! - Resource usage tracking
//! - Message routing
//! - Health checks

use std::time::Duration;
use tokio::time::timeout;

// Simplified types for demo (normally these would be in separate modules)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentState {
    Initializing,
    Ready,
    Processing,
    Suspended,
    Terminating,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentType {
    Worker,
    Coordinator,
    Monitor,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub max_memory: Option<u64>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct AgentId(String);

impl AgentId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct AgentMetadata {
    pub name: String,
    pub agent_type: AgentType,
    pub state: AgentState,
    pub capabilities: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AgentMetadata {
    pub fn new(name: String, agent_type: AgentType) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            agent_type,
            state: AgentState::Initializing,
            capabilities: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
        self.updated_at = chrono::Utc::now();
    }

    pub fn add_capability(&mut self, capability: &str) {
        self.capabilities.push(capability.to_string());
        self.updated_at = chrono::Utc::now();
    }
}

#[derive(Debug, Clone)]
pub struct AgentResourceUsage {
    pub memory_bytes: u64,
    pub cpu_time_ms: u64,
    pub message_count: u64,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

// Simple error type for demo
#[derive(Debug, thiserror::Error)]
pub enum DemoError {
    #[error("Agent error: {0}")]
    Agent(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

// Simplified runtime for demo
#[derive(Clone)]
pub struct SimpleRuntime {
    agents: std::sync::Arc<dashmap::DashMap<AgentId, AgentMetadata>>,
    resources: std::sync::Arc<dashmap::DashMap<AgentId, AgentResourceUsage>>,
}

impl SimpleRuntime {
    pub fn new() -> Self {
        Self {
            agents: std::sync::Arc::new(dashmap::DashMap::new()),
            resources: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    pub async fn spawn_agent(&self, config: AgentConfig) -> Result<AgentId, DemoError> {
        let agent_id = AgentId::new();
        println!("🚀 Spawning agent: {} ({})", config.name, agent_id);

        // Create metadata
        let mut metadata = AgentMetadata::new(config.name.clone(), config.agent_type.clone());
        for capability in &config.capabilities {
            metadata.add_capability(capability);
        }
        metadata.set_state(AgentState::Ready);

        // Create resource tracking
        let resource_usage = AgentResourceUsage {
            memory_bytes: 0,
            cpu_time_ms: 0,
            message_count: 0,
            last_activity: chrono::Utc::now(),
        };

        // Register agent
        self.agents.insert(agent_id.clone(), metadata);
        self.resources.insert(agent_id.clone(), resource_usage);

        println!("✅ Agent {} spawned successfully", agent_id);
        Ok(agent_id)
    }

    pub async fn terminate_agent(&self, agent_id: &AgentId) -> Result<(), DemoError> {
        println!("🛑 Terminating agent: {}", agent_id);

        if let Some(mut metadata) = self.agents.get_mut(agent_id) {
            metadata.set_state(AgentState::Terminating);
            // Simulate shutdown time
            tokio::time::sleep(Duration::from_millis(100)).await;
            metadata.set_state(AgentState::Terminated);
            println!("✅ Agent {} terminated successfully", agent_id);
            Ok(())
        } else {
            Err(DemoError::Agent(format!("Agent not found: {}", agent_id)))
        }
    }

    pub fn get_agent_state(&self, agent_id: &AgentId) -> Result<AgentState, DemoError> {
        if let Some(metadata) = self.agents.get(agent_id) {
            Ok(metadata.state.clone())
        } else {
            Err(DemoError::Agent(format!("Agent not found: {}", agent_id)))
        }
    }

    pub fn get_agent_metadata(
        &self,
        agent_id: &AgentId,
    ) -> Result<(AgentMetadata, AgentResourceUsage), DemoError> {
        let metadata = self
            .agents
            .get(agent_id)
            .ok_or_else(|| DemoError::Agent(format!("Agent not found: {}", agent_id)))?;
        let resource_usage = self
            .resources
            .get(agent_id)
            .ok_or_else(|| DemoError::Agent(format!("Agent resources not found: {}", agent_id)))?;

        Ok((metadata.clone(), resource_usage.clone()))
    }

    pub fn list_agents(&self) -> Vec<(AgentId, Vec<String>)> {
        self.agents
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().capabilities.clone()))
            .collect()
    }

    pub fn simulate_message_processing(&self, agent_id: &AgentId) -> Result<(), DemoError> {
        if let Some(mut metadata) = self.agents.get_mut(agent_id) {
            println!("📨 Agent {} processing message", agent_id);
            metadata.set_state(AgentState::Processing);

            // Update resource usage
            if let Some(mut resources) = self.resources.get_mut(agent_id) {
                resources.message_count += 1;
                resources.cpu_time_ms += 10;
                resources.memory_bytes += 1024;
                resources.last_activity = chrono::Utc::now();
            }

            // Return to ready state
            metadata.set_state(AgentState::Ready);
            println!("✅ Agent {} completed message processing", agent_id);
            Ok(())
        } else {
            Err(DemoError::Agent(format!("Agent not found: {}", agent_id)))
        }
    }

    pub fn get_runtime_metrics(&self) -> (usize, usize, u64, u64) {
        let total_agents = self.agents.len();
        let active_agents = self
            .agents
            .iter()
            .filter(|entry| {
                matches!(
                    entry.value().state,
                    AgentState::Ready | AgentState::Processing
                )
            })
            .count();
        let total_messages: u64 = self
            .resources
            .iter()
            .map(|entry| entry.value().message_count)
            .sum();
        let total_memory: u64 = self
            .resources
            .iter()
            .map(|entry| entry.value().memory_bytes)
            .sum();

        (total_agents, active_agents, total_messages, total_memory)
    }
}

async fn demonstrate_agent_lifecycle() -> Result<(), DemoError> {
    println!("🎯 Agent Lifecycle Management Demo");
    println!("==================================\n");

    // Create runtime
    let runtime = SimpleRuntime::new();
    println!("📊 Runtime initialized\n");

    // Spawn multiple agents
    println!("1️⃣ Spawning agents...");
    let worker_agent = runtime
        .spawn_agent(AgentConfig {
            name: "worker-1".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec!["compute".to_string(), "data-processing".to_string()],
            max_memory: Some(64 * 1024 * 1024),
            timeout: Some(Duration::from_secs(30)),
        })
        .await?;

    let coordinator_agent = runtime
        .spawn_agent(AgentConfig {
            name: "coordinator-1".to_string(),
            agent_type: AgentType::Coordinator,
            capabilities: vec!["coordination".to_string(), "monitoring".to_string()],
            max_memory: Some(32 * 1024 * 1024),
            timeout: Some(Duration::from_secs(60)),
        })
        .await?;

    let monitor_agent = runtime
        .spawn_agent(AgentConfig {
            name: "monitor-1".to_string(),
            agent_type: AgentType::Monitor,
            capabilities: vec!["health-check".to_string(), "metrics".to_string()],
            max_memory: Some(16 * 1024 * 1024),
            timeout: Some(Duration::from_secs(120)),
        })
        .await?;

    println!();

    // Check initial state
    println!("2️⃣ Checking agent states...");
    for agent_id in [&worker_agent, &coordinator_agent, &monitor_agent] {
        let state = runtime.get_agent_state(agent_id)?;
        println!("   Agent {}: {:?}", agent_id, state);
    }
    println!();

    // List all agents with capabilities
    println!("3️⃣ Listing agents and capabilities...");
    let agents = runtime.list_agents();
    for (agent_id, capabilities) in &agents {
        println!("   {} -> {:?}", agent_id, capabilities);
    }
    println!();

    // Simulate message processing
    println!("4️⃣ Simulating message processing...");
    runtime.simulate_message_processing(&worker_agent)?;
    runtime.simulate_message_processing(&coordinator_agent)?;
    runtime.simulate_message_processing(&monitor_agent)?;
    runtime.simulate_message_processing(&worker_agent)?; // Process another message
    println!();

    // Check updated metadata and resource usage
    println!("5️⃣ Checking resource usage...");
    for agent_id in [&worker_agent, &coordinator_agent, &monitor_agent] {
        let (metadata, resources) = runtime.get_agent_metadata(agent_id)?;
        println!("   Agent {} ({}):", agent_id, metadata.name);
        println!("     State: {:?}", metadata.state);
        println!("     Messages: {}", resources.message_count);
        println!("     CPU time: {}ms", resources.cpu_time_ms);
        println!("     Memory: {} bytes", resources.memory_bytes);
        println!(
            "     Last activity: {}",
            resources.last_activity.format("%H:%M:%S%.3f")
        );
    }
    println!();

    // Show runtime metrics
    println!("6️⃣ Runtime metrics...");
    let (total, active, messages, memory) = runtime.get_runtime_metrics();
    println!("   Total agents: {}", total);
    println!("   Active agents: {}", active);
    println!("   Total messages processed: {}", messages);
    println!("   Total memory used: {} bytes", memory);
    println!();

    // Demonstrate graceful termination
    println!("7️⃣ Terminating agents...");
    runtime.terminate_agent(&worker_agent).await?;
    runtime.terminate_agent(&coordinator_agent).await?;
    runtime.terminate_agent(&monitor_agent).await?;
    println!();

    // Final state check
    println!("8️⃣ Final state verification...");
    for agent_id in [&worker_agent, &coordinator_agent, &monitor_agent] {
        let state = runtime.get_agent_state(agent_id)?;
        println!("   Agent {}: {:?}", agent_id, state);
    }

    let (total, active, messages, memory) = runtime.get_runtime_metrics();
    println!(
        "   Final metrics - Total: {}, Active: {}, Messages: {}, Memory: {} bytes",
        total, active, messages, memory
    );
    println!();

    println!("✨ Agent Lifecycle Demo completed successfully!");
    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    if let Err(e) = demonstrate_agent_lifecycle().await {
        eprintln!("❌ Demo failed: {}", e);
        std::process::exit(1);
    }
}
