//! Schema ingestion pipeline performance benchmarks.
//!
//! # Summary
//!
//! Benchmarks the full schema ingestion pipeline from file I/O through
//! resolution, measuring each stage individually and end-to-end to identify
//! performance bottlenecks and detect regressions.
//!
//! # Motivation
//!
//! Schema loading happens at vault startup and impacts initial LSP response
//! time. The pipeline has multiple distinct stages (file I/O, parsing,
//! dereferencing, DAG construction, property merging) with different
//! performance characteristics. Measuring each stage in isolation enables
//! targeted optimization and regression detection.
//!
//! Historical context: Initial benchmarks only measured file I/O and
//! PropertyBank validation, missing 80%+ of the actual pipeline cost (DAG
//! construction, property merging). This comprehensive suite provides
//! actionable performance data for optimization decisions.
//!
//! # Scope
//!
//! **Included**:
//! - File I/O + Parsing (TOML/JSON → RawSchema via Ingestor)
//! - PropertyBank validation (RawPropertyBank → PropertyBank)
//! - PropertyBank lookup performance (HashMap get for $ref resolution)
//! - Dereferencing ($ref resolution against PropertyBank)
//! - DAG construction (topological sort, cycle detection via Extender)
//! - Property merging (inheritance resolution via Resolver)
//! - Full pipeline end-to-end (file → resolved Schema)
//! - Scaling behavior (tiny → small → medium → large vaults)
//!
//! **Excluded**:
//! - Database staleness checking (requires DB integration)
//! - Database write performance (Command adapter benchmarks)
//! - Concurrent access patterns (single-threaded focus)
//! - Cache warming effects (fresh setup per benchmark)
//! - Network I/O, distributed scenarios
//!
//! # Benchmark Style
//!
//! - **Micro-benchmarks**: Individual pipeline stages measured in isolation
//! - **Macro-benchmark**: Full end-to-end pipeline including all stages
//! - **Scaling-focused**: Tests across vault sizes (5, 20, 40, 100 schemas)
//! - **Single-threaded**: No concurrent loading (matches production usage)
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs (100 samples, 3s warmup, 5s measurement)
//! - **Black-boxing**: All results through `black_box()` to prevent elision
//! - **Setup separation**: Test data generation outside timed region
//! - **Measurement focus**: Operation latency and throughput (schemas/sec)
//! - **Compilation**: `--release` mode (criterion default)
//! - **Environment**: tmpfs-backed `TempDir` eliminates disk I/O variance
//!
//! # Input Model
//!
//! Based on real schemas from `example_vault/.lithos/schemas/`:
//!
//! **Property Bank** (19 properties):
//! - String types (simple, with options, with patterns)
//! - Number types (with bounds, with step validation)
//! - Date types (strftime format strings: `%Y-%m-%d`, `%Y-%m-%dT%H:%M`)
//! - File types (directory constraints, file_class filters)
//! - Mix of single and multi-value properties (`multi: true`)
//!
//! **Schemas** (5 base templates):
//! - **task**: 14 properties (complex, uses $ref extensively)
//! - **task_project**: extends task, overrides type, excludes 3 properties
//! - **task_meeting**: extends task, simple override
//! - **lib**: 3 properties (moderate complexity)
//! - **lib_book**: extends lib, adds 1 property
//!
//! **Inheritance structure**:
//! - 2-level max depth (task → task_project)
//! - Mix of root and derived schemas (3 root, 2 derived)
//! - Property exclusion and override patterns
//! - ~60% of properties use $ref (realistic $ref density)
//!
//! **Vault Sizes**:
//! - **Tiny**: 5 schemas (matches SCHEMAS template set exactly)
//! - **Small**: 20 schemas (4x replication with naming variations)
//! - **Medium**: 40 schemas (8x replication)
//! - **Large**: 100 schemas (20x replication, stress test)
//!
//! # Controls and Fairness
//!
//! - **Same inputs**: All benchmarks use identical schema templates
//! - **Deterministic IDs**: UUIDs are v7 (time-based) but measurements are
//!   stable
//! - **Compilation**: `--release` mode, no special target-cpu or LTO
//! - **Environment**: tmpfs-backed `TempDir` for file operations (eliminates
//!   disk I/O variance)
//! - **Allocation**: System allocator (no custom allocator)
//! - **Setup separation**: Vault generation, data parsing outside `b.iter()`
//!
//! # Expected Characteristics
//!
//! Based on typical schema workloads and architectural analysis:
//!
//! **File I/O + Parse** (~200-500 µs for tiny, ~2-5 ms for large):
//! - Dominated by serde deserialization (TOML/JSON → RawSchema)
//! - Should scale linearly with schema count: O(n)
//! - tmpfs eliminates disk I/O latency (pure CPU deserialization)
//! - Bottleneck: serde overhead, not filesystem
//!
//! **PropertyBank Validation** (~20-30 µs, constant):
//! - PropertySpec construction and validation logic
//! - Should be O(1) regardless of schema count (fixed 19 properties)
//! - Recent optimizations measured: HIGH-001 (regex caching), E-02 (epsilon
//!   handling)
//! - Bottleneck: PropertySpec validation, not HashMap construction
//!
//! **PropertyBank Lookup** (~5-10 ns per lookup):
//! - HashMap get performance for $ref resolution
//! - Should be O(1) with good hash distribution
//! - Measures hot path cost during dereferencing
//! - Bottleneck: hash function quality, not table size
//!
//! **Dereferencing** (~50-150 µs for tiny, ~1-3 ms for large):
//! - $ref pointer resolution against PropertyBank
//! - HashMap lookups + PropertySpec cloning per $ref
//! - Should scale linearly with total $ref count: O(n * avg_refs_per_schema)
//! - Bottleneck: PropertySpec cloning, not HashMap lookups
//!
//! **DAG Construction** (~30-80 µs for tiny, ~200-500 µs for large):
//! - Topological sort via Kahn's algorithm
//! - Cycle detection via DFS
//! - Should scale O(n + e) where e = inheritance edges
//! - Bottleneck: HashMap operations, not algorithm itself
//!
//! **Property Merging** (~40-100 µs for tiny, ~300-800 µs for large):
//! - Two-pointer sorted merge for inheritance
//! - Arc<Property> sharing for memory efficiency
//! - Should scale linearly with (depth * property_count): O(n * d * p)
//! - Bottleneck: Arc cloning + merge logic, not traversal
//!
//! **Total Pipeline** (sum of stages):
//! - Tiny (5 schemas): ~400-900 µs
//! - Large (100 schemas): ~5-12 ms
//! - Should scale linearly overall (file I/O dominates at scale)
//! - Expected distribution: File I/O ~50%, Deref ~25%, DAG ~10%, Merge ~10%,
//!   Validation ~5%
//!
//! # Interpreting Results
//!
//! **Bottleneck Identification**:
//! - If **file I/O** >60% of total → normal, serde-bound (expected)
//! - If **dereferencing** >35% of total → PropertySpec cloning overhead,
//!   consider lazy evaluation
//! - If **DAG construction** >20% of total → HashMap overhead or algorithm
//!   issue
//! - If **property merging** >25% of total → Arc cloning overhead, verify
//!   sharing
//! - If **PropertyBank validation** >10% of total → PropertySpec construction
//!   regressed
//!
//! **Meaningful Changes**:
//! - >20% regression in any single stage → investigate root cause immediately
//! - >10% regression in total pipeline → likely user-visible startup impact
//! - Scaling becomes superlinear (O(n²)) → algorithm regression (critical bug)
//! - Throughput drops below 20K schemas/sec → performance degradation
//!
//! **Regression Signals** (watch for):
//! - PropertyBank validation time increasing → PropertySpec construction
//!   regressed (check HIGH-001 regex caching)
//! - PropertyBank lookup >15 ns → HashMap hash function degraded
//! - Dereferencing not scaling linearly → HashMap collisions or excessive
//!   cloning
//! - DAG construction spiking → cycle detection or topological sort algorithm
//!   issue
//! - Property merging growing faster than O(n) → Arc sharing broken or merge
//!   logic regressed
//! - Full pipeline >15 ms for 100 schemas → investigate all stages
//!
//! **Valid Comparisons**:
//! - Within-machine, same session: Reliable (±5% expected variance)
//! - Across machines: Trends only, not absolute numbers
//! - Before/after code changes: Use `--save-baseline` and `--baseline`
//! - CI vs local: Expect significant variance, use for trend detection only
//!
//! **Noise Sources**:
//! - File system cache state (mitigated by tmpfs + consistent setup)
//! - Background processes (close unnecessary applications during benchmark)
//! - Thermal throttling (long runs may show progressive degradation)
//! - System allocator variance (first allocation may be slower)
//! - CPU frequency scaling (enable performance mode for consistency)
//!
//! # Reporting and Workflow
//!
//! **Local development**:
//! ```bash
//! # Establish baseline before changes
//! cargo bench --bench schema_loader -- --save-baseline before_changes
//!
//! # Make code changes...
//!
//! # Compare against baseline
//! cargo bench --bench schema_loader -- --baseline before_changes
//! ```
//!
//! **PR workflow**:
//! - Run full suite before creating PR
//! - Note any regressions/improvements in PR description
//! - Include criterion HTML report links if significant changes
//! - Flag >10% regressions for review
//!
//! **Performance tracking**:
//! - Baseline numbers documented in `benches/RESULTS.md`
//! - Update RESULTS.md after confirmed optimizations
//! - Track trends over time for each stage
//!
//! # Maintenance Contract
//!
//! **Update when**:
//! - Schema file format changes (new fields, TOML structure changes)
//! - PropertyBank structure changes (new property types, validation logic
//!   changes)
//! - Resolution pipeline changes (dereferencer, extender, resolver module
//!   refactoring)
//! - Domain model changes (Schema, Property, PropertySpec field additions)
//! - Performance characteristics change (new hash function, different data
//!   structures)
//!
//! **Adding benchmarks**:
//! - Group by pipeline stage (file_io, validation, lookup, deref, dag, merge,
//!   total)
//! - Use `Throughput::Elements(n)` for per-item measurements
//! - Follow `bench_<stage>_<variant>` naming convention
//! - Document expected complexity and bottlenecks in per-bench comment
//! - Always black-box results to prevent compiler elision
//! - Separate setup from measurement (use `b.iter()` for timed code only)
//!
//! **Stability expectations**:
//! - Results stable within ±5% across runs on same machine
//! - Flaky benchmarks (>10% variance) must be investigated or removed
//! - Use `--quick` mode for fast feedback during development
//! - Use full mode for accurate measurements before commits
//!
//! # Known Limitations
//!
//! - **No DB integration**: Cannot measure staleness checking, hybrid vs cold
//!   start comparison
//! - **Synthetic data**: Uses example_vault templates, not production corpus
//!   (real vaults may have different $ref density, inheritance depth)
//! - **tmpfs variance**: File I/O measurements may not reflect real SSD/HDD
//!   performance (typically faster)
//! - **No concurrency**: Single-threaded only, does not test parallel schema
//!   loading (future optimization opportunity)
//! - **No cache warming**: Does not model repeated schema access patterns or
//!   PropertyBank reuse
//! - **Module visibility**: Internal modules exposed via `#[doc(hidden)] pub`
//!   for benchmarking (not part of public API)
//!
//! # Benchmark Index
//!
//! | Group                        | Focus                                              | Expected Time |
//! | ---------------------------- | -------------------------------------------------- | ------------- |
//! | `file_io_and_parse`          | TOML/JSON deserialization (Ingestor)              | 200-500 µs    |
//! | `property_bank_validation`   | PropertySpec construction and validation           | 20-30 µs      |
//! | `property_bank_lookup`       | HashMap get performance for $ref resolution        | 5-10 ns       |
//! | `dereferencing`              | $ref resolution against PropertyBank (Dereferencer)| 50-150 µs     |
//! | `dag_construction`           | Topological sort and cycle detection (Extender)    | 30-80 µs      |
//! | `property_merging`           | Inheritance resolution (Resolver)                  | 40-100 µs     |
//! | `full_pipeline`              | End-to-end ingestion (file → resolved Schema)      | 400-900 µs    |
//!
//! # Safety
//!
//! Benchmark code uses `unwrap`/`expect` for simplicity (failures indicate test
//! setup errors, not runtime conditions). Production code should never panic.

