---
title: "Implementation Patterns & Consistency Rules"
description: "Development patterns, naming conventions, and consistency rules for Lithos implementation"
author: "Jack"
date: "2026-01-23"
last_updated: "2026-01-23"
section: "Implementation Standards"
---

# Implementation Patterns & Consistency Rules

## Pattern Categories Defined

**Critical Conflict Points Identified:** 30+ areas where AI agents could make different choices in async Rust CLI applications with hexagonal architecture, CQRS, and event-driven patterns.

## Naming Patterns

**Rust Naming Conventions:**

- **Crate/Package Names:** Cargo *package* names are `kebab-case` (e.g., `lithos-core`, `lithos-cli`). In Rust code, the crate import path is `snake_case` (e.g., `lithos_core`).
- **Modules & Files:** `snake_case` (e.g., `vault_indexer.rs`, `frontmatter_service.rs`)
- **Functions & Variables:** `snake_case` (e.g., `execute_template`, `vault_path`)
- **Structs & Enums:** `PascalCase` (e.g., `Note`, `DomainError`, `TemplateEngine`)
- **Traits:** `PascalCase` ending with trait name (e.g., `CacheWriter`, `VaultReader`) or `Port` (e.g., `StoragePort`)
- **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `MAX_VAULT_SIZE`, `DEFAULT_TIMEOUT`)
- **Test Functions:** `snake_case` with descriptive names that read like a sentence (e.g., `returns_blob_when_larger_than_b`). Avoid generic `test_*` prefixes; use `mod <unit_under_test>` to group related tests when helpful.
- **Macros:** `snake_case` (e.g., `my_macro!`)

**Naming Discipline (Semantic Consistency):**

- **Say what it means:** Prefer names that make roles and direction obvious (e.g., `needle`/`haystack`, `source`/`destination`, `before`/`after`).
- **Consistent word order:** Pick a project-wide pattern and stick to it. If most functions read as `verb_noun`, new APIs should follow `verb_noun` unless there is a strong reason not to.
- **Be concise (but not cryptic):** Avoid nonstandard abbreviations and “inside jokes”; shorten only when meaning remains obvious.
- **Use simple, correct words:** Prefer the smallest set of small words that preserve meaning; avoid terms that can be read two ways.
- **Unify concept names:** One term per concept. If we choose “vault” (not “repo”/“workspace”) for the user’s note collection, use “vault” consistently in APIs, docs, and modules.
- **Avoid type-noise in names:** Don’t encode types (e.g., `*_str`, `*_vec`) unless it disambiguates two values with the same conceptual meaning.

**Pattern-Match Variable Naming:**

- When destructuring, keep field names whenever possible, and avoid renaming to single letters.
- Use struct field shorthand to preserve the domain vocabulary.

✅ Prefer:

```rust
if let Some(response) = response { /* ... */ }
let Self { name, path } = self;
match state {
    State::Reading(file) => { /* ... */ }
    State::Evaluating { workload, .. } => { /* ... */ }
}
```

⚠️ Avoid:

```rust
if let Some(r) = response { /* ... */ }
let Self { name: some_name, path: name } = self;
match state {
    State::Reading(data_source) => { /* ... */ }
    State::Evaluating { workload: to_eval, .. } => { /* ... */ }
}
```

**Pattern Matching Discipline:**

- **Match exhaustively to draw attention:** Prefer destructuring structs/enums to make it obvious which fields are considered. This helps the compiler alert us when structures evolve.
- **Don’t pattern-match references:** Prefer explicit dereferencing (`|x| *x`) over `|&x| x`.
- **Avoid numeric tuple indexing:** Prefer destructuring into named values (`let (x, y) = point`) over `.0`/`.1`.
- **Avoid pattern-matching in `fn` parameters:** Unpack on the first line inside the function; keep signatures clean.

✅ Prefer:

