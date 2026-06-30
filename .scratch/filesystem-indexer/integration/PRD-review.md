# PRD Review: Indexer → Context Processor Integration

**Reviewed**: 2026-06-30
**PRD path**: .scratch/filesystem-indexer/integration/PRD.md
**Reviewer**: agent (PRD-review pass, GitNexus + source audit)

## Summary

The PRD is **architecturally sound but not shippable as-is**. The high-level
shape — event sink trait, in-memory fan-out, per-context consumers, app
orchestration, two-phase delivery — is coherent and the rename/storage blast
radius is genuinely contained to `crates/indexer/` (verified: zero external
consumers today). However, the PRD makes several concrete claims that the code
contradicts. Two are blocking design errors: (1) **schema's `Builder::load_all`
runs its own discovery internally and cannot be "driven by buffered events"
without being rewritten** — the PRD treats schema as "note but batched" when
schema's discovery does property-bank separation, inheritance-graph loading, and
topological-graph deletion detection that the indexer's event stream cannot
express; and (2) **`TemplateService` is explicitly documented as *not* `Send +
Sync`, which directly contradicts the PRD's "spawn one OS thread per service"
orchestration**. Several supporting claims (storage consolidation is a clean
precursor; processors enter "past Discovery and Comparison"; seven note tests;
`Config` type) are inaccurate against the code.

Recommendation: **revise** (substantive, not cosmetic). Schema's consumer model
and template's threading model each need a dedicated decision before
implementation planning. See Recommendations.

## Critical Issues (must fix before implementation)

1. **Schema's `Builder::load_all` cannot be driven by buffered events as
   written.** (PRD §6 line 249, §7 line 289, §2 line 62; user story 6 line 45)
   `Builder::load_all(&mut self)` (`crates/schema/src/builder.rs:48`)
   *internally* calls `DiscoveryEngine::run(&schema_spec, &self.repository)`
   (builder.rs:69), which re-scans the filesystem and re-queries the DB. There is
   no seam to inject a pre-buffered event set. The PRD's repeated claim that
   schema's consumer "buffers events until `ScanCompleted` and then runs
   `Builder::load_all` against the accumulated set" is infeasible without
   rewriting `load_all` to accept a `DiscoveryResult` (or equivalent) instead of
   constructing one. **Resolution**: either (a) add a `Builder::load_from(events)`
   entry that maps buffered `FileIndexEvent`s into a `DiscoveryResult`, or (b)
   keep schema running its own discovery and have its consumer only *trigger*
   `load_all` on `ScanCompleted` (ignoring event payloads) — but then schema does
   NOT share the indexer's filesystem scan, defeating user story 1 for schema.
   The PRD must pick one and say so.

2. **Schema discovery does materially more than the indexer's scan; the PRD
   under-specifies the gap.** (PRD §6 line 249, problem statement line 14)
   `DiscoveryEngine::run` (`crates/schema/src/discovery.rs:142-170`) performs four
   jobs the indexer has no concept of:
   - **Property-bank separation** via `spec.property_bank_file_path()`
     (discovery.rs:211-250) — routing one specific file out of the schema set.
   - **Inheritance-graph loading** via `repo.get_topological_graph()`
     (discovery.rs:286).
   - **Per-schema cached-view staleness** via `RawSchemaView`
     (discovery.rs:296-303), distinct from the indexer's `IndexStatus`.
   - **Topological-graph deletion detection** (`detect_deleted_schemas`,
     discovery.rs:385-395): a schema is "deleted" if it's in the inheritance
     graph's topo order but absent from the filesystem ID set. This is
     **semantically different** from the indexer's path-absence `FileDeleted`
     event. A schema orphaned in the graph but still on disk is handled here, and
     no `IndexEvent` can express it.
   The PRD's "buffers events and runs the builder" framing omits all four.
   **Resolution**: a dedicated section on how schema's `DiscoveryResult` fields
   (`property_bank`, `graph`, per-schema `cached`, `deleted_ids`) are reconstructed
   from `IndexEvent`s, or an explicit decision that schema keeps its own discovery.

