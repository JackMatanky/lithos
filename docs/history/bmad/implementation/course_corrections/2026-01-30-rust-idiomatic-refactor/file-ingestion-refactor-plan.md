---
title: "File Ingestion Refactor - Implementation Plan"
description: "Step-by-step implementation plan for refactoring file ingestion to use Service Layer pattern"
author: "Lithos Development Team"
date: "2026-02-16"
status: "active"
related_docs:
  - "../adr/010-file-ingestion.md"
  - "../design/016-file-ingestion-architecture.md"
  - "../design/017-file-ingestion-performance.md"
---

# File Ingestion Refactor - Implementation Plan

## Overview

This plan guides the implementation of the Service Layer pattern for file ingestion, separating file I/O from database operations and fixing the architectural violation where CQRS ports mix concerns.

**Total Estimated Effort**: 8-10 days (split across 4 phases)

**Reference Documents**:
- [ADR 010: File Ingestion Architecture](../adr/010-file-ingestion.md) - Decision rationale and alternatives
- [Design Doc 016: Implementation Design](../design/016-file-ingestion-architecture.md) - Code structure and patterns
- [Design Doc 017: Performance Analysis](../design/017-file-ingestion-performance.md) - Performance targets and optimization strategies

## Success Criteria

Before marking this refactor complete:

- [ ] All tests pass (`mise run test`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] ADR 010 validation checklist 100% complete
- [ ] Zero file I/O methods in any CQRS port trait
- [ ] All ingestion services have >80% test coverage
- [ ] Benchmarks meet performance targets (1K files < 500ms)
- [ ] `project-context.md` updated with new architectural rules
- [ ] Architecture tests prevent port violations in CI

---

## Phase 1: Infrastructure Foundation

**Goal**: Add `FileSource` trait and file parsers without breaking existing code.

**Effort**: 2-3 days

**Status**: ⏳ Not Started

### 1.1: Create FileSource Trait

**File**: `lithos-core/src/fs/source.rs` (new file)

**Tasks**:

1. **Define `FileSource` trait**:
   ```rust
   pub trait FileSource: Send + Sync {
       type Error: std::error::Error + Send + Sync + 'static;
       fn read_to_string(&self, path: &Path) -> Result<String, Self::Error>;
       fn exists(&self, path: &Path) -> bool;
       fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error>;
   }
   ```

2. **Implement `FsFileSource`** (filesystem access via `std::fs`):
   - Store `root: PathBuf` for scoped access
   - `read_to_string()`: Join root + path, call `std::fs::read_to_string`
   - `exists()`: Join root + path, call `Path::exists`
   - `list_files()`: Use `walkdir` or `glob` crate (add dependency)

3. **Implement `InMemoryFileSource`** (for testing):
   - Store `HashMap<PathBuf, String>` of fake files
   - `read_to_string()`: Lookup in HashMap
   - `exists()`: Check HashMap contains key
   - `list_files()`: Return all keys (ignore pattern for now)

**Testing**:
```rust
#[test]
fn fs_file_source_reads_real_files() { /* ... */ }

#[test]
fn in_memory_file_source_reads_from_hashmap() { /* ... */ }

#[test]
fn fs_file_source_returns_error_for_missing_file() { /* ... */ }
```

**Reference**: See Design Doc 016, Layer 1 (lines 85-174)

**Dependencies**:
- Add `walkdir = "2.5"` or `glob = "0.3"` to `Cargo.toml`

---

### 1.2: Create File Parsers

**File**: `lithos-core/src/fs/parsers.rs` (new file)

**Tasks**:

1. **Create `parse_schema_file()` function**:
   - Accept `&impl FileSource` and `&Path`
   - Read file content via source
   - Detect format by extension (`.json`, `.toml`, `.yaml`)
   - Deserialize into `schema::raw::RawSchema`
   - Return `Result<RawSchema, ParseError>`

2. **Create `parse_template_file()` function**:
   - Accept `&impl FileSource` and `&Path`
   - Read file content via source
   - Construct `template::raw::RawTemplate` with content + metadata
   - Return `Result<RawTemplate, ParseError>`

3. **Create `parse_note_file()` function**:
   - Accept `&impl FileSource` and `&Path`
   - Read file content via source
   - Delegate to existing `note::parser::parse()`
   - Return `Result<ParsedNote, ParseError>`

