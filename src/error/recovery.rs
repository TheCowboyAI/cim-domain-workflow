//! Error Recovery and Self-Healing Mechanisms
//!
//! Implements automatic error recovery strategies, self-healing patterns,
//! and degraded service modes for maintaining system availability.

use crate::error::types::{WorkflowError, WorkflowResult, ErrorCategory, ErrorSeverity, ErrorContext, RecoveryAction};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Recovery strategy for different error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Immediate retry with exponential backoff
    ImmediateRetry {
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
    },
    /// Delayed retry with circuit breaker
    DelayedRetry {
        delay: Duration,
        max_attempts: u32,
        circuit_breaker: Option<String>,
    },
    /// Fallback to alternative implementation
    Fallback {
        fallback_operation: String,
        parameters: HashMap<String, serde_json::Value>,
        timeout: Duration,
    },
    /// Graceful degradation
    Degrade {
        degraded_mode: String,
        performance_level: f32,
        feature_set: Vec<String>,
    },
    /// Component restart
    Restart {
        component: String,
        graceful_shutdown: bool,
        restart_timeout: Duration,
    },
    /// Scale resources up or down
    Scale {
        resource_type: String,
        target_instances: u32,
        scale_timeout: Duration,
    },
    /// Manual intervention required
    ManualIntervention {
        priority: InterventionPriority,
        description: String,
        contact_info: Vec<String>,
    },
}

/// Priority levels for manual intervention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionPriority {
    Low,
    Medium,
    High,
    Critical,
    Emergency,
}

/// Recovery execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    /// Whether recovery was successful
    pub success: bool,
    /// Recovery strategy used
    pub strategy: RecoveryStrategy,
    /// Time taken for recovery
    pub recovery_time: Duration,
    /// Recovery details
    pub details: String,
    /// Additional recovery actions triggered
    pub cascaded_actions: Vec<RecoveryAction>,
}

/// Recovery context with error history and system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryContext {
    /// Original error that triggered recovery
    pub original_error: WorkflowError,
    /// Error history leading to this recovery
    pub error_history: Vec<WorkflowError>,
    /// Current system health metrics
    pub system_metrics: HashMap<String, f64>,
    /// Available recovery resources
    pub available_resources: HashMap<String, u32>,
    /// Recovery attempt count
    pub recovery_attempts: u32,
}

/// Self-healing system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingState {
    /// Active degraded modes
    pub degraded_modes: HashMap<String, DegradedModeInfo>,
    /// Active recovery operations
    pub active_recoveries: HashMap<Uuid, RecoveryOperation>,
    /// System health score (0.0 to 1.0)
    pub health_score: f32,
    /// Last health check timestamp
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Information about degraded service mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradedModeInfo {
    /// Mode identifier
    pub mode_id: String,
    /// Performance level (0.0 to 1.0)
    pub performance_level: f32,
    /// Available features
    pub available_features: Vec<String>,
    /// Disabled features
    pub disabled_features: Vec<String>,
    /// When degradation started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Expected duration
    pub expected_duration: Option<Duration>,
}

/// Active recovery operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryOperation {
    /// Operation ID
    pub operation_id: Uuid,
    /// Recovery strategy being executed
    pub strategy: RecoveryStrategy,
    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Expected completion time
    pub expected_completion: Option<chrono::DateTime<chrono::Utc>>,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Status description
    pub status: String,
}

/// Recovery manager trait
#[async_trait]
pub trait RecoveryManager: Send + Sync {
    /// Execute recovery strategy for given error
    async fn execute_recovery(
        &self,
        error: &WorkflowError,
        context: RecoveryContext,
    ) -> WorkflowResult<RecoveryResult>;

    /// Get recommended recovery strategy for error
    fn recommend_strategy(&self, error: &WorkflowError) -> Option<RecoveryStrategy>;

    /// Check if recovery is needed
    async fn needs_recovery(&self, error: &WorkflowError) -> bool;

    /// Get current system health
    async fn get_system_health(&self) -> SelfHealingState;
}

/// Default recovery manager implementation
pub struct DefaultRecoveryManager {
    /// Recovery strategies by error category
    strategies: HashMap<ErrorCategory, Vec<RecoveryStrategy>>,
    /// Active recovery operations
    active_recoveries: Arc<RwLock<HashMap<Uuid, RecoveryOperation>>>,
    /// Self-healing state
    state: Arc<RwLock<SelfHealingState>>,
    /// Recovery statistics
    statistics: Arc<RwLock<RecoveryStatistics>>,
}

/// Recovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStatistics {
    /// Total recovery attempts
    pub total_attempts: u64,
    /// Successful recoveries
    pub successful_recoveries: u64,
    /// Failed recoveries
    pub failed_recoveries: u64,
    /// Average recovery time
    pub average_recovery_time: Duration,
    /// Recovery success rate by category
    pub success_rate_by_category: HashMap<ErrorCategory, f32>,
    /// Most common recovery strategies
    pub strategy_usage: HashMap<String, u64>,
}

/// Health monitor for proactive recovery
pub struct HealthMonitor {
    /// Health check interval
    check_interval: Duration,
    /// Health thresholds
    thresholds: HealthThresholds,
    /// Monitoring metrics
    metrics: Arc<RwLock<HashMap<String, HealthMetric>>>,
}

/// Health check thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    /// CPU usage threshold (0.0 to 1.0)
    pub cpu_threshold: f32,
    /// Memory usage threshold (0.0 to 1.0)
    pub memory_threshold: f32,
    /// Error rate threshold (errors per second)
    pub error_rate_threshold: f32,
    /// Response time threshold (milliseconds)
    pub response_time_threshold: f32,
}

/// Health metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    /// Metric name
    pub name: String,
    /// Current value
    pub value: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Trend (positive, negative, stable)
    pub trend: MetricTrend,
}

/// Metric trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricTrend {
    Increasing,
    Decreasing,
    Stable,
}

