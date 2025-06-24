//! Workflow execution context value object

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Workflow execution context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Context variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Current user or actor
    pub actor: Option<String>,
    /// Execution metadata
    pub metadata: HashMap<String, String>,
    /// Correlation ID for tracking across systems
    pub correlation_id: Option<Uuid>,
    /// Tenant or organization context
    pub tenant_id: Option<String>,
    /// Execution start time
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last updated time
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl WorkflowContext {
    /// Create a new workflow context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            actor: None,
            metadata: HashMap::new(),
            correlation_id: Some(Uuid::new_v4()),
            tenant_id: None,
            started_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
        }
    }

    /// Create a context with an actor
    pub fn with_actor(actor: String) -> Self {
        let mut context = Self::new();
        context.actor = Some(actor);
        context
    }

    /// Create a context with a tenant
    pub fn with_tenant(tenant_id: String) -> Self {
        let mut context = Self::new();
        context.tenant_id = Some(tenant_id);
        context
    }

    /// Set a variable in the context
    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Get a variable from the context
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// Set a string variable
    pub fn set_string(&mut self, key: String, value: String) {
        self.set_variable(key, serde_json::Value::String(value));
    }

    /// Get a string variable
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_variable(key)?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Set a number variable
    pub fn set_number(&mut self, key: String, value: f64) {
        self.set_variable(key, serde_json::json!(value));
    }

    /// Get a number variable
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get_variable(key)?
            .as_f64()
    }

    /// Set a boolean variable
    pub fn set_bool(&mut self, key: String, value: bool) {
        self.set_variable(key, serde_json::Value::Bool(value));
    }

    /// Get a boolean variable
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_variable(key)?
            .as_bool()
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set the actor
    pub fn set_actor(&mut self, actor: String) {
        self.actor = Some(actor);
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Set the tenant
    pub fn set_tenant(&mut self, tenant_id: String) {
        self.tenant_id = Some(tenant_id);
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Check if the context has a specific variable
    pub fn has_variable(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    /// Check if the context has a specific metadata key
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Merge another context into this one
    pub fn merge(&mut self, other: &WorkflowContext) {
        for (key, value) in &other.variables {
            self.variables.insert(key.clone(), value.clone());
        }
        for (key, value) in &other.metadata {
            self.metadata.insert(key.clone(), value.clone());
        }
        if other.actor.is_some() {
            self.actor = other.actor.clone();
        }
        if other.tenant_id.is_some() {
            self.tenant_id = other.tenant_id.clone();
        }
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Create a snapshot of the current context
    pub fn snapshot(&self) -> WorkflowContext {
        self.clone()
    }

    /// Clear all variables but keep metadata and identity
    pub fn clear_variables(&mut self) {
        self.variables.clear();
        self.updated_at = Some(chrono::Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test context creation
    ///
    /// ```mermaid
    /// graph TD
    ///     A[New Context] --> B[Default Values]
    ///     B --> C[Correlation ID]
    ///     B --> D[Timestamps]
    ///     B --> E[Empty Collections]
    /// ```
    #[test]
    fn test_context_creation() {
        let context = WorkflowContext::new();

        assert!(context.variables.is_empty());
        assert!(context.metadata.is_empty());
        assert!(context.correlation_id.is_some());
        assert!(context.started_at.is_some());
        assert!(context.updated_at.is_some());
        assert!(context.actor.is_none());
        assert!(context.tenant_id.is_none());
    }

    /// Test context with actor
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create with Actor] --> B[Actor Set]
    ///     B --> C[Other Fields Default]
    /// ```
    #[test]
    fn test_context_with_actor() {
        let context = WorkflowContext::with_actor("john_doe".to_string());

        assert_eq!(context.actor, Some("john_doe".to_string()));
        assert!(context.correlation_id.is_some());
        assert!(context.started_at.is_some());
    }

    /// Test context with tenant
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Create with Tenant] --> B[Tenant Set]
    ///     B --> C[Other Fields Default]
    /// ```
    #[test]
    fn test_context_with_tenant() {
        let context = WorkflowContext::with_tenant("tenant_123".to_string());

        assert_eq!(context.tenant_id, Some("tenant_123".to_string()));
        assert!(context.correlation_id.is_some());
        assert!(context.started_at.is_some());
    }

    /// Test variable operations
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Set Variable] --> B[Update Timestamp]
    ///     B --> C[Get Variable]
    ///     C --> D[Check Existence]
    /// ```
    #[test]
    fn test_variable_operations() {
        let mut context = WorkflowContext::new();
        let initial_updated = context.updated_at;

        // Add a delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Set string variable
        context.set_string("name".to_string(), "test".to_string());
        assert_eq!(context.get_string("name"), Some("test".to_string()));
        assert!(context.has_variable("name"));
        assert!(context.updated_at > initial_updated);

        // Set number variable
        context.set_number("count".to_string(), 42.5);
        assert_eq!(context.get_number("count"), Some(42.5));

        // Set boolean variable
        context.set_bool("active".to_string(), true);
        assert_eq!(context.get_bool("active"), Some(true));

        // Set complex JSON variable
        let json_value = serde_json::json!({
            "nested": {
                "key": "value"
            }
        });
        context.set_variable("complex".to_string(), json_value.clone());
        assert_eq!(context.get_variable("complex"), Some(&json_value));

        // Non-existent variable
        assert!(context.get_variable("nonexistent").is_none());
        assert!(!context.has_variable("nonexistent"));
    }

    /// Test metadata operations
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Set Metadata] --> B[Update Timestamp]
    ///     B --> C[Get Metadata]
    ///     C --> D[Check Existence]
    /// ```
    #[test]
    fn test_metadata_operations() {
        let mut context = WorkflowContext::new();

        // Set metadata
        context.set_metadata("environment".to_string(), "production".to_string());
        assert_eq!(context.get_metadata("environment"), Some(&"production".to_string()));
        assert!(context.has_metadata("environment"));

        // Non-existent metadata
        assert!(context.get_metadata("nonexistent").is_none());
        assert!(!context.has_metadata("nonexistent"));
    }

    /// Test context merge
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Context 1] --> C[Merge]
    ///     B[Context 2] --> C
    ///     C --> D[Combined Context]
    ///     D --> E[Updated Timestamp]
    /// ```
    #[test]
    fn test_context_merge() {
        let mut context1 = WorkflowContext::new();
        context1.set_string("var1".to_string(), "value1".to_string());
        context1.set_metadata("meta1".to_string(), "metavalue1".to_string());

        let mut context2 = WorkflowContext::new();
        context2.set_string("var2".to_string(), "value2".to_string());
        context2.set_metadata("meta2".to_string(), "metavalue2".to_string());
        context2.set_actor("actor2".to_string());
        context2.set_tenant("tenant2".to_string());

        // Merge context2 into context1
        context1.merge(&context2);

        // Check all values are present
        assert_eq!(context1.get_string("var1"), Some("value1".to_string()));
        assert_eq!(context1.get_string("var2"), Some("value2".to_string()));
        assert_eq!(context1.get_metadata("meta1"), Some(&"metavalue1".to_string()));
        assert_eq!(context1.get_metadata("meta2"), Some(&"metavalue2".to_string()));
        assert_eq!(context1.actor, Some("actor2".to_string()));
        assert_eq!(context1.tenant_id, Some("tenant2".to_string()));
    }

    /// Test context snapshot
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Original Context] --> B[Snapshot]
    ///     B --> C[Independent Copy]
    ///     A --> D[Modify Original]
    ///     C --> E[Unchanged]
    /// ```
    #[test]
    fn test_context_snapshot() {
        let mut original = WorkflowContext::new();
        original.set_string("key".to_string(), "value".to_string());

        let snapshot = original.snapshot();

        // Modify original
        original.set_string("key".to_string(), "new_value".to_string());
        original.set_string("key2".to_string(), "value2".to_string());

        // Snapshot should be unchanged
        assert_eq!(snapshot.get_string("key"), Some("value".to_string()));
        assert!(!snapshot.has_variable("key2"));
    }

    /// Test clear variables
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Context with Vars] --> B[Clear Variables]
    ///     B --> C[Empty Variables]
    ///     B --> D[Metadata Preserved]
    ///     B --> E[Identity Preserved]
    /// ```
    #[test]
    fn test_clear_variables() {
        let mut context = WorkflowContext::new();
        context.set_string("var1".to_string(), "value1".to_string());
        context.set_string("var2".to_string(), "value2".to_string());
        context.set_metadata("meta1".to_string(), "metavalue1".to_string());
        context.set_actor("actor".to_string());

        let correlation_id = context.correlation_id;

        // Clear variables
        context.clear_variables();

        // Variables should be empty
        assert!(context.variables.is_empty());
        assert!(!context.has_variable("var1"));
        assert!(!context.has_variable("var2"));

        // Metadata and identity should be preserved
        assert_eq!(context.get_metadata("meta1"), Some(&"metavalue1".to_string()));
        assert_eq!(context.actor, Some("actor".to_string()));
        assert_eq!(context.correlation_id, correlation_id);
    }

    /// Test type conversions
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Set as JSON] --> B[Get as String]
    ///     B --> C{Type Match?}
    ///     C -->|Yes| D[Return Value]
    ///     C -->|No| E[Return None]
    /// ```
    #[test]
    fn test_type_conversions() {
        let mut context = WorkflowContext::new();

        // Set number, try to get as string
        context.set_number("num".to_string(), 42.0);
        assert!(context.get_string("num").is_none());
        assert_eq!(context.get_number("num"), Some(42.0));

        // Set string, try to get as number
        context.set_string("str".to_string(), "not_a_number".to_string());
        assert!(context.get_number("str").is_none());
        assert_eq!(context.get_string("str"), Some("not_a_number".to_string()));

        // Set bool, try to get as other types
        context.set_bool("bool".to_string(), true);
        assert!(context.get_string("bool").is_none());
        assert!(context.get_number("bool").is_none());
        assert_eq!(context.get_bool("bool"), Some(true));
    }
} 