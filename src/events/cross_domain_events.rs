//! Cross-domain workflow events
//!
//! These events enable workflows to orchestrate operations across multiple CIM domains
//! while maintaining domain boundaries and event-driven architecture.

use crate::value_objects::{WorkflowId, StepId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};

/// A workflow is requesting an operation in another domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainOperationRequested {
    /// ID of the workflow making the request
    pub workflow_id: WorkflowId,
    /// ID of the step making the request
    pub step_id: StepId,
    /// Target domain (e.g., "document", "git", "identity")
    pub target_domain: String,
    /// Operation to perform in the target domain
    pub operation: String,
    /// Parameters for the operation
    pub parameters: Value,
    /// Correlation ID for tracking the operation
    pub correlation_id: String,
    /// When the request was made
    pub requested_at: DateTime<Utc>,
    /// User who initiated the request
    pub requested_by: Option<String>,
}

/// Another domain has completed an operation requested by a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainOperationCompleted {
    /// ID of the workflow that made the request
    pub workflow_id: WorkflowId,
    /// ID of the step that made the request
    pub step_id: StepId,
    /// Domain that completed the operation
    pub source_domain: String,
    /// Operation that was completed
    pub operation: String,
    /// Result of the operation
    pub result: OperationResult,
    /// Correlation ID linking to the original request
    pub correlation_id: String,
    /// When the operation completed
    pub completed_at: DateTime<Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Another domain has reported a failure for a requested operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainOperationFailed {
    /// ID of the workflow that made the request
    pub workflow_id: WorkflowId,
    /// ID of the step that made the request
    pub step_id: StepId,
    /// Domain that failed the operation
    pub source_domain: String,
    /// Operation that failed
    pub operation: String,
    /// Error details
    pub error: DomainError,
    /// Correlation ID linking to the original request
    pub correlation_id: String,
    /// When the failure occurred
    pub failed_at: DateTime<Utc>,
    /// Whether the operation can be retried
    pub retryable: bool,
}

/// Result of a cross-domain operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    /// Operation succeeded with data
    Success {
        /// Output data from the operation
        data: Value,
        /// Any warnings or notes
        warnings: Vec<String>,
    },
    /// Operation succeeded with no data
    Acknowledged,
    /// Operation was skipped due to conditions
    Skipped {
        /// Reason for skipping
        reason: String,
    },
}

/// Error from a cross-domain operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainError {
    /// Error code from the domain
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    pub details: Option<Value>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
}

/// A workflow is subscribing to events from another domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainEventSubscriptionRequested {
    /// ID of the workflow subscribing
    pub workflow_id: WorkflowId,
    /// ID of the step that needs the events
    pub step_id: StepId,
    /// Domain to subscribe to
    pub target_domain: String,
    /// Event pattern to subscribe to (e.g., "document.created", "git.commit.*")
    pub event_pattern: String,
    /// Filter criteria for events
    pub filter: Option<Value>,
    /// Correlation ID for the subscription
    pub subscription_id: String,
    /// When the subscription was requested
    pub requested_at: DateTime<Utc>,
}

/// A workflow is unsubscribing from domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainEventSubscriptionCancelled {
    /// ID of the workflow
    pub workflow_id: WorkflowId,
    /// ID of the subscription to cancel
    pub subscription_id: String,
    /// When the cancellation was requested
    pub cancelled_at: DateTime<Utc>,
}

/// An event from another domain that a workflow is subscribed to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainEventReceived {
    /// ID of the workflow that subscribed
    pub workflow_id: WorkflowId,
    /// ID of the step that receives the event
    pub step_id: StepId,
    /// Domain that emitted the event
    pub source_domain: String,
    /// Type of event
    pub event_type: String,
    /// Event payload
    pub event_data: Value,
    /// ID of the subscription
    pub subscription_id: String,
    /// When the event was received
    pub received_at: DateTime<Utc>,
}

/// Workflow is coordinating a distributed transaction across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainTransactionStarted {
    /// ID of the coordinating workflow
    pub workflow_id: WorkflowId,
    /// Unique transaction ID
    pub transaction_id: String,
    /// Domains participating in the transaction
    pub participating_domains: Vec<String>,
    /// Transaction timeout in seconds
    pub timeout_seconds: u32,
    /// When the transaction started
    pub started_at: DateTime<Utc>,
}

/// All domains have prepared their parts of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainTransactionPrepared {
    /// ID of the coordinating workflow
    pub workflow_id: WorkflowId,
    /// Transaction ID
    pub transaction_id: String,
    /// Domains that have prepared
    pub prepared_domains: Vec<String>,
    /// When all domains were prepared
    pub prepared_at: DateTime<Utc>,
}

/// Transaction committed across all domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainTransactionCommitted {
    /// ID of the coordinating workflow
    pub workflow_id: WorkflowId,
    /// Transaction ID
    pub transaction_id: String,
    /// When the transaction was committed
    pub committed_at: DateTime<Utc>,
}

/// Transaction rolled back across all domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainTransactionRolledBack {
    /// ID of the coordinating workflow
    pub workflow_id: WorkflowId,
    /// Transaction ID
    pub transaction_id: String,
    /// Reason for rollback
    pub reason: String,
    /// Which domain caused the rollback
    pub failed_domain: Option<String>,
    /// When the rollback occurred
    pub rolled_back_at: DateTime<Utc>,
}