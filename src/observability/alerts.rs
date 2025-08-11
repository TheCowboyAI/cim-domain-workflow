//! Alerting and Notification System
//!
//! Provides comprehensive alerting capabilities with support for multiple
//! notification channels, alert routing, escalation, and alert management.

use crate::error::types::{WorkflowError, WorkflowResult, ErrorCategory, ErrorSeverity, ErrorContext};
use crate::observability::health::{HealthStatus, HealthCheckResult, SystemHealthSummary};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alerts
    Info,
    /// Low priority alerts
    Low,
    /// Medium priority alerts
    Medium,
    /// High priority alerts
    High,
    /// Critical alerts requiring immediate attention
    Critical,
    /// Emergency alerts for system-wide failures
    Emergency,
}

/// Alert status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is active and unacknowledged
    Active,
    /// Alert has been acknowledged but not resolved
    Acknowledged,
    /// Alert has been resolved
    Resolved,
    /// Alert has been suppressed
    Suppressed,
    /// Alert has expired
    Expired,
}

/// Alert source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSource {
    /// Alert from health check
    HealthCheck {
        check_id: String,
        component: String,
    },
    /// Alert from error handling system
    Error {
        error_id: Uuid,
        category: ErrorCategory,
    },
    /// Alert from metrics threshold
    Metric {
        metric_name: String,
        threshold: f64,
        actual_value: f64,
    },
    /// Manual alert
    Manual {
        created_by: String,
        reason: String,
    },
    /// Alert from external system
    External {
        system: String,
        external_id: String,
    },
}

/// Alert definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert identifier
    pub alert_id: Uuid,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert status
    pub status: AlertStatus,
    /// Alert source
    pub source: AlertSource,
    /// When alert was created
    pub created_at: SystemTime,
    /// When alert was last updated
    pub updated_at: SystemTime,
    /// When alert was acknowledged
    pub acknowledged_at: Option<SystemTime>,
    /// Who acknowledged the alert
    pub acknowledged_by: Option<String>,
    /// When alert was resolved
    pub resolved_at: Option<SystemTime>,
    /// Resolution notes
    pub resolution_notes: Option<String>,
    /// Alert metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Alert labels for routing
    pub labels: HashMap<String, String>,
    /// Number of times this alert has fired
    pub fire_count: u32,
    /// Alert expiry time
    pub expires_at: Option<SystemTime>,
}

/// Alert rule for automated alert generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule identifier
    pub rule_id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule condition
    pub condition: AlertCondition,
    /// Alert severity to generate
    pub severity: AlertSeverity,
    /// Alert title template
    pub title_template: String,
    /// Alert description template
    pub description_template: String,
    /// Evaluation interval
    pub evaluation_interval: Duration,
    /// Rule labels
    pub labels: HashMap<String, String>,
    /// Whether rule is enabled
    pub enabled: bool,
}

/// Alert rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Health check condition
    HealthCheck {
        component: String,
        status: HealthStatus,
    },
    /// Error rate condition
    ErrorRate {
        category: Option<ErrorCategory>,
        threshold: f64,
        time_window: Duration,
    },
    /// Metric threshold condition
    MetricThreshold {
        metric_name: String,
        operator: ThresholdOperator,
        threshold: f64,
    },
    /// Custom condition
    Custom {
        expression: String,
    },
}

/// Threshold comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Notification channel trait
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Channel identifier
    fn channel_id(&self) -> &str;
    
    /// Channel type name
    fn channel_type(&self) -> &str;
    
    /// Send notification
    async fn send_notification(&self, alert: &Alert) -> WorkflowResult<NotificationResult>;
    
    /// Check if channel is healthy
    async fn health_check(&self) -> bool;
    
    /// Get channel configuration
    fn get_config(&self) -> &NotificationChannelConfig;
}

/// Notification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    /// Whether notification was sent successfully
    pub success: bool,
    /// Notification ID from the channel
    pub notification_id: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Response time
    pub response_time: Duration,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelConfig {
    /// Channel identifier
    pub channel_id: String,
    /// Channel type
    pub channel_type: String,
    /// Channel settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Rate limiting
    pub rate_limit: Option<RateLimit>,
}

