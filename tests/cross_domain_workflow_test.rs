//! Integration tests for cross-domain workflow functionality

use cim_domain_workflow::{
    Workflow,
    domain_events::WorkflowDomainEvent,
    events::*,
    handlers::{CrossDomainHandler, EventMetadata},
    value_objects::{WorkflowId, StepId, StepType},
};
use serde_json::json;
use chrono::Utc;

/// Test cross-domain operation request creation
#[test]
fn test_cross_domain_operation_request() {
    let workflow_id = WorkflowId::new();
    let step_id = StepId::new();
    
    let event = CrossDomainOperationRequested {
        workflow_id,
        step_id,
        target_domain: "document".to_string(),
        operation: "create_document".to_string(),
        parameters: json!({
            "title": "Test Document",
            "content": "This is a test document",
            "format": "markdown"
        }),
        correlation_id: "test-correlation-123".to_string(),
        requested_at: Utc::now(),
        requested_by: Some("test-user".to_string()),
    };
    
    assert_eq!(event.target_domain, "document");
    assert_eq!(event.operation, "create_document");
    assert_eq!(event.parameters["title"], "Test Document");
}

/// Test operation result handling
#[test]
fn test_operation_results() {
    // Test success result
    let success = OperationResult::Success {
        data: json!({
            "document_id": "doc-123",
            "created_at": "2025-08-02T10:00:00Z"
        }),
        warnings: vec!["Document title was truncated".to_string()],
    };
    
    match success {
        OperationResult::Success { data, warnings } => {
            assert_eq!(data["document_id"], "doc-123");
            assert_eq!(warnings.len(), 1);
        }
        _ => panic!("Expected success result"),
    }
    
    // Test acknowledged result
    let ack = OperationResult::Acknowledged;
    assert!(matches!(ack, OperationResult::Acknowledged));
    
    // Test skipped result
    let skipped = OperationResult::Skipped {
        reason: "Document already exists".to_string(),
    };
    
    match skipped {
        OperationResult::Skipped { reason } => {
            assert_eq!(reason, "Document already exists");
        }
        _ => panic!("Expected skipped result"),
    }
}

/// Test cross-domain error handling
#[test]
fn test_cross_domain_errors() {
    let error = DomainError {
        code: "DOC_NOT_FOUND".to_string(),
        message: "Document with ID doc-123 not found".to_string(),
        details: Some(json!({
            "searched_locations": ["database", "cache", "archive"]
        })),
        stack_trace: None,
    };
    
    assert_eq!(error.code, "DOC_NOT_FOUND");
    assert!(error.details.is_some());
}

/// Test transaction lifecycle events
#[test]
fn test_transaction_events() {
    let workflow_id = WorkflowId::new();
    let transaction_id = "txn-456".to_string();
    
    // Start transaction
    let start_event = CrossDomainTransactionStarted {
        workflow_id,
        transaction_id: transaction_id.clone(),
        participating_domains: vec!["document".to_string(), "identity".to_string(), "git".to_string()],
        timeout_seconds: 30,
        started_at: Utc::now(),
    };
    
    assert_eq!(start_event.participating_domains.len(), 3);
    assert_eq!(start_event.timeout_seconds, 30);
    
    // Prepare transaction
    let prepare_event = CrossDomainTransactionPrepared {
        workflow_id,
        transaction_id: transaction_id.clone(),
        prepared_domains: vec!["document".to_string(), "identity".to_string()],
        prepared_at: Utc::now(),
    };
    
    assert_eq!(prepare_event.prepared_domains.len(), 2);
    
    // Rollback transaction
    let rollback_event = CrossDomainTransactionRolledBack {
        workflow_id,
        transaction_id,
        reason: "Git domain failed to prepare".to_string(),
        failed_domain: Some("git".to_string()),
        rolled_back_at: Utc::now(),
    };
    
    assert_eq!(rollback_event.failed_domain, Some("git".to_string()));
}

/// Test event subscription
#[test]
fn test_event_subscription() {
    let workflow_id = WorkflowId::new();
    let step_id = StepId::new();
    
    let subscription = CrossDomainEventSubscriptionRequested {
        workflow_id,
        step_id,
        target_domain: "document".to_string(),
        event_pattern: "document.created".to_string(),
        filter: Some(json!({
            "format": "markdown",
            "author": "test-user"
        })),
        subscription_id: "sub-789".to_string(),
        requested_at: Utc::now(),
    };
    
    assert_eq!(subscription.event_pattern, "document.created");
    assert!(subscription.filter.is_some());
}

