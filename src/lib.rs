//! # CIM Domain Workflow
//!
//! This crate provides workflow domain functionality for the Composable Information Machine (CIM).
//! It handles business process workflows, workflow execution, and workflow composition.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use cim_domain::{DomainEvent, DomainResult, DomainError};

/// Unique identifier for a workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    /// Create a new workflow ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a workflow step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

impl StepId {
    /// Create a new step ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a state
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateId(pub String);

impl StateId {
    /// Create a new state ID
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

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

/// Trait for workflow states
pub trait WorkflowState: Clone + Send + Sync + std::fmt::Debug {
    /// Get the unique identifier for this state
    fn id(&self) -> StateId;
    
    /// Check if this is a terminal state
    fn is_terminal(&self) -> bool;
    
    /// Get the name of this state
    fn name(&self) -> &str;
}

/// Trait for transition inputs
pub trait TransitionInput: Clone + Send + Sync + std::fmt::Debug {
    /// Get the input type identifier
    fn input_type(&self) -> &str;
}

/// Trait for transition outputs
pub trait TransitionOutput: Clone + Send + Sync + std::fmt::Debug {
    /// Get the output type identifier
    fn output_type(&self) -> &str;
}

/// Workflow execution context
#[derive(Debug, Clone, Default)]
pub struct WorkflowContext {
    /// Context variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Current user or actor
    pub actor: Option<String>,
    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

/// Trait for workflow transitions
pub trait WorkflowTransition<S, I, O>: Send + Sync + std::fmt::Debug
where
    S: WorkflowState,
    I: TransitionInput,
    O: TransitionOutput,
{
    /// Get the source state
    fn source(&self) -> &S;
    
    /// Get the target state
    fn target(&self) -> &S;
    
    /// Check if this transition accepts the given input
    fn accepts_input(&self, input: &I) -> bool;
    
    /// Check if the guard condition is satisfied
    fn guard(&self, context: &WorkflowContext) -> bool;
    
    /// Execute the transition and produce output
    fn execute(&self, input: &I, context: &WorkflowContext) -> DomainResult<O>;
    
    /// Get the transition name
    fn name(&self) -> &str;
}

/// Simple implementation of WorkflowState
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleState {
    /// State identifier
    pub id: StateId,
    /// State name
    pub name: String,
    /// Whether this is a terminal state
    pub is_terminal: bool,
}

impl SimpleState {
    /// Create a new simple state
    pub fn new(name: &str) -> Self {
        Self {
            id: StateId::new(name.to_string()),
            name: name.to_string(),
            is_terminal: false,
        }
    }
    
    /// Create a new terminal state
    pub fn terminal(name: &str) -> Self {
        Self {
            id: StateId::new(name.to_string()),
            name: name.to_string(),
            is_terminal: true,
        }
    }
}

impl WorkflowState for SimpleState {
    fn id(&self) -> StateId {
        self.id.clone()
    }
    
