//! Domain Primitives

/// User request to be processed by an agent
///
/// Contains the user's input and any context needed for agent execution.
/// From ADR-0044: "on-demand execution model" - fresh process per request
#[derive(Debug, Clone)]
pub struct AgentRequest;

/// Configuration for an agent
///
/// Pre-loaded agent configuration data for execution.
/// From ADR-0045: pure function principle - all needed data passed as parameters.
#[derive(Debug, Clone)]
pub struct AgentConfig;

/// Action that results from agent request processing
///
/// Represents what the agent determined should happen next.
/// From ADR-0046: agent tool call pattern for workflow continuation.
#[derive(Debug, Clone)]
pub struct AgentAction;

/// Errors that can occur during agent execution
///
/// Covers all failure modes for the `execute_agent_request` workflow.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {}
