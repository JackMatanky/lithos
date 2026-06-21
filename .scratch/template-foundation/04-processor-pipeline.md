---
title: 04-processor-pipeline
category: enhancement
label: ready-for-agent
status: in-review
branch: feature/04-processor-pipeline
merge_commit:
date_created: 2026-06-11
date_completed:
---

# Template Processor Pipeline

Status: in-review

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Implement the Template Processor: a dual-typestate ingestion pipeline that scans the configured template directory, compares files against cached `RawTemplateView`s, and produces persisted `Template` aggregates.

Pipeline stages:
1. **Discovery** — scan the template directory for `.md` files using `DirScanner`; produce file paths
2. **Comparison** — load cached `RawTemplateView`s from the repository via batch read; compare by content hash and file metadata to determine fresh/stale/new/deleted
3. **Parsed** — read stale or new files via `FileReader`; produce `RawTemplate` DTOs
4. **Refresh** — update `RawTemplateView` records for changed files
5. **Construction** — resolve or generate `TemplateId` (look up existing by path, or `TemplateId::new()` for new templates); construct `Template` aggregates
6. **Completed** — persist `Template` and updated `RawTemplateView`s to the repository; pipeline ends here

The processor stops at `Completed`. There is no `Compiled` or `Validated` stage — compilability is a live, on-demand engine check, not an ingestion state.

`TemplateId` is resolved once in the Construction stage and carried through to `Completed`, eliminating redundant repository lookups.

## Acceptance criteria

- [x] Processor dual typestate phases are defined: `Discovery`, `Comparison`, `Parsed`, `Refresh`, `Construction`, `Completed`
- [x] No `Compiled` or `Validated` stage exists
- [x] `TemplateId` is resolved exactly once (Construction stage) and not looked up again in `Completed`
- [x] File reads use `FileReader`, not raw `std::fs`
- [x] Directory scanning uses `DirScanner`, scoped to `.md` files
- [x] Tests cover: fresh (no-op), new file (full construction path), stale content (refresh + re-construction), stale timestamp only (metadata-only refresh without re-construction), deleted-cache detection, batch path comparison correctness
- [ ] Deleted-cache execution removes repository entries (deferred to issue-07)

---

## Implementation Notes

### Stage-Based Typestate Pipeline
The `TemplateProcessor` is implemented using a dual-typestate pattern: `TemplateProcessor<Phase, Status>`. This ensures that only valid transitions are possible at compile-time.

- **Discovery**: Entry point, initializes the pipeline with discovered filesystem paths.
- **Comparison**: Interacts with the repository to classify paths into `Missing`, `Present`, `Suspect`, or `Deleted`.
- **Parsed**: Handles I/O via `FileReader` for `Missing` or `StaleContent` paths.
- **Refresh**: Prepares `RawTemplateView` updates for `StaleTimestamps`.
- **Construction**: Resolves `TemplateId` (one-time lookup) and constructs `Template` aggregates.
- **Completed**: Terminal stage that persists changes (templates and views) back to the repository.

### Static Dispatch
Repository interactions use static dispatch (`<R: ReadRepository>`, `<R: WriteRepository>`) to avoid virtual call overhead and align with project performance patterns.

### Failure Injection & Testing
The `InMemoryRepository` in `testing.rs` was enhanced with a fluent API for failure injection:
- `with_failure_injector(Box<dyn FailureInjector + Send + Sync>)`
- `with_harness(Arc<InMemoryHarness>)`

This allows for fine-grained control over repository errors in unit tests without exposing internal harness fields.

### IO Failure Simulation
Since `FilePath::try_new` validates file existence, simulating IO errors during read requires a "create-then-delete" strategy in tests to trigger `std::io::ErrorKind::NotFound` or similar at the point of `FileReader` usage.

### Test Hygiene
- Followed `unit-naming.md` (Structure A, verb-first naming).
- Used `pretty_assertions` for all diff-based checks.
- Zero `unwrap()`/`expect()` in production code; all errors are mapped to the domain-specific `TemplateError`.

### Verification
- `mise run test:unit -p template processor` (183 tests passed)
- `mise run lint` (Passed)
- `mise run fmt` (Passed)

### Follow-up: TemplateService Integration

The processor pipeline is now driven by `TemplateService::load` rather than only by processor unit tests. The service performs the batch orchestration that the processor intentionally does not own:

- Scans the configured template directory with `DirScanner` and markdown filtering.
- Batch-loads cached `RawTemplateView`s with `find_raw_template_views_by_paths`.
- Resolves existing template IDs through the path index with `find_template_id_by_path`.
- Drives the processor through fresh, new, stale-content, and stale-metadata branches.
- Persists new and stale-content templates plus their fresh `RawTemplateView`s.
- Persists metadata-only `RawTemplateView` refreshes without rewriting the aggregate.
- Detects deleted cached paths by diffing `list_raw_template_view_paths()` against discovered paths; actual deletion remains deferred to issue-07.

