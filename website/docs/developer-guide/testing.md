---
title: Testing Strategies and Best Practices
layout: documentation
description: Comprehensive guide to testing multi-agent systems in Caxton, including unit tests, integration tests, and load testing strategies.
---

# Testing Strategies and Best Practices

Testing multi-agent systems presents unique challenges due to their distributed, asynchronous, and interactive nature. This guide provides comprehensive strategies for testing Caxton agents and the platform itself, ensuring reliability, performance, and correctness.

## Testing Philosophy

### Multi-Layer Testing Strategy

```
┌─────────────────────────────────────────────────────────┐
│ End-to-End Tests                                       │
│ • Full system scenarios                                │
│ • Multi-agent workflows                                │
│ • Performance under load                               │
└─────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────┐
│ Integration Tests                                      │
│ • Agent-to-agent communication                         │
│ • API endpoint testing                                 │
│ • Protocol compliance                                  │
└─────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────┐
│ Component Tests                                        │
│ • WASM agent testing                                   │
│ • Message routing                                      │
│ • Resource management                                  │
└─────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────┐
│ Unit Tests                                             │
│ • Individual functions                                 │
│ • Data structures                                      │
│ • Algorithm correctness                                │
└─────────────────────────────────────────────────────────┘
```

### Testing Principles

1. **Isolation**: Each test should be independent and not affect others
2. **Repeatability**: Tests must produce consistent results across environments
3. **Observability**: Tests should provide clear failure diagnosis
4. **Performance**: Tests should execute quickly to enable frequent runs
5. **Realism**: Test scenarios should reflect real-world usage patterns

## Unit Testing

### Testing WASM Agents

#### Test Framework Setup

```rust
// tests/test_framework.rs
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockHost {
    sent_messages: Arc<Mutex<Vec<FipaMessage>>>,
    stored_data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    current_timestamp: u64,
}

impl MockHost {
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            stored_data: Arc::new(Mutex::new(HashMap::new())),
            current_timestamp: 1642684800000000, // Fixed timestamp for tests
        }
    }
    
    pub fn get_sent_messages(&self) -> Vec<FipaMessage> {
        self.sent_messages.lock().unwrap().clone()
    }
    
    pub fn clear_sent_messages(&self) {
        self.sent_messages.lock().unwrap().clear();
    }
    
    pub fn set_stored_data(&self, key: &str, data: &[u8]) {
        self.stored_data.lock().unwrap().insert(key.to_string(), data.to_vec());
    }
    
    pub fn get_stored_data(&self, key: &str) -> Option<Vec<u8>> {
        self.stored_data.lock().unwrap().get(key).cloned()
    }
}

// Mock host functions
static mut MOCK_HOST: Option<MockHost> = None;

#[no_mangle]
pub extern "C" fn send_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    let msg_bytes = unsafe { std::slice::from_raw_parts(msg_ptr, msg_len) };
    
    if let Ok(message) = serde_json::from_slice::<FipaMessage>(msg_bytes) {
        unsafe {
            if let Some(ref host) = MOCK_HOST {
                host.sent_messages.lock().unwrap().push(message);
                return 0;
            }
        }
    }
    
    1
}

#[no_mangle]
pub extern "C" fn current_timestamp() -> u64 {
    unsafe {
        MOCK_HOST.as_ref().map_or(0, |host| host.current_timestamp)
    }
}

#[no_mangle]
pub extern "C" fn store_data(key_ptr: *const u8, key_len: usize, 
                             data_ptr: *const u8, data_len: usize) -> i32 {
    let key_bytes = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };
    let data_bytes = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
    
    if let Ok(key) = std::str::from_utf8(key_bytes) {
        unsafe {
            if let Some(ref host) = MOCK_HOST {
                host.set_stored_data(key, data_bytes);
                return 0;
            }
        }
    }
    
    1
}

pub fn setup_test_environment() -> MockHost {
    let host = MockHost::new();
    unsafe {
        MOCK_HOST = Some(MockHost::new());
    }
    host
}

pub fn teardown_test_environment() {
    unsafe {
        MOCK_HOST = None;
    }
}
```

#### Agent Unit Tests

