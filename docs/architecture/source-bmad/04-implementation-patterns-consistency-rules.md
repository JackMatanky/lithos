---
title: "Implementation Patterns & Consistency Rules"
description: "Development patterns, naming conventions, and consistency rules for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-03-11"
section: "Implementation Standards"
---

# Implementation Patterns & Consistency Rules

## Related Documentation

**MUST READ FIRST:**

- [Core Architectural Decisions](./03-core-architectural-decisions.md) - Files as source of truth, unified Repository pattern
- [ADR 002: Storage Pattern](../../../docs/adr/002-storage-pattern.md) - Unified Repository traits architecture
- [ADR 003: Serialization Strategy](../../../docs/adr/003-serialization-strategy.md) - Raw/Domain/View shapes model
- [State Machine Pattern Reference](../../../docs/refs/rust/state-machine-pattern.md) - Multi-phase pipeline patterns
- [Rust Naming Taxonomy](../../naming-taxonomy.md) - Method naming conventions
- [Rust Idioms Reference](../../../docs/refs/rust/idioms.md) - General Rust patterns (ownership, strings, etc.)
- [Rust Style Guide](../../../docs/refs/rust/style.md) - Code style and documentation

## Overview

**Purpose:** Ensure multiple AI agents write compatible, consistent code for the Lithos file-based CLI tool with key-value database.

**Architecture Summary:**

- **Files as Source of Truth:** User-editable vault files are authoritative
- **Database as Cache:** Redb provides rebuildable, query-optimized projection
- **Unified Repository Pattern:** Single trait per context (no CQRS split)
- **File Ingestion Pipeline:** File → Raw → Domain → Storage → Database
- **Context Isolation:** Business contexts don't cross-import
- **Zero-Copy Access:** rkyv enables fast reads without deserialization

---

## Critical Patterns (Project-Specific)

### 1. File Ingestion to Database Pipeline ("Parse, Don't Validate")

**THE CORE PATTERN for this project** - Every context follows this flow.

**Critical Principle: "Parse, Don't Validate"**

The distinction between validation and parsing:

- **Validation**: Checks correctness but throws away information (`fn validate(x) -> Result<(), Error>`)
- **Parsing**: Transforms less-structured input to more-structured output, preserving information in types (`fn parse(x) -> Result<ValidType, Error>`)

Each phase in our pipeline is **parsing**, not just validation:

```text
┌─────────────────────────────────────────────────────────────────┐
│ VAULT FILES (Source of Truth - Least Structured)                │
│ - User edits in Obsidian/Vim                                    │
│ - Markdown files with YAML frontmatter                          │
│ - Schema definitions (YAML)                                     │
│ - Templates (Jinja2)                                            │
└─────────────────┬───────────────────────────────────────────────┘
                  │ FileReader (security-validated)
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 1: PARSE SYNTAX (File → Raw)                              │
│ - Read file contents via FileReader                               │
│ - Parse with serde (YAML/TOML/JSON)                             │
│ - Creates Raw* type (syntactically valid, semantically unknown) │
│ - Location: <context>/ingestor.rs or loader.rs                  │
│                                                                 │
│ PARSING: Bytes → Structured data (but unvalidated)              │
└─────────────────┬───────────────────────────────────────────────┘
                  │ serde::Deserialize
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ RAW TYPES (Syntax Layer - Unvalidated Semantics)                │
│ - RawSchema, RawNote, RawTemplate, RawConfig                    │
│ - All fields Optional<T> for better error messages              │
│ - No behavior (zero impl blocks except TryFrom)                 │
│ - Location: <context>/raw.rs                                    │
│                                                                 │
│ Purpose: Separate serde concerns from domain logic              │
└─────────────────┬───────────────────────────────────────────────┘
                  │ TryFrom<Raw*> (SEMANTIC PARSING BOUNDARY)
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 2: PARSE SEMANTICS (Raw → Domain)                         │
│ - TryFrom<Raw*> for Domain PARSES validated invariants          │
│ - Syntax validation (regex, identifier format)                  │
│ - Semantic validation (refs exist, no cycles)                   │
│ - Multi-phase for complex parsing (schema has 8 phases)         │
│                                                                 │
│ PARSING: Unvalidated data → Validated domain types              │
│ Information preserved: Validated in type system, not thrown away│
└─────────────────┬───────────────────────────────────────────────┘
                  │ Result<Domain, Error>
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ DOMAIN TYPES (Fully Validated - Most Structured)                │
│ - Schema, Note, Template, Config (or Stored* in current impl)   │
│ - Private fields, validated constructors ONLY                   │
│ - rkyv derives for zero-copy storage                            │
│ - Optional serde (feature-gated) for CLI JSON output            │
│ - Location: <context>/aggregate.rs or storage.rs                │
│                                                                 │
│ Guarantee: If you have a Domain type, it's valid. No re-checks. │
└─────────────────┬───────────────────────────────────────────────┘
                  │ Repository::save()
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 3: PERSIST (Domain → Database)                            │
│ - Repository trait: save(), get(), list(), with_archived()      │
│ - Implementations: RedbStorage, InMemoryStorage, FakeStorage    │
│ - rkyv serializes to bytes                                      │
│ - Location: <context>/storage.rs                                │
│                                                                 │
│ No validation here: Domain types are already valid              │
└─────────────────┬───────────────────────────────────────────────┘
                  │ rkyv::Archive
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ DATABASE (Redb - Zero-Copy Key-Value Store)                     │
│ - Stores rkyv-serialized bytes                                  │
│ - Tables: schema_by_id, note_by_id, etc.                        │
│ - Metadata tables for staleness (file hash, mtime)              │
│ - Location: db/ infrastructure                                  │
└─────────────────┬───────────────────────────────────────────────┘
                  │ with_archived(id, |archived| ...)
                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ ARCHIVED* TYPES (Zero-Copy Reads)                               │
│ - rkyv-generated: ArchivedSchema, ArchivedNote, etc.            │
│ - Memory-mapped from database (no deserialization)              │
│ - Fast queries via closure-based access                         │
│ - Only create *View if domain shape is inefficient              │
│                                                                 │
│ Still valid: Archive preserves domain type invariants           │
└─────────────────────────────────────────────────────────────────┘
```

**Key Insights:**