Service integration required the redb storage adapter to maintain the `TEMPLATE_ID_BY_PATH` index on save/delete so construction can resolve IDs by path without scanning all templates. It also added raw-view path listing for deletion detection and length validation around batch raw-view lookup so mismatched repository results fail as corruption instead of silently truncating work.

### Remaining Dead-Code Allowances

`lithos-core/src/template/processor.rs` still has a module-level `#![allow(dead_code, unused_imports)]`. The original reason string is stale: the processor is no longer completely unused because `TemplateService` exercises the main ingestion branches.

The allowance remains because this issue introduced a compile-time typestate surface larger than the currently wired service path. Several marker/status types, branch wrappers, and test-facing accessors exist to document legal transitions or preserve forward-compatible states, but they are not all referenced as runtime values outside processor tests yet. Removing the broad allowance should be a cleanup task once issue-07 deletion execution and any remaining service-facing branches are wired, replacing it with narrower `#[cfg(test)]` or item-level allowances where needed.

## Blocked by

- `issue-02-config-spec.md`
- `issue-03-repository-traits.md`

---

> *This was generated by AI during triage.*

## Agent Brief

**Category:** enhancement
**Summary:** Implement the Template Processor dual-typestate ingestion pipeline (Discovery → Comparison → Parsed → Refresh → Construction → Completed)

**Current behavior:**
No template ingestion pipeline exists. There is no mechanism to scan the configured template directory, compare files against cached views, or produce persisted `Template` aggregates.

**Desired behavior:**
A `TemplateProcessor<Phase, Status>` type implements a six-phase, dual-typestate pipeline for template ingestion, mirroring the shape of `PropertyBankProcessor`:

1. **`Discovery` phase** — constructed from `TemplateConfigSpec`; resolves the configured declarative template directory against the vault root; uses `DirScanner` to enumerate `.md` files recursively; transitions to `Comparison` with discovered filesystem paths and storage keys
2. **`Comparison` phase** — uses the batch `find_raw_template_views_by_paths` repository method to load cached `RawTemplateView`s; status branches classify each discovered file as `Missing`, `Present`, `Suspect`, or equivalent branch-specific status; also detects cached views whose paths are no longer discovered as deleted
3. **`Parsed` phase** — only reached for content that must be read/parsed; reads stale-content or new files using vault-scoped `FileReader` (not raw `std::fs`); produces `RawTemplate` DTOs and downstream construction status
4. **`Refresh` phase** — only reached for metadata-only refresh paths; prepares `RawTemplateView` updates for stale-timestamp-only files and carries the processor back toward fresh construction/fetch behavior
5. **`Construction` phase** — resolves each affected template's `TemplateId` exactly once by path before constructing or fetching `Template` aggregates; statuses distinguish fresh, new, stale, and deleted outcomes; timestamp-only refreshes do not reconstruct `Template` aggregates
6. **`Completed` phase** — terminal phase with the completed ingestion outcome; persists newly constructed or reconstructed `Template` aggregates and updated `RawTemplateView`s through the template repository write capability; removes deleted template/cache entries; pipeline ends here

Individual paths skip phases when the status makes the phase unnecessary. For example, the fresh path can flow from `Comparison<Present>` to `Construction<Fresh>` without `Parsed` or `Refresh`; the timestamp-only path can flow through `Refresh<StaleTimestamps>` before returning to `Construction<Fresh>`.

No `Compiled` or `Validated` stage is added — engine compilation is a live on-demand check, not an ingestion state.

**Triage context:**
- Category/state recommendation remains `enhancement` + `ready-for-agent`.
- No matching `.out-of-scope/` record exists for this request.
- GitNexus found analogous schema processor flows for fresh/no-op, timestamp-only metadata normalization, stale-content refresh, and deleted-entry handling; the indexed graph is currently 6 commits behind HEAD, so the implementing agent must verify current symbols before editing.
- Relevant architectural decisions: file ingestion separates file I/O from parsing/domain/persistence; template engine compilation is runtime/on-demand; repository boundaries use segregated read/write traits; storage keys use `PathKey`; filesystem access uses the FS context path taxonomy.

**Key interfaces and contracts:**
- `TemplateProcessor<Phase, Status>` — generic dual-typestate processor; the first parameter represents the logical phase, the second represents the branch/status proven inside that phase. Only legal transitions are callable for the current phase/status pair.
- `TemplateConfigSpec` — the narrowed config contract consumed by discovery; it exposes the vault root, declarative relative template directory, derived directory path, and directory `PathKey`.
- `DirScanner` — FS discovery utility; use extension filtering for `.md` files and keep discovery scoped to the configured template directory.
- `FileReader` — vault-scoped read adapter; all template body reads flow through it, preserving testability and path-boundary policy.
- `PathKey` — the only repository/storage boundary representation for template paths; derive it at filesystem-to-storage seams, not by ad hoc string manipulation.
- `ReadRepository` + `WriteRepository` — template repository capabilities used for batch cache lookup, template lookup/persistence, raw view persistence, and deletions. The processor should depend on the narrowest capability per stage where practical.
- Template ID lookup by path — construction requires an efficient read contract to resolve an existing template identity from a `PathKey` before rebuilding. If the completed repository slice does not expose this directly, add the minimal repository read method or index needed for this processor rather than scanning all templates.
- `TemplateId`, `Template`, `RawTemplate`, `RawTemplateView`, `TemplateName`, `TemplateBody` — domain/value types produced or refreshed by the pipeline. `RawTemplate` carries unvalidated source content; `TemplateBody` rejects empty content; `TemplateName` is path-derived relative to the configured template directory.
- Stale detection — content staleness is based on `Blake3Hash`; timestamp-only staleness is based on file metadata with matching content hash. These are separate outcomes with different write behavior.
- Error handling — return typed `Result` errors; avoid `unwrap()`/`expect()` in production paths; wrap repository failures in the template context's repository/domain error model rather than erasing them with `anyhow`.

