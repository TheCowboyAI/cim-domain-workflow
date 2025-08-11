//! Error Tracing and Observability
//!
//! Provides comprehensive error tracing, correlation tracking, and observability
//! features for debugging and monitoring workflow system errors.

use crate::error::types::{WorkflowError, ErrorContext, ErrorCategory, ErrorSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Error trace information for debugging and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrace {
    /// Trace identifier
    pub trace_id: Uuid,
    /// Root error that started the trace
    pub root_error: WorkflowError,
    /// Related errors in chronological order
    pub related_errors: Vec<WorkflowError>,
    /// Span information
    pub spans: Vec<TraceSpan>,
    /// Trace metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// End time (if trace is complete)
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Trace span representing a unit of work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    /// Span identifier
    pub span_id: Uuid,
    /// Parent span ID (if this is a child span)
    pub parent_span_id: Option<Uuid>,
    /// Operation name
    pub operation: String,
    /// Service or component name
    pub service: String,
    /// Span start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Span end time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Duration
    pub duration: Option<Duration>,
    /// Span tags
    pub tags: HashMap<String, String>,
    /// Log entries within this span
    pub logs: Vec<SpanLog>,
    /// Span status
    pub status: SpanStatus,
    /// Error information (if span failed)
    pub error: Option<WorkflowError>,
}

/// Span log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    /// Log timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Log fields
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Span execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    /// Span is still active
    Active,
    /// Span completed successfully
    Success,
    /// Span completed with error
    Error,
    /// Span was cancelled
    Cancelled,
    /// Span timed out
    Timeout,
}

/// Error correlation tracker
pub struct ErrorCorrelationTracker {
    /// Active traces
    traces: Arc<RwLock<HashMap<Uuid, ErrorTrace>>>,
    /// Correlation mappings
    correlations: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    /// Trace retention policy
    retention_duration: Duration,
}

/// Metrics collector for error analysis
pub struct ErrorMetricsCollector {
    /// Error counts by category
    error_counts: Arc<RwLock<HashMap<ErrorCategory, u64>>>,
    /// Error counts by severity
    severity_counts: Arc<RwLock<HashMap<ErrorSeverity, u64>>>,
    /// Error rates over time
    error_rates: Arc<RwLock<Vec<ErrorRateDataPoint>>>,
    /// Component error statistics
    component_stats: Arc<RwLock<HashMap<String, ComponentErrorStats>>>,
}

/// Error rate data point for time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRateDataPoint {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Error count in time window
    pub error_count: u64,
    /// Total operations in time window
    pub total_operations: u64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Time window size
    pub window_size: Duration,
}

/// Component error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentErrorStats {
    /// Component name
    pub component: String,
    /// Total errors
    pub total_errors: u64,
    /// Errors by category
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    /// Average error frequency (errors per hour)
    pub error_frequency: f64,
    /// Most common error patterns
    pub common_patterns: Vec<ErrorPattern>,
    /// Last error timestamp
    pub last_error: Option<chrono::DateTime<chrono::Utc>>,
}

/// Error pattern for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Pattern identifier
    pub pattern_id: String,
    /// Pattern description
    pub description: String,
    /// Occurrence count
    pub count: u64,
    /// Pattern confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Example errors matching this pattern
    pub examples: Vec<Uuid>,
}

/// Trace context for passing through operations
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Current trace ID
    pub trace_id: Uuid,
    /// Current span ID
    pub span_id: Uuid,
    /// Parent span ID
    pub parent_span_id: Option<Uuid>,
    /// Baggage (key-value pairs that propagate)
    pub baggage: HashMap<String, String>,
}

/// Tracer for creating and managing traces
pub struct WorkflowTracer {
    /// Service name
    service_name: String,
    /// Active spans
    active_spans: Arc<RwLock<HashMap<Uuid, TraceSpan>>>,
    /// Correlation tracker
    correlation_tracker: Arc<ErrorCorrelationTracker>,
    /// Metrics collector
    metrics_collector: Arc<ErrorMetricsCollector>,
}

