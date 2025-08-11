//! NATS-Based Domain Workflow Orchestration
//!
//! Demonstrates how multiple domains coordinate workflows through NATS messaging,
//! including event publishing, subscription patterns, correlation tracking,
//! and distributed workflow execution across domain boundaries.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};

use cim_domain_workflow::{
    // Algebraic operations
    algebra::{
        WorkflowEvent, LifecycleEventType, StepEventType,
        EventPayload, EventContext, WorkflowEventAlgebra,
        SequentialComposition, ParallelComposition, SubjectBuilder,
    },
    
    // NATS messaging infrastructure
    messaging::{
        WorkflowEventBroker, BrokerConfiguration,
    },
    
    // Primitives
    primitives::{UniversalWorkflowId, WorkflowInstanceId, WorkflowContext},
};

/// Multi-domain event representing cross-domain workflow coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultiDomainEvent {
    // Document domain events
    DocumentWorkflowStarted {
        workflow_id: WorkflowInstanceId,
        document_id: String,
        initiated_by: String,
        workflow_type: String,
    },
    DocumentProcessingCompleted {
        workflow_id: WorkflowInstanceId,
        document_id: String,
        processing_result: String,
    },
    
    // User domain events
    UserTaskAssigned {
        workflow_id: WorkflowInstanceId,
        user_id: String,
        task_id: String,
        task_type: String,
        due_date: chrono::DateTime<chrono::Utc>,
    },
    UserTaskCompleted {
        workflow_id: WorkflowInstanceId,
        user_id: String,
        task_id: String,
        completion_result: String,
    },
    
    // Notification domain events
    NotificationSent {
        workflow_id: WorkflowInstanceId,
        recipient: String,
        notification_type: String,
        channel: String,
        status: String,
    },
    NotificationDeliveryConfirmed {
        workflow_id: WorkflowInstanceId,
        recipient: String,
        delivered_at: chrono::DateTime<chrono::Utc>,
    },
    
    // Approval domain events
    ApprovalRequested {
        workflow_id: WorkflowInstanceId,
        item_id: String,
        approver_id: String,
        approval_type: String,
        priority: String,
    },
    ApprovalDecisionMade {
        workflow_id: WorkflowInstanceId,
        item_id: String,
        approver_id: String,
        decision: String,
        reason: Option<String>,
    },
    
    // Orchestration events
    WorkflowOrchestrationStarted {
        orchestration_id: Uuid,
        participating_domains: Vec<String>,
        coordination_type: String,
    },
    CrossDomainSynchronizationPoint {
        orchestration_id: Uuid,
        sync_point_id: String,
        awaiting_domains: Vec<String>,
    },
    OrchestrationCompleted {
        orchestration_id: Uuid,
        final_status: String,
        completion_summary: HashMap<String, serde_json::Value>,
    },
}

/// Domain-specific event processing for NATS-based coordination
#[derive(Clone)]
pub struct DomainEventProcessor {
    domain_name: String,
    processed_events: Arc<tokio::sync::RwLock<Vec<MultiDomainEvent>>>,
}

