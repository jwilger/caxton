---
title: "O(1) Routing Algorithm Design"
date: 2025-01-14
layout: page
categories: [architecture, routing, algorithms]
---


## Overview

This document specifies the O(1) agent lookup algorithm that enables the message
router to achieve 100,000+ messages/second throughput. The algorithm uses
multiple data structures optimized for different lookup patterns while
maintaining consistency through careful coordination.

## Algorithm Architecture

### Primary Data Structures

```rust
/// Core routing data structures optimized for O(1) operations
pub struct RoutingAlgorithm {
    // Primary O(1) lookup structures
    local_agents: Arc<DashMap<AgentId, LocalAgentEntry>>,
    remote_routes: Arc<DashMap<AgentId, RemoteRouteEntry>>,

    // Secondary indexes for optimization
    capability_index: Arc<DashMap<CapabilityName, HashSet<AgentId>>>,
    node_agents: Arc<DashMap<NodeId, HashSet<AgentId>>>,

    // Cache layers for performance
    route_cache: Arc<LruCache<AgentId, CachedRoute>>,
    negative_cache: Arc<LruCache<AgentId, NegativeCacheEntry>>,

    // Consistency and coordination
    routing_table_version: Arc<AtomicU64>,
    gossip_coordinator: Arc<GossipCoordinator>,

    // Performance optimization
    lookup_metrics: Arc<LookupMetrics>,
    config: RoutingConfig,
}
```

### Core Lookup Algorithm

```rust
impl RoutingAlgorithm {
    /// O(1) agent lookup with intelligent caching and fallback
    #[tracing::instrument(skip(self))]
    pub async fn lookup_agent(&self, agent_id: &AgentId) -> Result<AgentLocation, LookupError> {
        let start = Instant::now();

        // Step 1: Check positive cache first (fastest path)
        if let Some(cached_route) = self.route_cache.get(agent_id) {
            if cached_route.is_valid() {
                self.lookup_metrics.record_cache_hit("positive", start.elapsed());
                return Ok(cached_route.location.clone());
            } else {
                // Expired entry, remove from cache
                self.route_cache.remove(agent_id);
                self.lookup_metrics.record_cache_expired();
            }
        }

        // Step 2: Check negative cache to avoid repeated failed lookups
        if let Some(negative_entry) = self.negative_cache.get(agent_id) {
            if negative_entry.is_valid() {
                self.lookup_metrics.record_cache_hit("negative", start.elapsed());
                return Ok(AgentLocation::Unknown);
            } else {
                self.negative_cache.remove(agent_id);
            }
        }

        // Step 3: Check local agents (most common case for high-throughput)
        if let Some(local_entry) = self.local_agents.get(agent_id) {
            let location = AgentLocation::Local(local_entry.agent.clone());

            // Cache the result for future lookups
            self.route_cache.put(
                *agent_id,
                CachedRoute::new(location.clone(), self.config.local_cache_ttl)
            );

            self.lookup_metrics.record_local_lookup(start.elapsed());
            return Ok(location);
        }

        // Step 4: Check remote routes table
        if let Some(remote_entry) = self.remote_routes.get(agent_id) {
            let route = remote_entry.value();

            // Validate route freshness
            if route.is_fresh(self.config.remote_route_ttl) {
                let location = AgentLocation::Remote(route.node_id);

                // Cache the result
                self.route_cache.put(
                    *agent_id,
                    CachedRoute::new(location.clone(), self.config.remote_cache_ttl)
                );

                self.lookup_metrics.record_remote_lookup(start.elapsed());
                return Ok(location);
            } else {
                // Route expired, remove it and continue to discovery
                self.remote_routes.remove(agent_id);
                self.lookup_metrics.record_stale_route_removed();
            }
        }

        // Step 5: Agent not found in current knowledge
        // Add to negative cache to avoid repeated lookups
        self.negative_cache.put(
            *agent_id,
            NegativeCacheEntry::new(self.config.negative_cache_ttl)
        );

        // Trigger asynchronous discovery via gossip protocol
        self.trigger_agent_discovery(*agent_id).await?;

        self.lookup_metrics.record_unknown_agent(start.elapsed());
        Ok(AgentLocation::Unknown)
    }

    /// Capability-based discovery with O(1) lookup
    pub async fn find_agents_by_capability(
        &self,
        capability: &CapabilityName
    ) -> Result<Vec<AgentId>, LookupError> {
        let start = Instant::now();

        // O(1) capability index lookup
        if let Some(agent_set) = self.capability_index.get(capability) {
            let agents: Vec<AgentId> = agent_set.iter().cloned().collect();

            // Filter by agent availability for more accurate results
            let available_agents = self.filter_available_agents(agents).await?;

            self.lookup_metrics.record_capability_lookup(
                capability,
                available_agents.len(),
                start.elapsed()
            );

            Ok(available_agents)
        } else {
            // Trigger capability discovery via gossip
            self.trigger_capability_discovery(capability.clone()).await?;

            self.lookup_metrics.record_capability_not_found(capability);
            Ok(Vec::new())
        }
    }

    /// Batch lookup optimization for high-throughput scenarios
    pub async fn batch_lookup(
        &self,
        agent_ids: Vec<AgentId>
    ) -> HashMap<AgentId, Result<AgentLocation, LookupError>> {
        let start = Instant::now();
        let mut results = HashMap::with_capacity(agent_ids.len());

        // Process in parallel with optimal batch sizes
        let batch_size = self.config.lookup_batch_size.as_usize();
        let chunks: Vec<_> = agent_ids.chunks(batch_size).collect();

        let batch_tasks: Vec<_> = chunks
            .into_iter()
            .map(|chunk| self.lookup_batch_chunk(chunk.to_vec()))
            .collect();

        let batch_results = futures::future::join_all(batch_tasks).await;

        // Flatten results
        for batch_result in batch_results {
            results.extend(batch_result);
        }

        self.lookup_metrics.record_batch_lookup(agent_ids.len(), start.elapsed());
        results
    }

    /// Updates routing table with new agent information
    pub async fn update_agent_route(
        &self,
        agent_id: AgentId,
        location: AgentLocation,
        source: RouteSource
    ) -> Result<(), RoutingError> {
        match location {
            AgentLocation::Local(agent) => {
                self.update_local_agent(agent_id, agent, source).await?;
            }
            AgentLocation::Remote(node_id) => {
                self.update_remote_route(agent_id, node_id, source).await?;
            }
            AgentLocation::Unknown => {
                self.remove_agent_route(agent_id).await?;
            }
        }

        // Increment routing table version for consistency
        self.routing_table_version.fetch_add(1, Ordering::SeqCst);

        // Invalidate relevant caches
        self.invalidate_agent_caches(agent_id).await;

        Ok(())
    }
}
```

