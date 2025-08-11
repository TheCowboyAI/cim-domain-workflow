//! Comprehensive Observability and Monitoring Demo
//!
//! Demonstrates the complete observability suite including:
//! - Metrics collection and export
//! - Health monitoring and checks
//! - Alerting and notifications
//! - Distributed tracing
//! - Dashboard rendering

use cim_domain_workflow::error::types::{WorkflowError, ErrorContext, ErrorCategory, ErrorSeverity};
use cim_domain_workflow::observability::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Observability and Monitoring Demo");
    println!("=====================================\n");

    // Create observability suite
    let observability = Arc::new(ObservabilitySuite::new());
    
    // Initialize the suite
    println!("🚀 Initializing observability suite...");
    observability.initialize().await?;
    println!("✅ Observability suite initialized\n");

    // Demo 1: Metrics Collection
    println!("📊 Demo 1: Metrics Collection");
    println!("-----------------------------");
    demo_metrics_collection(&observability).await?;

    // Demo 2: Health Monitoring
    println!("\n🏥 Demo 2: Health Monitoring");
    println!("-----------------------------");
    demo_health_monitoring(&observability).await?;

    // Demo 3: Alerting System
    println!("\n🚨 Demo 3: Alerting System");
    println!("---------------------------");
    demo_alerting_system(&observability).await?;

    // Demo 4: Distributed Tracing
    println!("\n🔍 Demo 4: Distributed Tracing");
    println!("-------------------------------");
    demo_distributed_tracing(&observability).await?;

    // Demo 5: Dashboard Rendering
    println!("\n📈 Demo 5: Dashboard Rendering");
    println!("-------------------------------");
    demo_dashboard_rendering(&observability).await?;

    // Demo 6: Integrated Monitoring
    println!("\n🌟 Demo 6: Integrated Monitoring");
    println!("---------------------------------");
    demo_integrated_monitoring(&observability).await?;

    println!("\n✅ Observability demo completed successfully!");
    println!("All monitoring, metrics, alerting, and tracing systems are working correctly.");

    Ok(())
}

/// Demo metrics collection and export
async fn demo_metrics_collection(observability: &ObservabilitySuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating workflow metrics...");
    
    // Create workflow-specific metrics
    let workflow_metrics = WorkflowMetrics::new(&observability.metrics).await;
    
    // Simulate some workflow activity
    for i in 0..5 {
        println!("  📈 Simulating workflow execution {}", i + 1);
        
        // Start workflow
        workflow_metrics.workflows_started.increment().await;
        workflow_metrics.active_workflows.increment(1.0).await;
        
        // Simulate execution time
        let execution_timer = workflow_metrics.workflow_duration.start_timer();
        sleep(Duration::from_millis(100 + i * 20)).await;
        drop(execution_timer); // Timer records duration on drop
        
        // Complete workflow
        workflow_metrics.workflows_completed.increment().await;
        workflow_metrics.active_workflows.decrement(1.0).await;
        
        // Simulate template instantiation
        workflow_metrics.template_instantiations.increment().await;
        let template_timer = workflow_metrics.template_instantiation_duration.start_timer();
        sleep(Duration::from_millis(10)).await;
        drop(template_timer);
        
        // Simulate cross-domain events
        workflow_metrics.cross_domain_events.increment().await;
        workflow_metrics.nats_messages_processed.increment().await;
    }
    
    println!("  📊 Collecting and displaying metrics...");
    
    // Collect and display metrics
    let metrics = observability.metrics.collect_metrics().await;
    println!("  📈 Collected {} metrics:", metrics.len());
    
    for metric in metrics.iter().take(5) {
        match &metric.value {
            MetricValue::Counter(value) => {
                println!("    • {} (counter): {}", metric.name, value);
            }
            MetricValue::Gauge(value) => {
                println!("    • {} (gauge): {:.2}", metric.name, value);
            }
            MetricValue::Histogram { count, sum, .. } => {
                let avg = if *count > 0 { sum / (*count as f64) } else { 0.0 };
                println!("    • {} (histogram): count={}, avg={:.3}s", metric.name, count, avg);
            }
            _ => {}
        }
    }
    
    // Export metrics
    println!("  📤 Exporting metrics...");
    observability.metrics.export_metrics().await;
    
    Ok(())
}

