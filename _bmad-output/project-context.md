---
project_name: 'lithos'
user_name: 'Jack'
date: '2026-01-12'
sections_completed: ['technology_stack', 'architectural_integrity', 'language_rules', 'framework_rules', 'testing_rules', 'quality_rules', 'workflow_rules', 'anti_patterns']
status: 'complete'
rule_count: 58
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

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
- **Figment 0.10**: Hierarchical configuration (Global -> User -> Project -> Vault).
- **thiserror 2.0 & anyhow 1.0**: Structured error definition and ergonomic context chaining.

### Tooling & Orchestration
- **mise**: **Primary and authorized entry point** for all tasks and tool version management (via `mise.toml`).
    - *Safety Invariant*: All commands MUST be executed via `mise run <task>` to ensure toolchain parity across environments.
    - **Available Tasks Reference:**
        - **Quality Gates:** `quality` (fmt+lint+validate), `verify` (full gates + tests), `fmt`, `lint`, `deny`
        - **Testing:** `test` (unit+integration, alias: `t`), `test:unit`, `test:integration`, `test:e2e`, `test:arch`, `test:coverage`, `test:watch`, `test:unit:*` (crate-specific)
        - **CI/CD:** `ci` (pipeline simulation), `verify` (alias: `v`)
        - **Development:** `build`, `clean`, `doc`, `dev-setup`, `bench`
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
- **Hexagonal Boundary Enforcement:**
    - `crates/domain` must have NO external dependencies and NO I/O.
    - All **Ports** MUST be defined as traits. Use `#[async_trait]` for all async trait definitions and implementations.
    - Use `pub(crate)` by default; reserve `pub` strictly for the crate's public interface.
- **CQRS & Event Discipline:**
    - Separate **Command** (Write) and **Query** (Read) models. Query models must be optimized for snapshots (Dumb DTOs) and never expose raw domain logic.
    - Use the **Hybrid Event Bus**: `mpsc` for Indexing (Reliable), `broadcast` for Signals (Control), and `watch` for snapshots (State/LSP).
- **Unit of Work (Transactional Context):**
    - **Atomic Commands:** Every Command Handler MUST use a `UnitOfWork` to wrap persistence logic.
    - **Repository Access:** Repositories must be accessed via the `TransactionContext` to ensure they share the same Redb `WriteTransaction`.
    - **Deferred Dispatch:** Domain events must be staged within the context and dispatched ONLY after a successful `commit()`. This prevents "phantom events" from failed transactions.
- **Dependency Injection:** Use constructor injection with `Arc<dyn Trait>` for app-level Port injection. Favor **Generics (`impl Trait`)** for internal adapter-level utility functions to maximize static dispatch performance.

### Language-Specific Rules (Rust)

#### Idiomatic Patterns
- **Standard Traits:** Always derive `Debug`, `Clone`, and `PartialEq`. Use `Default` for configurations.
- **Conversion Traits:** Mandate `From/Into` (infallible) and `TryFrom/TryInto` (fallible). AI must use these instead of ad-hoc `to_x()` methods.
- **Collection Safety:** Use the `HashMap::entry` API for updates. Use `.get()` or `.first()` instead of index-based access (`[0]`).
- **Exhaustive Matching:** Use `match` for Enums with `#[non_exhaustive]` on domain types. Prohibit `_ => {}` catch-alls for domain logic to ensure new variants are handled.

#### AI Pitfall Protections
- **Path Protocol**: Use `PathBuf` (owned) or `&Path` (borrowed) for all file paths. NEVER use `String` for paths. Prohibit `std::env::current_dir` and `std::fs::canonicalize` in favor of Figment-managed absolute paths.
- **Async Resource Safety:**
    - In `async` contexts, ONLY use `tokio::fs`. Use `spawn_blocking` for `std::fs` or heavy CPU tasks.
    - **Concurrency Throttling:** Use `tokio::sync::Semaphore` to limit concurrent I/O (e.g., when indexing 10k+ files).
