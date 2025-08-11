# Cross-Domain Workflow Pattern

## Overview

The Cross-Domain Workflow pattern enables business processes to span multiple CIM domains seamlessly, using event correlation and functorial transformations to maintain data consistency and process integrity.

## Mathematical Foundation

Cross-domain workflows implement **Functors** between domain **Categories**:

- **Source Category**: Starting domain (e.g., Document)  
- **Target Category**: Destination domain (e.g., Person)
- **Functor**: Structure-preserving transformation between domains
- **Natural Transformation**: Systematic way to transform contexts

```
Document Category    →    Person Category
F(Objects)          →    G(Objects)  
F(Morphisms)        →    G(Morphisms)
```

## Problem

Traditional approaches handle cross-domain operations poorly:

1. **Tight Coupling**: Direct service calls between domains
2. **Data Inconsistency**: No atomic transactions across domains
3. **Poor Observability**: Lost context in cross-domain calls
4. **Manual Coordination**: Complex orchestration code

```rust
// BAD: Direct cross-domain coupling
impl DocumentService {
    async fn approve_document(&self, doc_id: &str) -> Result<(), Error> {
        // Direct call to person domain - tight coupling
        self.person_service.notify_author(doc_id).await?;
        
        // Direct call to organization domain - tight coupling  
        self.org_service.record_approval(doc_id).await?;
        
        // No correlation tracking, no atomic transactions
        self.update_document_status(doc_id, "approved").await?;
    }
}
```

## Solution: Cross-Domain Workflow Orchestration

Use workflow engine to coordinate cross-domain operations with:
1. **Event Correlation**: Track operations across domain boundaries
2. **Context Transformation**: Safely transform data between domains
3. **Atomic Execution**: Ensure consistency across domains
4. **Observability**: Full visibility into cross-domain processes

## Implementation Pattern

### 1. Cross-Domain Event Correlation

