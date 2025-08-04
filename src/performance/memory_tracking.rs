//! # Memory Allocation Tracking and Optimization
//!
//! This module provides comprehensive memory monitoring and optimization for Caxton:
//! - Real-time memory allocation tracking
//! - Memory pool management for frequently allocated objects
//! - Garbage collection pressure monitoring
//! - Memory leak detection and reporting
//! - WASM memory usage optimization

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{instrument, debug, info, warn, error};
use metrics::{counter, gauge, histogram};
use smallvec::SmallVec;
use bytes::{Bytes, BytesMut};

/// Global memory allocator wrapper for tracking allocations
pub struct TrackedAllocator {
    inner: System,
    total_allocated: AtomicU64,
    total_deallocated: AtomicU64,
    current_usage: AtomicU64,
    peak_usage: AtomicU64,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
}

impl TrackedAllocator {
    pub const fn new() -> Self {
        Self {
            inner: System,
            total_allocated: AtomicU64::new(0),
            total_deallocated: AtomicU64::new(0),
            current_usage: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
        }
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_deallocated: self.total_deallocated.load(Ordering::Relaxed),
            current_usage: self.current_usage.load(Ordering::Relaxed),
            peak_usage: self.peak_usage.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
        }
    }

    /// Reset statistics (useful for testing)
    #[cfg(test)]
    pub fn reset_stats(&self) {
        self.total_allocated.store(0, Ordering::Relaxed);
        self.total_deallocated.store(0, Ordering::Relaxed);
        self.current_usage.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
        self.deallocation_count.store(0, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        
        if !ptr.is_null() {
            let size = layout.size() as u64;
            
            self.total_allocated.fetch_add(size, Ordering::Relaxed);
            self.allocation_count.fetch_add(1, Ordering::Relaxed);
            
            let current = self.current_usage.fetch_add(size, Ordering::Relaxed) + size;
            
            // Update peak usage
            let mut peak = self.peak_usage.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_usage.compare_exchange_weak(
                    peak, 
                    current, 
                    Ordering::Relaxed, 
                    Ordering::Relaxed
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
            
            // Update metrics
            counter!("caxton_memory_allocations_total", 1);
            counter!("caxton_memory_allocated_bytes_total", size);
            gauge!("caxton_memory_current_usage_bytes", current as f64);
        }
        
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size() as u64;
        
        self.total_deallocated.fetch_add(size, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        
        let current = self.current_usage.fetch_sub(size, Ordering::Relaxed) - size;
        
        counter!("caxton_memory_deallocations_total", 1);
        counter!("caxton_memory_deallocated_bytes_total", size);
        gauge!("caxton_memory_current_usage_bytes", current as f64);
        
        self.inner.dealloc(ptr, layout);
    }
}

/// Global tracked allocator instance
#[global_allocator]
static TRACKED_ALLOCATOR: TrackedAllocator = TrackedAllocator::new();

/// Get global memory statistics
pub fn get_global_memory_stats() -> MemoryStats {
    TRACKED_ALLOCATOR.get_stats()
}

/// Memory statistics structure
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_allocated: u64,
    pub total_deallocated: u64,
    pub current_usage: u64,
    pub peak_usage: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

impl MemoryStats {
    /// Calculate memory efficiency ratio
    pub fn efficiency_ratio(&self) -> f64 {
        if self.total_allocated == 0 {
            1.0
        } else {
            self.total_deallocated as f64 / self.total_allocated as f64
        }
    }

    /// Calculate average allocation size
    pub fn average_allocation_size(&self) -> f64 {
        if self.allocation_count == 0 {
            0.0
        } else {
            self.total_allocated as f64 / self.allocation_count as f64
        }
    }

    /// Check if there might be a memory leak
    pub fn potential_leak(&self) -> bool {
        // Simple heuristic: if current usage is > 90% of peak and we've done
        // significant allocations, there might be a leak
        self.allocation_count > 1000 && 
        self.current_usage as f64 > (self.peak_usage as f64 * 0.9)
    }
}

/// Memory pool for frequently allocated objects
#[derive(Debug)]
pub struct MemoryPool<T> {
    pool: Arc<RwLock<Vec<T>>>,
    max_size: usize,
    allocated_count: AtomicUsize,
    reused_count: AtomicUsize,
}

impl<T> MemoryPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            max_size,
            allocated_count: AtomicUsize::new(0),
            reused_count: AtomicUsize::new(0),
        }
    }

    /// Get an object from the pool or create a new one
    pub async fn get<F>(&self, factory: F) -> T
    where
        F: FnOnce() -> T,
    {
        // Try to get from pool first
        {
            let mut pool = self.pool.write().await;
            if let Some(item) = pool.pop() {
                self.reused_count.fetch_add(1, Ordering::Relaxed);
                counter!("caxton_memory_pool_reused_total", 1);
                return item;
            }
        }

        // Create new item if pool is empty
        let item = factory();
        self.allocated_count.fetch_add(1, Ordering::Relaxed);
        counter!("caxton_memory_pool_allocated_total", 1);
        item
    }

    /// Return an object to the pool
    pub async fn return_item(&self, item: T) {
        let mut pool = self.pool.write().await;
        if pool.len() < self.max_size {
            pool.push(item);
            counter!("caxton_memory_pool_returned_total", 1);
        }
        // If pool is full, item is dropped
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let pool = self.pool.read().await;
        PoolStats {
            pooled_items: pool.len(),
            total_allocated: self.allocated_count.load(Ordering::Relaxed),
            total_reused: self.reused_count.load(Ordering::Relaxed),
            max_size: self.max_size,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub pooled_items: usize,
    pub total_allocated: usize,
    pub total_reused: usize,
    pub max_size: usize,
}

impl PoolStats {
    pub fn reuse_ratio(&self) -> f64 {
        let total_requests = self.total_allocated + self.total_reused;
        if total_requests == 0 {
            0.0
        } else {
            self.total_reused as f64 / total_requests as f64
        }
    }
}

/// Memory monitor for tracking and optimizing memory usage patterns
#[derive(Debug)]
pub struct MemoryMonitor {
    /// Allocation tracking by category
    categories: Arc<RwLock<HashMap<String, CategoryStats>>>,
    /// Memory leak detection
    leak_detector: Arc<MemoryLeakDetector>,
    /// Configuration
    config: MemoryMonitorConfig,
}

#[derive(Debug, Clone)]
pub struct MemoryMonitorConfig {
    /// Monitoring interval
    pub monitoring_interval: Duration,
    /// Leak detection threshold
    pub leak_detection_threshold: f64,
    /// Maximum categories to track
    pub max_categories: usize,
}

impl Default for MemoryMonitorConfig {
    fn default() -> Self {
        Self {
            monitoring_interval: Duration::from_secs(30),
            leak_detection_threshold: 0.8, // 80% retention suggests leak
            max_categories: 100,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct CategoryStats {
    allocated_bytes: u64,
    allocation_count: u64,
    last_updated: Option<Instant>,
}

impl MemoryMonitor {
    pub fn new(config: MemoryMonitorConfig) -> Self {
        let monitor = Self {
            categories: Arc::new(RwLock::new(HashMap::new())),
            leak_detector: Arc::new(MemoryLeakDetector::new()),
            config,
        };

        monitor.start_monitoring();
        monitor
    }

    /// Track allocation for a specific category
    #[instrument(skip(self))]
    pub async fn track_allocation(&self, category: &str, size: u64) {
        let mut categories = self.categories.write().await;
        
        if categories.len() >= self.config.max_categories && !categories.contains_key(category) {
            warn!(category = category, "Maximum categories reached, ignoring new category");
            return;
        }

        let stats = categories.entry(category.to_string()).or_default();
        stats.allocated_bytes += size;
        stats.allocation_count += 1;
        stats.last_updated = Some(Instant::now());

        gauge!(format!("caxton_memory_category_bytes_{}", category), stats.allocated_bytes as f64);
        counter!(format!("caxton_memory_category_allocations_{}", category), 1);
    }

    /// Get memory usage by category
    #[instrument(skip(self))]
    pub async fn get_category_stats(&self) -> HashMap<String, CategoryStats> {
        self.categories.read().await.clone()
    }

    /// Generate memory optimization recommendations
    #[instrument(skip(self))]
    pub async fn get_optimization_recommendations(&self) -> Vec<MemoryOptimizationRecommendation> {
        let mut recommendations = Vec::new();
        let global_stats = get_global_memory_stats();
        let categories = self.get_category_stats().await;

        // Check for potential memory leaks
        if global_stats.potential_leak() {
            recommendations.push(MemoryOptimizationRecommendation {
                title: "Potential Memory Leak Detected".to_string(),
                description: format!(
                    "Current memory usage ({:.1}MB) is {:.1}% of peak usage. This may indicate a memory leak.",
                    global_stats.current_usage as f64 / 1_000_000.0,
                    (global_stats.current_usage as f64 / global_stats.peak_usage as f64) * 100.0
                ),
                priority: OptimizationPriority::High,
                actions: vec![
                    "Profile memory allocations to identify leak sources".to_string(),
                    "Review object lifecycle management".to_string(),
                    "Consider implementing reference counting for shared objects".to_string(),
                ],
            });
        }

        // Check for high allocation frequency
        if global_stats.allocation_count > 1_000_000 {
            recommendations.push(MemoryOptimizationRecommendation {
                title: "High Allocation Frequency".to_string(),
                description: format!(
                    "System has performed {} allocations with average size {:.1} bytes",
                    global_stats.allocation_count,
                    global_stats.average_allocation_size()
                ),
                priority: OptimizationPriority::Medium,
                actions: vec![
                    "Implement object pooling for frequently allocated objects".to_string(),
                    "Use pre-allocated buffers where possible".to_string(),
                    "Consider using stack allocation for small objects".to_string(),
                ],
            });
        }

        // Check categories for optimization opportunities
        let mut category_vec: Vec<_> = categories.into_iter().collect();
        category_vec.sort_by(|a, b| b.1.allocated_bytes.cmp(&a.1.allocated_bytes));

        if let Some((category, stats)) = category_vec.first() {
            if stats.allocated_bytes > 100_000_000 { // 100MB
                recommendations.push(MemoryOptimizationRecommendation {
                    title: format!("High Memory Usage in '{}'", category),
                    description: format!(
                        "Category '{}' has allocated {:.1}MB across {} allocations",
                        category,
                        stats.allocated_bytes as f64 / 1_000_000.0,
                        stats.allocation_count
                    ),
                    priority: OptimizationPriority::Medium,
                    actions: vec![
                        format!("Review memory usage patterns in '{}'", category),
                        "Consider using more efficient data structures".to_string(),
                        "Implement lazy loading where appropriate".to_string(),
                    ],
                });
            }
        }

        recommendations
    }

    fn start_monitoring(&self) {
        let categories = Arc::clone(&self.categories);
        let leak_detector = Arc::clone(&self.leak_detector);
        let interval = self.config.monitoring_interval;

        tokio::spawn(async move {
            let mut monitoring_interval = tokio::time::interval(interval);

            loop {
                monitoring_interval.tick().await;

                // Update global metrics
                let stats = get_global_memory_stats();
                gauge!("caxton_memory_total_allocated_bytes", stats.total_allocated as f64);
                gauge!("caxton_memory_total_deallocated_bytes", stats.total_deallocated as f64);
                gauge!("caxton_memory_peak_usage_bytes", stats.peak_usage as f64);
                gauge!("caxton_memory_efficiency_ratio", stats.efficiency_ratio());

                // Check for memory leaks
                leak_detector.check_for_leaks(stats).await;

                debug!(
                    current_usage_mb = stats.current_usage as f64 / 1_000_000.0,
                    peak_usage_mb = stats.peak_usage as f64 / 1_000_000.0,
                    efficiency = stats.efficiency_ratio(),
                    "Memory monitoring update"
                );
            }
        });
    }
}

/// Memory leak detector
#[derive(Debug)]
struct MemoryLeakDetector {
    previous_stats: Arc<RwLock<Option<MemoryStats>>>,
    leak_warnings: AtomicUsize,
}

impl MemoryLeakDetector {
    fn new() -> Self {
        Self {
            previous_stats: Arc::new(RwLock::new(None)),
            leak_warnings: AtomicUsize::new(0),
        }
    }

    async fn check_for_leaks(&self, current_stats: MemoryStats) {
        let mut previous = self.previous_stats.write().await;
        
        if let Some(prev_stats) = previous.as_ref() {
            // Check if memory usage is consistently growing
            let growth_rate = if prev_stats.current_usage > 0 {
                (current_stats.current_usage as f64 - prev_stats.current_usage as f64) 
                / prev_stats.current_usage as f64
            } else {
                0.0
            };

            // If memory grew by more than 10% and efficiency is low, warn about potential leak
            if growth_rate > 0.1 && current_stats.efficiency_ratio() < 0.8 {
                let warning_count = self.leak_warnings.fetch_add(1, Ordering::Relaxed) + 1;
                
                warn!(
                    growth_rate = growth_rate,
                    efficiency = current_stats.efficiency_ratio(),
                    warning_count = warning_count,
                    "Potential memory leak detected"
                );

                counter!("caxton_memory_leak_warnings_total", 1);
            }
        }
        
        *previous = Some(current_stats);
    }
}

/// Memory optimization recommendation
#[derive(Debug, Clone)]
pub struct MemoryOptimizationRecommendation {
    pub title: String,
    pub description: String,
    pub priority: OptimizationPriority,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Specialized memory pools for common Caxton objects
pub struct CaxtonMemoryPools {
    pub message_pool: MemoryPool<BytesMut>,
    pub agent_id_pool: MemoryPool<Vec<u8>>,
    pub conversation_pool: MemoryPool<HashMap<String, String>>,
}

impl CaxtonMemoryPools {
    pub fn new() -> Self {
        Self {
            message_pool: MemoryPool::new(1000),     // Pool up to 1000 message buffers
            agent_id_pool: MemoryPool::new(500),     // Pool up to 500 agent ID buffers
            conversation_pool: MemoryPool::new(200), // Pool up to 200 conversation maps
        }
    }

    /// Get a message buffer from the pool
    pub async fn get_message_buffer(&self) -> BytesMut {
        self.message_pool.get(|| BytesMut::with_capacity(1024)).await
    }

    /// Return a message buffer to the pool
    pub async fn return_message_buffer(&self, mut buffer: BytesMut) {
        buffer.clear(); // Clear but keep capacity
        self.message_pool.return_item(buffer).await;
    }

    /// Get performance statistics for all pools
    pub async fn get_all_pool_stats(&self) -> HashMap<String, PoolStats> {
        let mut stats = HashMap::new();
        
        stats.insert("message_pool".to_string(), self.message_pool.get_stats().await);
        stats.insert("agent_id_pool".to_string(), self.agent_id_pool.get_stats().await);
        stats.insert("conversation_pool".to_string(), self.conversation_pool.get_stats().await);
        
        stats
    }
}

impl Default for CaxtonMemoryPools {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_calculations() {
        let stats = MemoryStats {
            total_allocated: 1000,
            total_deallocated: 800,
            current_usage: 200,
            peak_usage: 500,
            allocation_count: 10,
            deallocation_count: 8,
        };

        assert_eq!(stats.efficiency_ratio(), 0.8);
        assert_eq!(stats.average_allocation_size(), 100.0);
        assert!(!stats.potential_leak()); // Not enough allocations for leak detection
    }

    #[tokio::test]
    async fn test_memory_pool_basic_operations() {
        let pool = MemoryPool::new(10);
        
        // Get item from empty pool (creates new)
        let item1 = pool.get(|| String::from("test")).await;
        assert_eq!(item1, "test");
        
        // Return item to pool
        pool.return_item(item1).await;
        
        // Get item from pool (reuses)
        let item2 = pool.get(|| String::from("should not be created")).await;
        assert_eq!(item2, "test"); // Reused the returned item
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_allocated, 1);
        assert_eq!(stats.total_reused, 1);
    }

    #[tokio::test]
    async fn test_memory_monitor_category_tracking() {
        let config = MemoryMonitorConfig::default();
        let monitor = MemoryMonitor::new(config);
        
        monitor.track_allocation("test_category", 1024).await;
        monitor.track_allocation("test_category", 512).await;
        
        let stats = monitor.get_category_stats().await;
        let test_stats = stats.get("test_category").unwrap();
        
        assert_eq!(test_stats.allocated_bytes, 1536);
        assert_eq!(test_stats.allocation_count, 2);
    }
}