**Rust implementation constraints:**
- Use `PhantomData` or equivalent zero-cost markers for both phase and status so invalid transitions fail to compile rather than relying on runtime flags.
- All transitions go through a private `transition(self, _stage: NP, status: NS) -> TemplateProcessor<NP, NS>` method on `impl<P, S> TemplateProcessor<P, S>`, mirroring `BaseSchemaProcessor` and `PropertyBankProcessor`. A private `transition_from_parts` static helper is used where parts have already been destructured. `impl From` is not used for transitions.
- Prefer borrowing over cloning when carrying discovered paths, views, and raw content through stages; clone only when ownership transfer or persistence boundaries require it.
- Keep filesystem path types, display/config path types, and storage-key types distinct according to the path taxonomy.
- Keep MiniJinja/compiler checks out of this pipeline. Template Engine behavior may consume persisted templates later, but engine compilation is not an ingestion state.
- Public APIs added for the processor need doc comments and tests that demonstrate intended stage usage.

**Discovery input design (indexer forward-compatibility):**
Today, Discovery is constructed from a `DirScanner` scan of `TemplateConfigSpec`. Design the input boundary so it accepts a description of discovered files rather than performing the scan internally. The natural unit is one entry per template file carrying: the filesystem path (`FilePath`), the storage key (`PathKey`), and file metadata (`FileMetadata`). This matches the shape that the in-progress `indexer/` context already produces (`FileIndexEntry` carrying `FileRecord { path, name, metadata }` plus `FilePath`). When the indexer is wired in, Discovery will receive `FileIndexEntry` slices instead of running its own `DirScanner` walk — no processor internals need to change. For now, the processor should own a thin constructor that runs `DirScanner` and converts results to this input shape, keeping the scan logic behind a clear seam.

**Acceptance criteria:**
- [x] `TemplateProcessor<Phase, Status>` is the processor shape; it does not collapse phase and status into a single `State` parameter
- [x] Processor phases `Discovery`, `Comparison`, `Parsed`, `Refresh`, `Construction`, `Completed` are defined as distinct typestate parameter types
- [x] Branch/status types model the flow from the component model, including missing/new, present/fresh, suspect, stale-content, stale-timestamps, deleted, and completed outcomes as needed
- [x] All transitions use the private `transition()` / `transition_from_parts()` helpers on `TemplateProcessor<P, S>`, mirroring the existing processor pattern; `impl From` is not used for stage transitions
- [x] Discovery accepts a slice of pre-discovered entries (path + metadata) rather than running `DirScanner` internally; a thin constructor exists that produces this slice via `DirScanner` for the current direct-scan path
- [x] The input entry shape for Discovery is compatible with `FileIndexEntry` from the `indexer/` context so the future wiring requires only a constructor change, not a processor redesign
- [x] No `Compiled` or `Validated` stage exists anywhere in the processor
- [x] Directory scanning uses `DirScanner` scoped to `.md` files; no raw `std::fs::read_dir`
- [x] File reads use `FileReader`; no raw `std::fs::read_to_string` or `std::fs::read`
- [x] `TemplateId` is resolved exactly once in the `Construction` stage; the `Completed` stage does not re-query the repository for IDs
- [x] Existing-template ID resolution uses a repository path lookup/index, not `list_templates()` scanning
- [x] Fresh files produce no repository write (no-op path)
- [x] New files go through full construction and are persisted
- [x] Stale-content files are re-read, re-hashed, re-constructed, and persisted with a new `RawTemplateView`
- [x] Stale-timestamp-only files update `RawTemplateView` metadata without re-constructing the `Template`
- [ ] Deleted-cache entries (present in repository but not on disk) are removed from the repository (DEFERRED to issue-07)
- [x] Tests use the in-memory test double from issue-03, not a real redb instance
- [x] Tests are written before implementation and use descriptive behavior-focused names
- [x] Tests cover: fresh (no-op), new file, stale content, stale timestamp only, batch path comparison correctness, and deleted-cache detection (DELETED-CACHE REMOVAL DEFERRED)
- [x] Tests cover compile-time typestate intent where practical (for example, stage-specific method availability through positive compileable examples or doc tests; avoid runtime boolean state assertions)
- [x] Tests cover repository failure propagation for batch lookup, template persistence, and raw view persistence
- [ ] Tests cover repository failure propagation for deletion paths (DEFERRED to issue-07)
- [x] Production code contains no `unwrap()`/`expect()`/`panic!` for recoverable filesystem, repository, path conversion, or template validation failures
- [x] `mise run fmt` passes
- [x] `mise run lint` passes
- [x] `mise run test` passes

