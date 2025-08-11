//! Performance optimization utilities and profiling tools
//!
//! This module provides performance monitoring, optimization utilities,
//! and profiling capabilities for the workflow engine.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

pub mod profiler;
pub mod metrics;
pub mod optimizer;
pub mod memory;

/// Performance metrics for workflow operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Operation name
    pub operation: String,
    /// Execution duration
    pub duration: Duration,
    /// Memory usage delta
    pub memory_delta: i64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Timestamp when metric was recorded
    pub timestamp: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performance profile for a workflow operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProfile {
    /// Operation name
    pub operation: String,
    /// Total executions
    pub execution_count: u64,
    /// Average execution time
    pub avg_duration: Duration,
    /// Minimum execution time
    pub min_duration: Duration,
    /// Maximum execution time
    pub max_duration: Duration,
    /// 95th percentile execution time
    pub p95_duration: Duration,
    /// 99th percentile execution time
    pub p99_duration: Duration,
    /// Total memory allocated
    pub total_memory: u64,
    /// Average memory per execution
    pub avg_memory: u64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Throughput (operations per second)
    pub throughput: f64,
}

/// System performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    /// CPU utilization percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Memory usage percentage
    pub memory_percentage: f64,
    /// Active workflow count
    pub active_workflows: u32,
    /// Active step count
    pub active_steps: u32,
    /// Event queue size
    pub event_queue_size: u32,
    /// Events processed per second
    pub events_per_second: f64,
    /// System uptime
    pub uptime: Duration,
    /// Garbage collection statistics
    pub gc_stats: GcStats,
}

/// Garbage collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcStats {
    /// Number of GC cycles
    pub collections: u64,
    /// Total time spent in GC
    pub total_gc_time: Duration,
    /// Average GC pause time
    pub avg_pause_time: Duration,
    /// Memory freed by GC
    pub memory_freed: u64,
}

