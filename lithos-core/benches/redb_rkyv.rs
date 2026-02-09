//! redb + rkyv storage benchmarks.
//!
//! # Relevance contract
//!
//! These benchmarks exist to keep Lithos’s performance claims honest as the
//! codebase evolves.
//!
//! When you change any of the following, you should expect these benchmarks to
//! move and you should update the doc comments’ “Expected results” accordingly:
//! - On-disk encoding format, rkyv features, or validation approach
//! - Transaction boundaries (e.g., where/when commits happen)
//! - Data model shape for `Note` (more fields, bigger strings, more nested
//!   vecs)
//! - Namespacing strategy for keys
//!
//! # What to look for
//!
//! - **Trend, not absolutes**: raw numbers depend on CPU, filesystem, and SSD.
//! - **Relative comparisons** are the guardrail: archived-access reads should
//!   be faster than full deserialization, and batched writes should dominate
//!   per-op transactions.
//! - **Regression signals**:
//!   - `read_zero_copy/*` gets close to `read_deserialize/*`
//!   - `transaction_overhead/batch_txn` approaches `individual_txns`
//!   - `write_batch/*` stops scaling roughly linearly with batch size
//!
//! # Safety
//! Benchmark code uses unwrap/expect for simplicity.

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
    reason = "Criterion macros generate undocumented items; benchmark code \
              uses simple control flow and asserts for clarity"
)]

use std::collections::HashMap;

use criterion::{
    BenchmarkId, Criterion, Throughput, black_box, criterion_group,
    criterion_main,
};
use lithos_core::{
    config::task::{StatusSymbol, TaskConfig},
    db::Database,
    note::{
        aggregate::Note,
        frontmatter::Frontmatter,
        link::{Link, Target},
        structure::{Heading, Section},
        tag::Tag,
        task::Task,
        types::{HeadingLevel, NoteId, SourceByteOffset, SourceByteRange},
    },
};
use tempfile::TempDir;
use uuid::Uuid;

/// Creates a realistic `Note` value with nested structures.
///
/// Purpose:
/// - Ensures the benchmarks reflect the real read/write shapes Lithos expects
///   (LSP hot paths, indexing, rendering) rather than a toy struct.
///
/// Expected results:
/// - Increasing complexity here will slow both read and write benchmarks.
/// - Archived reads should generally degrade less than full deserialization as
///   fields are added, since they avoid constructing owned structures.
fn create_test_note(index: usize) -> Note {
    let id = NoteId::new();
    let path = format!("notes/test-{index:04}.md");

    let mut note = Note::new(id, path).expect("valid path");

    note.add_link(
        Link::new_wikilink(
            Target::Unresolved {
                raw: "other-note.md".into(),
            },
            None,
            None,
            SourceByteOffset::new(0),
        )
        .expect("valid link"),
    );
    note.add_link(
        Link::new_markdown_link(
            Target::External {
                url: "https://example.com".into(),
            },
            Some("Example".to_owned()),
            None,
            SourceByteOffset::new(50),
        )
        .expect("valid link"),
    );

    note.add_tag(Tag::new("#rust").expect("valid tag"));
    note.add_tag(Tag::new("#performance").expect("valid tag"));
    note.add_tag(Tag::new("#database/benchmarks").expect("valid tag"));

    note.add_heading(
        Heading::new(
            HeadingLevel::try_new(1).expect("valid level"),
            "Main Title".to_owned(),
            SourceByteOffset::new(0),
        )
        .expect("valid heading"),
    );
    note.add_heading(
        Heading::new(
            HeadingLevel::try_new(2).expect("valid level"),
            "Subsection".to_owned(),
            SourceByteOffset::new(10),
        )
        .expect("valid heading"),
    );

    let task_config = TaskConfig::default();
    let status = StatusSymbol::try_new(' ').expect("valid status");
    note.add_task(
        Task::from_checkbox(
            "Do something",
            status,
            SourceByteOffset::new(15),
            &task_config,
        )
        .expect("valid task"),
    );
    note.add_task(
        Task::from_checkbox(
            "Already done",
            StatusSymbol::try_new('x').expect("valid status"),
            SourceByteOffset::new(16),
            &task_config,
        )
        .expect("valid task"),
    );

    note.add_section(Section::new(
        None,
        "Test section content".to_owned(),
        SourceByteRange::new(
            SourceByteOffset::new(0),
            SourceByteOffset::new(100),
        ),
    ));

    note.set_frontmatter(Some(
        Frontmatter::new(HashMap::new()).expect("valid frontmatter"),
    ));

    note
}

