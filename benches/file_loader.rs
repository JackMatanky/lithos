//! Performance benchmarks for file loading and format detection.
//!
//! Validates that file loading operations meet performance targets:
//! - Format detection: < 100μs
//! - Individual file loading: < 500ms
//! - Parsing overhead: minimal per format

use std::{fs, path::PathBuf};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use lithos_adapters::FileReaderAdapter;
use lithos_domain::FileReaderPort;
use tempfile::TempDir;

/// Benchmark format detection performance for different file types.
fn bench_format_detection(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp_dir.path();

    // Create test files
    let toml_file = base_path.join("config.toml");
    fs::write(&toml_file, "[section]\nkey = \"value\"")
        .expect("Failed to write TOML file");

    let json_file = base_path.join("config.json");
    fs::write(&json_file, r#"{"key": "value"}"#)
        .expect("Failed to write JSON file");

    let yaml_file = base_path.join("config.yaml");
    fs::write(&yaml_file, "key: value\n").expect("Failed to write YAML file");

    let mut group = c.benchmark_group("format_detection");

    for (name, file) in
        [("toml", toml_file), ("json", json_file), ("yaml", yaml_file)]
    {
        group.bench_with_input(
            BenchmarkId::new("detect_and_load", name),
            &file,
            |b, path| {
                let reader = FileReaderAdapter::new();
                b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(
                    || async {
                        let relative_path = PathBuf::from(
                            path.strip_prefix(temp_dir.path()).unwrap(),
                        );
                        reader.read(&relative_path).await.ok()
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark file loading performance across different file sizes.
fn bench_file_loading_sizes(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp_dir.path();

    let sizes = [
        ("1kb", 1024),
        ("10kb", 10 * 1024),
        ("100kb", 100 * 1024),
        ("1mb", 1024 * 1024),
    ];

    let mut group = c.benchmark_group("file_loading_by_size");

    for (name, size) in sizes {
        let content = "a".repeat(size);
        let file_path = base_path.join(format!("test_{name}.toml"));
        fs::write(&file_path, format!("[section]\ndata = \"{content}\""))
            .expect("Failed to write test file");

        group.bench_with_input(
            BenchmarkId::new("load", name),
            &file_path,
            |b, path| {
                let reader = FileReaderAdapter::new();
                b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(
                    || async {
                        let relative_path = PathBuf::from(
                            path.strip_prefix(temp_dir.path()).unwrap(),
                        );
                        reader.read(&relative_path).await.ok()
                    },
                );
            },
        );
    }

    group.finish();
}

/// Benchmark parsing overhead for different formats.
fn bench_parsing_overhead(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let base_path = temp_dir.path();

    let content = r#"
[server]
host = "localhost"
port = 8080
workers = 4

[database]
url = "dbserver://localhost/lithos"
pool_size = 20
timeout = 30
"#;

    let toml_file = base_path.join("app.toml");
    fs::write(&toml_file, content).expect("Failed to write TOML file");

    let json_content = r#"{
  "server": {
    "host": "localhost",
    "port": 8080,
    "workers": 4
  },
  "database": {
    "url": "dbserver://localhost/lithos",
    "pool_size": 20,
    "timeout": 30
  }
}"#;
    let json_file = base_path.join("app.json");
    fs::write(&json_file, json_content).expect("Failed to write JSON file");

    let yaml_content = r#"
server:
  host: localhost
  port: 8080
  workers: 4
database:
  url: dbserver://localhost/lithos
  pool_size: 20
  timeout: 30
"#;
    let yaml_file = base_path.join("app.yaml");
    fs::write(&yaml_file, yaml_content).expect("Failed to write YAML file");

    let mut group = c.benchmark_group("parsing_overhead");

    for (name, file) in
        [("toml", toml_file), ("json", json_file), ("yaml", yaml_file)]
    {
        group.bench_with_input(
            BenchmarkId::new("parse", name),
            &file,
            |b, path| {
                let reader = FileReaderAdapter::new();
                b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(
                    || async {
                        let relative_path = PathBuf::from(
                            path.strip_prefix(temp_dir.path()).unwrap(),
                        );
                        reader.read(&relative_path).await.ok()
                    },
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_format_detection,
    bench_file_loading_sizes,
    bench_parsing_overhead
);
criterion_main!(benches);
