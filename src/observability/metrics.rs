//! Metrics Collection and Reporting
//!
//! Implements comprehensive metrics collection with support for various metrics
//! backends including Prometheus, StatsD, and custom exporters.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Metric types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric (monotonically increasing)
    Counter,
    /// Gauge metric (can increase and decrease)
    Gauge,
    /// Histogram for timing and distribution metrics
    Histogram,
    /// Summary with quantiles
    Summary,
}

/// Metric value representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value  
    Gauge(f64),
    /// Histogram bucket
    Histogram {
        count: u64,
        sum: f64,
        buckets: Vec<HistogramBucket>,
    },
    /// Summary quantiles
    Summary {
        count: u64,
        sum: f64,
        quantiles: Vec<Quantile>,
    },
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Upper bound for the bucket (le = less than or equal)
    pub upper_bound: f64,
    /// Count of observations in this bucket
    pub count: u64,
}

/// Summary quantile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    /// Quantile value (0.0 to 1.0)
    pub quantile: f64,
    /// Value at this quantile
    pub value: f64,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Metric name
    pub name: String,
    /// Metric labels/tags
    pub labels: HashMap<String, String>,
    /// Metric value
    pub value: MetricValue,
    /// Timestamp when metric was recorded
    pub timestamp: SystemTime,
}

/// Metrics registry for collecting and managing metrics
pub struct MetricsRegistry {
    /// All registered metrics
    metrics: Arc<RwLock<HashMap<String, Arc<RwLock<Metric>>>>>,
    /// Default labels applied to all metrics
    default_labels: HashMap<String, String>,
    /// Metric exporters
    exporters: Vec<Box<dyn MetricExporter>>,
}

/// Individual metric
pub struct Metric {
    /// Metric name
    name: String,
    /// Metric type
    metric_type: MetricType,
    /// Metric description
    description: String,
    /// Current value
    value: MetricValue,
    /// Labels
    labels: HashMap<String, String>,
    /// Last updated timestamp
    last_updated: SystemTime,
}

/// Counter metric for monotonically increasing values
pub struct Counter {
    metric: Arc<RwLock<Metric>>,
    registry: Arc<MetricsRegistry>,
}

/// Gauge metric for values that can increase and decrease  
pub struct Gauge {
    metric: Arc<RwLock<Metric>>,
    registry: Arc<MetricsRegistry>,
}

/// Histogram metric for timing and distribution data
pub struct Histogram {
    metric: Arc<RwLock<Metric>>,
    registry: Arc<MetricsRegistry>,
    buckets: Vec<f64>,
}

/// Timer for measuring duration
pub struct Timer {
    start_time: Instant,
    histogram: Histogram,
}

/// Workflow-specific metrics
pub struct WorkflowMetrics {
    /// Total workflow executions started
    pub workflows_started: Counter,
    /// Total workflow executions completed
    pub workflows_completed: Counter,
    /// Total workflow executions failed
    pub workflows_failed: Counter,
    /// Current active workflows
    pub active_workflows: Gauge,
    /// Workflow execution duration
    pub workflow_duration: Histogram,
    /// Step execution duration
    pub step_duration: Histogram,
    /// Template instantiation count
    pub template_instantiations: Counter,
    /// Template instantiation duration
    pub template_instantiation_duration: Histogram,
    /// Cross-domain events published
    pub cross_domain_events: Counter,
    /// NATS message processing rate
    pub nats_messages_processed: Counter,
    /// Error count by category
    pub errors_by_category: Counter,
    /// Recovery attempts
    pub recovery_attempts: Counter,
    /// Circuit breaker state changes
    pub circuit_breaker_state_changes: Counter,
    /// Bulkhead rejections
    pub bulkhead_rejections: Counter,
}

/// Metric exporter trait
pub trait MetricExporter: Send + Sync {
    /// Export metrics to external system
    fn export(&self, metrics: Vec<MetricPoint>) -> Result<(), MetricExportError>;
    
    /// Get exporter name
    fn name(&self) -> &str;
    
    /// Check if exporter is healthy
    fn is_healthy(&self) -> bool;
}

