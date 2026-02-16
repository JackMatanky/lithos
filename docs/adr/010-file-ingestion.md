---
name: file-ingestion
status: accepted
supersedes: []
date_proposed: 2026-02-16
date_decided: 2026-02-16
date_implemented: 2026-02-16
stakeholders: [Jack (Developer), Architecture Team]
---

# ADR 010: File Ingestion Architecture

## Context

Lithos needs to ingest configuration files (TOML/JSON/YAML), schemas, templates, and notes from the filesystem into the database. The project follows port-based CQRS architecture with strict separation of concerns: CQRS ports abstract database operations only, and the domain layer remains pure and infrastructure-free.

**Current Problem**: The existing architecture mistakenly mixes file I/O concerns into CQRS ports, violating separation of concerns and making the system harder to test, reason about, and evolve.

**Evaluation Criteria**:

1. **Architectural Integrity**: CQRS ports must remain database-only (no file I/O)
2. **Context Isolation**: Business contexts (config, schema, template, note) must not cross-import
3. **Testability**: File ingestion must be testable without touching the filesystem
4. **Performance**: Support incremental updates (file watching) and parallel processing
5. **Maintainability**: Clear separation between file I/O, parsing, validation, and persistence
6. **Real-world Validation**: Follow established patterns from mature Rust projects

**Forces at Play**:

- **Technical**: Need multi-stage pipeline (File → Parse → Validate → Persist) with clear boundaries
- **Business**: Support 100-100K files with acceptable performance (see design doc 014)
- **Architectural**: Maintain hexagonal architecture and port-based CQRS principles

## Decision

We will separate file ingestion from database persistence using a **Service Layer pattern** with explicit transformation pipelines:

```
File System → FileSource trait → Parsers → Raw* → Domain (TryFrom) → CQRS Ports → Database
            (abstraction)      (fs/)     (serde) (validation)   (db-only)
```

**Key Components**:

1. **FileSource Trait** (`fs/source.rs`): Abstracts file system access for testability
2. **File Parsers** (`fs/parsers.rs`): Convert files to unvalidated `Raw*` types
3. **Domain Validation**: `TryFrom<Raw*>` enforces business rules at boundary
4. **Application Services** (`application/services/`): Orchestrate the complete pipeline
5. **CQRS Ports**: Remain database-only (no file I/O methods)

**Critical Rules**:

- ✅ CQRS ports MUST NOT have file I/O methods (`load_from_file`, `scan_directory`, etc.)
- ✅ File ingestion MUST use `FileSource` trait for abstraction
- ✅ Application services orchestrate the `File → Raw → Domain → Database` workflow
- ✅ Parsing and validation are distinct phases with explicit boundaries

## Alternatives Considered

### Alternative 1: Repository Pattern (Single Abstraction for All Sources)

**Pattern**:

```rust
pub trait SchemaRepository {
    fn load_from_file(&self, path: &Path) -> Result<Schema, Error>;
    fn load_from_db(&self, id: SchemaId) -> Result<Option<Schema>, Error>;
    fn save(&self, schema: &Schema) -> Result<(), Error>;
}
```

**Pros**:

- Single trait for all data access
- Familiar pattern from OOP

**Cons**:

