//! Domain events enum for the Workflow domain

use crate::events::*;
use serde::{Deserialize, Serialize};

/// All workflow domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowDomainEvent {
    // Workflow events
    WorkflowCreated(WorkflowCreated),
    WorkflowStarted(WorkflowStarted),
    WorkflowCompleted(WorkflowCompleted),
    WorkflowFailed(WorkflowFailed),
    WorkflowPaused(WorkflowPaused),
    WorkflowResumed(WorkflowResumed),
    WorkflowCancelled(WorkflowCancelled),
    WorkflowMetadataAdded(WorkflowMetadataAdded),
    WorkflowMetadataRemoved(WorkflowMetadataRemoved),
    WorkflowContextVariableSet(WorkflowContextVariableSet),
    WorkflowContextVariableRemoved(WorkflowContextVariableRemoved),

    // Step events
    StepAdded(StepAdded),
    StepRemoved(StepRemoved),
    StepExecutionStarted(StepExecutionStarted),
    StepExecutionCompleted(StepExecutionCompleted),
    StepExecutionFailed(StepExecutionFailed),
    StepSkipped(StepSkipped),
    StepAssignmentChanged(StepAssignmentChanged),
    StepDependencyAdded(StepDependencyAdded),
    StepDependencyRemoved(StepDependencyRemoved),
    StepConfigurationAdded(StepConfigurationAdded),
    StepConfigurationRemoved(StepConfigurationRemoved),
    StepApprovalRequested(StepApprovalRequested),
    StepApprovalGranted(StepApprovalGranted),
    StepApprovalRejected(StepApprovalRejected),
}

impl WorkflowDomainEvent {
    /// Get the workflow ID for this event
    pub fn workflow_id(&self) -> crate::value_objects::WorkflowId {
        match self {
            // Workflow events
            WorkflowDomainEvent::WorkflowCreated(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowStarted(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowCompleted(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowFailed(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowPaused(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowResumed(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowCancelled(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowMetadataAdded(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowMetadataRemoved(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowContextVariableSet(e) => e.workflow_id,
            WorkflowDomainEvent::WorkflowContextVariableRemoved(e) => e.workflow_id,

            // Step events
            WorkflowDomainEvent::StepAdded(e) => e.workflow_id,
            WorkflowDomainEvent::StepRemoved(e) => e.workflow_id,
            WorkflowDomainEvent::StepExecutionStarted(e) => e.workflow_id,
            WorkflowDomainEvent::StepExecutionCompleted(e) => e.workflow_id,
            WorkflowDomainEvent::StepExecutionFailed(e) => e.workflow_id,
            WorkflowDomainEvent::StepSkipped(e) => e.workflow_id,
            WorkflowDomainEvent::StepAssignmentChanged(e) => e.workflow_id,
            WorkflowDomainEvent::StepDependencyAdded(e) => e.workflow_id,
            WorkflowDomainEvent::StepDependencyRemoved(e) => e.workflow_id,
            WorkflowDomainEvent::StepConfigurationAdded(e) => e.workflow_id,
            WorkflowDomainEvent::StepConfigurationRemoved(e) => e.workflow_id,
            WorkflowDomainEvent::StepApprovalRequested(e) => e.workflow_id,
            WorkflowDomainEvent::StepApprovalGranted(e) => e.workflow_id,
            WorkflowDomainEvent::StepApprovalRejected(e) => e.workflow_id,
        }
    }

    /// Get the event name for debugging and logging
    pub fn event_name(&self) -> &'static str {
        match self {
            WorkflowDomainEvent::WorkflowCreated(_) => "WorkflowCreated",
            WorkflowDomainEvent::WorkflowStarted(_) => "WorkflowStarted",
            WorkflowDomainEvent::WorkflowCompleted(_) => "WorkflowCompleted",
            WorkflowDomainEvent::WorkflowFailed(_) => "WorkflowFailed",
            WorkflowDomainEvent::WorkflowPaused(_) => "WorkflowPaused",
            WorkflowDomainEvent::WorkflowResumed(_) => "WorkflowResumed",
            WorkflowDomainEvent::WorkflowCancelled(_) => "WorkflowCancelled",
            WorkflowDomainEvent::WorkflowMetadataAdded(_) => "WorkflowMetadataAdded",
            WorkflowDomainEvent::WorkflowMetadataRemoved(_) => "WorkflowMetadataRemoved",
            WorkflowDomainEvent::WorkflowContextVariableSet(_) => "WorkflowContextVariableSet",
            WorkflowDomainEvent::WorkflowContextVariableRemoved(_) => "WorkflowContextVariableRemoved",
            WorkflowDomainEvent::StepAdded(_) => "StepAdded",
            WorkflowDomainEvent::StepRemoved(_) => "StepRemoved",
            WorkflowDomainEvent::StepExecutionStarted(_) => "StepExecutionStarted",
            WorkflowDomainEvent::StepExecutionCompleted(_) => "StepExecutionCompleted",
            WorkflowDomainEvent::StepExecutionFailed(_) => "StepExecutionFailed",
            WorkflowDomainEvent::StepSkipped(_) => "StepSkipped",
            WorkflowDomainEvent::StepAssignmentChanged(_) => "StepAssignmentChanged",
            WorkflowDomainEvent::StepDependencyAdded(_) => "StepDependencyAdded",
            WorkflowDomainEvent::StepDependencyRemoved(_) => "StepDependencyRemoved",
            WorkflowDomainEvent::StepConfigurationAdded(_) => "StepConfigurationAdded",
            WorkflowDomainEvent::StepConfigurationRemoved(_) => "StepConfigurationRemoved",
            WorkflowDomainEvent::StepApprovalRequested(_) => "StepApprovalRequested",
            WorkflowDomainEvent::StepApprovalGranted(_) => "StepApprovalGranted",
            WorkflowDomainEvent::StepApprovalRejected(_) => "StepApprovalRejected",
        }
    }
} 