//! Core workflow engine abstraction
//! 
//! This module defines the abstract workflow engine that all domain implementations
//! will use, providing a unified interface for workflow execution across all CIM domains.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::primitives::{
    WorkflowContext, 
    UniversalWorkflowId, 
    UniversalStepId, 
    WorkflowInstanceId,
    ContextError,
};
use crate::composition::extensions::DomainWorkflowExtension;

/// Abstract workflow engine trait that all implementations must follow
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Execute a workflow instance with the given context
    async fn execute_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        context: WorkflowContext,
    ) -> Result<WorkflowExecutionResult, WorkflowEngineError>;

    /// Execute a single step within a workflow
    async fn execute_step(
        &self,
        step_id: UniversalStepId,
        context: WorkflowContext,
    ) -> Result<StepExecutionResult, WorkflowEngineError>;

    /// Pause workflow execution at current step
    async fn pause_workflow(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> Result<(), WorkflowEngineError>;

    /// Resume paused workflow execution
    async fn resume_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        context: Option<WorkflowContext>,
    ) -> Result<WorkflowExecutionResult, WorkflowEngineError>;

    /// Cancel workflow execution
    async fn cancel_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        reason: String,
    ) -> Result<(), WorkflowEngineError>;

    /// Get current workflow status
    async fn get_workflow_status(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> Result<WorkflowStatus, WorkflowEngineError>;

    /// Get execution history for workflow
    async fn get_execution_history(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> Result<Vec<ExecutionHistoryEntry>, WorkflowEngineError>;

    /// Register a domain extension with the engine
    fn register_extension(
        &mut self,
        domain: String,
        extension: Box<dyn DomainWorkflowExtension>,
    ) -> Result<(), WorkflowEngineError>;

    /// Get registered extensions
    fn get_extensions(&self) -> &HashMap<String, Box<dyn DomainWorkflowExtension>>;

    /// Validate workflow definition
    async fn validate_workflow(
        &self,
        workflow_id: UniversalWorkflowId,
    ) -> Result<ValidationResult, WorkflowEngineError>;
}

/// Concrete implementation of the workflow engine
pub struct UnifiedWorkflowEngine {
    /// Registered domain extensions
    extensions: HashMap<String, Box<dyn DomainWorkflowExtension>>,
    /// Active workflow instances
    active_instances: HashMap<WorkflowInstanceId, WorkflowExecutionState>,
    /// Engine configuration
    config: EngineConfiguration,
    /// Execution statistics
    stats: ExecutionStatistics,
}

impl UnifiedWorkflowEngine {
    /// Create a new unified workflow engine
    pub fn new(config: EngineConfiguration) -> Self {
        Self {
            extensions: HashMap::new(),
            active_instances: HashMap::new(),
            config,
            stats: ExecutionStatistics::default(),
        }
    }

    /// Create engine with default configuration
    pub fn default() -> Self {
        Self::new(EngineConfiguration::default())
    }

    /// Get execution statistics
    pub fn stats(&self) -> &ExecutionStatistics {
        &self.stats
    }

    /// Get engine configuration
    pub fn config(&self) -> &EngineConfiguration {
        &self.config
    }

    /// Update engine configuration
    pub fn update_config(&mut self, config: EngineConfiguration) {
        self.config = config;
    }

    /// Get domain extension by name
    pub fn get_extension(&self, domain: &str) -> Option<&Box<dyn DomainWorkflowExtension>> {
        self.extensions.get(domain)
    }

    /// Check if domain is supported
    pub fn supports_domain(&self, domain: &str) -> bool {
        self.extensions.contains_key(domain)
    }

    /// Get list of supported domains
    pub fn supported_domains(&self) -> Vec<String> {
        self.extensions.keys().cloned().collect()
    }
}

#[async_trait]
impl WorkflowEngine for UnifiedWorkflowEngine {
    async fn execute_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        mut context: WorkflowContext,
    ) -> Result<WorkflowExecutionResult, WorkflowEngineError> {
        // Record execution start
        context.record_event(
            "workflow_execution_started".to_string(),
            serde_json::json!({
                "instance_id": instance_id.to_string(),
                "engine": "unified"
            })
        );

        // Validate context
        context.validate()
            .map_err(WorkflowEngineError::ContextValidation)?;

        // Get domain extension for workflow
        let origin_domain = context.workflow_id.origin_domain();
        let extension = self.extensions.get(origin_domain)
            .ok_or_else(|| WorkflowEngineError::UnsupportedDomain(origin_domain.to_string()))?;

        // Execute workflow using domain extension
        let execution_start = chrono::Utc::now();
        let result = extension.execute_workflow(&context).await
            .map_err(WorkflowEngineError::ExecutionError)?;

        // Update statistics
        let duration = chrono::Utc::now() - execution_start;
        // Note: Would need mutable self for this in a real implementation
        // self.stats.record_execution(duration, result.status == WorkflowExecutionStatus::Completed);

        // Record execution completion
        context.record_event(
            "workflow_execution_completed".to_string(),
            serde_json::json!({
                "instance_id": instance_id.to_string(),
                "status": result.status,
                "duration_ms": duration.num_milliseconds()
            })
        );

        Ok(result)
    }

    async fn execute_step(
        &self,
        step_id: UniversalStepId,
        mut context: WorkflowContext,
    ) -> Result<StepExecutionResult, WorkflowEngineError> {
        // Record step execution start
        context.record_event(
            "step_execution_started".to_string(),
            serde_json::json!({
                "step_id": step_id.to_string(),
                "executor_domain": step_id.executor_domain()
            })
        );

        // Get domain extension for step execution
        let executor_domain = step_id.executor_domain();
        let extension = self.extensions.get(executor_domain)
            .ok_or_else(|| WorkflowEngineError::UnsupportedDomain(executor_domain.to_string()))?;

        // Execute step using domain extension
        let result = extension.execute_step(&step_id, &context).await
            .map_err(WorkflowEngineError::ExecutionError)?;

        // Record step execution completion
        context.record_event(
            "step_execution_completed".to_string(),
            serde_json::json!({
                "step_id": step_id.to_string(),
                "status": result.status,
                "cross_domain": step_id.is_cross_domain()
            })
        );

        Ok(result)
    }

    async fn pause_workflow(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> Result<(), WorkflowEngineError> {
        // Implementation would pause the workflow instance
        // For now, just validate the instance exists
        if !self.active_instances.contains_key(&instance_id) {
            return Err(WorkflowEngineError::WorkflowNotFound(instance_id.to_string()));
        }
        Ok(())
    }

    async fn resume_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        context: Option<WorkflowContext>,
    ) -> Result<WorkflowExecutionResult, WorkflowEngineError> {
        // Implementation would resume the paused workflow
        // For now, return a placeholder result
        Ok(WorkflowExecutionResult {
            instance_id,
            status: WorkflowExecutionStatus::Running,
            completed_steps: Vec::new(),
            context: context.unwrap_or_else(|| {
                let workflow_id = UniversalWorkflowId::new("placeholder".to_string(), None);
                let instance_id_clone = WorkflowInstanceId::new(workflow_id.clone());
                WorkflowContext::new(workflow_id, instance_id_clone, None)
            }),
            error: None,
        })
    }

    async fn cancel_workflow(
        &self,
        instance_id: WorkflowInstanceId,
        _reason: String,
    ) -> Result<(), WorkflowEngineError> {
        // Implementation would cancel the workflow instance
        if !self.active_instances.contains_key(&instance_id) {
            return Err(WorkflowEngineError::WorkflowNotFound(instance_id.to_string()));
        }
        Ok(())
    }

    async fn get_workflow_status(
        &self,
        instance_id: WorkflowInstanceId,
    ) -> Result<WorkflowStatus, WorkflowEngineError> {
        // Implementation would return actual status
        Ok(WorkflowStatus {
            instance_id,
            current_status: WorkflowExecutionStatus::Running,
            current_step: None,
            progress: WorkflowProgress {
                total_steps: 5,
                completed_steps: 2,
                percentage: 40.0,
            },
            started_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn get_execution_history(
        &self,
        _instance_id: WorkflowInstanceId,
    ) -> Result<Vec<ExecutionHistoryEntry>, WorkflowEngineError> {
        // Implementation would return actual execution history
        Ok(Vec::new())
    }

    fn register_extension(
        &mut self,
        domain: String,
        extension: Box<dyn DomainWorkflowExtension>,
    ) -> Result<(), WorkflowEngineError> {
        // Validate extension
        if domain.is_empty() {
            return Err(WorkflowEngineError::InvalidExtension(
                "Domain name cannot be empty".to_string()
            ));
        }

        // Register extension
        self.extensions.insert(domain, extension);
        Ok(())
    }

    fn get_extensions(&self) -> &HashMap<String, Box<dyn DomainWorkflowExtension>> {
        &self.extensions
    }

    async fn validate_workflow(
        &self,
        workflow_id: UniversalWorkflowId,
    ) -> Result<ValidationResult, WorkflowEngineError> {
        let domain = workflow_id.origin_domain();
        let extension = self.extensions.get(domain)
            .ok_or_else(|| WorkflowEngineError::UnsupportedDomain(domain.to_string()))?;

        extension.validate_workflow(&workflow_id).await
            .map_err(WorkflowEngineError::ValidationError)
    }
}

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    /// Workflow instance that was executed
    pub instance_id: WorkflowInstanceId,
    /// Final execution status
    pub status: WorkflowExecutionStatus,
    /// Steps that were completed
    pub completed_steps: Vec<UniversalStepId>,
    /// Final context state
    pub context: WorkflowContext,
    /// Error if execution failed
    pub error: Option<String>,
}

/// Result of step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    /// Step that was executed
    pub step_id: UniversalStepId,
    /// Step execution status
    pub status: StepExecutionStatus,
    /// Updated context after step execution
    pub context: WorkflowContext,
    /// Output data from step execution
    pub output: Option<serde_json::Value>,
    /// Error if step failed
    pub error: Option<String>,
}

/// Workflow execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowExecutionStatus {
    /// Workflow is currently running
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with error
    Failed,
    /// Workflow was paused
    Paused,
    /// Workflow was cancelled
    Cancelled,
    /// Workflow is waiting for external event
    Waiting,
}

