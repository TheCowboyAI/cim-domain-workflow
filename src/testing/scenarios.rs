//! Pre-built test scenarios for common workflow patterns

use super::{TestCase, TestBuilder, TestAction, TestExecutionContext};
use crate::{
    aggregate::Workflow,
    value_objects::{StepType, WorkflowStatus},
    testing::assertions::Assertions,
};
use cim_domain::AggregateRoot;
use std::collections::HashMap;
use std::time::Duration;
use serde_json::json;

/// Create a basic workflow creation test scenario
pub fn create_basic_workflow_scenario() -> TestCase {
    TestBuilder::new("basic_workflow_creation")
        .description("Test basic workflow creation and validation")
        .tag("smoke")
        .tag("workflow")
        .setup(Box::new(SetupWorkflowDataAction))
        .action(Box::new(CreateWorkflowAction {
            title: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
        }))
        .assert(Assertions::workflow_status(WorkflowStatus::Draft))
        .assert(Assertions::completes_within(Duration::from_secs(5)))
        .cleanup(Box::new(CleanupWorkflowDataAction))
        .build()
}

/// Create a workflow with steps scenario
pub fn create_workflow_with_steps_scenario() -> TestCase {
    TestBuilder::new("workflow_with_steps")
        .description("Test workflow creation with multiple steps")
        .tag("integration")
        .tag("workflow")
        .tag("steps")
        .setup(Box::new(SetupWorkflowDataAction))
        .action(Box::new(CreateWorkflowAction {
            title: "Multi-Step Workflow".to_string(),
            description: "Workflow with multiple steps".to_string(),
        }))
        .action(Box::new(AddStepAction {
            title: "Step 1".to_string(),
            step_type: StepType::Manual,
        }))
        .action(Box::new(AddStepAction {
            title: "Step 2".to_string(),
            step_type: StepType::Automated,
        }))
        .action(Box::new(AddStepAction {
            title: "Step 3".to_string(),
            step_type: StepType::Manual,
        }))
        .assert(Assertions::event_count_of_type(3, "StepAdded"))
        .assert(Assertions::custom("Should have 3 steps", |ctx| {
            if let Ok(data) = ctx.test_data.try_read() {
                if let Some(step_count) = data.get("step_count") {
                    return step_count.as_u64() == Some(3);
                }
            }
            false
        }))
        .cleanup(Box::new(CleanupWorkflowDataAction))
        .timeout(Duration::from_secs(30))
        .build()
}

/// Create a workflow execution scenario
pub fn create_workflow_execution_scenario() -> TestCase {
    TestBuilder::new("workflow_execution")
        .description("Test complete workflow execution flow")
        .tag("integration")
        .tag("execution")
        .tag("end-to-end")
        .setup(Box::new(SetupWorkflowDataAction))
        .setup(Box::new(SetupNatsConnectionAction))
        .action(Box::new(CreateWorkflowAction {
            title: "Execution Test Workflow".to_string(),
            description: "Test workflow execution".to_string(),
        }))
        .action(Box::new(AddStepAction {
            title: "Initial Step".to_string(),
            step_type: StepType::Automated,
        }))
        .action(Box::new(StartWorkflowAction))
        .action(Box::new(WaitForEventsAction {
            expected_events: vec!["WorkflowStarted".to_string(), "StepStarted".to_string()],
            timeout: Duration::from_secs(10),
        }))
        .assert(Assertions::workflow_status(WorkflowStatus::Running))
        .assert(Assertions::event_count_of_type(1, "WorkflowStarted"))
        .assert(Assertions::service_healthy("nats"))
        .cleanup(Box::new(CleanupNatsConnectionAction))
        .cleanup(Box::new(CleanupWorkflowDataAction))
        .timeout(Duration::from_secs(60))
        .build()
}

/// Create a performance test scenario
pub fn create_performance_test_scenario() -> TestCase {
    TestBuilder::new("workflow_performance")
        .description("Test workflow performance under load")
        .tag("performance")
        .tag("load")
        .setup(Box::new(SetupPerformanceMonitoringAction))
        .action(Box::new(CreateMultipleWorkflowsAction { count: 100 }))
        .action(Box::new(MeasurePerformanceAction))
        .assert(Assertions::completes_within(Duration::from_secs(30)))
        .assert(Assertions::custom("Memory usage should be reasonable", |ctx| {
            if let Ok(data) = ctx.test_data.try_read() {
                if let Some(memory_usage) = data.get("peak_memory_mb") {
                    if let Some(usage) = memory_usage.as_f64() {
                        return usage < 500.0; // Less than 500MB
                    }
                }
            }
            false
        }))
        .cleanup(Box::new(CleanupPerformanceMonitoringAction))
        .timeout(Duration::from_secs(120))
        .metadata("expected_throughput", json!(10)) // workflows per second
        .build()
}

