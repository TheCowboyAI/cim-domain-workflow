//! Memory management and monitoring utilities

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, atomic::{AtomicU64, AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Memory manager for workflow operations
pub struct MemoryManager {
    /// Memory pools for different object types
    pools: Arc<RwLock<HashMap<String, MemoryPool>>>,
    /// Memory usage statistics
    stats: Arc<MemoryStats>,
    /// Memory management configuration
    config: MemoryConfig,
    /// Garbage collection scheduler
    gc_scheduler: Arc<RwLock<GcScheduler>>,
}

/// Memory pool for specific object types
pub struct MemoryPool {
    /// Pool name
    pub name: String,
    /// Total allocated memory
    pub total_allocated: AtomicU64,
    /// Currently used memory
    pub used_memory: AtomicU64,
    /// Peak memory usage
    pub peak_memory: AtomicU64,
    /// Number of allocations
    pub allocation_count: AtomicU64,
    /// Number of deallocations
    pub deallocation_count: AtomicU64,
    /// Pool configuration
    pub config: PoolConfig,
    /// Recent allocation history
    pub allocation_history: Arc<RwLock<VecDeque<AllocationRecord>>>,
}

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum pool size in bytes
    pub max_size: u64,
    /// Preallocation size
    pub prealloc_size: u64,
    /// Growth strategy
    pub growth_strategy: GrowthStrategy,
    /// Cleanup threshold
    pub cleanup_threshold: f64,
    /// Enable detailed tracking
    pub detailed_tracking: bool,
}

/// Pool growth strategies
#[derive(Debug, Clone)]
pub enum GrowthStrategy {
    /// Linear growth by fixed amount
    Linear(u64),
    /// Exponential growth by multiplier
    Exponential(f64),
    /// No automatic growth
    Fixed,
}

/// Memory allocation record
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    /// Allocation ID
    pub id: String,
    /// Size allocated
    pub size: u64,
    /// Allocation timestamp
    pub timestamp: Instant,
    /// Object type
    pub object_type: String,
    /// Stack trace (if enabled)
    pub stack_trace: Option<Vec<String>>,
}

/// Overall memory statistics
pub struct MemoryStats {
    /// Total system memory used
    pub total_used: AtomicU64,
    /// Peak memory usage
    pub peak_usage: AtomicU64,
    /// Number of active allocations
    pub active_allocations: AtomicUsize,
    /// Total allocations made
    pub total_allocations: AtomicU64,
    /// Memory fragmentation ratio
    pub fragmentation_ratio: AtomicU64, // Stored as fixed-point (multiply by 1000)
    /// Last GC run time
    pub last_gc_time: Arc<RwLock<Option<SystemTime>>>,
    /// GC statistics
    pub gc_stats: Arc<RwLock<GarbageCollectionStats>>,
}

/// Garbage collection statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GarbageCollectionStats {
    /// Number of GC cycles
    pub cycles: u64,
    /// Total time spent in GC
    pub total_time: Duration,
    /// Memory freed in last GC
    pub last_freed: u64,
    /// Total memory freed
    pub total_freed: u64,
    /// Average GC time
    pub avg_time: Duration,
    /// Objects collected in last GC
    pub last_objects_collected: u64,
    /// Total objects collected
    pub total_objects_collected: u64,
}

