---
title: DEPENDENCIES
date_created: 2026-05-14
date_modified: 2026-05-14
---

# Dependencies Registry

This file is optimized for fast scanning first, deep context second.

---

## Snapshot

Legend:
- `core`: architecturally critical
- `active`: currently used in manifests/code paths
- `dormant`: currently declared but not actively used
- `deferred`: intentionally not present now; re-add when needed

| Crate | State | Area | Why | Owner | Next |
| --- | --- | --- | --- | --- | --- |
| redb | core+active | persistence | embedded ACID KV | core | keep |
| rkyv | core+active | serialization | zero-copy reads | core | keep |
| thiserror | core+active | errors | typed errors | core | keep |
| miette | core+active | diagnostics | rich UX + LSP fit | core/cli | keep |
| minijinja | core+active | templates | runtime template packs | template | keep |
| pulldown-cmark | core+active | parsing | fast event parser | parser | keep |
| clap | active | CLI | stable CLI parsing | cli | keep |
| tracing | active | observability | structured logs | platform | keep |
| serde (+json/yaml/toml) | active | data/config | format IO | platform | keep |
| uuid | active | identity | stable IDs | core | keep |
| rayon | active | runtime | sync parallelism | core | keep |
| tokio | dormant | runtime | old async path | platform | remove |
| async-trait | dormant | runtime | old async traits | platform | remove |
| figment | dormant | config | legacy config stack | config | remove |
| convert_case | dormant | utils | text transforms | core | remove |
| slug | dormant | utils | slugify helper | core | remove |
| dhat | dormant | perf tooling | heap profiling | perf | remove (for now) |
| assert_cmd | dormant | testing | CLI process tests | cli/test | remove (re-add later) |
| predicates | dormant | testing | assertion matchers | cli/test | remove (re-add later) |
| mockall | dormant | testing | mocks | test | remove (unless strategy changes) |

---

## Action Queue

### Remove now (if `cargo check --workspace` + tests remain green)
- `tokio`
- `async-trait`
- `figment`
- `convert_case`
- `slug`
- `dhat`
- `assert_cmd`
- `predicates`
- `mockall`

### Keep
- `redb`, `rkyv`, `thiserror`, `miette`, `minijinja`, `pulldown-cmark`
- `serde` family, `uuid`, `rayon`, `clap`, `tracing`

### Revisit later
- `assert_cmd` + `predicates` when CLI integration/e2e test suite begins
- `dhat` when doing dedicated heap profiling work

---

## Core Architecture Dependencies

### redb
- Purpose: embedded transactional store for read-optimized projection/cache.
- Why selected: pure Rust embedded store with transactional semantics and MVCC snapshots.
- Where used: `lithos-core` DB/storage adapters.
- Important flags: default.
- Upgrade concerns: transaction semantics, on-disk behavior, migration coverage.
- Licensing/security: permissive Rust ecosystem crate; keep cargo-deny checks active.
- Integration pattern: filesystem is source of truth; redb stores rebuildable projections.
- References: `docs/adr/006-persistence-cache-infrastructure.md`

### rkyv
- Purpose: archived zero-copy serialization for hot read paths.
- Why selected: avoid parse+allocate overhead on hot paths.
- Where used: persisted views/models in storage layer.
- Important flags: `bytecheck`, `std`, `uuid-1`, `unaligned`.
- Upgrade concerns: archived layout compatibility and schema evolution discipline.
- Licensing/security: permissive; careful validation boundaries required.
- Integration pattern: borrow archived data within transaction/guard scope only.
- References: `docs/adr/006-persistence-cache-infrastructure.md`

### thiserror
- Purpose: typed, matchable error enums.
- Why selected: typed, matchable error taxonomy with low boilerplate.
- Where used: domain + adapter errors.
- Important flags: default.
- Upgrade concerns: derive behavior/display changes.
- Licensing/security: permissive.
- Integration pattern: granular enums per boundary; convert upward explicitly.
- References: `docs/adr/005-error-handling.md`

