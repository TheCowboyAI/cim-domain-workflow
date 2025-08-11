//! Performance monitoring and optimization demonstration
//! 
//! This example showcases the comprehensive performance monitoring,
//! profiling, optimization, and memory management capabilities.

use cim_domain_workflow::{
    aggregate::Workflow,
    value_objects::{WorkflowContext, StepType},
    performance::{
        PerformanceMonitor, PerformanceThresholds, PerformanceMetrics,
        profiler::{Profiler, ProfilerConfig},
        optimizer::{PerformanceOptimizer, OptimizerConfig},
        memory::{MemoryManager, MemoryConfig},
        metrics::{MetricsCollector, MetricsConfig},
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CIM Workflow Performance Monitoring Demo");
    println!("============================================\n");

    // Initialize performance monitoring components
    let performance_monitor = Arc::new(PerformanceMonitor::new(PerformanceThresholds::default()));
    let profiler = Arc::new(Profiler::new(performance_monitor.clone(), ProfilerConfig::default()));
    let optimizer = Arc::new(PerformanceOptimizer::new(performance_monitor.clone(), OptimizerConfig::default()));
    let memory_manager = Arc::new(MemoryManager::new(MemoryConfig::default()));
    let metrics_collector = Arc::new(MetricsCollector::new(MetricsConfig::default()));

    // Initialize memory manager
    memory_manager.initialize().await;

    println!("✅ Initialized performance monitoring components\n");

    // Demonstrate basic profiling
    demonstrate_profiling(&profiler).await?;
    
    // Demonstrate metrics collection
    demonstrate_metrics_collection(&metrics_collector).await?;
    
    // Demonstrate memory management
    demonstrate_memory_management(&memory_manager).await?;
    
    // Demonstrate performance monitoring
    demonstrate_performance_monitoring(&performance_monitor).await?;
    
    // Demonstrate optimization
    demonstrate_optimization(&optimizer).await?;
    
    // Generate comprehensive performance report
    generate_performance_report(&performance_monitor, &memory_manager, &metrics_collector).await?;

    println!("🎉 Performance monitoring demonstration completed!");
    
    Ok(())
}

/// Demonstrate profiling capabilities
async fn demonstrate_profiling(profiler: &Arc<Profiler>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demonstrating Profiling Capabilities");
    println!("----------------------------------------");

    // Profile workflow creation
    let handle = profiler.start_profiling("workflow_creation").await;
    
    handle.checkpoint("validation_start").await;
    
    // Simulate workflow creation
    let context = WorkflowContext::new();
    let (mut workflow, _events) = Workflow::new(
        "Performance Test Workflow".to_string(),
        "Workflow for performance testing".to_string(),
        context,
        Some("demo".to_string()),
    )?;
    
    handle.checkpoint("workflow_created").await;
    
    // Add steps to the workflow
    for i in 1..=5 {
        let _result = workflow.add_step(
            format!("Step {}", i),
            format!("Performance test step {}", i),
            StepType::Automated,
            Default::default(),
            vec![],
            Some(5),
            None,
            Some("demo".to_string()),
        );
        
        handle.checkpoint(&format!("step_{}_added", i)).await;
    }
    
    handle.checkpoint("all_steps_added").await;
    
    // Finish profiling
    if let Some(result) = handle.finish().await {
        println!("✅ Profiling completed:");
        println!("   Operation: {}", result.operation);
        println!("   Total Duration: {:?}", result.total_duration);
        println!("   Memory Delta: {} bytes", result.memory_delta);
        println!("   Checkpoints: {}", result.checkpoints.len());
        
        for checkpoint in &result.checkpoints {
            println!("   - {}: {:?}", checkpoint.name, checkpoint.elapsed);
        }
    }
    
    println!();
    Ok(())
}

/// Demonstrate metrics collection
async fn demonstrate_metrics_collection(collector: &Arc<MetricsCollector>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 Demonstrating Metrics Collection");
    println!("-----------------------------------");

    // Record various metrics
    let mut labels = HashMap::new();
    labels.insert("operation".to_string(), "demo".to_string());
    labels.insert("environment".to_string(), "test".to_string());

    // Time series metrics
    for i in 1..=20 {
        let value = 100.0 + (i as f64 * 10.0) + (rand::random::<f64>() * 20.0 - 10.0);
        collector.record_time_series("response_time_ms", value, labels.clone()).await;
        
        let memory_value = 50.0 + (i as f64 * 5.0) + (rand::random::<f64>() * 10.0 - 5.0);
        collector.record_time_series("memory_usage_mb", memory_value, labels.clone()).await;
        
        sleep(Duration::from_millis(10)).await;
    }

    // Histogram metrics (latency distribution)
    let latency_values = [0.1, 0.2, 0.15, 0.8, 1.2, 0.3, 0.25, 2.1, 0.4, 0.6, 1.8, 0.9, 0.35, 1.5, 0.7];
    for &latency in &latency_values {
        collector.record_histogram("request_latency_seconds", latency).await;
    }

    // Counter metrics
    for i in 1..=10 {
        collector.increment_counter("requests_total", labels.clone()).await;
        if i % 4 == 0 {
            let mut error_labels = labels.clone();
            error_labels.insert("status".to_string(), "error".to_string());
            collector.increment_counter("requests_total", error_labels).await;
        }
    }

    // Gauge metrics
    collector.set_gauge("active_workflows", 15.0, labels.clone()).await;
    collector.set_gauge("cpu_usage_percent", 67.5, labels.clone()).await;

    // Analyze collected metrics
    if let Some(series) = collector.get_time_series("response_time_ms").await {
        println!("✅ Time series metrics collected:");
        println!("   Metric: {}", series.name);
        println!("   Data points: {}", series.points.len());
        
        if let Some(latest) = series.points.back() {
            println!("   Latest value: {:.2}", latest.value);
        }
    }

    if let Some(histogram) = collector.get_histogram("request_latency_seconds").await {
        println!("✅ Histogram metrics collected:");
        println!("   Metric: {}", histogram.name);
        println!("   Total samples: {}", histogram.count);
        println!("   Min: {:.3}s, Max: {:.3}s", histogram.min, histogram.max);
        println!("   Average: {:.3}s", histogram.sum / histogram.count as f64);
        
        // Calculate percentiles
        if let Some(percentiles) = collector.calculate_percentiles("request_latency_seconds", &[50.0, 90.0, 95.0, 99.0]).await {
            println!("   Percentiles:");
            for (p, value) in &percentiles {
                println!("     {}: {:.3}s", p, value);
            }
        }
    }

    println!();
    Ok(())
}

/// Demonstrate memory management
async fn demonstrate_memory_management(memory_manager: &Arc<MemoryManager>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Demonstrating Memory Management");
    println!("----------------------------------");

    let mut allocation_ids = Vec::new();

    // Allocate memory in different pools
    println!("📝 Allocating memory in various pools...");
    
    // Workflow allocations
    for i in 1..=10 {
        if let Some(id) = memory_manager.allocate("workflows", 1024 * i as u64, "Workflow").await {
            allocation_ids.push(("workflows".to_string(), id, 1024 * i as u64));
        }
    }
    
    // Event allocations
    for i in 1..=20 {
        if let Some(id) = memory_manager.allocate("events", 512 * i as u64, "Event").await {
            allocation_ids.push(("events".to_string(), id, 512 * i as u64));
        }
    }
    
    // Context allocations
    for i in 1..=5 {
        if let Some(id) = memory_manager.allocate("context", 2048 * i as u64, "Context").await {
            allocation_ids.push(("context".to_string(), id, 2048 * i as u64));
        }
    }

    // Display memory statistics
    let stats = memory_manager.get_statistics().await;
    println!("✅ Memory allocation completed:");
    println!("   Total used: {} bytes", stats.total_used);
    println!("   Peak usage: {} bytes", stats.peak_usage);
    println!("   Active allocations: {}", stats.active_allocations);
    println!("   Memory pressure: {:.2}%", stats.memory_pressure * 100.0);

    for (pool_name, pool_stats) in &stats.pool_statistics {
        println!("   Pool '{}': {} bytes used ({:.1}% utilization)", 
                pool_name, pool_stats.used_memory, pool_stats.utilization * 100.0);
    }

    // Trigger garbage collection
    println!("🧹 Triggering garbage collection...");
    memory_manager.trigger_gc("all", cim_domain_workflow::performance::memory::GcPriority::Normal).await;
    let gc_result = memory_manager.run_gc().await;
    
    if gc_result.success {
        println!("✅ Garbage collection completed:");
        println!("   Duration: {:?}", gc_result.duration);
        println!("   Memory freed: {} bytes", gc_result.memory_freed);
        println!("   Objects collected: {}", gc_result.objects_collected);
    }

    // Deallocate some memory
    println!("📤 Deallocating some allocations...");
    for (pool_name, allocation_id, size) in allocation_ids.iter().step_by(2) {
        memory_manager.deallocate(pool_name, allocation_id, *size).await;
    }

    let final_stats = memory_manager.get_statistics().await;
    println!("✅ Final memory statistics:");
    println!("   Total used: {} bytes", final_stats.total_used);
    println!("   Active allocations: {}", final_stats.active_allocations);
    
    println!();
    Ok(())
}

/// Demonstrate performance monitoring
async fn demonstrate_performance_monitoring(monitor: &Arc<PerformanceMonitor>) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Demonstrating Performance Monitoring");
    println!("--------------------------------------");

    // Record performance metrics for various operations
    let operations = [
        ("workflow_creation", 50..150, 1024..4096),
        ("step_execution", 10..80, 512..2048),
        ("event_processing", 5..30, 256..1024),
        ("context_update", 2..15, 128..512),
    ];

    for (operation, duration_range, memory_range) in &operations {
        for i in 1..=10 {
            let duration_ms = rand::random::<u64>() % (duration_range.end - duration_range.start) + duration_range.start;
            let memory_delta = (rand::random::<u64>() % (memory_range.end - memory_range.start) + memory_range.start) as i64;
            
            let metric = PerformanceMetrics {
                operation: operation.to_string(),
                duration: Duration::from_millis(duration_ms),
                memory_delta,
                cpu_usage: rand::random::<f64>() * 30.0 + 10.0, // 10-40% CPU
                timestamp: SystemTime::now(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("iteration".to_string(), serde_json::Value::Number(i.into()));
                    map.insert("demo".to_string(), serde_json::Value::Bool(true));
                    map
                },
            };
            
            monitor.record_metric(metric).await;
        }
    }

    // Display performance profiles
    println!("✅ Performance metrics recorded for {} operations", operations.len());
    
    for (operation_name, _, _) in &operations {
        if let Some(profile) = monitor.get_profile(operation_name).await {
            println!("📊 Operation: {}", profile.operation);
            println!("   Executions: {}", profile.execution_count);
            println!("   Avg Duration: {:?}", profile.avg_duration);
            println!("   Min Duration: {:?}", profile.min_duration);
            println!("   Max Duration: {:?}", profile.max_duration);
            println!("   Avg Memory: {} bytes", profile.avg_memory);
            println!("   Error Rate: {:.2}%", profile.error_rate * 100.0);
        }
    }

    // Check performance thresholds
    let alerts = monitor.check_thresholds().await;
    if !alerts.is_empty() {
        println!("⚠️  Performance alerts:");
        for alert in &alerts {
            println!("   - {}: {}", 
                match alert.severity {
                    cim_domain_workflow::performance::AlertSeverity::Critical => "🔴 CRITICAL",
                    cim_domain_workflow::performance::AlertSeverity::Warning => "🟡 WARNING",
                    cim_domain_workflow::performance::AlertSeverity::Info => "🔵 INFO",
                },
                alert.message
            );
        }
    } else {
        println!("✅ All performance metrics within acceptable thresholds");
    }

    println!();
    Ok(())
}