#![allow(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown,
    clippy::doc_paragraphs_missing_punctuation,
    clippy::shadow_unrelated,
    clippy::pattern_type_mismatch,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::type_complexity,
    clippy::default_numeric_fallback,
    clippy::excessive_nesting,
    dead_code,
    reason = "Benchmark code: simplified error handling, test-only functions"
)]

use std::{collections::HashMap, fs};

use criterion::{
    BenchmarkId, Criterion, Throughput, black_box, criterion_group,
    criterion_main,
};
use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    db::Database,
    fs::FsReader,
    schema::{
        adapter::{
            command::CommandAdapter, ingestor::Ingestor, query::QueryAdapter,
        },
        aggregate::{Schema, SchemaId, Timestamp},
        bank::{BankVersion, PropertyBank},
        command::Command,
        dereferencer::Dereferencer,
        extender::Extender,
        query::Query,
        raw::RawSchema,
        resolver::Resolver,
    },
};
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────────────────
//  Test Data Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Vault size configuration for scaling tests.
struct VaultSize {
    name: &'static str,
    schema_count: usize,
}

const VAULT_SIZES: &[VaultSize] = &[
    VaultSize {
        name: "tiny",
        schema_count: 5,
    },
    VaultSize {
        name: "small",
        schema_count: 20,
    },
    VaultSize {
        name: "medium",
        schema_count: 40,
    },
    VaultSize {
        name: "large",
        schema_count: 100,
    },
];