### miette
- Purpose: rich diagnostics (codes, spans, help).
- Why selected: high-fidelity diagnostics and structured metadata compatible with LSP needs.
- Where used: user-facing error rendering path.
- Important flags: `fancy`.
- Upgrade concerns: report rendering and span behavior.
- Licensing/security: permissive.
- Integration pattern: annotate errors at presentation layer with contextual labels/help.
- References: `docs/adr/005-error-handling.md`

### minijinja
- Purpose: runtime template engine for user/community template packs.
- Why selected: runtime template loading and markdown-friendly rendering controls.
- Where used: template rendering pipeline.
- Important flags: default.
- Upgrade concerns: escaping defaults, whitespace behavior, rendering regressions.
- Licensing/security: permissive.
- Integration pattern: typed borrowed contexts from domain/archive-backed data.
- References: `docs/adr/007-template-engine.md`

### pulldown-cmark
- Purpose: high-throughput markdown event parsing.
- Why selected: one-pass event stream extraction with low memory overhead.
- Where used: note parsing, link extraction, markdown processing.
- Important flags: extension use configured in code.
- Upgrade concerns: parser event changes can affect extraction correctness.
- Licensing/security: permissive.
- Integration pattern: stream events and apply offset-safe transformations.
- References: `docs/adr/008-markdown-parsing.md`

---

## Active Supporting Dependencies

### clap
- Purpose: CLI argument parsing.
- Why selected: mature ecosystem standard.
- Where used: `lithos-cli` command entrypoints.
- Important flags: `env`.
- Upgrade concerns: derive/attribute changes.

### tracing
- Purpose: structured logs and instrumentation.
- Why selected: standard observability stack.
- Where used: cross-cutting diagnostics.
- Important flags: default.
- Upgrade concerns: subscriber compatibility and field naming consistency.

### serde / serde_json / serde_yaml / toml
- Purpose: typed data/config serialization.
- Why selected: format interoperability and ecosystem maturity.
- Where used: config/raw parsing and adapter boundaries.
- Important flags:
  - `serde`: `derive`, `rc`
  - `serde_json`: `default-features = false`, `std`
- Upgrade concerns: stricter parsing behavior and compatibility drift.

### uuid
- Purpose: stable identity values.
- Why selected: stable identifiers with sortable UUID v7 support.
- Where used: IDs across schema/storage.
- Important flags: `v7`, `v5`, `serde`.
- Upgrade concerns: format compatibility across persistence boundaries.
- References: `docs/adr/006-persistence-cache-infrastructure.md`

### rayon
- Purpose: bounded data parallelism in sync architecture.
- Why selected: performance without async runtime dependency.
- Where used: core processing utilities.
- Important flags: default.
- Upgrade concerns: deterministic behavior expectations in tests.

### chrono
- Purpose: date/time handling and timestamp serialization.
- Why selected: mature date/time API with straightforward serde integration.
- Where used: metadata timestamps and temporal fields across core models.
- Important flags: `serde`.
- Upgrade concerns: timezone/date parsing behavior and formatting defaults.

### regex
- Purpose: pattern matching for parsing/validation paths.
- Why selected: standard Rust regex engine with strong performance and reliability.
- Where used: parser/config/text validation utilities.
- Important flags: default.
- Upgrade concerns: subtle pattern-behavior differences across versions.

### walkdir
- Purpose: recursive filesystem traversal.
- Why selected: robust, battle-tested directory walking with useful filtering semantics.
- Where used: vault/file scanning adapters.
- Important flags: default.
- Upgrade concerns: symlink/traversal edge-case behavior and platform-specific path handling.

### glob
- Purpose: glob-pattern matching for path selection.
- Why selected: simple and interoperable glob semantics for include/exclude patterns.
- Where used: scanner and file filtering logic.
- Important flags: default.
- Upgrade concerns: pattern semantics and escaping edge cases.

### base64
- Purpose: binary-to-text encoding/decoding where required by formats/APIs.
- Why selected: standard, minimal-dependency implementation.
- Where used: support/util layers where textual transport of bytes is required.
- Important flags: default.
- Upgrade concerns: default engine/behavior changes and strictness modes.

