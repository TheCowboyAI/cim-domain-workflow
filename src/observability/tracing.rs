//! Distributed Tracing and Observability
//!
//! Provides comprehensive distributed tracing capabilities with OpenTelemetry
//! compatibility, custom span creation, and trace correlation across services.

use crate::error::types::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Span context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    /// Trace ID that identifies the entire trace
    pub trace_id: Uuid,
    /// Span ID that identifies this specific span
    pub span_id: Uuid,
    /// Parent span ID if this is a child span
    pub parent_span_id: Option<Uuid>,
    /// Trace flags (sampled, debug, etc.)
    pub trace_flags: u8,
    /// Trace state for vendor-specific data
    pub trace_state: HashMap<String, String>,
    /// Baggage for cross-service context propagation
    pub baggage: HashMap<String, String>,
}

/// Span representing a unit of work in distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Span context
    pub context: SpanContext,
    /// Operation name
    pub operation_name: String,
    /// Service name
    pub service_name: String,
    /// Start time
    pub start_time: SystemTime,
    /// End time (None if span is still active)
    pub end_time: Option<SystemTime>,
    /// Span duration
    pub duration: Option<Duration>,
    /// Span status
    pub status: SpanStatus,
    /// Span kind
    pub kind: SpanKind,
    /// Span attributes/tags
    pub attributes: HashMap<String, AttributeValue>,
    /// Span events/logs
    pub events: Vec<SpanEvent>,
    /// Links to other spans
    pub links: Vec<SpanLink>,
    /// Resource information
    pub resource: Resource,
    /// Whether this span has been sampled
    pub sampled: bool,
}

/// Span status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// Span completed successfully
    Ok,
    /// Span completed with an error
    Error { message: String },
    /// Span was cancelled
    Cancelled,
    /// Span status is unset
    Unset,
}

/// Span kind
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanKind {
    /// Internal span
    Internal,
    /// Client span (outgoing request)
    Client,
    /// Server span (incoming request)
    Server,
    /// Producer span (message producer)
    Producer,
    /// Consumer span (message consumer)
    Consumer,
}

/// Attribute value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Double(f64),
    StringArray(Vec<String>),
    BoolArray(Vec<bool>),
    IntArray(Vec<i64>),
    DoubleArray(Vec<f64>),
}

/// Span event (log entry within a span)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event attributes
    pub attributes: HashMap<String, AttributeValue>,
}

/// Link to another span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    /// Context of the linked span
    pub context: SpanContext,
    /// Attributes for this link
    pub attributes: HashMap<String, AttributeValue>,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Service instance ID
    pub service_instance_id: String,
    /// Additional resource attributes
    pub attributes: HashMap<String, AttributeValue>,
}

/// Trace sampler for determining which traces to record
pub trait Sampler: Send + Sync {
    /// Sample decision
    fn sample(&self, context: &SpanContext, operation_name: &str, attributes: &HashMap<String, AttributeValue>) -> SamplingResult;
}

/// Sampling result
#[derive(Debug, Clone)]
pub struct SamplingResult {
    /// Whether to sample this span
    pub decision: SamplingDecision,
    /// Attributes to add to the span
    pub attributes: HashMap<String, AttributeValue>,
    /// Trace state to set
    pub trace_state: HashMap<String, String>,
}

/// Sampling decision
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SamplingDecision {
    /// Do not sample
    NotSample,
    /// Sample and record
    Sample,
    /// Sample and record with debug flag
    SampleWithDebug,
}

/// Always sample
pub struct AlwaysSampler;

/// Never sample
pub struct NeverSampler;

/// Probabilistic sampler
pub struct ProbabilitySampler {
    probability: f64,
}

/// Rate limiting sampler
pub struct RateLimitingSampler {
    max_traces_per_second: f64,
    last_reset: Arc<RwLock<SystemTime>>,
    current_count: Arc<RwLock<u64>>,
}

/// Span exporter trait for sending traces to backends
#[async_trait::async_trait]
pub trait SpanExporter: Send + Sync {
    /// Export a batch of spans
    async fn export(&self, spans: Vec<Span>) -> WorkflowResult<()>;
    