```rust
pub struct WorkflowEventCorrelator {
    /// Active cross-domain workflows by correlation ID
    active_correlations: HashMap<CorrelationId, WorkflowCorrelationChain>,
    
    /// Domain participation tracking
    domain_participation: HashMap<WorkflowInstanceId, Vec<String>>,
    
    /// Event store for replay and debugging
    event_store: Arc<dyn EventStore>,
}

#[derive(Debug, Clone)]
pub struct WorkflowCorrelationChain {
    pub correlation_id: CorrelationId,
    pub instance_id: WorkflowInstanceId,
    pub root_event: CimWorkflowEvent,
    pub events: Vec<CimWorkflowEvent>,
    pub active_domains: HashSet<String>,
    pub completed_domains: HashSet<String>,
    pub started_at: DateTime<Utc>,
    pub timeout: Option<DateTime<Utc>>,
}

impl WorkflowEventCorrelator {
    /// Start a cross-domain workflow with correlation tracking
    pub async fn start_cross_domain_workflow(
        &mut self,
        initiating_event: CimWorkflowEvent,
        participating_domains: Vec<String>,
        timeout: Option<Duration>,
    ) -> WorkflowResult<CorrelationId> {
        let correlation_id = initiating_event.identity.correlation_id.clone();
        let instance_id = initiating_event.instance_id;
        
        // Create correlation chain
        let timeout_at = timeout.map(|d| Utc::now() + d);
        let chain = WorkflowCorrelationChain {
            correlation_id: correlation_id.clone(),
            instance_id,
            root_event: initiating_event.clone(),
            events: vec![initiating_event],
            active_domains: participating_domains.iter().cloned().collect(),
            completed_domains: HashSet::new(),
            started_at: Utc::now(),
            timeout: timeout_at,
        };
        
        self.active_correlations.insert(correlation_id.clone(), chain);
        self.domain_participation.insert(instance_id, participating_domains);
        
        // Emit cross-domain workflow started event
        self.emit_cross_domain_event(
            correlation_id.clone(),
            CrossDomainEventType::WorkflowStarted {
                participating_domains: self.domain_participation.get(&instance_id).unwrap().clone(),
            },
        ).await?;
        
        Ok(correlation_id)
    }
    
    /// Add correlated event from participating domain
    pub async fn add_correlated_event(
        &mut self,
        event: CimWorkflowEvent,
    ) -> WorkflowResult<Option<CrossDomainCompletionResult>> {
        let correlation_id = &event.identity.correlation_id;
        
        if let Some(chain) = self.active_correlations.get_mut(correlation_id) {
            // Add event to chain
            chain.events.push(event.clone());
            
            // Check if domain completed its work
            if let WorkflowEventType::WorkflowCompleted { .. } = &event.event {
                chain.completed_domains.insert(event.source_domain.clone());
                chain.active_domains.remove(&event.source_domain);
                
                // Emit domain completion event
                self.emit_cross_domain_event(
                    correlation_id.clone(),
                    CrossDomainEventType::DomainCompleted {
                        domain: event.source_domain.clone(),
                        remaining_domains: chain.active_domains.iter().cloned().collect(),
                    },
                ).await?;
            }
            
            // Check if all domains completed
            if chain.active_domains.is_empty() {
                return self.complete_cross_domain_workflow(correlation_id.clone()).await;
            }
            
            // Check for timeout
            if let Some(timeout) = chain.timeout {
                if Utc::now() > timeout {
                    return self.timeout_cross_domain_workflow(correlation_id.clone()).await;
                }
            }
        }
        
        Ok(None)
    }
    
    async fn complete_cross_domain_workflow(
        &mut self,
        correlation_id: CorrelationId,
    ) -> WorkflowResult<Option<CrossDomainCompletionResult>> {
        if let Some(chain) = self.active_correlations.remove(&correlation_id) {
            let completion_result = CrossDomainCompletionResult {
                correlation_id: correlation_id.clone(),
                instance_id: chain.instance_id,
                completed_domains: chain.completed_domains.clone(),
                total_events: chain.events.len(),
                duration: Utc::now() - chain.started_at,
                success: true,
            };
            
            // Emit completion event
            self.emit_cross_domain_event(
                correlation_id,
                CrossDomainEventType::WorkflowCompleted {
                    result: completion_result.clone(),
                },
            ).await?;
            
            // Clean up tracking
            self.domain_participation.remove(&chain.instance_id);
            
            Ok(Some(completion_result))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainCompletionResult {
    pub correlation_id: CorrelationId,
    pub instance_id: WorkflowInstanceId,
    pub completed_domains: HashSet<String>,
    pub total_events: usize,
    pub duration: chrono::Duration,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossDomainEventType {
    WorkflowStarted { participating_domains: Vec<String> },
    DomainCompleted { domain: String, remaining_domains: Vec<String> },
    WorkflowCompleted { result: CrossDomainCompletionResult },
    WorkflowFailed { reason: String, failed_domain: String },
    WorkflowTimedOut { timeout_duration: chrono::Duration },
}
```

### 2. Context Transformation Between Domains

