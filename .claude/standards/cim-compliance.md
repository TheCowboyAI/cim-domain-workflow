# CIM Compliance Standard for Workflows

## Overview

This standard defines the mandatory requirements for all workflow operations to be CIM-compliant, ensuring proper event correlation, cryptographic integrity, and mathematical consistency across the entire CIM ecosystem.

## Mandatory Requirements

### 1. Event Correlation and Causation

**REQUIREMENT**: Every workflow event MUST carry mandatory correlation and causation IDs.

#### Implementation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimWorkflowEvent {
    /// MANDATORY: Message identity with correlation/causation tracking
    pub identity: MessageIdentity,
    
    /// Workflow instance identifier
    pub instance_id: WorkflowInstanceId,
    
    /// Domain that generated this event
    pub source_domain: String,
    
    /// The specific workflow event
    pub event: WorkflowEventType,
    
    /// Event timestamp (UTC)
    pub timestamp: DateTime<Utc>,
    
    /// MANDATORY: CID for cryptographic integrity
    pub event_cid: Option<Cid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageIdentity {
    /// Unique message identifier
    pub message_id: MessageId,
    
    /// MANDATORY: Groups related workflow operations
    pub correlation_id: CorrelationId,
    
    /// MANDATORY: Tracks causal relationships
    pub causation_id: CausationId,
}
```

#### Validation Rules

```rust
impl CimWorkflowEvent {
    /// Validate CIM compliance
    pub fn validate_cim_compliance(&self) -> Result<(), CimComplianceError> {
        // Rule 1: Must have correlation ID
        if self.identity.correlation_id.is_empty() {
            return Err(CimComplianceError::MissingCorrelationId);
        }
        
        // Rule 2: Must have causation ID
        if self.identity.causation_id.is_empty() {
            return Err(CimComplianceError::MissingCausationId);
        }
        
        // Rule 3: Must have message ID
        if self.identity.message_id.is_empty() {
            return Err(CimComplianceError::MissingMessageId);
        }
        
        // Rule 4: Source domain must be valid
        if self.source_domain.is_empty() || !is_valid_domain(&self.source_domain) {
            return Err(CimComplianceError::InvalidSourceDomain { 
                domain: self.source_domain.clone() 
            });
        }
        
        // Rule 5: Event must have valid timestamp
        if self.timestamp > Utc::now() + Duration::minutes(5) {
            return Err(CimComplianceError::FutureTimestamp { 
                timestamp: self.timestamp 
            });
        }
        
        Ok(())
    }
    
    /// Check if this is a root event (starts correlation chain)
    pub fn is_root_event(&self) -> bool {
        self.identity.correlation_id == CorrelationId::from(self.identity.message_id)
    }
    
    /// Check if this event is properly caused by another event
    pub fn is_caused_by(&self, parent_event: &CimWorkflowEvent) -> bool {
        self.identity.causation_id == CausationId::from(parent_event.identity.message_id)
            && self.identity.correlation_id == parent_event.identity.correlation_id
    }
}
```

### 2. CID Integrity

**REQUIREMENT**: All workflow events MUST support content-addressed integrity verification.

#### Implementation

```rust
impl CimWorkflowEvent {
    /// Calculate content-addressed identifier for this event
    pub fn calculate_cid(&self) -> Cid {
        // Create canonical representation for CID calculation
        let canonical_event = CanonicalWorkflowEvent {
            identity: self.identity.clone(),
            instance_id: self.instance_id,
            source_domain: self.source_domain.clone(),
            event_type: self.event_type(),
            payload_hash: self.calculate_payload_hash(),
            timestamp: self.timestamp,
        };
        
        // Serialize to canonical form
        let canonical_bytes = canonical_event.to_canonical_bytes();
        
        // Calculate CID using SHA-256
        Cid::new_v1(0x70, multihash::Code::Sha2_256.digest(&canonical_bytes))
    }
    
    /// Verify event integrity using CID
    pub fn verify_integrity(&self) -> Result<bool, IntegrityError> {
        if let Some(stored_cid) = &self.event_cid {
            let calculated_cid = self.calculate_cid();
            Ok(stored_cid == &calculated_cid)
        } else {
            Err(IntegrityError::MissingCid)
        }
    }
    
    /// Set CID for this event (call before publishing)
    pub fn set_cid(&mut self) {
        self.event_cid = Some(self.calculate_cid());
    }
    
    fn calculate_payload_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let payload_bytes = serde_json::to_vec(&self.event).unwrap();
        let hash = Sha256::digest(&payload_bytes);
        format!("{:x}", hash)
    }
}

