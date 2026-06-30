---
title: 05-indexer-app-wiring
category: enhancement
label: ready-for-agent
status: open
branch:
merge_commit:
date_created: 2026-06-09
date_completed:
---

# Issue 05: `trace-app::index` wiring — `IndexCommand` and `run_index` flow

## What to build

Flesh out `crates/app` (placeholder introduced in commit `2f644fdc`, currently
published as `trace-app`) with the typed command and execution flow for the
Indexer use case. The `app` crate is the composition root: it constructs
concrete adapters from `trace-indexer`, wires ports, and exposes a typed
execution flow to the CLI adapter. No business logic lives here — only wiring.

Introduce a single `index` module inside `trace-app`:

- `IndexCommand` carrying `IndexScope` and `IndexOptions` (domain types
  re-exported from `trace-indexer`).
- `run_index(root: &DirPath, cache_dir: &DirPath, cmd: IndexCommand)
   -> Result<IndexResult, AppError>` — expects a pre-resolved vault root and
  cache directory (produced by the bootstrapper), constructs the walkdir
  scanner, the redb repository, and the `IndexerService` from `trace-indexer`,
  delegates to `IndexerService::run()`, and returns the result.
  Pipeline order (Discovery → Config → Indexer) is guaranteed by the caller.
  Does not receive `Config` — only the vault root and cache dir are needed.

No `flows/`, `composition/`, or `diagnostics/` sub-modules: composition is
trivial (open store → construct adapters → inject) and fits inline in `run_index`.
Diagnostics are handled by the existing error + report types + tracing.

## Required: visibility uplift in `trace-indexer`

Before this issue can be implemented, `trace-indexer`'s public API must expose
the types the app crate needs. Currently ALL types are `pub(crate)`.

The following must become `pub` (following hexagonal architecture: ports and
adapters are the public boundary of each context):

| Type | Reason | Module |
|------|--------|--------|
| `ScannerPort` trait | Port — bound on `IndexerService` | `port.rs` |
| `WalkIter` type alias | Return type of `ScannerPort::walk()` | `port.rs` |
| `ScanEntry` enum | Yielded by `WalkIter` — part of `ScannerPort` contract | `port.rs` |
| `ReadRepository`, `WriteRepository`, `Repository` traits | Ports — bounds on `IndexerService` | `repository.rs` |
| `IndexerService<S, R>` struct + `new()` + `run()` | Hex-arch domain Service | `service.rs` |
| `WalkdirAdapter` struct + impl | Adapter — app constructs and injects | `scanner/walkdir.rs` |
| `RedbRepository` struct + `try_new()` | Adapter — app constructs from `Arc<Store>` | `storage/mod.rs` |
| `IndexScope`, `IndexOptions`, `ScanFilters` | Domain types — app constructs from command | `scan.rs` |
| `IndexResult`, `IndexedNodes`, `DeletedNodes` | Return type — app returns to CLI | `summary.rs` |
| `FileIndexEntry`, `DirIndexEntry`, `IndexStatus` | Exposed via `IndexedNodes::files()`/`dirs()` | `entry.rs` |
| `FsRecordId` | Exposed via `DeletedNodes::files()`/`dirs()` | `model.rs` |
| `IndexReport`, `SkippedEntry`, `SkipReason`, `IndexNodeFailure` | Exposed via `IndexResult::report()` | `report.rs` |
| `IndexerError`, `IndexerRepositoryError`, `ScannerError` | Error chain — app wraps in `AppError` | `error.rs` |

Additionally, the `scanner` and `storage` modules in `lib.rs` must change from
`pub(crate)` to `pub` so their types are reachable.

Kept `pub(crate)`: `InMemoryRepository` (test-only), `FileRecord`, `DirRecord`,
`FsParentId`, `FsRecordType` (model internals not referenced in app paths),
builder internals.

This visibility pattern is reproducible: each domain crate exports its Service,
Ports, concrete Adapters, and domain types as `pub`; the app crate imports and
wires them.

## Acceptance criteria

- [ ] `trace-indexer` visibility uplift completed (all types above made `pub`).
- [ ] `trace-indexer` added as dependency in `crates/app/Cargo.toml`.
- [ ] `trace-db` added as dependency in `crates/app/Cargo.toml` (for `Store`).
- [ ] `AppError` renamed from `BootstrapError` with added `Indexer(#[from] IndexerError)` variant.
- [ ] CLI `From<trace_app::BootstrapError>` updated to `From<trace_app::AppError>`.
- [ ] `IndexCommand` defined with private `IndexScope` and `IndexOptions` fields,
      public accessor methods.
