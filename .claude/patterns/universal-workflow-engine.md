# Universal Workflow Engine Pattern

## Overview

The Universal Workflow Engine pattern enables a single workflow execution engine to serve all CIM domains through domain extensions and composition rather than inheritance.

## Problem

Traditional approaches create separate workflow engines per domain:
- `DocumentWorkflowEngine` for document processing
- `PersonWorkflowEngine` for person management
- `OrganizationWorkflowEngine` for org operations
- etc.

This leads to:
- Code duplication across domains
- Inconsistent workflow behavior
- Impossible cross-domain workflows
- Maintenance nightmare

## Solution

A single `WorkflowEngine` that:
1. Executes domain-agnostic workflow logic
2. Delegates domain-specific operations to extensions
3. Coordinates cross-domain workflows
4. Maintains consistent event handling

## Implementation Pattern

### Core Engine Structure

```rust
pub struct WorkflowEngine {
    /// Domain-specific extensions
    domain_extensions: HashMap<String, Arc<dyn DomainWorkflowExtension>>,
    
    /// State machine for workflow lifecycle
    state_machine: WorkflowStateMachine,
    
    /// Event correlation for cross-domain workflows
    event_correlator: WorkflowEventCorrelator,
    
    /// Template engine for workflow instantiation
    template_engine: TemplateEngine,
}

impl WorkflowEngine {
    /// Register domain-specific extension
    pub fn register_extension(&mut self, extension: Arc<dyn DomainWorkflowExtension>) {
        self.domain_extensions.insert(extension.domain().to_string(), extension);
    }
    
    /// Execute workflow step (delegates to appropriate domain)
    pub async fn execute_step(
        &self,
        instance_id: WorkflowInstanceId,
        step_id: StepId,
    ) -> WorkflowResult<Vec<CimWorkflowEvent>> {
        let step = self.get_step(instance_id, step_id)?;
        let domain = self.extract_domain_from_step_type(&step.step_type)?;
        
        if let Some(extension) = self.domain_extensions.get(domain) {
            let mut context = self.get_workflow_context(instance_id)?;
            let result = extension.execute_domain_step(&step.step_type, &mut context).await?;
            
            // Update workflow state and emit events
            self.update_workflow_state(instance_id, step_id, result).await
        } else {
            // Core workflow step (start, complete, etc.)
            self.execute_core_step(instance_id, step_id).await
        }
    }
    
    /// Start cross-domain workflow
    pub async fn start_cross_domain_workflow(
        &self,
        template_id: &str,
        initiating_domain: &str,
        target_domains: Vec<String>,
        initial_context: WorkflowContext,
    ) -> WorkflowResult<WorkflowInstanceId> {
        // Create workflow instance
        let instance_id = WorkflowInstanceId::new();
        
        // Start event correlation
        let correlation_id = self.event_correlator.start_cross_domain_workflow(
            CimWorkflowEvent::workflow_started(instance_id, initiating_domain),
            target_domains.clone(),
        ).await?;
        
        // Execute workflow steps across domains
        self.execute_template_workflow(template_id, instance_id, initial_context, correlation_id).await?;
        
        Ok(instance_id)
    }
}
```

### Domain Extension Interface

```rust
#[async_trait]
pub trait DomainWorkflowExtension: Send + Sync {
    /// Domain identifier (e.g., "document", "person", "organization")
    fn domain(&self) -> &'static str;
    
    /// Execute domain-specific workflow step
    async fn execute_domain_step(
        &self,
        step_type: &str,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<StepResult>;
    
    /// Validate domain-specific context requirements
    fn validate_context(&self, context: &WorkflowContext) -> WorkflowResult<()>;
    
    /// Transform context for cross-domain operations
    fn transform_context(
        &self, 
        context: &WorkflowContext, 
        target_domain: &str
    ) -> WorkflowResult<serde_json::Value>;
    
    /// Get supported step types
    fn supported_step_types(&self) -> Vec<String> {
        vec![
            format!("{}::create", self.domain()),
            format!("{}::update", self.domain()),
            format!("{}::delete", self.domain()),
            format!("{}::approve", self.domain()),
            format!("{}::notify", self.domain()),
        ]
    }
}
```

### Example: Document Domain Extension

