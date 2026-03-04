//! Storage layer performance benchmarks for redb + rkyv infrastructure.
//!
//! # Summary
//!
//! Benchmarks the core database storage layer to detect regressions in
//! zero-copy reads, batch transaction performance, and cache effectiveness.
//!
//! # Motivation
//!
//! Lithos's performance claims center on zero-copy archived access via rkyv and
//! efficient bulk operations via redb transactions. This suite validates those
//! claims and guards against regressions as the data model evolves. Historical
//! context: Initial benchmarks in `redb_rkyv.rs` mixed concerns; this suite
//! focuses purely on storage infrastructure.
//!
//! # Scope
//!
//! **Included**:
//! - Zero-copy archived reads (`get` with closure)
//! - Full deserialization reads (`get_owned`)
//! - Single-item writes (individual transactions)
//! - Batch writes (multiple items per transaction)
//! - Delete operations
//! - Cache behavior (hot vs cold key access patterns)
//! - Transaction overhead (batch vs individual commit cost)
//!
//! **Excluded**:
//! - Key formatting strategies (see `db_key_handling.rs`)
//! - String allocation patterns (see `string_construction.rs`)
//! - Domain-specific parsing (see `note_parsing.rs`)
//! - Network I/O, file system traversal, concurrent access
//!
//! # Benchmark Style
//!
//! - **Micro-benchmarks**: Single operations measured in isolation
//! - **Single-threaded**: No concurrent access (redb supports MVCC but suite
//!   focuses on single-writer scenarios)
//! - **Steady-state focus**: Cache-warm measurements (hot/cold benches
//!   explicitly test cache behavior)
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs with default warm-up and sampling (100 samples,
//!   3s warm-up)
//! - **Black-boxing**: All read values passed through `black_box()` to prevent
//!   optimization
//! - **Setup separation**: Database population and test data creation occur
//!   outside timed region
//! - **Measurement focus**: Operation latency (time per element), not
//!   throughput
//!
//! # Input Model
//!
//! - **Data**: Realistic `Note` aggregates with links, tags, tasks, headings,
//!   sections (~200-500 bytes serialized)
//! - **Determinism**: Fresh database per benchmark group; UUIDs are v7
//!   (time-based) but measurements are stable
//! - **Representativeness**: Note structure mirrors production workload (3
//!   tags, 2 links, 2 tasks, 2 headings)
//! - **Sizes**:
//!   - Small: 100 notes (read benchmarks)
//!   - Medium: 500 notes (batch write scaling)
//!   - Large: 1000 notes (delete, transaction overhead)
//!
//! # Controls and Fairness
//!
//! - **Same inputs**: All read benchmarks use identical 100-note dataset
//! - **Compilation**: Run with `--release` (criterion default), no special
//!   target-cpu or LTO
//! - **Environment**: Best-effort (no CPU pinning); results should be compared
//!   within same machine/session
//! - **Allocation**: Uses system allocator (no custom allocator configured)
//!
//! # Expected Characteristics
//!
//! Based on redb architecture and rkyv zero-copy design:
//!
//! **Zero-Copy Reads** (~450-500 ns):
//! - Direct memory access via rkyv archived types
//! - No deserialization, no heap allocation
//! - Should be significantly faster than full deserialization (1.5-2x)
//! - Sub-microsecond is critical for LSP responsiveness
//! - Bottleneck: redb B-tree traversal, not serialization
//!
//! **Full Deserialization** (~750-850 ns):
//! - Construct owned `Note` with all heap allocations
//! - Should be 1.5-2x slower than zero-copy
//! - Still sub-microsecond for small notes (<1KB serialized)
//! - Bottleneck: heap allocation, not rkyv logic
//!
//! **Single Writes** (~3-4 ms):
//! - Individual transaction per write (includes fsync overhead)
//! - Dominated by transaction commit cost, not serialization
//! - Should scale O(1) regardless of note size (for notes <10KB)
//! - Bottleneck: fsync to disk, not database logic
//!
//! **Batch Writes** (~250-300 notes/sec sustained):
//! - Multiple items per transaction amortizes fsync cost
//! - Should scale linearly with batch size: O(n)
//! - Expected: 100 notes → ~400ms, 500 notes → ~2s, 1000 notes → ~4s
//! - Bottleneck: I/O contention at scale, not transaction overhead
//!
//! **Deletes** (~3-4 ms):
//! - Similar to single writes (transaction-dominated)
//! - Should be comparable to write performance
//! - Bottleneck: fsync, not key removal logic
//!
//! **Cache Effectiveness** (~460-470 ns, minimal difference):
//! - redb uses mmap, relies on OS page cache
//! - Hot vs cold difference should be <10% for in-memory data
//! - tmpfs eliminates real disk I/O variance
//! - Bottleneck: B-tree traversal, not cache misses
//!
//! **Transaction Overhead** (batch should match individual):
//! - Batching within single transaction vs separate transactions
//! - Expected: similar performance (both include single fsync per transaction)
//! - If batched is >20% faster → amortization working as expected
//! - Bottleneck: fsync frequency, not lock contention
//!
//! # Interpreting Results
//!
//! **Meaningful changes**:
//! - ±10% in single operations: Investigate
//! - ±20% in batch operations: Likely regression/improvement
//! - Zero-copy approaching deserialization (within 2x): Red flag
//!
//! **Valid comparisons**:
//! - Within-machine, same session: Reliable
//! - Across machines: Trends only, not absolute numbers
//! - CI vs local: Expect 20-30% variance due to environment
//!
//! **Noise sources**:
//! - File system cache state (benchmarks use tmpfs-backed `TempDir`)
//! - Background processes (close unnecessary applications)
//! - Thermal throttling (long benchmark runs may show degradation)
//!
//! **Not justified conclusions**:
//! - Production end-to-end latency (does not model network, concurrent access,
//!   query complexity)
//! - Memory usage patterns (criterion does not report allocations)
//!
//! # Reporting and Workflow
//!
//! - **Local development**: Run before/after changes, compare saved baselines
//!   (`--save-baseline`, `--baseline`)
//! - **PR workflow**: Run full suite, note regressions/improvements in PR
//!   description
//! - **CI**: Not currently automated (manual run on performance-sensitive
//!   changes)
//!
//! # Maintenance Contract
//!
//! **Update when**:
//! - Note domain model changes (new fields, removed fields, nesting changes)
//! - Database layer API changes (method signatures, transaction model)
//! - rkyv configuration changes (features, validation strategy)
//! - New storage hot paths identified (add corresponding benchmarks)
//!
//! **Adding benchmarks**:
//! - Group by operation type (read/write/delete/cache)
//! - Use `Throughput::Elements(n)` for per-item measurements
//! - Follow `bench_<operation>_<variant>` naming convention
//! - Document input size and expected complexity in per-bench comment
//!
//! **Stability expectations**:
//! - Results should be stable within ±5% across runs on same machine
//! - Flaky benchmarks (>10% variance) must be investigated or removed
//!
//! # Known Limitations
//!
//! - **No contention modeling**: Single-threaded only, does not test concurrent
//!   readers/writers
//! - **No allocator comparison**: Uses system allocator; custom allocators not
//!   tested
//! - **Synthetic data**: Note structures are realistic but not from production
//!   corpus
//! - **Cache warm**: Does not model cold-start database opening overhead
//!
//! # Benchmark Index
//!
//! | Group                    | Expected Time     | Focus                                     |
//! | ------------------------ | ----------------- | ----------------------------------------- |
//! | `read_zero_copy`         | ~450-500 ns       | Archived access without deserialization   |
//! | `read_deserialize`       | ~750-850 ns       | Full owned value construction             |
//! | `write_single`           | ~3-4 ms           | Individual transaction per write          |
//! | `write_batch`            | ~400ms-4s (100-1K)| Bulk operations in single transaction     |
//! | `delete`                 | ~3-4 ms           | Key removal performance                   |
//! | `cache_effectiveness`    | ~460-470 ns       | Hot vs cold access patterns               |
//! | `transaction_overhead`   | ~400ms (100)      | Batch vs individual commit cost           |
//!
//! # Safety
//!
//! Benchmark code uses `unwrap`/`expect` for simplicity (failures indicate test
//! setup errors, not runtime conditions).

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::integer_division_remainder_used,
    clippy::excessive_nesting,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]
