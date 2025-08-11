//! Integration tests for NATS event publishing

use cim_domain_workflow::{
    Workflow,
    handlers::{NatsEventPublisher, EventMetadata},
    domain_events::WorkflowDomainEvent,
};
use async_nats;
use futures::StreamExt;

/// Test workflow event publishing to NATS
/// 
/// This test is marked as ignored by default since it requires a running NATS server.
/// Run with: cargo test test_workflow_nats_publishing -- --ignored
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_workflow_nats_publishing() {
    // Connect to NATS (assumes local NATS server)
    let client = async_nats::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    // Create publisher
    let publisher = NatsEventPublisher::new(client.clone(), "events".to_string());

    // Subscribe to workflow events
    let mut subscriber = client
        .subscribe("events.workflow.>")
        .await
        .expect("Failed to subscribe");

    // Create a workflow and generate events
    let (_workflow, events) = Workflow::new(
        "Test NATS Workflow".to_string(),
        "Testing NATS integration".to_string(),
        Default::default(),
        Some("test-user".to_string()),
    ).expect("Failed to create workflow");

    // Create root metadata for correlation tracking
    let metadata = EventMetadata::create_root(Some("test-user".to_string()));

    // Publish events
    publisher
        .publish_events(&events, &metadata)
        .await
        .expect("Failed to publish events");

    // Verify we receive the event
    let msg = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        subscriber.next()
    )
    .await
    .expect("Timeout waiting for message")
    .expect("No message received");

    // Verify headers
    assert!(msg.headers.is_some());
    let headers = msg.headers.as_ref().unwrap();
    
    assert!(headers.get("X-Correlation-ID").is_some());
    assert!(headers.get("X-Causation-ID").is_some());
    assert!(headers.get("X-Message-ID").is_some());
    assert!(headers.get("X-Event-Type").is_some());
    assert_eq!(headers.get("X-Event-Type").unwrap().as_str(), "WorkflowCreated");
    
    // Verify payload
    let event: WorkflowDomainEvent = serde_json::from_slice(&msg.payload)
        .expect("Failed to deserialize event");
    
    assert_eq!(event.workflow_id(), _workflow.id);
    assert_eq!(event.event_name(), "WorkflowCreated");
}

/// Test correlation chain for multiple events
#[tokio::test]
#[ignore = "Requires running NATS server"]
async fn test_event_correlation_chain() {
    let client = async_nats::connect("nats://localhost:4222")
        .await
        .expect("Failed to connect to NATS");

    let publisher = NatsEventPublisher::new(client.clone(), "events".to_string());

    let mut subscriber = client
        .subscribe("events.workflow.>")
        .await
        .expect("Failed to subscribe");

    // Create workflow
    let (mut workflow, create_events) = Workflow::new(
        "Correlation Test Workflow".to_string(),
        "Testing event correlation".to_string(),
        Default::default(),
        Some("test-user".to_string()),
    ).expect("Failed to create workflow");

    // Start workflow
    let start_events = workflow.start(Default::default(), Some("test-user".to_string()))
        .expect("Failed to start workflow");

    // Combine all events
    let mut all_events = create_events;
    all_events.extend(start_events);

    // Create root metadata
    let root_metadata = EventMetadata::create_root(Some("test-user".to_string()));

    // Publish all events
    publisher
        .publish_events(&all_events, &root_metadata)
        .await
        .expect("Failed to publish events");

    // Collect messages
    let mut messages = Vec::new();
    for _ in 0..all_events.len() {
        let msg = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            subscriber.next()
        )
        .await
        .expect("Timeout waiting for message")
        .expect("No message received");
        
        messages.push(msg);
    }

    // Verify correlation chain
    let correlation_id = root_metadata.correlation_id.0.to_string();
    
    for (i, msg) in messages.iter().enumerate() {
        let headers = msg.headers.as_ref().expect("Missing headers");
        
        // All events should have same correlation ID
        assert_eq!(
            headers.get("X-Correlation-ID").unwrap().as_str(),
            correlation_id,
            "Event {} has wrong correlation ID", i
        );
        
        // First event should be self-caused
        if i == 0 {
            assert_eq!(
                headers.get("X-Causation-ID").unwrap().as_str(),
                headers.get("X-Message-ID").unwrap().as_str(),
                "First event should be self-caused"
            );
        } else {
            // Subsequent events should be caused by previous
            let prev_msg_id = messages[i-1].headers.as_ref().unwrap()
                .get("X-Message-ID").unwrap().as_str();
            assert_eq!(
                headers.get("X-Causation-ID").unwrap().as_str(),
                prev_msg_id,
                "Event {} has wrong causation ID", i
            );
        }
    }
}

/// Test mock NATS client for unit testing
#[tokio::test]
async fn test_mock_nats_publisher() {
    // This demonstrates how to test without a real NATS server
    // In a real implementation, we'd use a trait for the NATS client
    // and create a mock implementation for testing
    
    let (_workflow, events) = Workflow::new(
        "Mock Test Workflow".to_string(),
        "Testing with mock".to_string(),
        Default::default(),
        Some("test-user".to_string()),
    ).expect("Failed to create workflow");

    let metadata = EventMetadata::create_root(Some("test-user".to_string()));

    // Verify metadata creation
    assert_eq!(metadata.correlation_id.0, metadata.message_id.0);
    assert_eq!(metadata.causation_id.0, metadata.message_id.0);
    
    // Verify events were generated
    assert!(!events.is_empty());
    assert_eq!(events[0].event_name(), "WorkflowCreated");
}