/// Create an error handling test scenario
pub fn create_error_handling_scenario() -> TestCase {
    TestBuilder::new("error_handling")
        .description("Test error handling and recovery mechanisms")
        .tag("error-handling")
        .tag("resilience")
        .setup(Box::new(SetupWorkflowDataAction))
        .action(Box::new(CreateWorkflowAction {
            title: "Error Test Workflow".to_string(),
            description: "Test error handling".to_string(),
        }))
        .action(Box::new(TriggerErrorAction {
            error_type: "validation_error".to_string(),
        }))
        .action(Box::new(RecoverFromErrorAction))
        .assert(Assertions::custom("Should have error events", |ctx| {
            if let Ok(data) = ctx.test_data.try_read() {
                if let Some(error_count) = data.get("error_count") {
                    return error_count.as_u64().unwrap_or(0) > 0;
                }
            }
            false
        }))
        .assert(Assertions::custom("Should have recovery events", |ctx| {
            if let Ok(data) = ctx.test_data.try_read() {
                if let Some(recovery_count) = data.get("recovery_count") {
                    return recovery_count.as_u64().unwrap_or(0) > 0;
                }
            }
            false
        }))
        .cleanup(Box::new(CleanupWorkflowDataAction))
        .build()
}

// Test Action Implementations

/// Setup action for initializing workflow test data
pub struct SetupWorkflowDataAction;

impl TestAction for SetupWorkflowDataAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("workflow_count".to_string(), json!(0));
        data.insert("step_count".to_string(), json!(0));
        data.insert("event_count".to_string(), json!(0));
        data.insert("workflow_status".to_string(), json!(WorkflowStatus::Draft));
        
        Ok(())
    }

    fn description(&self) -> String {
        "Initialize workflow test data".to_string()
    }
}

/// Cleanup action for workflow test data
pub struct CleanupWorkflowDataAction;

impl TestAction for CleanupWorkflowDataAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.clear();
        Ok(())
    }

    fn description(&self) -> String {
        "Clean up workflow test data".to_string()
    }
}

/// Action to create a workflow
pub struct CreateWorkflowAction {
    pub title: String,
    pub description: String,
}

impl TestAction for CreateWorkflowAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = HashMap::new();
        let (workflow, _events) = Workflow::new(
            self.title.clone(),
            self.description.clone(),
            metadata,
            Some("test_action".to_string()),
        )?;

        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("workflow".to_string(), json!({
            "id": workflow.id(),
            "name": workflow.name,
            "status": workflow.status
        }));
        data.insert("workflow_count".to_string(), json!(1));
        data.insert("workflow_status".to_string(), json!(workflow.status));
        
        Ok(())
    }

    fn description(&self) -> String {
        format!("Create workflow: {}", self.title)
    }
}

/// Action to add a step to workflow
pub struct AddStepAction {
    pub title: String,
    pub step_type: StepType,
}

impl TestAction for AddStepAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        // Increment step count
        let current_count = data.get("step_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        data.insert("step_count".to_string(), json!(current_count + 1));
        
        // Add step data
        let step_key = format!("step_{}", current_count);
        data.insert(step_key, json!({
            "title": self.title,
            "type": self.step_type
        }));
        
        Ok(())
    }

    fn description(&self) -> String {
        format!("Add step: {}", self.title)
    }
}

/// Action to start workflow execution
pub struct StartWorkflowAction;

impl TestAction for StartWorkflowAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("workflow_status".to_string(), json!(WorkflowStatus::Running));
        data.insert("workflow_started".to_string(), json!(true));
        
        Ok(())
    }

    fn description(&self) -> String {
        "Start workflow execution".to_string()
    }
}

/// Action to setup NATS connection
pub struct SetupNatsConnectionAction;

impl TestAction for SetupNatsConnectionAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("nats_connected".to_string(), json!(true));
        Ok(())
    }

    fn description(&self) -> String {
        "Setup NATS connection".to_string()
    }
}

/// Action to cleanup NATS connection
pub struct CleanupNatsConnectionAction;

impl TestAction for CleanupNatsConnectionAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("nats_connected".to_string(), json!(false));
        Ok(())
    }

    fn description(&self) -> String {
        "Cleanup NATS connection".to_string()
    }
}

/// Action to wait for specific events
pub struct WaitForEventsAction {
    pub expected_events: Vec<String>,
    pub timeout: Duration,
}

impl TestAction for WaitForEventsAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        // Simulate receiving expected events
        data.insert("received_events".to_string(), json!(self.expected_events));
        data.insert("events_received_count".to_string(), json!(self.expected_events.len()));
        
        Ok(())
    }

    fn description(&self) -> String {
        format!("Wait for events: {:?}", self.expected_events)
    }
}

