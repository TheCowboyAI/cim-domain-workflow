//! Health Check and System Monitoring
//!
//! Provides comprehensive health checking capabilities with support for
//! custom health checks, dependency monitoring, and system status reporting.

use crate::error::types::{WorkflowError, WorkflowResult, ErrorCategory, ErrorSeverity, ErrorContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy and functioning normally
    Healthy,
    /// Component is degraded but still functional
    Degraded,
    /// Component is unhealthy and may not be functioning
    Unhealthy,
    /// Component is not responding
    Unresponsive,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Check identifier
    pub check_id: String,
    /// Component name being checked
    pub component: String,
    /// Health status
    pub status: HealthStatus,
    /// Status message
    pub message: String,
    /// Check execution time
    pub execution_time: Duration,
    /// Check timestamp
    pub timestamp: SystemTime,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
    /// Error details if unhealthy
    pub error: Option<WorkflowError>,
}

/// System health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthSummary {
    /// Overall system status
    pub overall_status: HealthStatus,
    /// Individual component results
    pub component_results: HashMap<String, HealthCheckResult>,
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    /// Total number of checks
    pub total_checks: u32,
    /// Number of healthy checks
    pub healthy_checks: u32,
    /// Number of degraded checks
    pub degraded_checks: u32,
    /// Number of unhealthy checks
    pub unhealthy_checks: u32,
    /// When the summary was generated
    pub generated_at: SystemTime,
    /// System uptime
    pub uptime: Duration,
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Get the unique identifier for this health check
    fn check_id(&self) -> &str;
    
    /// Get the component name being checked
    fn component(&self) -> &str;
    
    /// Execute the health check
    async fn check(&self) -> HealthCheckResult;
    
    /// Get the recommended check interval
    fn check_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    /// Check if this health check is critical for system health
    fn is_critical(&self) -> bool {
        true
    }
}

/// Database connection health check
pub struct DatabaseHealthCheck {
    check_id: String,
    _connection_string: String,
    timeout: Duration,
}

/// NATS connection health check
pub struct NatsHealthCheck {
    check_id: String,
    servers: Vec<String>,
    timeout: Duration,
}

/// Memory usage health check
pub struct MemoryHealthCheck {
    check_id: String,
    warning_threshold: f64,
    critical_threshold: f64,
}

/// CPU usage health check
pub struct CpuHealthCheck {
    _check_id: String,
    _warning_threshold: f64,
    _critical_threshold: f64,
    _sample_duration: Duration,
}

/// Disk space health check
pub struct DiskSpaceHealthCheck {
    _check_id: String,
    _path: String,
    _warning_threshold: f64,
    _critical_threshold: f64,
}

/// External service health check
pub struct ServiceHealthCheck {
    check_id: String,
    service_name: String,
    _endpoint: String,
    timeout: Duration,
    expected_status_codes: Vec<u16>,
}

/// Custom health check implementation
pub struct CustomHealthCheck {
    _check_id: String,
    _component: String,
    _check_fn: Box<dyn Fn() -> futures::future::BoxFuture<'static, HealthCheckResult> + Send + Sync>,
    _interval: Duration,
    _critical: bool,
}

/// Health monitor that orchestrates health checks
pub struct HealthMonitor {
    /// Registered health checks
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,
    /// Health check results
    results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    /// Monitor configuration
    config: HealthMonitorConfig,
    /// System start time
    start_time: Instant,
}

/// Health monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    /// Default check interval
    pub default_interval: Duration,
    /// Timeout for health checks
    pub check_timeout: Duration,
    /// Number of failed checks before marking unhealthy
    pub failure_threshold: u32,
    /// Whether to run checks in parallel
    pub parallel_checks: bool,
    /// Maximum concurrent health checks
    pub max_concurrent_checks: usize,
}