3. **`TemplateService` is documented as intentionally NOT `Send + Sync`,
   contradicting the thread-per-service orchestration.** (PRD §9 line 325, §7
   line 287; user story 12) `crates/template/src/service.rs:132-136`:
   *"`TemplateService` is intentionally not bound `Send + Sync + 'static`. Those
   bounds are runtime-specific … not hexagonal-architecture-intrinsic."*
   `run_sync` spawns `thread::spawn(move || service.sync(receiver))`
   (PRD line 325), which requires the moved service be `Send + 'static`. Template
   cannot satisfy this without changing its documented design, and it is also
   generic `TemplateService<R, W, E>` (service.rs:137), not the flat
   `<Context>Service` the PRD assumes. **Resolution**: decide whether template
   runs on its own thread (forcing `Send` bounds it explicitly rejects), or is
   processed inline on the orchestrator thread (no `Send` needed but breaks the
   "one OS thread per service" uniformity), or wrap it. The PRD must not assert
   uniform thread-per-service while template opts out.

4. **Storage consolidation breaks an existing contract-test invariant; it is not
   a side-effect-free precursor.** (PRD §5 lines 184-186; user stories 21-22;
   handoff concern #2) The shared repository contract
   (`crates/indexer/src/storage/contract.rs:67-75`,
   `assert_all_paths_deduplicates`) deliberately stores the **same path in both
   the file table AND the dir table** and asserts `all_paths` dedupes. Today
   file-vs-dir path uniqueness is *per-kind*: a file and a dir may share a path
   (two separate tables: `FILE_ID_BY_PATH`, `DIR_ID_BY_PATH`,
   `tables.rs:21-25`). Collapsing to one `FS_ID_BY_PATH` keyed by `PathKey` makes
   path uniqueness **global** — a file and dir at the same path can no longer
   coexist. The PRD does not acknowledge this semantic change, and the contract
   test (plus `all_paths` dedup logic at `read.rs:217-247` and the
   `file_path_taken_by_other`/`dir_path_taken_by_other` guards at
   `write.rs:114-134`) must be reworked. **Resolution**: state the new
   path-uniqueness invariant explicitly and list the contract/read/write changes,
   or value the table by `(FsRecordId, FsRecordKind)` *and* allow two rows per
   path keyed by `(PathKey, FsRecordKind)` — which the PRD's stated
   `PathKey`→`(FsRecordId, FsRecordKind)` single-value shape (line 185) does NOT
   permit.

## Inconsistencies (PRD contradicts itself or contradicts code)

1. **Rename of `IndexedNodes`/`DeletedNodes` to `IndexedEvents`/`DeletionEvents`
   violates the PRD's own vocabulary rule.** (PRD §1 lines 72-78 vs lines 86-87)
   The naming table says aggregate run-summaries keep `Result`/`Report` and
   storage keeps `Record`; user stories 18-19 (lines 57-58) want wire-`Event`
   types kept *distinct* from storage. But `IndexedNodes` and `DeletedNodes` are
   **aggregate/summary types** (`crates/indexer/src/summary.rs:63,100`), held by
   `IndexResult`, not per-item wire types. Renaming them to `*Events` drags
   storage-aggregate vocabulary into the wire domain — the exact mixing the PRD
   forbids. Worse, `DeletedNodes` carries only `Box<[FsRecordId]>` (no events at
   all, summary.rs:100-103), so `DeletionEvents` is an actively misleading name.

2. **Processors do not "enter past Discovery and Comparison" the way the PRD
   describes.** (PRD §8 lines 293, 310) For note, the `New`/`Changed` status
   structs (`crates/note/src/processor.rs:108-122`) hold an already-parsed
   `RawNote` — they are *post-Analysis* states, not entry points. A `New`
   `IndexEvent` still requires reading + parsing the markdown
   (`read_and_persist`, processor.rs:584); it cannot "enter at
   Construction-from-scratch." The only genuinely skippable work is the
   `discover()` repository lookup (processor.rs:357) and `check_metadata()`
   comparison (processor.rs:390). The PRD overstates the saving and mislabels the
   entry stages.

3. **`Config` type referenced throughout does not exist; the real type is
   `AppConfig`.** (PRD §7 lines 266-271, §8 lines 303-305) `NoteService` is shown
   holding `Arc<Config>` and `from_new(... config: &Config ...)`. Note's
   `process_file` takes `&AppConfig` (`traces_settings::aggregate::AppConfig`,
   processor.rs:25,297); schema's `Builder` holds `&'config AppConfig`
   (builder.rs:21). There is no `Config` newtype. Either the PRD means
   `AppConfig` (then `Arc<AppConfig>` — verify `AppConfig: Send + Sync`) or it is
   introducing a new type it never defines.

4. **The note integration test count is wrong.** (PRD §10 line 346, Testing line
   417; handoff says "seven") `crates/note/tests/note_reader.rs` has **nine**
   `#[test]` functions, not seven. Three of them
   (`load_skips_unchanged_notes`:284, `load_removes_missing_notes`:316,
   `full_scan_reports_pruned_files_for_removed_notes`:336) assert
   *freshness/deletion* behaviour of `VaultProcessor::process_full` — i.e. the
   indexer's job, not note's. Re-expressing them as synthetic-`IndexEvent` tests
   silently changes what they verify (note's event handler, not the scan). "All
   seven retained" is both miscounted and glosses over this shift.

5. **User story 12 (slow consumer must not block the indexer) conflicts with the
   bounded blocking-send design.** (PRD line 51 vs §3 lines 150, 142-143)
   `FanoutSink::emit` does sequential **blocking** `sender.send()` on bounded
   (1024) channels. A live-but-slow consumer back-pressures and *does* block the
   indexer's scan loop — directly the thing story 12 says must not happen. The
   PRD presents back-pressure as a feature (line 150) and non-blocking as a
   requirement (story 12) without reconciling them.

## Gaps (missing decisions or under-specified areas)

1. **Deletion-event emit requires restructuring `detect_deletions`, which the
   PRD's emit-point list assumes already exists.** (PRD §4 lines 174-175;
   handoff concern, indexer checklist) Today `detect_deletions`
   (`crates/indexer/src/service.rs:221-249`) *batches* — it returns a
   `DeletedNodes` of IDs only and **discards the path** (the `path` at
   service.rs:229 is in scope but not retained). The PRD's `DeletedRecordEvent`
   needs `(id, path)` (PRD lines 113-117). Feasible post-`find_id_by_path`, but
   it is a rewrite of the deletion pass, not just "add an emit call." The PRD
   should track this as part of §5/§4, not imply it's a drop-in.

2. **`all_paths()` is not addressed in the consolidation.** (indexer checklist)
   `all_paths` (`read.rs:217-247`) iterates BOTH path tables and dedupes across
   them. After consolidation it iterates one table and the dedup becomes dead
   code. The PRD §5 lists the consolidation deltas but omits `all_paths`.

3. **Path-type mismatch between event variants is unspecified.** `FileIndexEvent`
   (ex-`FileIndexEntry`) carries a `FilePath` (entry.rs:29) while
   `DeletedRecordEvent` carries a `PathKey` (PRD line 115). Consumers filtering
   by directory (`matches`, PRD line 281) must reconcile two path
   representations. The PRD doesn't say which path type the consumer filters on.

4. **`EventSinkDisconnected` does not fit the existing three-arm `IndexerError`
   soft-fail model.** (PRD §3 line 151; indexer error checklist)
   `IndexerError` is `#[non_exhaustive]` with exactly three arms — `Scanner`,
   `Repository`, `Path` — and the run loop classifies any non-`Path` error as
   fatal/fail-closed (service.rs:125-132). Adding a flat
   `EventSinkDisconnected` variant is workable but the PRD should say whether it
   is a fourth top-level arm or nested, and confirm the fail-closed branch
   (service.rs:132) treats it correctly (it will, by default — worth stating).

5. **`ScanCompleted` is NOT emitted on indexer error paths; consumers can hang.**
   (PRD §4 line 176, user story 27; handoff concern #8) `run`
   (service.rs:82-177) builds the report and would emit `ScanCompleted` only at
   the very end (line 176 in the PRD). On a scanner error (service.rs:98 `?`,
   132 `return Err`) or a fatal repository error, `run` returns **before** the
   terminator. Consumers that block on `IndexEventStream::next()` waiting for
   `ScanCompleted` would hang unless the channel disconnects first (it will, when
   the sink drops — but the PRD relies on the terminator, not on disconnect, for
   "deterministic flush"). The PRD should specify that the abnormal path is
   disconnect-driven, or guarantee a terminal event on every exit.

6. **Schema's `from_new`/`from_stale` siblings still re-derive freshness.** (PRD
   §8 line 312) `PropertyBankProcessor::from_discovery` + `run`
   (`crates/schema/src/property_bank_processor.rs:236,256`) does its own
   `check_timestamps` (line 279). Carrying `IndexStatus` forward is a real
   refactor of the `run` branching, not the "small change / repurpose existing
   one" the PRD implies.

7. **Note carries an unused `traces-app` dev-dependency that the PRD's
   dependency analysis misses.** (PRD §10 line 347 only flags `traces-vault`)
   `crates/note/Cargo.toml:44` has `traces-app` as a dev-dep with **zero uses**
   in note's source/tests (grep-confirmed). Once §9 makes `traces-note` a regular
   dep of app (PRD line 340), app→note(regular) + note→app(dev) is a
   dev/regular coupling Cargo tolerates but is fragile. The unused dev-dep should
   be removed as part of this work; the PRD doesn't mention it.

## Minor issues (typos, clarifications, naming)

1. The two `traces_vault::model::FileId` doc references the PRD cites (§10 line
   348) are confirmed at `crates/db/src/table.rs:140` and `:176` — accurate.
2. `status()` visibility claim is correct: `pub(crate)` today at
   entry.rs:75,132; `id()`/`node()`/`path()` are already `pub`. The PRD only
   needs to flip `status()`.
3. The blast-radius rename claim is accurate: `FileIndexEntry`, `DirIndexEntry`,
   `IndexedNodes`, `DeletedNodes` are referenced **only** inside
   `crates/indexer/` (entry.rs, summary.rs, builder.rs, service.rs, lib.rs).
   Vault has its own separate `FILE_ID_BY_PATH`/`DIR_ID_BY_PATH` over a different
   `FileId` type (`crates/vault/src/storage/tables.rs:68,77`) — untouched by the
   indexer consolidation, and vault is deleted separately.
4. `crossbeam_channel` disconnect semantics claim (§3, §11) is correct: a
   panicked consumer drops its `Receiver`, so `Sender::send` returns `Err`, which
   maps to `EventSinkDisconnected`. (The blocking back-pressure caveat is logged
   under Inconsistencies #5.)

## Audit results per crate

### crates/indexer/

- **entry.rs**: rename `entry.rs`→`event.rs` is clean; only intra-crate users.
  `status()` is `pub(crate)` (lines 75,132) as the PRD states. `FileIndexEntry`
  holds a `FilePath`, not `PathKey` — path-type mismatch with
  `DeletedRecordEvent` is unaddressed (Gap #3).
- **summary.rs**: `IndexedNodes`/`DeletedNodes` are aggregates (lines 63,100),
  NOT wire types — the `*Events` rename contradicts §1's vocabulary (Inconsistency
  #1). `DeletedNodes` holds IDs only (no paths) → deletion-event emit needs a
  rewrite (Gap #1).
- **service.rs**: `detect_deletions` (221-249) batches and discards paths; emit
  points described in §4 require restructuring (Gap #1). Error paths return before
  `ScanCompleted` (Gap #5). `IndexerService` is `{vault_root, scanner, repo}`
  (40-44); adding `Option<Box<dyn IndexEventSink>>` is clean.
- **storage/tables.rs + read.rs + write.rs + contract.rs**: consolidation touches
  far more than deletion detection — `remove_*_graph`, `*_path_taken_by_other`,
  `save_*_in_tx`, `clear`, and `all_paths` all reference the two separate tables
  (write.rs:62-198,362-392; read.rs:217-247). The contract test deliberately
  stores one path in both tables (contract.rs:67-75) — **breaks** under a single
  unique-`PathKey` table (Critical #4).
- **error.rs**: three-arm `#[non_exhaustive]` `IndexerError`; `EventSinkDisconnected`
  fits via the fail-closed default but placement should be stated (Gap #4).
- **repository.rs**: `find_id_by_path` addition matches the existing
  `ReadRepository` pattern cleanly (trait at repository.rs:15-86).
- **rkyv::access switch** (§5 line 188): feasible — `find_file_by_path`/
  `find_dir_by_path` currently use `rkyv::from_bytes` via `deserialize_file/dir`
  (read.rs:34-43). read.rs:9-11 already documents that local `rkyv::access` is
  available. No `ArchivedEntity` buffering needed for contiguous redb slices —
  claim is plausible.

### crates/note/

- **processor.rs**: `process_file` takes `&AppConfig` not `&Config`
  (Inconsistency #3). `New`/`Changed` are post-Analysis states holding parsed
  `RawNote` (lines 108-122) — `from_new`/`from_stale` cannot skip Analysis
  (Inconsistency #2). `record_deleted` already exists (line 330) and the PRD
  doesn't mention reusing it for `FileDeleted`/`DirDeleted`. Status structs and
  `transition` are private — fine since `from_*` live in-crate.
- **repository.rs / storage/mod.rs**: `RedbRepository` wraps `Arc<Store>` and is
  documented `Send + Sync` (storage/mod.rs:50-54). `Arc<dyn Repository + Send +
  Sync>` is satisfiable. ✓
- **tests/note_reader.rs**: nine tests, not seven (Inconsistency #4); three are
  really indexer-behaviour tests.
- **Cargo.toml**: `traces-vault` dev-dep (line 43) is the deletion blocker as
  stated; **but** an unused `traces-app` dev-dep (line 44) is also present and
  unmentioned (Gap #7). Adding `traces-indexer` as a regular dep is acyclic
  (indexer has no note dep).

### crates/schema/

- **discovery.rs**: `DiscoveryEngine::run` does property-bank separation,
  inheritance-graph load, per-schema cached-view staleness, and topo-graph
  deletion detection (Critical #2). Far more than a scan.
- **builder.rs**: `Builder::load_all` calls `DiscoveryEngine::run` internally
  (line 69) — no event-injection seam (Critical #1). Holds `&'config AppConfig`
  and a by-value generic `R: Repository` (lines 20-23), not `Arc<dyn Repository>`.
- **property_bank_processor.rs**: `from_discovery`→`run` re-derives freshness via
  `check_timestamps` (lines 236,279) — `from_new`/`from_stale` is a real refactor
  (Gap #6).
- **schema_processor.rs**: uses `from_discovery_result(discovery: DiscoveryResult)`
  (lines 746,800) — driven by a `DiscoveryResult`, reinforcing that the whole
  schema pipeline is `DiscoveryResult`-shaped, not event-shaped.
- **tests / Cargo.toml**: **no `traces-vault` dependency** in schema (answers the
  handoff's open question). Adding `traces-indexer` is acyclic.
- **`Arc<dyn Repository + Send + Sync>` for SchemaService**: schema's
  `RedbRepository` is `Arc<Store>`-backed like note's, so satisfiable — but the
  current `Builder` takes `R` by value, so the `Arc<dyn>` field is a new shape.

### crates/template/

- **service.rs**: `TemplateService<R, W, E>` is generic over three ports
  (line 137) and **explicitly not `Send + Sync`** (lines 132-136) — breaks the
  thread-per-service model (Critical #3). It already owns its own discovery
  (`scan_templates`, line 530; `process_all`, line 344; orphan detection
  `identify_deleted_template_paths`, line 449). `config` is `TemplateConfigSpec`,
  not `Arc<Config>`.
- **No `sync(rx)`, no event awareness today**; "extend with `sync(rx)`" (PRD
  line 287) is net-new and collides with the generic, non-Send shape.
- **Cargo.toml**: no `traces-vault` dep; adding `traces-indexer` is acyclic.
- Per-file vs batched: template is **batched** (`process_all`), contradicting the
  PRD's "likely per-event, like note" guess (PRD line 249).

### Cross-crate

- **app**: `run_index` (`crates/app/src/index.rs:64`) opens the store at
  `cache_dir/INDEX_DB_FILENAME` and builds `IndexerService` directly; does not
  build a `FileReader` today (that's net-new for `run_sync`, fine). app currently
  deps: settings, fs, indexer, db, template (Cargo.toml:25-29). Adding
  `traces-note` + `traces-schema` as regular deps is acyclic on the regular
  graph; the only wrinkle is note's unused `traces-app` dev-dep (Gap #7).
- **workspace Cargo.toml**: members are `crates/*`; no cycle is introduced on the
  regular dependency graph by `traces-{note,schema,template}` → `traces-indexer`
  (indexer depends on none of them).
- **db/table.rs**: exactly two `traces_vault::model::FileId` doc references
  (lines 140, 176) — PRD's "two" is correct.

## Specific-concern disposition (handoff §"Specific concerns")

1. Wire/storage naming split — **Inconsistency #1** (rename violates the split).
2. Storage consolidation as clean precursor — **Critical #4** (breaks contract
   test; changes path-uniqueness semantics).
3. `rkyv::access` switch — **Minor / Verified-plausible** (read.rs already
   documents local access availability).
4. `Send + Sync` constraints — **Critical #3** for template;
   **Verified-OK** for note/schema (`FileReader` is `PathBuf`+`Validator`,
   `RedbRepository` is `Arc<Store>`).
5. `'svc` lifetime across threads — **Not a bug as framed.** The `&'svc
   NoteService` lives only inside `sync`'s own stack frame; `move ||
   service.sync(rx)` moves the service into the thread and re-borrows locally.
   The real requirement is `NoteService: Send + 'static`, which reduces to
   concern #4. Reclassified as **Verified-OK (note/schema), Critical (template)**.
6. `run_sync` thread spawning — same as #5: compiles iff the service is `Send`.
   **Verified-OK for note/schema; Critical for template.**
7. Schema's batched flow via `Builder::load_all` — **Critical #1** (no
   event-injection seam exists).
8. `ScanCompleted` terminator on error paths — **Gap #5** (not emitted on abort;
   relies on disconnect).
9. Channel disconnect semantics — **Verified-OK** (crossbeam returns `Err` on
   all-receivers-dropped), **but** Inconsistency #5 flags the blocking
   back-pressure tension with user story 12.
10. "Subsequent `run_sync` → zero per-context work (all Fresh)" — **Partially
    verified.** The indexer classifies Fresh correctly (service.rs bumps
    fresh_count). The PRD says the service drops Fresh events with no work (PRD
    line 310) — feasible, but the processors' `from_*` entry points must be
    written to no-op on Fresh; today nothing handles a Fresh event because the
    entry points don't exist yet. Claim is achievable but unproven; depends on
    Critical #1/#2 resolutions for schema.

## Recommendations

1. **Re-grill schema integration** (highest priority). Decide explicitly whether
   schema (a) keeps its own `DiscoveryEngine` and only uses `ScanCompleted` as a
   trigger — accepting that schema does NOT share the indexer's scan — or (b)
   gets a new `Builder` entry that reconstructs `DiscoveryResult`
   (property-bank split, inheritance graph, cached views, topo-deletion) from
   buffered events. Option (b) is a large piece of work the PRD currently hides.
2. **Decide template's threading model.** Either add explicit `Send` bounds at
   the composition root (contradicting service.rs:132's stated design — needs a
   note in template's CONTEXT/ADR), or process template inline on the
   orchestrator thread and drop the "uniform thread-per-service" claim for it.
3. **Rewrite PRD §5 to own the path-uniqueness change.** State that
   consolidation makes path uniqueness global, fix the contract test
   (contract.rs:67-75), and update `all_paths`, `*_path_taken_by_other`, and the
   `clear`/`save`/`remove` write paths — or key the table by
   `(PathKey, FsRecordKind)` and revise the single-value `(FsRecordId,
   FsRecordKind)` shape in line 185.
4. **Fix the naming decisions**: drop the `IndexedNodes`→`IndexedEvents` /
   `DeletedNodes`→`DeletionEvents` renames (they are aggregates, not wire
   types), or explicitly carve out an exception to §1.
5. **Correct factual claims**: `AppConfig` not `Config`; nine note tests not
   seven; processor entry points skip only Discovery+Comparison (not Analysis);
   `detect_deletions` must be restructured to emit per-path; `all_paths` needs
   updating; note's unused `traces-app` dev-dep should be removed.
6. **Specify the abort-path terminator contract**: guarantee either a terminal
   event or document that consumers exit on channel disconnect, so a mid-scan
   indexer error cannot hang a consumer.

The storage consolidation, sink trait, fan-out, note consumer, and app
orchestration for note are all in good shape and could proceed to planning once
the above are resolved. Schema and template are the two areas that need design
work before any implementation issue is cut.
