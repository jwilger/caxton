---
title: WebAssembly Integration Guide
layout: documentation
description: Complete guide to WebAssembly integration in Caxton, including agent development, sandboxing, and performance optimization.
---

# WebAssembly Integration Guide

WebAssembly (WASM) provides the secure sandboxing foundation for Caxton's multi-agent system. This comprehensive guide covers everything from basic agent development to advanced optimization techniques and security considerations.

## Overview

Caxton uses WebAssembly to create isolated execution environments for agents, providing:

- **Security**: Memory-safe execution with controlled system access
- **Performance**: Near-native execution speed with minimal overhead
- **Portability**: Write agents in any language that compiles to WASM
- **Resource Control**: Fine-grained CPU and memory limits
- **Multi-tenancy**: Safe execution of untrusted code

## WebAssembly Runtime Architecture

### Execution Model

```
┌─────────────────────────────────────────────────────────┐
│ Caxton Host Runtime (Rust)                            │
│  ┌─────────────────────────────────────────────────┐   │
│  │ WASM Runtime (Wasmtime/Wasmer)                  │   │
│  │  ┌──────────────┐  ┌──────────────┐            │   │
│  │  │ Agent WASM   │  │ Agent WASM   │  ...       │   │
│  │  │   Instance   │  │   Instance   │            │   │
│  │  │              │  │              │            │   │
│  │  │ [Linear Mem] │  │ [Linear Mem] │            │   │
│  │  │ [Function    │  │ [Function    │            │   │
│  │  │  Imports]    │  │  Imports]    │            │   │
│  │  │ [Function    │  │ [Function    │            │   │
│  │  │  Exports]    │  │  Exports]    │            │   │
│  │  └──────────────┘  └──────────────┘            │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ Host Interface Layer                            │   │
│  │ • Message Bus                                   │   │
│  │ • Resource Manager                              │   │
│  │ • Security Monitor                              │   │
│  │ • Performance Profiler                         │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Security Boundaries

Each WASM agent runs in a completely isolated sandbox:

- **Memory Isolation**: Agents cannot access each other's memory
- **Function Isolation**: Only explicitly exported functions are accessible
- **System Call Mediation**: All system interactions go through controlled host functions
- **Resource Limits**: CPU time, memory usage, and network access are strictly controlled

## Agent Development

### Supported Languages

Caxton supports agents written in any language that compiles to WebAssembly:

#### Rust (Recommended)
```toml
# Cargo.toml
[package]
name = "my-caxton-agent"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
caxton-sdk = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]
```

```rust
// src/lib.rs
use caxton_sdk::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
struct ProcessRequest {
    data: String,
    operation: String,
}

#[derive(Serialize, Deserialize)]
struct ProcessResponse {
    result: String,
    timestamp: u64,
}

// Agent lifecycle hooks
#[wasm_bindgen]
pub fn agent_init() -> i32 {
    console_log("Agent initializing...");
    
    // Register message handlers
    register_handler("process_data", handle_process_data);
    register_handler("get_status", handle_get_status);
    
    0 // Success
}

#[wasm_bindgen]
pub fn agent_shutdown() {
    console_log("Agent shutting down...");
    // Cleanup resources
}

// Message handler
#[wasm_bindgen]
pub fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    let msg_bytes = unsafe { 
        std::slice::from_raw_parts(msg_ptr, msg_len) 
    };
    
    match serde_json::from_slice::<FipaMessage>(msg_bytes) {
        Ok(message) => {
            match message.performative.as_str() {
                "request" => handle_request(message),
                "inform" => handle_inform(message),
                _ => {
                    send_not_understood(&message);
                    1
                }
            }
        }
        Err(e) => {
            console_error(&format!("Failed to parse message: {}", e));
            1
        }
    }
}

fn handle_request(message: FipaMessage) -> i32 {
    match message.content.get("action").and_then(|v| v.as_str()) {
        Some("process_data") => {
            if let Ok(request) = serde_json::from_value::<ProcessRequest>(
                message.content["data"].clone()
            ) {
                let response = process_data(request);
                send_inform_response(&message, &response);
                0
            } else {
                send_failure(&message, "Invalid request format");
                1
            }
        }
        Some("get_status") => {
            let status = get_agent_status();
            send_inform_response(&message, &status);
            0
        }
        _ => {
            send_not_understood(&message);
            1
        }
    }
}

fn process_data(request: ProcessRequest) -> ProcessResponse {
    // Implement your processing logic here
    let result = match request.operation.as_str() {
        "uppercase" => request.data.to_uppercase(),
        "lowercase" => request.data.to_lowercase(),
        "reverse" => request.data.chars().rev().collect(),
        _ => format!("Unknown operation: {}", request.operation),
    };
    
    ProcessResponse {
        result,
        timestamp: current_timestamp(),
    }
}
```

#### AssemblyScript
```typescript
// assembly/index.ts
import { JSON } from "assemblyscript-json";
import { console } from "./console";