impl DefaultRecoveryManager {
    /// Create new recovery manager with default strategies
    pub fn new() -> Self {
        let mut strategies = HashMap::new();

        // Network error strategies
        strategies.insert(ErrorCategory::Network, vec![
            RecoveryStrategy::ImmediateRetry {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
            },
            RecoveryStrategy::Fallback {
                fallback_operation: "cached_response".to_string(),
                parameters: HashMap::new(),
                timeout: Duration::from_secs(5),
            },
        ]);

        // Infrastructure error strategies
        strategies.insert(ErrorCategory::Infrastructure, vec![
            RecoveryStrategy::DelayedRetry {
                delay: Duration::from_secs(1),
                max_attempts: 2,
                circuit_breaker: Some("infrastructure".to_string()),
            },
            RecoveryStrategy::Scale {
                resource_type: "compute_instances".to_string(),
                target_instances: 2,
                scale_timeout: Duration::from_secs(60),
            },
        ]);

        // Resource error strategies
        strategies.insert(ErrorCategory::Resource, vec![
            RecoveryStrategy::Scale {
                resource_type: "memory".to_string(),
                target_instances: 1,
                scale_timeout: Duration::from_secs(30),
            },
            RecoveryStrategy::Degrade {
                degraded_mode: "low_memory_mode".to_string(),
                performance_level: 0.7,
                feature_set: vec!["essential_operations".to_string()],
            },
        ]);

        // Dependency error strategies
        strategies.insert(ErrorCategory::Dependency, vec![
            RecoveryStrategy::Fallback {
                fallback_operation: "default_implementation".to_string(),
                parameters: HashMap::new(),
                timeout: Duration::from_secs(10),
            },
            RecoveryStrategy::Degrade {
                degraded_mode: "offline_mode".to_string(),
                performance_level: 0.5,
                feature_set: vec!["core_features".to_string()],
            },
        ]);

        Self {
            strategies,
            active_recoveries: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(SelfHealingState::default())),
            statistics: Arc::new(RwLock::new(RecoveryStatistics::default())),
        }
    }

    /// Add custom recovery strategy
    pub fn add_strategy(&mut self, category: ErrorCategory, strategy: RecoveryStrategy) {
        self.strategies.entry(category).or_insert_with(Vec::new).push(strategy);
    }

    /// Start recovery monitoring
    pub async fn start_monitoring(&self) {
        // Start background task for monitoring active recoveries
        let active_recoveries = self.active_recoveries.clone();
        let state = self.state.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Update recovery progress
                let mut recoveries = active_recoveries.write().await;
                let mut completed_operations = Vec::new();
                
                for (id, operation) in recoveries.iter_mut() {
                    // Simulate progress update (in real implementation, this would check actual progress)
                    operation.progress += 0.1;
                    if operation.progress >= 1.0 {
                        completed_operations.push(*id);
                    }
                }
                
                // Remove completed operations
                for id in completed_operations {
                    recoveries.remove(&id);
                }
                
                // Update system health
                let mut system_state = state.write().await;
                system_state.last_health_check = chrono::Utc::now();
                system_state.health_score = Self::calculate_health_score(&recoveries).await;
            }
        });
    }

    async fn calculate_health_score(active_recoveries: &HashMap<Uuid, RecoveryOperation>) -> f32 {
        if active_recoveries.is_empty() {
            1.0
        } else {
            // Health score decreases with active recovery operations
            (1.0 - (active_recoveries.len() as f32 * 0.1)).max(0.1)
        }
    }

    /// Execute immediate retry strategy
    async fn execute_immediate_retry(
        &self,
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
        _context: &RecoveryContext,
    ) -> WorkflowResult<RecoveryResult> {
        let start_time = Instant::now();
        let mut attempts = 0;
        
        while attempts < max_attempts {
            attempts += 1;
            
            // Calculate delay with exponential backoff
            if attempts > 1 {
                let delay = std::cmp::min(
                    Duration::from_nanos(
                        (initial_delay.as_nanos() as f64 * backoff_multiplier.powi(attempts as i32 - 2)) as u64
                    ),
                    max_delay,
                );
                tokio::time::sleep(delay).await;
            }
            
            // In a real implementation, this would retry the original operation
            // For now, we simulate success after some attempts
            if attempts >= 2 {
                return Ok(RecoveryResult {
                    success: true,
                    strategy: RecoveryStrategy::ImmediateRetry {
                        max_attempts,
                        initial_delay,
                        max_delay,
                        backoff_multiplier,
                    },
                    recovery_time: start_time.elapsed(),
                    details: format!("Recovery successful after {} attempts", attempts),
                    cascaded_actions: vec![],
                });
            }
        }
        
        // Recovery failed
        let recovery_context = ErrorContext::new("immediate_retry_recovery".to_string())
            .with_metadata("attempts".to_string(), serde_json::json!(attempts))
            .with_metadata("max_attempts".to_string(), serde_json::json!(max_attempts));
        
        Err(WorkflowError::new(
            ErrorCategory::Internal,
            ErrorSeverity::Error,
            format!("Recovery failed after {} attempts", max_attempts),
            crate::error::types::ErrorDetails::Generic {
                code: "RECOVERY_FAILED".to_string(),
                details: vec![
                    ("strategy".to_string(), serde_json::json!("immediate_retry")),
                    ("attempts".to_string(), serde_json::json!(attempts)),
                ].into_iter().collect(),
            },
            recovery_context,
        ))
    }

    /// Execute fallback strategy
    async fn execute_fallback(
        &self,
        fallback_operation: &str,
        parameters: &HashMap<String, serde_json::Value>,
        timeout: Duration,
        _context: &RecoveryContext,
    ) -> WorkflowResult<RecoveryResult> {
        let start_time = Instant::now();
        
        // Simulate fallback execution with timeout
        match tokio::time::timeout(timeout, async {
            tokio::time::sleep(Duration::from_millis(100)).await; // Simulate work
            Ok::<(), WorkflowError>(())
        }).await {
            Ok(Ok(())) => Ok(RecoveryResult {
                success: true,
                strategy: RecoveryStrategy::Fallback {
                    fallback_operation: fallback_operation.to_string(),
                    parameters: parameters.clone(),
                    timeout,
                },
                recovery_time: start_time.elapsed(),
                details: format!("Fallback to '{}' successful", fallback_operation),
                cascaded_actions: vec![],
            }),
            Ok(Err(error)) => Err(error),
            Err(_) => {
                let recovery_context = ErrorContext::new("fallback_recovery".to_string())
                    .with_metadata("operation".to_string(), serde_json::json!(fallback_operation))
                    .with_metadata("timeout_ms".to_string(), serde_json::json!(timeout.as_millis()));
                
                Err(WorkflowError::new(
                    ErrorCategory::Internal,
                    ErrorSeverity::Error,
                    format!("Fallback operation '{}' timed out", fallback_operation),
                    crate::error::types::ErrorDetails::Generic {
                        code: "FALLBACK_TIMEOUT".to_string(),
                        details: vec![
                            ("operation".to_string(), serde_json::json!(fallback_operation)),
                            ("timeout_ms".to_string(), serde_json::json!(timeout.as_millis())),
                        ].into_iter().collect(),
                    },
                    recovery_context,
                ))
            }
        }
    }

    /// Execute degradation strategy
    async fn execute_degradation(
        &self,
        degraded_mode: &str,
        performance_level: f32,
        feature_set: &[String],
        _context: &RecoveryContext,
    ) -> WorkflowResult<RecoveryResult> {
        let start_time = Instant::now();
        
        // Add degraded mode to system state
        let mut state = self.state.write().await;
        state.degraded_modes.insert(
            degraded_mode.to_string(),
            DegradedModeInfo {
                mode_id: degraded_mode.to_string(),
                performance_level,
                available_features: feature_set.to_vec(),
                disabled_features: vec![], // In real implementation, calculate disabled features
                started_at: chrono::Utc::now(),
                expected_duration: Some(Duration::from_secs(15 * 60)), // Default degradation duration (15 minutes)
            },
        );
        
        Ok(RecoveryResult {
            success: true,
            strategy: RecoveryStrategy::Degrade {
                degraded_mode: degraded_mode.to_string(),
                performance_level,
                feature_set: feature_set.to_vec(),
            },
            recovery_time: start_time.elapsed(),
            details: format!("Degraded to '{}' mode with {}% performance", degraded_mode, performance_level * 100.0),
            cascaded_actions: vec![
                RecoveryAction::Alert {
                    severity: crate::error::types::AlertSeverity::Medium,
                    message: format!("System degraded to '{}' mode", degraded_mode),
                },
            ],
        })
    }

    /// Update recovery statistics
    async fn update_statistics(&self, result: &RecoveryResult, error_category: &ErrorCategory) {
        let mut stats = self.statistics.write().await;
        
        stats.total_attempts += 1;
        
        if result.success {
            stats.successful_recoveries += 1;
        } else {
            stats.failed_recoveries += 1;
        }
        
        // Update average recovery time
        let total_recoveries = stats.successful_recoveries + stats.failed_recoveries;
        if total_recoveries > 0 {
            let current_avg = stats.average_recovery_time.as_nanos() as f64;
            let new_time = result.recovery_time.as_nanos() as f64;
            let new_avg = ((current_avg * (total_recoveries - 1) as f64) + new_time) / total_recoveries as f64;
            stats.average_recovery_time = Duration::from_nanos(new_avg as u64);
        }
        
        // Update success rate by category
        let category_success_rate = stats.success_rate_by_category.entry(error_category.clone()).or_insert(0.0);
        *category_success_rate = if result.success {
            (*category_success_rate + 1.0) / 2.0
        } else {
            *category_success_rate / 2.0
        };
        
        // Update strategy usage
        let strategy_name = match &result.strategy {
            RecoveryStrategy::ImmediateRetry { .. } => "immediate_retry",
            RecoveryStrategy::DelayedRetry { .. } => "delayed_retry",
            RecoveryStrategy::Fallback { .. } => "fallback",
            RecoveryStrategy::Degrade { .. } => "degrade",
            RecoveryStrategy::Restart { .. } => "restart",
            RecoveryStrategy::Scale { .. } => "scale",
            RecoveryStrategy::ManualIntervention { .. } => "manual",
        };
        *stats.strategy_usage.entry(strategy_name.to_string()).or_insert(0) += 1;
    }
}