/// Demo health monitoring
async fn demo_health_monitoring(observability: &ObservabilitySuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running health checks...");
    
    // Run all health checks
    let health_summary = observability.health_monitor.run_all_checks().await?;
    
    println!("  🏥 Health Summary:");
    println!("    • Overall Status: {:?}", health_summary.overall_status);
    println!("    • Health Score: {:.2}/1.0", health_summary.health_score);
    println!("    • Total Checks: {}", health_summary.total_checks);
    println!("    • Healthy: {}", health_summary.healthy_checks);
    println!("    • Degraded: {}", health_summary.degraded_checks);
    println!("    • Unhealthy: {}", health_summary.unhealthy_checks);
    println!("    • Uptime: {:?}", health_summary.uptime);
    
    println!("  📋 Individual Health Check Results:");
    for (component, result) in &health_summary.component_results {
        println!("    • {}: {:?} - {}", component, result.status, result.message);
        if !result.metrics.is_empty() {
            for (key, value) in &result.metrics {
                println!("      - {}: {:.2}", key, value);
            }
        }
    }
    
    // Run a specific health check
    println!("\n  🔍 Running specific health check...");
    let memory_result = observability.health_monitor.run_check("memory_usage").await?;
    println!("    Memory Check: {:?} - {}", memory_result.status, memory_result.message);
    
    Ok(())
}

/// Demo alerting system
async fn demo_alerting_system(observability: &ObservabilitySuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing alert system...");
    
    // Fire a manual alert
    println!("  🚨 Firing test alerts...");
    
    let mut alert_labels = HashMap::new();
    alert_labels.insert("severity".to_string(), "high".to_string());
    alert_labels.insert("component".to_string(), "workflow_engine".to_string());
    
    let mut alert_metadata = HashMap::new();
    alert_metadata.insert("threshold".to_string(), serde_json::json!(80.0));
    alert_metadata.insert("current_value".to_string(), serde_json::json!(92.5));
    
    let alert_id = observability.alert_manager.fire_alert(
        "High CPU Usage Alert".to_string(),
        "CPU usage has exceeded 90% for more than 5 minutes".to_string(),
        AlertSeverity::High,
        cim_domain_workflow::observability::alerts::AlertSource::Metric {
            metric_name: "cpu_usage_percent".to_string(),
            threshold: 80.0,
            actual_value: 92.5,
        },
        alert_labels,
        alert_metadata,
    ).await?;
    
    println!("    ✅ Alert fired with ID: {}", alert_id);
    
    // Create an error and fire an error alert
    let context = ErrorContext::new("demo_operation".to_string())
        .with_metadata("component".to_string(), serde_json::json!("workflow_processor"));
    
    let error = WorkflowError::network_error(
        "api.example.com".to_string(),
        Some(443),
        "https".to_string(),
        Some(Duration::from_secs(30)),
        context,
    );
    
    let error_alert_id = observability.alert_manager.fire_error_alert(&error).await?;
    println!("    ✅ Error alert fired with ID: {}", error_alert_id);
    
    // Simulate health check alert
    let health_result = HealthCheckResult {
        check_id: "simulated_failure".to_string(),
        component: "database".to_string(),
        status: HealthStatus::Unhealthy,
        message: "Database connection timeout".to_string(),
        execution_time: Duration::from_secs(5),
        timestamp: std::time::SystemTime::now(),
        metrics: HashMap::new(),
        error: None,
    };
    
    if let Some(health_alert_id) = observability.alert_manager.fire_health_alert(&health_result).await? {
        println!("    ✅ Health alert fired with ID: {}", health_alert_id);
    }
    
    // Check active alerts
    sleep(Duration::from_millis(100)).await; // Allow alerts to be processed
    let active_alerts = observability.alert_manager.get_active_alerts().await;
    println!("  📋 Active Alerts ({}):", active_alerts.len());
    
    for (id, alert) in active_alerts.iter().take(5) {
        println!("    • [{}] {}: {}", 
            alert.severity.to_string().to_uppercase(),
            alert.title, 
            alert.description
        );
        println!("      ID: {}, Status: {:?}, Created: {:?}", 
            id, alert.status, alert.created_at);
    }
    
    // Acknowledge and resolve first alert
    if !active_alerts.is_empty() {
        let first_alert_id = active_alerts.keys().next().unwrap();
        println!("  ✅ Acknowledging alert {}...", first_alert_id);
        observability.alert_manager.acknowledge_alert(*first_alert_id, "demo_user".to_string()).await?;
        
        println!("  ✅ Resolving alert {}...", first_alert_id);
        observability.alert_manager.resolve_alert(*first_alert_id, Some("Resolved by demo".to_string())).await?;
    }
    
    Ok(())
}