class ProcessRequest {
    data: string = "";
    operation: string = "";
}

class ProcessResponse {
    result: string = "";
    timestamp: u64 = 0;
}

export function agent_init(): i32 {
    console.log("AssemblyScript agent initializing...");
    return 0;
}

export function handle_message(msgPtr: i32, msgLen: i32): i32 {
    // Read message from memory
    const msgBuffer = new ArrayBuffer(msgLen);
    memory.copy(changetype<usize>(msgBuffer), msgPtr, msgLen);
    const msgStr = String.UTF8.decode(msgBuffer);
    
    // Parse FIPA message
    const jsonObj = JSON.parse(msgStr);
    const performative = jsonObj.getString("performative");
    
    if (performative == "request") {
        return handleRequest(jsonObj);
    } else if (performative == "inform") {
        return handleInform(jsonObj);
    }
    
    return 1; // Error
}

function handleRequest(message: JSON.Obj): i32 {
    const content = message.getObj("content");
    const action = content?.getString("action");
    
    if (action == "process_data") {
        const data = content!.getString("data");
        const operation = content!.getString("operation");
        
        let result: string;
        if (operation == "uppercase") {
            result = data.toUpperCase();
        } else if (operation == "lowercase") {
            result = data.toLowerCase();
        } else {
            result = "Unknown operation";
        }
        
        // Send response back to host
        sendInformResponse(message, result);
        return 0;
    }
    
    return 1;
}

function sendInformResponse(originalMessage: JSON.Obj, result: string): void {
    const response = new JSON.Obj();
    response.set("performative", "inform");
    response.set("sender", originalMessage.getString("receiver"));
    response.set("receiver", originalMessage.getString("sender"));
    response.set("in_reply_to", originalMessage.getString("reply_with"));
    
    const content = new JSON.Obj();
    content.set("result", result);
    content.set("timestamp", getCurrentTimestamp().toString());
    response.set("content", content);
    
    const responseStr = response.toString();
    const responseBytes = String.UTF8.encode(responseStr);
    
    // Send to host via imported function
    send_message(changetype<i32>(responseBytes), responseBytes.byteLength);
}

// Imported from host
declare function send_message(ptr: i32, len: i32): void;
declare function current_timestamp(): u64;

function getCurrentTimestamp(): u64 {
    return current_timestamp();
}
```

#### Go (TinyGo)
```go
// main.go
package main

import (
    "encoding/json"
    "unsafe"
)

//export agent_init
func agentInit() int32 {
    println("Go agent initializing...")
    return 0
}

//export agent_shutdown
func agentShutdown() {
    println("Go agent shutting down...")
}

//export handle_message
func handleMessage(msgPtr uintptr, msgLen int32) int32 {
    // Convert WASM memory to Go slice
    msgBytes := (*[1 << 30]byte)(unsafe.Pointer(msgPtr))[:msgLen:msgLen]
    
    var message FipaMessage
    if err := json.Unmarshal(msgBytes, &message); err != nil {
        println("Failed to parse message:", err.Error())
        return 1
    }
    
    switch message.Performative {
    case "request":
        return handleRequest(&message)
    case "inform":
        return handleInform(&message)
    default:
        sendNotUnderstood(&message)
        return 1
    }
}

type FipaMessage struct {
    Performative     string                 `json:"performative"`
    Sender          string                 `json:"sender"`
    Receiver        string                 `json:"receiver"`
    Content         map[string]interface{} `json:"content"`
    ConversationID  *string                `json:"conversation_id,omitempty"`
    ReplyWith       *string                `json:"reply_with,omitempty"`
    InReplyTo       *string                `json:"in_reply_to,omitempty"`
}

type ProcessRequest struct {
    Data      string `json:"data"`
    Operation string `json:"operation"`
}

type ProcessResponse struct {
    Result    string `json:"result"`
    Timestamp int64  `json:"timestamp"`
}

func handleRequest(message *FipaMessage) int32 {
    action, ok := message.Content["action"].(string)
    if !ok {
        sendFailure(message, "Missing action field")
        return 1
    }
    
    switch action {
    case "process_data":
        dataInterface, exists := message.Content["data"]
        if !exists {
            sendFailure(message, "Missing data field")
            return 1
        }
        
        dataBytes, _ := json.Marshal(dataInterface)
        var request ProcessRequest
        if err := json.Unmarshal(dataBytes, &request); err != nil {
            sendFailure(message, "Invalid data format")
            return 1
        }
        
        response := processData(&request)
        sendInformResponse(message, response)
        return 0
        
    case "get_status":
        status := getAgentStatus()
        sendInformResponse(message, status)
        return 0
        
    default:
        sendNotUnderstood(message)
        return 1
    }
}