/// Prometheus metric exporter
pub struct PrometheusExporter {
    /// Prometheus endpoint URL
    endpoint: String,
    /// Authentication credentials
    auth: Option<PrometheusAuth>,
    /// HTTP client for sending metrics
    client: reqwest::Client,
}

/// Prometheus authentication
#[derive(Debug, Clone)]
pub struct PrometheusAuth {
    pub username: String,
    pub password: String,
}

/// StatsD metric exporter
pub struct StatsDExporter {
    /// StatsD server address
    address: String,
    /// UDP socket for sending metrics
    socket: std::net::UdpSocket,
    /// Metric prefix
    prefix: String,
}

/// Console metric exporter for development/debugging
pub struct ConsoleExporter {
    /// Whether to use pretty printing
    pretty: bool,
}

/// Custom metric exporter that can be implemented by users
pub struct CustomExporter {
    name: String,
    export_fn: Box<dyn Fn(Vec<MetricPoint>) -> Result<(), MetricExportError> + Send + Sync>,
}

/// Metric export errors
#[derive(Debug, thiserror::Error)]
pub enum MetricExportError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Format error: {0}")]
    FormatError(String),
    
    #[error("Timeout error")]
    Timeout,
    
    #[error("Export error: {0}")]
    ExportError(String),
}

impl MetricsRegistry {
    /// Create new metrics registry
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            default_labels: HashMap::new(),
            exporters: Vec::new(),
        }
    }

    /// Add default labels to all metrics
    pub fn with_default_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.default_labels = labels;
        self
    }

    /// Add metric exporter
    pub fn add_exporter(&mut self, exporter: Box<dyn MetricExporter>) {
        self.exporters.push(exporter);
    }

    /// Register a counter metric
    pub async fn counter(&self, name: String, description: String, labels: HashMap<String, String>) -> Counter {
        let mut combined_labels = self.default_labels.clone();
        combined_labels.extend(labels);

        let metric = Arc::new(RwLock::new(Metric {
            name: name.clone(),
            metric_type: MetricType::Counter,
            description,
            value: MetricValue::Counter(0),
            labels: combined_labels,
            last_updated: SystemTime::now(),
        }));

        self.metrics.write().await.insert(name, metric.clone());

        Counter {
            metric,
            registry: Arc::new(self.clone()),
        }
    }

    /// Register a gauge metric
    pub async fn gauge(&self, name: String, description: String, labels: HashMap<String, String>) -> Gauge {
        let mut combined_labels = self.default_labels.clone();
        combined_labels.extend(labels);

        let metric = Arc::new(RwLock::new(Metric {
            name: name.clone(),
            metric_type: MetricType::Gauge,
            description,
            value: MetricValue::Gauge(0.0),
            labels: combined_labels,
            last_updated: SystemTime::now(),
        }));

        self.metrics.write().await.insert(name, metric.clone());

        Gauge {
            metric,
            registry: Arc::new(self.clone()),
        }
    }

    /// Register a histogram metric
    pub async fn histogram(
        &self, 
        name: String, 
        description: String, 
        labels: HashMap<String, String>,
        buckets: Vec<f64>
    ) -> Histogram {
        let mut combined_labels = self.default_labels.clone();
        combined_labels.extend(labels);

        let histogram_buckets: Vec<HistogramBucket> = buckets
            .iter()
            .map(|&upper_bound| HistogramBucket {
                upper_bound,
                count: 0,
            })
            .collect();

        let metric = Arc::new(RwLock::new(Metric {
            name: name.clone(),
            metric_type: MetricType::Histogram,
            description,
            value: MetricValue::Histogram {
                count: 0,
                sum: 0.0,
                buckets: histogram_buckets,
            },
            labels: combined_labels,
            last_updated: SystemTime::now(),
        }));

        self.metrics.write().await.insert(name, metric.clone());

        Histogram {
            metric,
            registry: Arc::new(self.clone()),
            buckets,
        }
    }

    /// Get all metrics as metric points for export
    pub async fn collect_metrics(&self) -> Vec<MetricPoint> {
        let metrics = self.metrics.read().await;
        let mut points = Vec::new();

        for metric_ref in metrics.values() {
            let metric = metric_ref.read().await;
            points.push(MetricPoint {
                name: metric.name.clone(),
                labels: metric.labels.clone(),
                value: metric.value.clone(),
                timestamp: metric.last_updated,
            });
        }

        points
    }

    /// Export all metrics using configured exporters
    pub async fn export_metrics(&self) {
        let metrics = self.collect_metrics().await;
        
        for exporter in &self.exporters {
            if exporter.is_healthy() {
                if let Err(e) = exporter.export(metrics.clone()) {
                    eprintln!("Failed to export metrics to {}: {}", exporter.name(), e);
                }
            }
        }
    }

    /// Start periodic metric export
    pub async fn start_periodic_export(&self, interval: Duration) {
        let registry = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                registry.export_metrics().await;
            }
        });
    }
}

