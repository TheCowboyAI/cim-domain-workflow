//! Monitoring and Observability Framework
//!
//! Provides comprehensive monitoring, metrics collection, health checks, alerting,
//! and integration capabilities with popular observability platforms for
//! production-ready workflow orchestration systems.

pub mod metrics;
pub mod health;
pub mod alerts;
pub mod tracing;
pub mod dashboards;

pub use metrics::*;
pub use health::*;
pub use alerts::*;
pub use tracing::*;
pub use dashboards::*;

use crate::error::types::WorkflowResult;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Comprehensive observability suite
pub struct ObservabilitySuite {
    /// Metrics registry
    pub metrics: Arc<MetricsRegistry>,
    /// Health monitor
    pub health_monitor: Arc<HealthMonitor>,
    /// Alert manager
    pub alert_manager: Arc<AlertManager>,
    /// Trace provider
    pub trace_provider: Arc<TraceProvider>,
    /// Dashboard renderer
    pub dashboard_renderer: Arc<RwLock<DashboardRenderer>>,
}

impl ObservabilitySuite {
    /// Create new observability suite
    pub fn new() -> Self {
        let metrics = Arc::new(MetricsRegistry::new());
        let health_monitor = Arc::new(HealthMonitor::new(HealthMonitorConfig::default()));
        let alert_manager = Arc::new(AlertManager::new(AlertManagerConfig::default()));
        
        let resource = Resource {
            service_name: "cim-domain-workflow".to_string(),
            service_version: "1.0.0".to_string(),
            service_instance_id: uuid::Uuid::new_v4().to_string(),
            attributes: std::collections::HashMap::new(),
        };
        let sampler = Box::new(ProbabilitySampler::new(0.1));
        let trace_provider = Arc::new(TraceProvider::new(resource, sampler));
        
        let dashboard_renderer = Arc::new(RwLock::new(DashboardRenderer::new(Some((*metrics).clone()))));
        
        Self {
            metrics,
            health_monitor,
            alert_manager,
            trace_provider,
            dashboard_renderer,
        }
    }
    
    /// Initialize observability suite
    pub async fn initialize(&self) -> WorkflowResult<()> {
        // Set up default health checks
        self.setup_default_health_checks().await?;
        
        // Set up default metrics
        self.setup_default_metrics().await?;
        
        // Set up alert routing
        self.setup_default_alert_routing().await?;
        
        // Start background services
        self.start_background_services().await?;
        
        Ok(())
    }
    
    /// Set up default health checks
    async fn setup_default_health_checks(&self) -> WorkflowResult<()> {
        // Memory health check
        self.health_monitor.register_check(Box::new(MemoryHealthCheck::new(
            "memory_usage".to_string(),
            0.8, // 80% warning
            0.9, // 90% critical
        ))).await;
        
        // Database health check (if applicable)
        self.health_monitor.register_check(Box::new(DatabaseHealthCheck::new(
            "database_connectivity".to_string(),
            "postgresql://localhost:5432/workflow".to_string(),
            std::time::Duration::from_secs(5),
        ))).await;
        
        // NATS health check (if applicable)
        self.health_monitor.register_check(Box::new(NatsHealthCheck::new(
            "nats_connectivity".to_string(),
            vec!["nats://localhost:4222".to_string()],
            std::time::Duration::from_secs(3),
        ))).await;
        
        Ok(())
    }
    
    /// Set up default metrics
    async fn setup_default_metrics(&self) -> WorkflowResult<()> {
        // Create workflow metrics
        let _workflow_metrics = WorkflowMetrics::new(&self.metrics).await;
        
        // Add console exporter for development
        let mut metrics = (*self.metrics).clone();
        metrics.add_exporter(Box::new(ConsoleExporter::new(false)));
        
        Ok(())
    }
    
    /// Set up default alert routing
    async fn setup_default_alert_routing(&self) -> WorkflowResult<()> {
        // Register console notification channel
        let console_config = NotificationChannelConfig {
            channel_id: "console".to_string(),
            channel_type: "console".to_string(),
            settings: std::collections::HashMap::new(),
            retry_config: RetryConfig::default(),
            rate_limit: None,
        };
        
        self.alert_manager.register_channel(Box::new(ConsoleNotificationChannel::new(
            console_config, 
            true // Use colors
        ))).await;
        
        Ok(())
    }
    
    /// Start background services
    async fn start_background_services(&self) -> WorkflowResult<()> {
        // Start periodic health checks
        self.health_monitor.start_periodic_checks().await;
        
        // Start alert manager
        self.alert_manager.start().await;
        
        // Start metrics export
        self.metrics.start_periodic_export(std::time::Duration::from_secs(30)).await;
        
        Ok(())
    }
    
    /// Get system overview
    pub async fn get_system_overview(&self) -> SystemOverview {
        let health_summary = self.health_monitor.get_system_health().await;
        let active_alerts = self.alert_manager.get_active_alerts().await;
        let metrics = self.metrics.collect_metrics().await;
        
        SystemOverview {
            health: health_summary,
            active_alerts: active_alerts.len(),
            total_metrics: metrics.len(),
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default(),
        }
    }
}

/// System overview information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemOverview {
    /// Health summary
    pub health: SystemHealthSummary,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Total number of metrics
    pub total_metrics: usize,
    /// System uptime
    pub uptime: std::time::Duration,
}

impl Default for ObservabilitySuite {
    fn default() -> Self {
        Self::new()
    }
}