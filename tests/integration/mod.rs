//! Integration tests for Caxton multi-agent orchestration platform
//! 
//! These tests verify end-to-end functionality across all system components:
//! - Agent lifecycle management
//! - FIPA message protocol compliance  
//! - WebAssembly isolation boundaries
//! - Observability event emission
//! - Tool integration via MCP

pub mod agent_coordination;
pub mod fipa_messaging;
pub mod observability;
pub mod wasm_isolation;
pub mod performance_benchmarks;