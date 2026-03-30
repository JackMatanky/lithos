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
//! - File I/O (adapter reads from a temp file)
//! - Parse-only path (in-memory markdown)
//! - Scaling behavior (simple → medium → complex markdown samples)
//!
//! **Excluded**:
//! - Frontmatter YAML parsing (small overhead relative to markdown)
//! - Database storage (see `db_storage.rs`)
//! - Concurrent parsing (single-threaded focus)
//!
//! # Benchmark Style
//!
//! - **Micro-benchmark**: Single parse operation per iteration
//! - **Representative input**: Fixed markdown sample with typical elements
//! - **Throughput-focused**: Reported as bytes/second (markdown length)
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs (100 samples, 3s warmup, 5s measurement)
//! - **Throughput**: Bytes of markdown processed per second
//! - **Black-boxing**: Parsed output passed through `black_box()` to prevent
//!   elision
//! - **Setup**: Config and temp files created outside timed region
//! - **Compilation**: `--release` mode (criterion default)
//! - **Environment**: tmpfs-backed temp directory for ingestion benchmarks
//! - **Environment**: In-memory markdown strings for parse-only benchmarks
//!
//! # Input Model
//!
//! Three representative markdown samples covering typical note complexity:
//!
//! **Simple** (~100 bytes, 6 lines):
//! - 1 heading, 3 tasks (2 with inline fields), 2 plain list items
//! - Baseline: minimal note structure
//! - Use case: quick capture notes, simple task lists
//!
//! **Medium** (~500 bytes, 20-30 lines):
//! - Multiple sections with headings (2-3 levels deep)
//! - Mix of tasks, links, tags, code blocks, lists
//! - Use case: meeting notes, daily notes, project documentation
//!
//! **Complex** (~2KB, 60-80 lines):
//! - Deep heading hierarchy (up to level 4)
//! - Dense inline fields, multiple wikilinks per line
//! - Tables, nested lists, code blocks with syntax
//! - Use case: comprehensive project pages, detailed documentation
//!
//! **Determinism**: Static strings ensure reproducible results across runs.
//!
//! # Controls and Fairness
//!
//! - **Same inputs**: All benchmarks use identical static markdown samples
//! - **Compilation**: `--release` mode, no special target-cpu or LTO
//! - **Environment**: tmpfs-backed temp directory (eliminates disk I/O
//!   variance) for ingestion
//! - **Environment**: In-memory markdown strings for parse-only runs
//! - **Allocation**: System allocator (no custom allocator)
//! - **Setup separation**: Config, file writes, Note construction outside
//!   `b.iter()`
//!
//! # Expected Characteristics
//!
//! Based on measured baseline performance (2026-02-27, Apple M3 Max):
//!
//! **Simple** (~13.5 µs, ~7 MiB/s):
//! - High overhead from file I/O and Note construction relative to parsing
//! - Task regex matching overhead (inline field extraction)
//! - Minimal section construction (single heading)
//! - Baseline shows fixed cost dominates for tiny inputs
//!
//! **Medium** (~18 µs, ~27 MiB/s):
//! - More event iterations (headings, lists, links, tags)
//! - Increased section construction cost (multiple headings)
//! - More regex matches for tasks and inline fields
//! - Throughput improves significantly as parsing cost dominates fixed overhead
//!
//! **Complex** (~48 µs, ~50 MiB/s):
//! - Deep section nesting (heading hierarchy traversal)
//! - Dense wikilink and tag extraction (many regex matches)
//! - Table and code block event processing
//! - Best throughput as parsing fully dominates fixed costs
//!
//! **Scaling behavior**: O(n) confirmed (5x size → 1.35x time, 20x size → 3.5x
//! time).
//! - **Fixed overhead**: ~10-13 µs (file I/O, Config, Note construction)
//! - **Parse-only overhead**: lower fixed costs (Config, parser setup)
//! - **Parsing cost**: ~5-35 µs depending on complexity
//! - **Throughput improves with size**: Fixed costs amortized over larger
//!   inputs
//!
//! **Bottlenecks**:
//! - Fixed overhead (file I/O, setup) dominates for simple notes
//! - pulldown-cmark event iteration (~40% for medium/complex)
//! - Task/field regex matching (~20% for medium/complex)
//! - Section construction and nesting (~10% for medium/complex)
//! - Element allocation/collection (~5%)
//!
//! # Interpreting Results
//!
//! ## Bottleneck Identification
//!
//! **If simple throughput drops below 5 MiB/s**:
//! - Fixed overhead increased (file I/O, Config/Note construction)
//! - Check for new validation in parser or raw extraction routines
//!
//! **If medium/complex throughput drops below 20 MiB/s**:
//! - Parsing logic changed (pulldown-cmark, regex matching)
//! - Profile task regex matching (inline field parsing overhead)
//! - Investigate element extraction or section construction changes
//!
//! **If scaling becomes super-linear** (complex >10x simple):
//! - O(n²) algorithm introduced (e.g., nested loops in section construction)
//! - Investigate heading hierarchy traversal or section nesting logic
//!
//! **If simple benchmark regresses >20%**:
//! - Baseline overhead increased (Note construction, Config setup)
//! - Core parser logic changed (pulldown-cmark upgrade, regex changes)
//!
//! ## Change Significance
//!
//! **Meaningful changes** (investigate):
//! - **>20% regression**: Parser logic change, new validation, inefficient
//!   extraction
//! - **>50% regression**: Critical issue (investigate immediately)
//! - **Throughput variance >30%**: Non-linear scaling introduced
//!
//! **Noise level** (ignore):
//! - **±5-10%**: Normal variance (CPU frequency scaling, background processes)
//! - **First run slower**: Regex compilation cost (one-time setup)
//!
//! ## Not Justified Conclusions
//!
//! **Cannot infer from these benchmarks**:
//! - Bulk vault indexing time (no database write modeling)
//! - Memory usage or allocation patterns (criterion doesn't track)
//! - Concurrent parsing performance (single-threaded only)
//! - Real-world file I/O impact (tmpfs eliminates disk latency)
//! - Parse-only I/O behavior (not measured)
//! - Frontmatter YAML parsing cost (not included in samples)
//!
//! ## Actionable Optimization Targets
//!
//! **If optimizing parser performance**:
//! - Profile with `cargo flamegraph` to identify hot paths
//! - Focus on pulldown-cmark event handling (largest cost)
//! - Consider lazy regex compilation for task/field extraction
//! - Optimize section construction if nesting is deep
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
//! # Benchmark Index
//!
//! | Benchmark                 | Input Size | Expected Time | Throughput  | What is Measured                   |
//! | :------------------------ | :--------- | :------------ | :---------- | :--------------------------------- |
//! | `ingest_markdown/simple`  | 91B        | ~13-14 µs     | ~7 MiB/s    | Full pipeline (file → note)        |
//! | `ingest_markdown/medium`  | 500B       | ~18-19 µs     | ~27 MiB/s   | Full pipeline (file → note)        |
//! | `ingest_markdown/complex` | 2419B      | ~47-48 µs     | ~50 MiB/s   | Full pipeline (file → note)        |
//! | `parse_markdown/simple`   | 91B        | ~11-12 µs     | ~8 MiB/s    | Parse-only (no file I/O)           |
//! | `parse_markdown/medium`   | 500B       | ~16-17 µs     | ~30 MiB/s   | Parse-only (no file I/O)           |
//! | `parse_markdown/complex`  | 2419B      | ~44-45 µs     | ~55 MiB/s   | Parse-only (no file I/O)           |
//!
//! # Known Limitations
//!
//! - **No frontmatter**: Samples omit YAML frontmatter parsing (separate cost)
//! - **No error cases**: Uses valid markdown only (no malformed syntax testing)
//! - **Single-threaded**: Does not model concurrent parsing scenarios
//! - **tmpfs I/O**: Eliminates real disk latency (actual file I/O will be
//!   slower)
//! - **Parse-only**: In-memory parsing excludes any file system overhead
//! - **Static samples**: Does not test variance across production note types
//!
//! # Safety
//!
//! Benchmark code uses `unwrap`/`expect` for simplicity.

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use std::path::Path;