```rust
// Exhaustive destructuring (future-proofing)
let Self { name, path, .. } = self;

// Explicit deref, not matching references
let values: Vec<_> = refs.iter().map(|x| *x).collect();

// Name tuple elements
let (x1, y1) = point1;
let (x2, y2) = point2;

// Keep function signature clean
fn new(config: ServerConfig) {
    let ServerConfig { db_path, working_path } = config;
    // ...
}
```

⚠️ Avoid:

```rust
// Implicit access hides evolution of the type
let (name, path) = (&self.name, &self.path);

// Reference pattern matching obscures deref
let values: Vec<_> = refs.iter().map(|&x| x).collect();

// Tuple indexing loses semantics
let gradient = (point2.1 - point1.1) / (point2.0 - point1.0);

// Pattern matching in parameters adds noise to signatures
fn new(ServerConfig { db_path, working_path }: ServerConfig) {
    // ...
}
```

**Generic Type Parameter Naming:**

- Generic type parameters should be single-letter to avoid looking like concrete types (common: `T`, `E`, `K`, `V`).

**Lifetime Parameter Naming:**

- Use lifetimes as “documentation”: pick names derived from what is being borrowed (e.g., `'db`, `'tx`, `'bytes`, `'src`).
- Avoid `'a`/`'b` unless there is a compelling reason; avoid numbers in lifetime names.

✅ Prefer:

```rust
pub struct ArchivedGuard<'db, T> { /* ... */ }
pub struct SchemaView<'bytes> { /* ... */ }
```

⚠️ Avoid:

```rust
pub struct ArchivedGuard<'a, T> { /* ... */ }
pub struct SchemaView<'a> { /* ... */ }
```

**API Contract Naming:**

- **Trait Methods:** `snake_case` with clear action verbs (e.g., `persist_note`, `find_templates`)
- **Port Traits:** Descriptive names ending with `Port` (e.g., `CacheWriterPort`, `VaultReaderPort`)
- **DTO Structs:** Prefer role-based names over type-suffixes (e.g., `VaultFile`, `VaultFileRecord`, `CreateNoteRequest`). If `Dto` is used, reserve it for strict boundary/wire types (CLI/adapters/serde), not domain/core.
- **Event Names:** `PascalCase` with past tense (e.g., `NoteIndexed`, `TemplateExecuted`)

**Builder Naming:**

- If a builder for `MyType` is provided, expose `MyType::builder() -> MyTypeBuilder` and `MyTypeBuilder::build() -> Result<MyType, _>`.
- Builder setters should read naturally and match field names where possible.

## Structure Patterns

**Workspace Organization:**

- **Crate Separation:** `lithos-core` (Logic + Infra) vs `lithos-cli` (Driver).
- **Module Organization:** Within crates, use `<module>.rs` + `<module>/` folder. NO `mod.rs`.
- **Test Placement:** Unit tests in same file (`#[cfg(test)]`), integration tests in `tests/`.
- **Binary Organization:** CLI crate delegating to `lithos-core`.

**File Structure Standards:**

- **Lithos Core:** `src/lib.rs`, `src/db.rs`, `src/fs/`, `src/<context>.rs`, `src/<context>/` (errors/events co-located).
- **Lithos CLI:** `src/main.rs`, `src/commands/`.
- **Common Patterns:** Group related items, keep files focused.

## Format Patterns

**Error Handling Standards:**

- **Core Errors:** `thiserror::Error` for typed, co-located error enums (e.g. `note::Error`).
- **Context Addition:** `anyhow::Result` only in `main.rs` if prototyping; otherwise `miette`.
- **CLI Output:** `miette` for user-facing errors with help/labels.
- **Logging:** `tracing` with structured spans.
- **Panic Avoidance:** Never use `unwrap()`, `expect()` in library code.

**Async Patterns:**

- **Sync-First:** Core domain logic and file I/O must be synchronous.
- **Async at Edge:** `lithos-cli` uses `tokio::main`.
- **Bridging:** Use `tokio::task::spawn_blocking` for concurrent core operations.
- **No Async Traits:** Do NOT use `#[async_trait]` in `lithos-core`.

