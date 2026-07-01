# PRD: Indexer → Context Processor Integration

**Status**: ready-for-agent
**Created**: 2026-06-30
**Builds on**: `.scratch/filesystem-indexer/foundation/PRD.md` (foundation — `crates/indexer/` already implemented)
**Storage precursor**: `.scratch/filesystem-indexer/foundation/07-storage-consolidation.md` (relocated out of this PRD; lands first)
**Id-unification precursor**: `.scratch/filesystem-indexer/foundation/08-id-unification.md` (context `*Id` → `FsRecordId`; implements ADR `docs/adr/indexer/0001`; depends on 07; lands before the schema section of this PRD — see §5.1)
**Supersedes**: `.scratch/filesystem-indexer/07-vault-deletion.md` (vault deletion folds into this work)

---

## Problem Statement

The `crates/indexer/` context now owns filesystem scanning, metadata comparison, and Fresh/Stale/New classification — but its output is consumed by nothing. Every downstream context (`crates/note/`, `crates/schema/`, `crates/template/`) still re-implements its own filesystem scan, its own metadata comparison, and its own freshness classification before doing the work that genuinely belongs to it (markdown parsing, schema resolution, template rendering).

Today that orchestration lives in `crates/vault/src/processor.rs`. `VaultProcessor::process_full` runs its own `DirScanner`, compares against its own `FileView` / `DirView` tables, then routes markdown files into `NoteProcessor` one at a time. Schema's `DiscoveryEngine` runs a parallel `DirScanner` against the same vault root. Neither uses the indexer.

The vault module is dead code in production (zero non-test callers in `crates/cli/`, `crates/app/`, or any downstream context) but cannot be deleted: `crates/note/tests/note_reader.rs` depends on `VaultProcessor::process_full` as its sole fixture-setup mechanism for all nine integration tests.

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
6. As a maintainer of `crates/schema/`, I want my schema ingestion to consume indexer events, so that `DiscoveryEngine` no longer runs its own filesystem scan and the schema service drives `BaseSchemaProcessor` from classified events (its `IndexStatus`) instead of re-scanning.
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
23. As a maintainer of `crates/schema/`, I want my consumer free to buffer events until `ScanCompleted` and then flush the accumulated set through the schema service, so that the schema's batched processing model is not forced into a per-event shape.
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

Renames (wire-domain per-item types only):

- `crates/indexer/src/entry.rs` → `crates/indexer/src/event.rs`
- `FileIndexEntry` → `FileIndexEvent`
- `DirIndexEntry` → `DirIndexEvent`

`IndexStatus`, `IndexResult`, `IndexReport`, `IndexScope`, `IndexOptions` are unchanged — they are not per-item wire types.

`IndexedNodes` and `DeletedNodes` (`crates/indexer/src/summary.rs:63,100`) are **aggregate/summary types** held by `IndexResult`, not per-item wire types — they keep their existing names. (A prior draft renamed them to `IndexedEvents` / `DeletionEvents`; that was dropped because it dragged storage-aggregate vocabulary into the wire domain, violating the table above, and because `DeletedNodes` carries only `Box<[FsRecordId]>` with no events at all.)

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

`FileIndexEvent` and `DirIndexEvent` retain the shape of the existing `*IndexEntry` structs (record + path + status). They become public-domain wire types and gain a `pub fn status() -> IndexStatus` (currently `pub(crate)`, `crates/indexer/src/entry.rs:75,132`).

Additionally, `FileRecord::metadata()` and `DirRecord::metadata()` (`crates/indexer/src/model.rs:146,234`) flip from `pub(crate)` to `pub`. The schema consumer reconstructs a `traces_fs::FileNode` from a buffered `FileIndexEvent` via `FileNode::new(event.path().clone(), event.node().metadata().clone())` (`crates/fs/src/entry.rs:190`) to feed `BaseSchemaProcessor::from_discovery`, which requires the metadata accessor to be public.

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