/// Performance monitor that tracks system and operation metrics
pub struct PerformanceMonitor {
    /// Operation profiles
    profiles: Arc<RwLock<HashMap<String, OperationProfile>>>,
    /// Recent metrics
    recent_metrics: Arc<RwLock<Vec<PerformanceMetrics>>>,
    /// System performance history
    system_history: Arc<RwLock<Vec<SystemPerformance>>>,
    /// Performance thresholds
    thresholds: PerformanceThresholds,
    /// Start time for uptime calculation
    start_time: Instant,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum acceptable execution time
    pub max_execution_time: Duration,
    /// Maximum memory usage per operation
    pub max_memory_per_operation: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_usage: f64,
    /// Maximum memory usage percentage
    pub max_memory_percentage: f64,
    /// Maximum error rate
    pub max_error_rate: f64,
    /// Minimum required throughput
    pub min_throughput: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_memory_per_operation: 100 * 1024 * 1024, // 100MB
            max_cpu_usage: 80.0,
            max_memory_percentage: 85.0,
            max_error_rate: 0.05, // 5%
            min_throughput: 100.0, // 100 ops/sec
        }
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            recent_metrics: Arc::new(RwLock::new(Vec::new())),
            system_history: Arc::new(RwLock::new(Vec::new())),
            thresholds,
            start_time: Instant::now(),
        }
    }

    /// Record a performance metric
    pub async fn record_metric(&self, metric: PerformanceMetrics) {
        // Update operation profile
        {
            let mut profiles = self.profiles.write().await;
            let profile = profiles.entry(metric.operation.clone())
                .or_insert_with(|| OperationProfile {
                    operation: metric.operation.clone(),
                    execution_count: 0,
                    avg_duration: Duration::from_nanos(0),
                    min_duration: Duration::from_secs(u64::MAX),
                    max_duration: Duration::from_nanos(0),
                    p95_duration: Duration::from_nanos(0),
                    p99_duration: Duration::from_nanos(0),
                    total_memory: 0,
                    avg_memory: 0,
                    error_rate: 0.0,
                    throughput: 0.0,
                });

            profile.execution_count += 1;
            profile.min_duration = profile.min_duration.min(metric.duration);
            profile.max_duration = profile.max_duration.max(metric.duration);
            
            // Update average duration
            let total_nanos = profile.avg_duration.as_nanos() * (profile.execution_count - 1) as u128
                + metric.duration.as_nanos();
            profile.avg_duration = Duration::from_nanos((total_nanos / profile.execution_count as u128) as u64);

            if metric.memory_delta > 0 {
                profile.total_memory += metric.memory_delta as u64;
                profile.avg_memory = profile.total_memory / profile.execution_count;
            }
        }

        // Store recent metric
        {
            let mut recent = self.recent_metrics.write().await;
            recent.push(metric);
            
            // Keep only recent metrics (last 1000)
            if recent.len() > 1000 {
                let drain_count = recent.len() - 1000;
                recent.drain(0..drain_count);
            }
        }
    }

    /// Get performance profile for an operation
    pub async fn get_profile(&self, operation: &str) -> Option<OperationProfile> {
        let profiles = self.profiles.read().await;
        profiles.get(operation).cloned()
    }

    /// Get all operation profiles
    pub async fn get_all_profiles(&self) -> HashMap<String, OperationProfile> {
        self.profiles.read().await.clone()
    }

    /// Get recent performance metrics
    pub async fn get_recent_metrics(&self, limit: Option<usize>) -> Vec<PerformanceMetrics> {
        let metrics = self.recent_metrics.read().await;
        let start = if let Some(limit) = limit {
            metrics.len().saturating_sub(limit)
        } else {
            0
        };
        metrics[start..].to_vec()
    }

    /// Record system performance snapshot
    pub async fn record_system_performance(&self, system_perf: SystemPerformance) {
        let mut history = self.system_history.write().await;
        history.push(system_perf);
        
        // Keep only recent system performance data (last 100)
        if history.len() > 100 {
            let drain_count = history.len() - 100;
            history.drain(0..drain_count);
        }
    }

    /// Get current system performance
    pub async fn get_system_performance(&self) -> Option<SystemPerformance> {
        let history = self.system_history.read().await;
        history.last().cloned()
    }

    /// Check if performance is within thresholds
    pub async fn check_thresholds(&self) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        
        // Check operation profiles
        let profiles = self.profiles.read().await;
        for profile in profiles.values() {
            if profile.avg_duration > self.thresholds.max_execution_time {
                alerts.push(PerformanceAlert {
                    alert_type: AlertType::SlowOperation,
                    operation: Some(profile.operation.clone()),
                    message: format!("Operation '{}' average duration ({:?}) exceeds threshold ({:?})",
                        profile.operation, profile.avg_duration, self.thresholds.max_execution_time),
                    value: profile.avg_duration.as_secs_f64(),
                    threshold: self.thresholds.max_execution_time.as_secs_f64(),
                    severity: AlertSeverity::Warning,
                });
            }

            if profile.error_rate > self.thresholds.max_error_rate {
                alerts.push(PerformanceAlert {
                    alert_type: AlertType::HighErrorRate,
                    operation: Some(profile.operation.clone()),
                    message: format!("Operation '{}' error rate ({:.2}%) exceeds threshold ({:.2}%)",
                        profile.operation, profile.error_rate * 100.0, self.thresholds.max_error_rate * 100.0),
                    value: profile.error_rate,
                    threshold: self.thresholds.max_error_rate,
                    severity: AlertSeverity::Critical,
                });
            }

            if profile.throughput < self.thresholds.min_throughput {
                alerts.push(PerformanceAlert {
                    alert_type: AlertType::LowThroughput,
                    operation: Some(profile.operation.clone()),
                    message: format!("Operation '{}' throughput ({:.2} ops/sec) below threshold ({:.2} ops/sec)",
                        profile.operation, profile.throughput, self.thresholds.min_throughput),
                    value: profile.throughput,
                    threshold: self.thresholds.min_throughput,
                    severity: AlertSeverity::Warning,
                });
            }
        }

        // Check system performance
        if let Some(system_perf) = self.get_system_performance().await {
            if system_perf.cpu_usage > self.thresholds.max_cpu_usage {
                alerts.push(PerformanceAlert {
                    alert_type: AlertType::HighCpuUsage,
                    operation: None,
                    message: format!("CPU usage ({:.2}%) exceeds threshold ({:.2}%)",
                        system_perf.cpu_usage, self.thresholds.max_cpu_usage),
                    value: system_perf.cpu_usage,
                    threshold: self.thresholds.max_cpu_usage,
                    severity: AlertSeverity::Critical,
                });
            }

            if system_perf.memory_percentage > self.thresholds.max_memory_percentage {
                alerts.push(PerformanceAlert {
                    alert_type: AlertType::HighMemoryUsage,
                    operation: None,
                    message: format!("Memory usage ({:.2}%) exceeds threshold ({:.2}%)",
                        system_perf.memory_percentage, self.thresholds.max_memory_percentage),
                    value: system_perf.memory_percentage,
                    threshold: self.thresholds.max_memory_percentage,
                    severity: AlertSeverity::Critical,
                });
            }
        }

        alerts
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let profiles = self.get_all_profiles().await;
        let system_perf = self.get_system_performance().await;
        let alerts = self.check_thresholds().await;
        let uptime = self.start_time.elapsed();

        let summary = self.generate_summary(&profiles, &system_perf).await;

        PerformanceReport {
            timestamp: SystemTime::now(),
            uptime,
            operation_profiles: profiles,
            system_performance: system_perf,
            alerts,
            summary,
        }
    }

    /// Generate performance summary
    async fn generate_summary(
        &self,
        profiles: &HashMap<String, OperationProfile>,
        system_perf: &Option<SystemPerformance>,
    ) -> PerformanceSummary {
        let total_operations = profiles.values().map(|p| p.execution_count).sum();
        let avg_duration = if !profiles.is_empty() {
            let total_nanos: u128 = profiles.values()
                .map(|p| p.avg_duration.as_nanos() * p.execution_count as u128)
                .sum();
            Duration::from_nanos((total_nanos / total_operations as u128) as u64)
        } else {
            Duration::from_nanos(0)
        };

        let slowest_operation = profiles.values()
            .max_by_key(|p| p.avg_duration)
            .map(|p| p.operation.clone());

        let highest_throughput = profiles.values()
            .max_by(|a, b| a.throughput.partial_cmp(&b.throughput).unwrap_or(std::cmp::Ordering::Equal))
            .map(|p| p.operation.clone());

        PerformanceSummary {
            total_operations,
            avg_execution_time: avg_duration,
            slowest_operation,
            highest_throughput_operation: highest_throughput,
            total_memory_usage: system_perf.as_ref().map(|s| s.memory_usage).unwrap_or(0),
            cpu_usage: system_perf.as_ref().map(|s| s.cpu_usage).unwrap_or(0.0),
            active_workflows: system_perf.as_ref().map(|s| s.active_workflows).unwrap_or(0),
            events_per_second: system_perf.as_ref().map(|s| s.events_per_second).unwrap_or(0.0),
        }
    }
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub operation: Option<String>,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
}

