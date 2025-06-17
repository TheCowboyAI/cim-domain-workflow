//! Workflow step identifier value object

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a workflow step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

impl StepId {
    /// Create a new step ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the UUID value
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for StepId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<StepId> for Uuid {
    fn from(id: StepId) -> Self {
        id.0
    }
} 