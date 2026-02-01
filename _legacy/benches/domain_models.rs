//! # Domain Models Benchmarks
//!
//! Performance benchmarks for core domain model operations.
//!
//! This module measures the execution time of fundamental domain operations
//! to ensure they remain fast and scalable for large vaults.
//!
//! ## Operations Benchmarked
//!
//! - **Note Creation**: Measures time to instantiate `Note` aggregates with
//!   path validation and UUID assignment.
//! - **Tag Parsing**: Benchmarks parsing of tag strings into `Tag` values.
//!
//! ## Performance Invariants
//!
//! - Note creation: <1ms per operation
//! - Tag parsing: <0.5ms per operation
//!
//! ## Regression Monitoring
//!
//! Benchmarks run in CI with thresholds:
//! - >5% degradation triggers warning
//! - >10% degradation blocks release

#![expect(
    clippy::disallowed_methods,
    reason = "Benchmarks use Result::unwrap() during setup and measurement \
              loops for uninterrupted iteration. Failures represent invalid \
              state, not logic under test."
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lithos_domain::{Note, Tag};
use uuid::Uuid;

fn bench_note_creation(c: &mut Criterion) {
    let test_id =
        Uuid::parse_str("01936b2e-8f4a-7890-abcd-ef1234567890").unwrap();
    c.bench_function("note_creation", |b| {
        b.iter(|| {
            let note = Note::new(
                black_box(test_id),
                black_box("bench/test.md".to_string()),
            )
            .unwrap();
            black_box(note);
        });
    });
}

fn bench_tag_parsing(c: &mut Criterion) {
    c.bench_function("tag_parsing", |b| {
        b.iter(|| {
            let tag = Tag::parse(black_box("#work/project")).unwrap();
            black_box(tag);
        });
    });
}

criterion_group!(benches, bench_note_creation, bench_tag_parsing);
criterion_main!(benches);