- [ ] `INDEX_DB_FILENAME` const added to `trace-indexer::storage` (`"index.redb"`).
- [ ] `run_index(root: &DirPath, cache_dir: &DirPath, cmd: IndexCommand)
      -> Result<IndexResult, AppError>` constructs `WalkdirAdapter`,
      `RedbRepository`, and `IndexerService`, then delegates to
      `IndexerService::run()`. Opens store at `cache_dir / INDEX_DB_FILENAME`.
- [ ] `run_index` does not run Discovery or Config itself (correct pipeline
      order guaranteed by caller).
- [ ] `trace-app` exposes no redb, walkdir, or adapter-specific types in its
      public surface.
- [ ] Integration test: calling `run_index` with `trace_db::testing::TestDb`
      (temp-dir vault + real redb) produces an `IndexResult` with correct
      counts and no panics.
- [ ] `app` module-level documentation describes the `index` module and scope
      guardrails (per ADR 021, updated for crate structure).
- [ ] `IndexerError` variants are reviewed and confirmed to cover all error
      conditions that can arise in the wiring layer.
- [ ] All existing tests still pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).

## Blocked by

- 04-application-service.md
- Visibility uplift in `trace-indexer` (all types above)

---

## Triage Notes

> *This was generated by AI during triage.*

**Verdict**: `ready-for-agent` — with prerequisites (visibility uplift + deps).

**What was checked (triage review, 2026-06-23):**

### Crate structure

The issue was originally written assuming `lithos-core::app`. The workspace has since been restructured to `crates/`:
- `crates/app` (published as `trace-app`) — composition root
- `crates/indexer` (published as `trace-indexer`) — indexer bounded context
- ADRs 021 and 024 reference the old `lithos-core::app` paths and need updating.

### Existing codebase state

- **`crates/app`**: Has `bootstrap.rs` (1162 lines, full `Bootstrapper<D: DiscoveryPort>` with `run()`, `run_discovery_only()`, `from_platform()`), `error.rs` (`BootstrapError` covering Discovery + Config), `lib.rs` (doc stubs for planned submodules). No `IndexCommand` or indexer wiring exists.
- **`crates/indexer`**: Mature. `IndexerService<S: ScannerPort, R: Repository>` with full test coverage (empty scan, single file/dir, reindex, deletions, dry-run, integration). `WalkdirAdapter` (scanner), `RedbRepository` + `InMemoryRepository` (storage). All types currently `pub(crate)` — **blocker**.

### Critical blocker: visibility

100% of `trace-indexer`'s types are `pub(crate)`. The app crate cannot use any of them. See the Required section above for the uplift list.

### Key changes from original issue

| Original | Revised |
|----------|---------|
| `lithos-core::app` | `crates/app` (`trace-app`) |
| 4 sub-modules (commands, flows, composition, diagnostics) | Single `index` module — composition is inline, diagnostics is dead pattern (replaced by errors + tracing + reports) |
| Separate `composition.rs` | Not needed — opening a store handle is a 1-liner |
| Separate `diagnostics.rs` | Removed — pattern is dead per maintainer confirmation |
| `run_index` as a method on a struct | Stateless function `run_index(config, cmd)` — mirroring `Bootstrapper::run()` |
| `BootstrapError` unchanged | Renamed to `AppError` with additional `Indexer(#[from] IndexerError)` variant |

### Hexagonal architecture assessment

The approach validated against `docs/refs/rust/guides/hexagonal_architecture.md`:
- Ports (`ScannerPort`, `Repository`) are `pub` domain traits ✅
- Adapters (`WalkdirAdapter`, `RedbRepository`) are `pub` concrete implementations ✅
- Service (`IndexerService`) is `pub` and generic over ports ✅
- Composition happens in app crate (composition root) ✅
- Types exposed through port/return-type contracts are `pub`; purely internal
  types (`InMemoryRepository`, model internals, builder) stay `pub(crate)` ✅
- Pattern is reproducible: each domain crate follows this same boundary

### Requirements for implementation

- [ ] Uplift visibility in `crates/indexer/src/` (see table above).
- [ ] Add `trace-indexer` + `trace-db` to `crates/app/Cargo.toml`.
- [ ] Rename `BootstrapError` → `AppError`, add `Indexer(#[from] IndexerError)` variant.
- [ ] Update CLI `From<trace_app::BootstrapError>` → `From<trace_app::AppError>`.
- [ ] Create `crates/app/src/index.rs` with `IndexCommand` and `run_index()`.
- [ ] Write integration test (temp-dir vault + real walkdir + `trace_db::testing::TestDb`).
- [ ] Update ADR 021 and ADR 024 crate path references.
- [ ] `mise run verify` (fmt + lint + tests) after changes.