```rust
pub struct CrossDomainContextTransformer {
    transformations: HashMap<(String, String), Box<dyn ContextTransformation>>,
}

#[async_trait]
pub trait ContextTransformation: Send + Sync {
    async fn transform(
        &self,
        source_context: &WorkflowContext,
        target_domain: &str,
    ) -> WorkflowResult<WorkflowContext>;
}

impl CrossDomainContextTransformer {
    pub fn new() -> Self {
        let mut transformer = Self {
            transformations: HashMap::new(),
        };
        
        // Register standard transformations
        transformer.register_transformation(
            "document", "person", 
            Box::new(DocumentToPersonTransformation::new()),
        );
        transformer.register_transformation(
            "person", "organization",
            Box::new(PersonToOrganizationTransformation::new()),
        );
        transformer.register_transformation(
            "document", "organization",
            Box::new(DocumentToOrganizationTransformation::new()),
        );
        
        transformer
    }
    
    pub fn register_transformation(
        &mut self,
        from_domain: &str,
        to_domain: &str,
        transformation: Box<dyn ContextTransformation>,
    ) {
        self.transformations.insert(
            (from_domain.to_string(), to_domain.to_string()),
            transformation,
        );
    }
    
    pub async fn transform_context(
        &self,
        source_context: &WorkflowContext,
        source_domain: &str,
        target_domain: &str,
    ) -> WorkflowResult<WorkflowContext> {
        let key = (source_domain.to_string(), target_domain.to_string());
        
        if let Some(transformation) = self.transformations.get(&key) {
            transformation.transform(source_context, target_domain).await
        } else {
            // Use default transformation if available
            self.default_transform(source_context, source_domain, target_domain)
        }
    }
    
    fn default_transform(
        &self,
        source_context: &WorkflowContext,
        source_domain: &str,
        target_domain: &str,
    ) -> WorkflowResult<WorkflowContext> {
        // Create new context with minimal transformation
        let mut target_context = WorkflowContext::new();
        
        // Copy universal variables
        for (key, value) in &source_context.variables {
            if !key.starts_with("_domain_") {
                target_context.variables.insert(key.clone(), value.clone());
            }
        }
        
        // Add transformation metadata
        target_context.variables.insert(
            "_transformation_source".to_string(),
            serde_json::json!(source_domain),
        );
        target_context.variables.insert(
            "_transformation_target".to_string(),
            serde_json::json!(target_domain),
        );
        
        Ok(target_context)
    }
}

// Example: Document → Person transformation
pub struct DocumentToPersonTransformation;

impl DocumentToPersonTransformation {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl ContextTransformation for DocumentToPersonTransformation {
    async fn transform(
        &self,
        source_context: &WorkflowContext,
        target_domain: &str,
    ) -> WorkflowResult<WorkflowContext> {
        let doc_ext = source_context.get_domain_extension("document")
            .ok_or(WorkflowError::MissingDomainExtension { 
                domain: "document".to_string() 
            })?;
        
        // Extract person-relevant information from document context
        let author_id = doc_ext.data.get("author_id")
            .ok_or(WorkflowError::MissingContextData { 
                field: "author_id".to_string() 
            })?;
        let document_id = doc_ext.data.get("document_id")
            .ok_or(WorkflowError::MissingContextData { 
                field: "document_id".to_string() 
            })?;
        let document_title = doc_ext.data.get("title").cloned()
            .unwrap_or_else(|| serde_json::json!("Untitled Document"));
        
        // Create person context
        let mut person_context = WorkflowContext::new();
        
        // Copy relevant universal variables
        for (key, value) in &source_context.variables {
            if ["workflow_id", "correlation_id", "causation_id"].contains(&key.as_str()) {
                person_context.variables.insert(key.clone(), value.clone());
            }
        }
        
        // Add person domain extension
        person_context.add_domain_extension(
            target_domain.to_string(),
            serde_json::json!({
                "person_id": author_id,
                "notification_type": "document_action_required",
                "notification_data": {
                    "document_id": document_id,
                    "document_title": document_title,
                    "action_required": "review_requested",
                },
                "source_domain": "document",
                "transformation_timestamp": Utc::now(),
            }),
            "1.0".to_string(),
        );
        
        Ok(person_context)
    }
}
```

### 3. Cross-Domain Workflow Templates

