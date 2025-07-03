//! Step state machine implementation
//!
//! Implements a formal state machine for workflow step lifecycle management with
//! explicit transitions, guards, and effects.

use crate::{
    value_objects::{StepId, StepStatus, StepType, WorkflowId},
    events::{TaskStarted, TaskAssigned, TaskCompleted, StepFailed},
    WorkflowDomainEvent,
};
use cim_domain::{DomainError, DomainResult};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Step state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepTransition {
    /// Start executing the step
    Start { executor: Option<String> },
    /// Mark step as in progress (for long-running steps)
    Progress { percentage: u8 },
    /// Request approval for the step
    RequestApproval { approver: String },
    /// Approve the step
    Approve { approved_by: String },
    /// Reject the step
    Reject { rejected_by: String, reason: String },
    /// Complete the step successfully
    Complete { output: Option<serde_json::Value> },
    /// Mark the step as failed
    Fail { error: String },
    /// Skip the step
    Skip { reason: String },
    /// Cancel the step
    Cancel { reason: String },
    /// Retry a failed step
    Retry { attempt: u32 },
}

// Custom PartialEq that only compares the variant type, not the values
impl PartialEq for StepTransition {
    fn eq(&self, other: &Self) -> bool {
        use StepTransition::*;
        match (self, other) {
            (Start { .. }, Start { .. }) => true,
            (Progress { .. }, Progress { .. }) => true,
            (RequestApproval { .. }, RequestApproval { .. }) => true,
            (Approve { .. }, Approve { .. }) => true,
            (Reject { .. }, Reject { .. }) => true,
            (Complete { .. }, Complete { .. }) => true,
            (Fail { .. }, Fail { .. }) => true,
            (Skip { .. }, Skip { .. }) => true,
            (Cancel { .. }, Cancel { .. }) => true,
            (Retry { .. }, Retry { .. }) => true,
            _ => false,
        }
    }
}

impl Eq for StepTransition {}

// Custom Hash that only hashes the variant type
impl std::hash::Hash for StepTransition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use StepTransition::*;
        match self {
            Start { .. } => 0.hash(state),
            Progress { .. } => 1.hash(state),
            RequestApproval { .. } => 2.hash(state),
            Approve { .. } => 3.hash(state),
            Reject { .. } => 4.hash(state),
            Complete { .. } => 5.hash(state),
            Fail { .. } => 6.hash(state),
            Skip { .. } => 7.hash(state),
            Cancel { .. } => 8.hash(state),
            Retry { .. } => 9.hash(state),
        }
    }
}

/// Context for step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepContext {
    pub step_id: StepId,
    pub step_type: StepType,
    pub dependencies: Vec<StepId>,
    pub completed_dependencies: Vec<StepId>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Guard condition for step transitions
pub type StepTransitionGuard = Box<dyn Fn(&StepContext) -> DomainResult<()> + Send + Sync>;

/// Effect to execute after successful transition
pub type StepTransitionEffect = Box<dyn Fn(&mut StepContext) -> Vec<WorkflowDomainEvent> + Send + Sync>;

/// Step state machine
pub struct StepStateMachine {
    step_id: StepId,
    step_type: StepType,
    current_state: StepStatus,
    transition_table: HashMap<(StepStatus, StepTransition), StepTransitionConfig>,
}

/// Configuration for a step transition
struct StepTransitionConfig {
    target_state: StepStatus,
    guards: Vec<StepTransitionGuard>,
    effects: Vec<StepTransitionEffect>,
}

impl StepStateMachine {
    /// Create a new step state machine
    pub fn new(step_id: StepId, step_type: StepType) -> Self {
        let mut state_machine = Self {
            step_id,
            step_type: step_type.clone(),
            current_state: StepStatus::Pending,
            transition_table: HashMap::new(),
        };
        
        state_machine.configure_transitions(&step_type);
        state_machine
    }