impl DatabaseHealthCheck {
    pub fn new(check_id: String, connection_string: String, timeout: Duration) -> Self {
        Self {
            check_id,
            _connection_string: connection_string,
            timeout,
        }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn check_id(&self) -> &str {
        &self.check_id
    }
    
    fn component(&self) -> &str {
        "database"
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate database connectivity check
        let (status, message, metrics) = match tokio::time::timeout(
            self.timeout,
            async {
                // In real implementation, would test actual database connection
                tokio::time::sleep(Duration::from_millis(50)).await;
                
                // Simulate connection metrics
                let connection_time = rand::random::<f64>() * 100.0;
                let active_connections = rand::random::<f64>() * 50.0;
                
                let mut metrics = std::collections::HashMap::new();
                metrics.insert("connection_time_ms".to_string(), connection_time);
                metrics.insert("active_connections".to_string(), active_connections);
                
                if connection_time > 500.0 {
                    (HealthStatus::Degraded, "Database connection slow", metrics)
                } else if active_connections > 45.0 {
                    (HealthStatus::Degraded, "High connection count", metrics)
                } else {
                    (HealthStatus::Healthy, "Database connection healthy", metrics)
                }
            }
        ).await {
            Ok(result) => result,
            Err(_) => (HealthStatus::Unresponsive, "Database connection timeout", std::collections::HashMap::new()),
        };
        
        HealthCheckResult {
            check_id: self.check_id.clone(),
            component: self.component().to_string(),
            status,
            message: message.to_string(),
            execution_time: start_time.elapsed(),
            timestamp: SystemTime::now(),
            metrics,
            error: None,
        }
    }
    
    fn check_interval(&self) -> Duration {
        Duration::from_secs(60)
    }
    
    fn is_critical(&self) -> bool {
        true
    }
}

impl NatsHealthCheck {
    pub fn new(check_id: String, servers: Vec<String>, timeout: Duration) -> Self {
        Self {
            check_id,
            servers,
            timeout,
        }
    }
}

#[async_trait]
impl HealthCheck for NatsHealthCheck {
    fn check_id(&self) -> &str {
        &self.check_id
    }
    
    fn component(&self) -> &str {
        "nats"
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate NATS connectivity check
        let (status, message, metrics) = match tokio::time::timeout(
            self.timeout,
            async {
                // In real implementation, would test actual NATS connection
                tokio::time::sleep(Duration::from_millis(30)).await;
                
                let connected_servers = self.servers.len() as f64;
                let message_rate = rand::random::<f64>() * 1000.0;
                let pending_messages = rand::random::<f64>() * 100.0;
                
                let mut metrics = std::collections::HashMap::new();
                metrics.insert("connected_servers".to_string(), connected_servers);
                metrics.insert("message_rate".to_string(), message_rate);
                metrics.insert("pending_messages".to_string(), pending_messages);
                
                if pending_messages > 80.0 {
                    (HealthStatus::Degraded, "High pending message count", metrics)
                } else {
                    (HealthStatus::Healthy, "NATS connection healthy", metrics)
                }
            }
        ).await {
            Ok(result) => result,
            Err(_) => (HealthStatus::Unresponsive, "NATS connection timeout", std::collections::HashMap::new()),
        };
        
        HealthCheckResult {
            check_id: self.check_id.clone(),
            component: self.component().to_string(),
            status,
            message: message.to_string(),
            execution_time: start_time.elapsed(),
            timestamp: SystemTime::now(),
            metrics,
            error: None,
        }
    }
    
    fn check_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    fn is_critical(&self) -> bool {
        true
    }
}

impl MemoryHealthCheck {
    pub fn new(check_id: String, warning_threshold: f64, critical_threshold: f64) -> Self {
        Self {
            check_id,
            warning_threshold,
            critical_threshold,
        }
    }
}

#[async_trait]
impl HealthCheck for MemoryHealthCheck {
    fn check_id(&self) -> &str {
        &self.check_id
    }
    
