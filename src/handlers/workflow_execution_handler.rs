//! Workflow execution handler
//!
//! This handler manages workflow execution tracking and runtime operations.

use crate::{
    Workflow,
    value_objects::{
        WorkflowId, ExecutionContext, StepExecutionResult,
        ErrorType, ExecutionSummary
    },
};
use cim_domain::{DomainResult, DomainError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler for workflow execution operations
pub struct WorkflowExecutionHandler {
    /// Store for workflow instances
    workflows: HashMap<WorkflowId, Workflow>,
    /// Active execution contexts
    execution_contexts: Arc<RwLock<HashMap<uuid::Uuid, ExecutionContext>>>,
}

impl WorkflowExecutionHandler {
    /// Create a new workflow execution handler
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            execution_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start workflow execution with tracking
    pub async fn start_execution(
        &mut self,
        workflow_id: WorkflowId,
        initial_context: HashMap<String, serde_json::Value>,
    ) -> DomainResult<uuid::Uuid> {
        // Verify the workflow exists
        if !self.workflows.contains_key(&workflow_id) {
            return Err(DomainError::generic("Workflow not found"));
        }
        
        // Create execution context
        let mut exec_context = ExecutionContext::new(workflow_id);
        exec_context.start();
        
        // Store initial context as runtime data
        for (key, value) in initial_context {
            exec_context.set_runtime_data(key, value);
        }
        
        let execution_id = exec_context.execution_id;
        
        // Store execution context
        let mut contexts = self.execution_contexts.write().await;
        contexts.insert(execution_id, exec_context);
        
        Ok(execution_id)
    }
    
    /// Track step execution
    pub async fn track_step_execution(
        &self,
        execution_id: uuid::Uuid,
        step_id: crate::value_objects::StepId,
        input_data: Option<serde_json::Value>,
    ) -> DomainResult<()> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        exec_context.start_step(step_id, input_data);
        Ok(())
    }
    
    /// Complete step execution
    pub async fn complete_step_execution(
        &self,
        execution_id: uuid::Uuid,
        step_id: crate::value_objects::StepId,
        success: bool,
        output_data: Option<serde_json::Value>,
        error_message: Option<String>,
    ) -> DomainResult<()> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        let result = if success {
            StepExecutionResult::Success
        } else {
            StepExecutionResult::Failed { 
                error: error_message.unwrap_or_else(|| "Unknown error".to_string())
            }
        };
        
        exec_context.complete_step(step_id, result, output_data);
        Ok(())
    }
    
    /// Pause workflow execution
    pub async fn pause_execution(
        &self,
        execution_id: uuid::Uuid,
    ) -> DomainResult<()> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        exec_context.pause();
        Ok(())
    }
    
    /// Resume workflow execution
    pub async fn resume_execution(
        &self,
        execution_id: uuid::Uuid,
    ) -> DomainResult<()> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        exec_context.resume();
        Ok(())
    }
    
    /// Complete workflow execution
    pub async fn complete_execution(
        &self,
        execution_id: uuid::Uuid,
    ) -> DomainResult<ExecutionSummary> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        exec_context.complete();
        Ok(exec_context.summary())
    }
    
    /// Fail workflow execution
    pub async fn fail_execution(
        &self,
        execution_id: uuid::Uuid,
        reason: String,
    ) -> DomainResult<ExecutionSummary> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        exec_context.fail(reason);
        Ok(exec_context.summary())
    }
    
    /// Cancel workflow execution
    pub async fn cancel_execution(
        &self,
        execution_id: uuid::Uuid,
    ) -> DomainResult<ExecutionSummary> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        exec_context.cancel();
        Ok(exec_context.summary())
    }
    
    /// Record execution error
    pub async fn record_execution_error(
        &self,
        execution_id: uuid::Uuid,
        error_type: ErrorType,
        message: String,
        step_id: Option<crate::value_objects::StepId>,
        details: Option<String>,
    ) -> DomainResult<()> {
        let mut contexts = self.execution_contexts.write().await;
        let exec_context = contexts
            .get_mut(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        exec_context.record_error(error_type, message, step_id, details);
        Ok(())
    }
    
    /// Get execution context
    pub async fn get_execution_context(
        &self,
        execution_id: uuid::Uuid,
    ) -> DomainResult<ExecutionContext> {
        let contexts = self.execution_contexts.read().await;
        let exec_context = contexts
            .get(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        Ok(exec_context.clone())
    }
    
    /// Get execution summary
    pub async fn get_execution_summary(
        &self,
        execution_id: uuid::Uuid,
    ) -> DomainResult<ExecutionSummary> {
        let contexts = self.execution_contexts.read().await;
        let exec_context = contexts
            .get(&execution_id)
            .ok_or_else(|| DomainError::generic("Execution context not found"))?;
        
        Ok(exec_context.summary())
    }
    
    /// List active executions for a workflow
    pub async fn list_active_executions(
        &self,
        workflow_id: WorkflowId,
    ) -> DomainResult<Vec<ExecutionSummary>> {
        let contexts = self.execution_contexts.read().await;
        let summaries: Vec<ExecutionSummary> = contexts
            .values()
            .filter(|ctx| ctx.workflow_id == workflow_id && !ctx.is_terminal())
            .map(|ctx| ctx.summary())
            .collect();
        
        Ok(summaries)
    }
    
    /// Cleanup completed executions (for memory management)
    pub async fn cleanup_completed_executions(&self, older_than_hours: u64) -> DomainResult<u32> {
        let mut contexts = self.execution_contexts.write().await;
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(older_than_hours as i64);
        
        let mut removed_count = 0;
        contexts.retain(|_, ctx| {
            if ctx.is_terminal() && ctx.completed_at.map(|t| t < cutoff_time).unwrap_or(false) {
                removed_count += 1;
                false
            } else {
                true
            }
        });
        
        Ok(removed_count)
    }
    
    /// Store a workflow (for testing)
    pub fn store_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.id, workflow);
    }
}

/// Commands for workflow execution operations
#[derive(Debug, Clone)]
pub struct StartWorkflowExecution {
    pub workflow_id: WorkflowId,
    pub initial_context: HashMap<String, serde_json::Value>,
    pub started_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TrackStepExecution {
    pub execution_id: uuid::Uuid,
    pub step_id: crate::value_objects::StepId,
    pub input_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct CompleteStepExecution {
    pub execution_id: uuid::Uuid,
    pub step_id: crate::value_objects::StepId,
    pub success: bool,
    pub output_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PauseExecution {
    pub execution_id: uuid::Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResumeExecution {
    pub execution_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct CancelExecution {
    pub execution_id: uuid::Uuid,
    pub reason: String,
}

impl Default for WorkflowExecutionHandler {
    fn default() -> Self {
        Self::new()
    }
}