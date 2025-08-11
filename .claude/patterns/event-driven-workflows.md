# Event-Driven Workflow Pattern

## Overview

The Event-Driven Workflow pattern implements the PRIMARY architecture where **Workflow = Collection of Events + StateMachine Transitions + Generic Domain**. This pattern ensures all workflows are fundamentally event sequences managed by state machines with domain-specific logic.

## Primary Implementation Pattern

### Core Structure

```rust
/// PRIMARY: Workflow as Event Collection + State Machine + Domain
pub struct Workflow<T: GenericDomain> {
    /// Collection of events that define the workflow execution
    events: EventCollection<T::Event>,
    
    /// StateMachine that defines valid transitions  
    state_machine: StateMachine<T::State>,
    
    /// Generic domain providing domain-specific operations
    domain: T,
    
    /// Current workflow state
    current_state: T::State,
    
    /// Workflow instance identifier
    instance_id: WorkflowInstanceId,
    
    /// Workflow correlation for cross-domain operations
    correlation_id: CorrelationId,
}

/// Generic domain trait that all domains must implement
pub trait GenericDomain: Send + Sync {
    type State: Clone + Debug + Send + Sync + PartialEq;
    type Event: WorkflowEvent + Clone + Debug;
    type Context: WorkflowContext;
    
    /// Define valid state transitions
    fn valid_transitions(&self) -> StateMachine<Self::State>;
    
    /// Handle domain-specific events and generate new events
    fn handle_event(&self, event: &Self::Event, context: &mut Self::Context) -> WorkflowResult<Vec<Self::Event>>;
    
    /// Initial state for new workflows
    fn initial_state(&self) -> Self::State;
    
    /// Domain identifier
    fn domain_name(&self) -> &'static str;
}
```

### Event Collection Management

```rust
/// Collection of events with ordering and integrity
#[derive(Debug, Clone)]
pub struct EventCollection<E: WorkflowEvent> {
    /// Ordered list of events
    events: Vec<E>,
    
    /// Event index for efficient lookup
    event_index: HashMap<EventId, usize>,
    
    /// CID chain for integrity verification
    cid_chain: Vec<Cid>,
    
    /// Collection metadata
    metadata: CollectionMetadata,
}

impl<E: WorkflowEvent> EventCollection<E> {
    /// Add event to collection maintaining order and integrity
    pub fn append_event(&mut self, event: E) -> WorkflowResult<()> {
        // Validate event ordering
        if let Some(last_event) = self.events.last() {
            if event.timestamp() < last_event.timestamp() {
                return Err(WorkflowError::InvalidEventOrdering {
                    event_id: event.event_id(),
                    timestamp: event.timestamp(),
                });
            }
        }
        
        // Calculate CID for integrity
        let event_cid = event.calculate_cid();
        
        // Ensure CID chain integrity
        let previous_cid = self.cid_chain.last().cloned();
        if let Some(prev) = &previous_cid {
            if event.previous_cid() != Some(prev.clone()) {
                return Err(WorkflowError::BrokenCidChain {
                    expected: Some(prev.clone()),
                    actual: event.previous_cid(),
                });
            }
        }
        
        // Add to collection
        let index = self.events.len();
        self.event_index.insert(event.event_id(), index);
        self.cid_chain.push(event_cid);
        self.events.push(event);
        
        Ok(())
    }
    
    /// Get events by criteria
    pub fn events_by_type<T: WorkflowEvent>(&self) -> Vec<&E> {
        self.events.iter()
            .filter(|e| std::mem::discriminant(*e) == std::mem::discriminant(&T::default()))
            .collect()
    }
    
    /// Verify complete event chain integrity
    pub fn verify_integrity(&self) -> WorkflowResult<bool> {
        for (i, event) in self.events.iter().enumerate() {
            let calculated_cid = event.calculate_cid();
            if calculated_cid != self.cid_chain[i] {
                return Err(WorkflowError::IntegrityVerificationFailed {
                    event_index: i,
                    expected_cid: self.cid_chain[i].clone(),
                    actual_cid: calculated_cid,
                });
            }
        }
        Ok(true)
    }
}
```

### State Machine Framework