---

## TDD Plan

Approved pre-implementation decisions:

| Decision | Resolution |
|----------|------------|
| `run_index` parameters | `(root: &DirPath, cache_dir: &DirPath, cmd: IndexCommand)` — no `Config` |
| `BootstrapError` → `AppError` | Rename, update CLI `From` impl, **no** deprecated type alias |
| Store path | `cache_dir / INDEX_DB_FILENAME` (const `"index.redb"` in `trace-indexer::storage`) |
| Integration storage | `trace_db::testing::TestDb` (real redb, not `InMemoryRepository`) |
| `IndexCommand` fields | Private, public accessor methods |
| Visibility promotion | Only the 21 types in the required table + module-level `pub` changes |
| Pipeline order | Caller guarantees Discovery → Config → Indexer |

### Slice V1: Visibility uplift

**Files:** `crates/indexer/src/lib.rs`, `port.rs`, `repository.rs`, `service.rs`, `scan.rs`, `summary.rs`, `entry.rs`, `model.rs`, `report.rs`, `error.rs`, `scanner/mod.rs`, `scanner/walkdir.rs`, `storage/mod.rs`

Change all `pub(crate)` → `pub` for the 21 types listed in the required table. Change `pub(crate) mod scanner` and `pub(crate) mod storage` to `pub mod`. Add `pub use walkdir::WalkdirAdapter;` to `scanner/mod.rs`.

Kept `pub(crate)`: `InMemoryRepository`, `FileRecord`, `DirRecord`, `FsParentId`, `FsRecordType`, builder internals.

**Tests:** Existing `test_indexer_exports` passes. `mise run test`.

### Slice V2: `INDEX_DB_FILENAME` const

**File:** `crates/indexer/src/storage/mod.rs`

```rust
pub const INDEX_DB_FILENAME: &str = "index.redb";
```

**Tests:** None — constant, no behavior.

### Slice V3: Dependencies

**File:** `crates/app/Cargo.toml`

Add `trace-indexer` to `[dependencies]`, `trace-db` to both `[dependencies]` and `[dev-dependencies]` (with `features = ["testing"]`).

**Tests:** `cargo check -p trace-app` compiles.

### Slice V4: `AppError` + CLI update

**RED** — Write 3 tests in `crates/app/src/error.rs`:

| Test | Verifies |
|------|----------|
| `converts_indexer_error_to_app_error` | `IndexerError` → `AppError::Indexer` via `#[from]` |
| `preserves_discovery_error_variant` | `DiscoveryError` → `AppError::Discovery` |
| `preserves_config_error_variant` | `ConfigError` → `AppError::Config` |

**GREEN** — Rename `BootstrapError` → `AppError` in `crates/app/src/error.rs`, add `Indexer(#[from] IndexerError)` variant. Update all references in `crates/app/src/bootstrap.rs`. Update `crates/cli/src/error.rs` `From<trace_app::BootstrapError>` → `From<trace_app::AppError>`.

### Slice V5: `IndexCommand`

**RED** — Write test in `crates/app/src/index.rs`:

```rust
mod index_command {
    mod constructor {
        #[test]
        fn creates_command_with_scope_and_options() { ... }
    }
}
```

**GREEN** — Create `crates/app/src/index.rs`:

```rust
pub struct IndexCommand {
    scope: IndexScope,
    opts: IndexOptions,
}

impl IndexCommand {
    pub fn new(scope: IndexScope, opts: IndexOptions) -> Self { ... }
    pub fn scope(&self) -> &IndexScope { ... }
    pub fn opts(&self) -> IndexOptions { ... }
}
```

### Slice V6: `run_index`

**RED** — Write test:

```rust
mod run_index {
    #[test]
    fn returns_app_error_when_store_fails() { ... }
}
```

**GREEN** — Implement `run_index`:

```rust
pub fn run_index(
    root: &DirPath,
    cache_dir: &DirPath,
    cmd: IndexCommand,
) -> Result<IndexResult, AppError> {
    let store = Store::open(&cache_dir.as_path().join(INDEX_DB_FILENAME))
        .map_err(|e| AppError::Indexer(IndexerError::Repository(e.into())))?;
    let repo = RedbRepository::try_new(Arc::new(store))?;
    let service = IndexerService::new(root.clone(), WalkdirAdapter, repo);
    Ok(service.run(cmd.scope(), cmd.opts())?)
}
```

