# Product Requirements Document (PRD): Lithos CLI (Rust Transition)

## Change Log

| Date       | Version | Description                                                 | Author |
| ---------- | ------- | ----------------------------------------------------------- | ------ |
| 2026-01-05 | 1.0.0   | Initial Rust-focused PRD derived from Go PRD artifacts.     | John   |

## Goals & Background

### Goals

- **Primary Goal:** Deliver a Rust-based CLI that matches or exceeds the existing Go implementation while unlocking safer concurrency, richer trait-driven abstractions, and tighter binary footprints.
- Maintain feature parity for template composition, schema validation, lookup/indexing, and interactive generation so that users can switch CLIs without rewriting their vault assets.
- Prove that the Rust architecture remains hexagonal: domain traits stay framework-agnostic, adapters encapsulate I/O boundaries, and tooling enforces `cargo fmt`/`cargo clippy`/`cargo test` gates (BMAD v6 compliance).

### Background Context

Lithos currently exists as a Go CLI that replicates the templating power of Obsidian outside the editor. Converting to Rust keeps the same user promise but shifts the runtime to a memory-safe systems language with first-class package management (`cargo`), expressive enums for domain state, and zero-cost abstractions for streaming vault data. Users still manage primarily Markdown templates, file-class schemas, and metadata validation—all features simply need to be restated with Rust idioms.

Key reasons for the transition:

1. **Unified Toolchain:** The project already carries both Go and Rust manifests. Standardising new development on Rust simplifies dependency management and reduces CGO cross-compilation friction.
2. **Safety & Performance:** Rust's ownership model guards the vault indexer, cache writers, and interactive prompts from dangling references and data races without runtime GC costs.
3. **Ecosystem Fit:** Crates like `clap`, `serde`, `tera`, and `ratatui` align closely with Lithos' CLI + templating + interactive use cases, decreasing the amount of bespoke glue code required in Go.

## Scope Guards

- **In-Scope:** Re-articulate every existing requirement for the Rust CLI, including template rendering, schema loading, vault indexing, validation, and interactive prompting.
- **Out-of-Scope:** Feature creep beyond the Go MVP (e.g., LSP server, GUI front ends) unless explicitly prioritised in later epics.
- **Compliance:** All new documents created during the Rust transition live under `docs/rust/` until the Go tree is decommissioned.

## Functional Requirements

### Core Engine & Templates

1. **FR1 – Template Composition:** Templates remain composable with reusable sections. In Rust, the template engine will standardise on `tera` (for user-friendly syntax) backed by a domain trait `Template`. Templates must fail fast on missing sections or recursive references, using `tera::Error::chain()` for detailed diagnostics.
2. **FR2 – Non-Interactive Execution:** `lithos new <template>` runs without additional flags. Implement via `clap` subcommands and `color-eyre` error reporting so automation scripts receive structured exit codes.
3. **FR3 – Template Function Library:** Provide PKM helpers within Tera's function map implemented as trait objects. Include:
   - String utilities (case transforms, wikilink builders) implemented in pure Rust and unit-tested.
   - Interactive functions exposed through async-friendly prompt traits (see FR10) but wrapped for synchronous CLI UX.
   - Query helpers bridging to the vault index (see FR9).
4. **FR4 – Template Lint Command (Stretch):** `lithos lint template` validates that template variables align with schema constraints using Rust's `schemars` or custom validators.

### Schema & Data

5. **FR5 – Metadata Class Integration:** Schemas load from JSON/YAML via `serde`/`schemars`, supporting inheritance and `$ref` property bank definitions. Domain types (`Schema`, `Property`, `PropertySpec`) become enums/structs with `serde(default)` to avoid brittle parsing.
6. **FR6 – Frontmatter Handling:** Implement YAML parsing with `serde_yaml` preserving unknown fields in a `serde_json::Value` map. Writing uses `serde_yaml::to_string` while keeping key order stable.
7. **FR7 – Schema-Based Validation:** Use enums + pattern matching to enforce property specs. Types include `Boolean`, `Integer`, `Float`, `String`, `Date`, and `File`. Date validation leverages `time` crate format strings; File validation delegates to the vault index (FR9).
8. **FR8 – Field Value Sourcing:** Allow schemas to attach query descriptors (directory filter, file class filter, custom SQL). Implement as declarative structs deserialised via `serde` and executed by the query layer.
9. **FR9 – Vault Lookup & Indexing:** Maintain an index keyed by vault-relative path, basename, or schema-defined key. Index lives in `sled` for MVP (ordered key-value store) with optional `tantivy` search integration post-MVP.

