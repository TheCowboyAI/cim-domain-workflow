//! Workflow state machine implementation
//!
//! Implements a formal state machine for workflow lifecycle management with
//! explicit transitions, guards, and effects.

use crate::{
    value_objects::{WorkflowId, WorkflowStatus, WorkflowContext},
    domain_events::WorkflowDomainEvent,
};
use cim_domain::{DomainError, DomainResult};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Workflow state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowTransition {
    /// Start the workflow
    Start,
    /// Complete the workflow successfully
    Complete,
    /// Mark the workflow as failed
    Fail { reason: String },
    /// Pause the workflow
    Pause { reason: String },
    /// Resume a paused workflow
    Resume,
    /// Cancel the workflow
    Cancel { reason: String },
}

// Custom PartialEq that only compares the variant type, not the values
impl PartialEq for WorkflowTransition {
    fn eq(&self, other: &Self) -> bool {
        use WorkflowTransition::*;
        match (self, other) {
            (Start, Start) => true,
            (Complete, Complete) => true,
            (Fail { .. }, Fail { .. }) => true,
            (Pause { .. }, Pause { .. }) => true,
            (Resume, Resume) => true,
            (Cancel { .. }, Cancel { .. }) => true,
            _ => false,
        }
    }
}

impl Eq for WorkflowTransition {}

// Custom Hash that only hashes the variant type
impl std::hash::Hash for WorkflowTransition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use WorkflowTransition::*;
        match self {
            Start => 0.hash(state),
            Complete => 1.hash(state),
            Fail { .. } => 2.hash(state),
            Pause { .. } => 3.hash(state),
            Resume => 4.hash(state),
            Cancel { .. } => 5.hash(state),
        }
    }
}

/// Guard condition for state transitions
pub type TransitionGuard = Box<dyn Fn(&WorkflowContext) -> DomainResult<()> + Send + Sync>;

/// Effect to execute after successful transition
pub type TransitionEffect = Box<dyn Fn(&mut WorkflowContext) -> Vec<WorkflowDomainEvent> + Send + Sync>;

/// Workflow state machine
pub struct WorkflowStateMachine {
    workflow_id: WorkflowId,
    current_state: WorkflowStatus,
    transition_table: HashMap<(WorkflowStatus, WorkflowTransition), TransitionConfig>,
}

/// Configuration for a state transition
struct TransitionConfig {
    target_state: WorkflowStatus,
    guards: Vec<TransitionGuard>,
    effects: Vec<TransitionEffect>,
}

impl WorkflowStateMachine {
    /// Create a new workflow state machine
    pub fn new(workflow_id: WorkflowId) -> Self {
        let mut state_machine = Self {
            workflow_id,
            current_state: WorkflowStatus::Draft,
            transition_table: HashMap::new(),
        };
        
        state_machine.configure_transitions();
        state_machine
    }

