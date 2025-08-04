//! Example demonstrating complete agent lifecycle management
//!
//! This example shows how to create, manage, monitor, and terminate agents
//! using the Caxton agent lifecycle management system.

use caxton::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("🚀 Starting Caxton Agent Lifecycle Example");

    // Create runtime configuration
    let config = CaxtonConfig {
        max_agents: 10,
        observability_enabled: true,
        tracing_enabled: true,
        metrics_enabled: true,
        ..Default::default()
    };

    // Initialize the runtime
    let runtime = Arc::new(CaxtonRuntime::new(config).await?);
    println!("✅ Runtime initialized successfully");

    // Create lifecycle manager
    let lifecycle_manager = AgentLifecycleManager::new(runtime.clone()).await?;
    println!("✅ Lifecycle manager created");

    // Create different types of agents
    let agent_configs = vec![
        AgentConfig {
            name: "data-processor".to_string(),
            agent_type: AgentType::Worker,
            capabilities: vec![
                "data-processing".to_string(),
                "batch-processing".to_string(),
                "stream-processing".to_string(),
            ],
            max_memory: Some(128 * 1024 * 1024),     // 128MB
            timeout: Some(Duration::from_secs(300)), // 5 minutes
        },
        AgentConfig {
            name: "system-coordinator".to_string(),
            agent_type: AgentType::Coordinator,
            capabilities: vec![
                "coordination".to_string(),
                "resource-management".to_string(),
                "load-balancing".to_string(),
            ],
            max_memory: Some(64 * 1024 * 1024),      // 64MB
            timeout: Some(Duration::from_secs(600)), // 10 minutes
        },
        AgentConfig {
            name: "health-monitor".to_string(),
            agent_type: AgentType::Monitor,
            capabilities: vec![
                "health-monitoring".to_string(),
                "performance-tracking".to_string(),
                "alerting".to_string(),
            ],
            max_memory: Some(32 * 1024 * 1024),      // 32MB
            timeout: Some(Duration::from_secs(120)), // 2 minutes
        },
    ];

    // Create all agents
    let mut agent_ids = Vec::new();
    for config in agent_configs {
        println!("🔄 Creating agent: {}", config.name);
        let agent_id = lifecycle_manager.create_agent(config).await?;
        agent_ids.push(agent_id.clone());
        println!("✅ Agent created with ID: {}", agent_id);
    }

    // Wait for agents to initialize
    sleep(Duration::from_secs(2)).await;

    // Display initial system statistics
    let stats = lifecycle_manager.get_lifecycle_stats().await;
    println!("\n📊 System Statistics:");
    println!("  Total agents: {}", stats.total_agents);
    println!("  Agents by state: {:?}", stats.agents_by_state);
    println!(
        "  Capability distribution: {:?}",
        stats.capability_distribution
    );
    println!("  Recovery enabled: {}", stats.recovery_enabled);

    // Demonstrate agent status monitoring
    println!("\n🔍 Agent Status Reports:");
    for agent_id in &agent_ids {
        let status = lifecycle_manager.get_agent_status(agent_id).await?;
        println!(
            "  Agent {}: {:?} - {} capabilities",
            agent_id,
            status.state,
            status.metadata.capabilities.len()
        );
        println!(
            "    Health: {}",
            if status.health_status.healthy {
                "✅ Healthy"
            } else {
                "❌ Unhealthy"
            }
        );
        println!("    Memory: {} bytes", status.resource_usage.memory_bytes);
        println!("    Messages: {}", status.resource_usage.message_count);
    }

    // Demonstrate message sending and processing
    println!("\n📤 Sending messages to agents...");
    for agent_id in &agent_ids {
        let message = FipaMessage {
            performative: FipaPerformative::Request,
            sender: AgentId::system(),
            receiver: agent_id.clone(),
            content: serde_json::json!({
                "task": "process_data",
                "priority": "high",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            conversation_id: Some(ConversationId::new()),
            reply_with: Some(format!("req-{}", uuid::Uuid::new_v4())),
            ..Default::default()
        };

        match runtime.send_message(message).await {
            Ok(response) => println!(
                "  ✅ Message sent to {}, got response: {:?}",
                agent_id, response.performative
            ),
            Err(e) => println!("  ❌ Failed to send message to {}: {:?}", agent_id, e),
        }
    }

    // Wait for message processing
    sleep(Duration::from_secs(1)).await;

    // Demonstrate agent suspension and resumption
    if let Some(first_agent) = agent_ids.first() {
        println!("\n⏸️  Suspending agent: {}", first_agent);
        lifecycle_manager
            .suspend_agent(first_agent, "Demonstration suspension".to_string())
            .await?;

        let status = lifecycle_manager.get_agent_status(first_agent).await?;
        println!("  Agent state after suspension: {:?}", status.state);

        sleep(Duration::from_secs(2)).await;

        println!("▶️  Resuming agent: {}", first_agent);
        lifecycle_manager.resume_agent(first_agent).await?;

        let status = lifecycle_manager.get_agent_status(first_agent).await?;
        println!("  Agent state after resumption: {:?}", status.state);
    }

    // Demonstrate resource limit updates
    if let Some(second_agent) = agent_ids.get(1) {
        println!("\n🔧 Updating resource limits for agent: {}", second_agent);
        runtime
            .update_agent_limits(
                second_agent,
                Some(256 * 1024 * 1024),        // 256MB
                Some(Duration::from_secs(900)), // 15 minutes
            )
            .await?;

        let (metadata, _) = runtime.get_agent_metadata(second_agent).await?;
        println!(
            "  Updated max memory: {} bytes",
            metadata
                .properties
                .get("max_memory")
                .unwrap_or(&"N/A".to_string())
        );
    }

    // Demonstrate health checks
    println!("\n🏥 Performing health checks...");
    for agent_id in &agent_ids {
        let health = runtime.health_check_agent(agent_id).await?;
        println!(
            "  Agent {}: {} ({}ms response time)",
            agent_id,
            if health.healthy {
                "✅ Healthy"
            } else {
                "❌ Unhealthy"
            },
            health
                .timestamp
                .signed_duration_since(chrono::Utc::now())
                .num_milliseconds()
                .abs()
        );

        if !health.warnings.is_empty() {
            println!("    Warnings: {:?}", health.warnings);
        }
        if !health.errors.is_empty() {
            println!("    Errors: {:?}", health.errors);
        }
    }

    // Final system statistics
    println!("\n📊 Final System Statistics:");
    let final_stats = lifecycle_manager.get_lifecycle_stats().await;
    println!("  Active agents: {}", final_stats.total_agents);

    let runtime_metrics = runtime.get_metrics();
    println!("  Runtime metrics:");
    println!("    Total spawned: {}", runtime_metrics.0);
    println!("    Currently active: {}", runtime_metrics.1);
    println!("    Messages processed: {}", runtime_metrics.2);
    println!(
        "    Total memory used: {} MB",
        runtime_metrics.3 / 1024 / 1024
    );

    // Demonstrate graceful shutdown
    println!("\n🛑 Initiating graceful shutdown...");

    // Stop all agents gracefully
    for (i, agent_id) in agent_ids.iter().enumerate() {
        println!(
            "  Stopping agent {} ({}/{})",
            agent_id,
            i + 1,
            agent_ids.len()
        );
        match lifecycle_manager
            .stop_agent(agent_id, Some("Example shutdown".to_string()))
            .await
        {
            Ok(()) => println!("    ✅ Agent stopped successfully"),
            Err(e) => println!("    ❌ Failed to stop agent: {:?}", e),
        }
    }

    // Shutdown runtime
    println!("🔄 Shutting down runtime...");
    runtime.shutdown(Duration::from_secs(30)).await?;
    println!("✅ Runtime shutdown complete");

    println!("\n🎉 Agent Lifecycle Example completed successfully!");
    Ok(())
}
