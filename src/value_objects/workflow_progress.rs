//! Workflow progress tracking value object

use serde::{Deserialize, Serialize};

/// Represents the overall progress of a workflow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowProgress {
    /// Total number of steps in the workflow
    pub total_steps: usize,
    /// Number of completed steps
    pub completed_steps: usize,
    /// Number of steps currently in progress
    pub in_progress_steps: usize,
    /// Number of pending steps
    pub pending_steps: usize,
    /// Number of failed steps
    pub failed_steps: usize,
    /// Percentage of completion (0.0 to 100.0)
    pub percentage_complete: f32,
}

impl WorkflowProgress {
    /// Create a new workflow progress
    pub fn new(
        total_steps: usize,
        completed_steps: usize,
        in_progress_steps: usize,
        pending_steps: usize,
        failed_steps: usize,
    ) -> Self {
        let percentage_complete = if total_steps > 0 {
            (completed_steps as f32 / total_steps as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_steps,
            completed_steps,
            in_progress_steps,
            pending_steps,
            failed_steps,
            percentage_complete,
        }
    }

    /// Check if the workflow is complete
    pub fn is_complete(&self) -> bool {
        self.completed_steps == self.total_steps
    }

    /// Check if the workflow has any failures
    pub fn has_failures(&self) -> bool {
        self.failed_steps > 0
    }

    /// Check if the workflow is currently active
    pub fn is_active(&self) -> bool {
        self.in_progress_steps > 0
    }
} 