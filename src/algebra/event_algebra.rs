//! Event Algebra Implementation
//!
//! Implements the mathematical foundation for the Workflow Event Algebra
//! 𝒲 = (𝔼, 𝔾, 𝒯, ℂ, ⊕, ⊗, →) with type-safe algebraic operations.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Event Domain (𝔼) - All possible workflow events across CIM domains
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Unique event identifier
    pub id: Uuid,
    /// Event type classification
    pub event_type: EventType,
    /// Originating domain
    pub domain: String,
    /// Correlation identifier for causation tracking
    pub correlation_id: Uuid,
    /// Causation chain showing event ancestry
    pub causation_chain: CausationChain,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event-specific payload
    pub payload: EventPayload,
    /// Event context information
    pub context: EventContext,
}

/// Event type classification for the algebra
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Workflow lifecycle events (𝔼_w)
    Lifecycle(LifecycleEventType),
    /// Step execution events (𝔼_s)  
    Step(StepEventType),
    /// Cross-domain coordination events (𝔼_c)
    CrossDomain(CrossDomainEventType),
    /// Extension-specific events (𝔼_x)
    Extension(ExtensionEventType),
}

/// Workflow lifecycle event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleEventType {
    WorkflowCreated,
    WorkflowStarted, 
    WorkflowPaused,
    WorkflowResumed,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
}

/// Step execution event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepEventType {
    StepCreated,
    StepStarted,
    StepCompleted,
    StepFailed,
    StepSkipped,
    StepWaiting,
}

/// Cross-domain coordination event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossDomainEventType {
    DomainTransition,
    CrossDomainRequest,
    CrossDomainResponse,
    DomainSynchronization,
    DistributedTransaction,
}

/// Extension-specific event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionEventType {
    Custom(String),
    Integration(String),
    Notification(String),
}

/// Event payload containing domain-specific data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventPayload {
    /// Structured data
    pub data: HashMap<String, serde_json::Value>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Event context for algebraic operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventContext {
    /// Workflow instance identifier
    pub workflow_instance_id: Option<Uuid>,
    /// Step instance identifier
    pub step_instance_id: Option<Uuid>,
    /// Template identifier if template-based
    pub template_id: Option<String>,
    /// Domain-specific context
    pub domain_context: HashMap<String, serde_json::Value>,
}

/// Causation Chain (ℂ) - Tracks event causation and dependencies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausationChain {
    /// Root correlation identifier
    pub root_correlation: Uuid,
    /// Ordered sequence of event identifiers showing causation
    pub chain: Vec<Uuid>,
    /// Relationship types between events
    pub relationships: HashMap<(Uuid, Uuid), RelationType>,
}

/// Relationship types in causation chain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    CausedBy,
    DependsOn,
    ParallelTo,
    SequenceAfter,
}

/// Gateway Set (𝔾) - Cross-domain coordination points
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gateway {
    /// Gateway identifier
    pub id: String,
    /// Source domain
    pub source_domain: String,
    /// Target domain  
    pub target_domain: String,
    /// Gateway type
    pub gateway_type: GatewayType,
    /// Configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Gateway types for cross-domain coordination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayType {
    EventBridge,
    DataTransformer,
    SecurityBoundary,
    TransactionCoordinator,
}

/// Template Space (𝒯) - Reusable workflow patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Template version
    pub version: String,
    /// Template parameters
    pub parameters: Vec<TemplateParameter>,
    /// Template event pattern
    pub event_pattern: EventPattern,
    /// Domain compatibility
    pub supported_domains: HashSet<String>,
}

/// Template parameter definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Required parameter
    pub required: bool,
    /// Default value
    pub default_value: Option<serde_json::Value>,
}

/// Template parameter types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Domain,
}

/// Event pattern for template matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventPattern {
    /// Pattern events in sequence
    pub sequence: Vec<EventPatternNode>,
    /// Parallel event groups
    pub parallel_groups: Vec<Vec<EventPatternNode>>,
    /// Conditional branches
    pub conditionals: Vec<ConditionalPattern>,
}