/// Step execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepExecutionStatus {
    /// Step completed successfully
    Completed,
    /// Step failed with error
    Failed,
    /// Step is waiting for manual input
    WaitingForInput,
    /// Step is waiting for external system
    WaitingForExternal,
    /// Step was skipped due to conditions
    Skipped,
}

/// Current workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    /// Workflow instance
    pub instance_id: WorkflowInstanceId,
    /// Current execution status
    pub current_status: WorkflowExecutionStatus,
    /// Current step being executed
    pub current_step: Option<UniversalStepId>,
    /// Execution progress
    pub progress: WorkflowProgress,
    /// When workflow started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Last status update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow execution progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    /// Total number of steps in workflow
    pub total_steps: u32,
    /// Number of completed steps
    pub completed_steps: u32,
    /// Completion percentage
    pub percentage: f64,
}

/// Execution history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistoryEntry {
    /// When this event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Type of event
    pub event_type: String,
    /// Step involved (if applicable)
    pub step_id: Option<UniversalStepId>,
    /// Event details
    pub details: serde_json::Value,
}

/// Workflow execution state (internal to engine)
#[derive(Debug)]
struct WorkflowExecutionState {
    /// Current context
    context: WorkflowContext,
    /// Current status
    status: WorkflowExecutionStatus,
    /// Execution start time
    started_at: chrono::DateTime<chrono::Utc>,
    /// Last update time
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfiguration {
    /// Maximum concurrent workflow executions
    pub max_concurrent_workflows: usize,
    /// Step execution timeout in seconds
    pub step_timeout_seconds: u64,
    /// Workflow execution timeout in seconds
    pub workflow_timeout_seconds: u64,
    /// Enable detailed execution logging
    pub enable_detailed_logging: bool,
    /// Retry configuration
    pub retry_config: RetryConfiguration,
}

impl Default for EngineConfiguration {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 100,
            step_timeout_seconds: 300, // 5 minutes
            workflow_timeout_seconds: 3600, // 1 hour
            enable_detailed_logging: true,
            retry_config: RetryConfiguration::default(),
        }
    }
}

