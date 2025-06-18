//! Query handler for workflow operations

use crate::{
    queries::*,
    projections::WorkflowProjection,
    value_objects::{WorkflowId, WorkflowStatus},
};
use cim_domain::DomainResult;

/// Trait for handling workflow queries
pub trait WorkflowQueryHandler: Send + Sync {
    /// Find a workflow by ID
    fn find_workflow(&self, query: FindWorkflow) -> DomainResult<Option<WorkflowView>>;
    
    /// List workflows with optional filtering
    fn list_workflows(&self, query: ListWorkflows) -> DomainResult<Vec<WorkflowView>>;
    
    /// Get steps for a workflow
    fn get_workflow_steps(&self, query: GetWorkflowSteps) -> DomainResult<Vec<StepView>>;
    
    /// Get executable steps for a workflow
    fn get_executable_steps(&self, query: GetExecutableSteps) -> DomainResult<Vec<StepView>>;
}

/// Implementation of workflow query handler
pub struct WorkflowQueryHandlerImpl {
    projection: WorkflowProjection,
}

impl WorkflowQueryHandlerImpl {
    /// Create a new query handler with the given projection
    pub fn new(projection: WorkflowProjection) -> Self {
        Self { projection }
    }
}

impl WorkflowQueryHandler for WorkflowQueryHandlerImpl {
    fn find_workflow(&self, query: FindWorkflow) -> DomainResult<Option<WorkflowView>> {
        Ok(self.projection.find_by_id(query.workflow_id))
    }
    
    fn list_workflows(&self, query: ListWorkflows) -> DomainResult<Vec<WorkflowView>> {
        let workflows = if let Some(status) = query.status {
            self.projection.find_by_status(status)
        } else {
            self.projection.list_all()
        };
        
        // Apply pagination
        let start = query.offset.unwrap_or(0);
        let end = start + query.limit.unwrap_or(100);
        
        Ok(workflows.into_iter()
            .skip(start)
            .take(end - start)
            .collect())
    }
    
    fn get_workflow_steps(&self, query: GetWorkflowSteps) -> DomainResult<Vec<StepView>> {
        Ok(self.projection.get_steps(query.workflow_id))
    }
    
    fn get_executable_steps(&self, query: GetExecutableSteps) -> DomainResult<Vec<StepView>> {
        Ok(self.projection.get_executable_steps(query.workflow_id))
    }
} 