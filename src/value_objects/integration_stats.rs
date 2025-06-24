//! Integration statistics value objects

use serde::{Deserialize, Serialize};

/// Statistics about integration retry attempts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationRetryStats {
    /// Name of the step
    pub step_name: String,
    /// Total number of attempts
    pub total_attempts: u32,
    /// Number of successful attempts
    pub successful_attempts: u32,
    /// Number of failed attempts
    pub failed_attempts: u32,
}

/// Circuit breaker status for a step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    /// Name of the step
    pub step_name: String,
    /// Current state of the circuit breaker
    pub state: String,
    /// Number of failures in current window
    pub failure_count: u32,
    /// Time until next retry (if in open state)
    pub next_retry_seconds: Option<u64>,
}

/// Async integration status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AsyncIntegrationStatus {
    /// Name of the step
    pub step_name: String,
    /// Pattern used (callback, polling, etc)
    pub pattern: String,
    /// Callback URL if applicable
    pub callback_url: Option<String>,
    /// Current status
    pub status: String,
} 