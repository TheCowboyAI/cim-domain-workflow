//! NATS Event Publishers
//!
//! Implements CIM-compliant event publishing using NATS with standardized subject
//! patterns based on the Workflow Subject Algebra for distributed coordination.

use async_nats::{Client, Message, Subscriber};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::algebra::{WorkflowEvent, Subject, SubjectBuilder};
use crate::messaging::correlation::{WorkflowEventCorrelator, CompletionCriteria};

/// CIM-compliant NATS event publisher with subject algebra integration
// Debug derive removed due to non-Debug correlator
pub struct WorkflowEventPublisher {
    /// NATS client
    client: Client,
    /// Subject prefix for this publisher
    subject_prefix: String,
    /// Event correlator for tracking
    correlator: Arc<WorkflowEventCorrelator>,
    /// Publication statistics
    stats: Arc<RwLock<PublicationStatistics>>,
    /// Active subscriptions
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionHandle>>>,
}

/// NATS event subscriber with algebraic subject matching
// Debug derive removed due to trait object in handlers field
pub struct WorkflowEventSubscriber {
    /// NATS client
    client: Client,
    /// Subject patterns for subscription
    subject_patterns: Vec<Subject>,
    /// Event correlator for tracking
    correlator: Arc<WorkflowEventCorrelator>,
    /// Subscription statistics
    stats: Arc<RwLock<SubscriptionStatistics>>,
    /// Message handlers
    handlers: Arc<RwLock<HashMap<String, EventHandler>>>,
}

/// Combined publisher/subscriber for bidirectional communication
// Debug derive removed due to non-Debug trait objects
pub struct WorkflowEventBroker {
    /// Publisher instance
    publisher: WorkflowEventPublisher,
    /// Subscriber instance
    subscriber: WorkflowEventSubscriber,
    /// Broker configuration
    config: BrokerConfiguration,
}

/// Handle for managing active subscriptions
#[derive(Debug)]
pub struct SubscriptionHandle {
    /// Subscription ID
    pub id: String,
    /// Subject pattern
    pub subject: Subject,
    /// NATS subscriber
    pub subscriber: Subscriber,
    /// Subscription start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Message count
    pub message_count: u64,
}

/// Event handler function type
pub type EventHandler = Box<dyn Fn(WorkflowEvent, EventMetadata) -> EventHandlerResult + Send + Sync>;

/// Result of event handler execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandlerResult {
    /// Whether handling was successful
    pub success: bool,
    /// Response events to publish
    pub response_events: Vec<WorkflowEvent>,
    /// Handler execution time
    pub execution_time: chrono::Duration,
    /// Error message if failed
    pub error: Option<String>,
}

/// Metadata for published/received events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Message ID from NATS
    pub message_id: String,
    /// Subject the event was published to
    pub subject: String,
    /// Publication timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Publisher identification
    pub publisher_id: String,
    /// Message size in bytes
    pub message_size: usize,
    /// Reply subject for request/response
    pub reply_subject: Option<String>,
}

/// Statistics for event publication
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PublicationStatistics {
    /// Total events published
    pub events_published: u64,
    /// Publication errors
    pub publication_errors: u64,
    /// Average publication time
    pub avg_publication_time_us: u64,
    /// Events per subject
    pub events_per_subject: HashMap<String, u64>,
    /// Bytes published
    pub bytes_published: u64,
}

/// Statistics for event subscription
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SubscriptionStatistics {
    /// Total events received
    pub events_received: u64,
    /// Processing errors
    pub processing_errors: u64,
    /// Average processing time
    pub avg_processing_time_us: u64,
    /// Events per subscription
    pub events_per_subscription: HashMap<String, u64>,
    /// Bytes received
    pub bytes_received: u64,
}

