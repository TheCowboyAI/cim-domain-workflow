//! Simple Order Processing Workflow Example
//!
//! This example demonstrates a basic 3-step workflow for processing customer orders.
//! It shows:
//! - Creating a workflow with automated and manual steps
//! - Starting and executing workflow steps
//! - Tracking workflow progress
//! - Event-driven execution model
//!
//! Run with: cargo run --example simple_order_workflow

use cim_domain_workflow::{
    aggregate::Workflow,
    value_objects::{WorkflowContext, StepType},
    WorkflowDomainEvent,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Simple Order Processing Workflow Example\n");
    println!("This example demonstrates a basic workflow with 3 steps:");
    println!("1. Validate Order (Automated)");
    println!("2. Process Payment (Automated)");
    println!("3. Fulfill Order (Manual)\n");

    // Create the workflow
    let (mut workflow, events) = Workflow::new(
        "Simple Order Processing".to_string(),
        "Process customer orders through validation, payment, and fulfillment".to_string(),
        HashMap::new(),
        Some("order-system".to_string()),
    )?;

    println!("✅ Workflow created: {}", workflow.name);
    println!("   ID: {:?}", workflow.id);
    println!("   Status: {:?}", workflow.status);
    println!("   Events generated: {}\n", events.len());

    // Add Step 1: Validate Order
    println!("📝 Adding workflow steps...\n");
    
    let events = workflow.add_step(
        "Validate Order".to_string(),
        "Check order details and inventory availability".to_string(),
        StepType::Automated,
        HashMap::from([
            ("validation_rules".to_string(), serde_json::json!([
                "check_inventory",
                "validate_customer",
                "verify_address"
            ])),
        ]),
        vec![], // No dependencies
        Some(5), // 5 minute timeout
        None, // No specific assignee (automated)
        Some("order-system".to_string()),
    )?;
    
    let validate_step_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => {
            println!("✅ Added step: {} (ID: {:?})", e.name, e.step_id);
            e.step_id
        }
        _ => panic!("Expected StepAdded event"),
    };

    // Add Step 2: Process Payment
    let events = workflow.add_step(
        "Process Payment".to_string(),
        "Charge customer payment method and verify transaction".to_string(),
        StepType::Automated,
        HashMap::from([
            ("payment_gateway".to_string(), serde_json::json!("stripe")),
            ("retry_attempts".to_string(), serde_json::json!(3)),
        ]),
        vec![validate_step_id], // Depends on validation
        Some(10), // 10 minute timeout
        None,
        Some("order-system".to_string()),
    )?;
    
    let payment_step_id = match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => {
            println!("✅ Added step: {} (ID: {:?})", e.name, e.step_id);
            e.step_id
        }
        _ => panic!("Expected StepAdded event"),
    };

    // Add Step 3: Fulfill Order
    let events = workflow.add_step(
        "Fulfill Order".to_string(),
        "Pick, pack, and ship order to customer".to_string(),
        StepType::Manual,
        HashMap::from([
            ("warehouse".to_string(), serde_json::json!("main-warehouse")),
            ("shipping_method".to_string(), serde_json::json!("standard")),
        ]),
        vec![payment_step_id], // Depends on payment
        Some(60), // 60 minute timeout
        Some("warehouse-team".to_string()),
        Some("order-system".to_string()),
    )?;

    match &events[0] {
        WorkflowDomainEvent::StepAdded(e) => {
            println!("✅ Added step: {} (ID: {:?})", e.name, e.step_id);
        }
        _ => panic!("Expected StepAdded event"),
    };

    println!("\n📊 Workflow structure created with {} steps", workflow.steps.len());

    // Start the workflow
    println!("\n🚀 Starting workflow execution...\n");
    
    let mut context = WorkflowContext::new();
    context.set_variable("order_id".to_string(), serde_json::json!("ORD-12345"));
    context.set_variable("customer_id".to_string(), serde_json::json!("CUST-789"));
    context.set_variable("total_amount".to_string(), serde_json::json!(99.99));
    context.set_variable("items".to_string(), serde_json::json!([
        {"sku": "WIDGET-001", "quantity": 2, "price": 29.99},
        {"sku": "GADGET-002", "quantity": 1, "price": 40.01}
    ]));
    
    let events = workflow.start(context, Some("order-system".to_string()))?;
    
    println!("✅ Workflow started!");
    println!("   Status: {:?}", workflow.status);
    println!("   Events generated: {}", events.len());
    
    for event in &events {
        match event {
            WorkflowDomainEvent::WorkflowStarted(e) => {
                println!("   📢 Event: WorkflowStarted at {}", e.started_at);
            }
            _ => {}
        }
    }

    // Check executable steps
    println!("\n📋 Checking executable steps...");
    let executable_steps = workflow.get_executable_steps();
    println!("   Found {} executable step(s):", executable_steps.len());
    
    for step in &executable_steps {
        println!("   - {} ({:?})", step.name, step.step_type);
    }

    // Execute Step 1: Validate Order
    println!("\n▶️  Executing: Validate Order");
    let events = workflow.execute_step(validate_step_id)?;
    println!("   Events generated: {}", events.len());
    
    // For automated steps, they complete automatically
    println!("   🔄 Automated validation running...");
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Check if step completed automatically
    let step = workflow.steps.get(&validate_step_id).unwrap();
    if step.status == cim_domain_workflow::value_objects::StepStatus::Completed {
        println!("   ✅ Validation completed automatically");
    } else {
        println!("   ⚠️  Step status: {:?}", step.status);
    }

    // Execute Step 2: Process Payment
    println!("\n▶️  Executing: Process Payment");
    let executable_steps = workflow.get_executable_steps();
    println!("   Found {} executable step(s)", executable_steps.len());
    if !executable_steps.is_empty() {
        println!("   Next step: {}", executable_steps[0].name);
    }
    
    let events = workflow.execute_step(payment_step_id)?;
    println!("   Events generated: {}", events.len());
    
    println!("   🔄 Processing payment...");
    std::thread::sleep(std::time::Duration::from_millis(800));
    
    // Check if step completed automatically
    let step = workflow.steps.get(&payment_step_id).unwrap();
    if step.status == cim_domain_workflow::value_objects::StepStatus::Completed {
        println!("   ✅ Payment processed automatically");
        // In a real system, payment details would be in the workflow context
        println!("      Transaction ID: txn_1234567890");
    } else {
        println!("   ⚠️  Step status: {:?}", step.status);
    }

    // Check workflow progress
    println!("\n📊 Workflow Progress:");
    let progress = workflow.get_progress();
    println!("   Total steps: {}", progress.total_steps);
    println!("   Completed: {}", progress.completed_steps);
    println!("   In progress: {}", progress.in_progress_steps);
    println!("   Pending: {}", progress.pending_steps);
    println!("   Progress: {:.1}%", 
        (progress.completed_steps as f64 / progress.total_steps as f64) * 100.0);

    // Execute Step 3: Fulfill Order (Manual)
    println!("\n▶️  Executing: Fulfill Order");
    let executable_steps = workflow.get_executable_steps();
    println!("   Found {} executable step(s)", executable_steps.len());
    
    if executable_steps.is_empty() {
        println!("   ❌ No executable steps found!");
        return Ok(());
    }
    
    let fulfill_step_id = executable_steps[0].id;
    let events = workflow.execute_step(fulfill_step_id)?;
    
    // Check events for task assignment
    let assigned_to = match events.iter().find_map(|e| match e {
        WorkflowDomainEvent::TaskAssigned(ta) => Some(ta.assigned_to.clone()),
        _ => None,
    }) {
        Some(assignee) => assignee,
        None => "warehouse-team".to_string(),
    };
    
    println!("   📢 Manual task started");
    println!("      Assigned to: {}", assigned_to);
    
    println!("   ⏳ Waiting for warehouse team to complete fulfillment...");
    println!("   (In a real system, this would wait for human action)");
    
    // Simulate manual completion
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Complete the task as the assigned user
    let complete_events = workflow.complete_task(
        fulfill_step_id,
        assigned_to.clone(),
        HashMap::from([
            ("tracking_number".to_string(), serde_json::json!("1Z999AA10123456784")),
            ("carrier".to_string(), serde_json::json!("UPS")),
            ("estimated_delivery".to_string(), serde_json::json!("2024-01-15")),
        ]),
    )?;
    
    println!("   ✅ Order fulfilled and shipped!");
    for event in &complete_events {
        match event {
            WorkflowDomainEvent::TaskCompleted(e) => {
                println!("      Tracking: {}", 
                    e.completion_data.get("tracking_number").unwrap_or(&serde_json::json!("N/A")));
            }
            _ => {}
        }
    }

    // Complete the workflow
    println!("\n🏁 Completing workflow...");
    let complete_events = workflow.complete()?;
    
    match &complete_events[0] {
        WorkflowDomainEvent::WorkflowCompleted(e) => {
            println!("   ✅ Workflow completed!");
            println!("   Total duration: {} seconds", e.duration_seconds);
        }
        _ => {}
    }

    // Final summary
    println!("\n📈 Final Workflow Summary:");
    println!("   Status: {:?}", workflow.status);
    
    let final_progress = workflow.get_progress();
    println!("   All {} steps completed successfully!", final_progress.completed_steps);
    
    // Show workflow context results
    println!("\n📦 Workflow Results:");
    if let Some(order_id) = workflow.context.get_variable("order_id") {
        println!("   Order ID: {}", order_id);
    }
    if let Some(txn_id) = workflow.context.get_variable("transaction_id") {
        println!("   Transaction: {}", txn_id);
    }
    if let Some(tracking) = workflow.context.get_variable("tracking_number") {
        println!("   Tracking: {}", tracking);
    }

    println!("\n✨ Example completed successfully!");
    
    Ok(())
}