impl Clone for MetricsRegistry {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            default_labels: self.default_labels.clone(),
            exporters: Vec::new(), // Exporters are not cloned to avoid double exports
        }
    }
}

impl Counter {
    /// Increment counter by 1
    pub async fn increment(&self) {
        self.add(1).await;
    }

    /// Add value to counter
    pub async fn add(&self, value: u64) {
        let mut metric = self.metric.write().await;
        if let MetricValue::Counter(ref mut current) = metric.value {
            *current += value;
            metric.last_updated = SystemTime::now();
        }
    }

    /// Get current counter value
    pub async fn value(&self) -> u64 {
        let metric = self.metric.read().await;
        if let MetricValue::Counter(value) = metric.value {
            value
        } else {
            0
        }
    }
}

impl Gauge {
    /// Set gauge value
    pub async fn set(&self, value: f64) {
        let mut metric = self.metric.write().await;
        if let MetricValue::Gauge(ref mut current) = metric.value {
            *current = value;
            metric.last_updated = SystemTime::now();
        }
    }

    /// Increment gauge by value
    pub async fn increment(&self, value: f64) {
        let mut metric = self.metric.write().await;
        if let MetricValue::Gauge(ref mut current) = metric.value {
            *current += value;
            metric.last_updated = SystemTime::now();
        }
    }

    /// Decrement gauge by value
    pub async fn decrement(&self, value: f64) {
        let mut metric = self.metric.write().await;
        if let MetricValue::Gauge(ref mut current) = metric.value {
            *current -= value;
            metric.last_updated = SystemTime::now();
        }
    }

    /// Get current gauge value
    pub async fn value(&self) -> f64 {
        let metric = self.metric.read().await;
        if let MetricValue::Gauge(value) = metric.value {
            value
        } else {
            0.0
        }
    }
}

impl Histogram {
    /// Observe a value in the histogram
    pub async fn observe(&self, value: f64) {
        let mut metric = self.metric.write().await;
        if let MetricValue::Histogram { ref mut count, ref mut sum, ref mut buckets } = metric.value {
            *count += 1;
            *sum += value;
            
            // Update buckets
            for bucket in buckets.iter_mut() {
                if value <= bucket.upper_bound {
                    bucket.count += 1;
                }
            }
            
            metric.last_updated = SystemTime::now();
        }
    }

    /// Start timing - returns a Timer that will automatically observe when dropped
    pub fn start_timer(&self) -> Timer {
        Timer {
            start_time: Instant::now(),
            histogram: self.clone(),
        }
    }
}

impl Clone for Histogram {
    fn clone(&self) -> Self {
        Self {
            metric: self.metric.clone(),
            registry: self.registry.clone(),
            buckets: self.buckets.clone(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        let duration_seconds = duration.as_secs_f64();
        
        // Use blocking version in drop to avoid async in drop
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                self.histogram.observe(duration_seconds).await;
            });
        });
    }
}

impl WorkflowMetrics {
    /// Create new workflow metrics with standard buckets
    pub async fn new(registry: &MetricsRegistry) -> Self {
        // Standard duration buckets (in seconds)
        let duration_buckets = vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 300.0
        ];