func processData(request *ProcessRequest) *ProcessResponse {
    var result string
    
    switch request.Operation {
    case "uppercase":
        result = strings.ToUpper(request.Data)
    case "lowercase":
        result = strings.ToLower(request.Data)
    case "reverse":
        runes := []rune(request.Data)
        for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
            runes[i], runes[j] = runes[j], runes[i]
        }
        result = string(runes)
    default:
        result = "Unknown operation: " + request.Operation
    }
    
    return &ProcessResponse{
        Result:    result,
        Timestamp: getCurrentTimestamp(),
    }
}

//go:wasmimport env send_message
func sendMessage(ptr uintptr, len int32)

//go:wasmimport env current_timestamp
func getCurrentTimestamp() int64

func sendInformResponse(originalMessage *FipaMessage, response interface{}) {
    responseMsg := FipaMessage{
        Performative: "inform",
        Sender:       originalMessage.Receiver,
        Receiver:     originalMessage.Sender,
        Content:      map[string]interface{}{"result": response},
        InReplyTo:    originalMessage.ReplyWith,
    }
    
    responseBytes, _ := json.Marshal(responseMsg)
    sendMessage(uintptr(unsafe.Pointer(&responseBytes[0])), int32(len(responseBytes)))
}

func main() {}
```

### Build Process

#### Rust Build
```bash
# Install WASM target
rustup target add wasm32-wasi

# Build optimized WASM
cargo build --target wasm32-wasi --release

# Optimize with wasm-opt (from binaryen)
wasm-opt -Os -o agent_optimized.wasm target/wasm32-wasi/release/my_caxton_agent.wasm
```

#### AssemblyScript Build
```bash
# Install dependencies
npm install assemblyscript @assemblyscript/wasi-shim

# Build
npx asc assembly/index.ts --target release --optimize --outFile agent.wasm

# Optimize
wasm-opt -Os -o agent_optimized.wasm agent.wasm
```

#### Go Build
```bash
# Install TinyGo
curl -L https://github.com/tinygo-org/tinygo/releases/download/v0.30.0/tinygo_0.30.0_amd64.deb -o tinygo.deb
sudo dpkg -i tinygo.deb

# Build
tinygo build -target=wasi -o agent.wasm main.go

# Optimize  
wasm-opt -Os -o agent_optimized.wasm agent.wasm
```

## Host Interface (WASI Extensions)

Caxton extends WASI with custom host functions for agent communication and system interaction.

### Message Interface
```rust
// Host-provided functions (imported by agents)
#[link(wasm_import_module = "caxton")]
extern "C" {
    /// Send a FIPA message to another agent
    fn send_message(msg_ptr: *const u8, msg_len: usize) -> i32;
    
    /// Subscribe to messages matching a filter
    fn subscribe_messages(filter_ptr: *const u8, filter_len: usize) -> i32;
    
    /// Get current timestamp (microseconds since Unix epoch)
    fn current_timestamp() -> u64;
    
    /// Log a message (for debugging)
    fn console_log(msg_ptr: *const u8, msg_len: usize);
    
    /// Report an error
    fn console_error(msg_ptr: *const u8, msg_len: usize);
    
    /// Generate a random number
    fn random_u32() -> u32;
    
    /// Sleep for specified microseconds
    fn sleep_micros(micros: u64);
}

// Agent-provided functions (exported to host)
extern "C" {
    /// Called when agent is loaded
    fn agent_init() -> i32;
    
    /// Called when agent is being unloaded
    fn agent_shutdown();
    
    /// Handle incoming FIPA message
    fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32;
    
    /// Handle timer events
    fn handle_timer(timer_id: u32) -> i32;
    
    /// Provide agent capabilities description
    fn get_capabilities(caps_ptr: *mut u8, caps_len: usize) -> i32;
}
```

### Resource Management
```rust
// Resource limit configuration
pub struct ResourceLimits {
    /// Maximum memory in bytes (0 = unlimited)
    pub max_memory_bytes: u64,
    
    /// Maximum CPU time per execution quantum (microseconds)
    pub max_cpu_micros: u64,
    
    /// Maximum execution time for single message (microseconds)
    pub max_execution_time_micros: u64,
    
    /// Maximum number of host function calls per execution
    pub max_host_calls: u32,
    
    /// Maximum outgoing messages per second
    pub max_messages_per_second: u32,
    
