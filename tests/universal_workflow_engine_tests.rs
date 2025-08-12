//! Universal Workflow Engine Tests
//! 
//! Basic tests for Universal Workflow Engine features.
//! These tests validate the core functionality that's actually implemented.

use cim_domain_workflow::{
    primitives::{
        identifiers::{UniversalWorkflowId, WorkflowInstanceId},
        context::WorkflowContext,
    },
    algebra::{
        WorkflowEvent, LifecycleEventType, EventPayload, EventContext,
    },
};
use uuid::Uuid;

// =============================================================================
// Basic Universal Workflow ID Tests
// =============================================================================

#[test]
fn test_universal_workflow_id_creation() {
    // Test creating universal workflow IDs with domain and template info
    let domain = "document";
    let template_id = Some("doc-review-v1".to_string());
    
    let workflow_id = UniversalWorkflowId::new(domain.to_string(), template_id.clone());
    
    assert!(workflow_id.to_string().contains(domain));
    // Basic validation that the ID was created successfully
    assert!(!workflow_id.to_string().is_empty());
}

#[test] 
fn test_deterministic_id_generation() {
    // Test that same inputs produce consistent IDs
    let domain = "person";
    let template = Some("onboarding".to_string());
    
    let id1 = UniversalWorkflowId::new(domain.to_string(), template.clone());
    let id2 = UniversalWorkflowId::new(domain.to_string(), template.clone());
    
    // Should be consistent for same inputs
    assert_eq!(id1.to_string(), id2.to_string());
}

#[test]
fn test_workflow_instance_id_creation() {
    // Test creating workflow instance IDs
    let workflow_id = UniversalWorkflowId::new("document".to_string(), None);
    let instance_id = WorkflowInstanceId::new(workflow_id.clone());
    
    // Basic validation that the ID was created
    assert!(!instance_id.id().to_string().is_empty());
    assert_eq!(instance_id.workflow_id(), &workflow_id);
}

// =============================================================================
// Basic Workflow Context Tests  
// =============================================================================

#[test]
fn test_workflow_context_creation() {
    // Test creating workflow context
    let workflow_id = UniversalWorkflowId::new("test".to_string(), None);
    let instance_id = WorkflowInstanceId::new(workflow_id.clone());
    
    let context = WorkflowContext::new(
        workflow_id.clone(),
        instance_id.clone(),
        Some("test-system".to_string())
    );
    
    assert_eq!(&context.workflow_id, &workflow_id);
    assert_eq!(&context.instance_id, &instance_id);
    assert_eq!(context.global_context.initiator.as_ref().unwrap(), "test-system");
}

#[test]
fn test_workflow_context_variables() {
    // Test basic context variable handling
    let workflow_id = UniversalWorkflowId::new("test".to_string(), None);
    let instance_id = WorkflowInstanceId::new(workflow_id.clone());
    
    let mut context = WorkflowContext::new(
        workflow_id.clone(),
        instance_id.clone(),
        Some("test-system".to_string())
    );
    
    // Test initial variables
    assert!(!context.global_context.variables.is_empty()); // Should have metadata
    
    // Test adding variables
    context.global_context.variables.insert("test_key".to_string(), serde_json::json!("test_value"));
    assert!(context.global_context.variables.contains_key("test_key"));
}

// =============================================================================
// Basic Workflow Event Tests
// =============================================================================

#[test]
fn test_workflow_event_creation() {
    // Test creating workflow events
    let correlation_id = Uuid::new_v4();
    let context_id = Uuid::new_v4();
    
    let mut payload = EventPayload::empty();
    payload.set_data("test_key".to_string(), serde_json::json!("test_value"));
    
    let event = WorkflowEvent::lifecycle(
        LifecycleEventType::WorkflowCreated,
        "test".to_string(),
        correlation_id,
        payload,
        EventContext::for_workflow(context_id),
    );
    
    assert_eq!(event.domain, "test");
    assert_eq!(event.correlation_id, correlation_id);
    assert_eq!(event.type_name(), "WorkflowCreated");
}

