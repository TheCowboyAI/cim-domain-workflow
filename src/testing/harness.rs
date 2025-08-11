//! Test harness for executing workflow tests

use super::{
    TestConfig, TestExecutionContext, TestCase, TestResult, TestStatus, TestSuiteResult,
    TestSuitePerformanceMetrics, TestService, AssertionResult,
    TestPerformanceMonitor, ReportFormat,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;

/// Main test harness for executing workflow tests
pub struct TestHarness {
    /// Test configuration
    config: TestConfig,
    /// External service connections
    service_connections: Arc<RwLock<HashMap<String, Box<dyn TestService>>>>,
    /// Performance monitors
    performance_monitors: Arc<RwLock<Vec<Box<dyn TestPerformanceMonitor>>>>,
    /// Test results
    results: Arc<RwLock<Vec<TestResult>>>,
    /// Test execution statistics
    stats: Arc<RwLock<TestExecutionStats>>,
}

/// Test execution statistics
#[derive(Debug, Default)]
struct TestExecutionStats {
    /// Total tests executed
    total_executed: usize,
    /// Total test time
    total_time: Duration,
    /// Memory usage tracking
    memory_usage: Vec<u64>,
    /// CPU usage tracking
    cpu_usage: Vec<f64>,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            service_connections: Arc::new(RwLock::new(HashMap::new())),
            performance_monitors: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(TestExecutionStats::default())),
        }
    }

    /// Register a test service
    pub async fn register_service(&self, name: String, service: Box<dyn TestService>) {
        let mut services = self.service_connections.write().await;
        services.insert(name, service);
    }

    /// Add a performance monitor
    pub async fn add_performance_monitor(&self, monitor: Box<dyn TestPerformanceMonitor>) {
        let mut monitors = self.performance_monitors.write().await;
        monitors.push(monitor);
    }

    /// Execute a single test case
    pub async fn execute_test(&self, test_case: TestCase) -> TestResult {
        let test_id = test_case.id.clone();
        let test_name = test_case.name.clone();
        let start_time = SystemTime::now();
        let execution_start = Instant::now();

        if self.config.verbose_logging {
            println!("🚀 Executing test: {}", test_name);
        }

        // Create execution context
        let context = TestExecutionContext {
            config: self.config.clone(),
            service_connections: self.service_connections.clone(),
            test_data: Arc::new(RwLock::new(HashMap::new())),
            performance_monitors: self.performance_monitors.clone(),
            output_buffer: Arc::new(RwLock::new(Vec::new())),
            start_time,
        };

        // Start performance monitoring
        let mut performance_monitors = self.performance_monitors.write().await;
        for monitor in performance_monitors.iter_mut() {
            monitor.start();
        }
        drop(performance_monitors);

        // Execute test with timeout
        let test_timeout = test_case.timeout;
        let execution_result = timeout(test_timeout, self.execute_test_internal(test_case, &context)).await;

        // Stop performance monitoring and collect metrics
        let mut performance_monitors = self.performance_monitors.write().await;
        let performance_metrics = if !performance_monitors.is_empty() {
            Some(performance_monitors[0].stop())
        } else {
            None
        };
        drop(performance_monitors);

        let end_time = SystemTime::now();
        let duration = execution_start.elapsed();

        // Process execution result
        let (status, error_message, assertions) = match execution_result {
            Ok(Ok(assertions)) => (TestStatus::Passed, None, assertions),
            Ok(Err(e)) => (TestStatus::Failed, Some(e.to_string()), vec![]),
            Err(_) => (TestStatus::Timeout, Some("Test execution timed out".to_string()), vec![]),
        };

        // Get test output
        let output = context.output_buffer.read().await.clone();

        // Create test result
        let result = TestResult {
            test_id,
            test_name: test_name.clone(),
            status: status.clone(),
            duration,
            start_time,
            end_time,
            error_message,
            output,
            performance_metrics,
            assertions,
            metadata: HashMap::new(),
        };

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_executed += 1;
            stats.total_time += duration;
            
            // Track memory usage (simulated)
            let memory_usage = (duration.as_millis() * 1024) as u64; // Rough estimate
            stats.memory_usage.push(memory_usage);
            
            // Track CPU usage (simulated)
            let cpu_usage = (duration.as_secs_f64() * 0.1).min(1.0); // Rough estimate
            stats.cpu_usage.push(cpu_usage);
            
            // Keep only recent measurements (last 1000)
            if stats.memory_usage.len() > 1000 {
                stats.memory_usage.remove(0);
            }
            if stats.cpu_usage.len() > 1000 {
                stats.cpu_usage.remove(0);
            }
        }

        // Store result
        {
            let mut results = self.results.write().await;
            results.push(result.clone());
        }

        if self.config.verbose_logging {
            match status {
                TestStatus::Passed => println!("✅ Test passed: {}", test_name),
                TestStatus::Failed => println!("❌ Test failed: {}", test_name),
                TestStatus::Timeout => println!("⏰ Test timed out: {}", test_name),
                _ => println!("⚠️  Test completed with status {:?}: {}", status, test_name),
            }
        }

        result
    }

    /// Internal test execution
    async fn execute_test_internal(
        &self,
        test_case: TestCase,
        context: &TestExecutionContext,
    ) -> Result<Vec<AssertionResult>, Box<dyn std::error::Error>> {
        let mut output_buffer = context.output_buffer.write().await;
        output_buffer.push(format!("Starting test: {}", test_case.name));
        drop(output_buffer);

        // Execute setup actions
        for action in &test_case.setup_actions {
            if let Err(e) = action.execute(context) {
                return Err(format!("Setup action failed: {}", e).into());
            }
        }

        // Execute test actions
        for action in &test_case.test_actions {
            if let Err(e) = action.execute(context) {
                // Execute cleanup actions before returning error
                for cleanup_action in &test_case.cleanup_actions {
                    let _ = cleanup_action.execute(context);
                }
                return Err(format!("Test action failed: {}", e).into());
            }
        }

        // Execute assertions
        let mut assertion_results = Vec::new();
        for assertion in &test_case.assertions {
            let result = assertion.assert(context);
            assertion_results.push(result);
        }

        // Check if any assertions failed
        let failed_assertions: Vec<_> = assertion_results.iter()
            .filter(|r| !r.passed)
            .collect();

        if !failed_assertions.is_empty() {
            let failure_messages: Vec<_> = failed_assertions.iter()
                .map(|a| format!("{}: expected {:?}, got {:?}", 
                    a.description, a.expected, a.actual))
                .collect();
            
            // Execute cleanup actions before returning error
            for cleanup_action in &test_case.cleanup_actions {
                let _ = cleanup_action.execute(context);
            }
            
            return Err(format!("Assertions failed: {}", failure_messages.join("; ")).into());
        }

        // Execute cleanup actions
        for cleanup_action in &test_case.cleanup_actions {
            if let Err(e) = cleanup_action.execute(context) {
                let mut output_buffer = context.output_buffer.write().await;
                output_buffer.push(format!("Cleanup action warning: {}", e));
                drop(output_buffer);
            }
        }

        let mut output_buffer = context.output_buffer.write().await;
        output_buffer.push(format!("Completed test: {}", test_case.name));
        drop(output_buffer);

        Ok(assertion_results)
    }

    /// Execute a test suite
    pub async fn execute_suite(&self, suite_name: String, test_cases: Vec<TestCase>) -> TestSuiteResult {
        let suite_start = Instant::now();
        let mut test_results = Vec::new();

        if self.config.verbose_logging {
            println!("📋 Executing test suite: {} ({} tests)", suite_name, test_cases.len());
        }

        // Connect to test services
        self.connect_services().await;

        // Execute tests
        if self.config.parallel_tests > 1 && test_cases.len() > 1 {
            // Parallel execution
            test_results = self.execute_tests_parallel(test_cases).await;
        } else {
            // Sequential execution
            for test_case in test_cases {
                let result = self.execute_test(test_case).await;
                test_results.push(result);
            }
        }

        // Disconnect from test services
        self.disconnect_services().await;

        let total_duration = suite_start.elapsed();

        // Calculate statistics
        let total_tests = test_results.len();
        let passed_tests = test_results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed_tests = test_results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let skipped_tests = test_results.iter().filter(|r| r.status == TestStatus::Skipped).count();
        let error_tests = test_results.iter().filter(|r| r.status == TestStatus::Error).count();
        let timeout_tests = test_results.iter().filter(|r| r.status == TestStatus::Timeout).count();

        // Calculate performance summary
        let performance_summary = self.calculate_performance_summary(&test_results).await;

        let result = TestSuiteResult {
            suite_name: suite_name.clone(),
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            error_tests,
            timeout_tests,
            total_duration,
            test_results,
            performance_summary,
            coverage_info: None, // TODO: Implement coverage collection
        };

        if self.config.verbose_logging {
            self.print_suite_summary(&result);
        }

        result
    }

    /// Execute tests in parallel
    async fn execute_tests_parallel(&self, test_cases: Vec<TestCase>) -> Vec<TestResult> {
        use futures::stream::{self, StreamExt};

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.parallel_tests));
        let results = stream::iter(test_cases)
            .map(|test_case| {
                let semaphore = semaphore.clone();
                let harness = self;
                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    harness.execute_test(test_case).await
                }
            })
            .buffer_unordered(self.config.parallel_tests)
            .collect::<Vec<_>>()
            .await;

        results
    }

    /// Connect to all test services
    async fn connect_services(&self) {
        let mut services = self.service_connections.write().await;
        for (name, service) in services.iter_mut() {
            if let Err(e) = service.connect().await {
                eprintln!("Failed to connect to service {}: {}", name, e);
            } else if self.config.verbose_logging {
                println!("🔧 Connected to service: {}", name);
            }
        }
    }

    /// Disconnect from all test services
    async fn disconnect_services(&self) {
        let mut services = self.service_connections.write().await;
        for (name, service) in services.iter_mut() {
            if let Err(e) = service.disconnect().await {
                eprintln!("Failed to disconnect from service {}: {}", name, e);
            } else if self.config.verbose_logging {
                println!("🛑 Disconnected from service: {}", name);
            }
        }
    }

    /// Calculate performance summary for the test suite
    async fn calculate_performance_summary(&self, test_results: &[TestResult]) -> Option<TestSuitePerformanceMetrics> {
        if test_results.is_empty() {
            return None;
        }

        let durations: Vec<Duration> = test_results.iter().map(|r| r.duration).collect();
        let avg_execution_time = Duration::from_nanos(
            durations.iter().map(|d| d.as_nanos()).sum::<u128>() as u64 / durations.len() as u64
        );

        let fastest_test = *durations.iter().min().unwrap_or(&Duration::from_nanos(0));
        let slowest_test = *durations.iter().max().unwrap_or(&Duration::from_nanos(0));

        let mut total_memory_used = 0u64;
        let mut peak_memory_usage = 0u64;
        let mut total_cpu_usage = 0.0f64;
        let mut total_db_operations = 0u32;
        let mut total_http_requests = 0u32;
        let mut total_events_processed = 0u32;
        let mut metric_count = 0;

        for result in test_results {
            if let Some(ref metrics) = result.performance_metrics {
                total_memory_used += metrics.memory_end.saturating_sub(metrics.memory_start);
                peak_memory_usage = peak_memory_usage.max(metrics.memory_peak);
                total_cpu_usage += metrics.cpu_usage;
                total_db_operations += metrics.db_queries;
                total_http_requests += metrics.http_requests;
                total_events_processed += metrics.events_published + metrics.events_received;
                metric_count += 1;
            }
        }

        let avg_cpu_usage = if metric_count > 0 {
            total_cpu_usage / metric_count as f64
        } else {
            0.0
        };

        Some(TestSuitePerformanceMetrics {
            avg_execution_time,
            fastest_test,
            slowest_test,
            total_memory_used,
            peak_memory_usage,
            avg_cpu_usage,
            total_db_operations,
            total_http_requests,
            total_events_processed,
        })
    }

    /// Print test suite summary
    fn print_suite_summary(&self, result: &TestSuiteResult) {
        println!("\n📊 Test Suite Summary: {}", result.suite_name);
        println!("═══════════════════════════════════════");
        println!("✅ Passed:  {} / {}", result.passed_tests, result.total_tests);
        println!("❌ Failed:  {}", result.failed_tests);
        println!("⏭️  Skipped: {}", result.skipped_tests);
        println!("💥 Errors:  {}", result.error_tests);
        println!("⏰ Timeouts: {}", result.timeout_tests);
        println!("⏱️  Duration: {:?}", result.total_duration);

        if let Some(ref perf) = result.performance_summary {
            println!("\n🚀 Performance Summary:");
            println!("   Average execution: {:?}", perf.avg_execution_time);
            println!("   Fastest test: {:?}", perf.fastest_test);
            println!("   Slowest test: {:?}", perf.slowest_test);
            println!("   Peak memory: {:.2} MB", perf.peak_memory_usage as f64 / 1024.0 / 1024.0);
            println!("   Average CPU: {:.2}%", perf.avg_cpu_usage);
        }

        let success_rate = (result.passed_tests as f64 / result.total_tests as f64) * 100.0;
        println!("\n🎯 Success Rate: {:.1}%", success_rate);

        if result.failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for test_result in &result.test_results {
                if test_result.status == TestStatus::Failed {
                    println!("   - {}", test_result.test_name);
                    if let Some(ref error) = test_result.error_message {
                        println!("     Error: {}", error);
                    }
                }
            }
        }
    }

    /// Generate test report in specified format
    pub async fn generate_report(&self, suite_result: &TestSuiteResult) -> String {
        match self.config.reporting_format {
            ReportFormat::JUnit => self.generate_junit_report(suite_result).await,
            ReportFormat::JSON => self.generate_json_report(suite_result).await,
            ReportFormat::TAP => self.generate_tap_report(suite_result).await,
            ReportFormat::HTML => self.generate_html_report(suite_result).await,
            ReportFormat::Console => self.generate_console_report(suite_result).await,
        }
    }

    /// Generate JUnit XML report
    async fn generate_junit_report(&self, suite_result: &TestSuiteResult) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<testsuite name="{}" tests="{}" failures="{}" errors="{}" skipped="{}" time="{:.3}">"#,
            suite_result.suite_name,
            suite_result.total_tests,
            suite_result.failed_tests,
            suite_result.error_tests,
            suite_result.skipped_tests,
            suite_result.total_duration.as_secs_f64()
        ));
        xml.push('\n');

        for test_result in &suite_result.test_results {
            xml.push_str(&format!(
                r#"  <testcase name="{}" classname="{}" time="{:.3}">"#,
                test_result.test_name,
                suite_result.suite_name,
                test_result.duration.as_secs_f64()
            ));
            xml.push('\n');

            match test_result.status {
                TestStatus::Failed => {
                    if let Some(ref error) = test_result.error_message {
                        xml.push_str(&format!(r#"    <failure message="{}">{}</failure>"#, error, error));
                        xml.push('\n');
                    }
                }
                TestStatus::Error => {
                    if let Some(ref error) = test_result.error_message {
                        xml.push_str(&format!(r#"    <error message="{}">{}</error>"#, error, error));
                        xml.push('\n');
                    }
                }
                TestStatus::Skipped => {
                    xml.push_str(r#"    <skipped/>"#);
                    xml.push('\n');
                }
                _ => {}
            }

            xml.push_str("  </testcase>");
            xml.push('\n');
        }

        xml.push_str("</testsuite>");
        xml.push('\n');

        xml
    }

    /// Generate JSON report
    async fn generate_json_report(&self, suite_result: &TestSuiteResult) -> String {
        serde_json::to_string_pretty(suite_result).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate TAP report
    async fn generate_tap_report(&self, suite_result: &TestSuiteResult) -> String {
        let mut tap = String::new();
        tap.push_str(&format!("1..{}\n", suite_result.total_tests));

        for (i, test_result) in suite_result.test_results.iter().enumerate() {
            let test_number = i + 1;
            match test_result.status {
                TestStatus::Passed => {
                    tap.push_str(&format!("ok {} - {}\n", test_number, test_result.test_name));
                }
                TestStatus::Failed => {
                    tap.push_str(&format!("not ok {} - {}", test_number, test_result.test_name));
                    if let Some(ref error) = test_result.error_message {
                        tap.push_str(&format!(" # {}", error));
                    }
                    tap.push('\n');
                }
                TestStatus::Skipped => {
                    tap.push_str(&format!("ok {} - {} # SKIP\n", test_number, test_result.test_name));
                }
                _ => {
                    tap.push_str(&format!("not ok {} - {} # ERROR\n", test_number, test_result.test_name));
                }
            }
        }

        tap
    }

    /// Generate HTML report
    async fn generate_html_report(&self, suite_result: &TestSuiteResult) -> String {
        let success_rate = (suite_result.passed_tests as f64 / suite_result.total_tests as f64) * 100.0;
        
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .header {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .summary {{ display: flex; gap: 20px; margin: 20px 0; }}
        .metric {{ background: #e9e9e9; padding: 15px; border-radius: 5px; text-align: center; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        .skipped {{ color: orange; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Test Report: {}</h1>
        <p>Generated: {}</p>
        <p>Success Rate: {:.1}%</p>
    </div>
    
    <div class="summary">
        <div class="metric">
            <h3>Total Tests</h3>
            <div>{}</div>
        </div>
        <div class="metric passed">
            <h3>Passed</h3>
            <div>{}</div>
        </div>
        <div class="metric failed">
            <h3>Failed</h3>
            <div>{}</div>
        </div>
        <div class="metric skipped">
            <h3>Skipped</h3>
            <div>{}</div>
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>Test Name</th>
                <th>Status</th>
                <th>Duration</th>
                <th>Error Message</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>"#,
            suite_result.suite_name,
            suite_result.suite_name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            success_rate,
            suite_result.total_tests,
            suite_result.passed_tests,
            suite_result.failed_tests,
            suite_result.skipped_tests,
            suite_result.test_results.iter().map(|r| format!(
                "<tr><td>{}</td><td class=\"{}\">{:?}</td><td>{:.3}s</td><td>{}</td></tr>",
                r.test_name,
                match r.status {
                    TestStatus::Passed => "passed",
                    TestStatus::Failed => "failed",
                    TestStatus::Skipped => "skipped",
                    _ => "error"
                },
                r.status,
                r.duration.as_secs_f64(),
                r.error_message.as_deref().unwrap_or("")
            )).collect::<Vec<_>>().join("\n            ")
        )
    }

    /// Generate console report
    async fn generate_console_report(&self, suite_result: &TestSuiteResult) -> String {
        let mut report = String::new();
        report.push_str(&format!("Test Suite: {}\n", suite_result.suite_name));
        report.push_str(&format!("Total: {}, Passed: {}, Failed: {}, Skipped: {}\n", 
            suite_result.total_tests, suite_result.passed_tests, 
            suite_result.failed_tests, suite_result.skipped_tests));
        report.push_str(&format!("Duration: {:?}\n", suite_result.total_duration));
        
        for test_result in &suite_result.test_results {
            report.push_str(&format!("  {} [{:?}] {:?}\n", 
                test_result.test_name, test_result.status, test_result.duration));
            if let Some(ref error) = test_result.error_message {
                report.push_str(&format!("    Error: {}\n", error));
            }
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestBuilder;

    #[tokio::test]
    async fn test_harness_creation() {
        let config = TestConfig::default();
        let harness = TestHarness::new(config);
        
        let stats = harness.stats.read().await;
        assert_eq!(stats.total_executed, 0);
    }

    #[tokio::test]
    async fn test_suite_execution() {
        let config = TestConfig {
            verbose_logging: false,
            ..TestConfig::default()
        };
        let harness = TestHarness::new(config);

        let test_cases = vec![
            TestBuilder::new("test_1")
                .description("First test")
                .build(),
            TestBuilder::new("test_2")
                .description("Second test")
                .build(),
        ];

        let result = harness.execute_suite("test_suite".to_string(), test_cases).await;
        
        assert_eq!(result.suite_name, "test_suite");
        assert_eq!(result.total_tests, 2);
    }
}