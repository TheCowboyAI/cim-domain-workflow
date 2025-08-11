# CIM Workflow Templates Context

This document describes the standard workflow templates that serve as building blocks for CIM domain operations.

## Template System Overview

Workflow templates define reusable patterns that can be instantiated across different contexts while maintaining CIM compliance and mathematical consistency.

### Template Structure
```rust
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub applicable_domains: Vec<String>,
    pub nodes: Vec<TemplateNode>,
    pub edges: Vec<TemplateEdge>,
    pub required_extensions: Vec<String>,
    pub timeout_strategy: TimeoutStrategy,
    pub rollback_strategy: RollbackStrategy,
}
```

## Core Template Categories

### 1. Single Domain Templates

#### Document Lifecycle Templates
- **document_review**: Standard document review workflow
- **document_approval**: Multi-stage document approval
- **document_publishing**: Content publishing pipeline
- **document_archival**: Document retention and archival

#### Person Management Templates  
- **identity_verification**: Identity verification workflow
- **role_assignment**: Role and permission assignment
- **notification_delivery**: Multi-channel notification delivery
- **profile_management**: Person profile lifecycle

#### Organization Operations Templates
- **department_approval**: Department-level approval processes  
- **hierarchy_update**: Organizational structure changes
- **resource_allocation**: Resource assignment workflows
- **compliance_audit**: Organizational compliance checking

### 2. Cross-Domain Templates

#### Document Approval Cross-Domain
```rust
pub fn document_approval_cross_domain_template() -> WorkflowTemplate {
    WorkflowTemplate {
        id: "document_approval_cross_domain".to_string(),
        name: "Cross-Domain Document Approval".to_string(),
        description: "Document review → Author notification → Department approval".to_string(),
        applicable_domains: vec!["document".to_string(), "person".to_string(), "organization".to_string()],
        nodes: vec![
            // Document domain: Submit for review
            TemplateNode {
                id: NodeId::new("document", "submit_for_review"),
                name: "Submit for Review".to_string(),
                step_type: "document::submit_for_review".to_string(),
                required_context: vec!["document_id".to_string(), "author_id".to_string()],
                timeout_minutes: Some(5),
            },
            // Person domain: Notify reviewer
            TemplateNode {
                id: NodeId::new("person", "notify_reviewer"),
                name: "Notify Reviewer".to_string(),
                step_type: "person::send_notification".to_string(),
                required_context: vec!["person_id".to_string(), "notification_type".to_string()],
                timeout_minutes: Some(2),
            },
            // Document domain: Conduct review
            TemplateNode {
                id: NodeId::new("document", "conduct_review"),
                name: "Conduct Review".to_string(),
                step_type: "document::review".to_string(),
                required_context: vec!["document_id".to_string(), "reviewer_id".to_string()],
                timeout_minutes: Some(1440), // 24 hours
            },
            // Person domain: Notify author
            TemplateNode {
                id: NodeId::new("person", "notify_author"),
                name: "Notify Author of Review Results".to_string(),
                step_type: "person::send_notification".to_string(),
                required_context: vec!["person_id".to_string(), "notification_type".to_string()],
                timeout_minutes: Some(2),
            },
            // Organization domain: Department approval
            TemplateNode {
                id: NodeId::new("organization", "department_approval"),
                name: "Department Approval".to_string(),
                step_type: "organization::approve".to_string(),
                required_context: vec!["organization_id".to_string(), "approval_level".to_string()],
                timeout_minutes: Some(720), // 12 hours
            },
            // Document domain: Publish
            TemplateNode {
                id: NodeId::new("document", "publish"),
                name: "Publish Document".to_string(),
                step_type: "document::publish".to_string(),
                required_context: vec!["document_id".to_string()],
                timeout_minutes: Some(10),
            },
        ],
        edges: vec![
            TemplateEdge { from: NodeId::new("document", "submit_for_review"), to: NodeId::new("person", "notify_reviewer") },
            TemplateEdge { from: NodeId::new("person", "notify_reviewer"), to: NodeId::new("document", "conduct_review") },
            TemplateEdge { from: NodeId::new("document", "conduct_review"), to: NodeId::new("person", "notify_author") },
            TemplateEdge { from: NodeId::new("person", "notify_author"), to: NodeId::new("organization", "department_approval") },
            TemplateEdge { from: NodeId::new("organization", "department_approval"), to: NodeId::new("document", "publish") },
        ],
        required_extensions: vec!["document".to_string(), "person".to_string(), "organization".to_string()],
        timeout_strategy: TimeoutStrategy::Cascade,
        rollback_strategy: RollbackStrategy::CompensatingTransactions,
    }
}
```