/// Memory management configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Enable memory profiling
    pub profiling_enabled: bool,
    /// Maximum total memory usage
    pub max_total_memory: u64,
    /// GC trigger threshold (percentage)
    pub gc_threshold: f64,
    /// GC run interval
    pub gc_interval: Duration,
    /// Enable stack trace collection
    pub stack_traces: bool,
    /// Memory warning threshold
    pub warning_threshold: f64,
    /// Emergency cleanup threshold
    pub emergency_threshold: f64,
    /// Pool configurations
    pub pool_configs: HashMap<String, PoolConfig>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        let mut pool_configs = HashMap::new();
        
        // Default pool for workflows
        pool_configs.insert("workflows".to_string(), PoolConfig {
            max_size: 100 * 1024 * 1024, // 100MB
            prealloc_size: 10 * 1024 * 1024, // 10MB
            growth_strategy: GrowthStrategy::Exponential(1.5),
            cleanup_threshold: 0.8,
            detailed_tracking: true,
        });
        
        // Default pool for events
        pool_configs.insert("events".to_string(), PoolConfig {
            max_size: 50 * 1024 * 1024, // 50MB
            prealloc_size: 5 * 1024 * 1024, // 5MB
            growth_strategy: GrowthStrategy::Linear(10 * 1024 * 1024),
            cleanup_threshold: 0.7,
            detailed_tracking: false,
        });
        
        // Default pool for context data
        pool_configs.insert("context".to_string(), PoolConfig {
            max_size: 75 * 1024 * 1024, // 75MB
            prealloc_size: 15 * 1024 * 1024, // 15MB
            growth_strategy: GrowthStrategy::Exponential(1.3),
            cleanup_threshold: 0.75,
            detailed_tracking: true,
        });

        Self {
            profiling_enabled: true,
            max_total_memory: 1024 * 1024 * 1024, // 1GB
            gc_threshold: 0.8, // 80%
            gc_interval: Duration::from_secs(60),
            stack_traces: false, // Expensive, enable only for debugging
            warning_threshold: 0.75, // 75%
            emergency_threshold: 0.95, // 95%
            pool_configs,
        }
    }
}

/// Garbage collection scheduler
pub struct GcScheduler {
    /// Last GC run time
    last_run: Option<Instant>,
    /// Scheduled GC operations
    scheduled_operations: VecDeque<GcOperation>,
    /// GC configuration
    config: GcConfig,
}

/// Garbage collection operation
#[derive(Debug, Clone)]
pub struct GcOperation {
    /// Operation type
    pub operation_type: GcOperationType,
    /// Target pools
    pub target_pools: Vec<String>,
    /// Scheduled time
    pub scheduled_time: Instant,
    /// Priority
    pub priority: GcPriority,
}

/// GC operation types
#[derive(Debug, Clone)]
pub enum GcOperationType {
    /// Full cleanup
    FullCleanup,
    /// Incremental cleanup
    IncrementalCleanup,
    /// Compaction
    Compaction,
    /// Emergency cleanup
    EmergencyCleanup,
}

/// GC priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GcPriority {
    Low,
    Normal,
    High,
    Emergency,
}

/// GC configuration
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// Enable automatic GC
    pub auto_gc: bool,
    /// GC trigger thresholds
    pub thresholds: GcThresholds,
    /// Maximum GC pause time
    pub max_pause_time: Duration,
    /// Incremental GC chunk size
    pub incremental_chunk_size: usize,
}