/// Event pattern node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventPatternNode {
    /// Event type pattern
    pub event_type: EventType,
    /// Optional conditions
    pub conditions: Vec<PatternCondition>,
    /// Node identifier
    pub node_id: String,
}

/// Conditional pattern for template logic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionalPattern {
    /// Condition to evaluate
    pub condition: PatternCondition,
    /// True branch events
    pub true_branch: Vec<EventPatternNode>,
    /// False branch events  
    pub false_branch: Vec<EventPatternNode>,
}

/// Pattern condition for template matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternCondition {
    /// Field path to evaluate
    pub field_path: String,
    /// Condition operator
    pub operator: ConditionOperator,
    /// Expected value
    pub value: serde_json::Value,
}

/// Condition operators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
    Matches,
    In,
}

/// Identity element for Sequential Composition (ε)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequentialIdentity;

/// Identity element for Parallel Composition (I)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelIdentity;

impl WorkflowEvent {
    /// Create a new workflow event
    pub fn new(
        event_type: EventType,
        domain: String,
        correlation_id: Uuid,
        payload: EventPayload,
        context: EventContext,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            domain,
            correlation_id,
            causation_chain: CausationChain::new(correlation_id),
            timestamp: Utc::now(),
            payload,
            context,
        }
    }

    /// Create a lifecycle event
    pub fn lifecycle(
        lifecycle_type: LifecycleEventType,
        domain: String,
        correlation_id: Uuid,
        payload: EventPayload,
        context: EventContext,
    ) -> Self {
        Self::new(
            EventType::Lifecycle(lifecycle_type),
            domain,
            correlation_id,
            payload,
            context,
        )
    }

    /// Create a step event
    pub fn step(
        step_type: StepEventType,
        domain: String,
        correlation_id: Uuid,
        payload: EventPayload,
        context: EventContext,
    ) -> Self {
        Self::new(
            EventType::Step(step_type),
            domain,
            correlation_id,
            payload,
            context,
        )
    }

    /// Create a cross-domain event
    pub fn cross_domain(
        cross_domain_type: CrossDomainEventType,
        domain: String,
        correlation_id: Uuid,
        payload: EventPayload,
        context: EventContext,
    ) -> Self {
        Self::new(
            EventType::CrossDomain(cross_domain_type),
            domain,
            correlation_id,
            payload,
            context,
        )
    }

    /// Check if event is from specific domain
    pub fn is_from_domain(&self, domain: &str) -> bool {
        self.domain == domain
    }

    /// Check if event is cross-domain
    pub fn is_cross_domain(&self) -> bool {
        matches!(self.event_type, EventType::CrossDomain(_))
    }

    /// Get event type name for subject generation
    pub fn type_name(&self) -> &'static str {
        match &self.event_type {
            EventType::Lifecycle(_) => "lifecycle",
            EventType::Step(_) => "step",
            EventType::CrossDomain(_) => "cross_domain",
            EventType::Extension(_) => "extension",
        }
    }
}

