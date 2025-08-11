# CIM Domain Workflow Migration Guide

<!-- Copyright 2024 The CIM Consortium. All rights reserved. -->

This comprehensive guide covers all migration scenarios for adopting the CIM Domain Workflow system, including legacy system migrations, domain consolidation, and enterprise deployment strategies.

## Migration Types

### 1. Domain Consolidation Migration
**Scenario**: Migrating from domain-specific workflow implementations to the shared workflow domain.

### 2. Legacy System Migration  
**Scenario**: Migrating from external workflow engines (BPMN, Temporal, Airflow, etc.) to CIM Domain Workflow.

### 3. Brownfield Integration
**Scenario**: Integrating CIM workflows into existing enterprise systems with gradual adoption.

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

---

# Part II: Legacy System Migration

## Legacy System Migration Overview

For organizations migrating from external workflow systems (Temporal, Apache Airflow, BPMN engines, etc.) to CIM Domain Workflow, this section provides comprehensive migration strategies, tools, and best practices.

### Architecture Migration Flow

```mermaid
graph TD
    A[Legacy System] --> B[Assessment Phase]
    B --> C[Data Mapping]
    C --> D[Schema Translation]
    D --> E[Event Migration]
    E --> F[Testing Phase]
    F --> G[Gradual Cutover]
    G --> H[CIM Workflow System]
    
    B --> B1[System Analysis]
    B --> B2[Dependency Mapping]
    B --> B3[Performance Baseline]
    
    C --> C1[Workflow Mapping]
    C --> C2[State Translation]
    C --> C3[Context Migration]
    
    E --> E1[Event Sourcing Setup]
    E --> E2[Message Migration]
    E --> E3[Causation Chains]
    
    F --> F1[Integration Tests]
    F --> F2[Performance Tests]
    F --> F3[Data Validation]
    
    G --> G1[Canary Deployment]
    G --> G2[Blue-Green Switch]
    G --> G3[Rollback Plan]
    
    classDef primary fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#fff
    classDef secondary fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#fff
    classDef process fill:#FFE66D,stroke:#FCC419,stroke-width:2px,color:#000
    classDef result fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    class A primary
    class H result
    class B,C,D,E,F,G secondary
    class B1,B2,B3,C1,C2,C3,E1,E2,E3,F1,F2,F3,G1,G2,G3 process
```

## Legacy Migration Tools

### Migration CLI Tool

Create a command-line interface for migration operations:

```bash
# Install migration tools
cargo install cim-migration-tools

# Initialize migration project
cim-migrate init --legacy-system temporal --output-dir ./migration

# Assess legacy system
cim-migrate assess --config migration/config.toml

# Generate migration scripts
cim-migrate generate --workflows --events --instances

# Execute migration
cim-migrate run --batch-size 100 --parallel-workers 4

# Validate migration
cim-migrate validate --comprehensive
```

### Migration Configuration

```toml
# migration/config.toml
[migration]
legacy_system = "temporal"
dry_run = false
batch_size = 100
parallel_workers = 4
rate_limit_per_second = 50
validation_enabled = true
backup_enabled = true

[legacy_source]
type = "temporal"
endpoint = "temporal.company.com:7233"
namespace = "default"
database_url = "postgresql://user:pass@temporal-db:5432/temporal"

[target]
database_url = "postgresql://user:pass@cim-db:5432/cim_workflows"
nats_endpoint = "nats://localhost:4222"

[validation]
performance_threshold_ms = 100
error_rate_threshold = 0.01
```

## Common Legacy System Migrations

### Temporal to CIM Migration

**Key Challenges:**
- Activity definitions → Step definitions
- Workflow context → CIM context extensions
- Event sourcing format differences
- Signal/query patterns → CIM event patterns

**Migration Strategy:**

1. **Workflow Definition Translation:**
   - Temporal Activities → CIM Steps
   - Workflow Context → Domain Extensions
   - Signal/Query → Event-driven patterns

2. **State Machine Migration:**
   - Temporal's internal state → CIM algebra state
   - History events → CIM event sourcing
   - Timer activities → CIM wait steps

3. **Data Migration:**
   - Workflow executions → CIM instances
   - Activity history → CIM step history
   - Search attributes → CIM metadata

### Apache Airflow to CIM Migration

**Key Challenges:**
- DAG structure → Workflow templates
- Operator types → Step types
- XCom data → Context variables
- Task dependencies → Step dependencies
- Scheduler logic → CIM execution engine

**Migration Strategy:**

