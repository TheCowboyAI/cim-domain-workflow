//! Test fixtures for setting up consistent test environments

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Test fixture for consistent test data setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFixture {
    /// Fixture name
    pub name: String,
    /// Fixture description
    pub description: String,
    /// Test data
    pub data: HashMap<String, Value>,
    /// Setup instructions
    pub setup: Vec<String>,
    /// Teardown instructions
    pub teardown: Vec<String>,
    /// Dependencies on other fixtures
    pub dependencies: Vec<String>,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            data: HashMap::new(),
            setup: Vec::new(),
            teardown: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set fixture description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Add test data
    pub fn with_data(mut self, key: &str, value: Value) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }

    /// Add setup instruction
    pub fn with_setup(mut self, instruction: &str) -> Self {
        self.setup.push(instruction.to_string());
        self
    }

    /// Add teardown instruction
    pub fn with_teardown(mut self, instruction: &str) -> Self {
        self.teardown.push(instruction.to_string());
        self
    }

    /// Add dependency
    pub fn with_dependency(mut self, fixture_name: &str) -> Self {
        self.dependencies.push(fixture_name.to_string());
        self
    }
}

/// Collection of predefined test fixtures
pub struct TestFixtures;

impl TestFixtures {
    /// Get basic workflow fixture
    pub fn basic_workflow() -> TestFixture {
        TestFixture::new("basic_workflow")
            .with_description("Basic workflow test data")
            .with_data("workflow_title", Value::String("Test Workflow".to_string()))
            .with_data("workflow_description", Value::String("A test workflow for unit testing".to_string()))
            .with_data("expected_status", Value::String("Draft".to_string()))
            .with_setup("Initialize workflow context")
            .with_setup("Set up test database")
            .with_teardown("Clean up test workflow")
            .with_teardown("Reset database state")
    }

    /// Get workflow with steps fixture
    pub fn workflow_with_steps() -> TestFixture {
        TestFixture::new("workflow_with_steps")
            .with_description("Workflow with predefined steps")
            .with_dependency("basic_workflow")
            .with_data("steps", serde_json::json!([
                {
                    "title": "Initial Step",
                    "description": "First step in the workflow",
                    "type": "Manual",
                    "timeout_minutes": 30
                },
                {
                    "title": "Processing Step",
                    "description": "Automated processing step",
                    "type": "Automated",
                    "timeout_minutes": 10
                },
                {
                    "title": "Approval Step",
                    "description": "Manual approval step",
                    "type": "Manual",
                    "timeout_minutes": 60,
                    "approval_required": "manager"
                }
            ]))
            .with_setup("Create workflow steps")
            .with_setup("Configure step dependencies")
            .with_teardown("Remove created steps")
    }

    /// Get NATS connection fixture
    pub fn nats_connection() -> TestFixture {
        TestFixture::new("nats_connection")
            .with_description("NATS message broker connection")
            .with_data("nats_url", Value::String("localhost:4222".to_string()))
            .with_data("connection_timeout", Value::Number(10.into()))
            .with_data("test_subjects", serde_json::json!([
                "workflow.events.test",
                "workflow.commands.test",
                "workflow.queries.test"
            ]))
            .with_setup("Connect to NATS server")
            .with_setup("Create test subjects")
            .with_teardown("Clean up test subjects")
            .with_teardown("Disconnect from NATS")
    }

    /// Get performance test fixture
    pub fn performance_test() -> TestFixture {
        TestFixture::new("performance_test")
            .with_description("Performance testing environment")
            .with_data("workflow_count", Value::Number(100.into()))
            .with_data("concurrent_workflows", Value::Number(10.into()))
            .with_data("max_execution_time_seconds", Value::Number(30.into()))
            .with_data("memory_limit_mb", Value::Number(512.into()))
            .with_data("expected_throughput_per_second", Value::Number(20.into()))
            .with_setup("Initialize performance monitoring")
            .with_setup("Set resource limits")
            .with_setup("Prepare test data")
            .with_teardown("Collect performance metrics")
            .with_teardown("Clean up test workflows")
            .with_teardown("Reset system state")
    }

