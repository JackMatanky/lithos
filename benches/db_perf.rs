//! Database performance benchmarks for zero-copy reads and batch writes.
//!
//! This benchmark suite validates the performance characteristics claimed in
//! Phase 6 implementation plan:
//! - Zero-copy reads should be 5-10x faster than full deserialization
//! - Batch writes should complete < 2s for 1000 notes
//! - Cache effectiveness for hot-path reads

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::integer_division_remainder_used,
    clippy::excessive_nesting,
    reason = "Benchmark code prioritizes readability over production-grade \
              error handling"
)]

use std::collections::HashMap;

use criterion::{
    BenchmarkId, Criterion, Throughput, black_box, criterion_group,
    criterion_main,
};
use lithos_core::{
    db::Database,
    note::{
        aggregate::Note,
        frontmatter::Frontmatter,
        link::{Link, Target},
        structure::{Heading, Section},
        tag::Tag,
        task::{Task, TaskStatus},
    },
};
use tempfile::TempDir;
use uuid::Uuid;

/// Create a realistic note with various subentities for benchmarking.
fn create_test_note(index: usize) -> Note {
    let id = Uuid::now_v7();
    let path = format!("notes/test-{index:04}.md");

    let mut note = Note::new(id, path).expect("valid path");

    // Add realistic content
    note.links = vec![
        Link::new_wikilink(
            Target::Unresolved {
                raw: "other-note.md".into(),
            },
            None,
            None,
            0,
        )
        .expect("valid link"),
        Link::new_markdown_link(
            Target::External {
                url: "https://example.com".into(),
            },
            Some("Example".to_owned()),
            None,
            50,
        )
        .expect("valid link"),
    ];

    note.tags = vec![
        Tag::new("#rust").expect("valid tag"),
        Tag::new("#performance").expect("valid tag"),
        Tag::new("#database/benchmarks").expect("valid tag"),
    ];

    note.headings = vec![
        Heading::new(1, "Main Title".to_owned(), 0).expect("valid heading"),
        Heading::new(2, "Subsection".to_owned(), 10).expect("valid heading"),
    ];

    note.tasks = vec![
        Task::new("Do something".to_owned(), TaskStatus::Incomplete, 15)
            .expect("valid task"),
        Task::new("Already done".to_owned(), TaskStatus::Complete, 16)
            .expect("valid task"),
    ];

    note.sections =
        vec![Section::new(None, "Test section content".to_owned(), 0..100)];

    // Add frontmatter
    note.frontmatter =
        Some(Frontmatter::new(HashMap::new()).expect("valid frontmatter"));

    note
}

/// Setup: Create a database with N notes for benchmarking.
fn setup_db_with_notes(count: usize) -> (TempDir, Database, Vec<Uuid>) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("bench.db");
    let db = Database::open(&db_path).expect("open database");

    let mut note_ids = Vec::with_capacity(count);

    // Use batch write for setup (not part of benchmark)
    db.batch_write(|batch_db| {
        for i in 0..count {
            let note = create_test_note(i);
            let id_str = note.id.to_string();
            note_ids.push(note.id);
            batch_db.put("notes", &id_str, &note).expect("insert note");
        }
        Ok(())
    })
    .expect("batch write");

    (temp_dir, db, note_ids)
}

/// Benchmark: Zero-copy read (hot path for LSP).
///
/// This measures the performance of reading archived data directly from the
/// database without full deserialization.
fn bench_zero_copy_read(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);
    let test_id = note_ids[50]; // Middle note for consistent results
    let id_str = test_id.to_string();

    let mut group = c.benchmark_group("read_zero_copy");
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_zero_copy", |b| {
        b.iter(|| {
            db.get::<Note, _, _>("notes", &id_str, |archived| {
                // Access archived data (zero-copy)
                black_box(archived.id);
                black_box(&archived.path);
                black_box(&archived.links);
            })
            .expect("get note")
        });
    });

    group.finish();
}

/// Benchmark: Full deserialization (cold path for mutations).
///
/// This measures the performance of full deserialization, which allocates
/// and copies data. Should be 5-10x slower than zero-copy reads.
fn bench_full_deserialize(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);
    let test_id = note_ids[50];
    let id_str = test_id.to_string();

    let mut group = c.benchmark_group("read_deserialize");
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_owned", |b| {
        b.iter(|| {
            let note: Option<Note> =
                db.get_owned("notes", &id_str).expect("get owned note");
            black_box(note.expect("note exists"));
        });
    });

    group.finish();
}