/// Retry configuration for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum notifications per time window
    pub max_notifications: u32,
    /// Time window for rate limiting
    pub time_window: Duration,
}

/// Email notification channel
pub struct EmailNotificationChannel {
    config: NotificationChannelConfig,
    smtp_settings: EmailSettings,
}

/// Email settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSettings {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub use_tls: bool,
}

/// Slack notification channel
pub struct SlackNotificationChannel {
    config: NotificationChannelConfig,
    webhook_url: String,
    channel: String,
    username: String,
}

/// Webhook notification channel
pub struct WebhookNotificationChannel {
    config: NotificationChannelConfig,
    webhook_url: String,
    headers: HashMap<String, String>,
    timeout: Duration,
}

/// Console notification channel for development
pub struct ConsoleNotificationChannel {
    config: NotificationChannelConfig,
    use_colors: bool,
}

/// Alert manager that orchestrates alerting
pub struct AlertManager {
    /// Active alerts
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    /// Alert rules
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    /// Notification channels
    channels: Arc<RwLock<HashMap<String, Box<dyn NotificationChannel>>>>,
    /// Alert routing configuration
    routing: Arc<RwLock<AlertRouting>>,
    /// Alert history
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    /// Configuration
    config: AlertManagerConfig,
}

/// Alert routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRouting {
    /// Default channels to use
    pub default_channels: Vec<String>,
    /// Routing rules based on labels
    pub label_routes: HashMap<String, Vec<String>>,
    /// Severity-based routing
    pub severity_routes: HashMap<AlertSeverity, Vec<String>>,
    /// Component-based routing
    pub component_routes: HashMap<String, Vec<String>>,
}

/// Alert manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    /// Maximum number of active alerts
    pub max_active_alerts: usize,
    /// History retention period
    pub history_retention: Duration,
    /// Default alert expiry time
    pub default_expiry: Duration,
    /// Notification batch size
    pub notification_batch_size: usize,
    /// Alert grouping interval
    pub grouping_interval: Duration,
}

impl EmailNotificationChannel {
    pub fn new(config: NotificationChannelConfig, smtp_settings: EmailSettings) -> Self {
        Self {
            config,
            smtp_settings,
        }
    }
}

#[async_trait]
impl NotificationChannel for EmailNotificationChannel {
    fn channel_id(&self) -> &str {
        &self.config.channel_id
    }
    
    fn channel_type(&self) -> &str {
        "email"
    }
    
    async fn send_notification(&self, alert: &Alert) -> WorkflowResult<NotificationResult> {
        let start_time = std::time::Instant::now();
        
        // Simulate email sending
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // In real implementation, would use SMTP client
        let success = rand::random::<f64>() > 0.05; // 95% success rate
        
        if success {
            Ok(NotificationResult {
                success: true,
                notification_id: Some(format!("email_{}", Uuid::new_v4())),
                error_message: None,
                response_time: start_time.elapsed(),
            })
        } else {
            Ok(NotificationResult {
                success: false,
                notification_id: None,
                error_message: Some("SMTP server unavailable".to_string()),
                response_time: start_time.elapsed(),
            })
        }
    }
    
    async fn health_check(&self) -> bool {
        // Simulate health check
        rand::random::<f64>() > 0.1
    }
    
    fn get_config(&self) -> &NotificationChannelConfig {
        &self.config
    }
}

impl SlackNotificationChannel {
    pub fn new(config: NotificationChannelConfig, webhook_url: String, channel: String, username: String) -> Self {
        Self {
            config,
            webhook_url,
            channel,
            username,
        }
    }
}

#[async_trait]
impl NotificationChannel for SlackNotificationChannel {
    fn channel_id(&self) -> &str {
        &self.config.channel_id
    }
    
    fn channel_type(&self) -> &str {
        "slack"
    }
    