### User Interaction

10. **FR10 – Interactive Input Engine:** Provide interactive prompts (text + suggester) backed by `inquire` or `dialoguer`. Must support streaming default values, query-backed lists, and non-interactive fallbacks for CI.

## Non-Functional Requirements

1. **NFR1 – Platform Support:** Ship universal macOS (x86_64 + arm64) binaries and validate Linux x86_64 via GitHub Actions matrix builds using `cross` if needed. Windows remains backlog.
2. **NFR2 – Portability:** Deliver single static binaries via `cargo build --release` + `cargo dist`. No runtime dependencies.
3. **NFR3 – Performance:**
   - Template render (no lookups) < 300 ms.
   - Incremental index update of 1,000 notes < 5 seconds using rayon-based parallelism.
   - First-note onboarding (using sample pack) < 2 minutes, matching Go benchmarks.
4. **NFR4 – Index Architecture:** Hybrid approach: `sled` (embedded tree store) for hot cache + `sqlite` (via `rusqlite`) for ad-hoc metadata queries. Both wrapped in traits to keep domain portable.
5. **NFR5 – Index Freshness:** Manual `lithos index` rebuild remains MVP. Introduce file-watch incremental updates post-MVP using `notify` crate.
6. **NFR6 – Observability:** Structured logging via `tracing` + `tracing-subscriber`, plus `color-eyre` panic hooks for human-readable stack traces.

## Technical Foundations (Rust)

| Concern                | Crate/Tooling                         | Rationale |
| ---------------------- | ------------------------------------- | --------- |
| CLI parsing            | `clap` + `clap_complete`              | Declarative CLI, auto docs, shell completions. |
| Config handling        | `figment` or `config` crate           | Merge JSON/TOML + env vars similar to Viper. |
| YAML/JSON parsing      | `serde_yaml`, `serde_json`            | Preserve unknown schema fields and allow conversion between formats. |
| Template engine        | `tera` (primary), `handlebars` (alt)  | Rich expression language, function map, whitespace control. |
| Interactive prompts    | `inquire` / `dialoguer`               | Cross-platform TUI prompts with minimal deps. |
| Markdown parsing       | `pulldown-cmark` + custom frontmatter scanner | Fast streaming parser; frontmatter handled by a dedicated module. |
| Index storage (MVP)    | `sled` (KV), `rusqlite` (relational)  | Balanced read/write needs; both battle-tested in Rust. |
| Async runtime          | `tokio` (opt-in)                      | Reserve for future async operations; CLI initially synchronous but runtime ready. |
| Logging & errors       | `tracing`, `tracing-error`, `color-eyre` | Structured logs + human-friendly errors. |
| Release automation     | `cargo dist` + GitHub Actions         | Multi-target binaries + signing, aligning with BMAD CI gates. |

### Architecture Guardrails

- Domain modules express behaviour via traits (`TemplateLoaderPort`, `VaultIndexerPort`). Adapters implement these traits in separate crates/modules.
- Error handling standardises on `thiserror` for typed domain errors and `color-eyre` for CLI-friendly stack traces.
- Commands accept `tokio::runtime::Handle` only if asynchronous operations are needed; default to blocking code for simplicity.
- Each adapter crate/unit enforces `#![forbid(unsafe_code)]` unless absolutely required.

## Epic Alignment (Rust Version)

| Epic | Outcome in Rust Context |
| ---- | ----------------------- |
| **E1 – Foundational CLI** | `clap`-based executable, trait-driven hexagonal layout, non-interactive template rendering via `tera` + domain Template trait. |
| **E2 – Config & Schema Loading** | `serde`-powered config and schema loader modules with `$ref` resolution using strongly typed enums. |
| **E3 – Vault Indexing Engine** | Streaming vault reader leveraging `walkdir` + `rayon`, indexing into `sled`/`sqlite`, events emitted through `tracing` spans. |
| **E4 – Schema-Driven Lookups & Validation** | Validation engine written with exhaustive `match` statements and zero `unwrap()`, plus query traits bridging to the index. |
| **E5 – Interactive Input Engine** | `inquire`-backed prompt service with built-in non-interactive fallback for automation. |

## Next Steps

1. Validate this PRD with engineering + architecture to confirm crate selection, trait boundaries, and alignment with BMAD v6 architecture docs.
2. Create matching architecture refresh (under `docs/rust/`) referencing the Rust components, data models, and coding standards.
3. Incrementally port Epics 1-5, ensuring each milestone ships a minimal, testable Rust CLI binary and maintains parity with Go behaviour before deprecating Go code paths.