- Violates Interface Segregation Principle (reads need writes)
- Mixes file I/O with database operations (can't optimize separately)
- Hard to test (mocks need both filesystem and database)
- Tight coupling between ingestion and persistence
- Can't split hot path (DB reads) from cold path (file ingestion)

**Why Rejected**: Violates port-based CQRS and separation of concerns. Can't optimize hot database reads independently from cold file ingestion.

---

### Alternative 2: Gateway Pattern (Separate Gateways per Source)

**Pattern**:

```rust
pub trait FileGateway {
    fn load_schema(&self, path: &Path) -> Result<Schema, Error>;
}

pub trait DatabaseGateway {
    fn find_schema(&self, id: SchemaId) -> Result<Option<Schema>, Error>;
    fn save_schema(&self, schema: &Schema) -> Result<(), Error>;
}
```

**Pros**:

- Separates file I/O from database operations
- Testable in isolation

**Cons**:

- Still mixes parsing/validation with I/O in `FileGateway`
- No clear orchestration layer (who calls what?)
- Validation boundary unclear (Gateway or caller?)

**Why Rejected**: Partial solution that doesn't fully separate concerns. Lacks explicit workflow coordination.

---

### Alternative 3: "Loader" Ports in CQRS (Current Anti-Pattern)

**Pattern**:

```rust
pub trait Command {
    fn load_from_file(&self, path: &Path) -> Result<Schema, Error>;  // ❌ File I/O
    fn save(&self, schema: &Schema) -> Result<(), Error>;  // ✅ Database
}
```

**Pros**:

- Simple to implement initially
- All operations in one place

**Cons**:

- Violates Single Responsibility (port does file I/O + parsing + validation + database)
- Untestable (mock must fake filesystem and database)
- Tight coupling (database adapter depends on filesystem)
- Lifecycle confusion (file ingestion is periodic workflow, not query operation)
- Performance (can't optimize hot reads separately from cold ingestion)

**Why Rejected**: Violates fundamental architectural principles. Creates tight coupling and makes testing nearly impossible.

---

### Alternative 4: Event Sourcing (File Changes → Events → Projections)

**Pattern**:

```rust
pub enum SchemaEvent {
    FileCreated { path: PathBuf, content: String },
    FileModified { path: PathBuf, content: String },
}

fn handle_schema_file_created(event: SchemaEvent::FileCreated) -> Result<(), Error> {
    // Parse, validate, persist
}
```

**Pros**:

- Supports incremental updates naturally
- Audit trail of all changes

**Cons**:

- Overkill for Phase 1 (no event bus yet)
- Adds complexity (event storage, replay)
- Doesn't address separation of concerns (still need file I/O abstraction)

**Why Rejected**: Deferred to Phase 2 (LSP server). Too complex for initial implementation when simpler Service Layer pattern addresses all current needs.

## Technical Validation

### Research Findings

Analysis of 5 mature Rust projects (Cargo, rustc, Diesel, config-rs, tree-sitter) shows **consistent patterns**:

| Project     | File I/O Layer       | Parsing Layer      | Validation Layer  | Persistence Layer     |
| ----------- | -------------------- | ------------------ | ----------------- | --------------------- |
| Cargo       | `fs::read_to_string` | `TomlManifest`     | `Manifest`        | `PackageRegistry`     |
| rustc       | `SourceFileLoader`   | `Parser → AST`     | `HIR::lower`      | Query system (cache)  |
| Diesel      | `MigrationSource`    | `Migration::parse` | SQL validation    | `Connection::execute` |
| config-rs   | `Provider::data`     | serde              | Type extraction   | N/A (read-only)       |
| tree-sitter | `fs::read_to_string` | `Parser::parse`    | N/A (syntax only) | N/A (in-memory)       |

**Common Themes Across All Projects**:

1. **File I/O is a separate concern** from parsing, validation, and persistence
2. **Trait-based source abstraction** enables testing without filesystem (Diesel's `MigrationSource`, config-rs's `Provider`)
3. **Multi-stage pipeline** with explicit boundaries (`File → Raw → Validated → Stored`)
4. **Application layer coordinates** the workflow, not infrastructure
5. **Incremental updates** track changes and minimize re-processing (rustc's query system, cargo's fingerprints)

**Key Insight from rustc**: The Rust compiler separates `SourceFileLoader` (file I/O) from `Parser` (syntax) from `HIR` (validated IR). This enables incremental compilation by caching at each stage independently.

**Key Insight from Cargo**: `TomlManifest` (unvalidated) vs `Manifest` (validated) mirrors our `Raw*` vs domain pattern. The `to_real_manifest()` method is an explicit validation boundary, just like our `TryFrom<Raw*>` implementations.

**Key Insight from Diesel**: The `MigrationSource` trait abstracts over filesystem, embedded resources, or custom sources—exactly what our `FileSource` trait achieves.

### Benchmarks & Prototypes

Performance analysis documented in `docs/design/014-file-ingestion-performance.md` shows:

**Bottleneck Hierarchy** (per-file latency):

- File I/O: 200-500 µs (SSD) — **100x slower than parsing**
- DB writes: 50-200 µs — **20-50x slower than parsing**
- Parsing: 3.5 µs (baseline) — **already optimized**

**Performance Validation**:

- Sequential processing: Acceptable for 1K files (~300ms total)
- Parallel processing (rayon): Required for 10K+ files (4-8x speedup)
- Incremental updates (file watching): Essential for 100K+ files (10-100x speedup)

**Real-World Performance Data** (from research):

- ripgrep: 350K files/second (parallel file search)
- fd-find: 7.5x faster than GNU find (parallel directory traversal)
- rust-analyzer: 10-100x speedup via incremental indexing

**Conclusion**: Separating file I/O from database operations enables independent optimization. Hot path (database reads) can use zero-copy patterns while cold path (file ingestion) can use parallel processing.

### Proof of Concept

The `config::ingest::build_merged_raw()` implementation already demonstrates this pattern using figment's `Provider` trait. This serves as the reference implementation for other contexts.

## Consequences

### Positive

1. **Testability**: Each layer can be tested independently
   - File I/O: Mock with `InMemoryFileSource`
   - Parsing: Test with fake file contents
   - Validation: Test `TryFrom<Raw*>` with constructed inputs
   - Persistence: Test CQRS ports with in-memory database

2. **Performance**: Hot path (database reads) optimized separately from cold path (file ingestion)
   - Zero-copy database reads via `with_archived()` methods
   - Parallel file processing via rayon
   - Incremental updates via file watching (future)

3. **Maintainability**: Clear separation of concerns
   - Each layer has single responsibility
   - Explicit boundaries make codebase easier to navigate
   - Follows established Rust patterns (matches Cargo, rustc, Diesel)

4. **Architectural Integrity**: Port-based CQRS intact
   - Ports remain database-only
   - Context isolation maintained
   - No circular dependencies

5. **Flexibility**: Easy to add new sources
   - Network sources via HTTP `FileSource` implementation
   - Embedded resources via `include_str!`
   - Testing with `InMemoryFileSource`

### Negative

1. **More Layers**: Service layer adds indirection
   - **Mitigation**: Each layer has clear responsibility, indirection is justified
   - **Benefit**: Testability and maintainability outweigh complexity cost

2. **Requires Discipline**: Developers must use services, not bypass to ports
   - **Mitigation**: Architecture tests prevent violations (fail CI if file I/O in ports)
   - **Mitigation**: Documentation in `project-context.md` makes rules explicit

3. **Migration Effort**: Refactor existing code to use new pattern
   - **Mitigation**: Phased implementation (infrastructure → services → CLI)
   - **Mitigation**: No breaking changes until Phase 3 (CLI integration)

### Risks

1. **Risk: Developers bypass services and add file I/O to ports**
   - **Likelihood**: Medium (if pattern not well-documented)
   - **Impact**: High (breaks architectural integrity)
   - **Mitigation**: Architecture tests in CI, ADR documentation, code review checklist

2. **Risk: Performance regression on large vaults (100K+ files)**
   - **Likelihood**: Low (design validated with benchmarks)
   - **Impact**: High (user-facing latency)
   - **Mitigation**: Criterion benchmarks in CI, performance targets documented

3. **Risk: File watching edge cases (renames, rapid edits, platform differences)**
   - **Likelihood**: Medium (file watching has platform quirks)
   - **Impact**: Medium (incremental updates may miss changes)
   - **Mitigation**: Periodic full re-index (daily), use battle-tested `notify` crate

## References

- [File Ingestion Architecture Design (Design Doc 016)](../../design/016-file-ingestion-architecture.md) - Implementation design and code structure
- [File Ingestion Performance Design (Design Doc 017)](../../design/017-file-ingestion-performance.md) - Performance optimization strategies and benchmarks
- [Cargo Source Code](https://github.com/rust-lang/cargo) - `TomlManifest` pattern reference
- [rustc Source Code](https://github.com/rust-lang/rust) - Multi-stage compilation pipeline reference
- [Diesel Migrations](https://github.com/diesel-rs/diesel/tree/master/diesel_migrations) - `MigrationSource` trait pattern
- [Figment Configuration](https://github.com/SergioBenitez/Figment) - `Provider` trait pattern (already used in Lithos)
- [ADR 003: Domain Serialization](./003-domain-serialization.md) - Raw\* types and validation boundaries
- [ADR 006: Persistence Cache Infrastructure](./006-persistence-cache-infrastructure.md) - Zero-copy database reads

## Appendix A: FileSource Trait Design

```rust
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
```

**Design Rationale**:

- **`Send + Sync`**: Enables use in parallel processing (rayon)
- **Associated Error Type**: Allows each implementation to define appropriate error type
- **`read_to_string()`**: Returns UTF-8 validated text (binary files rejected by design)
- **`exists()`**: Enables incremental update checks without full file read
- **`list_files()`**: Supports bulk ingestion with pattern matching

**Implementations**:

1. **`FsFileSource`**: Standard filesystem access via `std::fs`
2. **`InMemoryFileSource`**: HashMap-based for testing (no disk I/O)
3. **Future**: `EmbeddedFileSource` (via `include_str!`), `HttpFileSource` (network)

## Appendix B: Application Service Pattern

```rust
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
    /// Ingest a single schema file.
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
        self.command.save_with_metadata(&schema, &Default::default())?;

        Ok(schema.id())
    }
}
```

**Design Rationale**:

- **Generic over CQRS ports**: Supports any Query/Command implementation (redb, postgres, in-memory)
- **Explicit workflow steps**: Each phase clearly separated with comments
- **Partial failure tolerance**: `ingest_directory()` continues on individual file errors
- **Instrumentation**: Tracing spans at each step for observability
- **Testability**: Inject fake `FileSource` and fake CQRS ports

## Appendix C: Anti-Patterns to Avoid

### Anti-Pattern 1: File I/O in CQRS Ports

```rust
// ❌ NEVER DO THIS
pub trait Command {
    fn load_from_file(&self, path: &Path) -> Result<Schema, Error>;
    fn save(&self, schema: &Schema) -> Result<(), Error>;
}
```

**Why Wrong**: Port does file I/O AND database operations. Violates single responsibility. Impossible to test independently.

---

### Anti-Pattern 2: Bypassing Validation Boundaries

```rust
// ❌ NEVER DO THIS
fn load_schema_file(path: &Path) -> Result<Schema, Error> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)  // Deserializes directly into domain type!
}
```

**Why Wrong**: Skips Raw → Domain validation. Invalid domain states representable. Poor error messages.

**Correct Approach**:

```rust
// ✅ DO THIS
fn load_schema_file(path: &Path) -> Result<Schema, Error> {
    let content = fs::read_to_string(path)?;
    let raw: RawSchema = serde_json::from_str(&content)?;  // Parse
    Schema::try_from(raw)  // Validate (TryFrom boundary)
}
```

---

### Anti-Pattern 3: Circular Dependencies

```rust
// ❌ NEVER DO THIS
// In db/adapters/schema.rs
use crate::schema::Schema;  // Infrastructure importing domain

impl SchemaAdapter {
    pub fn load_all_from_vault(&self, vault_path: &Path) -> Result<Vec<Schema>, Error> {
        // File I/O + database writes in adapter
    }
}
```

**Why Wrong**: Adapter orchestrates workflow (that's application layer's job). Infrastructure depends on domain (inverted dependency).

**Correct Dependency Flow**:

```
Application Layer (orchestrates workflows)
    ↓ uses
Domain Layer (schema::Query, schema::Command)
    ↓ uses
Port Traits (schema::ports::Query, schema::ports::Command)
    ↑ implemented by
Adapter Layer (schema::adapter::query, schema::adapter::command)
    ↓ uses
Infrastructure (db::Database)
```

## Appendix D: Implementation Phases

### Phase 1: Infrastructure Foundation (No Breaking Changes)

**Goal**: Add `FileSource` trait and parsers without breaking existing code.

**Tasks**:

1. Create `fs/source.rs` with `FileSource` trait
2. Create `fs/parsers.rs` with parsing functions
3. Add comprehensive tests

**Result**: New infrastructure available, old code still works.

---

### Phase 2: Ingestion Services (Parallel to Existing)

**Goal**: Create application services that orchestrate pipelines.

**Tasks**:

1. Create `SchemaIngestionService` in `application/services/`
2. Create `TemplateIngestionService`
3. Create `NoteIngestionService`

**Result**: Services exist but not yet used by CLI.

---

### Phase 3: CLI Integration (Breaking Changes)

**Goal**: Wire up services in CLI commands.

**Tasks**:

1. Refactor `lithos init` to use ingestion services
2. Remove direct file I/O from CLI

**Result**: CLI uses new architecture, old code paths removed.

---

### Phase 4: Documentation & Enforcement

**Goal**: Ensure pattern is documented and tested.

**Tasks**:

1. Update `project-context.md` with new rules
2. Add architecture tests to prevent regression
3. Document in this ADR

**Result**: Pattern is documented and enforced via CI.

## Appendix E: Migration Checklist

**Before Implementation**:

- [ ] Review Design Doc 013 (architecture research)
- [ ] Review Design Doc 014 (performance analysis)
- [ ] Understand current config ingestion (reference implementation)

**Phase 1 Tasks**:

- [ ] Implement `FileSource` trait in `fs/source.rs`
- [ ] Implement `FsFileSource` (filesystem)
- [ ] Implement `InMemoryFileSource` (testing)
- [ ] Add `parse_schema_file()` in `fs/parsers.rs`
- [ ] Add `parse_template_file()` in `fs/parsers.rs`
- [ ] Add `ParseError` type in `fs/error.rs`
- [ ] Write unit tests for all new infrastructure

**Phase 2 Tasks**:

- [ ] Implement `SchemaIngestionService`
- [ ] Implement `TemplateIngestionService`
- [ ] Implement `NoteIngestionService`
- [ ] Add `IngestionError` type
- [ ] Write integration tests for services

**Phase 3 Tasks**:

- [ ] Refactor `lithos init` command to use services
- [ ] Remove any direct file I/O from CLI
- [ ] Update CLI tests

**Phase 4 Tasks**:

- [ ] Add architectural tests (fail if file I/O in ports)
- [ ] Update `project-context.md` with rules
- [ ] Add criterion benchmarks for ingestion
- [ ] Document in this ADR (done)

**Verification**:

- [ ] All tests pass (`mise run test`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] Benchmarks meet performance targets
- [ ] Architecture tests prevent violations

## Appendix F: Real-World Research

Analysis of file ingestion patterns in 5 mature Rust projects reveals consistent architectural themes that validate our Service Layer approach.

### Project 1: Cargo (Package Manager)

**Pipeline**:
```
File System (Cargo.toml)
    ↓ TomlManifest::from_str (parsing)
TomlManifest (unvalidated)
    ↓ TomlManifest::to_real_manifest (validation)
Manifest (validated domain)
    ↓ PackageRegistry::insert (persistence)
In-Memory Registry
```

**Key Patterns**:
- File I/O in `cargo::util::toml::TomlManifest::read_file()`
- Parsing produces `TomlManifest` (serde-based, tolerant)
- Validation converts to `Manifest` (strict, invariants enforced)
- Persistence is separate concern (`PackageRegistry`, in-memory HashMap)

**Lessons**: Separate parsing from validation (`TomlManifest` vs `Manifest` mirrors our `Raw*` vs domain pattern). Explicit conversion boundaries via `to_real_manifest()`. Workflow coordination at application layer.

---

### Project 2: rustc (Compiler)

**Pipeline**:
```
Source Files (.rs)
    ↓ SourceFileLoader (file I/O)
SourceFile (raw text + metadata)
    ↓ Parser (syntax analysis)
AST (Abstract Syntax Tree)
    ↓ HIR lowering (validation + desugaring)
HIR (High-level IR, validated)
    ↓ THIR/MIR/Codegen
Machine Code
```

**Key Patterns**:
- File loading is separate phase (`SourceFileLoader`, `SourceMap`)
- Parsing produces AST (unvalidated structure)
- HIR lowering is validation boundary (type checking, name resolution)
- Persistence (incremental cache) happens after validation
- Query system (`rustc_query_system`) separates reads from transformations

**Lessons**: Parsing ≠ validation (AST accepts invalid programs, HIR only accepts valid ones). Incremental computation via query system memoization. File source abstraction for filesystem vs in-memory vs stdin. Cache invalidation tracks file changes.

---

### Project 3: Diesel (ORM Migrations)

**Pipeline**:
```
migration.sql files
    ↓ MigrationHarness::run_pending_migrations
Migration (parsed SQL)
    ↓ Connection::execute
Database schema changes
```

**Key Patterns**:
- File discovery via `MigrationSource` trait finds `.sql` files
- Parsing in `Migration::from_file()` (SQL text → Migration struct)
- Execution is separate trait (`Connection::execute_batch`)
- State tracking stored in `__diesel_schema_migrations` table

**Lessons**: `MigrationSource` trait allows filesystem, embedded, or custom sources (exactly like our `FileSource`). Two-phase execution (discover → parse → execute). Idempotency via version tracking.

---

### Project 4: config-rs / figment (Configuration)

**Pipeline**:
```
Multiple sources (files, env, defaults)
    ↓ Provider trait (abstraction)
Figment (merged configuration)
    ↓ extract::<T>() (deserialization)
Typed config struct
```

**Key Patterns**:
- `Provider` trait abstracts over file, environment, command-line
- Merging happens in-memory (not tied to specific source)
- Extraction deserializes into typed structs (validation via serde or custom)
- No persistence layer (read-only)

**Lessons**: Source abstraction via traits. Composition over inheritance (Figment chains providers via `.merge()`). Lazy evaluation (files not read until `.extract()` called). **Already in Lithos**: We use this pattern in `config::ingest::build_merged_raw()` as reference implementation.

---

### Project 5: tree-sitter (Parser Generator)

**Pipeline**:
```
Source text (files)
    ↓ Parser::parse
Tree (AST)
    ↓ Query::captures (traversal)
Match results
```

**Key Patterns**:
- Parser state separate from source text
- Tree holds references to source text (zero-copy)
- Queries operate on trees, not files
- Incremental re-parsing of changed regions only

**Lessons**: Zero-copy parsing (tree references source via `&str` slices). Incremental updates track changed ranges. Separation of concerns (parsing produces trees, queries operate on trees). Lifecycle independence (Parser ≠ Tree ≠ Query).

---

### Cross-Project Analysis

| Project     | File I/O Layer       | Parsing Layer      | Validation Layer  | Persistence Layer     |
| ----------- | -------------------- | ------------------ | ----------------- | --------------------- |
| Cargo       | `fs::read_to_string` | `TomlManifest`     | `Manifest`        | `PackageRegistry`     |
| rustc       | `SourceFileLoader`   | `Parser → AST`     | `HIR::lower`      | Query system (cache)  |
| Diesel      | `MigrationSource`    | `Migration::parse` | SQL validation    | `Connection::execute` |
| config-rs   | `Provider::data`     | serde              | Type extraction   | N/A (read-only)       |
| tree-sitter | `fs::read_to_string` | `Parser::parse`    | N/A (syntax only) | N/A (in-memory)       |

**Common Themes Across All Projects**:

1. **File I/O is a separate concern** from parsing, validation, and persistence
2. **Trait-based source abstraction** enables testing without filesystem (Diesel's `MigrationSource`, config-rs's `Provider`, rustc's `SourceFileLoader`)
3. **Multi-stage pipeline** with explicit boundaries (`File → Raw → Validated → Stored`)
4. **Application layer coordinates** the workflow, not infrastructure
5. **Incremental updates** track changes and minimize re-processing (rustc's query system, cargo's fingerprints, tree-sitter's incremental parsing)

**Validation of Our Approach**: The Service Layer pattern with `FileSource` trait matches patterns proven in production Rust systems handling millions of files. Our `Raw*` → Domain validation mirrors Cargo's `TomlManifest` → `Manifest` and rustc's `AST` → `HIR` transformations.