```rust
// tests/agent_tests.rs
use super::test_framework::*;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_initialization() {
        let _host = setup_test_environment();
        
        // Test agent initialization
        let result = agent_init();
        assert_eq!(result, 0, "Agent initialization should succeed");
        
        teardown_test_environment();
    }
    
    #[test]
    fn test_simple_request_response() {
        let host = setup_test_environment();
        
        // Initialize agent
        agent_init();
        
        // Create test request message
        let request = json!({
            "performative": "request",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {
                "action": "echo",
                "data": "hello world"
            },
            "reply_with": "test_001"
        });
        
        let request_bytes = serde_json::to_vec(&request).unwrap();
        
        // Send message to agent
        let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
        assert_eq!(result, 0, "Message handling should succeed");
        
        // Check that agent sent a response
        let sent_messages = host.get_sent_messages();
        assert_eq!(sent_messages.len(), 1, "Agent should send one response");
        
        let response = &sent_messages[0];
        assert_eq!(response.performative, "inform");
        assert_eq!(response.receiver, "test_client");
        assert_eq!(response.in_reply_to.as_ref().unwrap(), "test_001");
        assert_eq!(response.content["result"], "hello world");
        
        teardown_test_environment();
    }
    
    #[test]
    fn test_invalid_message_handling() {
        let host = setup_test_environment();
        agent_init();
        
        // Send invalid JSON
        let invalid_json = b"invalid json data";
        let result = handle_message(invalid_json.as_ptr(), invalid_json.len());
        assert_ne!(result, 0, "Invalid JSON should be rejected");
        
        // Send message with invalid performative
        let invalid_performative = json!({
            "performative": "invalid_performative",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {}
        });
        
        let msg_bytes = serde_json::to_vec(&invalid_performative).unwrap();
        let result = handle_message(msg_bytes.as_ptr(), msg_bytes.len());
        
        // Should send NOT_UNDERSTOOD
        let sent_messages = host.get_sent_messages();
        assert!(!sent_messages.is_empty(), "Agent should respond to invalid performative");
        assert_eq!(sent_messages[0].performative, "not_understood");
        
        teardown_test_environment();
    }
    
    #[test]
    fn test_data_processing_operations() {
        let host = setup_test_environment();
        agent_init();
        
        let test_cases = vec![
            ("uppercase", "hello", "HELLO"),
            ("lowercase", "WORLD", "world"),
            ("reverse", "abc", "cba"),
            ("length", "test", "4"),
        ];
        
        for (operation, input, expected) in test_cases {
            host.clear_sent_messages();
            
            let request = json!({
                "performative": "request",
                "sender": "test_client",
                "receiver": "test_agent",
                "content": {
                    "action": "process_text",
                    "operation": operation,
                    "text": input
                },
                "reply_with": format!("test_{}", operation)
            });
            
            let request_bytes = serde_json::to_vec(&request).unwrap();
            let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
            
            assert_eq!(result, 0, "Operation {} should succeed", operation);
            
            let sent_messages = host.get_sent_messages();
            assert_eq!(sent_messages.len(), 1, "Should send one response for {}", operation);
            
            let response = &sent_messages[0];
            assert_eq!(response.performative, "inform");
            assert_eq!(
                response.content["result"].as_str().unwrap(),
                expected,
                "Operation {} should produce correct result",
                operation
            );
        }
        
        teardown_test_environment();
    }
    
    #[test]
    fn test_state_persistence() {
        let host = setup_test_environment();
        agent_init();
        
        // Set some state
        let set_request = json!({
            "performative": "request",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {
                "action": "set_counter",
                "counter_name": "test_counter",
                "value": 42
            },
            "reply_with": "set_001"
        });
        
        let request_bytes = serde_json::to_vec(&set_request).unwrap();
        let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
        assert_eq!(result, 0);
        
        // Verify state was stored
        let stored_data = host.get_stored_data("counters");
        assert!(stored_data.is_some(), "Counter data should be stored");
        
        // Get the state
        host.clear_sent_messages();
        let get_request = json!({
            "performative": "request",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {
                "action": "get_counter",
                "counter_name": "test_counter"
            },
            "reply_with": "get_001"
        });
        
        let request_bytes = serde_json::to_vec(&get_request).unwrap();
        let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
        assert_eq!(result, 0);
        
        let sent_messages = host.get_sent_messages();
        assert_eq!(sent_messages.len(), 1);
        assert_eq!(sent_messages[0].content["value"], 42);
        
        teardown_test_environment();
    }
    
    #[test]
    fn test_resource_limits() {
        let host = setup_test_environment();
        agent_init();
        
        // Test memory allocation limits
        let large_request = json!({
            "performative": "request",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {
                "action": "allocate_memory",
                "size": 100 * 1024 * 1024 // 100MB - should exceed limits
            },
            "reply_with": "memory_test"
        });
        
        let request_bytes = serde_json::to_vec(&large_request).unwrap();
        let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
        
        // Should either fail during processing or send failure response
        if result == 0 {
            let sent_messages = host.get_sent_messages();
            assert_eq!(sent_messages.len(), 1);
            assert_eq!(sent_messages[0].performative, "failure");
        } else {
            assert_ne!(result, 0, "Large allocation should fail");
        }
        
        teardown_test_environment();
    }
    
    #[test]
    fn test_concurrent_message_handling() {
        let host = setup_test_environment();
        agent_init();
        
        // Send multiple messages in sequence
        let message_count = 10;
        for i in 0..message_count {
            let request = json!({
                "performative": "request",
                "sender": "test_client",
                "receiver": "test_agent",
                "content": {
                    "action": "increment_counter",
                    "counter_name": "concurrent_test"
                },
                "reply_with": format!("concurrent_{}", i)
            });
            
            let request_bytes = serde_json::to_vec(&request).unwrap();
            let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
            assert_eq!(result, 0, "Message {} should succeed", i);
        }
        
        // Verify all messages were processed
        let sent_messages = host.get_sent_messages();
        assert_eq!(sent_messages.len(), message_count);
        
        // Check final counter value
        host.clear_sent_messages();
        let get_request = json!({
            "performative": "request",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {
                "action": "get_counter",
                "counter_name": "concurrent_test"
            },
            "reply_with": "get_final"
        });
        
        let request_bytes = serde_json::to_vec(&get_request).unwrap();
        handle_message(request_bytes.as_ptr(), request_bytes.len());
        
        let sent_messages = host.get_sent_messages();
        assert_eq!(sent_messages[0].content["value"], message_count);
        
        teardown_test_environment();
    }
}
```

### Property-Based Testing

Test agents with randomly generated inputs to discover edge cases:

```rust
// tests/property_tests.rs
use proptest::prelude::*;
use serde_json::json;

proptest! {
    #[test]
    fn test_message_content_robustness(
        content in prop::collection::hash_map(
            "[a-zA-Z0-9_]{1,20}",
            any::<serde_json::Value>(),
            0..10
        )
    ) {
        let host = setup_test_environment();
        agent_init();
        
        let request = json!({
            "performative": "request",
            "sender": "property_test",
            "receiver": "test_agent",
            "content": content,
            "reply_with": "property_001"
        });
        
        let request_bytes = serde_json::to_vec(&request).unwrap();
        let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
        
        // Agent should either succeed or fail gracefully (no panics)
        assert!(result == 0 || result != 0, "Agent should handle arbitrary content");
        
        // If it succeeded, should have sent a response
        if result == 0 {
            let sent_messages = host.get_sent_messages();
            assert!(!sent_messages.is_empty(), "Successful handling should send response");
        }
        
        teardown_test_environment();
    }
    
    #[test]
    fn test_text_processing_operations(
        text in "[\\PC]{0,1000}",
        operation in "uppercase|lowercase|reverse|length"
    ) {
        let host = setup_test_environment();
        agent_init();
        
        let request = json!({
            "performative": "request",
            "sender": "property_test",
            "receiver": "test_agent",
            "content": {
                "action": "process_text",
                "operation": operation,
                "text": text
            },
            "reply_with": "prop_test"
        });
        
        let request_bytes = serde_json::to_vec(&request).unwrap();
        let result = handle_message(request_bytes.as_ptr(), request_bytes.len());
        
        // Text processing should always succeed
        prop_assert_eq!(result, 0);
        
        let sent_messages = host.get_sent_messages();
        prop_assert_eq!(sent_messages.len(), 1);
        
        let response = &sent_messages[0];
        prop_assert_eq!(response.performative, "inform");
        
        // Verify operation correctness
        let result_str = response.content["result"].as_str().unwrap();
        match operation {
            "uppercase" => prop_assert_eq!(result_str, text.to_uppercase()),
            "lowercase" => prop_assert_eq!(result_str, text.to_lowercase()),
            "reverse" => prop_assert_eq!(result_str, text.chars().rev().collect::<String>()),
            "length" => prop_assert_eq!(result_str, text.len().to_string()),
            _ => unreachable!()
        }
        
        teardown_test_environment();
    }
}
```

## Integration Testing

### Multi-Agent Communication Testing

```rust
// tests/integration_tests.rs
use caxton_client::*;
use tokio::time::{timeout, Duration, sleep};
use std::fs;

#[tokio::test]
async fn test_agent_to_agent_communication() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Deploy sender agent
    let sender_wasm = fs::read("target/wasm32-wasi/release/sender_agent.wasm").unwrap();
    let sender = client.deploy_agent(DeployAgentRequest {
        wasm_module: sender_wasm,
        config: AgentConfig {
            name: "sender_agent".to_string(),
            capabilities: vec!["message_sending".to_string()],
            ..Default::default()
        },
    }).await.unwrap();
    
    // Deploy receiver agent
    let receiver_wasm = fs::read("target/wasm32-wasi/release/receiver_agent.wasm").unwrap();
    let receiver = client.deploy_agent(DeployAgentRequest {
        wasm_module: receiver_wasm,
        config: AgentConfig {
            name: "receiver_agent".to_string(),
            capabilities: vec!["message_receiving".to_string()],
            ..Default::default()
        },
    }).await.unwrap();
    
    // Wait for agents to be ready
    sleep(Duration::from_millis(500)).await;
    
    // Subscribe to messages from receiver
    let mut message_stream = client.subscribe_to_messages(MessageFilter {
        sender_ids: Some(vec![receiver.agent_id.clone()]),
        performatives: Some(vec!["inform".to_string()]),
        ..Default::default()
    }).await.unwrap();
    
    // Send command to sender agent
    client.send_message(FipaMessage {
        performative: "request".to_string(),
        sender: "integration_test".to_string(),
        receiver: sender.agent_id.clone(),
        content: json!({
            "action": "send_greeting",
            "target_agent": receiver.agent_id,
            "message": "Hello from integration test!"
        }),
        ..Default::default()
    }).await.unwrap();
    
    // Wait for inter-agent communication to complete
    let response = timeout(
        Duration::from_secs(10),
        message_stream.next()
    ).await.unwrap().unwrap();
    
    // Verify the communication chain worked
    assert_eq!(response.sender, receiver.agent_id);
    assert_eq!(response.performative, "inform");
    assert!(response.content["processed_greeting"].as_str().unwrap()
        .contains("Hello from integration test!"));
    
    // Cleanup
    client.remove_agent(&sender.agent_id).await.unwrap();
    client.remove_agent(&receiver.agent_id).await.unwrap();
}

#[tokio::test]
async fn test_contract_net_protocol() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Deploy task coordinator
    let coordinator_wasm = fs::read("target/wasm32-wasi/release/coordinator_agent.wasm").unwrap();
    let coordinator = client.deploy_agent(DeployAgentRequest {
        wasm_module: coordinator_wasm,
        config: AgentConfig {
            name: "task_coordinator".to_string(),
            capabilities: vec!["task_coordination".to_string()],
            ..Default::default()
        },
    }).await.unwrap();
    
    // Deploy multiple worker agents
    let worker_wasm = fs::read("target/wasm32-wasi/release/worker_agent.wasm").unwrap();
    let mut workers = Vec::new();
    
    for i in 0..3 {
        let worker = client.deploy_agent(DeployAgentRequest {
            wasm_module: worker_wasm.clone(),
            config: AgentConfig {
                name: format!("worker_agent_{}", i),
                capabilities: vec!["data_processing".to_string()],
                ..Default::default()
            },
        }).await.unwrap();
        workers.push(worker);
    }
    
    // Wait for all agents to be ready
    sleep(Duration::from_millis(1000)).await;
    
    // Subscribe to task completion messages
    let mut completion_stream = client.subscribe_to_messages(MessageFilter {
        performatives: Some(vec!["inform".to_string()]),
        content_filters: Some(vec![
            ("task_completed".to_string(), serde_json::Value::Bool(true))
        ]),
        ..Default::default()
    }).await.unwrap();
    
    // Start contract net protocol
    let task_request = FipaMessage {
        performative: "request".to_string(),
        sender: "integration_test".to_string(),
        receiver: coordinator.agent_id.clone(),
        content: json!({
            "action": "distribute_task",
            "task": {
                "id": "test_task_001",
                "description": "Process dataset",
                "requirements": {
                    "capability": "data_processing",
                    "deadline": "2024-01-15T18:00:00Z"
                },
                "data": {"size": 1000, "type": "json"}
            },
            "protocol": "contract_net"
        }),
        conversation_id: Some("contract_net_test_001".to_string()),
        ..Default::default()
    };
    
    client.send_message(task_request).await.unwrap();
    
    // Wait for task completion (contract net + execution)
    let completion_msg = timeout(
        Duration::from_secs(30),
        completion_stream.next()
    ).await.unwrap().unwrap();
    
    // Verify task was completed successfully
    assert_eq!(completion_msg.performative, "inform");
    assert_eq!(completion_msg.content["task_id"], "test_task_001");
    assert_eq!(completion_msg.content["status"], "completed");
    assert!(completion_msg.content["result"].is_object());
    
    // Cleanup all agents
    client.remove_agent(&coordinator.agent_id).await.unwrap();
    for worker in workers {
        client.remove_agent(&worker.agent_id).await.unwrap();
    }
}

#[tokio::test]
async fn test_message_ordering_and_delivery() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Deploy sequence processor agent
    let processor_wasm = fs::read("target/wasm32-wasi/release/sequence_processor.wasm").unwrap();
    let processor = client.deploy_agent(DeployAgentRequest {
        wasm_module: processor_wasm,
        config: AgentConfig {
            name: "sequence_processor".to_string(),
            ..Default::default()
        },
    }).await.unwrap();
    
    sleep(Duration::from_millis(200)).await;
    
    // Send sequence of numbered messages
    let message_count = 50;
    for i in 0..message_count {
        let message = FipaMessage {
            performative: "inform".to_string(),
            sender: "integration_test".to_string(),
            receiver: processor.agent_id.clone(),
            content: json!({
                "sequence_number": i,
                "data": format!("Message {}", i)
            }),
            ..Default::default()
        };
        
        client.send_message(message).await.unwrap();
        
        // Small delay to test ordering
        if i % 5 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    
    // Request sequence verification
    sleep(Duration::from_millis(500)).await;
    let verification_response = client.send_message_and_wait(
        FipaMessage {
            performative: "request".to_string(),
            sender: "integration_test".to_string(),
            receiver: processor.agent_id.clone(),
            content: json!({
                "action": "verify_sequence",
                "expected_count": message_count
            }),
            reply_with: Some("sequence_check".to_string()),
            ..Default::default()
        },
        Duration::from_secs(10)
    ).await.unwrap();
    
    // Verify all messages were received in order
    assert_eq!(verification_response.performative, "inform");
    assert_eq!(verification_response.content["messages_received"], message_count);
    assert_eq!(verification_response.content["sequence_valid"], true);
    
    client.remove_agent(&processor.agent_id).await.unwrap();
}
```