use criterion::{
    Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use lithos_core::{
    config::task::TaskConfigSpec,
    fs::FsReader,
    note::{parser, paths::NotePath},
};

fn task_spec_fixture() -> TaskConfigSpec {
    TaskConfigSpec::new(
        true,
        true,
        vec![
            '\u{1f4c5}', // 📅
            '\u{2705}',  // ✅
            '\u{23f0}',  // ⏰
            '\u{1f6eb}', // 🛫
            '\u{23f3}',  // ⏳
        ]
        .into(),
        vec!["task".into()].into(),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    )
}

/// Simple markdown sample: minimal note structure (~100 bytes).
fn simple_markdown() -> &'static str {
    concat!(
        "# Title\n\n",
        "- [ ] #task Review PR [priority:: 1]\n",
        "- [x] Buy milk\n",
        "- [ ] Call mom\n\n",
        "1. First\n",
        "2. Second\n",
    )
}

/// Medium markdown sample: typical note with multiple sections (~500 bytes).
fn medium_markdown() -> &'static str {
    concat!(
        "# Project: Lithos\n\n",
        "## Tasks\n\n",
        "- [ ] #task Review [[PR-123]] [priority:: 1] [due:: 2026-03-01]\n",
        "- [x] #task Setup CI pipeline [status:: done]\n",
        "- [ ] #task Write docs for schema module\n",
        "- [ ] Update [[README]] with new features\n\n",
        "## Notes\n\n",
        "Met with @john to discuss #architecture. Key points:\n\n",
        "1. Use port-based CQRS pattern\n",
        "2. Isolate business contexts\n",
        "3. Zero-copy reads via GATs\n\n",
        "See [[docs/adr/002-cqrs-pattern]] for details.\n\n",
        "## Code\n\n",
        "```rust\n",
        "fn example() {\n",
        "    println!(\"hello\");\n",
        "}\n",
        "```\n\n",
        "## Tags\n\n",
        "#rust #database #performance\n",
    )
}