/// GC thresholds
#[derive(Debug, Clone)]
pub struct GcThresholds {
    /// Memory usage threshold for GC trigger
    pub memory_threshold: f64,
    /// Fragmentation threshold for compaction
    pub fragmentation_threshold: f64,
    /// Idle time threshold for opportunistic GC
    pub idle_threshold: Duration,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            auto_gc: true,
            thresholds: GcThresholds {
                memory_threshold: 0.8,
                fragmentation_threshold: 0.3,
                idle_threshold: Duration::from_secs(30),
            },
            max_pause_time: Duration::from_millis(50),
            incremental_chunk_size: 1000,
        }
    }
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: MemoryConfig) -> Self {
        let stats = Arc::new(MemoryStats {
            total_used: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            active_allocations: AtomicUsize::new(0),
            total_allocations: AtomicU64::new(0),
            fragmentation_ratio: AtomicU64::new(0),
            last_gc_time: Arc::new(RwLock::new(None)),
            gc_stats: Arc::new(RwLock::new(GarbageCollectionStats::default())),
        });

        let gc_scheduler = Arc::new(RwLock::new(GcScheduler {
            last_run: None,
            scheduled_operations: VecDeque::new(),
            config: GcConfig::default(),
        }));

        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            stats,
            config,
            gc_scheduler,
        }
    }

    /// Initialize memory pools
    pub async fn initialize(&self) {
        let mut pools = self.pools.write().await;
        
        for (pool_name, pool_config) in &self.config.pool_configs {
            let pool = MemoryPool {
                name: pool_name.clone(),
                total_allocated: AtomicU64::new(0),
                used_memory: AtomicU64::new(0),
                peak_memory: AtomicU64::new(0),
                allocation_count: AtomicU64::new(0),
                deallocation_count: AtomicU64::new(0),
                config: pool_config.clone(),
                allocation_history: Arc::new(RwLock::new(VecDeque::new())),
            };
            
            pools.insert(pool_name.clone(), pool);
        }
    }

    /// Allocate memory in a specific pool
    pub async fn allocate(&self, pool_name: &str, size: u64, object_type: &str) -> Option<String> {
        if !self.config.profiling_enabled {
            return Some(uuid::Uuid::new_v4().to_string());
        }

        // Check if allocation would exceed pool limit first
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(pool_name) {
                let current_used = pool.used_memory.load(Ordering::Relaxed);
                if current_used + size > pool.config.max_size {
                    // Try to trigger GC first
                    drop(pools);
                    self.trigger_gc(pool_name, GcPriority::High).await;
                    
                    // Check again after GC
                    let pools_check = self.pools.read().await;
                    if let Some(pool_check) = pools_check.get(pool_name) {
                        let current_used_after_gc = pool_check.used_memory.load(Ordering::Relaxed);
                        if current_used_after_gc + size > pool_check.config.max_size {
                            return None; // Still can't allocate
                        }
                    }
                }
            }
        }

        // Now perform the allocation with write lock
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(pool_name)?;
        
        // Perform the allocation
        let allocation_id = uuid::Uuid::new_v4().to_string();
        
        pool.used_memory.fetch_add(size, Ordering::Relaxed);
        pool.total_allocated.fetch_add(size, Ordering::Relaxed);
        pool.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        // Update peak memory
        let current_used = pool.used_memory.load(Ordering::Relaxed);
        let mut peak = pool.peak_memory.load(Ordering::Relaxed);
        while current_used > peak {
            match pool.peak_memory.compare_exchange_weak(
                peak,
                current_used,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }

        // Record allocation if detailed tracking is enabled
        if pool.config.detailed_tracking {
            let record = AllocationRecord {
                id: allocation_id.clone(),
                size,
                timestamp: Instant::now(),
                object_type: object_type.to_string(),
                stack_trace: if self.config.stack_traces {
                    Some(self.capture_stack_trace())
                } else {
                    None
                },
            };

            let mut history = pool.allocation_history.write().await;
            history.push_back(record);
            
            // Maintain history size (keep last 1000 allocations)
            while history.len() > 1000 {
                history.pop_front();
            }
        }

        // Update global stats
        self.stats.total_used.fetch_add(size, Ordering::Relaxed);
        self.stats.active_allocations.fetch_add(1, Ordering::Relaxed);
        self.stats.total_allocations.fetch_add(1, Ordering::Relaxed);

        // Update peak usage
        let global_used = self.stats.total_used.load(Ordering::Relaxed);
        let mut global_peak = self.stats.peak_usage.load(Ordering::Relaxed);
        while global_used > global_peak {
            match self.stats.peak_usage.compare_exchange_weak(
                global_peak,
                global_used,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(new_peak) => global_peak = new_peak,
            }
        }

        // Check if we need to trigger GC
        self.check_gc_triggers().await;

        Some(allocation_id)
    }

    /// Deallocate memory
    pub async fn deallocate(&self, pool_name: &str, allocation_id: &str, size: u64) -> bool {
        if !self.config.profiling_enabled {
            return true;
        }

        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.get_mut(pool_name) {
            pool.used_memory.fetch_sub(size, Ordering::Relaxed);
            pool.deallocation_count.fetch_add(1, Ordering::Relaxed);

            // Update global stats
            self.stats.total_used.fetch_sub(size, Ordering::Relaxed);
            self.stats.active_allocations.fetch_sub(1, Ordering::Relaxed);

            // Remove from allocation history if detailed tracking is enabled
            if pool.config.detailed_tracking {
                let mut history = pool.allocation_history.write().await;
                history.retain(|record| record.id != allocation_id);
            }

            true
        } else {
            false
        }
    }

    /// Get memory statistics
    pub async fn get_statistics(&self) -> MemoryStatistics {
        let pools = self.pools.read().await;
        let mut pool_stats = HashMap::new();

        for (name, pool) in pools.iter() {
            pool_stats.insert(name.clone(), PoolStatistics {
                name: pool.name.clone(),
                total_allocated: pool.total_allocated.load(Ordering::Relaxed),
                used_memory: pool.used_memory.load(Ordering::Relaxed),
                peak_memory: pool.peak_memory.load(Ordering::Relaxed),
                allocation_count: pool.allocation_count.load(Ordering::Relaxed),
                deallocation_count: pool.deallocation_count.load(Ordering::Relaxed),
                utilization: if pool.config.max_size > 0 {
                    pool.used_memory.load(Ordering::Relaxed) as f64 / pool.config.max_size as f64
                } else {
                    0.0
                },
            });
        }

        let gc_stats = self.stats.gc_stats.read().await.clone();

        MemoryStatistics {
            total_used: self.stats.total_used.load(Ordering::Relaxed),
            peak_usage: self.stats.peak_usage.load(Ordering::Relaxed),
            active_allocations: self.stats.active_allocations.load(Ordering::Relaxed),
            total_allocations: self.stats.total_allocations.load(Ordering::Relaxed),
            fragmentation_ratio: self.stats.fragmentation_ratio.load(Ordering::Relaxed) as f64 / 1000.0,
            pool_statistics: pool_stats,
            gc_statistics: gc_stats,
            max_total_memory: self.config.max_total_memory,
            memory_pressure: self.calculate_memory_pressure().await,
        }
    }

    /// Calculate current memory pressure
    async fn calculate_memory_pressure(&self) -> f64 {
        let total_used = self.stats.total_used.load(Ordering::Relaxed) as f64;
        let max_memory = self.config.max_total_memory as f64;
        
        if max_memory > 0.0 {
            total_used / max_memory
        } else {
            0.0
        }
    }

    /// Check GC triggers and schedule GC if needed
    async fn check_gc_triggers(&self) {
        let memory_pressure = self.calculate_memory_pressure().await;
        
        if memory_pressure > self.config.emergency_threshold {
            self.trigger_gc("all", GcPriority::Emergency).await;
        } else if memory_pressure > self.config.gc_threshold {
            self.trigger_gc("all", GcPriority::High).await;
        } else if memory_pressure > self.config.warning_threshold {
            self.trigger_gc("all", GcPriority::Normal).await;
        }
    }

    /// Trigger garbage collection
    pub async fn trigger_gc(&self, pool_name: &str, priority: GcPriority) {
        let mut scheduler = self.gc_scheduler.write().await;
        
        let operation = GcOperation {
            operation_type: match priority {
                GcPriority::Emergency => GcOperationType::EmergencyCleanup,
                GcPriority::High => GcOperationType::FullCleanup,
                _ => GcOperationType::IncrementalCleanup,
            },
            target_pools: if pool_name == "all" {
                self.pools.read().await.keys().cloned().collect()
            } else {
                vec![pool_name.to_string()]
            },
            scheduled_time: Instant::now(),
            priority,
        };

        // Insert operation maintaining priority order
        let mut inserted = false;
        for (i, existing_op) in scheduler.scheduled_operations.iter().enumerate() {
            if operation.priority > existing_op.priority {
                scheduler.scheduled_operations.insert(i, operation.clone());
                inserted = true;
                break;
            }
        }
        
        if !inserted {
            scheduler.scheduled_operations.push_back(operation);
        }
    }

    /// Run scheduled garbage collection operations
    pub async fn run_gc(&self) -> GarbageCollectionResult {
        let mut scheduler = self.gc_scheduler.write().await;
        
        if let Some(operation) = scheduler.scheduled_operations.pop_front() {
            drop(scheduler); // Release scheduler lock
            
            let start_time = Instant::now();
            let mut total_freed = 0u64;
            let mut objects_collected = 0u64;

            for pool_name in &operation.target_pools {
                let result = self.run_pool_gc(pool_name, &operation.operation_type).await;
                total_freed += result.memory_freed;
                objects_collected += result.objects_collected;
            }

            let duration = start_time.elapsed();

            // Update GC statistics
            {
                let mut gc_stats = self.stats.gc_stats.write().await;
                gc_stats.cycles += 1;
                gc_stats.total_time += duration;
                gc_stats.last_freed = total_freed;
                gc_stats.total_freed += total_freed;
                gc_stats.last_objects_collected = objects_collected;
                gc_stats.total_objects_collected += objects_collected;
                gc_stats.avg_time = Duration::from_nanos(
                    (gc_stats.total_time.as_nanos() / gc_stats.cycles as u128) as u64
                );
            }

            // Update last GC time
            {
                let mut last_gc = self.stats.last_gc_time.write().await;
                *last_gc = Some(SystemTime::now());
            }

            GarbageCollectionResult {
                success: true,
                duration,
                memory_freed: total_freed,
                objects_collected,
                pools_cleaned: operation.target_pools,
            }
        } else {
            GarbageCollectionResult {
                success: false,
                duration: Duration::from_nanos(0),
                memory_freed: 0,
                objects_collected: 0,
                pools_cleaned: vec![],
            }
        }
    }

    /// Run GC for a specific pool
    async fn run_pool_gc(&self, pool_name: &str, operation_type: &GcOperationType) -> PoolGcResult {
        let mut pools = self.pools.write().await;
        
        if let Some(pool) = pools.get_mut(pool_name) {
            let mut freed_memory = 0u64;
            let mut collected_objects = 0u64;

            match operation_type {
                GcOperationType::IncrementalCleanup => {
                    // Clean up old allocations incrementally
                    if pool.config.detailed_tracking {
                        let mut history = pool.allocation_history.write().await;
                        let cutoff_time = Instant::now() - Duration::from_secs(300); // 5 minutes
                        
                        let mut to_remove = Vec::new();
                        for (i, record) in history.iter().enumerate() {
                            if record.timestamp < cutoff_time {
                                to_remove.push(i);
                                freed_memory += record.size;
                                collected_objects += 1;
                            }
                            
                            if to_remove.len() >= 100 { // Incremental limit
                                break;
                            }
                        }
                        
                        // Remove in reverse order to maintain indices
                        for &index in to_remove.iter().rev() {
                            history.remove(index);
                        }
                    }
                },
                GcOperationType::FullCleanup => {
                    // More aggressive cleanup
                    if pool.config.detailed_tracking {
                        let mut history = pool.allocation_history.write().await;
                        let cutoff_time = Instant::now() - Duration::from_secs(60); // 1 minute
                        
                        let initial_len = history.len();
                        history.retain(|record| record.timestamp >= cutoff_time);
                        
                        collected_objects = (initial_len - history.len()) as u64;
                        // Estimate freed memory (simplified)
                        freed_memory = collected_objects * 1024; // Average 1KB per object
                    }
                },
                GcOperationType::EmergencyCleanup => {
                    // Emergency cleanup - clear most allocations
                    if pool.config.detailed_tracking {
                        let mut history = pool.allocation_history.write().await;
                        collected_objects = history.len() as u64;
                        freed_memory = history.iter().map(|r| r.size).sum();
                        history.clear();
                    }
                    
                    // Reset counters for emergency cleanup
                    let used = pool.used_memory.load(Ordering::Relaxed);
                    pool.used_memory.store(used / 2, Ordering::Relaxed); // Emergency reduction
                },
                _ => {}
            }

            // Update pool statistics
            pool.used_memory.fetch_sub(freed_memory, Ordering::Relaxed);
            
            // Update global statistics
            self.stats.total_used.fetch_sub(freed_memory, Ordering::Relaxed);
            self.stats.active_allocations.fetch_sub(collected_objects as usize, Ordering::Relaxed);

            PoolGcResult {
                pool_name: pool_name.to_string(),
                memory_freed: freed_memory,
                objects_collected: collected_objects,
            }
        } else {
            PoolGcResult {
                pool_name: pool_name.to_string(),
                memory_freed: 0,
                objects_collected: 0,
            }
        }
    }

    /// Capture stack trace (placeholder implementation)
    fn capture_stack_trace(&self) -> Vec<String> {
        // In a real implementation, this would capture the actual stack trace
        // For now, return a placeholder
        vec![
            "MemoryManager::allocate".to_string(),
            "WorkflowEngine::create_workflow".to_string(),
            "main".to_string(),
        ]
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub total_used: u64,
    pub peak_usage: u64,
    pub active_allocations: usize,
    pub total_allocations: u64,
    pub fragmentation_ratio: f64,
    pub pool_statistics: HashMap<String, PoolStatistics>,
    pub gc_statistics: GarbageCollectionStats,
    pub max_total_memory: u64,
    pub memory_pressure: f64,
}