### API Testing

```rust
// tests/api_tests.rs
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_agent_lifecycle_api() {
    let client = Client::new();
    let base_url = "http://localhost:8080/api/v1";
    
    // Load test agent WASM
    let wasm_bytes = std::fs::read("target/wasm32-wasi/release/test_agent.wasm").unwrap();
    let wasm_b64 = base64::encode(&wasm_bytes);
    
    // Test agent deployment
    // Note: capabilities are registered in code, not config
    // In test code, capabilities should be mocked/stubbed rather than configured via JSON
    let deploy_request = json!({
        "wasm_module": wasm_b64,
        "config": {
            "name": "api_test_agent",
            "resources": {
                "memory": "50MB",
                "cpu": "100m"
            }
            // capabilities field removed - handle via mocks in test code
        }
    });
    
    let deploy_response = client
        .post(&format!("{}/agents", base_url))
        .json(&deploy_request)
        .send()
        .await
        .unwrap();
    
    assert_eq!(deploy_response.status(), 200);
    let agent_info: serde_json::Value = deploy_response.json().await.unwrap();
    let agent_id = agent_info["agent_id"].as_str().unwrap();
    
    // Test agent listing
    let list_response = client
        .get(&format!("{}/agents", base_url))
        .send()
        .await
        .unwrap();
    
    assert_eq!(list_response.status(), 200);
    let agents_list: serde_json::Value = list_response.json().await.unwrap();
    assert!(agents_list["agents"].as_array().unwrap().len() > 0);
    
    // Test get agent details
    let agent_response = client
        .get(&format!("{}/agents/{}", base_url, agent_id))
        .send()
        .await
        .unwrap();
    
    assert_eq!(agent_response.status(), 200);
    let agent_details: serde_json::Value = agent_response.json().await.unwrap();
    assert_eq!(agent_details["agent_id"], agent_id);
    assert_eq!(agent_details["name"], "api_test_agent");
    assert_eq!(agent_details["status"], "running");
    
    // Test message sending
    let message_request = json!({
        "performative": "request",
        "sender": "api_test",
        "receiver": agent_id,
        "content": {
            "action": "ping"
        },
        "reply_with": "api_ping_001"
    });
    
    let message_response = client
        .post(&format!("{}/messages", base_url))
        .json(&message_request)
        .send()
        .await
        .unwrap();
    
    assert_eq!(message_response.status(), 200);
    let send_result: serde_json::Value = message_response.json().await.unwrap();
    assert_eq!(send_result["status"], "delivered");
    
    // Test agent stopping
    let stop_response = client
        .post(&format!("{}/agents/{}/stop", base_url, agent_id))
        .json(&json!({"grace_period_seconds": 5}))
        .send()
        .await
        .unwrap();
    
    assert_eq!(stop_response.status(), 200);
    
    // Wait for graceful shutdown
    tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
    
    // Test agent removal
    let remove_response = client
        .delete(&format!("{}/agents/{}", base_url, agent_id))
        .send()
        .await
        .unwrap();
    
    assert_eq!(remove_response.status(), 200);
    
    // Verify agent is gone
    let check_response = client
        .get(&format!("{}/agents/{}", base_url, agent_id))
        .send()
        .await
        .unwrap();
    
    assert_eq!(check_response.status(), 404);
}

#[tokio::test]
async fn test_metrics_api() {
    let client = Client::new();
    let base_url = "http://localhost:8080/api/v1";
    
    // Test system metrics
    let system_metrics_response = client
        .get(&format!("{}/metrics/system", base_url))
        .send()
        .await
        .unwrap();
    
    assert_eq!(system_metrics_response.status(), 200);
    let metrics: serde_json::Value = system_metrics_response.json().await.unwrap();
    
    // Verify expected metric fields
    assert!(metrics["agents"].is_object());
    assert!(metrics["messages"].is_object());
    assert!(metrics["resources"].is_object());
    assert!(metrics["performance"].is_object());
    
    assert!(metrics["agents"]["total"].is_number());
    assert!(metrics["messages"]["rate_per_second"].is_number());
    assert!(metrics["resources"]["memory_used_mb"].is_number());
    
    // Test metrics filtering
    let filtered_response = client
        .get(&format!("{}/metrics/system?fields=agents,messages", base_url))
        .send()
        .await
        .unwrap();
    
    assert_eq!(filtered_response.status(), 200);
    let filtered_metrics: serde_json::Value = filtered_response.json().await.unwrap();
    
    assert!(filtered_metrics["agents"].is_object());
    assert!(filtered_metrics["messages"].is_object());
    assert!(filtered_metrics["resources"].is_null());
}

#[tokio::test]
async fn test_websocket_api() {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use futures_util::{SinkExt, StreamExt};
    
    // Connect to WebSocket
    let (ws_stream, _) = connect_async("ws://localhost:8080/ws")
        .await
        .unwrap();
    
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to agent events
    let subscribe_message = json!({
        "type": "subscribe",
        "events": ["agent.*", "message.*"],
        "filters": {
            "agent_ids": []
        }
    });
    
    write.send(Message::Text(subscribe_message.to_string()))
        .await
        .unwrap();
    
    // Create an agent to generate events
    let client = reqwest::Client::new();
    let wasm_bytes = std::fs::read("target/wasm32-wasi/release/test_agent.wasm").unwrap();
    let deploy_request = json!({
        "wasm_module": base64::encode(&wasm_bytes),
        "config": {
            "name": "ws_test_agent"
        }
    });
    
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = client
            .post("http://localhost:8080/api/v1/agents")
            .json(&deploy_request)
            .send()
            .await;
    });
    
    // Wait for deployment event
    let event_timeout = tokio::time::timeout(
        tokio::time::Duration::from_secs(10),
        read.next()
    );
    
    let event = event_timeout.await.unwrap().unwrap().unwrap();
    
    if let Message::Text(event_text) = event {
        let event_data: serde_json::Value = serde_json::from_str(&event_text).unwrap();
        assert_eq!(event_data["type"], "agent.deployed");
        assert!(event_data["agent_id"].is_string());
    } else {
        panic!("Expected text message");
    }
}
```