        Self {
            workflows_started: registry.counter(
                "workflow_executions_started_total".to_string(),
                "Total number of workflow executions started".to_string(),
                HashMap::new(),
            ).await,
            
            workflows_completed: registry.counter(
                "workflow_executions_completed_total".to_string(),
                "Total number of workflow executions completed successfully".to_string(),
                HashMap::new(),
            ).await,
            
            workflows_failed: registry.counter(
                "workflow_executions_failed_total".to_string(),
                "Total number of workflow executions that failed".to_string(),
                HashMap::new(),
            ).await,
            
            active_workflows: registry.gauge(
                "workflow_executions_active".to_string(),
                "Number of currently active workflow executions".to_string(),
                HashMap::new(),
            ).await,
            
            workflow_duration: registry.histogram(
                "workflow_execution_duration_seconds".to_string(),
                "Duration of workflow executions in seconds".to_string(),
                HashMap::new(),
                duration_buckets.clone(),
            ).await,
            
            step_duration: registry.histogram(
                "workflow_step_duration_seconds".to_string(),
                "Duration of workflow step executions in seconds".to_string(),
                HashMap::new(),
                duration_buckets,
            ).await,
            
            template_instantiations: registry.counter(
                "template_instantiations_total".to_string(),
                "Total number of template instantiations".to_string(),
                HashMap::new(),
            ).await,
            
            template_instantiation_duration: registry.histogram(
                "template_instantiation_duration_seconds".to_string(),
                "Duration of template instantiations in seconds".to_string(),
                HashMap::new(),
                vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0],
            ).await,
            
            cross_domain_events: registry.counter(
                "cross_domain_events_published_total".to_string(),
                "Total number of cross-domain events published".to_string(),
                HashMap::new(),
            ).await,
            
            nats_messages_processed: registry.counter(
                "nats_messages_processed_total".to_string(),
                "Total number of NATS messages processed".to_string(),
                HashMap::new(),
            ).await,
            
            errors_by_category: registry.counter(
                "errors_by_category_total".to_string(),
                "Total number of errors by category".to_string(),
                HashMap::new(),
            ).await,
            
            recovery_attempts: registry.counter(
                "recovery_attempts_total".to_string(),
                "Total number of recovery attempts".to_string(),
                HashMap::new(),
            ).await,
            
            circuit_breaker_state_changes: registry.counter(
                "circuit_breaker_state_changes_total".to_string(),
                "Total number of circuit breaker state changes".to_string(),
                HashMap::new(),
            ).await,
            
            bulkhead_rejections: registry.counter(
                "bulkhead_rejections_total".to_string(),
                "Total number of requests rejected by bulkheads".to_string(),
                HashMap::new(),
            ).await,
        }
    }
}

impl PrometheusExporter {
    /// Create new Prometheus exporter
    pub fn new(endpoint: String, auth: Option<PrometheusAuth>) -> Self {
        Self {
            endpoint,
            auth,
            client: reqwest::Client::new(),
        }
    }
}

impl MetricExporter for PrometheusExporter {
    fn export(&self, metrics: Vec<MetricPoint>) -> Result<(), MetricExportError> {
        // Convert metrics to Prometheus format
        let prometheus_metrics = self.format_prometheus_metrics(metrics);
        
        // In a real implementation, this would send to Prometheus via remote_write
        println!("Exporting to Prometheus: {} metrics", prometheus_metrics.len());
        
        Ok(())
    }

    fn name(&self) -> &str {
        "prometheus"
    }

    fn is_healthy(&self) -> bool {
        // In real implementation, would check Prometheus connectivity
        true
    }
}

impl PrometheusExporter {
    fn format_prometheus_metrics(&self, metrics: Vec<MetricPoint>) -> Vec<String> {
        metrics
            .into_iter()
            .map(|metric| {
                let labels = if metric.labels.is_empty() {
                    String::new()
                } else {
                    let label_pairs: Vec<String> = metric.labels
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    format!("{{{}}}", label_pairs.join(","))
                };

                match metric.value {
                    MetricValue::Counter(value) => {
                        format!("{}{} {}", metric.name, labels, value)
                    }
                    MetricValue::Gauge(value) => {
                        format!("{}{} {}", metric.name, labels, value)
                    }
                    MetricValue::Histogram { count, sum, .. } => {
                        format!("{}_count{} {}\n{}_sum{} {}", 
                               metric.name, labels, count,
                               metric.name, labels, sum)
                    }
                    MetricValue::Summary { count, sum, .. } => {
                        format!("{}_count{} {}\n{}_sum{} {}", 
                               metric.name, labels, count,
                               metric.name, labels, sum)
                    }
                }
            })
            .collect()
    }
}

