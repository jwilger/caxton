---
title: "Building Your First Agent"
date: 2025-09-10
layout: page
categories: [Getting Started]
---

Learn how to create WebAssembly agents for Caxton in your preferred programming
language.

## Agent Basics

Every Caxton agent must:

1. Export a `handle_message` function
2. Accept FIPA-formatted messages
3. Return valid FIPA responses (or null)
4. Compile to WebAssembly

## Agent Lifecycle

Agents follow a managed lifecycle:

- **Deployment**: WASM modules are validated and loaded
- **Execution**: Agents process messages in isolated sandboxes
- **Management**: Hot reload enables zero-downtime updates
- **Monitoring**: Resource usage and health are tracked

For production deployment and management, see
[Agent Lifecycle Management](../operations/agent-lifecycle-management.md).

## JavaScript Agent

### Simple Echo Agent

Create `echo-agent.js`:

```javascript
// Echo agent that responds to any message
export function handle_message(message_bytes) {
    // Parse the incoming message
    const message = JSON.parse(new TextDecoder().decode(message_bytes));

    console.log(`[${message.receiver}] Received from ${message.sender}: ${message.content.text}`);

    // Create response
    const response = {
        performative: 'inform',
        sender: message.receiver,  // Our ID
        receiver: message.sender,   // Reply to sender
        conversation_id: message.conversation_id,
        in_reply_to: message.id,
        content: {
            text: `Echo: ${message.content.text}`,
            timestamp: Date.now()
        }
    };

    // Return encoded response
    return new TextEncoder().encode(JSON.stringify(response));
}

// Optional: Handle agent initialization
export function init() {
    console.log("Echo agent initialized");
    return 0;  // Success
}

// Optional: Handle agent shutdown
export function shutdown() {
    console.log("Echo agent shutting down");
    return 0;  // Success
}
```

### Compile to WebAssembly

