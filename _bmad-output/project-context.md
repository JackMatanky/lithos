---
project_name: 'lithos'
user_name: 'Jack'
date: '2026-01-23'
sections_completed: ['technology_stack', 'architectural_integrity', 'language_rules', 'framework_rules', 'testing_rules', 'quality_rules', 'workflow_rules', 'anti_patterns']
status: 'complete'
rule_count: 68
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss. Rules are mandatory unless marked as 'prefer' or 'avoid'. When rules conflict, choose the more restrictive option._

---

## Technology Stack & Versions

### Core Runtime
- **Rust 1.92+**: Mandatory for memory safety patterns and zero-cost abstractions.
- **Tokio 1.49**: Async runtime with **'full' features** enabled for concurrent vault operations and CLI responsiveness.
  - *Safety Invariant*: NEVER perform blocking I/O or heavy CPU tasks (e.g., Redb transactions) inside an async fn without `tokio::task::spawn_blocking`.

### Data & Persistence
- **Redb 3.1 & rkyv 0.8**: Embedded ACID KV storage with **zero-copy deserialization** for high-frequency lookups.
  - *Safety Invariant*: All storage types MUST use `rkyv` validation (`bytecheck`) before access to prevent memory corruption from malformed vault data.
- **UUID 1.19 (v7)**: Time-ordered, sortable unique identifiers for note identity.

### Application Engine
- **MiniJinja 2.14**: VM-based template engine for user-defined Markdown templates.
- **miette 7.6**: High-fidelity terminal diagnostics and error reporting.
- **Figment 0.10**: Two-tier configuration (Global -> Vault).
- **thiserror 2.0 & anyhow 1.0**: Structured error definition and ergonomic context chaining.

### Tooling & Orchestration
- **mise**: **Primary and authorized entry point** for all tasks and tool version management (via `mise.toml`).
  - *Safety Invariant*: All commands MUST be executed via `mise run <task>` to ensure toolchain parity across environments.
  - **Available Tasks Reference:**
    - **Quality Gates:** `quality` (fmt+lint+adr:validate), `verify` (full gates + tests), `fmt`, `lint`, `deny`
    - **Testing:** `test` (unit+integration+e2e, alias: `t`), `test:unit`, `test:integration`, `test:e2e`, `test:coverage`, `test:watch`, `test:burn-in`, `test:changed`, `test:unit:*` (module-specific)
    - **CI/CD:** `ci` (pipeline simulation), `verify` (alias: `v`)
    - **Development:** `build`, `clean`, `doc`, `dev-setup`, `test:bench`
    - **ADR Management:** `adr:validate`, `adr:metrics`
- **pre-commit**: Mandatory quality gate for linting, formatting, and complexity checks before every commit. Bypassing hooks is strictly prohibited.

### Code Quality & Standards
- **clippy**: Enforces **cognitive complexity < 25** and custom quality lints (via `clippy.toml`).
- **rustfmt**: Enforces project-wide formatting and import sorting (via `rustfmt.toml`).
- **nextest**: Optimized test runner for high-performance concurrent execution.
- **tarpaulin**: Code coverage analysis tool targeting **80%+ coverage**.

### CI/CD
- **GitHub Actions**: Automated pipeline for cross-platform testing, linting, and release builds.

---

## Critical Implementation Rules

### Architectural Integrity
- **Dependency Flow Architecture:**
  - **Single Core Crate:** `lithos-core` contains all business logic and infrastructure. Dependencies flow:
    - Pure Infrastructure (`db/`, `fs/`) → All Contexts
    - Cross-Cutting Business Rules (`config/`) → Business Contexts
    - Business Contexts (`note/`, `schema/`, `template/`) → CLI
  - **Context Isolation:**
    - Business contexts (note, schema, template) MUST NOT import each other.
    - Business contexts MAY depend on config context (user-configurable rules) and pure infrastructure (generic utilities).
  - **Boundaries:** Use `pub(crate)` to enforce internal boundaries. `pub` is reserved for the crate's external API (used by `lithos-cli`).
