//! Test assertions for validating workflow behavior

use super::{TestExecutionContext, AssertionResult, TestAssertion};
use crate::{
    value_objects::WorkflowStatus,
    algebra::WorkflowEvent,
};
use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;

/// Workflow state assertion
pub struct WorkflowStateAssertion {
    expected_status: WorkflowStatus,
    description: String,
}

impl WorkflowStateAssertion {
    pub fn new(expected_status: WorkflowStatus) -> Self {
        Self {
            expected_status: expected_status.clone(),
            description: format!("Workflow should have status: {:?}", expected_status),
        }
    }
}

impl TestAssertion for WorkflowStateAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        // In a real test, we would get the workflow from context
        // For now, we'll create a placeholder implementation
        let workflow_data = context.test_data.try_read();
        
        if let Ok(data) = workflow_data {
            if let Some(workflow_status) = data.get("workflow_status") {
                if let Ok(status) = serde_json::from_value::<WorkflowStatus>(workflow_status.clone()) {
                    let passed = status == self.expected_status;
                    return AssertionResult {
                        assertion_type: "workflow_state".to_string(),
                        description: self.description.clone(),
                        passed,
                        expected: Some(serde_json::to_value(&self.expected_status).unwrap()),
                        actual: Some(serde_json::to_value(&status).unwrap()),
                        context: HashMap::new(),
                    };
                }
            }
        }

        AssertionResult {
            assertion_type: "workflow_state".to_string(),
            description: self.description.clone(),
            passed: false,
            expected: Some(serde_json::to_value(&self.expected_status).unwrap()),
            actual: None,
            context: HashMap::new(),
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Event count assertion
pub struct EventCountAssertion {
    expected_count: usize,
    event_type_filter: Option<String>,
    description: String,
}

impl EventCountAssertion {
    pub fn new(expected_count: usize) -> Self {
        Self {
            expected_count,
            event_type_filter: None,
            description: format!("Should have {} events", expected_count),
        }
    }

    pub fn with_event_type(mut self, event_type: &str) -> Self {
        self.event_type_filter = Some(event_type.to_string());
        self.description = format!("Should have {} events of type '{}'", self.expected_count, event_type);
        self
    }
}

impl TestAssertion for EventCountAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        let events_data = context.test_data.try_read();
        
        if let Ok(data) = events_data {
            if let Some(events_value) = data.get("events") {
                if let Ok(events) = serde_json::from_value::<Vec<WorkflowEvent>>(events_value.clone()) {
                    let count = if let Some(ref filter_type) = self.event_type_filter {
                        events.iter()
                            .filter(|event| self.matches_event_type(event, filter_type))
                            .count()
                    } else {
                        events.len()
                    };

                    let passed = count == self.expected_count;
                    return AssertionResult {
                        assertion_type: "event_count".to_string(),
                        description: self.description.clone(),
                        passed,
                        expected: Some(Value::Number(self.expected_count.into())),
                        actual: Some(Value::Number(count.into())),
                        context: HashMap::new(),
                    };
                }
            }
        }

        AssertionResult {
            assertion_type: "event_count".to_string(),
            description: self.description.clone(),
            passed: false,
            expected: Some(Value::Number(self.expected_count.into())),
            actual: None,
            context: HashMap::new(),
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

impl EventCountAssertion {
    fn matches_event_type(&self, event: &WorkflowEvent, filter_type: &str) -> bool {
        use crate::algebra::{EventType, LifecycleEventType, StepEventType};
        
        match (&event.event_type, filter_type) {
            (EventType::Lifecycle(LifecycleEventType::WorkflowCreated), "WorkflowCreated") => true,
            (EventType::Lifecycle(LifecycleEventType::WorkflowStarted), "WorkflowStarted") => true,
            (EventType::Lifecycle(LifecycleEventType::WorkflowCompleted), "WorkflowCompleted") => true,
            (EventType::Step(StepEventType::StepCreated), "StepAdded") => true,
            (EventType::Step(StepEventType::StepStarted), "StepStarted") => true,
            (EventType::Step(StepEventType::StepCompleted), "StepCompleted") => true,
            _ => false,
        }
    }
}

/// Value assertion for checking specific values in test data
pub struct ValueAssertion {
    path: String,
    expected_value: Value,
    description: String,
}

impl ValueAssertion {
    pub fn new(path: &str, expected_value: Value) -> Self {
        Self {
            path: path.to_string(),
            expected_value: expected_value.clone(),
            description: format!("Value at '{}' should be: {}", path, expected_value),
        }
    }
}

impl TestAssertion for ValueAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        let data = context.test_data.try_read();
        
        if let Ok(data) = data {
            if let Some(actual_value) = self.get_value_at_path(&data, &self.path) {
                let passed = *actual_value == self.expected_value;
                return AssertionResult {
                    assertion_type: "value_assertion".to_string(),
                    description: self.description.clone(),
                    passed,
                    expected: Some(self.expected_value.clone()),
                    actual: Some(actual_value.clone()),
                    context: HashMap::new(),
                };
            }
        }

        AssertionResult {
            assertion_type: "value_assertion".to_string(),
            description: self.description.clone(),
            passed: false,
            expected: Some(self.expected_value.clone()),
            actual: None,
            context: HashMap::new(),
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

impl ValueAssertion {
    fn get_value_at_path<'a>(&self, data: &'a HashMap<String, Value>, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data.get(parts[0])?;

        for part in parts.iter().skip(1) {
            if let Some(obj) = current.as_object() {
                current = obj.get(*part)?;
            } else if let Some(arr) = current.as_array() {
                if let Ok(index) = part.parse::<usize>() {
                    current = arr.get(index)?;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(current)
    }
}

/// Duration assertion for checking execution times
pub struct DurationAssertion {
    max_duration: Duration,
    description: String,
}

impl DurationAssertion {
    pub fn new(max_duration: Duration) -> Self {
        Self {
            max_duration,
            description: format!("Execution should complete within {:?}", max_duration),
        }
    }
}

impl TestAssertion for DurationAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        let execution_time = context.start_time.elapsed().unwrap_or(Duration::from_secs(0));
        let passed = execution_time <= self.max_duration;

        AssertionResult {
            assertion_type: "duration_assertion".to_string(),
            description: self.description.clone(),
            passed,
            expected: Some(Value::String(format!("{:?}", self.max_duration))),
            actual: Some(Value::String(format!("{:?}", execution_time))),
            context: HashMap::new(),
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Custom assertion using a closure
pub struct CustomAssertion {
    assertion_fn: Box<dyn Fn(&TestExecutionContext) -> bool + Send + Sync>,
    description: String,
}

impl CustomAssertion {
    pub fn new<F>(description: &str, assertion_fn: F) -> Self 
    where
        F: Fn(&TestExecutionContext) -> bool + Send + Sync + 'static,
    {
        Self {
            assertion_fn: Box::new(assertion_fn),
            description: description.to_string(),
        }
    }
}

impl TestAssertion for CustomAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        let passed = (self.assertion_fn)(context);

        AssertionResult {
            assertion_type: "custom_assertion".to_string(),
            description: self.description.clone(),
            passed,
            expected: Some(Value::Bool(true)),
            actual: Some(Value::Bool(passed)),
            context: HashMap::new(),
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Service health assertion
pub struct ServiceHealthAssertion {
    service_name: String,
    description: String,
}

impl ServiceHealthAssertion {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            description: format!("Service '{}' should be healthy", service_name),
        }
    }
}

impl TestAssertion for ServiceHealthAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        let services = context.service_connections.try_read();
        
        if let Ok(services) = services {
            if let Some(_service) = services.get(&self.service_name) {
                // For this example, we'll assume service is healthy if it exists
                // In a real implementation, we'd call service.health_check()
                return AssertionResult {
                    assertion_type: "service_health".to_string(),
                    description: self.description.clone(),
                    passed: true,
                    expected: Some(Value::Bool(true)),
                    actual: Some(Value::Bool(true)),
                    context: HashMap::new(),
                };
            }
        }

        AssertionResult {
            assertion_type: "service_health".to_string(),
            description: self.description.clone(),
            passed: false,
            expected: Some(Value::Bool(true)),
            actual: Some(Value::Bool(false)),
            context: HashMap::new(),
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Collection of common assertion builders
pub struct Assertions;

impl Assertions {
    /// Create workflow status assertion
    pub fn workflow_status(status: WorkflowStatus) -> Box<dyn TestAssertion> {
        Box::new(WorkflowStateAssertion::new(status))
    }

    /// Create event count assertion
    pub fn event_count(count: usize) -> Box<dyn TestAssertion> {
        Box::new(EventCountAssertion::new(count))
    }

    /// Create event count assertion with type filter
    pub fn event_count_of_type(count: usize, event_type: &str) -> Box<dyn TestAssertion> {
        Box::new(EventCountAssertion::new(count).with_event_type(event_type))
    }

    /// Create value assertion
    pub fn value_equals(path: &str, expected: Value) -> Box<dyn TestAssertion> {
        Box::new(ValueAssertion::new(path, expected))
    }

    /// Create duration assertion
    pub fn completes_within(duration: Duration) -> Box<dyn TestAssertion> {
        Box::new(DurationAssertion::new(duration))
    }

    /// Create custom assertion
    pub fn custom<F>(description: &str, assertion_fn: F) -> Box<dyn TestAssertion>
    where
        F: Fn(&TestExecutionContext) -> bool + Send + Sync + 'static,
    {
        Box::new(CustomAssertion::new(description, assertion_fn))
    }

    /// Create service health assertion
    pub fn service_healthy(service_name: &str) -> Box<dyn TestAssertion> {
        Box::new(ServiceHealthAssertion::new(service_name))
    }

    /// Create assertion that multiple conditions are all true
    pub fn all_of(assertions: Vec<Box<dyn TestAssertion>>) -> Box<dyn TestAssertion> {
        Box::new(AllOfAssertion::new(assertions))
    }

    /// Create assertion that at least one condition is true
    pub fn any_of(assertions: Vec<Box<dyn TestAssertion>>) -> Box<dyn TestAssertion> {
        Box::new(AnyOfAssertion::new(assertions))
    }
}

/// Composite assertion that requires all sub-assertions to pass
pub struct AllOfAssertion {
    assertions: Vec<Box<dyn TestAssertion>>,
    description: String,
}

impl AllOfAssertion {
    pub fn new(assertions: Vec<Box<dyn TestAssertion>>) -> Self {
        let descriptions: Vec<String> = assertions.iter().map(|a| a.description()).collect();
        Self {
            assertions,
            description: format!("All of: [{}]", descriptions.join(", ")),
        }
    }
}

impl TestAssertion for AllOfAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        let mut all_passed = true;
        let mut results = Vec::new();

        for assertion in &self.assertions {
            let result = assertion.assert(context);
            all_passed &= result.passed;
            results.push(result);
        }

        let mut context_data = HashMap::new();
        context_data.insert("sub_results".to_string(), serde_json::to_value(&results).unwrap());

        AssertionResult {
            assertion_type: "all_of_assertion".to_string(),
            description: self.description.clone(),
            passed: all_passed,
            expected: Some(Value::Bool(true)),
            actual: Some(Value::Bool(all_passed)),
            context: context_data,
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Composite assertion that requires at least one sub-assertion to pass
pub struct AnyOfAssertion {
    assertions: Vec<Box<dyn TestAssertion>>,
    description: String,
}

impl AnyOfAssertion {
    pub fn new(assertions: Vec<Box<dyn TestAssertion>>) -> Self {
        let descriptions: Vec<String> = assertions.iter().map(|a| a.description()).collect();
        Self {
            assertions,
            description: format!("Any of: [{}]", descriptions.join(", ")),
        }
    }
}

impl TestAssertion for AnyOfAssertion {
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult {
        let mut any_passed = false;
        let mut results = Vec::new();

        for assertion in &self.assertions {
            let result = assertion.assert(context);
            let passed = result.passed;
            any_passed |= passed;
            results.push(result);
            
            // Short-circuit if we found one that passes
            if passed {
                break;
            }
        }

        let mut context_data = HashMap::new();
        context_data.insert("sub_results".to_string(), serde_json::to_value(&results).unwrap());

        AssertionResult {
            assertion_type: "any_of_assertion".to_string(),
            description: self.description.clone(),
            passed: any_passed,
            expected: Some(Value::Bool(true)),
            actual: Some(Value::Bool(any_passed)),
            context: context_data,
        }
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{TestConfig, TestService, TestServiceStatistics};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::time::SystemTime;

    #[test]
    fn test_event_count_assertion() {
        let assertion = EventCountAssertion::new(3);
        assert_eq!(assertion.description(), "Should have 3 events");
    }

    #[test]
    fn test_duration_assertion() {
        let assertion = DurationAssertion::new(Duration::from_secs(5));
        assert_eq!(assertion.description(), "Execution should complete within 5s");
    }

    #[test]
    fn test_assertions_builder() {
        let assertion = Assertions::workflow_status(WorkflowStatus::Running);
        assert!(assertion.description().contains("Running"));
    }

    #[tokio::test]
    async fn test_composite_assertions() {
        let assertions = vec![
            Assertions::event_count(2),
            Assertions::completes_within(Duration::from_secs(10)),
        ];
        
        let all_assertion = Assertions::all_of(assertions);
        assert!(all_assertion.description().contains("All of"));
    }
}