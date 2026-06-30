# PRD: Indexer → Context Processor Integration

**Status**: ready-for-agent
**Created**: 2026-06-30
**Builds on**: `.scratch/filesystem-indexer/PRD.md` (foundation — `crates/indexer/` already implemented)
**Supersedes**: `.scratch/filesystem-indexer/07-vault-deletion.md` (vault deletion folds into this work)

---

## Problem Statement

The `crates/indexer/` context now owns filesystem scanning, metadata comparison, and Fresh/Stale/New classification — but its output is consumed by nothing. Every downstream context (`crates/note/`, `crates/schema/`, `crates/template/`) still re-implements its own filesystem scan, its own metadata comparison, and its own freshness classification before doing the work that genuinely belongs to it (markdown parsing, schema resolution, template rendering).

Today that orchestration lives in `crates/vault/src/processor.rs`. `VaultProcessor::process_full` runs its own `DirScanner`, compares against its own `FileView` / `DirView` tables, then routes markdown files into `NoteProcessor` one at a time. Schema's `DiscoveryEngine` runs a parallel `DirScanner` against the same vault root. Neither uses the indexer.

The vault module is dead code in production (zero non-test callers in `crates/cli/`, `crates/app/`, or any downstream context) but cannot be deleted: `crates/note/tests/note_reader.rs` depends on `VaultProcessor::process_full` as its sole fixture-setup mechanism for all seven integration tests.

The user-facing symptoms:

- Three filesystem scans happen instead of one whenever a contributor wires a full ingest pipeline.
- Freshness classification is computed at least twice (once by indexer, once by each downstream context).
- The vault crate must continue to exist solely to keep note's integration tests green.
- The indexer's per-file `IndexStatus` classification is produced once during a run and then thrown away — downstream processors must re-derive it.
- There is no defined wire between the indexer and downstream contexts, so any first integration attempt has to invent the wire shape under pressure.

## Solution

Introduce an **event-driven integration layer** between the indexer and each downstream context. The indexer emits `IndexEvent` notifications during its scan; each downstream context (`note`, `schema`, `template`) consumes those events through a per-context consumer that delegates to its own service and, ultimately, its own processor.

The wire is a typed sink trait, implemented by an in-memory fan-out today and a persistent event log tomorrow. The sink is owned by the indexer crate — it is part of the indexer's "for downstream consumption" boundary. Each context owns its own consumer struct, its own per-context service, and a refactor of its processor to expose `IndexStatus`-keyed entry points so the early Discovery and Comparison stages can be skipped (the indexer has already done that work).

The application crate (`crates/app/`) owns only orchestration: channel wiring, thread spawning, report aggregation. It knows nothing about processors, filter rules, or status routing.

Vault deletion falls out naturally once the integration ships. Note's integration tests are rewritten against the new event-driven flow; the vault crate is then removed from the workspace.

Delivery is staged in two phases. Phase 1 ships the event types, sink trait, in-memory fan-out, per-context consumers, app orchestration, and processor refactors — the minimum that lets us delete vault. Phase 2 adds persistent event logging and per-consumer watermarks for resumability.

## User Stories

