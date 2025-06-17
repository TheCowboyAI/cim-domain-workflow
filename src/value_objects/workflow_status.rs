//! Workflow status value object

use serde::{Deserialize, Serialize};
use cim_domain::{DomainError, DomainResult};

/// Represents the status of a workflow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is defined but not started
    Draft,
    /// Workflow is currently executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed during execution
    Failed,
    /// Workflow was paused
    Paused,
    /// Workflow was cancelled
    Cancelled,
}

impl WorkflowStatus {
    /// Check if this status can transition to another status
    pub fn can_transition_to(&self, target: &WorkflowStatus) -> bool {
        use WorkflowStatus::*;
        
        match (self, target) {
            // From Draft
            (Draft, Running) => true,
            (Draft, Cancelled) => true,
            
            // From Running
            (Running, Completed) => true,
            (Running, Failed) => true,
            (Running, Paused) => true,
            (Running, Cancelled) => true,
            
            // From Paused
            (Paused, Running) => true,
            (Paused, Cancelled) => true,
            (Paused, Failed) => true,
            
            // Terminal states cannot transition
            (Completed, _) => false,
            (Failed, _) => false,
            (Cancelled, _) => false,
            
            // All other transitions are invalid
            _ => false,
        }
    }

    /// Validate and perform a status transition
    pub fn transition_to(&self, target: WorkflowStatus) -> DomainResult<WorkflowStatus> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(DomainError::generic(format!(
                "Invalid workflow status transition from {:?} to {:?}",
                self, target
            )))
        }
    }

    /// Check if this is a terminal status
    pub fn is_terminal(&self) -> bool {
        matches!(self, WorkflowStatus::Completed | WorkflowStatus::Failed | WorkflowStatus::Cancelled)
    }

    /// Check if the workflow is active (can execute steps)
    pub fn is_active(&self) -> bool {
        matches!(self, WorkflowStatus::Running)
    }
} 