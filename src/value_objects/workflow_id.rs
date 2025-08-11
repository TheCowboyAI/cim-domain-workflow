//! Workflow identifier value object

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    /// Create a new workflow ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the UUID value
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for WorkflowId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<WorkflowId> for Uuid {
    fn from(id: WorkflowId) -> Self {
        id.0
    }
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
} 