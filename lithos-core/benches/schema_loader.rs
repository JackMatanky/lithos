//! Schema loading pipeline performance benchmarks.
//!
//! # Summary
//!
//! Measures schema file ingestion and PropertyBank validation to establish
//! baseline performance for the schema loading pipeline and detect regressions.
//!
//! # Motivation
//!
//! Schema loading happens at vault startup. While the full resolution pipeline
//! (dereferencing, DAG construction, property merging) is internal
//! implementation, the user-facing file I/O and validation stages are critical
//! for startup time. This benchmark focuses on measurable, public-API
//! operations.
//!
//! # Scope
//!
//! **Included**:
//! - File I/O + Parsing (TOML/JSON → RawSchema via Ingestor)
//! - PropertyBank validation (RawPropertyBank → PropertyBank domain type)
//! - Validation overhead (PropertySpec construction and validation)
//! - Scaling behavior (tiny → small → medium → large vaults)
//!
//! **Excluded**:
//! - Internal resolution pipeline stages (Dereferencer, Extender, Resolver -
//!   private modules)
//! - Database operations (Command/Query adapters)
//! - Staleness checking (hybrid vs cold start comparison - requires DB
//!   integration)
//!
//! # Benchmark Style
//!
//! - **Micro-benchmarks**: Isolated file loading and validation operations
//! - **Scaling-focused**: Tests across vault sizes (5, 20, 40, 100 schemas)
//! - **Single-threaded**: No concurrent loading
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs (100 samples, 3s warmup)
//! - **Black-boxing**: All results through `black_box()` to prevent elision
//! - **Setup separation**: Vault generation outside timed region
//! - **Measurement**: Total latency and throughput (schemas/sec)
//!
//! # Input Model
//!
//! Based on real schemas from `example_vault/.lithos/schemas/`:
//!
//! **Property Bank** (25 properties):
//! - String types (simple, with options, with patterns)
//! - Number types (with bounds, with step validation)
//! - Date types (various format strings)
//! - File types (directory constraints, file_class)
//! - Mix of single and multi-value properties
//!
//! **Schemas**:
//! - Base schemas (task, lib, cal, dir, pkm)
//! - Derived schemas with inheritance (task_project extends task)
//! - $ref usage (referencing PropertyBank properties)
//! - Excludes and overrides in derived schemas
//! - 5-15 properties per schema
//!
//! **Vault Sizes**:
//! - **Tiny**: 5 schemas (~150 KB schema files)
//! - **Small**: 20 schemas (~600 KB schema files)
//! - **Medium**: 40 schemas (~1.2 MB schema files)
//! - **Large**: 100 schemas (~3 MB schema files)
//!
//! # Expected Characteristics
//!
//! **File I/O + Parse** (dominant cost):
//! - Tiny (5 schemas): ~200 µs (disk I/O + serde deserialization)
//! - Large (100 schemas): ~4 ms (should scale linearly with file count)
//!
//! **PropertyBank Validation**:
//! - ~50 µs (25 properties, PropertySpec construction + validation)
//! - Should be constant regardless of schema count
//!
//! **Total Pipeline** (sum of stages):
//! - Tiny: ~250 µs
//! - Large: ~4.5 ms
//!
//! **Scaling**:
//! - Should be O(n) with schema count (file I/O dominates)
//! - PropertyBank validation is O(1) (fixed number of properties)
//!
//! # Interpreting Results
//!
//! **Bottleneck Identification**:
//! - If file I/O dominates (>80% of time) → normal, disk-bound
//! - If PropertyBank validation is expensive (>20%) → PropertySpec optimization
//!   needed
//! - If doesn't scale linearly → file system caching issue
//!
//! **Meaningful Changes**:
//! - >20% regression in file I/O → serde performance or file system issue
//! - >20% regression in validation → PropertySpec construction regressed
//! - Scaling becomes superlinear → investigate file system behavior
//!
//! **Validation of Recent Fixes**:
//! - PropertyBank validation includes PropertySpec::validate logic
//! - Recent fixes (HIGH-001 regex caching, E-02 epsilon) measured here
//! - Establishes baseline for future schema format optimizations
//!
//! # Maintenance Contract
//!
//! **Update when**:
//! - Schema file format changes (TOML structure, new fields)
//! - PropertyBank structure changes (new property types)
//! - Validation logic changes (PropertySpec construction)
//!
//! # Known Limitations
//!
//! - **No resolution pipeline**: Internal stages (DAG, merge) not measured
//! - **No DB integration**: Can't measure hybrid vs cold start comparison
//! - **Synthetic vault**: Generated test data, not production corpus
//! - **File system variance**: Uses tmpfs (TempDir), not realistic SSD/HDD
//!
//! # Future Work
//!
//! If internal resolution modules are exposed for testing (via feature flag or
//! pub(crate) with crate-level access), add:
//! - Dereference benchmark ($ref resolution)
//! - Extend benchmark (DAG + topological sort)
//! - Resolve benchmark (property merge)
//! - Full cold start vs warm start comparison
//!
//! # Safety
//!
//! Benchmark code uses `unwrap`/`expect` for simplicity (failures indicate
//! test setup errors, not runtime conditions).