    /// Configure all valid state transitions
    fn configure_transitions(&mut self) {
        use WorkflowStatus::*;
        use WorkflowTransition::*;

        // Draft → Running
        self.add_transition(
            Draft,
            Start,
            Running,
            vec![
                // Guard: Must have at least one step
                Box::new(|ctx| {
                    if ctx.variables.is_empty() {
                        Err(DomainError::generic("Workflow must have context to start"))
                    } else {
                        Ok(())
                    }
                }),
            ],
            vec![
                // Effect: Initialize workflow metrics
                Box::new(|ctx| {
                    ctx.set_variable("started_at".to_string(), serde_json::json!(chrono::Utc::now()));
                    vec![]
                }),
            ],
        );

        // Draft → Cancelled
        self.add_transition(
            Draft,
            Cancel { reason: String::new() },
            Cancelled,
            vec![],
            vec![],
        );

        // Running → Completed
        self.add_transition(
            Running,
            Complete,
            Completed,
            vec![
                // Guard: All required steps must be completed
                Box::new(|_ctx| {
                    // This would check if all non-optional steps are completed
                    // For now, we'll allow completion
                    Ok(())
                }),
            ],
            vec![
                // Effect: Calculate final metrics
                Box::new(|ctx| {
                    ctx.set_variable("completed_at".to_string(), serde_json::json!(chrono::Utc::now()));
                    vec![]
                }),
            ],
        );

        // Running → Failed
        self.add_transition(
            Running,
            Fail { reason: String::new() },
            Failed,
            vec![],
            vec![
                // Effect: Record failure details
                Box::new(|ctx| {
                    ctx.set_variable("failed_at".to_string(), serde_json::json!(chrono::Utc::now()));
                    vec![]
                }),
            ],
        );

        // Running → Paused
        self.add_transition(
            Running,
            Pause { reason: String::new() },
            Paused,
            vec![],
            vec![
                // Effect: Save pause state
                Box::new(|ctx| {
                    ctx.set_variable("paused_at".to_string(), serde_json::json!(chrono::Utc::now()));
                    vec![]
                }),
            ],
        );

        // Running → Cancelled
        self.add_transition(
            Running,
            Cancel { reason: String::new() },
            Cancelled,
            vec![],
            vec![],
        );

        // Paused → Running
        self.add_transition(
            Paused,
            Resume,
            Running,
            vec![],
            vec![
                // Effect: Record resume time
                Box::new(|ctx| {
                    ctx.set_variable("resumed_at".to_string(), serde_json::json!(chrono::Utc::now()));
                    vec![]
                }),
            ],
        );

        // Paused → Cancelled
        self.add_transition(
            Paused,
            Cancel { reason: String::new() },
            Cancelled,
            vec![],
            vec![],
        );

        // Paused → Failed
        self.add_transition(
            Paused,
            Fail { reason: String::new() },
            Failed,
            vec![],
            vec![],
        );
    }

    /// Add a transition to the state machine
    fn add_transition(
        &mut self,
        from_state: WorkflowStatus,
        transition: WorkflowTransition,
        to_state: WorkflowStatus,
        guards: Vec<TransitionGuard>,
        effects: Vec<TransitionEffect>,
    ) {
        self.transition_table.insert(
            (from_state, transition),
            TransitionConfig {
                target_state: to_state,
                guards,
                effects,
            },
        );
    }

    /// Get the current state
    pub fn current_state(&self) -> &WorkflowStatus {
        &self.current_state
    }

    /// Set the current state (for restoration from events)
    pub fn set_state(&mut self, state: WorkflowStatus) {
        self.current_state = state;
    }

    /// Check if a transition is valid from the current state
    pub fn can_transition(&self, transition: &WorkflowTransition) -> bool {
        self.transition_table.contains_key(&(self.current_state.clone(), transition.clone()))
    }