#[async_trait]
impl RecoveryManager for DefaultRecoveryManager {
    async fn execute_recovery(
        &self,
        error: &WorkflowError,
        context: RecoveryContext,
    ) -> WorkflowResult<RecoveryResult> {
        // Get recommended strategy
        let strategy = self.recommend_strategy(error)
            .ok_or_else(|| {
                let error_context = ErrorContext::new("execute_recovery".to_string())
                    .with_metadata("error_category".to_string(), serde_json::json!(error.category))
                    .with_metadata("error_id".to_string(), serde_json::json!(error.error_id));
                
                WorkflowError::new(
                    ErrorCategory::Internal,
                    ErrorSeverity::Error,
                    "No recovery strategy available for error".to_string(),
                    crate::error::types::ErrorDetails::Generic {
                        code: "NO_RECOVERY_STRATEGY".to_string(),
                        details: HashMap::new(),
                    },
                    error_context,
                )
            })?;

        // Execute strategy
        let result = match strategy {
            RecoveryStrategy::ImmediateRetry { max_attempts, initial_delay, max_delay, backoff_multiplier } => {
                self.execute_immediate_retry(max_attempts, initial_delay, max_delay, backoff_multiplier, &context).await
            }
            RecoveryStrategy::Fallback { fallback_operation, parameters, timeout } => {
                self.execute_fallback(&fallback_operation, &parameters, timeout, &context).await
            }
            RecoveryStrategy::Degrade { degraded_mode, performance_level, feature_set } => {
                self.execute_degradation(&degraded_mode, performance_level, &feature_set, &context).await
            }
            _ => {
                // Other strategies would be implemented here
                let error_context = ErrorContext::new("execute_recovery".to_string())
                    .with_metadata("strategy".to_string(), serde_json::json!(format!("{:?}", strategy)));
                
                Err(WorkflowError::new(
                    ErrorCategory::Internal,
                    ErrorSeverity::Warning,
                    "Recovery strategy not yet implemented".to_string(),
                    crate::error::types::ErrorDetails::Generic {
                        code: "STRATEGY_NOT_IMPLEMENTED".to_string(),
                        details: HashMap::new(),
                    },
                    error_context,
                ))
            }
        };

        // Update statistics
        if let Ok(ref recovery_result) = result {
            self.update_statistics(recovery_result, &error.category).await;
        }

        result
    }

