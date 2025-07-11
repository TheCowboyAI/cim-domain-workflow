//! Workflow execution context tracking
//!
//! This module provides detailed tracking of workflow execution state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::value_objects::{StepId, WorkflowId};

/// Detailed execution context for workflow tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Workflow being executed
    pub workflow_id: WorkflowId,
    
    /// Execution ID for this run
    pub execution_id: uuid::Uuid,
    
    /// Current execution phase
    pub phase: ExecutionPhase,
    
    /// Execution start time
    pub started_at: DateTime<Utc>,
    
    /// Execution end time (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Current step being executed
    pub current_step: Option<StepId>,
    
    /// Steps execution history
    pub step_history: Vec<StepExecution>,
    
    /// Execution metrics
    pub metrics: ExecutionMetrics,
    
    /// Error tracking
    pub errors: Vec<ExecutionError>,
    
    /// Runtime variables specific to this execution
    pub runtime_data: HashMap<String, serde_json::Value>,
}

/// Execution phases
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionPhase {
    /// Preparing for execution
    Initializing,
    /// Executing workflow steps
    Running,
    /// Execution is paused
    Paused,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
}

/// Step execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    /// Step that was executed
    pub step_id: StepId,
    
    /// When the step started
    pub started_at: DateTime<Utc>,
    
    /// When the step completed
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Step execution result
    pub result: StepExecutionResult,
    
    /// Input data for the step
    pub input_data: Option<serde_json::Value>,
    
    /// Output data from the step
    pub output_data: Option<serde_json::Value>,
    
    /// Execution duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Step execution results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StepExecutionResult {
    /// Step completed successfully
    Success,
    /// Step failed with error
    Failed { error: String },
    /// Step was skipped
    Skipped { reason: String },
    /// Step is still running
    Running,
}

/// Execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total steps executed
    pub steps_executed: u32,
    
    /// Successful steps
    pub steps_succeeded: u32,
    
    /// Failed steps
    pub steps_failed: u32,
    
    /// Skipped steps
    pub steps_skipped: u32,
    
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    
    /// Average step duration in milliseconds
    pub avg_step_duration_ms: u64,
    
    /// Number of retries performed
    pub retry_count: u32,
}

/// Execution error record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    /// When the error occurred
    pub occurred_at: DateTime<Utc>,
    
    /// Step where error occurred (if applicable)
    pub step_id: Option<StepId>,
    
    /// Error type
    pub error_type: ErrorType,
    
    /// Error message
    pub message: String,
    
    /// Stack trace or additional details
    pub details: Option<String>,
}

