---
title: PRD - PropertyBankProcessor File Identity Context Deepening
labels: needs-triage
status: draft
created: 2026-05-27
---

# PRD: PropertyBankProcessor File Identity Context Deepening

## Problem Statement

`PropertyBankProcessor` (`lithos-core/src/schema/property_bank_processor.rs`) implements a dual-typestate pipeline for property bank ingestion. Its status structs carry `FileMetadata` (content freshness data) but **not `FilePath`** (file identity). This forces the caller (`Builder`) to thread `config_path: &Path`, `path_key: &PathKey`, and `source: &FsReader` as three separate arguments through every method — even though `FsFile` already bundles `FilePath` + `FileMetadata` in one compile-time unit.

`PropertyBankDiscovery` (`lithos-core/src/schema/discovery.rs:79`) carries `FsEntry` (a `File`/`Dir` enum), but the builder must unwrap it with `.metadata().as_file().cloned()` — an extra runtime check that should be a compile-time guarantee.

`persist()` in `New` (line 654) and `persist()` in `Changed` (line 714) use **different** `HashRecord` sources (`raw.properties().compute_hashes()` vs `self.status.raw_hash`), meaning the cached view can diverge between creation and update paths — a potential cache corruption bug.

## Solution

Three deepening candidates (Stale statuses stay separate per type-safety):

### Candidate 1: Carry `FsFile` as Processor Root Context

`PropertyBankProcessor<P, S>` gains a `file: FsFile` root field. All status methods derive:
- `FilePath::as_path()` → `config_path` (file I/O)
- `FilePath::as_key()` → `PathKey` (storage)
- `FileMetadata` (from `FsFile.metadata()`)

The `config_path` and `path_key` external args **vanish** from every method signature. The processor's typestate now proves file identity invariant across all stages.

### Candidate 2: Unified `persist()` Source

Both `New::persist` and `Changed::persist` compute `HashRecord` from the same source: `&self.status.raw` (the `RawPropertyBank`). Currently `New` computes from `self.status.raw.properties()` (un-analyzed) while `Changed` uses `self.status.raw_hash` (analysis-derived). One `persist()` that always computes from `&RawPropertyBank` — never two.

### Candidate 3: Remove `Discovery` Dead Stage

`PropertyBankProcessor<Discovery, Unknown>` has only `new()` and `Default::default()` — no `discover()` method. The `DiscoveryEngine::run()` does all discovery. Remove the stage entirely; processor starts at `Comparison`.

### Supporting Changes

**`PropertyBankDiscovery`**: `entry: FsEntry` → `entry: FsFile`. Removes the `.as_file()` check at the builder boundary — the type system already guarantees it's a file.

**`PathKey` construction**: Always through `FilePath::as_key()` — never `PathKey::try_new()`. The processor derives `PathKey` once at construction from `FsFile.path`.

## User Stories

1. As a developer, I want `PropertyBankProcessor` to carry `FsFile` in its root, so that file identity is a compile-time invariant across all pipeline stages.

2. As a developer, I want `PropertyBankDiscovery.entry` to be `FsFile` instead of `FsEntry`, so that caller never needs to `.as_file()` check at the builder boundary.

3. As a developer, I want `persist()` to produce the same `HashRecord` from the same `RawPropertyBank` source regardless of which path created it, so that the cached view cannot diverge between `New` and `Changed` paths.

4. As a developer, I want `PathKey` derived from `FilePath::as_key()` at processor construction, so that every `save_raw_property_bank_view` call passes the correct storage key without external args.

5. As a developer, I want the `Discovery` stage marker removed from the processor type, so that the dual-typestate has only 1 real stage parameter instead of 2.

6. As a developer, I want `path_key` and `config_path` removed from all method signatures in the processor, so that no method accepts both a filesystem path and a storage key — eliminating the potential for semantic drift.

7. As a developer, I want the processor's `status` structs to carry only domain-relevant data (content freshness, cached view state), not file metadata that `FsFile` already provides.

8. As a developer, I want existing `builder.rs` integration tests to pass unchanged after this refactor, so that the change is verifiable as a pure internal interface shift.

9. As a developer, I want `InMemoryRepository` tests to continue working without modification, so that test infrastructure stays stable.

