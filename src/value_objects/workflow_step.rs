//! Workflow step value object

use crate::value_objects::{StepId, StepStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use cim_domain::{DomainError, DomainResult};

/// Type of workflow step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    /// Manual step requiring human intervention
    Manual,
    /// Automated step that executes programmatically
    Automated,
    /// Decision point with multiple possible outcomes
    Decision,
    /// Approval step requiring authorization
    Approval,
    /// Integration with external system
    Integration,
    /// Parallel execution of multiple sub-steps
    Parallel,
    /// Custom step type
    Custom(String),
}

impl StepType {
    /// Check if this step type requires human intervention
    pub fn requires_human_intervention(&self) -> bool {
        matches!(self, StepType::Manual | StepType::Approval)
    }

    /// Check if this step type can execute automatically
    pub fn can_auto_execute(&self) -> bool {
        matches!(self, StepType::Automated | StepType::Integration)
    }
}

// StepStatus is now imported from step_status.rs

/// Represents a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique identifier for this step
    pub id: StepId,
    /// Human-readable name for this step
    pub name: String,
    /// Description of what this step does
    pub description: String,
    /// Type of step
    pub step_type: StepType,
    /// Current status of this step
    pub status: StepStatus,
    /// Configuration data for this step
    pub config: HashMap<String, serde_json::Value>,
    /// Dependencies on other steps
    pub dependencies: Vec<StepId>,
    /// Estimated duration in minutes
    pub estimated_duration_minutes: Option<u32>,
    /// Assigned user or role
    pub assigned_to: Option<String>,
}

impl WorkflowStep {
    /// Create a new workflow step
    pub fn new(
        name: String,
        description: String,
        step_type: StepType,
    ) -> Self {
        Self {
            id: StepId::new(),
            name,
            description,
            step_type,
            status: StepStatus::Pending,
            config: HashMap::new(),
            dependencies: Vec::new(),
            estimated_duration_minutes: None,
            assigned_to: None,
        }
    }

    /// Create a manual step
    pub fn manual(name: String, description: String) -> Self {
        Self::new(name, description, StepType::Manual)
    }

    /// Create an automated step
    pub fn automated(name: String, description: String) -> Self {
        Self::new(name, description, StepType::Automated)
    }

    /// Create a decision step
    pub fn decision(name: String, description: String) -> Self {
        Self::new(name, description, StepType::Decision)
    }

    /// Create an approval step
    pub fn approval(name: String, description: String) -> Self {
        Self::new(name, description, StepType::Approval)
    }

    /// Add a dependency on another step
    pub fn add_dependency(&mut self, step_id: StepId) {
        if !self.dependencies.contains(&step_id) {
            self.dependencies.push(step_id);
        }
    }

    /// Set estimated duration
    pub fn with_duration(mut self, minutes: u32) -> Self {
        self.estimated_duration_minutes = Some(minutes);
        self
    }

    /// Assign step to a user or role
    pub fn assign_to(mut self, assignee: String) -> Self {
        self.assigned_to = Some(assignee);
        self
    }

    /// Add configuration value
    pub fn with_config(mut self, key: String, value: serde_json::Value) -> Self {
        self.config.insert(key, value);
        self
    }

    /// Check if step can be executed (all dependencies met)
    pub fn can_execute(&self, completed_steps: &[StepId]) -> bool {
        self.status == StepStatus::Pending &&
        self.dependencies.iter().all(|dep| completed_steps.contains(dep))
    }

    /// Start step execution
    pub fn start_execution(&mut self) -> DomainResult<()> {
        if self.status != StepStatus::Pending {
            return Err(DomainError::generic(format!(
                "Step {} cannot be started from status {:?}",
                self.name, self.status
            )));
        }
        self.status = StepStatus::Running;
        Ok(())
    }

    /// Complete step execution
    pub fn complete(&mut self) -> DomainResult<()> {
        match self.status {
            StepStatus::Running | StepStatus::WaitingApproval => {
                self.status = StepStatus::Completed;
                Ok(())
            }
            _ => Err(DomainError::generic(format!(
                "Step {} cannot be completed from status {:?}",
                self.name, self.status
            )))
        }
    }

    /// Fail step execution
    pub fn fail(&mut self, reason: String) -> DomainResult<()> {
        if self.status == StepStatus::Running {
            self.status = StepStatus::Failed;
            self.config.insert("failure_reason".to_string(), serde_json::Value::String(reason));
            Ok(())
        } else {
            Err(DomainError::generic(format!(
                "Step {} cannot be failed from status {:?}",
                self.name, self.status
            )))
        }
    }

    /// Skip step execution
    pub fn skip(&mut self, reason: String) -> DomainResult<()> {
        if self.status == StepStatus::Pending {
            self.status = StepStatus::Skipped;
            self.config.insert("skip_reason".to_string(), serde_json::Value::String(reason));
            Ok(())
        } else {
            Err(DomainError::generic(format!(
                "Step {} cannot be skipped from status {:?}",
                self.name, self.status
            )))
        }
    }

    /// Check if step is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status, StepStatus::Completed | StepStatus::Skipped)
    }

    /// Check if step is terminal (completed, failed, or skipped)
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, StepStatus::Completed | StepStatus::Failed | StepStatus::Skipped)
    }
} 