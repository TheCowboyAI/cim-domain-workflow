use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cim_domain_workflow::{
    aggregate::Workflow,
    commands::CreateWorkflow,
    events::WorkflowEvent,
    handlers::{NatsEventPublisher, EventMetadata},
    value_objects::{StepType, WorkflowContext, WorkflowId, StepId},
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Benchmark workflow creation performance
fn benchmark_workflow_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_creation");
    
    for size in [1, 10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("create_workflow", size), size, |b, &size| {
            b.iter(|| {
                let workflow_id = WorkflowId::new();
                let context = WorkflowContext::new();
                
                let (mut workflow, _events) = Workflow::new(
                    black_box(format!("Benchmark Workflow {}", size)),
                    black_box(format!("Performance test workflow with {} steps", size)),
                    black_box(context),
                    Some("benchmark".to_string()),
                ).unwrap();
                
                // Add steps to the workflow
                for i in 0..*size {
                    let _result = workflow.add_step(
                        black_box(format!("Step {}", i)),
                        black_box(format!("Benchmark step {}", i)),
                        black_box(StepType::Automated),
                        black_box(Default::default()),
                        black_box(vec![]),
                        Some(5),
                        None,
                        Some("benchmark".to_string()),
                    );
                }
                
                black_box(workflow)
            })
        });
    }
    group.finish();
}

/// Benchmark workflow state transitions
fn benchmark_state_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_transitions");
    
    // Setup workflow
    let context = WorkflowContext::new();
    let (mut workflow, _events) = Workflow::new(
        "Benchmark Workflow".to_string(),
        "Performance test workflow".to_string(),
        context,
        Some("benchmark".to_string()),
    ).unwrap();
    
    // Add a step
    let _result = workflow.add_step(
        "Test Step".to_string(),
        "Benchmark step".to_string(),
        StepType::Automated,
        Default::default(),
        vec![],
        Some(5),
        None,
        Some("benchmark".to_string()),
    );
    
    group.bench_function("start_workflow", |b| {
        b.iter(|| {
            let mut test_workflow = workflow.clone();
            let _result = test_workflow.start(
                black_box(Default::default()),
                black_box(Some("benchmark".to_string())),
            );
            black_box(test_workflow)
        })
    });
    
    group.finish();
}

/// Benchmark event serialization and deserialization
fn benchmark_event_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_serialization");
    
    // Create sample events
    let workflow_id = WorkflowId::new();
    let step_id = StepId::new();
    let context = WorkflowContext::new();
    
    let events = vec![
        WorkflowEvent::WorkflowCreated {
            workflow_id,
            title: "Benchmark Workflow".to_string(),
            description: "Performance test workflow".to_string(),
            context: context.clone(),
            actor: Some("benchmark".to_string()),
        },
        WorkflowEvent::StepAdded {
            workflow_id,
            step_id,
            title: "Test Step".to_string(),
            description: "Benchmark step".to_string(),
            step_type: StepType::Automated,
            config: Default::default(),
            dependencies: vec![],
            timeout_minutes: Some(5),
            approval_required: None,
            actor: Some("benchmark".to_string()),
        },
        WorkflowEvent::WorkflowStarted {
            workflow_id,
            context: Default::default(),
            actor: Some("benchmark".to_string()),
        },
    ];
    
    group.bench_function("serialize_events", |b| {
        b.iter(|| {
            let serialized: Vec<String> = events.iter()
                .map(|event| serde_json::to_string(event).unwrap())
                .collect();
            black_box(serialized)
        })
    });
    
    let serialized_events: Vec<String> = events.iter()
        .map(|event| serde_json::to_string(event).unwrap())
        .collect();
    
    group.bench_function("deserialize_events", |b| {
        b.iter(|| {
            let deserialized: Vec<WorkflowEvent> = serialized_events.iter()
                .map(|event_str| serde_json::from_str(event_str).unwrap())
                .collect();
            black_box(deserialized)
        })
    });
    
    group.finish();
}

/// Benchmark cross-domain operations
fn benchmark_cross_domain_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_domain_operations");
    
    let workflow_id = WorkflowId::new();
    let step_id = StepId::new();
    
    group.bench_function("create_cross_domain_request", |b| {
        b.iter(|| {
            let request = json!({
                "workflow_id": workflow_id,
                "step_id": step_id,
                "target_domain": "inventory",
                "operation": "check_availability",
                "data": {
                    "items": [
                        {"sku": "WIDGET-001", "quantity": 2},
                        {"sku": "WIDGET-002", "quantity": 1}
                    ]
                },
                "actor": "benchmark"
            });
            black_box(request)
        })
    });
    
    group.finish();
}