1. As an architecture reviewer, I want each downstream context to consume indexer output rather than re-scanning the filesystem, so that filesystem scanning happens exactly once per indexing run.
2. As an architecture reviewer, I want the indexer's `IndexStatus` classification to flow to downstream consumers, so that Fresh/Stale/New is computed exactly once per file per run.
3. As a Lithos maintainer, I want the `crates/vault/` crate deleted from the workspace, so that there is one canonical owner of filesystem node state and no dead module to mislead future contributors.
4. As a maintainer of `crates/note/`, I want my integration tests to set up fixtures without depending on `crates/vault/`, so that my context's tests do not block vault deletion.
5. As a maintainer of `crates/note/`, I want `NoteProcessor` entry points keyed by `IndexStatus`, so that I can drive the right pipeline branch directly from an `IndexEvent` without paying for Discovery and Comparison stages that the indexer already ran.
6. As a maintainer of `crates/schema/`, I want my schema ingestion to consume indexer events, so that `DiscoveryEngine` no longer runs its own filesystem scan and `Builder::load_all` is driven by classified events.
7. As a maintainer of `crates/template/`, I want my template service to consume indexer events for template-directory files, so that template processing is wired through the same integration seam as note and schema.
8. As a future maintainer adding a new bounded context, I want a documented event contract and a stable sink interface, so that I can add a new downstream consumer without modifying the indexer.
9. As a CLI user, I want a single `sync` command that runs the indexer and then drives all downstream context processing in one orchestrated pass, so that one invocation refreshes the entire derived state of the vault.
10. As a CLI user, I want per-context reports inside the unified sync report, so that I can see how many notes, schemas, and templates were created/updated/deleted in one run.
11. As a CLI user, I want a failure in one context's processing to not silently corrupt another context's state, so that schema parse errors do not prevent note ingestion (or vice versa).
12. As an operator, I want a slow downstream context to not block the indexer from completing its scan, so that incomplete downstream processing does not hold open scanner-side transactions.
13. As an operator, I want the indexer to fail loudly if no downstream consumer is attached to its event sink, so that misconfigured deployments do not silently drop events.
14. As an event-log designer, I want the Phase 2 persistent log to be additive over Phase 1, so that adding restartability does not require changing event types, consumer code, or app orchestration.
15. As a future file-watcher maintainer, I want the consumer side designed against an event stream rather than against a one-shot scan, so that pushing watcher events into the same consumers requires no consumer changes.
16. As a test author, I want pure per-event handler logic separated from channel mechanics, so that I can unit-test "given event X, dispatch Y" without spinning up threads or channels.
17. As a test author, I want the channel transport primitive testable in isolation against a hand-rolled receiver, so that I can verify drain semantics without involving any context.
18. As a domain reviewer, I want event names that follow the indexer CONTEXT vocabulary (Index Record, Index Status, Deleted Record), so that storage-domain terminology remains stable and wire-domain terminology stays clearly distinguished.
19. As a reviewer of `crates/indexer/`, I want the wire-layer `*IndexEvent` types distinct from the storage-layer `*Record` types, so that "event" and "record" describe different invariants in the codebase.
20. As a performance-focused engineer, I want indexer reads in `find_file_by_path` and `find_dir_by_path` to use `rkyv::access` rather than full `rkyv::from_bytes` materialisation, so that hot deletion-detection paths skip unnecessary deserialisation.
21. As a maintainer of `crates/indexer/`, I want a single `FS_ID_BY_PATH` table replacing `FILE_ID_BY_PATH` and `DIR_ID_BY_PATH`, so that path-lookup paths and deletion detection traverse one table instead of two.
22. As a maintainer of `crates/indexer/`, I want a new `find_id_by_path` returning `(FsRecordId, FsRecordKind)`, so that deletion detection no longer needs to consult the primary `FILES` / `DIRS` tables to discover kind.
23. As a maintainer of `crates/schema/`, I want my consumer free to buffer events until `ScanCompleted` and then run `Builder::load_all` against the accumulated set, so that the schema's batched processing model is not forced into a per-event shape.
24. As an architecture reviewer, I want the per-context consumer logic, dispatch logic, and report aggregation to live inside each context crate, so that `crates/app/` does not become a routing monolith that knows about every processor.
25. As an architecture reviewer, I want `crates/app/` to know only how to wire channels, spawn consumer threads, and assemble a top-level `SyncReport`, so that adding a new context to the sync pipeline is a one-line orchestration change.
26. As a future maintainer, I want the indexer's sink trait to be implementable by alternative back-ends (in-memory fan-out, persistent log, tee-to-stdout for debugging), so that observability and persistence concerns can be plugged in without changing the indexer's emit path.
27. As a debugging engineer, I want a `ScanCompleted` event terminator on the stream, so that consumers can deterministically know when to flush batched work and exit cleanly.

## Implementation Decisions

### 1. Naming and Vocabulary

Wire-domain names follow the suffix **`Event`**. Storage-domain names keep their existing suffixes (`Record`, `Status`, `Scope`). Aggregate run-summary names keep their existing suffixes (`Result`, `Report`).

