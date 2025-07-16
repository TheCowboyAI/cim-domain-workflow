# CIM Workflow Domain

This domain implements workflow management following Domain-Driven Design (DDD) principles with event sourcing and CQRS patterns.

## Features

- **Event-Driven Architecture**: All workflow changes generate domain events
- **CQRS Pattern**: Separate command and query models
- **Step Dependencies**: Define complex workflow dependencies
- **Status Management**: Track workflow and step execution states
- **ContextGraph Export**: Export workflows to JSON format for visualization

## ContextGraph Projection

The workflow domain includes a powerful projection feature that converts workflow aggregates into ContextGraph JSON format. This enables:

- **Visualization**: Export workflows as graphs for visualization tools
- **Analysis**: Get statistics about workflow complexity and structure
- **Integration**: Standard JSON format for system integration
- **DOT Export**: Generate Graphviz DOT files for visual diagrams

### Example Usage

```rust
use cim_domain_workflow::{
    aggregate::Workflow,
    projections::WorkflowContextGraph,
};

// Create workflow
let (workflow, _events) = Workflow::new(
    "Deploy Application".to_string(),
    "Deployment pipeline".to_string(),
    HashMap::new(),
    Some("admin".to_string()),
)?;

// Export to ContextGraph JSON
let contextgraph = WorkflowContextGraph::from_workflow(&workflow);
let json = contextgraph.to_json()?;
println!("{}", json);

// Generate DOT for Graphviz
let dot = contextgraph.to_dot();
println!("{}", dot);

// Get statistics
let stats = contextgraph.statistics();
println!("Nodes: {}, Edges: {}", stats.total_nodes, stats.total_edges);
```

### JSON Structure

The ContextGraph JSON format includes:

- **Graph metadata**: Workflow ID, name, description, status
- **Nodes**: Start/end markers and workflow steps with full details
- **Edges**: Dependencies and sequential relationships
- **Components**: Extensible metadata for nodes and edges

### Running the Example

```bash
cargo run --example contextgraph_export
```

This demonstrates a complete software deployment pipeline with 7 steps including code review, building, testing, security scanning, staging deployment, validation, and production deployment.

## Domain Structure

- `aggregate/` - Workflow aggregate root
- `value_objects/` - Domain value objects (WorkflowId, StepId, etc.)
- `commands/` - CQRS command objects
- `events/` - Domain events for event sourcing
- `handlers/` - Command and query handlers
- `projections/` - Read model projections (including ContextGraph)
- `queries/` - Query objects

## Testing

Run all tests:

```bash
cargo test
```

Tests include:
- Workflow aggregate behavior
- Command handler functionality
- Step management and dependencies
- ContextGraph projection accuracy 