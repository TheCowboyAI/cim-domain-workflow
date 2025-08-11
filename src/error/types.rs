//! Unified Error Types and Categories
//!
//! Provides a comprehensive error taxonomy for the workflow system with proper
//! categorization, context preservation, and error correlation for debugging.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Unified workflow system error with comprehensive context and categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    /// Error identifier for correlation and tracking
    pub error_id: Uuid,
    /// Error category for handling decisions
    pub category: ErrorCategory,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Human-readable error message
    pub message: String,
    /// Structured error details
    pub details: ErrorDetails,
    /// Error context information
    pub context: ErrorContext,
    /// When the error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional underlying error chain
    pub caused_by: Option<Box<WorkflowError>>,
}

/// Error categories for proper handling and routing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Domain-specific business logic errors
    Domain,
    /// Infrastructure and external service errors
    Infrastructure,
    /// Data validation and constraint errors
    Validation,
    /// Authorization and security errors
    Authorization,
    /// Network and communication errors
    Network,
    /// Resource availability errors (memory, disk, etc.)
    Resource,
    /// Temporary errors that may resolve with retry
    Transient,
    /// Configuration and setup errors
    Configuration,
    /// Internal system errors and bugs
    Internal,
    /// External dependency errors
    Dependency,
}

/// Error severity levels for prioritization and handling
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low-impact errors that don't affect workflow execution
    Info,
    /// Non-critical errors with potential workarounds
    Warning,
    /// Errors that prevent current operation but allow recovery
    Error,
    /// Critical errors requiring immediate attention
    Critical,
    /// System-level errors requiring shutdown or maintenance
    Fatal,
}

/// Structured error details for specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorDetails {
    /// Domain validation error
    DomainValidation {
        domain: String,
        field: String,
        value: serde_json::Value,
        constraint: String,
    },
    /// Template processing error
    TemplateError {
        template_id: String,
        step_id: Option<String>,
        parameter: Option<String>,
        reason: String,
    },
    /// NATS messaging error
    MessagingError {
        operation: String,
        subject: Option<String>,
        broker_error: String,
    },
    /// Workflow execution error
    ExecutionError {
        workflow_id: Uuid,
        step_id: Option<String>,
        state: String,
        reason: String,
    },
    /// External service error
    ServiceError {
        service: String,
        endpoint: Option<String>,
        status_code: Option<u16>,
        response: Option<String>,
    },
    /// Resource constraint error
    ResourceError {
        resource_type: String,
        requested: Option<u64>,
        available: Option<u64>,
        limit: Option<u64>,
    },
    /// Network connectivity error
    NetworkError {
        host: String,
        port: Option<u16>,
        protocol: String,
        timeout: Option<std::time::Duration>,
    },
    /// Configuration error
    ConfigurationError {
        component: String,
        setting: String,
        value: Option<String>,
        expected: String,
    },
    /// Generic error with custom details
    Generic {
        code: String,
        details: HashMap<String, serde_json::Value>,
    },
}

/// Error context for debugging and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Workflow instance ID if applicable
    pub workflow_id: Option<Uuid>,
    /// Domain context
    pub domain: Option<String>,
    /// Operation being performed
    pub operation: String,
    /// Request correlation ID
    pub correlation_id: Option<Uuid>,
    /// User or system context
    pub actor: Option<String>,
    /// Additional context data
    pub metadata: HashMap<String, serde_json::Value>,
    /// Call stack or trace information
    pub trace: Vec<String>,
}

/// Error recovery suggestions and actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecovery {
    /// Whether error is recoverable
    pub recoverable: bool,
    /// Suggested retry policy
    pub retry_policy: Option<RetryPolicy>,
    /// Manual recovery actions
    pub manual_actions: Vec<String>,
    /// Automated recovery attempts
    pub auto_recovery: Vec<RecoveryAction>,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: std::time::Duration,
    /// Maximum delay between retries
    pub max_delay: std::time::Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter to prevent thundering herd
    pub jitter: bool,
    /// Conditions under which to retry
    pub retry_conditions: Vec<RetryCondition>,
}

