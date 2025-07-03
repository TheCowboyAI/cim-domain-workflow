//! Example: Exporting Workflows to ContextGraph JSON Format
//!
//! This example demonstrates how to:
//! 1. Create a workflow with multiple steps and dependencies
//! 2. Export it to ContextGraph JSON format
//! 3. Analyze the graph structure
//! 4. Export to DOT format for visualization

use cim_domain_workflow::{
    aggregate::Workflow, projections::WorkflowContextGraph, value_objects::*,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Workflow ContextGraph Export Example ===\n");

    // Create a realistic workflow example
    let workflow = create_software_deployment_workflow()?;

    // Export to ContextGraph JSON
    let contextgraph = WorkflowContextGraph::from_workflow(&workflow);

    println!("📊 Workflow Statistics:");
    let stats = contextgraph.statistics();
    println!("  • Total nodes: {stats.total_nodes}");
    println!("  • Step nodes: {stats.step_nodes}");
    println!("  • Total edges: {stats.total_edges}");
    println!("  • Dependency edges: {stats.dependency_edges}");
    println!("  • Max depth: {stats.max_depth}");
    println!("  • Is cyclic: {stats.is_cyclic}");
    println!();

    // Generate JSON export
    let json = contextgraph.to_json()?;
    println!("📄 ContextGraph JSON Export:");
    println!("{json}");
    println!();

    // Generate DOT export for Graphviz
    let dot = contextgraph.to_dot();
    println!("🎨 DOT Export (for Graphviz visualization):");
    println!("{dot}");
    println!();

    // Demonstrate JSON round-trip
    println!("🔄 Testing JSON round-trip...");
    let reconstructed = WorkflowContextGraph::from_json(&json)?;
    println!("✅ Successfully reconstructed workflow: {reconstructed.name}");

    // Show step analysis
    println!("\n📋 Step Analysis:");
    for node in contextgraph.get_step_nodes() {
        if let cim_domain_workflow::projections::ContextGraphNodeValue::Step {
            name,
            step_type,
            status,
            estimated_duration_minutes,
            assigned_to,
            ..
        } = &node.value
        {
            println!("  • {name} ({:?})", step_type);
            println!("    Status: {:?}", status);
            if let Some(duration) = estimated_duration_minutes {
                println!("    Duration: {duration} minutes");
            }
            if let Some(assignee) = assigned_to {
                println!("    Assigned to: {assignee}");
            }
        }
    }

    println!("\n🔗 Dependency Analysis:");
    for edge in contextgraph.get_dependency_edges() {
        println!("  • {edge.source} → {edge.target} ({edge.edge_type})");
    }

    Ok(())
}

