//! Cross-domain workflow handler
//!
//! Handles workflow interactions with other CIM domains through event-driven patterns

use crate::{
    domain_events::WorkflowDomainEvent,
    events::{
        CrossDomainOperationRequested, CrossDomainOperationCompleted, 
        CrossDomainOperationFailed, CrossDomainEventReceived,
        CrossDomainTransactionStarted, OperationResult, DomainError as CrossDomainError,
    },
    handlers::{NatsEventPublisher, EventMetadata},
    value_objects::{WorkflowId, StepId},
};
use cim_domain::{DomainResult, DomainError};
use async_nats::{Client, HeaderMap};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;
use futures::StreamExt;

/// Handles cross-domain workflow operations
pub struct CrossDomainHandler {
    nats_client: Client,
    publisher: Arc<NatsEventPublisher>,
    /// Active operations waiting for responses
    pending_operations: Arc<Mutex<HashMap<String, PendingOperation>>>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashMap<String, async_nats::Subscriber>>>,
}

/// Information about a pending cross-domain operation
#[derive(Debug, Clone)]
struct PendingOperation {
    workflow_id: WorkflowId,
    step_id: StepId,
    operation: String,
    _target_domain: String,
    _requested_at: chrono::DateTime<chrono::Utc>,
}

impl CrossDomainHandler {
    /// Create a new cross-domain handler
    pub fn new(nats_client: Client, subject_prefix: String) -> Self {
        let publisher = Arc::new(NatsEventPublisher::new(nats_client.clone(), subject_prefix));
        
        Self {
            nats_client,
            publisher,
            pending_operations: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Request an operation in another domain
    pub async fn request_operation(
        &self,
        workflow_id: WorkflowId,
        step_id: StepId,
        target_domain: String,
        operation: String,
        parameters: Value,
        requested_by: Option<String>,
    ) -> DomainResult<String> {
        let correlation_id = Uuid::new_v4().to_string();
        let requested_at = Utc::now();

        // Create the request event
        let event = WorkflowDomainEvent::CrossDomainOperationRequested(
            CrossDomainOperationRequested {
                workflow_id,
                step_id,
                target_domain: target_domain.clone(),
                operation: operation.clone(),
                parameters: parameters.clone(),
                correlation_id: correlation_id.clone(),
                requested_at,
                requested_by: requested_by.clone(),
            }
        );

        // Store pending operation
        let pending = PendingOperation {
            workflow_id,
            step_id,
            operation: operation.clone(),
            _target_domain: target_domain.clone(),
            _requested_at: requested_at,
        };
        
        self.pending_operations.lock().await.insert(correlation_id.clone(), pending);

        // Create metadata for event
        let metadata = EventMetadata::create_root(requested_by);

        // Publish to workflow's event stream
        self.publisher.publish_event(&event, &metadata).await?;

        // Publish cross-domain request to target domain
        let subject = format!("domains.{}.requests", target_domain);
        let mut headers = HeaderMap::new();
        headers.insert("X-Correlation-ID", correlation_id.clone());
        headers.insert("X-Source-Domain", "workflow");
        headers.insert("X-Workflow-ID", workflow_id.to_string());
        headers.insert("X-Operation", operation.clone());

        let request_payload = json!({
            "workflow_id": workflow_id,
            "step_id": step_id,
            "operation": operation,
            "parameters": parameters,
            "correlation_id": correlation_id,
        });

        self.nats_client
            .publish_with_headers(subject, headers, serde_json::to_vec(&request_payload)?.into())
            .await
            .map_err(|e| DomainError::generic(&format!("Failed to publish cross-domain request: {}", e)))?;

        Ok(correlation_id)
    }

    /// Handle a response from another domain
    pub async fn handle_operation_response(
        &self,
        correlation_id: String,
        source_domain: String,
        success: bool,
        result: Value,
        duration_ms: u64,
    ) -> DomainResult<()> {
        // Look up pending operation
        let pending = {
            let mut ops = self.pending_operations.lock().await;
            ops.remove(&correlation_id)
        };

        let pending = pending.ok_or_else(|| 
            DomainError::generic(&format!("No pending operation for correlation ID: {}", correlation_id))
        )?;

        let event = if success {
            // Parse result
            let operation_result = if result.get("data").is_some() {
                OperationResult::Success {
                    data: result["data"].clone(),
                    warnings: result.get("warnings")
                        .and_then(|w| w.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect())
                        .unwrap_or_default(),
                }
            } else if result.get("skipped").is_some() {
                OperationResult::Skipped {
                    reason: result["reason"].as_str().unwrap_or("Unknown").to_string(),
                }
            } else {
                OperationResult::Acknowledged
            };

            WorkflowDomainEvent::CrossDomainOperationCompleted(
                CrossDomainOperationCompleted {
                    workflow_id: pending.workflow_id,
                    step_id: pending.step_id,
                    source_domain,
                    operation: pending.operation,
                    result: operation_result,
                    correlation_id,
                    completed_at: Utc::now(),
                    duration_ms,
                }
            )
        } else {
            // Parse error
            let error = CrossDomainError {
                code: result.get("code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("UNKNOWN")
                    .to_string(),
                message: result.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Operation failed")
                    .to_string(),
                details: result.get("details").cloned(),
                stack_trace: result.get("stack_trace")
                    .and_then(|s| s.as_str())
                    .map(String::from),
            };

            WorkflowDomainEvent::CrossDomainOperationFailed(
                CrossDomainOperationFailed {
                    workflow_id: pending.workflow_id,
                    step_id: pending.step_id,
                    source_domain,
                    operation: pending.operation,
                    error,
                    correlation_id,
                    failed_at: Utc::now(),
                    retryable: result.get("retryable")
                        .and_then(|r| r.as_bool())
                        .unwrap_or(false),
                }
            )
        };

        // Publish event
        let metadata = EventMetadata::create_root(None);
        self.publisher.publish_event(&event, &metadata).await?;

        Ok(())
    }

    /// Subscribe to events from another domain
    pub async fn subscribe_to_domain_events(
        &self,
        workflow_id: WorkflowId,
        step_id: StepId,
        target_domain: String,
        event_pattern: String,
        filter: Option<Value>,
    ) -> DomainResult<String> {
        let subscription_id = Uuid::new_v4().to_string();
        
        // Create subscription subject
        let subject = format!("domains.{}.events.{}", target_domain, event_pattern);
        
        // Subscribe to NATS
        let subscriber = self.nats_client
            .subscribe(subject)
            .await
            .map_err(|e| DomainError::generic(&format!("Failed to subscribe: {}", e)))?;

        // Store subscription
        self.subscriptions.lock().await.insert(subscription_id.clone(), subscriber);

        // Start processing events in background
        let handler = self.clone();
        let sub_id = subscription_id.clone();
        let wf_id = workflow_id;
        let s_id = step_id;
        let domain = target_domain.clone();
        
        tokio::spawn(async move {
            handler.process_domain_events(sub_id, wf_id, s_id, domain, filter).await;
        });

        Ok(subscription_id)
    }

    /// Process events from a domain subscription
    async fn process_domain_events(
        &self,
        subscription_id: String,
        workflow_id: WorkflowId,
        step_id: StepId,
        source_domain: String,
        filter: Option<Value>,
    ) {
        let subscriber = {
            let mut subs = self.subscriptions.lock().await;
            match subs.remove(&subscription_id) {
                Some(sub) => sub,
                None => return,
            }
        };

        let mut subscriber = subscriber;
        while let Some(msg) = subscriber.next().await {
            // Parse event
            if let Ok(event_data) = serde_json::from_slice::<Value>(&msg.payload) {
                // Apply filter if provided
                if let Some(filter) = &filter {
                    if !self.matches_filter(&event_data, filter) {
                        continue;
                    }
                }

                // Extract event type from headers or data
                let event_type = msg.headers.as_ref()
                    .and_then(|h| h.get("X-Event-Type"))
                    .map(|v| v.to_string())
                    .or_else(|| event_data.get("event_type")
                        .and_then(|t| t.as_str())
                        .map(String::from))
                    .unwrap_or_else(|| "unknown".to_string());

                // Create received event
                let received_event = WorkflowDomainEvent::CrossDomainEventReceived(
                    CrossDomainEventReceived {
                        workflow_id,
                        step_id,
                        source_domain: source_domain.clone(),
                        event_type,
                        event_data,
                        subscription_id: subscription_id.clone(),
                        received_at: Utc::now(),
                    }
                );

                // Publish to workflow
                let metadata = EventMetadata::create_root(None);
                if let Err(e) = self.publisher.publish_event(&received_event, &metadata).await {
                    eprintln!("Failed to publish received event: {}", e);
                }
            }
        }

        // Remove subscription when done
        self.subscriptions.lock().await.remove(&subscription_id);
    }

    /// Check if event data matches filter
    fn matches_filter(&self, data: &Value, filter: &Value) -> bool {
        // Simple equality check for now
        // Could be extended with more complex filtering
        match (data, filter) {
            (Value::Object(data_map), Value::Object(filter_map)) => {
                for (key, filter_val) in filter_map {
                    if let Some(data_val) = data_map.get(key) {
                        if data_val != filter_val {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => data == filter,
        }
    }

    /// Start a distributed transaction across domains
    pub async fn start_transaction(
        &self,
        workflow_id: WorkflowId,
        participating_domains: Vec<String>,
        timeout_seconds: u32,
    ) -> DomainResult<String> {
        let transaction_id = Uuid::new_v4().to_string();
        
        let event = WorkflowDomainEvent::CrossDomainTransactionStarted(
            CrossDomainTransactionStarted {
                workflow_id,
                transaction_id: transaction_id.clone(),
                participating_domains: participating_domains.clone(),
                timeout_seconds,
                started_at: Utc::now(),
            }
        );

        let metadata = EventMetadata::create_root(None);
        self.publisher.publish_event(&event, &metadata).await?;

        // Send prepare requests to all domains
        for domain in participating_domains {
            let subject = format!("domains.{}.transactions.prepare", domain);
            let mut headers = HeaderMap::new();
            headers.insert("X-Transaction-ID", transaction_id.clone());
            headers.insert("X-Workflow-ID", workflow_id.to_string());

            let prepare_msg = json!({
                "transaction_id": transaction_id,
                "workflow_id": workflow_id,
                "timeout_seconds": timeout_seconds,
            });

            self.nats_client
                .publish_with_headers(subject, headers, serde_json::to_vec(&prepare_msg)?.into())
                .await
                .map_err(|e| DomainError::generic(&format!("Failed to send prepare: {}", e)))?;
        }

        Ok(transaction_id)
    }
}

impl Clone for CrossDomainHandler {
    fn clone(&self) -> Self {
        Self {
            nats_client: self.nats_client.clone(),
            publisher: self.publisher.clone(),
            pending_operations: self.pending_operations.clone(),
            subscriptions: self.subscriptions.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_result_serialization() {
        // Test that operation results serialize correctly
        let success = OperationResult::Success {
            data: json!({"id": "123"}),
            warnings: vec!["Warning 1".to_string()],
        };
        
        let serialized = serde_json::to_string(&success).unwrap();
        assert!(serialized.contains("Success"));
        assert!(serialized.contains("123"));
        
        let ack = OperationResult::Acknowledged;
        let ack_json = serde_json::to_string(&ack).unwrap();
        assert_eq!(ack_json, "\"Acknowledged\"");
        
        let skipped = OperationResult::Skipped {
            reason: "Already processed".to_string(),
        };
        let skipped_json = serde_json::to_string(&skipped).unwrap();
        assert!(skipped_json.contains("Skipped"));
        assert!(skipped_json.contains("Already processed"));
    }

    #[test]
    fn test_domain_error_serialization() {
        let error = CrossDomainError {
            code: "NOT_FOUND".to_string(),
            message: "Resource not found".to_string(),
            details: Some(json!({"id": "123"})),
            stack_trace: None,
        };
        
        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("NOT_FOUND"));
        assert!(serialized.contains("Resource not found"));
        assert!(serialized.contains("123"));
    }

    #[test]
    fn test_filter_matching_simple() {
        // Create a standalone function to test filter logic
        fn matches_filter(data: &Value, filter: &Value) -> bool {
            match (data, filter) {
                (Value::Object(data_map), Value::Object(filter_map)) => {
                    for (key, filter_val) in filter_map {
                        if let Some(data_val) = data_map.get(key) {
                            if data_val != filter_val {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    true
                }
                _ => data == filter,
            }
        }
        
        // Test exact match
        let data = json!({"status": "active", "type": "order"});
        let filter = json!({"status": "active"});
        assert!(matches_filter(&data, &filter));

        // Test non-match
        let filter2 = json!({"status": "inactive"});
        assert!(!matches_filter(&data, &filter2));

        // Test multiple fields
        let filter3 = json!({"status": "active", "type": "order"});
        assert!(matches_filter(&data, &filter3));
    }
}