- **Lock Discipline:** NEVER hold a `std::sync::MutexGuard` across an `.await`. If state must persist across awaits, use `tokio::sync::Mutex`.
- **Numeric Safety:** Prohibit `as` casting; use `.try_into().expect("...")` or `.context("...")` to prevent silent truncation.

#### Persistence & Performance
- **rkyv Requirements:** Domain types for storage MUST derive `Archive`, `Serialize`, `Deserialize`, and `CheckBytes`.
- **Zero-Copy Boundary:** Usage of `rkyv` validation and byte-casting must be isolated to the `adapters/spi/storage` layer. Domain entities should remain ergonomically usable.
- **Note Identity:** Use **UUID v7** for primary keys in Redb to ensure time-ordered insertion and logical stability during file renames.
- **Memory Strategy:** Favor `Box<str>` or `SmolStr` for immutable identifiers. Use `Cow<'a, str>` for metadata frequently read from storage buffers.

#### Error & Diagnostic Standards
- **Layered Errors:** `thiserror` in `domain`, `anyhow` + `.context()` in `app/adapters`, and `miette` in `cli`.
- **Rich Diagnostics:** All `miette` errors MUST include a **Help** label and, where possible, a **Source Snippet** pointing to the exact line/offset in the vault file (via `pulldown-cmark` offsets).
- **Main Loop Resilience:** Implement a `catch_unwind` or a top-level `Result` handler in the CLI main loop to prevent a single bad template or file from crashing the entire process.
- **No-Panic Policy:** NEVER use `unwrap()`, `expect()`, `todo()`, or `unimplemented()` in production code.

### Framework-Specific Rules

#### Tokio (Async Runtime & Safety)
- **Executor Health:** NEVER block an async thread for >10ms. Use `tokio::task::spawn_blocking` for all `std::fs` operations, heavy CPU rendering, or `Redb` write transactions.
- **Deadlock Prevention:** Strictly prohibit holding a `std::sync::MutexGuard` across an `.await`.
- **Shutdown Resilience:** All actors must use `tokio::select!` to listen for a global `broadcast::Receiver` shutdown signal. On shutdown, actors MUST complete the current atomic transaction before exiting.
- **Resource Throttling:** Use a `tokio::sync::Semaphore` to limit concurrent vault file reads to prevent exceeding OS file descriptor limits.

#### Redb & rkyv (Persistence & Performance)
- **Transaction Locality:** Wrap `Redb` transactions in a custom RAII guard. Ensure Write Transactions are opened ONLY after all data transformation (e.g., Markdown parsing) is complete to minimize lock contention.
- **Read-Model Optimization:** Leverage `Redb`'s ability to return byte slices. Map these directly to `rkyv::CheckBytes` to verify and access data with **zero-copy** overhead.
- **Corruption Recovery:** Implement a "Clean Slate" protocol. If `Redb` fails to open due to version mismatch or corruption, rename the old database and trigger a full vault re-index.
- **Write Coordination:** Only the `IndexerActor` is authorized to open Write Transactions. All other services must request changes via the `MPSC` Data Plane.

#### MiniJinja (Templating Engine)
- **Template Lifecycle:** Compile templates **exactly once** at startup or upon file change. Store the compiled `Template` objects in an `Arc<Environment>`.
- **Recursive Safety:** Explicitly set a `max_template_depth` (e.g., 10) in the MiniJinja environment to prevent stack overflows.
- **Error Fidelity:** Map MiniJinja `Error` objects to `miette` diagnostics, including line/column of the template source.
- **Strict Undefineds:** Use `UndefinedBehavior::Strict`. Agents must ensure all variables required by a template are either provided or have a defined default.

### Testing Rules
- **Hexagonal Testing Hierarchy:**
    - **Domain Tests:** Pure unit tests with zero dependencies. Focus on logic, math, and conversions.
    - **Integration Tests:** Use `nextest` to run concurrent tests. Mock all `SPI` ports (Storage, Filesystem) to test the `app` layer in isolation.
    - **E2E CLI Tests:** Use `assert_cmd` or similar to test the compiled binary against real temporary vaults.