/// Test cross-domain workflow scenario
#[test]
fn test_cross_domain_workflow_scenario() {
    // Create a workflow that orchestrates across multiple domains
    let (mut workflow, _) = Workflow::new(
        "Cross-Domain Order Processing".to_string(),
        "Process order across inventory, payment, and shipping domains".to_string(),
        Default::default(),
        Some("test-user".to_string()),
    ).expect("Failed to create workflow");
    
    // Add step that interacts with inventory domain
    let check_inventory_events = workflow.add_step(
        "Check Inventory".to_string(),
        "Verify items are in stock".to_string(),
        StepType::Integration,
        Default::default(),
        vec![],
        Some(5),
        None,
        Some("test-user".to_string()),
    ).expect("Failed to add inventory step");
    
    assert_eq!(check_inventory_events.len(), 1);
    
    // Add step that interacts with payment domain
    let process_payment_events = workflow.add_step(
        "Process Payment".to_string(),
        "Charge customer payment method".to_string(),
        StepType::Integration,
        Default::default(),
        vec![],
        Some(10),
        None,
        Some("test-user".to_string()),
    ).expect("Failed to add payment step");
    
    assert_eq!(process_payment_events.len(), 1);
    
    // Add step that interacts with shipping domain
    let create_shipment_events = workflow.add_step(
        "Create Shipment".to_string(),
        "Schedule delivery with shipping provider".to_string(),
        StepType::Integration,
        Default::default(),
        vec![],
        Some(15),
        None,
        Some("test-user".to_string()),
    ).expect("Failed to add shipping step");
    
    assert_eq!(create_shipment_events.len(), 1);
    
    // Verify workflow has 3 integration steps
    assert_eq!(workflow.steps.len(), 3);
    let integration_steps: Vec<_> = workflow.steps.values()
        .filter(|s| s.step_type == StepType::Integration)
        .collect();
    assert_eq!(integration_steps.len(), 3);
}

/// Test event received from another domain
#[test]
fn test_cross_domain_event_received() {
    let workflow_id = WorkflowId::new();
    let step_id = StepId::new();
    
    let received = CrossDomainEventReceived {
        workflow_id,
        step_id,
        source_domain: "document".to_string(),
        event_type: "DocumentCreated".to_string(),
        event_data: json!({
            "document_id": "doc-999",
            "title": "Important Document",
            "created_by": "other-user",
            "created_at": "2025-08-02T12:00:00Z"
        }),
        subscription_id: "sub-123".to_string(),
        received_at: Utc::now(),
    };
    
    assert_eq!(received.source_domain, "document");
    assert_eq!(received.event_type, "DocumentCreated");
    assert_eq!(received.event_data["document_id"], "doc-999");
}

/// Test workflow domain event wrapping
#[test]
fn test_workflow_domain_event_wrapping() {
    let workflow_id = WorkflowId::new();
    let step_id = StepId::new();
    
    // Test wrapping operation requested
    let op_requested = WorkflowDomainEvent::CrossDomainOperationRequested(
        CrossDomainOperationRequested {
            workflow_id,
            step_id,
            target_domain: "git".to_string(),
            operation: "create_commit".to_string(),
            parameters: json!({"message": "Initial commit"}),
            correlation_id: "corr-123".to_string(),
            requested_at: Utc::now(),
            requested_by: Some("developer".to_string()),
        }
    );
    
    assert_eq!(op_requested.workflow_id(), workflow_id);
    assert_eq!(op_requested.event_name(), "CrossDomainOperationRequested");
    
    // Test wrapping operation completed
    let op_completed = WorkflowDomainEvent::CrossDomainOperationCompleted(
        CrossDomainOperationCompleted {
            workflow_id,
            step_id,
            source_domain: "git".to_string(),
            operation: "create_commit".to_string(),
            result: OperationResult::Success {
                data: json!({"commit_id": "abc123"}),
                warnings: vec![],
            },
            correlation_id: "corr-123".to_string(),
            completed_at: Utc::now(),
            duration_ms: 250,
        }
    );
    
    assert_eq!(op_completed.workflow_id(), workflow_id);
    assert_eq!(op_completed.event_name(), "CrossDomainOperationCompleted");
}

/// Test metadata correlation for cross-domain events
#[test]
fn test_cross_domain_event_correlation() {
    // Create root metadata for initial request
    let root_metadata = EventMetadata::create_root(Some("orchestrator".to_string()));
    
    // Create metadata for response (caused by request)
    let response_metadata = EventMetadata::create_caused_by(
        &root_metadata,
        2,
        Some("document-service".to_string()),
    );
    
    // Verify correlation chain
    assert_eq!(response_metadata.correlation_id, root_metadata.correlation_id);
    assert_eq!(response_metadata.causation_id.0, root_metadata.message_id.0);
    assert_eq!(response_metadata.sequence, 2);
    assert_eq!(response_metadata.actor, Some("document-service".to_string()));
}