/// Types of execution errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Step execution failed
    StepFailure,
    /// Timeout exceeded
    Timeout,
    /// Resource not available
    ResourceUnavailable,
    /// Invalid input data
    InvalidInput,
    /// System error
    SystemError,
    /// Custom error
    Custom(String),
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(workflow_id: WorkflowId) -> Self {
        Self {
            workflow_id,
            execution_id: uuid::Uuid::new_v4(),
            phase: ExecutionPhase::Initializing,
            started_at: Utc::now(),
            completed_at: None,
            current_step: None,
            step_history: Vec::new(),
            metrics: ExecutionMetrics::default(),
            errors: Vec::new(),
            runtime_data: HashMap::new(),
        }
    }
    
    /// Start execution
    pub fn start(&mut self) {
        self.phase = ExecutionPhase::Running;
        self.started_at = Utc::now();
    }
    
    /// Start executing a step
    pub fn start_step(&mut self, step_id: StepId, input_data: Option<serde_json::Value>) {
        self.current_step = Some(step_id);
        
        let step_execution = StepExecution {
            step_id,
            started_at: Utc::now(),
            completed_at: None,
            result: StepExecutionResult::Running,
            input_data,
            output_data: None,
            duration_ms: None,
        };
        
        self.step_history.push(step_execution);
    }
    
    /// Complete a step execution
    pub fn complete_step(
        &mut self, 
        step_id: StepId, 
        result: StepExecutionResult,
        output_data: Option<serde_json::Value>
    ) {
        let now = Utc::now();
        
        // Update step history
        if let Some(step_exec) = self.step_history.iter_mut()
            .rev()
            .find(|s| s.step_id == step_id && s.result == StepExecutionResult::Running) 
        {
            step_exec.completed_at = Some(now);
            step_exec.result = result.clone();
            step_exec.output_data = output_data;
            
            let duration_ms = (now - step_exec.started_at).num_milliseconds() as u64;
            step_exec.duration_ms = Some(duration_ms);
            
            // Update metrics
            self.metrics.steps_executed += 1;
            match &result {
                StepExecutionResult::Success => self.metrics.steps_succeeded += 1,
                StepExecutionResult::Failed { .. } => self.metrics.steps_failed += 1,
                StepExecutionResult::Skipped { .. } => self.metrics.steps_skipped += 1,
                StepExecutionResult::Running => {} // Should not happen
            }
            
            // Update average duration
            self.update_average_duration();
        }
        
        // Clear current step if it matches
        if self.current_step == Some(step_id) {
            self.current_step = None;
        }
    }
    
    /// Record an error
    pub fn record_error(
        &mut self,
        error_type: ErrorType,
        message: String,
        step_id: Option<StepId>,
        details: Option<String>
    ) {
        self.errors.push(ExecutionError {
            occurred_at: Utc::now(),
            step_id,
            error_type,
            message,
            details,
        });
    }
    
    /// Pause execution
    pub fn pause(&mut self) {
        self.phase = ExecutionPhase::Paused;
    }
    
    /// Resume execution
    pub fn resume(&mut self) {
        if self.phase == ExecutionPhase::Paused {
            self.phase = ExecutionPhase::Running;
        }
    }
    
    /// Complete execution
    pub fn complete(&mut self) {
        self.phase = ExecutionPhase::Completed;
        self.completed_at = Some(Utc::now());
        self.update_total_duration();
    }
    
    /// Fail execution
    pub fn fail(&mut self, reason: String) {
        self.phase = ExecutionPhase::Failed;
        self.completed_at = Some(Utc::now());
        self.record_error(
            ErrorType::SystemError,
            reason,
            self.current_step,
            None
        );
        self.update_total_duration();
    }
    
    /// Cancel execution
    pub fn cancel(&mut self) {
        self.phase = ExecutionPhase::Cancelled;
        self.completed_at = Some(Utc::now());
        self.update_total_duration();
    }
    
    /// Set runtime data
    pub fn set_runtime_data(&mut self, key: String, value: serde_json::Value) {
        self.runtime_data.insert(key, value);
    }
    
    /// Get runtime data
    pub fn get_runtime_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.runtime_data.get(key)
    }
    
    /// Update average step duration
    fn update_average_duration(&mut self) {
        let completed_steps: Vec<&StepExecution> = self.step_history.iter()
            .filter(|s| s.duration_ms.is_some())
            .collect();
        
        if !completed_steps.is_empty() {
            let total_duration: u64 = completed_steps.iter()
                .filter_map(|s| s.duration_ms)
                .sum();
            self.metrics.avg_step_duration_ms = total_duration / completed_steps.len() as u64;
        }
    }
    
    /// Update total execution duration
    fn update_total_duration(&mut self) {
        if let Some(completed_at) = self.completed_at {
            self.metrics.total_duration_ms = 
                (completed_at - self.started_at).num_milliseconds() as u64;
        }
    }
    
    /// Check if execution is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.phase,
            ExecutionPhase::Completed | ExecutionPhase::Failed | ExecutionPhase::Cancelled
        )
    }
    
    /// Get execution summary
    pub fn summary(&self) -> ExecutionSummary {
        ExecutionSummary {
            execution_id: self.execution_id,
            workflow_id: self.workflow_id,
            phase: self.phase.clone(),
            started_at: self.started_at,
            completed_at: self.completed_at,
            duration_ms: self.metrics.total_duration_ms,
            steps_total: self.step_history.len() as u32,
            steps_succeeded: self.metrics.steps_succeeded,
            steps_failed: self.metrics.steps_failed,
            steps_skipped: self.metrics.steps_skipped,
            error_count: self.errors.len() as u32,
        }
    }
}

/// Execution summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub execution_id: uuid::Uuid,
    pub workflow_id: WorkflowId,
    pub phase: ExecutionPhase,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: u64,
    pub steps_total: u32,
    pub steps_succeeded: u32,
    pub steps_failed: u32,
    pub steps_skipped: u32,
    pub error_count: u32,
}