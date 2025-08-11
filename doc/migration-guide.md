# Migration Guide: Consolidating to Shared Workflow Domain

## Overview

```mermaid
flowchart TD
    A[Domain-Specific Workflows] --> B{Migration Process}
    B --> C[Phase 1: Dependencies]
    B --> D[Phase 2: Data Migration]
    B --> E[Phase 3: API Updates]
    B --> F[Phase 4: Testing & Validation]
    C --> G[Consolidated Workflow Domain]
    D --> G
    E --> G
    F --> G
    
    classDef primary fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#fff
    classDef secondary fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#fff
    classDef choice fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    classDef result fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    classDef start fill:#2D3436,stroke:#000,stroke-width:4px,color:#fff
    
    class A start
    class B choice
    class C,D,E,F secondary
    class G result
```

This guide provides step-by-step instructions for migrating domain-specific workflow implementations to use the consolidated cim-domain-workflow system.

## Prerequisites

```mermaid
graph LR
    A[Prerequisites] --> B[Design Understanding]
    A --> C[Codebase Access]
    A --> D[Category Theory Knowledge]
    
    B --> B1[Consolidated Workflow Design]
    C --> C1[Domain-specific Code]
    D --> D1[CIM Principles]
    
    classDef primary fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#fff
    classDef secondary fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#fff
    classDef choice fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    classDef result fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    classDef start fill:#2D3436,stroke:#000,stroke-width:4px,color:#fff
    
    class A start
    class B,C,D secondary
    class B1,C1,D1 result
```

- Understanding of the [Consolidated Workflow Design](./consolidated-workflow-design.md)
- Access to domain-specific codebase
- Basic understanding of Category Theory principles in CIM

## Migration Phases

```mermaid
sequenceDiagram
    participant D as Domain Code
    participant M as Migration Process
    participant C as Compatibility Layer
    participant W as Workflow Core
    
    D->>M: Start Migration
    M->>C: Create Compatibility Layer
    C->>W: Add Core Dependency
    W-->>C: Provide Shared Types
    C-->>D: Gradual API Transition
    D->>W: Direct Integration
    W-->>D: Consolidated Workflows
```

### Phase 1: Dependencies and Compatibility Layer

#### Step 1.1: Add Workflow Core Dependency

Update your domain's `Cargo.toml`:

```toml
[dependencies]
# Add the shared workflow core
cim-workflow-core = { path = "../cim-domain-workflow" }

# Keep existing dependencies for gradual migration
# ... existing dependencies
```

#### Step 1.2: Create Compatibility Layer

Create `src/workflow/compatibility.rs` in your domain:

```rust
//! Compatibility layer for gradual migration to shared workflow system

// Re-export shared types with domain-specific aliases
pub use cim_workflow_core::primitives::{
    WorkflowId as CoreWorkflowId,
    StepId as CoreStepId, 
    WorkflowContext as CoreWorkflowContext,
    WorkflowStatus as CoreWorkflowStatus,
};

// Maintain existing types during transition
pub type DomainWorkflowId = CoreWorkflowId;
pub type DomainStepId = CoreStepId;
pub type DomainWorkflowContext = CoreWorkflowContext;

// Legacy type aliases (remove after migration)
#[deprecated(note = "Use CoreWorkflowId instead")]
pub type WorkflowId = CoreWorkflowId;

#[deprecated(note = "Use CoreStepId instead")]  
pub type StepId = CoreStepId;
```

#### Step 1.3: Update Module Imports

Update your domain's `src/lib.rs` or workflow module:

```rust
// Old domain-specific exports
// pub use workflow::{WorkflowId, StepId}; // Remove these

// New shared exports with compatibility
pub use workflow::compatibility::{
    DomainWorkflowId as WorkflowId,  // Temporary alias
    DomainStepId as StepId,          // Temporary alias
    CoreWorkflowContext as WorkflowContext,
};

// Re-export core functionality
pub use cim_workflow_core::{
    WorkflowEngine,
    WorkflowTemplate,
    CimWorkflowEvent,
};
```

### Phase 2: Domain Extension Implementation

#### Step 2.1: Implement Domain Extension

Create `src/workflow/extension.rs`:

