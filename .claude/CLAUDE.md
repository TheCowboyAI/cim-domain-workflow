# CIM Universal Workflow Domain - AI Assistant Instructions

## CRITICAL: WORKFLOW DOMAIN IDENTITY

You **MUST** read /doc/progress/progress.json, this is your EventStore

When you complete ANY action, write what you did as an event to progress.json and add a relationship to other related actions (a correlation)

You are working with the **UNIVERSAL WORKFLOW DOMAIN** - the central nervous system of the entire CIM ecosystem. This is not just another domain; **EVERYTHING** in CIM flows through workflows:

- **Document processing** → Document workflows
- **Person onboarding** → Person workflows  
- **Organization management** → Organization workflows
- **Location validation** → Location workflows
- **Cross-domain operations** → Integration workflows

## PRIMARY MISSION

Transform cim-domain-workflow from a single-domain implementation into a **consolidated abstract Workflow Domain** that serves ALL CIM domains through:

1. **Universal Workflow Engine** - One engine for all domain workflows
2. **Domain Extensions** - Domain-specific logic via composition, not inheritance
3. **Cross-Domain Coordination** - Native multi-domain workflow support
4. **CIM Compliance** - Mandatory correlation/causation IDs with CID integrity
5. **Template System** - Reusable workflow patterns across domains

## ARCHITECTURAL FOUNDATION: PRIMARY IMPLEMENTATION PATTERN

**WORKFLOW = COLLECTION OF EVENTS + STATEMACHINE TRANSITIONS + GENERIC DOMAIN**

This is the fundamental architecture where every workflow is implemented as:

```rust
pub struct Workflow<T: GenericDomain> {
    /// Collection of events that define the workflow execution
    events: EventCollection<T::Event>,
    
    /// StateMachine that defines valid transitions  
    state_machine: StateMachine<T::State>,
    
    /// Generic domain providing domain-specific operations
    domain: T,
    
    /// Current workflow state
    current_state: T::State,
    
    /// Workflow instance identifier
    instance_id: WorkflowInstanceId,
}

pub trait GenericDomain: Send + Sync {
    type State: Clone + Debug + Send + Sync;
    type Event: WorkflowEvent;
    type Context: WorkflowContext;
    
    fn valid_transitions(&self) -> StateMachine<Self::State>;
    fn handle_event(&self, event: &Self::Event, context: &mut Self::Context) -> WorkflowResult<Vec<Self::Event>>;
    fn initial_state(&self) -> Self::State;
}
```

### Category Theory Foundation

Built on **Category Theory** principles where workflows are **Event-Driven State Machines**:
- **Objects**: Workflow states and domain entities  
- **Morphisms**: StateMachine transitions sequencing events
- **Event Collections**: Ordered sequences representing workflow execution paths
- **Functors**: Structure-preserving transformations between domain workflows  
- **Natural Transformations**: Context transformations maintaining event relationships
- **Composition**: Event sequence composition across domain boundaries

## DESIGN REFERENCES (MANDATORY READING)

1. **[Consolidated Workflow Design](../doc/consolidated-workflow-design.md)** - Complete architectural specification
2. **[Migration Guide](../doc/migration-guide.md)** - Domain migration patterns and examples
3. **[Implementation Roadmap](../doc/implementation-roadmap.md)** - 8-week implementation plan

## CORE PRINCIPLES

### 1. Universal Workflow Abstraction
Every workflow operation must work across ALL domains:
```rust
// BAD: Domain-specific workflow types
DocumentWorkflowId, PersonWorkflowId, OrganizationWorkflowId

// GOOD: Universal workflow types  
WorkflowId, StepId, WorkflowContext (with domain extensions)
```

### 2. Composition Over Inheritance
Domain-specific logic via extensions, not subclassing:
```rust
// BAD: Inheritance-based domain workflows
trait DocumentWorkflow: BaseWorkflow { ... }

// GOOD: Composition-based domain extensions
trait DomainWorkflowExtension {
    fn domain(&self) -> &'static str;
    async fn execute_domain_step(&self, step_type: &str, context: &mut WorkflowContext) -> WorkflowResult<StepResult>;
}
```

### 3. CIM-Compliant Events
ALL workflow events MUST carry mandatory correlation/causation:
```rust
// Every workflow event must include
pub struct CimWorkflowEvent {
    pub identity: MessageIdentity,           // MANDATORY correlation/causation
    pub instance_id: WorkflowInstanceId,
    pub source_domain: String,
    pub event: WorkflowEventType,
    pub event_cid: Option<Cid>,            // Cryptographic integrity
}
```

### 4. Cross-Domain Integration
Workflows must seamlessly span multiple domains:
```rust
// Example: Document review → Person notification → Organization approval
WorkflowTemplate::cross_domain_workflow()
    .with_step("document::review", document_context)
    .with_step("person::notify", person_context)  
    .with_step("organization::approve", org_context)
```

## IMPLEMENTATION RULES

### Code Organization
```
src/
├── core/                    # Domain-agnostic workflow engine
│   ├── engine.rs           # Abstract workflow execution
│   ├── state_machine.rs    # Unified state machine framework
│   └── coordinator.rs      # Cross-domain coordination
├── primitives/             # Shared workflow building blocks
│   ├── identifiers.rs      # Universal IDs (WorkflowId, StepId, etc.)
│   ├── context.rs          # Extensible context framework
│   └── states.rs           # Common state definitions
├── events/                 # CIM-compliant workflow events
│   ├── lifecycle.rs        # Workflow lifecycle events
│   └── integration.rs      # Cross-domain events
├── composition/            # Domain composition patterns
│   ├── extensions.rs       # Domain extension trait system
│   ├── templates.rs        # Reusable workflow templates
│   └── adapters.rs         # Domain service integration
└── messaging/              # Event-driven communication
    ├── correlation.rs      # Cross-domain event correlation
    └── publishers.rs       # NATS event publishing
```

