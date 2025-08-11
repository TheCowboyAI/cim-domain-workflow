//! Pure Algebraic Operations
//!
//! Simple mathematical operations on workflow events:
//! - Sequential: a ⊕ b (events in order)
//! - Parallel: a ⊗ b (events concurrent)  
//! - Transform: a → b (conditional routing)

use super::event_algebra::WorkflowEvent;

/// Simple algebraic operations on events
pub struct EventAlgebra;

impl EventAlgebra {
    /// Sequential composition: a ⊕ b
    pub fn sequential(a: WorkflowEvent, b: WorkflowEvent) -> Vec<WorkflowEvent> {
        vec![a, b]
    }

    /// Parallel composition: a ⊗ b  
    pub fn parallel(a: WorkflowEvent, b: WorkflowEvent) -> Vec<WorkflowEvent> {
        vec![a, b] // Execute concurrently
    }

    /// Conditional transformation: a → b when condition
    pub fn transform(a: WorkflowEvent, condition: bool) -> Option<WorkflowEvent> {
        if condition { Some(a) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::event_algebra::*;

    #[test]
    fn test_sequential_composition() {
        let event1 = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "test".to_string(),
            uuid::Uuid::new_v4(),
            EventPayload::empty(),
            EventContext::empty(),
        );

        let event2 = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowStarted,
            "test".to_string(),
            uuid::Uuid::new_v4(),
            EventPayload::empty(),
            EventContext::empty(),
        );

        let sequence = EventAlgebra::sequential(event1, event2);
        assert_eq!(sequence.len(), 2);
    }

    #[test]
    fn test_parallel_composition() {
        let event1 = WorkflowEvent::step(
            StepEventType::StepStarted,
            "test".to_string(),
            uuid::Uuid::new_v4(),
            EventPayload::empty(),
            EventContext::empty(),
        );

        let event2 = WorkflowEvent::step(
            StepEventType::StepCompleted,
            "test".to_string(),
            uuid::Uuid::new_v4(),
            EventPayload::empty(),
            EventContext::empty(),
        );

        let parallel = EventAlgebra::parallel(event1, event2);
        assert_eq!(parallel.len(), 2);
    }

    #[test]
    fn test_conditional_transformation() {
        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "test".to_string(),
            uuid::Uuid::new_v4(),
            EventPayload::empty(),
            EventContext::empty(),
        );

        // Transform when condition is true
        let result_true = EventAlgebra::transform(event.clone(), true);
        assert!(result_true.is_some());

        // Don't transform when condition is false
        let result_false = EventAlgebra::transform(event, false);
        assert!(result_false.is_none());
    }
}