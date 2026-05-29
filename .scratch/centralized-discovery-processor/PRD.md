# PRD: Centralized Discovery Processor

**Status**: locked-in-design
**Created**: 2026-05-25
**Updated**: 2026-05-29
**Context**: Comprehensive rewrite with architectural corrections applied

---

## Problem Statement

Lithos currently repeats filesystem discovery logic across multiple contexts (notably Schema and Config), while Vault already maintains file and directory identity tables. This duplication increases maintenance cost, creates inconsistent discovery behavior, and makes it harder to evolve freshness checks and indexing safely.

The project needs a centralized filesystem discovery processor that starts from filesystem primitives, persists canonical file/directory identity once, and then lets each context perform context-specific processing without re-implementing base discovery.

## Solution

Refactor the existing Vault module into the initial base of a discovery module (incrementally, without a full module move in this session). The `FsDiscoveryProcessor` typestate processor will become the centralized filesystem discovery processor and will:

- Run scoped scans using configurable scan input.
- Compare scan results against persisted views to classify freshness.
- Persist only deltas (new, stale metadata updates, deletions) rather than rewriting all records.
- Return a discovery result contract for context-specific processors.

Context processors (Schema, Note, Template) remain standalone and consume discovery results as input to their own pipelines, then continue with context-specific parsing, hashing, validation, and persistence.

**Critical Constraint**: Config is resolved BEFORE discovery runs. Config provides the "lens" (extensions, exclusions) that discovery requires to operate.

## User Stories

1. As a Lithos maintainer, I want one centralized filesystem discovery processor, so that file discovery behavior is consistent across contexts.
2. As a Lithos maintainer, I want to avoid duplicate scan code in Schema and Config, so that refactors are safer and faster.
3. As a Schema processor maintainer, I want discovery to provide canonical file identity, so that SchemaId can be replaced by FileId.
4. As a Note processor maintainer, I want discovery classifications (new/stale/fresh/deleted), so that note ingestion can skip unnecessary work.
5. As a Template processor maintainer, I want indexed file metadata query support, so that template querying can evolve toward Obsidian-like behavior.
6. As a Config processor maintainer, I want config resolution to run first in orchestrated flows via Ascending Discovery, so that downstream processors (including the filesystem discovery processor) use resolved configuration as their prerequisite lens.
7. As a performance-focused engineer, I want delta persistence in discovery, so that indexing avoids full table rewrites.
8. As a cross-platform user, I want safe normalized storage keys and valid filesystem read paths, so that indexing works consistently on all OSes.
9. As an architecture reviewer, I want context processors to stay standalone after discovery, so that downstream stages can vary independently.
10. As a concurrency-focused engineer, I want independent context processors after discovery, so that compatible processors can run in parallel.
11. As a reliability-focused engineer, I want deletion detection in base discovery, so that stale records are pruned deterministically.
12. As a schema maintainer, I want property-bank and schema files to be filtered from shared discovery results, so that schema logic focuses only on domain concerns.
13. As a config maintainer, I want structured-file freshness checks in config-owned views, so that config can preserve semantic hash behavior.
14. As a note/template maintainer, I want lightweight content freshness checks, so that unstructured files can use simpler hash records.
15. As a persistence maintainer, I want read/write repository seams preserved, so that storage adapters remain testable and swappable.
16. As a developer onboarding to Lithos, I want discovery responsibilities clearly separated from context parsing responsibilities, so that module boundaries are easier to understand.
17. As a test author, I want deterministic discovery result classification, so that tests can validate behavior without relying on implementation details.
18. As a future refactor owner, I want this change staged incrementally from Vault, so that migration risk stays manageable.

## Key Decisions & Architecture

### 1. Architecture & Boundaries

- **Config-First Orchestration**: Config is resolved BEFORE discovery runs. Discovery consumes config as a prerequisite lens (extensions, exclusions).
- **Centralized Processor**: The `FsDiscoveryProcessor` typestate processor lives in the discovery layer (incrementally refactored from `vault/`) and replaces context-specific stateless discovery functions.
- **Standalone Processors**: Context processors remain standalone and consume discovery results as input to their own pipelines, preserving independent evolution and parallel execution.
- **Parsing**: Parsing remains context-specific and is not owned by base discovery.
- **Incremental Refactor**: Discovery remains an incremental refactor from the current Vault module first; full module renaming/re-homing is deferred.

### 2. Scanning & Classification

- **Scoped Scans**: Discovery processing is scoped by scan input and supports partial or targeted scans.
- **Built-in Freshness Checking**: Discovery includes metadata-based comparison (timestamp AND size) to classify records by freshness; it does not blindly scan-and-write.
- **Event-Driven Scans (Future)**: Future file watcher integration will support event-driven scans (processing specific FileEvent lists without directory traversal).
- **Result Contract**:
  ```rust
  /// Result of filesystem discovery classification for a single file.
  ///
  /// The `view` field represents the CURRENT state after classification:
  /// - For `New` files: freshly constructed FileView from scan (recorded_at = now)
  /// - For `Fresh`/`Stale` files: updated FileView with current metadata (recorded_at = now)
  /// - For `Deleted` files: NOT represented here (see DiscoveryResult.deleted_file_ids)
  pub struct DiscoveredFile {
      pub id: FileId,
      pub view: FileView,        // CURRENT view after classification
      pub path: FilePath,        // From scan, for immediate reads
      pub status: DiscoveryStatus,
  }

  pub enum DiscoveryStatus {
      New,      // Not in DB (FileView newly constructed)
      Fresh,    // Metadata unchanged (timestamp AND size match persisted view)
      Stale,    // Metadata changed (timestamp OR size differs from persisted view)
  }

  pub struct DiscoveryResult {
      pub files: Vec<DiscoveredFile>,       // New, Fresh, Stale only
      pub deleted_file_ids: Vec<FileId>,    // Separate collection (no DiscoveredFile)
  }
  ```
- **Freshness Comparison Logic**: Freshness is determined by comparing scanned `FsFile.metadata` (from `DirScanner`) against persisted `FileView.metadata` in the database. The `recorded_at` field in `FileView` is NOT part of freshness comparison—it tracks when the view was last persisted, not when the filesystem entity was modified. Comparison uses existing `FileMetadata::is_timestamp_match()` and `FileMetadata::is_size_match()` methods. A file is `Fresh` only if BOTH match; `Stale` if EITHER differs.

### 3. Identity & Paths

**Decision: FileId replaces SchemaId/NoteId for all file identity**

- **Canonical Identity**: `FileId` and `DirId` become the ONLY identity for file-backed entities. `SchemaId` and `NoteId` are REMOVED.
  - Rationale: Source of truth is files. Schema inheritance = "file A extends file B" via FileId relationships.
  - Migration: Replace all `SchemaId`/`NoteId` usage with `FileId` in tables, indexes, and inheritance graphs.

- **Cross-Platform Path Storage**:
  ```rust
  pub struct FileView {
      id: FileId,
      parent_id: Option<DirId>,
      path: PathKey,              // ✅ LOCKED: Forward-slash normalized path
      name: FileName,
      format: FileFormat,
      metadata: FileMetadata,
      #[rkyv(with = rkyv::with::AsUnixTime)]
      recorded_at: SystemTime,    // ✅ LOCKED: When view was persisted
  }

  pub struct DirView {
      id: DirId,
      parent_id: Option<DirId>,
      path: PathKey,              // ✅ LOCKED: Forward-slash normalized path
      name: DirName,
      metadata: DirMetadata,
      #[rkyv(with = rkyv::with::AsUnixTime)]
      recorded_at: SystemTime,    // ✅ LOCKED: When view was persisted
  }
  ```

- **PathKey Cross-Platform Guarantee**:
  - `PathKey` stores forward slashes (e.g., `"notes/daily/2026.md"`)
  - Research confirmed: `vault_root.join(view.path.as_str())` produces correct `FilePath` on Windows/macOS/Linux
  - Forward slashes work universally (Windows has supported `/` since MS-DOS 2.0)
  - Database portability: Same redb file works on all OSes without re-indexing
  - Reference: `.scratch/CROSS_PLATFORM_PATH_FINDINGS.md`