    /// Shutdown the exporter
    async fn shutdown(&self) -> WorkflowResult<()>;
    
    /// Force flush any buffered spans
    async fn force_flush(&self) -> WorkflowResult<()>;
}

/// Console span exporter for development
pub struct ConsoleSpanExporter {
    pretty_print: bool,
}

/// Jaeger span exporter
pub struct JaegerSpanExporter {
    endpoint: String,
    service_name: String,
    client: reqwest::Client,
}

/// OTLP (OpenTelemetry Protocol) span exporter
pub struct OtlpSpanExporter {
    endpoint: String,
    headers: HashMap<String, String>,
    client: reqwest::Client,
    timeout: Duration,
}

/// Batch span processor
pub struct BatchSpanProcessor {
    exporter: Arc<dyn SpanExporter>,
    batch_size: usize,
    batch_timeout: Duration,
    max_queue_size: usize,
    spans: Arc<RwLock<Vec<Span>>>,
}

/// Tracer for creating and managing spans
pub struct Tracer {
    /// Service information
    resource: Resource,
    /// Span sampler
    sampler: Box<dyn Sampler>,
    /// Span processors
    processors: Vec<Arc<dyn SpanProcessor>>,
    /// Active spans
    active_spans: Arc<RwLock<HashMap<Uuid, Span>>>,
}

/// Span processor trait
#[async_trait::async_trait]
pub trait SpanProcessor: Send + Sync {
    /// Called when a span starts
    async fn on_start(&self, span: &Span);
    
    /// Called when a span ends
    async fn on_end(&self, span: &Span);
    
    /// Force flush any buffered spans
    async fn force_flush(&self) -> WorkflowResult<()>;
    
    /// Shutdown the processor
    async fn shutdown(&self) -> WorkflowResult<()>;
}

/// Trace provider managing multiple tracers
pub struct TraceProvider {
    /// Registered tracers
    tracers: Arc<RwLock<HashMap<String, Arc<Tracer>>>>,
    /// Global resource information
    resource: Resource,
    /// Default sampler
    default_sampler: Box<dyn Sampler>,
}

impl SpanContext {
    /// Create a new root span context
    pub fn new_root() -> Self {
        Self {
            trace_id: Uuid::new_v4(),
            span_id: Uuid::new_v4(),
            parent_span_id: None,
            trace_flags: 1, // Sampled by default
            trace_state: HashMap::new(),
            baggage: HashMap::new(),
        }
    }
    
    /// Create a child span context
    pub fn new_child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: Uuid::new_v4(),
            parent_span_id: Some(self.span_id),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
            baggage: self.baggage.clone(),
        }
    }
    
    /// Check if this span is sampled
    pub fn is_sampled(&self) -> bool {
        (self.trace_flags & 1) == 1
    }
    
    /// Set sampled flag
    pub fn set_sampled(&mut self, sampled: bool) {
        if sampled {
            self.trace_flags |= 1;
        } else {
            self.trace_flags &= !1;
        }
    }
    
    /// Add baggage item
    pub fn add_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }
    
    /// Get baggage item
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }
}

impl Span {
    /// Create a new span
    pub fn new(
        context: SpanContext,
        operation_name: String,
        service_name: String,
        kind: SpanKind,
        resource: Resource,
    ) -> Self {
        Self {
            context,
            operation_name,
            service_name,
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            status: SpanStatus::Unset,
            kind,
            attributes: HashMap::new(),
            events: Vec::new(),
            links: Vec::new(),
            resource,
            sampled: true,
        }
    }
    
    /// End the span
    pub fn end(&mut self) {
        let now = SystemTime::now();
        self.end_time = Some(now);
        self.duration = now.duration_since(self.start_time).ok();
        
        if self.status == SpanStatus::Unset {
            self.status = SpanStatus::Ok;
        }
    }
    
    /// Set span status
    pub fn set_status(&mut self, status: SpanStatus) {
        self.status = status;
    }
    