/// Demo distributed tracing
async fn demo_distributed_tracing(observability: &ObservabilitySuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Demonstrating distributed tracing...");
    
    // Create trace processor and exporter
    let exporter = Arc::new(ConsoleSpanExporter::new(true));
    let processor = Arc::new(BatchSpanProcessor::new(
        exporter,
        10,    // batch size
        Duration::from_secs(1), // batch timeout
        100,   // max queue size
    ));
    
    let processors: Vec<Arc<dyn SpanProcessor>> = vec![processor];
    
    // Get tracer
    let tracer = observability.trace_provider.get_tracer(
        "workflow_engine".to_string(),
        processors,
    ).await;
    
    // Start root span
    println!("  🔍 Starting distributed trace...");
    let root_span = tracer.start_span(
        "process_workflow".to_string(),
        SpanKind::Server,
        None,
    ).await;
    
    let root_context = root_span.context.clone();
    tracer.end_span(root_span).await;
    
    // Create child spans
    for i in 0..3 {
        let child_span = tracer.start_span(
            format!("workflow_step_{}", i + 1),
            SpanKind::Internal,
            Some(root_context.clone()),
        ).await;
        
        let child_context = child_span.context.clone();
        
        // Add attributes and events
        let mut span_with_data = child_span;
        span_with_data.set_attribute("step.id".to_string(), AttributeValue::String(format!("step_{}", i + 1)));
        span_with_data.set_attribute("step.type".to_string(), AttributeValue::String("processing".to_string()));
        span_with_data.set_attribute("user.id".to_string(), AttributeValue::String("user_123".to_string()));
        
        let mut event_attributes = HashMap::new();
        event_attributes.insert("operation".to_string(), AttributeValue::String("data_processing".to_string()));
        span_with_data.add_event("step_started".to_string(), event_attributes);
        
        // Simulate work
        sleep(Duration::from_millis(50)).await;
        
        // Create nested span
        let nested_span = tracer.start_span(
            format!("database_query_{}", i + 1),
            SpanKind::Client,
            Some(child_context.clone()),
        ).await;
        
        let mut nested_with_data = nested_span;
        nested_with_data.set_attribute("db.system".to_string(), AttributeValue::String("postgresql".to_string()));
        nested_with_data.set_attribute("db.operation".to_string(), AttributeValue::String("SELECT".to_string()));
        nested_with_data.set_attribute("db.table".to_string(), AttributeValue::String("workflows".to_string()));
        
        sleep(Duration::from_millis(20)).await;
        
        // End nested span
        tracer.end_span(nested_with_data).await;
        
        // Add completion event
        let mut completion_attrs = HashMap::new();
        completion_attrs.insert("result".to_string(), AttributeValue::String("success".to_string()));
        span_with_data.add_event("step_completed".to_string(), completion_attrs);
        
        // End child span
        tracer.end_span(span_with_data).await;
    }
    
    // Create an error span
    let error_span = tracer.start_span(
        "error_operation".to_string(),
        SpanKind::Internal,
        Some(root_context.clone()),
    ).await;
    
    let context = ErrorContext::new("trace_demo".to_string());
    let error = WorkflowError::new(
        ErrorCategory::Validation,
        ErrorSeverity::Warning,
        "Simulated validation error for tracing demo".to_string(),
        cim_domain_workflow::error::types::ErrorDetails::Generic {
            code: "DEMO_ERROR".to_string(),
            details: HashMap::new(),
        },
        context,
    );
    
    let mut error_span_with_data = error_span;
    error_span_with_data.record_exception(&error);
    tracer.end_span(error_span_with_data).await;
    
    // Get active spans
    let active_spans = tracer.get_active_spans().await;
    println!("  📊 Active spans: {}", active_spans.len());
    
    // Force flush to see all traces
    println!("  📤 Flushing traces...");
    tracer.force_flush().await?;
    
    Ok(())
}

