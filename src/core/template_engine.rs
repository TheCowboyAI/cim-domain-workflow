//! Core Template Engine
//!
//! Implements the core template instantiation engine that integrates with the
//! algebraic event system and NATS messaging for distributed workflow coordination.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::composition::templates::*;
use crate::messaging::publishers::{WorkflowEventBroker};
use crate::messaging::correlation::{WorkflowEventCorrelator, CompletionCriteria};
use crate::primitives::{WorkflowContext};
use crate::core::WorkflowEngine;

/// Core template engine integrating template system with workflow execution
pub struct CoreTemplateEngine {
    /// Template instantiation engine
    instantiation_engine: TemplateInstantiationEngine,
    /// Event broker for publishing template events
    event_broker: Arc<WorkflowEventBroker>,
    /// Event correlator for tracking template instantiation workflows
    correlator: Arc<WorkflowEventCorrelator>,
    /// Template registry
    template_registry: Arc<dyn TemplateRepository>,
    /// Standard template library
    standard_library: StandardTemplateLibrary,
}

/// Template execution context
#[derive(Debug, Clone)]
pub struct TemplateExecutionContext {
    /// Workflow context
    pub workflow_context: WorkflowContext,
    /// Template instantiation request
    pub request: TemplateInstantiationRequest,
    /// Template being executed
    pub template: WorkflowTemplate,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Template execution result
#[derive(Debug, Clone)]
pub struct TemplateExecutionResult {
    /// Instantiation result
    pub instantiation_result: TemplateInstantiationResult,
    /// Published event metadata
    pub published_events: Vec<crate::messaging::publishers::EventMetadata>,
    /// Correlation chain ID
    pub correlation_chain_id: Uuid,
    /// Execution timing
    pub execution_time: chrono::Duration,
}

/// Template execution coordinator
#[async_trait]
pub trait TemplateExecutionCoordinator: Send + Sync {
    /// Execute template with cross-domain coordination
    async fn execute_template(
        &self,
        request: TemplateInstantiationRequest,
    ) -> Result<TemplateExecutionResult, TemplateExecutionError>;
    
    /// Get template execution status
    async fn get_execution_status(
        &self,
        correlation_id: Uuid,
    ) -> Result<TemplateExecutionStatus, TemplateExecutionError>;
    
    /// Cancel template execution
    async fn cancel_execution(
        &self,
        correlation_id: Uuid,
    ) -> Result<(), TemplateExecutionError>;
}

/// Template execution status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateExecutionStatus {
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Current execution phase
    pub phase: TemplateExecutionPhase,
    /// Completed steps
    pub completed_steps: Vec<String>,
    /// Failed steps
    pub failed_steps: Vec<String>,
    /// Execution progress percentage
    pub progress_percentage: f32,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Template execution phases
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TemplateExecutionPhase {
    /// Template validation phase
    Validation,
    /// Parameter processing phase
    ParameterProcessing,
    /// Event generation phase
    EventGeneration,
    /// Event publication phase
    EventPublication,
    /// Cross-domain coordination phase
    CrossDomainCoordination,
    /// Execution complete
    Completed,
    /// Execution failed
    Failed,
    /// Execution cancelled
    Cancelled,
}

/// Standard template library with common workflow patterns
pub struct StandardTemplateLibrary {
    /// Built-in templates
    templates: HashMap<TemplateId, WorkflowTemplate>,
}

/// In-memory template repository for testing and development
pub struct InMemoryTemplateRepository {
    /// Template storage
    templates: Arc<tokio::sync::RwLock<HashMap<TemplateId, WorkflowTemplate>>>,
}

impl CoreTemplateEngine {
    /// Create new core template engine
    pub async fn new(
        template_repository: Arc<dyn TemplateRepository>,
        workflow_engine: Box<dyn WorkflowEngine>,
        event_broker: Arc<WorkflowEventBroker>,
    ) -> Result<Self, TemplateEngineError> {
        let correlator = Arc::new(WorkflowEventCorrelator::new());
        
        let instantiation_engine = TemplateInstantiationEngine::new(
            template_repository.clone(),
            workflow_engine,
        );
        
        let standard_library = StandardTemplateLibrary::new().await?;
        
        Ok(Self {
            instantiation_engine,
            event_broker,
            correlator,
            template_registry: template_repository,
            standard_library,
        })
    }
    