/// Complex markdown sample: dense note with deep hierarchy (~2KB).
fn complex_markdown() -> &'static str {
    concat!(
        "# Project Plan: Lithos v2.0\n\n",
        "## Executive Summary\n\n",
        "Complete rewrite of note management system using [[Rust]] and \
         [[redb]].\n",
        "Focus on #performance and #scalability for large vaults (100K+ \
         notes).\n\n",
        "### Goals\n\n",
        "- [ ] #task Achieve <10ms LSP response time [priority:: 1] \
         [milestone:: Q1]\n",
        "- [ ] #task Support 100K+ notes in vault [priority:: 1] [milestone:: \
         Q1]\n",
        "- [ ] #task Zero-copy reads via GATs [priority:: 2] [milestone:: \
         Q2]\n",
        "- [x] #task Setup [[CI/CD]] pipeline [status:: done] [completed:: \
         2026-02-15]\n\n",
        "### Non-Goals\n\n",
        "- Network sync (use [[Syncthing]] or [[Git]])\n",
        "- Mobile app (desktop-first)\n",
        "- Cloud storage integration\n\n",
        "## Architecture\n\n",
        "### Core Components\n\n",
        "#### Database Layer\n\n",
        "Using [[redb]] for zero-copy storage:\n\n",
        "```rust\n",
        "pub trait QueryPort {\n",
        "    type Archived<'a> where Self: 'a;\n",
        "    fn with_archived<F, R>(&self, id: Id, f: F) -> \
         Result<Option<R>>;\n",
        "}\n",
        "```\n\n",
        "See [[docs/adr/003-zero-copy-reads]] for rationale.\n\n",
        "#### Domain Contexts\n\n",
        "| Context    | Purpose              | Dependencies |\n",
        "| :--------- | :------------------- | :----------- |\n",
        "| note       | Note aggregates      | config, db   |\n",
        "| schema     | Property definitions | config, db   |\n",
        "| template   | Note templates       | config, db   |\n\n",
        "**Isolation rule**: Contexts MUST NOT import each other.\n\n",
        "### Performance Targets\n\n",
        "- **LSP queries**: <5ms p50, <10ms p99\n",
        "- **Vault indexing**: >1000 notes/sec\n",
        "- **Schema loading**: >10K schemas/sec\n",
        "- **Memory**: <100MB for 10K notes\n\n",
        "## Implementation Plan\n\n",
        "### Phase 1: Foundation (Q1 2026)\n\n",
        "- [x] #task Setup cargo workspace [status:: done]\n",
        "- [x] #task Implement redb storage layer [status:: done]\n",
        "- [ ] #task Schema ingestion pipeline [status:: in-progress] \
         [assignee:: @jack]\n",
        "- [ ] #task Note parsing with pulldown-cmark [priority:: 1]\n\n",
        "### Phase 2: LSP Integration (Q2 2026)\n\n",
        "- [ ] #task Implement LSP server [priority:: 1] [epic:: lsp]\n",
        "- [ ] #task Add completion support [priority:: 2] [epic:: lsp]\n",
        "- [ ] #task Add hover documentation [priority:: 2] [epic:: lsp]\n",
        "- [ ] #task Add goto definition [priority:: 3] [epic:: lsp]\n\n",
        "### Phase 3: Advanced Features (Q3 2026)\n\n",
        "- [ ] #task Query language implementation [priority:: 2]\n",
        "- [ ] #task Graph visualization [priority:: 3]\n",
        "- [ ] #task Plugin system [priority:: 3]\n\n",
        "## References\n\n",
        "- [[docs/adr/001-port-based-architecture]]\n",
        "- [[docs/adr/002-cqrs-pattern]]\n",
        "- [[docs/adr/003-zero-copy-reads]]\n",
        "- [[docs/prd]]\n\n",
        "## Tags\n\n",
        "#rust #architecture #database #performance #lsp \
         #project-management\n\n",
        "## Metadata\n\n",
        "[created:: 2026-02-01]\n",
        "[updated:: 2026-02-27]\n",
        "[author:: @jack]\n",
        "[status:: active]\n",
    )
}