    /// Get error handling fixture
    pub fn error_handling() -> TestFixture {
        TestFixture::new("error_handling")
            .with_description("Error handling test scenarios")
            .with_data("error_scenarios", serde_json::json!([
                {
                    "type": "ValidationError",
                    "trigger": "invalid_workflow_data",
                    "expected_recovery": true
                },
                {
                    "type": "TimeoutError",
                    "trigger": "step_timeout",
                    "expected_recovery": true
                },
                {
                    "type": "ServiceUnavailable",
                    "trigger": "external_service_down",
                    "expected_recovery": false
                }
            ]))
            .with_data("retry_attempts", Value::Number(3.into()))
            .with_data("retry_delay_ms", Value::Number(1000.into()))
            .with_setup("Configure error scenarios")
            .with_setup("Set up error injection")
            .with_teardown("Reset error conditions")
            .with_teardown("Verify error recovery")
    }

    /// Get cross-domain workflow fixture
    pub fn cross_domain_workflow() -> TestFixture {
        TestFixture::new("cross_domain_workflow")
            .with_description("Cross-domain workflow coordination")
            .with_dependency("nats_connection")
            .with_data("source_domain", Value::String("workflow".to_string()))
            .with_data("target_domains", serde_json::json!([
                "inventory",
                "billing",
                "notification"
            ]))
            .with_data("coordination_timeout_seconds", Value::Number(30.into()))
            .with_data("expected_responses", Value::Number(3.into()))
            .with_setup("Configure domain routing")
            .with_setup("Set up cross-domain subjects")
            .with_teardown("Clean up domain subscriptions")
            .with_teardown("Reset routing configuration")
    }

    /// Get database fixture
    pub fn database() -> TestFixture {
        TestFixture::new("database")
            .with_description("Database test environment")
            .with_data("database_url", Value::String("sqlite::memory:".to_string()))
            .with_data("migration_scripts", serde_json::json!([
                "001_create_workflows_table.sql",
                "002_create_workflow_steps_table.sql",
                "003_create_workflow_events_table.sql"
            ]))
            .with_data("test_data_files", serde_json::json!([
                "test_workflows.json",
                "test_steps.json"
            ]))
            .with_setup("Create test database")
            .with_setup("Run migrations")
            .with_setup("Load test data")
            .with_teardown("Clean test data")
            .with_teardown("Drop test database")
    }

    /// Get integration test fixture
    pub fn integration_test() -> TestFixture {
        TestFixture::new("integration_test")
            .with_description("Full integration test environment")
            .with_dependency("database")
            .with_dependency("nats_connection")
            .with_data("services", serde_json::json!([
                {
                    "name": "workflow_engine",
                    "port": 8080,
                    "health_endpoint": "/health"
                },
                {
                    "name": "event_processor",
                    "port": 8081,
                    "health_endpoint": "/health"
                }
            ]))
            .with_data("api_base_url", Value::String("http://localhost:8080/api/v1".to_string()))
            .with_data("health_check_timeout_seconds", Value::Number(5.into()))
            .with_setup("Start test services")
            .with_setup("Wait for services to be ready")
            .with_setup("Configure service routing")
            .with_teardown("Stop test services")
            .with_teardown("Clean up service state")
    }

    /// Get observability fixture
    pub fn observability() -> TestFixture {
        TestFixture::new("observability")
            .with_description("Observability and monitoring test setup")
            .with_data("metrics_enabled", Value::Bool(true))
            .with_data("tracing_enabled", Value::Bool(true))
            .with_data("log_level", Value::String("debug".to_string()))
            .with_data("metrics_port", Value::Number(9090.into()))
            .with_data("expected_metrics", serde_json::json!([
                "workflow_created_total",
                "workflow_completed_total",
                "workflow_duration_seconds",
                "workflow_errors_total"
            ]))
            .with_setup("Initialize metrics collection")
            .with_setup("Configure trace sampling")
            .with_setup("Set up log capture")
            .with_teardown("Export collected metrics")
            .with_teardown("Flush trace data")
            .with_teardown("Archive logs")
    }

