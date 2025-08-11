//! Test data generators for workflow testing

use crate::{
    aggregate::Workflow,
    value_objects::{WorkflowContext, WorkflowId},
    algebra::WorkflowEvent,
};
use cim_domain::AggregateRoot;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Test data generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Random seed for reproducible generation
    pub seed: Option<u64>,
    /// Generation templates
    pub templates: HashMap<String, GeneratorTemplate>,
    /// Default value ranges
    pub value_ranges: ValueRanges,
}

/// Generator template for specific data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorTemplate {
    /// Template name
    pub name: String,
    /// Template type
    pub template_type: String,
    /// Template parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Value constraints
    pub constraints: Option<ValueConstraints>,
}

/// Value constraints for generated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueConstraints {
    /// String length constraints
    pub string_length: Option<(usize, usize)>,
    /// Number range constraints
    pub number_range: Option<(f64, f64)>,
    /// Array size constraints
    pub array_size: Option<(usize, usize)>,
    /// Required fields
    pub required_fields: Option<Vec<String>>,
}

/// Default value ranges for generation
#[derive(Debug, Clone)]
pub struct ValueRanges {
    /// String length range
    pub string_length: (usize, usize),
    /// Integer range
    pub integer_range: (i64, i64),
    /// Float range
    pub float_range: (f64, f64),
    /// Array size range
    pub array_size: (usize, usize),
}

impl Default for ValueRanges {
    fn default() -> Self {
        Self {
            string_length: (5, 50),
            integer_range: (1, 1000),
            float_range: (0.1, 100.0),
            array_size: (1, 10),
        }
    }
}

/// Main test data generator
pub struct TestDataGenerator {
    config: GeneratorConfig,
    rng_state: u64, // Simple LCG state
}

