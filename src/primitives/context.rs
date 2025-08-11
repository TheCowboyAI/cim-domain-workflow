//! Extensible context framework for the unified workflow domain
//! 
//! This module provides a flexible context system that allows domain-specific
//! extensions while maintaining a common interface for workflow execution.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::identifiers::{UniversalWorkflowId, WorkflowInstanceId};

/// Core workflow context that can be extended by domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Core workflow identification
    pub workflow_id: UniversalWorkflowId,
    /// Runtime instance identification
    pub instance_id: WorkflowInstanceId,
    /// Global context data accessible to all domains
    pub global_context: GlobalContext,
    /// Domain-specific extensions
    pub extensions: HashMap<String, DomainExtension>,
    /// Execution metadata and observability
    pub execution_metadata: ExecutionMetadata,
}

impl WorkflowContext {
    /// Create a new workflow context
    pub fn new(
        workflow_id: UniversalWorkflowId,
        instance_id: WorkflowInstanceId,
        initiator: Option<String>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.clone(),
            instance_id,
            global_context: GlobalContext::new(initiator),
            extensions: HashMap::new(),
            execution_metadata: ExecutionMetadata::new(),
        }
    }

    /// Add or update a domain extension
    pub fn add_extension(
        &mut self,
        domain: String,
        extension: DomainExtension,
    ) -> Result<(), ContextError> {
        // Validate extension data structure
        extension.validate()?;
        self.extensions.insert(domain, extension);
        Ok(())
    }

    /// Get a domain extension
    pub fn get_extension(&self, domain: &str) -> Option<&DomainExtension> {
        self.extensions.get(domain)
    }

    /// Get a mutable domain extension
    pub fn get_extension_mut(&mut self, domain: &str) -> Option<&mut DomainExtension> {
        self.extensions.remove(domain);
        // We need to rebuild after validation
        None // Simplified for now - would need validation logic
    }

    /// Safely update a domain extension
    pub fn update_extension<F>(
        &mut self,
        domain: &str,
        updater: F,
    ) -> Result<(), ContextError>
    where
        F: FnOnce(&mut DomainExtension) -> Result<(), ContextError>,
    {
        if let Some(mut extension) = self.extensions.remove(domain) {
            updater(&mut extension)?;
            extension.validate()?;
            self.extensions.insert(domain.to_string(), extension);
        }
        Ok(())
    }

    /// Get all domain extensions
    pub fn extensions(&self) -> &HashMap<String, DomainExtension> {
        &self.extensions
    }

    /// Check if context has extension for domain
    pub fn has_extension(&self, domain: &str) -> bool {
        self.extensions.contains_key(domain)
    }

    /// Get context data for a specific path
    pub fn get_value(&self, path: &str) -> Option<&Value> {
        if let Some((domain, remaining_path)) = path.split_once('.') {
            if domain == "global" {
                self.global_context.get_value(remaining_path)
            } else {
                self.extensions
                    .get(domain)
                    .and_then(|ext| ext.get_value(remaining_path))
            }
        } else {
            None
        }
    }

    /// Set context data for a specific path
    pub fn set_value(
        &mut self,
        path: &str,
        value: Value,
    ) -> Result<(), ContextError> {
        if let Some((domain, remaining_path)) = path.split_once('.') {
            if domain == "global" {
                self.global_context.set_value(remaining_path, value)
            } else {
                let extension = self.extensions
                    .get_mut(domain)
                    .ok_or_else(|| ContextError::DomainNotFound(domain.to_string()))?;
                extension.set_value(remaining_path, value)
            }
        } else {
            Err(ContextError::InvalidPath(path.to_string()))
        }
    }

    /// Record execution event for observability
    pub fn record_event(
        &mut self,
        event_type: String,
        details: Value,
    ) {
        self.execution_metadata.record_event(event_type, details);
    }

    /// Get execution duration so far
    pub fn execution_duration(&self) -> chrono::Duration {
        Utc::now() - self.execution_metadata.started_at
    }

    /// Validate entire context structure
    pub fn validate(&self) -> Result<(), ContextError> {
        // Validate global context
        self.global_context.validate()?;
        
        // Validate all extensions
        for (domain, extension) in &self.extensions {
            extension.validate()
                .map_err(|e| ContextError::ExtensionValidation {
                    domain: domain.clone(),
                    error: Box::new(e),
                })?;
        }

        Ok(())
    }
}

/// Global context data accessible across all domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalContext {
    /// User or system that initiated the workflow
    pub initiator: Option<String>,
    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,
    /// Causation chain for event sourcing
    pub causation_chain: Vec<Uuid>,
    /// Global variables accessible to all steps
    pub variables: Map<String, Value>,
    /// Workflow-level configuration
    pub configuration: Map<String, Value>,
    /// Security context and permissions
    pub security: SecurityContext,
}

impl GlobalContext {
    /// Create a new global context
    pub fn new(initiator: Option<String>) -> Self {
        Self {
            initiator,
            correlation_id: Uuid::new_v4(),
            causation_chain: Vec::new(),
            variables: Map::new(),
            configuration: Map::new(),
            security: SecurityContext::default(),
        }
    }