```rust
pub struct CrossDomainWorkflowTemplate {
    pub template: WorkflowTemplate,
    pub domain_sequence: Vec<String>,
    pub required_transformations: Vec<(String, String)>,
    pub rollback_strategy: RollbackStrategy,
}

impl CrossDomainWorkflowTemplate {
    /// Create a document approval workflow spanning document, person, and organization domains
    pub fn document_approval_workflow() -> Self {
        let template = WorkflowTemplate {
            id: "document_approval_cross_domain".to_string(),
            name: "Cross-Domain Document Approval".to_string(),
            description: "Document review → Author notification → Department approval".to_string(),
            applicable_domains: vec!["document".to_string(), "person".to_string(), "organization".to_string()],
            nodes: vec![
                TemplateNode {
                    id: NodeId::new("document", "submit_for_review"),
                    name: "Submit for Review".to_string(),
                    step_type: "document::submit_for_review".to_string(),
                    required_context: vec!["document_id".to_string(), "author_id".to_string()],
                    timeout_minutes: Some(5),
                },
                TemplateNode {
                    id: NodeId::new("person", "notify_reviewer"),
                    name: "Notify Reviewer".to_string(),
                    step_type: "person::send_notification".to_string(),
                    required_context: vec!["person_id".to_string(), "notification_type".to_string()],
                    timeout_minutes: Some(2),
                },
                TemplateNode {
                    id: NodeId::new("document", "conduct_review"),
                    name: "Conduct Review".to_string(),
                    step_type: "document::review".to_string(),
                    required_context: vec!["document_id".to_string(), "reviewer_id".to_string()],
                    timeout_minutes: Some(1440), // 24 hours
                },
                TemplateNode {
                    id: NodeId::new("person", "notify_author"),
                    name: "Notify Author".to_string(),
                    step_type: "person::send_notification".to_string(),
                    required_context: vec!["person_id".to_string(), "notification_type".to_string()],
                    timeout_minutes: Some(2),
                },
                TemplateNode {
                    id: NodeId::new("organization", "department_approval"),
                    name: "Department Approval".to_string(),
                    step_type: "organization::approve".to_string(),
                    required_context: vec!["organization_id".to_string(), "approval_level".to_string()],
                    timeout_minutes: Some(720), // 12 hours
                },
                TemplateNode {
                    id: NodeId::new("document", "publish"),
                    name: "Publish Document".to_string(),
                    step_type: "document::publish".to_string(),
                    required_context: vec!["document_id".to_string()],
                    timeout_minutes: Some(10),
                },
            ],
            edges: vec![
                TemplateEdge { 
                    from: NodeId::new("document", "submit_for_review"), 
                    to: NodeId::new("person", "notify_reviewer") 
                },
                TemplateEdge { 
                    from: NodeId::new("person", "notify_reviewer"), 
                    to: NodeId::new("document", "conduct_review") 
                },
                TemplateEdge { 
                    from: NodeId::new("document", "conduct_review"), 
                    to: NodeId::new("person", "notify_author") 
                },
                TemplateEdge { 
                    from: NodeId::new("person", "notify_author"), 
                    to: NodeId::new("organization", "department_approval") 
                },
                TemplateEdge { 
                    from: NodeId::new("organization", "department_approval"), 
                    to: NodeId::new("document", "publish") 
                },
            ],
            required_extensions: vec!["document".to_string(), "person".to_string(), "organization".to_string()],
        };
        
        Self {
            template,
            domain_sequence: vec!["document".to_string(), "person".to_string(), "organization".to_string()],
            required_transformations: vec![
                ("document".to_string(), "person".to_string()),
                ("person".to_string(), "organization".to_string()),
                ("organization".to_string(), "document".to_string()),
            ],
            rollback_strategy: RollbackStrategy::CompensatingTransactions,
        }
    }
    
    /// Employee onboarding workflow spanning person, organization, and document domains
    pub fn employee_onboarding_workflow() -> Self {
        let template = WorkflowTemplate {
            id: "employee_onboarding_cross_domain".to_string(),
            name: "Cross-Domain Employee Onboarding".to_string(),
            description: "Person creation → Organization assignment → Document generation".to_string(),
            applicable_domains: vec!["person".to_string(), "organization".to_string(), "document".to_string()],
            nodes: vec![
                TemplateNode {
                    id: NodeId::new("person", "create_profile"),
                    name: "Create Employee Profile".to_string(),
                    step_type: "person::create".to_string(),
                    required_context: vec!["employee_data".to_string()],
                    timeout_minutes: Some(30),
                },
                TemplateNode {
                    id: NodeId::new("organization", "assign_to_department"),
                    name: "Assign to Department".to_string(),
                    step_type: "organization::assign_member".to_string(),
                    required_context: vec!["person_id".to_string(), "department_id".to_string()],
                    timeout_minutes: Some(15),
                },
                TemplateNode {
                    id: NodeId::new("document", "generate_contract"),
                    name: "Generate Employment Contract".to_string(),
                    step_type: "document::generate_from_template".to_string(),
                    required_context: vec!["template_id".to_string(), "employee_data".to_string()],
                    timeout_minutes: Some(10),
                },
                TemplateNode {
                    id: NodeId::new("person", "send_welcome_package"),
                    name: "Send Welcome Package".to_string(),
                    step_type: "person::send_notification".to_string(),
                    required_context: vec!["person_id".to_string(), "welcome_package".to_string()],
                    timeout_minutes: Some(5),
                },
            ],
            edges: vec![
                TemplateEdge { 
                    from: NodeId::new("person", "create_profile"), 
                    to: NodeId::new("organization", "assign_to_department") 
                },
                TemplateEdge { 
                    from: NodeId::new("organization", "assign_to_department"), 
                    to: NodeId::new("document", "generate_contract") 
                },
                TemplateEdge { 
                    from: NodeId::new("document", "generate_contract"), 
                    to: NodeId::new("person", "send_welcome_package") 
                },
            ],
            required_extensions: vec!["person".to_string(), "organization".to_string(), "document".to_string()],
        };
        
        Self {
            template,
            domain_sequence: vec!["person".to_string(), "organization".to_string(), "document".to_string()],
            required_transformations: vec![
                ("person".to_string(), "organization".to_string()),
                ("organization".to_string(), "document".to_string()),
                ("document".to_string(), "person".to_string()),
            ],
            rollback_strategy: RollbackStrategy::Saga,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackStrategy {
    /// Each domain provides compensating transactions
    CompensatingTransactions,
    /// Saga pattern with explicit rollback steps
    Saga,
    /// Best effort cleanup (no guarantees)
    BestEffort,
    /// No rollback (forward-only)
    None,
}
```