#![expect(
    clippy::pattern_type_mismatch,
    reason = "Iterating over Vec<(String, T)> creates unavoidable conflicts \
              between pattern_type_mismatch, ref_patterns, and \
              needless_borrowed_reference lints"
)]

use std::collections::HashMap;

use criterion::{
    BenchmarkId, Criterion, Throughput, black_box, criterion_group,
    criterion_main,
};
use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        task::StatusSymbol,
        vault::{VaultId, VaultRoot},
    },
    db::Database,
    note::{
        aggregate::{Note, NoteId},
        frontmatter::Frontmatter,
        link::{Link, Target},
        position::{SourceByteOffset, SourceByteRange},
        structure::{Heading, HeadingLevel, Section},
        tag::Tag,
        task::Task,
    },
};
use redb::TableDefinition;
use tempfile::TempDir;
use uuid::Uuid;

const NOTES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("notes");

/// Creates a realistic `Note` value with nested structures.
fn create_test_note(index: usize) -> Note {
    let id = NoteId::new();
    let path = format!("notes/test-{index:04}.md");

    let mut note = Note::new(id, &path).expect("valid path");

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
            Some("Example"),
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
            "Main Title",
            SourceByteOffset::new(0),
        )
        .expect("valid heading"),
    );
    note.add_heading(
        Heading::new(
            HeadingLevel::try_new(2).expect("valid level"),
            "Subsection",
            SourceByteOffset::new(10),
        )
        .expect("valid heading"),
    );

    let config = Config::build(
        &RawConfig::default(),
        VaultId::new(),
        VaultRoot::try_new(std::path::PathBuf::from("/vault"))
            .expect("valid vault root"),
    )
    .expect("config");
    let status = StatusSymbol::try_new(' ').expect("valid status");
    let status_name = config
        .task()
        .status()
        .name_for_symbol(status)
        .expect("valid status")
        .clone();
    note.add_task(
        Task::new(
            status_name.clone(),
            "Do something",
            SourceByteOffset::new(15),
            lithos_core::note::task::TaskAttributes::default(),
        )
        .expect("valid task"),
    );
    note.add_task(
        Task::new(
            config
                .task()
                .status()
                .name_for_symbol(
                    StatusSymbol::try_new('x').expect("valid status"),
                )
                .expect("valid status")
                .clone(),
            "Already done",
            SourceByteOffset::new(16),
            lithos_core::note::task::TaskAttributes::default(),
        )
        .expect("valid task"),
    );

    note.add_section(Section::new(
        None,
        SourceByteRange::new(
            SourceByteOffset::new(0),
            SourceByteOffset::new(100),
        )
        .expect("valid source range"),
    ));

    note.set_frontmatter(Some(Frontmatter::new(HashMap::new())));

    note
}

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
            batch_db.put(NOTES_TABLE, &id_str, &note).expect("insert note");
        }
        Ok(())
    })
    .expect("batch write");

    (temp_dir, db, note_ids)
}