    /// Initialize standard templates in repository
    pub async fn initialize_standard_templates(&self) -> Result<(), TemplateEngineError> {
        for template in self.standard_library.get_all_templates() {
            self.template_registry.store_template(template.clone()).await
                .map_err(|e| TemplateEngineError::RepositoryError(e))?;
        }
        Ok(())
    }
    
    /// Get template by ID with domain-specific adaptation
    pub async fn get_adapted_template(
        &self,
        template_id: &TemplateId,
        target_domain: &str,
    ) -> Result<WorkflowTemplate, TemplateEngineError> {
        let mut template = self.template_registry.get_template(template_id).await
            .map_err(|e| TemplateEngineError::RepositoryError(e))?;
        
        // Adapt template for target domain if needed
        if !template.target_domains.contains(&target_domain.to_string()) {
            template = self.adapt_template_for_domain(template, target_domain).await?;
        }
        
        Ok(template)
    }
    
    /// Adapt template for specific domain
    async fn adapt_template_for_domain(
        &self,
        mut template: WorkflowTemplate,
        target_domain: &str,
    ) -> Result<WorkflowTemplate, TemplateEngineError> {
        // Add target domain to supported domains
        template.target_domains.push(target_domain.to_string());
        
        // Adapt step configurations for domain-specific requirements
        for step in &mut template.steps {
            // Add domain-specific configuration if needed
            step.configuration.insert(
                "adapted_for_domain".to_string(),
                serde_json::json!(target_domain)
            );
            
            // Update cross-domain steps
            if let TemplateStepType::CrossDomain { target_domain: step_domain, coordination_type } = &step.step_type {
                if step_domain == "*" {
                    step.step_type = TemplateStepType::CrossDomain {
                        target_domain: target_domain.to_string(),
                        coordination_type: coordination_type.clone(),
                    };
                }
            }
        }
        
        Ok(template)
    }
}

#[async_trait]
impl TemplateExecutionCoordinator for CoreTemplateEngine {
    async fn execute_template(
        &self,
        request: TemplateInstantiationRequest,
    ) -> Result<TemplateExecutionResult, TemplateExecutionError> {
        let start_time = std::time::Instant::now();
        
        // Start correlation chain for template execution
        let completion_criteria = CompletionCriteria {
            required_terminals: Vec::new(),
            max_duration: Some(chrono::Duration::hours(1)),
            required_domains: vec![request.target_domain.clone()].into_iter().collect(),
            min_event_count: Some(1),
        };
        
        self.correlator.start_chain(request.correlation_id, completion_criteria.clone())
            .map_err(|e| TemplateExecutionError::CorrelationError(e.to_string()))?;
        
        // Instantiate template
        let instantiation_result = self.instantiation_engine.instantiate(request.clone()).await
            .map_err(|e| TemplateExecutionError::InstantiationError(e))?;
        
        // Publish instantiation events
        let mut published_events = Vec::new();
        for event in &instantiation_result.events {
            let metadata = self.event_broker.publish_with_correlation(
                event.clone(),
                Some(completion_criteria.clone()),
            ).await
                .map_err(|e| TemplateExecutionError::PublicationError(e.to_string()))?;
            published_events.push(metadata);
        }
        
        let execution_time = chrono::Duration::from_std(start_time.elapsed()).unwrap_or_else(|_| chrono::Duration::zero());
        
        Ok(TemplateExecutionResult {
            instantiation_result,
            published_events,
            correlation_chain_id: request.correlation_id,
            execution_time,
        })
    }
    
    async fn get_execution_status(
        &self,
        correlation_id: Uuid,
    ) -> Result<TemplateExecutionStatus, TemplateExecutionError> {
        let analysis = self.correlator.analyze_completion(correlation_id)
            .map_err(|e| TemplateExecutionError::CorrelationError(e.to_string()))?;
        
        let phase = if analysis.is_complete {
            TemplateExecutionPhase::Completed
        } else {
            // Determine current phase based on completion analysis
            TemplateExecutionPhase::EventPublication
        };
        
        let progress_percentage = if analysis.statistics.total_events > 0 {
            (analysis.statistics.total_events as f32 / analysis.statistics.total_events as f32) * 100.0
        } else {
            0.0
        };
        
        Ok(TemplateExecutionStatus {
            correlation_id,
            phase,
            completed_steps: vec![], // Would be populated from correlation analysis
            failed_steps: vec![],    // Would be populated from error tracking
            progress_percentage,
            error_message: None,
        })
    }
    