/// Demonstrate optimization capabilities
async fn demonstrate_optimization(optimizer: &Arc<PerformanceOptimizer>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Demonstrating Performance Optimization");
    println!("-----------------------------------------");

    // Get current parameters
    let initial_params = optimizer.get_parameters().await;
    println!("📋 Initial optimization parameters:");
    println!("   Thread pool size: {}", initial_params.thread_pool_size);
    println!("   Buffer sizes: {:?}", initial_params.buffer_sizes);
    println!("   Cache settings: {} entries", initial_params.cache_settings.len());

    // Run optimization
    println!("🚀 Running optimization analysis...");
    if let Some(optimization_run) = optimizer.optimize().await {
        println!("✅ Optimization completed:");
        println!("   Run ID: {}", optimization_run.run_id);
        println!("   Strategy: {:?}", optimization_run.strategy);
        println!("   Changes made: {}", optimization_run.changes.len());
        
        for change in &optimization_run.changes {
            println!("   - {}: {} → {}", 
                change.parameter_name,
                change.old_value,
                change.new_value
            );
            println!("     Reason: {}", change.change_reason);
        }
    } else {
        println!("ℹ️  No optimization changes recommended at this time");
    }

    // Get updated parameters
    let updated_params = optimizer.get_parameters().await;
    println!("📋 Updated optimization parameters:");
    println!("   Thread pool size: {}", updated_params.thread_pool_size);
    println!("   Buffer sizes: {:?}", updated_params.buffer_sizes);

    // Show optimization history
    let history = optimizer.get_history().await;
    println!("📚 Optimization history: {} runs", history.len());
    
    println!();
    Ok(())
}