| Layer            | Suffix | Examples                                              |
| ---------------- | ------ | ----------------------------------------------------- |
| Wire (per-item)  | Event  | `FileIndexEvent`, `DirIndexEvent`, `DeletedRecordEvent`, `IndexEvent` |
| Storage (per-row) | Record | `FileRecord`, `DirRecord`, `FsRecordId`, `FsRecordKind` |
| Run-level summary | Result / Report | `IndexResult`, `IndexReport`, `SyncReport`, `NoteSyncReport` |

Renames:

- `crates/indexer/src/entry.rs` → `crates/indexer/src/event.rs`
- `FileIndexEntry` → `FileIndexEvent`
- `DirIndexEntry` → `DirIndexEvent`
- `IndexedNodes` → `IndexedEvents`
- `DeletedNodes` → `DeletionEvents`

`IndexStatus`, `IndexResult`, `IndexReport`, `IndexScope`, `IndexOptions` are unchanged — they are not per-item wire types.

Indexer CONTEXT.md will gain a glossary entry for **Index Event** ("a single classified item flowing through the sink") alongside the existing Index Record entry.

### 2. Event Types

The `IndexEvent` enum lives in `crates/indexer/src/event.rs` and is the wire-domain envelope flowing through any sink implementation:

```rust
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum IndexEvent {
    FileIndexed(FileIndexEvent),
    DirIndexed(DirIndexEvent),
    FileDeleted(DeletedRecordEvent),
    DirDeleted(DeletedRecordEvent),
    ScanCompleted(IndexReport),
}
```

`FileIndexEvent` and `DirIndexEvent` retain the shape of the existing `*IndexEntry` structs (record + path + status). They become public-domain wire types and gain a `pub fn status() -> IndexStatus` (currently `pub(crate)`).

`DeletedRecordEvent` is new and minimal:

```rust
pub struct DeletedRecordEvent {
    id: FsRecordId,
    path: PathKey,
}
```

The deletion event deliberately omits the full `FileRecord` / `DirRecord` payload. Downstream consumers need `(FsRecordId, PathKey)` to remove their derived state; carrying the rest of the record wastes channel bandwidth and forces unnecessary cloning. The variant on `IndexEvent` (`FileDeleted` vs `DirDeleted`) carries the kind discriminant.

`ScanCompleted` carries an owned `IndexReport`. Consumers use it as the deterministic terminator on the stream (drive any deferred batched work, then exit).

### 3. Sink Trait and In-Memory Fan-Out

The sink abstraction lives in `crates/indexer/src/sink.rs`:

```rust
pub trait IndexEventSink: Send {
    fn emit(&mut self, event: IndexEvent) -> Result<(), IndexerError>;
}
```

The Phase 1 implementation is a blocking clone-and-broadcast fan-out backed by `crossbeam_channel`:

```rust
pub struct FanoutSink {
    senders: Vec<crossbeam_channel::Sender<IndexEvent>>,
}

impl IndexEventSink for FanoutSink {
    fn emit(&mut self, event: IndexEvent) -> Result<(), IndexerError> {
        for sender in &self.senders {
            sender.send(event.clone())
                  .map_err(|_| IndexerError::EventSinkDisconnected)?;
        }
        Ok(())
    }
}
```

Channels are bounded (default 1024 slots per consumer) so a slow consumer back-pressures the indexer without unbounded memory growth. `IndexerError::EventSinkDisconnected` is added as a new variant — disconnection during a run is fatal, since the indexer cannot tell whether downstream state is now inconsistent.

`crossbeam_channel` is the chosen channel implementation. It is already in `Cargo.lock` transitively (via `redb`), has a richer API than `std::sync::mpsc` (`select!`, `recv_timeout`, cloneable senders), and clearer disconnect semantics. No new top-level dependency on `tokio` is introduced; sync threads are sufficient for Phase 1 and Phase 2.

### 4. Indexer Service Integration

`IndexerService` gains an optional sink and a builder method:

```rust
pub struct IndexerService {
    // existing fields ...
    event_sink: Option<Box<dyn IndexEventSink>>,
}

impl IndexerService {
    pub fn with_event_sink(mut self, sink: Box<dyn IndexEventSink>) -> Self { ... }
}
```

Emit points are added inside the existing scan loop and deletion-detection pass:

- After each file classification: `IndexEvent::FileIndexed(FileIndexEvent::new(record, path, status))`.
- After each directory classification: `IndexEvent::DirIndexed(DirIndexEvent::new(record, path, status))`.
- Inside `detect_deletions`, when a missing path resolves to a file: `IndexEvent::FileDeleted(DeletedRecordEvent::new(id, path))`.
- Inside `detect_deletions`, when a missing path resolves to a dir: `IndexEvent::DirDeleted(DeletedRecordEvent::new(id, path))`.
- After report construction, before returning: `IndexEvent::ScanCompleted(report.clone())`.

When `event_sink` is `None`, emit is a no-op. Existing callers (`run_index` in app, indexer unit tests) continue to work unchanged.

### 5. Storage Consolidation (Precursor)

The integration depends on a storage refactor that is large enough to track as its own implementation issue but small enough to ship as a precursor before consumer work begins:

- Replace `FILE_ID_BY_PATH` and `DIR_ID_BY_PATH` with a single `FS_ID_BY_PATH` table.
- The new table is keyed by `PathKey` (via `DbPathKey`) and valued by `(FsRecordId, FsRecordKind)`. `FsRecordKind` is a new enum `{ File, Dir }` in `crates/indexer/src/model.rs`, sized for redb-friendly storage (e.g. tagged byte).
- Add a new repository method `find_id_by_path(p: &PathKey) -> Option<(FsRecordId, FsRecordKind)>` that hits only `FS_ID_BY_PATH`. This is the workhorse for `detect_deletions`: iterate `FS_ID_BY_PATH`, for each path not in `seen_paths`, push the id into the right deletion bucket by kind. No reads of `FILES` or `DIRS` during deletion detection.
- Rewrite `find_file_by_path` and `find_dir_by_path` internals to chain through `find_id_by_path`, then read the primary table.
- Switch primary-table reads (`find_file`, `find_dir`, and the by-path methods above) from `rkyv::from_bytes` to `rkyv::access` followed by `rkyv::deserialize`. This matches the pattern used by `note`, `schema`, `template`, and `vault` storage layers (each goes through `traces_db::ArchivedEntity::access`, which wraps `rkyv::access`). The indexer uses bare `rkyv::access` directly because redb yields contiguous slices that do not need the `ArchivedEntity` alignment-buffering ceremony.
- A future zero-copy `view_file_by_path` / `view_dir_by_path` API returning `&ArchivedFileRecord` / `&ArchivedDirRecord` is deferred until a downstream consumer needs field-level access without materialisation.

This precursor lands first and is tracked as its own implementation issue. No consumer-side or app-side work in this PRD depends on the `view_*` API.

### 6. Per-Context Consumer Layering (Hybrid)

The consumer side splits into two cooperating types:

**Transport primitive** — one shared, lives in `crates/indexer/src/sink.rs`:

```rust
pub struct IndexEventStream {
    rx: crossbeam_channel::Receiver<IndexEvent>,
}

impl IndexEventStream {
    pub fn new(rx: Receiver<IndexEvent>) -> Self { ... }

    /// Returns the next event, or None on channel disconnect.
    /// Does NOT auto-stop on ScanCompleted — the consumer chooses
    /// whether to break after handling it.
    pub fn next(&mut self) -> Option<IndexEvent> { ... }
}
```

This primitive owns the channel, knows nothing about contexts, and has one job: produce events until disconnect. It is the shared transport seam.

**Semantic dispatch + state** — one per context, lives in `crates/<context>/src/sync.rs`:

```rust
// crates/note/src/sync.rs
pub struct NoteIndexEventConsumer<'svc> {
    stream: IndexEventStream,
    service: &'svc NoteService,
    report: NoteSyncReport,
}

impl<'svc> NoteIndexEventConsumer<'svc> {
    pub fn new(stream: IndexEventStream, service: &'svc NoteService) -> Self { ... }

    pub fn drain(mut self) -> Result<NoteSyncReport, NoteSyncError> {
        while let Some(event) = self.stream.next() {
            let done = matches!(event, IndexEvent::ScanCompleted(_));
            self.handle(&event)?;
            if done { break; }
        }
        Ok(self.report)
    }

    fn handle(&mut self, event: &IndexEvent) -> Result<(), NoteSyncError> { ... }
}
```

