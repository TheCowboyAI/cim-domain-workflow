# Domain Composition Pattern

## Overview

The Domain Composition pattern enables domain-specific workflow logic through composition rather than inheritance, allowing the universal workflow engine to serve all CIM domains while maintaining clean domain boundaries.

## Mathematical Foundation: Category Theory

This pattern implements domains as **Categories** with **Functorial** composition:

- **Objects**: Domain entities (Documents, Persons, Organizations)
- **Morphisms**: Domain operations (create, update, approve, etc.)
- **Functors**: Cross-domain transformations preserving structure
- **Natural Transformations**: Context transformations between domains

## Problem

Traditional inheritance-based approaches create tight coupling:

```rust
// BAD: Inheritance creates tight coupling
trait BaseWorkflow {
    fn execute(&self);
}

trait DocumentWorkflow: BaseWorkflow {
    fn review_document(&self);
}

trait PersonWorkflow: BaseWorkflow {
    fn verify_identity(&self);
}
```

Issues:
- Tight coupling between domains
- Difficult to share behavior across domains
- Complex inheritance hierarchies
- Impossible to compose multiple domain behaviors

## Solution: Compositional Domain Extensions

Use composition to inject domain-specific behavior into a universal workflow engine:

```rust
// GOOD: Composition enables flexible domain integration
pub struct WorkflowEngine {
    domain_extensions: HashMap<String, Arc<dyn DomainWorkflowExtension>>,
}

#[async_trait]
pub trait DomainWorkflowExtension: Send + Sync {
    fn domain(&self) -> &'static str;
    async fn execute_domain_step(&self, step_type: &str, context: &mut WorkflowContext) -> WorkflowResult<StepResult>;
}
```

## Implementation Pattern

### 1. Domain Extension Trait

```rust
#[async_trait]
pub trait DomainWorkflowExtension: Send + Sync {
    /// Domain identifier (must be unique across CIM)
    fn domain(&self) -> &'static str;
    
    /// Execute domain-specific workflow step
    async fn execute_domain_step(
        &self,
        step_type: &str,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<StepResult>;
    
    /// Validate domain-specific context requirements
    fn validate_context(&self, context: &WorkflowContext) -> WorkflowResult<()>;
    
    /// Transform context for cross-domain operations (Functorial mapping)
    fn transform_context(
        &self, 
        context: &WorkflowContext, 
        target_domain: &str
    ) -> WorkflowResult<serde_json::Value>;
    
    /// Get list of supported step types
    fn supported_step_types(&self) -> Vec<String>;
    
    /// Initialize domain-specific context
    fn initialize_context(&self, entity_id: &str) -> WorkflowResult<WorkflowContext> {
        let mut context = WorkflowContext::new();
        context.add_domain_extension(
            self.domain().to_string(),
            serde_json::json!({ "entity_id": entity_id }),
            "1.0".to_string(),
        );
        Ok(context)
    }
}
```

### 2. Domain Extension Registry

```rust
pub struct DomainExtensionRegistry {
    extensions: HashMap<String, Arc<dyn DomainWorkflowExtension>>,
    step_type_mappings: HashMap<String, String>, // step_type -> domain
}

impl DomainExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            step_type_mappings: HashMap::new(),
        }
    }
    
    pub fn register_extension(&mut self, extension: Arc<dyn DomainWorkflowExtension>) {
        let domain = extension.domain().to_string();
        
        // Register step type mappings
        for step_type in extension.supported_step_types() {
            self.step_type_mappings.insert(step_type, domain.clone());
        }
        
        self.extensions.insert(domain, extension);
    }
    
    pub fn get_extension_for_step(&self, step_type: &str) -> Option<&Arc<dyn DomainWorkflowExtension>> {
        let domain = self.step_type_mappings.get(step_type)?;
        self.extensions.get(domain)
    }
    
    pub fn get_extension(&self, domain: &str) -> Option<&Arc<dyn DomainWorkflowExtension>> {
        self.extensions.get(domain)
    }
    
    pub fn list_domains(&self) -> Vec<String> {
        self.extensions.keys().cloned().collect()
    }
}
```