#[derive(Debug, Serialize)]
struct CanonicalWorkflowEvent {
    identity: MessageIdentity,
    instance_id: WorkflowInstanceId,
    source_domain: String,
    event_type: String,
    payload_hash: String,
    timestamp: DateTime<Utc>,
}

impl CanonicalWorkflowEvent {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        // Use deterministic serialization (sorted keys, no whitespace)
        serde_json::to_vec(self).unwrap()
    }
}
```

### 3. NATS Subject Standards

**REQUIREMENT**: All workflow events MUST follow standardized NATS subject patterns.

#### Subject Pattern Specification

```
events.workflow.{domain}.{event_type}.{instance_id}
```

Where:
- `domain`: Source domain (document, person, organization, etc.)
- `event_type`: Workflow event type (started, step_completed, failed, etc.)
- `instance_id`: Workflow instance UUID

#### Implementation

```rust
impl CimWorkflowEvent {
    /// Generate standardized NATS subject for this event
    pub fn to_nats_subject(&self) -> String {
        format!(
            "events.workflow.{}.{}.{}",
            self.source_domain,
            self.event_type(),
            self.instance_id.as_uuid()
        )
    }
    
    /// Get event type string for subject generation
    pub fn event_type(&self) -> &'static str {
        match &self.event {
            WorkflowEventType::WorkflowStarted { .. } => "started",
            WorkflowEventType::WorkflowCompleted { .. } => "completed",
            WorkflowEventType::WorkflowFailed { .. } => "failed",
            WorkflowEventType::WorkflowPaused { .. } => "paused",
            WorkflowEventType::WorkflowResumed { .. } => "resumed",
            WorkflowEventType::WorkflowCancelled { .. } => "cancelled",
            WorkflowEventType::StepStarted { .. } => "step_started",
            WorkflowEventType::StepCompleted { .. } => "step_completed",
            WorkflowEventType::StepFailed { .. } => "step_failed",
            WorkflowEventType::CrossDomainTransition { .. } => "cross_domain",
            WorkflowEventType::DomainSpecific { event_type, .. } => event_type,
        }
    }
}

/// Subject pattern utilities
pub struct WorkflowSubjects;

impl WorkflowSubjects {
    /// Subscribe to all events for a specific workflow instance
    pub fn instance_subscription(instance_id: WorkflowInstanceId) -> String {
        format!("events.workflow.*.*.{}", instance_id.as_uuid())
    }
    
    /// Subscribe to all events for a specific domain
    pub fn domain_subscription(domain: &str) -> String {
        format!("events.workflow.{}.*.*", domain)
    }
    
    /// Subscribe to specific event type across all domains
    pub fn event_type_subscription(event_type: &str) -> String {
        format!("events.workflow.*.{}.*", event_type)
    }
    
    /// Subscribe to all cross-domain workflow events
    pub fn cross_domain_subscription() -> String {
        "events.workflow.*.cross_domain.*".to_string()
    }
    
    /// Subscribe to all workflow events (use carefully - high volume)
    pub fn all_workflow_events() -> String {
        "events.workflow.*.*.*".to_string()
    }
}
```

### 4. Context Extension Standards

**REQUIREMENT**: Domain-specific data MUST use the extensible context framework.

#### Context Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Universal variables accessible to all workflows
    pub variables: HashMap<String, serde_json::Value>,
    
    /// Domain-specific extensions (REQUIRED for domain data)
    pub extensions: HashMap<String, DomainExtension>,
    
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainExtension {
    /// Domain identifier (must match domain extension)
    pub domain: String,
    
    /// Domain-specific data (JSON value)
    pub data: serde_json::Value,
    
    /// Extension version for compatibility
    pub version: String,
    
    /// When extension was created
    pub created_at: DateTime<Utc>,
    
    /// When extension was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Correlation ID for this workflow execution
    pub correlation_id: CorrelationId,
    
    /// Optional trace ID for distributed tracing
    pub trace_id: Option<String>,
    
    /// Execution-specific context
    pub execution_context: HashMap<String, serde_json::Value>,
}
```

#### Validation Rules