**Note on `detect_deletions` (`crates/indexer/src/service.rs:221-249`)**: today it *batches* — it returns a `DeletedNodes` of ids only and discards each path (the `path` at service.rs:229 is in scope but not retained). Emitting per-path `DeletedRecordEvent { id, path }` therefore requires restructuring the deletion pass to emit per-path, not just adding an emit call. Foundation issue 07's `find_id_by_path` makes `(id, kind, path)` cheaply available at that point so this restructuring stays small.

**Note on `ScanCompleted` and error paths**: `run` builds the report and emits `ScanCompleted` only at the very end. On a scanner error (service.rs:98, 132) or a fatal repository error, `run` returns *before* the terminator. Consumers that block waiting for `ScanCompleted` must not hang on an aborted scan — the abnormal-exit path is **disconnect-driven**: when the sink is dropped, `IndexEventStream::next()` returns `None`, which consumers treat as a terminal signal (drain any buffered work, then exit). The `ScanCompleted` terminator is the *normal*-path deterministic flush signal; channel disconnect is the *abnormal*-path one. Both must be handled by every consumer's drain loop.

When `event_sink` is `None`, emit is a no-op. Existing callers (`run_index` in app, indexer unit tests) continue to work unchanged.

### 5. Storage Consolidation (Precursor)

The integration depends on a storage refactor that has been **moved out of this PRD** and tracked as a foundation issue: `.scratch/filesystem-indexer/foundation/07-storage-consolidation.md`. The indexer storage layer is foundation-owned and the refactor touches only `crates/indexer/` internals (zero external consumers), so it lands and merges independently, before any consumer work in this PRD begins.

Summary of what that foundation issue covers (see it for full detail):

- Replace `FILE_ID_BY_PATH` and `DIR_ID_BY_PATH` with a single `FS_ID_BY_PATH` table.
- **The table is keyed by `(PathKey, FsRecordKind)` (via `DbPathKey`) and valued by `FsRecordId`** — *not* keyed by `PathKey` alone valued by `(FsRecordId, FsRecordKind)`. Keying by `(PathKey, FsRecordKind)` preserves today's per-kind path uniqueness (a file and a dir may share a path), so the existing repository contract test (`crates/indexer/src/storage/contract.rs:67-75`, which stores one path in both tables) passes unchanged. A `PathKey`-only key would have made uniqueness global and broken that contract. `FsRecordKind` is a new enum `{ File, Dir }` in `crates/indexer/src/model.rs`.
- Add `find_id_by_path(p: &PathKey) -> Option<(FsRecordId, FsRecordKind)>`. This is the workhorse for `detect_deletions`: iterate the table, for each path not in `seen_paths`, push the id into the right deletion bucket by kind. No reads of `FILES` or `DIRS` during deletion detection.
- Rewrite `find_file_by_path` / `find_dir_by_path` to chain through `find_id_by_path`; switch primary-table reads from `rkyv::from_bytes` to `rkyv::access` + `rkyv::deserialize`; update `all_paths` to iterate the single table while still returning distinct `PathKey`s.
- A future zero-copy `view_file_by_path` / `view_dir_by_path` API is deferred until a downstream consumer needs field-level access without materialisation.

No consumer-side or app-side work in this PRD depends on the `view_*` API. The only dependency is that foundation issue 07 lands first.

### 5.1 Context Id Unification (Precursor — Foundation Issue 08)

The schema section (§6/§7/§8) depends on a second foundation refactor: **unifying the per-context identity types onto `FsRecordId`**. This is an already-accepted architecture decision — ADR `docs/adr/indexer/0001-fileid-as-universal-identity.md` — re-expressed against issue 07's `FsRecordId` / `FS_ID_BY_PATH` names. It is tracked as foundation issue `.scratch/filesystem-indexer/foundation/08-id-unification.md` (depends on issue 07).

What it covers:

- Delete the per-context id newtypes (`SchemaId` at `crates/schema/src/identifier.rs:52`; the template and note equivalents) and replace every use site with `FsRecordId`. This re-keys schema's inheritance graph (`InheritanceGraph<()>` topo order, `crates/schema/src/discovery.rs:386`), the schema index (`crates/schema/src/index.rs`), and all name↔id / path↔id maps onto `FsRecordId`.
- Route every `*_id_by_path` lookup through the consolidated `FS_ID_BY_PATH` table from issue 07, so a path resolves to exactly one `FsRecordId` regardless of context.
- **Identity becomes derived-from-file**: a schema/template/note's identity *is* its file's `FsRecordId`. There is no independent `SchemaId::new()` (`crates/schema/src/aggregate.rs:62`, `crates/schema/src/base_processor.rs:317`) minting a fresh id — the processor adopts the `FsRecordId` the indexer already assigned and carries on the `FileIndexEvent`. One file → one schema, identity flowing from the indexer.

**Why this is a hard prerequisite for the schema section**: schema deletion (§6/§8) is driven purely by indexer `FileDeleted`/`DirDeleted` events, which carry `(FsRecordId, PathKey)`. Because a schema's identity *is* its `FsRecordId`, the delete path (`repo.delete_base_schema(id)` + `repo.delete_schema(id)`, `crates/schema/src/base_processor.rs:64-66`) reads the id straight off the event with no `PathKey → SchemaId` repository lookup, and schema's own topological-graph deletion pass (`detect_deleted_schemas`, `crates/schema/src/discovery.rs:385-395`) is removed entirely. Without unification, schema would have to keep a path→id repo query on every deletion. This is a foundation-storage change spanning schema/note/template — it is deliberately **not** described inline in the schema sections; those sections assume issue 08 has landed.

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

Each context is free to differ. Schema's and template's consumers accumulate their filtered `FileIndexEvent`s (plus `FileDeleted`/`DirDeleted`) into a private buffer and flush the accumulated set through their service on `ScanCompleted`, rather than dispatching per-event. Note dispatches per-event. The shared `IndexEventStream` does not care.

**Schema buffer shape.** Schema's consumer buffers into an `IndexedSchemaSet`:

```rust
// crates/schema/src/sync.rs
pub struct IndexedSchemaSet {
    /// The single file matching `spec.property_bank_file_path()`, if present.
    property_bank: Option<FileIndexEvent>,
    /// The remaining schema-dir files, each carrying its IndexStatus.
    schemas: Vec<FileIndexEvent>,
    /// Deletion events for schema-dir paths (id + path).
    deleted: Vec<DeletedRecordEvent>,
}
```

During the scan the consumer appends raw `FileIndexEvent`s; the property-bank split runs **once at flush**, not per event, mirroring today's `separate_property_bank` (`crates/schema/src/discovery.rs:211-250`) by comparing each buffered path against `spec.property_bank_file_path()`. The struct carries **events only** — the inheritance graph and per-schema `RawSchemaView`s are **not** buffered; they are read from schema's own repository at flush (see §7).

**Template buffer shape.** Template's consumer buffers its filtered template-dir `FileIndexEvent`s and deletion events and, on `ScanCompleted`, runs a `process_all`-equivalent over the accumulated set (see §7). Batched-on-flush is the natural fit because template's orphan detection (`identify_deleted_template_paths`, `crates/template/src/service.rs:449`) is a set-difference over the *whole* discovered path set against cached paths — it structurally needs the complete set, exactly like schema.

**Backup options considered and rejected for the PRD baseline** (kept here as fallbacks in case the hybrid hits an unforeseen issue during implementation):

- **Option A — single shared `IndexEventConsumer` in `crates/indexer/`** that takes a closure per event. Strongest on uniformity and trivially testable in isolation, but leaks schema's buffering pattern into `SchemaService::sync` and forces watermark abstractions into a shared type that not every context needs. Selected for the transport sub-layer (`IndexEventStream`) only.
- **Option B — fully per-context consumer struct, no shared primitive**. Strongest on per-context customisation but duplicates drain-loop boilerplate three times and couples every context to channel-mechanism changes. Selected for the dispatch sub-layer (`<Context>IndexEventConsumer`) only.

