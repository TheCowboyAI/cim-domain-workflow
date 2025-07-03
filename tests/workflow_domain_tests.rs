//! Integration tests for the Workflow domain

use cim_domain::{CommandEnvelope, CommandStatus};
use cim_domain_workflow::{
    aggregate::Workflow,
    commands::*,
    handlers::{WorkflowCommandHandler, WorkflowCommandHandlerImpl},
    value_objects::*,
};
use std::collections::HashMap;

/// Test workflow creation
#[test]
fn test_workflow_creation() {
    let (workflow, _events) = Workflow::new(
        "Test Workflow".to_string(),
        "A test workflow".to_string(),
        HashMap::new(),
        Some("test-user".to_string()),
    )
    .unwrap();

    assert_eq!(workflow.name, "Test Workflow");
    assert_eq!(workflow.status, WorkflowStatus::Draft);
    assert_eq!(workflow.steps.len(), 0);
}

/// Test command handler
#[test]
fn test_command_handler() {
    let mut handler = WorkflowCommandHandlerImpl::new();

    let command = CreateWorkflow {
        name: "Test Workflow".to_string(),
        description: "A test workflow".to_string(),
        metadata: HashMap::new(),
        created_by: Some("test-user".to_string()),
    };

    let result = handler.handle_create_workflow(command);

    assert!(result.is_ok());
    let events = result.unwrap();
    assert!(!events.is_empty());
}

/// Test step management
#[test]
fn test_step_management() {
    let step = WorkflowStep::new(
        "Test Step".to_string(),
        "A test step".to_string(),
        StepType::Manual,
    );

    assert_eq!(step.name, "Test Step");
    assert_eq!(step.status, StepStatus::Pending);
    assert!(step.is_completed() == false);
}

/// Test step types
#[test]
fn test_step_types() {
    assert!(StepType::Manual.requires_human_intervention());
    assert!(StepType::Automated.can_auto_execute());
    assert!(!StepType::Manual.can_auto_execute());
}