- **Table Simplification**:
  - ✅ REMOVE: `PATH_BY_FILE_ID` and `PATH_BY_DIR_ID` reverse index tables
  - ✅ KEEP: `FILE_ID_BY_PATH` and `DIR_ID_BY_PATH` (path → ID lookups)
  - Rationale: Path now in `FileView`/`DirView`, so reverse lookup via view fetch (no separate table needed)

### 3.5: Filesystem Discovery Typestate Processor

**Decision: Centralized Typestate Processor Replacing Context-Specific Discovery**

The filesystem discovery processor follows the same typestate pattern as `schema/property_bank_processor.rs`, adapted for filesystem-level discovery. It processes BOTH files AND directories through a multi-stage pipeline.

**Module Location**: Discovery code currently lives in `lithos-core/src/vault/` and will be renamed to `lithos-core/src/discovery/` as part of this PRD's implementation. References to "discovery module" refer to this target location.

#### Processor Structure

```rust
/// Filesystem discovery typestate processor.
///
/// Dual typestate dimensions:
/// - **Stage**: Pipeline phase (Collection, Comparison, Refresh, Construction, Completion)
/// - **Status**: Knowledge state (Missing, Present, New, Stale, Fresh)
pub struct FsDiscoveryProcessor<Stage, Status> {
    _stage: PhantomData<Stage>,
    _status: PhantomData<Status>,
}
```

#### Stage Definitions

| Stage | Purpose | Transitions To |
|-------|---------|----------------|
| **Collection** | Scan filesystem via `DirScanner`, collect `FsFile`/`FsDir` entries | Comparison |
| **Comparison** | Compare scanned metadata against persisted views, classify as New/Stale/Fresh | Refresh, Construction |
| **Refresh** | Update metadata for Fresh files (timestamp/size sync only) | Construction |
| **Construction** | Construct new `FileView`/`DirView` or update stale views | Completion |
| **Completion** | Produce `DiscoveryResult` for context routing | (terminal) |

#### Status Definitions

| Status | Meaning | Valid Stages |
|--------|---------|--------------|
| **Missing** | No persisted view found in DB | Comparison → Construction |
| **Present** | Persisted view found in DB | Comparison → Refresh/Construction |
| **New** | Not in DB (Missing classification) | Construction |
| **Stale** | Metadata changed (timestamp OR size differs) | Construction |
| **Fresh** | Metadata unchanged (timestamp AND size match) | Refresh → Construction |

#### Example Flow

```rust
// Stage 1: Collection
let processor = FsDiscoveryProcessor::<Collection, Unknown>::new(scope, config);
let scanned = processor.scan(scanner)?; // Collects FsFile/FsDir from DirScanner

// Stage 2: Comparison
let comparison = scanned.transition_to_comparison(repository)?;
match comparison.classify(file_id)? {
    ClassificationResult::Missing(p) => {
        // New file: no persisted view
        let new_view = p.construct_file_view(scanned_metadata)?;
        p.persist_new(new_view)?;
    }
    ClassificationResult::Fresh(p) => {
        // Metadata matches: skip or refresh recorded_at
        p.refresh_timestamp()?;
    }
    ClassificationResult::Stale(p) => {
        // Metadata differs: update view
        let updated_view = p.update_file_view(scanned_metadata)?;
        p.persist_updated(updated_view)?;
    }
}

// Stage 3: Completion
let result = processor.complete()?; // Produces DiscoveryResult
```

#### Directory Processing

Directories follow the same classification logic as files:
- **New directories**: Construct `DirView`, persist to `DIR_VIEWS` table
- **Stale directories**: Update `DirView.metadata`, persist delta
- **Fresh directories**: No-op or timestamp refresh
- **Deleted directories**: Collect `deleted_dir_ids` separately (same as files)

**Reference**: `schema/property_bank_processor.rs` (lines 1-100) for typestate pattern.

### 3.6: Existing Filesystem Types (Reference)

Discovery leverages existing types from the `fs/` module:

- **`FileFormat`** (`fs/format.rs`): Enum representing supported file formats, detected via `FileFormat::from_extension()`. Supports Json, Toml, Yaml, Markdown, Image, Pdf, Document, Archive, Binary, Unknown.

- **`FileName`** (`fs/name.rs`): Owned UTF-8 filename newtype (`Box<str>`). Wraps validated values; does not perform path validation.

- **`DirName`** (`fs/name.rs`): Owned UTF-8 directory name newtype (`Box<str>`). Wraps validated values; does not perform path validation.

- **`DirScanInput`** (`fs/scanner.rs`): Scan configuration struct with fields:
  - `pattern: Option<&str>` — glob pattern for path matching
  - `extensions: Option<&[&str]>` — file extension filter (AND semantics with pattern)
  - `include_dirs: bool` — whether to include directories in results
  - `follow_symlinks: bool` — symlink traversal policy
  - `recursive: bool` — recursive vs single-level scan

**Batch Size Configuration**: Discovery batch size (default 100) is defined as a constant in the discovery processor implementation. Context-specific processors may define their own batch sizes independently (e.g., schema may use smaller batches than notes).

**Reference**: `fs/format.rs`, `fs/name.rs`, `fs/scanner.rs` for complete API documentation.

### 4. Delta Persistence Strategy

- **Delta Persistence**: Discovery persists only deltas (new files, stale metadata updates, deletions) and uses batch repository operations for efficient writes/deletes.
- **Retained Indexes**: path, parent, format, and primary views.
- **Basename Index**: Removed from general discovery concerns (context-specific if needed).

### 5. Hashing & Content Staleness

- **Context Ownership**: File-level content hashing is context-owned for freshness checks; discovery will not force file content hashing in `FileView`.
- **Structured Contexts**: Structured contexts (Schema/Config) require both content hash and entry/property hash indexing in their own view models.
- **Hash Contracts**: Hash capability contracts are crate-private and based on support hash primitives, utilizing traits: `HasContentHash`, `HasContentHashMut`, `HasEntryHashes`, `HasEntryHashesMut`.

### 6. Pipeline Resilience & Restartability

**Decision: Context-Specific Event Sourcing with Shared Infrastructure**

**Architecture Pattern**: Event sourcing enables complete pipeline restartability after crashes, preserving all completed work and providing audit trails for debugging.

#### Scope Boundary

**IMPORTANT**: This PRD's scope is strictly limited to:

1. **Generic event storage infrastructure** (`EventId`, `EventTable`, `EventStore` trait) in `db/event.rs`
2. **Filesystem discovery processor restartability** (discovery events, projector, resumption)

**OUT OF SCOPE** (deferred to separate PRD):
- Schema, Note, Template, Config processor event modeling
- Cross-context completion tracking and coordination
- Full domain-specific event type definitions for all contexts
- Compaction strategies beyond discovery processor

**Discovery Event Scope**: Discovery events support restartability of the **filesystem discovery typestate processor** only. They track scanning, classification, and persistence of `FileView`/`DirView` records.

#### Core Components

1. **EventId Newtype** (follows `db/` module pattern):
   ```rust
   /// Monotonically increasing event sequence number
   #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
   pub struct EventId(pub u64);

   impl_redb_key!(EventId);  // Matches UuidTable pattern
   ```

2. **EventTable Newtype** (follows `db/` module pattern):
   ```rust
   /// Type-safe event table wrapper
   pub struct EventTable<V: Value + 'static> {
       definition: TableDefinition<'static, EventId, V>,
   }

   impl<V: Value + 'static> EventTable<V> {
       pub const fn new(name: &'static str) -> Self {
           Self {
               definition: TableDefinition::new(name),
           }
       }

       pub fn definition(&self) -> TableDefinition<'static, EventId, V> {
           self.definition
       }
   }
   ```

3. **Event Sequence Table** (per-context EventId allocation):
   ```rust
   /// Per-context event sequence table (stores next available EventId).
   ///
   /// Single-row table: key is context name, value is next u64.
   pub const EVENT_SEQUENCES: TableDefinition<'static, &str, u64> =
       TableDefinition::new("event_sequences");
   ```