/// Create a minimal config for benchmarking.
fn bench_config(vault_root: &std::path::Path) -> Config {
    let mut raw = RawConfig::default();
    // Configure schemas_dir to match generate_vault structure (.lithos/schemas)
    raw.paths.schemas_dir = Some(".lithos/schemas".to_owned());

    let vault_id = VaultId::new();
    let vault_root_str = vault_root.to_string_lossy().to_string();
    let vault_root = VaultRoot::try_from(vault_root_str)
        .expect("Failed to create vault root");
    Config::build(&raw, vault_id, vault_root).expect("Failed to build config")
}

/// PropertyBank content (realistic from example_vault, with minor fixes).
///
/// Note: Removed `%Y` date formats (date_year, year_published) that fail
/// validation due to chrono's inability to round-trip year-only formats.
const PROPERTY_BANK_JSON: &str = r#"{
  "properties": {
    "about": { "type": "string" },
    "aliases": { "type": "string" },
    "city": { "multi": true, "type": "string" },
    "contact": { "multi": true, "type": "file", "directory": "51_contacts/" },
    "context": { "type": "string", "options": ["education", "habit_ritual", "personal", "professional", "work"] },
    "date_iso_8601": { "type": "date", "format": "%Y-%m-%d" },
    "datetime_local": { "type": "date", "format": "%Y-%m-%dT%H:%M" },
    "doi": { "type": "string" },
    "goal": { "multi": true, "type": "file", "directory": "(30_goals)/" },
    "library": { "multi": true, "type": "file", "directory": "(60_library)/" },
    "library_course": { "multi": true, "type": "file", "directory": "(60_library/68_courses)/" },
    "organization": { "multi": true, "type": "file", "directory": "(52_organizations)/" },
    "parent_task": { "multi": true, "type": "file", "directory": "(41_personal|42_education|43_professional|44_work|45_habit_ritual)/", "file_class": "task_parent" },
    "pillar": { "multi": true, "type": "file", "directory": "(20_pillars)/" },
    "project": { "multi": true, "type": "file", "directory": "(41_personal|42_education|43_professional|44_work|45_habit_ritual)/", "file_class": "task_project" },
    "task_status": { "type": "string", "options": { "1": "to_do", "2": "in_progress", "3": "done", "4": "on_hold", "5": "schedule", "6": "discarded" } },
    "title": { "type": "string" },
    "url": { "type": "string" },
    "volume": { "type": "number", "step": 1.0 }
  }
}"#;