```rust
/// Generic state machine for workflow transitions
#[derive(Debug, Clone)]
pub struct StateMachine<S: Clone + Debug + PartialEq> {
    /// Valid state transitions
    transitions: HashMap<S, Vec<StateTransition<S>>>,
    
    /// Initial state
    initial_state: S,
    
    /// Final states
    final_states: HashSet<S>,
}

#[derive(Debug, Clone)]
pub struct StateTransition<S> {
    /// Target state
    pub to_state: S,
    
    /// Condition that must be met for transition
    pub condition: TransitionCondition,
    
    /// Events that trigger this transition
    pub triggering_events: Vec<String>,
    
    /// Actions to perform during transition
    pub actions: Vec<TransitionAction>,
}

#[derive(Debug, Clone)]
pub enum TransitionCondition {
    /// Always valid transition
    Always,
    
    /// Context-based condition
    ContextCondition { field: String, expected_value: serde_json::Value },
    
    /// Event-based condition  
    EventCondition { event_type: String, event_data: HashMap<String, serde_json::Value> },
    
    /// Custom condition function
    Custom(fn(&WorkflowContext) -> bool),
}

impl<S: Clone + Debug + PartialEq> StateMachine<S> {
    /// Check if transition is valid
    pub fn can_transition(&self, from: &S, to: &S, context: &WorkflowContext) -> bool {
        if let Some(transitions) = self.transitions.get(from) {
            transitions.iter().any(|t| {
                t.to_state == *to && self.evaluate_condition(&t.condition, context)
            })
        } else {
            false
        }
    }
    
    /// Get valid next states from current state
    pub fn valid_next_states(&self, current: &S, context: &WorkflowContext) -> Vec<S> {
        if let Some(transitions) = self.transitions.get(current) {
            transitions.iter()
                .filter(|t| self.evaluate_condition(&t.condition, context))
                .map(|t| t.to_state.clone())
                .collect()
        } else {
            vec![]
        }
    }
    
    /// Execute state transition
    pub fn transition(&self, from: &S, to: &S, context: &mut WorkflowContext) -> WorkflowResult<Vec<TransitionAction>> {
        if !self.can_transition(from, to, context) {
            return Err(WorkflowError::InvalidTransition {
                from: format!("{:?}", from),
                to: format!("{:?}", to),
                reason: "Transition not allowed by state machine".to_string(),
            });
        }
        
        if let Some(transitions) = self.transitions.get(from) {
            if let Some(transition) = transitions.iter().find(|t| t.to_state == *to) {
                // Execute transition actions
                for action in &transition.actions {
                    action.execute(context)?;
                }
                return Ok(transition.actions.clone());
            }
        }
        
        Ok(vec![])
    }
    
    /// Check if state is final
    pub fn is_final_state(&self, state: &S) -> bool {
        self.final_states.contains(state)
    }
}
```

## Domain Implementation Examples

### Document Domain