    async fn send_notification(&self, alert: &Alert) -> WorkflowResult<NotificationResult> {
        let start_time = std::time::Instant::now();
        
        // Simulate Slack webhook call
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        let success = rand::random::<f64>() > 0.02; // 98% success rate
        
        if success {
            Ok(NotificationResult {
                success: true,
                notification_id: Some(format!("slack_{}", Uuid::new_v4())),
                error_message: None,
                response_time: start_time.elapsed(),
            })
        } else {
            Ok(NotificationResult {
                success: false,
                notification_id: None,
                error_message: Some("Slack webhook failed".to_string()),
                response_time: start_time.elapsed(),
            })
        }
    }
    
    async fn health_check(&self) -> bool {
        rand::random::<f64>() > 0.05
    }
    
    fn get_config(&self) -> &NotificationChannelConfig {
        &self.config
    }
}

impl ConsoleNotificationChannel {
    pub fn new(config: NotificationChannelConfig, use_colors: bool) -> Self {
        Self {
            config,
            use_colors,
        }
    }
}

#[async_trait]
impl NotificationChannel for ConsoleNotificationChannel {
    fn channel_id(&self) -> &str {
        &self.config.channel_id
    }
    
    fn channel_type(&self) -> &str {
        "console"
    }
    
    async fn send_notification(&self, alert: &Alert) -> WorkflowResult<NotificationResult> {
        let start_time = std::time::Instant::now();
        
        // Print alert to console with colors if enabled
        if self.use_colors {
            let color = match alert.severity {
                AlertSeverity::Emergency | AlertSeverity::Critical => "\x1b[31m", // Red
                AlertSeverity::High => "\x1b[33m", // Yellow
                AlertSeverity::Medium => "\x1b[36m", // Cyan
                AlertSeverity::Low => "\x1b[32m", // Green
                AlertSeverity::Info => "\x1b[37m", // White
            };
            let reset = "\x1b[0m";
            
            println!("{}🚨 ALERT [{}] {}: {}{}", 
                color, 
                alert.severity.to_string().to_uppercase(), 
                alert.title, 
                alert.description,
                reset
            );
        } else {
            println!("🚨 ALERT [{}] {}: {}", 
                alert.severity.to_string().to_uppercase(), 
                alert.title, 
                alert.description
            );
        }
        
        Ok(NotificationResult {
            success: true,
            notification_id: Some(format!("console_{}", alert.alert_id)),
            error_message: None,
            response_time: start_time.elapsed(),
        })
    }
    
    async fn health_check(&self) -> bool {
        true // Console is always available
    }
    
    fn get_config(&self) -> &NotificationChannelConfig {
        &self.config
    }
}

impl AlertManager {
    /// Create new alert manager
    pub fn new(config: AlertManagerConfig) -> Self {
        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            routing: Arc::new(RwLock::new(AlertRouting::default())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            config,
        }
    }
    
    /// Register notification channel
    pub async fn register_channel(&self, channel: Box<dyn NotificationChannel>) {
        let channel_id = channel.channel_id().to_string();
        self.channels.write().await.insert(channel_id, channel);
    }
    
    /// Add alert rule
    pub async fn add_rule(&self, rule: AlertRule) {
        let rule_id = rule.rule_id.clone();
        self.rules.write().await.insert(rule_id, rule);
    }
    
    /// Remove alert rule
    pub async fn remove_rule(&self, rule_id: &str) {
        self.rules.write().await.remove(rule_id);
    }
    
