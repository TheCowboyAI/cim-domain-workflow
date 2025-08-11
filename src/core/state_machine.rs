//! Unified state machine framework for the consolidated workflow domain
//! 
//! This module provides abstract state machine traits and implementations that
//! enable consistent state management across all domain extensions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use crate::primitives::WorkflowContext;

/// Abstract state machine trait for workflow and step state management
#[async_trait]
pub trait StateMachine<S, E, A>: Send + Sync + Debug 
where
    S: State + Send + Sync + 'static,
    E: Event + Send + Sync + 'static,
    A: Action + Send + Sync + 'static,
{
    /// Get current state
    fn current_state(&self) -> &S;

    /// Process an event and potentially transition to a new state
    async fn process_event(
        &mut self,
        event: E,
        context: &WorkflowContext,
    ) -> Result<StateTransitionResult<S, A>, StateMachineError>;

    /// Check if transition is valid
    fn can_transition(&self, from: &S, to: &S, event: &E) -> bool;

    /// Get all valid transitions from current state
    fn valid_transitions(&self) -> Vec<StateTransition<S, E>>;

    /// Get all reachable states from current state
    fn reachable_states(&self) -> HashSet<S>;

    /// Validate state machine configuration
    fn validate(&self) -> Result<(), StateMachineError>;

    /// Get state machine metadata
    fn metadata(&self) -> StateMachineMetadata;
}

/// Trait for state types
pub trait State: Clone + PartialEq + Eq + std::hash::Hash + Debug + Send + Sync + 'static {
    /// Get state name for display
    fn name(&self) -> &str;
    
    /// Check if this is a terminal state
    fn is_terminal(&self) -> bool;
    
    /// Check if this is an initial state
    fn is_initial(&self) -> bool;

    /// Get state category for grouping
    fn category(&self) -> StateCategory;
}

/// Trait for event types
pub trait Event: Clone + PartialEq + Eq + std::hash::Hash + Debug + Send + Sync + 'static {
    /// Get event name for display
    fn name(&self) -> &str;
    
    /// Get event type for categorization
    fn event_type(&self) -> EventType;
    
    /// Check if event requires validation
    fn requires_validation(&self) -> bool;
}

/// Trait for action types
pub trait Action: Clone + Debug + Send + Sync + 'static {
    /// Get action name for display
    fn name(&self) -> &str;
    
    /// Execute the action
    async fn execute(&self, _context: &WorkflowContext) -> Result<ActionResult, ActionError>;
    
    /// Check if action has side effects
    fn has_side_effects(&self) -> bool;
}

/// State categories for logical grouping
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateCategory {
    /// Initial states
    Initial,
    /// Active processing states
    Active,
    /// Waiting states (for input, approval, etc.)
    Waiting,
    /// Error states
    Error,
    /// Terminal states (success or failure)
    Terminal,
    /// Custom domain-specific category
    Custom(String),
}

/// Event types for categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// User-initiated events
    User,
    /// System-generated events
    System,
    /// Timer/schedule events
    Timer,
    /// External system events
    External,
    /// Error events
    Error,
    /// Custom domain-specific event
    Custom(String),
}