The hybrid takes Option A's win on transport reuse and Option B's win on per-context state ownership. If during implementation the per-context consumers turn out to have nothing differentiating them (filters become identical, no batching, no watermarks), collapsing to pure Option A is mechanical.

### 7. Per-Context Service Layer

Each downstream context exposes a `<Context>Service` as its orchestration root. The service owns config, source reader, repository, and the sync entry point:

The config type is `traces_settings::aggregate::AppConfig` (note's `process_file` takes `&AppConfig`, `crates/note/src/processor.rs:297`; schema's `Builder` holds `&'config AppConfig`, `crates/schema/src/builder.rs:21`, and `SchemaService` carries the same). There is no `Config` newtype. `AppConfig` is `Send + Sync` (confirmed: a plain validated aggregate value with no interior mutability, `crates/settings/src/config/aggregate.rs:29`), so the `Arc<AppConfig>` field crosses a thread boundary safely (see §9 threading).

```rust
// crates/note/src/service.rs (example shape)
pub struct NoteService {
    config: Arc<AppConfig>,
    source: FileReader,
    repository: Arc<dyn Repository + Send + Sync>,
}

impl NoteService {
    pub fn new(config: Arc<AppConfig>, source: FileReader, repository: Arc<dyn Repository + Send + Sync>) -> Self { ... }

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

`NoteService` and `SchemaService` are new and may be introduced as part of this PRD's implementation. **Two dispatch models coexist**: note is *per-event* — its `NoteService::handle_event` is the dispatch surface the consumer calls back into per event, keeping processor-aware logic next to processor invocation while the consumer stays a pure channel-drain shell. Schema and template are *batched-on-flush* — their consumers buffer and call a single `flush`/`sync` entry on `ScanCompleted` (see §7.1, §7.2), so they expose no per-event `handle_event`.

`AppConfig` is confirmed `Send + Sync` — it is a plain validated aggregate value with no interior mutability (`crates/settings/src/config/aggregate.rs:29`; nested `Arc` fields are themselves `Send + Sync`), so the `Arc<AppConfig>` field crosses a thread boundary safely.

#### 7.1 Schema Service — flush model

For schema, `SchemaService` does **not** dispatch per-event. Its consumer buffers into the `IndexedSchemaSet` from §6; on `ScanCompleted`, `SchemaService::flush(IndexedSchemaSet)` owns the orchestration that previously lived inside `Builder::load_all` (`crates/schema/src/builder.rs:70-131`), **minus the `DiscoveryEngine::run` call** — the file set already arrived as events. The flush loop:

1. **Property bank first.** If `property_bank` is present, run `PropertyBankProcessor::from_discovery(bank_file, root)` → `run(view, source, repo)` (`crates/schema/src/builder.rs:154-162`), yielding a `PropertyBankResolution`. The bank's `RawPropertyBankView` is read from schema's own repository. The bank is a *dependency* of schema processing, so it is resolved before the per-schema loop.
2. **Per schema.** For each buffered schema `FileIndexEvent`, adapt it to a `FileNode` (§2) and run `BaseSchemaProcessor::from_new` / `from_stale` (§8), threading `Some(&bank_resolution)`. The per-schema `RawSchemaView` is read from schema's own repository, keyed by path. `Fresh` events never reach the processor — the service drops them.
3. **Deletions.** For each buffered `DeletedRecordEvent`, call the caller-level delete path (`repo.delete_base_schema(id)` + `repo.delete_schema(id)`, `crates/schema/src/base_processor.rs:64-66`) using the `FsRecordId` carried on the event (post-issue-08, that id *is* the schema's identity).

**The seam is the service, not `Builder`.** `SchemaService::flush` calls `PropertyBankProcessor` and `BaseSchemaProcessor` directly; `Builder` is **off the integration path**. The schema context is migrating from the `Builder` design to a service design, so investing a new `Builder::load_from` sibling would prop up a type on its way out. `Builder::load_all` (and the old `SchemaProcessor` it drives via `from_discovery_result`) remains only for whatever legacy/test callers survive until it is removed.

What schema reuses from the indexer is the **filesystem scan + per-file `IndexStatus`** — it stops running `DiscoveryEngine::scan_filesystem` (`crates/schema/src/discovery.rs:184`) entirely (user stories 1, 6). The inheritance graph and cached views still come from schema's own repository; the property-bank split still runs at flush. That is the intended and bounded win.

#### 7.2 Template Service — sync(rx) and threading

`TemplateService<R, W, E>` already exists (`crates/template/src/service.rs:137`) and is extended with a `sync(rx)` entry point. **The generics are kept**; `sync` stays generic under the existing port bounds:

```rust
impl<R, W, E> TemplateService<R, W, E>
where
    R: ReadRepository + WriteRepository,
    W: FileWriter,
    E: TemplateEngine,
{
    pub fn sync(&self, rx: Receiver<IndexEvent>) -> Result<TemplateSyncReport, TemplateSyncError> {
        TemplateIndexEventConsumer::new(IndexEventStream::new(rx), self).drain()
    }
}
```

`sync` takes `&self` (matching `process_all`, `crates/template/src/service.rs:344`, which is `&self` — the engine `E` is touched only by `create()`'s render path, not by ingestion). The **composition root pins the concrete triple** `TemplateService<RedbRepository, Writer, MiniJinjaEngine>` when it constructs and spawns the service; the `Send + 'static` bounds `thread::spawn` requires are satisfied by those concrete types at the spawn site, not written into the generic method. The service is not erased to a concrete type internally — the generics exist for hexagonal testability with in-memory doubles.

