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
    StepFailed(StepFailed),
    StepAssignmentChanged(StepAssignmentChanged),
    StepDependencyAdded(StepDependencyAdded),
    StepDependencyRemoved(StepDependencyRemoved),
    StepConfigurationAdded(StepConfigurationAdded),
    StepConfigurationRemoved(StepConfigurationRemoved),
    StepApprovalRequested(StepApprovalRequested),
    StepApprovalGranted(StepApprovalGranted),
    StepApprovalRejected(StepApprovalRejected),

    // Task assignment events
    TaskStarted(TaskStarted),
    TaskAssigned(TaskAssigned),
    TaskReassigned(TaskReassigned),
    TaskCompleted(TaskCompleted),

    // Cross-domain events
    CrossDomainOperationRequested(CrossDomainOperationRequested),
    CrossDomainOperationCompleted(CrossDomainOperationCompleted),
    CrossDomainOperationFailed(CrossDomainOperationFailed),
    CrossDomainEventSubscriptionRequested(CrossDomainEventSubscriptionRequested),
    CrossDomainEventSubscriptionCancelled(CrossDomainEventSubscriptionCancelled),
    CrossDomainEventReceived(CrossDomainEventReceived),
    CrossDomainTransactionStarted(CrossDomainTransactionStarted),
    CrossDomainTransactionPrepared(CrossDomainTransactionPrepared),
    CrossDomainTransactionCommitted(CrossDomainTransactionCommitted),
    CrossDomainTransactionRolledBack(CrossDomainTransactionRolledBack),
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
            WorkflowDomainEvent::StepFailed(e) => e.workflow_id,
            WorkflowDomainEvent::StepAssignmentChanged(e) => e.workflow_id,
            WorkflowDomainEvent::StepDependencyAdded(e) => e.workflow_id,
            WorkflowDomainEvent::StepDependencyRemoved(e) => e.workflow_id,
            WorkflowDomainEvent::StepConfigurationAdded(e) => e.workflow_id,
            WorkflowDomainEvent::StepConfigurationRemoved(e) => e.workflow_id,
            WorkflowDomainEvent::StepApprovalRequested(e) => e.workflow_id,
            WorkflowDomainEvent::StepApprovalGranted(e) => e.workflow_id,
            WorkflowDomainEvent::StepApprovalRejected(e) => e.workflow_id,

            // Task assignment events
            WorkflowDomainEvent::TaskStarted(e) => e.workflow_id,
            WorkflowDomainEvent::TaskAssigned(e) => e.workflow_id,
            WorkflowDomainEvent::TaskReassigned(e) => e.workflow_id,
            WorkflowDomainEvent::TaskCompleted(e) => e.workflow_id,

            // Cross-domain events
            WorkflowDomainEvent::CrossDomainOperationRequested(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainOperationCompleted(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainOperationFailed(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainEventSubscriptionRequested(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainEventSubscriptionCancelled(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainEventReceived(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainTransactionStarted(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainTransactionPrepared(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainTransactionCommitted(e) => e.workflow_id,
            WorkflowDomainEvent::CrossDomainTransactionRolledBack(e) => e.workflow_id,
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
            WorkflowDomainEvent::StepFailed(_) => "StepFailed",
            WorkflowDomainEvent::StepAssignmentChanged(_) => "StepAssignmentChanged",
            WorkflowDomainEvent::StepDependencyAdded(_) => "StepDependencyAdded",
            WorkflowDomainEvent::StepDependencyRemoved(_) => "StepDependencyRemoved",
            WorkflowDomainEvent::StepConfigurationAdded(_) => "StepConfigurationAdded",
            WorkflowDomainEvent::StepConfigurationRemoved(_) => "StepConfigurationRemoved",
            WorkflowDomainEvent::StepApprovalRequested(_) => "StepApprovalRequested",
            WorkflowDomainEvent::StepApprovalGranted(_) => "StepApprovalGranted",
            WorkflowDomainEvent::StepApprovalRejected(_) => "StepApprovalRejected",
            WorkflowDomainEvent::TaskStarted(_) => "TaskStarted",
            WorkflowDomainEvent::TaskAssigned(_) => "TaskAssigned",
            WorkflowDomainEvent::TaskReassigned(_) => "TaskReassigned",
            WorkflowDomainEvent::TaskCompleted(_) => "TaskCompleted",
            WorkflowDomainEvent::CrossDomainOperationRequested(_) => "CrossDomainOperationRequested",
            WorkflowDomainEvent::CrossDomainOperationCompleted(_) => "CrossDomainOperationCompleted",
            WorkflowDomainEvent::CrossDomainOperationFailed(_) => "CrossDomainOperationFailed",
            WorkflowDomainEvent::CrossDomainEventSubscriptionRequested(_) => "CrossDomainEventSubscriptionRequested",
            WorkflowDomainEvent::CrossDomainEventSubscriptionCancelled(_) => "CrossDomainEventSubscriptionCancelled",
            WorkflowDomainEvent::CrossDomainEventReceived(_) => "CrossDomainEventReceived",
            WorkflowDomainEvent::CrossDomainTransactionStarted(_) => "CrossDomainTransactionStarted",
            WorkflowDomainEvent::CrossDomainTransactionPrepared(_) => "CrossDomainTransactionPrepared",
            WorkflowDomainEvent::CrossDomainTransactionCommitted(_) => "CrossDomainTransactionCommitted",
            WorkflowDomainEvent::CrossDomainTransactionRolledBack(_) => "CrossDomainTransactionRolledBack",
        }
    }
} 