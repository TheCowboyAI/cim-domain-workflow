//! Backward compatibility layer for the consolidated workflow domain
//! 
//! This module provides compatibility shims and re-exports to ensure existing
//! code continues to work during the migration to the unified architecture.

use crate::primitives::{UniversalWorkflowId, WorkflowContext, UniversalStepId};
use crate::Workflow as CoreWorkflow;
use cim_domain::AggregateRoot;

// Re-export existing types for backward compatibility
pub use crate::Workflow;
pub use crate::WorkflowId;
pub use crate::WorkflowStatus as LegacyWorkflowStatus;
pub use crate::WorkflowStep;
pub use crate::StepId;

/// Compatibility wrapper for the new UniversalWorkflowId
#[deprecated(since = "0.4.0", note = "Use UniversalWorkflowId instead")]
pub type CompatWorkflowId = UniversalWorkflowId;

/// Compatibility wrapper for the new WorkflowContext
#[deprecated(since = "0.4.0", note = "Use WorkflowContext from primitives module instead")]
pub type CompatWorkflowContext = WorkflowContext;

/// Compatibility wrapper for the new UniversalStepId
#[deprecated(since = "0.4.0", note = "Use UniversalStepId instead")]
pub type CompatStepId = UniversalStepId;

/// Migration helper functions
pub mod migration {
    use super::*;
    
    /// Convert legacy WorkflowId to UniversalWorkflowId
    pub fn convert_workflow_id(_legacy_id: &crate::WorkflowId) -> UniversalWorkflowId {
        UniversalWorkflowId::new(
            "legacy".to_string(),
            None
        )
    }
    
    /// Convert legacy StepId to UniversalStepId  
    pub fn convert_step_id(
        _legacy_step_id: &crate::StepId,
        workflow_id: &UniversalWorkflowId,
        sequence: u32
    ) -> UniversalStepId {
        UniversalStepId::new(
            workflow_id.clone(),
            sequence,
            "legacy".to_string()
        )
    }
    
    /// Create a workflow context from legacy data
    pub fn create_context_from_legacy(
        workflow: &CoreWorkflow,
        initiator: Option<String>
    ) -> WorkflowContext {
        let universal_workflow_id = convert_workflow_id(&workflow.id());
        let instance_id = crate::primitives::WorkflowInstanceId::new(universal_workflow_id.clone());
        WorkflowContext::new(universal_workflow_id, instance_id, initiator)
    }
}

/// Compatibility traits for gradual migration
pub trait ToUniversal {
    type Output;
    
    /// Convert to new universal type
    fn to_universal(&self) -> Self::Output;
}

impl ToUniversal for crate::WorkflowId {
    type Output = UniversalWorkflowId;
    
    fn to_universal(&self) -> Self::Output {
        migration::convert_workflow_id(self)
    }
}

/// Extension trait for legacy workflow operations
pub trait LegacyWorkflowExt {
    /// Get the universal workflow ID for this legacy workflow
    fn universal_id(&self) -> UniversalWorkflowId;
    
    /// Create a unified context for this legacy workflow
    fn unified_context(&self, initiator: Option<String>) -> WorkflowContext;
}

impl LegacyWorkflowExt for CoreWorkflow {
    fn universal_id(&self) -> UniversalWorkflowId {
        migration::convert_workflow_id(&self.id())
    }
    
    fn unified_context(&self, initiator: Option<String>) -> WorkflowContext {
        migration::create_context_from_legacy(self, initiator)
    }
}

/// Migration guide constants
pub mod migration_guide {
    pub const PHASE_1_COMPLETE: &str = "Phase 1: Core infrastructure and primitives";
    pub const PHASE_2_PLANNED: &str = "Phase 2: Event system and templates (upcoming)";
    pub const PHASE_3_PLANNED: &str = "Phase 3: Domain integration (upcoming)";
    pub const PHASE_4_PLANNED: &str = "Phase 4: Full consolidation (upcoming)";
    
    pub const MIGRATION_NOTES: &str = r#"
Migration Notes for Consolidated Workflow Domain:

Phase 1 (COMPLETED):
- Core primitives and unified identifiers implemented
- Extensible context framework available
- Core workflow engine abstraction ready
- Unified state machine framework operational
- Domain extension trait system established
- Backward compatibility layer active

Next Steps:
1. Gradually migrate domain-specific code to use UniversalWorkflowId
2. Replace WorkflowContext usage with new extensible context
3. Implement domain extensions using DomainWorkflowExtension trait
4. Plan for Phase 2 event system migration

Breaking Changes:
- None in Phase 1 - all existing APIs remain functional
- Deprecation warnings added for types being replaced

For detailed migration guides, see: doc/migration-guide.md
"#;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migration_helpers() {
        use crate::WorkflowId;
        use uuid::Uuid;
        
        let legacy_id = WorkflowId::new();
        let universal_id = migration::convert_workflow_id(&legacy_id);
        
        assert_eq!(universal_id.origin_domain(), "legacy");
        assert!(universal_id.is_from_domain("legacy"));
    }
    
    #[test]
    fn test_to_universal_trait() {
        use crate::WorkflowId;
        use uuid::Uuid;
        
        let legacy_id = WorkflowId::new();
        let universal_id = legacy_id.to_universal();
        
        assert_eq!(universal_id.origin_domain(), "legacy");
    }
}