```rust
impl WorkflowContext {
    /// Validate context follows CIM standards
    pub fn validate_cim_compliance(&self) -> Result<(), CimComplianceError> {
        // Rule 1: Must have correlation ID in metadata
        if self.metadata.correlation_id.is_empty() {
            return Err(CimComplianceError::MissingCorrelationId);
        }
        
        // Rule 2: Domain extensions must have valid structure
        for (domain_name, extension) in &self.extensions {
            if extension.domain != *domain_name {
                return Err(CimComplianceError::DomainExtensionMismatch {
                    declared: domain_name.clone(),
                    actual: extension.domain.clone(),
                });
            }
            
            if extension.version.is_empty() {
                return Err(CimComplianceError::MissingExtensionVersion {
                    domain: domain_name.clone(),
                });
            }
            
            if !is_valid_domain(domain_name) {
                return Err(CimComplianceError::InvalidDomainName {
                    domain: domain_name.clone(),
                });
            }
        }
        
        // Rule 3: Universal variables must not conflict with extensions
        for key in self.variables.keys() {
            if key.starts_with("_domain_") && !self.extensions.contains_key(&key[8..]) {
                return Err(CimComplianceError::OrphanedDomainVariable {
                    variable: key.clone(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Add domain extension with validation
    pub fn add_domain_extension_validated(
        &mut self,
        domain: String,
        data: serde_json::Value,
        version: String,
    ) -> Result<(), CimComplianceError> {
        // Validate domain name
        if !is_valid_domain(&domain) {
            return Err(CimComplianceError::InvalidDomainName { domain });
        }
        
        // Validate version format
        if !is_valid_version(&version) {
            return Err(CimComplianceError::InvalidVersionFormat { version });
        }
        
        // Add extension
        let now = Utc::now();
        self.extensions.insert(
            domain.clone(),
            DomainExtension {
                domain: domain.clone(),
                data,
                version,
                created_at: now,
                updated_at: now,
            }
        );
        
        Ok(())
    }
}

fn is_valid_domain(domain: &str) -> bool {
    // Domain names must be lowercase alphanumeric with underscores
    domain.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && domain.len() > 0
        && domain.len() <= 50
        && !domain.starts_with('_')
        && !domain.ends_with('_')
}

fn is_valid_version(version: &str) -> bool {
    // Version must follow semantic versioning (simplified)
    version.split('.').all(|part| part.parse::<u32>().is_ok())
        && version.split('.').count() >= 2
        && version.len() <= 20
}
```

### 5. Error Handling Standards

**REQUIREMENT**: All workflow errors MUST be CIM-compliant and provide proper context.

#### Error Types

```rust
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum CimComplianceError {
    #[error("Missing correlation ID")]
    MissingCorrelationId,
    
    #[error("Missing causation ID")]
    MissingCausationId,
    
    #[error("Missing message ID")]
    MissingMessageId,
    
    #[error("Invalid source domain: {domain}")]
    InvalidSourceDomain { domain: String },
    
    #[error("Future timestamp not allowed: {timestamp}")]
    FutureTimestamp { timestamp: DateTime<Utc> },
    
    #[error("Domain extension mismatch - declared: {declared}, actual: {actual}")]
    DomainExtensionMismatch { declared: String, actual: String },
    
    #[error("Missing extension version for domain: {domain}")]
    MissingExtensionVersion { domain: String },
    
    #[error("Invalid domain name: {domain}")]
    InvalidDomainName { domain: String },
    
    #[error("Invalid version format: {version}")]
    InvalidVersionFormat { version: String },
    
    #[error("Orphaned domain variable: {variable}")]
    OrphanedDomainVariable { variable: String },
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum WorkflowError {
    #[error("CIM compliance error: {0}")]
    CimCompliance(#[from] CimComplianceError),
    
    #[error("Workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: String },
    
    #[error("Invalid transition from {from} to {to}: {reason}")]
    InvalidTransition { from: String, to: String, reason: String },
    
    #[error("Unsupported step type: {step_type}")]
    UnsupportedStepType { step_type: String },
    
    #[error("Missing domain extension: {domain}")]
    MissingDomainExtension { domain: String },
    
    #[error("Missing context data: {field}")]
    MissingContextData { field: String },
    
    #[error("Cross-domain transition failed: {from} -> {to}")]
    CrossDomainTransitionFailed { from: String, to: String },
    
    #[error("Event integrity verification failed")]
    IntegrityVerificationFailed,
    
    #[error("Workflow execution timeout after {timeout_seconds} seconds")]
    ExecutionTimeout { timeout_seconds: u64 },
}
```

### 6. Testing Standards

**REQUIREMENT**: All CIM-compliant workflow components MUST have comprehensive tests.

#### Test Categories

