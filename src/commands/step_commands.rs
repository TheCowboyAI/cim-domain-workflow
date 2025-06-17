//! Workflow step commands

use crate::value_objects::{WorkflowId, StepId, StepType};
use crate::aggregate::Workflow;
use cim_domain::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Add a step to a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStep {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
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
}

impl Command for AddStep {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Remove a step from a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveStep {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step to remove
    pub step_id: StepId,
    /// Reason for removal
    pub reason: String,
    /// Removed by user
    pub removed_by: Option<String>,
}

impl Command for RemoveStep {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Start step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartStepExecution {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step to start
    pub step_id: StepId,
    /// Started by user
    pub started_by: Option<String>,
}

impl Command for StartStepExecution {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Complete step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteStepExecution {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step to complete
    pub step_id: StepId,
    /// Completion output
    pub output: HashMap<String, serde_json::Value>,
    /// Completed by user
    pub completed_by: Option<String>,
}

impl Command for CompleteStepExecution {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Fail step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailStepExecution {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step to fail
    pub step_id: StepId,
    /// Error message
    pub error: String,
    /// Error details
    pub error_details: HashMap<String, serde_json::Value>,
}

impl Command for FailStepExecution {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Skip step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipStep {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step to skip
    pub step_id: StepId,
    /// Reason for skipping
    pub reason: String,
    /// Skipped by user
    pub skipped_by: Option<String>,
}

impl Command for SkipStep {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Change step assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStepAssignment {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// New assignee
    pub new_assignee: Option<String>,
    /// Changed by user
    pub changed_by: Option<String>,
}

impl Command for ChangeStepAssignment {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Add step dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStepDependency {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// ID of the dependency step
    pub dependency_step_id: StepId,
    /// Added by user
    pub added_by: Option<String>,
}

impl Command for AddStepDependency {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Remove step dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveStepDependency {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// ID of the dependency step to remove
    pub dependency_step_id: StepId,
    /// Removed by user
    pub removed_by: Option<String>,
}

impl Command for RemoveStepDependency {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Add step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStepConfiguration {
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
}

impl Command for AddStepConfiguration {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Remove step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveStepConfiguration {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Configuration key to remove
    pub key: String,
    /// Removed by user
    pub removed_by: Option<String>,
}

impl Command for RemoveStepConfiguration {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Request step approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestStepApproval {
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
}

impl Command for RequestStepApproval {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Grant step approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantStepApproval {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Approval message
    pub message: Option<String>,
    /// Approved by user
    pub approved_by: String,
}

impl Command for GrantStepApproval {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
}

/// Reject step approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectStepApproval {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the step
    pub step_id: StepId,
    /// Rejection reason
    pub reason: String,
    /// Rejected by user
    pub rejected_by: String,
}

impl Command for RejectStepApproval {
    type Aggregate = Workflow;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_uuid(*self.workflow_id.as_uuid()))
    }
} 