```rust
//! Domain-specific workflow extension

use cim_workflow_core::{
    DomainWorkflowExtension, 
    WorkflowContext, 
    StepResult, 
    WorkflowResult,
    WorkflowError
};
use async_trait::async_trait;
use std::sync::Arc;

/// Your domain's workflow extension
pub struct YourDomainWorkflowExtension {
    // Inject your domain services
    domain_service: Arc<dyn YourDomainService>,
    notification_service: Arc<dyn NotificationService>,
}

impl YourDomainWorkflowExtension {
    pub fn new(
        domain_service: Arc<dyn YourDomainService>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            domain_service,
            notification_service,
        }
    }
}

#[async_trait]
impl DomainWorkflowExtension for YourDomainWorkflowExtension {
    fn domain(&self) -> &'static str { 
        "your_domain" // Replace with your domain name
    }
    
    async fn execute_domain_step(
        &self,
        step_type: &str,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<StepResult> {
        match step_type {
            // Map your existing workflow steps to the new system
            "your_domain::step1" => self.execute_step1(context).await,
            "your_domain::step2" => self.execute_step2(context).await,
            "your_domain::approval" => self.execute_approval(context).await,
            _ => Err(WorkflowError::UnsupportedStepType { 
                step_type: step_type.to_string() 
            })
        }
    }
    
    fn validate_context(&self, context: &WorkflowContext) -> WorkflowResult<()> {
        // Validate domain-specific context requirements
        if let Some(domain_ext) = context.get_domain_extension("your_domain") {
            // Check for required fields
            if domain_ext.data.get("entity_id").is_none() {
                return Err(WorkflowError::MissingContextData { 
                    field: "entity_id".to_string() 
                });
            }
        } else {
            return Err(WorkflowError::MissingDomainExtension { 
                domain: "your_domain".to_string() 
            });
        }
        Ok(())
    }
    
    fn transform_context(
        &self, 
        context: &WorkflowContext, 
        target_domain: &str
    ) -> WorkflowResult<serde_json::Value> {
        // Transform context for cross-domain workflows
        match target_domain {
            "document" => {
                // Extract data relevant to document domain
                let entity_id = context
                    .get_domain_extension("your_domain")
                    .and_then(|ext| ext.data.get("entity_id"))
                    .ok_or(WorkflowError::MissingContextData { 
                        field: "entity_id".to_string() 
                    })?;
                    
                Ok(serde_json::json!({
                    "document_id": entity_id,
                    "source_domain": "your_domain"
                }))
            }
            _ => Err(WorkflowError::UnsupportedTransformation { 
                from: "your_domain".to_string(),
                to: target_domain.to_string()
            })
        }
    }
}

impl YourDomainWorkflowExtension {
    async fn execute_step1(&self, context: &mut WorkflowContext) -> WorkflowResult<StepResult> {
        // Implement your domain-specific logic
        let entity_id = context
            .get_domain_extension("your_domain")
            .and_then(|ext| ext.data.get("entity_id"))
            .ok_or(WorkflowError::MissingContextData { 
                field: "entity_id".to_string() 
            })?;
            
        // Execute domain service logic
        let result = self.domain_service.execute_step1(entity_id).await?;
        
        Ok(StepResult::Completed { 
            output: serde_json::to_value(result)?
        })
    }
    
    async fn execute_step2(&self, context: &mut WorkflowContext) -> WorkflowResult<StepResult> {
        // Similar implementation for other steps
        todo!("Implement your domain-specific step")
    }
    
    async fn execute_approval(&self, context: &mut WorkflowContext) -> WorkflowResult<StepResult> {
        // Approval step implementation
        todo!("Implement approval logic")
    }
}
```

#### Step 2.2: Create Domain Workflow Service

Create `src/workflow/service.rs`:

```rust
//! Domain workflow service that integrates with shared workflow core

use cim_workflow_core::{
    WorkflowEngine, 
    WorkflowTemplate,
    WorkflowContext,
    WorkflowInstanceId,
    CimWorkflowEvent,
};
use super::extension::YourDomainWorkflowExtension;
use std::sync::Arc;

pub struct YourDomainWorkflowService {
    core_engine: Arc<WorkflowEngine>,
    domain_extension: Arc<YourDomainWorkflowExtension>,
}

impl YourDomainWorkflowService {
    pub fn new(
        domain_service: Arc<dyn YourDomainService>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        let domain_extension = Arc::new(YourDomainWorkflowExtension::new(
            domain_service,
            notification_service,
        ));
        
        let mut core_engine = WorkflowEngine::new();
        core_engine.register_domain_extension(domain_extension.clone());
        
        Self {
            core_engine: Arc::new(core_engine),
            domain_extension,
        }
    }
    
    /// Start a domain-specific workflow
    pub async fn start_workflow(
        &self,
        template_id: &str,
        entity_id: &str,
        initiated_by: Option<String>,
    ) -> WorkflowResult<WorkflowInstanceId> {
        // Create domain-specific context
        let mut context = WorkflowContext::new();
        context.add_domain_extension(
            "your_domain".to_string(),
            serde_json::json!({
                "entity_id": entity_id,
                "initiated_by": initiated_by,
            }),
            "1.0".to_string(),
        );
        
        // Start workflow using core engine
        let instance_id = self.core_engine.start_workflow_from_template(
            template_id,
            context,
        ).await?;
        
        Ok(instance_id)
    }
    
    /// Execute next steps in workflow
    pub async fn execute_next_steps(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> WorkflowResult<Vec<CimWorkflowEvent>> {
        self.core_engine.execute_next_steps(instance_id).await
    }
    
    /// Complete a manual step
    pub async fn complete_manual_step(
        &self,
        instance_id: WorkflowInstanceId,
        step_id: StepId,
        completed_by: String,
        form_data: serde_json::Value,
    ) -> WorkflowResult<()> {
        self.core_engine.complete_manual_step(
            instance_id,
            step_id,
            completed_by,
            form_data,
        ).await
    }
}
```

### Phase 3: Event Migration

#### Step 3.1: Migrate Domain Events

Replace existing domain events with CIM-compliant events:

```rust
//! Migrated domain events using shared event system

use cim_workflow_core::{
    CimWorkflowEvent,
    WorkflowEventType,
    WorkflowInstanceId,
    MessageIdentity,
};

// Old domain-specific event (to be removed)
/*
pub struct YourDomainWorkflowStarted {
    pub entity_id: EntityId,
    pub workflow_id: WorkflowId,
    pub started_by: Option<String>,
}
*/

// New approach: Use shared event system with domain-specific helper
pub struct YourDomainWorkflowEvents;

impl YourDomainWorkflowEvents {
    pub fn workflow_started(
        instance_id: WorkflowInstanceId,
        entity_id: &str,
        started_by: Option<String>,
        correlation_id: Option<CorrelationId>,
    ) -> CimWorkflowEvent {
        let mut context = WorkflowContext::new();
        context.add_domain_extension(
            "your_domain".to_string(),
            serde_json::json!({
                "entity_id": entity_id,
                "started_by": started_by,
            }),
            "1.0".to_string(),
        );
        
        CimWorkflowEvent::new_correlated(
            instance_id,
            "your_domain".to_string(),
            WorkflowEventType::WorkflowStarted {
                workflow_id: WorkflowId::from(instance_id),
                context,
                started_by,
            },
            correlation_id,
            None,
        )
    }
    
    pub fn step_completed(
        instance_id: WorkflowInstanceId,
        step_id: StepId,
        step_type: &str,
        output: serde_json::Value,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> CimWorkflowEvent {
        CimWorkflowEvent::new_correlated(
            instance_id,
            "your_domain".to_string(),
            WorkflowEventType::StepCompleted {
                step_id,
                step_type: step_type.to_string(),
                output,
                duration_seconds: 0, // Calculate actual duration
            },
            Some(correlation_id),
            Some(causation_id),
        )
    }
}
```

#### Step 3.2: Update Event Handlers

Update your event handlers to use the new event system:

```rust
//! Updated event handlers for shared workflow events

use cim_workflow_core::{CimWorkflowEvent, WorkflowEventType};

pub struct YourDomainWorkflowEventHandler {
    // Your existing dependencies
}

impl YourDomainWorkflowEventHandler {
    pub async fn handle_workflow_event(&self, event: CimWorkflowEvent) -> Result<(), Error> {
        // Only handle events from your domain
        if event.source_domain != "your_domain" {
            return Ok(());
        }
        
        match &event.event {
            WorkflowEventType::WorkflowStarted { workflow_id, context, started_by } => {
                self.handle_workflow_started(&event, workflow_id, context, started_by).await?;
            }
            WorkflowEventType::WorkflowCompleted { workflow_id, final_context, duration_seconds } => {
                self.handle_workflow_completed(&event, workflow_id, final_context, *duration_seconds).await?;
            }
            WorkflowEventType::StepCompleted { step_id, step_type, output, duration_seconds } => {
                self.handle_step_completed(&event, step_id, step_type, output, *duration_seconds).await?;
            }
            WorkflowEventType::DomainSpecific { domain, event_type, data } => {
                if domain == "your_domain" {
                    self.handle_domain_specific_event(&event, event_type, data).await?;
                }
            }
            _ => {
                // Handle other event types as needed
            }
        }
        
        Ok(())
    }
    
    async fn handle_workflow_started(
        &self,
        event: &CimWorkflowEvent,
        workflow_id: &WorkflowId,
        context: &WorkflowContext,
        started_by: &Option<String>,
    ) -> Result<(), Error> {
        // Extract domain-specific context
        if let Some(domain_ext) = context.get_domain_extension("your_domain") {
            let entity_id = domain_ext.data
                .get("entity_id")
                .and_then(|v| v.as_str())
                .ok_or(Error::MissingContextData)?;
                
            // Execute your domain-specific logic
            self.on_workflow_started(entity_id, started_by).await?;
        }
        
        Ok(())
    }
}
```

