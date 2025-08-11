//! Integration testing framework for workflow operations
//!
//! This module provides comprehensive testing utilities including
//! test harnesses, mock services, test data generators, and assertion helpers.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub mod harness;
pub mod services;
pub mod generators;
pub mod assertions;
pub mod scenarios;
pub mod fixtures;

/// Test configuration for workflow testing
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Test environment name
    pub environment: String,
    /// Test timeout
    pub timeout: Duration,
    /// Enable detailed logging
    pub verbose_logging: bool,
    /// Test data directory
    pub data_directory: String,
    /// External service configurations
    pub service_configs: HashMap<String, ServiceConfig>,
    /// Test parallelism level
    pub parallel_tests: usize,
    /// Enable performance monitoring during tests
    pub performance_monitoring: bool,
    /// Test result reporting format
    pub reporting_format: ReportFormat,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            environment: "test".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes
            verbose_logging: false,
            data_directory: "tests/data".to_string(),
            service_configs: HashMap::new(),
            parallel_tests: num_cpus::get(),
            performance_monitoring: false,
            reporting_format: ReportFormat::JUnit,
        }
    }
}

/// External service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: ServiceType,
    /// Service endpoint
    pub endpoint: String,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Additional configuration parameters
    pub config_params: HashMap<String, String>,
}

/// Types of external services
#[derive(Debug, Clone)]
pub enum ServiceType {
    /// NATS message broker
    Nats,
    /// HTTP REST service
    HttpRest,
    /// Database service
    Database,
    /// Custom service type
    Custom(String),
}

/// Test reporting formats
#[derive(Debug, Clone)]
pub enum ReportFormat {
    /// JUnit XML format
    JUnit,
    /// TAP (Test Anything Protocol)
    TAP,
    /// JSON format
    JSON,
    /// HTML format
    HTML,
    /// Console output
    Console,
}

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test case identifier
    pub test_id: String,
    /// Test name
    pub test_name: String,
    /// Test status
    pub status: TestStatus,
    /// Execution duration
    pub duration: Duration,
    /// Start time
    pub start_time: SystemTime,
    /// End time
    pub end_time: SystemTime,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Test output/logs
    pub output: Vec<String>,
    /// Performance metrics
    pub performance_metrics: Option<TestPerformanceMetrics>,
    /// Test assertions
    pub assertions: Vec<AssertionResult>,
    /// Test metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Test execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test was skipped
    Skipped,
    /// Test timed out
    Timeout,
    /// Test execution error
    Error,
}

/// Performance metrics collected during test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPerformanceMetrics {
    /// Memory usage at start
    pub memory_start: u64,
    /// Memory usage at end
    pub memory_end: u64,
    /// Peak memory usage
    pub memory_peak: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Database queries executed
    pub db_queries: u32,
    /// HTTP requests made
    pub http_requests: u32,
    /// Events published
    pub events_published: u32,
    /// Events received
    pub events_received: u32,
}

/// Test assertion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    /// Assertion type
    pub assertion_type: String,
    /// Assertion description
    pub description: String,
    /// Whether assertion passed
    pub passed: bool,
    /// Expected value (if applicable)
    pub expected: Option<serde_json::Value>,
    /// Actual value (if applicable)
    pub actual: Option<serde_json::Value>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

/// Test suite result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    /// Suite name
    pub suite_name: String,
    /// Total test count
    pub total_tests: usize,
    /// Passed tests
    pub passed_tests: usize,
    /// Failed tests
    pub failed_tests: usize,
    /// Skipped tests
    pub skipped_tests: usize,
    /// Error tests
    pub error_tests: usize,
    /// Timeout tests
    pub timeout_tests: usize,
    /// Total execution time
    pub total_duration: Duration,
    /// Individual test results
    pub test_results: Vec<TestResult>,
    /// Suite-level performance metrics
    pub performance_summary: Option<TestSuitePerformanceMetrics>,
    /// Test coverage information
    pub coverage_info: Option<TestCoverageInfo>,
}

/// Test suite performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuitePerformanceMetrics {
    /// Average test execution time
    pub avg_execution_time: Duration,
    /// Fastest test time
    pub fastest_test: Duration,
    /// Slowest test time
    pub slowest_test: Duration,
    /// Total memory used
    pub total_memory_used: u64,
    /// Peak memory usage across all tests
    pub peak_memory_usage: u64,
    /// Average CPU usage
    pub avg_cpu_usage: f64,
    /// Total database operations
    pub total_db_operations: u32,
    /// Total HTTP requests
    pub total_http_requests: u32,
    /// Total events processed
    pub total_events_processed: u32,
}

/// Test coverage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageInfo {
    /// Lines covered
    pub lines_covered: u32,
    /// Total lines
    pub total_lines: u32,
    /// Coverage percentage
    pub coverage_percentage: f64,
    /// Uncovered modules
    pub uncovered_modules: Vec<String>,
    /// Coverage by module
    pub module_coverage: HashMap<String, f64>,
}

/// Test execution context
pub struct TestExecutionContext {
    /// Test configuration
    pub config: TestConfig,
    /// External service connections
    pub service_connections: Arc<RwLock<HashMap<String, Box<dyn TestService>>>>,
    /// Test data storage
    pub test_data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Performance monitors
    pub performance_monitors: Arc<RwLock<Vec<Box<dyn TestPerformanceMonitor>>>>,
    /// Test output buffer
    pub output_buffer: Arc<RwLock<Vec<String>>>,
    /// Test start time
    pub start_time: SystemTime,
}

/// Trait for test services
#[async_trait::async_trait]
pub trait TestService: Send + Sync {
    /// Connect to the service
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Disconnect from the service
    async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Check if service is healthy/available
    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>>;
    
