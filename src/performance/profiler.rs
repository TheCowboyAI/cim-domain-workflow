//! Performance profiler for detailed operation analysis

use super::{PerformanceMetrics, PerformanceMonitor};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

/// Profiler for tracking detailed performance metrics
pub struct Profiler {
    /// Performance monitor
    monitor: Arc<PerformanceMonitor>,
    /// Active profiling sessions
    sessions: Arc<RwLock<HashMap<String, ProfilingSession>>>,
    /// Profiler configuration
    config: ProfilerConfig,
}

/// Profiling session for tracking a specific operation
#[derive(Debug, Clone)]
pub struct ProfilingSession {
    /// Session ID
    pub session_id: String,
    /// Operation being profiled
    pub operation: String,
    /// Start time
    pub start_time: Instant,
    /// Start memory usage
    pub start_memory: u64,
    /// Checkpoints within the operation
    pub checkpoints: Vec<ProfilingCheckpoint>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Checkpoint within a profiling session
#[derive(Debug, Clone)]
pub struct ProfilingCheckpoint {
    /// Checkpoint name
    pub name: String,
    /// Time since session start
    pub elapsed: Duration,
    /// Memory usage at checkpoint
    pub memory_usage: u64,
    /// Custom data
    pub data: HashMap<String, serde_json::Value>,
}

/// Profiler configuration
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Whether profiling is enabled
    pub enabled: bool,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Session timeout
    pub session_timeout: Duration,
    /// Whether to collect memory statistics
    pub collect_memory: bool,
    /// Whether to collect CPU statistics
    pub collect_cpu: bool,
    /// Sampling interval for continuous profiling
    pub sampling_interval: Duration,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_sessions: 1000,
            session_timeout: Duration::from_secs(300), // 5 minutes
            collect_memory: true,
            collect_cpu: true,
            sampling_interval: Duration::from_millis(100),
        }
    }
}

impl Profiler {
    /// Create a new profiler
    pub fn new(monitor: Arc<PerformanceMonitor>, config: ProfilerConfig) -> Self {
        Self {
            monitor,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Start profiling an operation
    pub async fn start_profiling(&self, operation: &str) -> ProfilerHandle {
        if !self.config.enabled {
            return ProfilerHandle::disabled();
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = ProfilingSession {
            session_id: session_id.clone(),
            operation: operation.to_string(),
            start_time: Instant::now(),
            start_memory: self.get_memory_usage(),
            checkpoints: Vec::new(),
            metadata: HashMap::new(),
        };

        // Clean up old sessions
        self.cleanup_expired_sessions().await;

        // Check if we're at the session limit
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.config.max_sessions {
                return ProfilerHandle::disabled();
            }
        }

        // Store the session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        ProfilerHandle {
            profiler: Arc::new(self.clone()),
            session_id,
            enabled: true,
        }
    }

    /// Add a checkpoint to a profiling session
    pub async fn add_checkpoint(
        &self,
        session_id: &str,
        checkpoint_name: &str,
        data: HashMap<String, serde_json::Value>,
    ) {
        if !self.config.enabled {
            return;
        }

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            let checkpoint = ProfilingCheckpoint {
                name: checkpoint_name.to_string(),
                elapsed: session.start_time.elapsed(),
                memory_usage: self.get_memory_usage(),
                data,
            };
            session.checkpoints.push(checkpoint);
        }
    }

    /// Finish profiling and record metrics
    pub async fn finish_profiling(&self, session_id: &str) -> Option<ProfilingResult> {
        if !self.config.enabled {
            return None;
        }

        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
        }?;

        let total_duration = session.start_time.elapsed();
        let memory_delta = self.get_memory_usage() as i64 - session.start_memory as i64;

        // Create performance metric
        let metric = PerformanceMetrics {
            operation: session.operation.clone(),
            duration: total_duration,
            memory_delta,
            cpu_usage: self.get_cpu_usage(),
            timestamp: SystemTime::now(),
            metadata: session.metadata.clone(),
        };

        // Record the metric
        self.monitor.record_metric(metric).await;

        Some(ProfilingResult {
            session_id: session.session_id,
            operation: session.operation,
            total_duration,
            memory_delta,
            checkpoints: session.checkpoints,
            metadata: session.metadata,
        })
    }

    /// Clean up expired sessions
    async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Instant::now();
        
        sessions.retain(|_, session| {
            now.duration_since(session.start_time) < self.config.session_timeout
        });
    }

    /// Get current memory usage (placeholder implementation)
    fn get_memory_usage(&self) -> u64 {
        // In a real implementation, this would get actual memory usage
        // For now, we'll use a placeholder
        0
    }

    /// Get current CPU usage (placeholder implementation)
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, this would get actual CPU usage
        // For now, we'll use a placeholder
        0.0
    }

    /// Get profiling statistics
    pub async fn get_statistics(&self) -> ProfilingStatistics {
        let sessions = self.sessions.read().await;
        ProfilingStatistics {
            active_sessions: sessions.len(),
            max_sessions: self.config.max_sessions,
            enabled: self.config.enabled,
        }
    }
}