- **Port-Based CQRS:**
  - **Port Traits:** Each context defines split storage capabilities via `<Context>::ports::Query` and `<Context>::ports::Command` traits (e.g., `schema::ports::QueryPort`, `schema::ports::CommandPort`) with GATs for zero-copy reads.
  - **Generic CQRS:** Command/Query types are generic over respective ports (e.g., `Query<Q: SchemaQueryPort>`, `Command<C: SchemaCommandPort>`).
  - **Concrete Adapters:** Infrastructure provides concrete implementations (e.g., `RedbSchemaQueryAdapter`, `RedbSchemaCommandAdapter`).
  - **Type Aliases:** Use type aliases for ergonomics (e.g., `RedbSchemaQuery::new_redb(&db).find(...)`).
  - **Port Split Benefits:** Read-only test fakes don't implement writes, prevents interface bloat, enables future backend flexibility.
- **Naming Convention:**
  - Queries: `find_*`, `get_*`, `list_*`, `count_*`
  - Commands: `save`, `delete`, `update`, `create`
  - Ports: `<Context>Store` (e.g., `NoteStore`)
- **Sync-First Execution Model:**
  - **Default to Sync:** Core domain logic and file I/O must be synchronous.
  - **Async at Edges:** Use `async` ONLY for LSP server, network I/O, or explicit concurrency (e.g., parallel indexing).
  - **Bridge:** Use `tokio::task::spawn_blocking` to bridge sync core logic into async contexts (CLI/LSP).
- **Unit of Work (Transactional Context):**
  - **Batch Operations:** For bulk updates (indexing), use `db.batch_write()` which handles transaction lifecycle and durability settings.
  - **Deferred Dispatch:** Domain events must be staged and dispatched only after successful transaction commit.
- **Dependency Injection:** Favor **Generics (`impl Trait` or `&ConcreteType`)** for internal adapter-level utility functions to maximize static dispatch performance. Explicit constructor injection is optional for core types.

### Language-Specific Rules (Rust)

#### Idiomatic Patterns
- **Type-Driven Design:** Enforce invariants at compile time. Use private fields with validated constructors (`new() -> Result<Self>`) and newtype wrappers for domain constraints (`struct NoteId(Uuid)`). Make illegal states unrepresentable.
- **Standard Traits:** Always derive `Debug`, `Clone`, and `PartialEq`. Use `Default` for configurations.
- **Conversion Traits:** Mandate `From/Into` (infallible) and `TryFrom/TryInto` (fallible). AI must use these instead of ad-hoc `to_x()` methods.
- **Collection Safety:** Use the `HashMap::entry` API for updates. Use `.get()` or `.first()` instead of index-based access (`[0]`).
- **Exhaustive Matching:** Use `match` for Enums with `#[non_exhaustive]` on domain types. Prohibit `_ => {}` catch-alls for domain logic to ensure new variants are handled.

#### AI Pitfall Protections
- **Path Protocol**: Use `PathBuf` (owned) or `&Path` (borrowed) for all file paths. NEVER use `String` for paths. Strictly avoid std::env::current_dir and std::fs::canonicalize due to platform inconsistencies; always use Figment-managed absolute paths for reliability. Exception: In error recovery scenarios, use canonicalize only if Figment paths are invalid.
- **Async Resource Safety:**
  - In `async` contexts, ONLY use `tokio::fs`. Use `spawn_blocking` for `std::fs` or heavy CPU tasks.
  - **Concurrency Throttling:** Use tokio::sync::Semaphore to limit concurrent I/O, especially for large-scale operations (e.g., vault indexing). Set permits based on system limits, e.g., Semaphore::new(100) for file reads to avoid exceeding OS file descriptors.
- **Lock Discipline:** NEVER hold a std::sync::MutexGuard across an .await, as it blocks the async runtime thread. If state must persist across awaits, use tokio::sync::Mutex, which is async-aware and releases locks during suspension.
- **Numeric Safety:** Prohibit 'as' casting; use .try_into().context("...") to prevent silent truncation and handle errors gracefully. Avoid .expect() in production—propagate errors instead.

#### Persistence & Performance
- **rkyv Requirements:** Domain types MUST derive `Archive`, `Serialize`, `Deserialize`, and `CheckBytes` for zero-copy database operations. `Stored*` types used only when domain shape inefficient.
- **Three-Shape Model (ADR 003):**
  - **`Raw*` (serde):** Unvalidated input from filesystem for tolerant parsing
  - **Domain (rkyv + serde feature-gated):** Validated entities with rkyv derives, used throughout application
  - **`Stored*` (rkyv, optional):** Storage-optimized representation, only when domain shape causes performance issues
  - **Default Strategy:** Store domain types directly; introduce `Stored*` only when profiling reveals inefficiency