    /// Execute a state transition
    pub fn transition(
        &mut self,
        transition: WorkflowTransition,
        context: &mut WorkflowContext,
    ) -> DomainResult<(WorkflowStatus, Vec<WorkflowDomainEvent>)> {
        let key = (self.current_state.clone(), transition.clone());
        
        let config = self.transition_table.get(&key)
            .ok_or_else(|| DomainError::generic(format!(
                "Invalid transition {:?} from state {:?}",
                transition, self.current_state
            )))?;

        // Check all guards
        for guard in &config.guards {
            guard(context)?;
        }

        // Transition is valid, update state
        let old_state = self.current_state.clone();
        let new_state = config.target_state.clone();
        self.current_state = new_state.clone();

        // Execute effects and collect events
        let mut events = Vec::new();
        for effect in &config.effects {
            let effect_events = effect(context);
            events.extend(effect_events);
        }

        // Log the state transition in context
        context.set_variable(
            "last_state_transition".to_string(),
            serde_json::json!({
                "from": format!("{:?}", old_state),
                "to": format!("{:?}", new_state),
                "transition": format!("{:?}", transition),
                "timestamp": chrono::Utc::now(),
            })
        );

        // Add state transition event
        let transition_event = match &transition {
            WorkflowTransition::Start => {
                // Record the transition from draft/initial state
                context.set_variable(
                    "started_from_state".to_string(),
                    serde_json::json!(format!("{:?}", old_state))
                );
                
                // Extract started_by from context
                let started_by = context.get_variable("_started_by")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                WorkflowDomainEvent::WorkflowStarted(crate::events::WorkflowStarted {
                    workflow_id: self.workflow_id,
                    context: context.clone(),
                    started_by,
                    started_at: chrono::Utc::now(),
                })
            }
            WorkflowTransition::Complete => {
                // Calculate duration based on when we started
                let duration_seconds = if let Some(started_at) = context.get_variable("started_at") {
                    if let Some(start_time) = started_at.as_str() {
                        if let Ok(start) = chrono::DateTime::parse_from_rfc3339(start_time) {
                            (chrono::Utc::now() - start.with_timezone(&chrono::Utc)).num_seconds() as u64
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                };
                
                WorkflowDomainEvent::WorkflowCompleted(crate::events::WorkflowCompleted {
                    workflow_id: self.workflow_id,
                    final_context: context.clone(),
                    completed_at: chrono::Utc::now(),
                    duration_seconds,
                })
            }
            WorkflowTransition::Fail { reason } => {
                // Include the state we failed from in the event
                let enhanced_reason = format!("Failed from {old_state:?}: {reason}");
                
                // Calculate duration before failure
                let duration_seconds = if let Some(started_at) = context.get_variable("started_at") {
                    if let Some(start_time) = started_at.as_str() {
                        if let Ok(start) = chrono::DateTime::parse_from_rfc3339(start_time) {
                            (chrono::Utc::now() - start.with_timezone(&chrono::Utc)).num_seconds() as u64
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                };
                
                WorkflowDomainEvent::WorkflowFailed(crate::events::WorkflowFailed {
                    workflow_id: self.workflow_id,
                    error: enhanced_reason,
                    failure_context: context.clone(),
                    failed_at: chrono::Utc::now(),
                    duration_seconds,
                })
            }
            WorkflowTransition::Pause { reason } => {
                // Record what state we paused from
                context.set_variable(
                    "paused_from_state".to_string(),
                    serde_json::json!(format!("{:?}", old_state))
                );
                
                WorkflowDomainEvent::WorkflowPaused(crate::events::WorkflowPaused {
                    workflow_id: self.workflow_id,
                    reason: reason.clone(),
                    pause_context: context.clone(),
                    paused_by: None, // Would need to be passed in context
                    paused_at: chrono::Utc::now(),
                })
            }
            WorkflowTransition::Resume => {
                // Verify we're resuming from a paused state
                if old_state != WorkflowStatus::Paused {
                    // This shouldn't happen due to transition guards, but log it
                    context.set_variable(
                        "unexpected_resume".to_string(),
                        serde_json::json!(format!("Resumed from {:?} instead of Paused", old_state))
                    );
                }
                
                WorkflowDomainEvent::WorkflowResumed(crate::events::WorkflowResumed {
                    workflow_id: self.workflow_id,
                    resume_context: context.clone(),
                    resumed_by: None, // Would need to be passed in context
                    resumed_at: chrono::Utc::now(),
                })
            }
            WorkflowTransition::Cancel { reason } => {
                // Record what state we cancelled from
                let enhanced_reason = format!("Cancelled from {old_state:?}: {reason}");
                
                WorkflowDomainEvent::WorkflowCancelled(crate::events::WorkflowCancelled {
                    workflow_id: self.workflow_id,
                    reason: enhanced_reason,
                    cancellation_context: context.clone(),
                    cancelled_by: None, // Would need to be passed in context
                    cancelled_at: chrono::Utc::now(),
                })
            }
        };

        events.push(transition_event);

        Ok((new_state, events))
    }

    /// Get all valid transitions from the current state
    pub fn available_transitions(&self) -> Vec<WorkflowTransition> {
        self.transition_table
            .keys()
            .filter(|(state, _)| state == &self.current_state)
            .map(|(_, transition)| transition.clone())
            .collect()
    }

    /// Visualize the state machine as a Mermaid diagram
    pub fn to_mermaid(&self) -> String {
        let mut diagram = String::from("```mermaid\nstateDiagram-v2\n");
        
        // Add states
        diagram.push_str("    [*] --> Draft\n");
        
        // Add transitions
        for ((from_state, transition), config) in &self.transition_table {
            let transition_label = match transition {
                WorkflowTransition::Start => "Start",
                WorkflowTransition::Complete => "Complete",
                WorkflowTransition::Fail { .. } => "Fail",
                WorkflowTransition::Pause { .. } => "Pause",
                WorkflowTransition::Resume => "Resume",
                WorkflowTransition::Cancel { .. } => "Cancel",
            };
            
            diagram.push_str(&format!("    {:?} --> {:?} : {from_state}\n", config.target_state, transition_label));
        }
        
        // Mark terminal states
        diagram.push_str("    Completed --> [*]\n");
        diagram.push_str("    Failed --> [*]\n");
        diagram.push_str("    Cancelled --> [*]\n");
        
        diagram.push_str("```");
        diagram
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_machine_transitions() {
        let workflow_id = WorkflowId::new();
        let mut state_machine = WorkflowStateMachine::new(workflow_id);
        let mut context = WorkflowContext::new();
        
        // Initial state should be Draft
        assert_eq!(state_machine.current_state(), &WorkflowStatus::Draft);
        
        // Can't complete from Draft
        assert!(!state_machine.can_transition(&WorkflowTransition::Complete));
        
        // Can start from Draft
        assert!(state_machine.can_transition(&WorkflowTransition::Start));
        
        // Add required context
        context.set_variable("test".to_string(), serde_json::json!("value"));
        
        // Start the workflow
        let (new_state, events) = state_machine
            .transition(WorkflowTransition::Start, &mut context)
            .expect("Should start workflow");
        
        assert_eq!(new_state, WorkflowStatus::Running);
        assert!(!events.is_empty());
        
        // Can complete from Running
        assert!(state_machine.can_transition(&WorkflowTransition::Complete));
        
        // Complete the workflow
        let (final_state, events) = state_machine
            .transition(WorkflowTransition::Complete, &mut context)
            .expect("Should complete workflow");
        
        assert_eq!(final_state, WorkflowStatus::Completed);
        assert!(!events.is_empty());
        
        // Can't transition from terminal state
        assert!(!state_machine.can_transition(&WorkflowTransition::Start));
    }

    #[test]
    fn test_pause_resume_workflow() {
        let workflow_id = WorkflowId::new();
        let mut state_machine = WorkflowStateMachine::new(workflow_id);
        let mut context = WorkflowContext::new();
        
        // Start workflow
        context.set_variable("test".to_string(), serde_json::json!("value"));
        state_machine.transition(WorkflowTransition::Start, &mut context).unwrap();
        
        // Pause workflow
        let (state, _) = state_machine
            .transition(
                WorkflowTransition::Pause { reason: "User requested".to_string() },
                &mut context
            )
            .expect("Should pause workflow");
        
        assert_eq!(state, WorkflowStatus::Paused);
        
        // Resume workflow
        let (state, _) = state_machine
            .transition(WorkflowTransition::Resume, &mut context)
            .expect("Should resume workflow");
        
        assert_eq!(state, WorkflowStatus::Running);
    }

    #[test]
    fn test_invalid_transition() {
        let workflow_id = WorkflowId::new();
        let mut state_machine = WorkflowStateMachine::new(workflow_id);
        let mut context = WorkflowContext::new();
        
        // Try to complete from Draft (invalid)
        let result = state_machine.transition(WorkflowTransition::Complete, &mut context);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid transition"));
    }

    #[test]
    fn test_guard_prevents_transition() {
        let workflow_id = WorkflowId::new();
        let mut state_machine = WorkflowStateMachine::new(workflow_id);
        let context = WorkflowContext::new(); // Empty context
        
        // Try to start without required context
        let result = state_machine.transition(WorkflowTransition::Start, &mut context.clone());
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must have context"));
    }
} 