10. As a developer, I want the processor's `status` structs to retain `FileMetadata` for content freshness (not `FsFile`), so that content-staleness invariants remain distinct from file-identity invariants.

## Implementation Decisions

### Processor Root Context

- `PropertyBankProcessor<P, S>` gets `file: FsFile` as a private field
- `transition()` keeps its existing signature; `from_fs_file()` is the public constructor
- All status methods access `self.file.path()` and `self.file.metadata()` instead of receiving them as args
- `PathKey` derived via `PathKey::from_path(file.path())` — called once, not per method
- `WithMetadata` pattern: status-only data (content freshness, view) stays in status; file data (path, metadata) lives at processor root

### Persistence Unification

- `persist()` moves to `impl<P, S> PropertyBankProcessor<P, S>` (generic over any stage)
- Takes `(&RawPropertyBank, &PathKey, &WriteRepository)` — single `HashRecord` source
- `New::persist` and `Changed::persist` both call the same `persist()` with `&self.status.raw`
- `Changed::persist` passes `self.status.raw` (already present) — not `self.status.raw_hash`

### PropertyBankDiscovery Type

- `PropertyBankDiscovery` (`lithos-core/src/schema/discovery.rs:79`): `entry: FsEntry` → `entry: FsFile`
- `PropertyBankDiscovery.entry()` returns `&FsFile` — no `.as_file()` needed
- Affects `lithos-core/src/schema/builder.rs:170` — `file_info = bank_discovery.entry().metadata().clone()` → `file_info = bank_discovery.entry().metadata()`

### `FsFile` vs `FileMetadata` in Status

- Status structs (`Missing`, `Present`, `Suspect`, `Stale`, `StaleTimestamps`, `StaleContent`, `New`, `Changed`) keep `FileMetadata` for content-freshness comparison **only**
- They do NOT carry `FilePath` — file identity lives at processor root, not in status
- This is the correct split: `FileMetadata` is a **content** invariant; `FilePath` is an **identity** invariant

## Testing Decisions

### What Makes a Good Test

Tests should verify behavior through the **processor's public interface** — the `PropertyBankProcessor` entry points and its `Completed` output types. Not internal status transitions or `fs_file` field access.

### Which Modules Are Tested

- **Unit**: typestate transitions (`AnalysisBranch::Empty` → `Refresh::StaleContent`, `AnalysisBranch::Delta` → `Construction::Changed`)
- **Integration**: `builder.rs` tests (line 301-394) — these already test through the processor
- **New**: `persist()` hash equivalence — one test per pair, verify `HashRecord` matches for same `RawPropertyBank`

### Test Scope

- `persist()` hash equivalence: `New::persist` and `Changed::persist` on same `RawPropertyBank` produce identical `HashRecord`
- `FsFile` access: processor correctly reads `file.path()` and `file.metadata()` from root context
- `Discovery` stage removal: no code path references `PropertyBankProcessor<Discovery, Unknown>` after removal

## Out of Scope

- `SchemaProcessor` (`lithos-core/src/schema/schema_processor.rs`) — handled by separate `BaseSchemaProcessor` split architecture (`.scratch/base-schema/`)
- `FsEntry` enum type changes — `FsFile` already exists; only `PropertyBankDiscovery` changes
- ADR updates — `PathKey` is already the correct repository boundary type (per ADR-019); `PathKey::try_new` is a public constructor that will be deprecated
- `Stale` status merger — rejected; `Option<T>` weakens compile-time guarantees
- Any changes to `PropertyBank` or `RawPropertyBank` domain types
- `HashRecord` computation algorithm — only the **source** of `HashRecord` changes, not the algorithm
- Parallel processing or performance optimization — this is a type-safety/interface cleanup, not a performance PRD

## Further Notes

- This PRD is intentionally **non-breaking**: it adds `file: FsFile` to the processor root without changing any existing `Builder` integration test output
- The `persist()` unification is the most critical correctness fix — it prevents potential view cache divergence
- `PropertyBankDiscovery` type change (`FsEntry` → `FsFile`) is the most impactful — it removes the `as_file()` runtime check from every builder call path
- After this PRD, the implementation should be decomposed into 3 independent issues (one per candidate) for sequential implementation
- The `Stale` status consolidation is intentionally deferred — separate statuses for separate invariants are correct; `sync_metadata` duplication is the real problem

