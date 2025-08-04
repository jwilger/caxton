//! Error types for Caxton

use crate::*;

/// Main error type for Caxton operations
#[derive(Error, Debug)]
pub enum CaxtonError {
    #[error("Agent error: {0}")]
    Agent(String),

    #[error("FIPA message error: {0}")]
    InvalidMessage(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("WASM error: {0}")]
    Wasm(String),

    #[error("WASM runtime error: {0}")]
    WasmRuntimeError(String),

    #[error("WASM validation error: {0}")]
    WasmValidationError(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(crate::core::agent::AgentId),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Invalid FIPA message: {0}")]
    InvalidFipaMessage(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
