# CIM-Domain-Workflow: Consolidated Abstract Workflow Domain Design

## Executive Summary

This document outlines the transformation of cim-domain-workflow from a single-domain workflow system into a **consolidated abstract Workflow Domain** that serves all CIM domains through composition and events. The design follows Category Theory principles and maintains CIM compliance with mandatory correlation/causation IDs.

## Current State Analysis

### Workflow Implementations Found Across CIM Domains

1. **cim-domain-workflow** - Full DDD aggregate with event sourcing, state machines, step management
2. **cim-domain-document** - Document-specific workflows with CID integrity, content intelligence integration  
3. **cim-domain-person** - Person onboarding, identity verification workflows
4. **cim-domain-organization** - Organizational hierarchy and role-based workflows
5. **cim-domain-location** - Geographic validation and territory management workflows

### Key Pattern Inconsistencies Identified

- **ID Type Duplication**: Each domain redefines `WorkflowId`, `NodeId`, `StepId`
- **State Machine Variations**: Different state machine implementations across domains  
- **Event Schema Divergence**: Inconsistent event structures and naming
- **Context Object Differences**: Domain-specific context implementations
- **Integration Complexity**: No standard way for workflows to span multiple domains

## Consolidated Architecture Design

### Core Principle: Abstract Workflow Domain

Transform workflows from domain-specific implementations into a **Category** where:
- **Objects** = Workflow instances across all domains
- **Morphisms** = State transitions and cross-domain interactions  
- **Composition** = Chaining workflows across domain boundaries
- **Identity** = Domain-specific workflow extensions via composition

### Domain Structure

```
cim-domain-workflow/
├── src/
│   ├── core/                    # Domain-agnostic workflow engine
│   │   ├── mod.rs
│   │   ├── engine.rs           # Abstract workflow execution engine
│   │   ├── state_machine.rs    # Unified state machine framework
│   │   └── coordinator.rs      # Cross-domain workflow coordination
│   ├── primitives/             # Shared workflow building blocks
│   │   ├── mod.rs
│   │   ├── identifiers.rs      # Unified ID types (WorkflowId, StepId, etc.)
│   │   ├── states.rs           # Common state definitions
│   │   ├── transitions.rs      # Standard transition types
│   │   └── context.rs          # Extensible context framework
│   ├── events/                 # CIM-compliant workflow events
│   │   ├── mod.rs
│   │   ├── lifecycle.rs        # Workflow lifecycle events
│   │   ├── execution.rs        # Step/task execution events  
│   │   └── integration.rs      # Cross-domain integration events
│   ├── composition/            # Domain composition patterns
│   │   ├── mod.rs
│   │   ├── extensions.rs       # Domain-specific extension points
│   │   ├── templates.rs        # Reusable workflow templates
│   │   └── adapters.rs         # Domain service integration
│   ├── messaging/              # Event-driven communication
│   │   ├── mod.rs
│   │   ├── publishers.rs       # Event publishing infrastructure
│   │   ├── subscribers.rs      # Cross-domain event handlers
│   │   └── correlation.rs      # Event correlation and causation
│   └── lib.rs                  # Public API exports
```

## Key Design Components

### 1. Unified Identifier System

**Problem**: Each domain defines its own `WorkflowId`, `StepId`, etc., causing type conflicts and preventing cross-domain workflows.

**Solution**: Universal identifier types with domain-aware construction:

```rust
/// Universal workflow identifier - replaces domain-specific WorkflowId types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn new_deterministic(name: &str, domain: &str) -> Self {
        // Create deterministic ID from domain + name for consistent workflows
        let namespace = format!("{}::{}", domain, name);
        Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, namespace.as_bytes()))
    }
}

/// Universal node identifier for workflow graphs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(domain: &str, node_name: &str) -> Self {
        Self(format!("{}::{}", domain, node_name))
    }
}
```

### 2. Extensible Context Framework

**Problem**: Each domain needs different context data, but workflows must coordinate across domains.

**Solution**: Core context with domain-specific extensions:

```rust
/// Core workflow context with extension points for domain-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Universal variables accessible to all workflows
    pub variables: HashMap<String, Value>,
    
    /// Domain-specific extensions
    pub extensions: HashMap<String, DomainExtension>,
    
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Domain-specific context extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExtension {
    pub domain: String,
    pub data: Value,
    pub version: String,
}

impl WorkflowContext {
    /// Create context with domain-specific extension
    pub fn with_domain_extension(domain: &str, data: Value) -> Self {
        let mut context = Self::new();
        context.add_domain_extension(domain.to_string(), data, "1.0".to_string());
        context
    }
    
    /// Get domain-specific data
    pub fn get_domain_extension(&self, domain: &str) -> Option<&DomainExtension> {
        self.extensions.get(domain)
    }
}
```

### 3. Domain Extension Pattern