### Slice V7: Integration test

**File:** `crates/app/tests/index.rs` (new)

| Test | Verifies |
|------|----------|
| `run_index_with_temp_vault_returns_correct_counts` | Real vault files → correct file/dir counts, new_count |
| `run_index_handles_empty_vault` | Empty vault → dirs=1 (vault root), files=0, no panic |

Uses `trace_db::testing::TestDb` for temp dir + redb store.

### Slice V8: ADR updates

**Files:** `docs/adr/021-app-composition-root.md`, `docs/adr/024-bootstrapper-orchestration.md`

Replace `lithos-core::app` → `crates/app` / `trace-app`. Update planned submodule descriptions in ADR 021 to reflect single `index` module.

### Verification

```
mise run verify    # fmt + lint + test
```

### Complete test inventory

| Loc | Module | Test |
|-----|--------|------|
| `crates/app/src/error.rs` | `conversions` | `converts_indexer_error_to_app_error` |
| `crates/app/src/error.rs` | `conversions` | `preserves_discovery_error_variant` |
| `crates/app/src/error.rs` | `conversions` | `preserves_config_error_variant` |
| `crates/app/src/index.rs` | `index_command::constructor` | `creates_command_with_scope_and_options` |
| `crates/app/src/index.rs` | `run_index` | `returns_app_error_when_store_fails` |
| `crates/app/tests/index.rs` | — | `run_index_with_temp_vault_returns_correct_counts` |
| `crates/app/tests/index.rs` | — | `run_index_handles_empty_vault` |

## Implementation Notes

**1. `trace_fs` validation and future files**
During implementation of `run_index`, it was noted that we cannot use `trace_fs` methods like `DirPath::append_filename()` to construct the path for `index.redb`. `trace_fs` enforces strict runtime validations against the filesystem state, meaning it will return a `PathError::NotAFile` if the database does not exist yet. Consequently, `Store::open()` accepts a standard `&std::path::Path`, and standard library `.join()` operations are structurally correct for constructing the path to a file that is about to be created.

**2. Vault root indexing and empty vaults**
The original acceptance criteria assumed an empty vault would result in `dirs=1` (accounting for the vault root). However, `trace_fs::path::PathKey` (which is required by `DirRecord` and `FileRecord` to represent vault-relative paths) explicitly disallows empty paths. Because the relative path of the vault root to itself is empty, it cannot be represented as a `PathKey` and is therefore mathematically not indexed as a `DirRecord`. An empty vault correctly yields `0` directories and `0` files. The integration test was updated to assert this reality.

**3. Visibility uplift and `private_interfaces`**
To adhere perfectly to Hexagonal Architecture, the Ports (`ReadRepository`, `WriteRepository`) were made `pub`. However, to honor the constraint that domain models (`FileRecord`, `DirRecord`) remain `pub(crate)`, we utilized `#![allow(private_interfaces, private_bounds)]` at the crate root. This satisfies the compiler while preventing the composition root from coupling to the internal data representations of the bounded context.

**4. Adversarial Review & Corrections**
An adversarial review was conducted following the initial implementation, resulting in several critical structural and logic corrections:
- **CLI Exit Codes:** Corrected the `CliError::exit_code` mapping for `AppError::Indexer(_)`. It previously returned `1` ("vault not found"); it now correctly returns `3` (POSIX I/O / permission error), reflecting the actual traversal and storage failures typical of indexing operations.
- **Hexagonal Architecture Visibility:** Re-evaluated the `#![allow(private_interfaces, private_bounds)]` suppression block. Rather than keeping domain models like `FileRecord` and `DirRecord` as `pub(crate)` and masking the compiler error, their visibility was elevated to `pub`. Because these models serve as the structured data payload destined for downstream bounded contexts (Schema, Note, Template) in the very next execution flow, public visibility is the architecturally correct choice and cleanly resolves the lint.
- **Strict Lint Compliance:** Removed the redundant local `#![deny(missing_docs)]` crate attribute, as the workspace `Cargo.toml` already strictly enforces it. Authored complete, structured doc-comments (`///`) and `# Errors` sections for all newly-public Indexer traits, methods, and error definitions.
- **Test Standardization:** Standardized test names in `crates/indexer/src/scanner/walkdir.rs` to strictly follow the `[action]_[expected]_[condition]` formula established in `docs/engineering/testing/unit-naming.md`. Added `pretty_assertions` to `crates/app/tests/index.rs` to conform to assertion visibility guidelines.