## Load Testing

### Performance Testing Framework

```rust
// tests/load_tests.rs
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{Instant, Duration};
use futures::future::join_all;

#[derive(Clone)]
pub struct LoadTestMetrics {
    pub requests_sent: Arc<AtomicU64>,
    pub responses_received: Arc<AtomicU64>,
    pub errors: Arc<AtomicU64>,
    pub total_latency_micros: Arc<AtomicU64>,
    pub start_time: Instant,
}

impl LoadTestMetrics {
    pub fn new() -> Self {
        Self {
            requests_sent: Arc::new(AtomicU64::new(0)),
            responses_received: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            total_latency_micros: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }
    
    pub fn record_request(&self) {
        self.requests_sent.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_response(&self, latency_micros: u64) {
        self.responses_received.fetch_add(1, Ordering::Relaxed);
        self.total_latency_micros.fetch_add(latency_micros, Ordering::Relaxed);
    }
    
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn print_summary(&self) {
        let duration = self.start_time.elapsed();
        let requests = self.requests_sent.load(Ordering::Relaxed);
        let responses = self.responses_received.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let total_latency = self.total_latency_micros.load(Ordering::Relaxed);
        
        println!("Load Test Summary:");
        println!("  Duration: {:?}", duration);
        println!("  Requests sent: {}", requests);
        println!("  Responses received: {}", responses);
        println!("  Errors: {}", errors);
        println!("  Success rate: {:.2}%", 
            (responses as f64 / requests as f64) * 100.0);
        println!("  Average latency: {:.2} ms", 
            (total_latency as f64 / responses as f64) / 1000.0);
        println!("  Throughput: {:.2} req/sec", 
            requests as f64 / duration.as_secs_f64());
    }
}

#[tokio::test]
async fn test_message_throughput() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Deploy high-performance echo agent
    let echo_wasm = std::fs::read("target/wasm32-wasi/release/echo_agent.wasm").unwrap();
    let echo_agent = client.deploy_agent(DeployAgentRequest {
        wasm_module: echo_wasm,
        config: AgentConfig {
            name: "load_test_echo".to_string(),
            resources: ResourceLimits {
                max_memory_bytes: 100 * 1024 * 1024, // 100MB
                max_cpu_micros: 1_000_000, // 1 second per message
                ..Default::default()
            },
            ..Default::default()
        },
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let metrics = LoadTestMetrics::new();
    let concurrent_clients = 50;
    let messages_per_client = 100;
    
    // Spawn concurrent message senders
    let mut tasks = Vec::new();
    for client_id in 0..concurrent_clients {
        let client = client.clone();
        let agent_id = echo_agent.agent_id.clone();
        let metrics = metrics.clone();
        
        let task = tokio::spawn(async move {
            for msg_id in 0..messages_per_client {
                let start_time = Instant::now();
                
                let message = FipaMessage {
                    performative: "request".to_string(),
                    sender: format!("load_client_{}", client_id),
                    receiver: agent_id.clone(),
                    content: json!({
                        "action": "echo",
                        "data": format!("Message {} from client {}", msg_id, client_id)
                    }),
                    reply_with: Some(format!("load_{}_{}", client_id, msg_id)),
                    ..Default::default()
                };
                
                metrics.record_request();
                
                match client.send_message_and_wait(message, Duration::from_secs(5)).await {
                    Ok(response) => {
                        let latency = start_time.elapsed().as_micros() as u64;
                        metrics.record_response(latency);
                        
                        // Verify response
                        if response.performative != "inform" {
                            metrics.record_error();
                        }
                    }
                    Err(_) => {
                        metrics.record_error();
                    }
                }
                
                // Small delay to prevent overwhelming
                if msg_id % 10 == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    join_all(tasks).await;
    
    metrics.print_summary();
    
    // Verify performance targets
    let responses = metrics.responses_received.load(Ordering::Relaxed);
    let errors = metrics.errors.load(Ordering::Relaxed);
    let total_expected = (concurrent_clients * messages_per_client) as u64;
    
    assert!(responses > total_expected * 95 / 100, 
        "Should have >95% success rate, got {}/{}", responses, total_expected);
    assert!(errors < total_expected * 5 / 100, 
        "Should have <5% error rate, got {}/{}", errors, total_expected);
    
    let avg_latency = metrics.total_latency_micros.load(Ordering::Relaxed) / responses;
    assert!(avg_latency < 100_000, // 100ms
        "Average latency should be <100ms, got {}ms", avg_latency / 1000);
    
    client.remove_agent(&echo_agent.agent_id).await.unwrap();
}

#[tokio::test]
async fn test_agent_scaling() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    let wasm_bytes = std::fs::read("target/wasm32-wasi/release/simple_agent.wasm").unwrap();
    
    let agent_count = 100;
    let mut agents = Vec::new();
    
    let deployment_start = Instant::now();
    
    // Deploy many agents concurrently
    let mut deploy_tasks = Vec::new();
    for i in 0..agent_count {
        let client = client.clone();
        let wasm_bytes = wasm_bytes.clone();
        
        let task = tokio::spawn(async move {
            client.deploy_agent(DeployAgentRequest {
                wasm_module: wasm_bytes,
                config: AgentConfig {
                    name: format!("scale_test_agent_{}", i),
                    resources: ResourceLimits {
                        max_memory_bytes: 10 * 1024 * 1024, // 10MB each
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }).await
        });
        
        deploy_tasks.push(task);
    }
    
    // Wait for all deployments
    let deployment_results = join_all(deploy_tasks).await;
    let deployment_time = deployment_start.elapsed();
    
    println!("Deployed {} agents in {:?}", agent_count, deployment_time);
    
    for result in deployment_results {
        let agent = result.unwrap().unwrap();
        agents.push(agent);
    }
    
    // Verify all agents are running
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    let agent_list = client.list_agents(ListAgentsRequest {
        status_filter: Some(AgentStatus::Running),
        ..Default::default()
    }).await.unwrap();
    
    assert!(agent_list.agents.len() >= agent_count, 
        "Should have at least {} running agents", agent_count);
    
    // Test concurrent message handling
    let metrics = LoadTestMetrics::new();
    let mut message_tasks = Vec::new();
    
    for (i, agent) in agents.iter().take(50).enumerate() {
        let client = client.clone();
        let agent_id = agent.agent_id.clone();
        let metrics = metrics.clone();
        
        let task = tokio::spawn(async move {
            for j in 0..10 {
                let start = Instant::now();
                metrics.record_request();
                
                match client.send_message_and_wait(
                    FipaMessage {
                        performative: "request".to_string(),
                        sender: "scale_test".to_string(),
                        receiver: agent_id.clone(),
                        content: json!({"action": "ping", "id": j}),
                        reply_with: Some(format!("scale_{}_{}", i, j)),
                        ..Default::default()
                    },
                    Duration::from_secs(10)
                ).await {
                    Ok(_) => {
                        metrics.record_response(start.elapsed().as_micros() as u64);
                    }
                    Err(_) => {
                        metrics.record_error();
                    }
                }
            }
        });
        
        message_tasks.push(task);
    }
    
    join_all(message_tasks).await;
    metrics.print_summary();
    
    // Cleanup all agents
    let mut cleanup_tasks = Vec::new();
    for agent in agents {
        let client = client.clone();
        let agent_id = agent.agent_id.clone();
        
        let task = tokio::spawn(async move {
            client.remove_agent(&agent_id).await
        });
        
        cleanup_tasks.push(task);
    }
    
    join_all(cleanup_tasks).await;
    
    // Verify performance characteristics
    let responses = metrics.responses_received.load(Ordering::Relaxed);
    let total_sent = metrics.requests_sent.load(Ordering::Relaxed);
    assert!(responses > total_sent * 95 / 100, "High success rate required under load");
}

#[tokio::test]
async fn test_memory_pressure() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Deploy agent with limited memory
    let memory_agent_wasm = std::fs::read("target/wasm32-wasi/release/memory_test_agent.wasm").unwrap();
    let memory_agent = client.deploy_agent(DeployAgentRequest {
        wasm_module: memory_agent_wasm,
        config: AgentConfig {
            name: "memory_pressure_test".to_string(),
            resources: ResourceLimits {
                max_memory_bytes: 20 * 1024 * 1024, // 20MB limit
                ..Default::default()
            },
            ..Default::default()
        },
    }).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Send increasingly large data
    for size in [1024, 10240, 102400, 1048576] { // 1KB to 1MB
        let large_data = "x".repeat(size);
        
        let response = client.send_message_and_wait(
            FipaMessage {
                performative: "request".to_string(),
                sender: "memory_test".to_string(),
                receiver: memory_agent.agent_id.clone(),
                content: json!({
                    "action": "process_large_data",
                    "data": large_data
                }),
                reply_with: Some(format!("memory_test_{}", size)),
                ..Default::default()
            },
            Duration::from_secs(30)
        ).await;
        
        match response {
            Ok(resp) => {
                println!("Successfully processed {} bytes", size);
                assert_eq!(resp.performative, "inform");
            }
            Err(e) => {
                if size > 512 * 1024 { // Expect failures for very large data
                    println!("Expected failure for {} bytes: {}", size, e);
                } else {
                    panic!("Unexpected failure for {} bytes: {}", size, e);
                }
            }
        }
        
        // Allow memory to be released
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    client.remove_agent(&memory_agent.agent_id).await.unwrap();
}
```