    /// Maximum memory allocations per execution
    pub max_allocations: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 50 * 1024 * 1024, // 50MB
            max_cpu_micros: 100_000, // 100ms
            max_execution_time_micros: 1_000_000, // 1 second
            max_host_calls: 1000,
            max_messages_per_second: 100,
            max_allocations: 10000,
        }
    }
}
```

### Capability Declaration
```rust
// Agent capabilities descriptor
#[derive(Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Agent version
    pub version: String,
    
    /// Supported FIPA performatives
    pub performatives: Vec<String>,
    
    /// Supported protocols
    pub protocols: Vec<String>,
    
    /// Supported ontologies
    pub ontologies: Vec<String>,
    
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    
    /// Service descriptions
    pub services: Vec<ServiceDescription>,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_memory_mb: u32,
    pub preferred_memory_mb: u32,
    pub cpu_intensive: bool,
    pub network_access: bool,
    pub persistence_needed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceDescription {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub estimated_latency_ms: u32,
}
```

## Advanced Features

### State Persistence

Agents can persist state between executions:

```rust
#[link(wasm_import_module = "caxton")]
extern "C" {
    /// Store persistent data
    fn store_data(key_ptr: *const u8, key_len: usize, 
                  data_ptr: *const u8, data_len: usize) -> i32;
    
    /// Retrieve persistent data  
    fn load_data(key_ptr: *const u8, key_len: usize,
                 data_ptr: *mut u8, data_len: usize) -> i32;
    
    /// Delete persistent data
    fn delete_data(key_ptr: *const u8, key_len: usize) -> i32;
}

// Agent implementation
use std::collections::HashMap;

struct AgentState {
    counters: HashMap<String, u64>,
    last_update: u64,
    configuration: AgentConfig,
}

impl AgentState {
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_vec(self)?;
        let key = b"agent_state";
        
        let result = unsafe {
            store_data(
                key.as_ptr(),
                key.len(),
                serialized.as_ptr(),
                serialized.len(),
            )
        };
        
        if result == 0 { Ok(()) } else { Err("Failed to save state".into()) }
    }
    
    fn load() -> Result<AgentState, Box<dyn std::error::Error>> {
        let key = b"agent_state";
        let mut buffer = vec![0u8; 4096]; // Max state size
        
        let result = unsafe {
            load_data(
                key.as_ptr(),
                key.len(),
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };
        
        if result > 0 {
            buffer.truncate(result as usize);
            let state: AgentState = serde_json::from_slice(&buffer)?;
            Ok(state)
        } else {
            Ok(AgentState::default())
        }
    }
}
```

### Asynchronous Operations

Handle long-running operations without blocking:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct HostTimer {
    timer_id: u32,
    completed: bool,
}

impl Future for HostTimer {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

// Set a timer from host
#[link(wasm_import_module = "caxton")]
extern "C" {
    fn set_timer(delay_micros: u64) -> u32;
    fn cancel_timer(timer_id: u32) -> i32;
}

// Handle long-running operations
#[wasm_bindgen]
pub fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    let message = parse_message(msg_ptr, msg_len);
    
    match message.content.get("action").and_then(|v| v.as_str()) {
        Some("long_operation") => {
            // Start async operation
            start_long_operation(&message);
            
            // Return immediately - result will be sent via timer
            0
        }
        _ => {
            // Handle synchronously
            handle_sync_message(message)
        }
    }
}

fn start_long_operation(message: &FipaMessage) {
    // Set timer for 5 seconds
    let timer_id = unsafe { set_timer(5_000_000) };
    
    // Store operation context
    store_operation_context(timer_id, message);
}

#[wasm_bindgen]
pub fn handle_timer(timer_id: u32) -> i32 {
    if let Some(context) = load_operation_context(timer_id) {
        // Complete the operation
        let result = complete_long_operation(&context);
        
        // Send result back
        send_inform_response(&context.original_message, &result);
        
        // Cleanup
        cleanup_operation_context(timer_id);
    }
    
    0
}
```

### Inter-Agent Communication

Direct communication between agents in the same runtime:

```rust
// High-performance message passing for local agents
#[link(wasm_import_module = "caxton")]
extern "C" {
    /// Send message to local agent (shared memory)
    fn send_local_message(agent_id_ptr: *const u8, agent_id_len: usize,
                         msg_ptr: *const u8, msg_len: usize) -> i32;
    
    /// Broadcast message to multiple agents
    fn broadcast_message(filter_ptr: *const u8, filter_len: usize,
                        msg_ptr: *const u8, msg_len: usize) -> i32;
}

// Efficient local messaging
pub fn send_to_agent(agent_id: &str, message: &FipaMessage) -> Result<(), String> {
    let agent_bytes = agent_id.as_bytes();
    let msg_bytes = serde_json::to_vec(message).map_err(|e| e.to_string())?;
    
    let result = unsafe {
        send_local_message(
            agent_bytes.as_ptr(),
            agent_bytes.len(),
            msg_bytes.as_ptr(),
            msg_bytes.len(),
        )
    };
    
    if result == 0 {
        Ok(())
    } else {
        Err(format!("Failed to send message, error code: {}", result))
    }
}

// Broadcast to multiple agents
pub fn broadcast_to_capability(capability: &str, message: &FipaMessage) -> Result<u32, String> {
    // Note: capabilities are used here for runtime filtering, not agent configuration
    // Agents register their capabilities programmatically, not through static config
    let filter = json!({
        "capabilities": [capability]
    });
    let filter_bytes = serde_json::to_vec(&filter).map_err(|e| e.to_string())?;
    let msg_bytes = serde_json::to_vec(message).map_err(|e| e.to_string())?;
    
    let result = unsafe {
        broadcast_message(
            filter_bytes.as_ptr(),
            filter_bytes.len(),
            msg_bytes.as_ptr(),
            msg_bytes.len(),
        )
    };
    
    if result >= 0 {
        Ok(result as u32)
    } else {
        Err(format!("Failed to broadcast message, error code: {}", result))
    }
}
```

## Performance Optimization

### Memory Management

Efficient memory usage in WebAssembly environments:

```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

// Custom allocator for WASM
pub struct WasmAllocator {
    allocated: usize,
    max_allocated: usize,
    allocation_count: usize,
}

impl WasmAllocator {
    pub const fn new() -> Self {
        Self {
            allocated: 0,
            max_allocated: 0,
            allocation_count: 0,
        }
    }
    
    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { alloc(layout) };
        
        if !ptr.is_null() {
            self.allocated += size;
            self.allocation_count += 1;
            if self.allocated > self.max_allocated {
                self.max_allocated = self.allocated;
            }
        }
        
        ptr
    }
    
    pub fn deallocate(&mut self, ptr: *mut u8, size: usize, align: usize) {
        let layout = Layout::from_size_align(size, align).unwrap();
        unsafe { dealloc(ptr, layout) };
        
        self.allocated -= size;
    }
    
    pub fn stats(&self) -> AllocationStats {
        AllocationStats {
            current_allocated: self.allocated,
            max_allocated: self.max_allocated,
            allocation_count: self.allocation_count,
        }
    }
}

// Use stack allocation for small, short-lived data
pub fn process_small_message(data: &[u8]) -> Vec<u8> {
    const STACK_SIZE: usize = 1024;
    
    if data.len() <= STACK_SIZE {
        // Use stack allocation
        let mut stack_buffer = [0u8; STACK_SIZE];
        let result_len = process_in_place(&mut stack_buffer[..data.len()]);
        stack_buffer[..result_len].to_vec()
    } else {
        // Fall back to heap allocation
        let mut heap_buffer = data.to_vec();
        let result_len = process_in_place(&mut heap_buffer);
        heap_buffer.truncate(result_len);
        heap_buffer
    }
}
```

### Computation Optimization

```rust
// Optimize hot paths with manual loop unrolling
pub fn fast_data_transform(input: &[u8], output: &mut [u8]) {
    assert_eq!(input.len(), output.len());
    
    let len = input.len();
    let chunks = len / 4;
    let remainder = len % 4;
    
    // Process 4 bytes at a time (loop unrolling)
    for i in 0..chunks {
        let base = i * 4;
        output[base] = transform_byte(input[base]);
        output[base + 1] = transform_byte(input[base + 1]);
        output[base + 2] = transform_byte(input[base + 2]);
        output[base + 3] = transform_byte(input[base + 3]);
    }
    
    // Handle remaining bytes
    for i in 0..remainder {
        let idx = chunks * 4 + i;
        output[idx] = transform_byte(input[idx]);
    }
}

// Use lookup tables for expensive computations
static TRANSFORM_TABLE: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = ((i as f32 * 1.5) as u8).wrapping_add(42);
        i += 1;
    }
    table
};

#[inline(always)]
fn transform_byte(byte: u8) -> u8 {
    TRANSFORM_TABLE[byte as usize]
}
```

### Binary Size Optimization

Minimize WASM binary size:

```rust
// Use smaller integer types when possible
use std::num::{NonZeroU16, NonZeroU32};

// Prefer stack-allocated arrays over Vec when size is known
const MAX_AGENTS: usize = 64;
type AgentArray = [AgentId; MAX_AGENTS];

// Use bit fields for flags
pub struct AgentFlags {
    flags: u32,
}

impl AgentFlags {
    const ACTIVE: u32 = 1 << 0;
    const PERSISTENT: u32 = 1 << 1;
    const HIGH_PRIORITY: u32 = 1 << 2;
    
    pub fn new() -> Self {
        Self { flags: 0 }
    }
    
    pub fn is_active(&self) -> bool {
        self.flags & Self::ACTIVE != 0
    }
    
    pub fn set_active(&mut self, active: bool) {
        if active {
            self.flags |= Self::ACTIVE;
        } else {
            self.flags &= !Self::ACTIVE;
        }
    }
}

// Use const generics to avoid runtime overhead
pub struct FixedBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> FixedBuffer<N> {
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
        }
    }
    
    pub fn push(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.len >= N {
            Err("Buffer full")
        } else {
            self.data[self.len] = byte;
            self.len += 1;
            Ok(())
        }
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
}
```