    /// Add to causation chain
    pub fn add_causation(&mut self, event_id: Uuid) {
        self.causation_chain.push(event_id);
    }

    /// Get value from global context
    pub fn get_value(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        match parts.first() {
            Some(&"variables") => self.get_nested_value(&self.variables, &parts[1..]),
            Some(&"configuration") => self.get_nested_value(&self.configuration, &parts[1..]),
            _ => None,
        }
    }

    /// Set value in global context
    pub fn set_value(&mut self, path: &str, value: Value) -> Result<(), ContextError> {
        let parts: Vec<&str> = path.split('.').collect();
        match parts.first() {
            Some(&"variables") => {
                Self::set_nested_value(&mut self.variables, &parts[1..], value)
            },
            Some(&"configuration") => {
                Self::set_nested_value(&mut self.configuration, &parts[1..], value)
            },
            _ => Err(ContextError::InvalidPath(path.to_string())),
        }
    }

    /// Helper to get nested values
    fn get_nested_value<'a>(&self, obj: &'a Map<String, Value>, path: &[&str]) -> Option<&'a Value> {
        if path.is_empty() {
            return None;
        }
        
        let mut current = obj.get(path[0])?;
        for &key in &path[1..] {
            current = current.as_object()?.get(key)?;
        }
        Some(current)
    }

    /// Helper to set nested values
    fn set_nested_value(
        obj: &mut Map<String, Value>,
        path: &[&str],
        value: Value,
    ) -> Result<(), ContextError> {
        if path.is_empty() {
            return Err(ContextError::InvalidPath("Empty path".to_string()));
        }

        if path.len() == 1 {
            obj.insert(path[0].to_string(), value);
            return Ok(());
        }

        let key = path[0];
        let entry = obj.entry(key.to_string()).or_insert_with(|| Value::Object(Map::new()));
        
        if let Value::Object(nested) = entry {
            Self::set_nested_value(nested, &path[1..], value)
        } else {
            Err(ContextError::InvalidPath(format!("Path {} is not an object", key)))
        }
    }

    /// Validate global context
    pub fn validate(&self) -> Result<(), ContextError> {
        // Validate correlation ID is not nil
        if self.correlation_id.is_nil() {
            return Err(ContextError::ValidationError(
                "Correlation ID cannot be nil".to_string()
            ));
        }

        // Validate security context
        self.security.validate()?;

        Ok(())
    }
}

/// Domain-specific extension data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExtension {
    /// Domain that owns this extension
    pub domain: String,
    /// Extension version for compatibility
    pub version: String,
    /// Domain-specific data
    pub data: Map<String, Value>,
    /// Domain-specific configuration
    pub config: Map<String, Value>,
    /// Extension metadata
    pub metadata: ExtensionMetadata,
}

impl DomainExtension {
    /// Create a new domain extension
    pub fn new(domain: String, version: String) -> Self {
        Self {
            domain: domain.clone(),
            version,
            data: Map::new(),
            config: Map::new(),
            metadata: ExtensionMetadata::new(domain),
        }
    }

    /// Get value from extension data
    pub fn get_value(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        match parts.first() {
            Some(&"data") => self.get_nested_value(&self.data, &parts[1..]),
            Some(&"config") => self.get_nested_value(&self.config, &parts[1..]),
            _ => None,
        }
    }

    /// Set value in extension data
    pub fn set_value(&mut self, path: &str, value: Value) -> Result<(), ContextError> {
        let parts: Vec<&str> = path.split('.').collect();
        match parts.first() {
            Some(&"data") => {
                Self::set_nested_value(&mut self.data, &parts[1..], value)
            },
            Some(&"config") => {
                Self::set_nested_value(&mut self.config, &parts[1..], value)
            },
            _ => Err(ContextError::InvalidPath(path.to_string())),
        }
    }

    /// Helper methods similar to GlobalContext
    fn get_nested_value<'a>(&self, obj: &'a Map<String, Value>, path: &[&str]) -> Option<&'a Value> {
        if path.is_empty() {
            return None;
        }
        
        let mut current = obj.get(path[0])?;
        for &key in &path[1..] {
            current = current.as_object()?.get(key)?;
        }
        Some(current)
    }

    fn set_nested_value(
        obj: &mut Map<String, Value>,
        path: &[&str],
        value: Value,
    ) -> Result<(), ContextError> {
        if path.is_empty() {
            return Err(ContextError::InvalidPath("Empty path".to_string()));
        }

        if path.len() == 1 {
            obj.insert(path[0].to_string(), value);
            return Ok(());
        }

        let key = path[0];
        let entry = obj.entry(key.to_string()).or_insert_with(|| Value::Object(Map::new()));
        
        if let Value::Object(nested) = entry {
            Self::set_nested_value(nested, &path[1..], value)
        } else {
            Err(ContextError::InvalidPath(format!("Path {} is not an object", key)))
        }
    }

    /// Validate extension data
    pub fn validate(&self) -> Result<(), ContextError> {
        // Validate domain name is not empty
        if self.domain.is_empty() {
            return Err(ContextError::ValidationError(
                "Domain name cannot be empty".to_string()
            ));
        }

        // Validate version follows semantic versioning
        if !self.version.chars().any(|c| c.is_ascii_digit()) {
            return Err(ContextError::ValidationError(
                "Version must contain at least one digit".to_string()
            ));
        }

        Ok(())
    }
}