impl Clone for Profiler {
    fn clone(&self) -> Self {
        Self {
            monitor: self.monitor.clone(),
            sessions: self.sessions.clone(),
            config: self.config.clone(),
        }
    }
}

/// Handle for a profiling session
pub struct ProfilerHandle {
    profiler: Arc<Profiler>,
    session_id: String,
    enabled: bool,
}

impl ProfilerHandle {
    /// Create a disabled profiler handle
    fn disabled() -> Self {
        // Create a minimal dummy profiler for disabled state
        use crate::performance::PerformanceThresholds;
        let thresholds = PerformanceThresholds {
            max_execution_time: Duration::from_secs(60),
            max_memory_per_operation: 1024 * 1024 * 1024, // 1GB
            max_cpu_usage: 80.0,
            max_memory_percentage: 80.0,
            max_error_rate: 0.01, // 1%
            min_throughput: 1.0,
        };
        
        let dummy_monitor = Arc::new(crate::performance::PerformanceMonitor::new(thresholds));
        let dummy_profiler = Profiler {
            monitor: dummy_monitor,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config: ProfilerConfig {
                enabled: false,
                max_sessions: 0,
                session_timeout: Duration::from_secs(0),
                collect_memory: false,
                collect_cpu: false,
                sampling_interval: Duration::from_secs(1),
            },
        };
        
        Self {
            profiler: Arc::new(dummy_profiler),
            session_id: String::new(),
            enabled: false,
        }
    }

    /// Add a checkpoint to the profiling session
    pub async fn checkpoint(&self, name: &str) {
        self.checkpoint_with_data(name, HashMap::new()).await;
    }

    /// Add a checkpoint with custom data
    pub async fn checkpoint_with_data(&self, name: &str, data: HashMap<String, serde_json::Value>) {
        if !self.enabled {
            return;
        }

        self.profiler.add_checkpoint(&self.session_id, name, data).await;
    }

    /// Finish profiling and get results
    pub async fn finish(self) -> Option<ProfilingResult> {
        if !self.enabled {
            return None;
        }

        self.profiler.finish_profiling(&self.session_id).await
    }
}

/// Result of a profiling session
#[derive(Debug, Clone)]
pub struct ProfilingResult {
    pub session_id: String,
    pub operation: String,
    pub total_duration: Duration,
    pub memory_delta: i64,
    pub checkpoints: Vec<ProfilingCheckpoint>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Profiling statistics
#[derive(Debug, Clone)]
pub struct ProfilingStatistics {
    pub active_sessions: usize,
    pub max_sessions: usize,
    pub enabled: bool,
}

/// Macro for easy profiling
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $operation:expr, $body:expr) => {{
        let handle = $profiler.start_profiling($operation).await;
        let result = $body;
        handle.finish().await;
        result
    }};
}

/// Macro for profiling with checkpoints
#[macro_export]
macro_rules! profile_with_checkpoints {
    ($profiler:expr, $operation:expr, $body:expr) => {{
        let handle = $profiler.start_profiling($operation).await;
        let checkpoint = |name: &str| async {
            handle.checkpoint(name).await;
        };
        let result = $body;
        handle.finish().await;
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::{PerformanceMonitor, PerformanceThresholds};

    #[tokio::test]
    async fn test_profiler() {
        let monitor = Arc::new(PerformanceMonitor::new(PerformanceThresholds::default()));
        let profiler = Profiler::new(monitor, ProfilerConfig::default());

        let handle = profiler.start_profiling("test_operation").await;
        assert!(handle.enabled);

        handle.checkpoint("step1").await;
        
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        handle.checkpoint("step2").await;
        
        let result = handle.finish().await;
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.operation, "test_operation");
        assert_eq!(result.checkpoints.len(), 2);
        assert_eq!(result.checkpoints[0].name, "step1");
        assert_eq!(result.checkpoints[1].name, "step2");
    }

    #[tokio::test]
    async fn test_disabled_profiler() {
        let monitor = Arc::new(PerformanceMonitor::new(PerformanceThresholds::default()));
        let config = ProfilerConfig {
            enabled: false,
            ..Default::default()
        };
        let profiler = Profiler::new(monitor, config);

        let handle = profiler.start_profiling("test_operation").await;
        assert!(!handle.enabled);

        let result = handle.finish().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let monitor = Arc::new(PerformanceMonitor::new(PerformanceThresholds::default()));
        let config = ProfilerConfig {
            session_timeout: Duration::from_millis(10),
            ..Default::default()
        };
        let profiler = Profiler::new(monitor, config);

        // Start a session
        let _handle = profiler.start_profiling("test_operation").await;
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Start another session, which should trigger cleanup
        let _handle2 = profiler.start_profiling("test_operation2").await;
        
        let stats = profiler.get_statistics().await;
        assert_eq!(stats.active_sessions, 1); // Only the new session should remain
    }
}