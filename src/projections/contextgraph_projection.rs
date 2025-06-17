//! ContextGraph projection for workflow domain
//!
//! This projection converts workflow aggregates into ContextGraph JSON format
//! for visualization, analysis, and integration with other systems.

use crate::{
    aggregate::Workflow,
    value_objects::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// ContextGraph representation of a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContextGraph {
    /// Graph metadata
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: WorkflowGraphMetadata,
    
    /// Graph structure
    pub nodes: Vec<ContextGraphNode>,
    pub edges: Vec<ContextGraphEdge>,
}

/// Metadata for the workflow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraphMetadata {
    /// Workflow status
    pub status: WorkflowStatus,
    /// Created by user
    pub created_by: Option<String>,
    /// Creation timestamp
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Started timestamp
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completed timestamp
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Workflow version
    pub version: u64,
    /// Custom workflow metadata
    pub properties: HashMap<String, serde_json::Value>,
}

/// Node in the workflow contextgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGraphNode {
    /// Unique node identifier
    pub id: String,
    /// Node type
    pub node_type: String,
    /// Node value/content
    pub value: ContextGraphNodeValue,
    /// Node components for extensibility
    pub components: HashMap<String, serde_json::Value>,
}

/// Values that can be stored in workflow nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ContextGraphNodeValue {
    /// Workflow step node
    Step {
        name: String,
        description: String,
        step_type: StepType,
        status: StepStatus,
        config: HashMap<String, serde_json::Value>,
        estimated_duration_minutes: Option<u32>,
        assigned_to: Option<String>,
    },
    /// Start node marker
    Start {
        name: String,
    },
    /// End node marker
    End {
        name: String,
    },
    /// Decision node
    Decision {
        name: String,
        description: String,
        conditions: HashMap<String, serde_json::Value>,
    },
}

/// Edge in the workflow contextgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGraphEdge {
    /// Unique edge identifier
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Edge type
    pub edge_type: String,
    /// Edge value/weight
    pub value: ContextGraphEdgeValue,
    /// Edge components for extensibility
    pub components: HashMap<String, serde_json::Value>,
}

/// Values that can be stored in workflow edges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ContextGraphEdgeValue {
    /// Sequential dependency
    Sequence {
        order: Option<u32>,
    },
    /// Conditional flow
    Conditional {
        condition: String,
        outcome: String,
    },
    /// Parallel flow
    Parallel {
        group: Option<String>,
    },
}

impl WorkflowContextGraph {
    /// Create a new workflow contextgraph from a workflow aggregate
    pub fn from_workflow(workflow: &Workflow) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Create start node
        let start_node_id = Uuid::new_v4().to_string();
        nodes.push(ContextGraphNode {
            id: start_node_id.clone(),
            node_type: "start".to_string(),
            value: ContextGraphNodeValue::Start {
                name: "Start".to_string(),
            },
            components: HashMap::new(),
        });

        // Create end node
        let end_node_id = Uuid::new_v4().to_string();
        nodes.push(ContextGraphNode {
            id: end_node_id.clone(),
            node_type: "end".to_string(),
            value: ContextGraphNodeValue::End {
                name: "End".to_string(),
            },
            components: HashMap::new(),
        });

        // Create step nodes
        let mut step_node_ids = HashMap::new();
        for (step_id, step) in &workflow.steps {
            let node_id = Uuid::new_v4().to_string();
            step_node_ids.insert(*step_id, node_id.clone());
            
            let mut components = HashMap::new();
            components.insert("step_id".to_string(), serde_json::json!(step_id.as_uuid()));
            
            nodes.push(ContextGraphNode {
                id: node_id,
                node_type: "step".to_string(),
                value: ContextGraphNodeValue::Step {
                    name: step.name.clone(),
                    description: step.description.clone(),
                    step_type: step.step_type.clone(),
                    status: step.status.clone(),
                    config: step.config.clone(),
                    estimated_duration_minutes: step.estimated_duration_minutes,
                    assigned_to: step.assigned_to.clone(),
                },
                components,
            });
        }

