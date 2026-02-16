//! # Schema Benchmarks
//!
//! Performance benchmarks for schema-related domain operations.
//!
//! This module measures the execution time of core schema operations to ensure
//! they meet non-functional requirements for vault performance.
//!
//! ## Operations Benchmarked
//!
//! - **Property Creation**: Measures time to create `Property` instances with
//!   validation and UUID generation.
//! - **Schema Creation**: Benchmarks instantiation of `Schema` aggregates.
//! - **Inheritance Resolution**: Tests performance of resolving schema
//!   dependency order in a graph.
//!
//! ## Performance Invariants
//!
//! - Property creation: <1ms per operation
//! - Schema creation: <1ms per operation
//! - Inheritance resolution: <10ms for typical vault schemas (<100 nodes)
//!
//! ## Regression Monitoring
//!
//! Benchmarks run in CI with thresholds:
//! - >5% degradation triggers warning
//! - >10% degradation blocks release

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lithos_domain::{
    Property, PropertyName, PropertySpec, Schema, SchemaGraph, SchemaName,
    StringSpec,
};
use uuid::Uuid;

fn bench_property_creation(c: &mut Criterion) {
    let spec = PropertySpec::String(StringSpec::default());
    let name = PropertyName::new("test_property".to_string()).unwrap();
    let id = Uuid::now_v7();

    c.bench_function("property_creation", |b| {
        b.iter(|| {
            let prop = Property::new(
                black_box(id),
                black_box(name.clone()),
                black_box(true),
                black_box(false),
                black_box(spec.clone()),
            )
            .unwrap();
            black_box(prop);
        });
    });
}

fn bench_schema_inheritance_resolution(c: &mut Criterion) {
    let parent_name = SchemaName::new("parent".to_string()).unwrap();
    let child_name = SchemaName::new("child".to_string()).unwrap();

    let mut graph = SchemaGraph::new();
    graph.add_node(parent_name.clone(), None);
    graph.add_node(child_name.clone(), Some(parent_name));

    c.bench_function("schema_inheritance_resolution", |b| {
        b.iter(|| {
            let order = graph.resolve_order().unwrap();
            black_box(order);
        });
    });
}

fn bench_schema_creation(c: &mut Criterion) {
    let id = Uuid::now_v7();
    let name = SchemaName::new("test-schema".to_string()).unwrap();

    c.bench_function("schema_creation", |b| {
        b.iter(|| {
            let schema = Schema::new(
                black_box(id),
                black_box(name.clone()),
                black_box(vec![]),
            )
            .unwrap();
            black_box(schema);
        });
    });
}

criterion_group!(
    benches,
    bench_property_creation,
    bench_schema_inheritance_resolution,
    bench_schema_creation
);
criterion_main!(benches);
