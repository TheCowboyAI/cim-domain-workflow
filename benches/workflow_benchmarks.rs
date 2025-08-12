use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cim_domain_workflow::{
    aggregate::Workflow,
    value_objects::StepType,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Benchmark workflow creation performance
fn benchmark_workflow_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_creation");
    
    for size in [1, 10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("create_workflow", size), size, |b, &size| {
            b.iter(|| {
                let context = HashMap::new();
                
                let (mut workflow, _events) = Workflow::new(
                    black_box(format!("Benchmark Workflow {}", size)),
                    black_box(format!("Performance test workflow with {} steps", size)),
                    black_box(context),
                    Some("benchmark".to_string()),
                ).unwrap();
                
                // Add steps to the workflow
                for i in 0..size {
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
    
    group.bench_function("start_workflow", |b| {
        b.iter(|| {
            // Setup fresh workflow for each iteration
            let context = HashMap::new();
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
            
            // Start the workflow
            let mut start_context = cim_domain_workflow::value_objects::WorkflowContext::new();
            start_context.set_variable("test".to_string(), json!("benchmark"));
            
            let _result = workflow.start(
                black_box(start_context),
                black_box(Some("benchmark".to_string())),
            );
            
            black_box(workflow)
        })
    });
    
    group.finish();
}

/// Benchmark simple data operations
fn benchmark_data_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_operations");
    
    group.bench_function("uuid_generation", |b| {
        b.iter(|| {
            black_box(Uuid::new_v4())
        })
    });
    
    group.bench_function("json_serialization", |b| {
        let data = json!({
            "workflow_id": Uuid::new_v4(),
            "title": "Benchmark Workflow",
            "description": "Performance test",
            "steps": (0..10).map(|i| json!({
                "id": Uuid::new_v4(),
                "name": format!("Step {}", i),
                "type": "automated"
            })).collect::<Vec<_>>()
        });
        
        b.iter(|| {
            black_box(serde_json::to_string(&data).unwrap())
        })
    });
    
    group.finish();
}

criterion_group!(benches, 
    benchmark_workflow_creation,
    benchmark_state_transitions,
    benchmark_data_operations
);
criterion_main!(benches);