**Out of scope:**
- redb storage adapter
- `Compiled` or `Validated` processor stages
- Engine compilation or rendering
- Artifact pipeline and CLI
- Frontmatter parsing or query semantics

---

## TDD Plan

### Pre-step: Add `find_template_id_by_path` and `find_template_by_path` to `ReadRepository`

The current `ReadRepository` trait (`lithos-core/src/template/repository.rs`) does not expose a path lookup. The `Construction` stage requires an efficient `find_template_id_by_path` to resolve an existing `TemplateId` without scanning all templates or deserializing full aggregates. A convenience `find_template_by_path` (which delegates to the ID lookup) should also be added.

Files to change:
- `lithos-core/src/template/storage/tables.rs` — add `pub(crate) const TEMPLATE_ID_BY_PATH: PathUuidTable<TemplateId> = PathUuidTable::new("template_id_by_path");`
- `lithos-core/src/template/repository.rs` — add `find_template_id_by_path(&self, path: &PathKey) -> Result<Option<TemplateId>, TemplateRepositoryError>` and `find_template_by_path(&self, path: &PathKey) -> Result<Option<Template>, TemplateRepositoryError>` to `ReadRepository`
- `lithos-core/src/template/storage/testing.rs` — implement on `InMemoryRepository` (will require adding a `HashMap<PathKey, TemplateId>` index internally, updated during `save_template`/`delete_template`)
- `lithos-core/src/template/storage/read.rs` — implement on `RedbRepository` (redb adapter implementation deferred per out-of-scope rules; stub or `todo!()` acceptable if adapter is not yet wired)

Tests (in `testing.rs`):
- `find_template_id_by_path_returns_none_when_repository_is_empty`
- `find_template_id_by_path_returns_some_id_after_saving_template`
- `find_template_by_path_returns_none_for_unknown_path_after_saving_different_template`
- `find_template_by_path_returns_template_after_saving`

### Error types

Add `TemplateReadError` to `lithos-core/src/template/error.rs`:

```rust
/// Errors returned when reading a template file from the filesystem.
///
/// Wraps [`crate::fs::ReadError`] so file I/O failures surface through the
/// template error hierarchy without leaking `fs` internals into call sites.
#[derive(Debug, thiserror::Error)]
pub enum TemplateReadError {
    #[error(transparent)]
    Read(#[from] crate::fs::ReadError),
}
```

Add a `Read` variant to `TemplateError`:

```rust
/// A template file read error.
#[error(transparent)]
Read(#[from] TemplateReadError),
```

`TemplateError` then covers all processor failure modes:

| Failure kind           | Path                                                              |
|------------------------|-------------------------------------------------------------------|
| File read / I/O        | `fs::ReadError` → `TemplateReadError` → `TemplateError::Read`    |
| Empty body             | `TemplateBodyError` → `TemplateError::Body`                      |
| Name derivation        | `TemplateNameError` → `TemplateError::Name`                      |
| Repository persistence | `TemplateRepositoryError` → `TemplateError::Repository`          |

The processor uses `TemplateError` as its single `Result` error type throughout. No separate loader error type is introduced.

Tests for the new error type:
- `template_read_error_wraps_fs_read_error_via_from`
- `template_error_read_variant_wraps_template_read_error_via_from`

### Phase 1 — `Discovery` stage

**File:** `lithos-core/src/template/processor.rs` (new file)

Entry struct: `TemplateProcessor<Discovery, Discovered>` built from a slice of pre-discovered entries. Each entry carries `FilePath`, `PathKey`, and `FileMetadata` — the same shape as `FileIndexEntry` from `indexer/` for forward-compatibility. A thin `from_scan` constructor runs `DirScanner` scoped to `.md` files and converts results into this slice.

Tests:
- `from_entries_with_empty_slice_produces_processor_with_no_entries`
- `from_entries_stores_path_and_metadata_for_each_entry`
- `from_scan_produces_same_shape_as_from_entries` (integration: requires a temp dir with `.md` files)

### Phase 2 — `Comparison` stage, fresh path (tracer bullet)

Batch-load `RawTemplateView`s via `find_raw_template_views_by_paths`. Classify each discovered entry against its cached view. Also collect `PathKey`s present in the repository but absent from discovery as deleted.

Status branches produced by comparison:
- `Missing` — no cached view exists
- `Fresh` — timestamps match; no content read needed
- `Suspect` — timestamps differ; content has been read for hash comparison
- `Deleted` — view exists in repository but path not discovered on disk

All branch transitions use `self.transition(NextStage, next_status)`. Branching outcomes are expressed as branch enums (e.g. `ComparisonBranch`) returning the correctly-typed processor variant for each arm.

