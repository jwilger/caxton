//! Domain types for semantic routing with embedded vectors
//!
//! This module defines the core types for capability-based semantic routing
//! using embedded vector representations (All-MiniLM-L6-v2 model with 384 dimensions).
//!
//! Design principles:
//! - Make illegal states unrepresentable
//! - Parse, don't validate
//! - Use nutype for domain primitives
//! - Railway-oriented error handling

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// =============================================================================
// Domain Primitives
// =============================================================================

/// A capability identifier following kebab-case convention (e.g., "data-analysis")
#[nutype(
    sanitize(trim, lowercase),
    validate(predicate = |s: &str| s.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit()) && !s.starts_with('-') && !s.ends_with('-')),
    derive(Clone, Debug, Eq, PartialEq, Hash, Display, Serialize, Deserialize)
)]
pub struct CapabilityId(String);

/// Human-readable description of what a capability does
#[nutype(
    sanitize(trim),
    validate(len_char_min = 10, len_char_max = 500),
    derive(Clone, Debug, Eq, PartialEq, Display, Serialize, Deserialize)
)]
pub struct CapabilityDescription(String);

/// Agent identifier for routing decisions
#[nutype(
    sanitize(trim),
    validate(len_char_min = 1, len_char_max = 64),
    derive(Clone, Debug, Eq, PartialEq, Hash, Display, Serialize, Deserialize)
)]
pub struct AgentId(String);

/// Semantic similarity score between 0.0 and 1.0
#[nutype(
    validate(finite, greater_or_equal = 0.0, less_or_equal = 1.0),
    derive(Clone, Debug, PartialEq, PartialOrd, Display, Serialize, Deserialize)
)]
pub struct SimilarityScore(f32);

/// Threshold for semantic similarity matching
#[nutype(
    validate(finite, greater = 0.0, less_or_equal = 1.0),
    derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)
)]
pub struct SimilarityThreshold(f32);

// =============================================================================
// Embedding Types
// =============================================================================

/// A 384-dimensional embedding vector from All-MiniLM-L6-v2 model
#[derive(Clone, Debug, PartialEq)]
pub struct EmbeddingVector {
    /// The embedding values - exactly 384 dimensions for All-MiniLM-L6-v2
    values: Vec<f32>,
}

// Manual Serialize/Deserialize implementation for large arrays
impl Serialize for EmbeddingVector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.values.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EmbeddingVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values: Vec<f32> = Vec::deserialize(deserializer)?;
        if values.len() != 384 {
            return Err(serde::de::Error::custom(format!(
                "Expected exactly 384 dimensions, got {}",
                values.len()
            )));
        }
        Ok(Self { values })
    }
}

impl EmbeddingVector {
    /// Create a new embedding vector from exactly 384 values
    #[must_use]
    pub fn new(values: [f32; 384]) -> Self {
        Self {
            values: values.to_vec(),
        }
    }

    /// Try to create from a Vec, returning error if not exactly 384 dimensions
    ///
    /// # Errors
    ///
    /// Returns an error if the vector does not have exactly 384 dimensions
    pub fn try_from_vec(values: Vec<f32>) -> Result<Self, String> {
        if values.len() != 384 {
            return Err(format!(
                "Expected exactly 384 dimensions, got {}",
                values.len()
            ));
        }
        Ok(Self { values })
    }

    /// Calculate cosine similarity with another embedding
    ///
    /// # Panics
    ///
    /// This function should never panic in practice, but could theoretically panic
    /// if `SimilarityScore::try_new` fails (which shouldn't happen as the result
    /// is clamped to [0, 1])
    #[must_use]
    pub fn cosine_similarity(&self, other: &Self) -> SimilarityScore {
        let dot_product: f32 = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a * b)
            .sum();

        let magnitude_self: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_other: f32 = other.values.iter().map(|x| x * x).sum::<f32>().sqrt();

        let similarity = if magnitude_self > f32::EPSILON && magnitude_other > f32::EPSILON {
            dot_product / (magnitude_self * magnitude_other)
        } else {
            0.0
        };

        // Clamp to [0, 1] range to handle floating point precision issues
        SimilarityScore::try_new(similarity.clamp(0.0, 1.0))
            .expect("Cosine similarity always in valid range")
    }

    /// Get the raw values for interop with embedding libraries
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.values
    }
}

/// An embedded capability with its vector representation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedCapability {
    /// The capability identifier
    pub id: CapabilityId,
    /// Human-readable description of the capability
    pub description: CapabilityDescription,
    /// The embedding vector for semantic matching
    pub embedding: EmbeddingVector,
    /// The agent that provides this capability
    pub agent_id: AgentId,
}

// =============================================================================
// Routing Decision Types
// =============================================================================

