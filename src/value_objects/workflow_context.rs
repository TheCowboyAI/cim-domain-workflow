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