    fn recommend_strategy(&self, error: &WorkflowError) -> Option<RecoveryStrategy> {
        self.strategies.get(&error.category)?
            .first()
            .cloned()
    }

    async fn needs_recovery(&self, error: &WorkflowError) -> bool {
        // Recovery is needed for recoverable errors with appropriate severity
        error.is_recoverable() && error.severity >= ErrorSeverity::Warning
    }

    async fn get_system_health(&self) -> SelfHealingState {
        self.state.read().await.clone()
    }
}

impl Default for SelfHealingState {
    fn default() -> Self {
        Self {
            degraded_modes: HashMap::new(),
            active_recoveries: HashMap::new(),
            health_score: 1.0,
            last_health_check: chrono::Utc::now(),
        }
    }
}

impl Default for RecoveryStatistics {
    fn default() -> Self {
        Self {
            total_attempts: 0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            average_recovery_time: Duration::from_millis(0),
            success_rate_by_category: HashMap::new(),
            strategy_usage: HashMap::new(),
        }
    }
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new(check_interval: Duration, thresholds: HealthThresholds) -> Self {
        Self {
            check_interval,
            thresholds,
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start health monitoring
    pub async fn start_monitoring(&self, _recovery_manager: Arc<dyn RecoveryManager>) {
        let check_interval = self.check_interval;
        let thresholds = self.thresholds.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            
            loop {
                interval.tick().await;
                
                // Simulate health check metrics
                let mut current_metrics = metrics.write().await;
                
                // CPU usage
                let cpu_usage = rand::random::<f64>() * 0.8; // Simulate CPU usage
                current_metrics.insert("cpu_usage".to_string(), HealthMetric {
                    name: "cpu_usage".to_string(),
                    value: cpu_usage,
                    timestamp: chrono::Utc::now(),
                    trend: if cpu_usage > 0.5 { MetricTrend::Increasing } else { MetricTrend::Stable },
                });
                
                // Memory usage
                let memory_usage = rand::random::<f64>() * 0.9; // Simulate memory usage
                current_metrics.insert("memory_usage".to_string(), HealthMetric {
                    name: "memory_usage".to_string(),
                    value: memory_usage,
                    timestamp: chrono::Utc::now(),
                    trend: if memory_usage > 0.7 { MetricTrend::Increasing } else { MetricTrend::Stable },
                });
                
                // Check thresholds and trigger recovery if needed
                if cpu_usage > thresholds.cpu_threshold as f64 {
                    // In real implementation, would trigger CPU-related recovery
                    println!("CPU usage {} exceeds threshold {}", cpu_usage, thresholds.cpu_threshold);
                }
                
                if memory_usage > thresholds.memory_threshold as f64 {
                    // In real implementation, would trigger memory-related recovery
                    println!("Memory usage {} exceeds threshold {}", memory_usage, thresholds.memory_threshold);
                }
            }
        });
    }

    /// Get current health metrics
    pub async fn get_metrics(&self) -> HashMap<String, HealthMetric> {
        self.metrics.read().await.clone()
    }
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 0.8,
            memory_threshold: 0.85,
            error_rate_threshold: 10.0,
            response_time_threshold: 1000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_recovery_manager() {
        let manager = DefaultRecoveryManager::new();
        
        // Create test error
        let context = ErrorContext::new("test_operation".to_string());
        let error = WorkflowError::network_error(
            "example.com".to_string(),
            Some(80),
            "http".to_string(),
            Some(Duration::from_secs(30)),
            context,
        );

        // Test recovery recommendation
        let strategy = manager.recommend_strategy(&error);
        assert!(strategy.is_some());

        // Test recovery execution
        let recovery_context = RecoveryContext {
            original_error: error.clone(),
            error_history: vec![],
            system_metrics: HashMap::new(),
            available_resources: HashMap::new(),
            recovery_attempts: 0,
        };

        let result = manager.execute_recovery(&error, recovery_context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_monitor() {
        let thresholds = HealthThresholds::default();
        let monitor = HealthMonitor::new(Duration::from_millis(100), thresholds);
        
        // Wait a bit for metrics to be generated
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Health monitor would be started with actual recovery manager in real usage
        // For test, just verify it can be created
        assert_eq!(monitor.check_interval, Duration::from_millis(100));
    }
}