/// Pool-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub name: String,
    pub total_allocated: u64,
    pub used_memory: u64,
    pub peak_memory: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub utilization: f64,
}

/// Garbage collection result
#[derive(Debug, Clone)]
pub struct GarbageCollectionResult {
    pub success: bool,
    pub duration: Duration,
    pub memory_freed: u64,
    pub objects_collected: u64,
    pub pools_cleaned: Vec<String>,
}

/// Pool GC result
#[derive(Debug, Clone)]
struct PoolGcResult {
    pool_name: String,
    memory_freed: u64,
    objects_collected: u64,
}

impl GcScheduler {
    /// Create a new GC scheduler
    pub fn new(config: GcConfig) -> Self {
        Self {
            last_run: None,
            scheduled_operations: VecDeque::new(),
            config,
        }
    }
    
    /// Check if GC should run
    pub fn should_run_gc(&mut self) -> bool {
        let now = Instant::now();
        
        // Check if enough time has passed since last run  
        if let Some(last_run) = self.last_run {
            let elapsed = now.duration_since(last_run);
            // Use max_pause_time as a minimum interval between GC runs
            if elapsed < self.config.max_pause_time {
                return false;
            }
        }
        
        // Update last run time
        self.last_run = Some(now);
        true
    }
    
    /// Schedule a GC operation
    pub fn schedule_operation(&mut self, operation: GcOperation) {
        self.scheduled_operations.push_back(operation);
    }
    