Tests:
- `comparison_classifies_entry_as_fresh_when_timestamps_match`
- `comparison_classifies_entry_as_missing_when_no_cached_view_exists`
- `comparison_classifies_entry_as_suspect_when_timestamps_differ`
- `comparison_detects_deleted_entry_when_path_absent_from_discovery`
- `comparison_batch_correctly_classifies_mixed_entries` (fresh + missing + suspect + deleted in one batch)
- `fresh_path_produces_no_repository_writes`

### Phase 3 — New file path (`Missing` → `Parsed` → `Construction` → `Completed`)

`Parsed<Missing>` reads content via `FileReader`, computes `Blake3Hash`, constructs `TemplateName` and `TemplateBody`, builds a `Template` with a fresh `TemplateId::new()`, and persists both `Template` and a new `RawTemplateView`.

Tests:
- `new_file_reads_content_via_file_reader_not_std_fs`
- `new_file_constructs_template_name_from_path_relative_to_template_dir`
- `new_file_persists_template_and_raw_template_view`
- `new_file_assigns_new_template_id`
- `new_file_with_empty_content_returns_template_error_body`
- `new_file_with_io_failure_returns_template_error_read`
- `new_file_repository_write_failure_returns_template_error_repository`

### Phase 4 — Stale content path (`Suspect` → content hash mismatch → `Parsed<Stale>` → `Construction` → `Completed`)

When timestamps differ and the content hash also differs, the file is fully stale. The processor re-reads content (already read in `Comparison` to compute the hash), looks up the existing `TemplateId` via `find_template_id_by_path`, reconstructs the `Template` aggregate, and persists both `Template` and a new `RawTemplateView`.

Tests:
- `stale_content_detects_hash_mismatch_after_timestamp_mismatch`
- `stale_content_resolves_existing_template_id_via_path_lookup`
- `stale_content_persists_updated_template_and_new_raw_template_view`
- `stale_content_repository_write_failure_returns_template_error_repository`

### Phase 5 — Stale timestamp only (`Suspect` → content hash match → `Refresh<StaleTimestamps>` → `Construction<Fresh>` → `Completed`)

When timestamps differ but the content hash matches, only the `RawTemplateView` metadata is updated. The `Template` aggregate is not re-written.

Tests:
- `stale_timestamp_only_detects_hash_match_after_timestamp_mismatch`
- `stale_timestamp_only_saves_raw_template_view_with_updated_metadata`
- `stale_timestamp_only_does_not_call_save_template`
- `stale_timestamp_only_fetches_existing_template_without_reconstruction`

### Phase 6 — Deleted-cache entries (DEFERRED to issue-07)

Entries present in the repository's cached views but absent from the discovered set are deleted from both `Template` and `RawTemplateView` storage. Handling of these deletions is deferred to the `TemplateService` in issue-07.

Tests:
- `deleted_entry_removes_template_from_repository`
- `deleted_entry_removes_raw_template_view_from_repository`
- `deleted_entry_delete_failure_returns_template_error_repository`

### Phase 7 — Typestate compile-time intent

Doc tests on stage-specific methods confirm that only legal transitions are callable at each phase. All transitions go through `self.transition(NextStage, next_status)`; branching outcomes use branch enums.

Examples to verify at compile time (via doc tests or `compile_fail` tests where appropriate):
- `TemplateProcessor<Discovery, Discovered>` exposes `compare` but not `parse` or `construct`
- `TemplateProcessor<Construction, Fresh>` exposes `fetch` but not `create`
- `TemplateProcessor<Construction, New>` exposes `create` but not `fetch`
- `transition()` called inside stage-specific `impl` blocks produces the correct next typestate

### Test coverage matrix

| Scenario               | Phase path                                                         | Repository writes                              |
|------------------------|--------------------------------------------------------------------|------------------------------------------------|
| Fresh (no-op)          | Discovery → Comparison → Construction → Completed                  | None                                           |
| New file               | Discovery → Comparison → Parsed → Construction → Completed         | `save_template` + `save_raw_template_view`     |
| Stale content          | Discovery → Comparison → Parsed → Construction → Completed         | `save_template` + `save_raw_template_view`     |
| Stale timestamp only   | Discovery → Comparison → Refresh → Construction → Completed        | `save_raw_template_view` only                  |
| Deleted-cache entry    | Detected in Comparison batch diff (DEFERRED)                       | `delete_template` + `delete_raw_template_view` |
| Batch path correctness | Mixed batch with all of the above                                  | Correct per-entry branching                    |
| Repository failure     | Any write or delete path                                           | `Err` propagated, no panic                     |
| File read failure      | New file or stale content path                                     | `Err(TemplateError::Read(...))`, no panic      |

### TemplateService Orchestration

To orchestrate the ingestion pipeline and avoid coupling `TemplateProcessor` to directory scanning or batch logic, a basic `TemplateService` acts as the coordinator:
- **`scan_templates`**: Private method that uses `DirScanner` to discover `.md` files in the configured template directory.
- **`fetch_cached_views`**: Private method that calls `ReadRepository::find_raw_template_views_by_paths` for all discovered files to retrieve existing `RawTemplateView`s.
- **`load`**: Orchestration method that ties the scan, cached view fetching, and `TemplateProcessor` together, advancing each discovered file through its appropriate state transitions to the `Completed` stage.