    async fn cancel_execution(
        &self,
        correlation_id: Uuid,
    ) -> Result<(), TemplateExecutionError> {
        // Cancel active correlation chain
        self.correlator.cancel_correlation(correlation_id).await
            .map_err(|e| TemplateExecutionError::ExecutionFailed(
                format!("Failed to cancel correlation {}: {}", correlation_id, e)
            ))?;
        
        // Publish cancellation events through broker
        {
            let cancellation_subject = format!("cim.workflow.instance.lifecycle.cancelled.{}", correlation_id);
            let cancellation_data = serde_json::json!({
                "correlation_id": correlation_id,
                "cancelled_at": chrono::Utc::now().to_rfc3339(),
                "reason": "User requested cancellation"
            });
            
            // For now, skip publishing raw JSON data since broker expects WorkflowEvent
            // TODO: Create a proper WorkflowEvent for cancellation
            let _ = (cancellation_subject, cancellation_data); // Silence unused variable warning
        }
        
        Ok(())
    }
}

impl StandardTemplateLibrary {
    /// Create new standard template library
    pub async fn new() -> Result<Self, TemplateEngineError> {
        let mut templates = HashMap::new();
        
        // Add basic approval workflow template
        templates.insert(
            TemplateId::new(
                "workflow".to_string(),
                "approval".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            Self::create_approval_template(),
        );
        
        // Add sequential processing template
        templates.insert(
            TemplateId::new(
                "workflow".to_string(),
                "sequential_processing".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            Self::create_sequential_processing_template(),
        );
        
        // Add parallel processing template
        templates.insert(
            TemplateId::new(
                "workflow".to_string(),
                "parallel_processing".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            Self::create_parallel_processing_template(),
        );
        
        // Add cross-domain coordination template
        templates.insert(
            TemplateId::new(
                "integration".to_string(),
                "cross_domain_coordination".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            Self::create_cross_domain_template(),
        );
        
        Ok(Self { templates })
    }
    
    /// Get all templates
    pub fn get_all_templates(&self) -> Vec<&WorkflowTemplate> {
        self.templates.values().collect()
    }
    
    /// Get template by ID
    pub fn get_template(&self, id: &TemplateId) -> Option<&WorkflowTemplate> {
        self.templates.get(id)
    }
    
    /// Create basic approval workflow template
    fn create_approval_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "workflow".to_string(),
                "approval".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Basic Approval Workflow".to_string(),
            description: "Simple approval workflow with requestor, approver, and completion steps".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["workflow".to_string(), "person".to_string(), "document".to_string()],
            parameters: vec![
                (
                    "requestor".to_string(),
                    TemplateParameter {
                        name: "requestor".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the user making the request".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "approver".to_string(),
                    TemplateParameter {
                        name: "approver".to_string(),
                        param_type: ParameterType::String,
                        description: "ID of the approving user".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "approval_timeout".to_string(),
                    TemplateParameter {
                        name: "approval_timeout".to_string(),
                        param_type: ParameterType::Duration,
                        description: "Timeout for approval in minutes".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(1440)), // 24 hours
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "request_submission".to_string(),
                    name_template: "Submit Request".to_string(),
                    description_template: "Request submitted by {requestor}".to_string(),
                    step_type: TemplateStepType::Manual,
                    dependencies: vec![],
                    configuration: HashMap::new(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "approval_review".to_string(),
                    name_template: "Approval Review".to_string(),
                    description_template: "Review and approve/reject request".to_string(),
                    step_type: TemplateStepType::Manual,
                    dependencies: vec!["request_submission".to_string()],
                    configuration: vec![
                        ("assigned_user".to_string(), serde_json::json!("{approver}")),
                        ("timeout_minutes".to_string(), serde_json::json!("{approval_timeout}")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "approval_completion".to_string(),
                    name_template: "Complete Approval".to_string(),
                    description_template: "Finalize approval decision".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["approval_review".to_string()],
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
                tags: vec!["approval".to_string(), "manual".to_string(), "basic".to_string()],
                category: "Business Process".to_string(),
                documentation_url: None,
                examples: vec![
                    TemplateExample {
                        name: "Document Approval".to_string(),
                        description: "Approve a document for publication".to_string(),
                        parameters: vec![
                            ("requestor".to_string(), serde_json::json!("user123")),
                            ("approver".to_string(), serde_json::json!("manager456")),
                            ("approval_timeout".to_string(), serde_json::json!(720)), // 12 hours
                        ].into_iter().collect(),
                        expected_outcome: "Document approved and published".to_string(),
                    },
                ],
            },
            validation_rules: vec![],
        }
    }
    
    /// Create sequential processing template
    fn create_sequential_processing_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "workflow".to_string(),
                "sequential_processing".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Sequential Processing".to_string(),
            description: "Process items in sequential order with dependencies".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["workflow".to_string(), "document".to_string(), "data".to_string()],
            parameters: vec![
                (
                    "items".to_string(),
                    TemplateParameter {
                        name: "items".to_string(),
                        param_type: ParameterType::Array(Box::new(ParameterType::String)),
                        description: "List of items to process sequentially".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "initialize_processing".to_string(),
                    name_template: "Initialize Processing".to_string(),
                    description_template: "Set up sequential processing context".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec![],
                    configuration: HashMap::new(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "process_items".to_string(),
                    name_template: "Process Items Sequentially".to_string(),
                    description_template: "Process each item in sequential order".to_string(),
                    step_type: TemplateStepType::Sequential,
                    dependencies: vec!["initialize_processing".to_string()],
                    configuration: vec![
                        ("items".to_string(), serde_json::json!("{items}")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: Some(RetryPolicy {
                        max_attempts: 3,
                        initial_delay: chrono::Duration::seconds(30),
                        max_delay: chrono::Duration::minutes(5),
                        backoff_multiplier: 2.0,
                        retry_conditions: vec!["processing_error".to_string()],
                    }),
                },
                TemplateStep {
                    id: "finalize_processing".to_string(),
                    name_template: "Finalize Processing".to_string(),
                    description_template: "Complete sequential processing and generate results".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["process_items".to_string()],
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
                tags: vec!["sequential".to_string(), "processing".to_string(), "automated".to_string()],
                category: "Data Processing".to_string(),
                documentation_url: None,
                examples: vec![],
            },
            validation_rules: vec![],
        }
    }
    
    /// Create parallel processing template
    fn create_parallel_processing_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "workflow".to_string(),
                "parallel_processing".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Parallel Processing".to_string(),
            description: "Process multiple items concurrently with synchronization".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["workflow".to_string(), "document".to_string(), "data".to_string()],
            parameters: vec![
                (
                    "items".to_string(),
                    TemplateParameter {
                        name: "items".to_string(),
                        param_type: ParameterType::Array(Box::new(ParameterType::String)),
                        description: "List of items to process in parallel".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "max_concurrency".to_string(),
                    TemplateParameter {
                        name: "max_concurrency".to_string(),
                        param_type: ParameterType::Integer,
                        description: "Maximum number of concurrent processing tasks".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(5)),
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "initialize_parallel".to_string(),
                    name_template: "Initialize Parallel Processing".to_string(),
                    description_template: "Set up parallel processing context".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec![],
                    configuration: HashMap::new(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "process_parallel".to_string(),
                    name_template: "Process Items in Parallel".to_string(),
                    description_template: "Process items concurrently up to max_concurrency limit".to_string(),
                    step_type: TemplateStepType::Parallel,
                    dependencies: vec!["initialize_parallel".to_string()],
                    configuration: vec![
                        ("items".to_string(), serde_json::json!("{items}")),
                        ("max_concurrency".to_string(), serde_json::json!("{max_concurrency}")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: Some(RetryPolicy {
                        max_attempts: 3,
                        initial_delay: chrono::Duration::seconds(15),
                        max_delay: chrono::Duration::minutes(2),
                        backoff_multiplier: 1.5,
                        retry_conditions: vec!["processing_error".to_string()],
                    }),
                },
                TemplateStep {
                    id: "synchronize_results".to_string(),
                    name_template: "Synchronize Results".to_string(),
                    description_template: "Wait for all parallel tasks to complete and merge results".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["process_parallel".to_string()],
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
                tags: vec!["parallel".to_string(), "processing".to_string(), "automated".to_string()],
                category: "Data Processing".to_string(),
                documentation_url: None,
                examples: vec![],
            },
            validation_rules: vec![],
        }
    }
    
    /// Create cross-domain coordination template
    fn create_cross_domain_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: TemplateId::new(
                "integration".to_string(),
                "cross_domain_coordination".to_string(),
                TemplateVersion::new(1, 0, 0),
            ),
            name: "Cross-Domain Coordination".to_string(),
            description: "Coordinate workflow execution across multiple CIM domains".to_string(),
            version: TemplateVersion::new(1, 0, 0),
            target_domains: vec!["integration".to_string(), "workflow".to_string(), "person".to_string(), "document".to_string()],
            parameters: vec![
                (
                    "source_domain".to_string(),
                    TemplateParameter {
                        name: "source_domain".to_string(),
                        param_type: ParameterType::Domain,
                        description: "Source domain initiating coordination".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "target_domains".to_string(),
                    TemplateParameter {
                        name: "target_domains".to_string(),
                        param_type: ParameterType::Array(Box::new(ParameterType::Domain)),
                        description: "Target domains to coordinate with".to_string(),
                        required: true,
                        default_value: None,
                        constraints: vec![],
                    },
                ),
                (
                    "coordination_timeout".to_string(),
                    TemplateParameter {
                        name: "coordination_timeout".to_string(),
                        param_type: ParameterType::Duration,
                        description: "Timeout for cross-domain coordination".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(300)), // 5 minutes
                        constraints: vec![],
                    },
                ),
            ].into_iter().collect(),
            steps: vec![
                TemplateStep {
                    id: "initiate_coordination".to_string(),
                    name_template: "Initiate Cross-Domain Coordination".to_string(),
                    description_template: "Start coordination between {source_domain} and {target_domains}".to_string(),
                    step_type: TemplateStepType::CrossDomain {
                        target_domain: "*".to_string(),
                        coordination_type: "initiation".to_string(),
                    },
                    dependencies: vec![],
                    configuration: vec![
                        ("source_domain".to_string(), serde_json::json!("{source_domain}")),
                        ("target_domains".to_string(), serde_json::json!("{target_domains}")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: None,
                },
                TemplateStep {
                    id: "coordinate_execution".to_string(),
                    name_template: "Execute Cross-Domain Coordination".to_string(),
                    description_template: "Manage coordination execution across domains".to_string(),
                    step_type: TemplateStepType::CrossDomain {
                        target_domain: "*".to_string(),
                        coordination_type: "execution".to_string(),
                    },
                    dependencies: vec!["initiate_coordination".to_string()],
                    configuration: vec![
                        ("timeout_seconds".to_string(), serde_json::json!("{coordination_timeout}")),
                    ].into_iter().collect(),
                    condition: None,
                    retry_policy: Some(RetryPolicy {
                        max_attempts: 3,
                        initial_delay: chrono::Duration::seconds(5),
                        max_delay: chrono::Duration::seconds(30),
                        backoff_multiplier: 2.0,
                        retry_conditions: vec!["coordination_timeout".to_string(), "domain_unavailable".to_string()],
                    }),
                },
                TemplateStep {
                    id: "finalize_coordination".to_string(),
                    name_template: "Finalize Cross-Domain Coordination".to_string(),
                    description_template: "Complete coordination and synchronize results".to_string(),
                    step_type: TemplateStepType::Automated,
                    dependencies: vec!["coordinate_execution".to_string()],
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
                tags: vec!["cross-domain".to_string(), "coordination".to_string(), "integration".to_string()],
                category: "Integration".to_string(),
                documentation_url: None,
                examples: vec![],
            },
            validation_rules: vec![],
        }
    }
}

impl InMemoryTemplateRepository {
    /// Create new in-memory template repository
    pub fn new() -> Self {
        Self {
            templates: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TemplateRepository for InMemoryTemplateRepository {
    async fn store_template(&self, template: WorkflowTemplate) -> Result<(), TemplateError> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id.clone(), template);
        Ok(())
    }
    
    async fn get_template(&self, id: &TemplateId) -> Result<WorkflowTemplate, TemplateError> {
        let templates = self.templates.read().await;
        templates.get(id).cloned()
            .ok_or_else(|| TemplateError::TemplateNotFound(id.to_string()))
    }
    
    async fn list_templates_by_domain(&self, domain: &str) -> Result<Vec<TemplateId>, TemplateError> {
        let templates = self.templates.read().await;
        let matching_ids: Vec<TemplateId> = templates.values()
            .filter(|template| template.target_domains.contains(&domain.to_string()))
            .map(|template| template.id.clone())
            .collect();
        Ok(matching_ids)
    }
    
    async fn search_templates(&self, query: &TemplateSearchQuery) -> Result<Vec<TemplateId>, TemplateError> {
        let templates = self.templates.read().await;
        let matching_ids: Vec<TemplateId> = templates.values()
            .filter(|template| {
                // Simple search implementation
                let matches_domain = query.domain.as_ref()
                    .map(|d| template.target_domains.contains(d))
                    .unwrap_or(true);
                
                let matches_category = query.category.as_ref()
                    .map(|c| template.metadata.category == *c)
                    .unwrap_or(true);
                
                let matches_author = query.author.as_ref()
                    .map(|a| template.metadata.author == *a)
                    .unwrap_or(true);
                
                let matches_tags = query.tags.is_empty() || 
                    query.tags.iter().any(|tag| template.metadata.tags.contains(tag));
                
                matches_domain && matches_category && matches_author && matches_tags
            })
            .map(|template| template.id.clone())
            .collect();
        Ok(matching_ids)
    }
    
    async fn update_template(&self, template: WorkflowTemplate) -> Result<(), TemplateError> {
        let mut templates = self.templates.write().await;
        if templates.contains_key(&template.id) {
            templates.insert(template.id.clone(), template);
            Ok(())
        } else {
            Err(TemplateError::TemplateNotFound(template.id.to_string()))
        }
    }
    
    async fn delete_template(&self, id: &TemplateId) -> Result<(), TemplateError> {
        let mut templates = self.templates.write().await;
        if templates.remove(id).is_some() {
            Ok(())
        } else {
            Err(TemplateError::TemplateNotFound(id.to_string()))
        }
    }
}

/// Template engine errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateEngineError {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] TemplateError),

    #[error("Standard library initialization error: {0}")]
    StandardLibraryError(String),

    #[error("Template adaptation error: {0}")]
    AdaptationError(String),
}

/// Template execution errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateExecutionError {
    #[error("Instantiation error: {0}")]
    InstantiationError(#[from] TemplateError),

    #[error("Publication error: {0}")]
    PublicationError(String),

    #[error("Correlation error: {0}")]
    CorrelationError(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Execution timeout: {0}")]
    ExecutionTimeout(String),

    #[error("Execution cancelled: {0}")]
    ExecutionCancelled(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::publishers::BrokerConfiguration;

    #[tokio::test]
    async fn test_standard_template_library() {
        let library = StandardTemplateLibrary::new().await.unwrap();
        let templates = library.get_all_templates();
        
        assert!(!templates.is_empty());
        
        let approval_id = TemplateId::new(
            "workflow".to_string(),
            "approval".to_string(),
            TemplateVersion::new(1, 0, 0),
        );
        
        let approval_template = library.get_template(&approval_id);
        assert!(approval_template.is_some());
        assert_eq!(approval_template.unwrap().name, "Basic Approval Workflow");
    }

    #[tokio::test]
    async fn test_in_memory_template_repository() {
        let repo = InMemoryTemplateRepository::new();
        let library = StandardTemplateLibrary::new().await.unwrap();
        
        let template = library.get_all_templates()[0].clone();
        let template_id = template.id.clone();
        
        // Store template
        repo.store_template(template).await.unwrap();
        
        // Retrieve template
        let retrieved = repo.get_template(&template_id).await.unwrap();
        assert_eq!(retrieved.id, template_id);
        
        // List templates by domain
        let domain_templates = repo.list_templates_by_domain("workflow").await.unwrap();
        assert!(!domain_templates.is_empty());
    }
}