/// Prepares a new database file populated with `count` notes.
///
/// Purpose:
/// - Separates dataset creation from timed benchmark loops so we do not
///   accidentally measure setup or I/O unrelated to the target operation.
///
/// Expected results:
/// - This function should not dominate benchmark time, because it runs outside
///   the timed loops.
fn setup_db_with_notes(count: usize) -> (TempDir, Database, Vec<NoteId>) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("bench.db");
    let db = Database::open(&db_path).expect("open database");

    let mut note_ids = Vec::with_capacity(count);

    db.batch_write(|batch_db| {
        for i in 0..count {
            let note = create_test_note(i);
            let id_str = Uuid::from(note.id()).to_string();
            note_ids.push(note.id());
            batch_db.put("notes", &id_str, &note).expect("insert note");
        }
        Ok(())
    })
    .expect("batch write");

    (temp_dir, db, note_ids)
}

/// Benchmarks archived-access reads via `Database::get`.
///
/// Purpose:
/// - Models the “LSP hot path”: inspect a subset of fields without building an
///   owned `Note`.
///
/// Expected results:
/// - Should be faster than `bench_full_deserialize`.
/// - In this codebase, “zero-copy” includes an alignment copy into an
///   `AlignedVec`, so expect a speedup closer to ~1.5–3× rather than 10×.
fn bench_zero_copy_read(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);
    let test_id = note_ids[50];
    let id_str = Uuid::from(test_id).to_string();

    let mut group = c.benchmark_group("read_zero_copy");
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_zero_copy", |b| {
        b.iter(|| {
            db.get::<Note, _, _>("notes", &id_str, |archived| {
                black_box(archived);
            })
            .expect("get note")
        });
    });

    group.finish();
}

/// Benchmarks full deserialization via `Database::get_owned`.
///
/// Purpose:
/// - Models mutation/indexing flows where an owned `Note` is required.
///
/// Expected results:
/// - Should be slower than `bench_zero_copy_read` because it allocates and
///   constructs owned nested structures.
/// - If this becomes close to archived-access reads, it likely means we are
///   deserializing less data than we think or doing more work in the archived
///   path than intended.
fn bench_full_deserialize(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);
    let test_id = note_ids[50];
    let id_str = Uuid::from(test_id).to_string();

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

/// Benchmarks individual writes using `Database::put`.
///
/// Purpose:
/// - Estimates the cost of one write transaction + commit per item.
///
/// Expected results:
/// - Much slower per inserted element than `bench_batch_write` for large N.
/// - If this looks “too fast”, the filesystem may be caching aggressively; the
///   transaction-overhead benchmark should still show the batching advantage.
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
            let id_str = Uuid::from(note.id()).to_string();
            counter = counter.wrapping_add(1);
            db.put("notes", &id_str, &note).expect("put note");
        });
    });

    group.finish();
}

