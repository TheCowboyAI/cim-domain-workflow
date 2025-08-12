//! Cross-Domain Workflow Coordination Example
//!
//! Simplified demonstration of cross-domain workflow coordination.
//! Shows how different domains can coordinate workflows through event patterns.

use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use cim_domain_workflow::{
    // Core workflow functionality
    composition::{
        WorkflowTemplate, TemplateId, TemplateVersion,
    },
    
    // Algebraic operations
    algebra::{
        WorkflowEvent, LifecycleEventType, StepEventType,
        EventPayload, EventContext,
    },
    
    // Primitives
    primitives::{UniversalWorkflowId, WorkflowInstanceId, WorkflowContext},
};

/// Document domain value objects
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct DocumentId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub content_type: String,
    pub size_bytes: u64,
}

/// User domain value objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub email: String,
    pub role: String,
    pub department: String,
}

/// Cross-domain coordination event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationEvent {
    DocumentSubmitted {
        document_id: DocumentId,
        metadata: DocumentMetadata,
        submitted_by: UserId,
        workflow_id: WorkflowInstanceId,
    },
    ReviewerAssigned {
        document_id: DocumentId,
        reviewer: UserId,
        assigned_by: UserId,
        workflow_id: WorkflowInstanceId,
    },
    ReviewCompleted {
        document_id: DocumentId,
        reviewer: UserId,
        decision: String,
        comments: Option<String>,
        workflow_id: WorkflowInstanceId,
    },
    NotificationSent {
        recipient: UserId,
        message: String,
        notification_type: String,
        workflow_id: WorkflowInstanceId,
    },
    WorkflowCompleted {
        workflow_id: WorkflowInstanceId,
        final_status: String,
        completion_time: chrono::DateTime<chrono::Utc>,
    },
}

/// Domain coordinator for cross-domain workflows
pub struct CrossDomainCoordinator {
    active_workflows: HashMap<WorkflowInstanceId, CoordinationWorkflow>,
    domain_processors: HashMap<String, DomainProcessor>,
}

#[derive(Debug, Clone)]
pub struct CoordinationWorkflow {
    pub workflow_id: WorkflowInstanceId,
    pub workflow_type: String,
    pub participating_domains: Vec<String>,
    pub current_stage: String,
    pub events_processed: Vec<CoordinationEvent>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct DomainProcessor {
    pub domain_name: String,
    pub processed_events: u32,
    pub active_tasks: HashMap<String, String>,
}

impl DomainProcessor {
    pub fn new(domain_name: String) -> Self {
        Self {
            domain_name,
            processed_events: 0,
            active_tasks: HashMap::new(),
        }
    }
    
    pub fn process_coordination_event(&mut self, event: &CoordinationEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.processed_events += 1;
        
        println!("🏢 Domain '{}' processing event #{}", self.domain_name, self.processed_events);
        
        match event {
            CoordinationEvent::DocumentSubmitted { document_id, metadata, .. } => {
                println!("   📄 Document submitted: {} - {}", document_id.0, metadata.title);
                self.active_tasks.insert(document_id.0.to_string(), "submitted".to_string());
            },
            CoordinationEvent::ReviewerAssigned { document_id, reviewer, .. } => {
                println!("   👤 Reviewer assigned: {} for document {}", reviewer.0, document_id.0);
                self.active_tasks.insert(format!("review_{}", document_id.0), "assigned".to_string());
            },
            CoordinationEvent::ReviewCompleted { document_id, decision, .. } => {
                println!("   ✅ Review completed for document {}: {}", document_id.0, decision);
                self.active_tasks.insert(format!("review_{}", document_id.0), "completed".to_string());
            },
            CoordinationEvent::NotificationSent { recipient, message, .. } => {
                println!("   📧 Notification sent to {}: {}", recipient.0, message);
            },
            CoordinationEvent::WorkflowCompleted { workflow_id, final_status, .. } => {
                println!("   🏁 Workflow completed: {} with status {}", workflow_id.id(), final_status);
            },
        }
        
        Ok(())
    }
}

impl CrossDomainCoordinator {
    /// Create a new cross-domain coordinator
    pub fn new() -> Self {
        let mut domain_processors = HashMap::new();
        
        // Initialize domain processors
        let domains = vec!["document", "user", "notification", "approval"];
        for domain in domains {
            domain_processors.insert(domain.to_string(), DomainProcessor::new(domain.to_string()));
        }
        
        Self {
            active_workflows: HashMap::new(),
            domain_processors,
        }
    }
    
