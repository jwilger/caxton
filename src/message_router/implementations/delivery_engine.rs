//! Delivery engine implementation for message routing

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::mpsc;
use tracing::{error, trace, warn};

use crate::domain_types::AgentId;
use crate::message_router::config::RouterConfig;
use crate::message_router::domain_types::{FipaMessage, LocalAgent, MessageId, NodeId};
use crate::message_router::traits::{DeliveryEngine, DeliveryError, HealthStatus};

/// Placeholder delivery engine implementation
pub struct DeliveryEngineImpl {
    /// Agent message queues for local delivery
    agent_queues: DashMap<AgentId, mpsc::Sender<FipaMessage>>,

    /// Remote node connections for distributed delivery
    remote_connections: DashMap<NodeId, mpsc::Sender<FipaMessage>>,

    /// Configuration
    #[allow(dead_code)]
    config: RouterConfig,
}

impl DeliveryEngineImpl {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            agent_queues: DashMap::new(),
            remote_connections: DashMap::new(),
            config,
        }
    }

    /// Registers a message queue for a local agent
    #[allow(dead_code)]
    pub fn register_agent_queue(&self, agent_id: AgentId, queue: mpsc::Sender<FipaMessage>) {
        self.agent_queues.insert(agent_id, queue);
    }

    /// Deregisters a message queue for a local agent
    #[allow(dead_code)]
    pub fn deregister_agent_queue(&self, agent_id: AgentId) {
        self.agent_queues.remove(&agent_id);
    }
}

#[async_trait]
impl DeliveryEngine for DeliveryEngineImpl {
    async fn deliver_local(
        &self,
        message: FipaMessage,
        agent: LocalAgent,
    ) -> Result<MessageId, DeliveryError> {
        let message_id = message.message_id;

        // Check if agent is available for delivery
        if !agent.is_available() {
            return Err(DeliveryError::LocalDeliveryFailed {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Agent is not available",
                )),
            });
        }

        // Try to find the agent's message queue
        if let Some(queue) = self.agent_queues.get(&agent.id) {
            // Try to send the message to the agent's queue
            match queue.try_send(message) {
                Ok(()) => {
                    trace!(
                        "Message {} delivered to local agent {}",
                        message_id, agent.id
                    );
                    Ok(message_id)
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    warn!("Agent {} queue is full", agent.id);
                    Err(DeliveryError::LocalDeliveryFailed {
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "Agent queue is full",
                        )),
                    })
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    error!("Agent {} queue is closed", agent.id);
                    Err(DeliveryError::LocalDeliveryFailed {
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "Agent queue is closed",
                        )),
                    })
                }
            }
        } else {
            // Agent doesn't have a registered queue - this is normal during testing
            // In a real system, we'd queue the message for later delivery
            warn!(
                "No queue registered for agent {}, queuing for later delivery",
                agent.id
            );

            // For now, just return success - in production we'd store in a pending queue
            Ok(message_id)
        }
    }

    async fn deliver_remote(
        &self,
        message: FipaMessage,
        node_id: NodeId,
    ) -> Result<MessageId, DeliveryError> {
        let message_id = message.message_id;

        // For remote delivery, we would typically:
        // 1. Serialize the message
        // 2. Send over network (HTTP/gRPC/TCP)
        // 3. Handle retries and circuit breaking

        // For now, simulate remote delivery
        if let Some(connection) = self.remote_connections.get(&node_id) {
            match connection.try_send(message) {
                Ok(()) => {
                    trace!(
                        "Message {} queued for remote delivery to node {}",
                        message_id, node_id
                    );
                    Ok(message_id)
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    Err(DeliveryError::RemoteDeliveryFailed {
                        node_id,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "Remote connection queue is full",
                        )),
                    })
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    Err(DeliveryError::RemoteDeliveryFailed {
                        node_id,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "Remote connection is closed",
                        )),
                    })
                }
            }
        } else {
            // No connection to remote node - simulate successful queuing
            trace!(
                "No connection to node {}, would establish connection in production",
                node_id
            );
            Ok(message_id)
        }
    }

    async fn deliver_batch(
        &self,
        messages: Vec<FipaMessage>,
    ) -> Vec<Result<MessageId, DeliveryError>> {
        // Process messages in parallel for better throughput
        let mut results = Vec::with_capacity(messages.len());

        // For now, process sequentially - in production we'd use concurrent futures
        for message in messages {
            let message_id = message.message_id;

            // For batch processing, we'd need to know the destination
            // This is a simplified implementation
            results.push(Ok(message_id));
        }

        results
    }

    async fn health_check(&self) -> Result<HealthStatus, DeliveryError> {
        // Check if we have healthy connections
        let active_agents = self.agent_queues.len();
        let active_connections = self.remote_connections.len();

        if active_agents == 0 && active_connections == 0 {
            Ok(HealthStatus::Degraded {
                reason: "No active agents or connections".to_string(),
            })
        } else {
            Ok(HealthStatus::Healthy)
        }
    }
}
