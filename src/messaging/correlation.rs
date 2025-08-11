//! Event Correlation System
//!
//! Implements CIM-compliant event correlation and causation tracking for
//! cross-domain workflow coordination based on the Workflow Subject Algebra.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::algebra::{WorkflowEvent, RelationType, Subject};

/// CIM-compliant event correlator for tracking cross-domain workflows
// Debug derive removed due to trait objects in completion_listeners
pub struct WorkflowEventCorrelator {
    /// Active correlation chains
    active_chains: Arc<RwLock<HashMap<Uuid, CorrelationChainState>>>,
    /// Event correlation index
    event_index: Arc<RwLock<HashMap<Uuid, EventCorrelationInfo>>>,
    /// Domain correlation mappings
    domain_mappings: Arc<RwLock<HashMap<String, HashSet<Uuid>>>>,
    /// Completion listeners
    completion_listeners: Arc<RwLock<HashMap<Uuid, Vec<CompletionCallback>>>>,
}

/// State of a correlation chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationChainState {
    /// Root correlation ID
    pub root_correlation: Uuid,
    /// Chain creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Events in the chain
    pub events: HashMap<Uuid, WorkflowEvent>,
    /// Causation relationships
    pub relationships: HashMap<(Uuid, Uuid), RelationType>,
    /// Domains involved in this chain
    pub involved_domains: HashSet<String>,
    /// Chain status
    pub status: CorrelationChainStatus,
    /// Completion criteria
    pub completion_criteria: CompletionCriteria,
}

/// Status of a correlation chain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationChainStatus {
    /// Chain is active and processing events
    Active,
    /// Chain is waiting for external events
    Waiting,
    /// Chain has completed successfully
    Completed,
    /// Chain has failed
    Failed,
    /// Chain has timed out
    TimedOut,
    /// Chain was cancelled
    Cancelled,
}

/// Criteria for determining chain completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionCriteria {
    /// Required terminal events
    pub required_terminals: Vec<TerminalEventPattern>,
    /// Maximum chain duration
    pub max_duration: Option<chrono::Duration>,
    /// Required domains to participate
    pub required_domains: HashSet<String>,
    /// Minimum event count
    pub min_event_count: Option<u32>,
}

/// Pattern for terminal events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalEventPattern {
    /// Subject pattern that matches terminal events
    pub subject_pattern: String,
    /// Required payload conditions
    pub payload_conditions: HashMap<String, serde_json::Value>,
}

/// Event correlation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCorrelationInfo {
    /// Event ID
    pub event_id: Uuid,
    /// Correlation chain ID
    pub correlation_chain_id: Uuid,
    /// Event position in chain
    pub chain_position: u32,
    /// Direct causal parent
    pub causal_parent: Option<Uuid>,
    /// Direct causal children
    pub causal_children: HashSet<Uuid>,
    /// Parallel siblings
    pub parallel_siblings: HashSet<Uuid>,
    /// Domain context
    pub domain_context: String,
    /// Event subject
    pub subject: Subject,
}

/// Callback for chain completion events
pub type CompletionCallback = Box<dyn Fn(Uuid, CorrelationChainStatus) + Send + Sync>;

/// Chain completion analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainCompletionAnalysis {
    /// Whether the chain is complete
    pub is_complete: bool,
    /// Completion reason
    pub completion_reason: CompletionReason,
    /// Missing requirements
    pub missing_requirements: Vec<String>,
    /// Chain statistics
    pub statistics: ChainStatistics,
}

/// Reason for chain completion or incompletion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionReason {
    /// All requirements met
    AllRequirementsMet,
    /// Timed out waiting for events
    Timeout,
    /// Chain failed due to error
    ChainFailed,
    /// Chain was cancelled
    Cancelled,
    /// Still waiting for events
    WaitingForEvents,
    /// Missing required domains
    MissingDomains,
    /// Insufficient event count
    InsufficientEvents,
}

/// Statistics for a correlation chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatistics {
    /// Total events in chain
    pub total_events: u32,
    /// Events per domain
    pub events_per_domain: HashMap<String, u32>,
    /// Chain duration so far
    pub duration: chrono::Duration,
    /// Average event processing time
    pub avg_event_processing_time: chrono::Duration,
    /// Cross-domain transitions
    pub cross_domain_transitions: u32,
}

