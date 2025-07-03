//! Demo: Workflow State Machine Implementation
//!
//! This example demonstrates the complete state machine implementation for
//! workflows and steps, showing formal state transitions with guards and effects.

use cim_domain_workflow::{
    aggregate::Workflow,
    value_objects::{WorkflowContext, StepType},
    state_machine::WorkflowStateMachine,
};
use std::collections::HashMap;

fn main() {
    println!("🔄 Workflow State Machine Demo\n");

    // Create a workflow
    let (mut workflow, _events) = Workflow::new(
        "Document Approval Workflow".to_string(),
        "Automated document approval process with state machine".to_string(),
        HashMap::new(),
        Some("admin".to_string()),
    ).unwrap();

    println!("✅ Created workflow: {workflow.name}");
    println!("📊 Initial state: {:?}\n", workflow.status);

    // Add workflow steps
    println!("📝 Adding workflow steps...");
    
    // Step 1: Upload document
    let events = workflow.add_step(
        "Upload Document".to_string(),
        "User uploads document for approval".to_string(),
        StepType::Manual,
        HashMap::new(),
        vec![],
        Some(5),
        Some("user@example.com".to_string()),
        Some("admin".to_string()),
    ).unwrap();
    
    let upload_step_id = match &events[0] {
        cim_domain_workflow::WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => unreachable!(),
    };

    // Step 2: Review document
    let events = workflow.add_step(
        "Review Document".to_string(),
        "Manager reviews the uploaded document".to_string(),
        StepType::Approval,
        HashMap::from([
            ("approver".to_string(), serde_json::json!("manager@example.com")),
        ]),
        vec![upload_step_id],
        Some(30),
        Some("manager@example.com".to_string()),
        Some("admin".to_string()),
    ).unwrap();
    
    let review_step_id = match &events[0] {
        cim_domain_workflow::WorkflowDomainEvent::StepAdded(e) => e.step_id,
        _ => unreachable!(),
    };

    // Step 3: Process approved document
    workflow.add_step(
        "Process Document".to_string(),
        "System processes the approved document".to_string(),
        StepType::Automated,
        HashMap::from([
            ("script".to_string(), serde_json::json!("process_document.sh")),
        ]),
        vec![review_step_id],
        Some(10),
        None,
        Some("admin".to_string()),
    ).unwrap();

    println!("✅ Added {workflow.steps.len(} steps\n"));

    // Demonstrate workflow state machine
    println!("🎯 Demonstrating Workflow State Machine:");
    
    // Create standalone state machine for visualization
    let state_machine = WorkflowStateMachine::new(workflow.id);
    println!("\n📊 State Machine Diagram:");
    println!("{state_machine.to_mermaid(}\n"));

    // Try to start workflow without context (should fail)
    println!("❌ Attempting to start workflow without context...");
    let result = workflow.start(WorkflowContext::new(), Some("user".to_string()));
    match result {
        Err(e) => println!("   Failed as expected: {e}"),
        Ok(_) => println!("   Unexpected success!"),
    }

    // Start workflow with proper context
    println!("\n✅ Starting workflow with proper context...");
    let mut context = WorkflowContext::new();
    context.set_variable("document_id".to_string(), serde_json::json!("DOC-12345"));
    context.set_variable("document_type".to_string(), serde_json::json!("invoice"));
    
    let events = workflow.start(context, Some("user".to_string())).unwrap();
    println!("   State: {:?}", workflow.status);
    println!("   Events generated: {events.len(}"));

    // Show available transitions
    println!("\n📋 Available transitions from Running state:");
    println!("   - Complete");
    println!("   - Fail");
    println!("   - Pause");
    println!("   - Cancel");

    // Demonstrate pause/resume
    println!("\n⏸️  Pausing workflow...");
    let events = workflow.pause("System maintenance".to_string(), Some("admin".to_string())).unwrap();
    println!("   State: {:?}", workflow.status);
    match &events[0] {
        cim_domain_workflow::WorkflowDomainEvent::WorkflowPaused(e) => {
            println!("   Reason: {e.reason}");
        }
        _ => {}
    }

    println!("\n▶️  Resuming workflow...");
    workflow.resume(Some("admin".to_string())).unwrap();
    println!("   State: {:?}", workflow.status);

    // Demonstrate step state machine
    println!("\n🎯 Demonstrating Step State Machine:");
    
    // Get executable steps
    let first_step_id = {
        let executable_steps = workflow.get_executable_steps();
        println!("\n📋 Executable steps: {executable_steps.len(}"));
        for step in &executable_steps {
            println!("   - {step.name} ({step.step_type})");
        }
        
        executable_steps.first().map(|s| (s.id, s.name.clone()))
    };

    // Execute first step
    if let Some((step_id, step_name)) = first_step_id {
        println!("\n▶️  Executing step: {step_name}");
        let events = workflow.execute_step(step_id).unwrap();
        println!("   Events generated: {events.len(}"));
        
        // For manual steps, they need to be completed manually
        println!("\n✅ Completing manual step...");
        let events = workflow.complete_task(
            step_id,
            "user@example.com".to_string(),
            HashMap::from([
                ("document_url".to_string(), serde_json::json!("https://example.com/doc.pdf")),
            ]),
        ).unwrap();
        println!("   Step completed with {events.len(} events"));
    }

    // Show workflow progress
    let progress = workflow.get_progress();
    println!("\n📊 Workflow Progress:");
    println!("   Total steps: {progress.total_steps}");
    println!("   Completed: {progress.completed_steps}");
    println!("   In progress: {progress.in_progress_steps}");
    println!("   Pending: {progress.pending_steps}");
    println!("   Failed: {progress.failed_steps}");
    println!("   Progress: {:.1}%", (progress.completed_steps as f64 / progress.total_steps as f64) * 100.0);

    // Demonstrate failure handling
    println!("\n❌ Demonstrating failure handling...");
    let mut failing_workflow = create_test_workflow();
    failing_workflow.start(create_test_context(), Some("user".to_string())).unwrap();
    
    let events = failing_workflow.fail("Network connection lost".to_string()).unwrap();
    match &events[0] {
        cim_domain_workflow::WorkflowDomainEvent::WorkflowFailed(e) => {
            println!("   Error: {e.error}");
            println!("   Duration: {e.duration_seconds} seconds");
        }
        _ => {}
    }

    // Show state transition history
    println!("\n📜 State Transition History:");
    if let Some(history) = workflow.context.get_variable("last_state_transition") {
        println!("   Last transition: {serde_json::to_string_pretty(history}").unwrap());
    }

    println!("\n✨ Demo completed!");
}

fn create_test_workflow() -> Workflow {
    let (mut workflow, _) = Workflow::new(
        "Test Workflow".to_string(),
        "A test workflow for demonstration".to_string(),
        HashMap::new(),
        Some("test_user".to_string()),
    ).unwrap();
    
    // Add a simple step
    workflow.add_step(
        "Test Step".to_string(),
        "A test step".to_string(),
        StepType::Manual,
        HashMap::new(),
        vec![],
        None,
        None,
        None,
    ).unwrap();
    
    workflow
}

fn create_test_context() -> WorkflowContext {
    let mut context = WorkflowContext::new();
    context.set_variable("test".to_string(), serde_json::json!("value"));
    context
} 