```rust
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
                
                // Update context with review results
                context.add_domain_extension(
                    "document".to_string(),
                    serde_json::json!({
                        "review_id": review_result.id,
                        "status": "in_review",
                        "assigned_reviewer": review_result.reviewer,
                    }),
                    "1.0".to_string(),
                );
                
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
    
    fn validate_context(&self, context: &WorkflowContext) -> WorkflowResult<()> {
        if let Some(doc_ext) = context.get_domain_extension("document") {
            if doc_ext.data.get("document_id").is_none() {
                return Err(WorkflowError::MissingContextData { 
                    field: "document_id".to_string() 
                });
            }
        } else {
            return Err(WorkflowError::MissingDomainExtension { 
                domain: "document".to_string() 
            });
        }
        Ok(())
    }
    
    fn transform_context(
        &self, 
        context: &WorkflowContext, 
        target_domain: &str
    ) -> WorkflowResult<serde_json::Value> {
        match target_domain {
            "person" => {
                // Extract person-relevant data from document context
                let doc_ext = context.get_domain_extension("document")
                    .ok_or(WorkflowError::MissingDomainExtension { domain: "document".to_string() })?;
                    
                let author_id = doc_ext.data.get("author_id")
                    .ok_or(WorkflowError::MissingContextData { field: "author_id".to_string() })?;
                    
                Ok(serde_json::json!({
                    "person_id": author_id,
                    "notification_type": "document_review_requested",
                    "source_domain": "document"
                }))
            }
            
            "organization" => {
                // Extract org-relevant data from document context  
                let doc_ext = context.get_domain_extension("document")
                    .ok_or(WorkflowError::MissingDomainExtension { domain: "document".to_string() })?;
                    
                let department_id = doc_ext.data.get("department_id")
                    .ok_or(WorkflowError::MissingContextData { field: "department_id".to_string() })?;
                    
                Ok(serde_json::json!({
                    "organization_id": department_id,
                    "approval_level": "department",
                    "source_domain": "document"
                }))
            }
            
            _ => Err(WorkflowError::UnsupportedTransformation { 
                from: "document".to_string(),
                to: target_domain.to_string()
            })
        }
    }
    
    fn supported_step_types(&self) -> Vec<String> {
        vec![
            "document::create".to_string(),
            "document::review".to_string(),
            "document::approve".to_string(),
            "document::publish".to_string(),
            "document::extract_entities".to_string(),
            "document::summarize".to_string(),
            "document::verify_cid_chain".to_string(),
        ]
    }
}
```

## Usage Examples

### Single Domain Workflow

```rust
// Document review workflow
let document_extension = Arc::new(DocumentWorkflowExtension::new(
    document_service,
    content_intelligence,
));

let mut engine = WorkflowEngine::new();
engine.register_extension(document_extension);

let context = WorkflowContext::with_domain_extension(
    "document",
    serde_json::json!({
        "document_id": "doc_123",
        "author_id": "user_456",
    })
);

let instance_id = engine.start_workflow_from_template(
    "document_review",
    context,
).await?;
```

### Cross-Domain Workflow

```rust
// Document review → Person notification → Organization approval
let context = WorkflowContext::with_domain_extension(
    "document",
    serde_json::json!({
        "document_id": "doc_123",
        "author_id": "user_456", 
        "department_id": "dept_789",
    })
);

let instance_id = engine.start_cross_domain_workflow(
    "cross_domain_approval",
    "document",
    vec!["person".to_string(), "organization".to_string()],
    context,
).await?;
```

## Benefits

1. **Code Reuse**: Single engine implementation shared across all domains
2. **Consistency**: Uniform workflow behavior and patterns
3. **Cross-Domain Support**: Natural multi-domain workflow execution
4. **Maintainability**: Changes to core engine benefit all domains
5. **Performance**: Optimized execution path for all workflows
6. **Observability**: Unified monitoring and debugging

## Integration with CIM Architecture

### Event Correlation
All workflows use CIM-compliant events with mandatory correlation/causation:
```rust
let workflow_event = CimWorkflowEvent::new_correlated(
    instance_id,
    "document".to_string(),
    WorkflowEventType::StepCompleted { 
        step_id, 
        step_type: "document::review".to_string(),
        output: review_result,
    },
    correlation_id,
    causation_id,
);
```

### NATS Subject Patterns
Standardized subjects enable cross-domain subscription:
```
events.workflow.document.step_completed.{instance_id}
events.workflow.person.step_started.{instance_id}  
events.workflow.integration.cross_domain.{correlation_id}
```

### CID Integrity
All workflow events include CID for cryptographic verification:
```rust
workflow_event.event_cid = Some(calculate_event_cid(&workflow_event));
```

This pattern transforms workflows from fragmented domain-specific implementations into a unified, mathematically sound system that serves as the central nervous system of the entire CIM architecture.