    /// Get all available fixtures
    pub fn all() -> Vec<TestFixture> {
        vec![
            Self::basic_workflow(),
            Self::workflow_with_steps(),
            Self::nats_connection(),
            Self::performance_test(),
            Self::error_handling(),
            Self::cross_domain_workflow(),
            Self::database(),
            Self::integration_test(),
            Self::observability(),
        ]
    }

    /// Get fixtures by name
    pub fn by_names(names: &[&str]) -> Vec<TestFixture> {
        let all_fixtures = Self::all();
        let name_set: std::collections::HashSet<&str> = names.iter().cloned().collect();
        
        all_fixtures.into_iter()
            .filter(|fixture| name_set.contains(fixture.name.as_str()))
            .collect()
    }

    /// Resolve fixture dependencies
    pub fn resolve_dependencies(fixtures: Vec<TestFixture>) -> Result<Vec<TestFixture>, String> {
        let mut resolved = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();
        
        let fixture_map: HashMap<String, TestFixture> = fixtures.into_iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        fn visit(
            fixture_name: &str,
            fixture_map: &HashMap<String, TestFixture>,
            resolved: &mut Vec<TestFixture>,
            visited: &mut std::collections::HashSet<String>,
            visiting: &mut std::collections::HashSet<String>,
        ) -> Result<(), String> {
            if visiting.contains(fixture_name) {
                return Err(format!("Circular dependency detected involving: {}", fixture_name));
            }
            
            if visited.contains(fixture_name) {
                return Ok(());
            }

            visiting.insert(fixture_name.to_string());

            if let Some(fixture) = fixture_map.get(fixture_name) {
                // Visit dependencies first
                for dep in &fixture.dependencies {
                    visit(dep, fixture_map, resolved, visited, visiting)?;
                }
                
                resolved.push(fixture.clone());
                visited.insert(fixture_name.to_string());
            } else {
                return Err(format!("Fixture not found: {}", fixture_name));
            }

            visiting.remove(fixture_name);
            Ok(())
        }

        for fixture_name in fixture_map.keys() {
            visit(fixture_name, &fixture_map, &mut resolved, &mut visited, &mut visiting)?;
        }

        Ok(resolved)
    }
}

/// Fixture manager for handling fixture lifecycle
pub struct FixtureManager {
    active_fixtures: HashMap<String, TestFixture>,
}

impl FixtureManager {
    /// Create a new fixture manager
    pub fn new() -> Self {
        Self {
            active_fixtures: HashMap::new(),
        }
    }

    /// Load fixtures
    pub fn load_fixtures(&mut self, fixtures: Vec<TestFixture>) -> Result<(), String> {
        let resolved = TestFixtures::resolve_dependencies(fixtures)?;
        
        for fixture in resolved {
            self.active_fixtures.insert(fixture.name.clone(), fixture);
        }
        
        Ok(())
    }