- **Zero-Copy Boundary:** Port traits use GATs (`type Archived<'a>`) to enable closure-based archived reads without leaking transaction lifetimes.
- **Note Identity:** Use **UUID v7** for primary keys in Redb to ensure time-ordered insertion and logical stability during file renames.
- **Memory Strategy:** Favor `Box<str>` or `SmolStr` for immutable identifiers. Use `Cow<'a, str>` for metadata frequently read from storage buffers.

#### Error & Diagnostic Standards
- **Layered Errors:** `thiserror` in `domain`, `anyhow` + `.context()` in `app/adapters`, and `miette` in `cli`.
- **Rich Diagnostics:** All `miette` errors MUST include a **Help** label and, where possible, a **Source Snippet** pointing to the exact line/offset in the vault file (via `pulldown-cmark` offsets).
- **Main Loop Resilience:** Implement a `catch_unwind` or a top-level `Result` handler in the CLI main loop to prevent a single bad template or file from crashing the entire process.
- **No-Panic Policy:** NEVER use `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in production code.

#### Observability & Tracing Standards
- **Mandatory Instrumentation:** ALL public methods in `app` and `adapters` layers MUST use `#[tracing::instrument]` per architecture.md FR40 (audit logging).
- **Skip Large Parameters:** Always skip `self`, large values, and sensitive data: `#[tracing::instrument(skip(self, config, value))]`
- **Operation Fields:** Add semantic fields to spans: `fields(operation = "get_schema", schema_name)`
- **Appropriate Levels:**
  - `level = "info"` for commands (load_all, commit, backup, refresh)
  - `level = "debug"` for queries, cache operations, individual steps
  - `level = "warn"` for errors, fallbacks, rollbacks, slow operations
  - `level = "error"` for corruption, critical failures
- **Structured Logging Patterns:**
  - Cache hits: `tracing::debug!(key, cache_layer = "memory", "Cache hit")`
  - Cache misses: `tracing::debug!(key, cache_layer = "disk", "Cache miss")`
  - Errors: `tracing::error!(?error, context, "Operation failed")`
  - Warnings: `tracing::warn!(?error, fallback, "Degraded mode activated")`
  - Info events: `tracing::info!(count, duration_ms, "Operation completed")`
- **Span Attributes:** Include contextual data: operation, entity_name, count, duration_ms, success (bool), cache_hit (bool)
- **Slow Operation Detection:** Log warnings for operations exceeding NFR targets (e.g., queries >500ms, cache operations >100ms)
- **Testing Instrumentation:** Use `tracing-test` crate to verify spans are emitted in integration tests

### Framework-Specific Rules

