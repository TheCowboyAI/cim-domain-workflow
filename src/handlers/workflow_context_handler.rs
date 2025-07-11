//! Workflow context handler
//!
//! This handler manages workflow context data and transition conditions.

use crate::{
    Workflow,
    value_objects::{WorkflowId, WorkflowContext},
    domain_events::WorkflowDomainEvent,
    state_machine::WorkflowTransition,
};
use cim_domain::{DomainResult, DomainError};
use std::collections::HashMap;
use serde_json::Value;

/// Handler for workflow context operations
pub struct WorkflowContextHandler {
    /// Store for workflow instances (would be event store in production)
    workflows: HashMap<WorkflowId, Workflow>,
}

impl WorkflowContextHandler {
    /// Create a new workflow context handler
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }
    
    /// Update workflow context data
    pub fn update_workflow_context(
        &mut self, 
        workflow_id: WorkflowId,
        updates: HashMap<String, Value>
    ) -> DomainResult<Vec<WorkflowDomainEvent>> {
        let workflow = self.workflows
            .get_mut(&workflow_id)
            .ok_or_else(|| DomainError::generic("Workflow not found"))?;
        
        // Update context variables and generate events
        let mut events = Vec::new();
        
        for (key, value) in updates {
            workflow.context.set_variable(key.clone(), value.clone());
            
            // Generate event for each variable set
            let event = WorkflowDomainEvent::WorkflowContextVariableSet(
                crate::events::WorkflowContextVariableSet {
                    workflow_id,
                    key,
                    value,
                    set_by: None,
                    set_at: chrono::Utc::now(),
                }
            );
            events.push(event);
        }
        
        Ok(events)
    }
    
    /// Get workflow context data
    pub fn get_workflow_context(&self, workflow_id: &WorkflowId) -> DomainResult<WorkflowContext> {
        let workflow = self.workflows
            .get(workflow_id)
            .ok_or_else(|| DomainError::generic("Workflow not found"))?;
        
        Ok(workflow.context.clone())
    }
    
    /// Evaluate transition conditions for a workflow
    pub fn evaluate_transition_conditions(
        &self,
        workflow_id: &WorkflowId,
        transition: &WorkflowTransition,
    ) -> DomainResult<bool> {
        let workflow = self.workflows
            .get(workflow_id)
            .ok_or_else(|| DomainError::generic("Workflow not found"))?;
        
        // Create transition guards based on the workflow's current state and transition type
        let guards = self.create_transition_guards(workflow, transition);
        
        // Evaluate all guards
        for guard in guards {
            guard(&workflow.context)?;
        }
        
        Ok(true)
    }
    
    /// Create transition guards based on workflow state
    fn create_transition_guards(
        &self,
        workflow: &Workflow,
        transition: &WorkflowTransition,
    ) -> Vec<Box<dyn Fn(&WorkflowContext) -> DomainResult<()> + Send + Sync>> {
        let mut guards: Vec<Box<dyn Fn(&WorkflowContext) -> DomainResult<()> + Send + Sync>> = Vec::new();
        
        match transition {
            WorkflowTransition::Start => {
                // Guard: Workflow must have required context data
                guards.push(Box::new(|ctx| {
                    if ctx.variables.is_empty() {
                        return Err(DomainError::generic("Workflow context cannot be empty"));
                    }
                    Ok(())
                }));
                
                // Guard: Check for required variables based on workflow metadata
                if let Some(required_vars) = workflow.metadata.get("required_context_variables") {
                    if let Some(vars) = required_vars.as_array() {
                        let required: Vec<String> = vars.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        
                        guards.push(Box::new(move |ctx| {
                            for var in &required {
                                if !ctx.has_variable(var) {
                                    return Err(DomainError::generic(
                                        format!("Required context variable '{}' is missing", var)
                                    ));
                                }
                            }
                            Ok(())
                        }));
                    }
                }
            }
            
            WorkflowTransition::Complete => {
                // Guard: All required steps must be completed
                let incomplete_step_names: Vec<String> = workflow.steps.values()
                    .filter(|step| !step.is_completed())
                    .filter(|step| {
                        // Check if step is required (not optional)
                        !step.config.get("optional")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    })
                    .map(|step| step.name.clone())
                    .collect();
                
                guards.push(Box::new(move |_ctx| {
                    if !incomplete_step_names.is_empty() {
                        return Err(DomainError::generic(
                            format!("Cannot complete workflow: {} steps are not completed", 
                                incomplete_step_names.join(", "))
                        ));
                    }
                    Ok(())
                }));
                
                // Guard: Check for required output data
                if let Some(required_outputs) = workflow.metadata.get("required_outputs") {
                    if let Some(outputs) = required_outputs.as_array() {
                        let required: Vec<String> = outputs.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        
                        guards.push(Box::new(move |ctx| {
                            for output in &required {
                                if !ctx.has_variable(output) {
                                    return Err(DomainError::generic(
                                        format!("Required output '{}' is missing", output)
                                    ));
                                }
                            }
                            Ok(())
                        }));
                    }
                }
            }
            
            WorkflowTransition::Pause { reason: _ } => {
                // Guard: Workflow must be in a pausable state
                guards.push(Box::new(move |ctx| {
                    if ctx.get_bool("is_critical_operation").unwrap_or(false) {
                        return Err(DomainError::generic(
                            "Cannot pause workflow during critical operation"
                        ));
                    }
                    Ok(())
                }));
            }
            
            _ => {
                // Default guards for other transitions
            }
        }
        
        guards
    }
    
    /// Store a workflow (for testing)
    pub fn store_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.id, workflow);
    }
}