1. **Both phases are parsing**: `File → Raw` parses syntax, `Raw → Domain` parses semantics
2. **Information flows forward**: Each type is more structured than the previous
3. **Validate once, use everywhere**: Once you have a `Domain` type, assume it's valid
4. **No shotgun parsing**: Validation happens at clear boundaries, not scattered throughout code
5. **Type system enforces correctness**: Invalid states are unrepresentable (can't construct invalid domain types)

**Implementation Checklist:**

For each context (note, schema, template, config):

- [ ] **Raw type** in `<context>/raw.rs` (serde only, Option<T> fields, zero behavior)
- [ ] **Domain/Stored type** in `<context>/aggregate.rs` or `storage.rs` (rkyv derives, private fields)
- [ ] **TryFrom<Raw> parsing** (NOT just validation - transforms to stronger type)
- [ ] **Smart constructors** on domain types (`try_new()`, not public fields)
- [ ] **Repository trait** in `<context>/storage.rs` (get, save, list, with_archived)
- [ ] **Loader** in `<context>/loader.rs` (orchestrates File → Raw → Domain → Storage)
- [ ] **FileReader integration** for secure file access
- [ ] **Hash-based staleness** in metadata tables (optional, for optimization)
- [ ] **No redundant validation** - if domain type exists, it's valid (no re-checking)

**Parse, Don't Validate Checklist:**

- [ ] `TryFrom<Raw>` returns `Result<Domain, Error>` (parsed type, not `Result<(), Error>`)
- [ ] Domain types have private fields (can't be constructed invalidly)
- [ ] Validation errors are structured (not generic strings)
- [ ] Business logic accepts domain types (not `Raw*` or primitives that need validation)
- [ ] No validation inside domain methods (validation at construction only)

### 2. Unified Repository Pattern (No CQRS Split)

**Pattern:** Single trait per context combining reads and writes.

✅ **Correct (Unified Repository):**

```rust
// Define once per context
pub trait Repository {
    type Error: std::error::Error;

    // Reads
    fn get(&self, id: &Id) -> Result<Option<T>, Self::Error>;
    fn list(&self) -> Result<Vec<T>, Self::Error>;
    fn with_archived<F, R>(&self, id: &Id, f: F) -> Result<Option<R>, Self::Error>
        where F: FnOnce(&Archived<T>) -> R;

    // Writes
    fn save(&self, entity: &T) -> Result<(), Self::Error>;
    fn delete(&self, id: &Id) -> Result<(), Self::Error>;
}

// Implement for each backend
pub struct RedbRepository<'db> { /* ... */ }
impl<'db> Repository for RedbRepository<'db> { /* ... */ }

pub struct InMemoryRepository { /* ... */ }
impl Repository for InMemoryRepository { /* ... */ }

pub struct FakeRepository { /* ... */ }
impl Repository for FakeRepository { /* ... */ }
```

❌ **Wrong (CQRS Split):**

```rust
// DON'T split into Query and Command traits
pub trait QueryPort { /* ... */ }
pub trait CommandPort { /* ... */ }
```

**Benefits:**

- Simpler dependency management
- Easier testing (single mock)
- Avoids interface bloat
- Idiomatic Rust (like std::fs::File)

### 3. Context Isolation (No Cross-Imports)

**Rule:** Business contexts (note, schema, template) MUST NOT import each other.

**Dependency Flow:**

```text
┌─────────────────────────────────────────────────────────┐
│ CLI LAYER (lithos-cli)                                  │
│ - Orchestrates cross-context workflows                  │
│ - Can import any context                                │
└──────────────────────┬──────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ note/        │ │ schema/      │ │ template/    │
│ (BUSINESS)   │ │ (BUSINESS)   │ │ (BUSINESS)   │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       └────────────────┼────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
        ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ config/      │ │ db/          │ │ fs/          │
│ (CROSS-CUT)  │ │ (INFRA)      │ │ (INFRA)      │
└──────────────┘ └──────────────┘ └──────────────┘
```

✅ **Allowed Imports:**

```rust
// From any business context (note, schema, template):
use crate::config::Global;      // Cross-cutting config
use crate::db::Database;        // Infrastructure
use crate::fs::FileReader;        // Infrastructure
use crate::bounds::Bounds;      // Cross-cutting utility

// Within same context:
use super::raw::RawSchema;
use super::storage::Repository;
use super::error::SchemaError;
```

❌ **Forbidden Imports:**

```rust
// Business contexts importing each other:
use crate::note::Note;          // ❌ From schema context
use crate::schema::Schema;      // ❌ From note context
use crate::template::Template;  // ❌ From config context
```

**Enforcement:** Architecture tests verify no forbidden imports exist.

### 4. Serialization Shapes (Raw → Domain → Archived)

**The Three Shapes:**

| Shape             | Purpose                   | Derives                                 | Location               | Persistence             |
| :---------------- | :------------------------ | :-------------------------------------- | :--------------------- | :---------------------- |
| **`Raw*`**        | Parse unvalidated input   | `serde::Deserialize`                    | `<context>/raw.rs`     | Never                   |
| **Domain/Stored** | Validated business entity | `rkyv::Archive` + feature-gated `serde` | `<context>/storage.rs` | Always                  |
| **`Archived*`**   | Zero-copy DB access       | Auto-generated by rkyv                  | N/A (generated)        | N/A                     |
| **`*View`**       | Read-optimized projection | `rkyv::Archive`                         | `<context>/view.rs`    | Rarely (only if needed) |

**Rules:**

1. **Raw types have zero behavior** - no impl blocks except parsing helpers
2. **Validation happens in `TryFrom<Raw*>`** - single validation boundary
3. **Domain types have private fields** - validated constructors only
4. **`*View` types are optional** - only create if domain shape is inefficient
5. **`Archived*` provides free optimization** - use before creating `*View`

**Example Flow:**

```rust
// 1. Parse file to Raw (unvalidated)
let raw: RawSchema = serde_yaml::from_str(file_contents)?;

// 2. Validate Raw → Domain (TryFrom boundary)
let domain: Schema = raw.try_into()?;

// 3. Persist Domain → Database (Repository trait)
storage.save(&domain)?;

// 4. Read with zero-copy (closure-based)
storage.with_archived(id, |archived: &ArchivedSchema| {
    // Use archived.property_name() directly
    archived.name()
})?;
```

### 5. State Machine Pattern for Multi-Phase Pipelines

**When to use:** Linear pipelines with strict phase ordering (e.g., schema loading with 8 phases).

**See:** [State Machine Pattern Reference](../../../docs/refs/rust/state-machine-pattern.md)

**Example: Schema Loader (8 Phases)**

```rust
// Phase 1: Discover
let discovered = SchemaLoader::discover(fs_reader)?;

// Phase 2: Parse
let parsed = discovered.parse()?;

// Phase 3: Validate
let validated = parsed.validate()?;

// Phase 4: Dereference ($ref expansion)
let dereferenced = validated.dereference(property_bank)?;

// Phase 5: Graph (build inheritance graph)
let graphed = dereferenced.graph()?;

// Phase 6: Sort (topological order)
let sorted = graphed.sort()?;

// Phase 7: Resolve (merge properties)
let resolved = sorted.resolve()?;

// Phase 8: Project (persist to DB)
let projected = resolved.project(storage)?;
```

**Key Points:**

- Each phase consumes previous state (linear progression)
- Type system prevents invalid state transitions
- Orchestration layer handles branching logic (cache checks)
- State machine only for linear pipeline

### 6. Database Table Naming & Organization

**Pattern:** Consistent naming for key-value tables and metadata.

**Tables per Context:**

```rust
// Schema context tables
const SCHEMA_BY_ID: TableDefinition<&str, &[u8]> = TableDefinition::new("schema_by_id");
const SCHEMA_METADATA: TableDefinition<&str, &[u8]> = TableDefinition::new("schema_metadata");
const RAW_SCHEMA_FILES: TableDefinition<&str, &[u8]> = TableDefinition::new("raw_schema_files");

// Note context tables
const NOTE_BY_ID: TableDefinition<&str, &[u8]> = TableDefinition::new("note_by_id");
const NOTE_BY_PATH: MultimapTableDefinition<&str, &str> = MultimapTableDefinition::new("note_by_path");
const NOTE_METADATA: TableDefinition<&str, &[u8]> = TableDefinition::new("note_metadata");

// Template context tables
const TEMPLATE_BY_ID: TableDefinition<&str, &[u8]> = TableDefinition::new("template_by_id");
const TEMPLATE_METADATA: TableDefinition<&str, &[u8]> = TableDefinition::new("template_metadata");

// Config context tables
const CONFIG_GLOBAL: TableDefinition<&str, &[u8]> = TableDefinition::new("config_global");
const CONFIG_VAULT: TableDefinition<&str, &[u8]> = TableDefinition::new("config_vault");
```

**Naming Convention:**

- Primary tables: `<context>_by_<key>` (e.g., `schema_by_id`, `note_by_path`)
- Metadata tables: `<context>_metadata` (stores file hash, mtime for staleness)
- Raw file tables: `raw_<context>_files` (versioned history, up to 5 versions)
- Multimap tables: Use `MultimapTableDefinition` for one-to-many relationships

---

## Naming Conventions

### Method Naming (Critical - Read Taxonomy)

**ALL methods MUST follow:** [Rust Naming Taxonomy](../../naming-taxonomy.md)

**Quick Reference:**

| Pattern          | Usage                             | Examples                           | Parse vs Validate            |
| :--------------- | :-------------------------------- | :--------------------------------- | :--------------------------- |
| `parse_*`        | Convert to validated type         | `parse_name()`, `parse_id()`       | ✅ Parse (returns type)      |
| `try_new()`      | Validated constructor             | `SchemaId::try_new(s)`             | ✅ Parse (smart constructor) |
| `validate_*`     | Check only (returns `Result<()>`) | AVOID - throws away info           | ❌ Validate (use parse)      |
| `find_*`         | Optional result                   | `find_schema()`, `find_by_name()`  | Query (after parsing)        |
| `get_*`          | Singleton (panics if not found)   | `get_config()`, `get_by_id()`      | Query (after parsing)        |
| `list_*`         | Multiple results (all)            | `list_schemas()`, `list_all()`     | Query (after parsing)        |
| `filter_*`       | Filtered subset                   | `filter_stale()`, `filter_by_tag()`| Query (after parsing)        |
| `all_*`          | Check all match predicate         | `all_stale()`, `all_valid()`       | Query (after parsing)        |
| `any_*`          | Check any match predicate         | `any_stale()`, `any_invalid()`     | Query (after parsing)        |
| `with_*`         | Zero-copy closure                 | `with_archived()`, `with_config()` | Query (after parsing)        |
| `save`           | Upsert operation                  | `save()`, `save_all()`             | Persistence                  |
| `create`         | Insert-only                       | `create()`, `create_new()`         | Persistence                  |
| `delete`         | Remove operation                  | `delete()`, `delete_by_id()`       | Persistence                  |
| `is_*` / `has_*` | Boolean checks (singular)         | `is_valid()`, `has_parent()`       | Query (after parsing)        |
| `as_*`           | Free conversion                   | `as_str()`, `as_ref()`             | Accessor                     |
| `to_*`           | Expensive conversion              | `to_string()`, `to_owned()`        | Conversion                   |
| `into_*`         | Consuming conversion              | `into_inner()`, `into_bytes()`     | Conversion                   |

**Parse vs Validate Naming:**

```rust
// ✅ GOOD: Parse (returns validated type)
impl SchemaName {
    pub fn try_new(s: &str) -> Result<Self, SchemaError> {
        // Validation + construction = parsing
        if !is_valid_identifier(s) {
            return Err(SchemaError::InvalidName);
        }
        Ok(Self(s.into()))
    }
}

// ✅ GOOD: Parse (TryFrom is parsing)
impl TryFrom<RawSchema> for Schema {
    type Error = SchemaError;
    fn try_from(raw: RawSchema) -> Result<Self, SchemaError> {
        // This IS parsing, not just validation
        Ok(Schema {
            name: SchemaName::try_new(&raw.name)?,
            // ...
        })
    }
}

// ❌ BAD: Validate (throws away information)
pub fn validate_schema_name(s: &str) -> Result<(), SchemaError> {
    if !is_valid_identifier(s) {
        return Err(SchemaError::InvalidName);
    }
    Ok(()) // Information lost!
}

// ❌ BAD: Would need to check again later
pub fn use_schema_name(s: &str) -> Result<(), Error> {
    validate_schema_name(s)?;  // Check once
    // ... later ...
    if !is_valid_identifier(s) { // Re-check (redundant!)
        // ...
    }
}
```

**NO `get_` prefix on simple getters:**

```rust
// ✅ Good
pub fn name(&self) -> &str { &self.name }

// ❌ Bad
pub fn get_name(&self) -> &str { &self.name }
```

**Constructor Naming:**

```rust
// ✅ Infallible constructor
pub fn new() -> Self { Self::default() }

// ✅ Fallible constructor (validates/parses)
pub fn try_new(value: T) -> Result<Self, Error> { /* ... */ }

// ✅ Parsing constructor
pub fn parse(s: &str) -> Result<Self, Error> { /* ... */ }

// ❌ Avoid "validate" prefix (use "try_new" or "parse")
pub fn validate_new(value: T) -> Result<Self, Error> { /* ... */ }
```

### Type Naming

```rust
// Crates/Packages
lithos-core     // kebab-case in Cargo.toml
lithos_core     // snake_case in Rust imports

// Modules & Files
note.rs, schema.rs, loader.rs  // snake_case

// Structs & Enums
Note, Schema, SchemaId, NoteError  // PascalCase

// Traits
Repository, FileReader, Validator  // PascalCase, descriptive

// Constants
MAX_DEPTH, DEFAULT_TIMEOUT  // SCREAMING_SNAKE_CASE

// Functions & Variables
load_schema, file_path, raw_input  // snake_case

// Lifetimes (use descriptive names)
'db, 'tx, 'bytes, 'src  // NOT 'a, 'b

// Type Parameters
T, E, K, V  // Single letter
```

---

## Type-Driven Design (Quick Reference)

**Core Principle:** Make illegal states unrepresentable.

**Parse, Don't Validate Principle:**

- **Validation** throws away information: `fn validate(x) -> Result<(), Error>`
- **Parsing** preserves information in types: `fn parse(x) -> Result<ValidType, Error>`
- Once parsed, downstream code can assume validity (no re-checking)

**Checklist:**

- [ ] Fields are private (use getters)
- [ ] Smart constructors for validation (`try_new()`, `parse()`)
- [ ] Use `TryFrom<Raw*>` for parsing boundaries (not `validate()` methods)
- [ ] Newtypes for domain constraints (`SchemaName`, `NoteId`)
- [ ] `#[non_exhaustive]` on public enums/structs
- [ ] Expose collections via iterators (not `&mut Vec`)
- [ ] Accept validated types as parameters (not primitives needing validation)
- [ ] No validation in domain methods (validation at construction only)

**See Full Guides:**

- [Appendix A: Type-Driven Design Patterns](#appendix-a-type-driven-design-patterns)
- [Type-Driven Design Reference](../../../docs/refs/rust/type-driven-design.md)

---

## Error Handling

```rust
// Context-specific errors (thiserror)
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("schema not found: {0}")]
    NotFound(SchemaId),

    #[error("circular dependency detected: {0}")]
    CircularDependency(String),
}

// Never unwrap/expect in production
// ✅ Use ? operator
let schema = storage.get(id)?;

// ✅ Use ok_or/context
let name = raw.name.ok_or(SchemaError::MissingName)?;

// CLI errors (miette)
use miette::{Diagnostic, SourceSpan};
#[derive(Debug, Diagnostic, thiserror::Error)]
#[error("invalid schema definition")]
struct InvalidSchema {
    #[source_code]
    src: String,
    #[label("expected non-empty name")]
    span: SourceSpan,
}
```

---

## Async Rules

```rust
// Core: ALWAYS synchronous
pub fn load_schema(path: &Path) -> Result<Schema, Error> {
    // Sync file I/O
}

// CLI: Async edges only
#[tokio::main]
async fn main() -> miette::Result<()> {
    // Bridge to sync core
    let result = tokio::task::spawn_blocking(|| {
        load_schema(path)
    }).await??;
}

// ❌ NEVER #[async_trait] in lithos-core
// ❌ NEVER async methods in Repository traits
```

---

## Enforcement Guidelines

**All AI Agents MUST:**

1. **Follow file ingestion pipeline** for every context (File → Raw → Domain → Storage)
2. **Use unified Repository trait** (not CQRS split)
3. **Respect context isolation** (no business context cross-imports)
4. **Implement serialization shapes** correctly (Raw → Domain → Archived)
5. **Use naming taxonomy** for all methods (find*/get*/list*/with*\*)
6. **Follow database naming** for tables (`<context>_by_<key>`)
7. **Never unwrap/expect** in production code
8. **Keep core synchronous** (async only in CLI)
9. **Private fields by default** (type-driven design)
10. **Run `mise run verify`** before committing (fmt + lint + test)

**Pre-Commit Checklist:**

- [ ] All patterns followed (file ingestion, storage, serialization)
- [ ] Context boundaries respected (no forbidden imports)
- [ ] Naming conventions correct (methods, types, tables)
- [ ] Tests pass (`mise run test`)
- [ ] Clippy clean (`mise run lint`)
- [ ] Formatted (`mise run fmt`)

---

## Appendix A: Type-Driven Design Patterns

### Validation Through Construction

```rust
/// A validated schema name (non-empty, lowercase alphanumeric).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaName(Box<str>);  // Private field

impl SchemaName {
    pub fn try_new(name: impl Into<String>) -> Result<Self, SchemaError> {
        let name = name.into();

        if name.is_empty() {
            return Err(SchemaError::EmptyName);
        }

        if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
            return Err(SchemaError::InvalidNameFormat);
        }

        Ok(Self(name.into_boxed_str()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### Newtype Pattern

```rust
/// UUID v7 (time-ordered) note identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteId(Uuid);

impl NoteId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Positive, non-zero count.
#[derive(Debug, Clone, Copy)]
pub struct Count(NonZeroUsize);

impl Count {
    pub fn new(value: usize) -> Result<Self, ValidationError> {
        NonZeroUsize::new(value)
            .map(Self)
            .ok_or(ValidationError::ZeroCount)
    }
}
```

### Visibility Control

```rust
pub struct Config {
    vault_path: PathBuf,        // Private
    max_cache_size: usize,      // Private
}

impl Config {
    // Read-only access
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    // Controlled mutation with validation
    pub fn set_max_cache_size(&mut self, size: usize) -> Result<(), ConfigError> {
        if size == 0 {
            return Err(ConfigError::InvalidCacheSize);
        }
        self.max_cache_size = size;
        Ok(())
    }
}
```

### Non-Exhaustive Types

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum LinkStyle {
    WikiLink,
    Markdown,
    Embed,
    // Can add variants without breaking code
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub title: String,
    pub date: SystemTime,
    // Can add fields without breaking code
}
```

---

## Appendix B: General Rust Patterns

For general Rust idioms (ownership, strings, conversions, etc.), see:

- [Rust Idioms Reference](../../../docs/refs/rust/idioms.md)
- [Rust Style Guide](../../../docs/refs/rust/style.md)

Key points:

- Prefer `&str`, `&Path`, `&[T]` in function parameters
- Use `Box<str>` for immutable owned strings (not `String`)
- Use `?` operator for error propagation
- Use `TryFrom`/`From` traits for conversions
- Use `impl Trait` for flexible inputs
- Keep `mut` scopes tight
- Use RAII for resource management

---

## Quick Reference Tables

### Decision Matrix: When to Create Each Type

| Type Layer        | Purpose                   | When to Create             | Location               | Derives                                 |
| :---------------- | :------------------------ | :------------------------- | :--------------------- | :-------------------------------------- |
| **Raw\***         | Accept external input     | Always (for file input)    | `<context>/raw.rs`     | `serde::Deserialize`                    |
| **Domain/Stored** | Validated business entity | Always                     | `<context>/storage.rs` | `rkyv::Archive` + feature-gated `serde` |
| **Archived\***    | Zero-copy DB access       | Automatic (rkyv-generated) | N/A (generated)        | N/A (generated)                         |
| **\*View**        | Read-optimized projection | Rarely (profiling only)    | `<context>/view.rs`    | `rkyv::Archive`                         |

### Repository Trait Pattern

```rust
// Define once per context
pub trait Repository {
    fn get(&self, id: &Id) -> Result<Option<T>, Error>;
    fn save(&self, entity: &T) -> Result<(), Error>;
    fn list(&self) -> Result<Vec<T>, Error>;
    fn with_archived<F, R>(&self, id: &Id, f: F) -> Result<Option<R>, Error>
        where F: FnOnce(&Archived<T>) -> R;
}

// Implement for each backend
impl Repository for RedbRepository { /* ... */ }
impl Repository for InMemoryRepository { /* ... */ }
impl Repository for FakeRepository { /* ... */ }
```

### Context Isolation Rules

| From Context                      | Can Import             | Cannot Import           |
| :-------------------------------- | :--------------------- | :---------------------- |
| Business (note, schema, template) | config, db, fs, bounds | Other business contexts |
| Cross-cutting (config)            | db, fs                 | Business contexts       |
| Infrastructure (db, fs)           | Nothing                | All contexts            |

### Pipeline Pattern

```text
File → [parse] → Raw* → [validate] → Domain → [persist] → Database
                  ↑                     ↑                      ↑
                serde              TryFrom<Raw*>         rkyv::Archive
```

### Critical Anti-Patterns

| ❌ Never Do This                       | ✅ Do This Instead                             |
| :------------------------------------- | :--------------------------------------------- |
| `pub fields` on domain types           | Private fields + accessor methods              |
| `String` for validated text            | Newtype wrapper (e.g., `SchemaName(Box<str>)`) |
| `unwrap()` / `expect()` in production  | `?` operator with `Result<T, E>`               |
| Business contexts importing each other | Use infrastructure or CLI orchestration        |
| `#[async_trait]` in `lithos-core`      | Sync-first core, async at edges                |
| Creating `*View` prematurely           | Use `Archived<Domain>` first, profile          |
| Methods on `Raw*` types                | `TryFrom<Raw*>` for validation boundary        |
| CQRS split (Query/Command traits)      | Unified `Repository` trait                     |
| Event sourcing for orchestration       | Functional composition with `Result<T, E>`     |

---

**End of Implementation Patterns & Consistency Rules**