#### Tokio (Async Runtime & Safety)
- **Executor Health:** NEVER block an async thread for more than 10ms (Tokio's default fairness threshold) to maintain responsiveness. Use tokio::time::timeout or profiling to identify blocking calls like std::fs operations.
- **Deadlock Prevention:** Strictly prohibit holding a `std::sync::MutexGuard` across an `.await`.
- **Shutdown Resilience:** All actors must use `tokio::select!` to listen for a global `broadcast::Receiver` shutdown signal. On shutdown, actors MUST complete the current atomic transaction before exiting.
- **Resource Throttling:** Use a `tokio::sync::Semaphore` to limit concurrent vault file reads to prevent exceeding OS file descriptor limits.

#### Redb & rkyv (Persistence & Performance)
- **Transaction Locality:** Wrap `Redb` transactions in a custom RAII guard. Ensure Write Transactions are opened ONLY after all data transformation (e.g., Markdown parsing) is complete to minimize lock contention.
- **Read-Model Optimization:** Leverage `Redb`'s ability to return byte slices. Map these directly to `rkyv::CheckBytes` to verify and access data with zero-copy overhead. Example: let archived = rkyv::check_archived_root::<T>(&byte_slice).expect("validation failed"); let data = archived.deserialize(&mut rkyv::Infallible).unwrap();
- **Corruption Recovery:** Implement a "Clean Slate" protocol. If `Redb` fails to open due to version mismatch or corruption, rename the old database and trigger a full vault re-index.
- **Write Coordination:** Only the `IndexerActor` is authorized to open Write Transactions. All other services must request changes via the `MPSC` Data Plane.

#### MiniJinja (Templating Engine)
- **Template Lifecycle:** Compile templates exactly once at startup or upon file change (detected via tokio::fs::watch or similar). Store compiled Template objects in an Arc<Environment> for sharing across requests.
- **Recursive Safety:** Explicitly set a `max_template_depth` (e.g., 10) in the MiniJinja environment to prevent stack overflows.
- **Error Fidelity:** Map MiniJinja `Error` objects to `miette` diagnostics, including line/column of the template source.
- **Strict Undefineds:** Use `UndefinedBehavior::Strict`. Template authors (agents) must ensure all variables are provided or have defaults in the MiniJinja context. Rendering code should fail fast on undefined variables.

### Testing Rules
- **Port-Based Testing Hierarchy:**
  - **Domain Tests:** Pure unit tests with zero dependencies. Focus on logic, invariants, and conversions.
  - **Integration Tests:** Use `FakeStore` implementations to test CQRS logic without DB. Use real `RedbStore` with temporary DBs for full integration. Locations: Unit in `lithos-core/src/`, Integration in `lithos-core/tests/`.
  - **E2E CLI Tests:** Use `assert_cmd` or similar to test the compiled binary against real temporary vaults.
- **Authorized Entry Points:** All testing via Mise: Use 'mise run test' for all types, 'test:unit:<module>' for specific modules (config/note/schema/template/db/fs), 'test:coverage' for tarpaulin reports, 'test:bench' for criterion. Example: 'mise run test:unit:note'.
- **Mandatory Tools:** nextest (runner), tarpaulin (coverage), insta (snapshots), pretty_assertions (diffs), criterion (benchmarks), proptest (property testing).
- **Schema Validation Authority:** Use the JSON schemas in `docs/schemas/` (from the Go implementation) as the source of truth for backward compatibility tests of the Rust Schema Engine.
- **Starter Kit Pipeline:**
  - **Conversion:** All `templater` scripts and templates (from `docs/refs/obsidian/`) must be converted to Lithos/MiniJinja template syntax before being promoted to test fixtures or starter kit assets.
  - **Sanitization:** ABSOLUTELY ALL personal information must be removed from sample files in `docs/refs/obsidian/` before they are used in tests or packaged.
  - **Asset Bundle:** The final starter kit must include a cohesive set of sanitized templates, Go-compatible JSON schemas, a validated `lithos.toml` config, and a standard directory structure.
- **Async Testing:** ALWAYS use `#[tokio::test(flavor = "multi_thread")]` for integration tests to surface race conditions in the event bus or indexer. Safety invariants: Use timeouts (e.g., with_timeout), semaphore for I/O throttling. Block limit: >10ms requires spawn_blocking.
- **Performance Benchmarking:** Use `criterion` for all NFR-critical paths (Indexing, Rendering). 10k-note vault benchmarks are mandatory for storage changes.
- **Coverage Target:** Enforce 80%+ coverage via tarpaulin in CI pipelines. Focus coverage efforts on domain and app logic; ignore generated code.
- **Deterministic Testing:** Use fixed UUIDs and timestamps in test fixtures to ensure reproducible results. For async timing, use `tokio::time::pause`/`advance` and fixed seeds for randomness; redact UUIDs/timestamps in snapshots if snapshots are introduced.
- **Test Authoring Standards:** Naming: Verb-first (e.g., returns_error_when_invalid). Organization: Module-per-function for complex units. Rules: One behavior/assertion per test, parameterized with rstest named cases. Attributes: #[ignore] for incomplete tests.
- **Doc-Tests:** Mandatory for public domain models and utilities. Use as living documentation with executable examples. Run via 'mise run test:unit'.
- **Linting in Tests:** Use #[expect(...)] for intentional violations (e.g., unwrap in setup); #[allow(...)] for generated code. Unwrap OK in Arrange phase, never in Assert.
- **Common Pitfalls:** Avoid thread starvation (use spawn_blocking), race conditions (multi_thread flavor), flakiness (paused clocks for async), shared state (use `tempfile::TempDir` per test).
- **Testability Assessment:** Controllability: Trait-based ports for mocking. Observability: Tracing/miette for inspection. Reliability: Workspace separation, Unit of Work for atomicity.
- **Quality Gates:** Done criteria: Deterministic (0% flakiness), Isolated (no dependencies), Explicit (visible assertions), Fast (<10ms unit), Self-cleaning (RAII cleanup).
- **NFR Testing:** Security: Validate encryption at SPI layer. Performance: Criterion benchmarks, regression testing. Reliability: Fault injection, clean slate recovery.
- **Environment Requirements:** Local: Mise toolchains. CI: GitHub Actions multi-OS. Data: Sharded vaults for scaling tests.

### Code Quality & Style Rules

#### Naming & Mechanical Sympathy
- **Transparency:** Prohibit hiding expensive clones or allocations behind getter methods. If a method clones, it MUST be named `clone_x()` or `to_x_owned()`.
- **Port/Adapter Naming:**
  - **Ports:** `<Context>::ports::Store` (e.g., `note::ports::Store`).
  - **Adapters:** `Redb<Context>Store` (e.g., `RedbNoteStore`).
- **DTO Isolation:** Use `Stored*` prefix for persistence DTOs (e.g., `StoredNote`). These live in the storage layer and never leak into the public domain API.

#### Documentation as "Agent Glue"
- **The "Why" Mandate:** Doc comments (`///`) must focus on **Invariants** and **Architectural Context**. (e.g. "Must be wrapped in `Arc` because it is shared across threads").
- **Example-Driven Specs:** All public traits must include a runnable `/// # Example`. This acts as a "mini-spec" for AI agents.
- **Error Transparency:** Every `Result`-returning function must have an `# Errors` section detailing the `thiserror` variants.

#### Structural Quality (Boundary Protection)
- **Strict Module Visibility**: Use `pub(crate)` by default. Only promote to `pub` at the crate root (`lib.rs` and context roots) to explicitly define the public API surface.
- **Context Isolation**: Business contexts must not import each other. Check dependencies in review.
- **No Unsafe**: Strictly prohibit `unsafe` code. `Cargo.toml` enforces `unsafe_code = "forbid"`.
- **Import Grouping**: Strictly enforce `rustfmt`'s `group_imports = "StdExternalCrate"` to maintain a clear dependency hierarchy.

#### Clippy as Guardrail
- **Cognitive Complexity:** Enforce hard limit of 25 via clippy deny. Refactor by extracting functions if exceeded.
- **Anti-Pattern Deny:** Prohibit `clippy::unwrap_used`, `clippy::expect_used`, `clippy::todo`, `clippy::panic`, `clippy::unimplemented`, and `clippy::dbg_macro` in production code.
- **Lint Suppression Policy:**
  - Prefer `#[expect(...)]` over `#[allow(...)]` for intentional violations.
  - Every `#[expect]` must explain the constraint, why unavoidable, and how idiomatic:
    ```rust
    #[expect(lint_name, reason = "[WHAT constraint]. [WHY unavoidable]. [HOW idiomatic].")]
    ```
  - Examples:
    - Pattern Type Mismatch: "Enum has mixed Copy/non-Copy fields. Cannot dereference without moving. Matching on &self is idiomatic."
    - Test Setup: "Test fixture uses unwrap() for clear errors. Acceptable in tests."
    - Float Arithmetic: "Validation requires epsilon comparison for precision."
  - Consolidate multiple `#[expect]` on the same item into one with unified reason.

#### Process & Orchestration
- **Mise-First Formatting:** Formatting and linting MUST be run through `mise run verify` to ensure the exact toolchain versions from `mise.toml` are used.
- **Pre-commit Integrity:** The `pre-commit` hook is the final authority. Agents must fix the code to pass the hook, never bypass it.

#### Git Workflow Standards
- **Explicit Staging Only:** Use `git add <file-path>` for specific files. **NEVER use `git add -A`, `git add .`, or `git add --all`** unless explicitly instructed by the user.
- **Scope Enforcement:** Only stage files directly related to the current task. If `git status` shows modified files outside the task scope, do not stage them.
- **Verification Required:** Always run `git diff --cached --stat` before committing to verify exactly what will be committed.
- **Single Task per Commit:** Each commit should address one concern. Do not bundle unrelated changes.
- **Correct Workflow:**
  1. Identify files modified for the current task
  2. Stage explicitly: `git add path/to/specific-file.md`
  3. Verify: `git diff --cached --stat`
  4. Commit only those files with a descriptive message

### Conflicts and Exceptions
- Rules are designed to be non-conflicting, but if a conflict arises (e.g., between performance needs and safety invariants), prioritize safety. Document any project-specific exceptions in this section.

---

## Usage Guidelines

**For AI Agents:**
- Read this file before implementing any code.
- Follow ALL rules exactly as documented.
- When in doubt, prefer the more restrictive option.
- Update this file if new patterns emerge.

**For Humans:**
- Keep this file lean and focused on agent needs.
- Update when technology stack changes.
- Review quarterly for outdated rules.

Last Updated: 2026-01-23
