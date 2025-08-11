//! Template Library for Common Domain Workflows
//!
//! This module provides a comprehensive library of pre-built workflow templates
//! that domains can use for common patterns like approval workflows, review processes,
//! document management, user onboarding, and cross-domain coordination.

use std::collections::HashMap;
//use serde::{Deserialize, Serialize};
//use uuid::Uuid;

use crate::composition::{
    WorkflowTemplate, TemplateId, TemplateVersion, TemplateParameter, TemplateStep,
    TemplateStepType, ParameterType, TemplateMetadata, TemplateExample,
};

/// Standard template library providing common workflow patterns
pub struct StandardTemplateLibrary {
    /// Collection of built-in templates
    templates: HashMap<String, WorkflowTemplate>,
}

impl StandardTemplateLibrary {
    /// Create a new standard template library with all built-in templates
    pub fn new() -> Self {
        let mut library = Self {
            templates: HashMap::new(),
        };
        
        // Register all standard templates
        library.register_approval_templates();
        library.register_review_templates();
        library.register_document_templates();
        library.register_user_templates();
        library.register_coordination_templates();
        library.register_notification_templates();
        library.register_security_templates();
        
        library
    }
    
    /// Get a template by its full ID
    pub fn get_template(&self, template_key: &str) -> Option<&WorkflowTemplate> {
        self.templates.get(template_key)
    }
    
    /// Get all templates in a specific category
    pub fn get_templates_by_category(&self, category: &str) -> Vec<&WorkflowTemplate> {
        self.templates.values()
            .filter(|t| t.metadata.category == category)
            .collect()
    }
    
    /// Get all templates for a specific domain
    pub fn get_templates_by_domain(&self, domain: &str) -> Vec<&WorkflowTemplate> {
        self.templates.values()
            .filter(|t| t.target_domains.contains(&domain.to_string()))
            .collect()
    }
    
    /// List all available templates
    pub fn list_templates(&self) -> Vec<&WorkflowTemplate> {
        self.templates.values().collect()
    }
    
    /// Register approval workflow templates
    fn register_approval_templates(&mut self) {
        // Single Approval Template
        let single_approval = self.create_single_approval_template();
        self.templates.insert(self.template_key(&single_approval.id), single_approval);
        
        // Multi-level Approval Template
        let multi_approval = self.create_multi_level_approval_template();
        self.templates.insert(self.template_key(&multi_approval.id), multi_approval);
        
        // Parallel Approval Template
        let parallel_approval = self.create_parallel_approval_template();
        self.templates.insert(self.template_key(&parallel_approval.id), parallel_approval);
        
        // Conditional Approval Template
        let conditional_approval = self.create_conditional_approval_template();
        self.templates.insert(self.template_key(&conditional_approval.id), conditional_approval);
    }
    
    /// Register review workflow templates
    fn register_review_templates(&mut self) {
        // Peer Review Template
        let peer_review = self.create_peer_review_template();
        self.templates.insert(self.template_key(&peer_review.id), peer_review);
        
        // Quality Assurance Review Template
        let qa_review = self.create_qa_review_template();
        self.templates.insert(self.template_key(&qa_review.id), qa_review);
        
        // Security Review Template
        let security_review = self.create_security_review_template();
        self.templates.insert(self.template_key(&security_review.id), security_review);
    }
    
    /// Register document management templates
    fn register_document_templates(&mut self) {
        // Document Publication Template
        let doc_publication = self.create_document_publication_template();
        self.templates.insert(self.template_key(&doc_publication.id), doc_publication);
        
        // Document Archival Template
        let doc_archival = self.create_document_archival_template();
        self.templates.insert(self.template_key(&doc_archival.id), doc_archival);
        
        // Document Collaboration Template
        let doc_collaboration = self.create_document_collaboration_template();
        self.templates.insert(self.template_key(&doc_collaboration.id), doc_collaboration);
    }
    
    /// Register user management templates
    fn register_user_templates(&mut self) {
        // User Onboarding Template
        let user_onboarding = self.create_user_onboarding_template();
        self.templates.insert(self.template_key(&user_onboarding.id), user_onboarding);
        
        // User Offboarding Template
        let user_offboarding = self.create_user_offboarding_template();
        self.templates.insert(self.template_key(&user_offboarding.id), user_offboarding);
        
        // Access Request Template
        let access_request = self.create_access_request_template();
        self.templates.insert(self.template_key(&access_request.id), access_request);
    }
    