    /// Configure transitions based on step type
    fn configure_transitions(&mut self, step_type: &StepType) {
        use StepStatus::*;
        use StepTransition::*;

        // Common transitions for all step types
        
        // Pending → Running (Start)
        self.add_transition(
            Pending,
            Start { executor: None },
            Running,
            vec![
                // Guard: Dependencies must be met
                Box::new(|ctx| {
                    let unmet_deps: Vec<_> = ctx.dependencies.iter()
                        .filter(|dep| !ctx.completed_dependencies.contains(dep))
                        .collect();
                    
                    if !unmet_deps.is_empty() {
                        Err(DomainError::generic(format!(
                            "Unmet dependencies: {unmet_deps:?}"
                        )))
                    } else {
                        Ok(())
                    }
                }),
            ],
            vec![
                // Effect: Record start time
                Box::new(|ctx| {
                    ctx.metadata.insert(
                        "started_at".to_string(),
                        serde_json::json!(chrono::Utc::now())
                    );
                    vec![]
                }),
            ],
        );

        // Pending → Skipped
        self.add_transition(
            Pending,
            Skip { reason: String::new() },
            Skipped,
            vec![],
            vec![],
        );

        // Running → InProgress (for progress updates)
        self.add_transition(
            Running,
            Progress { percentage: 0 },
            InProgress,
            vec![],
            vec![],
        );

        // InProgress → InProgress (progress updates)
        self.add_transition(
            InProgress,
            Progress { percentage: 0 },
            InProgress,
            vec![],
            vec![],
        );

        // Running/InProgress → Completed
        self.add_transition(
            Running,
            Complete { output: None },
            Completed,
            vec![],
            vec![],
        );

        self.add_transition(
            InProgress,
            Complete { output: None },
            Completed,
            vec![],
            vec![],
        );

        // Running/InProgress → Failed
        self.add_transition(
            Running,
            Fail { error: String::new() },
            Failed,
            vec![],
            vec![],
        );

        self.add_transition(
            InProgress,
            Fail { error: String::new() },
            Failed,
            vec![],
            vec![],
        );

        // Running/InProgress → Cancelled
        self.add_transition(
            Running,
            Cancel { reason: String::new() },
            Cancelled,
            vec![],
            vec![],
        );

        self.add_transition(
            InProgress,
            Cancel { reason: String::new() },
            Cancelled,
            vec![],
            vec![],
        );

        // Failed → Running (Retry)
        self.add_transition(
            Failed,
            Retry { attempt: 0 },
            Running,
            vec![
                // Guard: Check retry limit
                Box::new(|ctx| {
                    let max_retries = ctx.metadata
                        .get("max_retries")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(3);
                    
                    let current_attempts = ctx.metadata
                        .get("retry_attempts")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    
                    if current_attempts >= max_retries {
                        Err(DomainError::generic("Max retries exceeded"))
                    } else {
                        Ok(())
                    }
                }),
            ],
            vec![
                Box::new(|ctx| {
                    let attempts = ctx.metadata
                        .get("retry_attempts")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) + 1;
                    
                    ctx.metadata.insert(
                        "retry_attempts".to_string(),
                        serde_json::json!(attempts)
                    );
                    vec![]
                }),
            ],
        );

