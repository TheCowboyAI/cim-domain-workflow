//! Workflow command handlers

use crate::{
    Workflow,
    commands::*,
    domain_events::WorkflowDomainEvent,
    value_objects::WorkflowId,
};
use cim_domain::{DomainResult, DomainError};
use std::collections::HashMap;

/// Trait for handling workflow commands
pub trait WorkflowCommandHandler: Send + Sync {
    /// Handle create workflow command
    fn handle_create_workflow(&mut self, cmd: CreateWorkflow) -> DomainResult<Vec<WorkflowDomainEvent>>;
    
    /// Handle start workflow command
    fn handle_start_workflow(&mut self, cmd: StartWorkflow) -> DomainResult<Vec<WorkflowDomainEvent>>;
    
    /// Handle add step command
    fn handle_add_step(&mut self, cmd: AddStep) -> DomainResult<Vec<WorkflowDomainEvent>>;
}

/// Handler for workflow commands
/// 
/// Processes workflow commands through proper DDD patterns:
/// Command -> Validate -> Load Aggregate -> Apply Business Logic -> Generate Events -> Store Events -> Return Events
pub struct WorkflowCommandHandlerImpl {
    /// In-memory store for testing/demo - replace with actual event store
    workflows: HashMap<WorkflowId, Workflow>,
}

impl WorkflowCommandHandlerImpl {
    /// Create a new command handler
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }

    /// Get workflow for testing/debugging
    pub fn get_workflow(&self, workflow_id: &WorkflowId) -> Option<&Workflow> {
        self.workflows.get(workflow_id)
    }
}

impl WorkflowCommandHandler for WorkflowCommandHandlerImpl {
    fn handle_create_workflow(&mut self, cmd: CreateWorkflow) -> DomainResult<Vec<WorkflowDomainEvent>> {
        // Validate command
        if cmd.name.trim().is_empty() {
            return Err(DomainError::generic("Workflow name cannot be empty"));
        }

        // Create new workflow aggregate
        let (workflow, events) = Workflow::new(
            cmd.name,
            cmd.description,
            cmd.metadata,
            cmd.created_by,
        )?;

        // Store workflow
        let workflow_id = workflow.id;
        self.workflows.insert(workflow_id, workflow);

        Ok(events)
    }

    fn handle_start_workflow(&mut self, cmd: StartWorkflow) -> DomainResult<Vec<WorkflowDomainEvent>> {
        // Load workflow
        let workflow = self.workflows
            .get_mut(&cmd.workflow_id)
            .ok_or_else(|| DomainError::generic("Workflow not found"))?;

        // Start workflow
        let events = workflow.start(cmd.context, cmd.started_by)?;

        Ok(events)
    }

    fn handle_add_step(&mut self, cmd: AddStep) -> DomainResult<Vec<WorkflowDomainEvent>> {
        // Load workflow
        let workflow = self.workflows
            .get_mut(&cmd.workflow_id)
            .ok_or_else(|| DomainError::generic("Workflow not found"))?;

        // Add step
        let events = workflow.add_step(
            cmd.name,
            cmd.description,
            cmd.step_type,
            cmd.config,
            cmd.dependencies,
            cmd.estimated_duration_minutes,
            cmd.assigned_to,
            cmd.added_by,
        )?;

        Ok(events)
    }
}

impl Default for WorkflowCommandHandlerImpl {
    fn default() -> Self {
        Self::new()
    }
}