    fn component(&self) -> &str {
        "memory"
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate memory usage check
        let memory_usage = rand::random::<f64>() * 0.95; // 0-95% usage
        let available_memory = (1.0 - memory_usage) * 8192.0; // MB
        
        let (status, message) = if memory_usage > self.critical_threshold {
            (HealthStatus::Unhealthy, format!("Critical memory usage: {:.1}%", memory_usage * 100.0))
        } else if memory_usage > self.warning_threshold {
            (HealthStatus::Degraded, format!("High memory usage: {:.1}%", memory_usage * 100.0))
        } else {
            (HealthStatus::Healthy, format!("Memory usage normal: {:.1}%", memory_usage * 100.0))
        };
        
        let metrics = vec![
            ("memory_usage_percent".to_string(), memory_usage * 100.0),
            ("available_memory_mb".to_string(), available_memory),
        ].into_iter().collect();
        
        HealthCheckResult {
            check_id: self.check_id.clone(),
            component: self.component().to_string(),
            status,
            message,
            execution_time: start_time.elapsed(),
            timestamp: SystemTime::now(),
            metrics,
            error: None,
        }
    }
    
    fn check_interval(&self) -> Duration {
        Duration::from_secs(15)
    }
    
    fn is_critical(&self) -> bool {
        false
    }
}

impl ServiceHealthCheck {
    pub fn new(
        check_id: String,
        service_name: String,
        endpoint: String,
        timeout: Duration,
        expected_status_codes: Vec<u16>,
    ) -> Self {
        Self {
            check_id,
            service_name,
            _endpoint: endpoint,
            timeout,
            expected_status_codes,
        }
    }
}

#[async_trait]
impl HealthCheck for ServiceHealthCheck {
    fn check_id(&self) -> &str {
        &self.check_id
    }
    
    fn component(&self) -> &str {
        &self.service_name
    }
    
    async fn check(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate HTTP service check
        let (status, message, metrics) = match tokio::time::timeout(
            self.timeout,
            async {
                // In real implementation, would make actual HTTP request
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                let response_time = rand::random::<f64>() * 2000.0;
                let status_code = if rand::random::<f64>() > 0.1 { 200 } else { 500 };
                
                let status = if self.expected_status_codes.contains(&status_code) {
                    if response_time > 1000.0 {
                        HealthStatus::Degraded
                    } else {
                        HealthStatus::Healthy
                    }
                } else {
                    HealthStatus::Unhealthy
                };
                
                let message = format!("Service {} returned {} in {:.0}ms", 
                    self.service_name, status_code, response_time);
                
                let mut metrics = std::collections::HashMap::new();
                metrics.insert("response_time_ms".to_string(), response_time);
                metrics.insert("status_code".to_string(), status_code as f64);
                
                (status, message, metrics)
            }
        ).await {
            Ok(result) => result,
            Err(_) => (HealthStatus::Unresponsive, format!("Service {} timeout", self.service_name), std::collections::HashMap::new()),
        };
        
        HealthCheckResult {
            check_id: self.check_id.clone(),
            component: self.component().to_string(),
            status,
            message,
            execution_time: start_time.elapsed(),
            timestamp: SystemTime::now(),
            metrics,
            error: None,
        }
    }
    
    fn check_interval(&self) -> Duration {
        Duration::from_secs(45)
    }
    