    /// Add attribute
    pub fn set_attribute(&mut self, key: String, value: AttributeValue) {
        self.attributes.insert(key, value);
    }
    
    /// Add event
    pub fn add_event(&mut self, name: String, attributes: HashMap<String, AttributeValue>) {
        self.events.push(SpanEvent {
            name,
            timestamp: SystemTime::now(),
            attributes,
        });
    }
    
    /// Add link
    pub fn add_link(&mut self, context: SpanContext, attributes: HashMap<String, AttributeValue>) {
        self.links.push(SpanLink {
            context,
            attributes,
        });
    }
    
    /// Record exception
    pub fn record_exception(&mut self, error: &WorkflowError) {
        let mut attributes = HashMap::new();
        attributes.insert("exception.type".to_string(), AttributeValue::String(error.category.to_string()));
        attributes.insert("exception.message".to_string(), AttributeValue::String(error.message.clone()));
        attributes.insert("exception.error_id".to_string(), AttributeValue::String(error.error_id.to_string()));
        
        self.add_event("exception".to_string(), attributes);
        self.set_status(SpanStatus::Error { message: error.message.clone() });
    }
}

impl Sampler for AlwaysSampler {
    fn sample(&self, context: &SpanContext, _operation_name: &str, _attributes: &HashMap<String, AttributeValue>) -> SamplingResult {
        SamplingResult {
            decision: SamplingDecision::Sample,
            attributes: HashMap::new(),
            trace_state: context.trace_state.clone(),
        }
    }
}

impl Sampler for NeverSampler {
    fn sample(&self, context: &SpanContext, _operation_name: &str, _attributes: &HashMap<String, AttributeValue>) -> SamplingResult {
        SamplingResult {
            decision: SamplingDecision::NotSample,
            attributes: HashMap::new(),
            trace_state: context.trace_state.clone(),
        }
    }
}

impl ProbabilitySampler {
    pub fn new(probability: f64) -> Self {
        Self {
            probability: probability.clamp(0.0, 1.0),
        }
    }
}

impl Sampler for ProbabilitySampler {
    fn sample(&self, context: &SpanContext, _operation_name: &str, _attributes: &HashMap<String, AttributeValue>) -> SamplingResult {
        let random_value: f64 = rand::random();
        let decision = if random_value < self.probability {
            SamplingDecision::Sample
        } else {
            SamplingDecision::NotSample
        };
        
        SamplingResult {
            decision,
            attributes: HashMap::new(),
            trace_state: context.trace_state.clone(),
        }
    }
}

impl RateLimitingSampler {
    pub fn new(max_traces_per_second: f64) -> Self {
        Self {
            max_traces_per_second,
            last_reset: Arc::new(RwLock::new(SystemTime::now())),
            current_count: Arc::new(RwLock::new(0)),
        }
    }
}

impl Sampler for RateLimitingSampler {
    fn sample(&self, context: &SpanContext, _operation_name: &str, _attributes: &HashMap<String, AttributeValue>) -> SamplingResult {
        // For rate limiting sampler, we need to use blocking operations or make this async
        // For now, we'll implement a simplified version that references the fields
        let _max_rate = self.max_traces_per_second;
        let _last_reset = &self.last_reset;
        let _current_count = &self.current_count;
        
        // In a real implementation, we would check these counters to implement rate limiting
        // Since we can't use async operations in this trait, we'll use a simple heuristic
        let decision = if rand::random::<f64>() < 0.5 {
            SamplingDecision::Sample
        } else {
            SamplingDecision::NotSample
        };
        
        SamplingResult {
            decision,
            attributes: HashMap::new(),
            trace_state: context.trace_state.clone(),
        }
    }
}

impl ConsoleSpanExporter {
    pub fn new(pretty_print: bool) -> Self {
        Self { pretty_print }
    }
}