4. **EventStore Trait** (context repositories implement this):
   ```rust
   /// Event storage behavior for context-specific event logs.
   ///
   /// Implementations MUST ensure:
   /// - EventId allocation is transactional (same txn as append)
   /// - EventIds are monotonically increasing per context
   /// - Concurrent appends serialize via redb MVCC
   pub trait EventStore {
       type Event: ArchivedEntity;  // Uses db/codec.rs for serialization

       /// Append single event within existing write transaction.
       ///
       /// Allocates next EventId atomically in same transaction.
       fn append_event(
           &self,
           txn: &mut WriteTransaction,
           event: &Self::Event,
       ) -> Result<EventId, DbError>;

       /// Load all events for state rehydration (in EventId order).
       fn load_all_events(&self) -> Result<Vec<Self::Event>, DbError>;

       /// Compact event log (delete events for completed work).
       fn compact_events(
           &self,
           txn: &mut WriteTransaction,
           completed_event_ids: &[EventId],
       ) -> Result<(), DbError>;
   }
   ```

#### EventId Allocation Strategy

**Decision: Per-Context Sequence Tables with Transactional Increment**

Each context maintains its own monotonic sequence in the `EVENT_SEQUENCES` table. `EventId` allocation occurs within the same write transaction as event append to ensure atomicity and crash safety.

##### Reference Implementation Pattern

```rust
impl EventStore for DiscoveryRepository {
    type Event = FsDiscoveryEvent;

    fn append_event(
        &self,
        txn: &mut WriteTransaction,
        event: &Self::Event,
    ) -> Result<EventId, DbError> {
        // 1. Read current sequence (transactional)
        let sequences = txn.open_table(EVENT_SEQUENCES)?;
        let next_id = sequences
            .get("discovery")?
            .map(|guard| guard.value())
            .unwrap_or(0);

        // 2. Serialize event via ArchivedEntity trait (db/codec.rs)
        let event_bytes = event.to_bytes()?;

        // 3. Write event with allocated ID
        let events = txn.open_table(DISCOVERY_EVENTS.definition())?;
        events.insert(EventId(next_id), event_bytes.as_slice())?;

        // 4. Increment sequence (same transaction)
        sequences.insert("discovery", next_id + 1)?;

        Ok(EventId(next_id))
    }

    fn compact_events(
        &self,
        txn: &mut WriteTransaction,
        completed_event_ids: &[EventId],
    ) -> Result<(), DbError> {
        let events = txn.open_table(DISCOVERY_EVENTS.definition())?;
        for event_id in completed_event_ids {
            events.remove(event_id)?;
        }
        Ok(())
    }

    // ... load_all_events implementation
}
```

##### Crash Safety Properties

| Scenario | Behavior |
|----------|----------|
| **Crash before txn commit** | Sequence unchanged, event not written (both rolled back) |
| **Crash after txn commit** | Sequence incremented, event persisted (both committed) |
| **Concurrent appends** | redb MVCC serializes write transactions (automatic ordering) |
| **Restart after crash** | `load_all_events()` returns all committed events in EventId order |

##### Why Not AtomicU64?

`AtomicU64` increments are NOT transactional. If a transaction allocates ID=42 but crashes before commit, the sequence is advanced but event 42 never exists (gap in log). Transactional sequence tables ensure atomicity: either BOTH sequence increment AND event append succeed, or NEITHER does.

##### Scope Per Context

Each context has its own sequence key in `EVENT_SEQUENCES`:
- `"discovery"` → discovery event IDs
- `"schema"` → schema event IDs (future)
- `"note"` → note event IDs (future)
- `"template"` → template event IDs (future)
- `"config"` → config event IDs (future)

This prevents EventId collisions across contexts and allows independent compaction.

5. **Context-Specific Event Tables** (maintains bounded contexts):
   - `DISCOVERY_EVENTS: EventTable<&[u8]>` (discovery module) - Filesystem discovery processor events
   - `SCHEMA_EVENTS: EventTable<&[u8]>` (schema module, future) - Schema processing events
   - `NOTE_EVENTS: EventTable<&[u8]>` (note module, future) - Note processing events
   - `TEMPLATE_EVENTS: EventTable<&[u8]>` (template module, future) - Template processing events
   - `CONFIG_EVENTS: EventTable<&[u8]>` (config module, future) - Config processing events

   **Serialization**: All event tables use **rkyv** via the `ArchivedEntity` trait (`db/codec.rs`). Events must derive `Archive + Serialize + Deserialize` with `Portable` archived form and `CheckBytes` validation. The `ArchivedEntity` trait handles alignment, validation, and serialization automatically.

6. **Filesystem Discovery Event Types** (typestate transitions):
   ```rust
   /// Filesystem discovery events track typestate transitions for restartability.
   ///
   /// These events enable resuming discovery after crashes without re-scanning
   /// completed batches.
   #[derive(Archive, Deserialize, Serialize)]
   #[rkyv(derive(CheckBytes))]  // Required for validation
   pub enum FsDiscoveryEvent {
       /// Batch scan started (filesystem traversal begun)
       BatchScanStarted { batch_id: u64, started_at: SystemTime },

       /// Filesystem entry collected (file or directory scanned)
       EntryCollected { file_id: FileId, path: PathKey, collected_at: SystemTime },

       /// Entry classified (New/Stale/Fresh determination completed)
       EntryClassified { file_id: FileId, status: DiscoveryStatus, classified_at: SystemTime },

       /// View persisted (FileView/DirView written to database)
       ViewPersisted { file_id: FileId, persisted_at: SystemTime },

       /// Batch completed (all entries in batch processed)
       BatchCompleted { batch_id: u64, file_count: usize, completed_at: SystemTime },

       /// Discovery failed for entry
       EntryFailed { file_id: FileId, error: String, failed_at: SystemTime },
   }
   ```

7. **Projector Pattern** (rehydrates state from events):
   ```rust
   /// Rehydrated state from discovery events for resumption.
   pub struct PendingDiscoveryState {
       pub completed_batches: HashSet<u64>,
       pub pending_files: HashMap<FileId, PathKey>,
       pub failed_files: HashMap<FileId, String>,  // FileId -> error
   }

   impl PendingDiscoveryState {
       pub fn from_events(events: &[FsDiscoveryEvent]) -> Self {
           let mut state = Self {
               completed_batches: HashSet::new(),
               pending_files: HashMap::new(),
               failed_files: HashMap::new(),
           };

           for event in events {
               match event {
                   FsDiscoveryEvent::BatchCompleted { batch_id, .. } => {
                       state.completed_batches.insert(*batch_id);
                   }
                   FsDiscoveryEvent::EntryCollected { file_id, path, .. } => {
                       state.pending_files.insert(*file_id, path.clone());
                   }
                   FsDiscoveryEvent::ViewPersisted { file_id, .. } => {
                       state.pending_files.remove(file_id);
                   }
                   FsDiscoveryEvent::EntryFailed { file_id, error, .. } => {
                       state.failed_files.insert(*file_id, error.clone());
                       state.pending_files.remove(file_id);
                   }
                   _ => {}
               }
           }

           state
       }

       pub fn should_skip_batch(&self, batch_id: u64) -> bool {
           self.completed_batches.contains(&batch_id)
       }

       pub fn has_pending_work(&self) -> bool {
           !self.pending_files.is_empty()
       }
   }
   ```

#### Partial Success & Resumption Flow

**Scenario**: Discovery scans 1000 files in 10 batches → Processes 3 batches (300 files) → **CRASH** → Restart

**Recovery Process**:
1. **Rehydrate State**: `let all_events = discovery_repo.load_all_events()?;`
2. **Project State**: `let pending_state = PendingDiscoveryState::from_events(&all_events);`
   - 3 completed batches (batch_id 0, 1, 2)
   - 7 pending batches (batch_id 3-9)
   - 300 persisted files, 700 pending files
3. **Resume Processing**: Skip completed batches, process only batches 3-9
4. **Emit Events**: Append `BatchCompleted` events as batches finish
5. **Compact Log**: After discovery completes, compact completed batch events

**Key Benefits**:
- ✅ **Zero Work Lost**: All completed batches preserved across crashes
- ✅ **Audit Trail**: Full history of discovery transitions for debugging
- ✅ **Bounded Context Isolation**: Discovery crash cannot corrupt context processor event logs
- ✅ **Natural Typestate Fit**: Events emitted at processor stage transitions

#### Typestate-Driven Embedded Commits (MVCC)

**Decentralized Parallel Write Strategy**:

Context processors run in **complete parallel isolation** and leverage redb's MVCC. The insertion of the new typestate (e.g., `SCHEMAS` table) and the appending of the transition event (e.g., `SCHEMA_EVENTS` table) **MUST occur inside the exact same `Store::write(|txn| { ... })` transaction closure**.