/// Alert type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    SlowOperation,
    HighErrorRate,
    LowThroughput,
    HighCpuUsage,
    HighMemoryUsage,
    SystemUnresponsive,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: SystemTime,
    pub uptime: Duration,
    pub operation_profiles: HashMap<String, OperationProfile>,
    pub system_performance: Option<SystemPerformance>,
    pub alerts: Vec<PerformanceAlert>,
    pub summary: PerformanceSummary,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_operations: u64,
    pub avg_execution_time: Duration,
    pub slowest_operation: Option<String>,
    pub highest_throughput_operation: Option<String>,
    pub total_memory_usage: u64,
    pub cpu_usage: f64,
    pub active_workflows: u32,
    pub events_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(PerformanceThresholds::default());

        let metric = PerformanceMetrics {
            operation: "test_operation".to_string(),
            duration: Duration::from_millis(100),
            memory_delta: 1024,
            cpu_usage: 5.0,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        monitor.record_metric(metric).await;

        let profile = monitor.get_profile("test_operation").await;
        assert!(profile.is_some());

        let profile = profile.unwrap();
        assert_eq!(profile.operation, "test_operation");
        assert_eq!(profile.execution_count, 1);
        assert_eq!(profile.avg_duration, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_performance_thresholds() {
        let thresholds = PerformanceThresholds {
            max_execution_time: Duration::from_millis(50),
            max_error_rate: 0.01,
            ..Default::default()
        };

        let monitor = PerformanceMonitor::new(thresholds);

        // Record a slow operation
        let metric = PerformanceMetrics {
            operation: "slow_operation".to_string(),
            duration: Duration::from_millis(100),
            memory_delta: 1024,
            cpu_usage: 5.0,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        monitor.record_metric(metric).await;

        let alerts = monitor.check_thresholds().await;
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| matches!(a.alert_type, AlertType::SlowOperation)));
    }
}