/// Base schema templates (realistic from example_vault).
const SCHEMAS: &[(&str, &str)] = &[
    (
        "task",
        r#"{"name":"task","properties":{"date":{"$ref":"property_bank#/date_iso_8601"},"task_start":{"type":"date","format":"%Y-%m-%d"},"task_end":{"type":"date","format":"%Y-%m-%d"},"due_do":{"type":"string","options":["do","due"]},"pillar":{"$ref":"property_bank#/pillar"},"goal":{"$ref":"property_bank#/goal"},"context":{"$ref":"property_bank#/context"},"project":{"$ref":"property_bank#/project"},"parent_task":{"$ref":"property_bank#/parent_task"},"status":{"$ref":"property_bank#/task_status"},"type":{"type":"string","options":["action_item","habit","meeting","parent_task","project","ritual"]},"organization":{"$ref":"property_bank#/organization"},"contact":{"$ref":"property_bank#/contact"},"library":{"$ref":"property_bank#/library"}}}"#,
    ),
    (
        "task_project",
        r#"{"name":"task_project","extends":"task","properties":{"type":{"type":"string","options":["project"]}},"excludes":["date","project","parent_task"]}"#,
    ),
    (
        "task_meeting",
        r#"{"name":"task_meeting","extends":"task","properties":{"type":{"type":"string","options":["meeting"]}}}"#,
    ),
    (
        "lib",
        r#"{"name":"lib","properties":{"title":{"$ref":"property_bank#/title"},"url":{"$ref":"property_bank#/url"},"doi":{"$ref":"property_bank#/doi"}}}"#,
    ),
    (
        "lib_book",
        r#"{"name":"lib_book","extends":"lib","properties":{"volume":{"$ref":"property_bank#/volume"}}}"#,
    ),
];