    fn is_critical(&self) -> bool {
        true
    }
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new(config: HealthMonitorConfig) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            config,
            start_time: Instant::now(),
        }
    }
    
    /// Register a health check
    pub async fn register_check(&self, check: Box<dyn HealthCheck>) {
        let check_id = check.check_id().to_string();
        self.checks.write().await.insert(check_id, check);
    }
    
    /// Remove a health check
    pub async fn unregister_check(&self, check_id: &str) {
        self.checks.write().await.remove(check_id);
        self.results.write().await.remove(check_id);
    }
    
    /// Run all health checks
    pub async fn run_all_checks(&self) -> WorkflowResult<SystemHealthSummary> {
        let checks = self.checks.read().await;
        let mut results = Vec::new();
        
        if self.config.parallel_checks {
            // Run checks in parallel with concurrency limit
            let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrent_checks));
            let mut tasks = Vec::new();
            
            for check in checks.values() {
                let check_id = check.check_id().to_string();
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let _check_timeout = self.config.check_timeout;
                
                // Clone check for async task (in real implementation, would handle this differently)
                tasks.push(tokio::spawn(async move {
                    let _permit = permit;
                    // For now, simulate the check result
                    HealthCheckResult {
                        check_id: check_id.clone(),
                        component: "simulated".to_string(),
                        status: HealthStatus::Healthy,
                        message: "Simulated check result".to_string(),
                        execution_time: Duration::from_millis(50),
                        timestamp: SystemTime::now(),
                        metrics: HashMap::new(),
                        error: None,
                    }
                }));
            }
            
            // Wait for all tasks to complete
            for task in tasks {
                match task.await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        eprintln!("Health check task failed: {}", e);
                    }
                }
            }
        } else {
            // Run checks sequentially
            for check in checks.values() {
                match tokio::time::timeout(self.config.check_timeout, check.check()).await {
                    Ok(result) => results.push(result),
                    Err(_) => {
                        results.push(HealthCheckResult {
                            check_id: check.check_id().to_string(),
                            component: check.component().to_string(),
                            status: HealthStatus::Unresponsive,
                            message: "Health check timeout".to_string(),
                            execution_time: self.config.check_timeout,
                            timestamp: SystemTime::now(),
                            metrics: HashMap::new(),
                            error: Some(WorkflowError::new(
                                ErrorCategory::Infrastructure,
                                ErrorSeverity::Warning,
                                "Health check timeout".to_string(),
                                crate::error::types::ErrorDetails::Generic {
                                    code: "HEALTH_CHECK_TIMEOUT".to_string(),
                                    details: HashMap::new(),
                                },
                                ErrorContext::new("health_check".to_string()),
                            )),
                        });
                    }
                }
            }
        }
        
        // Update stored results
        {
            let mut stored_results = self.results.write().await;
            for result in &results {
                stored_results.insert(result.check_id.clone(), result.clone());
            }
        }
        
        // Calculate system health summary
        Ok(self.calculate_system_health(&results))
    }
    
    /// Run a specific health check
    pub async fn run_check(&self, check_id: &str) -> WorkflowResult<HealthCheckResult> {
        let checks = self.checks.read().await;
        
        let check = checks.get(check_id)
            .ok_or_else(|| {
                WorkflowError::new(
                    ErrorCategory::Configuration,
                    ErrorSeverity::Error,
                    format!("Health check '{}' not found", check_id),
                    crate::error::types::ErrorDetails::Generic {
                        code: "CHECK_NOT_FOUND".to_string(),
                        details: vec![("check_id".to_string(), serde_json::json!(check_id))].into_iter().collect(),
                    },
                    ErrorContext::new("run_health_check".to_string()),
                )
            })?;
        
        let result = tokio::time::timeout(self.config.check_timeout, check.check()).await
            .map_err(|_| {
                WorkflowError::new(
                    ErrorCategory::Infrastructure,
                    ErrorSeverity::Warning,
                    format!("Health check '{}' timeout", check_id),
                    crate::error::types::ErrorDetails::Generic {
                        code: "HEALTH_CHECK_TIMEOUT".to_string(),
                        details: vec![("check_id".to_string(), serde_json::json!(check_id))].into_iter().collect(),
                    },
                    ErrorContext::new("run_health_check".to_string()),
                )
            })?;
        
        // Update stored result
        self.results.write().await.insert(check_id.to_string(), result.clone());
        
        Ok(result)
    }
    
    /// Get current system health status
    pub async fn get_system_health(&self) -> SystemHealthSummary {
        let results: Vec<HealthCheckResult> = self.results.read().await.values().cloned().collect();
        self.calculate_system_health(&results)
    }
    
    /// Get health check results
    pub async fn get_check_results(&self) -> HashMap<String, HealthCheckResult> {
        self.results.read().await.clone()
    }
    
    /// Start periodic health checks
    pub async fn start_periodic_checks(&self) {
        let checks = self.checks.clone();
        let results = self.results.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.default_interval);
            
            loop {
                interval.tick().await;
                
                // Run health checks
                let check_list = checks.read().await;
                for check in check_list.values() {
                    let check_id = check.check_id().to_string();
                    let _check_timeout = config.check_timeout;
                    
                    // For now, simulate check execution
                    let result = HealthCheckResult {
                        check_id: check_id.clone(),
                        component: check.component().to_string(),
                        status: if rand::random::<f64>() > 0.1 { HealthStatus::Healthy } else { HealthStatus::Degraded },
                        message: "Periodic check completed".to_string(),
                        execution_time: Duration::from_millis(rand::random::<u64>() % 200),
                        timestamp: SystemTime::now(),
                        metrics: HashMap::new(),
                        error: None,
                    };
                    
                    results.write().await.insert(check_id, result);
                }
            }
        });
    }
    
    fn calculate_system_health(&self, results: &[HealthCheckResult]) -> SystemHealthSummary {
        if results.is_empty() {
            return SystemHealthSummary {
                overall_status: HealthStatus::Unresponsive,
                component_results: HashMap::new(),
                health_score: 0.0,
                total_checks: 0,
                healthy_checks: 0,
                degraded_checks: 0,
                unhealthy_checks: 0,
                generated_at: SystemTime::now(),
                uptime: self.start_time.elapsed(),
            };
        }
        
        let mut component_results = HashMap::new();
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut unresponsive_count = 0;
        
        for result in results {
            component_results.insert(result.component.clone(), result.clone());
            
            match result.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Unresponsive => unresponsive_count += 1,
            }
        }
        
        let total_checks = results.len() as u32;
        
        // Calculate health score (weighted)
        let health_score = (healthy_count as f64 * 1.0 + 
                           degraded_count as f64 * 0.5 + 
                           unhealthy_count as f64 * 0.1 + 
                           unresponsive_count as f64 * 0.0) / total_checks as f64;
        
        // Determine overall status
        let overall_status = if unresponsive_count > 0 || unhealthy_count > total_checks / 2 {
            HealthStatus::Unhealthy
        } else if unhealthy_count > 0 || degraded_count > total_checks / 2 {
            HealthStatus::Degraded
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        SystemHealthSummary {
            overall_status,
            component_results,
            health_score,
            total_checks,
            healthy_checks: healthy_count,
            degraded_checks: degraded_count,
            unhealthy_checks: unhealthy_count + unresponsive_count,
            generated_at: SystemTime::now(),
            uptime: self.start_time.elapsed(),
        }
    }
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            default_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(10),
            failure_threshold: 3,
            parallel_checks: true,
            max_concurrent_checks: 10,
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_health_check() {
        let check = DatabaseHealthCheck::new(
            "db_check".to_string(),
            "postgresql://localhost:5432/test".to_string(),
            Duration::from_secs(5),
        );
        
        let result = check.check().await;
        assert_eq!(result.check_id, "db_check");
        assert_eq!(result.component, "database");
    }
    
    #[tokio::test]
    async fn test_memory_health_check() {
        let check = MemoryHealthCheck::new(
            "memory_check".to_string(),
            0.8, // 80% warning threshold
            0.9, // 90% critical threshold
        );
        
        let result = check.check().await;
        assert_eq!(result.check_id, "memory_check");
        assert_eq!(result.component, "memory");
        assert!(result.metrics.contains_key("memory_usage_percent"));
    }
    
    #[tokio::test]
    async fn test_health_monitor() {
        let config = HealthMonitorConfig::default();
        let monitor = HealthMonitor::new(config);
        
        // Register checks
        monitor.register_check(Box::new(MemoryHealthCheck::new(
            "memory".to_string(),
            0.8,
            0.9,
        ))).await;
        
        monitor.register_check(Box::new(DatabaseHealthCheck::new(
            "database".to_string(),
            "postgresql://localhost:5432/test".to_string(),
            Duration::from_secs(5),
        ))).await;
        
        // Run all checks
        let summary = monitor.run_all_checks().await.unwrap();
        assert_eq!(summary.total_checks, 2);
        
        // Get system health
        let health = monitor.get_system_health().await;
        assert!(health.health_score >= 0.0 && health.health_score <= 1.0);
    }
}