/// Configuration for the event broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfiguration {
    /// NATS server URLs
    pub nats_urls: Vec<String>,
    /// Subject prefix for this broker
    pub subject_prefix: String,
    /// Publisher configuration
    pub publisher_config: PublisherConfiguration,
    /// Subscriber configuration  
    pub subscriber_config: SubscriberConfiguration,
    /// Correlation tracking settings
    pub correlation_config: CorrelationConfiguration,
}

/// Publisher-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherConfiguration {
    /// Publisher ID
    pub publisher_id: String,
    /// Message acknowledgment timeout
    pub ack_timeout_ms: u64,
    /// Max pending acknowledgments
    pub max_pending_acks: usize,
    /// Retry configuration
    pub retry_config: RetryConfiguration,
}

/// Subscriber-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriberConfiguration {
    /// Queue group for load balancing
    pub queue_group: Option<String>,
    /// Maximum concurrent handlers
    pub max_concurrent_handlers: usize,
    /// Message processing timeout
    pub processing_timeout_ms: u64,
    /// Buffer size for incoming messages
    pub buffer_size: usize,
}

/// Correlation tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationConfiguration {
    /// Enable automatic correlation tracking
    pub enable_tracking: bool,
    /// Default completion criteria
    pub default_completion_criteria: CompletionCriteria,
    /// Chain cleanup interval
    pub cleanup_interval_seconds: u64,
}

