//! Workflow projection for read models

use crate::{
    value_objects::{WorkflowId, WorkflowStatus, StepId, StepStatus},
    queries::{WorkflowView, StepView},
    events::*,
};
use std::collections::HashMap;

/// In-memory projection of workflow state
#[derive(Debug, Clone, Default)]
pub struct WorkflowProjection {
    workflows: HashMap<WorkflowId, WorkflowView>,
    steps: HashMap<WorkflowId, HashMap<StepId, StepView>>,
}

impl WorkflowProjection {
    /// Create a new empty projection
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Find workflow by ID
    pub fn find_by_id(&self, id: WorkflowId) -> Option<WorkflowView> {
        self.workflows.get(&id).cloned()
    }
    
    /// Find workflows by status
    pub fn find_by_status(&self, status: WorkflowStatus) -> Vec<WorkflowView> {
        self.workflows.values()
            .filter(|w| w.status == status)
            .cloned()
            .collect()
    }
    
    /// List all workflows
    pub fn list_all(&self) -> Vec<WorkflowView> {
        self.workflows.values().cloned().collect()
    }
    
    /// Get steps for a workflow
    pub fn get_steps(&self, workflow_id: WorkflowId) -> Vec<StepView> {
        self.steps.get(&workflow_id)
            .map(|steps| steps.values().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get executable steps for a workflow
    pub fn get_executable_steps(&self, workflow_id: WorkflowId) -> Vec<StepView> {
        if let Some(steps) = self.steps.get(&workflow_id) {
            let completed_step_ids: Vec<StepId> = steps.values()
                .filter(|s| s.status == "Completed" || s.status == "Skipped")
                .map(|s| s.id)
                .collect();
            
            steps.values()
                .filter(|step| {
                    step.status == "Pending" &&
                    step.dependencies.iter().all(|dep| completed_step_ids.contains(dep))
                })
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }
    
    /// Handle workflow created event
    pub fn handle_workflow_created(&mut self, event: &WorkflowCreated) {
        let view = WorkflowView {
            id: event.workflow_id,
            name: event.name.clone(),
            description: event.description.clone(),
            status: WorkflowStatus::Draft,
            step_count: 0,
            created_by: event.created_by.clone(),
            created_at: Some(event.created_at),
            started_at: None,
            completed_at: None,
        };
        
        self.workflows.insert(event.workflow_id, view);
        self.steps.insert(event.workflow_id, HashMap::new());
    }
    
    /// Handle workflow started event
    pub fn handle_workflow_started(&mut self, event: &WorkflowStarted) {
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.status = WorkflowStatus::Running;
            workflow.started_at = Some(event.started_at);
        }
    }
    
    /// Handle workflow completed event
    pub fn handle_workflow_completed(&mut self, event: &WorkflowCompleted) {
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.status = WorkflowStatus::Completed;
            workflow.completed_at = Some(event.completed_at);
        }
    }
    
    /// Handle workflow failed event
    pub fn handle_workflow_failed(&mut self, event: &WorkflowFailed) {
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.status = WorkflowStatus::Failed;
        }
    }
    
    /// Handle workflow paused event
    pub fn handle_workflow_paused(&mut self, event: &WorkflowPaused) {
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.status = WorkflowStatus::Paused;
        }
    }
    
    /// Handle workflow resumed event
    pub fn handle_workflow_resumed(&mut self, event: &WorkflowResumed) {
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.status = WorkflowStatus::Running;
        }
    }
    
    /// Handle workflow cancelled event
    pub fn handle_workflow_cancelled(&mut self, event: &WorkflowCancelled) {
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.status = WorkflowStatus::Cancelled;
        }
    }
    
    /// Handle step added event
    pub fn handle_step_added(&mut self, event: &StepAdded) {
        let step_view = StepView {
            id: event.step_id,
            name: event.name.clone(),
            description: event.description.clone(),
            step_type: format!("{:?}", event.step_type),
            status: "Pending".to_string(),
            dependencies: event.dependencies.clone(),
            assigned_to: event.assigned_to.clone(),
            estimated_duration_minutes: event.estimated_duration_minutes,
        };
        
        if let Some(steps) = self.steps.get_mut(&event.workflow_id) {
            steps.insert(event.step_id, step_view);
        }
        
        // Update step count
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.step_count = self.steps.get(&event.workflow_id).map(|s| s.len()).unwrap_or(0);
        }
    }
    
    /// Handle step removed event
    pub fn handle_step_removed(&mut self, event: &StepRemoved) {
        if let Some(steps) = self.steps.get_mut(&event.workflow_id) {
            steps.remove(&event.step_id);
        }
        
        // Update step count
        if let Some(workflow) = self.workflows.get_mut(&event.workflow_id) {
            workflow.step_count = self.steps.get(&event.workflow_id).map(|s| s.len()).unwrap_or(0);
        }
    }
    
    /// Handle step started event
    pub fn handle_step_started(&mut self, event: &StepExecutionStarted) {
        if let Some(steps) = self.steps.get_mut(&event.workflow_id) {
            if let Some(step) = steps.get_mut(&event.step_id) {
                step.status = "Running".to_string();
            }
        }
    }
    
    /// Handle step completed event
    pub fn handle_step_completed(&mut self, event: &StepExecutionCompleted) {
        if let Some(steps) = self.steps.get_mut(&event.workflow_id) {
            if let Some(step) = steps.get_mut(&event.step_id) {
                step.status = "Completed".to_string();
            }
        }
    }
    
    /// Handle step failed event
    pub fn handle_step_failed(&mut self, event: &StepExecutionFailed) {
        if let Some(steps) = self.steps.get_mut(&event.workflow_id) {
            if let Some(step) = steps.get_mut(&event.step_id) {
                step.status = "Failed".to_string();
            }
        }
    }
    
    /// Handle step skipped event
    pub fn handle_step_skipped(&mut self, event: &StepSkipped) {
        if let Some(steps) = self.steps.get_mut(&event.workflow_id) {
            if let Some(step) = steps.get_mut(&event.step_id) {
                step.status = "Skipped".to_string();
            }
        }
    }
} 