### 3. Contextual Domain Data

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Universal variables accessible to all workflows
    pub variables: HashMap<String, serde_json::Value>,
    
    /// Domain-specific extensions (compositional data)
    pub extensions: HashMap<String, DomainExtension>,
    
    /// Execution metadata for correlation/causation
    pub metadata: ExecutionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExtension {
    pub domain: String,
    pub data: serde_json::Value,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkflowContext {
    /// Add domain-specific data (composition)
    pub fn add_domain_extension(&mut self, domain: String, data: serde_json::Value, version: String) {
        let now = Utc::now();
        self.extensions.insert(
            domain.clone(), 
            DomainExtension { 
                domain, 
                data, 
                version,
                created_at: now,
                updated_at: now,
            }
        );
    }
    
    /// Get domain-specific data
    pub fn get_domain_extension(&self, domain: &str) -> Option<&DomainExtension> {
        self.extensions.get(domain)
    }
    
    /// Update domain extension data
    pub fn update_domain_extension(&mut self, domain: &str, data: serde_json::Value) -> WorkflowResult<()> {
        if let Some(extension) = self.extensions.get_mut(domain) {
            extension.data = data;
            extension.updated_at = Utc::now();
            Ok(())
        } else {
            Err(WorkflowError::MissingDomainExtension { 
                domain: domain.to_string() 
            })
        }
    }
}
```

## Example Implementations

### Document Domain Extension

```rust
pub struct DocumentWorkflowExtension {
    document_service: Arc<dyn DocumentService>,
    content_intelligence: Arc<dyn ContentIntelligenceService>,
    review_service: Arc<dyn DocumentReviewService>,
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
            "document::create" => self.execute_create_step(context).await,
            "document::review" => self.execute_review_step(context).await,
            "document::approve" => self.execute_approve_step(context).await,
            "document::publish" => self.execute_publish_step(context).await,
            "document::extract_entities" => self.execute_entity_extraction(context).await,
            "document::summarize" => self.execute_summarization(context).await,
            "document::verify_cid_chain" => self.execute_cid_verification(context).await,
            _ => Err(WorkflowError::UnsupportedStepType { 
                step_type: step_type.to_string() 
            })
        }
    }
    
    fn validate_context(&self, context: &WorkflowContext) -> WorkflowResult<()> {
        let doc_ext = context.get_domain_extension("document")
            .ok_or(WorkflowError::MissingDomainExtension { 
                domain: "document".to_string() 
            })?;
            
        // Validate required fields
        let required_fields = ["document_id"];
        for field in &required_fields {
            if doc_ext.data.get(field).is_none() {
                return Err(WorkflowError::MissingContextData { 
                    field: field.to_string() 
                });
            }
        }
        
        Ok(())
    }
    
    fn transform_context(
        &self, 
        context: &WorkflowContext, 
        target_domain: &str
    ) -> WorkflowResult<serde_json::Value> {
        let doc_ext = context.get_domain_extension("document")
            .ok_or(WorkflowError::MissingDomainExtension { domain: "document".to_string() })?;
        
        match target_domain {
            "person" => {
                // Transform document context to person context
                Ok(serde_json::json!({
                    "person_id": doc_ext.data.get("author_id"),
                    "notification_type": "document_review_requested",
                    "document_reference": doc_ext.data.get("document_id"),
                    "source_domain": "document"
                }))
            }
            
            "organization" => {
                // Transform document context to organization context
                Ok(serde_json::json!({
                    "organization_id": doc_ext.data.get("department_id"),
                    "approval_request": {
                        "document_id": doc_ext.data.get("document_id"),
                        "approval_level": "department",
                        "requested_by": doc_ext.data.get("author_id"),
                    },
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
            "document::archive".to_string(),
        ]
    }
}

impl DocumentWorkflowExtension {
    async fn execute_create_step(&self, context: &mut WorkflowContext) -> WorkflowResult<StepResult> {
        let doc_ext = context.get_domain_extension("document")
            .ok_or(WorkflowError::MissingDomainExtension { domain: "document".to_string() })?;
        
        let content = doc_ext.data.get("content")
            .ok_or(WorkflowError::MissingContextData { field: "content".to_string() })?;
        let title = doc_ext.data.get("title")
            .ok_or(WorkflowError::MissingContextData { field: "title".to_string() })?;
            
        let document = self.document_service.create_document(
            title.as_str().unwrap(),
            content.as_str().unwrap(),
        ).await?;
        
        // Update context with created document
        context.update_domain_extension(
            "document",
            serde_json::json!({
                "document_id": document.id,
                "status": "draft",
                "created_at": document.created_at,
            })
        )?;
        
        Ok(StepResult::Completed { 
            output: serde_json::to_value(document)? 
        })
    }
    
    async fn execute_review_step(&self, context: &mut WorkflowContext) -> WorkflowResult<StepResult> {
        let doc_id = self.extract_document_id(context)?;
        let reviewer_id = context.get_domain_extension("document")
            .and_then(|ext| ext.data.get("reviewer_id"))
            .ok_or(WorkflowError::MissingContextData { field: "reviewer_id".to_string() })?;
            
        let review = self.review_service.start_review(
            doc_id,
            reviewer_id.as_str().unwrap(),
        ).await?;
        
        context.update_domain_extension(
            "document",
            serde_json::json!({
                "document_id": doc_id,
                "review_id": review.id,
                "status": "under_review",
                "reviewer_id": reviewer_id,
            })
        )?;
        
        Ok(StepResult::InProgress { 
            waiting_for: "reviewer_decision".to_string(),
            timeout_minutes: Some(1440), // 24 hours
        })
    }
    
    fn extract_document_id(&self, context: &WorkflowContext) -> WorkflowResult<&str> {
        context.get_domain_extension("document")
            .and_then(|ext| ext.data.get("document_id"))
            .and_then(|v| v.as_str())
            .ok_or(WorkflowError::MissingContextData { 
                field: "document_id".to_string() 
            })
    }
}
```

### Person Domain Extension

```rust
pub struct PersonWorkflowExtension {
    person_service: Arc<dyn PersonService>,
    identity_service: Arc<dyn IdentityVerificationService>,
    notification_service: Arc<dyn NotificationService>,
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
            "person::create" => self.execute_create_step(context).await,
            "person::verify_identity" => self.execute_identity_verification(context).await,
            "person::send_notification" => self.execute_notification(context).await,
            "person::assign_role" => self.execute_role_assignment(context).await,
            _ => Err(WorkflowError::UnsupportedStepType { 
                step_type: step_type.to_string() 
            })
        }
    }
    
    fn transform_context(
        &self, 
        context: &WorkflowContext, 
        target_domain: &str
    ) -> WorkflowResult<serde_json::Value> {
        let person_ext = context.get_domain_extension("person")
            .ok_or(WorkflowError::MissingDomainExtension { domain: "person".to_string() })?;
        
        match target_domain {
            "document" => {
                // Transform person context to document context
                Ok(serde_json::json!({
                    "author_id": person_ext.data.get("person_id"),
                    "author_name": person_ext.data.get("name"),
                    "author_department": person_ext.data.get("department_id"),
                    "source_domain": "person"
                }))
            }
            
            "organization" => {
                // Transform person context to organization context
                Ok(serde_json::json!({
                    "member_id": person_ext.data.get("person_id"),
                    "organization_id": person_ext.data.get("organization_id"),
                    "role": person_ext.data.get("role"),
                    "source_domain": "person"
                }))
            }
            
            _ => Err(WorkflowError::UnsupportedTransformation { 
                from: "person".to_string(),
                to: target_domain.to_string()
            })
        }
    }
    
    fn supported_step_types(&self) -> Vec<String> {
        vec![
            "person::create".to_string(),
            "person::verify_identity".to_string(),
            "person::send_notification".to_string(),
            "person::assign_role".to_string(),
            "person::update_profile".to_string(),
        ]
    }
}
```

## Cross-Domain Composition Example

```rust
// Example: Document Review → Person Notification → Organization Approval
pub async fn execute_cross_domain_approval(
    engine: &WorkflowEngine,
    document_id: &str,
    author_id: &str,
    department_id: &str,
) -> WorkflowResult<WorkflowInstanceId> {
    // Initialize context with document domain data
    let mut context = WorkflowContext::new();
    context.add_domain_extension(
        "document".to_string(),
        serde_json::json!({
            "document_id": document_id,
            "author_id": author_id,
            "department_id": department_id,
            "status": "pending_review"
        }),
        "1.0".to_string(),
    );
    
    // Start workflow using cross-domain template
    let instance_id = engine.start_workflow_from_template(
        "cross_domain_approval",
        context,
    ).await?;
    
    Ok(instance_id)
}

// Template defines the composition of domain operations
pub fn cross_domain_approval_template() -> WorkflowTemplate {
    WorkflowTemplate {
        id: "cross_domain_approval".to_string(),
        name: "Cross-Domain Document Approval".to_string(),
        applicable_domains: vec!["document".to_string(), "person".to_string(), "organization".to_string()],
        nodes: vec![
            TemplateNode {
                id: NodeId::new("document", "review"),
                step_type: "document::review".to_string(),
                required_context: vec!["document_id".to_string(), "reviewer_id".to_string()],
                timeout_minutes: Some(1440),
            },
            TemplateNode {
                id: NodeId::new("person", "notify"),
                step_type: "person::send_notification".to_string(),
                required_context: vec!["person_id".to_string(), "notification_type".to_string()],
                timeout_minutes: Some(5),
            },
            TemplateNode {
                id: NodeId::new("organization", "approve"),
                step_type: "organization::approve".to_string(), 
                required_context: vec!["organization_id".to_string(), "approval_level".to_string()],
                timeout_minutes: Some(720),
            },
        ],
        edges: vec![
            TemplateEdge { 
                from: NodeId::new("document", "review"), 
                to: NodeId::new("person", "notify") 
            },
            TemplateEdge { 
                from: NodeId::new("person", "notify"), 
                to: NodeId::new("organization", "approve") 
            },
        ],
        required_extensions: vec!["document".to_string(), "person".to_string(), "organization".to_string()],
    }
}
```

## Benefits of Domain Composition

### 1. Loose Coupling
Each domain extension is independent and can evolve separately:
```rust
// Document domain can change without affecting person domain
impl DocumentWorkflowExtension {
    // New document operations don't break other domains
    async fn execute_ai_analysis(&self, context: &mut WorkflowContext) -> WorkflowResult<StepResult> { ... }
}
```

### 2. Reusable Components
Domain extensions can be reused across different workflow engines:
```rust
// Same extension can be used in different contexts
let document_extension = Arc::new(DocumentWorkflowExtension::new(...));
workflow_engine_1.register_extension(document_extension.clone());
workflow_engine_2.register_extension(document_extension.clone());
```

### 3. Cross-Domain Operations
Functorial transformations enable seamless domain transitions:
```rust
// Transform document context to person context automatically
let person_context = document_extension.transform_context(&context, "person")?;
```

### 4. Mathematical Correctness
Category Theory foundation ensures compositional correctness:
- **Identity**: Each domain has identity morphisms (no-op operations)
- **Composition**: Domain operations compose associatively
- **Functorial**: Cross-domain transformations preserve structure

This pattern enables the universal workflow engine to serve all CIM domains while maintaining mathematical rigor and clean architectural boundaries.