impl ErrorCorrelationTracker {
    /// Create new correlation tracker
    pub fn new(retention_duration: Duration) -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            correlations: Arc::new(RwLock::new(HashMap::new())),
            retention_duration,
        }
    }

    /// Start a new error trace
    pub async fn start_trace(&self, root_error: WorkflowError) -> Uuid {
        let trace_id = Uuid::new_v4();
        let trace = ErrorTrace {
            trace_id,
            root_error,
            related_errors: vec![],
            spans: vec![],
            metadata: HashMap::new(),
            started_at: chrono::Utc::now(),
            ended_at: None,
        };

        self.traces.write().await.insert(trace_id, trace);
        trace_id
    }

    /// Add related error to existing trace
    pub async fn add_related_error(&self, trace_id: Uuid, error: WorkflowError) {
        if let Some(trace) = self.traces.write().await.get_mut(&trace_id) {
            trace.related_errors.push(error);
        }
    }

    /// Add span to trace
    pub async fn add_span(&self, trace_id: Uuid, span: TraceSpan) {
        if let Some(trace) = self.traces.write().await.get_mut(&trace_id) {
            trace.spans.push(span);
        }
    }

    /// Complete trace
    pub async fn complete_trace(&self, trace_id: Uuid) {
        if let Some(trace) = self.traces.write().await.get_mut(&trace_id) {
            trace.ended_at = Some(chrono::Utc::now());
        }
    }

    /// Get trace by ID
    pub async fn get_trace(&self, trace_id: Uuid) -> Option<ErrorTrace> {
        self.traces.read().await.get(&trace_id).cloned()
    }

    /// Correlate errors by correlation ID
    pub async fn correlate_errors(&self, correlation_id: Uuid, error_ids: Vec<Uuid>) {
        self.correlations.write().await.insert(correlation_id, error_ids);
    }

    /// Get correlated errors
    pub async fn get_correlated_errors(&self, correlation_id: Uuid) -> Vec<Uuid> {
        self.correlations.read().await
            .get(&correlation_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Clean up old traces
    pub async fn cleanup_old_traces(&self) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::from_std(self.retention_duration).unwrap();
        
        let mut traces = self.traces.write().await;
        traces.retain(|_, trace| trace.started_at > cutoff_time);
    }

    /// Start cleanup task
    pub async fn start_cleanup_task(&self) {
        let tracker = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60 * 60)); // 1 hour
            
            loop {
                interval.tick().await;
                tracker.cleanup_old_traces().await;
            }
        });
    }
}

impl Clone for ErrorCorrelationTracker {
    fn clone(&self) -> Self {
        Self {
            traces: self.traces.clone(),
            correlations: self.correlations.clone(),
            retention_duration: self.retention_duration,
        }
    }
}