```rust
impl SchemaProcessor<Parsed, Review> {
    pub fn analyze(self, repo: &impl Repository) -> Result<SchemaProcessor<Analyzed, Review>, Error> {
        // 1. CPU-bound work (no database lock)
        let analysis_result = self.analyze_properties()?;

        // 2. Atomic state + event write (single transaction)
        repo.write(|txn| {
            // Insert analyzed state
            txn.save_schema_state(&analysis_result)?;

            // Append event (same transaction)
            repo.append_event(txn, &SchemaEvent::Analyzed {
                file_id: self.file_id,
                analyzed_at: SystemTime::now(),
            })?;

            Ok(())
        })?;

        // 3. Typestate transition
        Ok(SchemaProcessor::new_analyzed(analysis_result))
    }
}
```

**Key Properties**:
- ✅ **Atomic state + event writes**: Both happen in same transaction (no inconsistency)
- ✅ **No scatter-gather bottleneck**: Each processor commits independently upon state transition
- ✅ **redb MVCC handles contention**: Write transactions automatically serialize without deadlock
- ✅ **Minimal lock duration**: Transactions held for 1-2ms (state insert + event append)
- ✅ **Natural typestate fit**: Events emitted exactly when state changes, not externalized

**Explicitly Rejected Alternative**: Sequential scatter-gather write coordination. Context processors must NOT accumulate events and coordinate a single batch write at pipeline end. This pattern creates a bottleneck, violates typestate cohesion, couples independent bounded contexts, and breaks atomicity guarantees.

#### Batch Performance

**Discovery**: Batch commits every N=100 files
```rust
const BATCH_SIZE: usize = 100;
for (batch_id, batch) in files.chunks(BATCH_SIZE).enumerate() {
    repo.write(|txn| {
        // Emit batch start event
        repo.append_event(txn, &FsDiscoveryEvent::BatchScanStarted {
            batch_id: batch_id as u64,
            started_at: SystemTime::now(),
        })?;

        // Atomic: persist views + emit events
        repo.persist_discovery_batch(txn, batch)?;
        for file in batch {
            repo.append_event(txn, &FsDiscoveryEvent::ViewPersisted {
                file_id: file.id,
                persisted_at: SystemTime::now(),
            })?;
        }

        // Emit batch completion event
        repo.append_event(txn, &FsDiscoveryEvent::BatchCompleted {
            batch_id: batch_id as u64,
            file_count: batch.len(),
            completed_at: SystemTime::now(),
        })?;

        Ok(())
    })?;
}
```

**Context Processing**: Per-file atomic commits
```rust
for file in pending_files {
    match process_schema_file(file) {
        Ok(schema) => {
            repo.write(|txn| {
                repo.save_schema(txn, &schema)?;
                repo.append_event(txn, &SchemaEvent::Completed { file_id: schema.file_id })?;
                Ok(())
            })?;
        }
        Err(e) => {
            repo.write(|txn| {
                repo.append_event(txn, &SchemaEvent::Failed {
                    file_id,
                    error: e.to_string(),
                })?;
                Ok(())
            })?;
        }
    }
}
```

#### Dependency-Aware Cleanup

**Cleanup timing respects context dependencies**:
```
Config → Discovery → {Schema, Note} → Template
```

- **Immediate Cleanup**: Schema, Note event logs (independent after discovery)
- **Deferred Cleanup**: Template events (after Schema/Note complete, since template depends on both)
- **Final Cleanup**: Discovery events (after ALL contexts complete), then Config events

```rust
// Schema and Note complete (independent)
schema_repo.write(|txn| {
    schema_repo.compact_events(txn, &completed_event_ids)?;
    Ok(())
})?;

note_repo.write(|txn| {
    note_repo.compact_events(txn, &completed_event_ids)?;
    Ok(())
})?;

// Template completes (depends on Schema + Note)
template_repo.write(|txn| {
    template_repo.compact_events(txn, &completed_event_ids)?;
    Ok(())
})?;

// All contexts complete
discovery_repo.write(|txn| {
    discovery_repo.compact_events(txn, &all_event_ids)?;
    Ok(())
})?;

config_repo.write(|txn| {
    config_repo.compact_events(txn, &config_event_ids)?;
    Ok(())
})?;
```

**Compaction Safety Rules**:

- **When to Compact**: After context processor completes (all files persisted or failed)
- **Never compact mid-pipeline**: Rehydration requires full event log
- **Orchestrator Responsibility**: Orchestrator triggers compaction after each processor finishes

#### Performance Characteristics

- **Append-only writes**: Lock-free, fast (1-2ms per atomic state+event write)
- **Redb MVCC**: No write contention (automatic serialization, no deadlock)
- **Log compaction**: Keeps event tables bounded (delete completed/failed)
- **Rehydration cost**: O(N) where N = pending files (typically small after compaction)

**Reference**: `.scratch/pipeline-restartability-research.md`

---

### 7. Orchestration Policy

**Decision: 5-Phase Pipeline with Config as Prerequisite Lens**

Previously, a circular dependency existed: local config defined vault_root, but root was needed to find config. This is resolved via **Ascending Discovery**.

#### 7.1: Ascending Discovery Algorithm (Stateless)

**Vault Root Resolution**:
Starting from CWD, traverse upward to `/` (or boundary like `.git`), stopping at first directory containing:
- `lithos.{toml|json|yaml|yml}`
- `.lithos.{toml|json|yaml|yml}`
- `.lithos/config.{toml|json|yaml|yml}`

If no vault found, fall back to global "trusted paths". CLI overrides (`--vault`) take precedence.

**Note**: CLI crate not fully set up yet. Pipeline design remains open to prepending Phase 0 for broader CLI flag parsing.

#### 7.2: Five-Phase Pipeline

```rust
// Phase 1: Context Resolution (stateless I/O)
let vault_root = resolve_vault_context(cwd, cli_overrides)?;

// Phase 2: Config Hydration (FAIL-FAST, uses config repository)
let db = Database::open_or_create(&vault_root)?;  // DB needed for config persistence
let config_repo = config::Repository::new(&db);
let config = ConfigBuilder::new(vault_root)
    .discover(&config_repo)?   // Discover config files (Ascending Discovery)
    .load(&config_repo)?        // Parse and validate config
    .build()?;                  // Freeze config for discovery handoff

// Phase 3: Discovery State Rehydration
let discovery_repo = discovery::Repository::new(&db);
let discovery_events = discovery_repo.load_all_events()?;
let pending_discovery = PendingDiscoveryState::from_events(&discovery_events);

// Phase 4: Filesystem Discovery (typestate processor)
let discovery_spec = config.to_discovery_spec();
let scope = cli.discovery_scope();  // ← Runtime parameter
let processor = FsDiscoveryProcessor::new(&discovery_spec, scope);
let discovery_result = if pending_discovery.has_pending_work() {
    // Resume from rehydrated state
    processor.resume(pending_discovery, &discovery_repo)?
} else {
    // Fresh discovery run
    processor.run(&discovery_repo)?
};

// Phase 5: Context Processing (SEQUENTIAL - see Section 7.6 for parallel analysis)
let router = ContextRouter::new(&config);
let routed_files = router.route(&discovery_result)?;

// Process in dependency order: Schema/Note (independent), then Template (depends on both)
let schema_repo = schema::Repository::new(&db);
let note_repo = note::Repository::new(&db);
let template_repo = template::Repository::new(&db);

SchemaProcessor::process(routed_files.schemas, &config, &schema_repo)?;
NoteProcessor::process(routed_files.notes, &config, &note_repo)?;
TemplateProcessor::process(routed_files.templates, &config, &template_repo)?;  // LAST
```

**Dependency Graph**:
```
Config → Discovery → {Schema, Note} → Template
```

**Processing Order**: Schema and Note are independent and can run in parallel (future optimization). Template depends on both Schema and Note, so it MUST run after both complete.

#### 7.3: Config-to-Discovery Handoff

**Static Config Contract**:
```rust
pub struct DiscoveryConfigSpec {
    pub root: VaultRoot,           // From Ascending Discovery (NOT config file)
    pub extensions: Extensions,     // Active file formats
    pub exclusions: Vec<PathPattern>, // User config + implicit (.git, cache_dir)
}

impl Config {
    pub fn to_discovery_spec(&self) -> DiscoveryConfigSpec { /* ... */ }
}
```