The per-context consumer owns:
- The stream
- A reference to its service
- Its own report accumulator
- Any per-context state (buffering, watermarks in Phase 2, retry budgets)
- Its own filter rules and dispatch logic

Each context is free to differ. Schema's consumer accumulates `FileIndexEvent`s into a private `Vec` and calls `Builder::load_all` on `ScanCompleted` rather than dispatching per-event. Note dispatches per-event. Template will be decided when wired (likely per-event, like note). The shared `IndexEventStream` does not care.

**Backup options considered and rejected for the PRD baseline** (kept here as fallbacks in case the hybrid hits an unforeseen issue during implementation):

- **Option A — single shared `IndexEventConsumer` in `crates/indexer/`** that takes a closure per event. Strongest on uniformity and trivially testable in isolation, but leaks schema's buffering pattern into `SchemaService::sync` and forces watermark abstractions into a shared type that not every context needs. Selected for the transport sub-layer (`IndexEventStream`) only.
- **Option B — fully per-context consumer struct, no shared primitive**. Strongest on per-context customisation but duplicates drain-loop boilerplate three times and couples every context to channel-mechanism changes. Selected for the dispatch sub-layer (`<Context>IndexEventConsumer`) only.

The hybrid takes Option A's win on transport reuse and Option B's win on per-context state ownership. If during implementation the per-context consumers turn out to have nothing differentiating them (filters become identical, no batching, no watermarks), collapsing to pure Option A is mechanical.

### 7. Per-Context Service Layer

Each downstream context exposes a `<Context>Service` as its orchestration root. The service owns config, source reader, repository, and the sync entry point:

```rust
// crates/note/src/service.rs (example shape)
pub struct NoteService {
    config: Arc<Config>,
    source: FileReader,
    repository: Arc<dyn Repository + Send + Sync>,
}

impl NoteService {
    pub fn new(config: Arc<Config>, source: FileReader, repository: Arc<dyn Repository + Send + Sync>) -> Self { ... }

    pub fn sync(&self, rx: Receiver<IndexEvent>) -> Result<NoteSyncReport, NoteSyncError> {
        NoteIndexEventConsumer::new(IndexEventStream::new(rx), self).drain()
    }

    pub(crate) fn handle_event(&self, event: &IndexEvent) -> Result<Option<NoteSyncOutcome>, NoteSyncError> {
        // dispatch by event variant and IndexStatus, returning per-item outcome
    }

    pub(crate) fn matches(&self, event: &FileIndexEvent) -> bool {
        // filter: is this event for this context?
    }
}
```

`TemplateService` already exists and will be extended with `sync(rx)`. `NoteService` and `SchemaService` are new and may be introduced as part of this PRD's implementation. Each service's `handle_event` is the per-event dispatch surface that the consumer calls back into; keeping `handle_event` on the service (not on the consumer) means processor-aware logic lives next to processor invocation, and the consumer stays a pure channel-drain shell.

For schema specifically, `SchemaService::handle_event` does not invoke the processor directly — it appends to an internal buffer. `SchemaIndexEventConsumer::drain` triggers `SchemaService::flush()` (or equivalent) on `ScanCompleted`, which runs `Builder::load_all` against the buffered set. This keeps schema's batched processing model intact without leaking into the shared transport.

### 8. Processor Refactor — `IndexStatus`-Keyed Entry Points

Each downstream processor currently begins with a Discovery stage (repo lookup) followed by a Comparison stage (metadata comparison) before branching into work. Both stages duplicate what the indexer already did. The processors gain new constructors that enter the pipeline past those stages, parameterised on the `IndexStatus` carried by the event:

```rust
// crates/note/src/processor.rs (illustrative)
impl NoteProcessor {
    // existing entry point retained for ad-hoc / test callers
    pub fn new() -> NoteProcessor<Discovery, Unknown> { ... }
    pub fn process_file(self, ...) -> Result<NoteProcessReport, _> { ... }

    // new entry points for event-driven callers
    pub fn from_new(repo: &impl Repository, config: &Config, source: &FileReader, event: &FileIndexEvent)
        -> Result<NoteProcessReport, NoteProcessError>;
    pub fn from_stale(repo: &impl Repository, config: &Config, source: &FileReader, event: &FileIndexEvent)
        -> Result<NoteProcessReport, NoteProcessError>;
}
```