```rust
/// Document workflow domain
#[derive(Debug, Clone)]
pub struct DocumentDomain {
    document_service: Arc<dyn DocumentService>,
}

/// Document workflow states
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentWorkflowState {
    Draft,
    InReview,
    ChangesRequested,
    Approved,
    Published,
    Archived,
    Rejected,
}

/// Document workflow events
#[derive(Debug, Clone)]
pub enum DocumentWorkflowEvent {
    DocumentCreated {
        event_id: EventId,
        document_id: DocumentId,
        author_id: String,
        title: String,
        timestamp: DateTime<Utc>,
        cid: Option<Cid>,
        previous_cid: Option<Cid>,
    },
    ReviewStarted {
        event_id: EventId,
        document_id: DocumentId,
        reviewer_id: String,
        timestamp: DateTime<Utc>,
        cid: Option<Cid>,
        previous_cid: Option<Cid>,
    },
    ReviewCompleted {
        event_id: EventId,
        document_id: DocumentId,
        reviewer_id: String,
        decision: ReviewDecision,
        comments: String,
        timestamp: DateTime<Utc>,
        cid: Option<Cid>,
        previous_cid: Option<Cid>,
    },
    DocumentPublished {
        event_id: EventId,
        document_id: DocumentId,
        published_by: String,
        timestamp: DateTime<Utc>,
        cid: Option<Cid>,
        previous_cid: Option<Cid>,
    },
}

impl GenericDomain for DocumentDomain {
    type State = DocumentWorkflowState;
    type Event = DocumentWorkflowEvent;
    type Context = WorkflowContext;
    
    fn valid_transitions(&self) -> StateMachine<Self::State> {
        let mut sm = StateMachine::new(DocumentWorkflowState::Draft);
        
        // Draft → InReview
        sm.add_transition(
            DocumentWorkflowState::Draft,
            StateTransition {
                to_state: DocumentWorkflowState::InReview,
                condition: TransitionCondition::EventCondition {
                    event_type: "DocumentWorkflowEvent::ReviewStarted".to_string(),
                    event_data: HashMap::new(),
                },
                triggering_events: vec!["review_started".to_string()],
                actions: vec![TransitionAction::NotifyReviewer],
            }
        );
        
        // InReview → Approved
        sm.add_transition(
            DocumentWorkflowState::InReview,
            StateTransition {
                to_state: DocumentWorkflowState::Approved,
                condition: TransitionCondition::EventCondition {
                    event_type: "DocumentWorkflowEvent::ReviewCompleted".to_string(),
                    event_data: HashMap::from([(\"decision\".to_string(), serde_json::json!(\"Approved\"))]),
                },
                triggering_events: vec![\"review_completed\".to_string()],
                actions: vec![TransitionAction::NotifyAuthor],
            }
        );
        
        // InReview → ChangesRequested  
        sm.add_transition(
            DocumentWorkflowState::InReview,
            StateTransition {
                to_state: DocumentWorkflowState::ChangesRequested,
                condition: TransitionCondition::EventCondition {
                    event_type: "DocumentWorkflowEvent::ReviewCompleted".to_string(),
                    event_data: HashMap::from([(\"decision\".to_string(), serde_json::json!(\"ChangesRequested\"))]),
                },
                triggering_events: vec![\"review_completed\".to_string()],
                actions: vec![TransitionAction::NotifyAuthor],
            }
        );
        
        // Approved → Published
        sm.add_transition(
            DocumentWorkflowState::Approved,
            StateTransition {
                to_state: DocumentWorkflowState::Published,
                condition: TransitionCondition::Always,
                triggering_events: vec![\"document_published\".to_string()],
                actions: vec![TransitionAction::PublishDocument],
            }
        );
        
        sm.add_final_states(vec![
            DocumentWorkflowState::Published,
            DocumentWorkflowState::Archived,
            DocumentWorkflowState::Rejected,
        ]);
        
        sm
    }
    
    fn handle_event(&self, event: &Self::Event, context: &mut Self::Context) -> WorkflowResult<Vec<Self::Event>> {
        let mut generated_events = Vec::new();
        
        match event {
            DocumentWorkflowEvent::DocumentCreated { document_id, author_id, title, .. } => {
                // Store document in context
                context.add_domain_extension(
                    "document".to_string(),
                    serde_json::json!({
                        \"document_id\": document_id,
                        \"author_id\": author_id,
                        \"title\": title,
                        \"status\": \"draft\"
                    }),
                    \"1.0\".to_string(),
                );
                
                // Generate workflow started event
                generated_events.push(DocumentWorkflowEvent::WorkflowStarted {
                    event_id: EventId::new(),
                    document_id: document_id.clone(),
                    timestamp: Utc::now(),
                    cid: None,
                    previous_cid: event.cid().cloned(),
                });
            }
            
            DocumentWorkflowEvent::ReviewCompleted { decision, document_id, .. } => {
                match decision {
                    ReviewDecision::Approved => {
                        // Generate approval event
                        generated_events.push(DocumentWorkflowEvent::DocumentApproved {
                            event_id: EventId::new(),
                            document_id: document_id.clone(),
                            timestamp: Utc::now(),
                            cid: None,
                            previous_cid: event.cid().cloned(),
                        });
                    }
                    ReviewDecision::ChangesRequested => {
                        // Generate changes requested event
                        generated_events.push(DocumentWorkflowEvent::ChangesRequested {
                            event_id: EventId::new(),
                            document_id: document_id.clone(),
                            timestamp: Utc::now(),
                            cid: None,
                            previous_cid: event.cid().cloned(),
                        });
                    }
                    ReviewDecision::Rejected => {
                        // Generate rejection event
                        generated_events.push(DocumentWorkflowEvent::DocumentRejected {
                            event_id: EventId::new(),
                            document_id: document_id.clone(),
                            timestamp: Utc::now(),
                            cid: None,
                            previous_cid: event.cid().cloned(),
                        });
                    }
                }
            }
            
            _ => {
                // Other events handled by default processing
            }
        }
        
        Ok(generated_events)
    }
    
    fn initial_state(&self) -> Self::State {
        DocumentWorkflowState::Draft
    }
    
    fn domain_name(&self) -> &'static str {
        "document"
    }
}
```

### Workflow Execution Engine