impl EventPayload {
    /// Create empty payload
    pub fn empty() -> Self {
        Self {
            data: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create payload with data
    pub fn with_data(data: HashMap<String, serde_json::Value>) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get data value by key
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Set data value
    pub fn set_data(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }
}

impl EventContext {
    /// Create empty context
    pub fn empty() -> Self {
        Self {
            workflow_instance_id: None,
            step_instance_id: None,
            template_id: None,
            domain_context: HashMap::new(),
        }
    }

    /// Create context for workflow instance
    pub fn for_workflow(workflow_id: Uuid) -> Self {
        Self {
            workflow_instance_id: Some(workflow_id),
            step_instance_id: None,
            template_id: None,
            domain_context: HashMap::new(),
        }
    }

    /// Create context for step instance
    pub fn for_step(workflow_id: Uuid, step_id: Uuid) -> Self {
        Self {
            workflow_instance_id: Some(workflow_id),
            step_instance_id: Some(step_id),
            template_id: None,
            domain_context: HashMap::new(),
        }
    }

    /// Add domain-specific context
    pub fn add_domain_context(&mut self, key: String, value: serde_json::Value) {
        self.domain_context.insert(key, value);
    }
}

impl CausationChain {
    /// Create new causation chain
    pub fn new(root_correlation: Uuid) -> Self {
        Self {
            root_correlation,
            chain: vec![root_correlation],
            relationships: HashMap::new(),
        }
    }

    /// Add event to causation chain
    pub fn add_caused_by(&mut self, event_id: Uuid, caused_by: Uuid) {
        self.chain.push(event_id);
        self.relationships.insert((event_id, caused_by), RelationType::CausedBy);
    }

    /// Add parallel relationship
    pub fn add_parallel(&mut self, event1: Uuid, event2: Uuid) {
        self.relationships.insert((event1, event2), RelationType::ParallelTo);
        self.relationships.insert((event2, event1), RelationType::ParallelTo);
    }

    /// Add dependency relationship
    pub fn add_dependency(&mut self, event_id: Uuid, depends_on: Uuid) {
        self.relationships.insert((event_id, depends_on), RelationType::DependsOn);
    }

    /// Get relationship between events
    pub fn get_relationship(&self, event1: Uuid, event2: Uuid) -> Option<&RelationType> {
        self.relationships.get(&(event1, event2))
    }

    /// Check if event is in chain
    pub fn contains_event(&self, event_id: Uuid) -> bool {
        self.chain.contains(&event_id)
    }

    /// Get chain length
    pub fn length(&self) -> usize {
        self.chain.len()
    }
}

impl Default for EventPayload {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for EventContext {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_workflow_event_creation() {
        let correlation_id = Uuid::new_v4();
        let payload = EventPayload::with_data(
            vec![("workflow_name".to_string(), json!("test-workflow"))]
                .into_iter()
                .collect()
        );
        let context = EventContext::for_workflow(Uuid::new_v4());

        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "workflow".to_string(),
            correlation_id,
            payload,
            context,
        );

        assert_eq!(event.domain, "workflow");
        assert_eq!(event.correlation_id, correlation_id);
        assert!(matches!(event.event_type, EventType::Lifecycle(LifecycleEventType::WorkflowCreated)));
        assert_eq!(event.type_name(), "lifecycle");
    }

    #[test]
    fn test_causation_chain() {
        let root_id = Uuid::new_v4();
        let event1_id = Uuid::new_v4();
        let event2_id = Uuid::new_v4();

        let mut chain = CausationChain::new(root_id);
        chain.add_caused_by(event1_id, root_id);
        chain.add_caused_by(event2_id, event1_id);

        assert_eq!(chain.length(), 3);
        assert!(chain.contains_event(event1_id));
        assert_eq!(
            chain.get_relationship(event1_id, root_id),
            Some(&RelationType::CausedBy)
        );
    }

    #[test]
    fn test_event_payload() {
        let mut payload = EventPayload::empty();
        payload.set_data("key1".to_string(), json!("value1"));
        payload.add_metadata("source".to_string(), "test".to_string());

        assert_eq!(payload.get_data("key1"), Some(&json!("value1")));
        assert_eq!(payload.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_event_context() {
        let workflow_id = Uuid::new_v4();
        let step_id = Uuid::new_v4();
        
        let mut context = EventContext::for_step(workflow_id, step_id);
        context.add_domain_context("priority".to_string(), json!("high"));

        assert_eq!(context.workflow_instance_id, Some(workflow_id));
        assert_eq!(context.step_instance_id, Some(step_id));
        assert_eq!(
            context.domain_context.get("priority"),
            Some(&json!("high"))
        );
    }
}