### 4. Cross-Domain Execution Engine

```rust
pub struct CrossDomainWorkflowExecutor {
    workflow_engine: Arc<WorkflowEngine>,
    event_correlator: Arc<Mutex<WorkflowEventCorrelator>>,
    context_transformer: Arc<CrossDomainContextTransformer>,
    nats_client: Arc<NatsClient>,
}

impl CrossDomainWorkflowExecutor {
    pub async fn execute_cross_domain_workflow(
        &self,
        template: &CrossDomainWorkflowTemplate,
        initial_context: WorkflowContext,
    ) -> WorkflowResult<WorkflowInstanceId> {
        // Create workflow instance
        let instance_id = WorkflowInstanceId::new();
        
        // Start event correlation
        let correlation_id = {
            let mut correlator = self.event_correlator.lock().await;
            let root_event = CimWorkflowEvent::new_root(
                instance_id,
                template.domain_sequence[0].clone(),
                WorkflowEventType::WorkflowStarted {
                    workflow_id: WorkflowId::from(instance_id),
                    context: initial_context.clone(),
                    started_by: initial_context.variables.get("started_by")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                },
                Some(ActorId::system("cross-domain-executor")),
            );
            
            correlator.start_cross_domain_workflow(
                root_event,
                template.domain_sequence.clone(),
                Some(Duration::hours(24)), // 24 hour timeout
            ).await?
        };
        
        // Execute workflow template
        self.execute_template(
            &template.template,
            instance_id,
            initial_context,
            correlation_id,
        ).await?;
        
        Ok(instance_id)
    }
    
    async fn execute_template(
        &self,
        template: &WorkflowTemplate,
        instance_id: WorkflowInstanceId,
        mut context: WorkflowContext,
        correlation_id: CorrelationId,
    ) -> WorkflowResult<()> {
        // Execute nodes in dependency order
        let execution_order = self.calculate_execution_order(template)?;
        
        for node_id in execution_order {
            let node = template.nodes.iter()
                .find(|n| n.id == node_id)
                .ok_or(WorkflowError::InvalidWorkflowDefinition { 
                    reason: format!("Node not found: {}", node_id.as_str()) 
                })?;
            
            // Check if we need to transform context for different domain
            let node_domain = self.extract_domain_from_step_type(&node.step_type);
            let current_domain = self.get_current_domain(&context);
            
            if node_domain != current_domain {
                // Transform context for target domain
                context = self.context_transformer.transform_context(
                    &context,
                    &current_domain,
                    &node_domain,
                ).await?;
            }
            
            // Execute the node
            let step_result = self.workflow_engine.execute_step_with_context(
                &node.step_type,
                &mut context,
                Some(correlation_id.clone()),
            ).await?;
            
            // Create and publish cross-domain event
            let event = CimWorkflowEvent::new_correlated(
                instance_id,
                node_domain,
                WorkflowEventType::StepCompleted {
                    step_id: StepId::from_node_id(&node.id),
                    step_type: node.step_type.clone(),
                    output: serde_json::to_value(step_result)?,
                    duration_seconds: 0, // TODO: measure actual duration
                },
                Some(correlation_id.clone()),
                Some(CausationId::new()), // TODO: proper causation tracking
            );
            
            self.publish_cross_domain_event(event).await?;
            
            // Add to correlation chain
            {
                let mut correlator = self.event_correlator.lock().await;
                correlator.add_correlated_event(event).await?;
            }
        }
        
        Ok(())
    }
    
    async fn publish_cross_domain_event(&self, event: CimWorkflowEvent) -> WorkflowResult<()> {
        let subject = format!(
            "events.workflow.cross_domain.{}.{}",
            event.event_type(),
            event.identity.correlation_id.as_str()
        );
        
        let payload = serde_json::to_vec(&event)?;
        self.nats_client.publish(&subject, payload).await?;
        
        Ok(())
    }
}
```

