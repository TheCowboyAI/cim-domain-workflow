//! Domain extension trait system
//! 
//! This module defines the extension traits that domains implement to integrate
//! with the unified workflow engine through composition.

use std::future::Future;
use std::pin::Pin;
use serde_json::Value;

use crate::primitives::{UniversalWorkflowId, UniversalStepId, WorkflowContext};
use crate::core::{WorkflowExecutionResult, StepExecutionResult, ValidationResult};

/// Trait for domain-specific workflow extensions
/// Using boxed futures to make the trait object-safe
pub trait DomainWorkflowExtension: Send + Sync {
    /// Get the domain name this extension handles
    fn domain_name(&self) -> &str;

    /// Get extension version
    fn version(&self) -> &str;

    /// Execute a workflow instance
    fn execute_workflow(
        &self,
        context: &WorkflowContext,
    ) -> Pin<Box<dyn Future<Output = Result<WorkflowExecutionResult, String>> + Send + '_>>;

    /// Execute a single step
    fn execute_step(
        &self,
        step_id: &UniversalStepId,
        context: &WorkflowContext,
    ) -> Pin<Box<dyn Future<Output = Result<StepExecutionResult, String>> + Send + '_>>;

    /// Validate workflow definition
    fn validate_workflow(
        &self,
        workflow_id: &UniversalWorkflowId,
    ) -> Pin<Box<dyn Future<Output = Result<ValidationResult, String>> + Send + '_>>;

    /// Get supported step types for this domain
    fn supported_step_types(&self) -> Vec<String>;

    /// Get extension configuration schema
    fn configuration_schema(&self) -> Value;
}

/// Base extension implementation that domains can inherit from
pub struct BaseWorkflowExtension {
    domain_name: String,
    version: String,
    supported_step_types: Vec<String>,
}

impl BaseWorkflowExtension {
    pub fn new(domain_name: String, version: String) -> Self {
        Self {
            domain_name,
            version,
            supported_step_types: vec!["manual".to_string(), "automated".to_string()],
        }
    }
}

impl DomainWorkflowExtension for BaseWorkflowExtension {
    fn domain_name(&self) -> &str {
        &self.domain_name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn execute_workflow(
        &self,
        _context: &WorkflowContext,
    ) -> Pin<Box<dyn Future<Output = Result<WorkflowExecutionResult, String>> + Send + '_>> {
        Box::pin(async move {
            Err("Not implemented - override in domain extension".to_string())
        })
    }

    fn execute_step(
        &self,
        _step_id: &UniversalStepId,
        _context: &WorkflowContext,
    ) -> Pin<Box<dyn Future<Output = Result<StepExecutionResult, String>> + Send + '_>> {
        Box::pin(async move {
            Err("Not implemented - override in domain extension".to_string())
        })
    }

    fn validate_workflow(
        &self,
        _workflow_id: &UniversalWorkflowId,
    ) -> Pin<Box<dyn Future<Output = Result<ValidationResult, String>> + Send + '_>> {
        Box::pin(async move {
            Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                metadata: None,
            })
        })
    }

    fn supported_step_types(&self) -> Vec<String> {
        self.supported_step_types.clone()
    }

    fn configuration_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "domain": {"type": "string"},
                "version": {"type": "string"}
            }
        })
    }
}