**Dynamic Runtime Scope** (CLI parameter, NOT stored in config):
```rust
pub enum DiscoveryScope {
    FullVault { bypass_freshness: bool },
    Contexts { contexts: Vec<ContextScope>, bypass_freshness: bool },
    Targeted { path: PathKey, bypass_freshness: bool },
    EventDriven { events: Vec<FileEvent> },  // Future file watcher
}
```

#### 7.4: Config Error Propagation & Missing Config Behavior

**Rule: Malformed Config is Fatal, Missing Config is Conditional**

##### Malformed Config (Fatal)

Invalid TOML/JSON/YAML syntax, missing required fields, or semantic validation errors ALWAYS halt the pipeline:

```rust
let config = ConfigBuilder::new(vault_root)
    .discover(&config_repo)?  // Find config files
    .load(&config_repo)?      // Parse + validate (FAIL-FAST here)
    .build()?;
```

**NO fallback to defaults. NO silent degradation.**

**Errors**:
- `ConfigError::ParseFailed` (syntax error in TOML/JSON/YAML)
- `ConfigError::ValidationFailed` (missing required fields, invalid values)
- `ConfigError::SemanticError` (logical inconsistency, e.g., schema directory equals note directory)

##### Missing Local Config (Conditional)

If **no local config file** is found during Ascending Discovery:

**Current Behavior** (strict):
```rust
let vault_root = resolve_vault_context(cwd, cli_overrides)?;
if vault_root.is_none() {
    return Err(Error::NoVaultFound { searched_path: cwd });
}
```

**Future Behavior** (interactive, out of scope for this PRD):
- Prompt user: "No vault found. Create lithos.toml in current directory? [y/N]"
- If yes: Generate default config, write to `.lithos/lithos.toml`, continue
- If no: Exit with error

##### Global Config (Optional)

Global config (`~/.config/lithos/lithos.toml`) is OPTIONAL. If missing, only local config is used.

If global config exists but is malformed, error is fatal (same as local config).

##### CLI Override Behavior

- `--config /path/to/config.toml`: Explicit path MUST exist and be valid (fatal if missing/malformed)
- `--vault /path/to/vault`: Overrides Ascending Discovery, config file at vault root MUST exist

#### 7.5: Context Routing Model

**Decision: Config-Driven Routing with ContextRouter**

`DiscoveryResult` contains all discovered files regardless of context. A separate `ContextRouter` partitions files by context using config-defined directory boundaries.

##### ContextRouter Design

```rust
/// Routes discovered files to context-specific processors.
pub struct ContextRouter<'config> {
    config: &'config Config,
}

impl<'config> ContextRouter<'config> {
    pub fn new(config: &'config Config) -> Self {
        Self { config }
    }

    /// Partition files by context using config directory boundaries.
    pub fn route(&self, result: &DiscoveryResult) -> Result<RoutedFiles, Error> {
        let mut routed = RoutedFiles::default();

        for file in &result.files {
            // Determine context from config-defined directory boundaries + format
            if self.is_template_file(&file)? {
                routed.templates.push(file.clone());
            } else if self.is_schema_file(&file)? {
                routed.schemas.push(file.clone());
            } else if self.is_note_file(&file)? {
                routed.notes.push(file.clone());
            }
            // Files not matching any context boundary are ignored (e.g., property bank files)
        }

        Ok(routed)
    }

    fn is_template_file(&self, file: &DiscoveredFile) -> Result<bool, Error> {
        // Template directory check (highest precedence)
        Ok(file.view.path().starts_with(self.config.template().directory()))
    }

    fn is_schema_file(&self, file: &DiscoveredFile) -> Result<bool, Error> {
        // Schema format check (TOML/JSON/YAML only)
        Ok(matches!(
            file.view.format(),
            FileFormat::Toml | FileFormat::Json | FileFormat::Yaml
        ) && file.view.path().starts_with(self.config.schema().directory()))
    }

    fn is_note_file(&self, file: &DiscoveredFile) -> Result<bool, Error> {
        // Note check (markdown files not in template directory)
        Ok(file.view.format() == FileFormat::Markdown
            && !file.view.path().starts_with(self.config.template().directory()))
    }
}

/// Files partitioned by context.
#[derive(Default)]
pub struct RoutedFiles {
    pub schemas: Vec<DiscoveredFile>,
    pub notes: Vec<DiscoveredFile>,
    pub templates: Vec<DiscoveredFile>,
}
```

##### Routing Precedence (Overlap Resolution)

Context boundary overlaps are unlikely but resolved via precedence:

1. **Template directory** (explicit config boundary, highest precedence)
2. **File format** (schema = TOML/JSON/YAML, implicit)
3. **Remaining markdown** (notes = markdown files not in template directory)

**Example**:
```rust
// If user sets template.directory = "templates/"
// File: templates/example.toml
// Result: Routed to Template context (directory precedence over format)
```

**Validation (Optional)**: Config validation MAY reject overlapping schema/template directories, but this is not required (user may intentionally place non-schema files in schema directory).

#### 7.6: Parallel vs Sequential Execution Analysis

**DECISION: Sequential Execution (Simplest)**

The PRD originally proposed parallel context processing, but redb's write transaction model requires careful analysis. After evaluation, this PRD adopts **sequential execution** for initial implementation.

##### Why Sequential?

**Pros**:
- Simplest orchestration (no coordination)
- No write contention
- Deterministic ordering (Schema → Note → Template)
- Easy to reason about restartability
- Per-file atomic commits preserved

**Cons**:
- No CPU parallelism (slower on multi-core)
- Schemas block notes block templates (sequential bottleneck)

##### Future Parallel Alternatives (Deferred)

**Alternative 2: Parallel CPU + Sequential Writes**
- Parse/validate in parallel (rayon)
- Write sequentially (batch per context)
- Requires memory buffering
- Loses per-file atomic commits

**Alternative 3: Per-File MVCC with Small Transactions**
- True parallelism (rayon thread pool)
- redb MVCC serializes writes automatically
- Requires redb MVCC performance validation

**Alternative 4: Discovery-First Sequential Persistence**
- Discovery completes before contexts start
- Contexts parallelize reads
- Batch writes minimize transactions

##### Recommendation

**Start with Sequential** (Alternative 1). This PRD implements sequential context processing. A future PRD will analyze profiling data and choose a parallel execution model if warranted.

**Event Logging Pattern**: See Section 6 "Typestate-Driven Embedded Commits" for atomic state + event writes within the same transaction closure.

---

### 8. Reindex Policy

**Decision: Freshness Checking by Default, Explicit Full Scan Triggers**

#### 8.1: Terminology (Strict Definitions)

**BANNED**: "Incremental" (overloaded term), "Schema Migration" (ambiguous)

**PRECISE TERMS**:

| Term | Definition |
|------|------------|
| **Freshness Checking** | Built-in DirScanner metadata comparison against FILE_VIEWS (timestamp AND size) |
| **Full Scan (Vault)** | Bypass freshness checks globally across entire vault |
| **Full Scan (Context)** | Bypass freshness checks within specific context directory |
| **Targeted Scan** | Scan specific directory subtree (e.g., `notes/daily/`) with freshness checking |
| **Event-Driven Scan** | Process specific FileEvent list from file watcher (skip traversal, future enhancement) |

**Internal Architecture Changes**:
- **Schema Context Update**: User modifies `.md` schema files → standard processor update
- **Meta-Schema Migration**: Changes to `.schema.json` validation schemas
- **Object Model Migration**: Changes to Rust struct shapes/types
- **Internal Database Migration**: Changes to redb table definitions/binary format

#### 8.2: Full Scan Triggers

**Default**: Freshness checking (metadata comparison via `FileView.recorded_at` + size)

**Full Scan Scope Definitions**:

- **Full Vault Scan**: Globally bypass freshness checks across the **entire vault** (treat all files as potentially stale, regardless of metadata)
- **Full Context Scan**: Bypass freshness checks **only within a targeted directory subtree** (e.g., `SchemaConfigSpec.directory` for schema files only)

**Orchestrator Constraint**: The orchestrator **MUST always prefer** a Full Context Scan over a Full Vault Scan if a configuration boundary change is localized to a specific context. Example: If only `SchemaConfigSpec.directory` changed, trigger Full Context Scan (schema directory only), not Full Vault Scan.

