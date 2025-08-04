//! WebAssembly integration and isolation

use crate::*;

/// WASM agent configuration
#[derive(Debug, Clone)]
pub struct WasmAgentConfig {
    pub name: String,
    pub wasm_module: Vec<u8>,
    pub max_memory_pages: u32,
    pub max_execution_time: Duration,
    pub capabilities: Vec<String>,
}

/// WASM value types
#[derive(Debug, Clone)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl WasmValue {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            WasmValue::I32(v) => serde_json::json!({"type": "i32", "value": v}),
            WasmValue::I64(v) => serde_json::json!({"type": "i64", "value": v}),
            WasmValue::F32(v) => serde_json::json!({"type": "f32", "value": v}),
            WasmValue::F64(v) => serde_json::json!({"type": "f64", "value": v}),
        }
    }
}