```rust
#[cfg(test)]
mod cim_compliance_tests {
    use super::*;
    
    /// Test that all workflow events are CIM-compliant
    #[tokio::test]
    async fn test_workflow_event_cim_compliance() {
        let event = CimWorkflowEvent::new_root(
            WorkflowInstanceId::new(),
            "document".to_string(),
            WorkflowEventType::WorkflowStarted {
                workflow_id: WorkflowId::new(),
                context: WorkflowContext::new(),
                started_by: Some("test_user".to_string()),
            },
            Some(ActorId::user(Uuid::new_v4())),
        );
        
        // Must pass CIM compliance validation
        assert!(event.validate_cim_compliance().is_ok());
        
        // Must have proper correlation/causation
        assert!(!event.identity.correlation_id.is_empty());
        assert!(!event.identity.causation_id.is_empty());
        assert!(!event.identity.message_id.is_empty());
        
        // Must support CID integrity
        let mut event_with_cid = event.clone();
        event_with_cid.set_cid();
        assert!(event_with_cid.event_cid.is_some());
        assert!(event_with_cid.verify_integrity().unwrap());
    }
    
    /// Test cross-domain event correlation
    #[tokio::test]
    async fn test_cross_domain_correlation() {
        let root_event = CimWorkflowEvent::new_root(
            WorkflowInstanceId::new(),
            "document".to_string(),
            WorkflowEventType::WorkflowStarted { /* ... */ },
            Some(ActorId::system("test")),
        );
        
        let caused_event = CimWorkflowEvent::new_caused_by(
            root_event.instance_id,
            "person".to_string(),
            WorkflowEventType::StepStarted { /* ... */ },
            &root_event.identity,
            Some(ActorId::system("test")),
        );
        
        // Must have same correlation ID
        assert_eq!(root_event.identity.correlation_id, caused_event.identity.correlation_id);
        
        // Must have proper causation relationship
        assert!(caused_event.is_caused_by(&root_event));
        
        // Root event must be identified correctly
        assert!(root_event.is_root_event());
        assert!(!caused_event.is_root_event());
    }
    
    /// Test context extension compliance
    #[test]
    fn test_context_extension_compliance() {
        let mut context = WorkflowContext::new();
        
        // Must validate successfully with proper extensions
        context.add_domain_extension_validated(
            "document".to_string(),
            serde_json::json!({"document_id": "test_123"}),
            "1.0.0".to_string(),
        ).unwrap();
        
        assert!(context.validate_cim_compliance().is_ok());
        
        // Must reject invalid domain names
        assert!(context.add_domain_extension_validated(
            "Invalid-Domain".to_string(),
            serde_json::json!({}),
            "1.0.0".to_string(),
        ).is_err());
    }
    
    /// Test NATS subject pattern compliance
    #[test]
    fn test_nats_subject_patterns() {
        let event = create_test_event();
        let subject = event.to_nats_subject();
        
        // Must follow standard pattern
        assert!(subject.starts_with("events.workflow."));
        assert!(subject.contains(&event.source_domain));
        assert!(subject.contains(&event.event_type()));
        assert!(subject.contains(&event.instance_id.as_uuid().to_string()));
        
        // Must not contain invalid characters
        assert!(!subject.contains(" "));
        assert!(!subject.contains("\n"));
        assert!(!subject.contains("\t"));
    }
}
```

## Compliance Checklist

### For Workflow Events
- [ ] Event has mandatory correlation ID
- [ ] Event has mandatory causation ID  
- [ ] Event has mandatory message ID
- [ ] Event has valid source domain
- [ ] Event timestamp is not in the future
- [ ] Event supports CID integrity calculation
- [ ] Event follows NATS subject patterns
- [ ] Event validates successfully against CIM compliance rules

### For Domain Extensions
- [ ] Extension uses valid domain name
- [ ] Extension has version information
- [ ] Extension data is properly structured
- [ ] Extension follows context standards
- [ ] Extension validates successfully

### For Cross-Domain Workflows
- [ ] Events maintain correlation across domains
- [ ] Context transformations preserve essential data
- [ ] Rollback strategies are implemented
- [ ] Error handling follows CIM standards
- [ ] Event chains are verifiable

### For Testing
- [ ] CIM compliance tests exist and pass
- [ ] Cross-domain correlation is tested
- [ ] Context extension validation is tested
- [ ] NATS subject patterns are tested
- [ ] Error handling paths are tested

This standard ensures that all workflow operations in the CIM ecosystem follow consistent patterns and maintain the mathematical rigor required for Category Theory-based composition.