impl TestDataGenerator {
    /// Create a new generator with configuration
    pub fn new(config: GeneratorConfig) -> Self {
        let rng_state = config.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
        });

        Self { config, rng_state }
    }

    /// Create generator with default configuration
    pub fn default() -> Self {
        Self::new(GeneratorConfig {
            seed: None,
            templates: HashMap::new(),
            value_ranges: ValueRanges::default(),
        })
    }

    /// Generate a test workflow
    pub fn generate_workflow(&mut self, template_name: Option<&str>) -> Result<Workflow, Box<dyn std::error::Error>> {
        let title_template = template_name
            .and_then(|t| self.config.templates.get(t))
            .and_then(|template| template.parameters.get("title_template"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let title = if let Some(template_str) = title_template {
            self.apply_template_string(&template_str)
        } else {
            format!("Generated Workflow {}", self.next_random() % 1000)
        };

        let description = format!("Generated test workflow: {}", title);
        let _context = self.generate_workflow_context(template_name)?;
        
        let metadata = std::collections::HashMap::new();
        let (workflow, _events) = Workflow::new(
            title,
            description,
            metadata,
            Some("test_generator".to_string()),
        )?;

        Ok(workflow)
    }

    /// Generate test workflow context
    pub fn generate_workflow_context(&mut self, template_name: Option<&str>) -> Result<WorkflowContext, Box<dyn std::error::Error>> {
        let mut context = WorkflowContext::new();
        
        // Generate variables
        let var_count = (self.next_random() % 5) + 1; // 1-5 variables
        for i in 0..var_count {
            let key = format!("test_var_{}", i);
            let value = self.generate_json_value("string")?;
            context.set_variable(key, value);
        }

        // Generate metadata
        let metadata_count = (self.next_random() % 3) + 1; // 1-3 metadata entries
        for i in 0..metadata_count {
            let key = format!("test_meta_{}", i);
            let value = json!(format!("meta_value_{}", i));
            context.set_metadata(key, value.as_str().unwrap_or("default").to_string());
        }

        // Add template-specific data if available
        if let Some(template) = template_name.and_then(|t| self.config.templates.get(t)) {
            if let Some(context_data) = template.parameters.get("context_data") {
                if let Some(obj) = context_data.as_object() {
                    for (key, value) in obj {
                        context.set_variable(key.clone(), value.clone());
                    }
                }
            }
        }

        Ok(context)
    }

    /// Generate test events
    pub fn generate_workflow_events(&mut self, workflow_id: WorkflowId, count: usize) -> Result<Vec<WorkflowEvent>, Box<dyn std::error::Error>> {
        use crate::algebra::{EventType, LifecycleEventType, StepEventType, EventPayload, EventContext, CausationChain};
        use uuid::Uuid;
        use chrono::Utc;
        
        let mut events = Vec::new();
        
        for i in 0..count {
            let event_id = Uuid::new_v4();
            let correlation_id = Uuid::new_v4();
            let mut payload_data = HashMap::new();
            payload_data.insert("workflow_id".to_string(), json!(workflow_id));
            
            let (event_type, additional_data) = match i % 4 {
                0 => {
                    let mut data = HashMap::new();
                    data.insert("step_title".to_string(), json!(format!("Generated Step {}", i)));
                    data.insert("step_type".to_string(), json!(if self.next_random() % 2 == 0 { "Automated" } else { "Manual" }));
                    (EventType::Step(StepEventType::StepCreated), data)
                },
                1 => {
                    let mut data = HashMap::new();
                    data.insert("started_by".to_string(), json!("test_generator"));
                    (EventType::Lifecycle(LifecycleEventType::WorkflowStarted), data)
                },
                2 => {
                    let mut data = HashMap::new();
                    data.insert("step_id".to_string(), json!(Uuid::new_v4()));
                    (EventType::Step(StepEventType::StepStarted), data)
                },
                3 => {
                    let mut data = HashMap::new();
                    data.insert("step_id".to_string(), json!(Uuid::new_v4()));
                    data.insert("result".to_string(), json!({"status": "success", "data": format!("result_{}", i)}));
                    (EventType::Step(StepEventType::StepCompleted), data)
                },
                _ => unreachable!(),
            };
            
            payload_data.extend(additional_data);
            
            let event = WorkflowEvent {
                id: event_id,
                event_type,
                domain: "test_domain".to_string(),
                correlation_id,
                causation_chain: CausationChain {
                    root_correlation: correlation_id,
                    chain: vec![event_id],
                    relationships: HashMap::new(),
                },
                timestamp: Utc::now(),
                payload: EventPayload {
                    data: payload_data,
                    metadata: HashMap::new(),
                },
                context: EventContext {
                    workflow_instance_id: Some(*workflow_id.as_uuid()),
                    step_instance_id: None,
                    template_id: None,
                    domain_context: HashMap::new(),
                },
            };
            
            events.push(event);
        }

        Ok(events)
    }

    /// Generate JSON value of specified type
    pub fn generate_json_value(&mut self, value_type: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match value_type {
            "string" => Ok(json!(self.generate_string())),
            "number" => Ok(json!(self.generate_number())),
            "integer" => Ok(json!(self.generate_integer())),
            "boolean" => Ok(json!(self.next_random() % 2 == 0)),
            "array" => {
                let size = (self.next_random() as usize % self.config.value_ranges.array_size.1) + self.config.value_ranges.array_size.0;
                let mut array = Vec::new();
                for _ in 0..size {
                    array.push(self.generate_json_value("string")?);
                }
                Ok(json!(array))
            },
            "object" => {
                let mut obj = serde_json::Map::new();
                let size = (self.next_random() as usize % 5) + 1;
                for i in 0..size {
                    obj.insert(format!("key_{}", i), self.generate_json_value("string")?);
                }
                Ok(json!(obj))
            },
            _ => Ok(json!(self.generate_string())),
        }
    }

    /// Generate random string
    fn generate_string(&mut self) -> String {
        let length = (self.next_random() as usize % (self.config.value_ranges.string_length.1 - self.config.value_ranges.string_length.0)) + self.config.value_ranges.string_length.0;
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
        (0..length)
            .map(|_| chars[self.next_random() as usize % chars.len()])
            .collect()
    }

    /// Generate random number (float)
    fn generate_number(&mut self) -> f64 {
        let range = self.config.value_ranges.float_range.1 - self.config.value_ranges.float_range.0;
        self.config.value_ranges.float_range.0 + (self.next_random() as f64 / u64::MAX as f64) * range
    }

    /// Generate random integer
    fn generate_integer(&mut self) -> i64 {
        let range = (self.config.value_ranges.integer_range.1 - self.config.value_ranges.integer_range.0) as u64;
        self.config.value_ranges.integer_range.0 + (self.next_random() % range) as i64
    }

    /// Apply template string with placeholders
    fn apply_template_string(&mut self, template: &str) -> String {
        let mut result = template.to_string();
        
        // Replace common placeholders
        result = result.replace("{{RANDOM}}", &(self.next_random() % 10000).to_string());
        result = result.replace("{{STRING}}", &self.generate_string());
        result = result.replace("{{NUMBER}}", &self.generate_number().to_string());
        result = result.replace("{{INTEGER}}", &self.generate_integer().to_string());
        
        result
    }

    /// Simple LCG random number generator for reproducible results
    fn next_random(&mut self) -> u64 {
        // Linear Congruential Generator parameters (from Numerical Recipes)
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.rng_state
    }
}

/// Batch data generator for creating multiple test items
pub struct BatchGenerator {
    generator: TestDataGenerator,
}

impl BatchGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            generator: TestDataGenerator::new(config),
        }
    }

    /// Generate multiple workflows
    pub fn generate_workflows(&mut self, count: usize, template_name: Option<&str>) -> Result<Vec<Workflow>, Box<dyn std::error::Error>> {
        let mut workflows = Vec::new();
        for _ in 0..count {
            workflows.push(self.generator.generate_workflow(template_name)?);
        }
        Ok(workflows)
    }

    /// Generate test dataset
    pub fn generate_dataset(&mut self, dataset_config: DatasetConfig) -> Result<TestDataset, Box<dyn std::error::Error>> {
        let mut dataset = TestDataset {
            workflows: Vec::new(),
            events: Vec::new(),
            contexts: Vec::new(),
            metadata: HashMap::new(),
        };

        // Generate workflows
        for _ in 0..dataset_config.workflow_count {
            dataset.workflows.push(self.generator.generate_workflow(None)?);
        }

        // Generate events for each workflow
        for workflow in &dataset.workflows {
            let events = self.generator.generate_workflow_events(WorkflowId::from(workflow.id()), dataset_config.events_per_workflow)?;
            dataset.events.extend(events);
        }

        // Generate standalone contexts
        for _ in 0..dataset_config.context_count {
            dataset.contexts.push(self.generator.generate_workflow_context(None)?);
        }

        // Add dataset metadata
        dataset.metadata.insert("generated_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        dataset.metadata.insert("generator_version".to_string(), json!("1.0.0"));
        dataset.metadata.insert("workflow_count".to_string(), json!(dataset_config.workflow_count));
        dataset.metadata.insert("total_events".to_string(), json!(dataset.events.len()));

        Ok(dataset)
    }
}