#### Employee Onboarding Cross-Domain
- **Person Creation**: Create employee profile
- **Organization Assignment**: Assign to department
- **Document Generation**: Generate employment contract
- **Welcome Package**: Send onboarding materials

#### Incident Response Cross-Domain  
- **Person Notification**: Alert response team
- **Location Assessment**: Evaluate incident location
- **Organization Escalation**: Escalate through hierarchy
- **Document Creation**: Generate incident reports

### 3. Integration Templates

#### External System Integration
- **api_integration**: Standard API integration pattern
- **webhook_processing**: Webhook event processing
- **batch_sync**: Batch data synchronization
- **real_time_sync**: Real-time data synchronization

## Template Configuration

### Context Requirements
Each template specifies required context extensions and variables:

```rust
pub struct TemplateContextRequirements {
    pub required_extensions: Vec<String>,
    pub required_variables: Vec<String>,
    pub optional_variables: Vec<String>,
    pub validation_rules: Vec<ContextValidationRule>,
}
```

### Timeout Strategies
- **Fixed**: Fixed timeout for each step
- **Dynamic**: Context-based timeout calculation
- **Cascade**: Timeout cascades through dependent steps
- **Progressive**: Increasing timeouts for retry attempts

### Rollback Strategies
- **Saga**: Explicit rollback steps with compensation
- **CompensatingTransactions**: Each step provides compensation
- **BestEffort**: Attempt rollback without guarantees
- **None**: Forward-only execution

## Template Instantiation

### Simple Instantiation
```rust
let instance_id = workflow_engine.start_workflow_from_template(
    "document_review",
    context,
).await?;
```

### Cross-Domain Instantiation
```rust
let instance_id = workflow_engine.start_cross_domain_workflow(
    "document_approval_cross_domain",
    "document",
    vec!["person".to_string(), "organization".to_string()],
    context,
).await?;
```

### Custom Template Creation
```rust
let custom_template = WorkflowTemplate {
    id: "custom_approval".to_string(),
    name: "Custom Approval Process".to_string(),
    // ... template definition
};

workflow_engine.register_template(custom_template).await?;
```

## Template Composition

Templates can be composed to create more complex workflows:

### Sequential Composition
```rust
// Template A → Template B → Template C
let composite_template = WorkflowTemplate::compose_sequential(vec![
    template_a,
    template_b, 
    template_c,
])?;
```

### Parallel Composition
```rust
// Template A || Template B (both execute in parallel)
let parallel_template = WorkflowTemplate::compose_parallel(vec![
    template_a,
    template_b,
])?;
```

### Conditional Composition
```rust
// Template A → (condition ? Template B : Template C)
let conditional_template = WorkflowTemplate::compose_conditional(
    template_a,
    condition_expression,
    template_b,
    template_c,
)?;
```

## Template Validation

### Structural Validation
- Node connectivity validation
- Required context availability
- Domain extension compatibility
- Timeout consistency

### CIM Compliance Validation
- Correlation ID propagation
- Event subject pattern compliance
- CID integrity requirements
- Context extension standards

### Domain Compatibility Validation
- Domain extension requirements
- Cross-domain transformation availability
- Step type support verification
- Context data compatibility

## Template Metrics

### Execution Metrics
- Success/failure rates
- Average execution time
- Timeout occurrence frequency
- Rollback invocation frequency

### Performance Metrics
- Node execution duration
- Cross-domain transition time
- Context transformation overhead
- Event correlation latency

### Business Metrics
- Template usage frequency
- Domain coverage
- Cross-domain workflow adoption
- Template composition patterns

## Template Evolution

### Version Management
- Template versioning strategy
- Backward compatibility maintenance
- Migration path definition
- Deprecation timeline management

### Template Optimization
- Performance profiling
- Bottleneck identification
- Context optimization
- Domain transition optimization

### Template Testing
- Unit test coverage
- Integration test scenarios
- Load testing protocols
- Cross-domain test validation

This template system provides the foundation for consistent, reusable, and CIM-compliant workflow patterns across all domains in the CIM ecosystem.