/// Conditions that determine when to retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    /// Retry on specific error categories
    ErrorCategory(ErrorCategory),
    /// Retry on specific error codes
    ErrorCode(String),
    /// Retry on HTTP status codes
    HttpStatus(u16),
    /// Retry on timeout errors
    Timeout,
    /// Custom retry condition
    Custom(String),
}

/// Automated recovery actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Retry the failed operation
    Retry {
        policy: RetryPolicy,
    },
    /// Fallback to alternative method
    Fallback {
        method: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    /// Circuit breaker activation
    CircuitBreaker {
        duration: std::time::Duration,
    },
    /// Scale resources
    Scale {
        resource: String,
        target: u32,
    },
    /// Restart component
    Restart {
        component: String,
        graceful: bool,
    },
    /// Alert operations team
    Alert {
        severity: AlertSeverity,
        message: String,
    },
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl WorkflowError {
    /// Create a new workflow error
    pub fn new(
        category: ErrorCategory,
        severity: ErrorSeverity,
        message: String,
        details: ErrorDetails,
        context: ErrorContext,
    ) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            category,
            severity,
            message,
            details,
            context,
            timestamp: chrono::Utc::now(),
            caused_by: None,
        }
    }

    /// Create a domain validation error
    pub fn domain_validation(
        domain: String,
        field: String,
        value: serde_json::Value,
        constraint: String,
        context: ErrorContext,
    ) -> Self {
        Self::new(
            ErrorCategory::Domain,
            ErrorSeverity::Error,
            format!("Domain validation failed for {}.{}: {}", domain, field, constraint),
            ErrorDetails::DomainValidation {
                domain,
                field,
                value,
                constraint,
            },
            context,
        )
    }

    /// Create a template processing error
    pub fn template_error(
        template_id: String,
        step_id: Option<String>,
        parameter: Option<String>,
        reason: String,
        context: ErrorContext,
    ) -> Self {
        let message = match (&step_id, &parameter) {
            (Some(step), Some(param)) => {
                format!("Template {} step {} parameter {}: {}", template_id, step, param, reason)
            }
            (Some(step), None) => {
                format!("Template {} step {}: {}", template_id, step, reason)
            }
            _ => format!("Template {}: {}", template_id, reason),
        };

        Self::new(
            ErrorCategory::Validation,
            ErrorSeverity::Error,
            message,
            ErrorDetails::TemplateError {
                template_id,
                step_id,
                parameter,
                reason,
            },
            context,
        )
    }

    /// Create a messaging error
    pub fn messaging_error(
        operation: String,
        subject: Option<String>,
        broker_error: String,
        context: ErrorContext,
    ) -> Self {
        let message = match &subject {
            Some(subj) => format!("Messaging {} on {}: {}", operation, subj, broker_error),
            None => format!("Messaging {}: {}", operation, broker_error),
        };

        Self::new(
            ErrorCategory::Infrastructure,
            ErrorSeverity::Error,
            message,
            ErrorDetails::MessagingError {
                operation,
                subject,
                broker_error,
            },
            context,
        )
    }

    /// Create an execution error
    pub fn execution_error(
        workflow_id: Uuid,
        step_id: Option<String>,
        state: String,
        reason: String,
        context: ErrorContext,
    ) -> Self {
        let message = match &step_id {
            Some(step) => format!("Workflow {} step {} in state {}: {}", workflow_id, step, state, reason),
            None => format!("Workflow {} in state {}: {}", workflow_id, state, reason),
        };

        Self::new(
            ErrorCategory::Domain,
            ErrorSeverity::Error,
            message,
            ErrorDetails::ExecutionError {
                workflow_id,
                step_id,
                state,
                reason,
            },
            context,
        )
    }

    /// Create a service error
    pub fn service_error(
        service: String,
        endpoint: Option<String>,
        status_code: Option<u16>,
        response: Option<String>,
        context: ErrorContext,
    ) -> Self {
        let message = match (&endpoint, &status_code) {
            (Some(ep), Some(code)) => {
                format!("Service {} endpoint {} returned {}: {}", service, ep, code, 
                       response.as_deref().unwrap_or(""))
            }
            (Some(ep), None) => {
                format!("Service {} endpoint {} error: {}", service, ep, 
                       response.as_deref().unwrap_or("Unknown error"))
            }
            _ => format!("Service {} error: {}", service, response.as_deref().unwrap_or("Unknown error")),
        };

        let severity = match status_code {
            Some(code) if code >= 500 => ErrorSeverity::Critical,
            Some(code) if code >= 400 => ErrorSeverity::Error,
            _ => ErrorSeverity::Warning,
        };

        Self::new(
            ErrorCategory::Dependency,
            severity,
            message,
            ErrorDetails::ServiceError {
                service,
                endpoint,
                status_code,
                response,
            },
            context,
        )
    }

    /// Create a network error
    pub fn network_error(
        host: String,
        port: Option<u16>,
        protocol: String,
        timeout: Option<std::time::Duration>,
        context: ErrorContext,
    ) -> Self {
        let message = match (port, timeout) {
            (Some(p), Some(t)) => {
                format!("Network error connecting to {}:{} via {} (timeout: {:?})", host, p, protocol, t)
            }
            (Some(p), None) => {
                format!("Network error connecting to {}:{} via {}", host, p, protocol)
            }
            _ => format!("Network error connecting to {} via {}", host, protocol),
        };

        Self::new(
            ErrorCategory::Network,
            ErrorSeverity::Error,
            message,
            ErrorDetails::NetworkError {
                host,
                port,
                protocol,
                timeout,
            },
            context,
        )
    }

    /// Chain this error with a causing error
    pub fn caused_by(mut self, cause: WorkflowError) -> Self {
        self.caused_by = Some(Box::new(cause));
        self
    }

    /// Add context metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.metadata.insert(key, value);
        self
    }

    /// Add trace information
    pub fn with_trace(mut self, trace: String) -> Self {
        self.context.trace.push(trace);
        self
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self.category, 
            ErrorCategory::Transient | 
            ErrorCategory::Network | 
            ErrorCategory::Infrastructure
        ) && self.severity < ErrorSeverity::Fatal
    }

    /// Get suggested retry policy
    pub fn retry_policy(&self) -> Option<RetryPolicy> {
        if !self.is_recoverable() {
            return None;
        }

        let (max_attempts, initial_delay, backoff_multiplier) = match self.category {
            ErrorCategory::Network => (5, std::time::Duration::from_millis(100), 2.0),
            ErrorCategory::Infrastructure => (3, std::time::Duration::from_millis(500), 1.5),
            ErrorCategory::Transient => (3, std::time::Duration::from_millis(200), 2.0),
            _ => return None,
        };

        Some(RetryPolicy {
            max_attempts,
            initial_delay,
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier,
            jitter: true,
            retry_conditions: vec![
                RetryCondition::ErrorCategory(self.category.clone()),
            ],
        })
    }

    /// Get error recovery suggestions
    pub fn recovery(&self) -> ErrorRecovery {
        ErrorRecovery {
            recoverable: self.is_recoverable(),
            retry_policy: self.retry_policy(),
            manual_actions: self.manual_actions(),
            auto_recovery: self.auto_recovery_actions(),
        }
    }

    fn manual_actions(&self) -> Vec<String> {
        match &self.category {
            ErrorCategory::Network => vec![
                "Check network connectivity".to_string(),
                "Verify DNS resolution".to_string(),
                "Check firewall rules".to_string(),
            ],
            ErrorCategory::Infrastructure => vec![
                "Check service health".to_string(),
                "Verify configuration".to_string(),
                "Review resource usage".to_string(),
            ],
            ErrorCategory::Configuration => vec![
                "Review configuration settings".to_string(),
                "Check environment variables".to_string(),
                "Validate configuration schema".to_string(),
            ],
            _ => vec!["Review error details and context".to_string()],
        }
    }

    fn auto_recovery_actions(&self) -> Vec<RecoveryAction> {
        let mut actions = Vec::new();

        if let Some(retry_policy) = self.retry_policy() {
            actions.push(RecoveryAction::Retry { policy: retry_policy });
        }

        match self.severity {
            ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                actions.push(RecoveryAction::Alert {
                    severity: AlertSeverity::Critical,
                    message: self.message.clone(),
                });
            }
            ErrorSeverity::Error => {
                actions.push(RecoveryAction::Alert {
                    severity: AlertSeverity::High,
                    message: self.message.clone(),
                });
            }
            _ => {}
        }

        actions
    }
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: String) -> Self {
        Self {
            workflow_id: None,
            domain: None,
            operation,
            correlation_id: None,
            actor: None,
            metadata: HashMap::new(),
            trace: Vec::new(),
        }
    }

    /// Create context for a workflow operation
    pub fn for_workflow(workflow_id: Uuid, operation: String) -> Self {
        Self {
            workflow_id: Some(workflow_id),
            domain: None,
            operation,
            correlation_id: None,
            actor: None,
            metadata: HashMap::new(),
            trace: Vec::new(),
        }
    }

    /// Create context for a domain operation
    pub fn for_domain(domain: String, operation: String) -> Self {
        Self {
            workflow_id: None,
            domain: Some(domain),
            operation,
            correlation_id: None,
            actor: None,
            metadata: HashMap::new(),
            trace: Vec::new(),
        }
    }

    /// Add correlation ID
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Add actor information
    pub fn with_actor(mut self, actor: String) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add trace information
    pub fn with_trace(mut self, trace: String) -> Self {
        self.trace.push(trace);
        self
    }
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {} ({})", 
               self.severity.to_string().to_uppercase(),
               self.category.to_string().to_uppercase(),
               self.message,
               self.error_id)
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ErrorCategory::Domain => "DOMAIN",
            ErrorCategory::Infrastructure => "INFRA",
            ErrorCategory::Validation => "VALIDATION",
            ErrorCategory::Authorization => "AUTH",
            ErrorCategory::Network => "NETWORK",
            ErrorCategory::Resource => "RESOURCE",
            ErrorCategory::Transient => "TRANSIENT",
            ErrorCategory::Configuration => "CONFIG",
            ErrorCategory::Internal => "INTERNAL",
            ErrorCategory::Dependency => "DEPENDENCY",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ErrorSeverity::Info => "INFO",
            ErrorSeverity::Warning => "WARN",
            ErrorSeverity::Error => "ERROR",
            ErrorSeverity::Critical => "CRITICAL",
            ErrorSeverity::Fatal => "FATAL",
        };
        write!(f, "{}", name)
    }
}