*Note: Identifying deleted templates (present in the repository but absent from the scan) is also managed by this orchestrator, but actual deletion processing is deferred to issue-07.*

### Revised TDD Plan: `TemplateService` Orchestration

Following the **vertical slicing** approach, we will first refactor the `TemplateProcessor` boundary, and then build out the `TemplateService` behaviors sequentially.

#### Cycle 0: Refactor `TemplateProcessor` for Batch Pre-fetching
**Goal:** Remove individual repository lookups from `TemplateProcessor::compare` to support batch processing.
*   **Test:** Update existing processor tests in `lithos-core/src/template/processor.rs` to pass `Option<TemplateId>` and `Option<RawTemplateView>` directly to `compare` instead of passing the repository.
*   **Implementation:**
    *   Change the signature: `pub(crate) fn compare(self, id: Option<TemplateId>, view: Option<RawTemplateView>) -> DiscoveryBranch`
    *   Remove `<R: ReadRepository>` generic from `compare`.

#### Cycle 1: The Tracer Bullet (Empty Directory)
**Goal:** Establish the basic struct, `scan_templates`, and the `load` entry point.
*   **Test:** `load_should_return_empty_list_when_template_directory_is_empty`
*   **Implementation:**
    *   Create `TemplateService` struct.
    *   Implement private `scan_templates` using `DirScanner::new().entries()` filtered to `.md` extensions and `recursive(true)`.
    *   Implement public `load` that calls `scan_templates` and returns an empty vector.

#### Cycle 2: Batch Cache Checking (`check_batch_existence`)
**Goal:** Implement the batch fetching mechanism that powers the comparison stage.
*   **Test:** `check_batch_existence_should_retrieve_views_and_ids_for_provided_paths`
*   **Implementation:**
    *   Implement `fn check_batch_existence<R: ReadRepository>(&self, repository: &R, paths: &[PathKey]) -> Result<Vec<(Option<TemplateId>, Option<RawTemplateView>)>, TemplateError>`
    *   Perform a lookup using `repository.find_raw_template_views_by_paths(paths)` and loop over paths with `repository.find_template_id_by_path(path)` to zip the results together.

#### Cycle 3: Discovering & Processing New Templates
**Goal:** Orchestrate the pipeline for entirely new markdown files.
*   **Test:** `load_should_process_new_markdown_files_and_ignore_other_extensions`
*   **Implementation:**
    *   In `load`, gather paths from `scan_templates`.
    *   Pass paths to `check_batch_existence`.
    *   Iterate over zipped `(file, path_key)` and `(id, view)`.
    *   Instantiate `TemplateProcessor::new(file, path_key)`.
    *   Drive the processor: `compare(id, view)` → `DiscoveryBranch::Missing` → `parse(file_reader)` → `create(repo, template_root)`.

#### Cycle 4: Processing Fresh Templates (No-op)
**Goal:** Ensure unmodified files skip parsing and reconstruction.
*   **Test:** `load_should_fetch_existing_templates_without_repository_writes_when_fresh`
*   **Implementation:**
    *   In `load`, handle `DiscoveryBranch::Present`.
    *   Call `check_metadata()` → matches `MetadataBranch::Match` → calls `fetch(repo)`.

#### Cycle 5: Processing Stale Content
**Goal:** Orchestrate the pipeline when file content has actually changed.
*   **Test:** `load_should_reconstruct_and_update_template_when_content_hash_changes`
*   **Implementation:**
    *   In `load`, handle `MetadataBranch::Mismatch`.
    *   Call `check_content(file_reader)` → `ContentBranch::Mismatch` → `parse()` → `update(repo, id, template_root)`.

#### Cycle 6: Processing Stale Metadata (Timestamp-only refresh)
**Goal:** Ensure files with changed timestamps but identical content hashes only sync metadata.
*   **Test:** `load_should_sync_metadata_without_reconstruction_when_only_timestamps_change`
*   **Implementation:**
    *   In `load`, handle `ContentBranch::Match`.
    *   Call `sync_metadata(repo)` → calls `fetch(repo)`.

#### Cycle 7: Deletion Detection (Deferred execution)
**Goal:** Ensure the orchestrator identifies deleted templates as requested by the handoff.
*   **Test:** `load_should_identify_deleted_templates_for_deferred_processing`
*   **Implementation:**
    *   Add a `// TODO(#issue-07): Process deletions for paths in cache but absent from disk.` note where we would process the diff between discovered paths and cached paths.

---

## Approved Update Plan: Processor-Owned Execution

### Goal

Refactor the Template Processor integration so `TemplateService` owns only scan and cache prefetch orchestration while `TemplateProcessor<Init, Discovered>` owns per-template execution through `run()`, `run_present()`, `run_missing()`, and `run_corrupt()`.

### Architecture