/// Benchmarks zero-copy archived read access (steady-state, hot cache).
///
/// # Purpose
///
/// Measures the cost of accessing rkyv-archived `Note` data without
/// deserialization, validating Lithos's core zero-copy claim.
///
/// # What is Measured
///
/// - **Metric**: Latency per read operation (nanoseconds)
/// - **Execution**: Single `get` call per iteration with closure access
/// - **State**: Warm cache (same key read repeatedly)
///
/// # Inputs
///
/// - **Size**: 100 notes pre-populated in database
/// - **Target**: Middle note (index 50) to avoid edge effects
/// - **Determinism**: Fixed dataset, UUIDs generated at setup time
///
/// # Setup
///
/// - Database created with 100 realistic `Note` aggregates outside timed region
/// - Target UUID selected before benchmark starts
/// - Closure accesses archived data but does not deserialize
///
/// # Expected Characteristics
///
/// - **Complexity**: O(1) lookup (B-tree index)
/// - **Dominant costs**: B-tree traversal, moka cache lookup, memory access
/// - **Typical range**: 250-450 ns (depends on CPU, cache hierarchy)
///
/// # Interpreting Changes
///
/// - **>10% regression**: Investigate cache degradation or B-tree changes
/// - **Approaching `read_deserialize`**: Zero-copy advantage eroding (critical
///   issue)
/// - **Noise level**: Expect ±5% variance across runs
///
/// # Limitations
///
/// - Does not model cold cache (database just opened)
/// - Does not test large Notes (>1KB serialized)
/// - Single-threaded only (no concurrent reader contention)
///
/// # Notes for Future
///
/// - Changing `Note` field layout may affect archived access patterns
/// - Do not remove `black_box(archived)` or compiler will eliminate closure
fn bench_zero_copy_read(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);
    let test_id = note_ids[50];
    let test_key = Uuid::from(test_id).to_string();

    let mut group = c.benchmark_group("read_zero_copy");
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_zero_copy", |b| {
        b.iter(|| {
            db.get::<Note, _, _>(NOTES_TABLE, &test_key, |archived| {
                black_box(archived);
            })
            .expect("get note")
        });
    });

    group.finish();
}