/// Generate a realistic vault with schemas.
fn generate_vault(size: &VaultSize) -> TempDir {
    let vault = TempDir::new().expect("Failed to create temp vault");
    let schemas_dir = vault.path().join(".lithos/schemas");
    fs::create_dir_all(&schemas_dir).expect("Failed to create schemas dir");

    // Write property bank
    fs::write(schemas_dir.join("property_bank.json"), PROPERTY_BANK_JSON)
        .expect("Failed to write property bank");

    // Write schemas (repeat as needed for vault size)
    let mut written = 0;
    while written < size.schema_count {
        for (name, content) in SCHEMAS {
            if written >= size.schema_count {
                break;
            }
            // Generate unique filename and update schema name in content
            let unique_name = if written < SCHEMAS.len() {
                (*name).to_owned()
            } else {
                format!("{name}_{written}")
            };
            let filename = format!("{unique_name}.json");

            // Update schema name in JSON content to be unique
            let updated_content = if written < SCHEMAS.len() {
                content.to_string()
            } else {
                // Replace the "name" field with unique name
                content.replace(
                    &format!(r#""name":"{name}""#),
                    &format!(r#""name":"{unique_name}""#),
                )
            };

            fs::write(schemas_dir.join(filename), updated_content)
                .expect("Failed to write schema");
            written += 1;
        }
    }

    vault
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage 1: File I/O + Parsing
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark: File I/O + Parsing (TOML/JSON → RawSchema via Ingestor).
///
/// Measures serde deserialization overhead from JSON files to RawSchema
/// structs. Expected to dominate total pipeline cost and scale linearly O(n).
///
/// **Bottleneck**: serde deserialization, not filesystem I/O (tmpfs)
fn bench_file_io_and_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_io_and_parse");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                let vault = generate_vault(size);
                let config = bench_config(vault.path());

                b.iter(|| {
                    let ingestor = Ingestor::new(
                        FsReader::new(vault.path().to_path_buf()),
                        &config,
                    );
                    let raw_schemas = ingestor
                        .scan_raw_schemas()
                        .expect("Failed to scan schemas");
                    black_box(raw_schemas.len())
                });
            },
        );
    }

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage 2: PropertyBank Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark: PropertyBank validation (RawPropertyBank → PropertyBank).
///
/// Measures PropertySpec construction and validation logic. Expected to be O(1)
/// regardless of schema count (fixed 19 properties). Recent optimizations
/// (HIGH-001 regex caching, E-02 epsilon) should be reflected here.
///
/// **Bottleneck**: PropertySpec validation, not HashMap construction
fn bench_property_bank_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("property_bank_validation");
    group.throughput(Throughput::Elements(1)); // One PropertyBank per iteration

    let vault = generate_vault(&VAULT_SIZES[0]); // Size doesn't matter
    let config = bench_config(vault.path());
    let ingestor =
        Ingestor::new(FsReader::new(vault.path().to_path_buf()), &config);
    let raw_bank =
        ingestor.load_raw_property_bank().expect("Failed to load raw bank");

    group.bench_function("validate", |b| {
        b.iter(|| {
            let bank = PropertyBank::from_raw(raw_bank.clone(), None)
                .expect("Failed to validate");
            black_box(bank.all().count())
        });
    });

    group.finish();
}

