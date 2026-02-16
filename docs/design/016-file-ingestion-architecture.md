---
title: "File Ingestion Implementation Design"
description: "Implementation design for file ingestion Service Layer pattern with FileSource trait, parsers, and application services"
author: "Claude (Senior Software Architect)"
date: "2026-02-16"
status: "proposal"
related_adrs:
  - "010-file-ingestion"
  - "003-domain-serialization"
  - "006-persistence-cache-infrastructure"
tags:
  - architecture
  - cqrs
  - file-ingestion
  - implementation
---

# File Ingestion Implementation Design

## Context

This document describes the **implementation design** for the file ingestion architecture documented in [ADR 010](../adr/010-file-ingestion.md).

For research findings and alternative analysis, see ADR 010 Appendix F.
For performance considerations, see [Design Doc 017](./017-file-ingestion-performance.md).

## Table of Contents

1. [Proposed Architecture](#proposed-architecture)
2. [Layer 1: FileSource Trait](#layer-1-filesource-trait)
3. [Layer 2: File Parsers](#layer-2-file-parsers)
4. [Layer 3: Ingestion Services](#layer-3-ingestion-services)
5. [Layer 4: CQRS Ports](#layer-4-cqrs-ports)
6. [Workflow Examples](#workflow-examples)
7. [Context-Specific Adaptations](#context-specific-adaptations)
8. [Implementation Plan](#implementation-plan)
9. [Testing Strategy](#testing-strategy)
10. [Migration Path](#migration-path)

---

## Proposed Architecture

### Overview: File Ingestion Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FILE INGESTION ARCHITECTURE                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│  File System     │  External boundary
│  (.json, .toml)  │
└────────┬─────────┘
         │
         ▼ FileSource trait (abstraction)
┌──────────────────┐
│  File I/O        │  fs/ infrastructure
│  (read, watch)   │
└────────┬─────────┘
         │
         ▼ Parse (serde)
┌──────────────────┐
│  Raw* Types      │  Unvalidated input
│  (serde derives) │
└────────┬─────────┘
         │
         ▼ TryFrom (validation boundary)
┌──────────────────┐
│  Domain Types    │  Validated entities
│  (rkyv derives)  │
└────────┬─────────┘
         │
         ▼ CQRS Command (persistence)
┌──────────────────┐
│  Database        │  Storage layer
│  (redb)          │
└──────────────────┘

ORCHESTRATED BY: Application Services (application/ layer)
```

---

## Layer 1: FileSource Trait

**Purpose**: Abstract over different file sources (filesystem, embedded, in-memory, network).

**Location**: `lithos-core/src/fs/source.rs`

```rust
use std::path::{Path, PathBuf};

/// Abstraction for reading raw file content.
///
/// This trait separates file I/O from parsing and validation. Implementations
/// can provide filesystem access, embedded resources, in-memory buffers, or
/// network sources.
pub trait FileSource: Send + Sync {
    /// Error type for file I/O operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Read a file's contents as a string.
    ///
    /// # Errors
    /// Returns I/O errors (file not found, permissions, encoding).
    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error>;

    /// Check if a file exists.
    fn exists(&self, path: &Path) -> bool;

    /// List all files matching a pattern (for bulk ingestion).
    ///
    /// # Errors
    /// Returns I/O errors (directory traversal, permissions).
    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error>;
}

/// Standard filesystem implementation.
pub struct FsFileSource {
    root: PathBuf,
}

impl FsFileSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl FileSource for FsFileSource {
    type Error = std::io::Error;

    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error> {
        let full_path = self.root.join(path);
        std::fs::read_to_string(full_path)
    }

    fn exists(&self, path: &Path) -> bool {
        self.root.join(path).exists()
    }

    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error> {
        // Use glob crate or walkdir
        // Return relative paths (not absolute)
        todo!("Implement with glob crate")
    }
}

/// In-memory file source for testing.
#[cfg(test)]
pub struct InMemoryFileSource {
    files: std::collections::HashMap<PathBuf, String>,
}

#[cfg(test)]
impl FileSource for InMemoryFileSource {
    type Error = std::io::Error;

    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn list_files(&self, _pattern: &str) -> Result<Vec<PathBuf>, Self::Error> {
        Ok(self.files.keys().cloned().collect())
    }
}
```

**Rationale**:

- ✅ **Testability**: Mock file system without touching disk
- ✅ **Flexibility**: Support embedded resources, network sources
- ✅ **Separation**: File I/O abstracted from parsing logic
- ✅ **Follows Diesel pattern**: Similar to `MigrationSource` trait

---

## Layer 2: File Parsers

**Purpose**: Convert raw file bytes to `Raw*` types (unvalidated).

**Location**: `lithos-core/src/fs/parsers.rs`

```rust
use std::path::Path;
use crate::fs::{FileSource, error::ParseError};

/// Parse a schema file into unvalidated RawSchema.
///
/// # Errors
/// Returns ParseError if file I/O or deserialization fails.
pub fn parse_schema_file(
    source: &impl FileSource,
    path: &Path,
) -> Result<crate::schema::raw::RawSchema, ParseError> {
    let content = source.read_to_string(path).map_err(ParseError::from)?;

    // Detect format by extension
    match path.extension().and_then(|s| s.to_str()) {
        Some("json") => {
            serde_json::from_str(&content).map_err(ParseError::from)
        }
        Some("toml") => {
            toml::from_str(&content).map_err(ParseError::from)
        }
        Some("yaml" | "yml") => {
            serde_yml::from_str(&content).map_err(ParseError::from)
        }
        _ => Err(ParseError::UnsupportedFormat(path.to_path_buf())),
    }
}

/// Parse a template file into unvalidated RawTemplate.
pub fn parse_template_file(
    source: &impl FileSource,
    path: &Path,
) -> Result<crate::template::raw::RawTemplate, ParseError> {
    // Templates are plain text (Jinja2 syntax)
    let content = source.read_to_string(path).map_err(ParseError::from)?;

    // RawTemplate just wraps the content + metadata
    Ok(crate::template::raw::RawTemplate {
        name: path.file_stem()
            .and_then(|s| s.to_str())
            .map(String::from),
        content: Some(content),
        path: Some(path.to_path_buf()),
    })
}

/// Parse a note file (Markdown + frontmatter).
pub fn parse_note_file(
    source: &impl FileSource,
    path: &Path,
) -> Result<crate::note::parser::ParsedNote, ParseError> {
    let content = source.read_to_string(path).map_err(ParseError::from)?;

    // Use existing note parser
    crate::note::parser::parse(&content, path.to_str().unwrap_or("unknown"))
        .map_err(ParseError::from)
}
```

**Rationale**:

- ✅ **Generic over source**: Works with any `FileSource` implementation
- ✅ **Format detection**: Supports JSON, TOML, YAML for schemas
- ✅ **Reuses existing parsers**: Note parser already exists
- ✅ **Error handling**: ParseError wraps I/O and serde errors

---

## Layer 3: Ingestion Services

**Purpose**: Orchestrate File → Raw → Domain → Database pipeline.

**Location**: `lithos-core/src/application/services/ingestion.rs`

```rust
use std::path::Path;
use tracing::instrument;

use crate::{
    fs::{FileSource, parsers},
    schema::{self, raw::RawSchema, Schema, SchemaId},
    template::{self, raw::RawTemplate, Template},
    config::{self, raw::RawConfig, Config},
};

/// Schema ingestion service.
///
/// Orchestrates the workflow: File → RawSchema → Schema → Database.
pub struct SchemaIngestionService<'a, Q, C> {
    query: &'a schema::query::Query<Q>,
    command: &'a schema::command::Command<C>,
}

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

    /// Ingest a single schema file.
    ///
    /// # Errors
    /// Returns error if file reading, parsing, validation, or persistence fails.
    #[instrument(skip(self, source), fields(path = %path.display()))]
    pub fn ingest_file(
        &self,
        source: &impl FileSource,
        path: &Path,
    ) -> Result<SchemaId, IngestionError> {
        // Step 1: File I/O + Parsing
        let raw = parsers::parse_schema_file(source, path)?;

        // Step 2: Validation (Raw → Domain)
        let schema = Schema::try_from(raw)?;

        // Step 3: Persistence (Database write)
        let id = schema.id();
        self.command.save_with_metadata(&schema, &Default::default())?;

        Ok(id)
    }

    /// Ingest all schema files matching a pattern.
    ///
    /// # Errors
    /// Returns error if file discovery or any ingestion step fails.
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
                    // Continue with other files (partial failure tolerance)
                }
            }
        }

        Ok(ids)
    }

    /// Check if a schema needs re-ingestion (file changed since last load).
    ///
    /// This supports incremental updates.
    pub fn needs_update(
        &self,
        source: &impl FileSource,
        path: &Path,
        id: SchemaId,
    ) -> Result<bool, IngestionError> {
        // Compare file modification time with database timestamp
        // (Requires storing metadata about last ingestion)
        // For now, always return true (always re-ingest)
        Ok(true)
    }
}

/// Template ingestion service (similar pattern).
pub struct TemplateIngestionService<'a, Q, C> {
    query: &'a template::query::Query<Q>,
    command: &'a template::command::Command<C>,
}

// Similar implementation...

/// Config ingestion service (already exists in config::ingest).
///
/// This is a reference implementation that others should follow.
pub use crate::config::ingest::build_merged_raw as ingest_config;
```

**Rationale**:

- ✅ **Single Responsibility**: Each service orchestrates one context's ingestion
- ✅ **Explicit workflow**: File → Raw → Domain → Database steps are clear
- ✅ **Partial failure tolerance**: Directory ingestion continues on errors
- ✅ **Tracing**: Each step is instrumented for observability
- ✅ **Incremental update support**: `needs_update()` enables file watching
- ✅ **Testable**: Can inject fake `FileSource` and fake CQRS ports

---

## Layer 4: CQRS Ports

**Critical Rule**: CQRS ports MUST NOT have file I/O methods.

**Current ports are correct** — they only have database operations:

```rust
// schema/ports.rs - ✅ CORRECT (database-only)
pub trait Query {
    fn find_by_id(&self, id: SchemaId) -> Result<Option<Schema>, Error>;
    fn list(&self) -> Result<Vec<Schema>, Error>;
    // ... other database queries
}

pub trait Command {
    fn save(&self, schema: &Schema) -> Result<(), Error>;
    fn delete(&self, id: SchemaId) -> Result<(), Error>;
    // ... other database commands
}
```

**Anti-pattern to avoid**:

```rust
// ❌ NEVER add methods like this to ports:
pub trait Query {
    fn load_from_file(&self, path: &Path) -> Result<Schema, Error>;  // WRONG!
    fn scan_directory(&self, dir: &Path) -> Result<Vec<Schema>, Error>;  // WRONG!
}
```

---

## Workflow Examples

### Workflow 1: Initial Vault Ingestion

```rust
// CLI command: lithos init <vault-path>
pub fn init_vault(vault_path: &Path, db: &Database) -> Result<(), Error> {
    // Setup file source
    let source = FsFileSource::new(vault_path);

    // Setup CQRS ports
    let schema_query = schema::query::Query::new_redb(&db);
    let schema_command = schema::command::Command::new_redb(&db);
    let config_query = config::query::Query::new_redb(&db);
    let config_command = config::command::Command::new_redb(&db);

    // Step 1: Ingest config (already implemented)
    let raw_config = config::ingest::build_merged_raw(vault_path)?;
    let config = Config::try_from(raw_config)?;
    config_command.save(&config)?;

    // Step 2: Ingest schemas
    let schema_service = SchemaIngestionService::new(&schema_query, &schema_command);
    let schema_ids = schema_service.ingest_directory(&source, "schemas/**/*.{json,toml,yaml}")?;
    tracing::info!(count = schema_ids.len(), "Schemas ingested");

    // Step 3: Ingest templates
    let template_service = TemplateIngestionService::new(&template_query, &template_command);
    let template_ids = template_service.ingest_directory(&source, "templates/**/*.md")?;
    tracing::info!(count = template_ids.len(), "Templates ingested");

    // Step 4: Ingest notes (bulk)
    let note_service = NoteIngestionService::new(&note_query, &note_command);
    let note_ids = note_service.ingest_directory(&source, "**/*.md")?;
    tracing::info!(count = note_ids.len(), "Notes ingested");

    Ok(())
}
```

### Workflow 2: Incremental Update (File Watcher)

```rust
// File watcher detects schema file changed
pub fn handle_schema_file_changed(
    path: &Path,
    vault_path: &Path,
    db: &Database,
) -> Result<(), Error> {
    let source = FsFileSource::new(vault_path);
    let schema_query = schema::query::Query::new_redb(&db);
    let schema_command = schema::command::Command::new_redb(&db);
    let service = SchemaIngestionService::new(&schema_query, &schema_command);

    // Re-ingest just the changed file
    let schema_id = service.ingest_file(&source, path)?;
    tracing::info!(schema_id = %schema_id, "Schema re-ingested");

    Ok(())
}
```

### Workflow 3: Hot Path Read (Database-Only)

```rust
// LSP query: "Get schema by name" (performance-critical)
pub fn get_schema_for_lsp(name: &str, db: &Database) -> Result<Option<Schema>, Error> {
    let query = schema::query::Query::new_redb(&db);

    // Zero-copy hot path (NO file I/O!)
    query.with_archived_by_name(name, |archived| {
        // Access archived data without deserialization
        archived.properties().len()
    })
}
```

---

## Context-Specific Adaptations

### Config Ingestion (Already Implemented Correctly)

**Location**: `lithos-core/src/config/ingest.rs`

**Pattern**: Uses `figment::Provider` trait for source abstraction.

**Workflow**:

```
Files (global, vault) → Figment (merge) → RawConfig → Config → Database
```

**This is the gold standard** — other contexts should follow this pattern.

---

### Schema Ingestion

**Files**: `.lithos/schemas/**/*.{json,toml,yaml}`

**Workflow**:

```
File → RawSchema (serde) → Schema (TryFrom) → Database
```

**Special concerns**:

- **DAG resolution**: Schemas with `extends` must be resolved in topological order
- **Batch ingestion**: All schemas loaded before resolution
- **PropertyBank**: Global property registry updated after all schemas ingested

**Implementation**:

```rust
pub fn ingest_schemas_with_resolution(
    source: &impl FileSource,
    db: &Database,
) -> Result<(), Error> {
    let query = schema::query::Query::new_redb(&db);
    let command = schema::command::Command::new_redb(&db);

    // Step 1: Load all raw schemas
    let paths = source.list_files("schemas/**/*.{json,toml,yaml}")?;
    let mut raw_schemas = Vec::new();
    for path in paths {
        let raw = parsers::parse_schema_file(source, &path)?;
        raw_schemas.push(raw);
    }

    // Step 2: Validate and build DAG
    let schemas: Vec<Schema> = raw_schemas
        .into_iter()
        .map(Schema::try_from)
        .collect::<Result<_, _>>()?;

    let graph = schema::resolver::SchemaGraph::build(&schemas)?;
    let resolved_order = graph.topological_sort()?;

    // Step 3: Persist in resolution order
    for schema_id in resolved_order {
        let schema = schemas.iter().find(|s| s.id() == schema_id).unwrap();
        command.save_with_metadata(schema, &Default::default())?;
    }

    Ok(())
}
```

---

### Template Ingestion

**Files**: `.lithos/templates/**/*.md` (Jinja2 templates)

**Workflow**:

```
File → RawTemplate (plain text) → Template (validation) → Database
```

**Special concerns**:

- **Template compilation**: MiniJinja compilation happens at ingestion time
- **Syntax validation**: Check Jinja2 syntax before persisting
- **Partial templates**: Track dependencies between templates

**Implementation**:

```rust
pub fn ingest_template(
    source: &impl FileSource,
    path: &Path,
    command: &impl template::ports::Command,
) -> Result<TemplateId, Error> {
    // Parse file
    let raw = parsers::parse_template_file(source, path)?;

    // Validate Jinja2 syntax (early failure)
    minijinja::Environment::new()
        .add_template_owned(raw.name.as_ref().unwrap(), raw.content.as_ref().unwrap())?;

    // Convert to domain
    let template = Template::try_from(raw)?;

    // Persist
    command.save(&template)?;

    Ok(template.id())
}
```

---

### Note Ingestion (Markdown + Frontmatter)

**Files**: `**/*.md` (user notes)

**Workflow**:

```
File → ParsedNote (parser) → Note (domain) → Database
```

**Special concerns**:

- **Bulk ingestion**: Initial vault indexing processes 1000s of notes
- **Incremental updates**: File watcher triggers re-parsing on save
- **Link extraction**: Backlinks/forward links indexed during parsing
- **Task extraction**: Dataview-style tasks indexed for queries

**Implementation**:

```rust
pub fn ingest_note(
    source: &impl FileSource,
    path: &Path,
    command: &impl note::ports::Command,
) -> Result<Uuid, Error> {
    // Parse markdown + frontmatter
    let parsed = parsers::parse_note_file(source, path)?;

    // Convert to domain (Note aggregate already constructed)
    let note = parsed.note;

    // Persist (command handles indexing)
    command.create(note.path())?;

    Ok(note.id())
}
```

---

## Implementation Plan

### Phase 1: Infrastructure Foundation

**Goal**: Add `FileSource` trait and file parsers without breaking existing code.

**Tasks**:

1. **Create `FileSource` trait** (`fs/source.rs`)
   - Define trait with `read_to_string`, `exists`, `list_files`
   - Implement `FsFileSource` (filesystem)
   - Implement `InMemoryFileSource` (testing)

2. **Extract file parsing logic** (`fs/parsers.rs`)
   - Move schema file parsing from wherever it currently lives
   - Add `parse_schema_file(source, path)` function
   - Add `parse_template_file(source, path)` function
   - Reuse existing `note::parser` for notes

3. **Add `ParseError` type** (`fs/error.rs`)
   - Wrap I/O errors and serde errors
   - Rich error messages for users

**Tests**:

```rust
#[test]
fn fs_file_source_reads_real_files() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.json");
    fs::write(&file_path, r#"{"name": "test"}"#).unwrap();

    let source = FsFileSource::new(temp.path());
    let content = source.read_to_string(Path::new("test.json")).unwrap();

    assert_eq!(content, r#"{"name": "test"}"#);
}

#[test]
fn in_memory_file_source_reads_from_hashmap() {
    let mut source = InMemoryFileSource::new();
    source.add_file("test.json", r#"{"name": "test"}"#);

    let content = source.read_to_string(Path::new("test.json")).unwrap();

    assert_eq!(content, r#"{"name": "test"}"#);
}

#[test]
fn parse_schema_file_deserializes_json() {
    let source = InMemoryFileSource::new();
    source.add_file("schema.json", r#"{"name": "Person", "properties": []}"#);

    let raw = parsers::parse_schema_file(&source, Path::new("schema.json")).unwrap();

    assert_eq!(raw.name, Some("Person".to_string()));
}
```

---

### Phase 2: Ingestion Services

**Goal**: Create application services that orchestrate File → Domain → Database.

**Tasks**:

1. **Create `SchemaIngestionService`** (`application/services/schema_ingestion.rs`)
   - `ingest_file(source, path)` method
   - `ingest_directory(source, pattern)` method
   - `needs_update(source, path, id)` method (for incremental)

2. **Create `TemplateIngestionService`** (`application/services/template_ingestion.rs`)
   - Similar to schema service
   - Add MiniJinja syntax validation

3. **Create `NoteIngestionService`** (`application/services/note_ingestion.rs`)
   - Bulk ingestion for initial vault load
   - Single-file ingestion for incremental updates

4. **Add `IngestionError` type** (`application/error.rs`)
   - Wraps ParseError, ValidationError, CommandError
   - User-facing error messages

**Tests**:

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
```

---

### Phase 3: CLI Integration

**Goal**: Wire up ingestion services in CLI commands.

**Tasks**:

1. **Refactor `lithos init` command** (`lithos-cli/src/commands/init.rs`)
   - Use `SchemaIngestionService` instead of direct file reads
   - Use `TemplateIngestionService` for templates
   - Use `NoteIngestionService` for notes

2. **Add `lithos refresh` command** (optional)
   - Re-ingest changed files only
   - Use `needs_update()` to skip unchanged files

3. **Add file watcher support** (Phase 2, LSP)
   - Use `notify` crate to watch vault directory
   - Call ingestion services on file changes

**CLI usage**:

```bash
# Initial ingestion
lithos init /path/to/vault

# Re-ingest everything
lithos refresh --all

# Re-ingest just schemas
lithos refresh --schemas

# Watch for changes (future)
lithos watch /path/to/vault
```

---

### Phase 4: Documentation & Testing

**Goal**: Ensure pattern is well-documented and tested.

**Tasks**:

1. **Integration tests** (`lithos-core/tests/ingestion_flow.rs`)
   - Test full pipeline: File → Raw → Domain → Database
   - Test partial failure handling (some files invalid)
   - Test incremental updates

2. **Update project-context.md**
   - Add rule: "CQRS ports MUST NOT have file I/O methods"
   - Add rule: "File ingestion MUST use FileSource trait"
   - Add rule: "Application services orchestrate File → Domain → Database"

---

## Testing Strategy

### Unit Testing (Per Layer)

#### FileSource Tests

```rust
// fs/source.rs
#[cfg(test)]
mod tests {
    #[test]
    fn fs_file_source_returns_not_found_for_missing_file() {
        let source = FsFileSource::new("/tmp");
        let result = source.read_to_string(Path::new("nonexistent.json"));
        assert!(result.is_err());
    }

    #[test]
    fn in_memory_file_source_lists_all_files() {
        let mut source = InMemoryFileSource::new();
        source.add_file("a.json", "{}");
        source.add_file("b.json", "{}");

        let files = source.list_files("*.json").unwrap();
        assert_eq!(files.len(), 2);
    }
}
```

#### Parser Tests

```rust
// fs/parsers.rs
#[cfg(test)]
mod tests {
    #[test]
    fn parse_schema_file_detects_format_by_extension() {
        let source = InMemoryFileSource::new();
        source.add_file("schema.json", r#"{"name": "Test"}"#);
        source.add_file("schema.toml", r#"name = "Test""#);

        let json_result = parse_schema_file(&source, Path::new("schema.json"));
        let toml_result = parse_schema_file(&source, Path::new("schema.toml"));

        assert!(json_result.is_ok());
        assert!(toml_result.is_ok());
    }

    #[test]
    fn parse_schema_file_returns_error_for_invalid_json() {
        let source = InMemoryFileSource::new();
        source.add_file("invalid.json", "{not valid json");

        let result = parse_schema_file(&source, Path::new("invalid.json"));

        assert!(matches!(result, Err(ParseError::JsonError(_))));
    }
}
```

#### Validation Tests (Already Exist)

```rust
// schema/aggregate.rs
#[cfg(test)]
mod tests {
    #[test]
    fn schema_try_from_rejects_empty_name() {
        let raw = RawSchema { name: Some("".to_string()), ..Default::default() };
        let result = Schema::try_from(raw);
        assert!(result.is_err());
    }
}
```

### Integration Testing (Full Pipeline)

```rust
// lithos-core/tests/ingestion_flow.rs
use lithos_core::{
    db::Database,
    fs::{FileSource, InMemoryFileSource, parsers},
    schema::{self, Schema},
    application::services::SchemaIngestionService,
};

#[test]
fn ingests_valid_schema_from_file_to_database() {
    // Setup
    let mut source = InMemoryFileSource::new();
    source.add_file(
        "schemas/person.json",
        r#"{
            "name": "Person",
            "properties": [
                {"name": "name", "type": "string", "required": true}
            ]
        }"#,
    );

    let db = Database::new_in_memory().unwrap();
    let query = schema::query::Query::new_redb(&db);
    let command = schema::command::Command::new_redb(&db);
    let service = SchemaIngestionService::new(&query, &command);

    // Execute
    let schema_id = service
        .ingest_file(&source, Path::new("schemas/person.json"))
        .expect("ingestion should succeed");

    // Verify
    let persisted = query.find_by_id(schema_id).unwrap().expect("schema should exist");
    assert_eq!(persisted.name().as_str(), "Person");
    assert_eq!(persisted.properties().len(), 1);
}

#[test]
fn ingestion_handles_partial_failures_in_directory() {
    let mut source = InMemoryFileSource::new();
    source.add_file("schemas/valid.json", r#"{"name": "Valid", "properties": []}"#);
    source.add_file("schemas/invalid.json", r#"{"name": "", "properties": []}"#);  // Empty name

    let db = Database::new_in_memory().unwrap();
    let query = schema::query::Query::new_redb(&db);
    let command = schema::command::Command::new_redb(&db);
    let service = SchemaIngestionService::new(&query, &command);

    let ids = service
        .ingest_directory(&source, "schemas/**/*.json")
        .unwrap();

    // Only valid schema was ingested
    assert_eq!(ids.len(), 1);
    assert_eq!(query.list().unwrap().len(), 1);
}
```

### End-to-End Testing (CLI)

```rust
// lithos-cli/tests/cli_ingestion.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_init_ingests_all_schemas_and_templates() {
    let temp = tempfile::tempdir().unwrap();
    let vault = temp.path();

    // Setup vault structure
    fs::create_dir_all(vault.join(".lithos/schemas")).unwrap();
    fs::write(
        vault.join(".lithos/schemas/person.json"),
        r#"{"name": "Person", "properties": []}"#,
    ).unwrap();

    // Execute
    Command::cargo_bin("lithos")
        .unwrap()
        .arg("init")
        .arg(vault)
        .assert()
        .success()
        .stdout(predicate::str::contains("Schemas ingested: 1"));

    // Verify database exists
    assert!(vault.join(".lithos/lithos.db").exists());
}
```

---

## Migration Path

### Step 1: Add Infrastructure (No Breaking Changes)

- Add `fs/source.rs` with `FileSource` trait
- Add `fs/parsers.rs` with parsing functions
- Add tests for new infrastructure

**Result**: Existing code still works, new code available for use.

---

### Step 2: Add Ingestion Services (Parallel to Existing)

- Add `application/services/schema_ingestion.rs`
- Add `application/services/template_ingestion.rs`
- Add `application/services/note_ingestion.rs`

**Result**: Services exist but not yet used by CLI.

---

### Step 3: Refactor CLI to Use Services

- Update `lithos-cli/src/commands/init.rs` to use ingestion services
- Remove any direct file I/O from CLI (delegate to services)

**Result**: CLI uses new architecture, old code paths removed.

---

### Step 4: Remove Legacy Code (If Any)

- Search for file I/O in CQRS ports (there shouldn't be any, but verify)
- Remove any `load_from_file` methods if they exist
- Update tests to use new patterns

**Result**: Clean architecture, no legacy code.

---

### Step 5: Document & Enforce

- Update `project-context.md` with new rules
- Add architecture tests to prevent regression

**Result**: Pattern is documented and enforced.

---

## Conclusion

### Summary of Recommendations

1. **Separate file I/O from CQRS ports** — Ports should only handle database operations
2. **Use `FileSource` trait** for file system abstraction (testability, flexibility)
3. **Parse files in `fs/` infrastructure** — Generic parsers that return `Raw*` types
4. **Validate in domain layer** — `TryFrom<Raw*>` for Domain types
5. **Orchestrate in application services** — Services coordinate File → Raw → Domain → Database
6. **Keep CQRS ports pure** — No file I/O methods in Query/Command traits

### Architectural Benefits

- ✅ **Testability**: Each layer can be tested independently
- ✅ **Performance**: Hot path (DB reads) optimized separately from cold path (file ingestion)
- ✅ **Maintainability**: Clear separation of concerns, single responsibility
- ✅ **Flexibility**: Easy to add new sources (network, embedded, in-memory)
- ✅ **Follows proven patterns**: Matches Cargo, rustc, Diesel implementations

### Next Steps

1. Implement Phase 1 (Infrastructure Foundation)
2. Implement Phase 2 (Ingestion Services)
3. Implement Phase 3 (CLI Integration)
4. Add comprehensive tests
5. Document in ADR and project-context.md

---

## References

- [ADR 010: File Ingestion Architecture](../adr/010-file-ingestion.md) - Architectural decision and research findings
- [Design Doc 017: File Ingestion Performance](./017-file-ingestion-performance.md) - Performance optimization strategies
- [ADR 003: Domain Serialization](../adr/003-domain-serialization.md) - Raw* types and validation boundaries
- [ADR 006: Persistence Cache Infrastructure](../adr/006-persistence-cache-infrastructure.md) - Zero-copy database reads