    /// Register cross-domain coordination templates
    fn register_coordination_templates(&mut self) {
        // Cross-Domain Transaction Template
        let cross_transaction = self.create_cross_domain_transaction_template();
        self.templates.insert(self.template_key(&cross_transaction.id), cross_transaction);
        
        // Event Synchronization Template
        let event_sync = self.create_event_synchronization_template();
        self.templates.insert(self.template_key(&event_sync.id), event_sync);
        
        // Distributed Process Template
        let distributed_process = self.create_distributed_process_template();
        self.templates.insert(self.template_key(&distributed_process.id), distributed_process);
    }
    
    /// Register notification templates
    fn register_notification_templates(&mut self) {
        // Escalation Notification Template
        let escalation_notification = self.create_escalation_notification_template();
        self.templates.insert(self.template_key(&escalation_notification.id), escalation_notification);
        
        // Broadcast Notification Template
        let broadcast_notification = self.create_broadcast_notification_template();
        self.templates.insert(self.template_key(&broadcast_notification.id), broadcast_notification);
    }
    
    /// Register security workflow templates
    fn register_security_templates(&mut self) {
        // Incident Response Template
        let incident_response = self.create_incident_response_template();
        self.templates.insert(self.template_key(&incident_response.id), incident_response);
        
        // Compliance Audit Template
        let compliance_audit = self.create_compliance_audit_template();
        self.templates.insert(self.template_key(&compliance_audit.id), compliance_audit);
    }
    
    /// Generate a template key from TemplateId
    fn template_key(&self, id: &TemplateId) -> String {
        id.to_string()
    }
    
    // Template Creation Methods
    
