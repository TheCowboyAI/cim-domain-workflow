//! NATS-Based Domain Workflow Orchestration
//!
//! Simplified demonstration of cross-domain workflow coordination concepts.
//! Shows event creation and basic orchestration patterns.

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
        EventPayload, EventContext,
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
        task_type: String,
        deadline: chrono::DateTime<chrono::Utc>,
    },
    UserTaskCompleted {
        workflow_id: WorkflowInstanceId,
        user_id: String,
        task_result: String,
    },
    
    // Notification domain events
    NotificationSent {
        workflow_id: WorkflowInstanceId,
        recipient: String,
        notification_type: String,
        message: String,
    },
}

/// Simple domain event processor
#[derive(Debug, Clone)]
pub struct DomainEventProcessor {
    domain: String,
    processed_events: u32,
}

impl DomainEventProcessor {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            processed_events: 0,
        }
    }
    
    pub fn process_event(&mut self, event: &MultiDomainEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.processed_events += 1;
        println!("📨 Domain '{}' processed event #{}", self.domain, self.processed_events);
        
        match event {
            MultiDomainEvent::DocumentWorkflowStarted { workflow_id, document_id, .. } => {
                println!("   📄 Document workflow started: {} for document {}", workflow_id.id(), document_id);
            },
            MultiDomainEvent::UserTaskAssigned { workflow_id, user_id, task_type, .. } => {
                println!("   👤 Task assigned to user {}: {} in workflow {}", user_id, task_type, workflow_id.id());
            },
            MultiDomainEvent::NotificationSent { recipient, message, .. } => {
                println!("   📬 Notification sent to {}: {}", recipient, message);
            },
            _ => {
                println!("   ⚡ Other event processed");
            }
        }
        
        Ok(())
    }
}

/// Simplified orchestration coordinator
pub struct NATSOrchestrationCoordinator {
    domain_processors: HashMap<String, DomainEventProcessor>,
    active_orchestrations: Arc<tokio::sync::RwLock<HashMap<Uuid, OrchestrationState>>>,
}

#[derive(Debug, Clone)]
struct OrchestrationState {
    orchestration_id: Uuid,
    participating_domains: Vec<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    events_processed: usize,
    status: String,
}

impl NATSOrchestrationCoordinator {
    /// Create a new NATS orchestration coordinator
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Set up domain processors
        let mut domain_processors = HashMap::new();
        let domains = vec!["document", "user", "notification", "approval", "security"];
        
        for domain in &domains {
            let processor = DomainEventProcessor::new(domain.to_string());
            domain_processors.insert(domain.to_string(), processor);
        }
        
        Ok(Self {
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
        
        println!("🚀 Starting cross-domain orchestration: {}", orchestration_id);
        println!("   Workflow type: {}", workflow_type);
        println!("   Participating domains: {:?}", participating_domains);
        
        // Track orchestration state
        let orchestration_state = OrchestrationState {
            orchestration_id,
            participating_domains: participating_domains.clone(),
            started_at: chrono::Utc::now(),
            events_processed: 0,
            status: "started".to_string(),
        };
        
        {
            let mut active = self.active_orchestrations.write().await;
            active.insert(orchestration_id, orchestration_state);
        }
        
        // Create and publish initial orchestration event
        let initial_event = self.create_orchestration_start_event(
            instance_id.clone(),
            workflow_type,
            participating_domains,
            initial_data,
        ).await?;
        
        self.publish_event_to_domains(&initial_event).await?;
        
        Ok(orchestration_id)
    }
    
    /// Create workflow events for orchestration
    async fn create_orchestration_start_event(
        &self,
        instance_id: WorkflowInstanceId,
        workflow_type: String,
        domains: Vec<String>,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowEvent, Box<dyn std::error::Error>> {
        let correlation_id = Uuid::new_v4();
        
        let mut payload = EventPayload::empty();
        payload.set_data("workflow_type".to_string(), serde_json::json!(workflow_type));
        payload.set_data("participating_domains".to_string(), serde_json::json!(domains));
        payload.set_data("initial_data".to_string(), serde_json::json!(data));
        
        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "orchestration".to_string(),
            correlation_id,
            payload,
            EventContext::for_workflow(instance_id.id().clone()),
        );
        
        Ok(event)
    }
    
    /// Simulate publishing events to domains
    async fn publish_event_to_domains(&mut self, event: &WorkflowEvent) -> Result<(), Box<dyn std::error::Error>> {
        println!("📡 Publishing orchestration event: {}", event.type_name());
        
        // Convert workflow event to domain event for demonstration
        let domain_event = MultiDomainEvent::DocumentWorkflowStarted {
            workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("test".to_string(), None)),
            document_id: "DOC-123".to_string(),
            initiated_by: "orchestrator".to_string(),
            workflow_type: "approval".to_string(),
        };
        
        // Process through available domain processors
        for (domain_name, processor) in self.domain_processors.iter_mut() {
            if domain_name == "document" {
                processor.process_event(&domain_event)?;
            }
        }
        
        Ok(())
    }
    