/// Demo dashboard rendering
async fn demo_dashboard_rendering(observability: &ObservabilitySuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Demonstrating dashboard rendering...");
    
    // Create dashboard configurations
    let system_dashboard = DashboardTemplates::system_overview();
    let error_dashboard = DashboardTemplates::error_monitoring();
    
    println!("  📊 System Overview Dashboard: '{}'", system_dashboard.title);
    println!("    • Description: {}", system_dashboard.description);
    println!("    • Panels: {}", system_dashboard.panels.len());
    for panel in &system_dashboard.panels {
        println!("      - {}: {:?} ({}x{})", panel.title, panel.panel_type, panel.layout.width, panel.layout.height);
    }
    
    // Render dashboard data
    println!("\n  🔄 Rendering dashboard data...");
    let mut dashboard_renderer = observability.dashboard_renderer.write().await;
    let panel_data = dashboard_renderer.render_dashboard(&system_dashboard).await?;
    
    println!("  📈 Dashboard Data:");
    for (panel_id, data) in &panel_data {
        match data {
            PanelData::TimeSeries(series) => {
                println!("    • {}: {} time series", panel_id, series.len());
                for (i, ts) in series.iter().take(2).enumerate() {
                    println!("      - Series {}: '{}' ({} points)", i + 1, ts.name, ts.points.len());
                }
            }
            PanelData::SingleValue { value, unit, trend } => {
                let trend_str = trend.map(|t| format!(" (trend: {:+.1})", t)).unwrap_or_default();
                println!("    • {}: {:.2} {}{}", panel_id, value, unit, trend_str);
            }
            PanelData::Status { status, message, .. } => {
                println!("    • {}: {} - {}", panel_id, status, message);
            }
            PanelData::Table { columns, rows } => {
                println!("    • {}: Table with {} columns and {} rows", panel_id, columns.len(), rows.len());
            }
            PanelData::Alerts(alerts) => {
                println!("    • {}: {} alerts", panel_id, alerts.len());
            }
            PanelData::Text(content) => {
                let preview = content.chars().take(50).collect::<String>();
                println!("    • {}: Text panel ({}...)", panel_id, preview);
            }
        }
    }
    
    // Demo Grafana export
    println!("\n  📤 Exporting to Grafana format...");
    let grafana_exporter = GrafanaDashboardExporter::new(
        "http://localhost:3000".to_string(),
        "demo_api_key".to_string(),
    );
    
    let export_result = grafana_exporter.export_dashboard(&system_dashboard).await?;
    println!("    ✅ Dashboard exported to Grafana: {}", export_result);
    
    Ok(())
}