impl WorkflowEventCorrelator {
    /// Create a new event correlator
    pub fn new() -> Self {
        Self {
            active_chains: Arc::new(RwLock::new(HashMap::new())),
            event_index: Arc::new(RwLock::new(HashMap::new())),
            domain_mappings: Arc::new(RwLock::new(HashMap::new())),
            completion_listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new correlation chain
    pub fn start_chain(
        &self,
        root_correlation: Uuid,
        completion_criteria: CompletionCriteria,
    ) -> Result<(), CorrelationError> {
        let mut chains = self.active_chains.write()
            .map_err(|_| CorrelationError::LockError("Failed to acquire chains lock".to_string()))?;

        if chains.contains_key(&root_correlation) {
            return Err(CorrelationError::ChainAlreadyExists(root_correlation));
        }

        let chain_state = CorrelationChainState {
            root_correlation,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            events: HashMap::new(),
            relationships: HashMap::new(),
            involved_domains: HashSet::new(),
            status: CorrelationChainStatus::Active,
            completion_criteria,
        };

        chains.insert(root_correlation, chain_state);
        Ok(())
    }

    /// Add event to correlation chain
    pub fn add_event(&self, event: WorkflowEvent) -> Result<(), CorrelationError> {
        let correlation_id = event.correlation_id;
        
        // Update active chains
        {
            let mut chains = self.active_chains.write()
                .map_err(|_| CorrelationError::LockError("Failed to acquire chains lock".to_string()))?;
            
            let chain_state = chains.get_mut(&correlation_id)
                .ok_or_else(|| CorrelationError::ChainNotFound(correlation_id))?;
            
            // Add event to chain
            chain_state.events.insert(event.id, event.clone());
            chain_state.involved_domains.insert(event.domain.clone());
            chain_state.updated_at = Utc::now();
            
            // Add causation relationships
            for (event_pair, relation_type) in &event.causation_chain.relationships {
                chain_state.relationships.insert(*event_pair, relation_type.clone());
            }
        }

        // Update event index
        {
            let mut index = self.event_index.write()
                .map_err(|_| CorrelationError::LockError("Failed to acquire index lock".to_string()))?;
            
            let correlation_info = EventCorrelationInfo {
                event_id: event.id,
                correlation_chain_id: correlation_id,
                chain_position: event.causation_chain.length() as u32,
                causal_parent: self.find_causal_parent(&event),
                causal_children: HashSet::new(),
                parallel_siblings: self.find_parallel_siblings(&event),
                domain_context: event.domain.clone(),
                subject: Subject::from_event(&event),
            };
            
            index.insert(event.id, correlation_info);
        }

        // Update domain mappings
        {
            let mut mappings = self.domain_mappings.write()
                .map_err(|_| CorrelationError::LockError("Failed to acquire mappings lock".to_string()))?;
            
            mappings.entry(event.domain.clone())
                .or_insert_with(HashSet::new)
                .insert(correlation_id);
        }

        // Check for chain completion
        self.check_chain_completion(correlation_id)?;

        Ok(())
    }

    /// Analyze chain completion status
    pub fn analyze_completion(&self, correlation_id: Uuid) -> Result<ChainCompletionAnalysis, CorrelationError> {
        let chains = self.active_chains.read()
            .map_err(|_| CorrelationError::LockError("Failed to acquire chains lock".to_string()))?;
        
        let chain_state = chains.get(&correlation_id)
            .ok_or_else(|| CorrelationError::ChainNotFound(correlation_id))?;

        let mut missing_requirements = Vec::new();
        let mut is_complete = true;

        // Check terminal events
        for terminal_pattern in &chain_state.completion_criteria.required_terminals {
            if !self.has_matching_terminal_event(chain_state, terminal_pattern) {
                missing_requirements.push(format!("Terminal event: {}", terminal_pattern.subject_pattern));
                is_complete = false;
            }
        }

        // Check required domains
        for required_domain in &chain_state.completion_criteria.required_domains {
            if !chain_state.involved_domains.contains(required_domain) {
                missing_requirements.push(format!("Domain: {}", required_domain));
                is_complete = false;
            }
        }

        // Check minimum event count
        if let Some(min_count) = chain_state.completion_criteria.min_event_count {
            if chain_state.events.len() < min_count as usize {
                missing_requirements.push(format!("Minimum {} events", min_count));
                is_complete = false;
            }
        }

        // Check timeout
        let completion_reason = if let Some(max_duration) = chain_state.completion_criteria.max_duration {
            let elapsed = Utc::now() - chain_state.created_at;
            if elapsed > max_duration {
                is_complete = false;
                CompletionReason::Timeout
            } else if is_complete {
                CompletionReason::AllRequirementsMet
            } else {
                CompletionReason::WaitingForEvents
            }
        } else if is_complete {
            CompletionReason::AllRequirementsMet
        } else {
            CompletionReason::WaitingForEvents
        };

        let statistics = self.calculate_chain_statistics(chain_state);

        Ok(ChainCompletionAnalysis {
            is_complete,
            completion_reason,
            missing_requirements,
            statistics,
        })
    }

    /// Get events correlated to a specific event
    pub fn get_correlated_events(&self, event_id: Uuid) -> Result<Vec<WorkflowEvent>, CorrelationError> {
        let index = self.event_index.read()
            .map_err(|_| CorrelationError::LockError("Failed to acquire index lock".to_string()))?;
        
        let correlation_info = index.get(&event_id)
            .ok_or_else(|| CorrelationError::EventNotFound(event_id))?;
        
        let chains = self.active_chains.read()
            .map_err(|_| CorrelationError::LockError("Failed to acquire chains lock".to_string()))?;
        
        let chain_state = chains.get(&correlation_info.correlation_chain_id)
            .ok_or_else(|| CorrelationError::ChainNotFound(correlation_info.correlation_chain_id))?;

        Ok(chain_state.events.values().cloned().collect())
    }

    /// Subscribe to chain completion events
    pub fn subscribe_to_completion(
        &self,
        correlation_id: Uuid,
        callback: CompletionCallback,
    ) -> Result<(), CorrelationError> {
        let mut listeners = self.completion_listeners.write()
            .map_err(|_| CorrelationError::LockError("Failed to acquire listeners lock".to_string()))?;
        
        listeners.entry(correlation_id)
            .or_insert_with(Vec::new)
            .push(callback);
        
        Ok(())
    }

    /// Get correlation chain statistics
    pub fn get_chain_statistics(&self, correlation_id: Uuid) -> Result<ChainStatistics, CorrelationError> {
        let chains = self.active_chains.read()
            .map_err(|_| CorrelationError::LockError("Failed to acquire chains lock".to_string()))?;
        
        let chain_state = chains.get(&correlation_id)
            .ok_or_else(|| CorrelationError::ChainNotFound(correlation_id))?;

        Ok(self.calculate_chain_statistics(chain_state))
    }

    /// Private helper methods
    
    fn find_causal_parent(&self, event: &WorkflowEvent) -> Option<Uuid> {
        for (event_pair, relation_type) in &event.causation_chain.relationships {
            if event_pair.0 == event.id && *relation_type == RelationType::CausedBy {
                return Some(event_pair.1);
            }
        }
        None
    }

    fn find_parallel_siblings(&self, event: &WorkflowEvent) -> HashSet<Uuid> {
        let mut siblings = HashSet::new();
        for (event_pair, relation_type) in &event.causation_chain.relationships {
            if event_pair.0 == event.id && *relation_type == RelationType::ParallelTo {
                siblings.insert(event_pair.1);
            } else if event_pair.1 == event.id && *relation_type == RelationType::ParallelTo {
                siblings.insert(event_pair.0);
            }
        }
        siblings
    }

    fn has_matching_terminal_event(&self, chain_state: &CorrelationChainState, pattern: &TerminalEventPattern) -> bool {
        for event in chain_state.events.values() {
            let subject = Subject::from_event(event);
            if subject.to_canonical_string().contains(&pattern.subject_pattern) {
                // Check payload conditions
                if self.check_payload_conditions(event, &pattern.payload_conditions) {
                    return true;
                }
            }
        }
        false
    }

    fn check_payload_conditions(&self, event: &WorkflowEvent, conditions: &HashMap<String, serde_json::Value>) -> bool {
        for (key, expected_value) in conditions {
            if let Some(actual_value) = event.payload.get_data(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn calculate_chain_statistics(&self, chain_state: &CorrelationChainState) -> ChainStatistics {
        let mut events_per_domain = HashMap::new();
        let mut cross_domain_transitions = 0;
        let mut last_domain: Option<&String> = None;
        let mut last_timestamp: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut total_processing_time = chrono::Duration::zero();

        // Sort events by timestamp for proper processing time calculation
        let mut sorted_events: Vec<_> = chain_state.events.values().collect();
        sorted_events.sort_by_key(|e| e.timestamp);

        for event in sorted_events {
            // Count events per domain
            *events_per_domain.entry(event.domain.clone()).or_insert(0) += 1;

            // Calculate processing time between events
            if let Some(last_ts) = last_timestamp {
                total_processing_time = total_processing_time + (event.timestamp - last_ts);
            }
            last_timestamp = Some(event.timestamp);

            // Count cross-domain transitions
            if let Some(prev_domain) = last_domain {
                if prev_domain != &event.domain {
                    cross_domain_transitions += 1;
                }
            }
            last_domain = Some(&event.domain);
        }

        let duration = chain_state.updated_at - chain_state.created_at;
        let avg_processing_time = if chain_state.events.is_empty() {
            chrono::Duration::zero()
        } else {
            duration / chain_state.events.len() as i32
        };

        ChainStatistics {
            total_events: chain_state.events.len() as u32,
            events_per_domain,
            duration,
            avg_event_processing_time: avg_processing_time,
            cross_domain_transitions,
        }
    }

    fn check_chain_completion(&self, correlation_id: Uuid) -> Result<(), CorrelationError> {
        let analysis = self.analyze_completion(correlation_id)?;
        
        if analysis.is_complete || matches!(analysis.completion_reason, CompletionReason::Timeout | CompletionReason::ChainFailed) {
            // Update chain status
            {
                let mut chains = self.active_chains.write()
                    .map_err(|_| CorrelationError::LockError("Failed to acquire chains lock".to_string()))?;
                
                if let Some(chain_state) = chains.get_mut(&correlation_id) {
                    chain_state.status = match analysis.completion_reason {
                        CompletionReason::AllRequirementsMet => CorrelationChainStatus::Completed,
                        CompletionReason::Timeout => CorrelationChainStatus::TimedOut,
                        CompletionReason::ChainFailed => CorrelationChainStatus::Failed,
                        CompletionReason::Cancelled => CorrelationChainStatus::Cancelled,
                        _ => CorrelationChainStatus::Active,
                    };
                }
            }

            // Notify completion listeners
            self.notify_completion_listeners(correlation_id, analysis.completion_reason)?;
        }

        Ok(())
    }

    fn notify_completion_listeners(&self, correlation_id: Uuid, reason: CompletionReason) -> Result<(), CorrelationError> {
        let status = match reason {
            CompletionReason::AllRequirementsMet => CorrelationChainStatus::Completed,
            CompletionReason::Timeout => CorrelationChainStatus::TimedOut,
            CompletionReason::ChainFailed => CorrelationChainStatus::Failed,
            CompletionReason::Cancelled => CorrelationChainStatus::Cancelled,
            _ => return Ok(()), // Don't notify for incomplete chains
        };

        let listeners = self.completion_listeners.read()
            .map_err(|_| CorrelationError::LockError("Failed to acquire listeners lock".to_string()))?;
        
        if let Some(callbacks) = listeners.get(&correlation_id) {
            for callback in callbacks {
                callback(correlation_id, status.clone());
            }
        }

        Ok(())
    }

    /// Cancel an active correlation chain
    pub async fn cancel_correlation(&self, correlation_id: Uuid) -> Result<(), CorrelationError> {
        let mut chains = self.active_chains.write()
            .map_err(|_| CorrelationError::LockError("Failed to acquire chains lock".to_string()))?;
        
        if let Some(chain) = chains.get_mut(&correlation_id) {
            chain.status = CorrelationChainStatus::Cancelled;
            chain.updated_at = Utc::now();
            
            // Notify completion listeners about cancellation
            self.notify_completion_listeners(correlation_id, CompletionReason::Cancelled)?;
        }
        
        Ok(())
    }
}

/// Correlation system errors
#[derive(Debug, thiserror::Error)]
pub enum CorrelationError {
    #[error("Chain already exists: {0}")]
    ChainAlreadyExists(Uuid),

    #[error("Chain not found: {0}")]
    ChainNotFound(Uuid),

    #[error("Event not found: {0}")]
    EventNotFound(Uuid),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Invalid completion criteria: {0}")]
    InvalidCompletionCriteria(String),

    #[error("Correlation processing error: {0}")]
    ProcessingError(String),
}

impl Default for CompletionCriteria {
    fn default() -> Self {
        Self {
            required_terminals: Vec::new(),
            max_duration: Some(chrono::Duration::hours(1)),
            required_domains: HashSet::new(),
            min_event_count: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algebra::event_algebra::*;

    #[test]
    fn test_correlation_chain_creation() {
        let correlator = WorkflowEventCorrelator::new();
        let correlation_id = Uuid::new_v4();
        
        let result = correlator.start_chain(correlation_id, CompletionCriteria::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_correlation() {
        let correlator = WorkflowEventCorrelator::new();
        let correlation_id = Uuid::new_v4();
        
        correlator.start_chain(correlation_id, CompletionCriteria::default()).unwrap();
        
        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "workflow".to_string(),
            correlation_id,
            EventPayload::empty(),
            EventContext::empty(),
        );
        
        let result = correlator.add_event(event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_completion_analysis() {
        let correlator = WorkflowEventCorrelator::new();
        let correlation_id = Uuid::new_v4();
        
        let mut completion_criteria = CompletionCriteria::default();
        completion_criteria.min_event_count = Some(2);
        
        correlator.start_chain(correlation_id, completion_criteria).unwrap();
        
        let event = WorkflowEvent::lifecycle(
            LifecycleEventType::WorkflowCreated,
            "workflow".to_string(),
            correlation_id,
            EventPayload::empty(),
            EventContext::empty(),
        );
        
        correlator.add_event(event).unwrap();
        
        let analysis = correlator.analyze_completion(correlation_id).unwrap();
        assert!(!analysis.is_complete);
        assert_eq!(analysis.completion_reason, CompletionReason::WaitingForEvents);
        assert!(!analysis.missing_requirements.is_empty());
    }
}