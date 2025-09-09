//! REST API module for Caxton management interface
//!
//! Provides HTTP endpoints for managing Caxton agents and server operations.
//! Following ADR-0026 and ADR-0027 for JSON over HTTP with shared types.

use crate::domain_types::{AgentId, AgentName};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use uuid::Uuid;

/// Maximum allowed memory per agent in bytes (1GB)
const MAX_AGENT_MEMORY_BYTES: u64 = 1_073_741_824;

/// Maximum allowed CPU time per agent in milliseconds
const MAX_AGENT_CPU_MILLIS: u64 = 1_000_000;

/// Health check response for the /api/v1/health endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// The status of the service, always "healthy" for a minimal implementation
    pub status: String,
}

/// Resource limits for agent deployment using domain types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU time in milliseconds
    pub max_cpu_millis: u64,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
}

impl ResourceLimits {
    /// Validate resource limits against domain constraints
    ///
    /// # Errors
    ///
    /// Returns a string error if any resource limit is zero or exceeds maximum bounds
    pub fn validate(&self) -> Result<(), String> {
        if self.max_memory_bytes == 0 {
            return Err("max_memory_bytes must be greater than 0".to_string());
        }
        if self.max_cpu_millis == 0 {
            return Err("max_cpu_millis must be greater than 0".to_string());
        }
        if self.max_execution_time_ms == 0 {
            return Err("max_execution_time_ms must be greater than 0".to_string());
        }
        // Add domain-specific validation
        if self.max_memory_bytes > MAX_AGENT_MEMORY_BYTES {
            return Err(format!(
                "max_memory_bytes exceeds maximum limit of {MAX_AGENT_MEMORY_BYTES} bytes"
            ));
        }
        if self.max_cpu_millis > MAX_AGENT_CPU_MILLIS {
            return Err(format!(
                "max_cpu_millis exceeds maximum limit of {MAX_AGENT_CPU_MILLIS} ms"
            ));
        }
        Ok(())
    }
}

/// Agent deployment request with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDeploymentRequest {
    /// Name of the agent
    pub name: String,
    /// WASM module bytecode
    pub wasm_module: String,
    /// Resource limits for the agent
    pub resource_limits: ResourceLimits,
}

impl AgentDeploymentRequest {
    /// Parse and validate the deployment request, converting to domain types
    ///
    /// # Errors
    ///
    /// Returns a string error if agent name validation fails, WASM module is empty, or resource limits are invalid
    pub fn parse(self) -> Result<ValidatedDeploymentRequest, String> {
        let agent_name = AgentName::try_new(self.name.trim().to_string())
            .map_err(|e| format!("Invalid agent name: {e}"))?;

        if self.wasm_module.trim().is_empty() {
            return Err("WASM module cannot be empty".to_string());
        }

        self.resource_limits.validate()?;

        Ok(ValidatedDeploymentRequest {
            name: agent_name,
            wasm_module: self.wasm_module,
            resource_limits: self.resource_limits,
        })
    }
}

/// Validated deployment request with domain types
pub struct ValidatedDeploymentRequest {
    /// Validated agent name
    pub name: AgentName,
    /// WASM module bytecode
    pub wasm_module: String,
    /// Validated resource limits
    pub resource_limits: ResourceLimits,
}

/// Agent representation with domain types for internal use
#[derive(Debug, Clone)]
pub struct DomainAgent {
    /// Unique agent identifier
    pub id: AgentId,
    /// Agent name
    pub name: AgentName,
    /// WASM module bytecode
    pub wasm_module: String,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Agent representation for API serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique agent identifier
    pub id: String,
    /// Agent name
    pub name: String,
    /// WASM module bytecode
    pub wasm_module: String,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl From<DomainAgent> for Agent {
    fn from(domain_agent: DomainAgent) -> Self {
        Self {
            id: domain_agent.id.to_string(),
            name: domain_agent.name.to_string(),
            wasm_module: domain_agent.wasm_module,
            resource_limits: domain_agent.resource_limits,
        }
    }
}

/// Agent deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResponse {
    /// The generated ID for the deployed agent
    pub id: String,
}

impl DeploymentResponse {
    /// Create a new deployment response from an agent ID
    #[must_use]
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            id: agent_id.to_string(),
        }
    }
}

/// Error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Optional detailed message
    pub details: Option<String>,
}

/// Agent storage with domain types internally
pub type AgentStore = Arc<Mutex<HashMap<AgentId, DomainAgent>>>;

/// Creates the Axum application router with all API endpoints
pub fn create_app() -> Router {
    // Create fresh agent store for each server instance
    let agent_store: AgentStore = Arc::new(Mutex::new(HashMap::new()));

    Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/agents", get(list_agents).post(deploy_agent))
        .route("/api/v1/agents/{id}", get(get_agent_by_id))
        .with_state(agent_store)
}

/// Starts the HTTP server on the specified address
///
/// # Errors
///
/// Returns an error if the server fails to bind to the address or serve requests
pub async fn start_server(
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_app();
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Handler for the health check endpoint
/// Returns a hardcoded "healthy" status to make the test pass
async fn health_check() -> Json<HealthCheckResponse> {
    Json(HealthCheckResponse {
        status: "healthy".to_string(),
    })
}

/// Handler for the list agents endpoint
/// Returns all deployed agents converted from domain types
async fn list_agents(State(store): State<AgentStore>) -> Json<Vec<Agent>> {
    let agents = store.lock().unwrap();
    let agent_list: Vec<Agent> = agents.values().cloned().map(Agent::from).collect();
    Json(agent_list)
}

/// Handler for the deploy agent endpoint
/// Creates a new agent with proper domain type validation
async fn deploy_agent(
    State(store): State<AgentStore>,
    Json(request): Json<AgentDeploymentRequest>,
) -> impl axum::response::IntoResponse {
    // Parse and validate using domain types at the boundary
    let validated_request = match request.parse() {
        Ok(req) => req,
        Err(error_msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid deployment request".to_string(),
                    details: Some(error_msg),
                }),
            )
                .into_response();
        }
    };

    // Generate a unique ID using domain type
    let agent_id = AgentId::generate();

    // Create the domain agent
    let domain_agent = DomainAgent {
        id: agent_id,
        name: validated_request.name,
        wasm_module: validated_request.wasm_module,
        resource_limits: validated_request.resource_limits,
    };

    // Store the agent in our storage
    let mut agents = store.lock().unwrap();
    agents.insert(agent_id, domain_agent);

    // Return the agent ID in the response
    (StatusCode::CREATED, Json(DeploymentResponse::new(agent_id))).into_response()
}

/// Handler for the get agent by ID endpoint
/// Returns the stored agent details or error responses
async fn get_agent_by_id(
    State(store): State<AgentStore>,
    Path(id_str): Path<String>,
) -> impl axum::response::IntoResponse {
    // Parse the ID string to AgentId domain type
    let agent_id = match id_str.parse::<Uuid>() {
        Ok(uuid) => AgentId::new(uuid),
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid agent ID format".to_string(),
                    details: Some("Agent ID must be a valid UUID".to_string()),
                }),
            )
                .into_response();
        }
    };

    let agents = store.lock().unwrap();

    match agents.get(&agent_id) {
        Some(domain_agent) => {
            let api_agent = Agent::from(domain_agent.clone());
            (StatusCode::OK, Json(api_agent)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Agent not found".to_string(),
                details: Some(format!("No agent found with ID: {id_str}")),
            }),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_response_can_be_created() {
        let response = HealthCheckResponse {
            status: "healthy".to_string(),
        };
        assert_eq!(response.status, "healthy");
    }
}