fn create_software_deployment_workflow() -> Result<Workflow, Box<dyn std::error::Error>> {
    // Create the main workflow
    let mut metadata = HashMap::new();
    metadata.insert("project".to_string(), serde_json::json!("alchemist"));
    metadata.insert("environment".to_string(), serde_json::json!("production"));
    metadata.insert("version".to_string(), serde_json::json!("1.0.0"));

    let (mut workflow, _events) = Workflow::new(
        "Software Deployment Pipeline".to_string(),
        "Automated deployment pipeline for software releases".to_string(),
        metadata,
        Some("deployment-bot".to_string()),
    )?;

    // Step 1: Code Review
    let review_events = workflow.add_step(
        "Code Review".to_string(),
        "Review code changes and approve for deployment".to_string(),
        StepType::Manual,
        {
            let mut config = HashMap::new();
            config.insert("required_reviewers".to_string(), serde_json::json!(2));
            config.insert("require_approval".to_string(), serde_json::json!(true));
            config
        },
        Vec::new(), // No dependencies
        Some(60),   // 1 hour
        Some("senior-dev-team".to_string()),
        Some("deployment-bot".to_string()),
    )?;

    // Extract step ID from the event
    let review_step_id =
        if let Some(cim_domain_workflow::WorkflowDomainEvent::StepAdded(ref event)) =
            review_events.first()
        {
            event.step_id
        } else {
            return Err("Failed to get review step ID".into());
        };

    // Step 2: Build Application
    let build_events = workflow.add_step(
        "Build Application".to_string(),
        "Compile and build the application artifacts".to_string(),
        StepType::Automated,
        {
            let mut config = HashMap::new();
            config.insert("build_command".to_string(), serde_json::json!("nix build"));
            config.insert("artifact_path".to_string(), serde_json::json!("./result"));
            config
        },
        vec![review_step_id], // Depends on code review
        Some(30),             // 30 minutes
        Some("ci-system".to_string()),
        Some("deployment-bot".to_string()),
    )?;

    let build_step_id =
        if let Some(cim_domain_workflow::WorkflowDomainEvent::StepAdded(ref event)) =
            build_events.first()
        {
            event.step_id
        } else {
            return Err("Failed to get build step ID".into());
        };

    // Step 3: Run Tests
    let test_events = workflow.add_step(
        "Run Test Suite".to_string(),
        "Execute all automated tests including unit and integration tests".to_string(),
        StepType::Automated,
        {
            let mut config = HashMap::new();
            config.insert(
                "test_command".to_string(),
                serde_json::json!("cargo test --all"),
            );
            config.insert("coverage_threshold".to_string(), serde_json::json!(80));
            config
        },
        vec![build_step_id], // Depends on build
        Some(45),            // 45 minutes
        Some("ci-system".to_string()),
        Some("deployment-bot".to_string()),
    )?;

    let test_step_id = if let Some(cim_domain_workflow::WorkflowDomainEvent::StepAdded(ref event)) =
        test_events.first()
    {
        event.step_id
    } else {
        return Err("Failed to get test step ID".into());
    };

    // Step 4: Security Scan
    let security_events = workflow.add_step(
        "Security Scan".to_string(),
        "Perform security vulnerability scanning on the built artifacts".to_string(),
        StepType::Automated,
        {
            let mut config = HashMap::new();
            config.insert("scan_tool".to_string(), serde_json::json!("cargo audit"));
            config.insert("fail_on_high".to_string(), serde_json::json!(true));
            config
        },
        vec![build_step_id], // Parallel with tests, depends on build
        Some(20),            // 20 minutes
        Some("security-scanner".to_string()),
        Some("deployment-bot".to_string()),
    )?;

    let security_step_id =
        if let Some(cim_domain_workflow::WorkflowDomainEvent::StepAdded(ref event)) =
            security_events.first()
        {
            event.step_id
        } else {
            return Err("Failed to get security step ID".into());
        };

    // Step 5: Deploy to Staging
    let staging_deploy_events = workflow.add_step(
        "Deploy to Staging".to_string(),
        "Deploy the application to the staging environment for final validation".to_string(),
        StepType::Automated,
        {
            let mut config = HashMap::new();
            config.insert("environment".to_string(), serde_json::json!("staging"));
            config.insert(
                "deploy_script".to_string(),
                serde_json::json!("./scripts/deploy.sh"),
            );
            config
        },
        vec![test_step_id, security_step_id], // Depends on both tests and security scan
        Some(15),                             // 15 minutes
        Some("deployment-system".to_string()),
        Some("deployment-bot".to_string()),
    )?;

    let staging_deploy_step_id =
        if let Some(cim_domain_workflow::WorkflowDomainEvent::StepAdded(ref event)) =
            staging_deploy_events.first()
        {
            event.step_id
        } else {
            return Err("Failed to get staging deploy step ID".into());
        };

    // Step 6: Staging Validation
    let validation_events = workflow.add_step(
        "Staging Validation".to_string(),
        "Manual validation of the deployment in staging environment".to_string(),
        StepType::Manual,
        {
            let mut config = HashMap::new();
            config.insert(
                "validation_checklist".to_string(),
                serde_json::json!([
                    "Verify application starts successfully",
                    "Check all critical features work",
                    "Validate database migrations",
                    "Confirm monitoring and alerting"
                ]),
            );
            config
        },
        vec![staging_deploy_step_id], // Depends on staging deployment
        Some(120),                    // 2 hours
        Some("qa-team".to_string()),
        Some("deployment-bot".to_string()),
    )?;

    let validation_step_id =
        if let Some(cim_domain_workflow::WorkflowDomainEvent::StepAdded(ref event)) =
            validation_events.first()
        {
            event.step_id
        } else {
            return Err("Failed to get validation step ID".into());
        };

    // Step 7: Production Deployment
    let _production_deploy_events = workflow.add_step(
        "Deploy to Production".to_string(),
        "Deploy the validated application to production environment".to_string(),
        StepType::Approval,
        {
            let mut config = HashMap::new();
            config.insert("environment".to_string(), serde_json::json!("production"));
            config.insert("requires_approval".to_string(), serde_json::json!(true));
            config.insert(
                "approvers".to_string(),
                serde_json::json!(["tech-lead", "product-owner"]),
            );
            config
        },
        vec![validation_step_id], // Depends on staging validation
        Some(30),                 // 30 minutes
        Some("deployment-system".to_string()),
        Some("deployment-bot".to_string()),
    )?;

    Ok(workflow)
}