**Full Scan Triggers**:

| Trigger | Scope | Detection | Example |
|---------|-------|-----------|---------|
| **Uninitialized DB** | Full Vault | Automatic | Empty FILE_VIEWS table → first run |
| **Explicit --force** | Vault OR Context | User CLI flag | `lithos index --force` (vault)<br>`lithos schema --force` (context) |
| **Database Corruption** | Full Vault | Automatic | redb integrity check fails |
| **Internal Database Migration** | Full Vault | Automatic | Version table mismatch vs binary |
| **Config Boundary Changes** | Vault OR Context | Automatic | Entry hash changed in `ConfigHashView` |

#### 8.3: Config Processing & Boundary Detection

**Config Identity Model** (decoupled from vault FileView):

```rust
/// Supported structured configuration file formats.
///
/// Discovery precedence (highest to lowest): TOML > JSON > YAML > YML
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum StructuredFileFormat {
    Toml,
    Json,
    Yaml,
    Yml,
}

impl StructuredFileFormat {
    /// Discovery precedence order (highest to lowest).
    pub const PRECEDENCE: [Self; 4] = [Self::Toml, Self::Json, Self::Yaml, Self::Yml];

    /// File extension without leading dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Yml => "yml",
        }
    }

    /// Lower value = higher precedence.
    pub const fn precedence_rank(self) -> u8 {
        match self {
            Self::Toml => 0,
            Self::Json => 1,
            Self::Yaml => 2,
            Self::Yml => 3,
        }
    }

    /// Detect format from file path extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Detect format from extension string (without leading dot).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            "yaml" => Some(Self::Yaml),
            "yml" => Some(Self::Yml),
            _ => None,
        }
    }
}

/// Global configuration discovery locations.
///
/// Discovery order: ExplicitOverride > EnvironmentOverride > XdgConfig > UserConfig > SystemConfig
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalConfigLocation {
    /// Explicit path from `--config`.
    ExplicitOverride(PathBuf),

    /// Path from `$LITHOS_CONFIG_FILE`.
    EnvironmentOverride(PathBuf),

    /// `$XDG_CONFIG_HOME/lithos/lithos.{toml,json,yaml,yml}`
    XdgConfig,

    /// `~/.config/lithos/lithos.{toml,json,yaml,yml}`
    UserConfig,

    /// `/etc/lithos/lithos.{toml,json,yaml,yml}`
    SystemConfig,
}

/// Local configuration discovery locations.
///
/// Supported file patterns per location:
/// - RootConfigFile: `<root>/lithos.{toml,json,yaml,yml}`
/// - HiddenRootConfigFile: `<root>/.lithos.{toml,json,yaml,yml}`
/// - ConfigDirectoryFile: `<root>/.lithos/config.{toml,json,yaml,yml}`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalConfigLocation {
    RootConfigFile,
    HiddenRootConfigFile,
    ConfigDirectoryFile,
}

impl LocalConfigLocation {
    /// Generates a concrete candidate path for a location/format pair.
    pub fn candidate_path(self, root: &Path, format: StructuredFileFormat) -> PathBuf {
        let ext = format.extension();
        match self {
            Self::RootConfigFile => root.join(format!("lithos.{ext}")),
            Self::HiddenRootConfigFile => root.join(format!(".lithos.{ext}")),
            Self::ConfigDirectoryFile => root.join(".lithos").join(format!("config.{ext}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigLocation {
    Global(GlobalConfigLocation),
    Local(LocalConfigLocation),
}

/// Concrete configuration file selected or discovered during config discovery.
///
/// - `location`: Why was this path searched?
/// - `path`: Which file was found?
/// - `format`: How should it be parsed?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDiscoveryResult {
    pub location: ConfigLocation,
    pub path: PathBuf,
    pub format: StructuredFileFormat,
}

// NEW: Config-specific views (NOT using vault FileView)
pub struct GlobalConfigFileView {
    pub location: ConfigLocation,
    pub path: PathBuf,
    pub format: StructuredFileFormat,
    pub metadata: FileMetadata,        // ✅ Directly embedded (NOT FileView)
    pub hash_state: ConfigHashView,    // Embedded hash state
}

pub struct LocalConfigFileView {
    pub location: ConfigLocation,
    pub path: PathBuf,
    pub format: StructuredFileFormat,
    pub metadata: FileMetadata,        // ✅ Directly embedded (NOT FileView)
    pub hash_state: ConfigHashView,
}

// NEW: Config hash state (NO separate DiscoveryMetadata table)
pub struct ConfigHashView {
    pub content_hash: Blake3Hash,           // Full file hash
    pub entry_hashes: BTreeMap<String, Blake3Hash>,  // Per-entry granular hashes
}

impl HasContentHash for ConfigHashView { /* ... */ }
impl HasEntryHashes for ConfigHashView { /* ... */ }
```

**Rationale for Direct FileMetadata Embedding**:

`GlobalConfigFileView` and `LocalConfigFileView` **directly embed** `fs::metadata::FileMetadata` rather than using the standard vault `FileView`.

This is a **load-bearing architectural constraint** necessary to sever global configurations from the vault-relative `PathKey` graph, thereby preventing circular dependencies during initialization:
- Global config files exist **outside** the vault boundary (e.g., `~/.config/lithos/lithos.toml`)
- Vault's `FileView` requires a vault-relative `PathKey` (e.g., `"notes/daily/2026.md"`)
- Using `FileView` for global config would violate vault's path invariants
- Direct `FileMetadata` embedding shares core filesystem logic (timestamp, size) without coupling to vault's path model

**Phase 2 Execution (Corrected Hash Comparison Flow)**:

1. **Freshness check** against `FileMetadata` (timestamp + size)
2. **If stale**, parse into `RawGlobalConfig`/`RawVaultConfig` structs
3. **Hash Raw* struct contents** → compare against stored `ConfigHashView.content_hash`
4. **Boundary change detection**: Compare specific entry hashes **inside `ConfigHashView`** (e.g., `entry_hashes.get("extensions")`, `entry_hashes.get("exclusions")`) to detect if discovery boundaries changed
5. **If boundaries changed** (extensions/exclusions), trigger appropriate Full Scan (Vault or Context)

**Critical Clarification**: `DiscoveryConfigSpec` is strictly the **output handoff** passed to the discovery scanner. It is **NOT the target** of hash comparison. Hash comparisons target the **`ConfigHashView` stored in config views**, which tracks granular per-entry hashes of the raw configuration file contents.

#### 8.3.1: Multiple Config Format Discovery

**Precedence with Stability**: If multiple config formats exist at the same location (e.g., both `lithos.toml` and `lithos.json`), use documented precedence with DB-based format stability.

**Precedence Order** (highest to lowest): `toml` > `json` > `yaml` > `yml`

**Discovery Algorithm**:

```rust
/// Find all config files that exist for a given local location.
pub fn find_local_config_candidates(
    root: &Path,
    location: LocalConfigLocation,
) -> Vec<ConfigDiscoveryResult> {
    StructuredFileFormat::PRECEDENCE
        .into_iter()
        .filter_map(|format| {
            let path = location.candidate_path(root, format);
            path.exists().then(|| ConfigDiscoveryResult {
                location: ConfigLocation::Local(location),
                path,
                format,
            })
        })
        .collect()
}

/// Selects the final config file when multiple formats exist.
pub fn select_config_candidate(
    mut candidates: Vec<ConfigDiscoveryResult>,
    persisted_format: Option<StructuredFileFormat>,
) -> Option<ConfigDiscoveryResult> {
    match candidates.len() {
        0 => None,
        1 => candidates.pop(),
        _ => {
            // Log warning about multiple configs
            let formats: Vec<_> = candidates.iter().map(|c| c.format.extension()).collect();
            warn!("Multiple configuration files detected: {}. Precedence: toml > json > yaml > yml", formats.join(", "));

            // Sort by precedence
            candidates.sort_by_key(|candidate| candidate.format.precedence_rank());

            // Prefer previously persisted format (stability)
            if let Some(format) = persisted_format {
                if let Some(candidate) = candidates.iter().find(|c| c.format == format) {
                    info!("Using previously persisted config format: {}", format.extension());
                    return Some(candidate.clone());
                }
            }

            // Use highest precedence (first after sort)
            candidates.into_iter().next()
        }
    }
}
```

