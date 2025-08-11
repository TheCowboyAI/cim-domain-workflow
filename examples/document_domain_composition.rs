//! Document Domain Composition Example
//!
//! Demonstrates how a document domain can compose with the unified workflow system
//! to leverage templates, algebraic event composition, and NATS coordination
//! without being tightly coupled to the workflow implementation.

use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use cim_domain_workflow::{
    // Composition framework
    composition::{
        WorkflowTemplate, TemplateId, TemplateVersion, TemplateInstantiationRequest,
        TemplateInstantiationEngine, TemplateParameter, TemplateStep, TemplateStepType,
        ParameterType, InMemoryTemplateRepository,
    },
    
    // Core workflow engine
    core::{CoreTemplateEngine, TemplateExecutionCoordinator},
    
    // Algebraic operations
    algebra::{
        WorkflowEvent, EventType, LifecycleEventType, StepEventType,
        EventPayload, EventContext, WorkflowEventAlgebra,
        SequentialComposition, ParallelComposition,
    },
    
    // NATS messaging
    messaging::{
        WorkflowEventPublisher, WorkflowEventSubscriber, WorkflowEventBroker,
        BrokerConfiguration, EventHandler,
    },
    
    // Primitives
    primitives::{UniversalWorkflowId, WorkflowInstanceId, WorkflowContext},
};