```rust
/// Universal workflow executor implementing the primary pattern
pub struct WorkflowExecutor {
    /// Active workflow instances
    workflows: HashMap<WorkflowInstanceId, Box<dyn WorkflowInstance>>,
    
    /// Event publishers for cross-domain coordination
    event_publishers: Vec<Arc<dyn EventPublisher>>,
    
    /// Workflow correlation tracker
    correlation_tracker: WorkflowCorrelationTracker,
}

/// Trait object for workflow instances
pub trait WorkflowInstance: Send + Sync {
    fn instance_id(&self) -> WorkflowInstanceId;
    fn current_state(&self) -> String;
    fn process_event(&mut self, event: Box<dyn WorkflowEvent>) -> WorkflowResult<Vec<Box<dyn WorkflowEvent>>>;
    fn can_transition_to(&self, state: &str) -> bool;
    fn transition_to(&mut self, state: &str) -> WorkflowResult<()>;
    fn is_complete(&self) -> bool;
    fn domain(&self) -> &'static str;
}

impl<T: GenericDomain + 'static> WorkflowInstance for Workflow<T> {
    fn instance_id(&self) -> WorkflowInstanceId {
        self.instance_id
    }
    
    fn current_state(&self) -> String {
        format!("{:?}", self.current_state)
    }
    
    fn process_event(&mut self, event: Box<dyn WorkflowEvent>) -> WorkflowResult<Vec<Box<dyn WorkflowEvent>>> {
        // Cast event to domain-specific type
        let domain_event = event.downcast::<T::Event>()
            .map_err(|_| WorkflowError::InvalidEventType)?;
            
        // Handle event with domain logic
        let generated_events = self.domain.handle_event(&domain_event, &mut self.context)?;
        
        // Add original event to collection
        self.events.append_event(*domain_event)?;
        
        // Process generated events
        let mut result_events = Vec::new();
        for gen_event in generated_events {
            self.events.append_event(gen_event.clone())?;
            result_events.push(Box::new(gen_event) as Box<dyn WorkflowEvent>);
        }
        
        Ok(result_events)
    }
    
    fn can_transition_to(&self, state: &str) -> bool {
        // Parse state string back to domain state type
        if let Ok(target_state) = self.parse_state(state) {
            self.state_machine.can_transition(&self.current_state, &target_state, &self.context)
        } else {
            false
        }
    }
    
    fn transition_to(&mut self, state: &str) -> WorkflowResult<()> {
        let target_state = self.parse_state(state)?;
        
        if self.state_machine.can_transition(&self.current_state, &target_state, &self.context) {
            let actions = self.state_machine.transition(&self.current_state, &target_state, &mut self.context)?;
            
            // Execute transition actions
            for action in actions {
                action.execute(&mut self.context)?;
            }
            
            self.current_state = target_state;
            Ok(())
        } else {
            Err(WorkflowError::InvalidTransition {
                from: format!("{:?}", self.current_state),
                to: state.to_string(),
                reason: "State machine does not allow this transition".to_string(),
            })
        }
    }
    
    fn is_complete(&self) -> bool {
        self.state_machine.is_final_state(&self.current_state)
    }
    
    fn domain(&self) -> &'static str {
        self.domain.domain_name()
    }
}
```

## Benefits of Event-Driven Workflows

### 1. **Complete Auditability**
Every workflow action is captured as an event with CID integrity:
```rust
// Full audit trail through event collection
let audit_trail = workflow.events.get_all_events();
for event in audit_trail {
    println!("Event: {:?} at {:?} with CID: {:?}", 
             event.event_type(), event.timestamp(), event.cid());
}
```

### 2. **State Machine Consistency**
Invalid transitions are prevented by the state machine:
```rust
// State machine prevents invalid transitions
if workflow.can_transition_to("Published") {
    workflow.transition_to("Published")?;
} else {
    // Transition not allowed by current state and conditions
}
```

### 3. **Event Replay and Recovery**
Workflows can be reconstructed from their event collections:
```rust
// Reconstruct workflow from events
let reconstructed_workflow = Workflow::from_events(events, domain)?;
assert_eq!(original_workflow.current_state(), reconstructed_workflow.current_state());
```

### 4. **Cross-Domain Coordination**
Events enable coordination across domain boundaries:
```rust
// Cross-domain event correlation
let document_event = DocumentWorkflowEvent::ReviewCompleted { /* ... */ };
let person_event = PersonWorkflowEvent::NotificationSent { 
    correlation_id: document_event.correlation_id(),
    /* ... */ 
};
```

This pattern ensures that ALL workflows in CIM follow the primary implementation of **Collection of Events + StateMachine Transitions + Generic Domain**, providing consistency, auditability, and mathematical rigor across the entire system.