### Phase 4: NATS Subject Migration

#### Step 4.1: Update Subject Patterns

Update your NATS subjects to follow the new standardized patterns:

```rust
//! Updated NATS subject patterns

// Old domain-specific subjects (to be removed)
/*
const OLD_WORKFLOW_STARTED: &str = "your_domain.workflow.started";
const OLD_STEP_COMPLETED: &str = "your_domain.workflow.step_completed";
*/

// New standardized subjects
use cim_workflow_core::messaging::WorkflowSubjects;

impl WorkflowSubjects {
    /// Subject for domain workflow events: events.workflow.{domain}.{event_type}.{instance_id}
    pub fn domain_workflow_event(domain: &str, event_type: &str, instance_id: &WorkflowInstanceId) -> String {
        format!("events.workflow.{}.{}.{}", domain, event_type, instance_id.as_uuid())
    }
    
    /// Subscription pattern for all domain workflow events: events.workflow.{domain}.>
    pub fn domain_workflow_subscription(domain: &str) -> String {
        format!("events.workflow.{}.*", domain)
    }
    
    /// Cross-domain workflow events: events.workflow.integration.>
    pub fn cross_domain_subscription() -> String {
        "events.workflow.integration.*".to_string()
    }
}

// Update your event publishers
pub struct YourDomainEventPublisher {
    nats_client: Arc<NatsClient>,
}

impl YourDomainEventPublisher {
    pub async fn publish_workflow_event(&self, event: CimWorkflowEvent) -> Result<(), Error> {
        let subject = event.to_nats_subject(); // This uses the standardized subject format
        let payload = serde_json::to_vec(&event)?;
        
        self.nats_client.publish(&subject, payload).await?;
        Ok(())
    }
}

// Update your event subscribers
pub struct YourDomainEventSubscriber {
    workflow_handler: Arc<YourDomainWorkflowEventHandler>,
}

impl YourDomainEventSubscriber {
    pub async fn subscribe_to_workflow_events(&self) -> Result<(), Error> {
        let subject = WorkflowSubjects::domain_workflow_subscription("your_domain");
        
        self.nats_client
            .subscribe(&subject, move |msg| {
                let event: CimWorkflowEvent = serde_json::from_slice(&msg.data)?;
                self.workflow_handler.handle_workflow_event(event).await
            })
            .await?;
            
        Ok(())
    }
}
```

## Domain-Specific Examples

### Document Domain Migration Example

```rust
// Document workflow extension
pub struct DocumentWorkflowExtension {
    document_service: Arc<dyn DocumentService>,
    content_intelligence: Arc<dyn ContentIntelligenceService>,
}

#[async_trait]
impl DomainWorkflowExtension for DocumentWorkflowExtension {
    fn domain(&self) -> &'static str { "document" }
    
    async fn execute_domain_step(
        &self,
        step_type: &str,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<StepResult> {
        match step_type {
            "document::review" => {
                let doc_id = self.extract_document_id(context)?;
                let review_result = self.document_service.start_review(doc_id).await?;
                Ok(StepResult::Completed { 
                    output: serde_json::to_value(review_result)?
                })
            }
            "document::extract_entities" => {
                let doc_id = self.extract_document_id(context)?;
                let entities = self.content_intelligence.extract_entities(doc_id).await?;
                Ok(StepResult::Completed {
                    output: serde_json::json!({ "entities": entities })
                })
            }
            "document::verify_cid_chain" => {
                let doc_id = self.extract_document_id(context)?;
                let is_valid = self.document_service.verify_cid_chain(doc_id).await?;
                Ok(StepResult::Completed {
                    output: serde_json::json!({ "cid_chain_valid": is_valid })
                })
            }
            _ => Err(WorkflowError::UnsupportedStepType { 
                step_type: step_type.to_string() 
            })
        }
    }
}
```

