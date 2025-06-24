//! Workflow status value object

use serde::{Deserialize, Serialize};
use cim_domain::{DomainError, DomainResult};

/// Represents the status of a workflow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkflowStatus {
    /// Workflow is defined but not started
    Draft,
    /// Workflow is currently executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed during execution
    Failed,
    /// Workflow was paused
    Paused,
    /// Workflow was cancelled
    Cancelled,
}

impl WorkflowStatus {
    /// Check if this status can transition to another status
    pub fn can_transition_to(&self, target: &WorkflowStatus) -> bool {
        use WorkflowStatus::*;
        
        match (self, target) {
            // From Draft
            (Draft, Running) => true,
            (Draft, Cancelled) => true,
            
            // From Running
            (Running, Completed) => true,
            (Running, Failed) => true,
            (Running, Paused) => true,
            (Running, Cancelled) => true,
            
            // From Paused
            (Paused, Running) => true,
            (Paused, Cancelled) => true,
            (Paused, Failed) => true,
            
            // Terminal states cannot transition
            (Completed, _) => false,
            (Failed, _) => false,
            (Cancelled, _) => false,
            
            // All other transitions are invalid
            _ => false,
        }
    }

    /// Validate and perform a status transition
    pub fn transition_to(&self, target: WorkflowStatus) -> DomainResult<WorkflowStatus> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(DomainError::generic(format!(
                "Invalid workflow status transition from {self:?} to {target:?}"
            )))
        }
    }

    /// Check if this is a terminal status
    pub fn is_terminal(&self) -> bool {
        matches!(self, WorkflowStatus::Completed | WorkflowStatus::Failed | WorkflowStatus::Cancelled)
    }

    /// Check if the workflow is active (can execute steps)
    pub fn is_active(&self) -> bool {
        matches!(self, WorkflowStatus::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test valid status transitions
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Draft] --> B[Running]
    ///     A --> C[Cancelled]
    ///     B --> D[Completed]
    ///     B --> E[Failed]
    ///     B --> F[Paused]
    ///     B --> C
    ///     F --> B
    ///     F --> C
    ///     F --> E
    /// ```
    #[test]
    fn test_valid_transitions() {
        // From Draft
        assert!(WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Running));
        assert!(WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Cancelled));
        assert!(!WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Completed));
        assert!(!WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Failed));
        assert!(!WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Paused));

        // From Running
        assert!(WorkflowStatus::Running.can_transition_to(&WorkflowStatus::Completed));
        assert!(WorkflowStatus::Running.can_transition_to(&WorkflowStatus::Failed));
        assert!(WorkflowStatus::Running.can_transition_to(&WorkflowStatus::Paused));
        assert!(WorkflowStatus::Running.can_transition_to(&WorkflowStatus::Cancelled));
        assert!(!WorkflowStatus::Running.can_transition_to(&WorkflowStatus::Draft));

        // From Paused
        assert!(WorkflowStatus::Paused.can_transition_to(&WorkflowStatus::Running));
        assert!(WorkflowStatus::Paused.can_transition_to(&WorkflowStatus::Cancelled));
        assert!(WorkflowStatus::Paused.can_transition_to(&WorkflowStatus::Failed));
        assert!(!WorkflowStatus::Paused.can_transition_to(&WorkflowStatus::Completed));
        assert!(!WorkflowStatus::Paused.can_transition_to(&WorkflowStatus::Draft));

        // Terminal states cannot transition
        assert!(!WorkflowStatus::Completed.can_transition_to(&WorkflowStatus::Running));
        assert!(!WorkflowStatus::Failed.can_transition_to(&WorkflowStatus::Running));
        assert!(!WorkflowStatus::Cancelled.can_transition_to(&WorkflowStatus::Running));
    }

    /// Test transition_to method
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Validate Transition] --> B{Valid?}
    ///     B -->|Yes| C[Return New Status]
    ///     B -->|No| D[Return Error]
    /// ```
    #[test]
    fn test_transition_to() {
        // Valid transition
        let result = WorkflowStatus::Draft.transition_to(WorkflowStatus::Running);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), WorkflowStatus::Running);

        // Invalid transition
        let result = WorkflowStatus::Draft.transition_to(WorkflowStatus::Completed);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid workflow status transition"));
    }

    /// Test terminal status detection
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Status] --> B{Terminal?}
    ///     B -->|Completed| C[True]
    ///     B -->|Failed| C
    ///     B -->|Cancelled| C
    ///     B -->|Others| D[False]
    /// ```
    #[test]
    fn test_is_terminal() {
        assert!(!WorkflowStatus::Draft.is_terminal());
        assert!(!WorkflowStatus::Running.is_terminal());
        assert!(!WorkflowStatus::Paused.is_terminal());
        assert!(WorkflowStatus::Completed.is_terminal());
        assert!(WorkflowStatus::Failed.is_terminal());
        assert!(WorkflowStatus::Cancelled.is_terminal());
    }

    /// Test active status detection
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Status] --> B{Active?}
    ///     B -->|Running| C[True]
    ///     B -->|Others| D[False]
    /// ```
    #[test]
    fn test_is_active() {
        assert!(!WorkflowStatus::Draft.is_active());
        assert!(WorkflowStatus::Running.is_active());
        assert!(!WorkflowStatus::Paused.is_active());
        assert!(!WorkflowStatus::Completed.is_active());
        assert!(!WorkflowStatus::Failed.is_active());
        assert!(!WorkflowStatus::Cancelled.is_active());
    }

    /// Test self-transitions
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Status] -.->|Self| A
    ///     B[All Self-Transitions] --> C[Should Be Invalid]
    /// ```
    #[test]
    fn test_self_transitions() {
        // No status should be able to transition to itself
        assert!(!WorkflowStatus::Draft.can_transition_to(&WorkflowStatus::Draft));
        assert!(!WorkflowStatus::Running.can_transition_to(&WorkflowStatus::Running));
        assert!(!WorkflowStatus::Paused.can_transition_to(&WorkflowStatus::Paused));
        assert!(!WorkflowStatus::Completed.can_transition_to(&WorkflowStatus::Completed));
        assert!(!WorkflowStatus::Failed.can_transition_to(&WorkflowStatus::Failed));
        assert!(!WorkflowStatus::Cancelled.can_transition_to(&WorkflowStatus::Cancelled));
    }

    /// Test serialization/deserialization
    ///
    /// ```mermaid
    /// graph TD
    ///     A[Status] --> B[Serialize]
    ///     B --> C[JSON]
    ///     C --> D[Deserialize]
    ///     D --> E[Status]
    ///     A -.->|Equal| E
    /// ```
    #[test]
    fn test_serialization() {
        let statuses = vec![
            WorkflowStatus::Draft,
            WorkflowStatus::Running,
            WorkflowStatus::Completed,
            WorkflowStatus::Failed,
            WorkflowStatus::Paused,
            WorkflowStatus::Cancelled,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: WorkflowStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
} 