Template's consumer buffers template-dir events and runs a `process_all`-equivalent on `ScanCompleted` (§6). This is batched-on-flush, the same shape as schema.

### 8. Processor Refactor — `IndexStatus`-Keyed Entry Points

Each downstream processor currently begins with a Discovery stage (repo lookup) followed by a Comparison stage (metadata comparison) before branching into work. Both stages duplicate what the indexer already did. The processors gain new constructors that enter the pipeline past those stages, parameterised on the `IndexStatus` carried by the event:

```rust
// crates/note/src/processor.rs (illustrative)
impl NoteProcessor {
    // existing entry point retained for ad-hoc / test callers
    pub fn new() -> NoteProcessor<Discovery, Unknown> { ... }
    pub fn process_file(self, ...) -> Result<NoteProcessReport, _> { ... }

    // new entry points for event-driven callers
    pub fn from_new(repo: &impl Repository, config: &AppConfig, source: &FileReader, event: &FileIndexEvent)
        -> Result<NoteProcessReport, NoteProcessError>;
    pub fn from_stale(repo: &impl Repository, config: &AppConfig, source: &FileReader, event: &FileIndexEvent)
        -> Result<NoteProcessReport, NoteProcessError>;
}
```

`Fresh` events are never passed to a processor — the service drops them with no work.

**What the entry points actually skip**: in note's typestate pipeline (`crates/note/src/processor.rs`), the only stages that duplicate the indexer's work are **Discovery** (the `discover()` repo lookup, processor.rs:357) and **Comparison** (`check_metadata()`, processor.rs:390). The `New` / `Changed` status structs (processor.rs:108-122) hold an already-*parsed* `RawNote` — they are post-Analysis states. A `New` event therefore still pays for **Analysis** (read from disk + markdown parse, via `read_and_persist`, processor.rs:584); it does **not** enter at Construction directly. So `from_new` enters past Discovery+Comparison into Analysis-then-Construction; `from_stale` does the same. The saving is the skipped repo lookup and metadata comparison, not the parse. (For deletions, note already exposes `record_deleted`, processor.rs:330, which the `FileDeleted`/`DirDeleted` handlers can reuse.)

**Schema's processor entry points.** The target is the new per-file `BaseSchemaProcessor` (`crates/schema/src/base_processor.rs`), not the retiring `schema_processor.rs`. Today `BaseSchemaProcessor::from_discovery(file, root)` → `run(view, source, repo, bank_resolution)` (`base_processor.rs:252,280`) branches on `view: Option<&RawSchemaView>` and, on the present path, calls `check_timestamps` internally (`run_present`, `base_processor.rs:360`) to re-derive freshness. The integration adds `from_new` / `from_stale` siblings that **thread the indexer's `IndexStatus` and skip `check_timestamps`**, mapping the three statuses onto the pipeline:

- **`IndexStatus::New`** → the missing path (no cached view): read → parse → construct → persist. `from_new` enters here directly; the timestamp check is never run.
- **`IndexStatus::Stale`** → a cached view exists and the file changed: `from_stale` enters directly at the content-hash comparison (`run_content_check`, `base_processor.rs:389`), skipping `check_timestamps`. **The content-hash compare is retained** — that is not freshness re-derivation, it is schema's own "did the bytes actually change" guard that decides a Stale-noop (identical content, sync metadata only) versus a Stale semantic update. Dropping it would force a re-parse whenever an mtime changed with identical bytes.
- **`IndexStatus::Fresh`** → never reaches the processor. `SchemaService::flush` drops Fresh events with no work (the biggest saving, satisfying user story 2 for schema).

This delivers the `IndexStatus` reuse fully for schema: the indexer's classification drives the branch selection instead of the processor recomputing it. The property-bank processor (`PropertyBankProcessor`) keeps its own `run` because the bank is resolved once, up front, not per-schema.

The original `process_file` / `from_discovery` constructors remain in place. They are still useful for tests that want to drive the full pipeline from raw inputs, and for any future ad-hoc caller that doesn't run through the indexer.

### 9. App Orchestration

`crates/app/src/sync.rs` introduces `run_sync` as the top-level entry point. Its responsibilities are limited to:

- Open the redb `Store` at the configured cache path.
- Construct `FileReader` from the vault root.
- Construct one `NoteService`, one `SchemaService`, one concrete `TemplateService<RedbRepository, Writer, MiniJinjaEngine>` from the shared store and config.
- Create one bounded `crossbeam_channel` per service (default capacity 1024).
- Construct a `FanoutSink` over the sender halves.
- Spawn one OS thread per service, running `service.sync(receiver)`. Every service is `Send + 'static`, including template once its concrete triple is pinned here (see the threading note below).
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

The app crate gains direct dependencies on `traces-note`, `traces-schema` (in addition to its existing `traces-template`, `traces-indexer`). The new `traces-note`, `traces-schema`, `traces-template` each gain a dependency on `traces-indexer` (for `IndexEvent` types and `IndexEventStream`). These dependencies are intentional: they encode the "indexer → downstream consumer" direction that CONTEXT.md describes. They are acyclic on the regular dependency graph (`traces-indexer` depends on none of `note`/`schema`/`template`). The one wrinkle: `crates/note/Cargo.toml:44` carries an **unused `traces-app` dev-dependency**; once app depends on note, remove that dev-dep (see §10) to avoid fragile dev/regular coupling.

**Threading model — uniform thread-per-service, including template.** Spawning one OS thread per service requires every service be `Send + 'static`. The full chain is confirmed:

| Service | Field / port | Send + Sync? |
| --- | --- | --- |
| all | `FileReader` (`PathBuf` + `Validator`) | ✓ |
| all | `Arc<AppConfig>` (`crates/settings/src/config/aggregate.rs:29`) | ✓ |
| note / schema | `RedbRepository` (`Arc<Store>`) | ✓ |
| template | `R = RedbRepository` (`Arc<Store>`) | ✓ |
| template | `W = Writer` (`PathBuf`, `crates/fs/src/writer.rs:34`) | ✓ |
| template | `E = MiniJinjaEngine` (`minijinja::Environment<'static>`, `crates/template/src/engine/mini_jinja.rs:20`) | ✓ |

`TemplateService` **runs on its own thread**, like note and schema — the uniform "one OS thread per service" claim holds for all three. `crates/template/src/service.rs:132-136` documents the service as *intentionally not* `Send + Sync`, but nothing in the concrete `TemplateService<RedbRepository, Writer, MiniJinjaEngine>` is intrinsically non-`Send`; that doc was a premature marker omission. The `Send + 'static` bounds are re-added at the **composition root** (the spawn site in `run_sync`), not inside the generic service — `sync` stays generic (§7.2). Because this reverses a documented design decision in the template context, it is recorded as an ADR (`crates/template/docs/adr/0002-template-service-send-at-composition-root.md`).

