//! Unified identifier system for the consolidated workflow domain
//! 
//! This module provides deterministic, domain-aware identifiers that enable
//! workflows to span multiple CIM domains while maintaining clear identity.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt::{self, Display};

/// Universal workflow identifier with deterministic creation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniversalWorkflowId {
    /// Base UUID for the workflow
    id: Uuid,
    /// Domain context that originated this workflow
    origin_domain: String,
    /// Optional template identifier if created from template
    template_id: Option<String>,
}

impl UniversalWorkflowId {
    /// Create a new workflow ID for a specific domain
    pub fn new(origin_domain: String, template_id: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            origin_domain,
            template_id,
        }
    }

    /// Create a deterministic workflow ID from components
    pub fn deterministic(
        origin_domain: String, 
        workflow_name: &str, 
        context_hash: &str
    ) -> Self {
        let namespace = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8")
            .expect("Invalid namespace UUID");
        let input = format!("{}:{}:{}", origin_domain, workflow_name, context_hash);
        
        Self {
            id: Uuid::new_v5(&namespace, input.as_bytes()),
            origin_domain,
            template_id: None,
        }
    }

    /// Get the base UUID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the origin domain
    pub fn origin_domain(&self) -> &str {
        &self.origin_domain
    }

    /// Get the template ID if applicable
    pub fn template_id(&self) -> Option<&str> {
        self.template_id.as_deref()
    }

    /// Check if this workflow originated from a specific domain
    pub fn is_from_domain(&self, domain: &str) -> bool {
        self.origin_domain == domain
    }
}

impl Display for UniversalWorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(template) = &self.template_id {
            write!(f, "{}:{}:{}", self.origin_domain, template, self.id)
        } else {
            write!(f, "{}:{}", self.origin_domain, self.id)
        }
    }
}

/// Universal step identifier for cross-domain step coordination
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniversalStepId {
    /// Step UUID
    id: Uuid,
    /// Parent workflow ID
    workflow_id: UniversalWorkflowId,
    /// Step sequence number within workflow
    sequence: u32,
    /// Domain responsible for executing this step
    executor_domain: String,
}

impl UniversalStepId {
    /// Create a new step ID
    pub fn new(
        workflow_id: UniversalWorkflowId,
        sequence: u32,
        executor_domain: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            sequence,
            executor_domain,
        }
    }

    /// Create a deterministic step ID
    pub fn deterministic(
        workflow_id: UniversalWorkflowId,
        step_name: &str,
        sequence: u32,
        executor_domain: String,
    ) -> Self {
        let namespace = Uuid::parse_str("6ba7b811-9dad-11d1-80b4-00c04fd430c8")
            .expect("Invalid namespace UUID");
        let input = format!("{}:{}:{}", workflow_id.id, step_name, sequence);
        
        Self {
            id: Uuid::new_v5(&namespace, input.as_bytes()),
            workflow_id,
            sequence,
            executor_domain,
        }
    }

    /// Get the step UUID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the parent workflow ID
    pub fn workflow_id(&self) -> &UniversalWorkflowId {
        &self.workflow_id
    }

    /// Get the sequence number
    pub fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Get the executor domain
    pub fn executor_domain(&self) -> &str {
        &self.executor_domain
    }

    /// Check if this step is cross-domain (executor differs from origin)
    pub fn is_cross_domain(&self) -> bool {
        self.executor_domain != self.workflow_id.origin_domain
    }
}

impl Display for UniversalStepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "{}:step-{}:{}:{}",
            self.workflow_id,
            self.sequence,
            self.executor_domain,
            self.id
        )
    }
}

/// Workflow instance identifier for runtime tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowInstanceId {
    /// Instance UUID
    id: Uuid,
    /// Workflow template ID
    workflow_id: UniversalWorkflowId,
    /// Instance creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Instance context hash for deterministic creation
    context_hash: Option<String>,
}

impl WorkflowInstanceId {
    /// Create a new workflow instance
    pub fn new(workflow_id: UniversalWorkflowId) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            created_at: chrono::Utc::now(),
            context_hash: None,
        }
    }

    /// Create a deterministic workflow instance
    pub fn deterministic(
        workflow_id: UniversalWorkflowId,
        context_hash: String,
    ) -> Self {
        let namespace = Uuid::parse_str("6ba7b812-9dad-11d1-80b4-00c04fd430c8")
            .expect("Invalid namespace UUID");
        let input = format!("{}:{}", workflow_id.id, context_hash);
        
        Self {
            id: Uuid::new_v5(&namespace, input.as_bytes()),
            workflow_id,
            created_at: chrono::Utc::now(),
            context_hash: Some(context_hash),
        }
    }

    /// Get the instance UUID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the workflow ID
    pub fn workflow_id(&self) -> &UniversalWorkflowId {
        &self.workflow_id
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created_at
    }

    /// Get context hash if deterministic
    pub fn context_hash(&self) -> Option<&str> {
        self.context_hash.as_deref()
    }
}

