//! State machine implementation for workflows and steps
//!
//! This module provides a formal state machine implementation for managing
//! workflow and step state transitions, ensuring all state changes follow
//! defined rules and generate appropriate events.

pub mod workflow_state_machine;
pub mod step_state_machine;
pub mod transition_rules;

pub use workflow_state_machine::{WorkflowStateMachine, WorkflowTransition};
pub use step_state_machine::{StepStateMachine, StepTransition};
pub use transition_rules::{TransitionRules, TransitionGuard, TransitionEffect}; 