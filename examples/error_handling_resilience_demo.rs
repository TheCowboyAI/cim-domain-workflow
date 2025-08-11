//! Error Handling and Resilience Demo
//!
//! Demonstrates comprehensive error handling, recovery strategies, circuit breakers,
//! bulkheads, timeouts, and observability features for production-ready workflows.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use cim_domain_workflow::error::{
    types::*,
    resilience::*,
    recovery::*,
    tracing::*,
};

/// Simulated external service for demonstration
struct ExternalService {
    name: String,
    failure_rate: f32,
    response_time: Duration,
}

impl ExternalService {
    fn new(name: String, failure_rate: f32, response_time: Duration) -> Self {
        Self {
            name,
            failure_rate,
            response_time,
        }
    }

    async fn call(&self, operation: &str) -> WorkflowResult<String> {
        // Simulate network delay
        tokio::time::sleep(self.response_time).await;

        // Simulate failures based on failure rate
        if rand::random::<f32>() < self.failure_rate {
            let context = ErrorContext::new(format!("{}_call", operation))
                .with_metadata("service".to_string(), serde_json::json!(self.name))
                .with_metadata("operation".to_string(), serde_json::json!(operation));

            return Err(WorkflowError::service_error(
                self.name.clone(),
                Some(operation.to_string()),
                Some(503),
                Some("Service temporarily unavailable".to_string()),
                context,
            ));
        }

        Ok(format!("Success from {} for {}", self.name, operation))
    }
}

/// Demo service that uses the resilience framework
struct WorkflowService {
    resilience_manager: ResilienceManager,
    recovery_manager: Arc<DefaultRecoveryManager>,
    tracer: WorkflowTracer,
    external_service: ExternalService,
}

impl WorkflowService {
    fn new() -> Self {
        let mut resilience_manager = ResilienceManager::new();

        // Configure circuit breakers
        resilience_manager.add_circuit_breaker(
            "external_service".to_string(),
            CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 2,
                timeout: Duration::from_secs(10),
                rolling_window: Duration::from_secs(60),
                half_open_max_calls: 2,
            },
        );

        // Configure bulkheads
        resilience_manager.add_bulkhead(
            "api_calls".to_string(),
            BulkheadConfig {
                max_concurrent: 5,
                queue_size: 10,
                acquire_timeout: Duration::from_secs(2),
            },
        );

        // Configure timeouts
        let mut timeout_config = TimeoutConfig::default();
        timeout_config.operation_timeouts.insert(
            "external_api_call".to_string(),
            Duration::from_secs(5),
        );
        resilience_manager.set_timeout_config(timeout_config);