/// Generate comprehensive performance report
async fn generate_performance_report(
    monitor: &Arc<PerformanceMonitor>,
    memory_manager: &Arc<MemoryManager>,
    metrics_collector: &Arc<MetricsCollector>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Generating Comprehensive Performance Report");
    println!("=============================================");

    // Performance report
    let perf_report = monitor.generate_report().await;
    println!("🕐 Report timestamp: {:?}", perf_report.timestamp);
    println!("⏱️  System uptime: {:?}", perf_report.uptime);
    
    println!("\n📊 Performance Summary:");
    println!("   Total operations: {}", perf_report.summary.total_operations);
    println!("   Average execution time: {:?}", perf_report.summary.avg_execution_time);
    if let Some(ref slowest) = perf_report.summary.slowest_operation {
        println!("   Slowest operation: {}", slowest);
    }
    if let Some(ref fastest) = perf_report.summary.highest_throughput_operation {
        println!("   Highest throughput: {}", fastest);
    }
    println!("   CPU usage: {:.2}%", perf_report.summary.cpu_usage);
    println!("   Active workflows: {}", perf_report.summary.active_workflows);
    println!("   Events/second: {:.2}", perf_report.summary.events_per_second);

    // Memory report
    let memory_stats = memory_manager.get_statistics().await;
    println!("\n🧠 Memory Report:");
    println!("   Total used: {:.2} MB", memory_stats.total_used as f64 / 1024.0 / 1024.0);
    println!("   Peak usage: {:.2} MB", memory_stats.peak_usage as f64 / 1024.0 / 1024.0);
    println!("   Memory pressure: {:.2}%", memory_stats.memory_pressure * 100.0);
    println!("   Active allocations: {}", memory_stats.active_allocations);
    println!("   Fragmentation: {:.2}%", memory_stats.fragmentation_ratio * 100.0);
    
    if memory_stats.gc_statistics.cycles > 0 {
        println!("   GC cycles: {}", memory_stats.gc_statistics.cycles);
        println!("   GC total time: {:?}", memory_stats.gc_statistics.total_time);
        println!("   GC average time: {:?}", memory_stats.gc_statistics.avg_time);
        println!("   Total memory freed: {:.2} MB", memory_stats.gc_statistics.total_freed as f64 / 1024.0 / 1024.0);
    }

    // Metrics snapshot
    let metrics_snapshot = metrics_collector.get_all_metrics().await;
    println!("\n📈 Metrics Summary:");
    println!("   Time series: {} metrics", metrics_snapshot.time_series.len());
    println!("   Histograms: {} metrics", metrics_snapshot.histograms.len());
    println!("   Counters: {} metrics", metrics_snapshot.counters.len());
    println!("   Gauges: {} metrics", metrics_snapshot.gauges.len());

    // Alerts summary
    if !perf_report.alerts.is_empty() {
        println!("\n⚠️  Active Alerts:");
        for alert in &perf_report.alerts {
            println!("   - {}: {}", 
                match alert.severity {
                    cim_domain_workflow::performance::AlertSeverity::Critical => "🔴 CRITICAL",
                    cim_domain_workflow::performance::AlertSeverity::Warning => "🟡 WARNING", 
                    cim_domain_workflow::performance::AlertSeverity::Info => "🔵 INFO",
                },
                alert.message
            );
        }
    } else {
        println!("\n✅ No active performance alerts");
    }

    println!("\n🎯 Performance Recommendations:");
    if memory_stats.memory_pressure > 0.8 {
        println!("   - Consider increasing memory limits or optimizing memory usage");
    }
    if perf_report.summary.avg_execution_time > Duration::from_millis(100) {
        println!("   - Consider optimizing slow operations or increasing parallelism");
    }
    if memory_stats.gc_statistics.cycles > 0 && memory_stats.gc_statistics.avg_time > Duration::from_millis(10) {
        println!("   - GC overhead is noticeable - consider tuning GC parameters");
    }
    
    println!("   - Monitor trends over time for performance degradation");
    println!("   - Set up automated alerting for critical thresholds");
    println!("   - Regular optimization runs recommended");

    println!();
    Ok(())
}