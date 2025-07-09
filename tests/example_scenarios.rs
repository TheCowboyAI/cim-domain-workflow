//! Tests for workflow example scenarios
//! These tests define the behavior expected from our workflow examples

use cim_domain_workflow::{
    aggregate::Workflow,
    value_objects::{WorkflowContext, WorkflowStatus, StepType, StepStatus},
    WorkflowDomainEvent,
};
use std::collections::HashMap;

#[test]
fn test_simple_workflow_execution() {
    // User Story 1: Simple workflow example
    // Given a simple 3-step workflow
    let (mut workflow, _) = Workflow::new(
        "Simple Order Processing".to_string(),
        "Process customer orders through validation, payment, and fulfillment".to_string(),
        HashMap::new(),
        Some("system".to_string()),
    ).unwrap();

    // When I add the steps
    let events = workflow.add_step(
        "Validate Order".to_string(),
        "Check order details and inventory".to_string(),
        StepType::Automated,
        HashMap::new(),
        vec![],
        Some(5),
        None,
        Some("system".to_string()),
    ).unwrap();
    
    let validate_step_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => panic!("Expected StepAdded event"),
    };

    let events = workflow.add_step(
        "Process Payment".to_string(),
        "Charge customer payment method".to_string(),
        StepType::Automated,
        HashMap::new(),
        vec![validate_step_id],
        Some(10),
        None,
        Some("system".to_string()),
    ).unwrap();
    
    let payment_step_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => panic!("Expected StepAdded event"),
    };

    workflow.add_step(
        "Fulfill Order".to_string(),
        "Ship order to customer".to_string(),
        StepType::Manual,
        HashMap::new(),
        vec![payment_step_id],
        Some(60),
        Some("warehouse".to_string()),
        Some("system".to_string()),
    ).unwrap();

    // Then the workflow should have 3 steps
    assert_eq!(workflow.steps.len(), 3);

    // And when I start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("order_id".to_string(), serde_json::json!("ORD-12345"));
    context.set_variable("total_amount".to_string(), serde_json::json!(99.99));
    
    let events = workflow.start(context, Some("system".to_string())).unwrap();
    
    // Then the workflow should be running
    assert_eq!(workflow.status, WorkflowStatus::Running);
    assert!(!events.is_empty());
    
    // And the first step should be executable
    let executable_steps = workflow.get_executable_steps();
    assert_eq!(executable_steps.len(), 1);
    assert_eq!(executable_steps[0].name, "Validate Order");
}

#[test]
fn test_advanced_branching_workflow() {
    // User Story 2: Advanced branching workflow
    // Given a workflow with conditional branching
    let (mut workflow, _) = Workflow::new(
        "Loan Application Process".to_string(),
        "Process loan applications with risk assessment".to_string(),
        HashMap::new(),
        Some("system".to_string()),
    ).unwrap();

    // When I create a workflow with parallel and conditional paths
    // Step 1: Initial Application
    let events = workflow.add_step(
        "Submit Application".to_string(),
        "Customer submits loan application".to_string(),
        StepType::Manual,
        HashMap::new(),
        vec![],
        Some(30),
        Some("customer".to_string()),
        Some("system".to_string()),
    ).unwrap();
    
    let submit_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => panic!("Expected StepAdded event"),
    };

    // Step 2a: Credit Check (parallel)
    let events = workflow.add_step(
        "Credit Check".to_string(),
        "Automated credit score verification".to_string(),
        StepType::Automated,
        HashMap::from([
            ("service".to_string(), serde_json::json!("credit_bureau_api")),
        ]),
        vec![submit_id],
        Some(5),
        None,
        Some("system".to_string()),
    ).unwrap();
    
    let credit_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => panic!("Expected StepAdded event"),
    };

    // Step 2b: Employment Verification (parallel)
    let events = workflow.add_step(
        "Employment Verification".to_string(),
        "Verify employment and income".to_string(),
        StepType::Manual,
        HashMap::new(),
        vec![submit_id],
        Some(120),
        Some("verification_team".to_string()),
        Some("system".to_string()),
    ).unwrap();
    
    let employment_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => panic!("Expected StepAdded event"),
    };

    // Step 3: Risk Assessment (depends on both parallel steps)
    let events = workflow.add_step(
        "Risk Assessment".to_string(),
        "Evaluate loan risk based on all data".to_string(),
        StepType::Automated,
        HashMap::from([
            ("risk_model".to_string(), serde_json::json!("v2.1")),
        ]),
        vec![credit_id, employment_id],
        Some(10),
        None,
        Some("system".to_string()),
    ).unwrap();
    
    let risk_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => panic!("Expected StepAdded event"),
    };

    // Step 4: Decision (conditional branching)
    workflow.add_step(
        "Loan Decision".to_string(),
        "Make final loan approval decision".to_string(),
        StepType::Approval,
        HashMap::from([
            ("auto_approve_threshold".to_string(), serde_json::json!(750)),
            ("auto_reject_threshold".to_string(), serde_json::json!(600)),
        ]),
        vec![risk_id],
        Some(30),
        Some("loan_officer".to_string()),
        Some("system".to_string()),
    ).unwrap();

    // Then the workflow should have parallel execution capability
    let mut context = WorkflowContext::new();
    context.set_variable("applicant_id".to_string(), serde_json::json!("APP-789"));
    workflow.start(context, Some("system".to_string())).unwrap();
    
    // After completing the first step, both parallel steps should be executable
    workflow.execute_step(submit_id).unwrap();
    workflow.complete_task(submit_id, "customer".to_string(), HashMap::new()).unwrap();
    
    let executable_steps = workflow.get_executable_steps();
    assert_eq!(executable_steps.len(), 2);
    assert!(executable_steps.iter().any(|s| s.name == "Credit Check"));
    assert!(executable_steps.iter().any(|s| s.name == "Employment Verification"));
}

