//! Cross-Domain Workflow Coordination Example
//!
//! Demonstrates how multiple domains coordinate complex workflows through
//! algebraic composition, template instantiation, and NATS messaging.
//! This example shows document, user, and notification domains coordinating
//! through the unified workflow system.

use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use cim_domain_workflow::{
    // Composition framework
    composition::{
        WorkflowTemplate, TemplateId, TemplateVersion, TemplateInstantiationRequest,
        TemplateInstantiationEngine, TemplateParameter, TemplateStep, TemplateStepType,
        ParameterType, TemplateMetadata,
    },
    
    // Core workflow engine
    core::{CoreTemplateEngine, TemplateExecutionCoordinator, InMemoryTemplateRepository},
    
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

/// Document domain value objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub classification: String,
}

/// User domain value objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub email: String,
    pub department: String,
    pub role: String,
    pub security_clearance: String,
}

/// Notification domain value objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    Slack,
    SMS,
    InApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub recipient: UserId,
    pub notification_type: NotificationType,
    pub subject: String,
    pub message: String,
    pub priority: NotificationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Cross-domain events that coordinate between domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossDomainEvent {
    // Document domain events
    DocumentCreated {
        document_id: DocumentId,
        metadata: DocumentMetadata,
        created_by: UserId,
    },
    DocumentRequiresReview {
        document_id: DocumentId,
        reviewers: Vec<UserId>,
        due_date: chrono::DateTime<chrono::Utc>,
        priority: ReviewPriority,
    },
    DocumentReviewCompleted {
        document_id: DocumentId,
        reviewer: UserId,
        decision: ReviewDecision,
        comments: Option<String>,
    },
    DocumentApprovalRequired {
        document_id: DocumentId,
        approvers: Vec<UserId>,
        approval_level: ApprovalLevel,
    },
    
    // User domain events
    UserAssigned {
        user_id: UserId,
        task_type: String,
        task_id: String,
        assigned_by: UserId,
    },
    UserNotificationPreferencesUpdated {
        user_id: UserId,
        preferences: HashMap<String, serde_json::Value>,
    },
    
    // Notification domain events
    NotificationSent {
        notification_id: NotificationId,
        recipient: UserId,
        notification_type: NotificationType,
        delivery_status: DeliveryStatus,
    },
    NotificationFailed {
        notification_id: NotificationId,
        recipient: UserId,
        error_reason: String,
        retry_count: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewDecision {
    Approved,
    Rejected,
    RequiresChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalLevel {
    Supervisor,
    Manager,
    Director,
    Executive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Cross-domain workflow orchestrator
pub struct CrossDomainWorkflowOrchestrator {
    template_engine: CoreTemplateEngine,
    event_broker: std::sync::Arc<WorkflowEventBroker>,
    algebra: WorkflowEventAlgebra,
    coordination_context: HashMap<String, serde_json::Value>,
}

impl CrossDomainWorkflowOrchestrator {
    /// Create a new cross-domain workflow orchestrator
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Set up template repository
        let template_repository = std::sync::Arc::new(InMemoryTemplateRepository::new());
        
        // Create a placeholder workflow engine
        let workflow_engine = Box::new(PlaceholderWorkflowEngine::new());
        
        // Set up NATS broker configuration for cross-domain coordination
        let broker_config = BrokerConfiguration {
            nats_urls: vec!["nats://localhost:4222".to_string()],
            subject_prefix: "cim.coordination".to_string(),
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
        
        // Initialize with cross-domain templates
        template_engine.initialize_standard_templates().await?;
        
        Ok(Self {
            template_engine,
            event_broker,
            algebra: WorkflowEventAlgebra,
            coordination_context: HashMap::new(),
        })
    }
    
    /// Execute a complex cross-domain document review workflow
    pub async fn execute_document_review_coordination(
        &mut self,
        document_id: DocumentId,
        document_metadata: DocumentMetadata,
        created_by: UserId,
        reviewers: Vec<UserId>,
        approvers: Vec<UserId>,
    ) -> Result<WorkflowInstanceId, Box<dyn std::error::Error>> {
        // Create template instantiation request for cross-domain coordination
        let template_id = TemplateId::new(
            "coordination".to_string(),
            "document-review-approval".to_string(),
            TemplateVersion::new(1, 0, 0),
        );
        
        let mut parameters = HashMap::new();
        parameters.insert("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
        parameters.insert("document_metadata".to_string(), serde_json::to_value(&document_metadata)?);
        parameters.insert("created_by".to_string(), serde_json::json!(created_by.0.to_string()));
        parameters.insert("reviewers".to_string(), serde_json::to_value(&reviewers)?);
        parameters.insert("approvers".to_string(), serde_json::to_value(&approvers)?);
        
        let request = TemplateInstantiationRequest {
            template_id,
            parameters,
            target_domain: "coordination".to_string(),
            correlation_id: Uuid::new_v4(),
            context: HashMap::new(),
        };
        
        // Execute the cross-domain template
        let execution_result = self.template_engine.execute_template(request).await?;
        let instance_id = execution_result.instantiation_result.instance_id;
        
        // Store coordination context
        self.coordination_context.insert(
            format!("workflow_{}", instance_id.id()),
            serde_json::json!({
                "document_id": document_id.0.to_string(),
                "created_by": created_by.0.to_string(),
                "reviewers": reviewers,
                "approvers": approvers,
                "phase": "initialization"
            })
        );
        
        Ok(instance_id)
    }
    
    /// Demonstrate complex algebraic composition across domains
    pub async fn compose_cross_domain_workflow(
        &mut self,
        document_id: DocumentId,
        reviewers: Vec<UserId>,
        approvers: Vec<UserId>,
    ) -> Result<Vec<WorkflowEvent>, Box<dyn std::error::Error>> {
        let correlation_id = Uuid::new_v4();
        let workflow_id = UniversalWorkflowId::new("coordination".to_string(), Some("cross-domain-review".to_string()));
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id, Some("cross-domain-orchestrator".to_string()));
        
        let mut composed_events = Vec::new();
        
        // Phase 1: Document Creation Event
        let document_created = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "document".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("phase".to_string(), serde_json::json!("document_creation"));
                payload.set_data("cross_domain".to_string(), serde_json::json!(true));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        composed_events.push(document_created.clone());
        
        // Phase 2: Parallel User Assignment Events (across user domain)
        let mut user_assignment_events = Vec::new();
        for reviewer in &reviewers {
            let assignment_event = WorkflowEvent::step(
                StepEventType::StepCreated,
                "user".to_string(),
                correlation_id,
                {
                    let mut payload = EventPayload::empty();
                    payload.set_data("user_id".to_string(), serde_json::json!(reviewer.0.to_string()));
                    payload.set_data("assignment_type".to_string(), serde_json::json!("reviewer"));
                    payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                    payload.set_data("phase".to_string(), serde_json::json!("user_assignment"));
                    payload
                },
                EventContext::for_workflow(*context.instance_id.id()),
            );
            user_assignment_events.push(assignment_event);
        }
        
        // Use parallel composition for user assignments
        if user_assignment_events.len() > 1 {
            let mut parallel_result = user_assignment_events[0].clone();
            for assignment_event in user_assignment_events.iter().skip(1) {
                let composition_result = self.algebra.compose_parallel(
                    parallel_result,
                    assignment_event.clone(),
                    &context,
                ).await?;
                parallel_result = composition_result.result;
                composed_events.extend(composition_result.events);
            }
        } else if let Some(assignment_event) = user_assignment_events.into_iter().next() {
            composed_events.push(assignment_event);
        }
        
        // Phase 3: Sequential Notification Events (across notification domain)
        let notification_event = WorkflowEvent::step(
            StepEventType::StepCompleted,
            "notification".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("notification_type".to_string(), serde_json::json!("assignment_notification"));
                payload.set_data("recipients".to_string(), serde_json::to_value(&reviewers)?);
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("phase".to_string(), serde_json::json!("notification"));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Sequential composition: notifications after assignments
        if let Some(last_event) = composed_events.last() {
            let sequential_result = self.algebra.compose_sequential(
                last_event.clone(),
                notification_event,
                &context,
            ).await?;
            composed_events.extend(sequential_result.events);
        }
        
        // Phase 4: Conditional Approval Events (using conditional transformation)
        let approval_condition_event = WorkflowEvent::step(
            StepEventType::StepCreated,
            "approval".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("condition_type".to_string(), serde_json::json!("review_completed"));
                payload.set_data("approvers".to_string(), serde_json::to_value(&approvers)?);
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("phase".to_string(), serde_json::json!("conditional_approval"));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Sequential composition: approval after notifications (demonstrating conditional workflow)
        if let Some(last_event) = composed_events.last() {
            let sequential_result = self.algebra.compose_sequential(
                last_event.clone(),
                approval_condition_event,
                &context,
            ).await?;
            composed_events.extend(sequential_result.events);
        }
        
        // Phase 5: Final Coordination Event
        let coordination_complete = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCompleted,
            "coordination".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("document_id".to_string(), serde_json::json!(document_id.0.to_string()));
                payload.set_data("coordination_status".to_string(), serde_json::json!("pending_execution"));
                payload.set_data("domains_involved".to_string(), serde_json::json!(["document", "user", "notification", "approval"]));
                payload.set_data("phase".to_string(), serde_json::json!("completion"));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        composed_events.push(coordination_complete);
        
        Ok(composed_events)
    }
    
    /// Handle cross-domain event coordination
    pub async fn handle_cross_domain_event(
        &mut self,
        domain_event: CrossDomainEvent,
        correlation_id: Uuid,
    ) -> Result<Vec<WorkflowEvent>, Box<dyn std::error::Error>> {
        let mut coordination_events = Vec::new();
        
        match domain_event {
            CrossDomainEvent::DocumentCreated { document_id, metadata, created_by } => {
                // Trigger cross-domain coordination for document creation
                let workflow_event = self.create_coordination_event(
                    "document_created",
                    correlation_id,
                    serde_json::json!({
                        "document_id": document_id.0.to_string(),
                        "metadata": metadata,
                        "created_by": created_by.0.to_string(),
                        "trigger_notifications": true,
                        "assign_reviewers": true
                    })
                )?;
                coordination_events.push(workflow_event);
            },
            
            CrossDomainEvent::DocumentRequiresReview { document_id, reviewers, due_date, priority } => {
                // Coordinate review assignment across user and notification domains
                let review_coordination = self.create_coordination_event(
                    "review_assignment",
                    correlation_id,
                    serde_json::json!({
                        "document_id": document_id.0.to_string(),
                        "reviewers": reviewers,
                        "due_date": due_date.to_rfc3339(),
                        "priority": priority,
                        "requires_user_assignment": true,
                        "requires_notifications": true
                    })
                )?;
                coordination_events.push(review_coordination);
            },
            
            CrossDomainEvent::DocumentReviewCompleted { document_id, reviewer, decision, comments } => {
                // Coordinate review completion across domains
                let completion_event = self.create_coordination_event(
                    "review_completed",
                    correlation_id,
                    serde_json::json!({
                        "document_id": document_id.0.to_string(),
                        "reviewer": reviewer.0.to_string(),
                        "decision": decision,
                        "comments": comments,
                        "trigger_approval_check": true,
                        "notify_stakeholders": true
                    })
                )?;
                coordination_events.push(completion_event);
            },
            
            CrossDomainEvent::NotificationSent { notification_id, recipient, notification_type, delivery_status } => {
                // Track notification delivery for workflow coordination
                let delivery_event = self.create_coordination_event(
                    "notification_delivered",
                    correlation_id,
                    serde_json::json!({
                        "notification_id": notification_id.0.to_string(),
                        "recipient": recipient.0.to_string(),
                        "type": notification_type,
                        "status": delivery_status,
                        "update_workflow_state": true
                    })
                )?;
                coordination_events.push(delivery_event);
            },
            
            _ => {
                // Handle other cross-domain events as needed
                let generic_event = self.create_coordination_event(
                    "cross_domain_event",
                    correlation_id,
                    serde_json::to_value(&domain_event)?
                )?;
                coordination_events.push(generic_event);
            }
        }
        
        Ok(coordination_events)
    }
    
    /// Create a coordination event for cross-domain workflows
    fn create_coordination_event(
        &self,
        event_type: &str,
        correlation_id: Uuid,
        event_data: serde_json::Value,
    ) -> Result<WorkflowEvent, Box<dyn std::error::Error>> {
        let workflow_id = UniversalWorkflowId::new("coordination".to_string(), Some("cross-domain".to_string()));
        let instance_id = WorkflowInstanceId::new(workflow_id);
        
        let workflow_event = WorkflowEvent::step(
            StepEventType::StepCompleted,
            "coordination".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("coordination_event_type".to_string(), serde_json::json!(event_type));
                payload.set_data("event_data".to_string(), event_data);
                payload.set_data("cross_domain".to_string(), serde_json::json!(true));
                payload.set_data("orchestrator".to_string(), serde_json::json!("cross-domain-orchestrator"));
                payload
            },
            EventContext::for_workflow(*instance_id.id()),
        );
        
        Ok(workflow_event)
    }
    
    /// Publish cross-domain events through the workflow system
    pub async fn publish_cross_domain_event(
        &self,
        coordination_events: Vec<WorkflowEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for event in coordination_events {
            self.event_broker.publisher().publish_event(&event, None).await?;
            
            // Small delay for demonstration purposes
            sleep(Duration::from_millis(50)).await;
        }
        
        Ok(())
    }
    
    /// Get coordination status for a workflow
    pub fn get_coordination_status(&self, instance_id: &WorkflowInstanceId) -> Option<serde_json::Value> {
        self.coordination_context.get(&format!("workflow_{}", instance_id.id())).cloned()
    }
}

/// Create cross-domain document review template
pub fn create_cross_domain_review_template() -> WorkflowTemplate {
    use cim_domain_workflow::composition::*;
    
    WorkflowTemplate {
        id: TemplateId::new(
            "coordination".to_string(),
            "document-review-approval".to_string(),
            TemplateVersion::new(1, 0, 0),
        ),
        name: "Cross-Domain Document Review & Approval".to_string(),
        description: "Coordinates document review across document, user, notification, and approval domains".to_string(),
        version: TemplateVersion::new(1, 0, 0),
        target_domains: vec!["document".to_string(), "user".to_string(), "notification".to_string(), "approval".to_string()],
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
                "reviewers".to_string(),
                TemplateParameter {
                    name: "reviewers".to_string(),
                    param_type: ParameterType::Array(Box::new(ParameterType::String)),
                    description: "List of reviewer user IDs".to_string(),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
            ),
            (
                "approvers".to_string(),
                TemplateParameter {
                    name: "approvers".to_string(),
                    param_type: ParameterType::Array(Box::new(ParameterType::String)),
                    description: "List of approver user IDs".to_string(),
                    required: true,
                    default_value: None,
                    constraints: vec![],
                },
            ),
            (
                "approval_threshold".to_string(),
                TemplateParameter {
                    name: "approval_threshold".to_string(),
                    param_type: ParameterType::Integer,
                    description: "Number of approvals required".to_string(),
                    required: false,
                    default_value: Some(serde_json::json!(1)),
                    constraints: vec![],
                },
            ),
        ].into_iter().collect(),
        steps: vec![
            TemplateStep {
                id: "assign_reviewers".to_string(),
                name_template: "Assign Reviewers".to_string(),
                description_template: "Assign document reviewers in user domain".to_string(),
                step_type: TemplateStepType::Automated,
                dependencies: vec![],
                configuration: vec![
                    ("target_domain".to_string(), serde_json::json!("user")),
                    ("operation".to_string(), serde_json::json!("assign_task")),
                    ("task_type".to_string(), serde_json::json!("document_review")),
                ].into_iter().collect(),
                condition: None,
                retry_policy: None,
            },
            TemplateStep {
                id: "send_notifications".to_string(),
                name_template: "Send Review Notifications".to_string(),
                description_template: "Send notifications to assigned reviewers".to_string(),
                step_type: TemplateStepType::Automated,
                dependencies: vec!["assign_reviewers".to_string()],
                configuration: vec![
                    ("target_domain".to_string(), serde_json::json!("notification")),
                    ("notification_type".to_string(), serde_json::json!("review_assignment")),
                    ("priority".to_string(), serde_json::json!("medium")),
                ].into_iter().collect(),
                condition: None,
                retry_policy: None,
            },
            TemplateStep {
                id: "await_reviews".to_string(),
                name_template: "Await Review Completion".to_string(),
                description_template: "Wait for all reviews to be completed".to_string(),
                step_type: TemplateStepType::Manual,
                dependencies: vec!["send_notifications".to_string()],
                configuration: vec![
                    ("target_domain".to_string(), serde_json::json!("document")),
                    ("completion_condition".to_string(), serde_json::json!("all_reviews_complete")),
                    ("timeout_hours".to_string(), serde_json::json!(72)),
                ].into_iter().collect(),
                condition: None,
                retry_policy: None,
            },
            TemplateStep {
                id: "initiate_approval".to_string(),
                name_template: "Initiate Approval Process".to_string(),
                description_template: "Start approval process if reviews are positive".to_string(),
                step_type: TemplateStepType::Conditional,
                dependencies: vec!["await_reviews".to_string()],
                configuration: vec![
                    ("target_domain".to_string(), serde_json::json!("approval")),
                    ("condition".to_string(), serde_json::json!("reviews_positive")),
                    ("approvers".to_string(), serde_json::json!("{approvers}")),
                ].into_iter().collect(),
                condition: None,
                retry_policy: None,
            },
            TemplateStep {
                id: "final_notification".to_string(),
                name_template: "Send Final Notifications".to_string(),
                description_template: "Notify all stakeholders of final decision".to_string(),
                step_type: TemplateStepType::Automated,
                dependencies: vec!["initiate_approval".to_string()],
                configuration: vec![
                    ("target_domain".to_string(), serde_json::json!("notification")),
                    ("notification_type".to_string(), serde_json::json!("workflow_completion")),
                    ("include_reviewers".to_string(), serde_json::json!(true)),
                    ("include_approvers".to_string(), serde_json::json!(true)),
                ].into_iter().collect(),
                condition: None,
                retry_policy: None,
            },
        ],
        constraints: vec![],
        metadata: TemplateMetadata {
            author: "Cross-Domain Orchestrator".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            tags: vec!["cross-domain".to_string(), "coordination".to_string(), "review".to_string(), "approval".to_string()],
            category: "Cross-Domain Workflows".to_string(),
            documentation_url: None,
            examples: vec![],
        },
        validation_rules: vec![],
    }
}

/// Placeholder workflow engine for demonstration
struct PlaceholderWorkflowEngine {
    extensions: HashMap<String, Box<dyn cim_domain_workflow::composition::extensions::DomainWorkflowExtension>>,
}

impl PlaceholderWorkflowEngine {
    fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl cim_domain_workflow::core::WorkflowEngine for PlaceholderWorkflowEngine {
    async fn execute_workflow(
        &self,
        _instance_id: WorkflowInstanceId,
        _context: WorkflowContext,
    ) -> Result<cim_domain_workflow::core::WorkflowExecutionResult, cim_domain_workflow::core::WorkflowEngineError> {
        println!("Executing cross-domain coordination workflow");
        Ok(cim_domain_workflow::core::WorkflowExecutionResult {
            instance_id: _instance_id,
            status: cim_domain_workflow::core::WorkflowExecutionStatus::Completed,
            completed_steps: Vec::new(),
            context: _context,
            error: None,
        })
    }

    async fn execute_step(
        &self,
        _step_id: cim_domain_workflow::primitives::UniversalStepId,
        _context: WorkflowContext,
    ) -> Result<cim_domain_workflow::core::StepExecutionResult, cim_domain_workflow::core::WorkflowEngineError> {
        Ok(cim_domain_workflow::core::StepExecutionResult {
            step_id: _step_id,
            status: cim_domain_workflow::core::StepExecutionStatus::Completed,
            context: _context,
            output: None,
            error: None,
        })
    }

    async fn pause_workflow(
        &self,
        _instance_id: WorkflowInstanceId,
    ) -> Result<(), cim_domain_workflow::core::WorkflowEngineError> {
        Ok(())
    }

    async fn resume_workflow(
        &self,
        _instance_id: WorkflowInstanceId,
        _context: Option<WorkflowContext>,
    ) -> Result<cim_domain_workflow::core::WorkflowExecutionResult, cim_domain_workflow::core::WorkflowEngineError> {
        let default_context = _context.unwrap_or_else(|| {
            let workflow_id = UniversalWorkflowId::new("placeholder".to_string(), None);
            let instance_id_clone = WorkflowInstanceId::new(workflow_id.clone());
            WorkflowContext::new(workflow_id, instance_id_clone, None)
        });
        self.execute_workflow(_instance_id, default_context).await
    }

    async fn cancel_workflow(
        &self,
        _instance_id: WorkflowInstanceId,
        _reason: String,
    ) -> Result<(), cim_domain_workflow::core::WorkflowEngineError> {
        Ok(())
    }

    async fn get_workflow_status(
        &self,
        _instance_id: WorkflowInstanceId,
    ) -> Result<cim_domain_workflow::core::WorkflowStatus, cim_domain_workflow::core::WorkflowEngineError> {
        Ok(cim_domain_workflow::core::WorkflowStatus {
            instance_id: _instance_id,
            current_status: cim_domain_workflow::core::WorkflowExecutionStatus::Running,
            current_step: None,
            progress: cim_domain_workflow::core::WorkflowProgress {
                total_steps: 1,
                completed_steps: 0,
                percentage: 0.0,
            },
            started_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn get_execution_history(
        &self,
        _instance_id: WorkflowInstanceId,
    ) -> Result<Vec<cim_domain_workflow::core::ExecutionHistoryEntry>, cim_domain_workflow::core::WorkflowEngineError> {
        Ok(Vec::new())
    }

    fn register_extension(
        &mut self,
        domain: String,
        extension: Box<dyn cim_domain_workflow::composition::extensions::DomainWorkflowExtension>,
    ) -> Result<(), cim_domain_workflow::core::WorkflowEngineError> {
        self.extensions.insert(domain, extension);
        Ok(())
    }

    fn get_extensions(&self) -> &HashMap<String, Box<dyn cim_domain_workflow::composition::extensions::DomainWorkflowExtension>> {
        &self.extensions
    }

    async fn validate_workflow(
        &self,
        _workflow_id: UniversalWorkflowId,
    ) -> Result<cim_domain_workflow::core::ValidationResult, cim_domain_workflow::core::WorkflowEngineError> {
        Ok(cim_domain_workflow::core::ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: None,
        })
    }
}

/// Example usage of cross-domain workflow coordination
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Cross-Domain Workflow Coordination Example");
    println!("==============================================");
    
    // Create cross-domain orchestrator
    println!("🔧 Initializing cross-domain workflow orchestrator...");
    let mut orchestrator = CrossDomainWorkflowOrchestrator::new().await?;
    
    // Set up example data
    let document_id = DocumentId(Uuid::new_v4());
    let document_metadata = DocumentMetadata {
        title: "Strategic Planning Document".to_string(),
        author: "Alice Johnson".to_string(),
        content_type: "application/pdf".to_string(),
        size_bytes: 2_500_000,
        classification: "confidential".to_string(),
    };
    let created_by = UserId(Uuid::new_v4());
    let reviewers = vec![UserId(Uuid::new_v4()), UserId(Uuid::new_v4()), UserId(Uuid::new_v4())];
    let approvers = vec![UserId(Uuid::new_v4()), UserId(Uuid::new_v4())];
    
    // Example 1: Template-based cross-domain coordination
    println!("\n📋 Example 1: Template-based cross-domain coordination");
    match orchestrator.execute_document_review_coordination(
        document_id.clone(),
        document_metadata.clone(),
        created_by.clone(),
        reviewers.clone(),
        approvers.clone(),
    ).await {
        Ok(instance_id) => {
            println!("✅ Cross-domain coordination workflow started: {:?}", instance_id);
            
            if let Some(status) = orchestrator.get_coordination_status(&instance_id) {
                println!("   Coordination context: {}", serde_json::to_string_pretty(&status)?);
            }
        },
        Err(e) => {
            println!("❌ Failed to start coordination workflow (likely NATS not running): {}", e);
        }
    }
    
    // Example 2: Algebraic composition across domains
    println!("\n🔄 Example 2: Algebraic composition across domains");
    match orchestrator.compose_cross_domain_workflow(
        document_id.clone(),
        reviewers.clone(),
        approvers.clone(),
    ).await {
        Ok(composed_events) => {
            println!("✅ Composed {} events across domains:", composed_events.len());
            for (i, event) in composed_events.iter().enumerate() {
                let phase = event.payload.data.get("phase")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");
                println!("   Event {}: {} - {} ({})", i + 1, event.type_name(), event.domain, phase);
            }
        },
        Err(e) => {
            println!("❌ Failed to compose cross-domain events: {}", e);
        }
    }
    
    // Example 3: Cross-domain event handling
    println!("\n📤 Example 3: Cross-domain event coordination");
    
    // Simulate document creation event
    let document_created_event = CrossDomainEvent::DocumentCreated {
        document_id: document_id.clone(),
        metadata: document_metadata.clone(),
        created_by: created_by.clone(),
    };
    
    match orchestrator.handle_cross_domain_event(document_created_event, Uuid::new_v4()).await {
        Ok(coordination_events) => {
            println!("✅ Generated {} coordination events for document creation", coordination_events.len());
            
            // Publish the coordination events
            match orchestrator.publish_cross_domain_event(coordination_events).await {
                Ok(_) => println!("   📡 All coordination events published successfully"),
                Err(e) => println!("   ❌ Failed to publish events (likely NATS not running): {}", e),
            }
        },
        Err(e) => {
            println!("❌ Failed to handle cross-domain event: {}", e);
        }
    }
    
    // Simulate review requirement event
    let review_required_event = CrossDomainEvent::DocumentRequiresReview {
        document_id: document_id.clone(),
        reviewers: reviewers.clone(),
        due_date: chrono::Utc::now() + chrono::Duration::days(3),
        priority: ReviewPriority::High,
    };
    
    match orchestrator.handle_cross_domain_event(review_required_event, Uuid::new_v4()).await {
        Ok(coordination_events) => {
            println!("✅ Generated {} coordination events for review assignment", coordination_events.len());
            
            // Publish the coordination events
            match orchestrator.publish_cross_domain_event(coordination_events).await {
                Ok(_) => println!("   📡 Review coordination events published successfully"),
                Err(e) => println!("   ❌ Failed to publish events: {}", e),
            }
        },
        Err(e) => {
            println!("❌ Failed to handle review coordination: {}", e);
        }
    }
    
    println!("\n🎯 Cross-domain workflow coordination examples completed!");
    println!("This demonstrates how multiple domains can coordinate complex workflows");
    println!("through algebraic composition, templates, and NATS messaging coordination.");
    
    Ok(())
}