/// Demo integrated monitoring with all components working together
async fn demo_integrated_monitoring(observability: &ObservabilitySuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Demonstrating integrated monitoring scenario...");
    
    // Get system overview
    let overview = observability.get_system_overview().await;
    println!("  🌟 System Overview:");
    println!("    • Overall Health: {:?}", overview.health.overall_status);
    println!("    • Health Score: {:.2}", overview.health.health_score);
    println!("    • Active Alerts: {}", overview.active_alerts);
    println!("    • Total Metrics: {}", overview.total_metrics);
    println!("    • Uptime: {:?}", overview.uptime);
    
    // Simulate a complex workflow scenario
    println!("\n  🔄 Simulating complex workflow scenario...");
    
    // Get tracer for workflow simulation
    let tracer = observability.trace_provider.get_tracer(
        "integrated_demo".to_string(),
        vec![],
    ).await;
    
    // Start main workflow span
    let workflow_span = tracer.start_span(
        "complex_business_process".to_string(),
        SpanKind::Server,
        None,
    ).await;
    
    let workflow_context = workflow_span.context.clone();
    
    // Simulate workflow metrics
    let workflow_metrics = WorkflowMetrics::new(&observability.metrics).await;
    workflow_metrics.workflows_started.increment().await;
    workflow_metrics.active_workflows.increment(1.0).await;
    
    // Simulate various operations with tracing and metrics
    for step in ["validate", "process", "persist", "notify"] {
        println!("    • Executing step: {}", step);
        
        // Start step span
        let step_span = tracer.start_span(
            format!("workflow_step_{}", step),
            SpanKind::Internal,
            Some(workflow_context.clone()),
        ).await;
        
        // Record step metrics
        let step_timer = workflow_metrics.step_duration.start_timer();
        
        // Simulate different scenarios for each step
        match step {
            "validate" => {
                sleep(Duration::from_millis(50)).await;
            }
            "process" => {
                sleep(Duration::from_millis(100)).await;
                workflow_metrics.template_instantiations.increment().await;
            }
            "persist" => {
                sleep(Duration::from_millis(75)).await;
                // Simulate database health check
                let _ = observability.health_monitor.run_check("database_connectivity").await;
            }
            "notify" => {
                sleep(Duration::from_millis(25)).await;
                workflow_metrics.cross_domain_events.increment().await;
                workflow_metrics.nats_messages_processed.increment().await;
                
                // Simulate potential error in notification
                if rand::random::<f64>() > 0.7 {
                    let error_context = ErrorContext::new("notification_step".to_string())
                        .with_metadata("workflow_id".to_string(), serde_json::json!(workflow_context.trace_id));
                    
                    let notification_error = WorkflowError::network_error(
                        "notification.service.internal".to_string(),
                        Some(443),
                        "https".to_string(),
                        Some(Duration::from_secs(10)),
                        error_context,
                    );
                    
                    // Record error in span
                    let mut error_step_span = step_span;
                    error_step_span.record_exception(&notification_error);
                    tracer.end_span(error_step_span).await;
                    
                    // Fire alert for the error
                    let _ = observability.alert_manager.fire_error_alert(&notification_error).await;
                    
                    // Record error metrics
                    workflow_metrics.errors_by_category.increment().await;
                    
                    continue;
                }
            }
            _ => {}
        }
        
        drop(step_timer); // Record step duration
        tracer.end_span(step_span).await;
    }
    
    // Complete workflow
    workflow_metrics.workflows_completed.increment().await;
    workflow_metrics.active_workflows.decrement(1.0).await;
    tracer.end_span(workflow_span).await;
    
    // Final system status
    sleep(Duration::from_millis(100)).await; // Allow processing
    
    println!("\n  📊 Final System Status:");
    let final_overview = observability.get_system_overview().await;
    println!("    • Health Score: {:.2}", final_overview.health.health_score);
    println!("    • Active Alerts: {}", final_overview.active_alerts);
    println!("    • Total Metrics: {}", final_overview.total_metrics);
    
    // Show recent alerts
    let alert_history = observability.alert_manager.get_alert_history(Some(3)).await;
    if !alert_history.is_empty() {
        println!("  🚨 Recent Alerts:");
        for alert in &alert_history {
            println!("    • [{}] {}: {}", 
                alert.severity.to_string().to_uppercase(),
                alert.title,
                format!("{:?}", alert.status)
            );
        }
    }
    
    Ok(())
}