    /// Fire alert manually
    pub async fn fire_alert(
        &self,
        title: String,
        description: String,
        severity: AlertSeverity,
        source: AlertSource,
        labels: HashMap<String, String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<Uuid> {
        let alert = Alert {
            alert_id: Uuid::new_v4(),
            title,
            description,
            severity,
            status: AlertStatus::Active,
            source,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            resolution_notes: None,
            metadata,
            labels,
            fire_count: 1,
            expires_at: Some(SystemTime::now() + self.config.default_expiry),
        };
        
        let alert_id = alert.alert_id;
        
        // Store alert
        self.alerts.write().await.insert(alert_id, alert.clone());
        
        // Send notifications
        self.send_alert_notifications(&alert).await?;
        
        Ok(alert_id)
    }
    
    /// Fire alert from health check
    pub async fn fire_health_alert(&self, health_result: &HealthCheckResult) -> WorkflowResult<Option<Uuid>> {
        // Only create alerts for degraded/unhealthy/unresponsive status
        if matches!(health_result.status, HealthStatus::Healthy) {
            return Ok(None);
        }
        
        let severity = match health_result.status {
            HealthStatus::Unresponsive => AlertSeverity::Critical,
            HealthStatus::Unhealthy => AlertSeverity::High,
            HealthStatus::Degraded => AlertSeverity::Medium,
            HealthStatus::Healthy => return Ok(None),
        };
        
        let title = format!("Health Check Failed: {}", health_result.component);
        let description = format!(
            "Health check '{}' for component '{}' is {}: {}",
            health_result.check_id,
            health_result.component,
            health_result.status.to_string().to_lowercase(),
            health_result.message
        );
        
        let source = AlertSource::HealthCheck {
            check_id: health_result.check_id.clone(),
            component: health_result.component.clone(),
        };
        
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), health_result.component.clone());
        labels.insert("check_id".to_string(), health_result.check_id.clone());
        labels.insert("status".to_string(), health_result.status.to_string());
        
        let mut metadata = HashMap::new();
        metadata.insert("execution_time_ms".to_string(), 
            serde_json::json!(health_result.execution_time.as_millis()));
        metadata.insert("metrics".to_string(), serde_json::json!(health_result.metrics));
        