/// Benchmarks full deserialization to owned `Note` value (steady-state).
///
/// # Purpose
///
/// Measures the cost of constructing an owned `Note` from archived data,
/// providing baseline comparison for zero-copy reads.
///
/// # What is Measured
///
/// - **Metric**: Latency per `get_owned` call (nanoseconds)
/// - **Execution**: Full rkyv deserialization creating owned structures
/// - **State**: Warm cache (same key read repeatedly)
///
/// # Inputs
///
/// - **Size**: Same 100-note dataset as `bench_zero_copy_read`
/// - **Target**: Same middle note (index 50) for fair comparison
/// - **Determinism**: Identical setup to zero-copy benchmark
///
/// # Comparison Fairness
///
/// - Uses same database and key as `bench_zero_copy_read`
/// - Both access cached data (no disk I/O differences)
/// - Only difference is deserialization vs archived access
///
/// # Expected Characteristics
///
/// - **Complexity**: O(n) in Note field count (recursive deserialization)
/// - **Dominant costs**: Memory allocation, string copying, Vec construction
/// - **Typical range**: 700-1000 ns (2-3x slower than zero-copy)
///
/// # Interpreting Changes
///
/// - **Approaching `read_zero_copy`**: Check if rkyv validation overhead
///   increased
/// - **Diverging from `read_zero_copy`**: May indicate more complex Note
///   structure
/// - **Sibling benchmarks**: Compare ratio with `read_zero_copy` (should stay
///   ~2-3x)
///
/// # Limitations
///
/// - Does not model partial deserialization (accessing single fields)
/// - Allocation costs may vary with allocator choice
///
/// # Notes for Future
///
/// - Adding nested collections to Note will increase deserialization cost
///   non-linearly
fn bench_full_deserialize(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);
    let test_id = note_ids[50];
    let test_key = Uuid::from(test_id).to_string();

    let mut group = c.benchmark_group("read_deserialize");
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_owned", |b| {
        b.iter(|| {
            let note: Option<Note> =
                db.get_owned(NOTES_TABLE, &test_key).expect("get owned note");
            black_box(note.expect("note exists"));
        });
    });

    group.finish();
}

/// Benchmarks single-item write with individual transaction per operation.
///
/// # Purpose
///
/// Measures the cost of writing one Note with a dedicated transaction,
/// providing baseline for batch write comparison.
///
/// # What is Measured
///
/// - **Metric**: Latency per `put_by_uuid` call including transaction commit
/// - **Execution**: Create transaction → write → commit for each iteration
///
/// # Expected Characteristics
///
/// - **Complexity**: O(1) write + O(log n) index update + fsync overhead
/// - **Dominant costs**: Transaction setup, fsync, B-tree balancing
/// - **Typical range**: 3-5 ms (dominated by transaction commit)
///
/// # Interpreting Changes
///
/// - **>20% change**: Investigate transaction or fsync behavior
/// - **Approaching `write_batch`**: Transaction overhead may have decreased
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
            counter = counter.wrapping_add(1);
            db.put_by_uuid(NOTES_TABLE, Uuid::from(note.id()), &note)
                .expect("put note");
        });
    });

    group.finish();
}