/// Document-specific value objects that would come from cim-domain-document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion(pub u32);

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
    DocumentUpdated {
        document_id: DocumentId,
        version: DocumentVersion,
        updated_by: Uuid,
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

/// Document domain service that composes with workflow system
pub struct DocumentWorkflowService {
    template_engine: CoreTemplateEngine,
    event_broker: std::sync::Arc<WorkflowEventBroker>,
    algebra: WorkflowEventAlgebra,
}

impl DocumentWorkflowService {
    /// Create a new document workflow service
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Set up template repository
        let template_repository = std::sync::Arc::new(InMemoryTemplateRepository::new());
        
        // Create a placeholder workflow engine (in real implementation, this would be your engine)
        let workflow_engine = Box::new(PlaceholderWorkflowEngine);
        
        // Set up NATS broker configuration
        let broker_config = BrokerConfiguration {
            nats_urls: vec!["nats://localhost:4222".to_string()],
            subject_prefix: "cim.document".to_string(),
            ..Default::default()
        };
        
        // Create event broker
        let event_broker = std::sync::Arc::new(
            WorkflowEventBroker::new(broker_config).await?
        );
        
        // Create template engine
        let template_engine = CoreTemplateEngine::new(
            template_repository,
            workflow_engine,
            event_broker.clone(),
        ).await?;
        
        // Initialize with document-specific templates
        template_engine.initialize_standard_templates().await?;
        
        Ok(Self {
            template_engine,
            event_broker,
            algebra: WorkflowEventAlgebra,
        })
    }
    
    /// Start a document review workflow using templates
    pub async fn start_document_review(
        &self,
        document_id: DocumentId,
        reviewer_id: Uuid,
        deadline_hours: u32,
    ) -> Result<WorkflowInstanceId, Box<dyn std::error::Error>> {
        // Create template instantiation request
        let template_id = TemplateId::new(
            "document".to_string(),
            "review".to_string(),
            TemplateVersion::new(1, 0, 0),
        );
        
        let mut parameters = HashMap::new();
        parameters.insert("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
        parameters.insert("reviewer_id".to_string(), serde_json::json!(reviewer_id.to_string()));
        parameters.insert("deadline_hours".to_string(), serde_json::json!(deadline_hours));
        
        let request = TemplateInstantiationRequest {
            template_id,
            parameters,
            target_domain: "document".to_string(),
            correlation_id: Uuid::new_v4(),
            context: HashMap::new(),
        };
        
        // Execute the template
        let execution_result = self.template_engine.execute_template(request).await?;
        
        Ok(execution_result.instantiation_result.instance_id)
    }
    
    /// Demonstrate algebraic composition of document events
    pub async fn compose_document_approval_flow(
        &self,
        document_id: DocumentId,
        reviewers: Vec<Uuid>,
    ) -> Result<Vec<WorkflowEvent>, Box<dyn std::error::Error>> {
        let correlation_id = Uuid::new_v4();
        let workflow_id = UniversalWorkflowId::new("document".to_string(), Some("approval".to_string()));
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id, Some("document-service".to_string()));
        
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
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Create review request events for each reviewer (parallel composition)
        let mut review_events = Vec::new();
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
                EventContext::for_workflow(*context.instance_id.id()),
            );
            review_events.push(review_event);
        }
        
        // Use algebraic operations to compose the workflow
        let mut composed_events = vec![document_created];
        
        // Compose review events in parallel
        if review_events.len() > 1 {
            let mut parallel_result = review_events[0].clone();
            for review_event in review_events.iter().skip(1) {
                let composition_result = self.algebra.compose_parallel(
                    parallel_result,
                    review_event.clone(),
                    &context,
                ).await?;
                parallel_result = composition_result.result;
                composed_events.extend(composition_result.events);
            }
        } else if let Some(review_event) = review_events.into_iter().next() {
            composed_events.push(review_event);
        }
        
        // Create final approval event
        let approval_event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCompleted,
            "document".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("approval_status".to_string(), serde_json::json!("pending"));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Compose approval event sequentially after reviews
        if let Some(last_event) = composed_events.last() {
            let sequential_result = self.algebra.compose_sequential(
                last_event.clone(),
                approval_event,
                &context,
            ).await?;
            composed_events.extend(sequential_result.events);
        }
        
        Ok(composed_events)
    }
    
    /// Publish document domain events through the workflow system
    pub async fn publish_document_event(
        &self,
        domain_event: DocumentDomainEvent,
        correlation_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert domain event to workflow event
        let workflow_event = self.convert_domain_event_to_workflow_event(domain_event, correlation_id)?;
        
        // Publish through the workflow event broker
        self.event_broker.publisher().publish_event(&workflow_event, None).await?;
        
        Ok(())
    }
    
    /// Convert document domain events to workflow events
    fn convert_domain_event_to_workflow_event(
        &self,
        domain_event: DocumentDomainEvent,
        correlation_id: Uuid,
    ) -> Result<WorkflowEvent, Box<dyn std::error::Error>> {
        let workflow_id = UniversalWorkflowId::new("document".to_string(), None);
        let instance_id = WorkflowInstanceId::new(workflow_id);
        
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
            DocumentDomainEvent::DocumentUpdated { document_id, version, updated_by } => {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("version".to_string(), serde_json::json!(version.0));
                payload.set_data("updated_by".to_string(), serde_json::json!(updated_by.to_string()));
                (EventType::Step(StepEventType::StepUpdated), payload)
            },
        };
        
        // Add domain context
        payload.set_data("source_domain".to_string(), serde_json::json!("document"));
        
        Ok(WorkflowEvent::new(
            event_type,
            "document".to_string(),
            correlation_id,
            payload,
            EventContext::for_workflow(*instance_id.id()),
        ))
    }
}