/// Result of semantic similarity matching
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticMatch {
    /// The agent that matches the request
    pub agent_id: AgentId,
    /// The capability being matched
    pub capability_id: CapabilityId,
    /// The similarity score between request and capability
    pub similarity: SimilarityScore,
}

/// Strategy for selecting an agent when multiple matches exist
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SelectionStrategy {
    /// Pick the agent with highest similarity score
    HighestSimilarity,
    /// Round-robin among agents above threshold
    RoundRobin {
        /// Index of the last selected agent for round-robin
        last_selected_index: usize,
    },
    /// Random selection among qualified agents
    Random {
        /// Seed for random number generation
        seed: u64,
    },
}

/// Fallback strategy when no semantic matches meet threshold
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// Return an error - no suitable agent found
    RejectRequest,
    /// Route to a specific default agent
    DefaultAgent(AgentId),
    /// Lower the similarity threshold and retry
    LowerThreshold {
        /// The new threshold to use for matching
        new_threshold: SimilarityThreshold,
    },
    /// Use round-robin among all agents with the capability
    RoundRobinAll,
}

/// A routing decision with its reasoning
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// The agent selected to handle the request
    pub selected_agent: AgentId,
    /// The reason for this routing decision
    pub reason: RoutingReason,
    /// The similarity score if semantic matching was used
    pub similarity_score: Option<SimilarityScore>,
}

/// Reason for a routing decision (for observability)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RoutingReason {
    /// Semantic similarity above threshold
    SemanticMatch,
    /// Retrieved from cache
    CacheHit,
    /// Fallback strategy was used
    Fallback(FallbackStrategy),
    /// Direct capability match (non-semantic)
    DirectCapabilityMatch,
}

// =============================================================================
// Router Configuration
// =============================================================================

/// Configuration for the semantic router
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Minimum similarity score to consider a match
    pub similarity_threshold: SimilarityThreshold,
    /// Selection strategy when multiple agents match
    pub selection_strategy: SelectionStrategy,
    /// Fallback when no agents meet threshold
    pub fallback_strategy: FallbackStrategy,
    /// Enable caching of routing decisions
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u32,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: SimilarityThreshold::try_new(0.7)
                .expect("Default threshold is valid"),
            selection_strategy: SelectionStrategy::HighestSimilarity,
            fallback_strategy: FallbackStrategy::RejectRequest,
            cache_enabled: true,
            cache_ttl_seconds: 300, // 5 minutes
        }
    }
}

// =============================================================================
// Request/Response Types
// =============================================================================

/// A request to be routed to an agent
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoutingRequest {
    /// The embedded representation of the request
    pub embedding: EmbeddingVector,
    /// Optional preferred capability
    pub preferred_capability: Option<CapabilityId>,
    /// Request ID for tracing
    pub request_id: uuid::Uuid,
}

/// Cache key for routing decisions
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RoutingCacheKey {
    /// Hash of the embedding vector
    embedding_hash: u64,
    /// Optional capability filter
    capability_id: Option<CapabilityId>,
}

impl RoutingCacheKey {
    /// Create a cache key from an embedding vector
    #[must_use]
    pub fn from_embedding(embedding: &EmbeddingVector, capability: Option<CapabilityId>) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Hash the embedding values
        for value in embedding.as_slice() {
            // Convert to bits for consistent hashing of floats
            value.to_bits().hash(&mut hasher);
        }

        Self {
            embedding_hash: hasher.finish(),
            capability_id: capability,
        }
    }
}

// =============================================================================
// State Machine for Router Lifecycle
// =============================================================================

/// Phantom types for router state machine
pub mod router_states {
    /// Router state before initialization
    pub struct Uninitialized;
    /// Router state after initialization but before capability registration
    pub struct Initialized;
    /// Router state when ready to route requests
    pub struct Ready;
}

/// Semantic router with compile-time state tracking
pub struct SemanticRouter<State> {
    #[allow(dead_code)] // Will be used in implementation
    config: RouterConfig,
    _state: PhantomData<State>,
}

impl SemanticRouter<router_states::Uninitialized> {
    /// Create a new uninitialized router
    #[must_use]
    pub fn new(config: RouterConfig) -> Self {
        Self {
            config,
            _state: PhantomData,
        }
    }

    /// Initialize the router (load model, etc.)
    ///
    /// # Errors
    ///
    /// Returns `RouterError::NotInitialized` if initialization fails
    pub fn initialize(self) -> Result<SemanticRouter<router_states::Initialized>, RouterError> {
        unimplemented!("Implementation by others")
    }
}