## Usage Examples

### Document Approval Across Domains

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = CrossDomainWorkflowExecutor::new().await?;
    let template = CrossDomainWorkflowTemplate::document_approval_workflow();
    
    // Initialize context with document information
    let mut initial_context = WorkflowContext::new();
    initial_context.add_domain_extension(
        "document".to_string(),
        serde_json::json!({
            "document_id": "doc_12345",
            "title": "Q4 Financial Report", 
            "author_id": "user_alice",
            "department_id": "finance",
            "reviewer_id": "user_bob",
        }),
        "1.0".to_string(),
    );
    
    // Start cross-domain workflow
    let instance_id = executor.execute_cross_domain_workflow(
        &template,
        initial_context,
    ).await?;
    
    println!("Started cross-domain workflow: {}", instance_id.as_uuid());
    
    // Workflow will automatically:
    // 1. Submit document for review (document domain)
    // 2. Notify reviewer (person domain) 
    // 3. Conduct review (document domain)
    // 4. Notify author of review results (person domain)
    // 5. Request department approval (organization domain)
    // 6. Publish approved document (document domain)
    
    Ok(())
}
```

### Employee Onboarding

```rust
async fn onboard_new_employee(
    executor: &CrossDomainWorkflowExecutor,
    employee_data: EmployeeData,
) -> WorkflowResult<WorkflowInstanceId> {
    let template = CrossDomainWorkflowTemplate::employee_onboarding_workflow();
    
    let mut context = WorkflowContext::new();
    context.add_domain_extension(
        "person".to_string(),
        serde_json::to_value(&employee_data)?,
        "1.0".to_string(),
    );
    
    executor.execute_cross_domain_workflow(&template, context).await
}
```

## Benefits

1. **Process Integrity**: Atomic cross-domain operations with correlation tracking
2. **Data Consistency**: Functorial transformations preserve data relationships  
3. **Observability**: Complete visibility into cross-domain workflows
4. **Fault Tolerance**: Built-in rollback and compensation strategies
5. **Scalability**: Asynchronous execution with event-driven coordination
6. **Maintainability**: Clean separation of concerns across domains

This pattern enables complex business processes to span multiple CIM domains while maintaining the mathematical rigor and architectural elegance of the Category Theory foundation.