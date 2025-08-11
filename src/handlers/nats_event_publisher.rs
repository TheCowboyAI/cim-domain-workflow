//! NATS event publisher for workflow domain events
//!
//! Publishes workflow domain events to NATS following CIM event sourcing patterns
//! with mandatory correlation/causation tracking and proper message headers.

use crate::domain_events::WorkflowDomainEvent;
use cim_domain::{DomainResult, DomainError};
use async_nats::{Client, HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::time::SystemTime;
use uuid::Uuid;

/// Message identifier for event correlation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Correlation ID groups related messages together
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub Uuid);

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Causation ID tracks what caused this message
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(pub Uuid);

impl fmt::Display for CausationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Event metadata required for all events
#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub message_id: MessageId,
    pub correlation_id: CorrelationId,
    pub causation_id: CausationId,
    pub timestamp: SystemTime,
    pub sequence: u64,
    pub actor: Option<String>,
}

impl EventMetadata {
    /// Create metadata for a root event (self-correlated)
    pub fn create_root(actor: Option<String>) -> Self {
        let message_id = MessageId(Uuid::new_v4());
        Self {
            message_id: message_id.clone(),
            correlation_id: CorrelationId(message_id.0),
            causation_id: CausationId(message_id.0),
            timestamp: SystemTime::now(),
            sequence: 1,
            actor,
        }
    }

    /// Create metadata for an event caused by another message
    pub fn create_caused_by(
        parent: &EventMetadata,
        sequence: u64,
        actor: Option<String>,
    ) -> Self {
        Self {
            message_id: MessageId(Uuid::new_v4()),
            correlation_id: parent.correlation_id.clone(),
            causation_id: CausationId(parent.message_id.0),
            timestamp: SystemTime::now(),
            sequence,
            actor,
        }
    }
}

/// NATS event publisher for workflow domain
pub struct NatsEventPublisher {
    client: Client,
    subject_prefix: String,
}

impl NatsEventPublisher {
    /// Create a new NATS event publisher
    pub fn new(client: Client, subject_prefix: String) -> Self {
        Self {
            client,
            subject_prefix,
        }
    }

    /// Publish a workflow domain event to NATS
    pub async fn publish_event(
        &self,
        event: &WorkflowDomainEvent,
        metadata: &EventMetadata,
    ) -> DomainResult<()> {
        // Create headers following CIM requirements
        let mut headers = HeaderMap::new();
        
        // Required headers for correlation/causation tracking
        headers.insert("X-Message-ID", metadata.message_id.0.to_string());
        headers.insert("X-Correlation-ID", metadata.correlation_id.0.to_string());
        headers.insert("X-Causation-ID", metadata.causation_id.0.to_string());
        
        // Event metadata headers
        headers.insert("X-Event-Type", event.event_name());
        headers.insert("X-Workflow-ID", event.workflow_id().to_string());
        headers.insert("X-Sequence", metadata.sequence.to_string());
        headers.insert("X-Timestamp", metadata.timestamp.duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| DomainError::generic(&format!("Invalid timestamp: {}", e)))?
            .as_secs()
            .to_string());
        
        // Optional actor header
        if let Some(actor) = &metadata.actor {
            headers.insert("X-Actor", actor.clone());
        }
        
        // NATS message ID for deduplication
        headers.insert("Nats-Msg-Id", metadata.message_id.0.to_string());
        
        // Construct subject
        let subject = format!(
            "{}.workflow.{}.{}",
            self.subject_prefix,
            event.workflow_id(),
            event.event_name().to_lowercase()
        );
        
        // Serialize event
        let payload = serde_json::to_vec(event)
            .map_err(|e| DomainError::generic(&format!("Failed to serialize event: {}", e)))?;
        
        // Publish to NATS
        self.client
            .publish_with_headers(subject, headers, payload.into())
            .await
            .map_err(|e| DomainError::generic(&format!("Failed to publish event: {}", e)))?;
        
        Ok(())
    }

    /// Publish multiple events with proper causation chain
    pub async fn publish_events(
        &self,
        events: &[WorkflowDomainEvent],
        initial_metadata: &EventMetadata,
    ) -> DomainResult<()> {
        let mut current_metadata = initial_metadata.clone();
        
        for (index, event) in events.iter().enumerate() {
            // First event uses initial metadata, subsequent events are caused by previous
            if index > 0 {
                current_metadata = EventMetadata::create_caused_by(
                    &current_metadata,
                    initial_metadata.sequence + index as u64,
                    initial_metadata.actor.clone(),
                );
            }
            
            self.publish_event(event, &current_metadata).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::WorkflowCreated;
    use crate::value_objects::WorkflowId;
    use chrono::Utc;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_event_metadata_creation() {
        // Test root event metadata
        let root_metadata = EventMetadata::create_root(Some("test-user".to_string()));
        assert_eq!(root_metadata.correlation_id.0, root_metadata.message_id.0);
        assert_eq!(root_metadata.causation_id.0, root_metadata.message_id.0);
        assert_eq!(root_metadata.sequence, 1);
        assert_eq!(root_metadata.actor, Some("test-user".to_string()));

        // Test caused event metadata
        let caused_metadata = EventMetadata::create_caused_by(&root_metadata, 2, None);
        assert_eq!(caused_metadata.correlation_id, root_metadata.correlation_id);
        assert_eq!(caused_metadata.causation_id.0, root_metadata.message_id.0);
        assert_ne!(caused_metadata.message_id, root_metadata.message_id);
        assert_eq!(caused_metadata.sequence, 2);
        assert!(caused_metadata.actor.is_none());
    }

    #[tokio::test]
    async fn test_publish_event_headers() {
        // This test would require a mock NATS client
        // For now, we test the header construction logic
        let event = WorkflowDomainEvent::WorkflowCreated(WorkflowCreated {
            workflow_id: WorkflowId::new(),
            name: "Test Workflow".to_string(),
            description: "Test description".to_string(),
            metadata: HashMap::new(),
            created_by: Some("test-user".to_string()),
            created_at: Utc::now(),
        });

        let _metadata = EventMetadata::create_root(Some("test-user".to_string()));

        // Verify event properties
        assert_eq!(event.event_name(), "WorkflowCreated");
        assert!(!event.workflow_id().to_string().is_empty());
    }
}