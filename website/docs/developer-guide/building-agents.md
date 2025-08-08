---
title: Building Agents
layout: documentation
description: Comprehensive guide to developing WebAssembly agents for Caxton multi-agent systems.
---

# Building Agents

Learn how to create production-ready WebAssembly agents that integrate seamlessly with the Caxton multi-agent orchestration platform.

## Table of Contents

- [Introduction](#introduction)
- [Prerequisites](#prerequisites)
- [Development Environment](#development-environment)
- [Creating Your First Agent](#creating-your-first-agent)
- [Agent Lifecycle](#agent-lifecycle)
- [FIPA Message Protocol](#fipa-message-protocol)
- [Testing Agents](#testing-agents)
- [Debugging](#debugging)
- [Performance Optimization](#performance-optimization)
- [Examples](#examples)

## Introduction

Caxton agents are WebAssembly (WASM) modules that run in isolated sandboxes within the Caxton orchestration server. Each agent operates independently while participating in coordinated multi-agent workflows through the Foundation for Intelligent Physical Agents (FIPA) messaging protocol.

### Key Benefits

- **Isolation**: Each agent runs in its own secure WebAssembly sandbox
- **Performance**: < 50μs message routing overhead
- **Language Agnostic**: Write agents in any language that compiles to WebAssembly
- **Observable**: Built-in OpenTelemetry support for tracing and metrics
- **Scalable**: Dynamic resource allocation and horizontal scaling

### Agent Architecture

```
┌─────────────────────────────────────┐
│            Caxton Server            │
├─────────────────────────────────────┤
│  Agent Sandbox (WASM Runtime)      │
│  ┌─────────────────────────────┐    │
│  │        Your Agent           │    │
│  │  ┌─────────────────────┐    │    │
│  │  │   Message Handler   │    │    │
│  │  │   State Manager     │    │    │
│  │  │   Business Logic    │    │    │
│  │  │   Tool Integrations │    │    │
│  │  └─────────────────────┘    │    │
│  └─────────────────────────────┘    │
├─────────────────────────────────────┤
│      Message Router & Protocol     │
│      Resource Manager              │
│      Observability Layer           │
└─────────────────────────────────────┘
```

## Prerequisites

### Required Knowledge

- **Rust Programming**: Primary supported language for agent development
- **WebAssembly Concepts**: Understanding of WASM compilation and runtime
- **Message-Passing Systems**: Experience with actor model or similar patterns
- **Protocol Understanding**: Basic familiarity with FIPA ACL or similar agent communication

### Optional Knowledge

- **Distributed Systems**: Helpful for complex multi-agent coordination
- **OpenTelemetry**: For advanced observability and debugging
- **MCP Protocol**: For external tool integrations

### System Requirements

- **Rust Toolchain**: Version 1.70+ with WebAssembly target
- **Caxton Server**: Development or production installation
- **Development Tools**: IDE with Rust and WASM support

## Development Environment

### Install Rust and WebAssembly Target

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-wasi

# Install helpful tools
cargo install wasm-pack
cargo install cargo-generate
```

### Project Setup

```bash
# Create new agent project
cargo new --lib my_agent
cd my_agent

# Configure Cargo.toml for WASM
cat >> Cargo.toml << EOF

[lib]
crate-type = ["cdylib"]

[dependencies]
caxton-agent = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["rt"] }

[dependencies.web-sys]
version = "0.3"
features = []
EOF
```

### IDE Configuration

#### VS Code Setup

```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.target": "wasm32-wasi",
  "rust-analyzer.checkOnSave.allTargets": false,
  "rust-analyzer.cargo.features": ["wasm"]
}
```

#### IntelliJ/CLion Setup

```xml
<!-- .idea/codeStyles/Project.xml -->
<component name="ProjectCodeStyleConfiguration">
  <option name="PREFERRED_PROJECT_CODE_STYLE" value="Default" />
  <option name="USE_PER_PROJECT_SETTINGS" value="true" />
  <option name="TARGET_TRIPLE" value="wasm32-wasi" />
</component>
```

## Creating Your First Agent

### Basic Agent Structure

Create a simple echo agent that responds to messages:

```rust
// src/lib.rs
use caxton_agent::{
    Agent, AgentContext, FipaMessage, MessageHandler, 
    AgentResult, AgentError
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoAgent {
    id: String,
    state: HashMap<String, String>,
}

impl EchoAgent {
    // Agent metadata - defined in code, not configuration
    const VERSION: &'static str = "1.0.0";
    const NAME: &'static str = "echo-agent";
    
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: HashMap::new(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Agent for EchoAgent {
    async fn initialize(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        tracing::info!("Echo agent {} initializing", self.id);
        
        // Register capabilities that this agent provides
        ctx.register_capability("echo").await?;
        ctx.register_capability("state_management").await?;
        
        Ok(())
    }

    async fn handle_message(
        &mut self,
        message: FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        match message.performative.as_str() {
            "request" => self.handle_request(message, ctx).await,
            "inform" => self.handle_inform(message, ctx).await,
            "query" => self.handle_query(message, ctx).await,
            _ => {
                tracing::warn!("Unsupported performative: {}", message.performative);
                self.send_not_understood(&message, ctx).await
            }
        }
    }

    async fn cleanup(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        tracing::info!("Echo agent {} cleaning up", self.id);
        // Persist state, close connections, etc.
        Ok(())
    }
}

impl EchoAgent {
    async fn handle_request(
        &mut self,
        message: FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        let content = message.content
            .as_object()
            .ok_or(AgentError::InvalidMessage("Missing content".to_string()))?;

        match content.get("action").and_then(|v| v.as_str()) {
            Some("echo") => {
                let text = content.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or(AgentError::InvalidMessage("Missing text".to_string()))?;

                // Echo the message back
                let reply = FipaMessage::new_inform(
                    &self.id,
                    &message.sender,
                    json!({
                        "echoed_text": text,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                )
                .with_conversation_id(message.conversation_id.clone())
                .with_in_reply_to(message.reply_with.clone());

                ctx.send_message(reply).await?;
            }
            Some("store") => {
                let key = content.get("key")
                    .and_then(|v| v.as_str())
                    .ok_or(AgentError::InvalidMessage("Missing key".to_string()))?;
                let value = content.get("value")
                    .and_then(|v| v.as_str())
                    .ok_or(AgentError::InvalidMessage("Missing value".to_string()))?;

                self.state.insert(key.to_string(), value.to_string());

                let reply = FipaMessage::new_inform(
                    &self.id,
                    &message.sender,
                    json!({ "status": "stored", "key": key }),
                )
                .with_conversation_id(message.conversation_id.clone())
                .with_in_reply_to(message.reply_with.clone());

                ctx.send_message(reply).await?;
            }
            _ => {
                self.send_not_understood(&message, ctx).await?;
            }
        }

        Ok(())
    }

    async fn handle_query(
        &mut self,
        message: FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        let content = message.content
            .as_object()
            .ok_or(AgentError::InvalidMessage("Missing content".to_string()))?;

        let key = content.get("key")
            .and_then(|v| v.as_str())
            .ok_or(AgentError::InvalidMessage("Missing key".to_string()))?;

        let reply = if let Some(value) = self.state.get(key) {
            FipaMessage::new_inform(
                &self.id,
                &message.sender,
                json!({ "key": key, "value": value }),
            )
        } else {
            FipaMessage::new_failure(
                &self.id,
                &message.sender,
                json!({ "error": "Key not found", "key": key }),
            )
        }
        .with_conversation_id(message.conversation_id.clone())
        .with_in_reply_to(message.reply_with.clone());

        ctx.send_message(reply).await?;
        Ok(())
    }

    async fn handle_inform(
        &mut self,
        message: FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        // Log received information
        tracing::info!(
            "Received information from {}: {:?}",
            message.sender,
            message.content
        );
        Ok(())
    }

    async fn send_not_understood(
        &self,
        original: &FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        let reply = FipaMessage::new_not_understood(
            &self.id,
            &original.sender,
            json!({
                "reason": "Unsupported action or performative",
                "original_performative": original.performative
            }),
        )
        .with_conversation_id(original.conversation_id.clone())
        .with_in_reply_to(original.reply_with.clone());

        ctx.send_message(reply).await
    }
}

// Export the agent for WASM loading
#[no_mangle]
pub extern "C" fn create_agent(id: *const u8, id_len: usize) -> *mut EchoAgent {
    let id_slice = unsafe { std::slice::from_raw_parts(id, id_len) };
    let id_str = String::from_utf8_lossy(id_slice).to_string();
    
    Box::into_raw(Box::new(EchoAgent::new(id_str)))
}

#[no_mangle]
pub extern "C" fn destroy_agent(agent: *mut EchoAgent) {
    unsafe {
        drop(Box::from_raw(agent));
    }
}
```

### Build Configuration

Create optimized WASM builds:

```bash
# Build for development (with debug info)
cargo build --target wasm32-wasi --release

# Build optimized for production
RUSTFLAGS="-C opt-level=z -C target-feature=+bulk-memory" \
cargo build --target wasm32-wasi --release

# Optimize WASM binary
wasm-opt -Oz -o target/wasm32-wasi/release/my_agent_opt.wasm \
  target/wasm32-wasi/release/my_agent.wasm
```

### Deployment Configuration

Create agent deployment manifest (Note: capabilities are registered in code, not config):

```json
// agent-manifest.json
{
  "name": "echo-agent",
  "resources": {
    "memory": "10MB",
    "cpu": "100m"
  },
  "environment": {
    "LOG_LEVEL": "info",
    "TIMEOUT_MS": "5000"
  },
  "scaling": {
    "min_instances": 1,
    "max_instances": 10,
    "target_cpu_utilization": 70
  }
}
```

**That's it!** The manifest is purely for deployment configuration. Caxton validates manifests against this schema on deployment:

### Manifest JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["name", "resources"],
  "additionalProperties": false,
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-z0-9-]+$",
      "description": "Agent identifier (lowercase, alphanumeric, hyphens)"
    },
    "resources": {
      "type": "object",
      "required": ["memory", "cpu"],
      "additionalProperties": false,
      "properties": {
        "memory": {
          "type": "string",
          "pattern": "^[0-9]+(Mi|Gi|MB|GB)$",
          "description": "Memory limit (e.g., '10MB', '256Mi')"
        },
        "cpu": {
          "type": "string",
          "pattern": "^[0-9]+m?$",
          "description": "CPU limit in millicores (e.g., '100m', '2000m')"
        }
      }
    },
    "environment": {
      "type": "object",
      "additionalProperties": {
        "type": "string"
      },
      "description": "Environment variables as key-value pairs"
    },
    "scaling": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "min_instances": {
          "type": "integer",
          "minimum": 0,
          "default": 1
        },
        "max_instances": {
          "type": "integer",
          "minimum": 1,
          "default": 10
        },
        "target_cpu_utilization": {
          "type": "integer",
          "minimum": 1,
          "maximum": 100,
          "default": 70,
          "description": "Target CPU percentage for autoscaling"
        }
      }
    }
  }
}
```

Invalid manifests are rejected at deployment with clear error messages.

## Capability Registration

Capabilities are registered programmatically in the agent's initialization method:

```rust
async fn initialize(&mut self, ctx: &AgentContext) -> AgentResult<()> {
    // Register what this agent can do
    ctx.register_capability("echo").await?;
    ctx.register_capability("state_management").await?;
    Ok(())
}
```

Capabilities determine which agents receive specific types of messages and tasks.

## Agent Lifecycle

Understanding the agent lifecycle is crucial for building robust agents:

### Lifecycle Phases

```rust
#[async_trait::async_trait(?Send)]
impl Agent for MyAgent {
    // 1. INITIALIZATION
    async fn initialize(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        // Called once when agent is deployed
        // - Load configuration
        // - Initialize state
        // - Register capabilities
        // - Subscribe to topics
        // - Connect to external services
        
        tracing::info!("Agent {} starting initialization", self.id);
        
        // Register what this agent can do
        ctx.register_capability("data_processing").await?;
        ctx.register_capability("file_operations").await?;
        
        // Subscribe to relevant topics
        ctx.subscribe("system.events").await?;
        ctx.subscribe("data.updates").await?;
        
        // Initialize connections
        self.database_client = Some(DatabaseClient::new(
            ctx.get_config("database_url")?
        ).await?);
        
        Ok(())
    }

    // 2. ACTIVE MESSAGE PROCESSING
    async fn handle_message(
        &mut self,
        message: FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        // Called for each received message
        // - Parse message
        // - Validate content
        // - Process request
        // - Send responses
        // - Update state
        
        // Add correlation ID for tracing
        let span = tracing::info_span!(
            "handle_message",
            message_id = %message.message_id,
            performative = %message.performative,
            sender = %message.sender
        );
        
        async move {
            match message.performative.as_str() {
                "request" => self.handle_request(message, ctx).await,
                "inform" => self.handle_inform(message, ctx).await,
                "subscribe" => self.handle_subscription(message, ctx).await,
                _ => self.handle_unknown(message, ctx).await,
            }
        }.instrument(span).await
    }

    // 3. PERIODIC OPERATIONS (Optional)
    async fn on_timer(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        // Called periodically (if configured)
        // - Cleanup expired data
        // - Send periodic reports
        // - Health checks
        // - Maintenance tasks
        
        self.cleanup_expired_cache().await?;
        self.send_health_report(ctx).await?;
        
        Ok(())
    }

    // 4. GRACEFUL SHUTDOWN
    async fn cleanup(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        // Called when agent is being stopped
        // - Finish processing current messages
        // - Save state
        // - Close connections
        // - Release resources
        
        tracing::info!("Agent {} beginning shutdown", self.id);
        
        // Finish processing queued work
        self.process_pending_work(ctx).await?;
        
        // Persist important state
        if let Some(state) = &self.persistent_state {
            ctx.save_state(&self.id, state).await?;
        }
        
        // Close external connections
        if let Some(client) = &mut self.database_client {
            client.close().await?;
        }
        
        Ok(())
    }
}
```

### State Management

Agents can maintain different types of state:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentState {
    // Ephemeral state (lost on restart)
    pub cache: HashMap<String, CacheEntry>,
    pub active_conversations: HashMap<String, ConversationState>,
    
    // Persistent state (survives restart)
    #[serde(skip)]
    pub persistent: PersistentState,
}

impl AgentState {
    pub async fn load(ctx: &AgentContext, agent_id: &str) -> AgentResult<Self> {
        let persistent = ctx.load_state(agent_id)
            .await
            .unwrap_or_default();
            
        Ok(Self {
            cache: HashMap::new(),
            active_conversations: HashMap::new(),
            persistent,
        })
    }
    
    pub async fn save(&self, ctx: &AgentContext, agent_id: &str) -> AgentResult<()> {
        ctx.save_state(agent_id, &self.persistent).await
    }
}
```

## FIPA Message Protocol

Caxton uses the Foundation for Intelligent Physical Agents (FIPA) Agent Communication Language (ACL) for inter-agent messaging.

### Message Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FipaMessage {
    // Required fields
    pub performative: String,      // Message intent
    pub sender: String,           // Sender agent ID
    pub receiver: String,         // Receiver agent ID
    pub content: serde_json::Value, // Message payload
    
    // Optional conversation management
    pub conversation_id: Option<String>,  // Groups related messages
    pub reply_with: Option<String>,       // Expected reply identifier
    pub in_reply_to: Option<String>,      // References previous message
    
    // Protocol and semantic information
    pub ontology: Option<String>,    // Domain vocabulary
    pub language: Option<String>,    // Content language
    pub protocol: Option<String>,    // Interaction protocol
    
    // System fields (managed by Caxton)
    pub message_id: String,         // Unique message identifier
    pub timestamp: DateTime<Utc>,   // Message creation time
}
```

### Common Performatives

```rust
impl FipaMessage {
    // Request another agent to perform an action
    pub fn new_request(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "request", content)
    }
    
    // Inform about facts or events
    pub fn new_inform(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "inform", content)
    }
    
    // Ask for information
    pub fn new_query(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "query-if", content)
    }
    
    // Positive response to a request
    pub fn new_agree(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "agree", content)
    }
    
    // Negative response to a request  
    pub fn new_refuse(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "refuse", content)
    }
    
    // Report successful completion
    pub fn new_inform_done(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "inform-done", content)
    }
    
    // Report failure
    pub fn new_failure(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "failure", content)
    }
    
    // Indicate message not understood
    pub fn new_not_understood(sender: &str, receiver: &str, content: Value) -> Self {
        Self::new(sender, receiver, "not-understood", content)
    }
}
```

### Protocol Examples

#### Request-Response Pattern

```rust
// Requester agent
async fn request_data_processing(
    &self,
    processor_id: &str,
    data: &ProcessingRequest,
    ctx: &AgentContext
) -> AgentResult<ProcessingResponse> {
    let request = FipaMessage::new_request(
        &self.id,
        processor_id,
        serde_json::to_value(data)?
    )
    .with_reply_with(format!("req-{}", Uuid::new_v4()))
    .with_protocol("request-response");

    // Send request and wait for reply
    let reply = ctx.send_and_wait(request, Duration::from_secs(30)).await?;
    
    match reply.performative.as_str() {
        "inform-done" => {
            Ok(serde_json::from_value(reply.content)?)
        }
        "failure" => {
            Err(AgentError::ProcessingFailed(
                reply.content.to_string()
            ))
        }
        _ => {
            Err(AgentError::UnexpectedResponse(reply.performative))
        }
    }
}

// Processor agent
async fn handle_processing_request(
    &mut self,
    message: FipaMessage,
    ctx: &AgentContext
) -> AgentResult<()> {
    let request: ProcessingRequest = serde_json::from_value(message.content)?;
    
    // Process the data
    match self.process_data(&request).await {
        Ok(result) => {
            let reply = FipaMessage::new_inform_done(
                &self.id,
                &message.sender,
                serde_json::to_value(result)?
            )
            .with_conversation_id(message.conversation_id.clone())
            .with_in_reply_to(message.reply_with.clone());
            
            ctx.send_message(reply).await?;
        }
        Err(error) => {
            let reply = FipaMessage::new_failure(
                &self.id,
                &message.sender,
                json!({ "error": error.to_string() })
            )
            .with_conversation_id(message.conversation_id.clone())
            .with_in_reply_to(message.reply_with.clone());
            
            ctx.send_message(reply).await?;
        }
    }
    
    Ok(())
}
```

#### Contract Net Protocol

```rust
// Task initiator
async fn distribute_task(
    &self,
    task: &Task,
    participants: &[String],
    ctx: &AgentContext
) -> AgentResult<String> {
    let conversation_id = format!("cnp-{}", Uuid::new_v4());
    
    // Send call for proposals
    let cfp_content = json!({
        "task": task,
        "deadline": chrono::Utc::now() + chrono::Duration::minutes(5)
    });
    
    for participant in participants {
        let cfp = FipaMessage::new_cfp(&self.id, participant, cfp_content.clone())
            .with_conversation_id(&conversation_id)
            .with_protocol("fipa-contract-net");
        
        ctx.send_message(cfp).await?;
    }
    
    // Collect proposals
    let proposals = ctx.collect_responses(
        &conversation_id,
        "propose",
        participants.len(),
        Duration::from_secs(30)
    ).await?;
    
    // Select best proposal
    let best_proposal = self.evaluate_proposals(&proposals)?;
    
    // Accept winning proposal, reject others
    for (participant, proposal) in proposals {
        if participant == best_proposal.sender {
            let accept = FipaMessage::new_accept_proposal(
                &self.id,
                &participant,
                json!({ "accepted": true })
            )
            .with_conversation_id(&conversation_id)
            .with_in_reply_to(proposal.reply_with.clone());
            
            ctx.send_message(accept).await?;
        } else {
            let reject = FipaMessage::new_reject_proposal(
                &self.id,
                &participant,
                json!({ "reason": "Better proposal selected" })
            )
            .with_conversation_id(&conversation_id)
            .with_in_reply_to(proposal.reply_with.clone());
            
            ctx.send_message(reject).await?;
        }
    }
    
    Ok(best_proposal.sender)
}
```

## Testing Agents

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use caxton_agent::testing::{TestAgentContext, TestMessage};
    use tokio_test;

    #[tokio::test]
    async fn test_echo_functionality() {
        let mut agent = EchoAgent::new("test-agent".to_string());
        let ctx = TestAgentContext::new();
        
        // Initialize agent
        agent.initialize(&ctx).await.unwrap();
        
        // Create test message
        let message = TestMessage::new_request(
            "client",
            "test-agent",
            json!({
                "action": "echo",
                "text": "Hello, World!"
            })
        );
        
        // Handle message
        agent.handle_message(message.into(), &ctx).await.unwrap();
        
        // Verify response
        let sent_messages = ctx.get_sent_messages();
        assert_eq!(sent_messages.len(), 1);
        
        let response = &sent_messages[0];
        assert_eq!(response.performative, "inform");
        assert_eq!(
            response.content["echoed_text"].as_str().unwrap(),
            "Hello, World!"
        );
    }

    #[tokio::test]
    async fn test_state_management() {
        let mut agent = EchoAgent::new("test-agent".to_string());
        let ctx = TestAgentContext::new();
        
        agent.initialize(&ctx).await.unwrap();
        
        // Store a value
        let store_msg = TestMessage::new_request(
            "client",
            "test-agent",
            json!({
                "action": "store",
                "key": "test-key",
                "value": "test-value"
            })
        );
        
        agent.handle_message(store_msg.into(), &ctx).await.unwrap();
        
        // Query the value
        let query_msg = TestMessage::new_query(
            "client",
            "test-agent",
            json!({ "key": "test-key" })
        );
        
        agent.handle_message(query_msg.into(), &ctx).await.unwrap();
        
        // Verify stored and retrieved value
        let responses = ctx.get_sent_messages();
        assert_eq!(responses.len(), 2);
        
        let query_response = &responses[1];
        assert_eq!(query_response.performative, "inform");
        assert_eq!(
            query_response.content["value"].as_str().unwrap(),
            "test-value"
        );
    }
}
```

### Integration Testing

```rust
// tests/integration_test.rs
use caxton_client::CaxtonClient;
use std::time::Duration;
use tokio;

#[tokio::test]
async fn test_agent_deployment_and_communication() {
    // Start test Caxton server
    let server = caxton_testing::TestServer::new().await;
    let client = CaxtonClient::new(server.endpoint()).await.unwrap();
    
    // Deploy test agent
    let wasm_bytes = include_bytes!("../target/wasm32-wasi/release/echo_agent.wasm");
    let agent = client.deploy_agent(
        wasm_bytes,
        AgentConfig {
            name: "integration-test-agent".to_string(),
            resources: ResourceLimits {
                memory: "10MB".to_string(),
                cpu: "100m".to_string(),
            },
            ..Default::default()
        }
    ).await.unwrap();
    
    // Wait for agent to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Send test message
    let response = client.send_message_and_wait(
        FipaMessage::new_request(
            "integration-test",
            &agent.id,
            json!({
                "action": "echo",
                "text": "Integration test message"
            })
        ),
        Duration::from_secs(5)
    ).await.unwrap();
    
    // Verify response
    assert_eq!(response.performative, "inform");
    assert_eq!(
        response.content["echoed_text"].as_str().unwrap(),
        "Integration test message"
    );
    
    // Cleanup
    client.remove_agent(&agent.id).await.unwrap();
}
```

### Performance Testing

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use caxton_agent::testing::TestAgentContext;

fn bench_message_processing(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("echo_message_processing", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut agent = EchoAgent::new("bench-agent".to_string());
            let ctx = TestAgentContext::new();
            
            agent.initialize(&ctx).await.unwrap();
            
            let message = TestMessage::new_request(
                "client",
                "bench-agent",
                json!({
                    "action": "echo",
                    "text": black_box("Benchmark message")
                })
            );
            
            agent.handle_message(message.into(), &ctx).await.unwrap();
        });
    });
}

criterion_group!(benches, bench_message_processing);
criterion_main!(benches);
```

## Debugging

### Logging and Tracing

```rust
use tracing::{info, warn, error, debug, instrument};

impl EchoAgent {
    #[instrument(skip(self, ctx), fields(agent_id = %self.id))]
    async fn handle_request(
        &mut self,
        message: FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        debug!("Processing request: {:?}", message.content);
        
        let processing_start = std::time::Instant::now();
        
        // ... processing logic ...
        
        let processing_time = processing_start.elapsed();
        info!(
            processing_time_ms = processing_time.as_millis(),
            "Request processed successfully"
        );
        
        Ok(())
    }
}

// Enable structured logging
#[no_mangle]
pub extern "C" fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("my_agent=debug".parse().unwrap())
        )
        .json()
        .init();
}
```

### OpenTelemetry Integration

```rust
use opentelemetry::{trace::Tracer, global};
use tracing_opentelemetry::OpenTelemetryLayer;

#[instrument(skip(self, ctx))]
async fn complex_operation(
    &self,
    data: &ProcessingData,
    ctx: &AgentContext
) -> AgentResult<ProcessingResult> {
    let tracer = global::tracer("echo-agent");
    
    let span = tracer.start("data_processing");
    let _guard = span.set_current();
    
    // Add custom attributes
    span.set_attribute("data_size", data.size() as i64);
    span.set_attribute("operation_type", "echo");
    
    // Simulate processing with child spans
    let result = {
        let child_span = tracer.start("validation");
        let _child_guard = child_span.set_current();
        
        self.validate_data(data).await?
    };
    
    {
        let child_span = tracer.start("processing");
        let _child_guard = child_span.set_current();
        
        self.process_validated_data(&result).await
    }
}
```

### Debug Tools

```bash
# View agent logs in real-time
caxton logs --agent echo-agent --follow

# Get agent metrics
caxton metrics --agent echo-agent --period 1h

# Trace specific message flows
caxton trace --conversation-id conv-123 --format json

# Debug WebAssembly issues
RUST_LOG=debug caxton agent deploy --debug-wasm agent.wasm

# Memory debugging
caxton debug memory --agent echo-agent --dump-heap
```

### Common Issues and Solutions

#### Memory Leaks

```rust
// ❌ Common mistake - holding references too long
struct BadAgent {
    message_cache: HashMap<String, FipaMessage>, // Never cleaned
}

// ✅ Proper memory management
struct GoodAgent {
    message_cache: LruCache<String, FipaMessage>, // Auto-eviction
}

impl GoodAgent {
    async fn on_timer(&mut self, _ctx: &AgentContext) -> AgentResult<()> {
        // Regular cleanup
        self.message_cache.clear_expired();
        Ok(())
    }
}
```

#### Deadlock Prevention

```rust
// ❌ Potential deadlock - nested message sending
async fn bad_request_handler(&mut self, msg: FipaMessage, ctx: &AgentContext) {
    let response = ctx.send_and_wait(/* another message */).await?; // Deadlock risk
    // Process response...
}

// ✅ Async coordination
async fn good_request_handler(&mut self, msg: FipaMessage, ctx: &AgentContext) {
    // Schedule async operation
    let task_handle = ctx.spawn_task(async move {
        // Process without blocking message handler
    });
    
    // Store task handle for later retrieval
    self.pending_tasks.insert(msg.message_id, task_handle);
}
```

## Performance Optimization

### Memory Optimization

```rust
// Use memory-efficient data structures
use im::{HashMap as ImHashMap, Vector as ImVector}; // Immutable collections
use compact_str::CompactString; // String optimization
use smallvec::SmallVec; // Stack-allocated vectors

#[derive(Debug)]
pub struct OptimizedAgent {
    // Use compact strings for small text
    id: CompactString,
    
    // Immutable collections for shared state
    config: ImHashMap<CompactString, String>,
    
    // Stack-allocated for small collections
    recent_messages: SmallVec<[MessageId; 8]>,
    
    // Pool reusable objects
    message_pool: Vec<FipaMessage>,
}

impl OptimizedAgent {
    fn get_pooled_message(&mut self) -> FipaMessage {
        self.message_pool.pop()
            .unwrap_or_else(|| FipaMessage::default())
    }
    
    fn return_message(&mut self, mut msg: FipaMessage) {
        // Clear and return to pool
        msg.clear();
        self.message_pool.push(msg);
    }
}
```

### CPU Optimization

```rust
// Batch processing for efficiency
impl Agent for BatchProcessor {
    async fn handle_message(&mut self, message: FipaMessage, ctx: &AgentContext) -> AgentResult<()> {
        // Add to batch instead of processing immediately
        self.message_batch.push(message);
        
        // Process in batches
        if self.message_batch.len() >= BATCH_SIZE {
            self.process_batch(ctx).await?;
        }
        
        Ok(())
    }
    
    async fn process_batch(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        let messages = std::mem::take(&mut self.message_batch);
        
        // Process all messages in parallel
        let results: Vec<_> = stream::iter(messages)
            .map(|msg| self.process_single_message(msg, ctx))
            .buffer_unordered(10) // Limit concurrency
            .collect()
            .await;
        
        // Handle results...
        Ok(())
    }
}
```

### WebAssembly Optimization

```bash
# Optimize Rust compilation
export RUSTFLAGS="-C target-feature=+bulk-memory,+mutable-globals,+sign-ext"
cargo build --target wasm32-wasi --release

# Post-process WASM
wasm-opt -Oz --enable-bulk-memory \
  --enable-mutable-globals \
  --enable-sign-ext \
  -o optimized.wasm \
  target.wasm

# Profile WASM execution
caxton profile --agent my-agent --duration 60s --output profile.json
```

### Monitoring and Metrics

```rust
use caxton_agent::metrics::{Counter, Histogram, Gauge};

#[derive(Debug)]
pub struct InstrumentedAgent {
    // Performance metrics
    messages_processed: Counter,
    processing_duration: Histogram,
    active_conversations: Gauge,
    
    // Business metrics
    successful_operations: Counter,
    failed_operations: Counter,
}

impl InstrumentedAgent {
    pub fn new(id: String) -> Self {
        Self {
            messages_processed: Counter::new("messages_processed")
                .with_tag("agent_id", id.clone()),
            processing_duration: Histogram::new("message_processing_duration_ms")
                .with_tag("agent_id", id.clone()),
            active_conversations: Gauge::new("active_conversations")
                .with_tag("agent_id", id.clone()),
            successful_operations: Counter::new("operations_total")
                .with_tags([("agent_id", id.clone()), ("status", "success")]),
            failed_operations: Counter::new("operations_total")
                .with_tags([("agent_id", id), ("status", "failure")]),
        }
    }
}

impl Agent for InstrumentedAgent {
    async fn handle_message(&mut self, message: FipaMessage, ctx: &AgentContext) -> AgentResult<()> {
        let start_time = std::time::Instant::now();
        self.messages_processed.increment(1);
        
        let result = self.process_message_impl(message, ctx).await;
        
        let duration = start_time.elapsed();
        self.processing_duration.record(duration.as_millis() as f64);
        
        match result {
            Ok(_) => self.successful_operations.increment(1),
            Err(_) => self.failed_operations.increment(1),
        }
        
        result
    }
}
```

## Examples

### Simple Calculator Agent

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatorAgent {
    id: String,
    precision: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculationRequest {
    operation: String,      // "add", "subtract", "multiply", "divide"
    operands: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalculationResult {
    result: f64,
    operation: String,
    operands: Vec<f64>,
}

impl CalculatorAgent {
    async fn handle_calculation_request(
        &self,
        request: CalculationRequest,
        message: &FipaMessage,
        ctx: &AgentContext,
    ) -> AgentResult<()> {
        let result = match request.operation.as_str() {
            "add" => request.operands.iter().sum(),
            "subtract" => {
                request.operands.iter()
                    .reduce(|acc, x| acc - x)
                    .unwrap_or(0.0)
            },
            "multiply" => {
                request.operands.iter()
                    .reduce(|acc, x| acc * x)
                    .unwrap_or(0.0)
            },
            "divide" => {
                request.operands.iter()
                    .reduce(|acc, x| if *x != 0.0 { acc / x } else { f64::NAN })
                    .unwrap_or(f64::NAN)
            },
            _ => {
                let error_response = FipaMessage::new_not_understood(
                    &self.id,
                    &message.sender,
                    json!({ "error": "Unknown operation", "operation": request.operation }),
                );
                ctx.send_message(error_response).await?;
                return Ok(());
            }
        };

        let response = CalculationResult {
            result,
            operation: request.operation,
            operands: request.operands,
        };

        let reply = FipaMessage::new_inform(
            &self.id,
            &message.sender,
            serde_json::to_value(response)?,
        )
        .with_conversation_id(message.conversation_id.clone())
        .with_in_reply_to(message.reply_with.clone());

        ctx.send_message(reply).await
    }
}
```

### Data Processing Pipeline Agent

```rust
use futures::StreamExt;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct DataPipelineAgent {
    id: String,
    processors: Vec<Box<dyn DataProcessor>>,
    input_queue: mpsc::Receiver<DataItem>,
    output_queue: mpsc::Sender<ProcessedData>,
}

trait DataProcessor: Send + Sync {
    async fn process(&self, data: DataItem) -> Result<DataItem, ProcessingError>;
}

impl DataPipelineAgent {
    async fn run_pipeline(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        while let Some(data_item) = self.input_queue.recv().await {
            let mut current_data = data_item;
            
            // Process through pipeline stages
            for processor in &self.processors {
                match processor.process(current_data).await {
                    Ok(processed) => current_data = processed,
                    Err(error) => {
                        tracing::error!("Pipeline processing failed: {:?}", error);
                        
                        // Notify about failure
                        let failure_msg = FipaMessage::new_inform(
                            &self.id,
                            "pipeline-monitor",
                            json!({
                                "event": "processing_failed",
                                "error": error.to_string(),
                                "stage": processor.name()
                            }),
                        );
                        
                        ctx.send_message(failure_msg).await?;
                        continue;
                    }
                }
            }
            
            // Send processed data
            if let Err(_) = self.output_queue.try_send(ProcessedData::from(current_data)) {
                tracing::warn!("Output queue full, dropping processed data");
            }
        }
        
        Ok(())
    }
}
```

### Multi-Agent Coordination Example

```rust
// Coordinator agent that manages a team of worker agents
#[derive(Debug)]
pub struct TeamCoordinator {
    id: String,
    workers: Vec<String>,
    task_queue: VecDeque<Task>,
    active_assignments: HashMap<String, Assignment>,
}

impl TeamCoordinator {
    async fn distribute_work(&mut self, ctx: &AgentContext) -> AgentResult<()> {
        while let Some(task) = self.task_queue.pop_front() {
            // Find available worker
            let available_worker = self.find_available_worker(ctx).await?;
            
            if let Some(worker_id) = available_worker {
                // Assign task
                let assignment = Assignment {
                    task_id: task.id.clone(),
                    worker_id: worker_id.clone(),
                    started_at: chrono::Utc::now(),
                };
                
                let work_request = FipaMessage::new_request(
                    &self.id,
                    &worker_id,
                    serde_json::to_value(&task)?,
                )
                .with_conversation_id(&task.id)
                .with_protocol("work-assignment");
                
                ctx.send_message(work_request).await?;
                self.active_assignments.insert(task.id.clone(), assignment);
            } else {
                // No workers available, put task back
                self.task_queue.push_front(task);
                break;
            }
        }
        
        Ok(())
    }
    
    async fn find_available_worker(&self, ctx: &AgentContext) -> AgentResult<Option<String>> {
        for worker_id in &self.workers {
            // Query worker status
            let status_query = FipaMessage::new_query(
                &self.id,
                worker_id,
                json!({ "query": "status" })
            );
            
            let response = ctx.send_and_wait(
                status_query,
                Duration::from_secs(5)
            ).await?;
            
            if let Some(available) = response.content.get("available") {
                if available.as_bool().unwrap_or(false) {
                    return Ok(Some(worker_id.clone()));
                }
            }
        }
        
        Ok(None)
    }
}
```

## Next Steps

Now that you understand the fundamentals of building agents for Caxton:

1. **Start Simple**: Begin with the echo agent example and gradually add functionality
2. **Read the API Reference**: Familiarize yourself with the complete [API documentation]({{ '/docs/developer-guide/api-reference/' | relative_url }})
3. **Study Examples**: Explore the [examples repository]({{ site.social.github }}/tree/main/examples) for more complex agent patterns
4. **Join the Community**: Participate in [GitHub Discussions]({{ site.social.github }}/discussions) to share experiences and get help
5. **Contribute**: Help improve Caxton by contributing to the [project]({{ site.social.github }})

For advanced topics, see:
- [Message Protocols]({{ '/docs/developer-guide/message-protocols/' | relative_url }}) - Deep dive into FIPA protocols
- [WebAssembly Integration]({{ '/docs/developer-guide/wasm-integration/' | relative_url }}) - Advanced WASM techniques
- [DevOps & Security Guide]({{ '/docs/operations/devops-security-guide/' | relative_url }}) - Production deployment