impl std::error::Error for WorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.caused_by.as_ref().map(|e| e.as_ref() as &dyn std::error::Error)
    }
}

/// Result type for workflow operations
pub type WorkflowResult<T> = Result<T, WorkflowError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_error_creation() {
        let context = ErrorContext::new("test_operation".to_string());
        let error = WorkflowError::domain_validation(
            "test_domain".to_string(),
            "test_field".to_string(),
            serde_json::json!("invalid_value"),
            "must be positive".to_string(),
            context,
        );

        assert_eq!(error.category, ErrorCategory::Domain);
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(error.message.contains("Domain validation failed"));
    }

    #[test]
    fn test_error_recoverability() {
        let context = ErrorContext::new("network_test".to_string());
        let error = WorkflowError::network_error(
            "example.com".to_string(),
            Some(80),
            "http".to_string(),
            Some(std::time::Duration::from_secs(30)),
            context,
        );

        assert!(error.is_recoverable());
        assert!(error.retry_policy().is_some());
    }

    #[test]
    fn test_error_chaining() {
        let context = ErrorContext::new("test".to_string());
        let cause = WorkflowError::network_error(
            "db.example.com".to_string(),
            Some(5432),
            "tcp".to_string(),
            None,
            context.clone(),
        );

        let error = WorkflowError::service_error(
            "user_service".to_string(),
            Some("/api/users".to_string()),
            Some(503),
            Some("Service Unavailable".to_string()),
            context,
        ).caused_by(cause);

        assert!(error.caused_by.is_some());
    }
}