/// Benchmark: PropertyBank lookup performance (get by name).
///
/// Measures HashMap lookup cost for $ref resolution during dereferencing.
/// Expected to be O(1) with good hash distribution (~5-10 ns per lookup).
///
/// **Bottleneck**: Hash function quality, not table size
fn bench_property_bank_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("property_bank_lookup");
    group.throughput(Throughput::Elements(1));

    let vault = generate_vault(&VAULT_SIZES[0]);
    let config = bench_config(vault.path());
    let ingestor =
        Ingestor::new(FsReader::new(vault.path().to_path_buf()), &config);
    let raw_bank = ingestor.load_raw_property_bank().expect("Failed to load");
    let bank =
        PropertyBank::from_raw(raw_bank, None).expect("Failed to validate");

    group.bench_function("get_by_name", |b| {
        b.iter(|| {
            // Lookup a frequently-used property ($ref'd in task schema)
            let prop = bank.get("pillar");
            black_box(prop.is_ok())
        });
    });

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage 3: Dereferencing
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark: Dereferencing ($ref resolution against PropertyBank).
///
/// Measures $ref pointer resolution overhead. Expected to scale linearly with
/// total number of $ref pointers across all schemas: O(n *
/// avg_refs_per_schema).
///
/// **Bottleneck**: PropertySpec cloning, not HashMap lookups
fn bench_dereferencing(c: &mut Criterion) {
    let mut group = c.benchmark_group("dereferencing");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                // Setup: load PropertyBank and raw schemas
                let vault = generate_vault(size);
                let config = bench_config(vault.path());
                let ingestor = Ingestor::new(
                    FsReader::new(vault.path().to_path_buf()),
                    &config,
                );
                let raw_bank = ingestor
                    .load_raw_property_bank()
                    .expect("Failed to load bank");
                let bank = PropertyBank::from_raw(raw_bank, None)
                    .expect("Failed to validate");
                let raw_schemas = ingestor
                    .scan_raw_schemas()
                    .expect("Failed to scan schemas");

                // Convert to (SchemaId, RawSchema) pairs
                let schemas_with_ids: Vec<(SchemaId, RawSchema)> = raw_schemas
                    .into_iter()
                    .map(|(raw, _, _)| (SchemaId::new(), raw))
                    .collect();

                b.iter(|| {
                    let dereferencer = Dereferencer::new(&bank);
                    let derefed = dereferencer
                        .deref(schemas_with_ids.clone())
                        .expect("Failed to dereference");
                    black_box(derefed.len())
                });
            },
        );
    }

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage 4: DAG Construction
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark: DAG construction (topological sort + cycle detection).
///
/// Measures Extender overhead (Kahn's algorithm for topological sort). Expected
/// to scale O(n + e) where e = number of inheritance edges.
///
/// **Bottleneck**: HashMap operations, not algorithm itself
fn bench_dag_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_construction");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                // Setup: dereference schemas
                let vault = generate_vault(size);
                let config = bench_config(vault.path());
                let ingestor = Ingestor::new(
                    FsReader::new(vault.path().to_path_buf()),
                    &config,
                );
                let raw_bank = ingestor
                    .load_raw_property_bank()
                    .expect("Failed to load bank");
                let bank = PropertyBank::from_raw(raw_bank, None)
                    .expect("Failed to validate");
                let raw_schemas = ingestor
                    .scan_raw_schemas()
                    .expect("Failed to scan schemas");

                let schemas_with_ids: Vec<(SchemaId, RawSchema)> = raw_schemas
                    .into_iter()
                    .map(|(raw, _, _)| (SchemaId::new(), raw))
                    .collect();

                let dereferencer = Dereferencer::new(&bank);
                let derefed = dereferencer
                    .deref(schemas_with_ids)
                    .expect("Failed to dereference");

                b.iter(|| {
                    let tree =
                        Extender::build(derefed.clone(), &HashMap::new())
                            .expect("Failed to build DAG");
                    black_box(tree.nodes().len())
                });
            },
        );
    }

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
//  Stage 5: Property Merging
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark: Property merging (inheritance resolution via Resolver).
///
/// Measures two-pointer sorted merge overhead for property inheritance.
/// Expected to scale linearly with (inheritance depth * property count): O(n *
/// d * p).
///
/// **Bottleneck**: Arc cloning + merge logic, not tree traversal
fn bench_property_merging(c: &mut Criterion) {
    let mut group = c.benchmark_group("property_merging");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                // Setup: build DAG
                let vault = generate_vault(size);
                let config = bench_config(vault.path());
                let ingestor = Ingestor::new(
                    FsReader::new(vault.path().to_path_buf()),
                    &config,
                );
                let raw_bank = ingestor
                    .load_raw_property_bank()
                    .expect("Failed to load bank");
                let bank = PropertyBank::from_raw(raw_bank, None)
                    .expect("Failed to validate");
                let raw_schemas = ingestor
                    .scan_raw_schemas()
                    .expect("Failed to scan schemas");

                let schemas_with_ids: Vec<(SchemaId, RawSchema)> = raw_schemas
                    .into_iter()
                    .map(|(raw, _, _)| (SchemaId::new(), raw))
                    .collect();

                let dereferencer = Dereferencer::new(&bank);
                let derefed = dereferencer
                    .deref(schemas_with_ids)
                    .expect("Failed to dereference");
                let tree = Extender::build(derefed, &HashMap::new())
                    .expect("Failed to build DAG");

                b.iter(|| {
                    let resolved = Resolver::resolve(&tree, &HashMap::new())
                        .expect("Failed to resolve");
                    black_box(resolved.len())
                });
            },
        );
    }

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
//  Database Operations: Batch vs Serial
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: Create and populate a test database with schemas.
fn setup_db_with_schemas(
    schema_count: usize,
) -> (TempDir, Database, Vec<SchemaId>) {
    let db_dir = TempDir::new().expect("Failed to create temp DB dir");
    let db_path = db_dir.path().join("bench.db");
    let db = Database::open(&db_path).expect("Failed to open DB");

    let cmd = Command::new(CommandAdapter::new(&db));

    // Create and save PropertyBank
    let bank = PropertyBank::new();
    cmd.save_property_bank(&bank).expect("Failed to save bank");

    // Generate schemas
    let mut schema_ids = Vec::with_capacity(schema_count);
    let mut schemas = Vec::with_capacity(schema_count);

    for i in 0..schema_count {
        let id = SchemaId::new();
        schema_ids.push(id);

        // Create a simple schema (name is unique)
        let name_str = format!("schema_{i}");
        let name = lithos_core::schema::aggregate::SchemaName::new(&name_str)
            .expect("Failed to create schema name");
        let schema = Schema::new(id, name, None, Vec::new())
            .expect("Failed to create schema");
        schemas.push(schema);
    }

    // Save all schemas (metadata is generated automatically)
    cmd.save_batch(&schemas).expect("Failed to save schemas");

    (db_dir, db, schema_ids)
}