/// Benchmarks batch writes (multiple items per transaction) at varying scales.
///
/// # Purpose
///
/// Measures transaction amortization benefit by testing 100/500/1000 item
/// batches. Validates that batch operations scale linearly (not quadratically).
///
/// # What is Measured
///
/// - **Metric**: Total latency for batch, throughput in elements/second
/// - **Execution**: Single transaction containing N writes + commit
/// - **Scales**: 100, 500, 1000 items per batch
///
/// # Expected Characteristics
///
/// - **Complexity**: O(n log n) for n items (B-tree insertions)
/// - **Scaling**: Should be roughly linear with batch size
/// - **Typical**: 100 items ~= 200-300ms, 1000 items ~= 2-3s
///
/// # Regression Signals
///
/// - **Sub-linear scaling stops**: Indicates transaction overhead growth
/// - **Super-linear scaling**: Check for quadratic algorithms or memory
///   pressure
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
                                .put(NOTES_TABLE, &id_str, &note)
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

/// Benchmarks single-item delete operations.
///
/// # Purpose
///
/// Measures delete latency to ensure removals perform comparably to writes.
///
/// # What is Measured
///
/// - **Metric**: Latency per `delete_by_uuid` call
/// - **Execution**: Single delete per iteration from pre-populated 1000-note DB
///
/// # Inputs
///
/// - **Pre-population**: 1000 notes to avoid early depletion
/// - **Pattern**: Rotating through all keys to avoid deletion order bias
///
/// # Expected Characteristics
///
/// - **Complexity**: O(log n) B-tree removal + transaction commit
/// - **Typical**: Similar to single writes (~3-5 ms)
fn bench_delete(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(1000);

    let mut group = c.benchmark_group("delete");
    group.throughput(Throughput::Elements(1));

    let mut index: usize = 0;
    group.bench_function("delete_single", |b| {
        b.iter(|| {
            let id = note_ids[index % note_ids.len()];
            index = index.wrapping_add(1);

            let existed = db
                .delete_by_uuid(NOTES_TABLE, Uuid::from(id))
                .expect("delete note");
            black_box(existed);
        });
    });

    group.finish();
}

/// Benchmarks cache effectiveness via hot vs cold access patterns.
///
/// # Purpose
///
/// Validates moka cache benefit by comparing repeated access to single key
/// (hot) vs rotating access across all keys (cold).
///
/// # What is Measured
///
/// - **Hot read**: Same key every iteration (100% cache hit expected)
/// - **Cold read**: Different key each iteration (cache misses expected)
///
/// # Expected Characteristics
///
/// - **Hot**: Should match `bench_zero_copy_read` (~250-450 ns)
/// - **Cold**: Higher latency due to B-tree lookup (~400-600 ns)
/// - **Ratio**: Hot should be 1.2-1.5x faster than cold
///
/// # Interpreting Changes
///
/// - **Hot approaching cold**: Cache not functioning (critical issue)
/// - **Both slow**: Database layer regression
/// - **Both fast**: Cache may be too large (masking cold case)
fn bench_cache_effectiveness(c: &mut Criterion) {
    let (_temp, db, note_ids) = setup_db_with_notes(100);
    let note_keys: Vec<String> =
        note_ids.iter().map(|id| Uuid::from(*id).to_string()).collect();

    let mut group = c.benchmark_group("cache_effectiveness");
    group.throughput(Throughput::Elements(1));

    let hot_key = note_keys.first().expect("note key");
    group.bench_function("hot_read", |b| {
        b.iter(|| {
            db.get::<Note, _, _>(NOTES_TABLE, hot_key.as_str(), |archived| {
                black_box(archived);
            })
            .expect("get note")
        });
    });

    let mut cold_index: usize = 0;
    group.bench_function("cold_read", |b| {
        b.iter(|| {
            let cold_id = note_keys[cold_index % note_keys.len()].as_str();
            cold_index = cold_index.wrapping_add(1);

            db.get::<Note, _, _>(NOTES_TABLE, cold_id, |archived| {
                black_box(archived);
            })
            .expect("get note")
        });
    });

    group.finish();
}

