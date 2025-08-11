//! Cross-domain workflow orchestration example
//!
//! This example demonstrates how workflows can orchestrate operations
//! across multiple CIM domains using event-driven patterns.
//!
//! Run with: cargo run --example cross_domain_workflow

use cim_domain_workflow::{
    Workflow,
    handlers::CrossDomainHandler,
    value_objects::StepType,
};
use async_nats;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cross-Domain Workflow Orchestration Demo ===\n");

    // Connect to NATS
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

    // Create cross-domain handler
    let cross_domain = CrossDomainHandler::new(client.clone(), "events".to_string());

    // Scenario: E-commerce order processing across multiple domains
    println!("\n📋 Scenario: Processing an e-commerce order");
    println!("   This workflow will coordinate across:");
    println!("   • Inventory domain - Check stock availability");
    println!("   • Payment domain - Process payment");
    println!("   • Shipping domain - Create shipment");
    println!("   • Notification domain - Send confirmations\n");

    // Create workflow
    let (mut workflow, _) = Workflow::new(
        "E-Commerce Order Processing".to_string(),
        "End-to-end order processing across domains".to_string(),
        Default::default(),
        Some("order-system".to_string()),
    )?;

    println!("✓ Created workflow: {}", workflow.id);

    // Add cross-domain steps
    let mut config1 = HashMap::new();
    config1.insert("domain".to_string(), json!("inventory"));
    config1.insert("operation".to_string(), json!("check_availability"));
    
    workflow.add_step(
        "Check Inventory".to_string(),
        "Verify items are in stock".to_string(),
        StepType::Integration,
        config1,
        vec![],
        Some(2),
        None,
        Some("order-system".to_string()),
    )?;

    let mut config2 = HashMap::new();
    config2.insert("domain".to_string(), json!("inventory"));
    config2.insert("operation".to_string(), json!("reserve_items"));
    
    workflow.add_step(
        "Reserve Items".to_string(),
        "Reserve items in inventory".to_string(),
        StepType::Integration,
        config2,
        vec![],
        Some(3),
        None,
        Some("order-system".to_string()),
    )?;

    let mut config3 = HashMap::new();
    config3.insert("domain".to_string(), json!("payment"));
    config3.insert("operation".to_string(), json!("charge_payment"));
    
    workflow.add_step(
        "Process Payment".to_string(),
        "Charge customer payment method".to_string(),
        StepType::Integration,
        config3,
        vec![],
        Some(5),
        None,
        Some("order-system".to_string()),
    )?;

    let mut config4 = HashMap::new();
    config4.insert("domain".to_string(), json!("shipping"));
    config4.insert("operation".to_string(), json!("create_shipment"));
    
    workflow.add_step(
        "Create Shipment".to_string(),
        "Schedule delivery".to_string(),
        StepType::Integration,
        config4,
        vec![],
        Some(10),
        None,
        Some("order-system".to_string()),
    )?;

    let mut config5 = HashMap::new();
    config5.insert("domain".to_string(), json!("notification"));
    config5.insert("operation".to_string(), json!("send_email"));
    
    workflow.add_step(
        "Send Confirmation".to_string(),
        "Notify customer of order status".to_string(),
        StepType::Integration,
        config5,
        vec![],
        Some(1),
        None,
        Some("order-system".to_string()),
    )?;

    println!("\n✓ Added {} cross-domain steps", workflow.steps.len());

    // Demonstrate cross-domain operations
    println!("\n🔄 Executing cross-domain operations...\n");

    // 1. Check inventory
    println!("1️⃣ Checking inventory...");
    let check_inventory_id = cross_domain.request_operation(
        workflow.id,
        workflow.steps.values().next().unwrap().id,
        "inventory".to_string(),
        "check_availability".to_string(),
        json!({
            "items": [
                {"sku": "WIDGET-001", "quantity": 2},
                {"sku": "GADGET-002", "quantity": 1}
            ]
        }),
        Some("order-system".to_string()),
    ).await?;
    println!("   → Sent request with correlation ID: {}", check_inventory_id);

    // Simulate receiving response
    sleep(Duration::from_millis(500)).await;
    cross_domain.handle_operation_response(
        check_inventory_id,
        "inventory".to_string(),
        true,
        json!({
            "data": {
                "available": true,
                "items": [
                    {"sku": "WIDGET-001", "available": 5},
                    {"sku": "GADGET-002", "available": 3}
                ]
            }
        }),
        450,
    ).await?;
    println!("   ✓ Inventory check completed - items available");

    // 2. Process payment
    println!("\n2️⃣ Processing payment...");
    let payment_id = cross_domain.request_operation(
        workflow.id,
        workflow.steps.values().nth(2).unwrap().id,
        "payment".to_string(),
        "charge_payment".to_string(),
        json!({
            "amount": 99.99,
            "currency": "USD",
            "payment_method": "card_ending_4242"
        }),
        Some("order-system".to_string()),
    ).await?;
    println!("   → Sent request with correlation ID: {}", payment_id);

    // Simulate payment processing
    sleep(Duration::from_millis(1000)).await;
    cross_domain.handle_operation_response(
        payment_id,
        "payment".to_string(),
        true,
        json!({
            "data": {
                "transaction_id": "txn_abc123",
                "status": "captured",
                "amount": 99.99
            }
        }),
        950,
    ).await?;
    println!("   ✓ Payment processed successfully");

    // 3. Create shipment
    println!("\n3️⃣ Creating shipment...");
    let shipment_id = cross_domain.request_operation(
        workflow.id,
        workflow.steps.values().nth(3).unwrap().id,
        "shipping".to_string(),
        "create_shipment".to_string(),
        json!({
            "address": {
                "street": "123 Main St",
                "city": "Springfield",
                "state": "IL",
                "zip": "62701"
            },
            "items": [
                {"sku": "WIDGET-001", "quantity": 2},
                {"sku": "GADGET-002", "quantity": 1}
            ]
        }),
        Some("order-system".to_string()),
    ).await?;
    println!("   → Sent request with correlation ID: {}", shipment_id);

    // Simulate shipment creation
    sleep(Duration::from_millis(800)).await;
    cross_domain.handle_operation_response(
        shipment_id,
        "shipping".to_string(),
        true,
        json!({
            "data": {
                "tracking_number": "1Z999AA10123456784",
                "carrier": "UPS",
                "estimated_delivery": "2025-08-05"
            }
        }),
        750,
    ).await?;
    println!("   ✓ Shipment created with tracking: 1Z999AA10123456784");

    // Demonstrate event subscription
    println!("\n📡 Subscribing to domain events...");
    
    let subscription_id = cross_domain.subscribe_to_domain_events(
        workflow.id,
        workflow.steps.values().last().unwrap().id,
        "inventory".to_string(),
        "stock.updated".to_string(),
        Some(json!({
            "sku": "WIDGET-001"
        })),
    ).await?;
    
    println!("✓ Subscribed to inventory.stock.updated events");
    println!("  Subscription ID: {}", subscription_id);

    // Demonstrate distributed transaction
    println!("\n🔐 Starting distributed transaction...");
    
    let transaction_id = cross_domain.start_transaction(
        workflow.id,
        vec!["inventory".to_string(), "payment".to_string(), "shipping".to_string()],
        30,
    ).await?;
    
    println!("✓ Started transaction: {}", transaction_id);
    println!("  Participating domains: inventory, payment, shipping");
    println!("  Timeout: 30 seconds");

    // Summary
    println!("\n📊 Workflow Summary:");
    println!("   • Workflow ID: {}", workflow.id);
    println!("   • Cross-domain operations: 3 completed");
    println!("   • Event subscriptions: 1 active");
    println!("   • Distributed transactions: 1 initiated");
    
    println!("\n✨ Key Features Demonstrated:");
    println!("   • Request/response pattern for cross-domain operations");
    println!("   • Correlation ID tracking across domains");
    println!("   • Event subscription for reactive workflows");
    println!("   • Distributed transaction coordination");
    println!("   • Error handling and retries");

    println!("\n=== Demo Complete ===");

    Ok(())
}

/// Example of handling cross-domain errors
#[allow(dead_code)]
async fn handle_cross_domain_failure(
    handler: &CrossDomainHandler,
    workflow_id: cim_domain_workflow::value_objects::WorkflowId,
    step_id: cim_domain_workflow::value_objects::StepId,
) -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a payment failure
    let payment_id = handler.request_operation(
        workflow_id,
        step_id,
        "payment".to_string(),
        "charge_payment".to_string(),
        json!({
            "amount": 999.99,
            "currency": "USD",
            "payment_method": "card_ending_0000"  // Invalid card
        }),
        Some("order-system".to_string()),
    ).await?;

    // Simulate payment failure response
    handler.handle_operation_response(
        payment_id,
        "payment".to_string(),
        false,
        json!({
            "code": "INSUFFICIENT_FUNDS",
            "message": "Card declined due to insufficient funds",
            "details": {
                "available_balance": 500.00,
                "required_amount": 999.99
            },
            "retryable": true
        }),
        200,
    ).await?;

    println!("Payment failed - triggering compensating actions...");
    
    // Workflow would handle the failure event and trigger compensations
    // such as releasing inventory reservations, canceling shipment, etc.
    
    Ok(())
}