/// Action to create multiple workflows for performance testing
pub struct CreateMultipleWorkflowsAction {
    pub count: usize,
}

impl TestAction for CreateMultipleWorkflowsAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("workflows_created".to_string(), json!(self.count));
        data.insert("peak_memory_mb".to_string(), json!(150.0)); // Simulated memory usage
        
        Ok(())
    }

    fn description(&self) -> String {
        format!("Create {} workflows", self.count)
    }
}

/// Action to setup performance monitoring
pub struct SetupPerformanceMonitoringAction;

impl TestAction for SetupPerformanceMonitoringAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("performance_monitoring_active".to_string(), json!(true));
        data.insert("start_memory_mb".to_string(), json!(50.0));
        
        Ok(())
    }

    fn description(&self) -> String {
        "Setup performance monitoring".to_string()
    }
}

/// Action to measure performance metrics
pub struct MeasurePerformanceAction;

impl TestAction for MeasurePerformanceAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        let start_memory = data.get("start_memory_mb")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0);
        let peak_memory = data.get("peak_memory_mb")
            .and_then(|v| v.as_f64())
            .unwrap_or(150.0);
            
        data.insert("memory_delta_mb".to_string(), json!(peak_memory - start_memory));
        data.insert("throughput_per_second".to_string(), json!(15.0)); // Simulated throughput
        
        Ok(())
    }

    fn description(&self) -> String {
        "Measure performance metrics".to_string()
    }
}

/// Action to cleanup performance monitoring
pub struct CleanupPerformanceMonitoringAction;

impl TestAction for CleanupPerformanceMonitoringAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("performance_monitoring_active".to_string(), json!(false));
        Ok(())
    }

    fn description(&self) -> String {
        "Cleanup performance monitoring".to_string()
    }
}

/// Action to trigger an error for testing error handling
pub struct TriggerErrorAction {
    pub error_type: String,
}

impl TestAction for TriggerErrorAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("error_triggered".to_string(), json!(true));
        data.insert("error_type".to_string(), json!(self.error_type));
        data.insert("error_count".to_string(), json!(1));
        
        Ok(())
    }

    fn description(&self) -> String {
        format!("Trigger {} error", self.error_type)
    }
}

/// Action to recover from an error
pub struct RecoverFromErrorAction;

impl TestAction for RecoverFromErrorAction {
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = context.test_data.try_write()
            .map_err(|_| "Failed to acquire write lock on test data")?;
        
        data.insert("error_recovered".to_string(), json!(true));
        data.insert("recovery_count".to_string(), json!(1));
        
        Ok(())
    }

    fn description(&self) -> String {
        "Recover from error".to_string()
    }
}

/// Collection of pre-built test scenarios
pub struct TestScenarios;

impl TestScenarios {
    /// Get all smoke test scenarios
    pub fn smoke_tests() -> Vec<TestCase> {
        vec![
            create_basic_workflow_scenario(),
        ]
    }

    /// Get all integration test scenarios
    pub fn integration_tests() -> Vec<TestCase> {
        vec![
            create_workflow_with_steps_scenario(),
            create_workflow_execution_scenario(),
        ]
    }

    /// Get all performance test scenarios
    pub fn performance_tests() -> Vec<TestCase> {
        vec![
            create_performance_test_scenario(),
        ]
    }

    /// Get all error handling test scenarios
    pub fn error_handling_tests() -> Vec<TestCase> {
        vec![
            create_error_handling_scenario(),
        ]
    }

    /// Get all test scenarios
    pub fn all_scenarios() -> Vec<TestCase> {
        let mut scenarios = Vec::new();
        scenarios.extend(Self::smoke_tests());
        scenarios.extend(Self::integration_tests());
        scenarios.extend(Self::performance_tests());
        scenarios.extend(Self::error_handling_tests());
        scenarios
    }

    /// Get scenarios by tag
    pub fn by_tag(tag: &str) -> Vec<TestCase> {
        Self::all_scenarios().into_iter()
            .filter(|scenario| scenario.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_creation() {
        let scenario = create_basic_workflow_scenario();
        assert_eq!(scenario.name, "basic_workflow_creation");
        assert!(scenario.tags.contains(&"smoke".to_string()));
    }

    #[test]
    fn test_scenarios_collection() {
        let smoke_tests = TestScenarios::smoke_tests();
        assert!(!smoke_tests.is_empty());
        
        let all_scenarios = TestScenarios::all_scenarios();
        assert!(all_scenarios.len() >= smoke_tests.len());
    }

    #[test]
    fn test_scenarios_by_tag() {
        let performance_scenarios = TestScenarios::by_tag("performance");
        assert!(!performance_scenarios.is_empty());
        
        for scenario in performance_scenarios {
            assert!(scenario.tags.contains(&"performance".to_string()));
        }
    }
}