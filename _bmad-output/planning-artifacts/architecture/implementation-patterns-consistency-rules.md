# Implementation Patterns & Consistency Rules

## Pattern Categories Defined

**Critical Conflict Points Identified:** 30+ areas where AI agents could make different choices in async Rust CLI applications with hexagonal architecture, CQRS, and event-driven patterns.

## Naming Patterns

**Rust Naming Conventions:**

- **Modules & Files:** `snake_case` (e.g., `vault_indexer.rs`, `frontmatter_service.rs`)
- **Functions & Variables:** `snake_case` (e.g., `execute_template`, `vault_path`)
- **Structs & Enums:** `PascalCase` (e.g., `Note`, `DomainError`, `TemplateEngine`)
- **Traits:** `PascalCase` ending with trait name (e.g., `CacheWriter`, `VaultReader`) or `Port` (e.g., `StoragePort`)
- **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `MAX_VAULT_SIZE`, `DEFAULT_TIMEOUT`)
- **Crate Names:** `snake_case` matching directory (e.g., `lithos-domain`, `lithos-app`)
- **Test Functions:** `snake_case` with `test_` prefix (e.g., `test_execute_template`)
- **Macros:** `snake_case` (e.g., `my_macro!`)

**API Contract Naming:**

- **Trait Methods:** `snake_case` with clear action verbs (e.g., `persist_note`, `find_templates`)
- **Port Traits:** Descriptive names ending with `Port` (e.g., `CacheWriterPort`, `VaultReaderPort`)
- **DTO Structs:** `PascalCase` ending with `Dto` (e.g., `VaultFileDto`, `FileDatesDto`)
- **Event Names:** `PascalCase` with past tense (e.g., `NoteIndexed`, `TemplateExecuted`)

## Structure Patterns

**Workspace Organization:**

- **Crate Separation:** Strict hexagonal boundaries - domain depends on nothing, app depends only on domain, adapters depend on domain + external crates, cli depends on app + adapters
- **Module Organization:** Within crates, use `mod.rs` for submodules, keep related functionality together; avoid deep nesting (max 3 levels)
- **Test Placement:** Unit tests in same file as implementation (`#[cfg(test)]`), integration tests in `tests/` directory at crate root, performance tests in `benches/`
- **Binary Organization:** CLI crate should be minimal, delegating to library crates

**File Structure Standards:**

- **Domain Crate:** `src/`, `src/ports/`, `src/services/` (for pure domain services)
- **App Crate:** `src/commands/`, `src/queries/`, `src/vault/`, `src/schema/`, `src/template/`
- **Adapters Crate:** `src/api/`, `src/spi/`, `src/dto/`
- **CLI Crate:** `src/main.rs` only, all logic in other crates
- **Common Patterns:** Group related items, use `prelude.rs` for common imports, keep files focused

## Format Patterns

**Error Handling Standards:**

- **Domain Errors:** `thiserror::Error` for typed error enums with descriptive messages and `#[from]` for conversions
- **Context Addition:** `anyhow::Result` for ergonomic error chaining in application code with `context!` macro
- **CLI Output:** User-friendly error messages with actionable guidance using `color-eyre` for pretty printing
- **Logging:** `tracing` with structured spans, consistent log levels, and subscriber setup in CLI crate
- **Panic Avoidance:** Never use `unwrap()`, `expect()`, `panic!()` in library code; prefer `Result`

**Async Patterns:**

- **Runtime:** Tokio as the async runtime throughout all crates with `#[tokio::main]` in CLI
- **Trait Methods:** Use `async_trait` for async trait methods with `Send + Sync` bounds
- **Error Propagation:** `?` operator for clean error bubbling, `map_err` for context addition
- **Cancellation:** Accept `CancellationToken` in long-running operations with `select!` for graceful shutdown
- **Channels:** Use `tokio::sync::mpsc` for event buses, bounded channels to prevent memory issues
- **Futures:** Prefer `async fn` over manual `Future` implementations, use `async move` for owned data

**Documentation Standards:**

- **Item Documentation:** Use `///` for functions, structs, traits, and other items
- **Module Documentation:** Use `//!` for module-level documentation
- **Examples:** Include code examples in documentation where helpful
- **Error Documentation:** Document error conditions and panic scenarios
- **Formatting:** Use markdown formatting in doc comments
- **Links:** Reference related items and external documentation

## Communication Patterns

**Event System Standards:**

- **Event Naming:** `PascalCase` with past tense (e.g., `NoteIndexed`, `VaultIndexingCompleted`)
- **Event Payloads:** Immutable structs with clear field names and derive macros for serialization
- **Event Bus:** Async channels with bounded capacity and weak subscriber references to prevent leaks
- **Subscriber Patterns:** Handler functions with proper error isolation and logging

**Inter-Crate Communication:**

- **Dependency Injection:** Constructor injection of port implementations with `Arc<dyn Trait>` for shared ownership
- **Trait Objects:** Use `Box<dyn Trait>` for runtime polymorphism with `Send + Sync` bounds
- **Type Safety:** Leverage Rust's compile-time guarantees over runtime checks
- **Configuration Passing:** Use `Arc<Config>` for shared immutable configuration across crates

## Process Patterns

**Testing Standards:**

- **Unit Tests:** Pure domain logic with no external dependencies using `#[cfg(test)]`; test both success and error cases
- **Integration Tests:** Cross-crate testing with test adapters using `tokio::test`; test real implementations
- **Performance Tests:** Criterion benchmarks for 500ms targets with statistical analysis
- **Async Testing:** `tokio::test` for concurrent operation testing with proper setup/teardown
- **Mocking:** Test doubles for ports using `mockall` crate or manual implementations
- **Property-Based Testing:** `proptest` for edge case discovery in domain logic
- **Test Organization:** Group tests by functionality, use descriptive names, avoid flaky tests; include doc tests

**Configuration Management:**

- **Hierarchy:** CLI flags > Environment variables > Config file > Defaults with precedence documentation
- **Validation:** Compile-time validation using Serde derive with custom validators
- **Singleton Pattern:** `OnceCell` for lazy initialization with thread-safe access
- **Hot Reload:** File watching with `notify` crate for development (optional)
- **Environment-Specific:** Different config files for dev/staging/prod

**Build & Development:**

- **Cargo Profiles:** Separate debug/release profiles with appropriate optimizations
- **Linting:** Clippy with all pedantic and nursery lints enabled; deny complexity violations
- **Formatting:** Rustfmt with standard configuration and import sorting
- **CI/CD:** GitHub Actions with cross-platform testing, coverage reporting, and security auditing
- **Development Tools:** `cargo-watch` for auto-rebuild, `cargo-expand` for macro debugging, **mise 2026.1.0** for tool version management and task execution via `.mise/tasks/`
- **Pre-commit Hooks:** Use pre-commit framework to run clippy, rustfmt, and tests before commits for maximum visibility and clean git history

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
async fn test_vault_indexing_success() {
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