## Chaos Testing

Test system resilience under failure conditions:

```rust
// tests/chaos_tests.rs
use rand::Rng;
use std::sync::Arc;
use tokio::time::{Duration, sleep, Instant};

#[tokio::test]
async fn test_agent_failure_recovery() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Deploy multiple agents
    let agent_count = 10;
    let mut agents = Vec::new();
    let wasm_bytes = std::fs::read("target/wasm32-wasi/release/resilient_agent.wasm").unwrap();
    
    for i in 0..agent_count {
        let agent = client.deploy_agent(DeployAgentRequest {
            wasm_module: wasm_bytes.clone(),
            config: AgentConfig {
                name: format!("chaos_agent_{}", i),
                ..Default::default()
            },
        }).await.unwrap();
        agents.push(agent);
    }
    
    sleep(Duration::from_millis(500)).await;
    
    // Start continuous message sending
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let message_task = {
        let client = client.clone();
        let agents = agents.clone();
        let running = running.clone();
        
        tokio::spawn(async move {
            let mut message_count = 0;
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                let agent = &agents[message_count % agents.len()];
                
                let _ = client.send_message(FipaMessage {
                    performative: "inform".to_string(),
                    sender: "chaos_test".to_string(),
                    receiver: agent.agent_id.clone(),
                    content: json!({"heartbeat": message_count}),
                    ..Default::default()
                }).await;
                
                message_count += 1;
                sleep(Duration::from_millis(10)).await;
            }
        })
    };
    
    // Randomly kill and restart agents
    let chaos_task = {
        let client = client.clone();
        let mut agents = agents.clone();
        let wasm_bytes = wasm_bytes.clone();
        
        tokio::spawn(async move {
            let mut rng = rand::thread_rng();
            
            for iteration in 0..20 {
                sleep(Duration::from_millis(rng.gen_range(100..1000))).await;
                
                if !agents.is_empty() {
                    // Pick random agent to kill
                    let victim_idx = rng.gen_range(0..agents.len());
                    let victim = agents.remove(victim_idx);
                    
                    println!("Chaos iteration {}: Killing agent {}", iteration, victim.agent_id);
                    let _ = client.remove_agent(&victim.agent_id).await;
                    
                    // Wait a bit, then restart
                    sleep(Duration::from_millis(rng.gen_range(100..500))).await;
                    
                    match client.deploy_agent(DeployAgentRequest {
                        wasm_module: wasm_bytes.clone(),
                        config: AgentConfig {
                            name: format!("chaos_agent_restart_{}", iteration),
                            ..Default::default()
                        },
                    }).await {
                        Ok(new_agent) => {
                            println!("Restarted as agent {}", new_agent.agent_id);
                            agents.push(new_agent);
                        }
                        Err(e) => {
                            eprintln!("Failed to restart agent: {}", e);
                        }
                    }
                }
            }
        })
    };
    
    // Run chaos for 30 seconds
    sleep(Duration::from_secs(30)).await;
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    
    // Wait for tasks to complete
    let _ = tokio::join!(message_task, chaos_task);
    
    // Verify system is still functional
    let final_agent_list = client.list_agents(ListAgentsRequest::default()).await.unwrap();
    assert!(!final_agent_list.agents.is_empty(), "System should still have agents running");
    
    // Test that remaining agents are responsive
    if let Some(agent) = final_agent_list.agents.first() {
        let response = client.send_message_and_wait(
            FipaMessage {
                performative: "request".to_string(),
                sender: "chaos_test".to_string(),
                receiver: agent.agent_id.clone(),
                content: json!({"action": "ping"}),
                reply_with: Some("post_chaos_ping".to_string()),
                ..Default::default()
            },
            Duration::from_secs(5)
        ).await;
        
        assert!(response.is_ok(), "Remaining agents should be responsive");
    }
    
    // Cleanup remaining agents
    for agent in final_agent_list.agents {
        let _ = client.remove_agent(&agent.agent_id).await;
    }
}

#[tokio::test]
async fn test_network_partition_simulation() {
    // This test simulates network partitions by temporarily blocking
    // communication between groups of agents
    
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Deploy agents in two "partitions"
    let partition_a_agents = deploy_agent_group(&client, "partition_a", 5).await;
    let partition_b_agents = deploy_agent_group(&client, "partition_b", 5).await;
    
    sleep(Duration::from_millis(500)).await;
    
    // Start inter-partition communication
    let communication_task = start_inter_partition_communication(
        &client, 
        &partition_a_agents, 
        &partition_b_agents
    ).await;
    
    // Simulate network partition for 10 seconds
    // (In a real test, you'd configure network rules or use proxy)
    println!("Simulating network partition...");
    sleep(Duration::from_secs(10)).await;
    
    // Resume communication
    println!("Restoring network connectivity...");
    sleep(Duration::from_secs(5)).await;
    
    // Stop communication test
    communication_task.abort();
    
    // Verify agents are still responsive within their partitions
    for agent in &partition_a_agents {
        let response = client.send_message_and_wait(
            FipaMessage {
                performative: "request".to_string(),
                sender: "partition_test".to_string(),
                receiver: agent.agent_id.clone(),
                content: json!({"action": "status_check"}),
                reply_with: Some("status_check".to_string()),
                ..Default::default()
            },
            Duration::from_secs(5)
        ).await;
        
        assert!(response.is_ok(), "Agent should be responsive after partition");
    }
    
    // Cleanup
    cleanup_agents(&client, &partition_a_agents).await;
    cleanup_agents(&client, &partition_b_agents).await;
}

async fn deploy_agent_group(
    client: &CaxtonClient, 
    group_name: &str, 
    count: usize
) -> Vec<DeployAgentResponse> {
    let mut agents = Vec::new();
    let wasm_bytes = std::fs::read("target/wasm32-wasi/release/partition_test_agent.wasm").unwrap();
    
    for i in 0..count {
        let agent = client.deploy_agent(DeployAgentRequest {
            wasm_module: wasm_bytes.clone(),
            config: AgentConfig {
                name: format!("{}_{}", group_name, i),
                ..Default::default()
            },
        }).await.unwrap();
        agents.push(agent);
    }
    
    agents
}

async fn start_inter_partition_communication(
    client: &CaxtonClient,
    partition_a: &[DeployAgentResponse],
    partition_b: &[DeployAgentResponse],
) -> tokio::task::JoinHandle<()> {
    let client = client.clone();
    let a_agents = partition_a.to_vec();
    let b_agents = partition_b.to_vec();
    
    tokio::spawn(async move {
        let mut message_id = 0;
        
        loop {
            // Send message from A to B
            if !a_agents.is_empty() && !b_agents.is_empty() {
                let sender = &a_agents[message_id % a_agents.len()];
                let receiver = &b_agents[message_id % b_agents.len()];
                
                let _ = client.send_message(FipaMessage {
                    performative: "inform".to_string(),
                    sender: sender.agent_id.clone(),
                    receiver: receiver.agent_id.clone(),
                    content: json!({
                        "partition_message": message_id,
                        "from_partition": "A"
                    }),
                    ..Default::default()
                }).await;
                
                message_id += 1;
            }
            
            sleep(Duration::from_millis(100)).await;
        }
    })
}

async fn cleanup_agents(client: &CaxtonClient, agents: &[DeployAgentResponse]) {
    for agent in agents {
        let _ = client.remove_agent(&agent.agent_id).await;
    }
}
```

