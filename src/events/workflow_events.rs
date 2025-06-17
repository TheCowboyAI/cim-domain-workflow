//! Workflow domain events

use crate::value_objects::{WorkflowId, WorkflowStatus, WorkflowContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A new workflow was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCreated {
    /// ID of the created workflow
    pub workflow_id: WorkflowId,
    /// Name of the workflow
    pub name: String,
    /// Description of the workflow
    pub description: String,
    /// Initial metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created by user
    pub created_by: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A workflow was started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStarted {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Execution context
    pub context: WorkflowContext,
    /// Started by user
    pub started_by: Option<String>,
    /// Start timestamp
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// A workflow was completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCompleted {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Final execution context
    pub final_context: WorkflowContext,
    /// Completion timestamp
    pub completed_at: chrono::DateTime<chrono::Utc>,
    /// Total duration in seconds
    pub duration_seconds: u64,
}

/// A workflow failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFailed {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Error message
    pub error: String,
    /// Context at time of failure
    pub failure_context: WorkflowContext,
    /// Failed timestamp
    pub failed_at: chrono::DateTime<chrono::Utc>,
    /// Duration before failure in seconds
    pub duration_seconds: u64,
}

/// A workflow was paused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPaused {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Reason for pausing
    pub reason: String,
    /// Context when paused
    pub pause_context: WorkflowContext,
    /// Paused by user
    pub paused_by: Option<String>,
    /// Pause timestamp
    pub paused_at: chrono::DateTime<chrono::Utc>,
}

/// A workflow was resumed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResumed {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Context when resumed
    pub resume_context: WorkflowContext,
    /// Resumed by user
    pub resumed_by: Option<String>,
    /// Resume timestamp
    pub resumed_at: chrono::DateTime<chrono::Utc>,
}

/// A workflow was cancelled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCancelled {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Reason for cancellation
    pub reason: String,
    /// Context when cancelled
    pub cancellation_context: WorkflowContext,
    /// Cancelled by user
    pub cancelled_by: Option<String>,
    /// Cancellation timestamp
    pub cancelled_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow metadata was added (following event sourcing - no updates, only additions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadataAdded {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Metadata key
    pub key: String,
    /// Metadata value
    pub value: serde_json::Value,
    /// Added by user
    pub added_by: Option<String>,
    /// Addition timestamp
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow metadata was removed (for event sourcing compliance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadataRemoved {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Metadata key that was removed
    pub key: String,
    /// Removed by user
    pub removed_by: Option<String>,
    /// Removal timestamp
    pub removed_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow context variable was set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContextVariableSet {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Variable key
    pub key: String,
    /// Variable value
    pub value: serde_json::Value,
    /// Set by user
    pub set_by: Option<String>,
    /// Set timestamp
    pub set_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow context variable was removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContextVariableRemoved {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// Variable key that was removed
    pub key: String,
    /// Removed by user
    pub removed_by: Option<String>,
    /// Removal timestamp
    pub removed_at: chrono::DateTime<chrono::Utc>,
} 