**Behavior**:
- **Always log warning** when multiple formats detected (helps user notice duplicates)
- **Prefer persisted format** if it still exists (prevents unexpected format switching)
- **Fall back to precedence** if no persisted format or persisted file deleted

**NOT an error**: Multiple formats are valid but discouraged. User may keep multiple for testing or migration.

**Future Enhancement**: `lithos config doctor` command to detect and interactively resolve duplicates.

**Reference**: Follows ESLint/Prettier precedence pattern.

#### 8.3.2: Config Boundary Change Detection & Behavior

**Granular Hash Comparison**: Config boundary changes are detected by comparing entry hashes inside `ConfigHashView`:

```rust
// Phase 2: After parsing config
let old_view = config_repo.find_local_config(&vault_root)?;
let old_extensions = old_view.hash_state.entry_hashes.get("extensions");
let new_extensions_hash = blake3_hash(&parsed_config.extensions);

if old_extensions != Some(&new_extensions_hash) {
    // Extensions boundary changed
    determine_scan_scope(&old_view, &parsed_config)?;
}
```

**Behavior by Change Type**:

| Change Type | Database Action | Scan Required? | Scan Scope |
|-------------|-----------------|----------------|------------|
| **Extension Added** (e.g., add `.pdf`) | None (new files not in DB yet) | Yes | Full Context Scan (new extension only) |
| **Extension Removed** (e.g., remove `.md`) | Delete FileView records matching removed extension | No | Delete only |
| **Exclusion Added** (e.g., add `drafts/`) | Delete FileView records under excluded path | No | Delete only |
| **Exclusion Removed** (e.g., remove `archive/`) | None (previously excluded files not in DB) | Yes | Targeted Scan (removed exclusion path) |
| **Context Directory Changed** (e.g., `schemas/` → `schema_files/`) | Delete old directory, scan new directory | Yes | Full Context Scan (new directory) |

**Implementation Strategy**:

```rust
pub enum BoundaryChange {
    ExtensionAdded { extension: String },
    ExtensionRemoved { extension: String },
    ExclusionAdded { pattern: PathPattern },
    ExclusionRemoved { pattern: PathPattern },
    ContextDirectoryChanged { old: PathKey, new: PathKey },
}

impl BoundaryChange {
    pub fn required_action(&self) -> BoundaryAction {
        match self {
            Self::ExtensionAdded { .. } => BoundaryAction::ScanNewExtension,
            Self::ExtensionRemoved { extension } => BoundaryAction::DeleteFiles { extension },
            Self::ExclusionAdded { pattern } => BoundaryAction::DeleteFiles { pattern },
            Self::ExclusionRemoved { pattern } => BoundaryAction::ScanPath { pattern },
            Self::ContextDirectoryChanged { old, new } => BoundaryAction::DeleteAndScan { old, new },
        }
    }
}
```

**Orchestrator Constraint (Updated)**:

The orchestrator MUST detect the specific boundary change type and apply the appropriate action:
- **Additive changes** (new extensions/unexcluded paths): Trigger targeted scan
- **Subtractive changes** (removed extensions/new exclusions): Delete matching records, no scan
- **Mixed changes**: Apply delete actions first, then scan actions

**Optimization**: If multiple context boundaries changed (e.g., schema + note extensions), prefer independent context scans over a full vault scan.

#### 8.4: Discovery Scope (Runtime Parameter)

**Compilation to DirScanInput** (uses existing infrastructure):
```rust
impl DiscoveryScope {
    pub fn to_scan_inputs(&self, spec: &DiscoveryConfigSpec) -> Vec<DirScanInput> {
        match self {
            Self::FullVault { bypass_freshness } => {
                vec![DirScanInput {
                    root: spec.root.clone(),
                    pattern: None,
                    bypass_freshness: *bypass_freshness,
                }]
            }
            Self::Contexts { contexts, bypass_freshness } => {
                contexts.iter().map(|ctx| {
                    DirScanInput {
                        root: spec.root.join(&ctx.directory),
                        pattern: Some(spec.extensions_for(ctx.context_type)),
                        bypass_freshness: *bypass_freshness,
                    }
                }).collect()
            }
            // ... other variants
        }
    }
}

// Uses existing DirScanner (NOT raw WalkDir)
let scan_inputs = scope.to_scan_inputs(&discovery_spec);
for input in scan_inputs {
    let results = DirScanner::scan(input, repository)?;
    // ...
}
```

**Key Properties**:
- ✅ Supports parallel context scans
- ✅ Compiles to existing `DirScanInput` infrastructure
- ✅ CLI-driven scope selection (NOT stored in config)

---

### 9. Table Ownership & Repository Pattern

**Decision: Discovery Owns All Vault Tables, Contexts Read-Only**

#### 9.1: Segregated Repository Interfaces

**Codebase Convention**: Repositories follow a clean module-local pattern without redundant context prefixes.

**Config (config module)**:
```rust
pub trait ReadRepository {
    fn find_global_config(&self) -> Result<Option<GlobalConfigFileView>>;
    fn find_local_config(&self, vault_root: &VaultRoot) -> Result<Option<LocalConfigFileView>>;
}

pub trait WriteRepository {
    fn persist_global_config(&self, txn: &mut WriteTransaction, view: &GlobalConfigFileView) -> Result<()>;
    fn persist_local_config(&self, txn: &mut WriteTransaction, view: &LocalConfigFileView) -> Result<()>;
}

pub trait Repository: ReadRepository + WriteRepository {}
```

**Discovery (discovery module)**:
```rust
// Read operations (contexts query this)
pub trait ReadRepository {
    fn find_file_by_path(&self, path: &PathKey) -> Result<Option<FileView>>;
    fn find_file_by_id(&self, id: FileId) -> Result<Option<FileView>>;
    fn find_files_by_basename(&self, name: &str) -> Result<Vec<FileView>>;
    fn find_files_by_parent(&self, parent: DirId) -> Result<Vec<FileView>>;
    fn find_files_by_format(&self, format: FileFormat) -> Result<Vec<FileView>>;
    fn find_dir_by_path(&self, path: &PathKey) -> Result<Option<DirView>>;
}

// Write operations (ONLY discovery processor)
pub trait WriteRepository {
    fn persist_file_views(&self, txn: &mut WriteTransaction, files: &[FileView]) -> Result<()>;
    fn persist_dir_views(&self, txn: &mut WriteTransaction, dirs: &[DirView]) -> Result<()>;
    fn delete_files(&self, txn: &mut WriteTransaction, ids: &[FileId]) -> Result<()>;
    fn delete_dirs(&self, txn: &mut WriteTransaction, ids: &[DirId]) -> Result<()>;
}

// Unified (codebase convention)
pub trait Repository: ReadRepository + WriteRepository {}
```

**Context Pattern** (Schema/Note/Template):
```rust
// In schema/repository.rs
pub trait ReadRepository: vault::ReadRepository {
    fn find_by_file_id(&self, file_id: FileId) -> Result<Option<Schema>>;
    fn find_by_name(&self, name: &str) -> Result<Option<Schema>>;
    // ... context-specific queries
}

pub trait WriteRepository {
    fn save(&self, txn: &mut WriteTransaction, schema: &Schema, file_id: FileId) -> Result<()>;
    fn save_many(&self, txn: &mut WriteTransaction, schemas: &[(Schema, FileId)]) -> Result<()>;
    fn delete_by_file_id(&self, txn: &mut WriteTransaction, file_id: FileId) -> Result<()>;
}

// Unified trait (codebase convention)
pub trait Repository: ReadRepository + WriteRepository + EventStore {}
```

**Repository Trait Unification**:

The codebase follows a consistent pattern where segregated `ReadRepository` and `WriteRepository` traits are unified under a single `Repository` trait **within their respective modules**:

```rust
pub trait Repository: ReadRepository + WriteRepository {}
```

This unification trait provides a convenient bound for code that requires both read and write access, while still preserving the segregation benefits (compile-time enforcement of read-only vs write access).

**NO redundant context prefixes**: Traits are NOT named `SchemaReadRepository`, `NoteWriteRepository`, etc. They are simply `ReadRepository`, `WriteRepository`, `Repository` within their module namespace.