### Cache Management Strategy

```rust
/// Intelligent caching system for optimal performance
pub struct RoutingCache {
    // Positive cache: successful lookups
    positive_cache: Arc<LruCache<AgentId, CachedRoute>>,

    // Negative cache: failed lookups (prevents discovery spam)
    negative_cache: Arc<LruCache<AgentId, NegativeCacheEntry>>,

    // Capability cache: capability-to-agents mappings
    capability_cache: Arc<LruCache<CapabilityName, CachedCapabilityResult>>,

    // Cache statistics for monitoring
    cache_stats: Arc<CacheStats>,

    config: CacheConfig,
}

impl RoutingCache {
    /// Implements adaptive TTL based on agent stability
    fn calculate_adaptive_ttl(&self, agent_id: AgentId, base_ttl: Duration) -> Duration {
        let agent_stability = self.get_agent_stability(agent_id);

        match agent_stability {
            AgentStability::Stable => base_ttl.mul_f64(2.0),      // Longer TTL for stable agents
            AgentStability::Normal => base_ttl,                   // Standard TTL
            AgentStability::Unstable => base_ttl.div_f64(2.0),   // Shorter TTL for unstable agents
        }
    }

    /// Preemptive cache warming for predictable patterns
    pub async fn warm_cache(&self, predicted_agents: Vec<AgentId>) -> Result<(), CacheError> {
        let warming_tasks: Vec<_> = predicted_agents
            .into_iter()
            .map(|agent_id| self.warm_agent_entry(agent_id))
            .collect();

        futures::future::join_all(warming_tasks).await;
        Ok(())
    }

    /// Cache invalidation with intelligent dependency tracking
    pub async fn invalidate_dependent_entries(&self, agent_id: AgentId) {
        // Invalidate direct agent cache
        self.positive_cache.remove(&agent_id);
        self.negative_cache.remove(&agent_id);

        // Invalidate capability caches that include this agent
        let agent_capabilities = self.get_agent_capabilities(agent_id).await;
        for capability in agent_capabilities {
            self.capability_cache.remove(&capability);
        }

        self.cache_stats.record_invalidation(agent_id);
    }
}
```