#[test]
fn test_event_payload_operations() {
    // Test event payload operations
    let mut payload = EventPayload::empty();
    
    payload.set_data("string_key".to_string(), serde_json::json!("string_value"));
    payload.set_data("number_key".to_string(), serde_json::json!(42));
    payload.set_data("boolean_key".to_string(), serde_json::json!(true));
    
    assert_eq!(payload.get_data("string_key"), Some(&serde_json::json!("string_value")));
    assert_eq!(payload.get_data("number_key"), Some(&serde_json::json!(42)));
    assert_eq!(payload.get_data("boolean_key"), Some(&serde_json::json!(true)));
    assert_eq!(payload.get_data("missing_key"), None);
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_complete_workflow_lifecycle() {
    // Test a complete workflow lifecycle
    let domain = "integration_test";
    let workflow_id = UniversalWorkflowId::new(domain.to_string(), Some("test_template".to_string()));
    let instance_id = WorkflowInstanceId::new(workflow_id.clone());
    
    // Create context
    let context = WorkflowContext::new(
        workflow_id.clone(),
        instance_id.clone(),
        Some("integration_test_system".to_string())
    );
    
    // Create workflow created event
    let correlation_id = Uuid::new_v4();
    let mut payload = EventPayload::empty();
    payload.set_data("workflow_type".to_string(), serde_json::json!("integration_test"));
    
    let created_event = WorkflowEvent::lifecycle(
        LifecycleEventType::WorkflowCreated,
        domain.to_string(),
        correlation_id,
        payload,
        EventContext::for_workflow(*instance_id.id()),
    );
    
    // Verify event creation
    assert_eq!(created_event.domain, domain);
    assert_eq!(created_event.correlation_id, correlation_id);
    assert_eq!(created_event.type_name(), "WorkflowCreated");
    
    // Verify context consistency
    assert_eq!(context.workflow_id, workflow_id);
    assert_eq!(context.instance_id, instance_id);
}

#[test]
fn test_cross_domain_workflow_coordination() {
    // Test coordination between multiple domains
    let domains = vec!["document", "user", "notification"];
    let correlation_id = Uuid::new_v4();
    let mut events = Vec::new();
    
    for domain in domains {
        let workflow_id = UniversalWorkflowId::new(domain.to_string(), Some("coordination_test".to_string()));
        let instance_id = WorkflowInstanceId::new(workflow_id);
        
        let mut payload = EventPayload::empty();
        payload.set_data("domain".to_string(), serde_json::json!(domain));
        payload.set_data("coordination_id".to_string(), serde_json::json!(correlation_id.to_string()));
        
        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            domain.to_string(),
            correlation_id, // Same correlation ID for coordination
            payload,
            EventContext::for_workflow(*instance_id.id()),
        );
        
        events.push(event);
    }
    
    // Verify all events have the same correlation ID
    assert_eq!(events.len(), 3);
    for event in &events {
        assert_eq!(event.correlation_id, correlation_id);
    }
    
    // Verify different domains
    let event_domains: Vec<&str> = events.iter().map(|e| e.domain.as_str()).collect();
    assert_eq!(event_domains, vec!["document", "user", "notification"]);
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_workflow_context_edge_cases() {
    // Test edge cases for workflow context
    let workflow_id = UniversalWorkflowId::new("".to_string(), None);
    let instance_id = WorkflowInstanceId::new(workflow_id.clone());
    
    // Should handle empty domain
    let context = WorkflowContext::new(
        workflow_id.clone(),
        instance_id.clone(),
        None // No initiator
    );
    
    assert_eq!(context.workflow_id, workflow_id);
    assert_eq!(context.instance_id, instance_id);
    assert_eq!(context.global_context.initiator, None);
}

#[test]
fn test_event_payload_edge_cases() {
    // Test edge cases for event payloads
    let mut payload = EventPayload::empty();
    
    // Test empty string key
    payload.set_data("".to_string(), serde_json::json!("empty_key_value"));
    assert_eq!(payload.get_data(""), Some(&serde_json::json!("empty_key_value")));
    
    // Test null value
    payload.set_data("null_key".to_string(), serde_json::json!(null));
    assert_eq!(payload.get_data("null_key"), Some(&serde_json::json!(null)));
    
    // Test complex nested object
    let complex_object = serde_json::json!({
        "nested": {
            "array": [1, 2, 3],
            "object": {"key": "value"}
        }
    });
    payload.set_data("complex".to_string(), complex_object.clone());
    assert_eq!(payload.get_data("complex"), Some(&complex_object));
}