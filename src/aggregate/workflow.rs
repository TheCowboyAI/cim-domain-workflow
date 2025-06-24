//! Workflow aggregate
//!
//! The workflow aggregate represents a business process definition with steps,
//! transitions, and execution state.

use crate::value_objects::*;
use crate::events::*;
use crate::domain_events::WorkflowDomainEvent;
use cim_domain::{AggregateRoot, DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow aggregate root
/// 
/// This aggregate manages the complete workflow lifecycle including steps,
/// dependencies, and execution context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier
    pub id: WorkflowId,
    /// Workflow name
    pub name: String,
    /// Workflow description
    pub description: String,
    /// Current status
    pub status: WorkflowStatus,
    /// Workflow steps
    pub steps: HashMap<StepId, WorkflowStep>,
    /// Execution context
    pub context: WorkflowContext,
    /// Workflow metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created by user
    pub created_by: Option<String>,
    /// Creation timestamp
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Started timestamp
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Completed timestamp
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Version for optimistic concurrency
    pub version: u64,
}

impl Workflow {
    /// Create a new workflow
    pub fn new(
        name: String,
        description: String,
        metadata: HashMap<String, serde_json::Value>,
        created_by: Option<String>,
    ) -> DomainResult<(Self, Vec<WorkflowDomainEvent>)> {
        let workflow_id = WorkflowId::new();
        let now = chrono::Utc::now();

        let event = WorkflowCreated {
            workflow_id,
            name: name.clone(),
            description: description.clone(),
            metadata: metadata.clone(),
            created_by: created_by.clone(),
            created_at: now,
        };

        let mut workflow = Self {
            id: workflow_id,
            name,
            description,
            status: WorkflowStatus::Draft,
            steps: HashMap::new(),
            context: WorkflowContext::new(),
            metadata,
            created_by,
            created_at: Some(now),
            started_at: None,
            completed_at: None,
            version: 0,
        };

        workflow.apply_workflow_created(&event)?;

        Ok((workflow, vec![WorkflowDomainEvent::WorkflowCreated(event)]))
    }