## Test Automation and CI/CD

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Comprehensive Testing

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-wasi
        components: rustfmt, clippy
        override: true
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install WASM tools
      run: |
        curl -L https://github.com/WebAssembly/binaryen/releases/download/version_116/binaryen-version_116-x86_64-linux.tar.gz | tar xz
        echo "$PWD/binaryen-version_116/bin" >> $GITHUB_PATH
    
    - name: Build WASM test agents
      run: |
        cd tests/test_agents
        cargo build --target wasm32-wasi --release
        wasm-opt -Os target/wasm32-wasi/release/*.wasm -o optimized/
    
    - name: Run unit tests
      run: cargo test --lib --bins --tests unit_tests
    
    - name: Run property-based tests
      run: cargo test property_tests
      env:
        PROPTEST_CASES: 1000

  integration-tests:
    runs-on: ubuntu-latest
    needs: unit-tests
    services:
      caxton:
        image: caxton:test
        ports:
          - 8080:8080
          - 50051:50051
        options: >-
          --health-cmd "curl -f http://localhost:8080/health"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Wait for Caxton to be ready
      run: |
        timeout 60 bash -c 'until curl -f http://localhost:8080/health; do sleep 2; done'
    
    - name: Run integration tests
      run: cargo test integration_tests
      env:
        CAXTON_ENDPOINT: http://localhost:8080
        RUST_LOG: info

  load-tests:
    runs-on: ubuntu-latest
    needs: integration-tests
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up load test environment
      run: |
        docker-compose -f docker-compose.loadtest.yml up -d
        sleep 30  # Wait for services to be ready
    
    - name: Run load tests
      run: |
        cargo test load_tests --release
      env:
        CAXTON_ENDPOINT: http://localhost:8080
        LOAD_TEST_DURATION: 300  # 5 minutes
        MAX_CONCURRENT_CLIENTS: 100
    
    - name: Collect performance metrics
      run: |
        docker-compose -f docker-compose.loadtest.yml exec caxton curl -s http://localhost:8080/api/v1/metrics/system > metrics.json
        
    - name: Upload performance results
      uses: actions/upload-artifact@v3
      with:
        name: load-test-results
        path: |
          metrics.json
          load_test_*.log

  chaos-tests:
    runs-on: ubuntu-latest
    needs: integration-tests
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up chaos test environment
      run: |
        # Install chaos engineering tools
        kubectl apply -f k8s/chaos-test-environment.yml
        sleep 60
    
    - name: Run chaos tests
      run: |
        cargo test chaos_tests --release -- --test-threads=1
      env:
        CAXTON_ENDPOINT: http://localhost:8080
        CHAOS_DURATION: 600  # 10 minutes
    
    - name: Generate test report
      if: always()
      run: |
        cargo test --no-run --message-format=json | jq -r 'select(.reason == "test") | .name' > test_results.txt
        
    - name: Upload chaos test results  
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: chaos-test-results
        path: |
          test_results.txt
          chaos_test_*.log
```

### Test Configuration

```toml
# tests/Cargo.toml
[package]
name = "caxton-tests"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test-runner"
path = "src/test_runner.rs"

[dependencies]
caxton-client = { path = "../client" }
caxton-sdk = { path = "../sdk" }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
futures = "0.3"
proptest = "1.0"
criterion = { version = "0.5", features = ["html_reports"] }
base64 = "0.21"
uuid = { version = "1.0", features = ["v4"] }
rand = "0.8"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"
```

This comprehensive testing guide provides strategies for validating all aspects of Caxton's multi-agent system, from individual agent behavior to system-wide performance and resilience. The layered approach ensures thorough coverage while maintaining test efficiency and reliability.