    /// Start a cross-domain document approval workflow
    pub async fn start_document_approval_workflow(
        &mut self,
        document_id: DocumentId,
        metadata: DocumentMetadata,
        submitted_by: UserId,
        reviewer: UserId,
    ) -> Result<WorkflowInstanceId, Box<dyn std::error::Error>> {
        // Create workflow identifiers
        let workflow_id = UniversalWorkflowId::new("document".to_string(), Some("approval".to_string()));
        let instance_id = WorkflowInstanceId::new(workflow_id);
        
        println!("🚀 Starting cross-domain document approval workflow");
        println!("   Workflow ID: {}", instance_id.id());
        println!("   Document: {} - {}", document_id.0, metadata.title);
        println!("   Submitted by: {}", submitted_by.0);
        println!("   Reviewer: {}", reviewer.0);
        
        // Create coordination workflow
        let coordination_workflow = CoordinationWorkflow {
            workflow_id: instance_id.clone(),
            workflow_type: "document_approval".to_string(),
            participating_domains: vec!["document".to_string(), "user".to_string(), "notification".to_string()],
            current_stage: "initiated".to_string(),
            events_processed: Vec::new(),
            started_at: chrono::Utc::now(),
            status: "running".to_string(),
        };
        
        self.active_workflows.insert(instance_id.clone(), coordination_workflow);
        
        // Stage 1: Document submission event
        let submission_event = CoordinationEvent::DocumentSubmitted {
            document_id: document_id.clone(),
            metadata: metadata.clone(),
            submitted_by: submitted_by.clone(),
            workflow_id: instance_id.clone(),
        };
        
        self.coordinate_event(&submission_event).await?;
        
        // Stage 2: Reviewer assignment event
        let assignment_event = CoordinationEvent::ReviewerAssigned {
            document_id: document_id.clone(),
            reviewer: reviewer.clone(),
            assigned_by: submitted_by.clone(),
            workflow_id: instance_id.clone(),
        };
        
        self.coordinate_event(&assignment_event).await?;
        
        // Stage 3: Send notification to reviewer
        let notification_event = CoordinationEvent::NotificationSent {
            recipient: reviewer.clone(),
            message: format!("You have been assigned to review document: {}", metadata.title),
            notification_type: "task_assignment".to_string(),
            workflow_id: instance_id.clone(),
        };
        
        self.coordinate_event(&notification_event).await?;
        
        // Update workflow stage
        if let Some(workflow) = self.active_workflows.get_mut(&instance_id) {
            workflow.current_stage = "awaiting_review".to_string();
        }
        
        Ok(instance_id)
    }
    
    /// Complete a document review
    pub async fn complete_document_review(
        &mut self,
        workflow_id: WorkflowInstanceId,
        document_id: DocumentId,
        reviewer: UserId,
        decision: String,
        comments: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📝 Completing document review for workflow: {}", workflow_id.id());
        
        // Stage 4: Review completion event
        let review_event = CoordinationEvent::ReviewCompleted {
            document_id: document_id.clone(),
            reviewer: reviewer.clone(),
            decision: decision.clone(),
            comments: comments.clone(),
            workflow_id: workflow_id.clone(),
        };
        
        self.coordinate_event(&review_event).await?;
        
        // Stage 5: Notify original submitter
        if let Some(workflow) = self.active_workflows.get(&workflow_id) {
            if let Some(CoordinationEvent::DocumentSubmitted { submitted_by, .. }) = 
                workflow.events_processed.iter().find(|e| matches!(e, CoordinationEvent::DocumentSubmitted { .. })) {
                
                let notification_message = match decision.as_str() {
                    "approved" => "Your document has been approved!",
                    "rejected" => "Your document requires revisions.",
                    _ => "Your document review is complete.",
                };
                
                let notification_event = CoordinationEvent::NotificationSent {
                    recipient: submitted_by.clone(),
                    message: notification_message.to_string(),
                    notification_type: "review_result".to_string(),
                    workflow_id: workflow_id.clone(),
                };
                
                self.coordinate_event(&notification_event).await?;
            }
        }
        
        // Stage 6: Complete workflow
        let completion_event = CoordinationEvent::WorkflowCompleted {
            workflow_id: workflow_id.clone(),
            final_status: decision.clone(),
            completion_time: chrono::Utc::now(),
        };
        
        self.coordinate_event(&completion_event).await?;
        
        // Update workflow status
        if let Some(workflow) = self.active_workflows.get_mut(&workflow_id) {
            workflow.status = "completed".to_string();
            workflow.current_stage = "completed".to_string();
        }
        
        Ok(())
    }
    
    /// Coordinate a single event across domains
    async fn coordinate_event(&mut self, event: &CoordinationEvent) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Coordinating event across domains...");
        
        // Add event to workflow history
        if let Some(workflow_id) = self.extract_workflow_id(event) {
            if let Some(workflow) = self.active_workflows.get_mut(&workflow_id) {
                workflow.events_processed.push(event.clone());
            }
        }
        
