//! Document Domain Composition Example
//!
//! Simplified example demonstrating how the workflow system can be used
//! from a document domain perspective with basic event composition.

use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use cim_domain_workflow::{
    
    // Algebraic operations
    algebra::{
        WorkflowEvent, EventType, LifecycleEventType, StepEventType,
        EventPayload, EventContext,
    },
    
    // Primitives
    primitives::{UniversalWorkflowId, WorkflowInstanceId, WorkflowContext},
};

/// Document-specific value objects that would come from cim-domain-document
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct DocumentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub content_type: String,
    pub size_bytes: u64,
}

/// Document domain events that compose with the workflow system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentDomainEvent {
    DocumentCreated {
        document_id: DocumentId,
        metadata: DocumentMetadata,
        created_by: Uuid,
    },
    DocumentReviewRequested {
        document_id: DocumentId,
        reviewer: Uuid,
        deadline: chrono::DateTime<chrono::Utc>,
    },
    DocumentApproved {
        document_id: DocumentId,
        approved_by: Uuid,
        approval_notes: Option<String>,
    },
    DocumentRejected {
        document_id: DocumentId,
        rejected_by: Uuid,
        rejection_reason: String,
    },
}

/// Simple document workflow service
pub struct DocumentWorkflowService {
    workflows: HashMap<DocumentId, UniversalWorkflowId>,
}

impl DocumentWorkflowService {
    /// Create a new document workflow service
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }
    
    /// Start a document review workflow using simple composition
    pub fn start_document_review(
        &mut self,
        document_id: DocumentId,
        reviewer_id: Uuid,
    ) -> Result<WorkflowInstanceId, Box<dyn std::error::Error>> {
        // Create workflow ID
        let workflow_id = UniversalWorkflowId::new("document".to_string(), Some("review".to_string()));
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        
        // Store the mapping
        self.workflows.insert(document_id.clone(), workflow_id);
        
        println!("Started document review workflow for document: {:?}", document_id);
        println!("Assigned reviewer: {}", reviewer_id);
        
        Ok(instance_id)
    }
    
    /// Demonstrate simple event composition for document workflow
    pub fn compose_document_events(
        &self,
        document_id: DocumentId,
        reviewers: Vec<Uuid>,
    ) -> Result<Vec<WorkflowEvent>, Box<dyn std::error::Error>> {
        let correlation_id = Uuid::new_v4();
        let workflow_id = UniversalWorkflowId::new("document".to_string(), Some("approval".to_string()));
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let _context = WorkflowContext::new(workflow_id, instance_id.clone(), Some("document-service".to_string()));
        
        let mut events = Vec::new();
        
        // Create initial document created event
        let document_created = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "document".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("workflow_type".to_string(), serde_json::json!("approval"));
                payload
            },
            EventContext::for_workflow(instance_id.id().clone()),
        );
        events.push(document_created);
        
        // Create review request events for each reviewer
        for reviewer in &reviewers {
            let review_event = WorkflowEvent::step(
                StepEventType::StepCreated,
                "document".to_string(),
                correlation_id,
                {
                    let mut payload = EventPayload::empty();
                    payload.set_data("step_type".to_string(), serde_json::json!("review"));
                    payload.set_data("reviewer_id".to_string(), serde_json::json!(reviewer.to_string()));
                    payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                    payload
                },
                EventContext::for_workflow(instance_id.id().clone()),
            );
            events.push(review_event);
        }
        
        Ok(events)
    }
    
    /// Convert document domain events to workflow events
    pub fn convert_domain_event(
        &self,
        domain_event: DocumentDomainEvent,
        correlation_id: Uuid,
    ) -> Result<WorkflowEvent, Box<dyn std::error::Error>> {
        let workflow_id = UniversalWorkflowId::new("document".to_string(), None);
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        
        let (event_type, mut payload) = match domain_event {
            DocumentDomainEvent::DocumentCreated { document_id, metadata, created_by } => {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("metadata".to_string(), serde_json::to_value(metadata)?);
                payload.set_data("created_by".to_string(), serde_json::json!(created_by.to_string()));
                (EventType::Lifecycle(LifecycleEventType::WorkflowCreated), payload)
            },
            DocumentDomainEvent::DocumentReviewRequested { document_id, reviewer, deadline } => {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("reviewer".to_string(), serde_json::json!(reviewer.to_string()));
                payload.set_data("deadline".to_string(), serde_json::json!(deadline.to_rfc3339()));
                (EventType::Step(StepEventType::StepCreated), payload)
            },
            DocumentDomainEvent::DocumentApproved { document_id, approved_by, approval_notes } => {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("approved_by".to_string(), serde_json::json!(approved_by.to_string()));
                if let Some(notes) = approval_notes {
                    payload.set_data("approval_notes".to_string(), serde_json::json!(notes));
                }
                (EventType::Step(StepEventType::StepCompleted), payload)
            },
            DocumentDomainEvent::DocumentRejected { document_id, rejected_by, rejection_reason } => {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("rejected_by".to_string(), serde_json::json!(rejected_by.to_string()));
                payload.set_data("rejection_reason".to_string(), serde_json::json!(rejection_reason));
                (EventType::Step(StepEventType::StepFailed), payload)
            },
        };
        
        // Add domain context
        payload.set_data("source_domain".to_string(), serde_json::json!("document"));
        
        Ok(WorkflowEvent::new(
            event_type,
            "document".to_string(),
            correlation_id,
            payload,
            EventContext::for_workflow(instance_id.id().clone()),
        ))
    }
}

