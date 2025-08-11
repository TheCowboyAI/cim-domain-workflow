//! Performance optimization utilities and automatic tuning

use super::PerformanceMonitor;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Performance optimizer that automatically tunes system parameters
pub struct PerformanceOptimizer {
    /// Performance monitor
    monitor: Arc<PerformanceMonitor>,
    /// Optimization configuration
    config: OptimizerConfig,
    /// Optimization history
    history: Arc<RwLock<VecDeque<OptimizationRun>>>,
    /// Current optimization parameters
    parameters: Arc<RwLock<OptimizationParameters>>,
    /// Adaptive learning state
    learning_state: Arc<RwLock<LearningState>>,
}

/// Configuration for the performance optimizer
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Whether optimization is enabled
    pub enabled: bool,
    /// Optimization interval
    pub optimization_interval: Duration,
    /// Maximum number of optimization runs to keep in history
    pub max_history: usize,
    /// Learning rate for adaptive optimization
    pub learning_rate: f64,
    /// Minimum improvement threshold to apply changes
    pub min_improvement_threshold: f64,
    /// Maximum parameter adjustment per run
    pub max_adjustment_ratio: f64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            optimization_interval: Duration::from_secs(300), // 5 minutes
            max_history: 100,
            learning_rate: 0.1,
            min_improvement_threshold: 0.05, // 5% improvement
            max_adjustment_ratio: 0.2, // 20% max adjustment
        }
    }
}

/// Optimization parameters that can be tuned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationParameters {
    /// Thread pool size
    pub thread_pool_size: usize,
    /// Buffer sizes for various operations
    pub buffer_sizes: HashMap<String, usize>,
    /// Timeout configurations
    pub timeouts: HashMap<String, Duration>,
    /// Cache configurations
    pub cache_settings: HashMap<String, CacheConfig>,
    /// Batch processing sizes
    pub batch_sizes: HashMap<String, usize>,
    /// Retry configurations
    pub retry_settings: HashMap<String, RetryConfig>,
}

impl Default for OptimizationParameters {
    fn default() -> Self {
        let mut buffer_sizes = HashMap::new();
        buffer_sizes.insert("event_buffer".to_string(), 1000);
        buffer_sizes.insert("metric_buffer".to_string(), 500);
        buffer_sizes.insert("log_buffer".to_string(), 2000);

        let mut timeouts = HashMap::new();
        timeouts.insert("operation_timeout".to_string(), Duration::from_secs(30));
        timeouts.insert("database_timeout".to_string(), Duration::from_secs(5));
        timeouts.insert("network_timeout".to_string(), Duration::from_secs(10));

        let mut cache_settings = HashMap::new();
        cache_settings.insert("workflow_cache".to_string(), CacheConfig {
            max_size: 1000,
            ttl: Duration::from_secs(3600),
            eviction_policy: EvictionPolicy::LRU,
        });

        let mut batch_sizes = HashMap::new();
        batch_sizes.insert("event_batch".to_string(), 50);
        batch_sizes.insert("metric_batch".to_string(), 25);

        let mut retry_settings = HashMap::new();
        retry_settings.insert("default".to_string(), RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        });

        Self {
            thread_pool_size: num_cpus::get(),
            buffer_sizes,
            timeouts,
            cache_settings,
            batch_sizes,
            retry_settings,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_size: usize,
    pub ttl: Duration,
    pub eviction_policy: EvictionPolicy,
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    FIFO, // First In, First Out
    TTL,  // Time To Live
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

/// Learning state for adaptive optimization
#[derive(Debug, Clone)]
struct LearningState {
    /// Parameter effectiveness scores
    parameter_scores: HashMap<String, f64>,
    /// Recent performance trends
    performance_trends: HashMap<String, PerformanceTrend>,
    /// Adjustment history
    adjustment_history: VecDeque<ParameterAdjustment>,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
struct PerformanceTrend {
    metric_name: String,
    trend_direction: TrendDirection,
    trend_strength: f64,
    confidence: f64,
}

/// Trend direction
#[derive(Debug, Clone)]
enum TrendDirection {
    Improving,
    Degrading,
    Stable,
}

/// Parameter adjustment record
#[derive(Debug, Clone)]
struct ParameterAdjustment {
    parameter_name: String,
    old_value: serde_json::Value,
    new_value: serde_json::Value,
    impact_score: f64,
    timestamp: SystemTime,
}

/// Optimization run result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRun {
    /// Run ID
    pub run_id: String,
    /// Timestamp
    pub timestamp: SystemTime,
    /// Parameters before optimization
    pub before_parameters: OptimizationParameters,
    /// Parameters after optimization
    pub after_parameters: OptimizationParameters,
    /// Performance metrics before optimization
    pub before_metrics: OptimizationMetrics,
    /// Performance metrics after optimization
    pub after_metrics: Option<OptimizationMetrics>,
    /// Changes made
    pub changes: Vec<ParameterChange>,
    /// Overall improvement score
    pub improvement_score: f64,
    /// Optimization strategy used
    pub strategy: OptimizationStrategy,
}

/// Optimization metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    pub avg_response_time: Duration,
    pub throughput: f64,
    pub error_rate: f64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub active_workflows: u32,
}

/// Parameter change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterChange {
    pub parameter_name: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub change_reason: String,
}