    /// Create single approval workflow template
    fn create_single_approval_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "approval".to_string(),
                "single-approval".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Single Approval Workflow".to_string(),
            description: "Simple single-step approval workflow for basic approvals".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["approval".to_string(), "document".to_string(), "user".to_string()],
            parameters: vec![
                (
                    "item_id".to_string(),
                    TemplateParameter {
                        name: "item_id".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the item requiring approval".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "approver_id".to_string(),
                    TemplateParameter {
                        name: "approver_id".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the designated approver".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "deadline_hours".to_string(),
                    TemplateParameter {
                        name: "deadline_hours".to_string(),
                        param_type: ParameterType::Integer,
                        description: "Hours until approval deadline".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(48)),
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "request_approval".to_string(),
                    name_template: "Request Approval".to_string(),
                    description_template: "Send approval request to {approver_id} for {item_id}".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec![],
                    configuration: vec![
                        ("notification_type".to_string(), serde_json::json!("approval_request")),
                        ("timeout_hours".to_string(), serde_json::json!("{deadline_hours}")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "await_decision".to_string(),
                    name_template: "Await Approval Decision".to_string(),
                    description_template: "Wait for approval decision from {approver_id}".to_string(),
                    step_type: TemplateStepType::Manual,
                    dependencies: vec!["request_approval".to_string()],
                    configuration: vec![
                        ("allowed_outcomes".to_string(), serde_json::json!(["approved", "rejected"])),
                        ("escalation_enabled".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "process_decision".to_string(),
                    name_template: "Process Approval Decision".to_string(),
                    description_template: "Handle the approval decision and notify stakeholders".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["await_decision".to_string()],
                    configuration: vec![
                        ("notify_requester".to_string(), serde_json::json!(true)),
                        ("update_item_status".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
            ],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "CIM Workflow System".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec!["approval".to_string(), "single".to_string(), "basic".to_string()],
                category: "Approval Workflows".to_string(),
                documentation_url: Some("https://docs.cim.ai/workflows/approval/single".to_string()),
                examples: vec![
                    TemplateExample {
                        name: "Document Approval".to_string(),
                        description: "Simple document approval example".to_string(),
                        parameters: vec![
                            ("item_id".to_string(), serde_json::json!("DOC-12345")),
                            ("approver_id".to_string(), serde_json::json!("user-456")),
                            ("deadline_hours".to_string(), serde_json::json!(24))
                        ].into_iter().collect(),
                        expected_outcome: "Document approved or rejected within deadline".to_string(),
                    }
                ],
            },
            validation_rules: vec![],
        }
    }
    
    /// Create multi-level approval workflow template
    fn create_multi_level_approval_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "approval".to_string(),
                "multi-level-approval".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Multi-Level Approval Workflow".to_string(),
            description: "Sequential multi-level approval workflow with escalation".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["approval".to_string(), "document".to_string(), "user".to_string()],
            parameters: vec![
                (
                    "item_id".to_string(),
                    TemplateParameter {
                        name: "item_id".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the item requiring approval".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "approval_levels".to_string(),
                    TemplateParameter {
                        name: "approval_levels".to_string(),
                        param_type: ParameterType::Array(Box::new(ParameterType::Object("{}".to_string()))),
                        description: "Array of approval levels with approver IDs".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "level_timeout_hours".to_string(),
                    TemplateParameter {
                        name: "level_timeout_hours".to_string(),
                        param_type: ParameterType::Integer,
                        description: "Hours timeout for each approval level".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(24)),
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "initiate_approval_chain".to_string(),
                    name_template: "Initiate Approval Chain".to_string(),
                    description_template: "Start multi-level approval process for {item_id}".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec![],
                    configuration: vec![
                        ("chain_type".to_string(), serde_json::json!("sequential")),
                        ("failure_policy".to_string(), serde_json::json!("stop_on_reject")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "level_approval".to_string(),
                    name_template: "Level {level_number} Approval".to_string(),
                    description_template: "Approval at level {level_number}".to_string(),
                    step_type: TemplateStepType::Sequential,
                    dependencies: vec!["initiate_approval_chain".to_string()],
                    configuration: vec![
                        ("loop_variable".to_string(), serde_json::json!("approval_levels")),
                        ("timeout_hours".to_string(), serde_json::json!("{level_timeout_hours}")),
                        ("escalation_enabled".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "finalize_approval".to_string(),
                    name_template: "Finalize Approval Process".to_string(),
                    description_template: "Complete multi-level approval and notify stakeholders".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["level_approval".to_string()],
                    configuration: vec![
                        ("final_notification".to_string(), serde_json::json!(true)),
                        ("update_permissions".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
            ],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "CIM Workflow System".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec!["approval".to_string(), "multi-level".to_string(), "sequential".to_string()],
                category: "Approval Workflows".to_string(),
                documentation_url: Some("https://docs.cim.ai/workflows/approval/multi-level".to_string()),
                examples: vec![
                    TemplateExample {
                        name: "Expense Approval".to_string(),
                        description: "Multi-level expense approval example".to_string(),
                        parameters: vec![
                            ("item_id".to_string(), serde_json::json!("EXPENSE-789")),
                            ("approval_levels".to_string(), serde_json::json!([
                                {"level": 1, "approver_id": "supervisor-123", "title": "Supervisor"},
                                {"level": 2, "approver_id": "manager-456", "title": "Manager"},
                                {"level": 3, "approver_id": "director-789", "title": "Director"}
                            ])),
                            ("level_timeout_hours".to_string(), serde_json::json!(48))
                        ].into_iter().collect(),
                        expected_outcome: "Expense approved through all levels or rejected at any level".to_string(),
                    }
                ],
            },
            validation_rules: vec![],
        }
    }
    
    /// Create parallel approval workflow template
    fn create_parallel_approval_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "approval".to_string(),
                "parallel-approval".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Parallel Approval Workflow".to_string(),
            description: "Parallel approval workflow where multiple approvers review simultaneously".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["approval".to_string(), "document".to_string(), "user".to_string()],
            parameters: vec![
                (
                    "item_id".to_string(),
                    TemplateParameter {
                        name: "item_id".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the item requiring approval".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "approvers".to_string(),
                    TemplateParameter {
                        name: "approvers".to_string(),
                        param_type: ParameterType::Array(Box::new(ParameterType::String)),
                        description: "List of approver IDs".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "approval_threshold".to_string(),
                    TemplateParameter {
                        name: "approval_threshold".to_string(),
                        param_type: ParameterType::Integer,
                        description: "Number of approvals required (consensus type)".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!("majority")),
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "broadcast_approval_request".to_string(),
                    name_template: "Broadcast Approval Request".to_string(),
                    description_template: "Send approval request to all approvers simultaneously".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec![],
                    configuration: vec![
                        ("broadcast_type".to_string(), serde_json::json!("parallel")),
                        ("notification_method".to_string(), serde_json::json!("multi_channel")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "collect_approvals".to_string(),
                    name_template: "Collect Parallel Approvals".to_string(),
                    description_template: "Wait for and collect approval decisions".to_string(),
                    step_type: TemplateStepType::Parallel,
                    dependencies: vec!["broadcast_approval_request".to_string()],
                    configuration: vec![
                        ("collection_strategy".to_string(), serde_json::json!("threshold_based")),
                        ("threshold".to_string(), serde_json::json!("{approval_threshold}")),
                        ("timeout_hours".to_string(), serde_json::json!(72)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "evaluate_consensus".to_string(),
                    name_template: "Evaluate Approval Consensus".to_string(),
                    description_template: "Determine final approval based on collected decisions".to_string(),
                    step_type: TemplateStepType::Conditional,
                    dependencies: vec!["collect_approvals".to_string()],
                    configuration: vec![
                        ("consensus_algorithm".to_string(), serde_json::json!("threshold")),
                        ("tie_breaker_policy".to_string(), serde_json::json!("reject")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
            ],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "CIM Workflow System".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec!["approval".to_string(), "parallel".to_string(), "consensus".to_string()],
                category: "Approval Workflows".to_string(),
                documentation_url: Some("https://docs.cim.ai/workflows/approval/parallel".to_string()),
                examples: vec![
                    TemplateExample {
                        name: "Proposal Approval".to_string(),
                        description: "Parallel proposal approval example".to_string(),
                        parameters: vec![
                            ("item_id".to_string(), serde_json::json!("PROPOSAL-456")),
                            ("approvers".to_string(), serde_json::json!(["reviewer-1", "reviewer-2", "reviewer-3", "reviewer-4"])),
                            ("approval_threshold".to_string(), serde_json::json!(3))
                        ].into_iter().collect(),
                        expected_outcome: "Proposal approved if threshold met, rejected otherwise".to_string(),
                    }
                ],
            },
            validation_rules: vec![],
        }
    }
    
    /// Create conditional approval workflow template
    fn create_conditional_approval_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "approval".to_string(),
                "conditional-approval".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Conditional Approval Workflow".to_string(),
            description: "Approval workflow with dynamic routing based on conditions".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["approval".to_string(), "document".to_string(), "user".to_string()],
            parameters: vec![
                (
                    "item_id".to_string(),
                    TemplateParameter {
                        name: "item_id".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the item requiring approval".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "approval_rules".to_string(),
                    TemplateParameter {
                        name: "approval_rules".to_string(),
                        param_type: ParameterType::Array(Box::new(ParameterType::Object("{}".to_string()))),
                        description: "Conditional rules defining approval paths".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "context_data".to_string(),
                    TemplateParameter {
                        name: "context_data".to_string(),
                        param_type: ParameterType::Object("{}".to_string()),
                        description: "Context data for rule evaluation".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "evaluate_conditions".to_string(),
                    name_template: "Evaluate Approval Conditions".to_string(),
                    description_template: "Determine appropriate approval path based on rules".to_string(),
                    step_type: TemplateStepType::Conditional,
                    dependencies: vec![],
                    configuration: vec![
                        ("rule_engine".to_string(), serde_json::json!("conditional")),
                        ("evaluation_context".to_string(), serde_json::json!("{context_data}")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "route_approval".to_string(),
                    name_template: "Route to Appropriate Approver".to_string(),
                    description_template: "Route approval request based on evaluated conditions".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["evaluate_conditions".to_string()],
                    configuration: vec![
                        ("routing_strategy".to_string(), serde_json::json!("dynamic")),
                        ("fallback_approver".to_string(), serde_json::json!("default_approver")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "process_conditional_decision".to_string(),
                    name_template: "Process Conditional Decision".to_string(),
                    description_template: "Handle approval decision according to matched rule".to_string(),
                    step_type: TemplateStepType::Conditional,
                    dependencies: vec!["route_approval".to_string()],
                    configuration: vec![
                        ("decision_processing".to_string(), serde_json::json!("rule_based")),
                        ("post_processing_actions".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
            ],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "CIM Workflow System".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec!["approval".to_string(), "conditional".to_string(), "dynamic".to_string()],
                category: "Approval Workflows".to_string(),
                documentation_url: Some("https://docs.cim.ai/workflows/approval/conditional".to_string()),
                examples: vec![
                    TemplateExample {
                        name: "Conditional Request Approval".to_string(),
                        description: "Amount-based conditional approval routing".to_string(),
                        parameters: vec![
                            ("item_id".to_string(), serde_json::json!("REQUEST-789")),
                            ("approval_rules".to_string(), serde_json::json!([
                                {"condition": "amount < 1000", "approver": "supervisor"},
                                {"condition": "amount >= 1000 && amount < 10000", "approver": "manager"},
                                {"condition": "amount >= 10000", "approver": "director"}
                            ])),
                            ("context_data".to_string(), serde_json::json!({"amount": 5000, "department": "engineering"}))
                        ].into_iter().collect(),
                        expected_outcome: "Request routed to manager based on amount condition".to_string(),
                    }
                ],
            },
            validation_rules: vec![],
        }
    }
    
    // Additional template creation methods for other categories...
    // (For brevity, I'll provide a few more key ones)
    
    /// Create peer review workflow template
    fn create_peer_review_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "review".to_string(),
                "peer-review".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Peer Review Workflow".to_string(),
            description: "Collaborative peer review process with feedback collection".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["review".to_string(), "document".to_string(), "user".to_string()],
            parameters: vec![
                (
                    "item_id".to_string(),
                    TemplateParameter {
                        name: "item_id".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the item being reviewed".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "reviewers".to_string(),
                    TemplateParameter {
                        name: "reviewers".to_string(),
                        param_type: ParameterType::Array(Box::new(ParameterType::String)),
                        description: "List of peer reviewer IDs".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "review_rounds".to_string(),
                    TemplateParameter {
                        name: "review_rounds".to_string(),
                        param_type: ParameterType::Integer,
                        description: "Number of review rounds".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(1)),
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "assign_reviewers".to_string(),
                    name_template: "Assign Peer Reviewers".to_string(),
                    description_template: "Assign peer reviewers to {item_id}".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec![],
                    configuration: vec![
                        ("assignment_strategy".to_string(), serde_json::json!("balanced")),
                        ("conflict_detection".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "conduct_reviews".to_string(),
                    name_template: "Conduct Peer Reviews".to_string(),
                    description_template: "Collect feedback from peer reviewers".to_string(),
                    step_type: TemplateStepType::Sequential,
                    dependencies: vec!["assign_reviewers".to_string()],
                    configuration: vec![
                        ("parallel_reviews".to_string(), serde_json::json!(true)),
                        ("feedback_template".to_string(), serde_json::json!("structured")),
                        ("anonymous_reviews".to_string(), serde_json::json!(false)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "consolidate_feedback".to_string(),
                    name_template: "Consolidate Review Feedback".to_string(),
                    description_template: "Aggregate and summarize peer review feedback".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["conduct_reviews".to_string()],
                    configuration: vec![
                        ("aggregation_method".to_string(), serde_json::json!("consensus")),
                        ("conflict_resolution".to_string(), serde_json::json!("moderator")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
            ],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "CIM Workflow System".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec!["review".to_string(), "peer".to_string(), "collaborative".to_string()],
                category: "Review Workflows".to_string(),
                documentation_url: Some("https://docs.cim.ai/workflows/review/peer".to_string()),
                examples: vec![],
            },
            validation_rules: vec![],
        }
    }
    
    /// Create user onboarding workflow template
    fn create_user_onboarding_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "user".to_string(),
                "onboarding".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "User Onboarding Workflow".to_string(),
            description: "Comprehensive user onboarding process with account setup and training".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["user".to_string(), "security".to_string(), "notification".to_string()],
            parameters: vec![
                (
                    "user_id".to_string(),
                    TemplateParameter {
                        name: "user_id".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the new user".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "department".to_string(),
                    TemplateParameter {
                        name: "department".to_string(),
                        param_type: ParameterType::String,
                        description: "User's department".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "role".to_string(),
                    TemplateParameter {
                        name: "role".to_string(),
                        param_type: ParameterType::String,
                        description: "User's role".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "create_account".to_string(),
                    name_template: "Create User Account".to_string(),
                    description_template: "Set up user account for {user_id}".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec![],
                    configuration: vec![
                        ("account_type".to_string(), serde_json::json!("standard")),
                        ("initial_permissions".to_string(), serde_json::json!("minimal")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "assign_permissions".to_string(),
                    name_template: "Assign Role Permissions".to_string(),
                    description_template: "Grant permissions based on {role} in {department}".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["create_account".to_string()],
                    configuration: vec![
                        ("permission_template".to_string(), serde_json::json!("{role}_{department}")),
                        ("approval_required".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "send_welcome".to_string(),
                    name_template: "Send Welcome Materials".to_string(),
                    description_template: "Send onboarding materials to new user".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["assign_permissions".to_string()],
                    configuration: vec![
                        ("welcome_package".to_string(), serde_json::json!("comprehensive")),
                        ("training_schedule".to_string(), serde_json::json!(true)),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
            ],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "CIM Workflow System".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec!["user".to_string(), "onboarding".to_string(), "setup".to_string()],
                category: "User Management".to_string(),
                documentation_url: Some("https://docs.cim.ai/workflows/user/onboarding".to_string()),
                examples: vec![],
            },
            validation_rules: vec![],
        }
    }
    
    // Placeholder methods for remaining templates - would be implemented similarly
    fn create_qa_review_template(&self) -> WorkflowTemplate { self.create_placeholder_template("review", "qa-review", "Quality Assurance Review") }
    fn create_security_review_template(&self) -> WorkflowTemplate { self.create_placeholder_template("review", "security-review", "Security Review") }
    fn create_document_publication_template(&self) -> WorkflowTemplate { self.create_placeholder_template("document", "publication", "Document Publication") }
    fn create_document_archival_template(&self) -> WorkflowTemplate { self.create_placeholder_template("document", "archival", "Document Archival") }
    fn create_document_collaboration_template(&self) -> WorkflowTemplate { self.create_placeholder_template("document", "collaboration", "Document Collaboration") }
    fn create_user_offboarding_template(&self) -> WorkflowTemplate { self.create_placeholder_template("user", "offboarding", "User Offboarding") }
    fn create_access_request_template(&self) -> WorkflowTemplate { self.create_placeholder_template("user", "access-request", "Access Request") }
    fn create_cross_domain_transaction_template(&self) -> WorkflowTemplate { self.create_placeholder_template("coordination", "cross-transaction", "Cross-Domain Transaction") }
    fn create_event_synchronization_template(&self) -> WorkflowTemplate { self.create_placeholder_template("coordination", "event-sync", "Event Synchronization") }
    fn create_distributed_process_template(&self) -> WorkflowTemplate { self.create_placeholder_template("coordination", "distributed-process", "Distributed Process") }
    fn create_escalation_notification_template(&self) -> WorkflowTemplate { self.create_placeholder_template("notification", "escalation", "Escalation Notification") }
    fn create_broadcast_notification_template(&self) -> WorkflowTemplate { self.create_placeholder_template("notification", "broadcast", "Broadcast Notification") }
    fn create_incident_response_template(&self) -> WorkflowTemplate { self.create_placeholder_template("security", "incident-response", "Incident Response") }
    fn create_compliance_audit_template(&self) -> WorkflowTemplate { self.create_placeholder_template("security", "compliance-audit", "Compliance Audit") }
    
    /// Create a placeholder template (for templates not yet fully implemented)
    fn create_placeholder_template(&self, domain: &str, name: &str, display_name: &str) -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                domain.to_string(),
                name.to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: display_name.to_string(),
            description: format!("{} workflow template", display_name),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec![domain.to_string()],
            parameters: HashMap::new(),
            steps: vec![
                TemplateStep {
                    id: "placeholder_step".to_string(),
                    name_template: format!("{} Step", display_name),
                    description_template: format!("Placeholder step for {}", display_name),
                    step_type: TemplateStepType::Manual,
                    dependencies: vec![],
                    configuration: HashMap::new(),
                    condition: None,
                    retry_policy: None,
                },
            ],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "CIM Workflow System".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec![domain.to_string(), name.to_string()],
                category: format!("{} Workflows", domain.to_title_case()),
                documentation_url: None,
                examples: vec![],
            },
            validation_rules: vec![],
        }
    }
}

impl Default for StandardTemplateLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for string title case conversion
trait ToTitleCase {
    fn to_title_case(&self) -> String;
}

impl ToTitleCase for str {
    fn to_title_case(&self) -> String {
        self.split_whitespace()
            .map(|word| {
                let mut chars: Vec<char> = word.chars().collect();
                if let Some(first) = chars.first_mut() {
                    *first = first.to_ascii_uppercase();
                }
                chars.iter().collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Template library service for managing and serving templates
pub struct TemplateLibraryService {
    library: StandardTemplateLibrary,
    custom_templates: HashMap<String, WorkflowTemplate>,
}

impl TemplateLibraryService {
    /// Create a new template library service
    pub fn new() -> Self {
        Self {
            library: StandardTemplateLibrary::new(),
            custom_templates: HashMap::new(),
        }
    }
    
    /// Get a template by domain, name, and version
    pub fn get_template(
        &self,
        domain: &str,
        name: &str,
        version: &TemplateVersion,
    ) -> Option<&WorkflowTemplate> {
        let key = format!("{}/{}@{}", domain, name, version.to_string());
        
        // Check custom templates first
        if let Some(template) = self.custom_templates.get(&key) {
            return Some(template);
        }
        
        // Check standard library
        self.library.get_template(&key)
    }
    
    /// Register a custom template
    pub fn register_custom_template(&mut self, template: WorkflowTemplate) {
        let key = self.library.template_key(&template.id);
        self.custom_templates.insert(key, template);
    }
    
    /// List all available templates
    pub fn list_all_templates(&self) -> Vec<&WorkflowTemplate> {
        let mut templates = self.library.list_templates();
        templates.extend(self.custom_templates.values());
        templates
    }
    
    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &str) -> Vec<&WorkflowTemplate> {
        let mut templates = self.library.get_templates_by_category(category);
        templates.extend(
            self.custom_templates.values()
                .filter(|t| t.metadata.category == category)
        );
        templates
    }
}

impl Default for TemplateLibraryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_template_library_creation() {
        let library = StandardTemplateLibrary::new();
        let templates = library.list_templates();
        
        // Should have templates from all categories
        assert!(!templates.is_empty());
        
        // Check for specific key templates
        let single_approval = library.get_template("approval/single-approval@1.0.0");
        assert!(single_approval.is_some());
        assert_eq!(single_approval.unwrap().name, "Single Approval Workflow");
    }
    
    #[test]
    fn test_template_library_service() {
        let mut service = TemplateLibraryService::new();
        
        // Test getting standard template
        let template = service.get_template(
            "approval",
            "single-approval",
            &TemplateVersion::new(1, 0, 0)
        );
        assert!(template.is_some());
        
        // Test registering custom template
        let custom_template = WorkflowTemplate {
            id: TemplateId::new(
                "custom".to_string(),
                "test".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Custom Test Template".to_string(),
            description: "A custom template for testing".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["test".to_string()],
            parameters: HashMap::new(),
            steps: vec![],
            constraints: vec![],
            metadata: TemplateMetadata {
                author: "Test".to_string(),
                created_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
                tags: vec!["test".to_string()],
                category: "Test".to_string(),
                documentation_url: None,
                examples: vec![],
            },
            validation_rules: vec![],
        };
        
        service.register_custom_template(custom_template);
        
        let custom = service.get_template(
            "custom",
            "test",
            &TemplateVersion::new(1, 0, 0)
        );
        assert!(custom.is_some());
        assert_eq!(custom.unwrap().name, "Custom Test Template");
    }
    
    #[test]
    fn test_template_categorization() {
        let library = StandardTemplateLibrary::new();
        
        let approval_templates = library.get_templates_by_category("Approval Workflows");
        assert!(!approval_templates.is_empty());
        
        let review_templates = library.get_templates_by_category("Review Workflows");
        assert!(!review_templates.is_empty());
    }
}