**Documentation Standards:**

- **Item Documentation:** Use `///` for public items.
- **Module Documentation:** Use `//!` at top of `<context>.rs`.
- **Examples:** Include code examples for public APIs.

**Serialization Patterns:**

- **Feature Flag:** Use `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.
- **Optional:** Serde is optional for domain types, required for DTOs/Config.
- **Zero-Copy:** Use `rkyv` for performance-critical storage types.

## Communication Patterns

**Event System Standards:**

- **Event Naming:** `PascalCase` with past tense (e.g., `NoteIndexed`).
- **Co-location:** Events defined in `<context>/events.rs`.
- **Dispatch:** Deferred dispatch via `UnitOfWork` or simple callbacks (Phase 1).

**Inter-Module Communication:**

- **Direct Calls:** `lithos-cli` calls `lithos-core` static methods directly.
- **Database:** Passed as `&Database` reference to domain methods.
- **No Traits:** Unless required for testing/mocking.

**Database Access Rules:**

- **Concrete Type:** Use `lithos_core::db::Database` directly.
- **Zero-Copy First:** Prefer `db.get_archived()` for hot paths.
- **Batch Operations:** Use `db.batch_write()` for bulk updates.

**CQRS Naming Conventions:**

- **Queries:** `find_*`, `get_*`, `list_*`, `count_*`.
- **Commands:** `save`, `delete`, `update`.

## Process Patterns

**Testing Standards:**

- **Unit Tests:** `#[cfg(test)]` in same file.
- **Integration Tests:** `tests/integration/` (CLI -> Core -> DB).
- **Architecture Tests:** `tests/arch/` (boundary enforcement).
- **Benchmarks:** `benches/` (zero-copy validation).

**Configuration Management:**

- **Hierarchy:** CLI args > Config file > Defaults.
- **Validation:** Serde validation.

**Build & Development:**

- **Mise:** Task runner (`mise run verify`, `mise run build`).
- **Hooks:** Pre-commit enforcement.

**Clippy Complexity Limits:**

- **Cyclomatic Complexity:** `clippy::cognitive_complexity` threshold set to 15 (warn) and 25 (deny) to prevent overly complex functions
- **Function Length:** `clippy::too_many_lines` with limit of 100 lines per function
- **Arguments:** `clippy::too_many_arguments` with max 7 arguments
- **Nesting:** `clippy::nested_if_else` and `clippy::too_many_nested_loops` limits enforced
- **Code Quality:** Deny `clippy::unwrap_used`, `clippy::expect_used`, `clippy::todo`, `clippy::unimplemented`, `clippy::dbg_macro`
- **Performance:** Enable `clippy::inefficient_to_string`, `clippy::redundant_clone`, `clippy::needless_collect`
- **Style:** Enforce `clippy::implicit_return`, `clippy::single_match_else`, `clippy::redundant_else`

## Enforcement Guidelines

**All AI Agents MUST:**

