//! NATS-enabled workflow demonstration
//!
//! This example shows how to use the workflow domain with NATS event publishing
//! for distributed event streaming and workflow orchestration.
//!
//! Run with: cargo run --example nats_workflow_demo

use cim_domain_workflow::{
    commands::*,
    handlers::NatsWorkflowCommandHandler,
    value_objects::{StepType, WorkflowContext},
};
use async_nats;
use futures::StreamExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NATS Workflow Demo ===\n");

    // Connect to NATS server
    println!("Connecting to NATS server...");
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    let client = match async_nats::connect(&nats_url).await {
        Ok(client) => {
            println!("✓ Connected to NATS at {}", nats_url);
            client
        }
        Err(e) => {
            println!("✗ Failed to connect to NATS: {}", e);
            println!("  Make sure NATS server is running: nats-server -js");
            return Err(e.into());
        }
    };

    // Set up event subscriber to monitor workflow events
    println!("\nSetting up event monitoring...");
    let mut subscriber = client.subscribe("events.workflow.>").await?;
    println!("✓ Subscribed to workflow events");

    // Create NATS-enabled command handler
    let handler = NatsWorkflowCommandHandler::new(client.clone(), "events".to_string());

    // Create a new workflow
    println!("\n1. Creating workflow...");
    let create_cmd = CreateWorkflow {
        name: "Order Processing Workflow".to_string(),
        description: "Process customer orders with payment and fulfillment".to_string(),
        metadata: Default::default(),
        created_by: Some("demo-user".to_string()),
    };

    let create_events = handler.handle_create_workflow(create_cmd).await?;
    let workflow_id = create_events[0].workflow_id();
    println!("✓ Created workflow: {}", workflow_id);

    // Add workflow steps
    println!("\n2. Adding workflow steps...");
    
    // Step 1: Validate order
    let add_validate = AddStep {
        workflow_id,
        name: "Validate Order".to_string(),
        description: "Check order details and inventory".to_string(),
        step_type: StepType::Automated,
        config: Default::default(),
        dependencies: vec![],
        estimated_duration_minutes: Some(5),
        assigned_to: None,
        added_by: Some("demo-user".to_string()),
    };
    handler.handle_add_step(add_validate).await?;
    println!("✓ Added step: Validate Order");

    // Step 2: Process payment
    let add_payment = AddStep {
        workflow_id,
        name: "Process Payment".to_string(),
        description: "Charge customer payment method".to_string(),
        step_type: StepType::Integration,
        config: Default::default(),
        dependencies: vec![],  // Dependencies need StepIds, not names
        estimated_duration_minutes: Some(10),
        assigned_to: None,
        added_by: Some("demo-user".to_string()),
    };
    handler.handle_add_step(add_payment).await?;
    println!("✓ Added step: Process Payment");

    // Step 3: Fulfill order
    let add_fulfill = AddStep {
        workflow_id,
        name: "Fulfill Order".to_string(),
        description: "Pack and ship the order".to_string(),
        step_type: StepType::Manual,
        config: Default::default(),
        dependencies: vec![],  // Dependencies need StepIds, not names
        estimated_duration_minutes: Some(30),
        assigned_to: Some("fulfillment-team".to_string()),
        added_by: Some("demo-user".to_string()),
    };
    handler.handle_add_step(add_fulfill).await?;
    println!("✓ Added step: Fulfill Order");

    // Start the workflow
    println!("\n3. Starting workflow execution...");
    let start_cmd = StartWorkflow {
        workflow_id,
        context: WorkflowContext::default(),
        started_by: Some("demo-user".to_string()),
    };
    handler.handle_start_workflow(start_cmd).await?;
    println!("✓ Workflow started");

    // Monitor events for a short time
    println!("\n4. Monitoring workflow events (5 seconds)...");
    println!("   Subject | Event Type | Correlation ID");
    println!("   --------|------------|----------------");

    // Collect events for 5 seconds
    let start_time = tokio::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(5) {
        match tokio::time::timeout(Duration::from_millis(100), subscriber.next()).await {
            Ok(Some(msg)) => {
                // Extract event info from headers
                if let Some(headers) = &msg.headers {
                    let event_type = headers.get("X-Event-Type")
                        .map(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let correlation_id = headers.get("X-Correlation-ID")
                        .map(|v| &v.as_str()[..8])
                        .unwrap_or("Unknown");
                    
                    let subject_display = if msg.subject.len() > 20 {
                        format!("{}...", &msg.subject[7..20])
                    } else {
                        msg.subject.to_string()
                    };
                    
                    println!("   {} | {} | {}...", 
                        subject_display,
                        event_type, 
                        correlation_id
                    );
                }
            }
            Ok(None) => break,
            Err(_) => continue, // Timeout, continue monitoring
        }
    }

    // Get final workflow state
    println!("\n5. Final workflow state:");
    if let Some(workflow) = handler.get_workflow(&workflow_id).await {
        println!("   Status: {:?}", workflow.status);
        println!("   Steps: {} total", workflow.steps.len());
        println!("   Created: {}", workflow.created_at);
    }

    println!("\n=== Demo Complete ===");
    println!("\nKey features demonstrated:");
    println!("- NATS connection and event publishing");
    println!("- Correlation/causation ID tracking");
    println!("- Event-driven workflow execution");
    println!("- Distributed event monitoring");

    Ok(())
}

/// Example of a distributed workflow monitor
/// This could run on a different machine/process
#[allow(dead_code)]
async fn distributed_monitor(nats_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect(nats_url).await?;
    let mut subscriber = client.subscribe("events.workflow.>").await?;
    
    println!("Distributed monitor connected, listening for events...");
    
    while let Some(msg) = subscriber.next().await {
        if let Some(headers) = &msg.headers {
            let correlation_id = headers.get("X-Correlation-ID")
                .map(|v| v.as_str())
                .unwrap_or("unknown");
            let event_type = headers.get("X-Event-Type")
                .map(|v| v.as_str())
                .unwrap_or("unknown");
            let workflow_id = headers.get("X-Workflow-ID")
                .map(|v| v.as_str())
                .unwrap_or("unknown");
            
            println!("[Monitor] Workflow {} - {} (correlation: {})",
                workflow_id, event_type, correlation_id);
            
            // Could trigger actions based on events
            match event_type {
                "WorkflowFailed" => {
                    println!("[Monitor] ALERT: Workflow failed, triggering recovery...");
                }
                "StepExecutionFailed" => {
                    println!("[Monitor] WARNING: Step failed, may need intervention");
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}