# Event Integrity Standard for CIM Workflows

## Overview

This standard defines the mandatory requirements for cryptographic integrity verification of all workflow events in the CIM ecosystem, using Content-Addressed Identifiers (CIDs) and blockchain-inspired chain verification techniques.

## Cryptographic Foundation

### Content-Addressed Identifiers (CIDs)

Every workflow event MUST have a deterministic, content-addressed identifier that enables:
1. **Tamper Detection**: Any modification changes the CID
2. **Content Verification**: Content can be verified against its CID
3. **Deduplication**: Identical content has identical CIDs
4. **Distributed Verification**: Any party can verify integrity independently

### Chain Integrity

Workflow events form cryptographic chains where:
1. Each event references the CID of the previous event
2. Chain integrity can be verified independently
3. Broken chains indicate tampering or corruption
4. Full audit trails are cryptographically verifiable

## Implementation Requirements

### 1. CID Calculation

```rust
use cid::{Cid, Codec};
use multihash::{Code, MultihashDigest};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Trait for objects that can be content-addressed
pub trait ContentAddressable {
    /// Calculate the CID for this object
    fn calculate_cid(&self) -> Cid;
    
    /// Get canonical bytes for CID calculation
    fn canonical_bytes(&self) -> Vec<u8>;
}

impl ContentAddressable for CimWorkflowEvent {
    fn calculate_cid(&self) -> Cid {
        let canonical_bytes = self.canonical_bytes();
        let hash = Code::Sha2_256.digest(&canonical_bytes);
        Cid::new_v1(Codec::DagCbor, hash)
    }
    
    fn canonical_bytes(&self) -> Vec<u8> {
        // Create canonical representation for consistent CID calculation
        let canonical_event = CanonicalWorkflowEvent::from(self);
        
        // Serialize using deterministic encoding
        serde_cbor::to_vec(&canonical_event)
            .expect("Failed to serialize canonical event")
    }
}

/// Canonical representation for CID calculation
/// This excludes the event_cid field to avoid circular dependencies
#[derive(Debug, Serialize, Deserialize)]
struct CanonicalWorkflowEvent {
    /// Message identity (correlation/causation/message IDs)
    identity: MessageIdentity,
    
    /// Workflow instance identifier
    instance_id: WorkflowInstanceId,
    
    /// Source domain
    source_domain: String,
    
    /// Event type string
    event_type: String,
    
    /// Event payload hash
    payload_hash: [u8; 32],
    
    /// Event timestamp (normalized to UTC)
    timestamp: i64, // Unix timestamp for deterministic serialization
    
    /// Previous event CID for chain linking
    previous_cid: Option<Cid>,
}

impl From<&CimWorkflowEvent> for CanonicalWorkflowEvent {
    fn from(event: &CimWorkflowEvent) -> Self {
        // Calculate payload hash
        let payload_bytes = serde_json::to_vec(&event.event)
            .expect("Failed to serialize event payload");
        let payload_hash = Sha256::digest(&payload_bytes).into();
        
        Self {
            identity: event.identity.clone(),
            instance_id: event.instance_id,
            source_domain: event.source_domain.clone(),
            event_type: event.event_type().to_string(),
            payload_hash,
            timestamp: event.timestamp.timestamp(),
            previous_cid: event.previous_cid.clone(),
        }
    }
}
```

### 2. Event Chain Verification