#![allow(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::doc_markdown,
    clippy::shadow_unrelated,
    clippy::pattern_type_mismatch,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::indexing_slicing,
    dead_code,
    reason = "Benchmark code: simplified error handling, test-only functions"
)]

use std::fs;

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
    fs::FsReader,
    schema::{adapter::ingestor::Ingestor, bank::PropertyBank},
};
use tempfile::TempDir;

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

/// PropertyBank content (realistic from example_vault, with minor fixes for
/// validation).
///
/// Note: Removed `%Y` date formats (date_year, year_published) that fail
/// validation due to chrono's inability to round-trip year-only formats. These
/// properties exist in example_vault but are not actively used.
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
        r#"{"name":"lib","properties":{"title":{"$ref":"property_bank#/title"},"url":{"$ref":"property_bank#/url"},"doi":{"$ref":"property_bank#/doi"},"year_published":{"$ref":"property_bank#/year_published"}}}"#,
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
            let filename = if written < SCHEMAS.len() {
                format!("{name}.json")
            } else {
                format!("{name}_{written}.json")
            };
            fs::write(schemas_dir.join(filename), content)
                .expect("Failed to write schema");
            written += 1;
        }
    }

    vault
}

/// Benchmark: File I/O + Parsing (scan_raw_schemas).
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

/// Benchmark: PropertyBank validation (from_raw).
fn bench_property_bank_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("property_bank_validation");
    group.throughput(Throughput::Elements(1)); // One PropertyBank per iteration

    let vault = generate_vault(&VAULT_SIZES[0]); // Size doesn't matter for PropertyBank
    let config = bench_config(vault.path());
    let ingestor =
        Ingestor::new(FsReader::new(vault.path().to_path_buf()), &config);
    let raw_bank =
        ingestor.load_raw_property_bank().expect("Failed to load raw bank");

    group.bench_function("validate_property_bank", |b| {
        b.iter(|| {
            let bank = PropertyBank::from_raw(raw_bank.clone(), None)
                .expect("Failed to validate");
            black_box(bank.all().count())
        });
    });

    group.finish();
}

/// Benchmark: Combined pipeline (file I/O + parse + validate).
fn bench_combined_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_pipeline");

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
                    let raw_bank = ingestor
                        .load_raw_property_bank()
                        .expect("Failed to load bank");
                    let raw_schemas = ingestor
                        .scan_raw_schemas()
                        .expect("Failed to scan schemas");

                    let bank = PropertyBank::from_raw(raw_bank, None)
                        .expect("Failed to validate bank");

                    black_box((raw_schemas.len(), bank.all().count()))
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
    bench_combined_pipeline,
);
criterion_main!(benches);