`TemplateService::load()` scans markdown files, batch-loads cached raw views and template IDs, detects deferred deletions, and then hands one discovered template record at a time to `TemplateProcessor<Init, Discovered>::run()`. The processor classifies the discovered cache state into missing, present, or recoverably corrupt paths and returns a real terminal typestate carrying the completed `Template`.

The processor must preserve the typestate pattern and use `self.into_parts()` / `transition_from_parts()` when moving state between phases. Content checks must use `status.view.is_content_match(&hash)` through `HasContentHash`; stale-content updates must mutate the carried `RawTemplateView` with `set_content_hash()` and `update_metadata()` instead of replacing it blindly.

### Worktree Guardrail

All implementation and verification work must run only under `/Users/jack/Documents/41_personal/lithos/.worktrees/04-processor-pipeline`. Agents and subagents must verify `pwd` before every command/read/edit and must not inspect or modify files outside `.worktrees/04-processor-pipeline`.

### Files

- Modify `lithos-core/src/template/service.rs`: scan output shape, cache-state classification, simplified `load()` loop.
- Modify `lithos-core/src/template/processor.rs`: `Init`/`Discovered` entry, processor-owned `run()` methods, real terminal typestate, `into_parts()` cleanup, ID-based fresh fetch, recoverable corrupt rebuild.
- Modify `lithos-core/src/template/views.rs` only if dead-code allowances become stale after `HasContentHash` use.
- Modify `.scratch/template-foundation/04-processor-pipeline.md`: record this approved update plan and any final implementation notes.

### Task 1: Service Scan Output

- Introduce `ScannedTemplate { file: FileNode, path_key: PathKey }` in `service.rs`.
- Change `scan_templates()` to return `Result<Vec<ScannedTemplate>, TemplateError>`.
- Build `DirScanInput` with `.with_extensions(&["md"]).include_dirs(false).recursive(true)`.
- Keep the `FsNode::File` match while using `DirScanner::entries()`, because the scanner API still returns `FsNode` even when directory output is disabled.
- Update scan/load tests to assert only markdown files are processed.

### Task 2: Service Builds Discovered Cache State

- Replace `type CacheExistence = (Option<TemplateId>, Option<RawTemplateView>)` with processor input records.
- Add processor-owned discovery input types:

```rust
pub(crate) struct Discovered {
    file: FileNode,
    path_key: PathKey,
    cache: DiscoveredCacheState,
}

pub(crate) enum DiscoveredCacheState {
    New(Missing),
    Exists(Present),
    Corrupt(Corrupted),
}
```

- Change `check_batch_existence()` to accept `Vec<ScannedTemplate>` and return `Vec<Discovered>`.
- Classify cache state as:
  - `(None, None)` → `New(Missing)`
  - `(Some(id), Some(view))` → `Exists(Present { id, view })`
  - `(Some(id), None)` → recoverable `Corrupt(Corrupted { id, view: None })`
  - `(None, Some(view))` → repository corruption error, because there is no stable `TemplateId` to preserve.
- Preserve the existing batch length mismatch corruption check.

### Task 3: Processor Owns Execution

- Replace service-visible `DiscoveryBranch`, `MetadataBranch`, and `ContentBranch` orchestration with `TemplateProcessor<Init, Discovered>::run()`.
- Add phases/statuses:

```rust
pub(crate) struct Init;
pub(crate) struct CompletedPhase;
pub(crate) struct Missing;
pub(crate) struct Present { id: TemplateId, view: RawTemplateView }
pub(crate) struct Corrupted { id: TemplateId, view: Option<RawTemplateView> }
```

- Add `TemplateProcessor::<Init, Discovered>::new(discovered: Discovered) -> Self`.
- Add `run()`, `run_missing()`, `run_present()`, and `run_corrupt()`.
- Keep internal branch enums where useful, but stop exposing branch handling to `TemplateService`.

### Task 4: Real Terminal Typestate

- Replace the marker-only `Completed` with a terminal status that carries the loaded or persisted template:

```rust
pub(crate) struct CompletedPhase;

pub(crate) struct Completed {
    template: Template,
}

impl TemplateProcessor<CompletedPhase, Completed> {
    pub(crate) fn into_template(self) -> Template {
        self.status.template
    }
}
```

- Change `create()`, `update()`, `fetch()`, and `run_corrupt()` to return `TemplateProcessor<CompletedPhase, Completed>`.
- `TemplateService::load()` should call `.run(...)? .into_template()`.

### Task 5: Fresh Fetch Uses TemplateId

- Keep `Fresh { id }` meaningful.
- Change `TemplateProcessor<Construction, Fresh>::fetch()` to call `ReadRepository::find_template_by_id(self.status.id)`.
- Return `TemplateRepositoryError::NotFoundById(id)` or the nearest existing not-found error if `find_template_by_id()` returns `None`.
- Do not fetch fresh templates by path.

### Task 6: Stale Content Updates Existing View

- In `TemplateProcessor<Construction, Changed>::update()`, destructure with `into_parts()`.
- Build the updated `Template` using the existing `TemplateId`.
- Mutate the carried `RawTemplateView`:

```rust
status.view.set_content_hash(status.content_hash);
status.view.update_metadata(file.metadata().clone());
```