        // Configure retry policies
        resilience_manager.add_retry_config(
            "external_service_call".to_string(),
            RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(5),
                backoff_multiplier: 2.0,
                jitter: true,
                retry_on: vec![
                    ErrorCategory::Network,
                    ErrorCategory::Infrastructure,
                    ErrorCategory::Dependency,
                ],
            },
        );

        let mut recovery_manager = DefaultRecoveryManager::new();

        // Add custom recovery strategies
        recovery_manager.add_strategy(
            ErrorCategory::Dependency,
            RecoveryStrategy::Fallback {
                fallback_operation: "cached_response".to_string(),
                parameters: HashMap::new(),
                timeout: Duration::from_secs(1),
            },
        );

        Self {
            resilience_manager,
            recovery_manager: Arc::new(recovery_manager),
            tracer: WorkflowTracer::new("workflow_service".to_string()),
            external_service: ExternalService::new(
                "payment_service".to_string(),
                0.3, // 30% failure rate for demonstration
                Duration::from_millis(100),
            ),
        }
    }

    /// Process payment with full resilience patterns
    async fn process_payment(&self, payment_id: Uuid, amount: f64) -> WorkflowResult<String> {
        let context = self.tracer.start_span("process_payment".to_string(), None).await;

        // Add tracing tags
        self.tracer.add_span_tag(
            context.span_id,
            "payment_id".to_string(),
            payment_id.to_string(),
        ).await;
        self.tracer.add_span_tag(
            context.span_id,
            "amount".to_string(),
            amount.to_string(),
        ).await;

        // Execute with all resilience patterns
        let result = self
            .resilience_manager
            .with_circuit_breaker("external_service", async {
                self.resilience_manager
                    .with_bulkhead("api_calls", async {
                        self.resilience_manager
                            .with_timeout("external_api_call", async {
                                self.resilience_manager
                                    .with_retry("external_service_call", || async {
                                        self.external_service.call("process_payment").await
                                    })
                                    .await
                            })
                            .await
                    })
                    .await
            })
            .await;

        // Handle result and finish span
        match result {
            Ok(response) => {
                self.tracer.add_span_log(
                    context.span_id,
                    LogLevel::Info,
                    "Payment processed successfully".to_string(),
                    vec![("response".to_string(), serde_json::json!(response))]
                        .into_iter()
                        .collect(),
                ).await;

                self.tracer.finish_span(context.span_id, SpanStatus::Success, None).await;
                Ok(response)
            }
            Err(error) => {
                self.tracer.add_span_log(
                    context.span_id,
                    LogLevel::Error,
                    "Payment processing failed".to_string(),
                    vec![("error".to_string(), serde_json::json!(error.message))]
                        .into_iter()
                        .collect(),
                ).await;

                // Attempt recovery
                let recovery_context = RecoveryContext {
                    original_error: error.clone(),
                    error_history: vec![],
                    system_metrics: HashMap::new(),
                    available_resources: HashMap::new(),
                    recovery_attempts: 0,
                };

                let recovery_result = self
                    .recovery_manager
                    .execute_recovery(&error, recovery_context)
                    .await;

                match recovery_result {
                    Ok(recovery) => {
                        if recovery.success {
                            self.tracer.add_span_log(
                                context.span_id,
                                LogLevel::Info,
                                "Recovery successful".to_string(),
                                vec![("recovery_details".to_string(), serde_json::json!(recovery.details))]
                                    .into_iter()
                                    .collect(),
                            ).await;

                            self.tracer.finish_span(context.span_id, SpanStatus::Success, None).await;
                            Ok("Payment processed via recovery".to_string())
                        } else {
                            self.tracer.finish_span(context.span_id, SpanStatus::Error, Some(error.clone())).await;
                            Err(error)
                        }
                    }
                    Err(recovery_error) => {
                        self.tracer.finish_span(context.span_id, SpanStatus::Error, Some(recovery_error.clone())).await;
                        Err(recovery_error)
                    }
                }
            }
        }
    }

    /// Get service health metrics
    async fn get_health_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut metrics = HashMap::new();

        // Circuit breaker metrics
        let cb_metrics = self.resilience_manager.get_circuit_breaker_metrics();
        metrics.insert("circuit_breakers".to_string(), serde_json::json!(cb_metrics));

        // Bulkhead metrics
        let bh_metrics = self.resilience_manager.get_bulkhead_metrics();
        metrics.insert("bulkheads".to_string(), serde_json::json!(bh_metrics));

        // Error statistics
        let error_stats = self.tracer.get_error_statistics().await;
        metrics.insert("error_statistics".to_string(), serde_json::json!(error_stats));

        // Error patterns
        let error_patterns = self.tracer.get_error_patterns().await;
        metrics.insert("error_patterns".to_string(), serde_json::json!(error_patterns));

        // Recovery system health
        let system_health = self.recovery_manager.get_system_health().await;
        metrics.insert("system_health".to_string(), serde_json::json!(system_health));

        metrics
    }
}