## Security Considerations

### Input Validation

Always validate inputs from the host and other agents:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct MessageContent {
    #[serde(deserialize_with = "validate_action")]
    action: String,
    
    #[serde(default, deserialize_with = "validate_data")]
    data: Option<serde_json::Value>,
    
    #[serde(default, deserialize_with = "validate_parameters")]
    parameters: HashMap<String, serde_json::Value>,
}

fn validate_action<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let action = String::deserialize(deserializer)?;
    
    // Whitelist allowed actions
    match action.as_str() {
        "process_data" | "get_status" | "calculate" | "transform" => Ok(action),
        _ => Err(serde::de::Error::custom(
            format!("Invalid action: {}", action)
        )),
    }
}

fn validate_data<'de, D>(deserializer: D) -> Result<Option<serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let data = Option::<serde_json::Value>::deserialize(deserializer)?;
    
    if let Some(ref value) = data {
        // Limit size of incoming data
        let size = estimate_json_size(value);
        if size > 1024 * 1024 { // 1MB limit
            return Err(serde::de::Error::custom("Data too large"));
        }
        
        // Validate data structure
        validate_json_structure(value)
            .map_err(|e| serde::de::Error::custom(e))?;
    }
    
    Ok(data)
}

fn validate_json_structure(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::Object(map) => {
            if map.len() > 100 {
                return Err("Object has too many keys".to_string());
            }
            for (key, val) in map {
                if key.len() > 256 {
                    return Err("Key too long".to_string());
                }
                validate_json_structure(val)?;
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.len() > 1000 {
                return Err("Array too large".to_string());
            }
            for val in arr {
                validate_json_structure(val)?;
            }
        }
        serde_json::Value::String(s) => {
            if s.len() > 10000 {
                return Err("String too long".to_string());
            }
        }
        _ => {}
    }
    Ok(())
}

fn estimate_json_size(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            map.iter().map(|(k, v)| k.len() + estimate_json_size(v)).sum::<usize>() + map.len() * 4
        }
        serde_json::Value::Array(arr) => {
            arr.iter().map(estimate_json_size).sum::<usize>() + arr.len() * 2
        }
        serde_json::Value::String(s) => s.len() + 2,
        _ => 16, // Approximate size for numbers, booleans, null
    }
}
```

### Resource Monitoring

Monitor resource usage to prevent abuse:

```rust
pub struct ResourceMonitor {
    memory_usage: usize,
    cpu_time_micros: u64,
    message_count: u32,
    allocation_count: u32,
    start_time: u64,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            memory_usage: 0,
            cpu_time_micros: 0,
            message_count: 0,
            allocation_count: 0,
            start_time: unsafe { current_timestamp() },
        }
    }
    
    pub fn check_limits(&self, limits: &ResourceLimits) -> Result<(), ResourceError> {
        if self.memory_usage > limits.max_memory_bytes as usize {
            return Err(ResourceError::MemoryLimitExceeded);
        }
        
        if self.cpu_time_micros > limits.max_cpu_micros {
            return Err(ResourceError::CpuLimitExceeded);
        }
        
        let elapsed = unsafe { current_timestamp() } - self.start_time;
        if elapsed > limits.max_execution_time_micros {
            return Err(ResourceError::TimeoutExceeded);
        }
        
        if self.allocation_count > limits.max_allocations {
            return Err(ResourceError::AllocationLimitExceeded);
        }
        
        Ok(())
    }
    
    pub fn record_allocation(&mut self, size: usize) {
        self.memory_usage += size;
        self.allocation_count += 1;
    }
    
    pub fn record_deallocation(&mut self, size: usize) {
        self.memory_usage = self.memory_usage.saturating_sub(size);
    }
    
    pub fn record_message_sent(&mut self) {
        self.message_count += 1;
    }
}

