# CIM Domain Workflow Context

This document provides essential context for understanding the CIM Domain Workflow system - the central nervous system through which ALL CIM operations flow.

## Architectural Overview

The CIM Domain Workflow system is the **universal workflow engine** that serves all CIM domains through composition and Category Theory foundations:

```
┌─────────────────────────────────────────────────────────────┐
│                    Universal Workflow Engine                 │
├─────────────────────────────────────────────────────────────┤
│  Document Domain  │  Person Domain   │  Organization Domain │
│  Extension        │  Extension       │  Extension           │
├─────────────────────────────────────────────────────────────┤
│               Cross-Domain Orchestration                   │
├─────────────────────────────────────────────────────────────┤
│                Event Correlation Layer                      │
├─────────────────────────────────────────────────────────────┤
│                    CID Integrity Verification               │
└─────────────────────────────────────────────────────────────┘
```

## Core Principles

### 1. Mathematical Foundation
- **Category Theory**: Domains are Categories, operations are Morphisms
- **Functorial Composition**: Structure-preserving transformations
- **Natural Transformations**: Context transformations between domains
- **CID Integrity**: Content-addressed identifiers for tamper-evident workflows

### 2. Universal Engine Pattern
- Single workflow engine serves ALL CIM domains
- Domain-specific logic through compositional extensions
- No inheritance - pure composition approach
- Cross-domain workflows with correlation tracking

### 3. CIM Compliance Standards
- Mandatory correlation/causation IDs on all events
- Standardized NATS subject patterns
- Cryptographic integrity verification
- Extensible context framework

## Domain Extension Architecture

### DomainWorkflowExtension Trait
```rust
#[async_trait]
pub trait DomainWorkflowExtension: Send + Sync {
    fn domain(&self) -> &'static str;
    async fn execute_domain_step(&self, step_type: &str, context: &mut WorkflowContext) -> WorkflowResult<StepResult>;
    fn validate_context(&self, context: &WorkflowContext) -> WorkflowResult<()>;
    fn transform_context(&self, context: &WorkflowContext, target_domain: &str) -> WorkflowResult<serde_json::Value>;
    fn supported_step_types(&self) -> Vec<String>;
}
```

### Supported Domains
1. **Document Domain**: Document lifecycle, review, approval, publishing
2. **Person Domain**: Identity verification, notifications, role management
3. **Organization Domain**: Department approvals, hierarchy operations
4. **Location Domain**: Geographic operations, site management
5. **Integration Domain**: External system integrations

### Cross-Domain Workflow Patterns
- **Event Correlation**: Track operations across domain boundaries
- **Context Transformation**: Safe data transformation between domains
- **Rollback Strategies**: Saga patterns, compensating transactions
- **Timeout Management**: Domain-specific timeout configurations

## Event System

### CimWorkflowEvent Structure
```rust
pub struct CimWorkflowEvent {
    pub identity: MessageIdentity,      // correlation/causation/message IDs
    pub instance_id: WorkflowInstanceId,
    pub source_domain: String,
    pub event: WorkflowEventType,
    pub timestamp: DateTime<Utc>,
    pub event_cid: Option<Cid>,         // cryptographic integrity
    pub previous_cid: Option<Cid>,      // chain linking
}
```

### NATS Subject Patterns
```
events.workflow.{domain}.{event_type}.{instance_id}
events.workflow.cross_domain.{event_type}.{correlation_id}
```

## Context Framework

### WorkflowContext Structure
```rust
pub struct WorkflowContext {
    pub variables: HashMap<String, serde_json::Value>,  // Universal variables
    pub extensions: HashMap<String, DomainExtension>,   // Domain-specific data
    pub metadata: ExecutionMetadata,                    // Correlation metadata
}
```

### Domain Extensions
- **document**: Document IDs, status, author info, review data
- **person**: Person IDs, notifications, roles, identity verification
- **organization**: Organization IDs, approval levels, hierarchy
- **location**: Site IDs, geographic data, facility information

## Workflow Templates

Pre-defined patterns for common cross-domain operations:
- **Document Approval**: Document → Person → Organization
- **Employee Onboarding**: Person → Organization → Document
- **Asset Management**: Location → Organization → Document
- **Incident Response**: Person → Organization → Location

## Development Guidelines

### 1. Domain Extension Development
- Implement `DomainWorkflowExtension` trait
- Focus on single domain responsibility
- Provide context transformation methods
- Include comprehensive error handling

### 2. Cross-Domain Workflows
- Use event correlation for tracking
- Implement proper timeout handling
- Design rollback strategies
- Test domain boundary transitions

### 3. CIM Compliance
- Always include correlation/causation IDs
- Use standardized NATS subjects
- Implement CID integrity verification
- Follow context extension patterns

## Integration Points

### With Other CIM Domains
- **cim-domain-document**: Document extension provides document operations
- **cim-domain-person**: Person extension handles identity and notifications
- **cim-domain-organization**: Organization extension manages approvals
- **cim-domain-location**: Location extension for geographic workflows

### With CIM Infrastructure
- **cim-graph**: Graph-based workflow visualization and analysis
- **NATS**: Event streaming and correlation
- **IPLD**: CID-based content addressing
- **Event Store**: Persistent workflow event storage

## Performance Considerations

### Scalability
- Asynchronous execution model
- Event-driven coordination
- Stateless domain extensions
- Horizontal scaling through NATS clustering

### Monitoring
- Workflow execution metrics
- Cross-domain transition tracking
- Event correlation analysis
- CID integrity verification status

## Security Model

### Cryptographic Integrity
- CID calculation for all events
- Chain verification for audit trails
- Tamper detection through hash validation
- Distributed verification capabilities

### Access Control
- Actor-based permissions
- Domain boundary enforcement
- Context data isolation
- Audit trail preservation

This context provides the foundation for understanding how ALL CIM operations flow through the unified workflow system, maintaining mathematical rigor while enabling practical business process automation.