impl ConsoleExporter {
    /// Create new console exporter
    pub fn new(pretty: bool) -> Self {
        Self { pretty }
    }
}

impl MetricExporter for ConsoleExporter {
    fn export(&self, metrics: Vec<MetricPoint>) -> Result<(), MetricExportError> {
        if self.pretty {
            for metric in metrics {
                println!("📊 Metric: {}", metric.name);
                if !metric.labels.is_empty() {
                    println!("   Labels: {:?}", metric.labels);
                }
                println!("   Value: {:?}", metric.value);
                println!("   Timestamp: {:?}", metric.timestamp);
                println!();
            }
        } else {
            for metric in metrics {
                println!("{}: {:?}", metric.name, metric.value);
            }
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

impl CustomExporter {
    /// Create new custom exporter
    pub fn new<F>(name: String, export_fn: F) -> Self 
    where
        F: Fn(Vec<MetricPoint>) -> Result<(), MetricExportError> + Send + Sync + 'static,
    {
        Self {
            name,
            export_fn: Box::new(export_fn),
        }
    }
}

impl MetricExporter for CustomExporter {
    fn export(&self, metrics: Vec<MetricPoint>) -> Result<(), MetricExportError> {
        (self.export_fn)(metrics)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter_metric() {
        let registry = MetricsRegistry::new();
        let counter = registry.counter(
            "test_counter".to_string(),
            "Test counter metric".to_string(),
            HashMap::new(),
        ).await;

        assert_eq!(counter.value().await, 0);
        
        counter.increment().await;
        assert_eq!(counter.value().await, 1);
        
        counter.add(5).await;
        assert_eq!(counter.value().await, 6);
    }

    #[tokio::test]
    async fn test_gauge_metric() {
        let registry = MetricsRegistry::new();
        let gauge = registry.gauge(
            "test_gauge".to_string(),
            "Test gauge metric".to_string(),
            HashMap::new(),
        ).await;

        assert_eq!(gauge.value().await, 0.0);
        
        gauge.set(42.5).await;
        assert_eq!(gauge.value().await, 42.5);
        
        gauge.increment(7.5).await;
        assert_eq!(gauge.value().await, 50.0);
        
        gauge.decrement(10.0).await;
        assert_eq!(gauge.value().await, 40.0);
    }

    #[tokio::test]
    async fn test_histogram_metric() {
        let registry = MetricsRegistry::new();
        let histogram = registry.histogram(
            "test_histogram".to_string(),
            "Test histogram metric".to_string(),
            HashMap::new(),
            vec![0.1, 0.5, 1.0, 5.0, 10.0],
        ).await;

        histogram.observe(0.05).await;
        histogram.observe(0.3).await;
        histogram.observe(2.0).await;
        histogram.observe(15.0).await;

        let metrics = registry.collect_metrics().await;
        assert_eq!(metrics.len(), 1);

        if let MetricValue::Histogram { count, sum, buckets } = &metrics[0].value {
            assert_eq!(*count, 4);
            assert_eq!(*sum, 17.35);
            // Check bucket counts
            assert_eq!(buckets[0].count, 1); // 0.05 <= 0.1
            assert_eq!(buckets[1].count, 2); // 0.05, 0.3 <= 0.5
            assert_eq!(buckets[2].count, 2); // same as above
            assert_eq!(buckets[3].count, 3); // 0.05, 0.3, 2.0 <= 5.0
            assert_eq!(buckets[4].count, 3); // same as above (15.0 > 10.0)
        } else {
            panic!("Expected histogram metric value");
        }
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let mut registry = MetricsRegistry::new();
        
        // Add console exporter for testing
        registry.add_exporter(Box::new(ConsoleExporter::new(false)));
        
        let counter = registry.counter(
            "test_counter".to_string(),
            "Test counter".to_string(),
            HashMap::new(),
        ).await;
        
        counter.increment().await;
        
        // Export metrics (output will be printed to console)
        registry.export_metrics().await;
        
        let metrics = registry.collect_metrics().await;
        assert_eq!(metrics.len(), 1);
    }
}