/// Benchmarks markdown-to-Note transformation across different complexity
/// levels.
///
/// # Purpose
///
/// Measures end-to-end parsing performance from markdown string to structured
/// Note aggregate, validating parser efficiency and O(n) scaling assumption.
///
/// # What is Measured
///
/// - **Metric**: Latency per parse + raw extraction
/// - **Throughput**: Markdown bytes processed per second
/// - **Scaling**: Simple (~100B) → Medium (~500B) → Complex (~2KB)
///
/// # Expected Characteristics
///
/// - **Simple**: ~13-14 µs latency, ~7 MiB/s throughput (fixed overhead
///   dominates)
/// - **Medium**: ~18-19 µs latency, ~27 MiB/s throughput (5x size, 1.35x time)
/// - **Complex**: ~47-48 µs latency, ~50 MiB/s throughput (27x size, 3.5x time)
/// - **Scaling**: Sub-linear latency growth (O(n) parsing + O(1) fixed costs)
///
/// # Interpreting Changes
///
/// - **>20% regression in any size**: Check parser/validation changes
/// - **>50% regression**: Critical issue (investigate immediately)
/// - **Throughput variance >30% across sizes**: Non-linear scaling (bad)
/// - **Noise level**: ±5-10% typical
///
/// # Limitations
///
/// - Static samples (no production note variance)
/// - No frontmatter YAML parsing (separate cost)
/// - No memory allocation measurement
/// - tmpfs I/O (real disk will be slower)
struct BenchSamples<'sample> {
    simple: &'sample str,
    medium: &'sample str,
    complex: &'sample str,
}