1. **DAG Translation:**
   ```python
   # Airflow DAG
   from airflow import DAG
   from airflow.operators.python import PythonOperator
   
   dag = DAG('user_onboarding', schedule_interval='@daily')
   
   validate_user = PythonOperator(
       task_id='validate_user',
       python_callable=validate_user_data,
       dag=dag
   )
   
   send_welcome = PythonOperator(
       task_id='send_welcome',
       python_callable=send_welcome_email,
       dag=dag
   )
   
   validate_user >> send_welcome
   ```

   Becomes:

   ```rust
   // CIM Workflow Template
   let mut template = WorkflowTemplate::new(
       "user_onboarding".to_string(),
       workflow_id.to_string(),
   )?;
   
   template.add_step(
       "validate_user".to_string(),
       "Validate user registration data".to_string(),
       StepType::Automated,
       serde_json::json!({
           "function": "validate_user_data",
           "domain": "user_management"
       }),
   )?;
   
   template.add_step(
       "send_welcome".to_string(),
       "Send welcome email to user".to_string(),
       StepType::Automated,
       serde_json::json!({
           "function": "send_welcome_email",
           "domain": "notification"
       }),
   )?;
   
   template.add_dependency("send_welcome", "validate_user")?;
   ```

### BPMN Engine Migration

**Key Challenges:**
- BPMN XML → CIM templates
- User tasks → Manual steps
- Service tasks → Automated steps
- Gateway logic → Decision steps
- Boundary events → Error handling

---

# Part III: Production Deployment Guide

## Deployment Strategies

### Blue-Green Deployment

Implement zero-downtime migration using blue-green deployment:

```mermaid
graph TB
    subgraph "Blue Environment (Legacy)"
        B1[Load Balancer Blue]
        B2[Legacy Workflow Engine]
        B3[Legacy Database]
        B4[Legacy Message Queue]
        
        B1 --> B2
        B2 --> B3
        B2 --> B4
    end
    
    subgraph "Green Environment (CIM)"
        G1[Load Balancer Green]
        G2[CIM Workflow Engine]
        G3[CIM Database]
        G4[NATS Message System]
        
        G1 --> G2
        G2 --> G3
        G2 --> G4
    end
    
    subgraph "Traffic Management"
        T1[DNS/Router]
        T2[Traffic Splitter]
        
        T1 --> T2
        T2 -->|90%| B1
        T2 -->|10%| G1
    end
    
    subgraph "Migration Tools"
        M1[Data Sync Service]
        M2[Event Bridge]
        M3[Health Monitors]
        
        M1 --> B3
        M1 --> G3
        M2 --> B4
        M2 --> G4
    end
    
    classDef blue fill:#4F8EF7,stroke:#2563EB,stroke-width:2px,color:#fff
    classDef green fill:#22C55E,stroke:#16A34A,stroke-width:2px,color:#fff
    classDef mgmt fill:#F59E0B,stroke:#D97706,stroke-width:2px,color:#000
    classDef tools fill:#8B5CF6,stroke:#7C3AED,stroke-width:2px,color:#fff
    
    class B1,B2,B3,B4 blue
    class G1,G2,G3,G4 green
    class T1,T2 mgmt
    class M1,M2,M3 tools
```

### Canary Deployment Configuration

```yaml
# deployment/canary-config.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: cim-workflow-migration
spec:
  replicas: 10
  strategy:
    canary:
      steps:
      - setWeight: 10
      - pause: {duration: 600s} # 10 minutes
      - analysis:
          templates:
          - templateName: success-rate
          args:
          - name: service-name
            value: cim-workflow-engine
      - setWeight: 50
      - pause: {duration: 1800s} # 30 minutes  
      - analysis:
          templates:
          - templateName: success-rate
          - templateName: latency-check
          args:
          - name: service-name
            value: cim-workflow-engine
      - setWeight: 100
  selector:
    matchLabels:
      app: workflow-engine
  template:
    metadata:
      labels:
        app: workflow-engine
    spec:
      containers:
      - name: workflow-engine
        image: cim-workflow:latest
        env:
        - name: MIGRATION_MODE
          value: "canary"
        - name: LEGACY_FALLBACK
          value: "enabled"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

## Rollback and Contingency Plans

### Automated Rollback System

```bash
#!/bin/bash
# scripts/automated-rollback.sh

set -e

echo "=== CIM Workflow Automated Rollback System ==="

# Configuration
HEALTH_CHECK_ENDPOINT="http://cim-workflow-engine:8080/health"
ERROR_THRESHOLD=5  # 5% error rate triggers rollback
RESPONSE_TIME_THRESHOLD=2000  # 2 seconds
CHECK_INTERVAL=30  # 30 seconds

# Health monitoring function
check_system_health() {
    local error_rate=$(curl -s "$HEALTH_CHECK_ENDPOINT/metrics" | grep error_rate | cut -d' ' -f2)
    local avg_response_time=$(curl -s "$HEALTH_CHECK_ENDPOINT/metrics" | grep avg_response_time | cut -d' ' -f2)
    
    echo "Current metrics: error_rate=${error_rate}%, response_time=${avg_response_time}ms"
    
    if (( $(echo "$error_rate > $ERROR_THRESHOLD" | bc -l) )); then
        echo "ERROR: Error rate ${error_rate}% exceeds threshold ${ERROR_THRESHOLD}%"
        return 1
    fi
    
    if (( $(echo "$avg_response_time > $RESPONSE_TIME_THRESHOLD" | bc -l) )); then
        echo "ERROR: Response time ${avg_response_time}ms exceeds threshold ${RESPONSE_TIME_THRESHOLD}ms"
        return 1
    fi
    
    return 0
}