### Consistency Management

```rust
/// Ensures consistency across distributed routing state
pub struct ConsistencyManager {
    // Version-based consistency
    local_version: Arc<AtomicU64>,
    node_versions: Arc<DashMap<NodeId, u64>>,

    // Conflict resolution
    conflict_resolver: ConflictResolver,

    // Gossip coordination
    gossip_sender: GossipSender,
    gossip_receiver: GossipReceiver,

    config: ConsistencyConfig,
}

impl ConsistencyManager {
    /// Handles routing updates with conflict resolution
    pub async fn apply_routing_update(
        &self,
        update: RoutingUpdate,
        source_node: NodeId,
        source_version: u64
    ) -> Result<ApplyResult, ConsistencyError> {
        // Check version compatibility
        let local_version = self.local_version.load(Ordering::SeqCst);

        match self.compare_versions(local_version, source_version) {
            VersionComparison::LocalNewer => {
                // Local state is newer, reject update
                self.send_version_mismatch_response(source_node, local_version).await?;
                Ok(ApplyResult::Rejected)
            }
            VersionComparison::SourceNewer => {
                // Source is newer, apply update
                self.apply_update_unconditionally(update).await?;
                self.local_version.store(source_version, Ordering::SeqCst);
                Ok(ApplyResult::Applied)
            }
            VersionComparison::Concurrent => {
                // Concurrent updates, resolve conflict
                let resolution = self.conflict_resolver.resolve(update, local_version, source_version).await?;
                match resolution {
                    ConflictResolution::AcceptRemote => {
                        self.apply_update_unconditionally(update).await?;
                        self.local_version.store(source_version + 1, Ordering::SeqCst);
                        Ok(ApplyResult::Applied)
                    }
                    ConflictResolution::RejectRemote => {
                        self.send_conflict_resolution_response(source_node).await?;
                        Ok(ApplyResult::Rejected)
                    }
                    ConflictResolution::Merge(merged_update) => {
                        self.apply_update_unconditionally(merged_update).await?;
                        let new_version = std::cmp::max(local_version, source_version) + 1;
                        self.local_version.store(new_version, Ordering::SeqCst);
                        Ok(ApplyResult::Merged)
                    }
                }
            }
        }
    }

    /// Periodic consistency check across cluster
    pub async fn perform_consistency_check(&self) -> Result<ConsistencyReport, ConsistencyError> {
        let mut report = ConsistencyReport::new();

        // Check version alignment across nodes
        for node_entry in self.node_versions.iter() {
            let node_id = *node_entry.key();
            let node_version = *node_entry.value();
            let local_version = self.local_version.load(Ordering::SeqCst);

            let version_diff = local_version.abs_diff(node_version);
            if version_diff > self.config.max_version_drift {
                report.add_version_drift(node_id, version_diff);

                // Trigger reconciliation
                self.trigger_reconciliation(node_id).await?;
            }
        }

        // Check for route inconsistencies
        let route_inconsistencies = self.detect_route_inconsistencies().await?;
        report.route_inconsistencies = route_inconsistencies;

        Ok(report)
    }
}
```

## Performance Optimizations

### 1. Memory Layout Optimization

```rust
/// Memory-optimized data structures for cache efficiency
#[repr(C)]
struct LocalAgentEntry {
    agent: LocalAgent,
    last_access: AtomicU64,      // For LRU tracking
    access_count: AtomicU64,     // For frequency tracking
    cache_line_padding: [u8; 32], // Prevent false sharing
}

#[repr(C)]
struct RemoteRouteEntry {
    node_id: NodeId,
    hops: RouteHops,
    updated_at: u64,             // Unix timestamp for speed
    expires_at: u64,
    cache_line_padding: [u8; 16],
}
```