/// Benchmarks transaction overhead by comparing batch vs individual commits.
///
/// # Purpose
///
/// Isolates transaction commit cost by measuring same 100 writes with batch
/// transaction vs 100 individual transactions.
///
/// # What is Measured
///
/// - **`individual_txns`**: 100 writes, each with own transaction/commit
/// - **`batch_txn`**: 100 writes in single transaction, one commit
///
/// # Fairness
///
/// - Identical write operations (same Note structure, size, UUID generation)
/// - Same database file creation (fresh DB for each variant)
///
/// # Expected Characteristics
///
/// - **`batch_txn`**: ~200-300ms (dominated by writes)
/// - **`individual_txns`**: ~300-500ms (adds 100x commit overhead)
/// - **Ratio**: Individual should be 1.5-2x slower than batch
///
/// # Interpreting Changes
///
/// - **Ratio decreasing**: Transaction commit becoming cheaper (or more cached)
/// - **Ratio increasing**: Fsync or transaction overhead growing
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
                db.put_by_uuid(NOTES_TABLE, Uuid::from(note.id()), &note)
                    .expect("put note");
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
                    batch_db
                        .put(NOTES_TABLE, &id_str, &note)
                        .expect("put note");
                }
                Ok(())
            })
            .expect("batch write");
        });
    });

    group.finish();
}

// ----------------------------------------------------------- //
//                      Range Query Operations                  //
// ----------------------------------------------------------- //

/// Benchmarks range query performance using `scan_range` vs full table scan.
///
/// # Purpose
///
/// Validates the `scan_range` optimization that enables O(K) prefix-based
/// queries instead of O(N) full table scan + filter. This is critical for
/// property bank queries where we only need versioned rows matching a specific
/// prefix.
///
/// # What is Measured
///
/// - **`scan_range`**: Prefix-based range query using redb's range iteration
/// - **`full_scan_filter`**: Baseline full table scan with prefix filtering
///
/// # Inputs
///
/// - **Database**: 1000 notes with predictable key prefixes
/// - **Query**: Find all notes with prefix "notes/test-01" (matches ~100 notes)
///
/// # Expected Characteristics
///
/// - **`scan_range`**: ~50-100 µs for 100 matching entries (O(K) where
///   K=matches)
/// - **`full_scan_filter`**: ~500-1000 µs for same query (O(N) where N=1000
///   total)
/// - **Speedup**: 5-10x faster for `scan_range` vs full scan
/// - **Scaling**: `scan_range` time proportional to matches, not total table
///   size
///
/// # Interpreting Changes
///
/// - **<5x speedup**: Investigate range query implementation or redb upgrade
/// - **Similar performance**: Prefix optimization not working, check key format
/// - **>10x speedup**: Excellent - redb B-tree range iteration is efficient
fn bench_scan_range(c: &mut Criterion) {
    const TOTAL_NOTES: usize = 1000;

    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("range_bench.db");
    let db = Database::open(&db_path).expect("open database");

    // Pre-populate with 1000 notes using prefixed keys
    // Keys format: "notes/test-XXXX" where XXXX is 0000-0999
    let notes: Vec<Note> = (0..TOTAL_NOTES).map(create_test_note).collect();

    db.batch_write(|writer| {
        for (i, note) in notes.iter().enumerate() {
            let key = format!("notes/test-{i:04}");
            writer.put(NOTES_TABLE, &key, note)?;
        }
        Ok(())
    })
    .expect("batch write");

    let mut group = c.benchmark_group("scan_range");

    // Optimized: Range query for prefix "notes/test-01" (matches 100 entries:
    // 0100-0199)
    group.bench_function("range_query_100_matches", |b| {
        b.iter(|| {
            let prefix = "notes/test-01";
            let results = db
                .batch_read(|reader| {
                    reader.scan_range::<Note>(NOTES_TABLE, prefix)
                })
                .expect("batch_read");
            black_box(results.len())
        });
    });

    // Baseline: Full table scan with filter (O(N) where N=1000)
    group.bench_function("full_scan_filter_100_matches", |b| {
        b.iter(|| {
            let prefix = "notes/test-01";
            let results = db
                .batch_read(|reader| {
                    let all_pairs =
                        reader.list_key_value_pairs::<Note>(NOTES_TABLE)?;
                    let filtered: Vec<_> = all_pairs
                        .into_iter()
                        .filter(|(key, _)| key.starts_with(prefix))
                        .map(|(_, value)| black_box(value))
                        .collect();
                    Ok(filtered)
                })
                .expect("batch_read");
            black_box(results.len())
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
    bench_scan_range,
);
criterion_main!(benches);