- Follow established naming conventions without exception
- Maintain hexagonal architecture boundaries (no domain → adapters dependencies)
- Use async/await consistently throughout the codebase with proper error handling
- Implement comprehensive error handling with typed errors and context
- Write tests for all public APIs and critical paths including async operations
- Document public traits and complex business logic with examples following Rust doc standards
- Use tracing for all logging with structured spans and consistent levels
- Keep cyclomatic complexity under 15 and cognitive complexity under 25 per function
- Never use `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in production code
- Run clippy on all code with complexity limits enforced before commits via pre-commit hooks

**Pattern Enforcement:**

- **Pre-commit Hooks:** Run clippy, rustfmt, and tests before commits to maintain clean git history and catch issues early
- **Code Reviews:** Automated checks for naming violations, dependency rules, architectural boundaries, and complexity metrics; manual review for logic and API design
- **CI Pipeline:** Clippy with complexity limits, rustfmt, and custom lint enforcement with failure on violations; require green CI for merges
- **Architecture Tests:** Integration tests verifying crate boundaries and hexagonal rules
- **Documentation:** Pattern violations documented in commit messages with remediation steps
- **Quality Gates:** Minimum test coverage (80%), no clippy warnings, performance regression checks, security audit passing

**Advanced Enforcement:**

- **Dependency Analysis:** Use `cargo deny` to prevent unwanted dependency introductions
- **Security Auditing:** Regular `cargo audit` runs to catch vulnerabilities
- **Performance Regression:** Automated benchmark comparisons to prevent performance degradation
- **Code Coverage:** Minimum coverage thresholds enforced in CI with `tarpaulin`
- **Style Consistency:** Automated import sorting and formatting checks; use `cargo fmt --check` in CI

## Pattern Examples

**Good Examples:**

````rust
/// A note in the vault with its metadata and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Vault-relative path serving as the unique identifier
    pub path: String,
    /// Parsed frontmatter metadata
    pub frontmatter: Frontmatter,
}

impl Note {
    /// Creates a new note with validation.
    ///
    /// # Errors
    /// Returns `DomainError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_domain::Note;
    /// let note = Note::new("path.md".to_string(), frontmatter)?;
    /// ```
    pub fn new(path: String, frontmatter: Frontmatter) -> Result<Self, DomainError> {
        if path.is_empty() {
            return Err(DomainError::InvalidPath);
        }
        Ok(Self { path, frontmatter })
    }
}

#[async_trait]
pub trait VaultWriterPort: Send + Sync {
    /// Persists a note to the vault storage.
    ///
    /// # Errors
    /// Returns `DomainError` if persistence fails.
    async fn persist_note(&self, note: Note) -> Result<(), DomainError>;
}

pub struct VaultIndexerService {
    vault_writer: Arc<dyn VaultWriterPort>,
    event_bus: Arc<dyn EventBus>,
}

impl VaultIndexerService {
    /// Indexes the vault and publishes completion events.
    ///
    /// This function maintains low complexity by delegating to helper methods.
    pub async fn index_vault(&self) -> Result<IndexStats, DomainError> {
        self.event_bus.publish(DomainEvent::VaultIndexingStarted).await?;

        let stats = self.perform_indexing().await?;

        self.event_bus.publish(DomainEvent::VaultIndexingCompleted { stats: stats.clone() }).await?;

        Ok(stats)
    }

    async fn perform_indexing(&self) -> Result<IndexStats, DomainError> {
        Ok(IndexStats::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Fired when vault indexing begins
    VaultIndexingStarted,
    /// Fired when vault indexing completes
    VaultIndexingCompleted { stats: IndexStats },
    /// Fired when a note is indexed
    NoteIndexed { note_path: String, indexed_at: DateTime<Utc> },
    /// Fired when a template is executed
    TemplateExecuted { template_id: String, success: bool },
}

#[tokio::test]
async fn vault_indexing_succeeds() {
    let mock_writer = Arc::new(MockVaultWriter::new());
    let mock_bus = Arc::new(MockEventBus::new());
    let service = VaultIndexerService::new(mock_writer, mock_bus);

    let result = service.index_vault().await;
    assert!(result.is_ok());

    let stats = result.unwrap();
    assert_eq!(stats.total_notes, 0);
}
````

**Anti-Patterns:**

- Functions exceeding 15 cyclomatic complexity or 100 lines
- Using `unwrap()` or `expect()` in production code
- Deeply nested control structures
- Inconsistent naming or missing documentation
- Blocking operations in async functions without `spawn_blocking`
- Tests that don't cover error cases or async behavior
- Missing doc examples for public APIs
- Not running clippy or ignoring warnings

**Resource References:**

- Rust Official Documentation
- Clippy Lints Reference
- Tokio Async Patterns