### Person Domain Migration Example

```rust
// Person workflow extension
pub struct PersonWorkflowExtension {
    person_service: Arc<dyn PersonService>,
    identity_service: Arc<dyn IdentityVerificationService>,
}

#[async_trait]
impl DomainWorkflowExtension for PersonWorkflowExtension {
    fn domain(&self) -> &'static str { "person" }
    
    async fn execute_domain_step(
        &self,
        step_type: &str,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<StepResult> {
        match step_type {
            "person::verify_identity" => {
                let person_id = self.extract_person_id(context)?;
                let verification_result = self.identity_service.verify_identity(person_id).await?;
                Ok(StepResult::Completed {
                    output: serde_json::to_value(verification_result)?
                })
            }
            "person::create_profile" => {
                let person_data = self.extract_person_data(context)?;
                let profile = self.person_service.create_profile(person_data).await?;
                Ok(StepResult::Completed {
                    output: serde_json::to_value(profile)?
                })
            }
            _ => Err(WorkflowError::UnsupportedStepType { 
                step_type: step_type.to_string() 
            })
        }
    }
}
```

## Testing Your Migration

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cim_workflow_core::testing::MockWorkflowEngine;

    #[tokio::test]
    async fn test_domain_extension() {
        let domain_service = Arc::new(MockYourDomainService::new());
        let notification_service = Arc::new(MockNotificationService::new());
        
        let extension = YourDomainWorkflowExtension::new(
            domain_service,
            notification_service,
        );
        
        let mut context = WorkflowContext::new();
        context.add_domain_extension(
            "your_domain".to_string(),
            serde_json::json!({"entity_id": "test123"}),
            "1.0".to_string(),
        );
        
        let result = extension.execute_domain_step(
            "your_domain::step1",
            &mut context,
        ).await;
        
        assert!(result.is_ok());
        matches!(result.unwrap(), StepResult::Completed { .. });
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_cross_domain_workflow() {
    let document_service = setup_document_service().await;
    let person_service = setup_person_service().await;
    
    let workflow_service = WorkflowService::new()
        .with_extension(DocumentWorkflowExtension::new(document_service))
        .with_extension(PersonWorkflowExtension::new(person_service));
    
    // Start cross-domain workflow
    let template = WorkflowTemplate::cross_domain_workflow();
    let instance_id = workflow_service.start_workflow_from_template(
        &template.id,
        WorkflowContext::with_domain_extension(
            "document",
            serde_json::json!({"document_id": "doc123"})
        ),
    ).await.unwrap();
    
    // Execute workflow steps
    let events = workflow_service.execute_next_steps(instance_id).await.unwrap();
    
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| matches!(e.event, WorkflowEventType::CrossDomainTransition { .. })));
}
```

## Troubleshooting

### Common Issues

1. **Type Conflicts**: Use compatibility layer and gradual migration
2. **Event Schema Mismatches**: Ensure proper domain extensions in context
3. **Missing Context Data**: Validate context in domain extension
4. **NATS Subject Errors**: Follow standardized subject patterns
5. **Cross-Domain Failures**: Check domain transformation logic

### Rollback Strategy

If migration issues arise, you can rollback by:

1. Removing cim-workflow-core dependency
2. Reverting to domain-specific types
3. Restoring original event handlers
4. Updating NATS subjects back to domain-specific patterns

### Getting Help

- Review the [Consolidated Workflow Design](./consolidated-workflow-design.md)
- Check the cim-domain-workflow examples
- Consult the CIM architecture documentation
- Join the CIM developer discussions

## Validation Checklist

Before completing migration:

- [ ] Domain extension implements all required workflow steps
- [ ] All domain events migrated to CimWorkflowEvent system
- [ ] NATS subjects follow standardized patterns
- [ ] Unit tests pass for domain extension
- [ ] Integration tests pass for cross-domain workflows
- [ ] Event correlation works across domain boundaries
- [ ] Performance meets requirements (< 100ms workflow execution)
- [ ] All workflow instances have proper CID integrity
- [ ] Documentation updated for domain-specific workflow patterns

## Next Steps

After successful migration:

1. Remove deprecated domain-specific workflow code
2. Update documentation and examples
3. Train team members on new workflow patterns
4. Monitor performance and adjust as needed
5. Consider implementing domain-specific workflow templates