#[test]
fn test_workflow_error_handling() {
    // User Story 4: Error handling workflow
    // Given a workflow that can fail
    let (mut workflow, _) = Workflow::new(
        "Data Processing Pipeline".to_string(),
        "Process large data files with error handling".to_string(),
        HashMap::new(),
        Some("system".to_string()),
    ).unwrap();

    // When I add steps that can fail
    let events = workflow.add_step(
        "Download Data".to_string(),
        "Download data from external source".to_string(),
        StepType::Automated,
        HashMap::from([
            ("retry_count".to_string(), serde_json::json!(3)),
            ("retry_delay_ms".to_string(), serde_json::json!(1000)),
        ]),
        vec![],
        Some(30),
        None,
        Some("system".to_string()),
    ).unwrap();
    
    let download_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => panic!("Expected StepAdded event"),
    };

    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("data_source".to_string(), serde_json::json!("https://api.example.com/data"));
    workflow.start(context, Some("system".to_string())).unwrap();

    // When a step fails
    workflow.execute_step(download_id).unwrap();
    
    // Simulate workflow failure due to step failure
    let events = workflow.fail("Network timeout on data download".to_string()).unwrap();
    
    // Then the workflow should be in failed state
    assert_eq!(workflow.status, WorkflowStatus::Failed);
    
    // And a WorkflowFailed event should be generated
    match &events[0] {
        WorkflowDomainEvent::WorkflowFailed(e) => {
            assert!(e.error.contains("Network timeout"));
        }
        _ => panic!("Expected WorkflowFailed event"),
    }
}

#[test]
fn test_workflow_monitoring_events() {
    // User Story 3: Real-time monitoring
    // Given a workflow that generates monitoring events
    let (mut workflow, _) = Workflow::new(
        "Monitoring Test Workflow".to_string(),
        "Test workflow event generation".to_string(),
        HashMap::new(),
        Some("monitor".to_string()),
    ).unwrap();

    // When I execute the workflow
    workflow.add_step(
        "Monitored Step".to_string(),
        "Step that generates events".to_string(),
        StepType::Automated,
        HashMap::new(),
        vec![],
        Some(1),
        None,
        Some("monitor".to_string()),
    ).unwrap();

    let mut context = WorkflowContext::new();
    context.set_variable("trace_id".to_string(), serde_json::json!("TRACE-123"));
    
    let start_events = workflow.start(context, Some("monitor".to_string())).unwrap();

    // Then monitoring events should be generated
    assert!(matches!(
        start_events[0],
        WorkflowDomainEvent::WorkflowStarted(_)
    ));

    // And progress can be tracked
    let progress = workflow.get_progress();
    assert_eq!(progress.total_steps, 1);
    assert_eq!(progress.pending_steps, 1);
    assert_eq!(progress.completed_steps, 0);
}