//! Workflow command handler

use crate::{
    aggregate::Workflow,
    commands::*,
    value_objects::{WorkflowId, StepId},
};
use cim_domain::{DomainResult, DomainError, CommandEnvelope, CommandAcknowledgment, CommandStatus};
use std::collections::HashMap;

/// Handler for workflow commands
/// 
/// Processes workflow commands through proper DDD patterns:
/// Command -> Validate -> Load Aggregate -> Apply Business Logic -> Generate Events -> Store Events -> Return Acknowledgment
pub struct WorkflowCommandHandler {
    /// In-memory store for testing/demo - replace with actual event store
    workflows: HashMap<WorkflowId, Workflow>,
}

impl WorkflowCommandHandler {
    /// Create a new command handler
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }

    /// Handle create workflow command
    pub fn handle_create_workflow(
        &mut self,
        envelope: CommandEnvelope<CreateWorkflow>,
    ) -> DomainResult<CommandAcknowledgment> {
        let cmd = envelope.command;

        // Validate command
        if cmd.name.trim().is_empty() {
            return Err(DomainError::generic("Workflow name cannot be empty"));
        }

        // Create new workflow aggregate
        let (workflow, _events) = Workflow::new(
            cmd.name,
            cmd.description,
            cmd.metadata,
            cmd.created_by,
        )?;

        // Store workflow
        let workflow_id = workflow.id;
        self.workflows.insert(workflow_id, workflow);

        Ok(CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        })
    }

    /// Handle start workflow command
    pub fn handle_start_workflow(
        &mut self,
        envelope: CommandEnvelope<StartWorkflow>,
    ) -> DomainResult<CommandAcknowledgment> {
        let cmd = envelope.command;

        // Load workflow
        let workflow = self.workflows
            .get_mut(&cmd.workflow_id)
            .ok_or_else(|| DomainError::generic("Workflow not found"))?;

        // Start workflow
        let _events = workflow.start(cmd.context, cmd.started_by)?;

        Ok(CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        })
    }

    /// Handle add step command
    pub fn handle_add_step(
        &mut self,
        envelope: CommandEnvelope<AddStep>,
    ) -> DomainResult<CommandAcknowledgment> {
        let cmd = envelope.command;

        // Load workflow
        let workflow = self.workflows
            .get_mut(&cmd.workflow_id)
            .ok_or_else(|| DomainError::generic("Workflow not found"))?;

        // Add step
        let _events = workflow.add_step(
            cmd.name,
            cmd.description,
            cmd.step_type,
            cmd.config,
            cmd.dependencies,
            cmd.estimated_duration_minutes,
            cmd.assigned_to,
            cmd.added_by,
        )?;

        Ok(CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        })
    }

    /// Get workflow for testing/debugging
    pub fn get_workflow(&self, workflow_id: &WorkflowId) -> Option<&Workflow> {
        self.workflows.get(workflow_id)
    }
}

impl Default for WorkflowCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

