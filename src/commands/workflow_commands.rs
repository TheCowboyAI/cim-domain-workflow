//! Workflow commands

use crate::value_objects::{WorkflowId, WorkflowContext};
use crate::aggregate::Workflow;
use cim_domain::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Create a new workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflow {
    /// Name of the workflow
    pub name: String,
    /// Description of the workflow
    pub description: String,
    /// Initial metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created by user
    pub created_by: Option<String>,
}

impl Command for CreateWorkflow {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        // No aggregate ID for creation command
        None
    }
}

/// Start workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartWorkflow {
    /// ID of the workflow to start
    pub workflow_id: WorkflowId,
    /// Execution context
    pub context: WorkflowContext,
    /// Started by user
    pub started_by: Option<String>,
}

impl Command for StartWorkflow {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Complete workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteWorkflow {
    /// ID of the workflow to complete
    pub workflow_id: WorkflowId,
    /// Final execution context
    pub final_context: WorkflowContext,
}

impl Command for CompleteWorkflow {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Fail workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailWorkflow {
    /// ID of the workflow to fail
    pub workflow_id: WorkflowId,
    /// Error message
    pub error: String,
    /// Context at time of failure
    pub failure_context: WorkflowContext,
}

impl Command for FailWorkflow {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Pause workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseWorkflow {
    /// ID of the workflow to pause
    pub workflow_id: WorkflowId,
    /// Reason for pausing
    pub reason: String,
    /// Paused by user
    pub paused_by: Option<String>,
}

impl Command for PauseWorkflow {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Resume workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeWorkflow {
    /// ID of the workflow to resume
    pub workflow_id: WorkflowId,
    /// Resumed by user
    pub resumed_by: Option<String>,
}

impl Command for ResumeWorkflow {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Cancel workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelWorkflow {
    /// ID of the workflow to cancel
    pub workflow_id: WorkflowId,
    /// Reason for cancellation
    pub reason: String,
    /// Cancelled by user
    pub cancelled_by: Option<String>,
}

impl Command for CancelWorkflow {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Add metadata to workflow (event sourcing compliant - no updates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddWorkflowMetadata {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Metadata key
    pub key: String,
    /// Metadata value
    pub value: serde_json::Value,
    /// Added by user
    pub added_by: Option<String>,
}

impl Command for AddWorkflowMetadata {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Remove metadata from workflow (event sourcing compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveWorkflowMetadata {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Metadata key to remove
    pub key: String,
    /// Removed by user
    pub removed_by: Option<String>,
}

impl Command for RemoveWorkflowMetadata {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Set workflow context variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetWorkflowContextVariable {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Variable key
    pub key: String,
    /// Variable value
    pub value: serde_json::Value,
    /// Set by user
    pub set_by: Option<String>,
}

impl Command for SetWorkflowContextVariable {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Remove workflow context variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveWorkflowContextVariable {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Variable key to remove
    pub key: String,
    /// Removed by user
    pub removed_by: Option<String>,
}

impl Command for RemoveWorkflowContextVariable {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
} 