impl DomainEventProcessor {
    pub fn new(domain_name: String) -> Self {
        Self {
            domain_name,
            processed_events: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
    
    pub async fn get_processed_events(&self) -> Vec<MultiDomainEvent> {
        self.processed_events.read().await.clone()
    }
    
    /// Create an event handler closure for this domain
    pub fn create_handler(&self) -> Box<dyn Fn(WorkflowEvent, cim_domain_workflow::messaging::EventMetadata) -> cim_domain_workflow::messaging::EventHandlerResult + Send + Sync> {
        let domain_name = self.domain_name.clone();
        let _processed_events = self.processed_events.clone();
        
        Box::new(move |event: WorkflowEvent, _metadata: cim_domain_workflow::messaging::EventMetadata| -> cim_domain_workflow::messaging::EventHandlerResult {
            let start_time = chrono::Utc::now();
            
            // Extract multi-domain event from workflow event payload
            if let Some(event_data) = event.payload.data.get("multi_domain_event") {
                if let Ok(multi_domain_event) = serde_json::from_value::<MultiDomainEvent>(event_data.clone()) {
                    println!("🔄 {} received: {:?}", domain_name, 
                        std::mem::discriminant(&multi_domain_event));
                    
                    // Store the event for tracking (note: this is simplified for the demo)
                    // In real implementation, this would be async and properly handled
                    
                    // Simulate domain responses based on event type
                    match &multi_domain_event {
                        MultiDomainEvent::DocumentWorkflowStarted { document_id, .. } => {
                            if domain_name == "user" {
                                println!("   👤 User domain: Assigning tasks for document {}", document_id);
                            } else if domain_name == "notification" {
                                println!("   📧 Notification domain: Sending workflow start notifications");
                            }
                        },
                        MultiDomainEvent::UserTaskAssigned { user_id, task_type, .. } => {
                            if domain_name == "notification" {
                                println!("   📧 Notification domain: Notifying {} of {} task", user_id, task_type);
                            }
                        },
                        MultiDomainEvent::ApprovalRequested { approver_id, approval_type, .. } => {
                            if domain_name == "user" {
                                println!("   👤 User domain: Routing {} approval to {}", approval_type, approver_id);
                            } else if domain_name == "notification" {
                                println!("   📧 Notification domain: Sending approval request notification");
                            }
                        },
                        _ => {
                            println!("   ℹ️  {} domain: Processing event", domain_name);
                        }
                    }
                }
            }
            
            cim_domain_workflow::messaging::EventHandlerResult {
                success: true,
                response_events: vec![],
                execution_time: chrono::Utc::now() - start_time,
                error: None,
            }
        })
    }
}

/// NATS-based orchestration coordinator
pub struct NATSOrchestrationCoordinator {
    broker: WorkflowEventBroker,
    algebra: WorkflowEventAlgebra,
    domain_processors: HashMap<String, DomainEventProcessor>,
    active_orchestrations: Arc<tokio::sync::RwLock<HashMap<Uuid, OrchestrationState>>>,
}

#[derive(Debug, Clone)]
struct OrchestrationState {
    orchestration_id: Uuid,
    participating_domains: Vec<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    events_processed: usize,
    sync_points_completed: usize,
    status: String,
}

impl NATSOrchestrationCoordinator {
    /// Create a new NATS orchestration coordinator
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Set up NATS broker for orchestration
        let broker_config = BrokerConfiguration {
            nats_urls: vec!["nats://localhost:4222".to_string()],
            subject_prefix: "cim.orchestration".to_string(),
            ..Default::default()
        };
        
        let mut broker = WorkflowEventBroker::new(broker_config).await?;
        
        // Set up domain processors and handlers
        let mut domain_processors = HashMap::new();
        let domains = vec!["document", "user", "notification", "approval", "security"];
        
        for domain in &domains {
            let processor = DomainEventProcessor::new(domain.to_string());
            let handler = processor.create_handler();
            
            // Subscribe to domain-specific subjects using proper CIM subject format
            let subject = SubjectBuilder::new()
                .domain(format!("cim.{}", domain))
                .context("orchestration")
                .any_event_type()  // Wildcard for any event type
                .any_specificity()  // Wildcard for any specificity
                .any_correlation()  // Wildcard for any correlation
                .build().unwrap();
            broker.subscriber().subscribe(subject, handler).await?;
            
            domain_processors.insert(domain.to_string(), processor);
        }
        
        // Also subscribe to global orchestration events
        let orchestrator_processor = DomainEventProcessor::new("orchestrator".to_string());
        let global_subject = SubjectBuilder::new()
            .domain("cim.orchestration")
            .context("global")
            .any_event_type()
            .any_specificity()
            .any_correlation()
            .build().unwrap();
        broker.subscriber().subscribe(global_subject, 
            orchestrator_processor.create_handler()).await?;
        
        Ok(Self {
            broker,
            algebra: WorkflowEventAlgebra,
            domain_processors,
            active_orchestrations: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }
    
    /// Start a cross-domain orchestrated workflow
    pub async fn start_cross_domain_workflow(
        &mut self,
        workflow_type: String,
        participating_domains: Vec<String>,
        initial_data: HashMap<String, serde_json::Value>,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let orchestration_id = Uuid::new_v4();
        let workflow_id = UniversalWorkflowId::new("orchestration".to_string(), Some(workflow_type.clone()));
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id, Some("nats-orchestrator".to_string()));
        
        println!("🚀 Starting cross-domain orchestration: {}", orchestration_id);
        println!("   Workflow type: {}", workflow_type);
        println!("   Participating domains: {:?}", participating_domains);
        
        // Track orchestration state
        let orchestration_state = OrchestrationState {
            orchestration_id,
            participating_domains: participating_domains.clone(),
            started_at: chrono::Utc::now(),
            events_processed: 0,
            sync_points_completed: 0,
            status: "started".to_string(),
        };
        
        self.active_orchestrations.write().await.insert(orchestration_id, orchestration_state);
        
        // Create orchestration started event
        let orchestration_event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "orchestration".to_string(),
            Uuid::new_v4(),
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::WorkflowOrchestrationStarted {
                        orchestration_id,
                        participating_domains: participating_domains.clone(),
                        coordination_type: workflow_type.clone(),
                    }
                )?);
                payload.set_data("initial_data".to_string(), serde_json::to_value(initial_data)?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Publish to global orchestration subject
        self.broker.publisher().publish_event(&orchestration_event, None).await?;
        
        // Start domain-specific workflows
        match workflow_type.as_str() {
            "document-review-approval" => {
                self.execute_document_review_approval_orchestration(orchestration_id, &context).await?;
            },
            "user-onboarding-coordination" => {
                self.execute_user_onboarding_orchestration(orchestration_id, &context).await?;
            },
            "incident-response-coordination" => {
                self.execute_incident_response_orchestration(orchestration_id, &context).await?;
            },
            _ => {
                self.execute_generic_orchestration(orchestration_id, &participating_domains, &context).await?;
            }
        }
        
        Ok(orchestration_id)
    }
    
    /// Execute document review approval orchestration
    async fn execute_document_review_approval_orchestration(
        &self,
        orchestration_id: Uuid,
        context: &WorkflowContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let correlation_id = Uuid::new_v4();
        let document_id = format!("DOC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let reviewer_id = format!("USER-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let approver_id = format!("USER-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        
        println!("📄 Executing document review approval orchestration");
        
        // Step 1: Document workflow initiation
        let doc_workflow_event = WorkflowEvent::step(
            StepEventType::StepCreated,
            "document".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::DocumentWorkflowStarted {
                        workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("document".to_string(), None)),
                        document_id: document_id.clone(),
                        initiated_by: "system".to_string(),
                        workflow_type: "review-approval".to_string(),
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Step 2: User task assignment  
        let user_task_event = WorkflowEvent::step(
            StepEventType::StepCreated,
            "user".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::UserTaskAssigned {
                        workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("user".to_string(), None)),
                        user_id: reviewer_id.clone(),
                        task_id: format!("REVIEW-{}", document_id),
                        task_type: "document-review".to_string(),
                        due_date: chrono::Utc::now() + chrono::Duration::days(2),
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Step 3: Approval request
        let approval_event = WorkflowEvent::step(
            StepEventType::StepCreated,
            "approval".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::ApprovalRequested {
                        workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("approval".to_string(), None)),
                        item_id: document_id.clone(),
                        approver_id: approver_id.clone(),
                        approval_type: "document-approval".to_string(),
                        priority: "medium".to_string(),
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Use sequential composition to chain the events
        let sequential_result = self.algebra.compose_sequential(
            doc_workflow_event,
            user_task_event,
            context,
        ).await?;
        
        let final_result = self.algebra.compose_sequential(
            sequential_result.result,
            approval_event,
            context,
        ).await?;
        
        // Publish all orchestrated events
        for event in final_result.events {
            self.broker.publisher().publish_event(&event, None).await?;
            sleep(Duration::from_millis(200)).await; // Simulate realistic timing
        }
        
        Ok(())
    }
    
    /// Execute user onboarding orchestration
    async fn execute_user_onboarding_orchestration(
        &self,
        orchestration_id: Uuid,
        context: &WorkflowContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let correlation_id = Uuid::new_v4();
        let new_user_id = format!("NEWUSER-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        
        println!("👤 Executing user onboarding orchestration");
        
        // Create parallel events for onboarding steps
        let user_creation_event = WorkflowEvent::step(
            StepEventType::StepCreated,
            "user".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::UserTaskAssigned {
                        workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("user".to_string(), None)),
                        user_id: new_user_id.clone(),
                        task_id: format!("ONBOARD-{}", new_user_id),
                        task_type: "account-setup".to_string(),
                        due_date: chrono::Utc::now() + chrono::Duration::hours(4),
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        let notification_event = WorkflowEvent::step(
            StepEventType::StepCreated,
            "notification".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::NotificationSent {
                        workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("notification".to_string(), None)),
                        recipient: new_user_id.clone(),
                        notification_type: "welcome".to_string(),
                        channel: "email".to_string(),
                        status: "sent".to_string(),
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Use parallel composition for simultaneous operations
        let parallel_result = self.algebra.compose_parallel(
            user_creation_event,
            notification_event,
            context,
        ).await?;
        
        // Publish orchestrated events
        for event in parallel_result.events {
            self.broker.publisher().publish_event(&event, None).await?;
            sleep(Duration::from_millis(150)).await;
        }
        
        Ok(())
    }
    
    /// Execute incident response orchestration
    async fn execute_incident_response_orchestration(
        &self,
        orchestration_id: Uuid,
        context: &WorkflowContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let correlation_id = Uuid::new_v4();
        let incident_id = format!("INC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        
        println!("🚨 Executing incident response orchestration");
        
        // Create high-priority notification event
        let incident_notification = WorkflowEvent::step(
            StepEventType::StepCreated,
            "notification".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::NotificationSent {
                        workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("security".to_string(), None)),
                        recipient: "security-team".to_string(),
                        notification_type: "incident-alert".to_string(),
                        channel: "sms-email-slack".to_string(),
                        status: "urgent".to_string(),
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload.set_data("incident_id".to_string(), serde_json::json!(incident_id));
                payload.set_data("priority".to_string(), serde_json::json!("critical"));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Publish incident response event
        self.broker.publisher().publish_event(&incident_notification, None).await?;
        
        Ok(())
    }
    
    /// Execute generic orchestration for custom workflows
    async fn execute_generic_orchestration(
        &self,
        orchestration_id: Uuid,
        participating_domains: &[String],
        context: &WorkflowContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚙️  Executing generic cross-domain orchestration");
        
        let correlation_id = Uuid::new_v4();
        
        // Create sync point event
        let sync_event = WorkflowEvent::step(
            StepEventType::StepCompleted,
            "orchestration".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::CrossDomainSynchronizationPoint {
                        orchestration_id,
                        sync_point_id: "initial-sync".to_string(),
                        awaiting_domains: participating_domains.to_vec(),
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Publish to all participating domains
        for _domain in participating_domains {
            self.broker.publisher().publish_event(&sync_event, None).await?;
        }
        
        Ok(())
    }
    
    /// Complete an orchestration
    pub async fn complete_orchestration(
        &mut self,
        orchestration_id: Uuid,
        final_status: String,
        completion_summary: HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("✅ Completing orchestration: {}", orchestration_id);
        
        // Update orchestration state
        if let Some(state) = self.active_orchestrations.write().await.get_mut(&orchestration_id) {
            state.status = final_status.clone();
        }
        
        let correlation_id = Uuid::new_v4();
        let workflow_id = UniversalWorkflowId::new("orchestration".to_string(), None);
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id, Some("nats-orchestrator".to_string()));
        
        // Create completion event
        let completion_event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCompleted,
            "orchestration".to_string(),
            correlation_id,
            {
                let mut payload = EventPayload::empty();
                payload.set_data("multi_domain_event".to_string(), serde_json::to_value(
                    MultiDomainEvent::OrchestrationCompleted {
                        orchestration_id,
                        final_status,
                        completion_summary,
                    }
                )?);
                payload.set_data("orchestration_id".to_string(), serde_json::json!(orchestration_id.to_string()));
                payload
            },
            EventContext::for_workflow(*context.instance_id.id()),
        );
        
        // Publish completion event
        self.broker.publisher().publish_event(&completion_event, None).await?;
        
        Ok(())
    }
    
    /// Get orchestration statistics
    pub async fn get_orchestration_stats(&self) -> HashMap<String, serde_json::Value> {
        let orchestrations = self.active_orchestrations.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("total_orchestrations".to_string(), serde_json::json!(orchestrations.len()));
        
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        for orchestration in orchestrations.values() {
            *status_counts.entry(orchestration.status.clone()).or_insert(0) += 1;
        }
        stats.insert("status_counts".to_string(), serde_json::to_value(status_counts).unwrap());
        
        let mut domain_participation: HashMap<String, usize> = HashMap::new();
        for orchestration in orchestrations.values() {
            for domain in &orchestration.participating_domains {
                *domain_participation.entry(domain.clone()).or_insert(0) += 1;
            }
        }
        stats.insert("domain_participation".to_string(), serde_json::to_value(domain_participation).unwrap());
        
        // Get event processing stats from domain processors
        let mut domain_event_counts = HashMap::new();
        for (domain, processor) in &self.domain_processors {
            let event_count = processor.get_processed_events().await.len();
            domain_event_counts.insert(domain.clone(), event_count);
        }
        stats.insert("domain_event_counts".to_string(), serde_json::to_value(domain_event_counts).unwrap());
        
        stats
    }
}

/// Example usage of NATS-based domain workflow orchestration
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 NATS-Based Domain Workflow Orchestration Demo");
    println!("=================================================");
    
    // Create NATS orchestration coordinator
    println!("🔧 Initializing NATS orchestration coordinator...");
    let mut coordinator = match NATSOrchestrationCoordinator::new().await {
        Ok(coord) => coord,
        Err(e) => {
            println!("❌ Failed to initialize coordinator (likely NATS not running): {}", e);
            println!("   To run this demo, start NATS server: nats-server");
            return Ok(());
        }
    };
    
    println!("✅ NATS orchestration coordinator initialized successfully");
    println!("   Subscribed to cross-domain coordination subjects");
    
    // Example 1: Document review approval orchestration
    println!("\n📋 Example 1: Document review approval orchestration");
    let orchestration1 = coordinator.start_cross_domain_workflow(
        "document-review-approval".to_string(),
        vec!["document".to_string(), "user".to_string(), "approval".to_string(), "notification".to_string()],
        vec![
            ("document_type".to_string(), serde_json::json!("technical-specification")),
            ("priority".to_string(), serde_json::json!("high")),
            ("department".to_string(), serde_json::json!("engineering"))
        ].into_iter().collect(),
    ).await?;
    
    println!("✅ Document review orchestration started: {}", orchestration1);
    
    // Wait for events to be processed
    sleep(Duration::from_secs(2)).await;
    
    // Example 2: User onboarding orchestration
    println!("\n📋 Example 2: User onboarding orchestration");
    let orchestration2 = coordinator.start_cross_domain_workflow(
        "user-onboarding-coordination".to_string(),
        vec!["user".to_string(), "notification".to_string(), "security".to_string()],
        vec![
            ("new_user_email".to_string(), serde_json::json!("alice.smith@company.com")),
            ("department".to_string(), serde_json::json!("marketing")),
            ("role".to_string(), serde_json::json!("manager"))
        ].into_iter().collect(),
    ).await?;
    
    println!("✅ User onboarding orchestration started: {}", orchestration2);
    
    // Wait for events to be processed
    sleep(Duration::from_secs(2)).await;
    
    // Example 3: Incident response orchestration
    println!("\n📋 Example 3: Incident response orchestration");
    let orchestration3 = coordinator.start_cross_domain_workflow(
        "incident-response-coordination".to_string(),
        vec!["security".to_string(), "notification".to_string(), "user".to_string()],
        vec![
            ("incident_type".to_string(), serde_json::json!("security-breach")),
            ("severity".to_string(), serde_json::json!("critical")),
            ("affected_systems".to_string(), serde_json::json!(["authentication", "user-data"]))
        ].into_iter().collect(),
    ).await?;
    
    println!("✅ Incident response orchestration started: {}", orchestration3);
    
    // Wait for all events to be processed
    sleep(Duration::from_secs(3)).await;
    
    // Example 4: Generic cross-domain orchestration
    println!("\n📋 Example 4: Generic cross-domain orchestration");
    let orchestration4 = coordinator.start_cross_domain_workflow(
        "custom-workflow".to_string(),
        vec!["document".to_string(), "user".to_string(), "notification".to_string()],
        vec![
            ("workflow_name".to_string(), serde_json::json!("quarterly-review")),
            ("participants".to_string(), serde_json::json!(["team-lead", "manager", "director"]))
        ].into_iter().collect(),
    ).await?;
    
    println!("✅ Generic orchestration started: {}", orchestration4);
    
    // Wait for processing
    sleep(Duration::from_secs(2)).await;
    
    // Example 5: Complete orchestrations
    println!("\n📋 Example 5: Completing orchestrations");
    
    coordinator.complete_orchestration(
        orchestration1,
        "completed".to_string(),
        vec![
            ("documents_reviewed".to_string(), serde_json::json!(1)),
            ("approvals_granted".to_string(), serde_json::json!(1)),
            ("notifications_sent".to_string(), serde_json::json!(3))
        ].into_iter().collect(),
    ).await?;
    
    coordinator.complete_orchestration(
        orchestration2,
        "completed".to_string(),
        vec![
            ("accounts_created".to_string(), serde_json::json!(1)),
            ("permissions_assigned".to_string(), serde_json::json!(1)),
            ("welcome_emails_sent".to_string(), serde_json::json!(1))
        ].into_iter().collect(),
    ).await?;
    
    sleep(Duration::from_secs(1)).await;
    
    // Example 6: Show orchestration statistics
    println!("\n📊 Example 6: Orchestration statistics");
    let stats = coordinator.get_orchestration_stats().await;
    
    println!("   📊 Orchestration Statistics:");
    for (key, value) in &stats {
        println!("      • {}: {}", key, serde_json::to_string_pretty(value)?);
    }
    
    println!("\n🎯 NATS-Based Domain Workflow Orchestration Demo Complete!");
    println!("This demonstration showed:");
    println!("• Cross-domain workflow coordination through NATS messaging");
    println!("• Event-driven orchestration with correlation tracking");
    println!("• Domain-specific event handlers and processing");
    println!("• Sequential and parallel composition across domains");
    println!("• Comprehensive orchestration lifecycle management");
    println!("• Real-time statistics and monitoring capabilities");
    
    Ok(())
}