/// Optimization strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    GradientDescent,
    SimulatedAnnealing,
    GeneticAlgorithm,
    AdaptiveLearning,
    RuleBasedTuning,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(monitor: Arc<PerformanceMonitor>, config: OptimizerConfig) -> Self {
        Self {
            monitor,
            config,
            history: Arc::new(RwLock::new(VecDeque::new())),
            parameters: Arc::new(RwLock::new(OptimizationParameters::default())),
            learning_state: Arc::new(RwLock::new(LearningState {
                parameter_scores: HashMap::new(),
                performance_trends: HashMap::new(),
                adjustment_history: VecDeque::new(),
            })),
        }
    }

    /// Run optimization cycle
    pub async fn optimize(&self) -> Option<OptimizationRun> {
        if !self.config.enabled {
            return None;
        }

        let run_id = uuid::Uuid::new_v4().to_string();
        let timestamp = SystemTime::now();

        // Collect current metrics
        let before_metrics = self.collect_optimization_metrics().await?;
        let before_parameters = self.parameters.read().await.clone();

        // Analyze performance trends
        let trends = self.analyze_performance_trends().await;
        
        // Determine optimization strategy
        let strategy = self.select_optimization_strategy(&trends).await;

        // Generate parameter changes
        let changes = self.generate_parameter_changes(&strategy, &trends, &before_parameters).await;

        if changes.is_empty() {
            return None;
        }

        // Apply changes
        let after_parameters = self.apply_parameter_changes(&before_parameters, &changes).await;

        let optimization_run = OptimizationRun {
            run_id,
            timestamp,
            before_parameters,
            after_parameters: after_parameters.clone(),
            before_metrics,
            after_metrics: None, // Will be filled in after a monitoring period
            changes,
            improvement_score: 0.0, // Will be calculated later
            strategy,
        };

        // Update parameters
        {
            let mut params = self.parameters.write().await;
            *params = after_parameters;
        }

        // Store in history
        {
            let mut history = self.history.write().await;
            history.push_back(optimization_run.clone());
            
            // Maintain history size limit
            while history.len() > self.config.max_history {
                history.pop_front();
            }
        }

        Some(optimization_run)
    }

    /// Collect current optimization metrics
    async fn collect_optimization_metrics(&self) -> Option<OptimizationMetrics> {
        let profiles = self.monitor.get_all_profiles().await;
        let system_perf = self.monitor.get_system_performance().await?;

        let total_executions: u64 = profiles.values().map(|p| p.execution_count).sum();
        let avg_response_time = if !profiles.is_empty() {
            let total_nanos: u128 = profiles.values()
                .map(|p| p.avg_duration.as_nanos() * p.execution_count as u128)
                .sum();
            Duration::from_nanos((total_nanos / total_executions as u128) as u64)
        } else {
            Duration::from_nanos(0)
        };

        let total_errors: f64 = profiles.values().map(|p| p.error_rate * p.execution_count as f64).sum();
        let error_rate = if total_executions > 0 {
            total_errors / total_executions as f64
        } else {
            0.0
        };

        let throughput = profiles.values().map(|p| p.throughput).sum::<f64>();

        Some(OptimizationMetrics {
            avg_response_time,
            throughput,
            error_rate,
            memory_usage: system_perf.memory_usage,
            cpu_usage: system_perf.cpu_usage,
            active_workflows: system_perf.active_workflows,
        })
    }

    /// Analyze performance trends
    async fn analyze_performance_trends(&self) -> HashMap<String, PerformanceTrend> {
        let mut trends = HashMap::new();
        
        // Get cached trends from learning state
        let learning_state = self.learning_state.read().await;
        let cached_trends = &learning_state.performance_trends;
        
        let recent_metrics = self.monitor.get_recent_metrics(Some(100)).await;
        if recent_metrics.len() < 10 {
            // Return cached trends if available
            return cached_trends.clone();
        }

        // Analyze response time trend
        let response_times: Vec<f64> = recent_metrics.iter()
            .map(|m| m.duration.as_secs_f64())
            .collect();
        
        if let Some(trend) = self.calculate_trend(&response_times) {
            trends.insert("response_time".to_string(), PerformanceTrend {
                metric_name: "response_time".to_string(),
                trend_direction: trend.0,
                trend_strength: trend.1,
                confidence: trend.2,
            });
        }

        // Analyze memory usage trend
        let memory_usage: Vec<f64> = recent_metrics.iter()
            .map(|m| m.memory_delta as f64)
            .collect();
        
        if let Some(trend) = self.calculate_trend(&memory_usage) {
            trends.insert("memory_usage".to_string(), PerformanceTrend {
                metric_name: "memory_usage".to_string(),
                trend_direction: trend.0,
                trend_strength: trend.1,
                confidence: trend.2,
            });
        }

        // Update learning state with new trends
        drop(learning_state); // Release the read lock
        let mut learning_state_write = self.learning_state.write().await;
        learning_state_write.performance_trends = trends.clone();

        trends
    }

    /// Calculate trend from a series of values
    fn calculate_trend(&self, values: &[f64]) -> Option<(TrendDirection, f64, f64)> {
        if values.len() < 5 {
            return None;
        }

        // Simple linear regression
        let n = values.len() as f64;
        let x_values: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        
        let x_mean = x_values.iter().sum::<f64>() / n;
        let y_mean = values.iter().sum::<f64>() / n;

        let numerator: f64 = x_values.iter().zip(values.iter())
            .map(|(&x, &y)| (x - x_mean) * (y - y_mean))
            .sum();

        let denominator: f64 = x_values.iter()
            .map(|&x| (x - x_mean).powi(2))
            .sum();

        if denominator == 0.0 {
            return Some((TrendDirection::Stable, 0.0, 0.0));
        }

        let slope = numerator / denominator;
        
        // Calculate correlation coefficient for confidence
        let y_variance: f64 = values.iter()
            .map(|&y| (y - y_mean).powi(2))
            .sum();

        let confidence = if y_variance == 0.0 {
            0.0
        } else {
            (numerator / (denominator * y_variance).sqrt()).abs()
        };

        let direction = if slope.abs() < 0.001 {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Degrading // Assuming higher values are worse
        } else {
            TrendDirection::Improving
        };

        Some((direction, slope.abs(), confidence))
    }

    /// Select optimization strategy based on trends
    async fn select_optimization_strategy(
        &self,
        trends: &HashMap<String, PerformanceTrend>,
    ) -> OptimizationStrategy {
        let degrading_trends = trends.values()
            .filter(|t| matches!(t.trend_direction, TrendDirection::Degrading))
            .count();

        let high_confidence_trends = trends.values()
            .filter(|t| t.confidence > 0.7)
            .count();

        match (degrading_trends, high_confidence_trends) {
            (0, _) => OptimizationStrategy::AdaptiveLearning,
            (1..=2, _) if high_confidence_trends > 0 => OptimizationStrategy::GradientDescent,
            (_, _) if high_confidence_trends > 2 => OptimizationStrategy::RuleBasedTuning,
            _ => OptimizationStrategy::SimulatedAnnealing,
        }
    }

    /// Generate parameter changes based on strategy and trends
    async fn generate_parameter_changes(
        &self,
        strategy: &OptimizationStrategy,
        trends: &HashMap<String, PerformanceTrend>,
        current_params: &OptimizationParameters,
    ) -> Vec<ParameterChange> {
        let mut changes = Vec::new();

        match strategy {
            OptimizationStrategy::RuleBasedTuning => {
                changes.extend(self.apply_rule_based_tuning(trends, current_params).await);
            },
            OptimizationStrategy::AdaptiveLearning => {
                changes.extend(self.apply_adaptive_learning(trends, current_params).await);
            },
            OptimizationStrategy::GradientDescent => {
                changes.extend(self.apply_gradient_descent(trends, current_params).await);
            },
            _ => {
                // Fallback to rule-based tuning
                changes.extend(self.apply_rule_based_tuning(trends, current_params).await);
            },
        }

        changes
    }

    /// Apply rule-based tuning
    async fn apply_rule_based_tuning(
        &self,
        trends: &HashMap<String, PerformanceTrend>,
        current_params: &OptimizationParameters,
    ) -> Vec<ParameterChange> {
        let mut changes = Vec::new();

        // Rule: If response time is degrading, increase buffer sizes
        if let Some(trend) = trends.get("response_time") {
            if matches!(trend.trend_direction, TrendDirection::Degrading) && trend.confidence > 0.5 {
                for (buffer_name, &current_size) in &current_params.buffer_sizes {
                    let new_size = (current_size as f64 * 1.2) as usize;
                    changes.push(ParameterChange {
                        parameter_name: format!("buffer_sizes.{}", buffer_name),
                        old_value: serde_json::Value::Number(current_size.into()),
                        new_value: serde_json::Value::Number(new_size.into()),
                        change_reason: "Increase buffer size due to degrading response time".to_string(),
                    });
                }
            }
        }

        // Rule: If memory usage is high, reduce cache sizes
        if let Some(trend) = trends.get("memory_usage") {
            if matches!(trend.trend_direction, TrendDirection::Degrading) && trend.confidence > 0.5 {
                for (cache_name, cache_config) in &current_params.cache_settings {
                    let new_size = (cache_config.max_size as f64 * 0.8) as usize;
                    changes.push(ParameterChange {
                        parameter_name: format!("cache_settings.{}.max_size", cache_name),
                        old_value: serde_json::Value::Number(cache_config.max_size.into()),
                        new_value: serde_json::Value::Number(new_size.into()),
                        change_reason: "Reduce cache size due to high memory usage".to_string(),
                    });
                }
            }
        }

        changes
    }

    /// Apply adaptive learning optimization
    async fn apply_adaptive_learning(
        &self,
        _trends: &HashMap<String, PerformanceTrend>,
        current_params: &OptimizationParameters,
    ) -> Vec<ParameterChange> {
        let mut changes = Vec::new();
        let learning_state = self.learning_state.read().await;

        // Use historical parameter effectiveness to make adjustments
        for (param_name, &score) in &learning_state.parameter_scores {
            if score < -0.1 { // Parameter has been ineffective
                // Make small adjustments based on learning
                if param_name.starts_with("buffer_sizes.") {
                    let buffer_name = param_name.trim_start_matches("buffer_sizes.");
                    if let Some(&current_size) = current_params.buffer_sizes.get(buffer_name) {
                        let adjustment = (current_size as f64 * self.config.learning_rate * score.abs()) as usize;
                        let new_size = current_size.saturating_sub(adjustment);
                        
                        changes.push(ParameterChange {
                            parameter_name: param_name.clone(),
                            old_value: serde_json::Value::Number(current_size.into()),
                            new_value: serde_json::Value::Number(new_size.into()),
                            change_reason: format!("Adaptive learning adjustment (score: {:.3})", score),
                        });
                    }
                }
            }
        }

        changes
    }

    /// Apply gradient descent optimization
    async fn apply_gradient_descent(
        &self,
        trends: &HashMap<String, PerformanceTrend>,
        current_params: &OptimizationParameters,
    ) -> Vec<ParameterChange> {
        let mut changes = Vec::new();

        // Simple gradient descent: move parameters in direction opposite to degradation
        for trend in trends.values() {
            if trend.confidence > 0.6 {
                match trend.metric_name.as_str() {
                    "response_time" => {
                        if matches!(trend.trend_direction, TrendDirection::Degrading) {
                            // Increase thread pool size
                            let new_size = (current_params.thread_pool_size as f64 * 1.1) as usize;
                            changes.push(ParameterChange {
                                parameter_name: "thread_pool_size".to_string(),
                                old_value: serde_json::Value::Number(current_params.thread_pool_size.into()),
                                new_value: serde_json::Value::Number(new_size.into()),
                                change_reason: "Gradient descent: increase threads for better response time".to_string(),
                            });
                        }
                    },
                    _ => {}
                }
            }
        }

        changes
    }

    /// Apply parameter changes
    async fn apply_parameter_changes(
        &self,
        current_params: &OptimizationParameters,
        changes: &[ParameterChange],
    ) -> OptimizationParameters {
        let mut new_params = current_params.clone();

        for change in changes {
            match change.parameter_name.as_str() {
                "thread_pool_size" => {
                    if let Some(size) = change.new_value.as_u64() {
                        new_params.thread_pool_size = size as usize;
                    }
                },
                name if name.starts_with("buffer_sizes.") => {
                    let buffer_name = name.trim_start_matches("buffer_sizes.");
                    if let Some(size) = change.new_value.as_u64() {
                        new_params.buffer_sizes.insert(buffer_name.to_string(), size as usize);
                    }
                },
                name if name.starts_with("cache_settings.") && name.ends_with(".max_size") => {
                    let cache_name = name.trim_start_matches("cache_settings.")
                        .trim_end_matches(".max_size");
                    if let Some(size) = change.new_value.as_u64() {
                        if let Some(cache_config) = new_params.cache_settings.get_mut(cache_name) {
                            cache_config.max_size = size as usize;
                        }
                    }
                },
                _ => {
                    // Log unhandled parameter changes
                    eprintln!("Unhandled parameter change: {}", change.parameter_name);
                }
            }
        }

        // Track parameter adjustments in learning state
        let timestamp = SystemTime::now();
        let mut learning_state = self.learning_state.write().await;
        
        for change in changes {
            let adjustment = ParameterAdjustment {
                parameter_name: change.parameter_name.clone(),
                old_value: change.old_value.clone(),
                new_value: change.new_value.clone(),
                impact_score: 0.0, // Will be calculated later when results are known
                timestamp,
            };
            
            learning_state.adjustment_history.push_back(adjustment);
            
            // Keep only recent adjustments (last 1000)
            if learning_state.adjustment_history.len() > 1000 {
                learning_state.adjustment_history.pop_front();
            }
        }

        new_params
    }

    /// Get current optimization parameters
    pub async fn get_parameters(&self) -> OptimizationParameters {
        self.parameters.read().await.clone()
    }

    /// Get optimization history
    pub async fn get_history(&self) -> Vec<OptimizationRun> {
        self.history.read().await.iter().cloned().collect()
    }

    /// Update learning state with performance feedback
    pub async fn update_learning_state(&self, run_id: &str, improvement_score: f64) {
        let mut learning_state = self.learning_state.write().await;
        let mut history = self.history.write().await;

        // Find the optimization run
        if let Some(run) = history.iter_mut().find(|r| r.run_id == run_id) {
            run.improvement_score = improvement_score;

            // Update parameter effectiveness scores
            for change in &run.changes {
                let current_score = learning_state.parameter_scores
                    .get(&change.parameter_name)
                    .cloned()
                    .unwrap_or(0.0);
                
                let new_score = current_score + self.config.learning_rate * improvement_score;
                learning_state.parameter_scores.insert(change.parameter_name.clone(), new_score);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::PerformanceThresholds;

    #[tokio::test]
    async fn test_optimizer_creation() {
        let monitor = Arc::new(PerformanceMonitor::new(PerformanceThresholds::default()));
        let optimizer = PerformanceOptimizer::new(monitor, OptimizerConfig::default());
        
        let params = optimizer.get_parameters().await;
        assert!(params.thread_pool_size > 0);
        assert!(!params.buffer_sizes.is_empty());
    }

    #[tokio::test]
    async fn test_trend_calculation() {
        let monitor = Arc::new(PerformanceMonitor::new(PerformanceThresholds::default()));
        let optimizer = PerformanceOptimizer::new(monitor, OptimizerConfig::default());
        
        // Test stable trend
        let stable_values = vec![1.0, 1.1, 0.9, 1.0, 1.1];
        let trend = optimizer.calculate_trend(&stable_values);
        assert!(trend.is_some());
        
        // Test increasing trend
        let increasing_values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let trend = optimizer.calculate_trend(&increasing_values);
        assert!(trend.is_some());
        let (direction, strength, confidence) = trend.unwrap();
        assert!(matches!(direction, TrendDirection::Degrading));
        assert!(strength > 0.0);
        assert!(confidence > 0.8);
    }
}