`Fresh` events are never passed to a processor — the service drops them with no work. `New` and `Stale` events each enter the pipeline at the correct branch (Construction-from-scratch or Analysis-then-Construction respectively).

The same shape applies to `crates/schema/src/property_bank_processor.rs` and `crates/schema/src/schema_processor.rs`. Schema's existing `PropertyBankProcessor::from_discovery` already accepts a typed entry point; the change is to add `from_new` and `from_stale` siblings (or repurpose the existing one) that carry the indexer's status forward instead of repeating Comparison work.

The original `process_file` / `from_discovery` constructors remain in place. They are still useful for tests that want to drive the full pipeline from raw inputs, and for any future ad-hoc caller that doesn't run through the indexer.

### 9. App Orchestration

`crates/app/src/sync.rs` introduces `run_sync` as the top-level entry point. Its responsibilities are limited to:

- Open the redb `Store` at the configured cache path.
- Construct `FileReader` from the vault root.
- Construct one `NoteService`, one `SchemaService`, one `TemplateService` from the shared store and config.
- Create one bounded `crossbeam_channel` per service (default capacity 1024).
- Construct a `FanoutSink` over the sender halves.
- Spawn one OS thread per service, running `service.sync(receiver)`.
- Construct the `IndexerService`, attach the fan-out sink, run the scan.
- Join all consumer threads, collect their reports, aggregate into a single `SyncReport`.

`SyncReport` is a flat struct:

```rust
pub struct SyncReport {
    pub index: IndexReport,
    pub note: NoteSyncReport,
    pub schema: SchemaSyncReport,
    pub template: TemplateSyncReport,
}
```

The app crate gains direct dependencies on `traces-note`, `traces-schema` (in addition to its existing `traces-template`, `traces-indexer`). The new `traces-note`, `traces-schema`, `traces-template` each gain a dependency on `traces-indexer` (for `IndexEvent` types and `IndexEventStream`). These dependencies are intentional: they encode the "indexer → downstream consumer" direction that CONTEXT.md describes.

### 10. Vault Deletion

Once each downstream context's service exists and is integrated:

- Rewrite `crates/note/tests/note_reader.rs` to drive fixtures through `NoteService::sync` (or directly through `NoteProcessor::from_new` / `from_stale` for the per-event test cases). The integration tests do not need to go through the indexer at all — they can construct synthetic `IndexEvent`s.
- Remove the `traces-vault` dev-dependency from `crates/note/Cargo.toml`.
- Fix the two doc-comment references in `crates/db/src/table.rs` that mention `traces_vault::model::FileId`.
- Remove `traces-vault` from the workspace `Cargo.toml`.
- Delete `crates/vault/`.

Vault deletion is a separate implementation issue inside this integration workstream but is no longer blocked — it lands once note's tests are rewritten, which is itself a small change once `NoteService::sync` is testable.

### 11. Error Policy

Per-event errors inside a single context are recorded in that context's `<Context>SyncReport` and processing continues. A schema parse failure on one file does not stop note processing on another file. Each consumer accumulates its own per-item failure list, exposed through its report.

Fatal errors (repository write failures, channel disconnect, indexer scanner failures) abort the affected branch. The fan-out sink returns `IndexerError::EventSinkDisconnected` if any sender is dropped, which aborts the indexer's scan loop. Consumer thread panics are surfaced through `JoinHandle::join` in `run_sync` and converted to `AppError`.

Each context owns its own per-event error policy inside its consumer's `handle`. Schema may choose to abort its buffer on the first malformed schema (since downstream property bank processing depends on a consistent set); note continues past per-file parse errors. This policy lives in the consumer, not in the shared transport.

### 12. Phase 2 — Persistent Event Log (Deferred to Follow-Up)

Phase 2 introduces an additive persistence layer behind the existing `IndexEventSink` trait. Nothing on the consumer side, indexer-emit side, event types, or app orchestration changes. The deltas are:

- A new `INDEX_EVENTS` table in `crates/indexer/src/storage/tables.rs`, keyed by a monotonic `EventId` (u64) with rkyv-serialised `IndexEvent` values.
- A new `EVENT_SEQUENCES` single-row table (or per-context sub-row) tracking the next `EventId` to allocate.
- A new sink implementation `LoggedFanoutSink` that wraps `FanoutSink`: each `emit` first appends to `INDEX_EVENTS` inside a redb write transaction, then forwards to the in-memory fan-out senders.
- Per-consumer watermark tables — one per downstream context, owned by that context's storage layer (e.g. `NOTE_INDEX_WATERMARK` in `crates/note/src/storage/tables.rs`). Each consumer reads its watermark on startup, skips events with `EventId ≤ watermark`, and updates the watermark after each successfully processed event.
- Replay support inside each `<Context>IndexEventConsumer`: instead of (or in addition to) reading from `IndexEventStream`, the consumer can iterate `INDEX_EVENTS` directly from a starting `EventId`.

The two-phase split exists because Phase 1 has self-contained value (vault dies, downstream contexts share filesystem scan work) and Phase 2 adds complexity that should be motivated by a concrete restartability requirement. Phase 2 is tracked as a follow-up issue, not as part of this PRD's acceptance.

### 13. Out-of-Scope Adjacent Refactors

- The `view_file_by_path` / `view_dir_by_path` zero-copy API (returning `&ArchivedFileRecord` lifetime-bound to a redb transaction) is deferred. No current consumer needs it. The Phase 1 internal switch to `rkyv::access` + `rkyv::deserialize` gives most of the win without exposing transaction-bound borrows.
- Migrating template's existing service to the new shape, beyond adding the `sync(rx)` entry point, is out of scope. The template service stays as it is.
- Async runtime adoption is out of scope. `crossbeam_channel` + OS threads cover the volume needed for Phase 1 and Phase 2.
- File watcher integration is out of scope but explicitly designed around. Watcher events would feed into the same per-context consumers as `IndexEvent`s, either by mapping watcher events into `IndexEvent` or by extending the sink trait.

## Testing Decisions

Good tests in this codebase assert externally observable behaviour and domain invariants, not private implementation order. Pure dispatch logic is tested by feeding canned events and asserting outcomes. End-to-end flow is tested by running the real indexer against a tempdir-backed vault, attaching real consumers backed by `TestDb`, and asserting that all three context reports reflect the expected state.

Tests will be authored TDD-style (red-green-refactor cycle) during implementation; the list below is the planned coverage surface, not the implementation order.

### Modules to test

**Indexer crate**

- `IndexEvent` constructors and field accessors — round-trip correctness for `FileIndexEvent`, `DirIndexEvent`, `DeletedRecordEvent`.
- `FanoutSink::emit` — clones each event to every registered sender; returns `EventSinkDisconnected` if any sender is closed.
- `IndexEventStream::next` — yields events in send order; returns `None` on channel disconnect; multiple `next()` calls after disconnect remain `None`.
- `IndexerService` emit-point integration — running a scan against a `MockScanner` with a captured-sender sink produces the expected sequence of events (file/dir indexed, file/dir deleted, scan completed) in the expected order.
- Storage precursor: `find_id_by_path` correctness on hand-seeded `FS_ID_BY_PATH` rows; `find_file_by_path` and `find_dir_by_path` correctness through the new `rkyv::access` path against existing contract tests.

**Per-context (`note`, `schema`, `template`)**

- `<Context>Service::matches` — filter predicate against fixture events covering format match, format non-match, template-directory exclusion (for note), schema-directory inclusion (for schema), and template-directory inclusion (for template).
- `<Context>Service::handle_event` — per-event dispatch correctness against hand-built events. One test per `IndexStatus` (`New`, `Stale`, `Fresh`), one test per deletion variant, one test for events the context should ignore.
- `<Context>IndexEventConsumer::drain` — end-to-end on a hand-rolled `crossbeam_channel::unbounded()` feeding canned events including a terminating `ScanCompleted`. Asserts the returned report reflects the dispatched events.
- Schema-specific: consumer buffering behaviour — events accumulate; `flush` runs only on `ScanCompleted`; buffer is empty after drain returns.
- Processor entry points: `NoteProcessor::from_new` and `from_stale` (and schema equivalents) produce the same observable result as the existing `process_file` path when fed equivalent inputs. These are direct comparisons, not integration tests.