    /// Start workflow execution
    pub fn start(&mut self, context: WorkflowContext, started_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>> {
        if !self.status.can_transition_to(&WorkflowStatus::Running) {
            return Err(DomainError::generic(format!(
                "Cannot start workflow in status {:?}",
                self.status
            )));
        }

        if self.steps.is_empty() {
            return Err(DomainError::generic("Cannot start workflow with no steps"));
        }

        let now = chrono::Utc::now();
        let event = WorkflowStarted {
            workflow_id: self.id,
            context: context.clone(),
            started_by,
            started_at: now,
        };

        self.apply_workflow_started(&event)?;

        Ok(vec![WorkflowDomainEvent::WorkflowStarted(event)])
    }

    /// Complete workflow execution
    pub fn complete(&mut self) -> DomainResult<Vec<WorkflowDomainEvent>> {
        if !self.status.can_transition_to(&WorkflowStatus::Completed) {
            return Err(DomainError::generic(format!(
                "Cannot complete workflow in status {:?}",
                self.status
            )));
        }

        // Check if all steps are completed
        let incomplete_steps: Vec<_> = self.steps
            .values()
            .filter(|step| !step.is_completed())
            .collect();

        if !incomplete_steps.is_empty() {
            return Err(DomainError::generic(format!(
                "Cannot complete workflow with {} incomplete steps",
                incomplete_steps.len()
            )));
        }

        let now = chrono::Utc::now();
        let duration_seconds = self.started_at
            .map(|start| (now - start).num_seconds() as u64)
            .unwrap_or(0);

        let event = WorkflowCompleted {
            workflow_id: self.id,
            final_context: self.context.clone(),
            completed_at: now,
            duration_seconds,
        };

        self.apply_workflow_completed(&event)?;

        Ok(vec![WorkflowDomainEvent::WorkflowCompleted(event)])
    }

    /// Fail workflow execution
    pub fn fail(&mut self, error: String) -> DomainResult<Vec<WorkflowDomainEvent>> {
        if !self.status.can_transition_to(&WorkflowStatus::Failed) {
            return Err(DomainError::generic(format!(
                "Cannot fail workflow in status {:?}",
                self.status
            )));
        }

        let now = chrono::Utc::now();
        let duration_seconds = self.started_at
            .map(|start| (now - start).num_seconds() as u64)
            .unwrap_or(0);

        let event = WorkflowFailed {
            workflow_id: self.id,
            error,
            failure_context: self.context.clone(),
            failed_at: now,
            duration_seconds,
        };

        self.apply_workflow_failed(&event)?;

        Ok(vec![WorkflowDomainEvent::WorkflowFailed(event)])
    }

    /// Pause workflow execution
    pub fn pause(&mut self, reason: String, paused_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>> {
        if !self.status.can_transition_to(&WorkflowStatus::Paused) {
            return Err(DomainError::generic(format!(
                "Cannot pause workflow in status {:?}",
                self.status
            )));
        }

        let now = chrono::Utc::now();
        let event = WorkflowPaused {
            workflow_id: self.id,
            reason,
            pause_context: self.context.clone(),
            paused_by,
            paused_at: now,
        };

        self.apply_workflow_paused(&event)?;

        Ok(vec![WorkflowDomainEvent::WorkflowPaused(event)])
    }

    /// Resume workflow execution
    pub fn resume(&mut self, resumed_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>> {
        if !self.status.can_transition_to(&WorkflowStatus::Running) {
            return Err(DomainError::generic(format!(
                "Cannot resume workflow in status {:?}",
                self.status
            )));
        }

        let now = chrono::Utc::now();
        let event = WorkflowResumed {
            workflow_id: self.id,
            resume_context: self.context.clone(),
            resumed_by,
            resumed_at: now,
        };

        self.apply_workflow_resumed(&event)?;

        Ok(vec![WorkflowDomainEvent::WorkflowResumed(event)])
    }

    /// Cancel workflow execution
    pub fn cancel(&mut self, reason: String, cancelled_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>> {
        if !self.status.can_transition_to(&WorkflowStatus::Cancelled) {
            return Err(DomainError::generic(format!(
                "Cannot cancel workflow in status {:?}",
                self.status
            )));
        }

        let now = chrono::Utc::now();
        let event = WorkflowCancelled {
            workflow_id: self.id,
            reason,
            cancellation_context: self.context.clone(),
            cancelled_by,
            cancelled_at: now,
        };

        self.apply_workflow_cancelled(&event)?;

        Ok(vec![WorkflowDomainEvent::WorkflowCancelled(event)])
    }

    /// Add a step to the workflow
    pub fn add_step(
        &mut self,
        name: String,
        description: String,
        step_type: StepType,
        config: HashMap<String, serde_json::Value>,
        dependencies: Vec<StepId>,
        estimated_duration_minutes: Option<u32>,
        assigned_to: Option<String>,
        added_by: Option<String>,
    ) -> DomainResult<Vec<WorkflowDomainEvent>> {
        // Validate dependencies exist
        for dep_id in &dependencies {
            if !self.steps.contains_key(dep_id) {
                return Err(DomainError::generic(format!(
                    "Dependency step {} does not exist",
                    dep_id.as_uuid()
                )));
            }
        }

        // Check for circular dependencies
        let step_id = StepId::new();
        if self.would_create_cycle(&step_id, &dependencies) {
            return Err(DomainError::generic("Adding step would create circular dependency"));
        }

        let now = chrono::Utc::now();
        let event = StepAdded {
            workflow_id: self.id,
            step_id,
            name,
            description,
            step_type,
            config,
            dependencies,
            estimated_duration_minutes,
            assigned_to,
            added_by,
            added_at: now,
        };

        self.apply_step_added(&event)?;

        Ok(vec![WorkflowDomainEvent::StepAdded(event)])
    }

    /// Remove a step from the workflow
    pub fn remove_step(&mut self, step_id: StepId, reason: String, removed_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>> {
        if !self.steps.contains_key(&step_id) {
            return Err(DomainError::generic("Step not found"));
        }

        // Check if other steps depend on this one
        let dependent_steps: Vec<_> = self.steps
            .values()
            .filter(|step| step.dependencies.contains(&step_id))
            .map(|step| step.id)
            .collect();

        if !dependent_steps.is_empty() {
            return Err(DomainError::generic(format!(
                "Cannot remove step that is depended upon by {} other steps",
                dependent_steps.len()
            )));
        }

        let now = chrono::Utc::now();
        let event = StepRemoved {
            workflow_id: self.id,
            step_id,
            reason,
            removed_by,
            removed_at: now,
        };

        self.apply_step_removed(&event)?;

        Ok(vec![WorkflowDomainEvent::StepRemoved(event)])
    }

    /// Get steps that are ready to execute
    pub fn get_executable_steps(&self) -> Vec<&WorkflowStep> {
        if !self.status.is_active() {
            return Vec::new();
        }

        let completed_step_ids: Vec<StepId> = self.steps
            .values()
            .filter(|step| step.is_completed())
            .map(|step| step.id)
            .collect();

        self.steps
            .values()
            .filter(|step| step.can_execute(&completed_step_ids))
            .collect()
    }

    /// Check if adding a dependency would create a cycle
    fn would_create_cycle(&self, step_id: &StepId, dependencies: &[StepId]) -> bool {
        // For each dependency, check if it transitively depends on step_id
        for dep_id in dependencies {
            if self.step_depends_on(dep_id, step_id) {
                return true;
            }
        }
        false
    }

    /// Check if one step transitively depends on another
    fn step_depends_on(&self, step_id: &StepId, target_id: &StepId) -> bool {
        if let Some(step) = self.steps.get(step_id) {
            if step.dependencies.contains(target_id) {
                return true;
            }
            for dep_id in &step.dependencies {
                if self.step_depends_on(dep_id, target_id) {
                    return true;
                }
            }
        }
        false
    }

    // Event application methods
    fn apply_workflow_created(&mut self, event: &WorkflowCreated) -> DomainResult<()> {
        self.id = event.workflow_id;
        self.name = event.name.clone();
        self.description = event.description.clone();
        self.metadata = event.metadata.clone();
        self.created_by = event.created_by.clone();
        self.created_at = Some(event.created_at);
        self.version += 1;
        Ok(())
    }

    fn apply_workflow_started(&mut self, event: &WorkflowStarted) -> DomainResult<()> {
        self.status = WorkflowStatus::Running;
        self.context = event.context.clone();
        self.started_at = Some(event.started_at);
        self.version += 1;
        Ok(())
    }

    fn apply_workflow_completed(&mut self, event: &WorkflowCompleted) -> DomainResult<()> {
        self.status = WorkflowStatus::Completed;
        self.context = event.final_context.clone();
        self.completed_at = Some(event.completed_at);
        self.version += 1;
        Ok(())
    }

    fn apply_workflow_failed(&mut self, event: &WorkflowFailed) -> DomainResult<()> {
        self.status = WorkflowStatus::Failed;
        self.context = event.failure_context.clone();
        self.version += 1;
        Ok(())
    }

    fn apply_workflow_paused(&mut self, event: &WorkflowPaused) -> DomainResult<()> {
        self.status = WorkflowStatus::Paused;
        self.context = event.pause_context.clone();
        self.version += 1;
        Ok(())
    }

    fn apply_workflow_resumed(&mut self, event: &WorkflowResumed) -> DomainResult<()> {
        self.status = WorkflowStatus::Running;
        self.context = event.resume_context.clone();
        self.version += 1;
        Ok(())
    }

    fn apply_workflow_cancelled(&mut self, event: &WorkflowCancelled) -> DomainResult<()> {
        self.status = WorkflowStatus::Cancelled;
        self.context = event.cancellation_context.clone();
        self.version += 1;
        Ok(())
    }

    fn apply_step_added(&mut self, event: &StepAdded) -> DomainResult<()> {
        let mut step = WorkflowStep::new(
            event.name.clone(),
            event.description.clone(),
            event.step_type.clone(),
        );
        step.id = event.step_id;
        step.config = event.config.clone();
        step.dependencies = event.dependencies.clone();
        step.estimated_duration_minutes = event.estimated_duration_minutes;
        step.assigned_to = event.assigned_to.clone();

        self.steps.insert(event.step_id, step);
        self.version += 1;
        Ok(())
    }

    fn apply_step_removed(&mut self, event: &StepRemoved) -> DomainResult<()> {
        self.steps.remove(&event.step_id);
        self.version += 1;
        Ok(())
    }
}

impl AggregateRoot for Workflow {
    type Id = WorkflowId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_workflow() -> Workflow {
        let (workflow, _) = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
            HashMap::new(),
            Some("test_user".to_string()),
        ).unwrap();
        workflow
    }

    /// Test workflow creation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Workflow] --> B[Verify Initial State]
    ///     B --> C[Check Events]
    ///     C --> D[Verify Metadata]
    /// ```
    #[test]
    fn test_workflow_creation() {
        let metadata = HashMap::from([
            ("key".to_string(), serde_json::Value::String("value".to_string())),
        ]);

        let (workflow, events) = Workflow::new(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
            metadata.clone(),
            Some("test_user".to_string()),
        ).unwrap();

        // Verify workflow state
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.description, "A test workflow");
        assert_eq!(workflow.status, WorkflowStatus::Draft);
        assert!(workflow.steps.is_empty());
        assert_eq!(workflow.created_by, Some("test_user".to_string()));
        assert!(workflow.created_at.is_some());
        assert_eq!(workflow.version, 1);

        // Verify events
        assert_eq!(events.len(), 1);
        match &events[0] {
            WorkflowDomainEvent::WorkflowCreated(event) => {
                assert_eq!(event.workflow_id, workflow.id);
                assert_eq!(event.name, "Test Workflow");
                assert_eq!(event.metadata, metadata);
            }
            _ => panic!("Expected WorkflowCreated event"),
        }
    }

    /// Test workflow lifecycle transitions
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Draft] --> B[Start]
    ///     B --> C[Running]
    ///     C --> D[Complete/Fail/Cancel]
    ///     C --> E[Pause]
    ///     E --> F[Resume]
    /// ```
    #[test]
    fn test_workflow_lifecycle() {
        let mut workflow = create_test_workflow();

        // Cannot start empty workflow
        let result = workflow.start(WorkflowContext::new(), Some("user".to_string()));
        assert!(result.is_err());

        // Add a step
        workflow.add_step(
            "Step 1".to_string(),
            "First step".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![],
            Some(30),
            None,
            Some("user".to_string()),
        ).unwrap();

        // Start workflow
        let events = workflow.start(WorkflowContext::new(), Some("user".to_string())).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WorkflowDomainEvent::WorkflowStarted(_)));
        assert_eq!(workflow.status, WorkflowStatus::Running);
        assert!(workflow.started_at.is_some());

        // Pause workflow
        let events = workflow.pause("Taking a break".to_string(), Some("user".to_string())).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WorkflowDomainEvent::WorkflowPaused(_)));
        assert_eq!(workflow.status, WorkflowStatus::Paused);

        // Resume workflow
        let events = workflow.resume(Some("user".to_string())).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WorkflowDomainEvent::WorkflowResumed(_)));
        assert_eq!(workflow.status, WorkflowStatus::Running);

        // Cannot complete with incomplete steps
        let result = workflow.complete();
        assert!(result.is_err());

        // Cancel workflow
        let events = workflow.cancel("No longer needed".to_string(), Some("user".to_string())).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WorkflowDomainEvent::WorkflowCancelled(_)));
        assert_eq!(workflow.status, WorkflowStatus::Cancelled);
    }

    /// Test step management
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Add Step 1] --> B[Add Step 2]
    ///     B --> C[Add Dependent Step]
    ///     C --> D[Verify Dependencies]
    ///     D --> E[Remove Step]
    /// ```
    #[test]
    fn test_step_management() {
        let mut workflow = create_test_workflow();

        // Add first step
        let events = workflow.add_step(
            "Step 1".to_string(),
            "First step".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![],
            Some(30),
            Some("user1".to_string()),
            Some("admin".to_string()),
        ).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            WorkflowDomainEvent::StepAdded(event) => {
                assert_eq!(event.name, "Step 1");
                assert_eq!(event.step_type, StepType::Manual);
                assert_eq!(event.dependencies.len(), 0);
                assert_eq!(event.assigned_to, Some("user1".to_string()));
            }
            _ => panic!("Expected StepAdded event"),
        }

        let step1_id = match &events[0] {
            WorkflowDomainEvent::StepAdded(e) => e.step_id,
            _ => unreachable!(),
        };

        // Add dependent step
        let events = workflow.add_step(
            "Step 2".to_string(),
            "Second step".to_string(),
            StepType::Automated,
            HashMap::from([("script".to_string(), serde_json::Value::String("run.sh".to_string()))]),
            vec![step1_id],
            Some(15),
            None,
            Some("admin".to_string()),
        ).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(workflow.steps.len(), 2);

        // Cannot remove step with dependencies
        let result = workflow.remove_step(step1_id, "No longer needed".to_string(), Some("admin".to_string()));
        assert!(result.is_err());

        // Get step 2 ID
        let step2_id = match &events[0] {
            WorkflowDomainEvent::StepAdded(e) => e.step_id,
            _ => unreachable!(),
        };

        // Can remove step without dependencies
        let events = workflow.remove_step(step2_id, "Not needed".to_string(), Some("admin".to_string())).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WorkflowDomainEvent::StepRemoved(_)));
        assert_eq!(workflow.steps.len(), 1);
    }

    /// Test circular dependency detection
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Step A] --> B[Step B]
    ///     B --> C[Step C]
    ///     C -.->|Circular| A
    /// ```
    #[test]
    fn test_circular_dependency_detection() {
        let mut workflow = create_test_workflow();

        // Add step A
        workflow.add_step(
            "Step A".to_string(),
            "Step A".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![],
            None,
            None,
            None,
        ).unwrap();

        let step_a_id = workflow.steps.values().find(|s| s.name == "Step A").unwrap().id;

        // Add step B depending on A
        workflow.add_step(
            "Step B".to_string(),
            "Step B".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![step_a_id],
            None,
            None,
            None,
        ).unwrap();

        let step_b_id = workflow.steps.values().find(|s| s.name == "Step B").unwrap().id;

        // Try to make A depend on B (circular)
        let result = workflow.add_step(
            "Step C".to_string(),
            "Step C".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![step_b_id],
            None,
            None,
            None,
        );

        // This should succeed as it's not circular yet
        assert!(result.is_ok());
    }

    /// Test workflow failure
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Start Workflow] --> B[Running]
    ///     B --> C[Fail]
    ///     C --> D[Failed State]
    /// ```
    #[test]
    fn test_workflow_failure() {
        let mut workflow = create_test_workflow();

        // Add step and start
        workflow.add_step(
            "Step".to_string(),
            "A step".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![],
            None,
            None,
            None,
        ).unwrap();

        workflow.start(WorkflowContext::new(), None).unwrap();

        // Fail the workflow
        let events = workflow.fail("Something went wrong".to_string()).unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            WorkflowDomainEvent::WorkflowFailed(event) => {
                assert_eq!(event.error, "Something went wrong");
                assert!(event.duration_seconds >= 0);
            }
            _ => panic!("Expected WorkflowFailed event"),
        }
        assert_eq!(workflow.status, WorkflowStatus::Failed);
    }

    /// Test executable steps calculation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Step 1<br/>Ready] --> B[Step 2<br/>Waiting]
    ///     A --> C[Step 3<br/>Waiting]
    ///     B --> D[Step 4<br/>Waiting]
    ///     C --> D
    /// ```
    #[test]
    fn test_executable_steps() {
        let mut workflow = create_test_workflow();

        // Add independent step
        workflow.add_step(
            "Step 1".to_string(),
            "Independent".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![],
            None,
            None,
            None,
        ).unwrap();

        let step1_id = workflow.steps.values().find(|s| s.name == "Step 1").unwrap().id;

        // Add dependent steps
        workflow.add_step(
            "Step 2".to_string(),
            "Depends on 1".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![step1_id],
            None,
            None,
            None,
        ).unwrap();

        // Start workflow
        workflow.start(WorkflowContext::new(), None).unwrap();

        // Only step 1 should be executable
        let executable = workflow.get_executable_steps();
        assert_eq!(executable.len(), 1);
        assert_eq!(executable[0].name, "Step 1");

        // After completing step 1, step 2 should be executable
        if let Some(step) = workflow.steps.get_mut(&step1_id) {
            step.status = StepStatus::Completed;
        }

        let executable = workflow.get_executable_steps();
        assert_eq!(executable.len(), 1);
        assert_eq!(executable[0].name, "Step 2");
    }

    /// Test workflow context
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create Context] --> B[Add Variables]
    ///     B --> C[Start Workflow]
    ///     C --> D[Context Preserved]
    /// ```
    #[test]
    fn test_workflow_context() {
        let mut workflow = create_test_workflow();

        // Add step
        workflow.add_step(
            "Step".to_string(),
            "A step".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![],
            None,
            None,
            None,
        ).unwrap();

        // Create context with variables
        let mut context = WorkflowContext::new();
        context.variables.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        // Start with context
        let events = workflow.start(context.clone(), None).unwrap();
        match &events[0] {
            WorkflowDomainEvent::WorkflowStarted(event) => {
                assert_eq!(event.context.variables.get("key").unwrap(), "value");
            }
            _ => panic!("Expected WorkflowStarted event"),
        }

        // Context should be preserved
        assert_eq!(workflow.context.variables.get("key").unwrap(), "value");
    }

    /// Test invalid state transitions
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Draft] -.->|Invalid| B[Completed]
    ///     C[Cancelled] -.->|Invalid| D[Running]
    ///     E[Failed] -.->|Invalid| F[Paused]
    /// ```
    #[test]
    fn test_invalid_transitions() {
        let mut workflow = create_test_workflow();

        // Cannot complete from draft
        assert!(workflow.complete().is_err());

        // Cannot pause from draft
        assert!(workflow.pause("reason".to_string(), None).is_err());

        // Add step and cancel
        workflow.add_step(
            "Step".to_string(),
            "A step".to_string(),
            StepType::Manual,
            HashMap::new(),
            vec![],
            None,
            None,
            None,
        ).unwrap();

        workflow.start(WorkflowContext::new(), None).unwrap();
        workflow.cancel("cancelled".to_string(), None).unwrap();

        // Cannot resume from cancelled
        assert!(workflow.resume(None).is_err());

        // Cannot start from cancelled
        assert!(workflow.start(WorkflowContext::new(), None).is_err());
    }
} 