impl SemanticRouter<router_states::Initialized> {
    /// Register capabilities and make router ready
    ///
    /// # Errors
    ///
    /// Returns `RouterError::InvalidConfiguration` if capabilities cannot be registered
    pub fn register_capabilities(
        self,
        _capabilities: Vec<EmbeddedCapability>,
    ) -> Result<SemanticRouter<router_states::Ready>, RouterError> {
        unimplemented!("Implementation by others")
    }
}

impl SemanticRouter<router_states::Ready> {
    /// Route a request to the best matching agent
    ///
    /// # Errors
    ///
    /// Returns `RouterError::NoAgentsAvailable` if no agents can handle the request,
    /// or `RouterError::BelowThreshold` if no agents meet the similarity threshold
    pub fn route(&self, _request: RoutingRequest) -> Result<RoutingDecision, RouterError> {
        unimplemented!("Implementation by others")
    }

    /// Update capability embeddings
    ///
    /// # Errors
    ///
    /// Returns `RouterError::InvalidConfiguration` if the capability cannot be updated
    pub fn update_capability(
        &mut self,
        _capability: EmbeddedCapability,
    ) -> Result<(), RouterError> {
        unimplemented!("Implementation by others")
    }
}

// =============================================================================
// Error Types
// =============================================================================

/// Errors that can occur during routing
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    /// No agents are available with the required capability
    #[error("No agents available with required capability")]
    NoAgentsAvailable,

    /// No agents meet the similarity threshold
    #[error("No agents meet similarity threshold of {threshold}")]
    BelowThreshold {
        /// The threshold that was not met
        threshold: SimilarityThreshold,
    },

    /// Failed to generate embedding for text
    #[error("Failed to generate embedding: {reason}")]
    EmbeddingError {
        /// The reason for embedding failure
        reason: String,
    },

    /// Router has not been initialized
    #[error("Router not initialized")]
    NotInitialized,

    /// Configuration is invalid
    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration {
        /// The reason the configuration is invalid
        reason: String,
    },
}

// =============================================================================
// Workflow Signatures (Implementation by others)
// =============================================================================

/// Generate embedding for a text description
///
/// # Errors
///
/// Returns `RouterError::EmbeddingError` if the embedding cannot be generated
pub fn generate_embedding(_text: &str) -> Result<EmbeddingVector, RouterError> {
    unimplemented!("Implementation by others - will use Candle.rs with All-MiniLM-L6-v2")
}

/// Find best matching agents for a request
#[must_use]
pub fn find_semantic_matches(
    _request: &RoutingRequest,
    _capabilities: &[EmbeddedCapability],
    _threshold: SimilarityThreshold,
) -> Vec<SemanticMatch> {
    unimplemented!("Implementation by others")
}

/// Select an agent from matches using the configured strategy
///
/// # Errors
///
/// Returns `RouterError::NoAgentsAvailable` if no agents can be selected
pub fn select_agent(
    _matches: Vec<SemanticMatch>,
    _strategy: &SelectionStrategy,
) -> Result<AgentId, RouterError> {
    unimplemented!("Implementation by others")
}

/// Apply fallback strategy when no semantic matches found
///
/// # Errors
///
/// Returns `RouterError::NoAgentsAvailable` if fallback cannot select an agent
pub fn apply_fallback(
    _strategy: &FallbackStrategy,
    _available_agents: &[AgentId],
) -> Result<AgentId, RouterError> {
    unimplemented!("Implementation by others")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical_vectors() {
        let values = [1.0; 384];
        let vec1 = EmbeddingVector::new(values);
        let vec2 = EmbeddingVector::new(values);

        let similarity = vec1.cosine_similarity(&vec2);
        assert!((similarity.into_inner() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal_vectors() {
        let mut values1 = [0.0; 384];
        let mut values2 = [0.0; 384];
        values1[0] = 1.0;
        values2[1] = 1.0;

        let vec1 = EmbeddingVector::new(values1);
        let vec2 = EmbeddingVector::new(values2);

        let similarity = vec1.cosine_similarity(&vec2);
        assert!(similarity.into_inner().abs() < 0.0001);
    }

    #[test]
    fn test_routing_cache_key_deterministic() {
        let values = [0.5; 384];
        let embedding = EmbeddingVector::new(values);
        let capability = CapabilityId::try_new("data-analysis").unwrap();

        let key1 = RoutingCacheKey::from_embedding(&embedding, Some(capability.clone()));
        let key2 = RoutingCacheKey::from_embedding(&embedding, Some(capability));

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_default_router_config() {
        let config = RouterConfig::default();
        assert!((config.similarity_threshold.into_inner() - 0.7).abs() < f32::EPSILON);
        assert_eq!(
            config.selection_strategy,
            SelectionStrategy::HighestSimilarity
        );
        assert_eq!(config.fallback_strategy, FallbackStrategy::RejectRequest);
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl_seconds, 300);
    }
}