        // Step type specific transitions
        match step_type {
            StepType::Approval => {
                // Running → WaitingApproval
                self.add_transition(
                    Running,
                    RequestApproval { approver: String::new() },
                    WaitingApproval,
                    vec![],
                    vec![],
                );

                // WaitingApproval → Completed (Approved)
                self.add_transition(
                    WaitingApproval,
                    Approve { approved_by: String::new() },
                    Completed,
                    vec![],
                    vec![
                        Box::new(|ctx| {
                            ctx.metadata.insert(
                                "approved_at".to_string(),
                                serde_json::json!(chrono::Utc::now())
                            );
                            vec![]
                        }),
                    ],
                );

                // WaitingApproval → Failed (Rejected)
                self.add_transition(
                    WaitingApproval,
                    Reject { rejected_by: String::new(), reason: String::new() },
                    Failed,
                    vec![],
                    vec![],
                );

                // WaitingApproval → Cancelled
                self.add_transition(
                    WaitingApproval,
                    Cancel { reason: String::new() },
                    Cancelled,
                    vec![],
                    vec![],
                );
            }
            _ => {
                // Other step types don't have approval-specific transitions
            }
        }
    }

    /// Add a transition to the state machine
    fn add_transition(
        &mut self,
        from_state: StepStatus,
        transition: StepTransition,
        to_state: StepStatus,
        guards: Vec<StepTransitionGuard>,
        effects: Vec<StepTransitionEffect>,
    ) {
        self.transition_table.insert(
            (from_state, transition),
            StepTransitionConfig {
                target_state: to_state,
                guards,
                effects,
            },
        );
    }

    /// Get the current state
    pub fn current_state(&self) -> &StepStatus {
        &self.current_state
    }

    /// Set the current state (for restoration from events)
    pub fn set_state(&mut self, state: StepStatus) {
        self.current_state = state;
    }

    /// Check if a transition is valid from the current state
    pub fn can_transition(&self, transition: &StepTransition) -> bool {
        self.transition_table.contains_key(&(self.current_state.clone(), transition.clone()))
    }

    /// Execute a state transition
    pub fn transition(
        &mut self,
        transition: StepTransition,
        context: &mut StepContext,
    ) -> DomainResult<(StepStatus, Vec<WorkflowDomainEvent>)> {
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

        // Add state transition event with proper context including the state transition
        let transition_event = match &transition {
            StepTransition::Start { executor } => {
                // Add transition info to context
                context.metadata.insert(
                    "previous_state".to_string(), 
                    serde_json::json!(format!("{:?}", old_state))
                );
                
                WorkflowDomainEvent::TaskStarted(TaskStarted {
                    workflow_id: WorkflowId::new(), // This would need to be passed in context
                    step_id: self.step_id,
                    started_by: executor.clone(),
                    started_at: chrono::Utc::now(),
                })
            }
            StepTransition::Complete { output } => {
                // Calculate duration if we have a start time
                let duration_seconds = if let Some(started_at) = context.metadata.get("started_at") {
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
                
                WorkflowDomainEvent::TaskCompleted(TaskCompleted {
                    workflow_id: WorkflowId::new(), // This would need to be passed in context
                    step_id: self.step_id,
                    completed_by: String::new(), // Would need to be passed in context
                    completion_data: output.as_ref()
                        .map(|v| HashMap::from([("output".to_string(), v.clone())]))
                        .unwrap_or_default(),
                    completed_at: chrono::Utc::now(),
                    duration_seconds,
                })
            }
            StepTransition::Fail { error } => {
                // Log the transition from old state to failed
                context.metadata.insert(
                    "failed_from_state".to_string(),
                    serde_json::json!(format!("{:?}", old_state))
                );
                
                WorkflowDomainEvent::StepFailed(StepFailed {
                    workflow_id: WorkflowId::new(), // This would need to be passed in context
                    step_id: self.step_id,
                    reason: format!("Failed from {old_state:?}: {error}"),
                })
            }
            StepTransition::RequestApproval { approver } => {
                WorkflowDomainEvent::TaskAssigned(TaskAssigned {
                    workflow_id: WorkflowId::new(), // This would need to be passed in context
                    step_id: self.step_id,
                    assigned_to: approver.clone(),
                    assigned_by: None,
                    assigned_at: chrono::Utc::now(),
                })
            }
            _ => {
                // For other transitions, we might not generate events
                // or we could create more specific event types
                // But we should still log the state transition
                context.metadata.insert(
                    "state_transition".to_string(),
                    serde_json::json!({
                        "from": format!("{:?}", old_state),
                        "to": format!("{:?}", new_state),
                        "transition": format!("{:?}", transition),
                    })
                );
                return Ok((new_state, events));
            }
        };

        events.push(transition_event);

        Ok((new_state, events))
    }

    /// Get all valid transitions from the current state
    pub fn available_transitions(&self) -> Vec<StepTransition> {
        self.transition_table
            .keys()
            .filter(|(state, _)| state == &self.current_state)
            .map(|(_, transition)| transition.clone())
            .collect()
    }

    /// Visualize the state machine as a Mermaid diagram
    pub fn to_mermaid(&self) -> String {
        let mut diagram = String::from("```mermaid\nstateDiagram-v2\n");
        
        diagram.push_str(&format!("    title: {} Step State Machine\n", self.step_type));
        diagram.push_str("    [*] --> Pending\n");
        
        // Add transitions
        let mut unique_transitions = HashMap::new();
        for ((from_state, transition), config) in &self.transition_table {
            let transition_label = match transition {
                StepTransition::Start { .. } => "Start".to_string(),
                StepTransition::Progress { percentage } => format!("Progress({percentage}%)"),
                StepTransition::RequestApproval { .. } => "Request Approval".to_string(),
                StepTransition::Approve { .. } => "Approve".to_string(),
                StepTransition::Reject { .. } => "Reject".to_string(),
                StepTransition::Complete { .. } => "Complete".to_string(),
                StepTransition::Fail { .. } => "Fail".to_string(),
                StepTransition::Skip { .. } => "Skip".to_string(),
                StepTransition::Cancel { .. } => "Cancel".to_string(),
                StepTransition::Retry { .. } => "Retry".to_string(),
            };
            
            let key = (from_state.clone(), config.target_state.clone());
            unique_transitions
                .entry(key)
                .or_insert_with(Vec::new)
                .push(transition_label);
        }
        
        for ((from, to), labels) in unique_transitions {
            diagram.push_str(&format!("    {:?} --> {:?} : {from}\n", to, labels.join(", ")
            ));
        }
        
        // Mark terminal states
        diagram.push_str("    Completed --> [*]\n");
        diagram.push_str("    Failed --> [*]\n");
        diagram.push_str("    Cancelled --> [*]\n");
        diagram.push_str("    Skipped --> [*]\n");
        
        diagram.push_str("```");
        diagram
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_state_machine_basic_flow() {
        let step_id = StepId::new();
        let mut state_machine = StepStateMachine::new(step_id, StepType::Automated);
        let mut context = StepContext {
            step_id,
            step_type: StepType::Automated,
            dependencies: vec![],
            completed_dependencies: vec![],
            metadata: HashMap::new(),
        };
        
        // Initial state should be Pending
        assert_eq!(state_machine.current_state(), &StepStatus::Pending);
        
        // Can start from Pending
        assert!(state_machine.can_transition(&StepTransition::Start { executor: None }));
        
        // Start the step
        let (new_state, events) = state_machine
            .transition(StepTransition::Start { executor: None }, &mut context)
            .expect("Should start step");
        
        assert_eq!(new_state, StepStatus::Running);
        assert!(!events.is_empty());
        
        // Can complete from Running
        assert!(state_machine.can_transition(&StepTransition::Complete { output: None }));
        
        // Complete the step
        let (final_state, events) = state_machine
            .transition(StepTransition::Complete { output: None }, &mut context)
            .expect("Should complete step");
        
        assert_eq!(final_state, StepStatus::Completed);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_approval_step_flow() {
        let step_id = StepId::new();
        let mut state_machine = StepStateMachine::new(step_id, StepType::Approval);
        let mut context = StepContext {
            step_id,
            step_type: StepType::Approval,
            dependencies: vec![],
            completed_dependencies: vec![],
            metadata: HashMap::new(),
        };
        
        // Start the step
        state_machine.transition(StepTransition::Start { executor: None }, &mut context).unwrap();
        
        // Request approval
        let (state, _) = state_machine
            .transition(
                StepTransition::RequestApproval { approver: "manager@example.com".to_string() },
                &mut context
            )
            .expect("Should request approval");
        
        assert_eq!(state, StepStatus::WaitingApproval);
        
        // Approve the step
        let (state, _) = state_machine
            .transition(
                StepTransition::Approve { approved_by: "manager@example.com".to_string() },
                &mut context
            )
            .expect("Should approve step");
        
        assert_eq!(state, StepStatus::Completed);
    }

    #[test]
    fn test_dependency_guard() {
        let step_id = StepId::new();
        let dep1 = StepId::new();
        let dep2 = StepId::new();
        
        let mut state_machine = StepStateMachine::new(step_id, StepType::Automated);
        let mut context = StepContext {
            step_id,
            step_type: StepType::Automated,
            dependencies: vec![dep1, dep2],
            completed_dependencies: vec![dep1], // Only one dependency completed
            metadata: HashMap::new(),
        };
        
        // Try to start with unmet dependencies
        let result = state_machine.transition(StepTransition::Start { executor: None }, &mut context);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unmet dependencies"));
    }

    #[test]
    fn test_retry_with_limit() {
        let step_id = StepId::new();
        let mut state_machine = StepStateMachine::new(step_id, StepType::Automated);
        let mut context = StepContext {
            step_id,
            step_type: StepType::Automated,
            dependencies: vec![],
            completed_dependencies: vec![],
            metadata: HashMap::from([
                ("max_retries".to_string(), serde_json::json!(2)),
            ]),
        };
        
        // Start and fail the step
        state_machine.transition(StepTransition::Start { executor: None }, &mut context).unwrap();
        state_machine.transition(StepTransition::Fail { error: "Error".to_string() }, &mut context).unwrap();
        
        // First retry should succeed
        let result = state_machine.transition(StepTransition::Retry { attempt: 1 }, &mut context);
        assert!(result.is_ok());
        
        // Fail again
        state_machine.transition(StepTransition::Fail { error: "Error".to_string() }, &mut context).unwrap();
        
        // Second retry should succeed
        let result = state_machine.transition(StepTransition::Retry { attempt: 2 }, &mut context);
        assert!(result.is_ok());
        
        // Fail again
        state_machine.transition(StepTransition::Fail { error: "Error".to_string() }, &mut context).unwrap();
        
        // Third retry should fail (exceeded limit)
        let result = state_machine.transition(StepTransition::Retry { attempt: 3 }, &mut context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max retries exceeded"));
    }
}
