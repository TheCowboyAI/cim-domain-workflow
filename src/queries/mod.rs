//! Query objects for the Workflow domain

use crate::value_objects::{WorkflowId, WorkflowStatus, StepId};
use serde::{Deserialize, Serialize};

/// Query to find a specific workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindWorkflow {
    pub workflow_id: WorkflowId,
}

/// Query to list workflows with optional filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkflows {
    pub status: Option<WorkflowStatus>,
    pub created_by: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// Query to get all steps in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkflowSteps {
    pub workflow_id: WorkflowId,
}

/// Query to get executable steps in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetExecutableSteps {
    pub workflow_id: WorkflowId,
}

/// Query to search workflows by name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWorkflows {
    pub query: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// View model for workflow information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowView {
    pub id: WorkflowId,
    pub name: String,
    pub description: String,
    pub status: WorkflowStatus,
    pub step_count: usize,
    pub created_by: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// View model for step information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepView {
    pub id: StepId,
    pub name: String,
    pub description: String,
    pub step_type: String,
    pub status: String,
    pub dependencies: Vec<StepId>,
    pub assigned_to: Option<String>,
    pub estimated_duration_minutes: Option<u32>,
} 