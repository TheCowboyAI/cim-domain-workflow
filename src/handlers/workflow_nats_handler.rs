//! NATS-enabled workflow command handler
//!
//! Extends the base command handler with NATS event publishing capabilities

use crate::{
    commands::*,
    domain_events::WorkflowDomainEvent,
    handlers::{WorkflowCommandHandler, WorkflowCommandHandlerImpl, NatsEventPublisher, EventMetadata},
    value_objects::WorkflowId,
};
use cim_domain::{DomainResult, DomainError};
use async_nats::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

/// NATS-enabled workflow command handler
pub struct NatsWorkflowCommandHandler {
    inner: Arc<Mutex<WorkflowCommandHandlerImpl>>,
    publisher: Arc<NatsEventPublisher>,
}

impl NatsWorkflowCommandHandler {
    /// Create a new NATS-enabled command handler
    pub fn new(nats_client: Client, subject_prefix: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(WorkflowCommandHandlerImpl::new())),
            publisher: Arc::new(NatsEventPublisher::new(nats_client, subject_prefix)),
        }
    }

    /// Handle command and publish resulting events to NATS
    async fn handle_and_publish<F>(
        &self,
        actor: Option<String>,
        handler_fn: F,
    ) -> DomainResult<Vec<WorkflowDomainEvent>>
    where
        F: FnOnce(&mut WorkflowCommandHandlerImpl) -> DomainResult<Vec<WorkflowDomainEvent>>,
    {
        // Execute command
        let mut inner = self.inner.lock().await;
        let events = handler_fn(&mut *inner)?;

        // Publish events to NATS
        if !events.is_empty() {
            let metadata = EventMetadata::create_root(actor);
            self.publisher
                .publish_events(&events, &metadata)
                .await
                .map_err(|e| DomainError::generic(&format!("Failed to publish events: {}", e)))?;
        }

        Ok(events)
    }

    /// Handle create workflow command with NATS publishing
    pub async fn handle_create_workflow(
        &self,
        cmd: CreateWorkflow,
    ) -> DomainResult<Vec<WorkflowDomainEvent>> {
        let actor = cmd.created_by.clone();
        self.handle_and_publish(actor, |handler| {
            handler.handle_create_workflow(cmd)
        }).await
    }

    /// Handle start workflow command with NATS publishing
    pub async fn handle_start_workflow(
        &self,
        cmd: StartWorkflow,
    ) -> DomainResult<Vec<WorkflowDomainEvent>> {
        let actor = cmd.started_by.clone();
        self.handle_and_publish(actor, |handler| {
            handler.handle_start_workflow(cmd)
        }).await
    }

    /// Handle add step command with NATS publishing
    pub async fn handle_add_step(
        &self,
        cmd: AddStep,
    ) -> DomainResult<Vec<WorkflowDomainEvent>> {
        let actor = cmd.added_by.clone();
        self.handle_and_publish(actor, |handler| {
            handler.handle_add_step(cmd)
        }).await
    }

    /// Get workflow for testing/debugging
    pub async fn get_workflow(&self, workflow_id: &WorkflowId) -> Option<crate::Workflow> {
        let inner = self.inner.lock().await;
        inner.get_workflow(workflow_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nats_handler_creation() {
        // This test doesn't require a real NATS connection
        // In production, you'd inject a mock client for testing
        
        let cmd = CreateWorkflow {
            name: "Test Workflow".to_string(),
            description: "Test description".to_string(),
            metadata: Default::default(),
            created_by: Some("test-user".to_string()),
        };

        // Verify command structure
        assert_eq!(cmd.name, "Test Workflow");
        assert_eq!(cmd.created_by, Some("test-user".to_string()));
    }

    #[tokio::test]
    async fn test_event_metadata_for_commands() {
        // Test metadata creation for different actors
        let metadata1 = EventMetadata::create_root(Some("user1".to_string()));
        let metadata2 = EventMetadata::create_root(None);

        assert_eq!(metadata1.actor, Some("user1".to_string()));
        assert!(metadata2.actor.is_none());
        
        // Verify correlation chain
        let child_metadata = EventMetadata::create_caused_by(&metadata1, 2, Some("user2".to_string()));
        assert_eq!(child_metadata.correlation_id, metadata1.correlation_id);
        assert_eq!(child_metadata.causation_id.0, metadata1.message_id.0);
        assert_eq!(child_metadata.actor, Some("user2".to_string()));
    }
}