impl Display for WorkflowInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "instance:{}:{}", self.workflow_id, self.id)
    }
}

/// Domain-aware node identifier for workflow graph topology
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    /// Node UUID
    id: Uuid,
    /// Domain this node belongs to
    domain: String,
    /// Node type (workflow, step, gateway, etc.)
    node_type: NodeType,
    /// Parent container (workflow or subprocess)
    parent_id: Option<Uuid>,
}

/// Types of nodes in the workflow graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// Workflow root node
    Workflow,
    /// Individual step
    Step,
    /// Decision gateway
    Gateway,
    /// Parallel fork
    Fork,
    /// Parallel join
    Join,
    /// Event trigger
    Event,
    /// Timer trigger
    Timer,
    /// Cross-domain bridge
    Bridge,
}

impl NodeId {
    /// Create a new node ID
    pub fn new(domain: String, node_type: NodeType, parent_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            domain,
            node_type,
            parent_id,
        }
    }

    /// Create a deterministic node ID
    pub fn deterministic(
        domain: String,
        node_type: NodeType,
        name: &str,
        parent_id: Option<Uuid>,
    ) -> Self {
        let namespace = Uuid::parse_str("6ba7b813-9dad-11d1-80b4-00c04fd430c8")
            .expect("Invalid namespace UUID");
        let parent_str = parent_id
            .map(|p| p.to_string())
            .unwrap_or_else(|| "root".to_string());
        let input = format!("{}:{}:{}:{}", domain, node_type.as_str(), name, parent_str);
        
        Self {
            id: Uuid::new_v5(&namespace, input.as_bytes()),
            domain,
            node_type,
            parent_id,
        }
    }

    /// Get the node UUID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the domain
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Get the node type
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    /// Get the parent ID
    pub fn parent_id(&self) -> Option<&Uuid> {
        self.parent_id.as_ref()
    }

    /// Check if this node is in a specific domain
    pub fn is_in_domain(&self, domain: &str) -> bool {
        self.domain == domain
    }

    /// Check if this is a cross-domain bridge node
    pub fn is_bridge(&self) -> bool {
        matches!(self.node_type, NodeType::Bridge)
    }
}

impl NodeType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Workflow => "workflow",
            NodeType::Step => "step",
            NodeType::Gateway => "gateway",
            NodeType::Fork => "fork",
            NodeType::Join => "join",
            NodeType::Event => "event",
            NodeType::Timer => "timer",
            NodeType::Bridge => "bridge",
        }
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(parent) = &self.parent_id {
            write!(
                f,
                "{}:{}:{}:parent-{}",
                self.domain,
                self.node_type.as_str(),
                self.id,
                parent
            )
        } else {
            write!(f, "{}:{}:{}", self.domain, self.node_type.as_str(), self.id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_workflow_id_creation() {
        let id = UniversalWorkflowId::new("document".to_string(), None);
        assert_eq!(id.origin_domain(), "document");
        assert!(id.template_id().is_none());
        assert!(id.is_from_domain("document"));
        assert!(!id.is_from_domain("person"));
    }

    #[test]
    fn test_deterministic_workflow_id() {
        let id1 = UniversalWorkflowId::deterministic(
            "document".to_string(),
            "review-workflow",
            "hash123",
        );
        let id2 = UniversalWorkflowId::deterministic(
            "document".to_string(),
            "review-workflow", 
            "hash123",
        );
        
        assert_eq!(id1.id(), id2.id());
        assert_eq!(id1.origin_domain(), id2.origin_domain());
    }

    #[test]
    fn test_universal_step_id_cross_domain() {
        let workflow_id = UniversalWorkflowId::new("document".to_string(), None);
        let step_id = UniversalStepId::new(workflow_id, 1, "person".to_string());
        
        assert!(step_id.is_cross_domain());
        assert_eq!(step_id.sequence(), 1);
        assert_eq!(step_id.executor_domain(), "person");
    }

    #[test]
    fn test_workflow_instance_id() {
        let workflow_id = UniversalWorkflowId::new("document".to_string(), None);
        let instance = WorkflowInstanceId::new(workflow_id.clone());
        
        assert_eq!(instance.workflow_id(), &workflow_id);
        assert!(instance.context_hash().is_none());
    }

    #[test]
    fn test_node_id_bridge() {
        let node = NodeId::new(
            "integration".to_string(),
            NodeType::Bridge,
            None,
        );
        
        assert!(node.is_bridge());
        assert!(node.is_in_domain("integration"));
        assert_eq!(node.node_type().as_str(), "bridge");
    }
}