impl ErrorMetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            severity_counts: Arc::new(RwLock::new(HashMap::new())),
            error_rates: Arc::new(RwLock::new(Vec::new())),
            component_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record error occurrence
    pub async fn record_error(&self, error: &WorkflowError, component: Option<String>) {
        // Update category counts
        {
            let mut counts = self.error_counts.write().await;
            *counts.entry(error.category.clone()).or_insert(0) += 1;
        }

        // Update severity counts
        {
            let mut counts = self.severity_counts.write().await;
            *counts.entry(error.severity.clone()).or_insert(0) += 1;
        }

        // Update component stats
        if let Some(comp) = component {
            let mut stats = self.component_stats.write().await;
            let component_stat = stats.entry(comp.clone()).or_insert_with(|| ComponentErrorStats {
                component: comp,
                total_errors: 0,
                errors_by_category: HashMap::new(),
                error_frequency: 0.0,
                common_patterns: vec![],
                last_error: None,
            });

            component_stat.total_errors += 1;
            *component_stat.errors_by_category.entry(error.category.clone()).or_insert(0) += 1;
            component_stat.last_error = Some(error.timestamp);
        }
    }

    /// Update error rate metrics
    pub async fn update_error_rate(&self, error_count: u64, total_operations: u64, window_size: Duration) {
        let error_rate = if total_operations > 0 {
            error_count as f64 / total_operations as f64
        } else {
            0.0
        };

        let data_point = ErrorRateDataPoint {
            timestamp: chrono::Utc::now(),
            error_count,
            total_operations,
            error_rate,
            window_size,
        };

        let mut rates = self.error_rates.write().await;
        rates.push(data_point);

        // Keep only last 1000 data points
        if rates.len() > 1000 {
            let drain_count = rates.len() - 1000;
            rates.drain(0..drain_count);
        }
    }

    /// Get error statistics
    pub async fn get_error_stats(&self) -> ErrorStatistics {
        let error_counts = self.error_counts.read().await.clone();
        let severity_counts = self.severity_counts.read().await.clone();
        let error_rates = self.error_rates.read().await.clone();
        let component_stats = self.component_stats.read().await.clone();

        ErrorStatistics {
            total_errors: error_counts.values().sum(),
            errors_by_category: error_counts,
            errors_by_severity: severity_counts,
            error_rate_history: error_rates,
            component_statistics: component_stats,
            collected_at: chrono::Utc::now(),
        }
    }

    /// Detect error patterns
    pub async fn detect_patterns(&self) -> Vec<ErrorPattern> {
        // Simplified pattern detection - in real implementation would use ML/statistical analysis
        let component_stats = self.component_stats.read().await;
        let mut patterns = Vec::new();

        // High frequency error pattern
        for (component, stats) in component_stats.iter() {
            if stats.error_frequency > 10.0 { // More than 10 errors per hour
                patterns.push(ErrorPattern {
                    pattern_id: format!("high_frequency_{}", component),
                    description: format!("High error frequency in component {}", component),
                    count: stats.total_errors,
                    confidence: 0.8,
                    examples: vec![], // Would contain actual error IDs
                });
            }
        }

        // Category concentration pattern
        let error_counts = self.error_counts.read().await;
        let total_errors: u64 = error_counts.values().sum();
        
        for (category, count) in error_counts.iter() {
            let concentration = *count as f64 / total_errors as f64;
            if concentration > 0.5 { // More than 50% of errors are from one category
                patterns.push(ErrorPattern {
                    pattern_id: format!("category_concentration_{:?}", category),
                    description: format!("High concentration of {:?} errors", category),
                    count: *count,
                    confidence: 0.7,
                    examples: vec![],
                });
            }
        }

        patterns
    }
}

/// Error statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// Total error count
    pub total_errors: u64,
    /// Errors by category
    pub errors_by_category: HashMap<ErrorCategory, u64>,
    /// Errors by severity
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,
    /// Error rate history
    pub error_rate_history: Vec<ErrorRateDataPoint>,
    /// Component statistics
    pub component_statistics: HashMap<String, ComponentErrorStats>,
    /// When statistics were collected
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