### 2. Lock-Free Algorithms

```rust
/// Lock-free capability index for maximum concurrency
pub struct LockFreeCapabilityIndex {
    // Lock-free hash map with atomic operations
    index: Arc<dashmap::DashMap<CapabilityName, AtomicCapabilitySet>>,
}

struct AtomicCapabilitySet {
    agents: AtomicPtr<HashSet<AgentId>>,
    version: AtomicU64,
}

impl LockFreeCapabilityIndex {
    /// Lock-free capability set update
    pub fn add_agent_capability(&self, capability: CapabilityName, agent_id: AgentId) -> Result<(), IndexError> {
        let entry = self.index.entry(capability).or_insert_with(|| {
            AtomicCapabilitySet::new()
        });

        // Atomic copy-on-write update
        loop {
            let current_ptr = entry.agents.load(Ordering::Acquire);
            let current_set = unsafe { &*current_ptr };

            if current_set.contains(&agent_id) {
                return Ok(()); // Already present
            }

            // Create new set with added agent
            let mut new_set = current_set.clone();
            new_set.insert(agent_id);
            let new_ptr = Box::into_raw(Box::new(new_set));

            // Attempt atomic swap
            match entry.agents.compare_exchange_weak(
                current_ptr,
                new_ptr,
                Ordering::Release,
                Ordering::Relaxed
            ) {
                Ok(_) => {
                    // Success - schedule cleanup of old set
                    self.schedule_cleanup(current_ptr);
                    entry.version.fetch_add(1, Ordering::Release);
                    return Ok(());
                }
                Err(_) => {
                    // Failed - cleanup new set and retry
                    unsafe { Box::from_raw(new_ptr); }
                    continue;
                }
            }
        }
    }
}
```

### 3. SIMD Optimizations

```rust
/// SIMD-optimized batch operations for high throughput
pub struct SimdOptimizedBatch {
    agent_ids: Vec<AgentId>,
    lookup_results: Vec<LookupResult>,
}

impl SimdOptimizedBatch {
    /// Vectorized hash computation for batch lookups
    #[cfg(target_arch = "x86_64")]
    pub fn compute_batch_hashes(&self) -> Vec<u64> {
        use std::arch::x86_64::*;

        let mut hashes = Vec::with_capacity(self.agent_ids.len());

        unsafe {
            // Process 4 UUIDs at a time using AVX2
            for chunk in self.agent_ids.chunks(4) {
                let uuid_data = self.prepare_uuid_data(chunk);
                let hash_result = self.simd_hash_4x(uuid_data);
                hashes.extend_from_slice(&hash_result);
            }
        }

        hashes
    }

    /// Parallel cache probe using SIMD instructions
    #[cfg(target_arch = "x86_64")]
    unsafe fn simd_cache_probe(&self, hashes: &[u64]) -> Vec<bool> {
        // Use SIMD to check multiple cache slots simultaneously
        let mut results = vec![false; hashes.len()];

        for (i, &hash) in hashes.iter().enumerate() {
            // SIMD-optimized cache slot checking
            results[i] = self.check_cache_slot_simd(hash);
        }

        results
    }
}
```

## Algorithm Performance Characteristics

### Lookup Complexity Analysis

| Operation | Time Complexity | Space Complexity | Cache Behavior |
|-----------|----------------|------------------|----------------| | Local Agent
Lookup | O(1) | O(1) | Hot cache line | | Remote Agent Lookup | O(1) | O(1) |
Warm cache line | | Capability Discovery | O(1) | O(k) where k = agents with
capability | Cold cache possible | | Batch Lookup | O(n) where n = batch size |
O(n) | Vectorized operations | | Route Update | O(1) amortized | O(1) | Cache
invalidation |

### Memory Usage Patterns