/// Result of state transition
#[derive(Debug, Clone)]
pub struct StateTransitionResult<S, A>
where
    S: State,
    A: Action,
{
    /// Previous state
    pub from_state: S,
    /// New state after transition
    pub to_state: S,
    /// Actions to execute as part of transition
    pub actions: Vec<A>,
    /// Whether transition was successful
    pub success: bool,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Definition of a state transition
#[derive(Debug, Clone)]
pub struct StateTransition<S, E>
where
    S: State,
    E: Event,
{
    /// Source state
    pub from_state: S,
    /// Target state
    pub to_state: S,
    /// Triggering event
    pub event: E,
    /// Guard conditions (simplified for now)
    pub guards: Vec<String>,
}

/// Guard condition for state transitions
#[async_trait]
pub trait Guard: Send + Sync + Debug {
    /// Evaluate guard condition
    async fn evaluate(&self, context: &WorkflowContext) -> bool;
    
    /// Get guard name for debugging
    fn name(&self) -> &str;
}

/// Simple boolean guard
#[derive(Debug)]
pub struct BooleanGuard {
    name: String,
    condition: bool,
}

impl BooleanGuard {
    pub fn new(name: String, condition: bool) -> Self {
        Self { name, condition }
    }
}

#[async_trait]
impl Guard for BooleanGuard {
    async fn evaluate(&self, _context: &WorkflowContext) -> bool {
        self.condition
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Action execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether action succeeded
    pub success: bool,
    /// Output data from action
    pub output: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Metadata from action execution
    pub metadata: HashMap<String, serde_json::Value>,
}

/// State machine metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineMetadata {
    /// State machine name
    pub name: String,
    /// State machine version
    pub version: String,
    /// Description
    pub description: String,
    /// Domain this state machine belongs to
    pub domain: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Concrete implementation of state machine
#[derive(Debug)]
pub struct ConcreteStateMachine<S, E, A>
where
    S: State,
    E: Event,
    A: Action,
{
    /// Current state
    current_state: S,
    /// All possible states
    states: HashSet<S>,
    /// State transition rules
    transitions: HashMap<(S, E), StateTransition<S, E>>,
    /// Entry actions for states
    entry_actions: HashMap<S, Vec<A>>,
    /// Exit actions for states
    exit_actions: HashMap<S, Vec<A>>,
    /// State machine metadata
    metadata: StateMachineMetadata,
}

impl<S, E, A> ConcreteStateMachine<S, E, A>
where
    S: State,
    E: Event,
    A: Action,
{
    /// Create a new state machine
    pub fn new(
        initial_state: S,
        metadata: StateMachineMetadata,
    ) -> Self {
        let mut states = HashSet::new();
        states.insert(initial_state.clone());
        
        Self {
            current_state: initial_state,
            states,
            transitions: HashMap::new(),
            entry_actions: HashMap::new(),
            exit_actions: HashMap::new(),
            metadata,
        }
    }

    /// Add a state to the state machine
    pub fn add_state(&mut self, state: S) {
        self.states.insert(state);
    }

    /// Add a transition
    pub fn add_transition(
        &mut self,
        from: S,
        to: S,
        event: E,
        guards: Vec<String>,
    ) -> Result<(), StateMachineError> {
        // Validate states exist
        if !self.states.contains(&from) {
            return Err(StateMachineError::InvalidState(from.name().to_string()));
        }
        if !self.states.contains(&to) {
            return Err(StateMachineError::InvalidState(to.name().to_string()));
        }

        let transition = StateTransition {
            from_state: from.clone(),
            to_state: to,
            event: event.clone(),
            guards,
        };

        self.transitions.insert((from, event), transition);
        Ok(())
    }

    /// Add entry action for a state
    pub fn add_entry_action(&mut self, state: S, action: A) {
        self.entry_actions.entry(state).or_default().push(action);
    }

    /// Add exit action for a state
    pub fn add_exit_action(&mut self, state: S, action: A) {
        self.exit_actions.entry(state).or_default().push(action);
    }

    /// Get entry actions for a state
    pub fn get_entry_actions(&self, state: &S) -> Vec<&A> {
        self.entry_actions
            .get(state)
            .map(|actions| actions.iter().collect())
            .unwrap_or_default()
    }

    /// Get exit actions for a state
    pub fn get_exit_actions(&self, state: &S) -> Vec<&A> {
        self.exit_actions
            .get(state)
            .map(|actions| actions.iter().collect())
            .unwrap_or_default()
    }
}

#[async_trait]
impl<S, E, A> StateMachine<S, E, A> for ConcreteStateMachine<S, E, A>
where
    S: State,
    E: Event,
    A: Action,
{
    fn current_state(&self) -> &S {
        &self.current_state
    }

    async fn process_event(
        &mut self,
        event: E,
        context: &WorkflowContext,
    ) -> Result<StateTransitionResult<S, A>, StateMachineError> {
        let from_state = self.current_state.clone();
        
        // Find transition for current state and event
        let transition = self.transitions
            .get(&(from_state.clone(), event.clone()))
            .ok_or_else(|| StateMachineError::InvalidTransition {
                from: from_state.name().to_string(),
                event: event.name().to_string(),
            })?;

        // Guards simplified for now - could be expanded later

        // Execute exit actions
        let mut actions = Vec::new();
        if let Some(exit_actions) = self.exit_actions.get(&from_state) {
            actions.extend(exit_actions.clone());
        }

        // Update state
        self.current_state = transition.to_state.clone();

        // Execute entry actions
        if let Some(entry_actions) = self.entry_actions.get(&self.current_state) {
            actions.extend(entry_actions.clone());
        }

        Ok(StateTransitionResult {
            from_state,
            to_state: self.current_state.clone(),
            actions,
            success: true,
            metadata: HashMap::new(),
        })
    }

    fn can_transition(&self, from: &S, to: &S, event: &E) -> bool {
        if let Some(transition) = self.transitions.get(&(from.clone(), event.clone())) {
            transition.to_state == *to
        } else {
            false
        }
    }

    fn valid_transitions(&self) -> Vec<StateTransition<S, E>> {
        self.transitions
            .values()
            .filter(|t| t.from_state == self.current_state)
            .cloned()
            .collect()
    }

    fn reachable_states(&self) -> HashSet<S> {
        let mut reachable = HashSet::new();
        let mut to_visit = vec![self.current_state.clone()];
        let mut visited = HashSet::new();

        while let Some(state) = to_visit.pop() {
            if visited.contains(&state) {
                continue;
            }
            visited.insert(state.clone());
            reachable.insert(state.clone());

            // Add states reachable from this state
            for transition in self.transitions.values() {
                if transition.from_state == state && !visited.contains(&transition.to_state) {
                    to_visit.push(transition.to_state.clone());
                }
            }
        }

        reachable
    }

    fn validate(&self) -> Result<(), StateMachineError> {
        // Validate initial state exists
        if !self.states.contains(&self.current_state) {
            return Err(StateMachineError::InvalidState(
                self.current_state.name().to_string()
            ));
        }

        // Validate all transitions reference valid states
        for transition in self.transitions.values() {
            if !self.states.contains(&transition.from_state) {
                return Err(StateMachineError::InvalidState(
                    transition.from_state.name().to_string()
                ));
            }
            if !self.states.contains(&transition.to_state) {
                return Err(StateMachineError::InvalidState(
                    transition.to_state.name().to_string()
                ));
            }
        }

        // Check for unreachable states (warning, not error)
        let reachable = self.reachable_states();
        let unreachable: Vec<_> = self.states
            .iter()
            .filter(|s| !reachable.contains(s))
            .collect();

        if !unreachable.is_empty() {
            // Log warning about unreachable states
        }

        Ok(())
    }

    fn metadata(&self) -> StateMachineMetadata {
        self.metadata.clone()
    }
}

/// State machine errors
#[derive(Debug, thiserror::Error)]
pub enum StateMachineError {
    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid transition from {from} on event {event}")]
    InvalidTransition { from: String, event: String },

    #[error("Guard failed: {0}")]
    GuardFailed(String),

    #[error("Action execution failed: {0}")]
    ActionFailed(#[from] ActionError),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Action execution errors
#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid context: {0}")]
    InvalidContext(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Builder for constructing state machines
pub struct StateMachineBuilder<S, E, A>
where
    S: State,
    E: Event,
    A: Action,
{
    states: HashSet<S>,
    transitions: Vec<(S, S, E, Vec<String>)>,
    entry_actions: HashMap<S, Vec<A>>,
    exit_actions: HashMap<S, Vec<A>>,
    initial_state: Option<S>,
    metadata: Option<StateMachineMetadata>,
}

impl<S, E, A> StateMachineBuilder<S, E, A>
where
    S: State,
    E: Event,
    A: Action,
{
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            states: HashSet::new(),
            transitions: Vec::new(),
            entry_actions: HashMap::new(),
            exit_actions: HashMap::new(),
            initial_state: None,
            metadata: None,
        }
    }

    /// Set initial state
    pub fn initial_state(mut self, state: S) -> Self {
        self.states.insert(state.clone());
        self.initial_state = Some(state);
        self
    }

    /// Add a state
    pub fn state(mut self, state: S) -> Self {
        self.states.insert(state);
        self
    }

    /// Add a transition
    pub fn transition(
        mut self,
        from: S,
        to: S,
        event: E,
        guards: Vec<String>,
    ) -> Self {
        self.states.insert(from.clone());
        self.states.insert(to.clone());
        self.transitions.push((from, to, event, guards));
        self
    }

    /// Add entry action
    pub fn entry_action(mut self, state: S, action: A) -> Self {
        self.entry_actions.entry(state).or_default().push(action);
        self
    }

    /// Add exit action
    pub fn exit_action(mut self, state: S, action: A) -> Self {
        self.exit_actions.entry(state).or_default().push(action);
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: StateMachineMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Build the state machine
    pub fn build(self) -> Result<ConcreteStateMachine<S, E, A>, StateMachineError> {
        let initial_state = self.initial_state
            .ok_or_else(|| StateMachineError::ValidationError(
                "Initial state must be specified".to_string()
            ))?;

        let metadata = self.metadata
            .ok_or_else(|| StateMachineError::ValidationError(
                "Metadata must be specified".to_string()
            ))?;

        let mut state_machine = ConcreteStateMachine::new(initial_state, metadata);

        // Add states
        for state in self.states {
            state_machine.add_state(state);
        }

        // Add transitions
        for (from, to, event, guards) in self.transitions {
            state_machine.add_transition(from, to, event, guards)?;
        }

        // Add actions
        for (state, actions) in self.entry_actions {
            for action in actions {
                state_machine.add_entry_action(state.clone(), action);
            }
        }

        for (state, actions) in self.exit_actions {
            for action in actions {
                state_machine.add_exit_action(state.clone(), action);
            }
        }

        // Validate and return
        state_machine.validate()?;
        Ok(state_machine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test implementations
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestState {
        Initial,
        Processing,
        Complete,
        Failed,
    }

    impl State for TestState {
        fn name(&self) -> &str {
            match self {
                TestState::Initial => "Initial",
                TestState::Processing => "Processing",
                TestState::Complete => "Complete",
                TestState::Failed => "Failed",
            }
        }

        fn is_terminal(&self) -> bool {
            matches!(self, TestState::Complete | TestState::Failed)
        }

        fn is_initial(&self) -> bool {
            matches!(self, TestState::Initial)
        }

        fn category(&self) -> StateCategory {
            match self {
                TestState::Initial => StateCategory::Initial,
                TestState::Processing => StateCategory::Active,
                TestState::Complete => StateCategory::Terminal,
                TestState::Failed => StateCategory::Error,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestEvent {
        Start,
        Complete,
        Fail,
    }

    impl Event for TestEvent {
        fn name(&self) -> &str {
            match self {
                TestEvent::Start => "Start",
                TestEvent::Complete => "Complete",
                TestEvent::Fail => "Fail",
            }
        }

        fn event_type(&self) -> EventType {
            EventType::User
        }

        fn requires_validation(&self) -> bool {
            false
        }
    }

    #[derive(Debug, Clone)]
    struct TestAction {
        name: String,
    }

    #[async_trait]
    impl Action for TestAction {
        fn name(&self) -> &str {
            &self.name
        }

        async fn execute(&self, _context: &WorkflowContext) -> Result<ActionResult, ActionError> {
            Ok(ActionResult {
                success: true,
                output: None,
                error: None,
                metadata: HashMap::new(),
            })
        }

        fn has_side_effects(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_state_machine_builder() {
        let metadata = StateMachineMetadata {
            name: "Test State Machine".to_string(),
            version: "1.0.0".to_string(),
            description: "Test state machine".to_string(),
            domain: "test".to_string(),
            created_at: chrono::Utc::now(),
        };

        let state_machine: Result<ConcreteStateMachine<TestState, TestEvent, TestAction>, _> = StateMachineBuilder::new()
            .initial_state(TestState::Initial)
            .state(TestState::Processing)
            .state(TestState::Complete)
            .state(TestState::Failed)
            .transition(
                TestState::Initial,
                TestState::Processing,
                TestEvent::Start,
                vec![],
            )
            .transition(
                TestState::Processing,
                TestState::Complete,
                TestEvent::Complete,
                vec![],
            )
            .transition(
                TestState::Processing,
                TestState::Failed,
                TestEvent::Fail,
                vec![],
            )
            .metadata(metadata)
            .build();

        assert!(state_machine.is_ok());
        let sm = state_machine.unwrap();
        assert_eq!(sm.current_state(), &TestState::Initial);
        assert_eq!(sm.valid_transitions().len(), 1);
    }

    #[tokio::test]
    async fn test_state_transition() {
        let metadata = StateMachineMetadata {
            name: "Test State Machine".to_string(),
            version: "1.0.0".to_string(),
            description: "Test state machine".to_string(),
            domain: "test".to_string(),
            created_at: chrono::Utc::now(),
        };

        let mut state_machine: ConcreteStateMachine<TestState, TestEvent, TestAction> = StateMachineBuilder::new()
            .initial_state(TestState::Initial)
            .transition(
                TestState::Initial,
                TestState::Processing,
                TestEvent::Start,
                vec![],
            )
            .metadata(metadata)
            .build()
            .unwrap();

        let workflow_id = crate::primitives::UniversalWorkflowId::new("test".to_string(), None);
        let instance_id = crate::primitives::WorkflowInstanceId::new(workflow_id.clone());
        let context = WorkflowContext::new(workflow_id, instance_id, None);

        let result = state_machine.process_event(TestEvent::Start, &context).await;
        assert!(result.is_ok());
        
        let transition_result: StateTransitionResult<TestState, TestAction> = result.unwrap();
        assert_eq!(transition_result.from_state, TestState::Initial);
        assert_eq!(transition_result.to_state, TestState::Processing);
        assert!(transition_result.success);
    }
}