    /// Setup all loaded fixtures
    pub async fn setup_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Fixtures are already in dependency order, so we can set them up in sequence
        for fixture in self.active_fixtures.values() {
            self.setup_fixture(fixture).await?;
        }
        Ok(())
    }

    /// Teardown all loaded fixtures
    pub async fn teardown_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Teardown in reverse order
        let mut fixtures: Vec<_> = self.active_fixtures.values().collect();
        fixtures.reverse();
        
        for fixture in fixtures {
            self.teardown_fixture(fixture).await?;
        }
        Ok(())
    }

    /// Setup a specific fixture
    async fn setup_fixture(&self, fixture: &TestFixture) -> Result<(), Box<dyn std::error::Error>> {
        println!("Setting up fixture: {} - {}", fixture.name, fixture.description);
        
        for instruction in &fixture.setup {
            println!("  Executing setup: {}", instruction);
            // In a real implementation, we would execute the setup instruction
            // For now, we'll just simulate it
        }
        
        Ok(())
    }

    /// Teardown a specific fixture
    async fn teardown_fixture(&self, fixture: &TestFixture) -> Result<(), Box<dyn std::error::Error>> {
        println!("Tearing down fixture: {}", fixture.name);
        
        for instruction in &fixture.teardown {
            println!("  Executing teardown: {}", instruction);
            // In a real implementation, we would execute the teardown instruction
        }
        
        Ok(())
    }

    /// Get fixture data
    pub fn get_fixture_data(&self, fixture_name: &str) -> Option<&HashMap<String, Value>> {
        self.active_fixtures.get(fixture_name).map(|f| &f.data)
    }

    /// Get all fixture data combined
    pub fn get_all_data(&self) -> HashMap<String, Value> {
        let mut combined = HashMap::new();
        
        for fixture in self.active_fixtures.values() {
            for (key, value) in &fixture.data {
                // Prefix with fixture name to avoid conflicts
                let prefixed_key = format!("{}_{}", fixture.name, key);
                combined.insert(prefixed_key, value.clone());
            }
        }
        
        combined
    }
}

impl Default for FixtureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creation() {
        let fixture = TestFixtures::basic_workflow();
        assert_eq!(fixture.name, "basic_workflow");
        assert!(!fixture.description.is_empty());
        assert!(fixture.data.contains_key("workflow_title"));
    }

    #[test]
    fn test_fixture_builder() {
        let fixture = TestFixture::new("test")
            .with_description("Test fixture")
            .with_data("key", Value::String("value".to_string()))
            .with_setup("Setup instruction")
            .with_teardown("Teardown instruction")
            .with_dependency("other_fixture");

        assert_eq!(fixture.name, "test");
        assert_eq!(fixture.description, "Test fixture");
        assert_eq!(fixture.data.get("key"), Some(&Value::String("value".to_string())));
        assert_eq!(fixture.setup.len(), 1);
        assert_eq!(fixture.teardown.len(), 1);
        assert_eq!(fixture.dependencies.len(), 1);
    }

    #[test]
    fn test_dependency_resolution() {
        let fixtures = vec![
            TestFixtures::workflow_with_steps(), // depends on basic_workflow
            TestFixtures::basic_workflow(),
        ];

        let resolved = TestFixtures::resolve_dependencies(fixtures);
        assert!(resolved.is_ok());

        let resolved = resolved.unwrap();
        assert_eq!(resolved.len(), 2);
        // basic_workflow should come first
        assert_eq!(resolved[0].name, "basic_workflow");
        assert_eq!(resolved[1].name, "workflow_with_steps");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let fixture_a = TestFixture::new("a").with_dependency("b");
        let fixture_b = TestFixture::new("b").with_dependency("a");
        let fixtures = vec![fixture_a, fixture_b];

        let result = TestFixtures::resolve_dependencies(fixtures);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[tokio::test]
    async fn test_fixture_manager() {
        let mut manager = FixtureManager::new();
        let fixtures = vec![TestFixtures::basic_workflow()];
        
        let result = manager.load_fixtures(fixtures);
        assert!(result.is_ok());
        
        let setup_result = manager.setup_all().await;
        assert!(setup_result.is_ok());
        
        let teardown_result = manager.teardown_all().await;
        assert!(teardown_result.is_ok());
    }

    #[test]
    fn test_fixture_data_retrieval() {
        let mut manager = FixtureManager::new();
        let fixtures = vec![TestFixtures::basic_workflow()];
        
        manager.load_fixtures(fixtures).unwrap();
        
        let data = manager.get_fixture_data("basic_workflow");
        assert!(data.is_some());
        assert!(data.unwrap().contains_key("workflow_title"));
        
        let all_data = manager.get_all_data();
        assert!(all_data.contains_key("basic_workflow_workflow_title"));
    }
}