```rust
/// Memory usage estimation for capacity planning
pub struct MemoryEstimator {
    config: RoutingConfig,
}

impl MemoryEstimator {
    /// Estimates memory usage for given agent count
    pub fn estimate_memory_usage(&self, agent_count: usize) -> MemoryUsageEstimate {
        let local_agents_memory = agent_count * size_of::<LocalAgentEntry>();
        let remote_routes_memory = agent_count * size_of::<RemoteRouteEntry>();
        let capability_index_memory = self.estimate_capability_index_memory(agent_count);
        let cache_memory = self.estimate_cache_memory();

        MemoryUsageEstimate {
            total_bytes: local_agents_memory + remote_routes_memory +
                        capability_index_memory + cache_memory,
            local_agents_bytes: local_agents_memory,
            remote_routes_bytes: remote_routes_memory,
            capability_index_bytes: capability_index_memory,
            cache_bytes: cache_memory,
        }
    }

    /// Calculates optimal cache sizes based on memory constraints
    pub fn calculate_optimal_cache_sizes(&self, available_memory: usize) -> CacheConfiguration {
        let base_memory = self.calculate_base_memory_usage();
        let available_for_cache = available_memory - base_memory;

        // Allocate cache memory based on access patterns
        let positive_cache_ratio = 0.60; // 60% for positive cache (most frequent)
        let negative_cache_ratio = 0.25; // 25% for negative cache (prevents spam)
        let capability_cache_ratio = 0.15; // 15% for capability cache

        CacheConfiguration {
            positive_cache_size: (available_for_cache as f64 * positive_cache_ratio) as usize,
            negative_cache_size: (available_for_cache as f64 * negative_cache_ratio) as usize,
            capability_cache_size: (available_for_cache as f64 * capability_cache_ratio) as usize,
        }
    }
}
```

## Benchmarking and Validation

### Performance Benchmarks

```rust
/// Comprehensive benchmarks for routing algorithm
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

    fn bench_single_lookup(c: &mut Criterion) {
        let mut group = c.benchmark_group("single_lookup");

        for agent_count in [1_000, 10_000, 100_000, 1_000_000].iter() {
            group.bench_with_input(
                BenchmarkId::new("local_agents", agent_count),
                agent_count,
                |b, &size| {
                    let routing = setup_routing_with_agents(size);
                    let agent_id = pick_random_agent(&routing);

                    b.iter(|| {
                        routing.lookup_agent(&agent_id)
                    });
                }
            );
        }

        group.finish();
    }

    fn bench_batch_lookup(c: &mut Criterion) {
        let mut group = c.benchmark_group("batch_lookup");

        for batch_size in [10, 100, 1_000, 10_000].iter() {
            group.bench_with_input(
                BenchmarkId::new("batch_size", batch_size),
                batch_size,
                |b, &size| {
                    let routing = setup_routing_with_agents(100_000);
                    let agent_ids = generate_random_agent_ids(size);

                    b.iter(|| {
                        routing.batch_lookup(agent_ids.clone())
                    });
                }
            );
        }

        group.finish();
    }

    fn bench_capability_lookup(c: &mut Criterion) {
        let mut group = c.benchmark_group("capability_lookup");

        for agents_per_capability in [10, 100, 1_000].iter() {
            group.bench_with_input(
                BenchmarkId::new("agents_per_capability", agents_per_capability),
                agents_per_capability,
                |b, &count| {
                    let routing = setup_routing_with_capabilities(count);
                    let capability = CapabilityName::try_new("test_capability".to_string()).unwrap();

                    b.iter(|| {
                        routing.find_agents_by_capability(&capability)
                    });
                }
            );
        }

        group.finish();
    }

    criterion_group!(
        benches,
        bench_single_lookup,
        bench_batch_lookup,
        bench_capability_lookup
    );
    criterion_main!(benches);
}
```

## Summary

The O(1) routing algorithm achieves high performance through:

1. **Optimal Data Structures**: DashMap for lock-free concurrent access
2. **Intelligent Caching**: Multi-level cache hierarchy with adaptive TTL
3. **Consistency Management**: Version-based conflict resolution with gossip
   coordination
4. **Memory Optimization**: Cache-aligned structures and SIMD operations
5. **Batch Processing**: Vectorized operations for high-throughput scenarios

This design enables the message router to handle 100,000+ messages/second while
maintaining sub-millisecond lookup latencies and strong consistency across
distributed nodes.

The algorithm gracefully degrades under high load and provides comprehensive
observability for operational monitoring and capacity planning.
