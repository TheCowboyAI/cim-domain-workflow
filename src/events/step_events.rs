//! Workflow step domain events

use crate::value_objects::{WorkflowId, StepId, StepType, StepStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A step was added to a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAdded {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Name of the step
    pub name: String,
    /// Description of the step
    pub description: String,
    /// Type of step
    pub step_type: StepType,
    /// Initial configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Dependencies on other steps
    pub dependencies: Vec<StepId>,
    /// Estimated duration in minutes
    pub estimated_duration_minutes: Option<u32>,
    /// Assigned user or role
    pub assigned_to: Option<String>,
    /// Added by user
    pub added_by: Option<String>,
    /// Addition timestamp
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// A step was removed from a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRemoved {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step that was removed
    pub step_id: StepId,
    /// Reason for removal
    pub reason: String,
    /// Removed by user
    pub removed_by: Option<String>,
    /// Removal timestamp
    pub removed_at: chrono::DateTime<chrono::Utc>,
}

/// Step execution was started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionStarted {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Started by user
    pub started_by: Option<String>,
    /// Start timestamp
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Step execution was completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionCompleted {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Completion output
    pub output: HashMap<String, serde_json::Value>,
    /// Completed by user
    pub completed_by: Option<String>,
    /// Completion timestamp
    pub completed_at: chrono::DateTime<chrono::Utc>,
    /// Duration in seconds
    pub duration_seconds: u64,
}

/// Step execution failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionFailed {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Error message
    pub error: String,
    /// Error details
    pub error_details: HashMap<String, serde_json::Value>,
    /// Failed timestamp
    pub failed_at: chrono::DateTime<chrono::Utc>,
    /// Duration before failure in seconds
    pub duration_seconds: u64,
}

/// Step was skipped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSkipped {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Reason for skipping
    pub reason: String,
    /// Skipped by user
    pub skipped_by: Option<String>,
    /// Skip timestamp
    pub skipped_at: chrono::DateTime<chrono::Utc>,
}

/// Step assignment was changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAssignmentChanged {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Previous assignee
    pub previous_assignee: Option<String>,
    /// New assignee
    pub new_assignee: Option<String>,
    /// Changed by user
    pub changed_by: Option<String>,
    /// Change timestamp
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

/// Step dependency was added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDependencyAdded {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// ID of the dependency step
    pub dependency_step_id: StepId,
    /// Added by user
    pub added_by: Option<String>,
    /// Addition timestamp
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// Step dependency was removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDependencyRemoved {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// ID of the dependency step that was removed
    pub dependency_step_id: StepId,
    /// Removed by user
    pub removed_by: Option<String>,
    /// Removal timestamp
    pub removed_at: chrono::DateTime<chrono::Utc>,
}

/// Step configuration was added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfigurationAdded {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Configuration key
    pub key: String,
    /// Configuration value
    pub value: serde_json::Value,
    /// Added by user
    pub added_by: Option<String>,
    /// Addition timestamp
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// Step configuration was removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfigurationRemoved {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Configuration key that was removed
    pub key: String,
    /// Removed by user
    pub removed_by: Option<String>,
    /// Removal timestamp
    pub removed_at: chrono::DateTime<chrono::Utc>,
}

/// Step approval was requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepApprovalRequested {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Approver user or role
    pub approver: String,
    /// Approval request message
    pub message: Option<String>,
    /// Requested by user
    pub requested_by: Option<String>,
    /// Request timestamp
    pub requested_at: chrono::DateTime<chrono::Utc>,
}

/// Step approval was granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepApprovalGranted {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Approval message
    pub message: Option<String>,
    /// Approved by user
    pub approved_by: String,
    /// Approval timestamp
    pub approved_at: chrono::DateTime<chrono::Utc>,
}

/// Step approval was rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepApprovalRejected {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Rejection reason
    pub reason: String,
    /// Rejected by user
    pub rejected_by: String,
    /// Rejection timestamp
    pub rejected_at: chrono::DateTime<chrono::Utc>,
} 