/// Example usage of document domain composition
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Document Domain Composition Example");
    println!("======================================");
    
    // Create document workflow service
    println!("📝 Initializing document workflow service...");
    let mut service = DocumentWorkflowService::new();
    
    // Example 1: Simple workflow creation
    println!("\n📋 Example 1: Starting document review workflow");
    let document_id = DocumentId(Uuid::new_v4());
    let reviewer_id = Uuid::new_v4();
    
    match service.start_document_review(document_id.clone(), reviewer_id) {
        Ok(instance_id) => {
            println!("✅ Document review workflow started with instance ID: {:?}", instance_id);
        },
        Err(e) => {
            println!("❌ Failed to start workflow: {}", e);
        }
    }
    
    // Example 2: Event composition
    println!("\n🔄 Example 2: Event composition for document approval flow");
    let reviewers = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    
    match service.compose_document_events(document_id.clone(), reviewers) {
        Ok(composed_events) => {
            println!("✅ Composed {} events for document approval flow", composed_events.len());
            for (i, event) in composed_events.iter().enumerate() {
                println!("   Event {}: {} in domain '{}'", i + 1, event.type_name(), event.domain);
            }
        },
        Err(e) => {
            println!("❌ Failed to compose events: {}", e);
        }
    }
    
    // Example 3: Domain event conversion
    println!("\n📤 Example 3: Converting domain events to workflow events");
    let domain_event = DocumentDomainEvent::DocumentCreated {
        document_id: document_id.clone(),
        metadata: DocumentMetadata {
            title: "Important Document".to_string(),
            author: "Alice Smith".to_string(),
            content_type: "application/pdf".to_string(),
            size_bytes: 1024000,
        },
        created_by: Uuid::new_v4(),
    };
    
    match service.convert_domain_event(domain_event, Uuid::new_v4()) {
        Ok(workflow_event) => {
            println!("✅ Converted domain event to workflow event: {}", workflow_event.type_name());
        },
        Err(e) => {
            println!("❌ Failed to convert event: {}", e);
        }
    }
    
    println!("\n🎯 Document domain composition examples completed!");
    println!("This demonstrates basic workflow composition patterns for document processing.");
    
    Ok(())
}