/// Benchmarks batched writes using `Database::batch_write`.
///
/// Purpose:
/// - Captures the “indexing-style” workload: many inserts under one write
///   transaction with one commit.
///
/// Expected results:
/// - Should be dramatically faster than doing the same number of inserts via
///   `Database::put` in a loop.
/// - Should scale roughly linearly with batch size (100 < 500 < 1000).
fn bench_batch_write(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");

    let mut group = c.benchmark_group("write_batch");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    for batch_size in [100, 500, 1000] {
        group.throughput(Throughput::Elements(batch_size));

        let mut file_index: u64 = 0;

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let db_path = temp_dir.path().join(format!(
                        "bench_batch_{batch_size}_{file_index}.db"
                    ));
                    file_index = file_index.wrapping_add(1);

                    let db = Database::open(&db_path).expect("open database");

                    db.batch_write(|batch_db| {
                        for i in 0..size {
                            let note = create_test_note(i as usize);
                            let id_str = Uuid::from(note.id()).to_string();
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

/// Benchmarks delete performance via `Database::delete`.
///
/// Purpose:
/// - Ensures deletes do not regress badly as key/value sizes change.
///
/// Expected results:
/// - Should remain low and relatively stable across releases.
/// - If it degrades significantly, check for unintended extra work (e.g.,
///   scanning or rebuilding indexes on delete).
fn bench_delete(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(1000);

    let mut group = c.benchmark_group("delete");
    group.throughput(Throughput::Elements(1));

    let mut index = 0;
    group.bench_function("delete_single", |b| {
        b.iter(|| {
            let id = note_ids[index % note_ids.len()];
            let id_str = Uuid::from(id).to_string();
            index = index.wrapping_add(1);

            let existed = db.delete("notes", &id_str).expect("delete note");
            black_box(existed);
        });
    });

    group.finish();
}

/// Benchmarks “hot key” reads vs “rotating key” reads.
///
/// Purpose:
/// - A sanity check for cache effects in the redb stack and OS page cache.
/// - Helps detect when an internal change accidentally defeats locality.
///
/// Expected results:
/// - Hot reads should generally be faster than cold reads.
/// - If both converge, it may mean the working set fits entirely in cache, or
///   the operation is dominated by non-cacheable work (e.g.,
///   validation/copies).
fn bench_cache_effectiveness(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);

    let mut group = c.benchmark_group("cache_effectiveness");
    group.throughput(Throughput::Elements(1));

    let hot_id = Uuid::from(note_ids[0]).to_string();
    group.bench_function("hot_read", |b| {
        b.iter(|| {
            db.get::<Note, _, _>("notes", &hot_id, |archived| {
                black_box(archived);
            })
            .expect("get note")
        });
    });

    let mut cold_index = 0;
    group.bench_function("cold_read", |b| {
        b.iter(|| {
            let cold_id =
                Uuid::from(note_ids[cold_index % note_ids.len()]).to_string();
            cold_index = cold_index.wrapping_add(1);

            db.get::<Note, _, _>("notes", &cold_id, |archived| {
                black_box(archived);
            })
            .expect("get note")
        });
    });

    group.finish();
}

/// Benchmarks transaction creation/commit overhead.
///
/// Purpose:
/// - Isolates the cost of “many commits” vs “one commit”, which is the main
///   lever behind write performance in redb.
///
/// Expected results:
/// - `batch_txn` should be much faster than `individual_txns` for the same N.
/// - If they converge, it likely means commits are not happening as expected
///   (or redb durability settings changed).
fn bench_transaction_overhead(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("create temp dir");

    let mut group = c.benchmark_group("transaction_overhead");
    let batch_size = 100;
    group.throughput(Throughput::Elements(batch_size));

    let mut file_index: u64 = 0;

    group.bench_function("individual_txns", |b| {
        b.iter(|| {
            let db_path = temp_dir
                .path()
                .join(format!("bench_txn_individual_{file_index}.db"));
            file_index = file_index.wrapping_add(1);

            let db = Database::open(&db_path).expect("open database");

            for i in 0..batch_size {
                let note = create_test_note(i as usize);
                let id_str = Uuid::from(note.id()).to_string();
                db.put("notes", &id_str, &note).expect("put note");
            }
        });
    });

    group.bench_function("batch_txn", |b| {
        b.iter(|| {
            let db_path = temp_dir
                .path()
                .join(format!("bench_txn_batch_{file_index}.db"));
            file_index = file_index.wrapping_add(1);

            let db = Database::open(&db_path).expect("open database");

            db.batch_write(|batch_db| {
                for i in 0..batch_size {
                    let note = create_test_note(i as usize);
                    let id_str = Uuid::from(note.id()).to_string();
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