# Rollback function
perform_rollback() {
    echo "INITIATING AUTOMATED ROLLBACK"
    
    # Step 1: Stop new traffic to CIM system
    kubectl patch ingress workflow-ingress --patch '{
        "spec": {
            "rules": [{
                "host": "workflow.company.com",
                "http": {
                    "paths": [{
                        "path": "/",
                        "pathType": "Prefix",
                        "backend": {
                            "service": {
                                "name": "legacy-workflow-service",
                                "port": {"number": 8080}
                            }
                        }
                    }]
                }
            }]
        }
    }'
    
    # Step 2: Scale down CIM deployment
    kubectl scale deployment cim-workflow-engine --replicas=0
    
    # Step 3: Send alerts
    curl -X POST -H 'Content-type: application/json' \
        --data "{\"text\":\"🚨 AUTOMATED ROLLBACK INITIATED 🚨\\nTime: $(date)\\nReason: System health check failed\"}" \
        $SLACK_WEBHOOK_URL
    
    echo "Rollback completed successfully"
}

# Main monitoring loop
while true; do
    if ! check_system_health; then
        perform_rollback
        exit 0
    fi
    
    sleep $CHECK_INTERVAL
done
```

### Migration Monitoring Dashboard

Create comprehensive monitoring for migration progress:

```yaml
# monitoring/grafana-dashboard.json
{
  "dashboard": {
    "title": "CIM Workflow Migration",
    "panels": [
      {
        "title": "Migration Progress",
        "type": "stat",
        "targets": [
          {
            "expr": "migration_workflows_completed_total",
            "legendFormat": "Workflows Migrated"
          }
        ]
      },
      {
        "title": "Error Rate Comparison",
        "type": "graph", 
        "targets": [
          {
            "expr": "rate(legacy_system_errors_total[5m])",
            "legendFormat": "Legacy System"
          },
          {
            "expr": "rate(cim_workflow_errors_total[5m])",
            "legendFormat": "CIM System"
          }
        ]
      },
      {
        "title": "Performance Comparison",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, legacy_workflow_duration_seconds)",
            "legendFormat": "Legacy P95"
          },
          {
            "expr": "histogram_quantile(0.95, cim_workflow_duration_seconds)", 
            "legendFormat": "CIM P95"
          }
        ]
      }
    ]
  }
}
```

## Post-Migration Validation

### Comprehensive Validation Checklist

- [ ] **Data Integrity**
  - [ ] All workflow definitions migrated correctly
  - [ ] Historical instances preserved with proper state
  - [ ] Event sourcing maintains causation chains
  - [ ] No data corruption detected

- [ ] **Functional Validation**
  - [ ] All workflow patterns execute correctly
  - [ ] Cross-domain workflows function properly
  - [ ] Error handling works as expected
  - [ ] Integration points remain functional

- [ ] **Performance Validation**
  - [ ] Throughput matches or exceeds legacy system
  - [ ] Response times within acceptable limits
  - [ ] Resource utilization optimized
  - [ ] Scalability requirements met

- [ ] **Operations Validation**
  - [ ] Monitoring and alerting functional
  - [ ] Backup and recovery procedures tested
  - [ ] Security controls properly implemented
  - [ ] Compliance requirements satisfied

### Migration Success Criteria

1. **Zero Data Loss**: All workflow data successfully migrated
2. **Performance Parity**: CIM system performs at least as well as legacy
3. **Functional Completeness**: All business workflows operational
4. **Operational Readiness**: Monitoring, alerting, and support procedures active
5. **Team Readiness**: Operations team trained on CIM system

---

# Part IV: Enterprise Integration

## Enterprise Deployment Scripts

### Kubernetes Deployment

```yaml
# deployment/k8s/cim-workflow-engine.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cim-workflow-engine
  labels:
    app: cim-workflow-engine
    version: v1.0.0
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cim-workflow-engine
  template:
    metadata:
      labels:
        app: cim-workflow-engine
    spec:
      containers:
      - name: workflow-engine
        image: cim-workflow:1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: cim-db-secret
              key: url
        - name: NATS_URL
          value: "nats://nats-cluster:4222"
        - name: LOG_LEVEL
          value: "info"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi" 
            cpu: "1000m"
```

### Helm Chart

```yaml
# deployment/helm/values.yaml
replicaCount: 3

image:
  repository: cim-workflow
  tag: "1.0.0"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080

ingress:
  enabled: true
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: workflow.company.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: workflow-tls
      hosts:
        - workflow.company.com

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80

postgresql:
  enabled: true
  global:
    postgresql:
      auth:
        postgresPassword: "secure-password"
        database: "cim_workflows"

nats:
  enabled: true
  cluster:
    enabled: true
    replicas: 3
```

This comprehensive migration guide provides organizations with complete tools and strategies for successfully migrating to the CIM Domain Workflow system, covering everything from legacy system assessment to production deployment and post-migration validation.