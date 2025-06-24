//! Task-related events

use crate::value_objects::{WorkflowId, StepId};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Event emitted when a task is started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStarted {
    pub workflow_id: WorkflowId,
    pub step_id: StepId,
    pub started_by: Option<String>,
    pub started_at: DateTime<Utc>,
}

/// Event emitted when a task is assigned to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssigned {
    pub workflow_id: WorkflowId,
    pub step_id: StepId,
    pub assigned_to: String,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
}

/// Event emitted when a task is reassigned from one user to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReassigned {
    pub workflow_id: WorkflowId,
    pub step_id: StepId,
    pub from_assignee: String,
    pub to_assignee: String,
    pub reassigned_by: Option<String>,
    pub reassigned_at: DateTime<Utc>,
    pub reason: Option<String>,
}

/// Event emitted when a task is completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleted {
    pub workflow_id: WorkflowId,
    pub step_id: StepId,
    pub completed_by: String,
    pub completion_data: HashMap<String, serde_json::Value>,
    pub completed_at: DateTime<Utc>,
    pub duration_seconds: u64,
} 