/// Benchmark workflow context operations
fn benchmark_context_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_operations");
    
    let mut context = WorkflowContext::new();
    
    group.bench_function("add_variables", |b| {
        b.iter(|| {
            let mut test_context = context.clone();
            for i in 0..100 {
                test_context.add_variable(
                    black_box(format!("var_{}", i)),
                    black_box(json!(format!("value_{}", i))),
                );
            }
            black_box(test_context)
        })
    });
    
    // Setup context with variables for lookup benchmarks
    for i in 0..100 {
        context.add_variable(format!("var_{}", i), json!(format!("value_{}", i)));
    }
    
    group.bench_function("get_variable", |b| {
        b.iter(|| {
            let variable = context.get_variable(black_box("var_50"));
            black_box(variable)
        })
    });
    
    group.bench_function("add_metadata", |b| {
        b.iter(|| {
            let mut test_context = context.clone();
            for i in 0..100 {
                test_context.add_metadata(
                    black_box(format!("meta_{}", i)),
                    black_box(json!(format!("meta_value_{}", i))),
                );
            }
            black_box(test_context)
        })
    });
    
    group.finish();
}

/// Benchmark concurrent workflow operations
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    
    group.bench_function("concurrent_workflow_creation", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    std::thread::spawn(move || {
                        let context = WorkflowContext::new();
                        let (workflow, _events) = Workflow::new(
                            format!("Concurrent Workflow {}", i),
                            format!("Concurrent test workflow {}", i),
                            context,
                            Some("benchmark".to_string()),
                        ).unwrap();
                        workflow
                    })
                })
                .collect();
            
            let workflows: Vec<_> = handles.into_iter()
                .map(|handle| handle.join().unwrap())
                .collect();
            
            black_box(workflows)
        })
    });
    
    group.finish();
}

/// Benchmark memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("large_workflow_creation", |b| {
        b.iter(|| {
            let context = WorkflowContext::new();
            let (mut workflow, _events) = Workflow::new(
                "Large Workflow".to_string(),
                "Large workflow with many steps and complex context".to_string(),
                context,
                Some("benchmark".to_string()),
            ).unwrap();
            
            // Create a workflow with many steps
            for i in 0..1000 {
                let _result = workflow.add_step(
                    black_box(format!("Step {}", i)),
                    black_box(format!("Complex step {} with detailed description and metadata", i)),
                    black_box(StepType::Automated),
                    black_box(json!({
                        "complexity": i % 10,
                        "priority": if i % 3 == 0 { "high" } else { "normal" },
                        "tags": vec![format!("tag_{}", i % 5), format!("category_{}", i % 7)],
                        "metadata": {
                            "created_by": "benchmark",
                            "step_number": i,
                            "batch_id": i / 100
                        }
                    })),
                    black_box(if i > 0 { vec![StepId::new()] } else { vec![] }),
                    Some(5 + (i % 30) as i32),
                    if i % 10 == 0 { Some("manager".to_string()) } else { None },
                    Some("benchmark".to_string()),
                );
            }
            
            black_box(workflow)
        })
    });
    
    group.finish();
}

/// Benchmark ID generation and operations
fn benchmark_id_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_operations");
    
    group.bench_function("workflow_id_generation", |b| {
        b.iter(|| {
            let id = WorkflowId::new();
            black_box(id)
        })
    });
    
    group.bench_function("step_id_generation", |b| {
        b.iter(|| {
            let id = StepId::new();
            black_box(id)
        })
    });
    
    let workflow_ids: Vec<WorkflowId> = (0..1000).map(|_| WorkflowId::new()).collect();
    
    group.bench_function("id_comparison", |b| {
        b.iter(|| {
            let mut matches = 0;
            for i in 0..workflow_ids.len() {
                for j in i+1..workflow_ids.len() {
                    if workflow_ids[i] == workflow_ids[j] {
                        matches += 1;
                    }
                }
            }
            black_box(matches)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_workflow_creation,
    benchmark_state_transitions,
    benchmark_event_serialization,
    benchmark_cross_domain_operations,
    benchmark_context_operations,
    benchmark_concurrent_operations,
    benchmark_memory_usage,
    benchmark_id_operations
);

criterion_main!(benches);