#![expect(
    clippy::disallowed_methods,
    reason = "Benchmarks use unwrap for simplicity in setup and iteration"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lithos_domain::models::{
    property::Property,
    property_spec::{PropertySpec, StringSpec},
    schema::{Schema, SchemaGraph, SchemaName},
};
use uuid::Uuid;

fn bench_property_creation(c: &mut Criterion) {
    let spec = PropertySpec::String(StringSpec::default());
    let name = lithos_domain::models::property::PropertyName::new(
        "test_property".to_string(),
    )
    .unwrap();
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
            let (schema, _event) = Schema::new(
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