**Testing**:
```rust
#[test]
fn parse_schema_file_deserializes_json() { /* ... */ }

#[test]
fn parse_schema_file_detects_format_by_extension() { /* ... */ }

#[test]
fn parse_schema_file_returns_error_for_invalid_json() { /* ... */ }

#[test]
fn parse_template_file_reads_plain_text() { /* ... */ }

#[test]
fn parse_note_file_delegates_to_existing_parser() { /* ... */ }
```

**Reference**: See Design Doc 016, Layer 2 (lines 185-257)

---

### 1.3: Add ParseError Type

**File**: `lithos-core/src/fs/error.rs` (modify existing)

**Tasks**:

1. **Add `ParseError` enum**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ParseError {
       #[error("I/O error: {0}")]
       Io(#[from] std::io::Error),

       #[error("JSON parse error: {0}")]
       Json(#[from] serde_json::Error),

       #[error("TOML parse error: {0}")]
       Toml(#[from] toml::de::Error),

       #[error("YAML parse error: {0}")]
       Yaml(#[from] serde_yml::Error),

       #[error("Unsupported format for file: {0}")]
       UnsupportedFormat(PathBuf),
   }
   ```

2. **Export from `fs/mod.rs`**:
   ```rust
   pub use error::ParseError;
   ```

**Reference**: See Design Doc 016, Layer 2 (lines 189-193)

---

### 1.4: Update fs/mod.rs

**File**: `lithos-core/src/fs/mod.rs` (modify existing)

**Tasks**:

1. **Add module declarations**:
   ```rust
   pub mod source;
   ```

2. **Add type aliases**:
   ```rust
   pub type FileSource = source::FileSource;
   pub type FsFileSource = source::FsFileSource;
   #[cfg(test)]
   pub type InMemoryFileSource = source::InMemoryFileSource;
   ```

3. **Re-export parsers**:
   ```rust
   pub use parsers::{parse_schema_file, parse_template_file, parse_note_file};
   ```

---

### 1.5: Phase 1 Verification

**Checklist**:

- [ ] `FileSource` trait compiles and is `Send + Sync`
- [ ] `FsFileSource` reads real files from disk
- [ ] `InMemoryFileSource` works for testing
- [ ] All parsers compile and delegate correctly
- [ ] `ParseError` wraps all relevant error types
- [ ] All unit tests pass (`mise run test:unit:fs`)
- [ ] No clippy warnings in new code
- [ ] Documentation comments for all public APIs

**Verification Command**:
```bash
mise run test:unit:fs
mise run lint
```

**Output**: Phase 1 is non-breaking. Existing code continues to work. New infrastructure is available but unused.

---

## Phase 2: Ingestion Services

**Goal**: Create application services that orchestrate File → Raw → Domain → Database pipeline.

**Effort**: 2-3 days

**Status**: ⏳ Not Started

**Prerequisites**: Phase 1 complete

### 2.1: Create Application Services Directory

**Tasks**:

1. **Create directory structure**:
   ```bash
   mkdir -p lithos-core/src/application/services
   ```

2. **Create module files**:
   - `lithos-core/src/application/mod.rs`
   - `lithos-core/src/application/services/mod.rs`
   - `lithos-core/src/application/error.rs`

3. **Add to lib.rs**:
   ```rust
   pub mod application;
   ```

---

### 2.2: Create IngestionError Type

**File**: `lithos-core/src/application/error.rs` (new file)

**Tasks**:

1. **Define `IngestionError` enum**:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum IngestionError {
       #[error("Parse error: {0}")]
       Parse(#[from] crate::fs::ParseError),

       #[error("Validation error: {0}")]
       Validation(String),

       #[error("Schema command error: {0}")]
       SchemaCommand(#[from] crate::schema::error::CommandError),

       #[error("Template command error: {0}")]
       TemplateCommand(String),  // TODO: Add when template errors exist

       #[error("Note command error: {0}")]
       NoteCommand(String),  // TODO: Add when note errors exist
   }
   ```

2. **Export from `application/mod.rs`**:
   ```rust
   pub use error::IngestionError;
   ```

---

### 2.3: Implement SchemaIngestionService

**File**: `lithos-core/src/application/services/schema_ingestion.rs` (new file)

**Tasks**:

1. **Define service struct**:
   ```rust
   pub struct SchemaIngestionService<'a, Q, C> {
       query: &'a schema::query::Query<Q>,
       command: &'a schema::command::Command<C>,
   }
   ```

2. **Implement `new()` constructor**:
   ```rust
   impl<'a, Q, C> SchemaIngestionService<'a, Q, C>
   where
       Q: schema::ports::Query,
       C: schema::ports::Command,
   {
       pub fn new(
           query: &'a schema::query::Query<Q>,
           command: &'a schema::command::Command<C>,
       ) -> Self {
           Self { query, command }
       }
   }
   ```

3. **Implement `ingest_file()` method**:
   ```rust
   #[instrument(skip(self, source), fields(path = %path.display()))]
   pub fn ingest_file(
       &self,
       source: &impl FileSource,
       path: &Path,
   ) -> Result<SchemaId, IngestionError> {
       // Step 1: Parse file
       let raw = parsers::parse_schema_file(source, path)?;

       // Step 2: Validate (Raw → Domain)
       let schema = Schema::try_from(raw)
           .map_err(|e| IngestionError::Validation(e.to_string()))?;

       // Step 3: Persist
       let id = schema.id();
       self.command.save_with_metadata(&schema, &Default::default())?;

       Ok(id)
   }
   ```

4. **Implement `ingest_directory()` method**:
   ```rust
   #[instrument(skip(self, source))]
   pub fn ingest_directory(
       &self,
       source: &impl FileSource,
       pattern: &str,
   ) -> Result<Vec<SchemaId>, IngestionError> {
       let paths = source.list_files(pattern)?;
       let mut ids = Vec::new();

       for path in paths {
           match self.ingest_file(source, &path) {
               Ok(id) => {
                   tracing::info!(schema_id = %id, path = %path.display(), "Schema ingested");
                   ids.push(id);
               }
               Err(e) => {
                   tracing::warn!(error = %e, path = %path.display(), "Failed to ingest schema");
                   // Continue with partial failure tolerance
               }
           }
       }

       Ok(ids)
   }
   ```

5. **Implement `needs_update()` method** (stub for now):
   ```rust
   pub fn needs_update(
       &self,
       source: &impl FileSource,
       path: &Path,
       id: SchemaId,
   ) -> Result<bool, IngestionError> {
       // TODO: Compare file mtime with DB timestamp
       // For now, always return true (always re-ingest)
       Ok(true)
   }
   ```

**Testing**:
```rust
#[test]
fn schema_ingestion_service_ingests_valid_file() {
    let source = InMemoryFileSource::new();
    source.add_file("person.json", r#"{"name": "Person", "properties": []}"#);

    let db = Database::new_in_memory().unwrap();
    let query = schema::query::Query::new_redb(&db);
    let command = schema::command::Command::new_redb(&db);
    let service = SchemaIngestionService::new(&query, &command);

    let schema_id = service.ingest_file(&source, Path::new("person.json")).unwrap();

    // Verify persisted
    let schema = query.find_by_id(schema_id).unwrap().unwrap();
    assert_eq!(schema.name().as_str(), "Person");
}

#[test]
fn schema_ingestion_service_rejects_invalid_file() {
    let source = InMemoryFileSource::new();
    source.add_file("invalid.json", r#"{"name": "", "properties": []}"#);  // Empty name

    let db = Database::new_in_memory().unwrap();
    let query = schema::query::Query::new_redb(&db);
    let command = schema::command::Command::new_redb(&db);
    let service = SchemaIngestionService::new(&query, &command);

    let result = service.ingest_file(&source, Path::new("invalid.json"));

    assert!(result.is_err());
    // Verify NOT persisted
    assert!(query.list().unwrap().is_empty());
}

#[test]
fn schema_ingestion_service_handles_partial_directory_failures() {
    let source = InMemoryFileSource::new();
    source.add_file("valid.json", r#"{"name": "Valid", "properties": []}"#);
    source.add_file("invalid.json", r#"{"name": "", "properties": []}"#);

    let db = Database::new_in_memory().unwrap();
    let query = schema::query::Query::new_redb(&db);
    let command = schema::command::Command::new_redb(&db);
    let service = SchemaIngestionService::new(&query, &command);

    let ids = service.ingest_directory(&source, "*.json").unwrap();

    // Only valid schema was ingested
    assert_eq!(ids.len(), 1);
    assert_eq!(query.list().unwrap().len(), 1);
}
```

**Reference**: See Design Doc 016, Layer 3 (lines 260-386)

---

### 2.4: Implement TemplateIngestionService

**File**: `lithos-core/src/application/services/template_ingestion.rs` (new file)

**Tasks**:

1. **Follow same pattern as `SchemaIngestionService`**:
   - Generic over `template::ports::Query` and `template::ports::Command`
   - `ingest_file()`, `ingest_directory()`, `needs_update()` methods

2. **Add MiniJinja syntax validation**:
   ```rust
   pub fn ingest_file(
       &self,
       source: &impl FileSource,
       path: &Path,
   ) -> Result<TemplateId, IngestionError> {
       // Parse file
       let raw = parsers::parse_template_file(source, path)?;

       // Validate Jinja2 syntax (early failure)
       minijinja::Environment::new()
           .add_template_owned(
               raw.name.as_ref().unwrap(),
               raw.content.as_ref().unwrap()
           )
           .map_err(|e| IngestionError::Validation(e.to_string()))?;

       // Convert to domain
       let template = Template::try_from(raw)
           .map_err(|e| IngestionError::Validation(e.to_string()))?;

       // Persist
       self.command.save(&template)?;

       Ok(template.id())
   }
   ```

**Testing**: Similar pattern to schema tests

**Reference**: See Design Doc 016, Context-Specific Adaptations (lines 575-614)

---

### 2.5: Implement NoteIngestionService

**File**: `lithos-core/src/application/services/note_ingestion.rs` (new file)

**Tasks**:

1. **Follow same pattern as `SchemaIngestionService`**:
   - Generic over `note::ports::Query` and `note::ports::Command`
   - `ingest_file()`, `ingest_directory()`, `needs_update()` methods

2. **Reuse existing note parser**:
   ```rust
   pub fn ingest_file(
       &self,
       source: &impl FileSource,
       path: &Path,
   ) -> Result<Uuid, IngestionError> {
       // Parse markdown + frontmatter
       let parsed = parsers::parse_note_file(source, path)?;

       // Domain conversion already done by parser
       let note = parsed.note;

       // Persist (command handles indexing)
       self.command.create(note.path())?;

       Ok(note.id())
   }
   ```

**Testing**: Similar pattern to schema tests

**Reference**: See Design Doc 016, Context-Specific Adaptations (lines 618-654)

---

### 2.6: Export Services

**File**: `lithos-core/src/application/services/mod.rs` (new file)

**Tasks**:

1. **Declare modules**:
   ```rust
   pub mod schema_ingestion;
   pub mod template_ingestion;
   pub mod note_ingestion;
   ```

2. **Re-export services**:
   ```rust
   pub use schema_ingestion::SchemaIngestionService;
   pub use template_ingestion::TemplateIngestionService;
   pub use note_ingestion::NoteIngestionService;
   ```

**File**: `lithos-core/src/application/mod.rs` (new file)

**Tasks**:

1. **Declare modules**:
   ```rust
   pub mod error;
   pub mod services;
   ```

2. **Re-export commonly used types**:
   ```rust
   pub use error::IngestionError;
   pub use services::{
       SchemaIngestionService,
       TemplateIngestionService,
       NoteIngestionService,
   };
   ```

---

### 2.7: Phase 2 Verification

**Checklist**:

- [ ] All three ingestion services compile
- [ ] All services are generic over CQRS ports
- [ ] All services have `ingest_file()` and `ingest_directory()` methods
- [ ] Partial failure tolerance works in directory ingestion
- [ ] All integration tests pass
- [ ] Tracing instrumentation in place
- [ ] No file I/O in CQRS ports (verify manually)

**Verification Commands**:
```bash
mise run test:unit:core
mise run lint
```

**Output**: Services exist but are not yet used by CLI. Existing code paths still work.

---

## Phase 3: Benchmarks & Performance Validation

**Goal**: Establish baseline performance metrics before optimization.

**Effort**: 1 day

**Status**: ⏳ Not Started

**Prerequisites**: Phases 1 & 2 complete

### 3.1: Create Benchmark File

**File**: `lithos-core/benches/file_ingestion.rs` (new file)

**Tasks**:

1. **Add benchmark setup**:
   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
   use lithos_core::{
       fs::{FsFileSource, InMemoryFileSource},
       application::SchemaIngestionService,
       db::Database,
   };
   use std::path::Path;
   use tempfile::tempdir;
   ```

2. **Create test data generator**:
   ```rust
   fn generate_test_vault(file_count: usize) -> (tempfile::TempDir, Vec<PathBuf>) {
       let temp_dir = tempdir().unwrap();
       let mut files = Vec::new();

       for i in 0..file_count {
           let file_path = temp_dir.path().join(format!("schema_{}.json", i));
           let content = format!(
               r#"{{"name": "Schema{}", "properties": []}}"#,
               i
           );
           std::fs::write(&file_path, content).unwrap();
           files.push(file_path);
       }

       (temp_dir, files)
   }
   ```

3. **Add File I/O baseline benchmark**:
   ```rust
   fn bench_file_read_baseline(c: &mut Criterion) {
       let (vault, files) = generate_test_vault(100);

       let mut group = c.benchmark_group("file_io");
       group.throughput(Throughput::Elements(100));

       group.bench_function("sequential_read_100_files", |b| {
           b.iter(|| {
               for path in &files {
                   let content = std::fs::read_to_string(path).unwrap();
                   black_box(content);
               }
           });
       });

       group.finish();
   }
   ```

4. **Add end-to-end ingestion benchmark**:
   ```rust
   fn bench_schema_ingestion(c: &mut Criterion) {
       let (vault, _files) = generate_test_vault(100);

       let mut group = c.benchmark_group("schema_ingestion");
       group.throughput(Throughput::Elements(100));

       group.bench_function("sequential_100_schemas", |b| {
           b.iter_with_setup(
               || {
                   let db = Database::new_in_memory().unwrap();
                   let query = schema::query::Query::new_redb(&db);
                   let command = schema::command::Command::new_redb(&db);
                   let source = FsFileSource::new(vault.path());
                   (db, query, command, source)
               },
               |(db, query, command, source)| {
                   let service = SchemaIngestionService::new(&query, &command);
                   let ids = service.ingest_directory(&source, "*.json").unwrap();
                   black_box(ids);
               }
           );
       });

       group.finish();
   }
   ```

5. **Register benchmarks**:
   ```rust
   criterion_group!(
       benches,
       bench_file_read_baseline,
       bench_schema_ingestion
   );
   criterion_main!(benches);
   ```

**Reference**: See Design Doc 017, Benchmark Design (lines 933-1093)

---

### 3.2: Run Benchmarks

**Tasks**:

1. **Add benchmark to Cargo.toml**:
   ```toml
   [[bench]]
   name = "file_ingestion"
   harness = false
   ```

2. **Run benchmarks**:
   ```bash
   mise run test:bench:core
   ```

3. **Document baseline results**:
   - Create `docs/benchmarks/file-ingestion-baseline.md`
   - Record throughput numbers
   - Compare against targets from Design Doc 017

**Performance Targets** (from Design Doc 017):

| File Count | Target Time | Acceptable? |
|------------|-------------|-------------|
| 100        | <50ms       | ✅          |
| 1,000      | <500ms      | ✅          |
| 10,000     | <3s         | ⚠️ (needs parallel) |

**Reference**: See Design Doc 017, Scalability Model (lines 860-927)

---

### 3.3: Phase 3 Verification

**Checklist**:

- [ ] Benchmarks compile and run
- [ ] Baseline numbers documented
- [ ] Performance meets targets for 100-1K files
- [ ] Identified bottlenecks match predictions (File I/O > DB > Parsing)

**Output**: Baseline metrics established. Ready for optimization if needed in future phases.

---

## Phase 4: Documentation & Enforcement

**Goal**: Ensure pattern is documented and enforced via CI.

**Effort**: 1-2 days

**Status**: ⏳ Not Started

**Prerequisites**: Phases 1-3 complete

### 4.1: Update project-context.md

**File**: `_bmad-output/project-context.md` (modify existing)

**Tasks**:

1. **Add new architectural rules** (insert after existing port-based CQRS rules):
   ```markdown
   ### File Ingestion Rules

   - **CQRS ports MUST NOT have file I/O methods**: No `load_from_file`, `scan_directory`, etc.
   - **File ingestion MUST use `FileSource` trait**: Abstract over filesystem for testability
   - **Application services orchestrate pipelines**: Services coordinate File → Raw → Domain → Database
   - **Parsing and validation are distinct phases**: File → Raw (parsing) → Domain (validation) → DB
   ```

2. **Add to Definition of Done**:
   ```markdown
   - [ ] No file I/O in CQRS ports (verify with architecture tests)
   - [ ] File ingestion uses `FileSource` trait (not direct `std::fs` in domain)
   ```

3. **Update "Where Does This Code Go?" section**:
   ```markdown
   **File ingestion orchestration?** → `lithos-core/src/application/services/`
   **File source abstraction?** → `lithos-core/src/fs/source.rs`
   **File parsing logic?** → `lithos-core/src/fs/parsers.rs`
   ```

---

### 4.2: Add Architecture Tests

**File**: `lithos-core/tests/architecture.rs` (new file)

**Tasks**:

1. **Add test to prevent file I/O in ports**:
   ```rust
   #[test]
   fn ports_must_not_import_std_fs() {
       let port_files = glob::glob("src/**/ports.rs").unwrap();

       for entry in port_files {
           let path = entry.unwrap();
           let content = std::fs::read_to_string(&path).unwrap();

           assert!(
               !content.contains("std::fs"),
               "Port file {:?} must not import std::fs",
               path
           );

           assert!(
               !content.contains("use std::path::PathBuf"),
               "Port file {:?} must not use PathBuf parameters (database-only)",
               path
           );
       }
   }
   ```

2. **Add test to verify CQRS ports are database-only**:
   ```rust
   #[test]
   fn port_traits_must_not_have_path_parameters() {
       // Parse all port files and check trait method signatures
       // Fail if any trait method has &Path or PathBuf parameter

       // This is a simplified check - could be more sophisticated with syn crate
       let port_files = glob::glob("src/**/ports.rs").unwrap();

       for entry in port_files {
           let path = entry.unwrap();
           let content = std::fs::read_to_string(&path).unwrap();

           assert!(
               !content.contains("fn load_from_file"),
               "Port trait in {:?} has file I/O method",
               path
           );

           assert!(
               !content.contains("fn scan_directory"),
               "Port trait in {:?} has directory scanning method",
               path
           );
       }
   }
   ```

**Dependencies**:
- Add `glob = "0.3"` to `[dev-dependencies]` in `Cargo.toml`

---

### 4.3: Update ADR 010 Status

**File**: `docs/adr/010-file-ingestion.md` (modify existing)

**Tasks**:

1. **Update frontmatter**:
   ```yaml
   status: implemented  # Change from "accepted"
   date_implemented: 2026-02-XX  # Fill in actual date
   ```

2. **Complete migration checklist** (Appendix E):
   - Mark all tasks as complete
   - Add notes about any deviations from original plan

---

### 4.4: Create Migration Guide

**File**: `docs/guides/migrating-to-service-layer.md` (new file)

**Tasks**:

1. **Document migration pattern for future contexts**:
   ```markdown
   # Migrating to Service Layer Pattern

   ## For New Contexts

   When adding a new domain context (e.g., "workspace"), follow this pattern:

   1. Create `Raw*` type with serde derives
   2. Add parser in `fs/parsers.rs`
   3. Implement `TryFrom<Raw*>` for domain type
   4. Create ingestion service in `application/services/`
   5. CQRS ports remain database-only

   ## Anti-Patterns to Avoid

   ❌ Adding `load_from_file()` to CQRS ports
   ❌ Calling `std::fs` directly in domain code
   ❌ Bypassing `Raw*` → Domain validation boundary
   ```

2. **Add code examples** from this refactor

---

### 4.5: Phase 4 Verification

**Checklist**:

- [ ] `project-context.md` updated with new rules
- [ ] Architecture tests pass and prevent violations
- [ ] ADR 010 marked as implemented
- [ ] Migration guide created for future developers
- [ ] All documentation references correct file paths

**Verification Commands**:
```bash
mise run test  # Includes architecture tests
mise run adr:validate
```

**Output**: Pattern is documented, enforced, and ready for future use.

---

## Phase 5: Config Port Cleanup (FINAL TASK)

**Goal**: Fix config port violation as final validation of new pattern.

**Effort**: 0.5-1 day

**Status**: ⏳ Not Started

**Prerequisites**: Phases 1-4 complete

**Why Last?**: This validates that the new architecture works by refactoring the one existing violation.

### 5.1: Audit Config Ports

**File**: `lithos-core/src/config/ports.rs`

**Tasks**:

1. **Review all methods in `Query` and `Command` traits**:
   - ✅ **GOOD**: `get_active_version()`, `with_archived()`, `save_merged()` - Database operations
   - ❌ **BAD** (if present): Any methods with `&Path` parameters or file I/O

2. **Document findings**:
   - Current status: Config ports appear clean (no file I/O methods found)
   - Verify by searching for `Path` imports: `grep -n "use std::path" lithos-core/src/config/ports.rs`

---

### 5.2: Verify Config Ingestion Pattern

**File**: `lithos-core/src/config/ingest.rs`

**Tasks**:

1. **Confirm `build_merged_raw()` follows correct pattern**:
   - ✅ Uses `figment::Provider` trait (file source abstraction)
   - ✅ Returns `RawConfig` (unvalidated)
   - ✅ Validation happens in `Config::try_from(RawConfig)` (not shown, but implied)
   - ✅ No database operations in this file

2. **Document as reference implementation**:
   - Add comment to `config/ingest.rs`:
     ```rust
     //! **Reference Implementation**: This module demonstrates the correct
     //! file ingestion pattern using Figment's Provider trait. Other contexts
     //! should follow this pattern with `FileSource` trait.
     ```

---

### 5.3: Create ConfigIngestionService (If Needed)

**File**: `lithos-core/src/application/services/config_ingestion.rs` (new file, only if pattern doesn't match)

**Tasks**:

**Current Assessment**: Config already follows the correct pattern via Figment. Only create this service if we want consistency with other contexts.

**If creating service**:

1. **Wrap existing `build_merged_raw()` function**:
   ```rust
   pub struct ConfigIngestionService<'a, Q, C> {
       query: &'a config::query::Query<Q>,
       command: &'a config::command::Command<C>,
   }

   impl<'a, Q, C> ConfigIngestionService<'a, Q, C>
   where
       Q: config::ports::Query,
       C: config::ports::Command,
   {
       pub fn ingest_vault_config(
           &self,
           vault_root: &Path,
       ) -> Result<(), IngestionError> {
           // Use existing figment-based ingestion
           let raw = config::ingest::build_merged_raw(vault_root)?;

           // Validate
           let config = Config::try_from(raw)
               .map_err(|e| IngestionError::Validation(e.to_string()))?;

           // Persist
           self.command.save_merged(vault_id, version, &config)?;

           Ok(())
       }
   }
   ```

**Decision**: Defer to implementation time. If config already works correctly, document it as the reference and move on.

---

### 5.4: Phase 5 Verification

**Checklist**:

- [ ] Config ports have zero file I/O methods
- [ ] Config ingestion pattern documented as reference
- [ ] Architecture tests pass for config ports
- [ ] No regressions in config functionality

**Verification Commands**:
```bash
mise run test:unit:config
grep -r "std::fs" lithos-core/src/config/ports.rs  # Should return nothing
```

**Output**: Config context validated as clean. Pattern proven with real example.

---

## Final Verification & Completion

**Tasks**:

1. **Run full quality gate**:
   ```bash
   mise run verify
   ```

2. **Manual verification**:
   ```bash
   # Check for file I/O in ports
   grep -r "std::fs" lithos-core/src/*/ports.rs

   # Check for Path parameters in port traits
   grep -r "fn.*&Path" lithos-core/src/*/ports.rs

   # Verify all services exist
   ls lithos-core/src/application/services/
   ```

3. **Performance verification**:
   ```bash
   mise run test:bench:core
   # Compare results against Design Doc 017 targets
   ```

4. **Documentation checklist**:
   - [ ] ADR 010 status = "implemented"
   - [ ] `project-context.md` updated
   - [ ] Migration guide created
   - [ ] Architecture tests prevent regressions
   - [ ] Benchmarks documented

5. **Commit work**:
   ```bash
   git add .
   git commit -m "feat: implement Service Layer pattern for file ingestion

   - Add FileSource trait for file system abstraction
   - Implement ingestion services for schema/template/note contexts
   - Add architecture tests to prevent port violations
   - Update documentation with new patterns

   Implements ADR 010 and Design Doc 016.
   Baseline benchmarks meet performance targets (100 files: <50ms)."
   ```

---

## Troubleshooting

### Problem: Circular Dependency Between Application and Domain

**Symptom**: Application services import domain types, domain types try to import services

**Solution**:
- Services import domain types: ✅ Allowed (application → domain)
- Domain NEVER imports services: ❌ Forbidden (domain must stay pure)

**Dependency Flow**:
```
Application Services (orchestration)
    ↓ imports
Domain (schema::Query, schema::Command)
    ↓ imports
Port Traits (schema::ports::Query, schema::ports::Command)
    ↑ implemented by
Adapters (schema::adapter)
    ↓ uses
Infrastructure (db::Database)
```

---

### Problem: FileSource trait conflicts with existing code

**Symptom**: Existing code already reads files directly

**Solution**:
- Phase 1-2: Add new code alongside existing code (no breaking changes)
- Phase 3+: Refactor existing code to use services (breaking changes)
- Use feature flags if gradual migration needed

---

### Problem: Tests fail after adding architecture tests

**Symptom**: `ports_must_not_import_std_fs` test fails on existing code

**Solution**:
1. Find violating port file: `grep -r "std::fs" lithos-core/src/*/ports.rs`
2. Move file I/O logic to application service
3. Update port to only have database operations
4. Verify with architecture test

---

### Problem: Benchmarks show poor performance

**Symptom**: Sequential ingestion >500ms for 1K files

**Solution**:
- Check bottleneck: File I/O vs DB vs Parsing
- If File I/O: Consider Phase 2 parallel processing (out of scope for this refactor)
- If DB: Add transaction batching
- If Parsing: Profile with `cargo flamegraph`

**Refer to**: Design Doc 017, Optimization Strategies (lines 439-690)

---

## Out of Scope (Future Phases)

These optimizations are **not** part of this refactor:

### ❌ Parallel Processing (rayon)
- **Status**: Deferred to Performance Phase 2
- **Why**: Current sequential processing meets targets for 1K files
- **When**: Implement when real vaults exceed 10K files

### ❌ File Watching (notify crate)
- **Status**: Deferred to LSP Server implementation
- **Why**: CLI doesn't need incremental updates
- **When**: Implement for live indexing in LSP server

### ❌ Content-Addressed Caching
- **Status**: Deferred to Performance Phase 4
- **Why**: Diminishing returns (Phase 3 file watching already provides 10-100x speedup)
- **When**: Only if file watching proves insufficient

### ❌ CLI Integration
- **Status**: Deferred to separate PR/epic
- **Why**: This refactor focuses on infrastructure and services only
- **When**: After all services proven stable with tests

**Refer to**: Design Doc 017, Implementation Roadmap (lines 1142-1326)

---

## Quick Reference

### Key Files Created

```
lithos-core/src/
├── application/
│   ├── mod.rs (new)
│   ├── error.rs (new)
│   └── services/
│       ├── mod.rs (new)
│       ├── schema_ingestion.rs (new)
│       ├── template_ingestion.rs (new)
│       └── note_ingestion.rs (new)
└── fs/
    ├── source.rs (new)
    └── parsers.rs (new, extract from existing parsers.rs)

lithos-core/tests/
└── architecture.rs (new)

lithos-core/benches/
└── file_ingestion.rs (new)

docs/
├── guides/
│   └── migrating-to-service-layer.md (new)
└── benchmarks/
    └── file-ingestion-baseline.md (new)
```

### Key Commands

```bash
# Run unit tests for specific module
mise run test:unit:fs
mise run test:unit:core

# Run architecture tests
mise run test:integration

# Run benchmarks
mise run test:bench:core

# Full verification
mise run verify

# Check for port violations
grep -r "std::fs" lithos-core/src/*/ports.rs
```

### Success Metrics

- [ ] 100 files ingested in <50ms (sequential)
- [ ] 1,000 files ingested in <500ms (sequential)
- [ ] Zero file I/O methods in CQRS ports
- [ ] Architecture tests prevent regressions
- [ ] All integration tests pass
- [ ] Documentation complete and accurate

---

## Estimated Timeline

| Phase | Effort | Deliverable |
|-------|--------|-------------|
| Phase 1: Infrastructure | 2-3 days | FileSource trait, parsers, tests |
| Phase 2: Services | 2-3 days | Ingestion services for 3 contexts |
| Phase 3: Benchmarks | 1 day | Baseline performance metrics |
| Phase 4: Documentation | 1-2 days | Updated docs, architecture tests |
| Phase 5: Config Cleanup | 0.5-1 day | Config port validation |
| **Total** | **8-10 days** | Complete Service Layer refactor |

---

## Questions & Decisions Log

**Q: Should we create ConfigIngestionService for consistency?**
- A: Defer to implementation. Config already uses Figment (correct pattern). Only create service wrapper if needed for consistency with other contexts.

**Q: Should parsers support async I/O?**
- A: No. File ingestion is batch work, not server work. Stay synchronous. See Design Doc 017, Async vs Sync Decision (lines 732-856).

**Q: How to handle schema DAG resolution?**
- A: Schema ingestion service will need special `ingest_with_resolution()` method. See Design Doc 016, Schema Ingestion (lines 520-571).

**Q: Should we add file watching in this refactor?**
- A: No. File watching is deferred to LSP server implementation. This refactor only adds the infrastructure to support it later.

---

**End of Implementation Plan**

For detailed rationale and alternatives, see [ADR 010](../adr/010-file-ingestion.md).
For code examples and patterns, see [Design Doc 016](../design/016-file-ingestion-architecture.md).
For performance analysis, see [Design Doc 017](../design/017-file-ingestion-performance.md).