/// Retry configuration for failed steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfiguration {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfiguration {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000, // 1 second
            max_delay_ms: 30000, // 30 seconds
            backoff_multiplier: 2.0,
        }
    }
}

/// Execution statistics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    /// Total workflows executed
    pub total_workflows: u64,
    /// Total successful workflows
    pub successful_workflows: u64,
    /// Total failed workflows
    pub failed_workflows: u64,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Domain-specific statistics
    pub domain_stats: HashMap<String, DomainStatistics>,
}

/// Statistics for a specific domain
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DomainStatistics {
    /// Workflows executed for this domain
    pub workflow_count: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average execution time for this domain
    pub average_execution_time_ms: f64,
}

/// Validation result for workflow definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validated workflow metadata
    pub metadata: Option<serde_json::Value>,
}

/// Workflow engine errors
#[derive(Debug, thiserror::Error)]
pub enum WorkflowEngineError {
    #[error("Unsupported domain: {0}")]
    UnsupportedDomain(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Context validation error: {0}")]
    ContextValidation(#[from] ContextError),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid extension: {0}")]
    InvalidExtension(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{UniversalWorkflowId, WorkflowInstanceId};

    #[test]
    fn test_engine_creation() {
        let engine = UnifiedWorkflowEngine::default();
        assert_eq!(engine.supported_domains().len(), 0);
        assert_eq!(engine.config().max_concurrent_workflows, 100);
    }

    #[test]
    fn test_workflow_execution_result() {
        let workflow_id = UniversalWorkflowId::new("test".to_string(), None);
        let instance_id = WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id.clone(), None);

        let result = WorkflowExecutionResult {
            instance_id,
            status: WorkflowExecutionStatus::Completed,
            completed_steps: Vec::new(),
            context,
            error: None,
        };

        assert_eq!(result.status, WorkflowExecutionStatus::Completed);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_engine_configuration() {
        let config = EngineConfiguration::default();
        assert_eq!(config.max_concurrent_workflows, 100);
        assert_eq!(config.step_timeout_seconds, 300);
        assert!(config.enable_detailed_logging);
    }

    #[tokio::test]
    async fn test_validation_result() {
        let workflow_id = UniversalWorkflowId::new("test".to_string(), None);
        let result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            metadata: None,
        };

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
}