### Naming Conventions
- **Universal Types**: `WorkflowId`, `StepId`, `WorkflowContext` (not domain-prefixed)
- **Domain Extensions**: `DocumentWorkflowExtension`, `PersonWorkflowExtension`
- **Templates**: `ApprovalWorkflow`, `CrossDomainWorkflow`, `NotificationWorkflow`
- **Events**: `CimWorkflowEvent` (always CIM-compliant)

### Testing Requirements
Every component must have:
```rust
// Domain extension tests
#[tokio::test]
async fn test_domain_extension_execution() { ... }

// Cross-domain workflow tests  
#[tokio::test]
async fn test_cross_domain_workflow_execution() { ... }

// Event correlation tests
#[tokio::test] 
async fn test_event_correlation_across_domains() { ... }

// Template instantiation tests
#[tokio::test]
async fn test_template_instantiation() { ... }
```

### Error Handling
Use domain-agnostic error types:
```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Unsupported step type: {step_type}")]
    UnsupportedStepType { step_type: String },
    
    #[error("Missing domain extension: {domain}")]
    MissingDomainExtension { domain: String },
    
    #[error("Cross-domain transition failed: {from} -> {to}")]
    CrossDomainTransitionFailed { from: String, to: String },
}
```

## DEVELOPMENT WORKFLOW

### Phase 1: Core Infrastructure (Current Priority)
1. **Implement Universal Primitives** (`src/primitives/`)
   - Unified identifier system
   - Extensible context framework
   - Common state definitions

2. **Build Core Engine** (`src/core/`)  
   - Abstract workflow execution engine
   - Domain extension registration
   - State machine framework

3. **Create Event System** (`src/events/`)
   - CIM-compliant event structures
   - Cross-domain event correlation
   - CID integrity verification

### Phase 2: Domain Integration
1. **Document Domain Extension** - First migration target
2. **Person Domain Extension** - Second migration target
3. **Cross-Domain Templates** - Approval, notification, integration workflows

### Phase 3: Production Readiness
1. **Performance Optimization** - <100ms workflow execution
2. **Monitoring & Observability** - Full workflow visibility
3. **Migration Support** - Tools and guides for remaining domains

## CRITICAL SUCCESS FACTORS

### 1. Backward Compatibility
During migration, existing domain workflows MUST continue working:
```rust
// Compatibility layer
pub use cim_workflow_core::{WorkflowId as CoreWorkflowId};
pub type DomainWorkflowId = CoreWorkflowId; // Temporary alias
```

### 2. Performance Requirements
- **Workflow Execution**: <100ms (95th percentile)
- **Memory Usage**: <20% increase vs current system
- **Concurrent Instances**: Support 1000+ workflows
- **Cross-Domain Latency**: <10ms for domain transitions

### 3. Event Integrity
ALL workflow events must:
- Carry mandatory correlation/causation IDs
- Include CID for cryptographic verification
- Follow standardized NATS subjects: `events.workflow.{domain}.{event_type}.{instance_id}`

### 4. Domain Independence  
Each domain extension must:
- Implement `DomainWorkflowExtension` trait
- Validate its own context requirements
- Transform context for cross-domain operations
- Handle domain-specific step types

## QUALITY GATES

Before any code is merged:
- [ ] All existing tests continue to pass
- [ ] New functionality has comprehensive test coverage
- [ ] Cross-domain workflows execute successfully
- [ ] Event correlation works across domain boundaries
- [ ] Performance requirements are met
- [ ] Documentation is updated
- [ ] Migration path is validated

## ANTI-PATTERNS TO AVOID

### ❌ Domain-Specific Workflow Engines
```rust
// DON'T create separate engines per domain
DocumentWorkflowEngine, PersonWorkflowEngine
```

### ❌ Inheritance-Based Extensions
```rust  
// DON'T use inheritance for domain logic
trait DocumentWorkflow: BaseWorkflow
```

### ❌ Non-CIM Events
```rust
// DON'T create events without correlation/causation
struct WorkflowStarted { workflow_id: WorkflowId } // Missing correlation!
```

### ❌ Domain-Specific Context Types
```rust
// DON'T create separate context types per domain
DocumentWorkflowContext, PersonWorkflowContext
```

## SUCCESS VISION

When complete, this consolidated workflow domain will:

1. **Unify All CIM Workflows** - Single engine serving all domains
2. **Enable Cross-Domain Workflows** - Business processes spanning multiple domains
3. **Provide Mathematical Foundation** - Category Theory-based composition
4. **Maintain CIM Compliance** - Full event correlation and CID integrity
5. **Reduce Development Time** - 50% faster workflow development across all domains
6. **Ensure Audit Compliance** - Cryptographic verification of all workflow events

This is the **most critical domain** in the entire CIM ecosystem. Every decision here impacts every other domain. Proceed with mathematical precision and architectural discipline.

## REMEMBER: Everything Flows Through Workflows

Every operation in CIM - document processing, person management, organization changes, location updates - ALL of it flows through this universal workflow system. You are building the **central nervous system** of the entire CIM architecture.

Build it with the precision and elegance it deserves.