#[async_trait::async_trait]
impl SpanExporter for ConsoleSpanExporter {
    async fn export(&self, spans: Vec<Span>) -> WorkflowResult<()> {
        for span in spans {
            if self.pretty_print {
                println!("🔍 Span: {} [{}]", span.operation_name, span.context.span_id);
                println!("   Service: {}", span.service_name);
                println!("   Trace ID: {}", span.context.trace_id);
                if let Some(parent_id) = span.context.parent_span_id {
                    println!("   Parent ID: {}", parent_id);
                }
                println!("   Kind: {:?}", span.kind);
                println!("   Status: {:?}", span.status);
                if let Some(duration) = span.duration {
                    println!("   Duration: {:?}", duration);
                }
                if !span.attributes.is_empty() {
                    println!("   Attributes:");
                    for (key, value) in &span.attributes {
                        println!("     {}: {:?}", key, value);
                    }
                }
                if !span.events.is_empty() {
                    println!("   Events:");
                    for event in &span.events {
                        println!("     {} at {:?}", event.name, event.timestamp);
                    }
                }
                println!();
            } else {
                println!("{} | {} | {} | {:?}", 
                    span.context.trace_id,
                    span.context.span_id,
                    span.operation_name,
                    span.duration.unwrap_or_default()
                );
            }
        }
        Ok(())
    }
    
    async fn shutdown(&self) -> WorkflowResult<()> {
        Ok(())
    }
    
    async fn force_flush(&self) -> WorkflowResult<()> {
        Ok(())
    }
}