```rust
/// Workflow event chain for integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEventChain {
    /// Workflow instance this chain belongs to
    pub instance_id: WorkflowInstanceId,
    
    /// Ordered list of events in the chain
    pub events: Vec<CimWorkflowEvent>,
    
    /// Chain metadata
    pub metadata: ChainMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetadata {
    /// When the chain was created
    pub created_at: DateTime<Utc>,
    
    /// Last verification timestamp
    pub last_verified_at: Option<DateTime<Utc>>,
    
    /// Chain integrity status
    pub integrity_status: ChainIntegrityStatus,
    
    /// Number of verification attempts
    pub verification_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainIntegrityStatus {
    /// Chain has been verified and is intact
    Verified,
    
    /// Chain has not been verified yet
    Unverified,
    
    /// Chain verification failed - potential tampering
    Corrupted { issue: IntegrityIssue },
    
    /// Chain is incomplete (missing events)
    Incomplete { missing_events: Vec<Cid> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrityIssue {
    /// Event CID doesn't match calculated CID
    CidMismatch { event_index: usize, expected: Cid, actual: Cid },
    
    /// Event chain link is broken
    BrokenChain { event_index: usize, expected_previous: Option<Cid>, actual_previous: Option<Cid> },
    
    /// Event timestamp is invalid
    InvalidTimestamp { event_index: usize, timestamp: DateTime<Utc> },
    
    /// Event correlation is invalid
    InvalidCorrelation { event_index: usize, reason: String },
}

impl WorkflowEventChain {
    /// Create new event chain
    pub fn new(instance_id: WorkflowInstanceId) -> Self {
        Self {
            instance_id,
            events: Vec::new(),
            metadata: ChainMetadata {
                created_at: Utc::now(),
                last_verified_at: None,
                integrity_status: ChainIntegrityStatus::Unverified,
                verification_count: 0,
            },
        }
    }
    
    /// Add event to chain with integrity checks
    pub fn add_event(&mut self, mut event: CimWorkflowEvent) -> Result<(), IntegrityError> {
        // Ensure event belongs to this workflow instance
        if event.instance_id != self.instance_id {
            return Err(IntegrityError::WrongWorkflowInstance {
                expected: self.instance_id,
                actual: event.instance_id,
            });
        }
        
        // Set previous CID for chain linking
        if let Some(last_event) = self.events.last() {
            event.previous_cid = last_event.event_cid.clone();
        }
        
        // Calculate and set CID for the event
        let calculated_cid = event.calculate_cid();
        event.event_cid = Some(calculated_cid);
        
        // Add to chain
        self.events.push(event);
        
        // Mark as unverified since we added new event
        self.metadata.integrity_status = ChainIntegrityStatus::Unverified;
        
        Ok(())
    }
    
    /// Verify complete chain integrity
    pub fn verify_integrity(&mut self) -> Result<ChainIntegrityStatus, IntegrityError> {
        self.metadata.verification_count += 1;
        self.metadata.last_verified_at = Some(Utc::now());
        
        // Check if chain is empty
        if self.events.is_empty() {
            self.metadata.integrity_status = ChainIntegrityStatus::Verified;
            return Ok(ChainIntegrityStatus::Verified);
        }
        
        // Verify each event in the chain
        for (index, event) in self.events.iter().enumerate() {
            // Verify event CID
            if let Some(stored_cid) = &event.event_cid {
                let calculated_cid = event.calculate_cid();
                if *stored_cid != calculated_cid {
                    let issue = IntegrityIssue::CidMismatch {
                        event_index: index,
                        expected: calculated_cid,
                        actual: stored_cid.clone(),
                    };
                    self.metadata.integrity_status = ChainIntegrityStatus::Corrupted { issue };
                    return Ok(self.metadata.integrity_status.clone());
                }
            } else {
                return Err(IntegrityError::MissingCid { event_index: index });
            }
            
            // Verify chain linkage
            if index > 0 {
                let previous_event = &self.events[index - 1];
                let expected_previous_cid = previous_event.event_cid.clone();
                
                if event.previous_cid != expected_previous_cid {
                    let issue = IntegrityIssue::BrokenChain {
                        event_index: index,
                        expected_previous: expected_previous_cid,
                        actual_previous: event.previous_cid.clone(),
                    };
                    self.metadata.integrity_status = ChainIntegrityStatus::Corrupted { issue };
                    return Ok(self.metadata.integrity_status.clone());
                }
            } else {
                // First event should not have previous CID
                if event.previous_cid.is_some() {
                    let issue = IntegrityIssue::BrokenChain {
                        event_index: index,
                        expected_previous: None,
                        actual_previous: event.previous_cid.clone(),
                    };
                    self.metadata.integrity_status = ChainIntegrityStatus::Corrupted { issue };
                    return Ok(self.metadata.integrity_status.clone());
                }
            }
            
            // Verify timestamp ordering
            if index > 0 {
                let previous_event = &self.events[index - 1];
                if event.timestamp < previous_event.timestamp {
                    let issue = IntegrityIssue::InvalidTimestamp {
                        event_index: index,
                        timestamp: event.timestamp,
                    };
                    self.metadata.integrity_status = ChainIntegrityStatus::Corrupted { issue };
                    return Ok(self.metadata.integrity_status.clone());
                }
            }
            
            // Verify correlation consistency
            if index > 0 {
                let previous_event = &self.events[index - 1];
                if event.identity.correlation_id != previous_event.identity.correlation_id {
                    let issue = IntegrityIssue::InvalidCorrelation {
                        event_index: index,
                        reason: "Correlation ID changed within workflow instance".to_string(),
                    };
                    self.metadata.integrity_status = ChainIntegrityStatus::Corrupted { issue };
                    return Ok(self.metadata.integrity_status.clone());
                }
            }
        }
        
        // All checks passed
        self.metadata.integrity_status = ChainIntegrityStatus::Verified;
        Ok(ChainIntegrityStatus::Verified)
    }
    
    /// Get chain summary for auditing
    pub fn get_chain_summary(&self) -> ChainSummary {
        let first_event = self.events.first();
        let last_event = self.events.last();
        
        ChainSummary {
            instance_id: self.instance_id,
            event_count: self.events.len(),
            first_event_timestamp: first_event.map(|e| e.timestamp),
            last_event_timestamp: last_event.map(|e| e.timestamp),
            integrity_status: self.metadata.integrity_status.clone(),
            verification_count: self.metadata.verification_count,
            last_verified_at: self.metadata.last_verified_at,
            chain_hash: self.calculate_chain_hash(),
        }
    }
    
    /// Calculate hash of the entire chain for quick integrity checks
    pub fn calculate_chain_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        for event in &self.events {
            if let Some(cid) = &event.event_cid {
                hasher.update(cid.to_bytes());
            }
        }
        
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSummary {
    pub instance_id: WorkflowInstanceId,
    pub event_count: usize,
    pub first_event_timestamp: Option<DateTime<Utc>>,
    pub last_event_timestamp: Option<DateTime<Utc>>,
    pub integrity_status: ChainIntegrityStatus,
    pub verification_count: u64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub chain_hash: [u8; 32],
}
```