        let alert_id = self.fire_alert(title, description, severity, source, labels, metadata).await?;
        Ok(Some(alert_id))
    }
    
    /// Fire alert from error
    pub async fn fire_error_alert(&self, error: &WorkflowError) -> WorkflowResult<Uuid> {
        let severity = match error.severity {
            ErrorSeverity::Fatal => AlertSeverity::Emergency,
            ErrorSeverity::Critical => AlertSeverity::Critical,
            ErrorSeverity::Error => AlertSeverity::High,
            ErrorSeverity::Warning => AlertSeverity::Medium,
            ErrorSeverity::Info => AlertSeverity::Low,
        };
        
        let title = format!("Error Alert: {}", error.category.to_string());
        let description = error.message.clone();
        
        let source = AlertSource::Error {
            error_id: error.error_id,
            category: error.category.clone(),
        };
        
        let mut labels = HashMap::new();
        labels.insert("error_category".to_string(), error.category.to_string());
        labels.insert("error_severity".to_string(), error.severity.to_string());
        if let Some(ref domain) = error.context.domain {
            labels.insert("domain".to_string(), domain.clone());
        }
        if let Some(ref workflow_id) = error.context.workflow_id {
            labels.insert("workflow_id".to_string(), workflow_id.to_string());
        }
        
        let mut metadata = HashMap::new();
        metadata.insert("error_id".to_string(), serde_json::json!(error.error_id));
        metadata.insert("timestamp".to_string(), serde_json::json!(error.timestamp));
        metadata.insert("context".to_string(), serde_json::json!(error.context));
        
        self.fire_alert(title, description, severity, source, labels, metadata).await
    }
    
    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid, acknowledged_by: String) -> WorkflowResult<()> {
        let mut alerts = self.alerts.write().await;
        
        let alert = alerts.get_mut(&alert_id)
            .ok_or_else(|| {
                WorkflowError::new(
                    ErrorCategory::Configuration,
                    ErrorSeverity::Error,
                    format!("Alert {} not found", alert_id),
                    crate::error::types::ErrorDetails::Generic {
                        code: "ALERT_NOT_FOUND".to_string(),
                        details: vec![("alert_id".to_string(), serde_json::json!(alert_id))].into_iter().collect(),
                    },
                    ErrorContext::new("acknowledge_alert".to_string()),
                )
            })?;
        
        if alert.status == AlertStatus::Active {
            alert.status = AlertStatus::Acknowledged;
            alert.acknowledged_at = Some(SystemTime::now());
            alert.acknowledged_by = Some(acknowledged_by);
            alert.updated_at = SystemTime::now();
        }
        
        Ok(())
    }
    
    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: Uuid, resolution_notes: Option<String>) -> WorkflowResult<()> {
        let mut alerts = self.alerts.write().await;
        
        let alert = alerts.get_mut(&alert_id)
            .ok_or_else(|| {
                WorkflowError::new(
                    ErrorCategory::Configuration,
                    ErrorSeverity::Error,
                    format!("Alert {} not found", alert_id),
                    crate::error::types::ErrorDetails::Generic {
                        code: "ALERT_NOT_FOUND".to_string(),
                        details: vec![("alert_id".to_string(), serde_json::json!(alert_id))].into_iter().collect(),
                    },
                    ErrorContext::new("resolve_alert".to_string()),
                )
            })?;
        
        alert.status = AlertStatus::Resolved;
        alert.resolved_at = Some(SystemTime::now());
        alert.resolution_notes = resolution_notes;
        alert.updated_at = SystemTime::now();
        
        // Move to history
        let mut history = self.alert_history.write().await;
        history.push_back(alert.clone());
        
        // Remove from active alerts
        drop(alerts);
        self.alerts.write().await.remove(&alert_id);
        
        Ok(())
    }
    
    /// Get active alerts
    pub async fn get_active_alerts(&self) -> HashMap<Uuid, Alert> {
        self.alerts.read().await.clone()
    }
    
    /// Get alert history
    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<Alert> {
        let history = self.alert_history.read().await;
        let alerts: Vec<Alert> = history.iter().cloned().collect();
        
        match limit {
            Some(n) => alerts.into_iter().rev().take(n).collect(),
            None => alerts.into_iter().rev().collect(),
        }
    }
    
    /// Send notifications for alert
    async fn send_alert_notifications(&self, alert: &Alert) -> WorkflowResult<()> {
        let channels_to_notify = self.determine_notification_channels(alert).await;
        let channels = self.channels.read().await;
        
        for channel_id in channels_to_notify {
            if let Some(channel) = channels.get(&channel_id) {
                if let Err(e) = channel.send_notification(alert).await {
                    eprintln!("Failed to send alert notification via {}: {}", channel_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Determine which channels to use for alert
    async fn determine_notification_channels(&self, alert: &Alert) -> Vec<String> {
        let routing = self.routing.read().await;
        let mut channels = Vec::new();
        
        // Check severity-based routing
        if let Some(severity_channels) = routing.severity_routes.get(&alert.severity) {
            channels.extend(severity_channels.clone());
        }
        
        // Check label-based routing
        for (label_key, label_value) in &alert.labels {
            let route_key = format!("{}:{}", label_key, label_value);
            if let Some(label_channels) = routing.label_routes.get(&route_key) {
                channels.extend(label_channels.clone());
            }
        }
        
        // Check component-based routing
        if let AlertSource::HealthCheck { component, .. } = &alert.source {
            if let Some(component_channels) = routing.component_routes.get(component) {
                channels.extend(component_channels.clone());
            }
        }
        
        // Use default channels if no specific routing found
        if channels.is_empty() {
            channels.extend(routing.default_channels.clone());
        }
        
        // Remove duplicates
        channels.sort();
        channels.dedup();
        
        channels
    }
    
    /// Start alert manager background tasks
    pub async fn start(&self) {
        self.start_alert_cleanup().await;
        self.start_rule_evaluation().await;
    }
    
    /// Start alert cleanup task
    async fn start_alert_cleanup(&self) {
        let alerts = self.alerts.clone();
        let alert_history = self.alert_history.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let now = SystemTime::now();
                
                // Clean up expired alerts
                let mut active_alerts = alerts.write().await;
                let mut expired_alerts = Vec::new();
                
                active_alerts.retain(|&id, alert| {
                    if let Some(expires_at) = alert.expires_at {
                        if now > expires_at {
                            let mut expired_alert = alert.clone();
                            expired_alert.status = AlertStatus::Expired;
                            expired_alerts.push(expired_alert);
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                });
                
                // Move expired alerts to history
                let mut history = alert_history.write().await;
                for expired_alert in expired_alerts {
                    history.push_back(expired_alert);
                }
                
                // Clean up old history
                let cutoff_time = now - config.history_retention;
                while let Some(front) = history.front() {
                    if front.created_at < cutoff_time {
                        history.pop_front();
                    } else {
                        break;
                    }
                }
            }
        });
    }
    
    /// Start rule evaluation task
    async fn start_rule_evaluation(&self) {
        let rules = self.rules.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Evaluate alert rules
                let rule_list = rules.read().await;
                for rule in rule_list.values() {
                    if rule.enabled {
                        // In real implementation, would evaluate rule conditions
                        // For now, just simulate rule evaluation
                        println!("Evaluating alert rule: {}", rule.name);
                    }
                }
            }
        });
    }
}

