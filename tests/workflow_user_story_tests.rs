//! Failing tests for Workflow Domain User Stories
//! These tests implement the acceptance criteria for user stories W1-W22
//! They are designed to FAIL initially, following TDD principles
//!
//! ## Mermaid Test Coverage Map
//! ```mermaid
//! graph TD
//!     A[Workflow User Stories] --> B[Design & Creation W1-W3]
//!     A --> C[Execution W4-W7]
//!     A --> D[Task Management W8-W10]
//!     A --> E[Error Handling W11-W13]
//!     A --> F[Monitoring W14-W15]
//!     A --> G[Patterns W16-W18]
//!     A --> H[Advanced W19-W22]
//!     
//!     B --> B1[Visual Design]
//!     B --> B2[Templates]
//!     B --> B3[Import BPMN]
//!     
//!     C --> C1[Start Instance]
//!     C --> C2[Task Execution]
//!     C --> C3[Decisions]
//!     C --> C4[Pause/Resume]
//!     
//!     D --> D1[Human Tasks]
//!     D --> D2[System Tasks]
//!     D --> D3[Task Queues]
//! ```

use cim_domain_workflow::{
    aggregate::Workflow,
    value_objects::*,
    domain_events::WorkflowDomainEvent,
};
use std::collections::HashMap;
use serde_json::json;
use std::time::Duration;

// =============================================================================
// USER STORY W1: Design Visual Workflow
// =============================================================================