/// Benchmark: Single note write.
///
/// Measures individual write transaction overhead.
fn bench_single_write(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("bench_write.db");
    let db = Database::open(&db_path).expect("open database");

    let mut group = c.benchmark_group("write_single");
    group.throughput(Throughput::Elements(1));

    let mut counter = 0;
    group.bench_function("put_single", |b| {
        b.iter(|| {
            let note = create_test_note(counter);
            let id_str = note.id.to_string();
            counter = counter.wrapping_add(1);
            db.put("notes", &id_str, &note).expect("put note");
        });
    });

    group.finish();
}

/// Benchmark: Batch write performance.
///
/// Per Phase 6 plan: Should complete < 2s for 1000 notes.
fn bench_batch_write(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("bench_batch.db");

    let mut group = c.benchmark_group("write_batch");
    group.sample_size(10); // Fewer samples for expensive operation
    group.measurement_time(std::time::Duration::from_secs(30));

    for batch_size in [100, 500, 1000] {
        group.throughput(Throughput::Elements(batch_size));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let db = Database::open(&db_path).expect("open database");

                    db.batch_write(|batch_db| {
                        for i in 0..size {
                            let note = create_test_note(i as usize);
                            let id_str = note.id.to_string();
                            batch_db
                                .put("notes", &id_str, &note)
                                .expect("put note");
                        }
                        Ok(())
                    })
                    .expect("batch write");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Delete operation.
fn bench_delete(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(1000);

    let mut group = c.benchmark_group("delete");
    group.throughput(Throughput::Elements(1));

    let mut index = 0;
    group.bench_function("delete_single", |b| {
        b.iter(|| {
            let id = note_ids[index % note_ids.len()];
            let id_str = id.to_string();
            index = index.wrapping_add(1);

            let existed = db.delete("notes", &id_str).expect("delete note");
            black_box(existed);
        });
    });

    group.finish();
}

/// Benchmark: Cache effectiveness (hot vs cold reads).
///
/// Measures performance difference between first read (cold cache)
/// and subsequent reads (hot cache).
fn bench_cache_effectiveness(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);

    let mut group = c.benchmark_group("cache_effectiveness");
    group.throughput(Throughput::Elements(1));

    // Hot read: Same key repeatedly (should hit cache)
    let hot_id = note_ids[0].to_string();
    group.bench_function("hot_read", |b| {
        b.iter(|| {
            db.get::<Note, _, _>("notes", &hot_id, |archived| {
                black_box(archived.id);
            })
            .expect("get note")
        });
    });

    // Cold read: Different key each time (cache miss)
    let mut cold_index = 0;
    group.bench_function("cold_read", |b| {
        b.iter(|| {
            let cold_id = note_ids[cold_index % note_ids.len()].to_string();
            cold_index = cold_index.wrapping_add(1);

            db.get::<Note, _, _>("notes", &cold_id, |archived| {
                black_box(archived.id);
            })
            .expect("get note")
        });
    });

    group.finish();
}

/// Benchmark: Transaction overhead.
///
/// Compares overhead of creating multiple transactions vs one batch
/// transaction.
fn bench_transaction_overhead(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("bench_txn.db");

    let mut group = c.benchmark_group("transaction_overhead");
    let batch_size = 100;
    group.throughput(Throughput::Elements(batch_size));

    // Multiple individual transactions
    group.bench_function("individual_txns", |b| {
        b.iter(|| {
            let db = Database::open(&db_path).expect("open database");

            for i in 0..batch_size {
                let note = create_test_note(i as usize);
                let id_str = note.id.to_string();
                db.put("notes", &id_str, &note).expect("put note");
            }
        });
    });

    // Single batch transaction
    group.bench_function("batch_txn", |b| {
        b.iter(|| {
            let db = Database::open(&db_path).expect("open database");

            db.batch_write(|batch_db| {
                for i in 0..batch_size {
                    let note = create_test_note(i as usize);
                    let id_str = note.id.to_string();
                    batch_db.put("notes", &id_str, &note).expect("put note");
                }
                Ok(())
            })
            .expect("batch write");
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_zero_copy_read,
    bench_full_deserialize,
    bench_single_write,
    bench_batch_write,
    bench_delete,
    bench_cache_effectiveness,
    bench_transaction_overhead,
);
criterion_main!(benches);