impl Default for AlertRouting {
    fn default() -> Self {
        Self {
            default_channels: vec!["console".to_string()],
            label_routes: HashMap::new(),
            severity_routes: HashMap::new(),
            component_routes: HashMap::new(),
        }
    }
}

impl Default for AlertManagerConfig {
    fn default() -> Self {
        Self {
            max_active_alerts: 1000,
            history_retention: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            default_expiry: Duration::from_secs(24 * 60 * 60), // 24 hours
            notification_batch_size: 10,
            grouping_interval: Duration::from_secs(60),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Low => "LOW",
            AlertSeverity::Medium => "MEDIUM",
            AlertSeverity::High => "HIGH",
            AlertSeverity::Critical => "CRITICAL",
            AlertSeverity::Emergency => "EMERGENCY",
        };
        write!(f, "{}", name)
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            HealthStatus::Healthy => "HEALTHY",
            HealthStatus::Degraded => "DEGRADED",
            HealthStatus::Unhealthy => "UNHEALTHY",
            HealthStatus::Unresponsive => "UNRESPONSIVE",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_console_notification_channel() {
        let config = NotificationChannelConfig {
            channel_id: "console".to_string(),
            channel_type: "console".to_string(),
            settings: HashMap::new(),
            retry_config: RetryConfig::default(),
            rate_limit: None,
        };
        
        let channel = ConsoleNotificationChannel::new(config, true);
        
        let alert = Alert {
            alert_id: Uuid::new_v4(),
            title: "Test Alert".to_string(),
            description: "This is a test alert".to_string(),
            severity: AlertSeverity::High,
            status: AlertStatus::Active,
            source: AlertSource::Manual {
                created_by: "test".to_string(),
                reason: "testing".to_string(),
            },
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            resolution_notes: None,
            metadata: HashMap::new(),
            labels: HashMap::new(),
            fire_count: 1,
            expires_at: None,
        };
        
        let result = channel.send_notification(&alert).await.unwrap();
        assert!(result.success);
    }
    
    #[tokio::test]
    async fn test_alert_manager() {
        let config = AlertManagerConfig::default();
        let alert_manager = AlertManager::new(config);
        
        // Register console channel
        let channel_config = NotificationChannelConfig {
            channel_id: "console".to_string(),
            channel_type: "console".to_string(),
            settings: HashMap::new(),
            retry_config: RetryConfig::default(),
            rate_limit: None,
        };
        
        alert_manager.register_channel(
            Box::new(ConsoleNotificationChannel::new(channel_config, false))
        ).await;
        
        // Fire an alert
        let alert_id = alert_manager.fire_alert(
            "Test Alert".to_string(),
            "This is a test alert".to_string(),
            AlertSeverity::Medium,
            AlertSource::Manual {
                created_by: "test".to_string(),
                reason: "testing".to_string(),
            },
            HashMap::new(),
            HashMap::new(),
        ).await.unwrap();
        
        // Check that alert exists
        let active_alerts = alert_manager.get_active_alerts().await;
        assert!(active_alerts.contains_key(&alert_id));
        
        // Acknowledge alert
        alert_manager.acknowledge_alert(alert_id, "test_user".to_string()).await.unwrap();
        
        // Resolve alert
        alert_manager.resolve_alert(alert_id, Some("Resolved for testing".to_string())).await.unwrap();
        
        // Check alert is no longer active
        let active_alerts = alert_manager.get_active_alerts().await;
        assert!(!active_alerts.contains_key(&alert_id));
        
        // Check alert is in history
        let history = alert_manager.get_alert_history(Some(10)).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].alert_id, alert_id);
        assert_eq!(history[0].status, AlertStatus::Resolved);
    }
}