impl WorkflowTracer {
    /// Create new tracer
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            correlation_tracker: Arc::new(ErrorCorrelationTracker::new(Duration::from_secs(7 * 24 * 60 * 60))), // 7 days
            metrics_collector: Arc::new(ErrorMetricsCollector::new()),
        }
    }

    /// Start a new span
    pub async fn start_span(&self, operation: String, parent_context: Option<TraceContext>) -> TraceContext {
        let span_id = Uuid::new_v4();
        let (trace_id, parent_span_id) = if let Some(ctx) = parent_context {
            (ctx.trace_id, Some(ctx.span_id))
        } else {
            (Uuid::new_v4(), None)
        };

        let span = TraceSpan {
            span_id,
            parent_span_id,
            operation: operation.clone(),
            service: self.service_name.clone(),
            start_time: chrono::Utc::now(),
            end_time: None,
            duration: None,
            tags: HashMap::new(),
            logs: vec![],
            status: SpanStatus::Active,
            error: None,
        };

        self.active_spans.write().await.insert(span_id, span);

        TraceContext {
            trace_id,
            span_id,
            parent_span_id,
            baggage: HashMap::new(),
        }
    }

    /// Add tag to span
    pub async fn add_span_tag(&self, span_id: Uuid, key: String, value: String) {
        if let Some(span) = self.active_spans.write().await.get_mut(&span_id) {
            span.tags.insert(key, value);
        }
    }

    /// Add log to span
    pub async fn add_span_log(&self, span_id: Uuid, level: LogLevel, message: String, fields: HashMap<String, serde_json::Value>) {
        if let Some(span) = self.active_spans.write().await.get_mut(&span_id) {
            span.logs.push(SpanLog {
                timestamp: chrono::Utc::now(),
                level,
                message,
                fields,
            });
        }
    }

    /// Finish span
    pub async fn finish_span(&self, span_id: Uuid, status: SpanStatus, error: Option<WorkflowError>) {
        if let Some(mut span) = self.active_spans.write().await.remove(&span_id) {
            let end_time = chrono::Utc::now();
            span.end_time = Some(end_time);
            span.duration = Some(Duration::from_nanos(
                (end_time - span.start_time).num_nanoseconds().unwrap_or(0) as u64
            ));
            span.status = status;
            span.error = error.clone();

            // Record error metrics if span failed
            if let Some(ref err) = error {
                self.metrics_collector.record_error(err, Some(span.service.clone())).await;
                
                // Start error trace if this is a root span
                if span.parent_span_id.is_none() {
                    self.correlation_tracker.start_trace(err.clone()).await;
                }
            }

            // In real implementation, would export span to tracing backend
            println!("Span completed: {} - {:?} - {:?}", span.operation, span.status, span.duration);
        }
    }

    /// Get current error statistics
    pub async fn get_error_statistics(&self) -> ErrorStatistics {
        self.metrics_collector.get_error_stats().await
    }

    /// Get error patterns
    pub async fn get_error_patterns(&self) -> Vec<ErrorPattern> {
        self.metrics_collector.detect_patterns().await
    }
}

/// Convenience macro for tracing operations
#[macro_export]
macro_rules! trace_operation {
    ($tracer:expr, $operation:expr, $parent:expr, $body:expr) => {{
        let context = $tracer.start_span($operation.to_string(), $parent).await;
        let span_id = context.span_id;
        
        let result = async move { $body }.await;
        
        match result {
            Ok(value) => {
                $tracer.finish_span(span_id, crate::error::tracing::SpanStatus::Success, None).await;
                Ok(value)
            }
            Err(error) => {
                $tracer.finish_span(span_id, crate::error::tracing::SpanStatus::Error, Some(error.clone())).await;
                Err(error)
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::*;

    #[tokio::test]
    async fn test_correlation_tracker() {
        let tracker = ErrorCorrelationTracker::new(Duration::from_hours(1));
        
        let context = ErrorContext::new("test".to_string());
        let error = WorkflowError::network_error(
            "example.com".to_string(),
            Some(80),
            "http".to_string(),
            Some(Duration::from_secs(30)),
            context,
        );

        let trace_id = tracker.start_trace(error.clone()).await;
        assert!(tracker.get_trace(trace_id).await.is_some());

        tracker.complete_trace(trace_id).await;
        let trace = tracker.get_trace(trace_id).await.unwrap();
        assert!(trace.ended_at.is_some());
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = ErrorMetricsCollector::new();
        
        let context = ErrorContext::new("test".to_string());
        let error = WorkflowError::network_error(
            "example.com".to_string(),
            Some(80),
            "http".to_string(),
            Some(Duration::from_secs(30)),
            context,
        );

        collector.record_error(&error, Some("test_component".to_string())).await;
        
        let stats = collector.get_error_stats().await;
        assert_eq!(stats.total_errors, 1);
        assert!(stats.errors_by_category.contains_key(&ErrorCategory::Network));
    }

    #[tokio::test]
    async fn test_tracer() {
        let tracer = WorkflowTracer::new("test_service".to_string());
        
        let context = tracer.start_span("test_operation".to_string(), None).await;
        
        tracer.add_span_tag(context.span_id, "user_id".to_string(), "12345".to_string()).await;
        tracer.add_span_log(
            context.span_id,
            LogLevel::Info,
            "Operation started".to_string(),
            HashMap::new()
        ).await;
        
        tracer.finish_span(context.span_id, SpanStatus::Success, None).await;
        
        let stats = tracer.get_error_statistics().await;
        assert_eq!(stats.total_errors, 0); // No errors in successful span
    }
}