        // Process event in relevant domains
        match event {
            CoordinationEvent::DocumentSubmitted { .. } => {
                if let Some(processor) = self.domain_processors.get_mut("document") {
                    processor.process_coordination_event(event)?;
                }
            },
            CoordinationEvent::ReviewerAssigned { .. } => {
                if let Some(processor) = self.domain_processors.get_mut("user") {
                    processor.process_coordination_event(event)?;
                }
            },
            CoordinationEvent::NotificationSent { .. } => {
                if let Some(processor) = self.domain_processors.get_mut("notification") {
                    processor.process_coordination_event(event)?;
                }
            },
            _ => {
                // Process in all relevant domains
                for processor in self.domain_processors.values_mut() {
                    processor.process_coordination_event(event)?;
                }
            }
        }
        
        // Simulate processing delay
        sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    /// Extract workflow ID from coordination event
    fn extract_workflow_id(&self, event: &CoordinationEvent) -> Option<WorkflowInstanceId> {
        match event {
            CoordinationEvent::DocumentSubmitted { workflow_id, .. } => Some(workflow_id.clone()),
            CoordinationEvent::ReviewerAssigned { workflow_id, .. } => Some(workflow_id.clone()),
            CoordinationEvent::ReviewCompleted { workflow_id, .. } => Some(workflow_id.clone()),
            CoordinationEvent::NotificationSent { workflow_id, .. } => Some(workflow_id.clone()),
            CoordinationEvent::WorkflowCompleted { workflow_id, .. } => Some(workflow_id.clone()),
        }
    }
    
    /// Get workflow status
    pub fn get_workflow_status(&self, workflow_id: &WorkflowInstanceId) -> Option<&CoordinationWorkflow> {
        self.active_workflows.get(workflow_id)
    }
    
    /// Get coordination statistics
    pub fn get_coordination_stats(&self) -> HashMap<String, u32> {
        self.domain_processors
            .iter()
            .map(|(name, processor)| (name.clone(), processor.processed_events))
            .collect()
    }
}

/// Main demonstration function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Cross-Domain Workflow Coordination Demo");
    println!("===========================================\n");
    
    // Create cross-domain coordinator
    println!("🔧 Initializing cross-domain coordinator...");
    let mut coordinator = CrossDomainCoordinator::new();
    println!("✅ Coordinator initialized with domain processors: {:?}\n", coordinator.domain_processors.keys().collect::<Vec<_>>());
    
    // Create sample data
    let document_id = DocumentId(Uuid::new_v4());
    let metadata = DocumentMetadata {
        title: "Project Proposal: AI Integration".to_string(),
        author: "Alice Johnson".to_string(),
        content_type: "application/pdf".to_string(),
        size_bytes: 2048000,
    };
    let submitted_by = UserId(Uuid::new_v4());
    let reviewer = UserId(Uuid::new_v4());
    
    // Demonstrate cross-domain document approval workflow
    println!("📋 Example: Cross-domain document approval workflow");
    let workflow_id = coordinator.start_document_approval_workflow(
        document_id.clone(),
        metadata.clone(),
        submitted_by.clone(),
        reviewer.clone(),
    ).await?;
    
    println!("✅ Document approval workflow initiated\n");
    
    // Simulate some processing time
    sleep(Duration::from_secs(1)).await;
    
    // Complete the review
    println!("📋 Completing document review...");
    coordinator.complete_document_review(
        workflow_id.clone(),
        document_id.clone(),
        reviewer.clone(),
        "approved".to_string(),
        Some("Excellent work! Approved for implementation.".to_string()),
    ).await?;
    
    println!("✅ Document review completed\n");
    
    // Show workflow status
    if let Some(workflow) = coordinator.get_workflow_status(&workflow_id) {
        println!("📊 Final Workflow Status:");
        println!("   Workflow ID: {}", workflow.workflow_id.id());
        println!("   Type: {}", workflow.workflow_type);
        println!("   Status: {}", workflow.status);
        println!("   Current Stage: {}", workflow.current_stage);
        println!("   Participating Domains: {:?}", workflow.participating_domains);
        println!("   Events Processed: {}", workflow.events_processed.len());
        println!("   Started At: {}", workflow.started_at.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    // Show coordination statistics
    println!("\n📈 Domain Coordination Statistics:");
    let stats = coordinator.get_coordination_stats();
    for (domain, count) in stats {
        println!("   {}: {} events processed", domain, count);
    }
    
    println!("\n🎯 Key Features Demonstrated:");
    println!("   • Cross-domain workflow coordination");
    println!("   • Multi-stage workflow execution");
    println!("   • Event-driven domain communication");
    println!("   • Workflow state tracking and management");
    println!("   • Domain-specific event processing");
    println!("   • Coordination statistics and monitoring");
    
    println!("\n✨ Cross-domain coordination demonstration completed!");
    
    Ok(())
}