Using [Javy](https://github.com/bytecodealliance/javy):

```bash
# Install Javy
npm install -g @bytecodealliance/javy

# Compile to WASM
javy compile echo-agent.js -o echo-agent.wasm

# Deploy to Caxton
caxton deploy echo-agent.wasm --name echo
```

### Advanced: Stateful Agent

```javascript
// Stateful counter agent
let counter = 0;
const state = new Map();

export function handle_message(message_bytes) {
    const message = JSON.parse(new TextDecoder().decode(message_bytes));

    switch(message.performative) {
        case 'request':
            return handleRequest(message);
        case 'query':
            return handleQuery(message);
        case 'subscribe':
            return handleSubscribe(message);
        default:
            return handleUnknown(message);
    }
}

function handleRequest(message) {
    const { action, params } = message.content;

    switch(action) {
        case 'increment':
            counter += params.amount || 1;
            break;
        case 'decrement':
            counter -= params.amount || 1;
            break;
        case 'reset':
            counter = 0;
            break;
    }

    return createResponse(message, {
        status: 'success',
        counter: counter
    });
}

function handleQuery(message) {
    return createResponse(message, {
        counter: counter,
        state: Object.fromEntries(state)
    });
}

function createResponse(originalMessage, content) {
    const response = {
        performative: 'inform',
        sender: originalMessage.receiver,
        receiver: originalMessage.sender,
        conversation_id: originalMessage.conversation_id,
        in_reply_to: originalMessage.id,
        content: content
    };

    return new TextEncoder().encode(JSON.stringify(response));
}
```

## Python Agent

### Setup

First, install the Python to WASM compiler:

```bash
pip install wasmtime-py pyodide-build
```

### Simple Agent

Create `weather_agent.py`:

```python
import json
import random
from datetime import datetime

class WeatherAgent:
    """Simulated weather information agent"""

    def __init__(self):
        self.cities = {
            'london': {'temp_base': 15, 'variance': 5},
            'new_york': {'temp_base': 20, 'variance': 8},
            'tokyo': {'temp_base': 18, 'variance': 6},
            'sydney': {'temp_base': 22, 'variance': 4}
        }

    def get_weather(self, city):
        """Generate simulated weather data"""
        if city.lower() not in self.cities:
            return None

        city_data = self.cities[city.lower()]
        temp = city_data['temp_base'] + random.uniform(-city_data['variance'], city_data['variance'])

        return {
            'city': city,
            'temperature': round(temp, 1),
            'unit': 'celsius',
            'conditions': random.choice(['sunny', 'cloudy', 'rainy', 'partly cloudy']),
            'humidity': random.randint(30, 80),
            'timestamp': datetime.now().isoformat()
        }

# Global agent instance
agent = WeatherAgent()

def handle_message(message_bytes):
    """Main message handler for Caxton"""
    try:
        # Decode and parse message
        message = json.loads(message_bytes.decode('utf-8'))

        # Handle different performatives
        if message['performative'] == 'query':
            return handle_query(message)
        elif message['performative'] == 'request':
            return handle_request(message)
        else:
            return create_not_understood(message)

    except Exception as e:
        return create_failure(message, str(e))

def handle_query(message):
    """Handle weather queries"""
    content = message.get('content', {})
    city = content.get('city')

    if not city:
        return create_failure(message, "No city specified")

    weather = agent.get_weather(city)

    if weather:
        response = {
            'performative': 'inform',
            'sender': message['receiver'],
            'receiver': message['sender'],
            'conversation_id': message.get('conversation_id'),
            'in_reply_to': message.get('id'),
            'content': {
                'weather': weather
            }
        }
    else:
        response = {
            'performative': 'failure',
            'sender': message['receiver'],
            'receiver': message['sender'],
            'conversation_id': message.get('conversation_id'),
            'in_reply_to': message.get('id'),
            'content': {
                'error': f"Unknown city: {city}"
            }
        }

    return json.dumps(response).encode('utf-8')

def create_not_understood(message):
    """Create not-understood response"""
    response = {
        'performative': 'not-understood',
        'sender': message['receiver'],
        'receiver': message['sender'],
        'conversation_id': message.get('conversation_id'),
        'in_reply_to': message.get('id'),
        'content': {
            'error': f"Unknown performative: {message['performative']}"
        }
    }
    return json.dumps(response).encode('utf-8')

def create_failure(message, error):
    """Create failure response"""
    response = {
        'performative': 'failure',
        'sender': message.get('receiver', 'unknown'),
        'receiver': message.get('sender', 'unknown'),
        'conversation_id': message.get('conversation_id'),
        'in_reply_to': message.get('id'),
        'content': {
            'error': error
        }
    }
    return json.dumps(response).encode('utf-8')
```

### Compile and Deploy

```bash
# Compile to WASM
pyodide build weather_agent.py -o weather_agent.wasm

# Deploy
caxton deploy weather_agent.wasm --name weather-service

# Test it
caxton message send \
  --to weather-service \
  --performative query \
  --content '{"city": "London"}'
```

## Go Agent

### Setup

```bash
# Install TinyGo (Go to WASM compiler)
wget https://github.com/tinygo-org/tinygo/releases/download/v0.30.0/tinygo_0.30.0_amd64.deb
sudo dpkg -i tinygo_0.30.0_amd64.deb
```

### Calculator Agent

Create `calculator_agent.go`:

```go
package main

import (
    "encoding/json"
    "fmt"
    "math"
)

// Message represents a FIPA message
type Message struct {
    Performative   string                 `json:"performative"`
    Sender         string                 `json:"sender"`
    Receiver       string                 `json:"receiver"`
    ConversationID string                 `json:"conversation_id"`
    ReplyWith      string                 `json:"reply_with,omitempty"`
    InReplyTo      string                 `json:"in_reply_to,omitempty"`
    Content        map[string]interface{} `json:"content"`
}

//export handle_message
func handle_message(messagePtr *byte, messageLen int) (*byte, int) {
    // Convert pointer to byte slice
    messageBytes := make([]byte, messageLen)
    for i := 0; i < messageLen; i++ {
        messageBytes[i] = *(*byte)(unsafe.Pointer(uintptr(unsafe.Pointer(messagePtr)) + uintptr(i)))
    }

    // Parse message
    var msg Message
    if err := json.Unmarshal(messageBytes, &msg); err != nil {
        return nil, 0
    }

    // Handle calculation requests
    if msg.Performative == "request" {
        response := processCalculation(msg)
        responseBytes, _ := json.Marshal(response)
        return &responseBytes[0], len(responseBytes)
    }

    return nil, 0
}

func processCalculation(msg Message) Message {
    operation, _ := msg.Content["operation"].(string)
    operands, _ := msg.Content["operands"].([]interface{})

    var result float64
    var err error

    switch operation {
    case "add":
        result = add(operands)
    case "multiply":
        result = multiply(operands)
    case "power":
        if len(operands) == 2 {
            base, _ := operands[0].(float64)
            exp, _ := operands[1].(float64)
            result = math.Pow(base, exp)
        }
    default:
        err = fmt.Errorf("unknown operation: %s", operation)
    }

    response := Message{
        Performative:   "inform",
        Sender:         msg.Receiver,
        Receiver:       msg.Sender,
        ConversationID: msg.ConversationID,
        InReplyTo:      msg.ReplyWith,
    }

    if err != nil {
        response.Performative = "failure"
        response.Content = map[string]interface{}{
            "error": err.Error(),
        }
    } else {
        response.Content = map[string]interface{}{
            "result": result,
            "operation": operation,
        }
    }

    return response
}

func add(operands []interface{}) float64 {
    sum := 0.0
    for _, op := range operands {
        if val, ok := op.(float64); ok {
            sum += val
        }
    }
    return sum
}

func multiply(operands []interface{}) float64 {
    product := 1.0
    for _, op := range operands {
        if val, ok := op.(float64); ok {
            product *= val
        }
    }
    return product
}

func main() {
    // Required for WASM
}
```

### Compile and Deploy

```bash
# Compile with TinyGo
tinygo build -o calculator.wasm -target wasi calculator_agent.go

# Deploy
caxton deploy calculator.wasm --name calculator

# Test
caxton message send \
  --to calculator \
  --performative request \
  --content '{"operation": "add", "operands": [5, 3, 2]}'
```

## Rust Agent

### Setup

```toml
# Cargo.toml
[package]
name = "rust-agent"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "z"
lto = true
```

### Smart Contract Agent

Create `src/lib.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
struct Message {
    performative: String,
    sender: String,
    receiver: String,
    conversation_id: Option<String>,
    in_reply_to: Option<String>,
    content: serde_json::Value,
}

#[derive(Default)]
struct ContractState {
    balances: HashMap<String, u64>,
    total_supply: u64,
}

static mut STATE: Option<ContractState> = None;

#[no_mangle]
pub extern "C" fn init() -> i32 {
    unsafe {
        STATE = Some(ContractState {
            balances: HashMap::new(),
            total_supply: 1_000_000,
        });
    }
    0 // Success
}

#[no_mangle]
pub extern "C" fn handle_message(ptr: *const u8, len: usize) -> *const u8 {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };

    let message: Message = match serde_json::from_slice(bytes) {
        Ok(msg) => msg,
        Err(_) => return std::ptr::null(),
    };

    let response = match message.performative.as_str() {
        "request" => handle_request(message),
        "query" => handle_query(message),
        _ => create_not_understood(message),
    };

    match response {
        Some(resp) => {
            let json = serde_json::to_vec(&resp).unwrap();
            Box::into_raw(json.into_boxed_slice()) as *const u8
        }
        None => std::ptr::null(),
    }
}

fn handle_request(msg: Message) -> Option<Message> {
    let state = unsafe { STATE.as_mut()? };

    let action = msg.content.get("action")?.as_str()?;

    match action {
        "transfer" => {
            let from = msg.content.get("from")?.as_str()?;
            let to = msg.content.get("to")?.as_str()?;
            let amount = msg.content.get("amount")?.as_u64()?;

            // Perform transfer
            let from_balance = state.balances.entry(from.to_string()).or_insert(0);
            if *from_balance < amount {
                return create_failure(msg, "Insufficient balance");
            }

            *from_balance -= amount;
            *state.balances.entry(to.to_string()).or_insert(0) += amount;

            Some(Message {
                performative: "inform".to_string(),
                sender: msg.receiver,
                receiver: msg.sender,
                conversation_id: msg.conversation_id,
                in_reply_to: Some(msg.conversation_id.unwrap_or_default()),
                content: serde_json::json!({
                    "status": "success",
                    "from": from,
                    "to": to,
                    "amount": amount,
                }),
            })
        }
        "mint" => {
            let to = msg.content.get("to")?.as_str()?;
            let amount = msg.content.get("amount")?.as_u64()?;

            *state.balances.entry(to.to_string()).or_insert(0) += amount;
            state.total_supply += amount;

            Some(Message {
                performative: "inform".to_string(),
                sender: msg.receiver,
                receiver: msg.sender,
                conversation_id: msg.conversation_id,
                in_reply_to: Some(msg.conversation_id.unwrap_or_default()),
                content: serde_json::json!({
                    "status": "success",
                    "minted": amount,
                    "to": to,
                    "total_supply": state.total_supply,
                }),
            })
        }
        _ => create_not_understood(msg),
    }
}

fn handle_query(msg: Message) -> Option<Message> {
    let state = unsafe { STATE.as_ref()? };

    let query_type = msg.content.get("type")?.as_str()?;

    match query_type {
        "balance" => {
            let account = msg.content.get("account")?.as_str()?;
            let balance = state.balances.get(account).unwrap_or(&0);

            Some(Message {
                performative: "inform".to_string(),
                sender: msg.receiver,
                receiver: msg.sender,
                conversation_id: msg.conversation_id,
                in_reply_to: Some(msg.conversation_id.unwrap_or_default()),
                content: serde_json::json!({
                    "account": account,
                    "balance": balance,
                }),
            })
        }
        "total_supply" => {
            Some(Message {
                performative: "inform".to_string(),
                sender: msg.receiver,
                receiver: msg.sender,
                conversation_id: msg.conversation_id,
                in_reply_to: Some(msg.conversation_id.unwrap_or_default()),
                content: serde_json::json!({
                    "total_supply": state.total_supply,
                }),
            })
        }
        _ => create_not_understood(msg),
    }
}

fn create_not_understood(msg: Message) -> Option<Message> {
    Some(Message {
        performative: "not-understood".to_string(),
        sender: msg.receiver,
        receiver: msg.sender,
        conversation_id: msg.conversation_id,
        in_reply_to: Some(msg.conversation_id.unwrap_or_default()),
        content: serde_json::json!({
            "error": "Message not understood",
        }),
    })
}

fn create_failure(msg: Message, error: &str) -> Option<Message> {
    Some(Message {
        performative: "failure".to_string(),
        sender: msg.receiver,
        receiver: msg.sender,
        conversation_id: msg.conversation_id,
        in_reply_to: Some(msg.conversation_id.unwrap_or_default()),
        content: serde_json::json!({
            "error": error,
        }),
    })
}
```

### Compile and Deploy

```bash
# Build for WASM
cargo build --target wasm32-wasi --release

# Deploy
caxton deploy target/wasm32-wasi/release/rust_agent.wasm --name token-contract

# Test minting
caxton message send \
  --to token-contract \
  --performative request \
  --content '{"action": "mint", "to": "alice", "amount": 1000}'

# Query balance
caxton message send \
  --to token-contract \
  --performative query \
  --content '{"type": "balance", "account": "alice"}'
```

## Testing Your Agent

### Unit Testing

Test your agent logic before deployment:

```javascript
// test-agent.js
import { handle_message } from './echo-agent.js';

function test_echo() {
    const testMessage = {
        performative: 'inform',
        sender: 'test-sender',
        receiver: 'echo-agent',
        content: { text: 'Hello' }
    };

    const input = new TextEncoder().encode(JSON.stringify(testMessage));
    const output = handle_message(input);
    const response = JSON.parse(new TextDecoder().decode(output));

    console.assert(response.content.text === 'Echo: Hello');
    console.log('✓ Echo test passed');
}

test_echo();
```

### Integration Testing

Test with Caxton running:

```bash
# Deploy test agent
caxton deploy my-agent.wasm --name test-agent

# Send test messages
caxton test agent test-agent --suite basic

# Custom test script
cat > test.sh << 'EOF'
#!/bin/bash
# Send message and check response
RESPONSE=$(caxton message send \
  --to test-agent \
  --performative request \
  --content '{"test": true}' \
  --wait-reply \
  --timeout 5s)

echo "$RESPONSE" | jq '.content.status' | grep -q "success"
EOF

chmod +x test.sh
./test.sh
```

## Best Practices

1. **Handle All Performatives**: Always handle unknown performatives gracefully
2. **Validate Input**: Check message format and content before processing
3. **Use Structured Logging**: Log important events for debugging
4. **Implement Timeouts**: Don't block indefinitely on operations
5. **Minimize State**: Keep agent state minimal and consider persistence
6. **Error Handling**: Return appropriate FIPA error responses
7. **Resource Limits**: Be mindful of memory and CPU usage
8. **Idempotency**: Make operations idempotent where possible

## Next Steps

- [Message Protocol Guide](../developer-guide/message-protocols.md) - Deep dive
  into FIPA protocols
- [API Reference](../developer-guide/api-reference.md) - Complete API
  documentation
- [Testing Guide](../developer-guide/testing.md) - Comprehensive testing
  strategies
- [Production Deployment](../operations/production-deployment.md) - Deploy to
  production