    fn is_terminal(&self) -> bool {
        self.is_terminal
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Simple implementation of TransitionInput
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimpleInput {
    /// Input data
    pub data: HashMap<String, serde_json::Value>,
}

impl TransitionInput for SimpleInput {
    fn input_type(&self) -> &str {
        "simple"
    }
}

/// Simple implementation of TransitionOutput
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimpleOutput {
    /// Output data
    pub data: HashMap<String, serde_json::Value>,
}

impl TransitionOutput for SimpleOutput {
    fn output_type(&self) -> &str {
        "simple"
    }
}

/// Simple implementation of WorkflowTransition
#[derive(Debug, Clone)]
pub struct SimpleTransitionImpl {
    /// Transition name
    pub name: String,
    /// Source state
    pub source: SimpleState,
    /// Target state
    pub target: SimpleState,
    /// Expected input
    pub expected_input: SimpleInput,
    /// Produced output
    pub output: SimpleOutput,
}

impl SimpleTransitionImpl {
    /// Create a new simple transition
    pub fn new(
        name: &str,
        source: SimpleState,
        target: SimpleState,
        expected_input: SimpleInput,
        output: SimpleOutput,
    ) -> Self {
        Self {
            name: name.to_string(),
            source,
            target,
            expected_input,
            output,
        }
    }
}

impl WorkflowTransition<SimpleState, SimpleInput, SimpleOutput> for SimpleTransitionImpl {
    fn source(&self) -> &SimpleState {
        &self.source
    }
    
    fn target(&self) -> &SimpleState {
        &self.target
    }
    
    fn accepts_input(&self, _input: &SimpleInput) -> bool {
        // Simple implementation accepts any input
        true
    }
    
    fn guard(&self, _context: &WorkflowContext) -> bool {
        // Simple implementation has no guard conditions
        true
    }
    
    fn execute(&self, _input: &SimpleInput, _context: &WorkflowContext) -> DomainResult<SimpleOutput> {
        Ok(self.output.clone())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Represents a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique identifier for this step
    pub id: StepId,
    /// Human-readable name for this step
    pub name: String,
    /// Description of what this step does
    pub description: String,
    /// Type of step (e.g., "manual", "automated", "decision")
    pub step_type: String,
    /// Configuration data for this step
    pub config: HashMap<String, serde_json::Value>,
}

/// Represents a complete workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier for this workflow
    pub id: WorkflowId,
    /// Human-readable name for this workflow
    pub name: String,
    /// Description of what this workflow does
    pub description: String,
    /// Current status of the workflow
    pub status: WorkflowStatus,
    /// Steps in this workflow
    pub steps: Vec<WorkflowStep>,
    /// Metadata for this workflow
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Workflow {
    /// Create a new workflow
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: WorkflowId::new(),
            name,
            description,
            status: WorkflowStatus::Draft,
            steps: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a step to this workflow
    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
    }

    /// Get a step by ID
    pub fn get_step(&self, step_id: StepId) -> Option<&WorkflowStep> {
        self.steps.iter().find(|step| step.id == step_id)
    }

    /// Start the workflow execution
    pub fn start(&mut self) -> DomainResult<()> {
        if self.status != WorkflowStatus::Draft {
            return Err(DomainError::generic("Workflow can only be started from Draft status"));
        }
        self.status = WorkflowStatus::Running;
        Ok(())
    }

    /// Complete the workflow
    pub fn complete(&mut self) -> DomainResult<()> {
        if self.status != WorkflowStatus::Running {
            return Err(DomainError::generic("Workflow can only be completed from Running status"));
        }
        self.status = WorkflowStatus::Completed;
        Ok(())
    }
}

/// Workflow events for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// A new workflow was created
    WorkflowCreated {
        /// ID of the created workflow
        workflow_id: WorkflowId,
        /// Name of the workflow
        name: String,
        /// Description of the workflow
        description: String,
    },
    /// A step was added to a workflow
    StepAdded {
        /// ID of the workflow
        workflow_id: WorkflowId,
        /// The step that was added
        step: WorkflowStep,
    },
    /// A workflow was started
    WorkflowStarted {
        /// ID of the workflow
        workflow_id: WorkflowId,
    },
    /// A workflow was completed
    WorkflowCompleted {
        /// ID of the workflow
        workflow_id: WorkflowId,
    },
    /// A workflow failed
    WorkflowFailed {
        /// ID of the workflow
        workflow_id: WorkflowId,
        /// Error message
        error: String,
    },
}

/// Workflow composition utilities for cim-compose
pub mod composition {
    use super::*;

    /// Create a simple linear workflow
    pub fn create_linear_workflow(
        name: String,
        description: String,
        step_names: Vec<String>,
    ) -> Workflow {
        let mut workflow = Workflow::new(name, description);

        for step_name in step_names {
            let step = WorkflowStep {
                id: StepId::new(),
                name: step_name.clone(),
                description: format!("Step: {}", step_name),
                step_type: "manual".to_string(),
                config: HashMap::new(),
            };
            workflow.add_step(step);
        }

        workflow
    }

    /// Create a workflow with decision points
    pub fn create_decision_workflow(
        name: String,
        description: String,
    ) -> Workflow {
        let mut workflow = Workflow::new(name, description);

        // Add a decision step
        let decision_step = WorkflowStep {
            id: StepId::new(),
            name: "Decision Point".to_string(),
            description: "A decision point in the workflow".to_string(),
            step_type: "decision".to_string(),
            config: HashMap::new(),
        };
        workflow.add_step(decision_step);

        workflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.description, "A test workflow");
        assert_eq!(workflow.status, WorkflowStatus::Draft);
        assert!(workflow.steps.is_empty());
    }

    #[test]
    fn test_workflow_lifecycle() {
        let mut workflow = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        // Start workflow
        assert!(workflow.start().is_ok());
        assert_eq!(workflow.status, WorkflowStatus::Running);

        // Complete workflow
        assert!(workflow.complete().is_ok());
        assert_eq!(workflow.status, WorkflowStatus::Completed);
    }

    #[test]
    fn test_add_step() {
        let mut workflow = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let step = WorkflowStep {
            id: StepId::new(),
            name: "Test Step".to_string(),
            description: "A test step".to_string(),
            step_type: "manual".to_string(),
            config: HashMap::new(),
        };

        let step_id = step.id;
        workflow.add_step(step);

        assert_eq!(workflow.steps.len(), 1);
        assert!(workflow.get_step(step_id).is_some());
    }

    #[test]
    fn test_linear_workflow_composition() {
        let workflow = composition::create_linear_workflow(
            "Linear Test".to_string(),
            "A linear test workflow".to_string(),
            vec!["Step 1".to_string(), "Step 2".to_string(), "Step 3".to_string()],
        );

        assert_eq!(workflow.steps.len(), 3);
        assert_eq!(workflow.steps[0].name, "Step 1");
        assert_eq!(workflow.steps[1].name, "Step 2");
        assert_eq!(workflow.steps[2].name, "Step 3");
    }
} 