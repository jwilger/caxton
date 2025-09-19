//! Domain types for Caxton multi-agent orchestration server

pub mod primitives;

pub use primitives::*;

/// =============================================================================
/// CORE BUSINESS WORKFLOW: Execute Agent Request
/// =============================================================================
/// This is the most fundamental workflow that delivers business value.
/// From requirements: "5-10 minute agent creation" and on-demand execution model.
/// Execute an agent request using the on-demand execution model
///
/// This is the core business workflow that processes user requests through
/// configuration-driven agents. From ADR-0044, agents are spawned as fresh
/// processes per request and exit naturally after completion.
///
/// # Errors
///
/// Returns `ExecutionError` when:
/// - System failures prevent processing (LLM unreachable, network timeouts)
/// - Resource constraints are exceeded (memory, processing limits)
/// - Runtime errors during agent execution
pub fn execute_agent_request(
    _config: AgentConfig,   // Pre-loaded agent configuration (pure function)
    _request: AgentRequest, // The user's request with context
) -> Result<AgentAction, ExecutionError> {
    // Returns Ok(action) for successful processing or tool call requests,
    // Err(error) for system failures that prevent processing
    unimplemented!()
}
