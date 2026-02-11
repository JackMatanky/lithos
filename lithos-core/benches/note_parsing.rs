//! Markdown parsing performance benchmarks for note ingestion.
//!
//! # Summary
//!
//! Measures markdown-to-Note transformation performance to detect regressions
//! in parsing logic and validate complexity assumptions.
//!
//! # Motivation
//!
//! Note ingestion is a hot path during vault indexing and note creation.
//! Parsing must remain fast (<10 µs) to support interactive editing and bulk
//! imports. This benchmark tracks parser performance as markdown features
//! evolve.
//!
//! # Scope
//!
//! **Included**:
//! - Markdown parsing via pulldown-cmark
//! - Task extraction (checkbox syntax, inline fields)
//! - Link extraction (wikilinks, markdown links)
//! - Tag extraction (hashtag syntax)
//! - Heading extraction
//! - Section construction
//!
//! **Excluded**:
//! - Frontmatter YAML parsing (small overhead relative to markdown)
//! - Database storage (see `db_storage.rs`)
//! - File I/O (benchmarks use in-memory strings)
//!
//! # Benchmark Style
//!
//! - **Micro-benchmark**: Single parse operation per iteration
//! - **Representative input**: Fixed markdown sample with typical elements
//! - **Throughput-focused**: Reported as bytes/second (markdown length)
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs (default configuration)
//! - **Throughput**: Bytes of markdown processed per second
//! - **Black-boxing**: Parsed Note passed through `black_box()` to prevent
//!   elision
//! - **Setup**: Note structure created outside timed region
//!
//! # Input Model
//!
//! - **Markdown**: Fixed 6-line sample (title, 3 tasks, 2 list items)
//! - **Size**: ~100 bytes (typical for simple notes)
//! - **Elements**: 1 heading, 3 tasks (2 with inline fields), 2 plain list
//!   items
//! - **Determinism**: Static string ensures reproducible results
//! - **Representativeness**: Typical note complexity (not worst-case)
//!
//! # Expected Characteristics
//!
//! - **Latency**: ~3.5 µs per parse (26 MiB/s throughput)
//! - **Complexity**: O(n) in markdown length + O(m) in element count
//! - **Dominant costs**: pulldown-cmark event iteration, task regex matching
//!
//! # Interpreting Results
//!
//! **Meaningful changes**:
//! - **>20% regression**: Investigate parser changes or new validation logic
//! - **Sub-linear scaling**: Good (indicates efficient parsing)
//! - **Super-linear scaling**: Bad (may indicate O(n²) algorithms)
//!
//! **Noise sources**:
//! - Regex compilation cost (first run may be slower)
//! - CPU frequency scaling (lock frequency for precision)
//!
//! **Not justified conclusions**:
//! - Bulk vault indexing time (does not model file I/O, database writes)
//! - Memory usage (criterion does not track allocations)
//!
//! # Maintenance Contract
//!
//! **Update when**:
//! - Markdown parsing logic changes (new task syntax, link formats)
//! - Note domain model changes (new fields requiring extraction)
//! - Parser library upgrades (pulldown-cmark version bumps)
//!
//! **Adding benchmarks**:
//! - Use realistic markdown samples from production notes
//! - Test varying complexities (simple/medium/complex)
//! - Document expected element counts in sample
//!
//! # Known Limitations
//!
//! - **Single sample**: Does not test variance across note types
//! - **No frontmatter**: Sample omits YAML frontmatter parsing
//! - **No large notes**: Does not test >1KB markdown documents
//! - **No error cases**: Uses valid markdown only (no malformed syntax)
//!
//! # Safety
//!
//! Benchmark code uses `unwrap`/`expect` for simplicity.

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use criterion::{
    Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    note::{
        aggregate::{Note, NoteId},
        parser::NoteParser,
    },
};

fn sample_markdown() -> &'static str {
    concat!(
        "# Title\n\n",
        "- [ ] #task Review PR [priority:: 1]\n",
        "- [x] Buy milk\n",
        "- [ ] Call mom\n\n",
        "1. First\n",
        "2. Second\n",
    )
}

/// Benchmarks markdown-to-Note transformation with typical elements.
///
/// # Purpose
///
/// Measures end-to-end parsing performance from markdown string to structured
/// Note aggregate, validating parser efficiency.
///
/// # What is Measured
///
/// - **Metric**: Latency per `NoteParser::apply()` call
/// - **Throughput**: Markdown bytes processed per second
/// - **Input**: 6-line sample (1 heading, 3 tasks, 2 list items, ~100 bytes)
///
/// # Expected Characteristics
///
/// - **Latency**: ~3.5 µs (from RESULTS.md)
/// - **Throughput**: ~26 MiB/s
/// - **Scaling**: O(n) in markdown length
///
/// # Interpreting Changes
///
/// - **>20% regression**: Check for new validation or inefficient task parsing
/// - **>50% regression**: Critical issue (investigate immediately)
/// - **Noise level**: ±5-10% typical
///
/// # Limitations
///
/// - Fixed simple input (does not test large notes or complex frontmatter)
/// - Does not measure memory allocations
fn bench_note_ingest(c: &mut Criterion) {
    let config = Config::build(
        &RawConfig::default(),
        VaultId::new(),
        VaultRoot::try_new(std::path::PathBuf::from("/vault"))
            .expect("valid vault root"),
    )
    .expect("config");
    let markdown = sample_markdown();

    let mut group = c.benchmark_group("note_parsing");
    group.throughput(Throughput::Bytes(markdown.len() as u64));

    group.bench_function("ingest_markdown", |b| {
        b.iter(|| {
            let mut note =
                Note::new(NoteId::new(), "notes/bench.md").expect("valid note");
            NoteParser::new(&config)
                .apply(&mut note, black_box(markdown))
                .expect("ingest markdown");
            black_box(note);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_note_ingest);
criterion_main!(benches);