    /// Demonstrate parallel domain coordination
    pub async fn coordinate_parallel_domains(
        &mut self,
        orchestration_id: Uuid,
        coordination_type: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Coordinating parallel domain operations: {}", coordination_type);
        
        // Simulate parallel task assignment
        let user_task = MultiDomainEvent::UserTaskAssigned {
            workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("test".to_string(), None)),
            user_id: "user123".to_string(),
            task_type: "review".to_string(),
            deadline: chrono::Utc::now() + chrono::Duration::hours(24),
        };
        
        let notification_event = MultiDomainEvent::NotificationSent {
            workflow_id: WorkflowInstanceId::new(UniversalWorkflowId::new("test".to_string(), None)),
            recipient: "user123@example.com".to_string(),
            notification_type: "task_assignment".to_string(),
            message: "You have been assigned a review task".to_string(),
        };
        
        // Process events in parallel simulation
        if let Some(user_processor) = self.domain_processors.get_mut("user") {
            user_processor.process_event(&user_task)?;
        }
        
        if let Some(notification_processor) = self.domain_processors.get_mut("notification") {
            notification_processor.process_event(&notification_event)?;
        }
        
        // Update orchestration state
        {
            let mut active = self.active_orchestrations.write().await;
            if let Some(state) = active.get_mut(&orchestration_id) {
                state.events_processed += 2;
                state.status = "parallel_coordination_complete".to_string();
            }
        }
        
        println!("✅ Parallel domain coordination completed");
        Ok(())
    }
    
    /// Get orchestration status
    pub async fn get_orchestration_status(&self, orchestration_id: Uuid) -> Option<OrchestrationState> {
        let active = self.active_orchestrations.read().await;
        active.get(&orchestration_id).cloned()
    }
}

/// Main demonstration function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 NATS-Based Domain Workflow Orchestration Demo");
    println!("=================================================\n");
    
    // Create orchestration coordinator
    println!("🔧 Initializing NATS orchestration coordinator...");
    let mut coordinator = NATSOrchestrationCoordinator::new().await?;
    println!("✅ Orchestration coordinator initialized with domain processors\n");
    
    // Demonstrate cross-domain workflow orchestration
    println!("📋 Example 1: Cross-domain document approval workflow");
    let initial_data = HashMap::from([
        ("document_id".to_string(), serde_json::json!("DOC-12345")),
        ("priority".to_string(), serde_json::json!("high")),
        ("approver_count".to_string(), serde_json::json!(2)),
    ]);
    
    let orchestration_id = coordinator.start_cross_domain_workflow(
        "document_approval".to_string(),
        vec!["document".to_string(), "user".to_string(), "notification".to_string()],
        initial_data,
    ).await?;
    
    println!("✅ Cross-domain workflow orchestration started: {}\n", orchestration_id);
    
    // Simulate some delay for processing
    sleep(Duration::from_millis(500)).await;
    
    // Demonstrate parallel domain coordination
    println!("📋 Example 2: Parallel domain coordination");
    coordinator.coordinate_parallel_domains(
        orchestration_id,
        "user_task_assignment".to_string(),
    ).await?;
    
    // Check orchestration status
    println!("\n📊 Orchestration Status Report:");
    if let Some(status) = coordinator.get_orchestration_status(orchestration_id).await {
        println!("   Orchestration ID: {}", status.orchestration_id);
        println!("   Status: {}", status.status);
        println!("   Participating Domains: {:?}", status.participating_domains);
        println!("   Events Processed: {}", status.events_processed);
        println!("   Started At: {}", status.started_at.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    println!("\n🎯 Key Features Demonstrated:");
    println!("   • Cross-domain workflow orchestration");
    println!("   • Domain event processing simulation");
    println!("   • Parallel domain coordination");
    println!("   • Orchestration state tracking");
    println!("   • Event-driven architecture patterns");
    
    println!("\n✨ NATS orchestration demonstration completed!");
    println!("In a real implementation, this would use actual NATS messaging");
    println!("for distributed event publishing and subscription across domains.");
    
    Ok(())
}