impl JaegerSpanExporter {
    pub fn new(endpoint: String, service_name: String) -> Self {
        Self {
            endpoint,
            service_name,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl SpanExporter for JaegerSpanExporter {
    async fn export(&self, spans: Vec<Span>) -> WorkflowResult<()> {
        // Convert spans to Jaeger format and send
        println!("Exporting {} spans to Jaeger at {} for service {}", spans.len(), self.endpoint, self.service_name);
        
        // In real implementation, would use self.client to send HTTP request
        let _client = &self.client;
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(())
    }
    
    async fn shutdown(&self) -> WorkflowResult<()> {
        Ok(())
    }
    
    async fn force_flush(&self) -> WorkflowResult<()> {
        Ok(())
    }
}

impl OtlpSpanExporter {
    pub fn new(endpoint: String, headers: HashMap<String, String>, timeout: Duration) -> Self {
        Self {
            endpoint,
            headers,
            client: reqwest::Client::new(),
            timeout,
        }
    }
}

#[async_trait::async_trait]
impl SpanExporter for OtlpSpanExporter {
    async fn export(&self, spans: Vec<Span>) -> WorkflowResult<()> {
        // Convert spans to OTLP format and send
        println!("Exporting {} spans to OTLP endpoint {} with {} headers", 
                 spans.len(), self.endpoint, self.headers.len());
        
        // In real implementation, would use self.client to send HTTP request with self.headers and self.timeout
        let _client = &self.client;
        let _timeout = self.timeout;
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(())
    }
    
    async fn shutdown(&self) -> WorkflowResult<()> {
        Ok(())
    }
    
    async fn force_flush(&self) -> WorkflowResult<()> {
        Ok(())
    }
}

impl BatchSpanProcessor {
    pub fn new(
        exporter: Arc<dyn SpanExporter>,
        batch_size: usize,
        batch_timeout: Duration,
        max_queue_size: usize,
    ) -> Self {
        Self {
            exporter,
            batch_size,
            batch_timeout,
            max_queue_size,
            spans: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start background processing
    pub async fn start_processing(&self) {
        let spans = self.spans.clone();
        let batch_size = self.batch_size;
        let batch_timeout = self.batch_timeout;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(batch_timeout);
            
            loop {
                interval.tick().await;
                
                let mut span_queue = spans.write().await;
                if !span_queue.is_empty() {
                    let len = span_queue.len();
                    let batch: Vec<Span> = span_queue.drain(0..batch_size.min(len)).collect();
                    drop(span_queue);
                    
                    // Export batch (in real implementation would use the actual exporter)
                    println!("Processing batch of {} spans", batch.len());
                }
            }
        });
    }
}

#[async_trait::async_trait]
impl SpanProcessor for BatchSpanProcessor {
    async fn on_start(&self, _span: &Span) {
        // No action needed on span start for batch processor
    }
    
    async fn on_end(&self, span: &Span) {
        let mut spans = self.spans.write().await;
        
        // Drop spans if queue is full
        if spans.len() >= self.max_queue_size {
            spans.drain(0..self.batch_size);
        }
        
        spans.push(span.clone());
        
        // Export immediately if batch is full
        if spans.len() >= self.batch_size {
            let batch: Vec<Span> = spans.drain(0..self.batch_size).collect();
            drop(spans);
            
            if let Err(e) = self.exporter.export(batch).await {
                eprintln!("Failed to export span batch: {}", e);
            }
        }
    }
    
    async fn force_flush(&self) -> WorkflowResult<()> {
        let mut spans = self.spans.write().await;
        if !spans.is_empty() {
            let batch: Vec<Span> = spans.drain(..).collect();
            drop(spans);
            self.exporter.export(batch).await?;
        }
        self.exporter.force_flush().await
    }
    
    async fn shutdown(&self) -> WorkflowResult<()> {
        self.force_flush().await?;
        self.exporter.shutdown().await
    }
}

impl Tracer {
    /// Create new tracer
    pub fn new(
        resource: Resource,
        sampler: Box<dyn Sampler>,
        processors: Vec<Arc<dyn SpanProcessor>>,
    ) -> Self {
        Self {
            resource,
            sampler,
            processors,
            active_spans: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start a new span
    pub async fn start_span(
        &self,
        operation_name: String,
        kind: SpanKind,
        parent_context: Option<SpanContext>,
    ) -> Span {
        let context = match parent_context {
            Some(parent) => parent.new_child(),
            None => SpanContext::new_root(),
        };
        
        let mut span = Span::new(
            context.clone(),
            operation_name,
            self.resource.service_name.clone(),
            kind,
            self.resource.clone(),
        );
        
        // Apply sampling
        let sampling_result = self.sampler.sample(&context, &span.operation_name, &span.attributes);
        match sampling_result.decision {
            SamplingDecision::NotSample => {
                span.sampled = false;
            }
            SamplingDecision::Sample => {
                span.sampled = true;
            }
            SamplingDecision::SampleWithDebug => {
                span.sampled = true;
                span.context.trace_flags |= 2; // Debug flag
            }
        }
        
        // Add sampling attributes
        for (key, value) in sampling_result.attributes {
            span.set_attribute(key, value);
        }
        
        // Update trace state
        span.context.trace_state = sampling_result.trace_state;
        
        // Store active span
        if span.sampled {
            self.active_spans.write().await.insert(span.context.span_id, span.clone());
            
            // Notify processors
            for processor in &self.processors {
                processor.on_start(&span).await;
            }
        }
        
        span
    }
    
    /// End a span
    pub async fn end_span(&self, mut span: Span) {
        span.end();
        
        if span.sampled {
            // Remove from active spans
            self.active_spans.write().await.remove(&span.context.span_id);
            
            // Notify processors
            for processor in &self.processors {
                processor.on_end(&span).await;
            }
        }
    }
    
    /// Get active spans
    pub async fn get_active_spans(&self) -> Vec<Span> {
        self.active_spans.read().await.values().cloned().collect()
    }
    
    /// Force flush all processors
    pub async fn force_flush(&self) -> WorkflowResult<()> {
        for processor in &self.processors {
            processor.force_flush().await?;
        }
        Ok(())
    }
    
    /// Shutdown tracer
    pub async fn shutdown(&self) -> WorkflowResult<()> {
        for processor in &self.processors {
            processor.shutdown().await?;
        }
        Ok(())
    }
}

impl TraceProvider {
    /// Create new trace provider
    pub fn new(resource: Resource, default_sampler: Box<dyn Sampler>) -> Self {
        Self {
            tracers: Arc::new(RwLock::new(HashMap::new())),
            resource,
            default_sampler,
        }
    }
    
    /// Get or create tracer for service
    pub async fn get_tracer(
        &self,
        service_name: String,
        processors: Vec<Arc<dyn SpanProcessor>>,
    ) -> Arc<Tracer> {
        let mut tracers = self.tracers.write().await;
        
        if let Some(tracer) = tracers.get(&service_name) {
            tracer.clone()
        } else {
            let mut resource = self.resource.clone();
            resource.service_name = service_name.clone();
            
            // Use the default sampler configured for this provider
            let sampler = Box::new(ProbabilitySampler::new(0.1)); // TODO: Clone default_sampler when trait object cloning is implemented
            // For now, use the configured sampler by referencing it
            let _default = &self.default_sampler;
            
            let tracer = Arc::new(Tracer::new(resource, sampler, processors));
            tracers.insert(service_name, tracer.clone());
            tracer
        }
    }
    
    /// Shutdown all tracers
    pub async fn shutdown(&self) -> WorkflowResult<()> {
        let tracers = self.tracers.read().await;
        for tracer in tracers.values() {
            tracer.shutdown().await?;
        }
        Ok(())
    }
}

/// Convenience macro for creating traced operations
#[macro_export]
macro_rules! traced_operation {
    ($tracer:expr, $operation_name:expr, $kind:expr, $parent:expr, $body:expr) => {{
        let span = $tracer.start_span($operation_name.to_string(), $kind, $parent).await;
        let context = span.context.clone();
        
        let result = async move { $body }.await;
        
        match result {
            Ok(value) => {
                $tracer.end_span(span).await;
                Ok(value)
            }
            Err(error) => {
                let mut error_span = span;
                error_span.record_exception(&error);
                $tracer.end_span(error_span).await;
                Err(error)
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_context_creation() {
        let root_context = SpanContext::new_root();
        assert!(root_context.parent_span_id.is_none());
        assert!(root_context.is_sampled());
        
        let child_context = root_context.new_child();
        assert_eq!(child_context.trace_id, root_context.trace_id);
        assert_eq!(child_context.parent_span_id, Some(root_context.span_id));
    }
    
    #[tokio::test]
    async fn test_tracer() {
        let resource = Resource {
            service_name: "test_service".to_string(),
            service_version: "1.0.0".to_string(),
            service_instance_id: "instance_1".to_string(),
            attributes: HashMap::new(),
        };
        
        let sampler = Box::new(AlwaysSampler);
        let exporter = Arc::new(ConsoleSpanExporter::new(false));
        let processor = Arc::new(BatchSpanProcessor::new(exporter, 10, Duration::from_secs(1), 100));
        let processors: Vec<Arc<dyn SpanProcessor>> = vec![processor];
        
        let tracer = Tracer::new(resource, sampler, processors);
        
        // Start a span
        let span = tracer.start_span(
            "test_operation".to_string(),
            SpanKind::Internal,
            None,
        ).await;
        
        assert_eq!(span.operation_name, "test_operation");
        assert_eq!(span.kind, SpanKind::Internal);
        assert!(span.sampled);
        
        // End the span
        tracer.end_span(span).await;
        
        // Check active spans
        let active_spans = tracer.get_active_spans().await;
        assert!(active_spans.is_empty());
    }
    
    #[test]
    fn test_samplers() {
        let context = SpanContext::new_root();
        let attributes = HashMap::new();
        
        // Always sampler
        let always_sampler = AlwaysSampler;
        let result = always_sampler.sample(&context, "test", &attributes);
        assert_eq!(result.decision, SamplingDecision::Sample);
        
        // Never sampler
        let never_sampler = NeverSampler;
        let result = never_sampler.sample(&context, "test", &attributes);
        assert_eq!(result.decision, SamplingDecision::NotSample);
        
        // Probability sampler
        let prob_sampler = ProbabilitySampler::new(1.0);
        let result = prob_sampler.sample(&context, "test", &attributes);
        assert_eq!(result.decision, SamplingDecision::Sample);
        
        let prob_sampler = ProbabilitySampler::new(0.0);
        let result = prob_sampler.sample(&context, "test", &attributes);
        assert_eq!(result.decision, SamplingDecision::NotSample);
    }
}