### 3. Integrity Service

```rust
/// Service for managing workflow event integrity
#[async_trait]
pub trait WorkflowIntegrityService: Send + Sync {
    /// Store event with integrity verification
    async fn store_event(&self, event: CimWorkflowEvent) -> Result<Cid, IntegrityError>;
    
    /// Verify event integrity
    async fn verify_event(&self, cid: &Cid) -> Result<bool, IntegrityError>;
    
    /// Get complete event chain for a workflow instance
    async fn get_event_chain(&self, instance_id: WorkflowInstanceId) -> Result<WorkflowEventChain, IntegrityError>;
    
    /// Verify complete chain integrity
    async fn verify_chain_integrity(&self, instance_id: WorkflowInstanceId) -> Result<ChainIntegrityStatus, IntegrityError>;
    
    /// Get chain summary for auditing
    async fn get_chain_summary(&self, instance_id: WorkflowInstanceId) -> Result<ChainSummary, IntegrityError>;
}

/// Default implementation using persistent storage
pub struct DefaultWorkflowIntegrityService {
    event_store: Arc<dyn EventStore>,
    chain_cache: Arc<Mutex<HashMap<WorkflowInstanceId, WorkflowEventChain>>>,
}

impl DefaultWorkflowIntegrityService {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            event_store,
            chain_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Load chain from storage or cache
    async fn load_chain(&self, instance_id: WorkflowInstanceId) -> Result<WorkflowEventChain, IntegrityError> {
        // Check cache first
        {
            let cache = self.chain_cache.lock().await;
            if let Some(chain) = cache.get(&instance_id) {
                return Ok(chain.clone());
            }
        }
        
        // Load from storage
        let events = self.event_store.get_events_by_instance(instance_id).await?;
        let mut chain = WorkflowEventChain::new(instance_id);
        
        for event in events {
            chain.events.push(event);
        }
        
        // Update cache
        {
            let mut cache = self.chain_cache.lock().await;
            cache.insert(instance_id, chain.clone());
        }
        
        Ok(chain)
    }
    
    /// Update chain in cache and storage
    async fn update_chain(&self, chain: WorkflowEventChain) -> Result<(), IntegrityError> {
        // Update cache
        {
            let mut cache = self.chain_cache.lock().await;
            cache.insert(chain.instance_id, chain.clone());
        }
        
        // Persist to storage
        for event in &chain.events {
            self.event_store.store_event(event.clone()).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl WorkflowIntegrityService for DefaultWorkflowIntegrityService {
    async fn store_event(&self, mut event: CimWorkflowEvent) -> Result<Cid, IntegrityError> {
        // Load or create chain
        let mut chain = self.load_chain(event.instance_id).await
            .unwrap_or_else(|_| WorkflowEventChain::new(event.instance_id));
        
        // Add event to chain (this calculates CID and sets previous_cid)
        chain.add_event(event.clone())?;
        
        // Update storage
        self.update_chain(chain).await?;
        
        // Return the calculated CID
        Ok(event.event_cid.unwrap())
    }
    
    async fn verify_event(&self, cid: &Cid) -> Result<bool, IntegrityError> {
        let event = self.event_store.get_event_by_cid(cid).await?;
        let calculated_cid = event.calculate_cid();
        Ok(*cid == calculated_cid)
    }
    
    async fn get_event_chain(&self, instance_id: WorkflowInstanceId) -> Result<WorkflowEventChain, IntegrityError> {
        self.load_chain(instance_id).await
    }
    
    async fn verify_chain_integrity(&self, instance_id: WorkflowInstanceId) -> Result<ChainIntegrityStatus, IntegrityError> {
        let mut chain = self.load_chain(instance_id).await?;
        let status = chain.verify_integrity()?;
        
        // Update cache with verification results
        self.update_chain(chain).await?;
        
        Ok(status)
    }
    
    async fn get_chain_summary(&self, instance_id: WorkflowInstanceId) -> Result<ChainSummary, IntegrityError> {
        let chain = self.load_chain(instance_id).await?;
        Ok(chain.get_chain_summary())
    }
}
```