/// Demonstrate error handling and resilience patterns
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Error Handling and Resilience Demo");
    println!("=====================================");

    // Create workflow service with resilience patterns
    println!("\n📋 Step 1: Initialize service with resilience patterns");
    let service = WorkflowService::new();
    println!("✅ Service initialized with:");
    println!("   • Circuit breakers for external services");
    println!("   • Bulkheads for resource isolation");
    println!("   • Timeout management");
    println!("   • Retry policies with exponential backoff");
    println!("   • Error recovery strategies");
    println!("   • Distributed tracing and metrics");

    // Start monitoring
    println!("\n📋 Step 2: Start recovery monitoring");
    service.recovery_manager.start_monitoring().await;
    println!("✅ Recovery monitoring started");

    // Demonstrate successful operations
    println!("\n📋 Step 3: Process successful payments");
    for i in 1..=3 {
        let payment_id = Uuid::new_v4();
        let amount = 100.0 * i as f64;

        match service.process_payment(payment_id, amount).await {
            Ok(response) => println!("   ✅ Payment {}: {}", payment_id, response),
            Err(error) => println!("   ❌ Payment {}: {}", payment_id, error),
        }
    }

    // Demonstrate error scenarios and recovery
    println!("\n📋 Step 4: Simulate error scenarios with recovery");
    
    // Process multiple payments to trigger errors and recovery
    let mut successful_payments = 0;
    let mut failed_payments = 0;

    for i in 1..=10 {
        let payment_id = Uuid::new_v4();
        let amount = 50.0 * i as f64;

        println!("\n   Processing payment {} (${:.2}):", i, amount);
        match service.process_payment(payment_id, amount).await {
            Ok(response) => {
                successful_payments += 1;
                println!("     ✅ Success: {}", response);
            }
            Err(error) => {
                failed_payments += 1;
                println!("     ❌ Failed: {} (Category: {:?}, Severity: {:?})", 
                        error.message, error.category, error.severity);
                
                if error.is_recoverable() {
                    println!("     🔄 Error is recoverable - recovery strategies available");
                    if let Some(retry_policy) = error.retry_policy() {
                        println!("     📋 Retry policy: {} attempts with {:.1}s backoff", 
                                retry_policy.max_attempts, 
                                retry_policy.backoff_multiplier);
                    }
                }
            }
        }

        // Small delay between payments to observe patterns
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    println!("\n📊 Payment Processing Summary:");
    println!("   • Successful payments: {}", successful_payments);
    println!("   • Failed payments: {}", failed_payments);
    println!("   • Success rate: {:.1}%", 
            (successful_payments as f64 / (successful_payments + failed_payments) as f64) * 100.0);

    // Show health metrics
    println!("\n📋 Step 5: System health and metrics");
    let health_metrics = service.get_health_metrics().await;

    // Display circuit breaker metrics
    if let Some(cb_metrics) = health_metrics.get("circuit_breakers") {
        println!("\n🔌 Circuit Breaker Metrics:");
        if let serde_json::Value::Object(circuits) = cb_metrics {
            for (name, metrics) in circuits {
                println!("   Circuit '{}': {}", name, serde_json::to_string_pretty(metrics)?);
            }
        }
    }

    // Display bulkhead metrics
    if let Some(bh_metrics) = health_metrics.get("bulkheads") {
        println!("\n🛡️  Bulkhead Metrics:");
        if let serde_json::Value::Object(bulkheads) = bh_metrics {
            for (name, metrics) in bulkheads {
                println!("   Bulkhead '{}': {}", name, serde_json::to_string_pretty(metrics)?);
            }
        }
    }

    // Display error statistics
    if let Some(error_stats) = health_metrics.get("error_statistics") {
        println!("\n📈 Error Statistics:");
        println!("{}", serde_json::to_string_pretty(error_stats)?);
    }

    // Display error patterns
    if let Some(patterns) = health_metrics.get("error_patterns") {
        if let serde_json::Value::Array(pattern_list) = patterns {
            if !pattern_list.is_empty() {
                println!("\n🔍 Detected Error Patterns:");
                for pattern in pattern_list {
                    println!("   Pattern: {}", serde_json::to_string_pretty(pattern)?);
                }
            }
        }
    }

    // Display system health
    if let Some(system_health) = health_metrics.get("system_health") {
        println!("\n💚 System Health:");
        println!("{}", serde_json::to_string_pretty(system_health)?);
    }

    // Demonstrate manual error creation and analysis
    println!("\n📋 Step 6: Demonstrate error types and categorization");

    // Create various error types
    let errors = vec![
        WorkflowError::domain_validation(
            "payment".to_string(),
            "amount".to_string(),
            serde_json::json!(-100.0),
            "amount must be positive".to_string(),
            ErrorContext::new("validate_payment".to_string()),
        ),
        WorkflowError::network_error(
            "api.payment.com".to_string(),
            Some(443),
            "https".to_string(),
            Some(Duration::from_secs(30)),
            ErrorContext::new("external_api_call".to_string()),
        ),
        WorkflowError::template_error(
            "payment_workflow".to_string(),
            Some("validate_card".to_string()),
            Some("card_number".to_string()),
            "invalid format".to_string(),
            ErrorContext::new("template_instantiation".to_string()),
        ),
    ];

    for (i, error) in errors.iter().enumerate() {
        println!("\n   Error {}: {}", i + 1, error);
        println!("     Category: {:?}", error.category);
        println!("     Severity: {:?}", error.severity);
        println!("     Recoverable: {}", error.is_recoverable());
        
        if let Some(retry_policy) = error.retry_policy() {
            println!("     Retry policy: {} attempts, {:.1}x backoff",
                    retry_policy.max_attempts,
                    retry_policy.backoff_multiplier);
        }

        let recovery = error.recovery();
        println!("     Recovery actions: {} available", recovery.auto_recovery.len());
        for action in &recovery.manual_actions {
            println!("       • {}", action);
        }
    }

    println!("\n🎯 Error Handling and Resilience Demo Complete!");
    println!("\nThis demonstration showed:");
    println!("• Comprehensive error type system with categorization");
    println!("• Circuit breakers preventing cascading failures");
    println!("• Bulkheads for resource isolation");
    println!("• Timeout management with jitter");
    println!("• Retry policies with exponential backoff");
    println!("• Automatic error recovery strategies");
    println!("• Distributed tracing and correlation");
    println!("• Error pattern detection and analysis");
    println!("• System health monitoring and metrics");
    println!("• Production-ready observability features");

    Ok(())
}