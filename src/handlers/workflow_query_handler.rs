//! Workflow query handler (placeholder)

use crate::value_objects::WorkflowId;
use cim_domain::DomainResult;

/// Handler for workflow queries
pub struct WorkflowQueryHandler {
    // Placeholder implementation
}

impl WorkflowQueryHandler {
    /// Create a new query handler
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WorkflowQueryHandler {
    fn default() -> Self {
        Self::new()
    }
} 