### 4. Error Types

```rust
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum IntegrityError {
    #[error("Missing CID for event at index {event_index}")]
    MissingCid { event_index: usize },
    
    #[error("CID mismatch - expected: {expected}, actual: {actual}")]
    CidMismatch { expected: Cid, actual: Cid },
    
    #[error("Event belongs to wrong workflow instance - expected: {expected}, actual: {actual}")]
    WrongWorkflowInstance { expected: WorkflowInstanceId, actual: WorkflowInstanceId },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    #[error("Chain verification failed: {reason}")]
    VerificationFailed { reason: String },
}

impl From<serde_json::Error> for IntegrityError {
    fn from(err: serde_json::Error) -> Self {
        IntegrityError::SerializationError { message: err.to_string() }
    }
}

impl From<serde_cbor::Error> for IntegrityError {
    fn from(err: serde_cbor::Error) -> Self {
        IntegrityError::SerializationError { message: err.to_string() }
    }
}
```

## Usage Examples

### 1. Creating Events with Integrity

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let integrity_service = Arc::new(DefaultWorkflowIntegrityService::new(event_store));
    
    // Create workflow started event
    let mut started_event = CimWorkflowEvent::new_root(
        WorkflowInstanceId::new(),
        "document".to_string(),
        WorkflowEventType::WorkflowStarted {
            workflow_id: WorkflowId::new(),
            context: WorkflowContext::new(),
            started_by: Some("user_123".to_string()),
        },
        Some(ActorId::user(Uuid::new_v4())),
    );
    
    // Store with integrity verification
    let cid = integrity_service.store_event(started_event).await?;
    println!("Stored event with CID: {}", cid);
    
    // Verify event integrity
    let is_valid = integrity_service.verify_event(&cid).await?;
    println!("Event integrity: {}", if is_valid { "VALID" } else { "INVALID" });
    
    Ok(())
}
```

### 2. Verifying Chain Integrity

```rust
async fn audit_workflow_integrity(
    integrity_service: &dyn WorkflowIntegrityService,
    instance_id: WorkflowInstanceId,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get chain summary
    let summary = integrity_service.get_chain_summary(instance_id).await?;
    
    println!("Workflow Instance: {}", summary.instance_id.as_uuid());
    println!("Event Count: {}", summary.event_count);
    println!("Integrity Status: {:?}", summary.integrity_status);
    println!("Verification Count: {}", summary.verification_count);
    
    // Verify full chain integrity
    match integrity_service.verify_chain_integrity(instance_id).await? {
        ChainIntegrityStatus::Verified => {
            println!("✅ Chain integrity verified - no tampering detected");
        }
        ChainIntegrityStatus::Corrupted { issue } => {
            println!("❌ Chain integrity compromised: {:?}", issue);
        }
        ChainIntegrityStatus::Incomplete { missing_events } => {
            println!("⚠️  Chain incomplete - missing {} events", missing_events.len());
        }
        ChainIntegrityStatus::Unverified => {
            println!("❓ Chain not yet verified");
        }
    }
    
    Ok(())
}
```

### 3. Cross-Domain Integrity Verification

```rust
async fn verify_cross_domain_workflow_integrity(
    integrity_service: &dyn WorkflowIntegrityService,
    correlation_id: CorrelationId,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find all workflow instances with this correlation ID
    let instances = find_instances_by_correlation(correlation_id).await?;
    
    let mut all_verified = true;
    
    for instance_id in instances {
        let status = integrity_service.verify_chain_integrity(instance_id).await?;
        
        match status {
            ChainIntegrityStatus::Verified => {
                println!("✅ Instance {} verified", instance_id.as_uuid());
            }
            _ => {
                println!("❌ Instance {} failed verification: {:?}", instance_id.as_uuid(), status);
                all_verified = false;
            }
        }
    }
    
    if all_verified {
        println!("✅ Cross-domain workflow integrity verified");
    } else {
        println!("❌ Cross-domain workflow integrity compromised");
    }
    
    Ok(())
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod integrity_tests {
    use super::*;
    
    #[test]
    fn test_cid_calculation_deterministic() {
        let event = create_test_event();
        let cid1 = event.calculate_cid();
        let cid2 = event.calculate_cid();
        
        assert_eq!(cid1, cid2, "CID calculation must be deterministic");
    }
    
    #[test]
    fn test_cid_changes_with_content() {
        let mut event1 = create_test_event();
        let mut event2 = event1.clone();
        
        // Modify event2
        event2.timestamp = event2.timestamp + Duration::seconds(1);
        
        let cid1 = event1.calculate_cid();
        let cid2 = event2.calculate_cid();
        
        assert_ne!(cid1, cid2, "CID must change when content changes");
    }
    
    #[tokio::test]
    async fn test_chain_integrity_verification() {
        let mut chain = WorkflowEventChain::new(WorkflowInstanceId::new());
        
        // Add valid events
        let event1 = create_test_event();
        let event2 = create_test_event();
        
        chain.add_event(event1).unwrap();
        chain.add_event(event2).unwrap();
        
        // Verify integrity
        let status = chain.verify_integrity().unwrap();
        assert_eq!(status, ChainIntegrityStatus::Verified);
    }
    
    #[tokio::test]  
    async fn test_tamper_detection() {
        let mut chain = WorkflowEventChain::new(WorkflowInstanceId::new());
        
        // Add event
        let mut event = create_test_event();
        chain.add_event(event.clone()).unwrap();
        
        // Tamper with the event after adding to chain
        chain.events[0].timestamp = chain.events[0].timestamp + Duration::hours(1);
        
        // Verification should detect tampering
        let status = chain.verify_integrity().unwrap();
        assert!(matches!(status, ChainIntegrityStatus::Corrupted { .. }));
    }
}
```

## Compliance Checklist

- [ ] All workflow events calculate deterministic CIDs
- [ ] Events are linked in cryptographic chains  
- [ ] Chain integrity verification detects tampering
- [ ] Broken chains are properly detected
- [ ] Event storage includes integrity metadata
- [ ] Cross-domain workflows maintain integrity
- [ ] Audit trails are cryptographically verifiable
- [ ] Performance requirements are met (<10ms CID calculation)

This standard ensures that all workflow events in the CIM ecosystem maintain cryptographic integrity and provide tamper-evident audit trails for compliance and security purposes.