    /// Get next scheduled operation
    pub fn next_operation(&mut self) -> Option<GcOperation> {
        self.scheduled_operations.pop_front()
    }
}

impl PoolGcResult {
    /// Check if this result should be used (using pool_name)
    pub fn is_significant(&self) -> bool {
        !self.pool_name.is_empty() && (self.memory_freed > 0 || self.objects_collected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_manager() {
        let config = MemoryConfig::default();
        let manager = MemoryManager::new(config);
        
        manager.initialize().await;
        
        // Test allocation
        let allocation_id = manager.allocate("workflows", 1024, "Workflow").await;
        assert!(allocation_id.is_some());
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_used, 1024);
        assert_eq!(stats.active_allocations, 1);
        
        // Test deallocation
        let success = manager.deallocate("workflows", &allocation_id.unwrap(), 1024).await;
        assert!(success);
        
        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_used, 0);
        assert_eq!(stats.active_allocations, 0);
    }

    #[tokio::test]
    async fn test_gc_operations() {
        let config = MemoryConfig::default();
        let manager = MemoryManager::new(config);
        
        manager.initialize().await;
        
        // Allocate some memory
        for i in 0..10 {
            let _ = manager.allocate("workflows", 1024, &format!("Workflow{}", i)).await;
        }
        
        // Trigger GC with emergency priority to ensure objects are collected
        manager.trigger_gc("workflows", GcPriority::Emergency).await;
        let gc_result = manager.run_gc().await;
        
        assert!(gc_result.success);
        assert!(gc_result.objects_collected > 0 || gc_result.memory_freed > 0);
    }
}