- **Schema Validation Authority:** Use the JSON schemas in `docs/schemas/` (from the Go implementation) as the source of truth for backward compatibility tests of the Rust Schema Engine.
- **Starter Kit Pipeline:**
    - **Conversion:** All `templater` scripts and templates (from `docs/refs/obsidian/`) must be converted to Lithos/MiniJinja template syntax before being promoted to test fixtures or starter kit assets.
    - **Sanitization:** ABSOLUTELY ALL personal information must be removed from sample files in `docs/refs/obsidian/` before they are used in tests or packaged.
    - **Asset Bundle:** The final starter kit must include a cohesive set of sanitized templates, Go-compatible JSON schemas, a validated `lithos.toml` config, and a standard directory structure.
- **Async Testing:** ALWAYS use `#[tokio::test(flavor = "multi_thread")]` for integration tests to surface race conditions in the event bus or indexer.
- **Performance Benchmarking:** Use `criterion` for all NFR-critical paths (Indexing, Rendering). 10k-note vault benchmarks are mandatory for storage changes.
- **Coverage Target:** Aim for **80%+ coverage** via `tarpaulin`. Focus on `app` and `domain` logic.
- **Deterministic Testing:** Use fixed UUIDs and Timestamps in test fixtures to ensure reproducible results.

### Code Quality & Style Rules

#### Naming & Mechanical Sympathy
- **Transparency:** Prohibit hiding expensive clones or allocations behind getter methods. If a method clones, it MUST be named `clone_x()` or `to_x_owned()`.
- **Port/Adapter Naming:**
    - **Ports:** `[Subject]Port` (e.g., `NoteRepositoryPort`).
    - **Adapters:** `[Subject][Technology]Adapter` (e.g., `NoteRedbAdapter`).
- **DTO Isolation:** End transport-specific structs with `Dto`. DTOs must live only in the `adapters` layer and never leak into the `domain`.

#### Documentation as "Agent Glue"
- **The "Why" Mandate:** Doc comments (`///`) must focus on **Invariants** and **Architectural Context**. (e.g. "Must be wrapped in `Arc` because it is shared across threads").
- **Example-Driven Specs:** All public traits must include a runnable `/// # Example`. This acts as a "mini-spec" for AI agents.
- **Error Transparency:** Every `Result`-returning function must have an `# Errors` section detailing the `thiserror` variants.

#### Structural Quality (Boundary Protection)
- **Strict Module Visibility**: Use `pub(crate)` by default. Only promote to `pub` at the crate root (`lib.rs`) to explicitly define the public API surface.
- **No Unsafe**: Strictly prohibit `unsafe` code. `Cargo.toml` enforces `unsafe_code = "forbid"`.
- **Import Grouping**: Strictly enforce `rustfmt`'s `group_imports = "StdExternalCrate"` to maintain a clear dependency hierarchy.

#### Clippy as Guardrail
- **Cognitive Complexity:** Hard limit of **25 (deny)**. Max function length of **100 lines (deny)**.
- **Anti-Pattern Deny:** Prohibit `clippy::unwrap_used`, `clippy::expect_used`, `clippy::todo`, `clippy::panic`, `clippy::unimplemented`, and `clippy::dbg_macro` in production code.
- **Audit Trail Mandate**: Disabling a lint requires: `// # LINT_DISABLE_REASON: [Reason] | Options tried: [List] | Justification: [Why]`.

#### Process & Orchestration
- **Mise-First Formatting:** Formatting and linting MUST be run through `mise run verify` to ensure the exact toolchain versions from `mise.toml` are used.
- **Pre-commit Integrity:** The `pre-commit` hook is the final authority. Agents must fix the code to pass the hook, never bypass it.

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

Last Updated: 2026-01-12
