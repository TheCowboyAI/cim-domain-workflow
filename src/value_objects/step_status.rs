//! Step status value object

use serde::{Deserialize, Serialize};

/// Represents the status of a workflow step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[derive(Default)]
pub enum StepStatus {
    /// Step is defined but not started
    #[default]
    Pending,
    /// Step is currently executing
    Running,
    /// Step is currently executing (alias for Running)
    InProgress,
    /// Step completed successfully
    Completed,
    /// Step failed during execution
    Failed,
    /// Step was skipped
    Skipped,
    /// Step was cancelled
    Cancelled,
    /// Step is waiting for approval
    WaitingApproval,
}

impl StepStatus {
    /// Check if this status indicates the step is completed
    pub fn is_completed(&self) -> bool {
        matches!(self, StepStatus::Completed | StepStatus::Skipped)
    }

    /// Check if this status indicates the step is active
    pub fn is_active(&self) -> bool {
        matches!(self, StepStatus::Running | StepStatus::InProgress | StepStatus::WaitingApproval)
    }

    /// Check if this status indicates the step has failed
    pub fn is_failed(&self) -> bool {
        matches!(self, StepStatus::Failed | StepStatus::Cancelled)
    }

    /// Check if the step can be started from this status
    pub fn can_start(&self) -> bool {
        matches!(self, StepStatus::Pending)
    }

    /// Check if the step can be completed from this status
    pub fn can_complete(&self) -> bool {
        matches!(self, StepStatus::Running | StepStatus::InProgress | StepStatus::WaitingApproval)
    }

    /// Check if the step can be cancelled from this status
    pub fn can_cancel(&self) -> bool {
        matches!(self, StepStatus::Pending | StepStatus::Running | StepStatus::InProgress | StepStatus::WaitingApproval)
    }
}

 