**App crate**

- `run_sync` end-to-end against a tempdir vault containing fixture markdown, schema, and template files. Uses `TestDb` for the shared store. Asserts that:
  - The returned `SyncReport.index` matches what `run_index` alone would have produced.
  - `SyncReport.note` reflects markdown files processed (created/updated/deleted counts).
  - `SyncReport.schema` reflects schemas processed.
  - `SyncReport.template` reflects template files processed.
  - A subsequent `run_sync` with no filesystem changes produces zero per-context work (all `Fresh`).
- `run_sync` failure-mode tests: indexer error aborts the run; consumer thread panic surfaces as `AppError`.

**Note integration test rewrite**

- `crates/note/tests/note_reader.rs` rewritten to drive fixtures through `NoteService::sync` (passing synthetic `IndexEvent`s) instead of `VaultProcessor::process_full`. All seven existing test functions retained; the fixture-construction helpers (`build_fixture`, `build_environment`) re-implemented against the new service.

### Prior art for test patterns

- `crates/indexer/src/service.rs` already has a comprehensive `MockScanner` and capturing-port pattern for testing the indexer pipeline. The fan-out sink tests follow the same shape: a captured `Sender` or pair of senders feeding into hand-rolled `Receiver`s assert what reaches them.
- `crates/schema/src/discovery.rs` test module already exercises a `CountingReadRepo` pattern for asserting repository call counts. Per-context consumer tests will use the same approach where a test needs to verify dispatch frequency.
- `crates/note/src/processor.rs` typestate pipeline tests show how to construct fixtures around `NoteFileInfo` and `FileMetadata`. The same fixture helpers extend to per-event tests for `NoteService::handle_event` once `IndexEvent` is in scope.
- `crates/template/` already exposes a `TemplateService` with an integration-test pattern that the new `NoteService` and `SchemaService` can mirror.
- `traces_db::testing::TestDb` is the standard in-memory store wrapper used across `note`, `schema`, `template` integration tests and will be the same store backing `run_sync` tests.

## Out of Scope

- Phase 2 persistent event log and per-consumer watermarks. Tracked as a follow-up implementation issue against this PRD's foundation.
- Zero-copy `view_file_by_path` / `view_dir_by_path` API returning `&ArchivedFileRecord`. Deferred until a downstream consumer needs field-level access without materialisation.
- Async runtime (`tokio`, `async-std`) introduction. Phase 1 and Phase 2 use `crossbeam_channel` and OS threads.
- File watcher integration. The event stream is designed to accommodate watcher events later, but no watcher work is part of this PRD.
- Parallel execution of items inside a single context. Each consumer thread processes its filtered events sequentially. Internal parallelism (rayon, work-stealing) is a per-context optimisation deferred until a context's throughput becomes the bottleneck.
- Cross-context coordination (e.g. "wait for all schemas to commit before any note runs"). Each consumer runs independently. If ordering between contexts becomes required, it will be added as a separate orchestration concern.
- Migrating the template context's existing service shape, beyond adding a `sync(rx)` entry point.
- Changing the CLI surface. A `traces sync` CLI command can be added in a follow-up; this PRD is library-surface only.

## Further Notes

- Each downstream context's CONTEXT.md will gain a "Consumes" entry pointing at `traces_indexer::IndexEvent` to make the data flow explicit at the context boundary.
- The indexer's CONTEXT.md will gain an Interface entry for `IndexEventSink` and a glossary entry for **Index Event**.
- After Phase 1 ships, the `.scratch/filesystem-indexer/foundation/` folder will hold the original indexer PRD and issues 01-06; `.scratch/filesystem-indexer/integration/` will hold this PRD and its implementation issues. Issue 07 (vault deletion) folds into integration as a leaf implementation issue.
- The hybrid consumer design intentionally allows collapsing to a simpler single-consumer-trait design if per-context customisation never materialises in practice. Re-evaluate after Phase 1 ships and again after Phase 2 ships.
- The `IndexEventSink` trait is the single extension point for alternative back-ends. A `TeeSink` that fans out to multiple sinks (e.g. `FanoutSink` plus a JSON tracing sink) is trivially implementable without changing the indexer.