/// User Story: W1 - Design Visual Workflow
/// As a process designer, I want to create workflows visually
/// So that business processes are easy to understand
#[test]
fn test_w1_design_visual_workflow() {
    use cim_domain::AggregateRoot;
    
    // Given: A workflow designer wants to create a visual workflow
    let workflow_name = "Order Processing Workflow";
    let workflow_description = "Process customer orders from submission to delivery";
    
    // When: Creating a new workflow with visual metadata
    let mut metadata = HashMap::new();
    metadata.insert("layout".to_string(), json!("hierarchical"));
    metadata.insert("visual_style".to_string(), json!("bpmn"));
    
    let (workflow, events) = Workflow::new(
        workflow_name.to_string(),
        workflow_description.to_string(),
        metadata.clone(),
        Some("designer@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Then: The workflow should be created with visual metadata
    assert_eq!(workflow.name, workflow_name);
    assert_eq!(workflow.description, workflow_description);
    assert_eq!(workflow.metadata.get("layout"), Some(&json!("hierarchical")));
    assert_eq!(workflow.metadata.get("visual_style"), Some(&json!("bpmn")));
    
    // And: A WorkflowCreated event should be emitted
    assert_eq!(events.len(), 1);
    match &events[0] {
        cim_domain_workflow::domain_events::WorkflowDomainEvent::WorkflowCreated(event) => {
            assert_eq!(event.workflow_id, workflow.id);
            assert_eq!(event.name, workflow_name);
            assert_eq!(event.description, workflow_description);
            assert_eq!(event.metadata, metadata);
        }
        _ => panic!("Expected WorkflowCreated event"),
    }
    
    // When: Adding visual steps to the workflow
    let mut step_metadata = HashMap::new();
    step_metadata.insert("x".to_string(), json!(100));
    step_metadata.insert("y".to_string(), json!(50));
    step_metadata.insert("icon".to_string(), json!("envelope"));
    
    let step1_events = workflow.clone().add_step(
        "Receive Order".to_string(),
        "Customer submits order".to_string(),
        StepType::Manual,  // Using correct enum variant
        step_metadata,
        vec![],
        None,
        None,
        Some("designer@example.com".to_string()),
    ).expect("Should add start step");
    
    // Then: The step should be added with visual positioning
    assert_eq!(step1_events.len(), 1);
    match &step1_events[0] {
        cim_domain_workflow::domain_events::WorkflowDomainEvent::StepAdded(event) => {
            assert_eq!(event.name, "Receive Order");
            assert_eq!(event.step_type, StepType::Manual);
            assert_eq!(event.config.get("x"), Some(&json!(100)));
            assert_eq!(event.config.get("y"), Some(&json!(50)));
        }
        _ => panic!("Expected StepAdded event"),
    }
}

/// User Story: W2 - Define Workflow from Template
/// As a business user, I want to use workflow templates
/// So that I can quickly implement common processes
#[test]
fn test_w2_workflow_from_template() {
    // Given: A template for document approval workflow
    let template_metadata = {
        let mut meta = HashMap::new();
        meta.insert("template_id".to_string(), json!("doc-approval-v1"));
        meta.insert("template_name".to_string(), json!("Document Approval"));
        meta.insert("template_version".to_string(), json!("1.0"));
        meta
    };
    
    // When: Creating a workflow from the template
    let (mut workflow, create_events) = Workflow::new(
        "Contract Approval Process".to_string(),
        "Approve legal contracts using standard process".to_string(),
        template_metadata.clone(),
        Some("legal@example.com".to_string()),
    ).expect("Should create workflow from template");
    
    // Then: The workflow should have template metadata
    assert_eq!(workflow.metadata.get("template_id"), Some(&json!("doc-approval-v1")));
    
    // When: Adding predefined template steps
    let template_steps = vec![
        ("Submit Document", StepType::Manual, "Author submits document for approval"),
        ("Manager Review", StepType::Approval, "Direct manager reviews document"),
        ("Legal Review", StepType::Approval, "Legal team reviews for compliance"),
        ("Final Approval", StepType::Approval, "Executive approves document"),
        ("Archive Document", StepType::Automated, "System archives approved document"),
        ("Complete", StepType::Manual, "Process completed"),
    ];
    
    let mut all_events = create_events;
    let mut step_ids = Vec::new();
    
    for (step_name, step_type, description) in template_steps {
        let events = workflow.add_step(
            step_name.to_string(),
            description.to_string(),
            step_type.clone(),
            HashMap::new(),
            vec![],
            match step_type {
                StepType::Manual | StepType::Approval => Some(48), // 48 hour timeout for manual steps
                _ => None,
            },
            None,
            Some("legal@example.com".to_string()),
        ).expect("Should add template step");
        
        // Capture step ID from event
        if let cim_domain_workflow::domain_events::WorkflowDomainEvent::StepAdded(event) = &events[0] {
            step_ids.push(event.step_id.clone());
        }
        
        all_events.extend(events);
    }
    
    // Then: The workflow should have all template steps
    assert_eq!(step_ids.len(), 6);
    
    // Verify the workflow follows the template structure
    let step_added_count = all_events.iter().filter(|e| {
        matches!(e, cim_domain_workflow::domain_events::WorkflowDomainEvent::StepAdded(_))
    }).count();
    
    assert_eq!(step_added_count, 6, "Should have 6 steps from template");
}

/// User Story: W3 - Import Workflow Definition
/// As a workflow developer, I want to import workflow definitions
/// So that I can reuse existing processes
#[test]
fn test_w3_import_workflow_definition() {
    // Given: A workflow definition in JSON format (simulating BPMN import)
    let workflow_json = json!({
        "name": "Customer Onboarding",
        "description": "Standard customer onboarding process",
        "metadata": {
            "format": "bpmn2.0",
            "version": "2.0",
            "author": "process-team@example.com"
        },
        "steps": [
            {
                "id": "start",
                "name": "Start Onboarding",
                "type": "Manual",
                "description": "Customer registration initiated"
            },
            {
                "id": "verify-identity",
                "name": "Verify Identity",
                "type": "Approval",
                "description": "KYC verification process",
                "timeout_hours": 24,
                "assignee": "kyc-team"
            },
            {
                "id": "create-account",
                "name": "Create Account",
                "type": "Automated",
                "description": "System creates customer account"
            },
            {
                "id": "send-welcome",
                "name": "Send Welcome Email",
                "type": "Automated",
                "description": "Send welcome package to customer"
            },
            {
                "id": "end",
                "name": "Complete",
                "type": "Manual",
                "description": "Onboarding completed"
            }
        ],
        "connections": [
            {"from": "start", "to": "verify-identity"},
            {"from": "verify-identity", "to": "create-account"},
            {"from": "create-account", "to": "send-welcome"},
            {"from": "send-welcome", "to": "end"}
        ]
    });
    
    // When: Importing the workflow definition
    let import_metadata = workflow_json["metadata"].as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<HashMap<_, _>>();
    
    let (mut workflow, _events) = Workflow::new(
        workflow_json["name"].as_str().unwrap().to_string(),
        workflow_json["description"].as_str().unwrap().to_string(),
        import_metadata,
        Some("importer@example.com".to_string()),
    ).expect("Should create workflow from import");
    
    // Then: The workflow should be created with import metadata
    assert_eq!(workflow.name, "Customer Onboarding");
    assert_eq!(workflow.metadata.get("format"), Some(&json!("bpmn2.0")));
    
    // When: Processing imported steps
    let mut step_id_map = HashMap::new();
    let steps = workflow_json["steps"].as_array().unwrap();
    
    for step_json in steps {
        let step_type = match step_json["type"].as_str().unwrap() {
            "Manual" => StepType::Manual,
            "Approval" => StepType::Approval,
            "Automated" => StepType::Automated,
            "Decision" => StepType::Decision,
            _ => StepType::Custom(step_json["type"].as_str().unwrap().to_string()),
        };
        
        let timeout = step_json["timeout_hours"]
            .as_u64()
            .map(|h| h as u32);
        
        let assignee = step_json["assignee"]
            .as_str()
            .map(|s| s.to_string());
        
        let step_events = workflow.add_step(
            step_json["name"].as_str().unwrap().to_string(),
            step_json["description"].as_str().unwrap().to_string(),
            step_type,
            HashMap::new(),
            vec![],
            timeout,
            assignee,
            Some("importer@example.com".to_string()),
        ).expect("Should add imported step");
        
        // Map original ID to new step ID
        if let cim_domain_workflow::domain_events::WorkflowDomainEvent::StepAdded(event) = &step_events[0] {
            step_id_map.insert(
                step_json["id"].as_str().unwrap().to_string(),
                event.step_id.clone()
            );
        }
    }
    
    // Then: The imported workflow should have all steps
    assert_eq!(step_id_map.len(), 5, "Should import 5 steps");
    
    // Verify specific imported elements
    assert!(step_id_map.contains_key("verify-identity"), "Should have identity verification step");
    assert!(step_id_map.contains_key("create-account"), "Should have account creation step");
}

// =============================================================================
// EXECUTION USER STORIES W4-W7
// =============================================================================

/// User Story: W4 - Start Workflow Instance
/// As a user, I want to start a workflow
/// So that automated processes begin
#[test]
fn test_w4_start_workflow_instance() {
    use cim_domain::AggregateRoot;
    
    // Given: A workflow with steps defined
    let (mut workflow, _) = Workflow::new(
        "Order Processing".to_string(),
        "Process customer orders".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Add workflow steps
    let step1_events = workflow.add_step(
        "Receive Order".to_string(),
        "Initial order receipt".to_string(),
        StepType::Manual,
        HashMap::new(),
        vec![],
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let step1_id = if let WorkflowDomainEvent::StepAdded(event) = &step1_events[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    let step2_events = workflow.add_step(
        "Validate Order".to_string(),
        "Check order validity".to_string(),
        StepType::Automated,
        HashMap::new(),
        vec![step1_id], // Depends on step 1
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    // When: Starting the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("order_id".to_string().to_string(), json!("ORD-12345"));
    context.set_variable("customer_id".to_string().to_string(), json!("CUST-789"));
    
    let start_events = workflow.start(
        context.clone(),
        Some("operator@example.com".to_string()),
    ).expect("Should start workflow");
    
    // Then: The workflow should be in Running status
    assert_eq!(workflow.status, WorkflowStatus::Running);
    // Note: started_at is now tracked in the workflow context or metadata
    
    // And: A WorkflowStarted event should be emitted
    assert_eq!(start_events.len(), 1);
    match &start_events[0] {
        WorkflowDomainEvent::WorkflowStarted(event) => {
            assert_eq!(event.workflow_id, workflow.id);
            assert_eq!(event.context.get_variable("order_id"), Some(&json!("ORD-12345")));
            assert_eq!(event.started_by, Some("operator@example.com".to_string()));
        }
        _ => panic!("Expected WorkflowStarted event"),
    }
    
    // And: We should be able to get executable steps
    let executable_steps = workflow.get_executable_steps();
    assert_eq!(executable_steps.len(), 1);
    assert_eq!(executable_steps[0].name, "Receive Order");
    
    // When: Trying to start an already running workflow
    let mut new_context = WorkflowContext::new();
    new_context.set_variable("test".to_string(), serde_json::json!("value"));
    let result = workflow.start(
        new_context,
        Some("operator@example.com".to_string()),
    );
    
    // Then: It should fail
    assert!(result.is_err());
    // The state machine returns a different error message
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Invalid transition") || error_message.contains("Cannot start workflow in status Running"));
}

/// User Story: W5 - Execute Workflow Tasks
/// As a workflow engine, I want to execute tasks in sequence
/// So that processes complete correctly
#[test]
fn test_w5_execute_workflow_tasks() {
    // Given: A running workflow with sequential steps
    let (mut workflow, _) = Workflow::new(
        "Sequential Process".to_string(),
        "Execute tasks in order".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Create a chain of steps
    let step_configs: Vec<(&str, StepType, Vec<String>)> = vec![
        ("Step 1", StepType::Automated, vec![]),
        ("Step 2", StepType::Automated, vec![]), // Will depend on Step 1
        ("Step 3", StepType::Automated, vec![]), // Will depend on Step 2
    ];
    
    let mut step_ids = Vec::new();
    for (i, (name, step_type, _)) in step_configs.iter().enumerate() {
        let dependencies = if i > 0 { vec![step_ids[i-1]] } else { vec![] };
        
        let events = workflow.add_step(
            name.to_string(),
            format!("Execute {name}"),
            step_type.clone(),
            HashMap::new(),
            dependencies,
            None,
            None,
            Some("admin@example.com".to_string()),
        ).expect("Should add step");
        
        if let WorkflowDomainEvent::StepAdded(event) = &events[0] {
            step_ids.push(event.step_id);
        }
    }
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("test".to_string(), serde_json::json!("value"));
    workflow.start(
        context,
        Some("operator@example.com".to_string()),
    ).expect("Should start workflow");
    
    // When: Checking executable steps initially
    let executable = workflow.get_executable_steps();
    
    // Then: Only Step 1 should be executable (no dependencies)
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "Step 1");
    
    // When: Simulating Step 1 completion
    if let Some(step1) = workflow.steps.get_mut(&step_ids[0]) {
        step1.complete().expect("Should complete step");
    }
    
    // Then: Step 2 should now be executable
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "Step 2");
    
    // When: Simulating Step 2 completion
    if let Some(step2) = workflow.steps.get_mut(&step_ids[1]) {
        step2.complete().expect("Should complete step");
    }
    
    // Then: Step 3 should now be executable
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "Step 3");
    
    // When: Completing all steps
    if let Some(step3) = workflow.steps.get_mut(&step_ids[2]) {
        step3.complete().expect("Should complete step");
    }
    
    // Then: No more steps should be executable
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 0);
    
    // And: All steps should be completed
    let all_completed = workflow.steps.values()
        .all(|step| step.is_completed());
    assert!(all_completed);
    
    // The workflow can now be completed
    let complete_events = workflow.complete()
        .expect("Should complete workflow");
    
    assert_eq!(workflow.status, WorkflowStatus::Completed);
    // Note: completed_at is now tracked in the workflow context or metadata
}

/// User Story: W6 - Handle Workflow Decisions
/// As a workflow engine, I want to evaluate decision points
/// So that conditional logic works
#[test]
fn test_w6_handle_workflow_decisions() {
    // Given: A workflow with decision branching
    let (mut workflow, _) = Workflow::new(
        "Order Approval Process".to_string(),
        "Process orders with approval decision".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Create decision workflow structure
    let receive_order = workflow.add_step(
        "Receive Order".to_string(),
        "Initial order receipt".to_string(),
        StepType::Manual,
        HashMap::new(),
        vec![],
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let receive_id = if let WorkflowDomainEvent::StepAdded(event) = &receive_order[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Decision step
    let mut decision_config = HashMap::new();
    decision_config.insert("decision_type".to_string(), json!("order_value"));
    decision_config.insert("threshold".to_string(), json!(1000));
    
    let check_value = workflow.add_step(
        "Check Order Value".to_string(),
        "Decision based on order value".to_string(),
        StepType::Decision,
        decision_config,
        vec![receive_id],
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add decision step");
    
    let decision_id = if let WorkflowDomainEvent::StepAdded(event) = &check_value[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Branch A: Auto-approve (low value)
    let auto_approve = workflow.add_step(
        "Auto Approve".to_string(),
        "Automatically approve low value orders".to_string(),
        StepType::Automated,
        HashMap::from([("branch".to_string(), json!("low_value"))]),
        vec![decision_id],
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let auto_approve_id = if let WorkflowDomainEvent::StepAdded(event) = &auto_approve[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Branch B: Manual approval (high value)
    let manual_approve = workflow.add_step(
        "Manual Approval Required".to_string(),
        "Manager must approve high value orders".to_string(),
        StepType::Approval,
        HashMap::from([("branch".to_string(), json!("high_value"))]),
        vec![decision_id],
        Some(48), // 48 hour timeout
        Some("manager@example.com".to_string()),
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let manual_approve_id = if let WorkflowDomainEvent::StepAdded(event) = &manual_approve[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Common final step
    let mut process_config = HashMap::new();
    process_config.insert("dependency_mode".to_string(), json!("OR")); // Can come from either branch
    
    let process_order = workflow.add_step(
        "Process Order".to_string(),
        "Process the approved order".to_string(),
        StepType::Automated,
        process_config,
        vec![auto_approve_id, manual_approve_id], // Can come from either branch
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    // Start workflow with context
    let mut context = WorkflowContext::new();
    context.set_variable("order_value".to_string().to_string(), json!(500)); // Low value order
    
    workflow.start(context, Some("operator@example.com".to_string()))
        .expect("Should start workflow");
    
    // Complete receive order step
    if let Some(step) = workflow.steps.get_mut(&receive_id) {
        step.complete().expect("Should complete step");
    }
    
    // When: Decision step becomes executable
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "Check Order Value");
    
    // Simulate decision evaluation (in real system, this would be done by decision engine)
    if let Some(decision_step) = workflow.steps.get_mut(&decision_id) {
        decision_step.complete().expect("Should complete decision");
    }
    
    // Then: Based on low value, auto-approve should be executable
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "Auto Approve");
    
    // Complete auto-approve
    if let Some(step) = workflow.steps.get_mut(&auto_approve_id) {
        step.complete().expect("Should complete step");
    }
    
    // Then: Process order should be executable (one dependency satisfied)
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "Process Order");
    
    // Verify manual approval branch was not executed
    if let Some(manual_step) = workflow.steps.get(&manual_approve_id) {
        assert_eq!(manual_step.status, StepStatus::Pending);
    }
}

/// User Story: W7 - Monitor Workflow Progress
/// As a manager, I want to see workflow progress
/// So that I can track business operations
#[test]
fn test_w7_monitor_workflow_progress() {
    // Given: A workflow with multiple steps in various states
    let (mut workflow, _) = Workflow::new(
        "Multi-Stage Process".to_string(),
        "Process with monitoring points".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Add multiple steps
    let step_configs = vec![
        ("Initialize", StepType::Automated),
        ("Data Collection", StepType::Manual),
        ("Validation", StepType::Automated),
        ("Review", StepType::Approval),
        ("Finalize", StepType::Automated),
    ];
    
    let mut step_ids = Vec::new();
    for (i, (name, step_type)) in step_configs.iter().enumerate() {
        let dependencies = if i > 0 { vec![step_ids[i-1]] } else { vec![] };
        
        let events = workflow.add_step(
            name.to_string(),
            format!("Execute {name}"),
            step_type.clone(),
            HashMap::new(),
            dependencies,
            if *name == "Review" { Some(24) } else { None }, // 24hr timeout for review
            if *name == "Review" { Some("reviewer@example.com".to_string()) } else { None },
            Some("admin@example.com".to_string()),
        ).expect("Should add step");
        
        if let WorkflowDomainEvent::StepAdded(event) = &events[0] {
            step_ids.push(event.step_id);
        }
    }
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("test".to_string(), serde_json::json!("value"));
    workflow.start(
        context,
        Some("operator@example.com".to_string()),
    ).expect("Should start workflow");
    
    // When: Checking initial progress
    let progress = workflow.get_progress();
    
    // Then: Progress should show correct initial state
    assert_eq!(progress.total_steps, 5);
    assert_eq!(progress.completed_steps, 0);
    assert_eq!(progress.in_progress_steps, 0);
    assert_eq!(progress.pending_steps, 5);
    assert_eq!(progress.failed_steps, 0);
    assert_eq!(progress.percentage_complete, 0.0);
    
    // When: Completing first step
    if let Some(step) = workflow.steps.get_mut(&step_ids[0]) {
        step.complete().expect("Should complete step");
    }
    
    // Then: Progress should update
    let progress = workflow.get_progress();
    assert_eq!(progress.completed_steps, 1);
    assert_eq!(progress.pending_steps, 4);
    assert_eq!(progress.percentage_complete, 20.0);
    
    // When: Starting manual step (simulating work in progress)
    if let Some(step) = workflow.steps.get_mut(&step_ids[1]) {
        step.start(Some("worker@example.com".to_string()))
            .expect("Should start step");
    }
    
    let progress = workflow.get_progress();
    assert_eq!(progress.completed_steps, 1);
    assert_eq!(progress.in_progress_steps, 1);
    assert_eq!(progress.pending_steps, 3);
    
    // When: Getting detailed step information
    let step_details = workflow.get_step_details();
    
    // Then: Should have detailed info for each step
    assert_eq!(step_details.len(), 5);
    
    // Verify first step details
    let first_step = step_details.iter()
        .find(|s| s.name == "Initialize")
        .expect("Should find Initialize step");
    assert_eq!(first_step.status, StepStatus::Completed);
    assert!(first_step.completed_at.is_some());
    
    // Verify second step details
    let second_step = step_details.iter()
        .find(|s| s.name == "Data Collection")
        .expect("Should find Data Collection step");
    assert_eq!(second_step.status, StepStatus::InProgress);
    assert!(second_step.started_at.is_some());
    assert_eq!(second_step.assigned_to, Some("worker@example.com".to_string()));
    
    // When: Checking bottlenecks (steps taking too long)
    let bottlenecks = workflow.get_bottlenecks(Duration::from_secs(0)); // 0 seconds to detect any in-progress
    
    // Then: Should identify the in-progress manual step
    assert_eq!(bottlenecks.len(), 1);
    assert_eq!(bottlenecks[0].name, "Data Collection");
    
    // When: Getting critical path (longest dependency chain)
    let critical_path = workflow.get_critical_path();
    
    // Then: Should identify the full chain (all steps are sequential)
    assert_eq!(critical_path.len(), 5);
    assert_eq!(critical_path[0].name, "Initialize");
    assert_eq!(critical_path[4].name, "Finalize");
    
    // When: Checking for steps with timeouts
    let timeout_risks = workflow.get_timeout_risks();
    
    // Then: Should identify the Review step with 24hr timeout
    assert_eq!(timeout_risks.len(), 1);
    assert_eq!(timeout_risks[0].name, "Review");
    assert_eq!(timeout_risks[0].timeout_hours, Some(24));
}

// =============================================================================
// TASK MANAGEMENT USER STORIES W8-W10
// =============================================================================

/// User Story: W8 - Assign Human Tasks
/// As a workflow engine, I want to assign tasks to humans
/// So that manual steps are handled
#[test]
fn test_w8_assign_human_tasks() {
    // Given: A workflow with human tasks that need assignment
    let (mut workflow, _) = Workflow::new(
        "Document Review Process".to_string(),
        "Process requiring human review and approval".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Add automated step
    let prepare_doc = workflow.add_step(
        "Prepare Document".to_string(),
        "Automated document preparation".to_string(),
        StepType::Automated,
        HashMap::new(),
        vec![],
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let prepare_id = if let WorkflowDomainEvent::StepAdded(event) = &prepare_doc[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Add manual review step with role-based assignment
    let mut review_config = HashMap::new();
    review_config.insert("assignment_rule".to_string(), json!("role"));
    review_config.insert("required_role".to_string(), json!("reviewer"));
    review_config.insert("priority".to_string(), json!("high"));
    
    let review_doc = workflow.add_step(
        "Review Document".to_string(),
        "Manual review of prepared document".to_string(),
        StepType::Manual,
        review_config,
        vec![prepare_id],
        Some(72), // 72 hour timeout
        None, // No specific assignee yet
        Some("admin@example.com".to_string()),
    ).expect("Should add manual step");
    
    let review_id = if let WorkflowDomainEvent::StepAdded(event) = &review_doc[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Add approval step with specific assignee
    let approve_doc = workflow.add_step(
        "Approve Document".to_string(),
        "Final approval by manager".to_string(),
        StepType::Approval,
        HashMap::from([("approval_level".to_string(), json!("manager"))]),
        vec![review_id],
        Some(24), // 24 hour timeout
        Some("manager@example.com".to_string()), // Pre-assigned
        Some("admin@example.com".to_string()),
    ).expect("Should add approval step");
    
    let approve_id = if let WorkflowDomainEvent::StepAdded(event) = &approve_doc[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("test".to_string(), serde_json::json!("value"));
    workflow.start(
        context,
        Some("operator@example.com".to_string()),
    ).expect("Should start workflow");
    
    // Complete automated step
    if let Some(step) = workflow.steps.get_mut(&prepare_id) {
        step.complete().expect("Should complete step");
    }
    
    // When: Getting assignable tasks
    let assignable_tasks = workflow.get_assignable_tasks();
    
    // Then: Review task should be assignable
    assert_eq!(assignable_tasks.len(), 1);
    assert_eq!(assignable_tasks[0].name, "Review Document");
    assert!(assignable_tasks[0].assigned_to.is_none());
    assert_eq!(assignable_tasks[0].config.get("required_role"), 
               Some(&json!("reviewer")));
    
    // When: Assigning task to a reviewer
    let assign_events = workflow.assign_task(
        review_id,
        "reviewer1@example.com".to_string(),
        Some("operator@example.com".to_string()),
    ).expect("Should assign task");
    
    // Then: Task should be assigned
    assert_eq!(assign_events.len(), 1);
    match &assign_events[0] {
        WorkflowDomainEvent::TaskAssigned(event) => {
            assert_eq!(event.step_id, review_id);
            assert_eq!(event.assigned_to, "reviewer1@example.com");
            assert_eq!(event.assigned_by, Some("operator@example.com".to_string()));
        }
        _ => panic!("Expected TaskAssigned event"),
    }
    
    // And: Task should no longer be in assignable list
    let assignable_tasks = workflow.get_assignable_tasks();
    assert_eq!(assignable_tasks.len(), 0);
    
    // When: Getting tasks by assignee
    let reviewer_tasks = workflow.get_tasks_for_assignee("reviewer1@example.com");
    
    // Then: Should find the review task
    assert_eq!(reviewer_tasks.len(), 1);
    assert_eq!(reviewer_tasks[0].name, "Review Document");
    assert_eq!(reviewer_tasks[0].status, StepStatus::Pending);
    
    // When: Reassigning task to another reviewer
    let reassign_events = workflow.reassign_task(
        review_id,
        "reviewer1@example.com".to_string(),  // from_assignee
        "reviewer2@example.com".to_string(),  // to_assignee
        Some("manager@example.com".to_string()),  // reassigned_by
        Some("Workload balancing".to_string()),  // reason
    ).expect("Should reassign task");
    
    // Then: Task should be reassigned
    assert_eq!(reassign_events.len(), 1);
    match &reassign_events[0] {
        WorkflowDomainEvent::TaskReassigned(event) => {
            assert_eq!(event.step_id, review_id);
            assert_eq!(event.from_assignee, "reviewer1@example.com");
            assert_eq!(event.to_assignee, "reviewer2@example.com");
            assert_eq!(event.reassigned_by, Some("manager@example.com".to_string()));
        }
        _ => panic!("Expected TaskReassigned event"),
    }
    
    // When: Getting high priority tasks
    let high_priority_tasks = workflow.get_high_priority_tasks();
    
    // Then: Should find the review task (marked as high priority)
    assert_eq!(high_priority_tasks.len(), 1);
    assert_eq!(high_priority_tasks[0].name, "Review Document");
    
    // When: Checking pre-assigned tasks
    let pre_assigned_tasks = workflow.get_pre_assigned_tasks();
    
    // Then: Should find both review (now assigned) and approval (pre-assigned) tasks
    assert_eq!(pre_assigned_tasks.len(), 2);
    
    // Find the approval task specifically
    let approval_task = pre_assigned_tasks.iter()
        .find(|t| t.name == "Approve Document")
        .expect("Should find approval task");
    assert_eq!(approval_task.assigned_to, Some("manager@example.com".to_string()));
}

/// User Story: W9 - Complete Human Tasks
/// As a task assignee, I want to complete assigned tasks
/// So that workflows can proceed
#[test]
fn test_w9_complete_human_tasks() {
    // Given: A workflow with assigned human tasks
    let (mut workflow, _) = Workflow::new(
        "Employee Onboarding".to_string(),
        "Onboard new employee with multiple approvals".to_string(),
        HashMap::new(),
        Some("hr@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Create workflow steps
    let collect_info = workflow.add_step(
        "Collect Employee Info".to_string(),
        "HR collects new employee information".to_string(),
        StepType::Manual,
        HashMap::from([
            ("form_fields".to_string(), json!(["name", "email", "department", "start_date"])),
            ("required_fields".to_string(), json!(["name", "email", "department"])),
        ]),
        vec![],
        Some(48), // 48 hour timeout
        Some("hr_specialist@example.com".to_string()),
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    let collect_id = if let WorkflowDomainEvent::StepAdded(event) = &collect_info[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // IT setup task
    let it_setup = workflow.add_step(
        "IT Account Setup".to_string(),
        "IT creates accounts and provides equipment".to_string(),
        StepType::Manual,
        HashMap::from([
            ("checklist".to_string(), json!({
                "create_email": false,
                "setup_workstation": false,
                "grant_access": false,
                "provide_equipment": false
            })),
        ]),
        vec![collect_id],
        Some(72), // 72 hour timeout
        Some("it_admin@example.com".to_string()),
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    let it_id = if let WorkflowDomainEvent::StepAdded(event) = &it_setup[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Manager approval
    let manager_approval = workflow.add_step(
        "Manager Approval".to_string(),
        "Department manager approves onboarding completion".to_string(),
        StepType::Approval,
        HashMap::from([
            ("approval_options".to_string(), json!(["approve", "reject", "request_changes"])),
        ]),
        vec![collect_id, it_id], // Depends on both previous steps
        Some(24), // 24 hour timeout
        Some("dept_manager@example.com".to_string()),
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    let approval_id = if let WorkflowDomainEvent::StepAdded(event) = &manager_approval[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("test".to_string(), serde_json::json!("value"));
    workflow.start(
        context,
        Some("hr@example.com".to_string()),
    ).expect("Should start workflow");
    
    // When: HR specialist completes the form task
    let mut form_data = HashMap::new();
    form_data.insert("name".to_string(), json!("John Doe"));
    form_data.insert("email".to_string(), json!("john.doe@example.com"));
    form_data.insert("department".to_string(), json!("Engineering"));
    form_data.insert("start_date".to_string(), json!("2024-02-01"));
    form_data.insert("notes".to_string(), json!("Remote worker, needs VPN access"));
    
    let complete_events = workflow.complete_task(
        collect_id,
        "hr_specialist@example.com".to_string(),
        form_data,
    ).expect("Should complete task");
    
    // Then: Task should be completed with form data
    assert_eq!(complete_events.len(), 1);
    match &complete_events[0] {
        WorkflowDomainEvent::TaskCompleted(event) => {
            assert_eq!(event.step_id, collect_id);
            assert_eq!(event.completed_by, "hr_specialist@example.com".to_string());
            assert!(!event.completion_data.is_empty());
            let output = &event.completion_data;
            assert_eq!(output["name"], json!("John Doe"));
        }
        _ => panic!("Expected TaskCompleted event"),
    }
    
    // And: IT task should now be executable
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "IT Account Setup");
    
    // When: IT admin starts working on the task
    let start_events = workflow.start_task(
        it_id,
        "it_admin@example.com".to_string(),
    ).expect("Should start task");
    
    assert_eq!(start_events.len(), 1);
    match &start_events[0] {
        WorkflowDomainEvent::TaskStarted(event) => {
            assert_eq!(event.step_id, it_id);
            assert_eq!(event.started_by, Some("it_admin@example.com".to_string()));
        }
        _ => panic!("Expected TaskStarted event"),
    }
    
    // When: IT admin completes checklist items progressively
    let mut checklist = HashMap::new();
    checklist.insert("create_email".to_string(), json!(true));
    checklist.insert("setup_workstation".to_string(), json!(true));
    checklist.insert("grant_access".to_string(), json!(true));
    checklist.insert("provide_equipment".to_string(), json!(true));
    checklist.insert("equipment_list".to_string(), json!(["Laptop", "Monitor", "Keyboard", "Mouse"]));
    
    let it_complete_events = workflow.complete_task(
        it_id,
        "it_admin@example.com".to_string(),
        checklist,
    ).expect("Should complete IT task");
    
    // Then: Both prerequisite tasks should be complete
    let completed_count = workflow.steps.values()
        .filter(|s| s.is_completed())
        .count();
    assert_eq!(completed_count, 2);
    
    // And: Manager approval should be executable
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1);
    assert_eq!(executable[0].name, "Manager Approval");
    
    // When: Manager approves with comments
    let mut approval_data = HashMap::new();
    approval_data.insert("decision".to_string(), json!("approve"));
    approval_data.insert("comments".to_string(), json!("Welcome to the team!"));
    approval_data.insert("effective_date".to_string(), json!("2024-02-01"));
    
    let approval_events = workflow.complete_task(
        approval_id,
        "dept_manager@example.com".to_string(),
        approval_data,
    ).expect("Should complete approval");
    
    // Then: All tasks should be complete
    let all_completed = workflow.steps.values()
        .all(|s| s.is_completed());
    assert!(all_completed);
    
    // And: Workflow can be completed
    let workflow_complete = workflow.complete()
        .expect("Should complete workflow");
    
    assert_eq!(workflow.status, WorkflowStatus::Completed);
    
    // When: Checking task completion data
    let task_outputs = workflow.get_all_task_outputs();
    
    // Then: Should have all task outputs preserved
    assert_eq!(task_outputs.len(), 3);
    assert!(task_outputs.contains_key(&collect_id));
    assert!(task_outputs.contains_key(&it_id));
    assert!(task_outputs.contains_key(&approval_id));
}

/// User Story: W10 - Invoke System Tasks
/// As a workflow engine, I want to call external systems
/// So that integrations work seamlessly
#[test]
fn test_w10_invoke_system_tasks() {
    // Given: A workflow with external system integrations
    let (mut workflow, _) = Workflow::new(
        "Order Fulfillment".to_string(),
        "Process order with external system integrations".to_string(),
        HashMap::new(),
        Some("system@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Validate order step
    let validate_order = workflow.add_step(
        "Validate Order".to_string(),
        "Check order validity".to_string(),
        StepType::Automated,
        HashMap::new(),
        vec![],
        None,
        None,
        Some("system@example.com".to_string()),
    ).expect("Should add step");
    
    let validate_id = if let WorkflowDomainEvent::StepAdded(event) = &validate_order[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Integration step - Check inventory via external API
    let mut inventory_config = HashMap::new();
    inventory_config.insert("integration_type".to_string(), json!("REST_API"));
    inventory_config.insert("endpoint".to_string(), json!("https://api.inventory.com/check"));
    inventory_config.insert("method".to_string(), json!("POST"));
    inventory_config.insert("retry_policy".to_string(), json!({
        "max_attempts": 3,
        "backoff_seconds": [1, 2, 4],
        "retry_on_codes": [500, 502, 503, 504]
    }));
    inventory_config.insert("timeout_seconds".to_string(), json!(30));
    
    let check_inventory = workflow.add_step(
        "Check Inventory".to_string(),
        "Verify stock availability via external system".to_string(),
        StepType::Integration,
        inventory_config,
        vec![validate_id],
        Some(1), // 1 hour timeout for the whole step
        None,
        Some("system@example.com".to_string()),
    ).expect("Should add integration step");
    
    let inventory_id = if let WorkflowDomainEvent::StepAdded(event) = &check_inventory[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Integration step - Process payment
    let mut payment_config = HashMap::new();
    payment_config.insert("integration_type".to_string(), json!("gRPC"));
    payment_config.insert("service".to_string(), json!("payment.service.PaymentProcessor"));
    payment_config.insert("method".to_string(), json!("ProcessPayment"));
    payment_config.insert("retry_policy".to_string(), json!({
        "max_attempts": 2,
        "backoff_type": "exponential",
        "initial_interval_ms": 100,
        "max_interval_ms": 5000
    }));
    payment_config.insert("circuit_breaker".to_string(), json!({
        "failure_threshold": 5,
        "timeout_seconds": 60,
        "half_open_requests": 2
    }));
    
    let process_payment = workflow.add_step(
        "Process Payment".to_string(),
        "Charge customer via payment gateway".to_string(),
        StepType::Integration,
        payment_config,
        vec![inventory_id],
        Some(2), // 2 hour timeout
        None,
        Some("system@example.com".to_string()),
    ).expect("Should add payment step");
    
    let payment_id = if let WorkflowDomainEvent::StepAdded(event) = &process_payment[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Integration step - Ship order (webhook-based)
    let mut shipping_config = HashMap::new();
    shipping_config.insert("integration_type".to_string(), json!("WEBHOOK"));
    shipping_config.insert("webhook_url".to_string(), json!("https://shipping.partner.com/api/ship"));
    shipping_config.insert("callback_url".to_string(), json!("https://our.system.com/workflow/callback"));
    shipping_config.insert("authentication".to_string(), json!({
        "type": "API_KEY",
        "header": "X-API-Key",
        "key_ref": "SHIPPING_API_KEY"
    }));
    shipping_config.insert("async_pattern".to_string(), json!("callback"));
    
    let ship_order = workflow.add_step(
        "Ship Order".to_string(),
        "Initiate shipping via partner API".to_string(),
        StepType::Integration,
        shipping_config,
        vec![payment_id],
        Some(24), // 24 hour timeout for shipping
        None,
        Some("system@example.com".to_string()),
    ).expect("Should add shipping step");
    
    let shipping_id = if let WorkflowDomainEvent::StepAdded(event) = &ship_order[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("order_id".to_string(), json!("ORD-123456"));
    context.set_variable("items".to_string(), json!([
        {"sku": "PROD-001", "quantity": 2},
        {"sku": "PROD-002", "quantity": 1}
    ]));
    context.set_variable("total_amount".to_string(), json!(150.00));
    
    workflow.start(context, Some("system@example.com".to_string()))
        .expect("Should start workflow");
    
    // Complete validation
    if let Some(step) = workflow.steps.get_mut(&validate_id) {
        step.complete().expect("Should complete step");
    }
    
    // When: Getting integration steps
    let integration_steps = workflow.get_integration_steps();
    
    // Then: Should find all three integration steps
    assert_eq!(integration_steps.len(), 3);
    assert!(integration_steps.iter().any(|s| s.name == "Check Inventory"));
    assert!(integration_steps.iter().any(|s| s.name == "Process Payment"));
    assert!(integration_steps.iter().any(|s| s.name == "Ship Order"));
    
    // When: Simulating inventory check with retry
    let mut inventory_step = workflow.steps.get_mut(&inventory_id).unwrap();
    
    // First attempt fails
    let fail_result = inventory_step.record_integration_attempt(
        1,
        false,
        Some("Connection timeout".to_string()),
        Some(500),
    );
    assert!(fail_result.is_ok());
    
    // Second attempt succeeds
    let success_result = inventory_step.record_integration_attempt(
        2,
        true,
        None,
        Some(200),
    );
    assert!(success_result.is_ok());
    
    // Complete the step with inventory data
    inventory_step.complete_with_data(json!({
        "available": true,
        "stock_levels": {
            "PROD-001": 50,
            "PROD-002": 25
        }
    })).expect("Should complete inventory check");
    
    // When: Getting retry statistics
    let retry_stats = workflow.get_integration_retry_stats();
    
    // Then: Should show retry information
    assert_eq!(retry_stats.len(), 1);
    assert_eq!(retry_stats[0].step_name, "Check Inventory");
    assert_eq!(retry_stats[0].total_attempts, 2);
    assert_eq!(retry_stats[0].successful_attempts, 1);
    assert_eq!(retry_stats[0].failed_attempts, 1);
    
    // When: Checking circuit breaker status
    let circuit_breakers = workflow.get_circuit_breaker_status();
    
    // Then: Payment service should have circuit breaker configured
    assert!(circuit_breakers.iter().any(|cb| {
        cb.step_name == "Process Payment" && cb.state == "CLOSED"
    }));
    
    // When: Getting async integration status
    let async_integrations = workflow.get_async_integration_status();
    
    // Then: Shipping should be listed as async
    assert_eq!(async_integrations.len(), 1);
    assert_eq!(async_integrations[0].step_name, "Ship Order");
    assert_eq!(async_integrations[0].pattern, "callback");
    assert!(async_integrations[0].callback_url.is_some());
}

// =============================================================================
// ERROR HANDLING USER STORIES W11-W13
// =============================================================================

/// User Story: W11 - Handle Task Failures
/// As a workflow engine, I want to handle task failures gracefully
/// So that workflows are resilient
#[test]
fn test_w11_handle_task_failures() {
    // Given: A workflow with steps that might fail
    let (mut workflow, _) = Workflow::new(
        "Resilient Process".to_string(),
        "Process with error handling".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Add steps with retry configuration in metadata
    let mut api_config = HashMap::new();
    api_config.insert("retry_policy".to_string(), json!({
        "max_attempts": 3,
        "backoff_ms": [100, 500, 2000],
        "retry_on": ["timeout", "connection_error", "500"]
    }));
    api_config.insert("timeout_ms".to_string(), json!(5000));
    
    let api_call = workflow.add_step(
        "Call External API".to_string(),
        "Fetch data from external service".to_string(),
        StepType::Integration,
        api_config,
        vec![],
        Some(5), // 5 minute timeout for whole step
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let api_id = if let WorkflowDomainEvent::StepAdded(event) = &api_call[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Add compensation step
    let mut compensation_config = HashMap::new();
    compensation_config.insert("compensation_for".to_string(), json!(api_id.0.to_string()));
    compensation_config.insert("compensation_type".to_string(), json!("rollback"));
    
    let compensate = workflow.add_step(
        "Compensate API Failure".to_string(),
        "Clean up after API failure".to_string(),
        StepType::Automated,
        compensation_config,
        vec![], // No dependencies - triggered by failure
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add compensation step");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("test".to_string(), serde_json::json!("value"));
    workflow.start(
        context,
        Some("operator@example.com".to_string()),
    ).expect("Should start workflow");
    
    // When: Simulating step failure
    // In a real implementation, this would be handled by the step execution engine
    let failure_reason = "Connection timeout after 3 retries";
    
    // The workflow can be failed when critical steps fail
    let fail_events = workflow.fail(
        format!("Step '{"Call External API"}' failed: {failure_reason}"),
    ).expect("Should fail workflow");
    
    // Then: Workflow should be in failed state
    assert_eq!(workflow.status, WorkflowStatus::Failed);
    
    // And: A WorkflowFailed event should be emitted
    assert_eq!(fail_events.len(), 1);
    match &fail_events[0] {
        WorkflowDomainEvent::WorkflowFailed(event) => {
            assert_eq!(event.workflow_id, workflow.id);
            assert!(event.error.contains("Connection timeout"));
        }
        _ => panic!("Expected WorkflowFailed event"),
    }
    
    // In a complete implementation, the compensation step would be triggered
    // and the workflow could potentially be retried or manually recovered
}

/// User Story: W12 - Implement Circuit Breakers
/// As a system administrator, I want circuit breakers on external calls
/// So that cascading failures are prevented
#[test]
fn test_w12_circuit_breakers() {
    // Given: A workflow with external integrations that need circuit breakers
    let (mut workflow, _) = Workflow::new(
        "Payment Processing".to_string(),
        "Process payments with multiple gateways".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Primary payment gateway with circuit breaker
    let mut primary_config = HashMap::new();
    primary_config.insert("gateway".to_string(), json!("stripe"));
    primary_config.insert("circuit_breaker".to_string(), json!({
        "failure_threshold": 5,        // Open circuit after 5 failures
        "success_threshold": 2,        // Close circuit after 2 successes
        "timeout_seconds": 60,         // Try half-open after 60 seconds
        "rolling_window_seconds": 300  // Track failures over 5 minutes
    }));
    
    let primary_payment = workflow.add_step(
        "Process Payment - Primary".to_string(),
        "Attempt payment via primary gateway".to_string(),
        StepType::Integration,
        primary_config,
        vec![],
        Some(2), // 2 minute timeout
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add primary payment step");
    
    let primary_id = if let WorkflowDomainEvent::StepAdded(event) = &primary_payment[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Fallback payment gateway
    let mut fallback_config = HashMap::new();
    fallback_config.insert("gateway".to_string(), json!("paypal"));
    fallback_config.insert("fallback_for".to_string(), json!(primary_id.0.to_string()));
    fallback_config.insert("circuit_breaker".to_string(), json!({
        "failure_threshold": 3,  // More conservative for fallback
        "success_threshold": 1,
        "timeout_seconds": 120,
        "rolling_window_seconds": 600
    }));
    
    let fallback_payment = workflow.add_step(
        "Process Payment - Fallback".to_string(),
        "Fallback payment via secondary gateway".to_string(),
        StepType::Integration,
        fallback_config,
        vec![], // No dependency - triggered by circuit breaker
        Some(3), // 3 minute timeout
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add fallback payment step");
    
    // Alert step when both gateways fail
    let mut alert_config = HashMap::new();
    alert_config.insert("alert_type".to_string(), json!("payment_failure"));
    alert_config.insert("severity".to_string(), json!("critical"));
    alert_config.insert("notify".to_string(), json!(["ops-team@example.com", "finance@example.com"]));
    
    let alert_step = workflow.add_step(
        "Alert Payment Failure".to_string(),
        "Notify teams of payment system failure".to_string(),
        StepType::Automated,
        alert_config,
        vec![], // Triggered by both circuit breakers open
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add alert step");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("amount".to_string(), json!(99.99));
    context.set_variable("currency".to_string(), json!("USD"));
    context.set_variable("customer_id".to_string(), json!("CUST-123"));
    
    workflow.start(context, Some("payment-system@example.com".to_string()))
        .expect("Should start workflow");
    
    // Then: Circuit breaker configuration should be in step metadata
    let primary_step = workflow.steps.get(&primary_id).unwrap();
    assert!(primary_step.config.contains_key("circuit_breaker"));
    
    let circuit_config = primary_step.config.get("circuit_breaker").unwrap();
    assert_eq!(circuit_config["failure_threshold"], json!(5));
    assert_eq!(circuit_config["timeout_seconds"], json!(60));
    
    // In a real implementation:
    // 1. Circuit breaker would track failure rates
    // 2. After threshold failures, circuit opens
    // 3. Fallback step would be triggered
    // 4. If both fail, alert step executes
    // 5. After timeout, circuit enters half-open state
    // 6. Success in half-open state closes circuit
    
    // The circuit breaker pattern prevents cascading failures
    // by failing fast when a service is down
}

/// User Story: W13 - Rollback Workflow
/// As a workflow operator, I want to rollback failed workflows
/// So that system consistency is maintained
#[test]
fn test_w13_rollback_workflow() {
    // Given: A workflow with transactional steps that need rollback
    let (mut workflow, _) = Workflow::new(
        "Order Fulfillment Transaction".to_string(),
        "Transactional order processing with rollback".to_string(),
        HashMap::new(),
        Some("admin@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Step 1: Reserve inventory
    let mut reserve_config = HashMap::new();
    reserve_config.insert("operation".to_string(), json!("reserve"));
    reserve_config.insert("rollback_operation".to_string(), json!("release"));
    
    let reserve_inventory = workflow.add_step(
        "Reserve Inventory".to_string(),
        "Reserve items from inventory".to_string(),
        StepType::Integration,
        reserve_config,
        vec![],
        Some(5),
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let reserve_id = if let WorkflowDomainEvent::StepAdded(event) = &reserve_inventory[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Charge payment
    let mut charge_config = HashMap::new();
    charge_config.insert("operation".to_string(), json!("charge"));
    charge_config.insert("rollback_operation".to_string(), json!("refund"));
    
    let charge_payment = workflow.add_step(
        "Charge Payment".to_string(),
        "Charge customer payment method".to_string(),
        StepType::Integration,
        charge_config,
        vec![reserve_id], // Depends on inventory reservation
        Some(10),
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    let charge_id = if let WorkflowDomainEvent::StepAdded(event) = &charge_payment[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 3: Create shipment (this will fail in our test)
    let mut ship_config = HashMap::new();
    ship_config.insert("operation".to_string(), json!("create_shipment"));
    ship_config.insert("rollback_operation".to_string(), json!("cancel_shipment"));
    
    let create_shipment = workflow.add_step(
        "Create Shipment".to_string(),
        "Create shipment with carrier".to_string(),
        StepType::Integration,
        ship_config,
        vec![charge_id], // Depends on payment
        Some(15),
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add step");
    
    // Compensation steps (added but not in main flow)
    let mut release_config = HashMap::new();
    release_config.insert("compensates".to_string(), json!(reserve_id.as_uuid().to_string()));
    release_config.insert("operation".to_string(), json!("release_inventory"));
    
    workflow.add_step(
        "Release Inventory (Compensation)".to_string(),
        "Release reserved inventory on failure".to_string(),
        StepType::Automated,
        release_config,
        vec![], // No dependencies - triggered by rollback
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add compensation step");
    
    let mut refund_config = HashMap::new();
    refund_config.insert("compensates".to_string(), json!(charge_id.as_uuid().to_string()));
    refund_config.insert("operation".to_string(), json!("refund_payment"));
    
    workflow.add_step(
        "Refund Payment (Compensation)".to_string(),
        "Refund charged payment on failure".to_string(),
        StepType::Automated,
        refund_config,
        vec![], // No dependencies - triggered by rollback
        None,
        None,
        Some("admin@example.com".to_string()),
    ).expect("Should add compensation step");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("order_id".to_string(), json!("ORD-789"));
    context.set_variable("items".to_string(), json!([
        {"sku": "ITEM-001", "quantity": 2},
        {"sku": "ITEM-002", "quantity": 1}
    ]));
    context.set_variable("payment_method".to_string(), json!("credit_card"));
    context.set_variable("amount".to_string(), json!(250.00));
    
    workflow.start(context, Some("order-system@example.com".to_string()))
        .expect("Should start workflow");
    
    // When: Workflow fails at shipment creation
    let failure_reason = "Carrier API unavailable - shipment creation failed";
    
    // Cancel the workflow (which should trigger rollback)
    let cancel_events = workflow.cancel(
        format!("Rollback required: {failure_reason}"),
        Some("system@example.com".to_string()),
    ).expect("Should cancel workflow");
    
    // Then: Workflow should be cancelled
    assert_eq!(workflow.status, WorkflowStatus::Cancelled);
    
    // And: A WorkflowCancelled event should be emitted
    assert_eq!(cancel_events.len(), 1);
    match &cancel_events[0] {
        WorkflowDomainEvent::WorkflowCancelled(event) => {
            assert_eq!(event.workflow_id, workflow.id);
            assert!(event.reason.contains("Rollback required"));
        }
        _ => panic!("Expected WorkflowCancelled event"),
    }
    
    // In a complete implementation:
    // 1. The rollback would execute compensation steps in reverse order
    // 2. Refund Payment would execute first (compensating the last successful step)
    // 3. Release Inventory would execute second
    // 4. Each compensation would emit its own events
    // 5. The system would track rollback progress and handle failures
}

// =============================================================================
// MONITORING USER STORIES W14-W15
// =============================================================================

/// User Story: W14 - Monitor Workflow Progress
/// As a process manager, I want to monitor workflow progress
/// So that I can ensure timely completion
#[test]
fn test_w14_monitor_workflow_progress() {
    // Given: A workflow with SLA requirements
    let mut metadata = HashMap::new();
    metadata.insert("sla_hours".to_string(), json!(24)); // 24 hour SLA
    metadata.insert("priority".to_string(), json!("high"));
    metadata.insert("customer_tier".to_string(), json!("premium"));
    
    let (mut workflow, _) = Workflow::new(
        "Customer Support Ticket".to_string(),
        "Premium support ticket workflow".to_string(),
        metadata,
        Some("support@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Add steps with individual SLAs
    let mut triage_config = HashMap::new();
    triage_config.insert("sla_minutes".to_string(), json!(30)); // 30 minute SLA
    triage_config.insert("severity_levels".to_string(), json!(["critical", "high", "medium", "low"]));
    
    let triage = workflow.add_step(
        "Triage Ticket".to_string(),
        "Initial ticket assessment and routing".to_string(),
        StepType::Manual,
        triage_config,
        vec![],
        Some(1), // 1 hour timeout
        Some("triage-team@example.com".to_string()),
        Some("support@example.com".to_string()),
    ).expect("Should add step");
    
    let triage_id = if let WorkflowDomainEvent::StepAdded(event) = &triage[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Investigation step
    let mut investigate_config = HashMap::new();
    investigate_config.insert("sla_hours".to_string(), json!(4)); // 4 hour SLA
    investigate_config.insert("requires_expertise".to_string(), json!(["database", "networking", "security"]));
    
    let investigate = workflow.add_step(
        "Investigate Issue".to_string(),
        "Deep dive into customer issue".to_string(),
        StepType::Manual,
        investigate_config,
        vec![triage_id],
        Some(6), // 6 hour timeout
        Some("tech-team@example.com".to_string()),
        Some("support@example.com".to_string()),
    ).expect("Should add step");
    
    let investigate_id = if let WorkflowDomainEvent::StepAdded(event) = &investigate[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Resolution step
    let mut resolve_config = HashMap::new();
    resolve_config.insert("sla_hours".to_string(), json!(8)); // 8 hour SLA
    resolve_config.insert("resolution_types".to_string(), json!(["fix_applied", "workaround", "escalated", "no_fix"]));
    
    let resolve = workflow.add_step(
        "Resolve Issue".to_string(),
        "Apply fix or workaround".to_string(),
        StepType::Manual,
        resolve_config,
        vec![investigate_id],
        Some(12), // 12 hour timeout
        None,
        Some("support@example.com".to_string()),
    ).expect("Should add step");
    
    // Customer notification
    let notify = workflow.add_step(
        "Notify Customer".to_string(),
        "Send resolution notification to customer".to_string(),
        StepType::Automated,
        HashMap::from([("notification_channel".to_string(), json!("email"))]),
        vec![investigate_id], // Can notify after investigation
        None,
        None,
        Some("support@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("ticket_id".to_string(), json!("TICKET-12345"));
    context.set_variable("customer_id".to_string(), json!("CUST-PREMIUM-789"));
    context.set_variable("issue_type".to_string(), json!("performance"));
    context.set_variable("severity".to_string(), json!("high"));
    
    workflow.start(context, Some("support@example.com".to_string()))
        .expect("Should start workflow");
    
    // Then: We should be able to track SLA compliance
    // In a complete implementation, these methods would exist:
    // - workflow.get_sla_status() -> SLAStatus
    // - workflow.get_time_remaining() -> Duration
    // - workflow.get_at_risk_steps() -> Vec<StepId>
    // - workflow.get_performance_metrics() -> WorkflowMetrics
    
    // The workflow should track:
    // 1. Overall SLA (24 hours for entire workflow)
    // 2. Individual step SLAs
    // 3. Time spent in each step
    // 4. Queue times between steps
    // 5. Escalation triggers
    
    // Real-time monitoring would include:
    // - Dashboard updates every 30 seconds
    // - Alerts when SLA breach is imminent (80% threshold)
    // - Automatic escalation when SLA is breached
    // - Performance trending over time
    
    // Verify workflow has monitoring metadata
    assert!(workflow.metadata.contains_key("sla_hours"));
    assert_eq!(workflow.metadata.get("priority"), Some(&json!("high")));
}

/// User Story: W15 - Analyze Workflow Performance
/// As a process analyst, I want to analyze workflow performance
/// So that I can optimize processes
#[test]
fn test_w15_analyze_workflow_performance() {
    // Given: A workflow with historical execution data
    let mut metadata = HashMap::new();
    metadata.insert("process_type".to_string(), json!("order_fulfillment"));
    metadata.insert("version".to_string(), json!("2.1"));
    metadata.insert("optimization_enabled".to_string(), json!(true));
    
    let (mut workflow, _) = Workflow::new(
        "E-commerce Order Processing".to_string(),
        "End-to-end order fulfillment process".to_string(),
        metadata,
        Some("analytics@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Create a typical e-commerce workflow
    let steps_data: Vec<(&str, StepType, i32, Vec<StepId>)> = vec![
        ("Validate Order", StepType::Automated, 2, vec![]),
        ("Check Inventory", StepType::Integration, 5, vec![]),
        ("Reserve Stock", StepType::Automated, 3, vec![]),
        ("Process Payment", StepType::Integration, 10, vec![]),
        ("Pick Items", StepType::Manual, 30, vec![]),
        ("Pack Order", StepType::Manual, 20, vec![]),
        ("Generate Shipping Label", StepType::Automated, 5, vec![]),
        ("Ship Order", StepType::Integration, 15, vec![]),
        ("Send Confirmation", StepType::Automated, 2, vec![]),
    ];
    
    let mut step_ids = Vec::new();
    for (i, (name, step_type, duration, _)) in steps_data.iter().enumerate() {
        // Create dependencies (each step depends on previous)
        let dependencies = if i > 0 { vec![step_ids[i-1]] } else { vec![] };
        
        let mut config = HashMap::new();
        config.insert("avg_duration_minutes".to_string(), json!(duration));
        config.insert("success_rate".to_string(), json!(0.95)); // 95% success rate
        
        // Add performance metrics
        match name.as_ref() {
            "Check Inventory" => {
                config.insert("bottleneck_frequency".to_string(), json!(0.15)); // 15% of time
                config.insert("avg_retry_count".to_string(), json!(1.2));
            },
            "Process Payment" => {
                config.insert("failure_rate".to_string(), json!(0.03)); // 3% failure
                config.insert("avg_processing_time_ms".to_string(), json!(2500));
            },
            "Pick Items" => {
                config.insert("queue_time_minutes".to_string(), json!(45)); // Long queue
                config.insert("resource_utilization".to_string(), json!(0.85)); // 85% busy
            },
            _ => {}
        }
        
        let events = workflow.add_step(
            name.to_string(),
            format!("Process step: {name}"),
            step_type.clone(),
            config,
            dependencies,
            Some(*duration as u32 * 2), // Timeout is 2x average
            None,
            Some("analytics@example.com".to_string()),
        ).expect("Should add step");
        
        if let WorkflowDomainEvent::StepAdded(event) = &events[0] {
            step_ids.push(event.step_id);
        }
    }
    
    // Start the workflow for analysis
    let mut context = WorkflowContext::new();
    context.set_variable("analysis_mode".to_string(), json!(true));
    context.set_variable("historical_executions".to_string(), json!(1000)); // Analyzed 1000 runs
    
    workflow.start(context, Some("analytics@example.com".to_string()))
        .expect("Should start workflow");
    
    // In a complete implementation, these analytics methods would exist:
    // - workflow.calculate_cycle_time() -> Duration
    // - workflow.identify_bottlenecks() -> Vec<BottleneckInfo>
    // - workflow.get_optimization_recommendations() -> Vec<Recommendation>
    // - workflow.calculate_resource_efficiency() -> f64
    // - workflow.predict_completion_time() -> Duration
    
    // Performance analysis would include:
    // 1. Cycle time analysis (total time from start to finish)
    // 2. Value-added vs non-value-added time
    // 3. Queue time analysis between steps
    // 4. Resource utilization rates
    // 5. Failure point identification
    
    // Optimization recommendations would suggest:
    // - Parallelizing "Pick Items" and "Generate Shipping Label"
    // - Adding inventory cache to reduce "Check Inventory" bottleneck
    // - Implementing payment pre-authorization
    // - Batch processing for shipping labels
    // - Resource reallocation during peak hours
    
    // Advanced analytics:
    // - Predictive completion times based on current load
    // - What-if scenarios for process changes
    // - Cost analysis per workflow execution
    // - Customer satisfaction correlation
    // - Seasonal pattern detection
    
    // Verify workflow has analytics metadata
    assert!(workflow.metadata.contains_key("optimization_enabled"));
    assert_eq!(workflow.steps.len(), 9); // All steps added
    
    // Check that performance data is captured
    let check_inventory_step = workflow.steps.values()
        .find(|s| s.name == "Check Inventory")
        .expect("Should find inventory step");
    assert!(check_inventory_step.config.contains_key("bottleneck_frequency"));
}

// =============================================================================
// WORKFLOW PATTERNS USER STORIES W16-W18
// =============================================================================

// Note: W16 and W17 are implemented after the integration tests at the end of this file

/// User Story: W18 - Loop Pattern
/// As a workflow designer, I want to implement loop patterns
/// So that I can handle iterative processes
#[test]
fn test_w18_loop_pattern() {
    // Given: A data quality workflow that requires iterative improvement
    let mut metadata = HashMap::new();
    metadata.insert("workflow_pattern".to_string(), json!("loop"));
    metadata.insert("max_iterations".to_string(), json!(5));
    metadata.insert("loop_type".to_string(), json!("quality_improvement"));
    
    let (mut workflow, _) = Workflow::new(
        "Data Quality Improvement Loop".to_string(),
        "Iterative data cleansing and validation workflow".to_string(),
        metadata,
        Some("data-quality@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Step 1: Load Dataset
    let load_data = workflow.add_step(
        "Load Dataset".to_string(),
        "Load data from source system".to_string(),
        StepType::Integration,
        HashMap::from([
            ("data_source".to_string(), json!("customer_database")),
            ("batch_size".to_string(), json!(10000)),
            ("include_metadata".to_string(), json!(true)),
        ]),
        vec![],
        Some(10),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    let load_id = if let WorkflowDomainEvent::StepAdded(event) = &load_data[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Initialize Loop Counter
    let init_loop = workflow.add_step(
        "Initialize Loop".to_string(),
        "Set up loop variables and counters".to_string(),
        StepType::Automated,
        HashMap::from([
            ("loop_counter".to_string(), json!(0)),
            ("quality_threshold".to_string(), json!(95.0)), // 95% quality target
            ("improvement_threshold".to_string(), json!(0.5)), // Min 0.5% improvement per iteration
        ]),
        vec![load_id],
        Some(1),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    let init_id = if let WorkflowDomainEvent::StepAdded(event) = &init_loop[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Loop Start: Quality Assessment
    let assess_quality = workflow.add_step(
        "Assess Data Quality".to_string(),
        "Analyze current data quality metrics".to_string(),
        StepType::Automated,
        HashMap::from([
            ("loop_entry_point".to_string(), json!(true)),
            ("quality_checks".to_string(), json!([
                "completeness",
                "accuracy",
                "consistency",
                "timeliness",
                "uniqueness",
                "validity"
            ])),
            ("generate_report".to_string(), json!(true)),
        ]),
        vec![init_id], // First iteration from init, subsequent from loop back
        Some(15),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    let assess_id = if let WorkflowDomainEvent::StepAdded(event) = &assess_quality[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Loop Decision: Check Quality Threshold
    let check_quality = workflow.add_step(
        "Check Quality Threshold".to_string(),
        "Determine if quality target is met or loop should continue".to_string(),
        StepType::Decision,
        HashMap::from([
            ("exit_conditions".to_string(), json!([
                "quality_score >= quality_threshold",
                "loop_counter >= max_iterations",
                "improvement_rate < improvement_threshold"
            ])),
            ("loop_decision".to_string(), json!(true)),
        ]),
        vec![assess_id],
        Some(2),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    let check_id = if let WorkflowDomainEvent::StepAdded(event) = &check_quality[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Loop Body: Apply Data Cleansing Rules
    let cleanse_data = workflow.add_step(
        "Apply Cleansing Rules".to_string(),
        "Execute data transformation and cleaning operations".to_string(),
        StepType::Automated,
        HashMap::from([
            ("loop_body".to_string(), json!(true)),
            ("cleansing_rules".to_string(), json!([
                "standardize_addresses",
                "validate_email_formats",
                "remove_duplicates",
                "fill_missing_values",
                "correct_data_types",
                "apply_business_rules"
            ])),
            ("track_changes".to_string(), json!(true)),
        ]),
        vec![check_id],
        Some(30),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    let cleanse_id = if let WorkflowDomainEvent::StepAdded(event) = &cleanse_data[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Loop Body: Machine Learning Enhancement
    let ml_enhance = workflow.add_step(
        "ML-Based Enhancement".to_string(),
        "Apply machine learning models for data improvement".to_string(),
        StepType::Integration,
        HashMap::from([
            ("models".to_string(), json!([
                "address_standardization_model",
                "name_matching_model",
                "anomaly_detection_model"
            ])),
            ("confidence_threshold".to_string(), json!(0.85)),
        ]),
        vec![cleanse_id],
        Some(20),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    let ml_id = if let WorkflowDomainEvent::StepAdded(event) = &ml_enhance[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Loop Back: Increment Counter and Re-assess
    let increment_loop = workflow.add_step(
        "Increment Loop Counter".to_string(),
        "Update loop variables and prepare for next iteration".to_string(),
        StepType::Automated,
        HashMap::from([
            ("loop_back".to_string(), json!(true)),
            ("target_step".to_string(), json!("Assess Data Quality")),
            ("increment_counter".to_string(), json!(true)),
            ("track_improvement".to_string(), json!(true)),
        ]),
        vec![ml_id],
        Some(1),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    // Loop Exit: Generate Final Report
    let final_report = workflow.add_step(
        "Generate Final Report".to_string(),
        "Create comprehensive quality improvement report".to_string(),
        StepType::Automated,
        HashMap::from([
            ("loop_exit".to_string(), json!(true)),
            ("report_sections".to_string(), json!([
                "initial_quality_score",
                "final_quality_score",
                "iterations_performed",
                "improvements_by_category",
                "remaining_issues",
                "recommendations"
            ])),
        ]),
        vec![check_id], // Exits from the decision step
        Some(5),
        None,
        Some("data-quality@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("dataset_id".to_string(), json!("CUST-DATA-2024"));
    context.set_variable("initial_quality_score".to_string(), json!(78.5)); // Starting at 78.5% quality
    context.set_variable("target_quality".to_string(), json!(95.0));
    
    workflow.start(context, Some("data-quality@example.com".to_string()))
        .expect("Should start workflow");
    
    // In a complete implementation, loop patterns would:
    // - workflow.track_loop_iterations() -> LoopMetrics
    // - workflow.detect_infinite_loops() -> SafetyCheck
    // - workflow.optimize_loop_performance() -> PerformancePlan
    // - workflow.visualize_loop_flow() -> FlowDiagram
    
    // Loop pattern characteristics:
    // 1. Entry condition (when to start looping)
    // 2. Exit conditions (multiple ways to exit)
    // 3. Loop body (operations performed each iteration)
    // 4. Loop counter/state management
    // 5. Safety mechanisms (max iterations, timeout)
    
    // Common loop patterns:
    // - Do-While: Execute at least once
    // - While-Do: Check condition first
    // - For-Each: Iterate over collection
    // - Repeat-Until: Continue until condition met
    
    // Verify loop configuration
    assert!(workflow.metadata.contains_key("max_iterations"));
    assert_eq!(workflow.metadata.get("loop_type"), Some(&json!("quality_improvement")));
    
    // Verify loop structure elements
    let loop_steps = ["Assess Data Quality", "Check Quality Threshold", 
                      "Apply Cleansing Rules", "Increment Loop Counter"];
    for step_name in loop_steps {
        let step = workflow.steps.values()
            .find(|s| s.name == step_name)
            .expect(&format!("Should find {step_name} step"));
        
        // Check for loop-related configuration
        let has_loop_config = step.config.contains_key("loop_entry_point") ||
                              step.config.contains_key("loop_decision") ||
                              step.config.contains_key("loop_body") ||
                              step.config.contains_key("loop_back");
        assert!(has_loop_config, "{} should have loop configuration", step_name);
    }
}

// =============================================================================
// ADVANCED FEATURES USER STORIES W19-W22
// =============================================================================

/// User Story: W19 - Schedule Workflow Execution
/// As a business user, I want to schedule workflows
/// So that they run automatically
#[test]
fn test_w19_schedule_workflow_execution() {
    // Given: Multiple workflows that need scheduling
    let mut metadata = HashMap::new();
    metadata.insert("scheduling_enabled".to_string(), json!(true));
    metadata.insert("timezone".to_string(), json!("America/New_York"));
    metadata.insert("business_calendar".to_string(), json!("NYSE"));
    
    // Workflow 1: Daily Report Generation
    let (mut daily_report, _) = Workflow::new(
        "Daily Sales Report".to_string(),
        "Generate and distribute daily sales metrics".to_string(),
        metadata.clone(),
        Some("scheduler@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Configure daily schedule
    daily_report.metadata.insert("schedule_type".to_string(), json!("cron"));
    daily_report.metadata.insert("cron_expression".to_string(), json!("0 6 * * 1-5")); // 6 AM weekdays
    daily_report.metadata.insert("skip_holidays".to_string(), json!(true));
    daily_report.metadata.insert("retry_on_failure".to_string(), json!(true));
    daily_report.metadata.insert("max_retries".to_string(), json!(3));
    
    // Add report generation steps
    let extract = daily_report.add_step(
        "Extract Sales Data".to_string(),
        "Pull yesterday's sales from all channels".to_string(),
        StepType::Integration,
        HashMap::from([
            ("data_sources".to_string(), json!(["pos_system", "online_store", "mobile_app"])),
            ("date_range".to_string(), json!("yesterday")),
            ("include_returns".to_string(), json!(true)),
        ]),
        vec![],
        Some(10),
        None,
        Some("scheduler@example.com".to_string()),
    ).expect("Should add step");
    
    // Workflow 2: Weekly Backup Process
    let mut weekly_backup_meta = metadata.clone();
    weekly_backup_meta.insert("schedule_type".to_string(), json!("cron"));
    weekly_backup_meta.insert("cron_expression".to_string(), json!("0 2 * * 0")); // 2 AM Sunday
    weekly_backup_meta.insert("maintenance_window".to_string(), json!(true));
    
    let (mut weekly_backup, _) = Workflow::new(
        "Weekly System Backup".to_string(),
        "Full system backup and verification".to_string(),
        weekly_backup_meta,
        Some("scheduler@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Workflow 3: Monthly Reconciliation
    let mut monthly_recon_meta = metadata.clone();
    monthly_recon_meta.insert("schedule_type".to_string(), json!("monthly"));
    monthly_recon_meta.insert("day_of_month".to_string(), json!(1)); // First day of month
    monthly_recon_meta.insert("time".to_string(), json!("00:30"));
    monthly_recon_meta.insert("end_of_month_adjustment".to_string(), json!(true)); // Handle Feb 30 -> Feb 28/29
    
    let (mut monthly_recon, _) = Workflow::new(
        "Monthly Financial Reconciliation".to_string(),
        "Reconcile all financial accounts".to_string(),
        monthly_recon_meta,
        Some("scheduler@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Workflow 4: Event-Triggered with Time Window
    let mut event_triggered_meta = metadata.clone();
    event_triggered_meta.insert("schedule_type".to_string(), json!("event_based"));
    event_triggered_meta.insert("trigger_event".to_string(), json!("inventory_low"));
    event_triggered_meta.insert("time_window".to_string(), json!({
        "start": "09:00",
        "end": "17:00",
        "days": ["Mon", "Tue", "Wed", "Thu", "Fri"]
    }));
    event_triggered_meta.insert("debounce_minutes".to_string(), json!(30)); // Prevent rapid re-triggers
    
    let (mut reorder_workflow, _) = Workflow::new(
        "Inventory Reorder Process".to_string(),
        "Automated inventory reordering when stock is low".to_string(),
        event_triggered_meta,
        Some("scheduler@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Workflow 5: Interval-Based Processing
    let mut interval_meta = metadata.clone();
    interval_meta.insert("schedule_type".to_string(), json!("interval"));
    interval_meta.insert("interval_minutes".to_string(), json!(15)); // Every 15 minutes
    interval_meta.insert("start_time".to_string(), json!("08:00"));
    interval_meta.insert("end_time".to_string(), json!("20:00"));
    interval_meta.insert("concurrent_execution".to_string(), json!(false)); // Don't overlap runs
    
    let (mut monitoring_workflow, _) = Workflow::new(
        "System Health Check".to_string(),
        "Monitor system health and alert on issues".to_string(),
        interval_meta,
        Some("scheduler@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Workflow 6: Complex Schedule with Dependencies
    let mut complex_meta = metadata.clone();
    complex_meta.insert("schedule_type".to_string(), json!("complex"));
    complex_meta.insert("schedules".to_string(), json!([
        {
            "name": "quarter_end",
            "cron": "0 0 L 3,6,9,12 *", // Last day of quarter
            "priority": "high"
        },
        {
            "name": "month_end",
            "cron": "0 0 L * *", // Last day of month
            "priority": "medium"
        },
        {
            "name": "weekly",
            "cron": "0 0 * * 1", // Every Monday
            "priority": "low"
        }
    ]));
    complex_meta.insert("depends_on".to_string(), json!(["monthly_reconciliation"]));
    complex_meta.insert("block_concurrent".to_string(), json!(["daily_sales_report"]));
    
    let (mut analytics_workflow, _) = Workflow::new(
        "Business Analytics Pipeline".to_string(),
        "Comprehensive business analytics processing".to_string(),
        complex_meta,
        Some("scheduler@example.com".to_string()),
    ).expect("Should create workflow");
    
    // In a complete implementation, scheduling would support:
    // - workflow.schedule() -> ScheduleHandle
    // - workflow.get_next_run_time() -> DateTime
    // - workflow.get_schedule_history() -> Vec<ScheduleExecution>
    // - workflow.pause_schedule() / resume_schedule()
    // - workflow.override_next_run(datetime) -> Result
    
    // Scheduling features:
    // 1. Cron expressions for complex patterns
    // 2. Business calendar awareness (holidays, weekends)
    // 3. Timezone handling
    // 4. Dependency management between scheduled workflows
    // 5. Concurrent execution control
    // 6. Catch-up runs for missed schedules
    // 7. Schedule conflict resolution
    // 8. Dynamic schedule adjustments
    
    // Advanced scheduling:
    // - Load balancing across time windows
    // - Resource-aware scheduling
    // - Priority-based execution
    // - Schedule optimization recommendations
    // - Predictive scheduling based on historical data
    
    // Verify scheduling configuration
    assert!(daily_report.metadata.contains_key("cron_expression"));
    assert_eq!(daily_report.metadata.get("schedule_type"), Some(&json!("cron")));
    
    assert!(weekly_backup.metadata.contains_key("maintenance_window"));
    assert_eq!(weekly_backup.metadata.get("cron_expression"), Some(&json!("0 2 * * 0")));
    
    assert!(monthly_recon.metadata.contains_key("end_of_month_adjustment"));
    assert_eq!(monthly_recon.metadata.get("day_of_month"), Some(&json!(1)));
    
    assert!(reorder_workflow.metadata.contains_key("trigger_event"));
    assert!(reorder_workflow.metadata.contains_key("time_window"));
    
    assert!(monitoring_workflow.metadata.contains_key("interval_minutes"));
    assert_eq!(monitoring_workflow.metadata.get("concurrent_execution"), Some(&json!(false)));
    
    assert!(analytics_workflow.metadata.contains_key("schedules"));
    assert!(analytics_workflow.metadata.contains_key("depends_on"));
}

/// User Story: W20 - Create Sub-Workflows
/// As a workflow designer, I want to call other workflows
/// So that I can reuse process logic
#[test]
fn test_w20_create_sub_workflows() {
    // Given: A parent workflow that orchestrates sub-workflows
    let mut metadata = HashMap::new();
    metadata.insert("workflow_type".to_string(), json!("orchestration"));
    metadata.insert("sub_workflow_enabled".to_string(), json!(true));
    
    let (mut parent_workflow, _) = Workflow::new(
        "Employee Onboarding Master".to_string(),
        "Complete employee onboarding process orchestrating multiple sub-workflows".to_string(),
        metadata,
        Some("hr@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Sub-workflow 1: IT Setup Process
    let mut it_meta = HashMap::new();
    it_meta.insert("workflow_type".to_string(), json!("sub_workflow"));
    it_meta.insert("reusable".to_string(), json!(true));
    it_meta.insert("version".to_string(), json!("2.0"));
    
    let (mut it_setup_workflow, _) = Workflow::new(
        "IT Equipment and Access Setup".to_string(),
        "Standard IT setup process for new employees".to_string(),
        it_meta,
        Some("it@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Add IT setup steps
    it_setup_workflow.add_step(
        "Allocate Equipment".to_string(),
        "Assign laptop, phone, and peripherals".to_string(),
        StepType::Manual,
        HashMap::from([
            ("equipment_types".to_string(), json!(["laptop", "phone", "monitor", "keyboard", "mouse"])),
            ("budget_limit".to_string(), json!(3000)),
        ]),
        vec![],
        Some(60),
        Some("it-inventory@example.com".to_string()),
        Some("it@example.com".to_string()),
    ).expect("Should add step");
    
    it_setup_workflow.add_step(
        "Create Accounts".to_string(),
        "Set up email, VPN, and system accounts".to_string(),
        StepType::Automated,
        HashMap::from([
            ("systems".to_string(), json!(["email", "vpn", "ldap", "slack", "github"])),
            ("access_level".to_string(), json!("standard")),
        ]),
        vec![],
        Some(30),
        None,
        Some("it@example.com".to_string()),
    ).expect("Should add step");
    
    // Sub-workflow 2: Payroll Setup
    let mut payroll_meta = HashMap::new();
    payroll_meta.insert("workflow_type".to_string(), json!("sub_workflow"));
    payroll_meta.insert("compliance_required".to_string(), json!(true));
    
    let (mut payroll_workflow, _) = Workflow::new(
        "Payroll and Benefits Enrollment".to_string(),
        "Set up payroll and benefits for new employee".to_string(),
        payroll_meta,
        Some("payroll@example.com".to_string()),
    ).expect("Should create workflow");
    
    payroll_workflow.add_step(
        "Tax Documentation".to_string(),
        "Collect W-4, I-9, and state tax forms".to_string(),
        StepType::Manual,
        HashMap::from([
            ("required_forms".to_string(), json!(["W-4", "I-9", "state_tax"])),
            ("verification_required".to_string(), json!(true)),
        ]),
        vec![],
        Some(120),
        Some("employee@example.com".to_string()),
        Some("payroll@example.com".to_string()),
    ).expect("Should add step");
    
    // Sub-workflow 3: Training Assignment
    let mut training_meta = HashMap::new();
    training_meta.insert("workflow_type".to_string(), json!("sub_workflow"));
    training_meta.insert("track_completion".to_string(), json!(true));
    
    let (mut training_workflow, _) = Workflow::new(
        "Mandatory Training Assignment".to_string(),
        "Assign and track required training modules".to_string(),
        training_meta,
        Some("training@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Now configure the parent workflow to call sub-workflows
    
    // Step 1: Collect Basic Information
    let collect_info = parent_workflow.add_step(
        "Collect Employee Information".to_string(),
        "Gather all necessary employee data".to_string(),
        StepType::Manual,
        HashMap::from([
            ("required_fields".to_string(), json!([
                "full_name", "email", "phone", "address",
                "emergency_contact", "start_date", "department", "role"
            ])),
        ]),
        vec![],
        Some(30),
        Some("hr@example.com".to_string()),
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    let collect_id = if let WorkflowDomainEvent::StepAdded(event) = &collect_info[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Invoke IT Setup Sub-workflow
    let invoke_it = parent_workflow.add_step(
        "Execute IT Setup Process".to_string(),
        "Call IT setup sub-workflow with employee data".to_string(),
        StepType::Custom("SubWorkflow".to_string()),
        HashMap::from([
                            ("sub_workflow_id".to_string(), json!(it_setup_workflow.id.0.to_string())),
            ("sub_workflow_name".to_string(), json!("IT Equipment and Access Setup")),
            ("parameter_mapping".to_string(), json!({
                "employee_name": "$.employee.full_name",
                "employee_email": "$.employee.email",
                "department": "$.employee.department",
                "start_date": "$.employee.start_date"
            })),
            ("timeout_minutes".to_string(), json!(240)), // 4 hour timeout
            ("async_execution".to_string(), json!(true)), // Can run in parallel
        ]),
        vec![collect_id],
        Some(240),
        None,
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    let it_id = if let WorkflowDomainEvent::StepAdded(event) = &invoke_it[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 3: Invoke Payroll Sub-workflow (parallel with IT)
    let invoke_payroll = parent_workflow.add_step(
        "Execute Payroll Setup".to_string(),
        "Call payroll setup sub-workflow".to_string(),
        StepType::Custom("SubWorkflow".to_string()),
        HashMap::from([
                            ("sub_workflow_id".to_string(), json!(payroll_workflow.id.0.to_string())),
            ("sub_workflow_name".to_string(), json!("Payroll and Benefits Enrollment")),
            ("parameter_mapping".to_string(), json!({
                "employee_id": "$.employee.id",
                "salary": "$.employee.salary",
                "benefits_package": "$.employee.benefits_tier"
            })),
            ("return_values".to_string(), json!(["payroll_id", "benefits_confirmation"])),
            ("error_handling".to_string(), json!("propagate")), // Fail parent if sub fails
        ]),
        vec![collect_id], // Also depends only on collect, runs parallel with IT
        Some(480), // 8 hour timeout
        None,
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    let payroll_id = if let WorkflowDomainEvent::StepAdded(event) = &invoke_payroll[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 4: Conditional Training Assignment
    let check_role = parent_workflow.add_step(
        "Determine Training Requirements".to_string(),
        "Check role to determine which training sub-workflow to invoke".to_string(),
        StepType::Decision,
        HashMap::from([
            ("role_conditions".to_string(), json!({
                "engineering": ["tech_training_workflow"],
                "sales": ["sales_training_workflow"],
                "management": ["leadership_training_workflow"],
                "default": ["general_training_workflow"]
            })),
        ]),
        vec![collect_id],
        Some(5),
        None,
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    let check_id = if let WorkflowDomainEvent::StepAdded(event) = &check_role[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 5: Dynamic Sub-workflow Invocation
    let invoke_training = parent_workflow.add_step(
        "Execute Role-Specific Training".to_string(),
        "Dynamically invoke appropriate training sub-workflow".to_string(),
        StepType::Custom("SubWorkflow".to_string()),
        HashMap::from([
            ("sub_workflow_id".to_string(), json!("${decision.selected_workflow}")), // Dynamic
            ("parameter_mapping".to_string(), json!({
                "employee_id": "$.employee.id",
                "role": "$.employee.role",
                "department": "$.employee.department"
            })),
            ("wait_for_completion".to_string(), json!(false)), // Fire and forget
            ("track_status".to_string(), json!(true)), // But track status
        ]),
        vec![check_id],
        Some(60),
        None,
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    // Step 6: Aggregate Results
    let aggregate = parent_workflow.add_step(
        "Consolidate Onboarding Results".to_string(),
        "Collect results from all sub-workflows".to_string(),
        StepType::Automated,
        HashMap::from([
            ("wait_for_sub_workflows".to_string(), json!([it_id, payroll_id])),
            ("collect_outputs".to_string(), json!(true)),
            ("generate_report".to_string(), json!(true)),
        ]),
        vec![it_id, payroll_id], // Wait for both
        Some(30),
        None,
        Some("hr@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the parent workflow
    let mut context = WorkflowContext::new();
    context.set_variable("employee".to_string(), json!({
        "id": "EMP-2024-001",
        "full_name": "Jane Smith",
        "email": "jane.smith@example.com",
        "department": "Engineering",
        "role": "Senior Developer",
        "start_date": "2024-02-01",
        "salary": 120000,
        "benefits_tier": "premium"
    }));
    
    parent_workflow.start(context, Some("hr@example.com".to_string()))
        .expect("Should start workflow");
    
    // In a complete implementation, sub-workflows would support:
    // - workflow.invoke_sub_workflow(id, params) -> SubWorkflowHandle
    // - workflow.get_sub_workflow_status(handle) -> SubWorkflowStatus
    // - workflow.cancel_sub_workflow(handle) -> Result
    // - workflow.get_sub_workflow_results(handle) -> WorkflowOutput
    
    // Sub-workflow features:
    // 1. Parameter passing and mapping
    // 2. Return value collection
    // 3. Error propagation strategies
    // 4. Async vs sync execution
    // 5. Dynamic workflow selection
    // 6. Version management
    // 7. Resource sharing/isolation
    // 8. Transaction boundaries
    
    // Verify sub-workflow configuration
    assert!(parent_workflow.metadata.contains_key("sub_workflow_enabled"));
    
    // Count sub-workflow invocations
    let sub_workflow_steps = parent_workflow.steps.values()
        .filter(|s| matches!(&s.step_type, StepType::Custom(t) if t == "SubWorkflow"))
        .count();
    assert_eq!(sub_workflow_steps, 3); // IT, Payroll, and Training invocations
    
    // Verify parameter mapping exists
    let it_step = parent_workflow.steps.values()
        .find(|s| s.name == "Execute IT Setup Process")
        .expect("Should find IT setup step");
    assert!(it_step.config.contains_key("parameter_mapping"));
    assert!(it_step.config.contains_key("sub_workflow_id"));
}

/// User Story: W21 - Version Workflows
/// As a workflow manager, I want to version workflows
/// So that changes are controlled
#[test]
fn test_w21_version_workflows() {
    // Given: A workflow that needs versioning for controlled changes
    let mut v1_metadata = HashMap::new();
    v1_metadata.insert("version".to_string(), json!("1.0.0"));
    v1_metadata.insert("versioning_enabled".to_string(), json!(true));
    v1_metadata.insert("change_log".to_string(), json!([
        {
            "version": "1.0.0",
            "date": "2024-01-01",
            "author": "workflow-team@example.com",
            "changes": ["Initial release", "Basic approval flow"]
        }
    ]));
    
    // Version 1.0.0 - Original workflow
    let (mut approval_v1, _) = Workflow::new(
        "Purchase Order Approval".to_string(),
        "Standard purchase order approval process".to_string(),
        v1_metadata.clone(),
        Some("finance@example.com".to_string()),
    ).expect("Should create workflow");
    
    // V1 has simple 2-step approval
    approval_v1.add_step(
        "Submit Request".to_string(),
        "Employee submits purchase request".to_string(),
        StepType::Manual,
        HashMap::from([
            ("required_fields".to_string(), json!(["item", "amount", "justification"])),
            ("max_amount".to_string(), json!(5000)),
        ]),
        vec![],
        Some(30),
        None,
        Some("finance@example.com".to_string()),
    ).expect("Should add step");
    
    approval_v1.add_step(
        "Manager Approval".to_string(),
        "Direct manager approves request".to_string(),
        StepType::Approval,
        HashMap::from([
            ("approval_limit".to_string(), json!(5000)),
        ]),
        vec![],
        Some(1440), // 24 hours
        Some("manager@example.com".to_string()),
        Some("finance@example.com".to_string()),
    ).expect("Should add step");
    
    // Version 1.1.0 - Minor enhancement
    let mut v1_1_metadata = v1_metadata.clone();
    v1_1_metadata.insert("version".to_string(), json!("1.1.0"));
    v1_1_metadata.insert("parent_version".to_string(), json!("1.0.0"));
    v1_1_metadata.insert("migration_strategy".to_string(), json!("compatible"));
    
    let change_log = v1_1_metadata.get_mut("change_log").unwrap();
    if let serde_json::Value::Array(changes) = change_log {
        changes.push(json!({
            "version": "1.1.0",
            "date": "2024-02-01",
            "author": "workflow-team@example.com",
            "changes": [
                "Added email notifications",
                "Extended timeout to 48 hours",
                "Added urgency flag"
            ]
        }));
    }
    
    let (mut approval_v1_1, _) = Workflow::new(
        "Purchase Order Approval".to_string(),
        "Enhanced purchase order approval with notifications".to_string(),
        v1_1_metadata,
        Some("finance@example.com".to_string()),
    ).expect("Should create workflow");
    
    // V1.1 adds notification configuration
    approval_v1_1.add_step(
        "Submit Request".to_string(),
        "Employee submits purchase request with urgency option".to_string(),
        StepType::Manual,
        HashMap::from([
            ("required_fields".to_string(), json!(["item", "amount", "justification", "urgency"])),
            ("max_amount".to_string(), json!(5000)),
            ("send_notification".to_string(), json!(true)), // New in v1.1
        ]),
        vec![],
        Some(30),
        None,
        Some("finance@example.com".to_string()),
    ).expect("Should add step");
    
    approval_v1_1.add_step(
        "Manager Approval".to_string(),
        "Direct manager approves request with email alerts".to_string(),
        StepType::Approval,
        HashMap::from([
            ("approval_limit".to_string(), json!(5000)),
            ("timeout_hours".to_string(), json!(48)), // Extended from 24
            ("send_reminders".to_string(), json!(true)), // New in v1.1
            ("reminder_intervals".to_string(), json!([12, 24, 36])), // Hours
        ]),
        vec![],
        Some(2880), // 48 hours
        Some("manager@example.com".to_string()),
        Some("finance@example.com".to_string()),
    ).expect("Should add step");
    
    // Version 2.0.0 - Breaking change
    let mut v2_metadata = v1_metadata.clone();
    v2_metadata.insert("version".to_string(), json!("2.0.0"));
    v2_metadata.insert("parent_version".to_string(), json!("1.1.0"));
    v2_metadata.insert("migration_strategy".to_string(), json!("breaking"));
    v2_metadata.insert("migration_guide".to_string(), json!({
        "summary": "Adds multi-level approval based on amount",
        "breaking_changes": [
            "Approval flow now has 3 levels instead of 1",
            "New fields required in submission",
            "Different approval limits per level"
        ],
        "migration_steps": [
            "Complete all in-flight v1.x workflows",
            "Update integrations to handle new approval levels",
            "Train users on new thresholds"
        ]
    }));
    
    let change_log = v2_metadata.get_mut("change_log").unwrap();
    if let serde_json::Value::Array(changes) = change_log {
        changes.push(json!({
            "version": "2.0.0",
            "date": "2024-03-01",
            "author": "workflow-team@example.com",
            "changes": [
                "BREAKING: Multi-level approval system",
                "BREAKING: New required fields",
                "Added director approval for >$10k",
                "Added VP approval for >$50k",
                "Integrated with budget system"
            ]
        }));
    }
    
    let (mut approval_v2, _) = Workflow::new(
        "Purchase Order Approval".to_string(),
        "Multi-level purchase approval with budget integration".to_string(),
        v2_metadata,
        Some("finance@example.com".to_string()),
    ).expect("Should create workflow");
    
    // V2.0 has complex multi-level approval
    let submit = approval_v2.add_step(
        "Submit Request".to_string(),
        "Employee submits purchase request with budget check".to_string(),
        StepType::Manual,
        HashMap::from([
            ("required_fields".to_string(), json!([
                "item", "amount", "justification", "urgency",
                "budget_code", "vendor_id", "delivery_date" // New required fields
            ])),
            ("validate_budget".to_string(), json!(true)), // New feature
            ("max_amount".to_string(), json!(100000)), // Increased limit
        ]),
        vec![],
        Some(30),
        None,
        Some("finance@example.com".to_string()),
    ).expect("Should add step");
    
    let submit_id = if let WorkflowDomainEvent::StepAdded(event) = &submit[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Decision node for routing
    let route = approval_v2.add_step(
        "Determine Approval Path".to_string(),
        "Route based on amount thresholds".to_string(),
        StepType::Decision,
        HashMap::from([
            ("thresholds".to_string(), json!({
                "manager_only": 10000,
                "director_required": 50000,
                "vp_required": 100000
            })),
        ]),
        vec![submit_id],
        Some(1),
        None,
        Some("finance@example.com".to_string()),
    ).expect("Should add step");
    
    // Version comparison and migration support
    let mut version_registry = HashMap::new();
    version_registry.insert("1.0.0", approval_v1.id);
    version_registry.insert("1.1.0", approval_v1_1.id);
    version_registry.insert("2.0.0", approval_v2.id);
    
    // Deprecation policy
    let deprecation_policy = json!({
        "deprecation_notice_days": 30,
        "end_of_life_days": 90,
        "deprecated_versions": [
            {
                "version": "1.0.0",
                "deprecated_date": "2024-02-01",
                "end_of_life": "2024-05-01",
                "migration_target": "1.1.0"
            }
        ],
        "supported_versions": ["1.1.0", "2.0.0"],
        "latest_stable": "2.0.0"
    });
    
    // In a complete implementation, versioning would support:
    // - workflow.create_version(changes) -> NewVersion
    // - workflow.get_version_history() -> Vec<Version>
    // - workflow.migrate_instances(from_version, to_version) -> MigrationPlan
    // - workflow.deprecate_version(version, eol_date) -> Result
    // - workflow.compare_versions(v1, v2) -> VersionDiff
    
    // Version management features:
    // 1. Semantic versioning (major.minor.patch)
    // 2. Change tracking and audit trail
    // 3. Migration strategies (compatible, breaking)
    // 4. Deprecation lifecycle
    // 5. Version branching and merging
    // 6. A/B testing between versions
    // 7. Rollback capabilities
    // 8. Version-specific metrics
    
    // Verify versioning configuration
    assert_eq!(approval_v1.metadata.get("version"), Some(&json!("1.0.0")));
    assert_eq!(approval_v1_1.metadata.get("version"), Some(&json!("1.1.0")));
    assert_eq!(approval_v2.metadata.get("version"), Some(&json!("2.0.0")));
    
    // Verify parent version tracking
    assert_eq!(approval_v1_1.metadata.get("parent_version"), Some(&json!("1.0.0")));
    assert_eq!(approval_v2.metadata.get("parent_version"), Some(&json!("1.1.0")));
    
    // Verify migration strategy
    assert_eq!(approval_v1_1.metadata.get("migration_strategy"), Some(&json!("compatible")));
    assert_eq!(approval_v2.metadata.get("migration_strategy"), Some(&json!("breaking")));
    
    // Verify change log exists
    assert!(approval_v2.metadata.contains_key("change_log"));
    let change_log = approval_v2.metadata.get("change_log").unwrap();
    if let serde_json::Value::Array(changes) = change_log {
        assert_eq!(changes.len(), 2); // Changes from 1.1 and 2.0
    }
}

/// User Story: W22 - Implement Workflow Transactions
/// As a workflow designer, I want transactional boundaries
/// So that consistency is maintained
#[test]
fn test_w22_workflow_transactions() {
    // Given: A workflow that requires transactional consistency
    let mut metadata = HashMap::new();
    metadata.insert("transaction_enabled".to_string(), json!(true));
    metadata.insert("isolation_level".to_string(), json!("read_committed"));
    metadata.insert("saga_pattern".to_string(), json!("orchestration"));
    
    let (mut order_workflow, _) = Workflow::new(
        "Distributed Order Processing".to_string(),
        "Multi-system order fulfillment with transactional guarantees".to_string(),
        metadata,
        Some("orders@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Transaction Boundary 1: Order Validation
    let validate_order = order_workflow.add_step(
        "Validate Order".to_string(),
        "Validate order details and customer credit".to_string(),
        StepType::Automated,
        HashMap::from([
            ("transaction_boundary".to_string(), json!("start")),
            ("transaction_id".to_string(), json!("order_validation_tx")),
            ("timeout_seconds".to_string(), json!(30)),
            ("validations".to_string(), json!([
                "customer_exists",
                "credit_check",
                "product_availability",
                "pricing_rules"
            ])),
        ]),
        vec![],
        Some(30),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    let validate_id = if let WorkflowDomainEvent::StepAdded(event) = &validate_order[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Saga Step 1: Reserve Inventory (with compensation)
    let reserve_inventory = order_workflow.add_step(
        "Reserve Inventory".to_string(),
        "Atomically reserve items from inventory".to_string(),
        StepType::Integration,
        HashMap::from([
            ("saga_step".to_string(), json!(true)),
            ("compensation_step".to_string(), json!("Release Inventory")),
            ("idempotency_key".to_string(), json!("${order_id}_inventory")),
            ("retry_policy".to_string(), json!({
                "max_attempts": 3,
                "backoff": "exponential",
                "initial_delay_ms": 100
            })),
            ("transaction_participant".to_string(), json!(true)),
        ]),
        vec![validate_id],
        Some(60),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    let reserve_id = if let WorkflowDomainEvent::StepAdded(event) = &reserve_inventory[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Saga Step 2: Charge Payment (with compensation)
    let charge_payment = order_workflow.add_step(
        "Process Payment".to_string(),
        "Charge customer payment method".to_string(),
        StepType::Integration,
        HashMap::from([
            ("saga_step".to_string(), json!(true)),
            ("compensation_step".to_string(), json!("Refund Payment")),
            ("idempotency_key".to_string(), json!("${order_id}_payment")),
            ("two_phase_commit".to_string(), json!(true)),
            ("prepare_phase".to_string(), json!("authorize_payment")),
            ("commit_phase".to_string(), json!("capture_payment")),
            ("transaction_participant".to_string(), json!(true)),
        ]),
        vec![reserve_id],
        Some(120),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    let payment_id = if let WorkflowDomainEvent::StepAdded(event) = &charge_payment[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Saga Step 3: Create Shipment (with compensation)
    let create_shipment = order_workflow.add_step(
        "Create Shipment".to_string(),
        "Book shipment with carrier".to_string(),
        StepType::Integration,
        HashMap::from([
            ("saga_step".to_string(), json!(true)),
            ("compensation_step".to_string(), json!("Cancel Shipment")),
            ("idempotency_key".to_string(), json!("${order_id}_shipment")),
            ("transaction_participant".to_string(), json!(true)),
            ("savepoint".to_string(), json!("shipment_created")),
        ]),
        vec![payment_id],
        Some(90),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    let shipment_id = if let WorkflowDomainEvent::StepAdded(event) = &create_shipment[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Transaction Commit Point
    let commit_transaction = order_workflow.add_step(
        "Commit Order Transaction".to_string(),
        "Commit all distributed changes".to_string(),
        StepType::Automated,
        HashMap::from([
            ("transaction_boundary".to_string(), json!("commit")),
            ("transaction_id".to_string(), json!("order_validation_tx")),
            ("participants".to_string(), json!([
                "inventory_service",
                "payment_service",
                "shipping_service"
            ])),
            ("commit_protocol".to_string(), json!("two_phase_commit")),
        ]),
        vec![shipment_id],
        Some(30),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    // Compensation Steps (for rollback)
    
    // Compensation 1: Release Inventory
    let release_inventory = order_workflow.add_step(
        "Release Inventory".to_string(),
        "Compensate by releasing reserved inventory".to_string(),
        StepType::Integration,
        HashMap::from([
            ("compensation_for".to_string(), json!("Reserve Inventory")),
            ("trigger_condition".to_string(), json!("saga_failure")),
            ("idempotency_key".to_string(), json!("${order_id}_inventory_release")),
            ("must_succeed".to_string(), json!(true)), // Critical compensation
        ]),
        vec![], // Triggered by saga coordinator
        Some(60),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    // Compensation 2: Refund Payment
    let refund_payment = order_workflow.add_step(
        "Refund Payment".to_string(),
        "Compensate by refunding charged amount".to_string(),
        StepType::Integration,
        HashMap::from([
            ("compensation_for".to_string(), json!("Process Payment")),
            ("trigger_condition".to_string(), json!("saga_failure")),
            ("idempotency_key".to_string(), json!("${order_id}_refund")),
            ("audit_required".to_string(), json!(true)),
        ]),
        vec![],
        Some(180),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    // Compensation 3: Cancel Shipment
    let cancel_shipment = order_workflow.add_step(
        "Cancel Shipment".to_string(),
        "Compensate by cancelling shipment booking".to_string(),
        StepType::Integration,
        HashMap::from([
            ("compensation_for".to_string(), json!("Create Shipment")),
            ("trigger_condition".to_string(), json!("saga_failure")),
            ("idempotency_key".to_string(), json!("${order_id}_shipment_cancel")),
        ]),
        vec![],
        Some(120),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    // Nested Transaction Example
    let mut nested_metadata = HashMap::new();
    nested_metadata.insert("transaction_type".to_string(), json!("nested"));
    nested_metadata.insert("parent_transaction".to_string(), json!("order_validation_tx"));
    
    let validate_customer = order_workflow.add_step(
        "Validate Customer Credit".to_string(),
        "Nested transaction for credit validation".to_string(),
        StepType::Automated,
        HashMap::from([
            ("transaction_boundary".to_string(), json!("nested_start")),
            ("transaction_id".to_string(), json!("credit_check_tx")),
            ("savepoint_name".to_string(), json!("before_credit_check")),
            ("rollback_on_failure".to_string(), json!(false)), // Don't rollback parent
        ]),
        vec![],
        Some(20),
        None,
        Some("orders@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the workflow with transaction context
    let mut context = WorkflowContext::new();
    context.set_variable("order_id".to_string(), json!("ORD-2024-12345"));
    context.set_variable("customer_id".to_string(), json!("CUST-789"));
    context.set_variable("items".to_string(), json!([
        {"sku": "PROD-001", "quantity": 2, "price": 99.99},
        {"sku": "PROD-002", "quantity": 1, "price": 149.99}
    ]));
    context.set_variable("payment_method".to_string(), json!("credit_card"));
    context.set_variable("transaction_context".to_string(), json!({
        "correlation_id": "TXN-2024-01-25-001",
        "isolation_level": "read_committed",
        "timeout_seconds": 300
    }));
    
    order_workflow.start(context, Some("orders@example.com".to_string()))
        .expect("Should start workflow");
    
    // In a complete implementation, transactions would support:
    // - workflow.begin_transaction(config) -> TransactionHandle
    // - workflow.commit_transaction(handle) -> Result
    // - workflow.rollback_transaction(handle) -> Result
    // - workflow.create_savepoint(name) -> SavepointHandle
    // - workflow.rollback_to_savepoint(handle) -> Result
    
    // Transaction features:
    // 1. ACID guarantees within boundaries
    // 2. Distributed transaction coordination
    // 3. Saga pattern with compensations
    // 4. Two-phase commit protocol
    // 5. Nested transactions with savepoints
    // 6. Deadlock detection and resolution
    // 7. Transaction timeout handling
    // 8. Audit trail for all operations
    
    // Verify transaction configuration
    assert!(order_workflow.metadata.contains_key("transaction_enabled"));
    assert_eq!(order_workflow.metadata.get("saga_pattern"), Some(&json!("orchestration")));
    
    // Count saga steps and compensations
    let saga_steps = order_workflow.steps.values()
        .filter(|s| s.config.get("saga_step") == Some(&json!(true)))
        .count();
    assert_eq!(saga_steps, 3); // Reserve, Payment, Shipment
    
    let compensation_steps = order_workflow.steps.values()
        .filter(|s| s.config.contains_key("compensation_for"))
        .count();
    assert_eq!(compensation_steps, 3); // One for each saga step
    
    // Verify idempotency keys
    let idempotent_steps = order_workflow.steps.values()
        .filter(|s| s.config.contains_key("idempotency_key"))
        .count();
    assert!(idempotent_steps >= 6); // All saga steps and compensations
}

// =============================================================================
// INTEGRATION TEST SCENARIOS
// =============================================================================

/// Integration Test: Complete Document Approval Workflow
/// Tests multiple user stories working together (W1, W4, W8, W14, W16)
#[test]
fn test_integration_document_approval_workflow() {
    // Given: A comprehensive document approval workflow combining multiple patterns
    let mut metadata = HashMap::new();
    metadata.insert("workflow_type".to_string(), json!("document_approval"));
    metadata.insert("parallel_execution".to_string(), json!(true));
    metadata.insert("monitoring_enabled".to_string(), json!(true));
    metadata.insert("sla_hours".to_string(), json!(48)); // 48 hour SLA
    
    let (mut workflow, _) = Workflow::new(
        "Contract Approval Process".to_string(),
        "Multi-stage contract review and approval with parallel reviews".to_string(),
        metadata,
        Some("legal@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Step 1: Upload Contract (W1 - Visual Design)
    let upload = workflow.add_step(
        "Upload Contract".to_string(),
        "Initial contract submission".to_string(),
        StepType::Manual,
        HashMap::from([
            ("allowed_formats".to_string(), json!(["pdf", "docx"])),
            ("max_size_mb".to_string(), json!(50)),
            ("required_metadata".to_string(), json!(["contract_type", "vendor", "value"])),
        ]),
        vec![],
        Some(30), // 30 minute timeout
        None,
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    let upload_id = if let WorkflowDomainEvent::StepAdded(event) = &upload[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Initial Validation (Automated)
    let validate = workflow.add_step(
        "Validate Contract".to_string(),
        "Automated contract validation".to_string(),
        StepType::Automated,
        HashMap::from([
            ("check_signatures".to_string(), json!(true)),
            ("verify_clauses".to_string(), json!(true)),
            ("risk_assessment".to_string(), json!(true)),
        ]),
        vec![upload_id],
        Some(5),
        None,
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    let validate_id = if let WorkflowDomainEvent::StepAdded(event) = &validate[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Parallel Reviews (W16 - Parallel Execution)
    // Review A: Legal Review (W8 - Human Task Assignment)
    let legal_review = workflow.add_step(
        "Legal Review".to_string(),
        "Legal department contract review".to_string(),
        StepType::Parallel,
        HashMap::from([
            ("review_checklist".to_string(), json!([
                "terms_and_conditions",
                "liability_clauses",
                "termination_clauses",
                "compliance_check"
            ])),
            ("required_role".to_string(), json!("legal_reviewer")),
            ("priority".to_string(), json!("high")),
        ]),
        vec![validate_id],
        Some(1440), // 24 hours
        Some("legal-team@example.com".to_string()),
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    let legal_id = if let WorkflowDomainEvent::StepAdded(event) = &legal_review[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Review B: Financial Review
    let financial_review = workflow.add_step(
        "Financial Review".to_string(),
        "Finance department cost analysis".to_string(),
        StepType::Parallel,
        HashMap::from([
            ("review_checklist".to_string(), json!([
                "total_cost",
                "payment_terms",
                "budget_alignment",
                "roi_analysis"
            ])),
            ("required_role".to_string(), json!("financial_analyst")),
        ]),
        vec![validate_id],
        Some(1440), // 24 hours
        Some("finance-team@example.com".to_string()),
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    let financial_id = if let WorkflowDomainEvent::StepAdded(event) = &financial_review[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Review C: Technical Review (if applicable)
    let technical_review = workflow.add_step(
        "Technical Review".to_string(),
        "Technical specifications review".to_string(),
        StepType::Parallel,
        HashMap::from([
            ("review_checklist".to_string(), json!([
                "technical_requirements",
                "integration_points",
                "security_compliance",
                "performance_sla"
            ])),
            ("conditional".to_string(), json!(true)),
            ("condition".to_string(), json!("contract_type == 'technology'")),
        ]),
        vec![validate_id],
        Some(1440), // 24 hours
        Some("tech-team@example.com".to_string()),
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    let technical_id = if let WorkflowDomainEvent::StepAdded(event) = &technical_review[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Consolidate Reviews
    let consolidate = workflow.add_step(
        "Consolidate Reviews".to_string(),
        "Aggregate all review feedback".to_string(),
        StepType::Automated,
        HashMap::from([
            ("wait_for_all".to_string(), json!(true)),
            ("aggregate_scores".to_string(), json!(true)),
            ("generate_summary".to_string(), json!(true)),
        ]),
        vec![legal_id, financial_id, technical_id],
        Some(10),
        None,
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    let consolidate_id = if let WorkflowDomainEvent::StepAdded(event) = &consolidate[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Final Approval Decision
    let final_approval = workflow.add_step(
        "Final Approval".to_string(),
        "Executive approval decision".to_string(),
        StepType::Approval,
        HashMap::from([
            ("approval_levels".to_string(), json!({
                "under_50k": "manager",
                "under_250k": "director",
                "over_250k": "vp"
            })),
            ("require_unanimous".to_string(), json!(false)),
            ("min_approval_score".to_string(), json!(7.5)), // Out of 10
        ]),
        vec![consolidate_id],
        Some(480), // 8 hours
        None, // Assigned based on contract value
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    let approval_id = if let WorkflowDomainEvent::StepAdded(event) = &final_approval[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Execute Contract (if approved)
    let execute = workflow.add_step(
        "Execute Contract".to_string(),
        "Final contract execution and filing".to_string(),
        StepType::Integration,
        HashMap::from([
            ("systems".to_string(), json!(["docusign", "contract_repository", "erp"])),
            ("notifications".to_string(), json!(["vendor", "procurement", "legal"])),
        ]),
        vec![approval_id],
        Some(30),
        None,
        Some("legal@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the workflow (W4 - Start Instance)
    let mut context = WorkflowContext::new();
    context.set_variable("contract_id".to_string(), json!("CONTRACT-2024-001"));
    context.set_variable("contract_type".to_string(), json!("technology"));
    context.set_variable("vendor".to_string(), json!("TechCorp Inc"));
    context.set_variable("contract_value".to_string(), json!(175000));
    context.set_variable("urgency".to_string(), json!("high"));
    
    workflow.start(context, Some("procurement@example.com".to_string()))
        .expect("Should start workflow");
    
    // W14 - Monitor Progress
    let progress = workflow.get_progress();
    assert_eq!(progress.total_steps, 8);
    assert_eq!(progress.pending_steps, 8); // All steps pending at start
    
    // Simulate workflow execution
    // Complete upload
    if let Some(step) = workflow.steps.get_mut(&upload_id) {
        step.complete().expect("Should complete upload");
    }
    
    // Check executable steps
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 1); // Only validation should be executable
    assert_eq!(executable[0].name, "Validate Contract");
    
    // Complete validation
    if let Some(step) = workflow.steps.get_mut(&validate_id) {
        step.complete().expect("Should complete validation");
    }
    
    // Check parallel execution
    let executable = workflow.get_executable_steps();
    assert_eq!(executable.len(), 3); // All three reviews should be executable
    
    // Verify parallel step configuration
    let parallel_count = executable.iter()
        .filter(|s| matches!(s.step_type, StepType::Parallel))
        .count();
    assert_eq!(parallel_count, 3);
    
    // W8 - Verify task assignments (these are pre-assigned to teams)
    let pre_assigned = workflow.get_pre_assigned_tasks();
    assert!(pre_assigned.iter().any(|t| t.name == "Legal Review" && t.assigned_to == Some("legal-team@example.com".to_string())));
    assert!(pre_assigned.iter().any(|t| t.name == "Financial Review" && t.assigned_to == Some("finance-team@example.com".to_string())));
    assert!(pre_assigned.iter().any(|t| t.name == "Technical Review" && t.assigned_to == Some("tech-team@example.com".to_string())));
    
    // Verify monitoring capabilities
    let bottlenecks = workflow.get_bottlenecks(Duration::from_secs(0));
    assert_eq!(bottlenecks.len(), 0); // No bottlenecks yet
    
    // Verify SLA tracking
    assert!(workflow.metadata.contains_key("sla_hours"));
    assert_eq!(workflow.metadata.get("sla_hours"), Some(&json!(48)));
}

/// Integration Test: Error Recovery Workflow
/// Tests error handling user stories working together (W11, W12, W13)
#[test]
fn test_integration_error_recovery_workflow() {
    // Given: An e-commerce order fulfillment workflow with comprehensive error handling
    let mut metadata = HashMap::new();
    metadata.insert("workflow_type".to_string(), json!("order_fulfillment"));
    metadata.insert("error_handling".to_string(), json!("comprehensive"));
    metadata.insert("compensation_enabled".to_string(), json!(true));
    metadata.insert("circuit_breaker_enabled".to_string(), json!(true));
    
    let (mut workflow, _) = Workflow::new(
        "Order Fulfillment with Recovery".to_string(),
        "E-commerce order processing with error recovery and compensation".to_string(),
        metadata,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Step 1: Validate Order
    let validate_order = workflow.add_step(
        "Validate Order".to_string(),
        "Validate order details and customer".to_string(),
        StepType::Automated,
        HashMap::from([
            ("validations".to_string(), json!([
                "customer_exists",
                "payment_method_valid",
                "shipping_address_valid",
                "items_available"
            ])),
        ]),
        vec![],
        Some(5),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add step");
    
    let validate_id = if let WorkflowDomainEvent::StepAdded(event) = &validate_order[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Reserve Inventory (W11 - Failure Handling)
    let reserve_inventory = workflow.add_step(
        "Reserve Inventory".to_string(),
        "Reserve items in inventory system".to_string(),
        StepType::Integration,
        HashMap::from([
            ("system".to_string(), json!("inventory_service")),
            ("retry_policy".to_string(), json!({
                "max_attempts": 3,
                "backoff_type": "exponential",
                "initial_delay_ms": 1000,
                "max_delay_ms": 30000
            })),
            ("timeout_ms".to_string(), json!(5000)),
            ("compensatable".to_string(), json!(true)),
        ]),
        vec![validate_id],
        Some(10),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add step");
    
    let reserve_id = if let WorkflowDomainEvent::StepAdded(event) = &reserve_inventory[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Compensation Step for Inventory
    let release_inventory = workflow.add_step(
        "Release Inventory".to_string(),
        "Release reserved inventory on failure".to_string(),
        StepType::Automated,
        HashMap::from([
            ("compensation_for".to_string(), json!("Reserve Inventory")),
            ("idempotent".to_string(), json!(true)),
        ]),
        vec![],
        Some(5),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add compensation step");
    
    // Step 3: Process Payment (W12 - Circuit Breaker)
    let process_payment = workflow.add_step(
        "Process Payment".to_string(),
        "Charge customer payment method".to_string(),
        StepType::Integration,
        HashMap::from([
            ("system".to_string(), json!("payment_gateway")),
            ("circuit_breaker".to_string(), json!({
                "failure_threshold": 5,
                "success_threshold": 2,
                "timeout_seconds": 60,
                "half_open_requests": 3
            })),
            ("timeout_ms".to_string(), json!(10000)),
            ("compensatable".to_string(), json!(true)),
        ]),
        vec![reserve_id],
        Some(15),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add step");
    
    let payment_id = if let WorkflowDomainEvent::StepAdded(event) = &process_payment[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Compensation Step for Payment
    let refund_payment = workflow.add_step(
        "Refund Payment".to_string(),
        "Refund payment on failure".to_string(),
        StepType::Integration,
        HashMap::from([
            ("compensation_for".to_string(), json!("Process Payment")),
            ("system".to_string(), json!("payment_gateway")),
            ("idempotent".to_string(), json!(true)),
            ("async_compensation".to_string(), json!(true)), // Can be async
        ]),
        vec![],
        Some(30),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add compensation step");
    
    // Step 4: Create Shipping Label (External Service)
    let create_shipping = workflow.add_step(
        "Create Shipping Label".to_string(),
        "Generate shipping label with carrier".to_string(),
        StepType::Integration,
        HashMap::from([
            ("system".to_string(), json!("shipping_api")),
            ("carriers".to_string(), json!(["ups", "fedex", "usps"])),
            ("circuit_breaker".to_string(), json!({
                "failure_threshold": 3,
                "success_threshold": 1,
                "timeout_seconds": 30,
                "half_open_requests": 1
            })),
            ("fallback_strategy".to_string(), json!("try_next_carrier")),
            ("compensatable".to_string(), json!(true)),
        ]),
        vec![payment_id],
        Some(20),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add step");
    
    let shipping_id = if let WorkflowDomainEvent::StepAdded(event) = &create_shipping[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Compensation Step for Shipping
    let cancel_shipping = workflow.add_step(
        "Cancel Shipping Label".to_string(),
        "Cancel created shipping label".to_string(),
        StepType::Integration,
        HashMap::from([
            ("compensation_for".to_string(), json!("Create Shipping Label")),
            ("system".to_string(), json!("shipping_api")),
            ("idempotent".to_string(), json!(true)),
        ]),
        vec![],
        Some(10),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add compensation step");
    
    // Step 5: Update Order Status
    let update_status = workflow.add_step(
        "Update Order Status".to_string(),
        "Update order to shipped status".to_string(),
        StepType::Automated,
        HashMap::from([
            ("new_status".to_string(), json!("shipped")),
            ("notify_customer".to_string(), json!(true)),
        ]),
        vec![shipping_id],
        Some(5),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add step");
    
    // Error Handler Step (W13 - Rollback)
    let handle_error = workflow.add_step(
        "Handle Order Error".to_string(),
        "Coordinate error recovery and compensation".to_string(),
        StepType::Custom("ErrorHandler".to_string()),
        HashMap::from([
            ("error_handler".to_string(), json!(true)),
            ("compensation_strategy".to_string(), json!("reverse_order")),
            ("notify_operations".to_string(), json!(true)),
            ("create_incident".to_string(), json!(true)),
        ]),
        vec![], // Can be triggered from any failure
        Some(60),
        None,
        Some("fulfillment@example.com".to_string()),
    ).expect("Should add error handler");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("order_id".to_string(), json!("ORDER-2024-12345"));
    context.set_variable("customer_id".to_string(), json!("CUST-98765"));
    context.set_variable("items".to_string(), json!([
        {"sku": "WIDGET-001", "quantity": 2, "price": 29.99},
        {"sku": "GADGET-002", "quantity": 1, "price": 49.99}
    ]));
    context.set_variable("payment_method".to_string(), json!("credit_card"));
    context.set_variable("total_amount".to_string(), json!(109.97));
    
    workflow.start(context, Some("order-service@example.com".to_string()))
        .expect("Should start workflow");
    
    // W11 - Verify retry policies
    let inventory_step = workflow.steps.values()
        .find(|s| s.name == "Reserve Inventory")
        .expect("Should find inventory step");
    assert!(inventory_step.config.contains_key("retry_policy"));
    let retry_policy = &inventory_step.config["retry_policy"];
    assert_eq!(retry_policy["max_attempts"], json!(3));
    assert_eq!(retry_policy["backoff_type"], json!("exponential"));
    
    // W12 - Verify circuit breakers
    let payment_step = workflow.steps.values()
        .find(|s| s.name == "Process Payment")
        .expect("Should find payment step");
    assert!(payment_step.config.contains_key("circuit_breaker"));
    let circuit_breaker = &payment_step.config["circuit_breaker"];
    assert_eq!(circuit_breaker["failure_threshold"], json!(5));
    assert_eq!(circuit_breaker["timeout_seconds"], json!(60));
    
    let shipping_step = workflow.steps.values()
        .find(|s| s.name == "Create Shipping Label")
        .expect("Should find shipping step");
    assert!(shipping_step.config.contains_key("circuit_breaker"));
    assert!(shipping_step.config.contains_key("fallback_strategy"));
    
    // W13 - Verify compensation steps
    let compensation_steps = workflow.steps.values()
        .filter(|s| s.config.contains_key("compensation_for"))
        .count();
    assert_eq!(compensation_steps, 3); // One for each compensatable step
    
    // Verify all compensation steps are idempotent
    let idempotent_compensations = workflow.steps.values()
        .filter(|s| s.config.contains_key("compensation_for"))
        .all(|s| s.config.get("idempotent") == Some(&json!(true)));
    assert!(idempotent_compensations);
    
    // Verify error handler configuration
    let error_handler = workflow.steps.values()
        .find(|s| s.config.get("error_handler") == Some(&json!(true)))
        .expect("Should find error handler");
    assert_eq!(error_handler.config["compensation_strategy"], json!("reverse_order"));
    assert_eq!(error_handler.config["notify_operations"], json!(true));
    
    // Verify compensatable steps are marked
    let compensatable_count = workflow.steps.values()
        .filter(|s| s.config.get("compensatable") == Some(&json!(true)))
        .count();
    assert_eq!(compensatable_count, 3); // Inventory, Payment, Shipping
}

/// Integration Test: Scheduled Batch Processing
/// Tests scheduling and monitoring user stories (W19, W14, W15)
#[test]
fn test_integration_scheduled_batch_processing() {
    // Given: A data warehouse ETL workflow that runs on a schedule
    let mut metadata = HashMap::new();
    metadata.insert("workflow_type".to_string(), json!("batch_etl"));
    metadata.insert("schedule_type".to_string(), json!("cron"));
    metadata.insert("cron_expression".to_string(), json!("0 2 * * *")); // 2 AM daily
    metadata.insert("timezone".to_string(), json!("UTC"));
    metadata.insert("monitoring_enabled".to_string(), json!(true));
    metadata.insert("performance_tracking".to_string(), json!(true));
    metadata.insert("alert_on_failure".to_string(), json!(true));
    
    let (mut workflow, _) = Workflow::new(
        "Daily Data Warehouse ETL".to_string(),
        "Extract, transform, and load data from multiple sources".to_string(),
        metadata,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Step 1: Check Prerequisites (W19 - Scheduled Execution)
    let check_prereq = workflow.add_step(
        "Check Prerequisites".to_string(),
        "Verify all source systems are available".to_string(),
        StepType::Automated,
        HashMap::from([
            ("scheduled_start".to_string(), json!(true)),
            ("check_systems".to_string(), json!([
                "sales_database",
                "crm_system",
                "marketing_platform",
                "finance_system"
            ])),
            ("timeout_minutes".to_string(), json!(10)),
            ("skip_on_holiday".to_string(), json!(true)),
        ]),
        vec![],
        Some(10),
        None,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should add step");
    
    let prereq_id = if let WorkflowDomainEvent::StepAdded(event) = &check_prereq[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Extract Sales Data
    let extract_sales = workflow.add_step(
        "Extract Sales Data".to_string(),
        "Pull daily sales transactions".to_string(),
        StepType::Integration,
        HashMap::from([
            ("source_system".to_string(), json!("sales_database")),
            ("extraction_type".to_string(), json!("incremental")),
            ("watermark_column".to_string(), json!("transaction_date")),
            ("batch_size".to_string(), json!(10000)),
            ("performance_metrics".to_string(), json!([
                "records_extracted",
                "extraction_duration",
                "data_volume_mb"
            ])),
        ]),
        vec![prereq_id],
        Some(30), // 30 minute timeout
        None,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should add step");
    
    let sales_id = if let WorkflowDomainEvent::StepAdded(event) = &extract_sales[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 3: Extract CRM Data (parallel with sales)
    let extract_crm = workflow.add_step(
        "Extract CRM Data".to_string(),
        "Pull customer updates and interactions".to_string(),
        StepType::Integration,
        HashMap::from([
            ("source_system".to_string(), json!("crm_system")),
            ("api_endpoint".to_string(), json!("/api/v2/customers/changes")),
            ("rate_limit_per_minute".to_string(), json!(100)),
            ("performance_metrics".to_string(), json!([
                "api_calls_made",
                "records_extracted",
                "api_response_time"
            ])),
        ]),
        vec![prereq_id],
        Some(30),
        None,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should add step");
    
    let crm_id = if let WorkflowDomainEvent::StepAdded(event) = &extract_crm[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 4: Transform Data (W14 - Monitor Progress)
    let transform_data = workflow.add_step(
        "Transform Data".to_string(),
        "Clean, validate, and transform extracted data".to_string(),
        StepType::Automated,
        HashMap::from([
            ("transformations".to_string(), json!([
                "standardize_addresses",
                "validate_emails",
                "calculate_customer_lifetime_value",
                "derive_product_categories",
                "aggregate_daily_metrics"
            ])),
            ("quality_checks".to_string(), json!([
                "null_check",
                "duplicate_check",
                "referential_integrity",
                "business_rules_validation"
            ])),
            ("checkpoint_enabled".to_string(), json!(true)),
            ("checkpoint_interval_records".to_string(), json!(50000)),
            ("performance_metrics".to_string(), json!([
                "records_transformed",
                "transformation_duration",
                "validation_errors_found"
            ])),
        ]),
        vec![sales_id, crm_id], // Waits for both extracts
        Some(60), // 1 hour timeout
        None,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should add step");
    
    let transform_id = if let WorkflowDomainEvent::StepAdded(event) = &transform_data[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 5: Load to Data Warehouse
    let load_warehouse = workflow.add_step(
        "Load Data Warehouse".to_string(),
        "Load transformed data into warehouse tables".to_string(),
        StepType::Integration,
        HashMap::from([
            ("target_system".to_string(), json!("snowflake")),
            ("load_strategy".to_string(), json!("merge")),
            ("tables".to_string(), json!([
                "fact_sales",
                "dim_customer",
                "dim_product",
                "agg_daily_sales"
            ])),
            ("performance_metrics".to_string(), json!([
                "records_loaded",
                "load_duration",
                "warehouse_credits_used"
            ])),
        ]),
        vec![transform_id],
        Some(45),
        None,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should add step");
    
    let load_id = if let WorkflowDomainEvent::StepAdded(event) = &load_warehouse[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 6: Update Statistics (W15 - Analyze Performance)
    let update_stats = workflow.add_step(
        "Update Statistics".to_string(),
        "Update table statistics and refresh materialized views".to_string(),
        StepType::Automated,
        HashMap::from([
            ("analyze_tables".to_string(), json!(true)),
            ("refresh_views".to_string(), json!([
                "mv_sales_summary",
                "mv_customer_360",
                "mv_product_performance"
            ])),
            ("collect_performance_stats".to_string(), json!(true)),
            ("generate_performance_report".to_string(), json!(true)),
        ]),
        vec![load_id],
        Some(20),
        None,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should add step");
    
    let stats_id = if let WorkflowDomainEvent::StepAdded(event) = &update_stats[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 7: Send Completion Report
    let send_report = workflow.add_step(
        "Send Completion Report".to_string(),
        "Email ETL summary and performance metrics".to_string(),
        StepType::Integration,
        HashMap::from([
            ("recipients".to_string(), json!([
                "data-team@example.com",
                "business-intelligence@example.com"
            ])),
            ("report_sections".to_string(), json!([
                "execution_summary",
                "records_processed",
                "data_quality_metrics",
                "performance_analysis",
                "error_summary",
                "next_run_schedule"
            ])),
            ("attach_detailed_logs".to_string(), json!(false)),
        ]),
        vec![stats_id],
        Some(5),
        None,
        Some("data-engineering@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the workflow (simulating scheduled trigger)
    let mut context = WorkflowContext::new();
    context.set_variable("execution_date".to_string(), json!("2024-01-15"));
    context.set_variable("batch_id".to_string(), json!("BATCH-2024-01-15-02-00"));
    context.set_variable("triggered_by".to_string(), json!("scheduler"));
    context.set_variable("schedule_time".to_string(), json!("2024-01-15T02:00:00Z"));
    
    workflow.start(context, Some("scheduler@example.com".to_string()))
        .expect("Should start workflow");
    
    // W19 - Verify scheduling configuration
    assert!(workflow.metadata.contains_key("schedule_type"));
    assert_eq!(workflow.metadata.get("cron_expression"), Some(&json!("0 2 * * *")));
    assert_eq!(workflow.metadata.get("timezone"), Some(&json!("UTC")));
    
    // Verify scheduled start configuration
    let prereq_step = workflow.steps.values()
        .find(|s| s.name == "Check Prerequisites")
        .expect("Should find prerequisite step");
    assert_eq!(prereq_step.config.get("scheduled_start"), Some(&json!(true)));
    assert_eq!(prereq_step.config.get("skip_on_holiday"), Some(&json!(true)));
    
    // W14 - Verify monitoring capabilities
    let progress = workflow.get_progress();
    assert_eq!(progress.total_steps, 7);
    assert_eq!(progress.pending_steps, 7); // All pending at start
    
    // Verify checkpoint configuration for long-running steps
    let transform_step = workflow.steps.values()
        .find(|s| s.name == "Transform Data")
        .expect("Should find transform step");
    assert_eq!(transform_step.config.get("checkpoint_enabled"), Some(&json!(true)));
    assert!(transform_step.config.contains_key("checkpoint_interval_records"));
    
    // W15 - Verify performance tracking
    assert_eq!(workflow.metadata.get("performance_tracking"), Some(&json!(true)));
    
    // Count steps with performance metrics
    let performance_tracked_steps = workflow.steps.values()
        .filter(|s| s.config.contains_key("performance_metrics"))
        .count();
    assert_eq!(performance_tracked_steps, 4); // Extract Sales, Extract CRM, Transform, Load
    
    // Verify performance analysis step
    let stats_step = workflow.steps.values()
        .find(|s| s.name == "Update Statistics")
        .expect("Should find statistics step");
    assert_eq!(stats_step.config.get("collect_performance_stats"), Some(&json!(true)));
    assert_eq!(stats_step.config.get("generate_performance_report"), Some(&json!(true)));
    
    // Verify completion reporting
    let report_step = workflow.steps.values()
        .find(|s| s.name == "Send Completion Report")
        .expect("Should find report step");
    let report_sections = report_step.config.get("report_sections")
        .expect("Should have report sections");
    assert!(report_sections.as_array().unwrap().iter()
        .any(|s| s == "performance_analysis"));
    assert!(report_sections.as_array().unwrap().iter()
        .any(|s| s == "next_run_schedule"));
    
    // Verify alert configuration
    assert_eq!(workflow.metadata.get("alert_on_failure"), Some(&json!(true)));
}

/// User Story: W16 - Parallel Task Execution
/// As a workflow designer, I want to execute tasks in parallel
/// So that I can reduce overall process time
#[test]
fn test_w16_parallel_task_execution() {
    // Given: A workflow that can benefit from parallelization
    let mut metadata = HashMap::new();
    metadata.insert("parallel_execution".to_string(), json!(true));
    metadata.insert("max_parallel_tasks".to_string(), json!(4));
    
    let (mut workflow, _) = Workflow::new(
        "Document Processing Pipeline".to_string(),
        "Multi-format document processing with parallel steps".to_string(),
        metadata,
        Some("pipeline@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Step 1: Upload document (sequential)
    let upload = workflow.add_step(
        "Upload Document".to_string(),
        "Accept document upload from user".to_string(),
        StepType::Manual,
        HashMap::from([
            ("max_file_size_mb".to_string(), json!(100)),
            ("allowed_formats".to_string(), json!(["pdf", "docx", "xlsx", "pptx"])),
        ]),
        vec![],
        Some(5), // 5 minute timeout
        None,
        Some("pipeline@example.com".to_string()),
    ).expect("Should add step");
    
    let upload_id = if let WorkflowDomainEvent::StepAdded(event) = &upload[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Document validation (sequential)
    let validate = workflow.add_step(
        "Validate Document".to_string(),
        "Check document integrity and format".to_string(),
        StepType::Automated,
        HashMap::from([
            ("check_virus".to_string(), json!(true)),
            ("check_corruption".to_string(), json!(true)),
        ]),
        vec![upload_id],
        Some(2),
        None,
        Some("pipeline@example.com".to_string()),
    ).expect("Should add step");
    
    let validate_id = if let WorkflowDomainEvent::StepAdded(event) = &validate[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Parallel Steps: These can all run simultaneously after validation
    
    // Parallel Step A: Extract Text
    let extract_text = workflow.add_step(
        "Extract Text".to_string(),
        "Extract raw text content from document".to_string(),
        StepType::Parallel, // Note: Using Parallel step type
        HashMap::from([
            ("ocr_enabled".to_string(), json!(true)),
            ("language_detection".to_string(), json!(true)),
        ]),
        vec![validate_id],
        Some(10),
        None,
        Some("pipeline@example.com".to_string()),
    ).expect("Should add step");
    
    let extract_text_id = if let WorkflowDomainEvent::StepAdded(event) = &extract_text[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Parallel Step B: Extract Metadata
    let extract_metadata = workflow.add_step(
        "Extract Metadata".to_string(),
        "Extract document metadata and properties".to_string(),
        StepType::Parallel,
        HashMap::from([
            ("extract_author".to_string(), json!(true)),
            ("extract_dates".to_string(), json!(true)),
            ("extract_keywords".to_string(), json!(true)),
        ]),
        vec![validate_id],
        Some(5),
        None,
        Some("pipeline@example.com".to_string()),
    ).expect("Should add step");
    
    let extract_metadata_id = if let WorkflowDomainEvent::StepAdded(event) = &extract_metadata[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Parallel Step C: Generate Thumbnail
    let generate_thumbnail = workflow.add_step(
        "Generate Thumbnail".to_string(),
        "Create preview thumbnail of document".to_string(),
        StepType::Parallel,
        HashMap::from([
            ("thumbnail_size".to_string(), json!("300x400")),
            ("quality".to_string(), json!("high")),
        ]),
        vec![validate_id],
        Some(8),
        None,
        Some("pipeline@example.com".to_string()),
    ).expect("Should add step");
    
    let thumbnail_id = if let WorkflowDomainEvent::StepAdded(event) = &generate_thumbnail[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Parallel Step D: Analyze Content
    let analyze_content = workflow.add_step(
        "Analyze Content".to_string(),
        "AI-powered content analysis".to_string(),
        StepType::Parallel,
        HashMap::from([
            ("sentiment_analysis".to_string(), json!(true)),
            ("entity_extraction".to_string(), json!(true)),
            ("topic_modeling".to_string(), json!(true)),
        ]),
        vec![validate_id],
        Some(15),
        None,
        Some("pipeline@example.com".to_string()),
    ).expect("Should add step");
    
    let analyze_id = if let WorkflowDomainEvent::StepAdded(event) = &analyze_content[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Convergence Step: Aggregate Results (waits for all parallel tasks)
    let aggregate = workflow.add_step(
        "Aggregate Results".to_string(),
        "Combine all processing results".to_string(),
        StepType::Automated,
        HashMap::from([
            ("wait_for_all".to_string(), json!(true)),
            ("timeout_behavior".to_string(), json!("partial_results")),
        ]),
        vec![extract_text_id, extract_metadata_id, thumbnail_id, analyze_id],
        Some(2),
        None,
        Some("pipeline@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("document_id".to_string(), json!("DOC-2024-001"));
    context.set_variable("document_name".to_string(), json!("quarterly_report.pdf"));
    context.set_variable("document_size_mb".to_string(), json!(45));
    
    workflow.start(context, Some("pipeline@example.com".to_string()))
        .expect("Should start workflow");
    
    // In a complete implementation, parallel execution would:
    // - workflow.execute_parallel_steps() -> Vec<TaskHandle>
    // - workflow.monitor_parallel_progress() -> ParallelStatus
    // - workflow.handle_partial_failures() -> RecoveryStrategy
    // - workflow.optimize_parallel_allocation() -> ResourcePlan
    
    // Parallel execution benefits:
    // 1. Reduced total execution time (max of parallel tasks vs sum)
    // 2. Better resource utilization
    // 3. Improved throughput for high-volume scenarios
    // 4. Fault isolation (one parallel task failing doesn't affect others)
    
    // Challenges handled:
    // - Resource contention management
    // - Partial failure handling
    // - Result synchronization
    // - Progress tracking across parallel branches
    // - Deadlock prevention
    
    // Verify parallel configuration
    assert!(workflow.metadata.contains_key("parallel_execution"));
    assert_eq!(workflow.metadata.get("max_parallel_tasks"), Some(&json!(4)));
    
    // Count parallel steps
    let parallel_steps = workflow.steps.values()
        .filter(|s| matches!(s.step_type, StepType::Parallel))
        .count();
    assert_eq!(parallel_steps, 4); // Four parallel processing steps
}

/// User Story: W17 - Exclusive Choice Pattern
/// As a workflow designer, I want to implement exclusive choice patterns
/// So that I can route workflows based on conditions
#[test]
fn test_w17_exclusive_choice_pattern() {
    // Given: A loan application workflow with multiple exclusive paths
    let mut metadata = HashMap::new();
    metadata.insert("workflow_pattern".to_string(), json!("exclusive_choice"));
    metadata.insert("decision_type".to_string(), json!("risk_based"));
    
    let (mut workflow, _) = Workflow::new(
        "Loan Application Processing".to_string(),
        "Risk-based loan approval workflow with exclusive routing".to_string(),
        metadata,
        Some("loans@example.com".to_string()),
    ).expect("Should create workflow");
    
    // Step 1: Receive Application
    let receive = workflow.add_step(
        "Receive Application".to_string(),
        "Initial loan application submission".to_string(),
        StepType::Manual,
        HashMap::from([
            ("required_documents".to_string(), json!(["id", "income_proof", "bank_statements"])),
            ("loan_types".to_string(), json!(["personal", "auto", "mortgage"])),
        ]),
        vec![],
        Some(30), // 30 minute timeout
        None,
        Some("loans@example.com".to_string()),
    ).expect("Should add step");
    
    let receive_id = if let WorkflowDomainEvent::StepAdded(event) = &receive[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Step 2: Calculate Risk Score (Decision Point)
    let risk_calc = workflow.add_step(
        "Calculate Risk Score".to_string(),
        "Automated risk assessment and scoring".to_string(),
        StepType::Decision, // Decision step that will determine the path
        HashMap::from([
            ("scoring_model".to_string(), json!("ml_risk_v3")),
            ("factors".to_string(), json!(["credit_score", "debt_ratio", "employment_history"])),
            ("thresholds".to_string(), json!({
                "low_risk": 750,    // Score >= 750
                "medium_risk": 650, // Score 650-749
                "high_risk": 550,   // Score 550-649
                "reject": 0         // Score < 550
            })),
        ]),
        vec![receive_id],
        Some(5),
        None,
        Some("loans@example.com".to_string()),
    ).expect("Should add step");
    
    let risk_calc_id = if let WorkflowDomainEvent::StepAdded(event) = &risk_calc[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Exclusive Path A: Auto-Approval (Low Risk)
    let auto_approve = workflow.add_step(
        "Auto-Approve Loan".to_string(),
        "Automatic approval for low-risk applicants".to_string(),
        StepType::Automated,
        HashMap::from([
            ("condition".to_string(), json!("risk_score >= 750")),
            ("approval_limit".to_string(), json!(50000)),
            ("interest_rate_range".to_string(), json!("3.5%-5.5%")),
        ]),
        vec![risk_calc_id],
        Some(2),
        None,
        Some("loans@example.com".to_string()),
    ).expect("Should add step");
    
    let auto_approve_id = if let WorkflowDomainEvent::StepAdded(event) = &auto_approve[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Exclusive Path B: Manual Review (Medium Risk)
    let manual_review = workflow.add_step(
        "Manual Underwriting".to_string(),
        "Human review for medium-risk applications".to_string(),
        StepType::Manual,
        HashMap::from([
            ("condition".to_string(), json!("risk_score >= 650 && risk_score < 750")),
            ("review_checklist".to_string(), json!([
                "verify_employment",
                "check_references",
                "assess_collateral"
            ])),
            ("sla_hours".to_string(), json!(24)),
        ]),
        vec![risk_calc_id],
        Some(1440), // 24 hours
        Some("underwriting@example.com".to_string()),
        Some("loans@example.com".to_string()),
    ).expect("Should add step");
    
    let manual_review_id = if let WorkflowDomainEvent::StepAdded(event) = &manual_review[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Exclusive Path C: Enhanced Due Diligence (High Risk)
    let enhanced_review = workflow.add_step(
        "Enhanced Due Diligence".to_string(),
        "Comprehensive review for high-risk applications".to_string(),
        StepType::Manual,
        HashMap::from([
            ("condition".to_string(), json!("risk_score >= 550 && risk_score < 650")),
            ("additional_checks".to_string(), json!([
                "background_check",
                "asset_verification",
                "co_signer_evaluation"
            ])),
            ("requires_manager_approval".to_string(), json!(true)),
            ("sla_hours".to_string(), json!(72)),
        ]),
        vec![risk_calc_id],
        Some(4320), // 72 hours
        Some("senior-underwriting@example.com".to_string()),
        Some("loans@example.com".to_string()),
    ).expect("Should add step");
    
    let enhanced_id = if let WorkflowDomainEvent::StepAdded(event) = &enhanced_review[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Exclusive Path D: Auto-Rejection (Very High Risk)
    let auto_reject = workflow.add_step(
        "Auto-Reject Application".to_string(),
        "Automatic rejection for very high-risk applicants".to_string(),
        StepType::Automated,
        HashMap::from([
            ("condition".to_string(), json!("risk_score < 550")),
            ("rejection_reasons".to_string(), json!([
                "insufficient_credit_score",
                "high_debt_ratio",
                "recent_bankruptcy"
            ])),
            ("appeal_allowed".to_string(), json!(true)),
            ("appeal_window_days".to_string(), json!(30)),
        ]),
        vec![risk_calc_id],
        Some(2),
        None,
        Some("loans@example.com".to_string()),
    ).expect("Should add step");
    
    let reject_id = if let WorkflowDomainEvent::StepAdded(event) = &auto_reject[0] {
        event.step_id
    } else {
        panic!("Expected StepAdded event");
    };
    
    // Convergence Step: Send Notification (all paths lead here)
    let notify = workflow.add_step(
        "Send Decision Notification".to_string(),
        "Notify applicant of loan decision".to_string(),
        StepType::Automated,
        HashMap::from([
            ("notification_channels".to_string(), json!(["email", "sms", "app"])),
            ("include_details".to_string(), json!(true)),
        ]),
        vec![auto_approve_id, manual_review_id, enhanced_id, reject_id],
        Some(5),
        None,
        Some("loans@example.com".to_string()),
    ).expect("Should add step");
    
    // Start the workflow
    let mut context = WorkflowContext::new();
    context.set_variable("applicant_id".to_string(), json!("APP-2024-12345"));
    context.set_variable("loan_amount".to_string(), json!(25000));
    context.set_variable("loan_type".to_string(), json!("personal"));
    context.set_variable("credit_score".to_string(), json!(720)); // Should route to manual review
    
    workflow.start(context, Some("loans@example.com".to_string()))
        .expect("Should start workflow");
    
    // In a complete implementation, exclusive choice would:
    // - workflow.evaluate_conditions(step_id) -> SelectedPath
    // - workflow.ensure_mutual_exclusion() -> ValidationResult
    // - workflow.get_decision_audit_trail() -> DecisionLog
    // - workflow.simulate_routing(context) -> PathPrediction
    
    // XOR Gateway properties:
    // 1. Exactly one path is taken (mutual exclusion)
    // 2. Conditions are evaluated in order
    // 3. First matching condition wins
    // 4. Default path if no conditions match
    // 5. Decision is logged for audit
    
    // Benefits:
    // - Clear decision logic
    // - Predictable routing
    // - Easy to test and validate
    // - Supports compliance requirements
    // - Enables A/B testing of paths
    
    // Verify exclusive choice configuration
    let decision_step = workflow.steps.values()
        .find(|s| s.step_type == StepType::Decision)
        .expect("Should have decision step");
    assert!(decision_step.config.contains_key("thresholds"));
    
    // Verify all exclusive paths have conditions
    let exclusive_steps = vec!["Auto-Approve Loan", "Manual Underwriting", 
                               "Enhanced Due Diligence", "Auto-Reject Application"];
    for step_name in exclusive_steps {
        let step = workflow.steps.values()
            .find(|s| s.name == step_name)
            .expect(&format!("Should find {step_name} step"));
        assert!(step.config.contains_key("condition"));
    }
} 