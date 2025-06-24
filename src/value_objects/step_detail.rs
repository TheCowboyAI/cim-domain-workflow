//! Step detail value object for monitoring

use crate::value_objects::{StepId, StepStatus, StepType};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Detailed information about a workflow step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepDetail {
    /// Step identifier
    pub step_id: StepId,
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Step type
    pub step_type: StepType,
    /// Current status
    pub status: StepStatus,
    /// Assigned to user/role
    pub assigned_to: Option<String>,
    /// When the step was started
    pub started_at: Option<DateTime<Utc>>,
    /// When the step was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Estimated duration in minutes
    pub estimated_duration_minutes: Option<u32>,
    /// Actual duration in seconds
    pub actual_duration_seconds: Option<u64>,
    /// Timeout in hours
    pub timeout_hours: Option<u32>,
    /// Configuration data
    pub configuration: std::collections::HashMap<String, serde_json::Value>,
}

impl StepDetail {
    /// Check if the step is overdue based on estimated duration
    pub fn is_overdue(&self) -> bool {
        if let (Some(started_at), Some(estimated_minutes)) = (self.started_at, self.estimated_duration_minutes) {
            if self.completed_at.is_none() && self.status.is_active() {
                let elapsed = Utc::now().signed_duration_since(started_at);
                let elapsed_minutes = elapsed.num_minutes() as u32;
                return elapsed_minutes > estimated_minutes;
            }
        }
        false
    }

    /// Get the elapsed time since the step started
    pub fn elapsed_duration(&self) -> Option<chrono::Duration> {
        if let Some(started_at) = self.started_at {
            if let Some(completed_at) = self.completed_at {
                Some(completed_at.signed_duration_since(started_at))
            } else {
                Some(Utc::now().signed_duration_since(started_at))
            }
        } else {
            None
        }
    }

    /// Check if the step has a timeout risk
    pub fn has_timeout_risk(&self) -> bool {
        self.timeout_hours.is_some() && self.status.is_active()
    }
} 