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
- `run_index(config, cmd) -> Result<IndexResult, AppError>` — expects a
  pre-resolved `Config` (produced by `bootstrap.rs`), constructs the walkdir
  scanner, the redb repository, and the `IndexerService` from `trace-indexer`,
  delegates to `IndexerService::run()`, and returns the result.
  Pipeline order (Discovery → Config → Indexer) is guaranteed by the caller.

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
- [ ] `AppError` in `crates/app/src/error.rs` extended with `Indexer` variant
      (wrapping `IndexerError`).
- [ ] `IndexCommand` is defined with `IndexScope` and `IndexOptions` fields.
- [ ] `run_index` constructs `WalkdirAdapter`, `RedbRepository`, and
      `IndexerService`, then delegates to `IndexerService::run()`.
- [ ] `run_index` receives a pre-resolved `Config` (produced upstream by
      `bootstrap.rs`) and does not run Discovery or Config itself (correct
      pipeline order: Discovery → Config → Indexer, with the first two
      stages handled by the caller).
- [ ] `trace-app` exposes no redb, walkdir, or adapter-specific types in its
      public surface.
- [ ] Integration test: calling `run_index` with a real (temp-dir) vault root
      produces an `IndexResult` with correct counts and no panics.
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
- [ ] Add `Indexer` variant to `AppError` in `crates/app/src/error.rs`.
- [ ] Create `crates/app/src/index.rs` with `IndexCommand` and `run_index()`.
- [ ] Write integration test (temp-dir vault + real walkdir + InMemoryRepository).
- [ ] Update ADR 021 and ADR 024 crate path references.
- [ ] `mise run verify` (fmt + lint + tests) after changes.