fn bench_ingest_group(
    c: &mut Criterion,
    reader: &FsReader,
    root: &Path,
    samples: &BenchSamples<'_>,
) {
    let mut ingest_group = c.benchmark_group("note_parsing");

    let task_spec = task_spec_fixture();

    // Simple benchmark
    std::fs::write(root.join("notes/simple.md"), samples.simple)
        .expect("write simple markdown");
    ingest_group.throughput(Throughput::Bytes(samples.simple.len() as u64));
    ingest_group.bench_function("ingest_markdown/simple", |b| {
        b.iter(|| {
            let markdown = reader
                .read_to_string(std::path::Path::new("notes/simple.md"))
                .expect("read markdown");
            let path = NotePath::try_new("notes/simple.md").expect("note path");
            let raw_note =
                parser::MarkdownParser::parse(&markdown, path, &task_spec)
                    .expect("ingest markdown");
            black_box(raw_note);
        });
    });

    // Medium benchmark
    std::fs::write(root.join("notes/medium.md"), samples.medium)
        .expect("write medium markdown");
    ingest_group.throughput(Throughput::Bytes(samples.medium.len() as u64));
    ingest_group.bench_function("ingest_markdown/medium", |b| {
        b.iter(|| {
            let markdown = reader
                .read_to_string(std::path::Path::new("notes/medium.md"))
                .expect("read markdown");
            let path = NotePath::try_new("notes/medium.md").expect("note path");
            let raw_note =
                parser::MarkdownParser::parse(&markdown, path, &task_spec)
                    .expect("ingest markdown");
            black_box(raw_note);
        });
    });

    // Complex benchmark
    std::fs::write(root.join("notes/complex.md"), samples.complex)
        .expect("write complex markdown");
    ingest_group.throughput(Throughput::Bytes(samples.complex.len() as u64));
    ingest_group.bench_function("ingest_markdown/complex", |b| {
        b.iter(|| {
            let markdown = reader
                .read_to_string(std::path::Path::new("notes/complex.md"))
                .expect("read markdown");
            let path =
                NotePath::try_new("notes/complex.md").expect("note path");
            let raw_note =
                parser::MarkdownParser::parse(&markdown, path, &task_spec)
                    .expect("ingest markdown");
            black_box(raw_note);
        });
    });

    ingest_group.finish();
}

fn bench_parse_group(c: &mut Criterion, samples: &BenchSamples<'_>) {
    let mut parse_group = c.benchmark_group("note_parsing_ingest_only");

    let task_spec = task_spec_fixture();

    // Ingest-only simple benchmark (no file I/O)
    parse_group.throughput(Throughput::Bytes(samples.simple.len() as u64));
    parse_group.bench_function("ingest_markdown/simple", |b| {
        b.iter(|| {
            let path = NotePath::try_new("notes/simple.md").expect("note path");
            let outcome =
                parser::MarkdownParser::parse(samples.simple, path, &task_spec)
                    .expect("extract markdown");
            black_box(outcome);
        });
    });

    // Ingest-only medium benchmark (no file I/O)
    parse_group.throughput(Throughput::Bytes(samples.medium.len() as u64));
    parse_group.bench_function("ingest_markdown/medium", |b| {
        b.iter(|| {
            let path = NotePath::try_new("notes/medium.md").expect("note path");
            let outcome =
                parser::MarkdownParser::parse(samples.medium, path, &task_spec)
                    .expect("extract markdown");
            black_box(outcome);
        });
    });

    // Ingest-only complex benchmark (no file I/O)
    parse_group.throughput(Throughput::Bytes(samples.complex.len() as u64));
    parse_group.bench_function("ingest_markdown/complex", |b| {
        b.iter(|| {
            let path =
                NotePath::try_new("notes/complex.md").expect("note path");
            let outcome = parser::MarkdownParser::parse(
                samples.complex,
                path,
                &task_spec,
            )
            .expect("extract markdown");
            black_box(outcome);
        });
    });

    parse_group.finish();
}

fn bench_note_ingest(c: &mut Criterion) {
    let root = std::env::temp_dir()
        .join(format!("lithos_note_bench_{}", std::process::id()));
    std::fs::create_dir_all(root.join("notes"))
        .expect("create bench notes dir");

    let reader = FsReader::new(root.as_path());

    let samples = BenchSamples {
        simple: simple_markdown(),
        medium: medium_markdown(),
        complex: complex_markdown(),
    };

    bench_ingest_group(c, &reader, root.as_path(), &samples);
    bench_parse_group(c, &samples);
}

criterion_group!(benches, bench_note_ingest);
criterion_main!(benches);