### 10. Vault Deletion

Once each downstream context's service exists and is integrated:

- Rewrite `crates/note/tests/note_reader.rs` to drive fixtures through `NoteService::sync` (or directly through `NoteProcessor::from_new` / `from_stale` for the per-event test cases). The integration tests do not need to go through the indexer at all — they can construct synthetic `IndexEvent`s. **There are nine test functions, not seven.** Three of them (`load_skips_unchanged_notes`, `load_removes_missing_notes`, `full_scan_reports_pruned_files_for_removed_notes`) currently assert *freshness/deletion* behaviour of `VaultProcessor::process_full` — i.e. the indexer's job, not note's. When re-expressed as synthetic-event tests they will assert note's *event handler* behaviour (given a `FileDeleted` event, the note is removed), not scan behaviour. That is the correct new home for those assertions, but note it is a change in what they verify, not a mechanical port.
- Remove the `traces-vault` dev-dependency from `crates/note/Cargo.toml`. Also remove the **unused `traces-app` dev-dependency** (`crates/note/Cargo.toml:44`, zero uses in note source/tests) — once §9 makes `traces-note` a regular dependency of `traces-app`, leaving an unused `traces-app` dev-dep on note is fragile dev/regular coupling.
- Fix the two doc-comment references in `crates/db/src/table.rs` (lines 140 and 176) that mention `traces_vault::model::FileId`.
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
- `NoteService::handle_event` (per-event dispatch) — correctness against hand-built events. One test per `IndexStatus` (`New`, `Stale`, `Fresh`), one test per deletion variant, one test for events the context should ignore.
- `<Context>IndexEventConsumer::drain` — end-to-end on a hand-rolled `crossbeam_channel::unbounded()` feeding canned events including a terminating `ScanCompleted`. Asserts the returned report reflects the dispatched events.
- Schema-specific (batched-on-flush): the property-bank split in `IndexedSchemaSet` routes the bank file out of the schema set; events accumulate; `SchemaService::flush` runs only on `ScanCompleted`; the buffer is empty after drain returns; a `FileDeleted` event drives the caller-level delete on flush.
- Template-specific (batched-on-flush): template-dir events accumulate; the `process_all`-equivalent runs only on `ScanCompleted`; orphan detection over the buffered set matches `identify_deleted_template_paths` behaviour.
- Processor entry points: `NoteProcessor::from_new`/`from_stale` and `BaseSchemaProcessor::from_new`/`from_stale` produce the same observable result as the existing `process_file` / `from_discovery`+`run` path when fed equivalent inputs, and the schema siblings skip `check_timestamps` while preserving the content-hash branch. These are direct comparisons, not integration tests.

**App crate**

- `run_sync` end-to-end against a tempdir vault containing fixture markdown, schema, and template files. Uses `TestDb` for the shared store. Asserts that:
  - The returned `SyncReport.index` matches what `run_index` alone would have produced.
  - `SyncReport.note` reflects markdown files processed (created/updated/deleted counts).
  - `SyncReport.schema` reflects schemas processed.
  - `SyncReport.template` reflects template files processed.
  - A subsequent `run_sync` with no filesystem changes produces zero per-context work (all `Fresh`).
- `run_sync` failure-mode tests: indexer error aborts the run; consumer thread panic surfaces as `AppError`.

**Note integration test rewrite**

- `crates/note/tests/note_reader.rs` rewritten to drive fixtures through `NoteService::sync` (passing synthetic `IndexEvent`s) instead of `VaultProcessor::process_full`. All **nine** existing test functions retained; the fixture-construction helpers (`build_fixture`, `build_environment`) re-implemented against the new service. Note that the three deletion/freshness tests change what they assert (note's event handler vs the vault scan) — see §10.

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