### blake3
- Purpose: fast cryptographic hashing for content/index checks.
- Why selected: high throughput and strong modern hash properties.
- Where used: hash/indexing and content-diff style utilities.
- Important flags: default.
- Upgrade concerns: output compatibility assumptions in persisted/indexed data.

### rand
- Purpose: randomness for IDs, tests, and non-cryptographic randomized behavior.
- Why selected: ecosystem standard with predictable ergonomics.
- Where used: utility/test paths and any randomized generation helpers.
- Important flags: default.
- Upgrade concerns: API/trait evolution across major versions; reproducibility in tests.

### Core-local (not workspace-managed): `itoa`, `ryu`, `zstd`, `hex`
- Purpose: fast formatting/compression/encoding primitives.
- Where used: `lithos-core` internals.
- Upgrade concerns: output compatibility + compression performance changes.

---

## Dormant / Candidate for Removal

### tokio
- Purpose: async runtime.
- Current state: declared in workspace; not actively used by member manifests.
- Recommendation: remove now; reintroduce only with explicit async boundary design.

### async-trait
- Purpose: async trait ergonomics.
- Current state: declared; no active usage path.
- Recommendation: remove with tokio cleanup.

### figment
- Purpose: layered config provider framework.
- Current state: selected historically for layered config; architecture has moved away from active usage.
- Recommendation: remove dependency declaration unless config path is reactivated.
- References: `docs/adr/009-configuration-management.md`

### convert_case / slug
- Purpose: string transform helpers.
- Current state: declared but not active in member manifests.
- Recommendation: remove; re-add when concrete feature needs them.

### dhat
- Purpose: heap profiler.
- Current state: not wired into current benchmark workflow.
- Recommendation: remove now; re-add for dedicated profiling sessions.
- References: `docs/adr/012-benchmarking-infrastructure.md`

### assert_cmd / predicates
- Purpose: robust CLI process assertions.
- Current state: not yet used in active CLI integration suite.
- Recommendation: remove now; re-add when CLI e2e tests start.
- References: `docs/adr/012-benchmarking-infrastructure.md`

### mockall
- Purpose: mock-based testing.
- Current state: not active in manifests/test setup.
- Recommendation: remove unless testing strategy explicitly adopts mocking.

---

## References

- `docs/adr/005-error-handling.md`
- `docs/adr/006-persistence-cache-infrastructure.md`
- `docs/adr/007-template-engine.md`
- `docs/adr/008-markdown-parsing.md`
- `docs/adr/009-configuration-management.md`
- `docs/adr/012-benchmarking-infrastructure.md` (proposed)

### Internal Crate References (`docs/refs/crates/`)

#### Persistence / Serialization
- `redb`: `docs/refs/crates/redb.md`
- `redb` deep dives: `docs/refs/crates/redb/`
- `rkyv`: `docs/refs/crates/rkyv.md`
- `rkyv` deep dives: `docs/refs/crates/rkyv/`

#### Parsing / Filesystem
- `pulldown-cmark`: `docs/refs/crates/pulldown-cmark.md`
- `pulldown-cmark` internals: `docs/refs/crates/pulldown-cmark-dev-guide.md`
- `walkdir`: `docs/refs/crates/walkdir/README.md`
- `walkdir` API notes: `docs/refs/crates/walkdir/struct_direntry.md`, `docs/refs/crates/walkdir/struct_filterentry.md`

#### Testing / Benchmarking
- `criterion`: `docs/refs/crates/criterion.md`
- `proptest`: `docs/refs/crates/proptest.md`

#### Additional Reference Files (currently not active dependencies)
- `moka`: `docs/refs/crates/moka.md`
- `petgraph`: `docs/refs/crates/petgraph.md`

---

## Governance

1. Add new dependencies only with clear architecture placement and ownership.
2. Prefer workspace-managed versions when shared by multiple crates.
3. Record major dependency strategy shifts via ADR updates.
4. Remove dormant dependencies instead of leaving commented placeholders.
5. Validate dependency changes with project gates (`fmt`, `lint`, `test`, security checks).