        // Create dependency edges
        for (step_id, step) in &workflow.steps {
            if let Some(step_node_id) = step_node_ids.get(step_id) {
                if step.dependencies.is_empty() {
                    // Connect to start node if no dependencies
                    edges.push(ContextGraphEdge {
                        id: Uuid::new_v4().to_string(),
                        source: start_node_id.clone(),
                        target: step_node_id.clone(),
                        edge_type: "sequence".to_string(),
                        value: ContextGraphEdgeValue::Sequence { order: None },
                        components: HashMap::new(),
                    });
                } else {
                    // Connect to dependency steps
                    for dependency_id in &step.dependencies {
                        if let Some(dependency_node_id) = step_node_ids.get(dependency_id) {
                            edges.push(ContextGraphEdge {
                                id: Uuid::new_v4().to_string(),
                                source: dependency_node_id.clone(),
                                target: step_node_id.clone(),
                                edge_type: "dependency".to_string(),
                                value: ContextGraphEdgeValue::Sequence { order: None },
                                components: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        // Connect terminal steps to end node
        for (step_id, step) in &workflow.steps {
            let is_terminal = !workflow.steps.values().any(|other_step| {
                other_step.dependencies.contains(step_id)
            });
            
            if is_terminal {
                if let Some(step_node_id) = step_node_ids.get(step_id) {
                    edges.push(ContextGraphEdge {
                        id: Uuid::new_v4().to_string(),
                        source: step_node_id.clone(),
                        target: end_node_id.clone(),
                        edge_type: "sequence".to_string(),
                        value: ContextGraphEdgeValue::Sequence { order: None },
                        components: HashMap::new(),
                    });
                }
            }
        }

        Self {
            id: workflow.id.as_uuid().to_string(),
            name: workflow.name.clone(),
            description: Some(workflow.description.clone()),
            metadata: WorkflowGraphMetadata {
                status: workflow.status.clone(),
                created_by: workflow.created_by.clone(),
                created_at: workflow.created_at,
                started_at: workflow.started_at,
                completed_at: workflow.completed_at,
                version: workflow.version,
                properties: workflow.metadata.clone(),
            },
            nodes,
            edges,
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get statistics about the workflow graph
    pub fn statistics(&self) -> WorkflowGraphStatistics {
        let step_nodes = self.nodes.iter()
            .filter(|n| n.node_type == "step")
            .count();
        
        let total_edges = self.edges.len();
        let dependency_edges = self.edges.iter()
            .filter(|e| e.edge_type == "dependency")
            .count();

        WorkflowGraphStatistics {
            total_nodes: self.nodes.len(),
            step_nodes,
            total_edges,
            dependency_edges,
            max_depth: self.calculate_max_depth(),
            is_cyclic: self.detect_cycles(),
        }
    }

    /// Calculate maximum depth of the workflow
    fn calculate_max_depth(&self) -> usize {
        // Simple topological depth calculation
        // In a real implementation, this would use proper graph traversal
        self.nodes.len() // Simplified for now
    }

    /// Detect if there are cycles in the workflow
    fn detect_cycles(&self) -> bool {
        // Simple cycle detection - in practice would use proper graph algorithms
        false // Simplified for now
    }

    /// Get all step nodes
    pub fn get_step_nodes(&self) -> Vec<&ContextGraphNode> {
        self.nodes.iter()
            .filter(|n| n.node_type == "step")
            .collect()
    }

    /// Get all dependency edges
    pub fn get_dependency_edges(&self) -> Vec<&ContextGraphEdge> {
        self.edges.iter()
            .filter(|e| e.edge_type == "dependency")
            .collect()
    }

    /// Export as DOT format for graphviz visualization
    pub fn to_dot(&self) -> String {
        let mut dot = format!("digraph \"{}\" {{\n", self.name);
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n");

        // Add nodes
        for node in &self.nodes {
            let label = match &node.value {
                ContextGraphNodeValue::Step { name, .. } => name.clone(),
                ContextGraphNodeValue::Start { name } => name.clone(),
                ContextGraphNodeValue::End { name } => name.clone(),
                ContextGraphNodeValue::Decision { name, .. } => name.clone(),
            };
            
            let shape = match &node.value {
                ContextGraphNodeValue::Start { .. } => "ellipse",
                ContextGraphNodeValue::End { .. } => "ellipse",
                ContextGraphNodeValue::Decision { .. } => "diamond",
                _ => "box",
            };

            dot.push_str(&format!("  \"{}\" [label=\"{}\" shape=\"{}\"];\n", 
                node.id, label, shape));
        }

        // Add edges
        for edge in &self.edges {
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", 
                edge.source, edge.target));
        }

        dot.push_str("}\n");
        dot
    }
}

/// Statistics about a workflow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraphStatistics {
    pub total_nodes: usize,
    pub step_nodes: usize,
    pub total_edges: usize,
    pub dependency_edges: usize,
    pub max_depth: usize,
    pub is_cyclic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_workflow_to_contextgraph() {
        // Create a test workflow
        let (mut workflow, _events) = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow for projection".to_string(),
            HashMap::new(),
            Some("test-user".to_string()),
        ).unwrap();

        // Add some steps
        workflow.add_step(
            "Step 1".to_string(),
            "First step".to_string(),
            StepType::Manual,
            HashMap::new(),
            Vec::new(),
            Some(30),
            Some("assignee1".to_string()),
            Some("creator".to_string()),
        ).unwrap();

        // Convert to contextgraph
        let contextgraph = WorkflowContextGraph::from_workflow(&workflow);

        // Verify structure
        assert_eq!(contextgraph.name, "Test Workflow");
        assert!(contextgraph.nodes.len() >= 3); // start + step + end
        assert!(!contextgraph.edges.is_empty());

        // Test JSON serialization
        let json = contextgraph.to_json().unwrap();
        assert!(json.contains("Test Workflow"));

        // Test deserialization
        let deserialized = WorkflowContextGraph::from_json(&json).unwrap();
        assert_eq!(deserialized.name, contextgraph.name);
    }

    #[test]
    fn test_contextgraph_statistics() {
        let (workflow, _events) = Workflow::new(
            "Stats Test".to_string(),
            "Testing statistics".to_string(),
            HashMap::new(),
            Some("test-user".to_string()),
        ).unwrap();

        let contextgraph = WorkflowContextGraph::from_workflow(&workflow);
        let stats = contextgraph.statistics();

        assert_eq!(stats.total_nodes, 2); // start + end only
        assert_eq!(stats.step_nodes, 0);
    }

    #[test]
    fn test_dot_export() {
        let (workflow, _events) = Workflow::new(
            "DOT Test".to_string(),
            "Testing DOT export".to_string(),
            HashMap::new(),
            Some("test-user".to_string()),
        ).unwrap();

        let contextgraph = WorkflowContextGraph::from_workflow(&workflow);
        let dot = contextgraph.to_dot();

        assert!(dot.contains("digraph"));
        assert!(dot.contains("DOT Test"));
        assert!(dot.contains("Start"));
        assert!(dot.contains("End"));
    }
} 