/// Create document review template for composition
pub fn create_document_review_template() -> WorkflowTemplate {
    use cim_domain_workflow::composition::*;
    
    WorkflowTemplate {
        id: TemplateId::new(
            "document".to_string(),
            "review".to_string(),
            TemplateVersion::new(1, 0, 0),
        ),
        name: "Document Review Workflow".to_string(),
        description: "Template for document review and approval process".to_string(),
        version: TemplateVersion::new(1, 0, 0),
        target_domains: vec!["document".to_string()],
        parameters: vec![
            (
                "document_id".to_string(),
                TemplateParameter {
                    name: "document_id".to_string(),
                    param_type: ParameterType::String,
                    description: "ID of the document to review".to_string(),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
            ),
            (
                "reviewer_id".to_string(),
                TemplateParameter {
                    name: "reviewer_id".to_string(),
                    param_type: ParameterType::String,
                    description: "ID of the assigned reviewer".to_string(),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
            ),
            (
                "deadline_hours".to_string(),
                TemplateParameter {
                    name: "deadline_hours".to_string(),
                    param_type: ParameterType::Integer,
                    description: "Hours until review deadline".to_string(),
                    required: false,
                    default_value: Some(serde_json::json!(48)),
                    constraints: vec![],
                },
            ),
        ].into_iter().collect(),
        steps: vec![
            TemplateStep {
                id: "notify_reviewer".to_string(),
                name_template: "Notify Reviewer".to_string(),
                description_template: "Send notification to {reviewer_id} about document {document_id}".to_string(),
                step_type: TemplateStepType::Automated,
                dependencies: vec![],
                configuration: vec![
                    ("notification_type".to_string(), serde_json::json!("email")),
                    ("template".to_string(), serde_json::json!("document_review_request")),
                ].into_iter().collect(),
                condition: None,
                retry_policy: None,
            },
            TemplateStep {
                id: "await_review".to_string(),
                name_template: "Await Review".to_string(),
                description_template: "Wait for review decision from {reviewer_id}".to_string(),
                step_type: TemplateStepType::Manual,
                dependencies: vec!["notify_reviewer".to_string()],
                configuration: vec![
                    ("timeout_hours".to_string(), serde_json::json!("{deadline_hours}")),
                    ("allowed_outcomes".to_string(), serde_json::json!(["approved", "rejected", "needs_changes"])),
                ].into_iter().collect(),
                condition: None,
                retry_policy: None,
            },
            TemplateStep {
                id: "process_outcome".to_string(),
                name_template: "Process Review Outcome".to_string(),
                description_template: "Handle the review decision and next steps".to_string(),
                step_type: TemplateStepType::Conditional,
                dependencies: vec!["await_review".to_string()],
                configuration: HashMap::new(),
                condition: None,
                retry_policy: None,
            },
        ],
        constraints: vec![],
        metadata: TemplateMetadata {
            author: "Document Domain".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            tags: vec!["document".to_string(), "review".to_string(), "approval".to_string()],
            category: "Document Management".to_string(),
            documentation_url: None,
            examples: vec![],
        },
        validation_rules: vec![],
    }
}

/// Placeholder workflow engine for demonstration
struct PlaceholderWorkflowEngine;

#[async_trait::async_trait]
impl cim_domain_workflow::core::WorkflowEngine for PlaceholderWorkflowEngine {
    async fn execute(
        &self,
        _workflow_id: UniversalWorkflowId,
        _instance_id: WorkflowInstanceId,
        _context: WorkflowContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        println!("Executing workflow in document domain");
        Ok(())
    }
}

/// Example usage of document domain composition
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Document Domain Composition Example");
    println!("======================================");
    
    // Create document workflow service
    println!("📝 Initializing document workflow service...");
    let service = DocumentWorkflowService::new().await?;
    
    // Example 1: Template-based workflow
    println!("\n📋 Example 1: Starting document review workflow using templates");
    let document_id = DocumentId(Uuid::new_v4());
    let reviewer_id = Uuid::new_v4();
    
    match service.start_document_review(document_id.clone(), reviewer_id, 48).await {
        Ok(instance_id) => {
            println!("✅ Document review workflow started with instance ID: {:?}", instance_id);
        },
        Err(e) => {
            println!("❌ Failed to start workflow (likely NATS not running): {}", e);
        }
    }
    
    // Example 2: Algebraic composition
    println!("\n🔄 Example 2: Algebraic composition of document approval flow");
    let reviewers = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    
    match service.compose_document_approval_flow(document_id.clone(), reviewers).await {
        Ok(composed_events) => {
            println!("✅ Composed {} events for document approval flow", composed_events.len());
            for (i, event) in composed_events.iter().enumerate() {
                println!("   Event {}: {} - {}", i + 1, event.type_name(), event.domain);
            }
        },
        Err(e) => {
            println!("❌ Failed to compose events: {}", e);
        }
    }
    
    // Example 3: Domain event publishing
    println!("\n📤 Example 3: Publishing domain events through workflow system");
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
    
    match service.publish_document_event(domain_event, Uuid::new_v4()).await {
        Ok(_) => {
            println!("✅ Document event published successfully");
        },
        Err(e) => {
            println!("❌ Failed to publish event (likely NATS not running): {}", e);
        }
    }
    
    println!("\n🎯 Document domain composition examples completed!");
    println!("This demonstrates how domains can compose with the workflow system without tight coupling.");
    
    Ok(())
}