**Problem**: Domains need custom workflow logic without duplicating core workflow infrastructure.

**Solution**: Composition-based extension system:

```rust
/// Domain-specific workflow extension trait
#[async_trait]
pub trait DomainWorkflowExtension: Send + Sync {
    /// Domain identifier
    fn domain(&self) -> &'static str;
    
    /// Execute domain-specific workflow logic  
    async fn execute_domain_step(
        &self,
        step_type: &str,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<StepResult>;
    
    /// Validate domain-specific context
    fn validate_context(&self, context: &WorkflowContext) -> WorkflowResult<()>;
    
    /// Transform context for cross-domain operations
    fn transform_context(&self, context: &WorkflowContext, target_domain: &str) -> WorkflowResult<Value>;
}

/// Document domain extension example
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
            "document::review" => self.execute_review_step(context).await,
            "document::approve" => self.execute_approval_step(context).await,
            "document::extract_entities" => self.execute_entity_extraction(context).await,
            "document::verify_cid_chain" => self.execute_cid_verification(context).await,
            _ => Err(WorkflowError::UnsupportedStepType { step_type: step_type.to_string() })
        }
    }
}
```

### 4. Template System

**Problem**: Similar workflow patterns are reimplemented across domains.

**Solution**: Reusable workflow templates:

```rust
/// Cross-domain workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub applicable_domains: Vec<String>,
    pub nodes: Vec<TemplateNode>,
    pub edges: Vec<TemplateEdge>,
    pub required_extensions: Vec<String>,
}

impl WorkflowTemplate {
    /// Approval workflow template - works across all domains
    pub fn approval_workflow() -> Self {
        Self {
            id: "approval_v1".to_string(),
            name: "Generic Approval Workflow".to_string(),
            applicable_domains: vec!["document", "person", "organization"],
            nodes: vec![
                TemplateNode {
                    id: NodeId::new("core", "start"),
                    step_type: "core::start".to_string(),
                    required_context: vec!["entity_id".to_string()],
                },
                TemplateNode {
                    id: NodeId::new("core", "review"),
                    step_type: "core::manual".to_string(),
                    required_context: vec!["reviewer".to_string()],
                    timeout_minutes: Some(1440), // 24 hours
                },
                TemplateNode {
                    id: NodeId::new("core", "approve"),
                    step_type: "core::approval".to_string(),
                    required_context: vec!["approver".to_string()],
                    timeout_minutes: Some(720), // 12 hours
                },
            ],
            // ... edges, etc.
        }
    }
}
```

### 5. CIM-Compliant Event System

**Problem**: Inconsistent event schemas across domains, missing correlation/causation tracking.

**Solution**: Universal event system with mandatory CIM compliance:

```rust
/// Universal workflow event with CIM compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimWorkflowEvent {
    /// CIM message identity with mandatory correlation/causation  
    pub identity: MessageIdentity,
    
    /// Workflow instance this event applies to
    pub instance_id: WorkflowInstanceId,
    
    /// Domain that triggered the event
    pub source_domain: String,
    
    /// The specific event
    pub event: WorkflowEventType,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// CID for event integrity verification
    pub event_cid: Option<Cid>,
}

/// All workflow event types across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEventType {
    // Core workflow lifecycle
    WorkflowStarted {
        workflow_id: WorkflowId,
        context: WorkflowContext,
        started_by: Option<String>,
    },
    WorkflowCompleted {
        workflow_id: WorkflowId,  
        final_context: WorkflowContext,
        duration_seconds: u64,
    },
    
    // Cross-domain events
    CrossDomainTransition {
        from_domain: String,
        to_domain: String,
        transition_data: serde_json::Value,
    },
    
    // Domain-specific events (extensible)
    DomainSpecific {
        domain: String,
        event_type: String,
        data: serde_json::Value,
    },
}
```

### 6. Cross-Domain Event Correlation

**Problem**: No way to track workflows that span multiple domains.

**Solution**: Event correlation system:

```rust
/// Tracks workflow event correlation across domain boundaries
pub struct WorkflowEventCorrelator {
    /// Active workflow correlations
    active_correlations: HashMap<CorrelationId, WorkflowCorrelationChain>,
    
    /// Cross-domain workflow instances
    cross_domain_instances: HashMap<WorkflowInstanceId, Vec<String>>,
}

/// Chain of correlated workflow events across domains
#[derive(Debug, Clone)]
pub struct WorkflowCorrelationChain {
    pub correlation_id: CorrelationId,
    pub root_event: CimWorkflowEvent,
    pub events: Vec<CimWorkflowEvent>,
    pub active_domains: Vec<String>,
    pub completed_domains: Vec<String>,
}
```

## Migration Strategy

### Phase 1: Foundation (Week 1-2)
**Goal**: Create shared workflow core without breaking existing domains

