#![allow(
    missing_docs,
    reason = "Benchmarks don't require public documentation"
)]
#![expect(
    clippy::expect_used,
    reason = "Benchmarks use expect for simplicity in setup and execution"
)]
#![expect(
    clippy::as_conversions,
    reason = "Benchmarks use as-conversions for throughput metrics"
)]

use std::{fs, path::PathBuf};

use criterion::{
    Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use lithos_core::{
    application::services::SchemaIngestionService,
    db::Database,
    fs::source::{FileSource as _, FsFileSource, InMemoryFileSource},
    schema::{RedbSchemaCommand, RedbSchemaQuery},
};
use tempfile::tempdir;

/// Generates a test vault with the specified number of schema files.
fn generate_test_vault(file_count: usize) -> (tempfile::TempDir, Vec<PathBuf>) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let mut files = Vec::new();

    for i in 0..file_count {
        let file_path = temp_dir.path().join(format!("schema_{i:04}.json"));
        let content = format!(
            r#"{{"id": "01933333-3333-7333-8333-{i:012x}", "name": "schema{i}", "properties": []}}"#
        );
        fs::write(&file_path, content).expect("Failed to write test file");
        files.push(file_path);
    }

    (temp_dir, files)
}

/// Baseline benchmark for sequential file reading.
fn bench_file_read_baseline(c: &mut Criterion) {
    let file_count = 100;
    let (vault, _files) = generate_test_vault(file_count);
    let source = FsFileSource::new(vault.path());

    let mut group = c.benchmark_group("file_io");
    group.throughput(Throughput::Elements(file_count as u64));

    group.bench_function("sequential_read_100_files", |b| {
        b.iter(|| {
            for i in 0..file_count {
                let path = PathBuf::from(format!("schema_{i:04}.json"));
                let content =
                    source.read_to_string(&path).expect("Read failed");
                black_box(content);
            }
        });
    });
    group.finish();
}

/// Benchmark for full schema ingestion pipeline.
fn bench_schema_ingestion(c: &mut Criterion) {
    let file_count = 100;
    let (vault, _files) = generate_test_vault(file_count);
    let source = FsFileSource::new(vault.path());

    // Set up database
    let db_dir = tempdir().expect("Failed to create db dir");
    let db_path = db_dir.path().join("bench.redb");
    let db = Database::open(&db_path).expect("Failed to open database");
    let query = RedbSchemaQuery::new_redb(&db);
    let command = RedbSchemaCommand::new_redb(&db);
    let service = SchemaIngestionService::new(&query, &command);

    let mut group = c.benchmark_group("ingestion");
    group.throughput(Throughput::Elements(file_count as u64));

    group.bench_function("schema_ingestion_100_files_fs", |b| {
        b.iter(|| {
            // We need to clear the database or use new IDs to avoid
            // constraints if any, but RedbSchemaCommand currently
            // overwrites.
            service.ingest_directory(&source, "*.json").expect("Ingest failed");
        });
    });

    let mut mem_source = InMemoryFileSource::new();
    for i in 0..file_count {
        let path = PathBuf::from(format!("schema_{i:04}.json"));
        let content = format!(
            r#"{{"id": "01933333-3333-7333-8333-{i:012x}", "name": "schema{i}", "properties": []}}"#
        );
        mem_source.insert(&path, content);
    }

    group.bench_function("schema_ingestion_100_files_mem", |b| {
        b.iter(|| {
            service
                .ingest_directory(&mem_source, "*.json")
                .expect("Ingest failed");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_file_read_baseline, bench_schema_ingestion);
criterion_main!(benches);