    /// Get service endpoint
    fn endpoint(&self) -> String;
    
    /// Get service statistics
    async fn statistics(&self) -> TestServiceStatistics;
    
    /// Reset/clean service state for testing
    async fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Test service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestServiceStatistics {
    /// Number of operations performed
    pub operations_count: u64,
    /// Number of errors encountered
    pub errors_count: u64,
    /// Average operation time
    pub avg_operation_time: Duration,
    /// Connection uptime
    pub uptime: Duration,
    /// Service-specific metrics
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

/// Trait for test performance monitoring
pub trait TestPerformanceMonitor: Send + Sync {
    /// Start monitoring
    fn start(&mut self);
    
    /// Stop monitoring and get metrics
    fn stop(&mut self) -> TestPerformanceMetrics;
    
    /// Get current metrics
    fn current_metrics(&self) -> TestPerformanceMetrics;
}

/// Test builder for creating complex test scenarios
pub struct TestBuilder {
    /// Test name
    name: String,
    /// Test description
    description: String,
    /// Test tags
    tags: Vec<String>,
    /// Setup actions
    setup_actions: Vec<Box<dyn TestAction>>,
    /// Test actions
    test_actions: Vec<Box<dyn TestAction>>,
    /// Cleanup actions
    cleanup_actions: Vec<Box<dyn TestAction>>,
    /// Test assertions
    assertions: Vec<Box<dyn TestAssertion>>,
    /// Test timeout
    timeout: Option<Duration>,
    /// Test metadata
    metadata: HashMap<String, serde_json::Value>,
}

impl TestBuilder {
    /// Create a new test builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            tags: Vec::new(),
            setup_actions: Vec::new(),
            test_actions: Vec::new(),
            cleanup_actions: Vec::new(),
            assertions: Vec::new(),
            timeout: None,
            metadata: HashMap::new(),
        }
    }

    /// Set test description
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Add test tag
    pub fn tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Add setup action
    pub fn setup(mut self, action: Box<dyn TestAction>) -> Self {
        self.setup_actions.push(action);
        self
    }

    /// Add test action
    pub fn action(mut self, action: Box<dyn TestAction>) -> Self {
        self.test_actions.push(action);
        self
    }

    /// Add cleanup action
    pub fn cleanup(mut self, action: Box<dyn TestAction>) -> Self {
        self.cleanup_actions.push(action);
        self
    }

    /// Add assertion
    pub fn assert(mut self, assertion: Box<dyn TestAssertion>) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Set test timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Build the test case
    pub fn build(self) -> TestCase {
        TestCase {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.name,
            description: self.description,
            tags: self.tags,
            setup_actions: self.setup_actions,
            test_actions: self.test_actions,
            cleanup_actions: self.cleanup_actions,
            assertions: self.assertions,
            timeout: self.timeout.unwrap_or(Duration::from_secs(60)),
            metadata: self.metadata,
        }
    }
}

/// A complete test case
pub struct TestCase {
    /// Test ID
    pub id: String,
    /// Test name
    pub name: String,
    /// Test description
    pub description: String,
    /// Test tags
    pub tags: Vec<String>,
    /// Setup actions
    pub setup_actions: Vec<Box<dyn TestAction>>,
    /// Test actions
    pub test_actions: Vec<Box<dyn TestAction>>,
    /// Cleanup actions
    pub cleanup_actions: Vec<Box<dyn TestAction>>,
    /// Test assertions
    pub assertions: Vec<Box<dyn TestAssertion>>,
    /// Test timeout
    pub timeout: Duration,
    /// Test metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl std::fmt::Debug for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestCase")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("tags", &self.tags)
            .field("timeout", &self.timeout)
            .field("metadata", &self.metadata)
            .field("setup_actions_count", &self.setup_actions.len())
            .field("test_actions_count", &self.test_actions.len())
            .field("cleanup_actions_count", &self.cleanup_actions.len())
            .field("assertions_count", &self.assertions.len())
            .finish()
    }
}

/// Trait for test actions
pub trait TestAction: Send + Sync {
    /// Execute the action
    fn execute(&self, context: &TestExecutionContext) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get action description
    fn description(&self) -> String;
}

/// Trait for test assertions
pub trait TestAssertion: Send + Sync {
    /// Execute the assertion
    fn assert(&self, context: &TestExecutionContext) -> AssertionResult;
    
    /// Get assertion description
    fn description(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = TestConfig::default();
        assert_eq!(config.environment, "test");
        assert_eq!(config.timeout, Duration::from_secs(300));
        assert!(!config.verbose_logging);
    }

    #[test]
    fn test_builder_pattern() {
        let test_case = TestBuilder::new("test_workflow_creation")
            .description("Test workflow creation functionality")
            .tag("unit")
            .tag("workflow")
            .timeout(Duration::from_secs(30))
            .metadata("priority", serde_json::Value::String("high".to_string()))
            .build();

        assert_eq!(test_case.name, "test_workflow_creation");
        assert_eq!(test_case.description, "Test workflow creation functionality");
        assert_eq!(test_case.tags.len(), 2);
        assert_eq!(test_case.timeout, Duration::from_secs(30));
        assert!(test_case.metadata.contains_key("priority"));
    }

    #[test]
    fn test_result_serialization() {
        let result = TestResult {
            test_id: "test-1".to_string(),
            test_name: "Test Case 1".to_string(),
            status: TestStatus::Passed,
            duration: Duration::from_millis(100),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            error_message: None,
            output: vec!["Test output".to_string()],
            performance_metrics: None,
            assertions: vec![],
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&result);
        assert!(serialized.is_ok());
    }
}