**Ownership Rules**:
- ✅ Discovery = ONLY writer to FILE_VIEWS, DIR_VIEWS, path indexes, basename/parent/format indexes
- ✅ Contexts = read-only access to discovery tables via `vault::ReadRepository`
- ✅ Contexts = write their own aggregate tables (SCHEMAS, NOTES, etc.) via their own `WriteRepository`

#### 9.2: Identity Resolution (FileId as Universal Foreign Key)

**❌ BANNED**: Context-specific path indexes
- ~~SCHEMA_ID_BY_PATH~~
- ~~NOTE_ID_BY_PATH~~
- ~~TEMPLATE_ID_BY_PATH~~

**✅ REQUIRED**: Central path resolution via discovery

```rust
// Step 1: Query discovery's path index
let file_id = vault_repo.find_file_by_path(&path)?
    .ok_or(Error::FileNotFound)?
    .id();

// Step 2: Query context aggregate using FileId
let schema = schema_repo.find_by_file_id(file_id)?;
```

**Migration Impact**:
```rust
// OLD: Schema aggregate with SchemaId
pub struct Schema {
    id: SchemaId,  // ❌ REMOVE
    name: String,
    // ...
}

// NEW: Schema aggregate with FileId
pub struct Schema {
    file_id: FileId,  // ✅ ADD (source file identity)
    name: String,     // Still unique within schemas
    // ...
}

// OLD: Table primary key
pub const SCHEMAS: UuidTable<SchemaId, &[u8]> = UuidTable::new("schemas");

// NEW: Table primary key
pub const SCHEMAS: UuidTable<FileId, &[u8]> = UuidTable::new("schemas");
```

**Inheritance Graph Migration**:
```rust
// OLD: SchemaId edges
pub struct InheritanceGraph {
    nodes: HashMap<SchemaId, SchemaNode>,
    edges: Vec<(SchemaId, SchemaId)>,  // parent → child
}

// NEW: FileId edges
pub struct InheritanceGraph {
    nodes: HashMap<FileId, SchemaNode>,
    edges: Vec<(FileId, FileId)>,  // parent file → child file
}
```

#### 9.3: Event Table Schema

**Context-Owned Event Tables**:
```rust
// Discovery (vault module)
pub const DISCOVERY_EVENTS: EventTable<&[u8]> = EventTable::new("discovery_events");

// Schema (schema module)
pub const SCHEMA_EVENTS: EventTable<&[u8]> = EventTable::new("schema_events");

// Note (note module)
pub const NOTE_EVENTS: EventTable<&[u8]> = EventTable::new("note_events");

// Template (template module)
pub const TEMPLATE_EVENTS: EventTable<&[u8]> = EventTable::new("template_events");

// Config (config module)
pub const CONFIG_EVENTS: EventTable<&[u8]> = EventTable::new("config_events");
```

**Primary Key Semantics**:
- **Key Type**: `EventId(u64)` newtype (monotonically increasing sequence number)
- **Ordering**: Strict chronological ordering for deterministic replay
- **Uniqueness**: Auto-increment per context (no shared sequence)

**Serialization Strategy**:
All event tables use **rkyv** for zero-copy deserialization:
- Events serialized via `rkyv::to_bytes::<_, 256>(event)?.into_vec()`
- Stored as `AlignedVec` byte slices (`&[u8]`)
- Deserialized via `rkyv::from_bytes::<ArchivedSchemaEvent>(bytes)?`

**Reference**: `.scratch/pipeline-restartability-research.md`

---

## Testing Decisions

- Good tests validate external behavior at module seams (scan/classify/persist/result), not internal implementation detail.
- Discovery tests should cover:
  - Correct classification (new/stale/fresh/deleted).
  - Delta persistence (batch save/delete only where needed).
  - Path safety and root scoping guarantees.
  - Deterministic ordering and stable result output where expected.
- Context processor tests should cover behavior when fed discovery results (not direct scanner mocks unless needed).
- Hash trait tests should validate:
  - Content-only hash record behavior.
  - Entry/property hash diff behavior.
  - Mutating trait behavior consistency.
- Event store tests should validate:
  - Atomic state + event writes (same transaction).
  - Event rehydration and projector pattern.
  - Compaction behavior (completed/failed events deleted).
- Prior art in codebase includes typestate processor tests, repository seam tests, and scanner/path validation tests; new tests should follow those conventions.

## Out of Scope

- Final renaming or relocation from Vault module to a fully separate discovery module namespace.
- Full redesign of all repository/table architecture across the entire codebase.
- Public API exposure of hash traits beyond crate-private boundaries.
- Immediate removal of all old context discovery code in a single change.
- Any UI/CLI redesign unrelated to orchestration ordering.
- **Full domain-specific event modeling** across all contexts (handled in separate PRD).

## Locked-In Design Summary

### Critical Decisions Resolved (2026-05-29)

**All architectural blind spots resolved. Design fully locked.**

1. **Identity Migration** (✅ LOCKED):
   - FileId replaces SchemaId/NoteId everywhere
   - Schema inheritance graph uses FileId (file-to-file relationships)
   - Simpler identity model, fewer index tables
   - NO context-specific path indexes (use central FILE_ID_BY_PATH)

2. **Discovery Result Contract** (✅ LOCKED):
   - `DiscoveredFile { id, view, path, status }`
   - `DiscoveryStatus::Fresh` (not "Unchanged")
   - Embedded `FileView` (not flattened fields)
   - Deleted files in separate `Vec<FileId>`

3. **Path Storage** (✅ LOCKED):
   - `PathKey` with forward slashes in `FileView`/`DirView`
   - Cross-platform guarantee: forward slashes work on all OSes
   - Remove `PATH_BY_FILE_ID`/`PATH_BY_DIR_ID` reverse indexes
   - Add `recorded_at: SystemTime` to views

4. **Pipeline Restartability** (✅ LOCKED):
   - Context-specific event sourcing (separate tables per context)
   - Generic event infrastructure: `EventId`, `EventTable<V>`, `EventStore` trait
   - Projector pattern for state rehydration
   - Atomic state + event writes (same transaction, MVCC)
   - Dependency-aware log compaction
   - Event table primary key: EventId(u64) newtype
   - Universal rkyv serialization (NO bincode)

5. **Orchestration Policy** (✅ LOCKED):
   - 5-phase pipeline: Context Resolution → Config Hydration → State Rehydration → Discovery → Context Processing
   - **Config-first**: Config is prerequisite lens (NOT discovery consumer)
   - Dependency graph: Config → Discovery → {Schema, Note, Template}
   - Ascending Discovery algorithm for vault root resolution
   - Parallel context processing with MVCC-based decentralized commits
   - Config errors are fatal (strict fail-fast, no fallback)

6. **Reindex Policy** (✅ LOCKED):
   - Default: Freshness checking (metadata comparison)
   - Full Scan triggers: Uninitialized DB, explicit `--force`, DB corruption, Internal Database Migration, config boundary changes
   - Config-specific identity model (GlobalConfigFileView/LocalConfigFileView with embedded FileMetadata)
   - Granular entry-level hash comparison for boundary change detection (targets ConfigHashView, NOT DiscoveryConfigSpec)
   - Scan terminology: Freshness Checking, Full Scan (Vault/Context), Targeted Scan, Event-Driven Scan
   - DiscoveryScope compiles to DirScanInput (uses existing infrastructure)

7. **Table Ownership** (✅ LOCKED):
   - Discovery owns ALL vault tables (FILE_VIEWS, DIR_VIEWS, path indexes, basename/parent/format indexes)
   - Contexts are read-only consumers via `vault::ReadRepository`
   - FileId is universal foreign key (NO context-specific path indexes)
   - Repository pattern: `ReadRepository` + `WriteRepository` unified as `Repository` (NO redundant context prefixes)
   - Event tables: Context-owned with EventId(u64) keys, rkyv serialization

### ALL ARCHITECTURAL DECISIONS COMPLETE

**Status**: Design fully locked, comprehensively polished, ready for implementation.

### Reference Documentation

- **Architecture Decision Records**: `docs/adr/discovery/` (6 ADRs documenting all architectural decisions)
- **Cross-Platform Paths**: `.scratch/CROSS_PLATFORM_PATH_FINDINGS.md`
- **Pipeline Restartability**: `.scratch/pipeline-restartability-research.md`
- **Existing Processor Patterns**: GitNexus analysis of current typestate processors