/// Configuration for dataset generation
#[derive(Debug, Clone)]
pub struct DatasetConfig {
    /// Number of workflows to generate
    pub workflow_count: usize,
    /// Number of events per workflow
    pub events_per_workflow: usize,
    /// Number of standalone contexts
    pub context_count: usize,
}

/// Generated test dataset
#[derive(Debug)]
pub struct TestDataset {
    /// Generated workflows
    pub workflows: Vec<Workflow>,
    /// Generated events
    pub events: Vec<WorkflowEvent>,
    /// Generated contexts
    pub contexts: Vec<WorkflowContext>,
    /// Dataset metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let config = GeneratorConfig {
            seed: Some(12345),
            templates: HashMap::new(),
            value_ranges: ValueRanges::default(),
        };
        
        let generator = TestDataGenerator::new(config);
        assert_eq!(generator.rng_state, 12345);
    }

    #[test]
    fn test_workflow_generation() {
        let mut generator = TestDataGenerator::default();
        let workflow = generator.generate_workflow(None);
        
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap();
        assert!(!workflow.name.is_empty());
    }

    #[test]
    fn test_json_value_generation() {
        let mut generator = TestDataGenerator::default();
        
        let string_val = generator.generate_json_value("string").unwrap();
        assert!(string_val.is_string());
        
        let number_val = generator.generate_json_value("number").unwrap();
        assert!(number_val.is_number());
        
        let array_val = generator.generate_json_value("array").unwrap();
        assert!(array_val.is_array());
        
        let object_val = generator.generate_json_value("object").unwrap();
        assert!(object_val.is_object());
    }

    #[test]
    fn test_batch_generation() {
        let config = GeneratorConfig {
            seed: Some(54321),
            templates: HashMap::new(),
            value_ranges: ValueRanges::default(),
        };
        
        let mut batch_generator = BatchGenerator::new(config);
        let workflows = batch_generator.generate_workflows(5, None);
        
        assert!(workflows.is_ok());
        assert_eq!(workflows.unwrap().len(), 5);
    }

    #[test]
    fn test_dataset_generation() {
        let config = GeneratorConfig {
            seed: Some(98765),
            templates: HashMap::new(),
            value_ranges: ValueRanges::default(),
        };
        
        let mut batch_generator = BatchGenerator::new(config);
        let dataset_config = DatasetConfig {
            workflow_count: 3,
            events_per_workflow: 4,
            context_count: 2,
        };
        
        let dataset = batch_generator.generate_dataset(dataset_config);
        assert!(dataset.is_ok());
        
        let dataset = dataset.unwrap();
        assert_eq!(dataset.workflows.len(), 3);
        assert_eq!(dataset.events.len(), 12); // 3 workflows * 4 events
        assert_eq!(dataset.contexts.len(), 2);
    }
}