/// Benchmark: Serial staleness checks (O(N) transactions).
///
/// Measures the cost of calling `is_schema_stale` N times individually.
/// Each call creates its own database transaction, representing the
/// original implementation before batch operations.
///
/// **Bottleneck**: Transaction creation overhead (N transactions)
fn bench_staleness_serial(c: &mut Criterion) {
    let mut group = c.benchmark_group("staleness_checks/serial");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                let (_db_dir, db, schema_ids) =
                    setup_db_with_schemas(size.schema_count);
                let qry = Query::new(QueryAdapter::new(&db));

                b.iter(|| {
                    let mut stale_count = 0;
                    for &id in &schema_ids {
                        let is_stale = qry
                            .is_schema_stale(
                                id,
                                None,
                                None,
                                BankVersion::initial(),
                            )
                            .expect("Staleness check failed");
                        if is_stale {
                            stale_count += 1;
                        }
                    }
                    black_box(stale_count)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Batch staleness checks (O(1) transaction).
///
/// Measures the cost of calling `batch_is_stale` once for all schemas.
/// Single database transaction for all staleness checks, representing
/// the optimized implementation.
///
/// **Bottleneck**: Schema count (linear within transaction)
fn bench_staleness_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("staleness_checks/batch");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                let (_db_dir, db, schema_ids) =
                    setup_db_with_schemas(size.schema_count);
                let qry = Query::new(QueryAdapter::new(&db));

                // Build staleness checks
                let checks: Vec<(
                    SchemaId,
                    Option<Timestamp>,
                    Option<Timestamp>,
                )> = schema_ids.iter().map(|&id| (id, None, None)).collect();

                b.iter(|| {
                    let staleness = qry
                        .batch_is_stale(&checks, BankVersion::initial())
                        .expect("Batch staleness check failed");
                    let stale_count =
                        staleness.values().filter(|&&v| v).count();
                    black_box(stale_count)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Serial schema lookups (O(N) transactions).
///
/// Measures the cost of calling `find_by_id` N times individually.
/// Each call creates its own database transaction, representing the
/// original implementation before batch operations.
///
/// **Bottleneck**: Transaction creation overhead (N transactions)
fn bench_schema_lookup_serial(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_lookup/serial");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                let (_db_dir, db, schema_ids) =
                    setup_db_with_schemas(size.schema_count);
                let qry = Query::new(QueryAdapter::new(&db));

                b.iter(|| {
                    let mut found_count = 0;
                    for &id in &schema_ids {
                        if qry.find_by_id(id).expect("Lookup failed").is_some()
                        {
                            found_count += 1;
                        }
                    }
                    black_box(found_count)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Batch schema lookups (O(1) transaction).
///
/// Measures the cost of calling `batch_find_by_ids` once for all schemas.
/// Single database transaction for all lookups, representing the
/// optimized implementation.
///
/// **Bottleneck**: Schema count (linear within transaction)
fn bench_schema_lookup_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("schema_lookup/batch");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                let (_db_dir, db, schema_ids) =
                    setup_db_with_schemas(size.schema_count);
                let qry = Query::new(QueryAdapter::new(&db));

                b.iter(|| {
                    let schemas = qry
                        .batch_find_by_ids(&schema_ids)
                        .expect("Batch lookup failed");
                    black_box(schemas.len())
                });
            },
        );
    }

    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
//  Full Pipeline
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark: Full end-to-end pipeline (file → resolved Schema).
///
/// Measures total ingestion cost including all stages. This is the user-facing
/// metric that determines vault startup time. Expected to scale linearly
/// overall (file I/O dominates at scale).
///
/// **Bottleneck distribution**: File I/O ~50%, Deref ~25%, DAG ~10%, Merge
/// ~10%, Validation ~5%
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    for size in VAULT_SIZES {
        group.throughput(Throughput::Elements(size.schema_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size.name),
            size,
            |b, size| {
                let vault = generate_vault(size);
                let config = bench_config(vault.path());

                b.iter(|| {
                    // Stage 1: File I/O + Parsing
                    let ingestor = Ingestor::new(
                        FsReader::new(vault.path().to_path_buf()),
                        &config,
                    );
                    let raw_bank = ingestor
                        .load_raw_property_bank()
                        .expect("Failed to load bank");
                    let raw_schemas = ingestor
                        .scan_raw_schemas()
                        .expect("Failed to scan schemas");

                    // Stage 2: PropertyBank Validation
                    let bank = PropertyBank::from_raw(raw_bank, None)
                        .expect("Failed to validate bank");

                    // Convert to (SchemaId, RawSchema) pairs
                    let schemas_with_ids: Vec<(SchemaId, RawSchema)> =
                        raw_schemas
                            .into_iter()
                            .map(|(raw, _, _)| (SchemaId::new(), raw))
                            .collect();

                    // Stage 3: Dereferencing
                    let dereferencer = Dereferencer::new(&bank);
                    let derefed = dereferencer
                        .deref(schemas_with_ids)
                        .expect("Failed to dereference");

                    // Stage 4: DAG Construction
                    let tree = Extender::build(derefed, &HashMap::new())
                        .expect("Failed to build DAG");

                    // Stage 5: Property Merging
                    let resolved = Resolver::resolve(&tree, &HashMap::new())
                        .expect("Failed to resolve");

                    black_box(resolved.len())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_file_io_and_parse,
    bench_property_bank_validation,
    bench_property_bank_lookup,
    bench_dereferencing,
    bench_dag_construction,
    bench_property_merging,
    bench_staleness_serial,
    bench_staleness_batch,
    bench_schema_lookup_serial,
    bench_schema_lookup_batch,
    bench_full_pipeline,
);
criterion_main!(benches);