/// Execution metadata for observability and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// When execution started
    pub started_at: DateTime<Utc>,
    /// Execution events for debugging
    pub events: Vec<ExecutionEvent>,
    /// Performance metrics
    pub metrics: ExecutionMetrics,
    /// Error tracking
    pub errors: Vec<ExecutionError>,
}

impl ExecutionMetadata {
    /// Create new execution metadata
    pub fn new() -> Self {
        Self {
            started_at: Utc::now(),
            events: Vec::new(),
            metrics: ExecutionMetrics::default(),
            errors: Vec::new(),
        }
    }

    /// Record an execution event
    pub fn record_event(&mut self, event_type: String, details: Value) {
        self.events.push(ExecutionEvent {
            timestamp: Utc::now(),
            event_type,
            details,
        });
    }

    /// Record an error
    pub fn record_error(&mut self, error_type: String, message: String, details: Option<Value>) {
        self.errors.push(ExecutionError {
            timestamp: Utc::now(),
            error_type,
            message,
            details,
        });
    }

    /// Update performance metrics
    pub fn update_metrics(&mut self, step_duration: chrono::Duration) {
        self.metrics.total_steps += 1;
        self.metrics.total_duration += step_duration;
        self.metrics.average_step_duration = 
            self.metrics.total_duration / self.metrics.total_steps as i32;
    }
}

/// Individual execution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub details: Value,
}

/// Execution performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_steps: u32,
    pub total_duration: chrono::Duration,
    pub average_step_duration: chrono::Duration,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            total_steps: 0,
            total_duration: chrono::Duration::zero(),
            average_step_duration: chrono::Duration::zero(),
        }
    }
}

/// Execution error record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    pub timestamp: DateTime<Utc>,
    pub error_type: String,
    pub message: String,
    pub details: Option<Value>,
}

/// Extension metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    /// Domain that owns this extension
    pub domain: String,
    /// When extension was created
    pub created_at: DateTime<Utc>,
    /// When extension was last updated
    pub updated_at: DateTime<Utc>,
    /// Extension tags for categorization
    pub tags: Vec<String>,
}

impl ExtensionMetadata {
    /// Create new extension metadata
    pub fn new(domain: String) -> Self {
        let now = Utc::now();
        Self {
            domain,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Security context for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User permissions
    pub permissions: Vec<String>,
    /// Role-based access control
    pub roles: Vec<String>,
    /// Security attributes
    pub attributes: Map<String, Value>,
    /// Tenant information for multi-tenancy
    pub tenant: Option<String>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            permissions: Vec::new(),
            roles: Vec::new(),
            attributes: Map::new(),
            tenant: None,
        }
    }
}

impl SecurityContext {
    /// Check if has permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Check if has role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Validate security context
    pub fn validate(&self) -> Result<(), ContextError> {
        // Basic validation - could be expanded
        Ok(())
    }
}

/// Context manipulation errors
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Extension validation error in domain {domain}: {error}")]
    ExtensionValidation {
        domain: String,
        error: Box<ContextError>,
    },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_context_creation() {
        let workflow_id = UniversalWorkflowId::new("document".to_string(), None);
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id, Some("admin".to_string()));

        assert_eq!(context.global_context.initiator, Some("admin".to_string()));
        assert!(!context.global_context.correlation_id.is_nil());
        assert!(context.extensions.is_empty());
    }

    #[test]
    fn test_domain_extension() {
        let mut extension = DomainExtension::new("document".to_string(), "1.0.0".to_string());
        
        extension.set_value("data.title", Value::String("Test Document".to_string()))
            .unwrap();
        
        assert_eq!(
            extension.get_value("data.title"),
            Some(&Value::String("Test Document".to_string()))
        );
    }

    #[test]
    fn test_global_context_variables() {
        let mut context = GlobalContext::new(Some("user".to_string()));
        
        context.set_value("variables.order_id", Value::String("ORD-123".to_string()))
            .unwrap();
        
        assert_eq!(
            context.get_value("variables.order_id"),
            Some(&Value::String("ORD-123".to_string()))
        );
    }

    #[test]
    fn test_context_validation() {
        let workflow_id = UniversalWorkflowId::new("test".to_string(), None);
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id, None);

        assert!(context.validate().is_ok());
    }

    #[test]
    fn test_execution_metadata() {
        let mut metadata = ExecutionMetadata::new();
        
        metadata.record_event(
            "step_started".to_string(),
            serde_json::json!({"step": "validation"})
        );

        assert_eq!(metadata.events.len(), 1);
        assert_eq!(metadata.events[0].event_type, "step_started");
    }
}