- Persist the template and the mutated view.
- Return the terminal completed processor.

### Task 7: Content Hash Trait And State Moves

- Import `HasContentHash` and `HasContentHashMut` in `processor.rs`.
- Replace `status.view.content_hash().is_match(&hash)` with `status.view.is_content_match(&hash)`.
- Refactor `check_metadata()` and `check_content()` to destructure with `into_parts()` once and use `transition_from_parts()` for both branches.
- Avoid cloning `RawTemplateView` when ownership can move through typestate status.

### Task 8: Corrupt Rebuild Path

- Implement `run_corrupt()` for recoverable `(Some(id), None)` cache state.
- Read the file through `FileReader`.
- Compute the hash.
- Construct `Template` using the preserved `TemplateId`, current `PathKey`, derived `TemplateName`, and `TemplateBody`.
- Build a new `RawTemplateView` when none exists, or update the carried view when present.
- Persist both template and raw view.
- Return the terminal completed processor.

### Task 9: Service Simplification

- Remove `process_branch()`, `process_present()`, and `process_suspect()` from `TemplateService`.
- `load()` should:
  1. scan templates;
  2. derive `PathKey`s for deferred deletion detection;
  3. call `check_batch_existence()`;
  4. call `TemplateProcessor::<Init, Discovered>::new(discovered).run(...)?.into_template()` for each discovered record.

### Task 10: Dead-Code Cleanup

- Remove the module-level `#![allow(dead_code, unused_imports)]` in `processor.rs` after the refactor.
- Remove obsolete test-only helpers and branch accessors that are no longer used.
- Use narrow item-level `#[expect(dead_code, reason = "...")]` only for intentionally retained typestate markers.

### TDD Checklist

- Write failing tests before production changes for cache classification, terminal typestate return, fresh ID fetch, stale-content view mutation, recoverable corrupt rebuild, and view-without-ID corruption.
- Run focused red tests before implementation.
- Implement the minimum code to pass.
- Run focused green tests after each slice.
- Run `mise run fmt`, `mise run lint`, `mise run test`, `git diff --check`, and GitNexus `detect_changes()` before completion.

### Implementation Result

- `TemplateService` now scans into `ScannedTemplate` records, explicitly sets `DirScanInput::include_dirs(false)`, classifies cache state in `check_batch_existence()`, and delegates per-template execution to `TemplateProcessor<Init, Discovered>::run()`.
- `check_batch_existence()` now maps discovered cache state to `New(Missing)`, `Exists(Present)`, or recoverable `Corrupt(Corrupted)` and treats `(None, Some(view))` as repository corruption because no stable `TemplateId` can be preserved.
- `TemplateProcessor` now owns `run()`, `run_missing()`, `run_present()`, and `run_corrupt()` and returns `TemplateProcessor<CompletedPhase, Completed>` from all terminal paths.
- Fresh templates fetch by `TemplateId` through `ReadRepository::find_template_by_id()`.
- Stale-content updates mutate the carried `RawTemplateView` with `set_content_hash()` and `update_metadata()` before persistence.
- Content comparison uses `status.view.is_content_match(&hash)` through `HasContentHash`.
- `check_metadata()` and `check_content()` now destructure with `into_parts()` and move state through `transition_from_parts()` without cloning raw views.
- The obsolete `Discovery`/`DiscoveryBranch`/`compare()` compatibility path and stale module-level dead-code allowance were removed.

Verification:
- Red check: `mise run test:unit:template` failed first with missing `DiscoveredTemplate`, `DiscoveredCacheState`, `Init`, `run()`, and terminal `into_template()` APIs.
- Focused green check: `mise run test:unit:template` passed.
- Lint: `mise run lint` passed.
- Full tests: `mise run test` passed.
- ADR validation: `mise run adr:validate` was up to date.
- Whitespace: `git diff --check` passed.
- GitNexus: `detect_changes(scope="all")` reported LOW risk and no affected processes.

### Typestate Alignment and Corrupt Path Handling
The processor was adjusted to ensure recoverable corrupt cache (entries with an ID but missing view) correctly traverse the typestate parsing phase.
- `run_corrupt()` now transitions to `TemplateProcessor<Parsed, Corrupted>` and reads file content via `parse(source)`.
- `TemplateProcessor<Construction, Changed>::update()` now correctly accepts `view: Option<RawTemplateView>` and constructs the missing view inline when writing.
- `RawTemplateView`'s `HasContentHash` trait implementation was given an explicit `is_content_match` method to bypass the default reference-based comparison behavior, correctly utilizing the value semantics of `Blake3Hash`.

### Typestate Dead Code Allowance Cleanup
The original broad `#![allow(dead_code)]` logic was replaced, but individual typestate transition methods (`parse`, `check_metadata`, `create`, `update`, etc.) were temporarily annotated with `#[cfg_attr(test, allow(dead_code))]`. Now that `TemplateProcessor::run()` dynamically routes execution through all these methods via matching the `DiscoveredCacheState`, all `#[cfg_attr(test, allow(dead_code))]` statements have been removed from the typestate implementations.