/// Commands for workflow context operations
#[derive(Debug, Clone)]
pub struct UpdateWorkflowContext {
    pub workflow_id: WorkflowId,
    pub updates: HashMap<String, Value>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EvaluateTransitionConditions {
    pub workflow_id: WorkflowId,
    pub transition: WorkflowTransition,
}

/// Response for transition evaluation
#[derive(Debug, Clone)]
pub struct TransitionEvaluation {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_context: Vec<String>,
}

impl WorkflowContextHandler {
    /// Handle update context command
    pub fn handle_update_context(
        &mut self,
        cmd: UpdateWorkflowContext
    ) -> DomainResult<Vec<WorkflowDomainEvent>> {
        // Validate the updates
        for (key, value) in &cmd.updates {
            if key.is_empty() {
                return Err(DomainError::generic("Context variable name cannot be empty"));
            }
            
            // Prevent overwriting system variables
            if key.starts_with("_system_") {
                return Err(DomainError::generic(
                    format!("Cannot modify system variable: {}", key)
                ));
            }
            
            // Validate value is not null (workflow context should not have null values)
            if value.is_null() {
                return Err(DomainError::generic(
                    format!("Cannot set context variable '{}' to null", key)
                ));
            }
        }
        
        self.update_workflow_context(cmd.workflow_id, cmd.updates)
    }
    
    /// Handle evaluate transition command
    pub fn handle_evaluate_transition(
        &self,
        cmd: EvaluateTransitionConditions
    ) -> DomainResult<TransitionEvaluation> {
        match self.evaluate_transition_conditions(&cmd.workflow_id, &cmd.transition) {
            Ok(allowed) => Ok(TransitionEvaluation {
                allowed,
                reason: if allowed { None } else { Some("Transition conditions not met".to_string()) },
                required_context: vec![],
            }),
            Err(e) => Ok(TransitionEvaluation {
                allowed: false,
                reason: Some(e.to_string()),
                required_context: self.extract_required_context(&e),
            })
        }
    }
    
    /// Extract required context variables from error message
    fn extract_required_context(&self, error: &DomainError) -> Vec<String> {
        // Simple extraction - in production this would be more sophisticated
        let error_msg = error.to_string();
        if error_msg.contains("Required context variable") {
            // Extract variable name from error message
            if let Some(start) = error_msg.find('\'') {
                if let Some(end) = error_msg[start+1..].find('\'') {
                    return vec![error_msg[start+1..start+1+end].to_string()];
                }
            }
        }
        vec![]
    }
}