#[derive(Debug)]
pub enum ResourceError {
    MemoryLimitExceeded,
    CpuLimitExceeded,
    TimeoutExceeded,
    AllocationLimitExceeded,
    MessageRateLimitExceeded,
}
```

## Testing and Debugging

### Unit Testing

Test WASM agents outside the runtime:

```rust
// tests/agent_test.rs
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_processing() {
        // Mock message
        let message = json!({
            "performative": "request",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {
                "action": "process_data",
                "data": {
                    "operation": "uppercase",
                    "text": "hello world"
                }
            },
            "reply_with": "test_001"
        });
        
        let message_bytes = serde_json::to_vec(&message).unwrap();
        
        // Test agent message handling
        let result = handle_message(
            message_bytes.as_ptr(),
            message_bytes.len()
        );
        
        assert_eq!(result, 0); // Success
    }
    
    #[test]
    fn test_invalid_message_format() {
        let invalid_message = b"invalid json";
        
        let result = handle_message(
            invalid_message.as_ptr(),
            invalid_message.len()
        );
        
        assert_ne!(result, 0); // Should fail
    }
    
    #[test]
    fn test_resource_limits() {
        let large_data = "x".repeat(1024 * 1024 * 2); // 2MB
        
        let message = json!({
            "performative": "request",
            "sender": "test_client",
            "receiver": "test_agent",
            "content": {
                "action": "process_data",
                "data": large_data
            }
        });
        
        let message_bytes = serde_json::to_vec(&message).unwrap();
        
        let result = handle_message(
            message_bytes.as_ptr(),
            message_bytes.len()
        );
        
        assert_ne!(result, 0); // Should fail due to size limit
    }
}
```

### Integration Testing

Test agents in the full Caxton runtime:

```rust
// integration_tests/agent_deployment.rs
use caxton_client::*;
use std::fs;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_agent_deployment_and_communication() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    
    // Load WASM agent
    let wasm_bytes = fs::read("target/wasm32-wasi/release/test_agent.wasm").unwrap();
    
    // Deploy agent
    let deployment = client.deploy_agent(DeployAgentRequest {
        wasm_module: wasm_bytes,
        config: AgentConfig {
            name: "test_agent".to_string(),
            resources: ResourceLimits::default(),
            capabilities: vec!["data_processing".to_string()],
            ..Default::default()
        },
    }).await.unwrap();
    
    // Wait for agent to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Send test message
    let response = client.send_message_and_wait(FipaMessage {
        performative: "request".to_string(),
        sender: "integration_test".to_string(),
        receiver: deployment.agent_id.clone(),
        content: json!({
            "action": "process_data",
            "data": {
                "operation": "uppercase",
                "text": "hello world"
            }
        }),
        reply_with: Some("test_001".to_string()),
        ..Default::default()
    }, Duration::from_secs(10)).await.unwrap();
    
    // Verify response
    assert_eq!(response.performative, "inform");
    assert_eq!(
        response.content["result"]["result"].as_str().unwrap(),
        "HELLO WORLD"
    );
    
    // Cleanup
    client.remove_agent(&deployment.agent_id).await.unwrap();
}

#[tokio::test]
async fn test_agent_resource_limits() {
    let client = CaxtonClient::new("http://localhost:8080").await.unwrap();
    let wasm_bytes = fs::read("target/wasm32-wasi/release/memory_hungry_agent.wasm").unwrap();
    
    // Deploy agent with strict memory limits
    let deployment = client.deploy_agent(DeployAgentRequest {
        wasm_module: wasm_bytes,
        config: AgentConfig {
            name: "memory_test_agent".to_string(),
            resources: ResourceLimits {
                max_memory_bytes: 1024 * 1024, // 1MB limit
                ..Default::default()
            },
            ..Default::default()
        },
    }).await.unwrap();
    
    // Send message that should exceed memory limit
    let result = timeout(
        Duration::from_secs(5),
        client.send_message_and_wait(FipaMessage {
            performative: "request".to_string(),
            sender: "integration_test".to_string(),
            receiver: deployment.agent_id.clone(),
            content: json!({
                "action": "allocate_large_buffer",
                "size": 2 * 1024 * 1024 // 2MB
            }),
            reply_with: Some("memory_test_001".to_string()),
            ..Default::default()
        }, Duration::from_secs(10))
    ).await;
    
    match result {
        Ok(Ok(response)) => {
            // Should receive failure message
            assert_eq!(response.performative, "failure");
            assert!(response.content["error"].as_str().unwrap().contains("memory"));
        }
        _ => panic!("Expected memory limit error"),
    }
    
    client.remove_agent(&deployment.agent_id).await.unwrap();
}
```

### Debugging Tools

Debug WASM agents with logging and profiling:

```rust
// Debug utilities for agents
pub struct DebugLogger {
    enabled: bool,
    log_level: LogLevel,
}