1. **Extract Common Primitives**
   - Move shared types from existing cim-domain-workflow to new core module
   - Create compatibility re-exports for existing code
   - Add cim-workflow-core dependency to all domain crates

2. **Compatibility Layer**
   ```rust
   // In each domain crate, create compatibility re-exports
   pub use cim_workflow_core::{WorkflowId, StepId, WorkflowContext};
   
   // Gradual migration alias
   pub type DomainWorkflowId = cim_workflow_core::WorkflowId;
   ```

### Phase 2: Domain Extensions (Week 3-4)  
**Goal**: Implement domain-specific extensions using shared core

1. **Create Extension Implementations**
   - Document domain: DocumentWorkflowExtension
   - Person domain: PersonWorkflowExtension
   - Organization domain: OrganizationWorkflowExtension
   - Location domain: LocationWorkflowExtension

2. **Update Domain Workflow Services**
   - Replace domain-specific workflow engines with core engine + extensions
   - Migrate domain-specific context to extension pattern

### Phase 3: Event Integration (Week 5-6)
**Goal**: Migrate to unified event system with backward compatibility

1. **Event System Migration**
   - Replace domain-specific events with CimWorkflowEvent
   - Implement event correlation for cross-domain workflows
   - Add CID integrity to all workflow events

2. **NATS Subject Updates**
   - Standardize subject patterns: `events.workflow.{domain}.{event_type}.{instance_id}`
   - Update all domain subscribers to new patterns

### Phase 4: Template System (Week 7-8)
**Goal**: Implement reusable workflow templates

1. **Template Implementation**
   - Create standard templates (approval, notification, integration)
   - Implement template instantiation system
   - Add template validation and testing

2. **Domain Template Specialization**
   - Document approval workflows
   - Person onboarding workflows  
   - Cross-domain integration workflows

## Mathematical Foundation (Category Theory)

This design implements workflows as a proper **Category** in the mathematical sense:

### Objects
- **Workflow Instances**: Each workflow instance is an Object in the category
- **Domain Extensions**: Specialized objects within domain subcategories
- **Templates**: Morphism generators that create workflow objects

### Morphisms  
- **State Transitions**: Morphisms between workflow states
- **Cross-Domain Interactions**: Functors between domain categories
- **Event Causation**: Morphism composition via causation chains

### Composition
- **Sequential Workflows**: Morphism composition within single domains
- **Cross-Domain Workflows**: Functor composition across domain boundaries
- **Template Instantiation**: Natural transformations from templates to instances

### Identity
- **Domain Identity**: Each domain has identity morphisms (workflows that don't change state)
- **Extension Identity**: Domain extensions preserve core workflow properties
- **Event Identity**: CIM message identity ensures event traceability

## Benefits

### Technical Benefits
1. **Single Source of Truth**: One workflow engine for all domains
2. **Type Safety**: Shared types prevent ID mismatches across domains
3. **Consistent Event Schema**: Unified event structure across domains  
4. **Cross-Domain Coordination**: Native support for multi-domain workflows
5. **Backward Compatibility**: Gradual migration without breaking existing code
6. **Observability**: Unified monitoring and tracing across all workflows

### Business Benefits
1. **Reduced Development Time**: Reusable workflow patterns across domains
2. **Consistent User Experience**: Similar workflow behavior across all domains
3. **Enhanced Integration**: Natural support for business processes spanning multiple domains
4. **Improved Reliability**: Centralized workflow engine with better testing and monitoring
5. **Audit Compliance**: CID-based event integrity provides cryptographic audit trails

### Mathematical Benefits (Category Theory)
1. **Compositional**: Workflows compose predictably across domain boundaries
2. **Structure-Preserving**: Domain boundaries preserved through functorial mapping
3. **Verifiable**: Mathematical properties enable formal verification of workflow properties
4. **Extensible**: New domains can be added as new categories in the system

## Implementation Priority

1. **High Priority**: Core primitives, domain extensions, event system
2. **Medium Priority**: Template system, cross-domain correlation
3. **Low Priority**: Advanced analytics, workflow visualization, performance optimization

## Success Metrics

1. **Technical Metrics**
   - All domains using shared workflow primitives
   - Zero workflow-related type conflicts across domains
   - All workflow events following CIM correlation standards
   - Cross-domain workflows executing successfully

2. **Performance Metrics**
   - Workflow execution time < 100ms (95th percentile) 
   - Memory usage increase < 20% vs current implementations
   - Support for 1000+ concurrent workflow instances

3. **Business Metrics**
   - 50% reduction in workflow development time for new domains
   - 100% audit compliance with cryptographic event verification
   - Zero workflow-related production incidents

This consolidated design transforms workflows from fragmented domain-specific implementations into a mathematically rigorous, practically useful workflow system that serves as the foundation for all business process automation across the entire CIM ecosystem.