/// Retry configuration for failed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfiguration {
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl WorkflowEventPublisher {
    /// Create a new workflow event publisher
    pub async fn new(
        client: Client,
        subject_prefix: String,
        correlator: Arc<WorkflowEventCorrelator>,
    ) -> Result<Self, PublisherError> {
        Ok(Self {
            client,
            subject_prefix,
            correlator,
            stats: Arc::new(RwLock::new(PublicationStatistics::default())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Publish a workflow event using subject algebra
    pub async fn publish_event(
        &self,
        event: &WorkflowEvent,
        metadata: Option<EventMetadata>,
    ) -> Result<EventMetadata, PublisherError> {
        let start_time = std::time::Instant::now();

        // Generate subject from event using Subject Algebra
        let subject = Subject::from_event(event);
        let full_subject = format!("{}.{}", self.subject_prefix, subject.to_canonical_string());

        // Serialize event
        let payload = serde_json::to_vec(event)
            .map_err(|e| PublisherError::SerializationError(e.to_string()))?;

        // Create metadata
        let event_metadata = metadata.unwrap_or_else(|| EventMetadata {
            message_id: Uuid::new_v4().to_string(),
            subject: full_subject.clone(),
            timestamp: chrono::Utc::now(),
            publisher_id: "workflow-publisher".to_string(),
            message_size: payload.len(),
            reply_subject: None,
        });

        // Clone values for later use
        let payload_len = payload.len();
        let full_subject_clone = full_subject.clone();
        
        // Publish to NATS
        self.client
            .publish(full_subject, payload.into())
            .await
            .map_err(|e| PublisherError::PublicationError(e.to_string()))?;

        // Add event to correlator
        if let Err(e) = self.correlator.add_event(event.clone()) {
            // Log correlation error but don't fail publication
            eprintln!("Correlation error: {}", e);
        }

        // Update statistics
        let duration = start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.events_published += 1;
        stats.avg_publication_time_us = 
            (stats.avg_publication_time_us + duration.as_micros() as u64) / 2;
        *stats.events_per_subject.entry(full_subject_clone).or_insert(0) += 1;
        stats.bytes_published += payload_len as u64;

        Ok(event_metadata)
    }

    /// Publish multiple events as a batch
    pub async fn publish_batch(
        &self,
        events: &[WorkflowEvent],
    ) -> Result<Vec<EventMetadata>, PublisherError> {
        let mut results = Vec::new();
        
        for event in events {
            let metadata = self.publish_event(event, None).await?;
            results.push(metadata);
        }

        Ok(results)
    }

    /// Request-response pattern using workflow events
    pub async fn request_response(
        &self,
        request_event: &WorkflowEvent,
        response_subject_pattern: &str,
        timeout: chrono::Duration,
    ) -> Result<WorkflowEvent, PublisherError> {
        let subject = Subject::from_event(request_event);
        let full_subject = format!("{}.{}", self.subject_prefix, subject.to_canonical_string());

        // Set up response subscription
        let reply_subject = format!("_INBOX.{}", Uuid::new_v4());
        let mut subscriber = self.client
            .subscribe(reply_subject.clone())
            .await
            .map_err(|e| PublisherError::SubscriptionError(e.to_string()))?;

        // Publish request with reply subject
        let payload = serde_json::to_vec(request_event)
            .map_err(|e| PublisherError::SerializationError(e.to_string()))?;

        self.client
            .publish_with_reply(full_subject, reply_subject.clone(), payload.into())
            .await
            .map_err(|e| PublisherError::PublicationError(e.to_string()))?;

        // Wait for response with timeout
        let response_future = subscriber.next();
        let timeout_future = tokio::time::sleep(timeout.to_std().unwrap());

        match tokio::select! {
            response = response_future => response,
            _ = timeout_future => None,
        } {
            Some(message) => {
                let response_event: WorkflowEvent = serde_json::from_slice(&message.payload)
                    .map_err(|e| PublisherError::DeserializationError(e.to_string()))?;
                Ok(response_event)
            }
            None => Err(PublisherError::TimeoutError(
                "Response timeout exceeded".to_string()
            )),
        }
    }

    /// Get publication statistics
    pub async fn get_statistics(&self) -> PublicationStatistics {
        self.stats.read().await.clone()
    }
}

impl WorkflowEventSubscriber {
    /// Create a new workflow event subscriber
    pub async fn new(
        client: Client,
        correlator: Arc<WorkflowEventCorrelator>,
    ) -> Result<Self, SubscriberError> {
        Ok(Self {
            client,
            subject_patterns: Vec::new(),
            correlator,
            stats: Arc::new(RwLock::new(SubscriptionStatistics::default())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Subscribe to events matching subject pattern
    pub async fn subscribe(
        &mut self,
        subject_pattern: Subject,
        handler: EventHandler,
    ) -> Result<String, SubscriberError> {
        let subscription_id = Uuid::new_v4().to_string();
        let subject_string = subject_pattern.to_canonical_string();

        // Create NATS subscription
        let subscriber = self.client
            .subscribe(subject_string)
            .await
            .map_err(|e| SubscriberError::SubscriptionError(e.to_string()))?;

        // Store handler
        {
            let mut handlers = self.handlers.write().await;
            handlers.insert(subscription_id.clone(), handler);
        }

        // Start message processing task
        self.start_message_processing(subscription_id.clone(), subscriber).await;

        self.subject_patterns.push(subject_pattern);
        Ok(subscription_id)
    }

    /// Subscribe to hierarchical subject pattern
    pub async fn subscribe_hierarchical(
        &mut self,
        base_subject: &str,
        handler: EventHandler,
    ) -> Result<String, SubscriberError> {
        let wildcard_subject = format!("{}.*", base_subject);
        let subject_pattern = Subject::from_str(&wildcard_subject)
            .map_err(|e| SubscriberError::InvalidSubject(e.to_string()))?;
        
        self.subscribe(subject_pattern, handler).await
    }

    /// Subscribe to cross-domain events
    pub async fn subscribe_cross_domain(
        &mut self,
        source_domain: &str,
        target_domain: &str,
        handler: EventHandler,
    ) -> Result<String, SubscriberError> {
        let cross_domain_subject = SubjectBuilder::new()
            .domain("integration")
            .context("cross_domain")
            .event_type("coordination")
            .any_specificity()
            .any_correlation()
            .build()
            .map_err(|e| SubscriberError::InvalidSubject(e.to_string()))?;

        self.subscribe(cross_domain_subject, handler).await
    }

    /// Start processing messages for a subscription
    async fn start_message_processing(&self, subscription_id: String, mut subscriber: Subscriber) {
        let handlers = self.handlers.clone();
        let stats = self.stats.clone();
        let correlator = self.correlator.clone();

        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                let start_time = std::time::Instant::now();

                // Deserialize event
                match serde_json::from_slice::<WorkflowEvent>(&message.payload) {
                    Ok(event) => {
                        // Create event metadata
                        let metadata = EventMetadata {
                            message_id: Uuid::new_v4().to_string(),
                            subject: message.subject.to_string(),
                            timestamp: chrono::Utc::now(),
                            publisher_id: "unknown".to_string(),
                            message_size: message.payload.len(),
                            reply_subject: message.reply.map(|s| s.to_string()),
                        };

                        // Get handler and process
                        let handlers_read = handlers.read().await;
                        if let Some(handler) = handlers_read.get(&subscription_id) {
                            let result = handler(event.clone(), metadata);

                            // Add event to correlator
                            if let Err(e) = correlator.add_event(event) {
                                eprintln!("Correlation error in subscriber: {}", e);
                            }

                            // Handle response events
                            for response_event in result.response_events {
                                // Could publish response events here if needed
                            }

                            // Update statistics
                            let duration = start_time.elapsed();
                            let mut stats_write = stats.write().await;
                            stats_write.events_received += 1;
                            if !result.success {
                                stats_write.processing_errors += 1;
                            }
                            stats_write.avg_processing_time_us = 
                                (stats_write.avg_processing_time_us + duration.as_micros() as u64) / 2;
                            *stats_write.events_per_subscription.entry(subscription_id.clone()).or_insert(0) += 1;
                            stats_write.bytes_received += message.payload.len() as u64;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize event: {}", e);
                        let mut stats_write = stats.write().await;
                        stats_write.processing_errors += 1;
                    }
                }
            }
        });
    }

    /// Get subscription statistics
    pub async fn get_statistics(&self) -> SubscriptionStatistics {
        self.stats.read().await.clone()
    }
}

impl WorkflowEventBroker {
    /// Create a new workflow event broker
    pub async fn new(config: BrokerConfiguration) -> Result<Self, BrokerError> {
        // Connect to NATS
        let client = async_nats::connect(&config.nats_urls.join(","))
            .await
            .map_err(|e| BrokerError::ConnectionError(e.to_string()))?;

        // Create correlator
        let correlator = Arc::new(WorkflowEventCorrelator::new());

        // Create publisher
        let publisher = WorkflowEventPublisher::new(
            client.clone(),
            config.subject_prefix.clone(),
            correlator.clone(),
        ).await
        .map_err(BrokerError::PublisherError)?;

        // Create subscriber
        let subscriber = WorkflowEventSubscriber::new(client, correlator)
            .await
            .map_err(BrokerError::SubscriberError)?;

        Ok(Self {
            publisher,
            subscriber,
            config,
        })
    }

    /// Get publisher instance
    pub fn publisher(&self) -> &WorkflowEventPublisher {
        &self.publisher
    }

    /// Get subscriber instance
    pub fn subscriber(&mut self) -> &mut WorkflowEventSubscriber {
        &mut self.subscriber
    }

    /// Get broker configuration
    pub fn config(&self) -> &BrokerConfiguration {
        &self.config
    }

    /// Publish event with automatic correlation tracking
    pub async fn publish_with_correlation(
        &self,
        event: WorkflowEvent,
        completion_criteria: Option<CompletionCriteria>,
    ) -> Result<EventMetadata, BrokerError> {
        // Start correlation chain if not exists
        if let Some(criteria) = completion_criteria {
            if let Err(e) = self.publisher.correlator.start_chain(event.correlation_id, criteria) {
                // Chain might already exist, which is ok
                match e {
                    crate::messaging::correlation::CorrelationError::ChainAlreadyExists(_) => {},
                    other => return Err(BrokerError::CorrelationError(other.to_string())),
                }
            }
        }

        // Publish event
        self.publisher.publish_event(&event, None)
            .await
            .map_err(BrokerError::PublisherError)
    }
}

/// Subject parsing helper for external integrations
impl Subject {
    /// Parse subject from string (implementing FromStr)
    pub fn from_str(s: &str) -> Result<Self, SubjectParseError> {
        s.parse()
    }
}

/// Error types for publishing operations
#[derive(Debug, thiserror::Error)]
pub enum PublisherError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Publication error: {0}")]
    PublicationError(String),

    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

/// Error types for subscription operations
#[derive(Debug, thiserror::Error)]
pub enum SubscriberError {
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    #[error("Invalid subject: {0}")]
    InvalidSubject(String),

    #[error("Handler error: {0}")]
    HandlerError(String),
}

/// Error types for broker operations
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Publisher error: {0}")]
    PublisherError(#[from] PublisherError),

    #[error("Subscriber error: {0}")]
    SubscriberError(#[from] SubscriberError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Correlation error: {0}")]
    CorrelationError(String),
}

/// Subject parsing error (re-export for convenience)
pub use crate::algebra::subject_algebra::SubjectParseError;

impl Default for BrokerConfiguration {
    fn default() -> Self {
        Self {
            nats_urls: vec!["nats://localhost:4222".to_string()],
            subject_prefix: "cim".to_string(),
            publisher_config: PublisherConfiguration::default(),
            subscriber_config: SubscriberConfiguration::default(),
            correlation_config: CorrelationConfiguration::default(),
        }
    }
}

impl Default for PublisherConfiguration {
    fn default() -> Self {
        Self {
            publisher_id: "workflow-publisher".to_string(),
            ack_timeout_ms: 5000,
            max_pending_acks: 1000,
            retry_config: RetryConfiguration::default(),
        }
    }
}

impl Default for SubscriberConfiguration {
    fn default() -> Self {
        Self {
            queue_group: None,
            max_concurrent_handlers: 100,
            processing_timeout_ms: 30000,
            buffer_size: 1000,
        }
    }
}

impl Default for CorrelationConfiguration {
    fn default() -> Self {
        Self {
            enable_tracking: true,
            default_completion_criteria: CompletionCriteria::default(),
            cleanup_interval_seconds: 300, // 5 minutes
        }
    }
}

impl Default for RetryConfiguration {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::event_algebra::*;

    // Note: These tests require a running NATS server for full integration testing
    // They serve as documentation of the API usage patterns

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_event_publication() {
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let correlator = Arc::new(WorkflowEventCorrelator::new());
        let publisher = WorkflowEventPublisher::new(client, "test".to_string(), correlator)
            .await.unwrap();

        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "workflow".to_string(),
            Uuid::new_v4(),
            EventPayload::empty(),
            EventContext::empty(),
        );

        let result = publisher.publish_event(&event, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_event_subscription() {
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let correlator = Arc::new(WorkflowEventCorrelator::new());
        let mut subscriber = WorkflowEventSubscriber::new(client, correlator).await.unwrap();

        let subject = SubjectBuilder::new()
            .domain("workflow")
            .any_context()
            .event_type("lifecycle")
            .any_specificity()
            .any_correlation()
            .build()
            .unwrap();

        let handler = Box::new(|event: WorkflowEvent, _metadata: EventMetadata| -> EventHandlerResult {
            EventHandlerResult {
                success: true,
                response_events: vec![],
                execution_time: chrono::Duration::milliseconds(10),
                error: None,
            }
        });

        let result = subscriber.subscribe(subject, handler).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_broker_configuration() {
        let config = BrokerConfiguration::default();
        assert_eq!(config.subject_prefix, "cim");
        assert_eq!(config.nats_urls, vec!["nats://localhost:4222"]);
    }
}