#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl DebugLogger {
    pub fn new() -> Self {
        Self {
            enabled: cfg!(debug_assertions),
            log_level: LogLevel::Info,
        }
    }
    
    pub fn error(&self, msg: &str) {
        if self.enabled && self.log_level >= LogLevel::Error {
            let full_msg = format!("[ERROR] {}", msg);
            unsafe {
                console_error(full_msg.as_ptr(), full_msg.len());
            }
        }
    }
    
    pub fn warn(&self, msg: &str) {
        if self.enabled && self.log_level >= LogLevel::Warn {
            let full_msg = format!("[WARN] {}", msg);
            unsafe {
                console_log(full_msg.as_ptr(), full_msg.len());
            }
        }
    }
    
    pub fn info(&self, msg: &str) {
        if self.enabled && self.log_level >= LogLevel::Info {
            let full_msg = format!("[INFO] {}", msg);
            unsafe {
                console_log(full_msg.as_ptr(), full_msg.len());
            }
        }
    }
    
    pub fn debug(&self, msg: &str) {
        if self.enabled && self.log_level >= LogLevel::Debug {
            let full_msg = format!("[DEBUG] {}", msg);
            unsafe {
                console_log(full_msg.as_ptr(), full_msg.len());
            }
        }
    }
}

// Performance profiler
pub struct Profiler {
    checkpoints: std::collections::HashMap<String, u64>,
    enabled: bool,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            checkpoints: std::collections::HashMap::new(),
            enabled: cfg!(debug_assertions),
        }
    }
    
    pub fn start(&mut self, name: &str) {
        if self.enabled {
            let timestamp = unsafe { current_timestamp() };
            self.checkpoints.insert(name.to_string(), timestamp);
        }
    }
    
    pub fn end(&mut self, name: &str) -> Option<u64> {
        if self.enabled {
            let end_time = unsafe { current_timestamp() };
            if let Some(start_time) = self.checkpoints.remove(name) {
                let duration = end_time - start_time;
                
                let log_msg = format!("Profile [{}]: {} microseconds", name, duration);
                unsafe {
                    console_log(log_msg.as_ptr(), log_msg.len());
                }
                
                Some(duration)
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Usage in agent code
static mut LOGGER: Option<DebugLogger> = None;
static mut PROFILER: Option<Profiler> = None;

#[wasm_bindgen]
pub fn agent_init() -> i32 {
    unsafe {
        LOGGER = Some(DebugLogger::new());
        PROFILER = Some(Profiler::new());
    }
    
    log_info("Agent initialized successfully");
    0
}

#[wasm_bindgen]
pub fn handle_message(msg_ptr: *const u8, msg_len: usize) -> i32 {
    profile_start("message_handling");
    
    let result = match parse_and_handle_message(msg_ptr, msg_len) {
        Ok(_) => {
            log_debug("Message handled successfully");
            0
        }
        Err(e) => {
            log_error(&format!("Message handling failed: {}", e));
            1
        }
    };
    
    profile_end("message_handling");
    result
}

fn log_info(msg: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.info(msg);
        }
    }
}

fn log_error(msg: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.error(msg);
        }
    }
}

fn log_debug(msg: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.debug(msg);
        }
    }
}

fn profile_start(name: &str) {
    unsafe {
        if let Some(ref mut profiler) = PROFILER {
            profiler.start(name);
        }
    }
}

fn profile_end(name: &str) {
    unsafe {
        if let Some(ref mut profiler) = PROFILER {
            profiler.end(name);
        }
    }
}
```

## Troubleshooting

### Common Issues

#### Agent Won't Load
```bash
# Check WASM validity
wasm-validate agent.wasm

# Inspect WASM exports
wasm-objdump -x agent.wasm

# Check for missing imports
wasm-objdump -j Import agent.wasm
```

#### Memory Issues
```rust
// Add memory tracking
static mut ALLOCATED_BYTES: usize = 0;

#[no_mangle]
pub extern "C" fn __wbindgen_malloc(size: usize) -> *mut u8 {
    unsafe {
        ALLOCATED_BYTES += size;
        if ALLOCATED_BYTES > 50 * 1024 * 1024 { // 50MB limit
            return std::ptr::null_mut(); // Signal allocation failure
        }
        
        std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(size, 1))
    }
}

#[no_mangle]
pub extern "C" fn __wbindgen_free(ptr: *mut u8, size: usize) {
    unsafe {
        ALLOCATED_BYTES -= size;
        std::alloc::dealloc(ptr, std::alloc::Layout::from_size_align_unchecked(size, 1));
    }
}
```

#### Performance Problems
```bash
# Profile WASM execution
perf record -g ./caxton run-agent agent.wasm
perf report

# Analyze binary size
wasm-objdump -h agent.wasm
wasm-opt --print-stats agent.wasm
```

This comprehensive guide covers all aspects of WebAssembly integration in Caxton, from basic agent development to advanced optimization and debugging techniques. The sandboxed execution model provides security and isolation while maintaining high performance for agent-based systems.