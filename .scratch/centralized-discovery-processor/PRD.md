# PRD: Centralized Discovery Processor

**Status**: locked-in-design
**Created**: 2026-05-25
**Updated**: 2026-05-28
**Context**: Architectural blind spots resolved through grilling session

---

## Problem Statement

Lithos currently repeats filesystem discovery logic across multiple contexts (notably Schema and Config), while Vault already maintains file and directory identity tables. This duplication increases maintenance cost, creates inconsistent discovery behavior, and makes it harder to evolve freshness checks and indexing safely.

The project needs a unified discovery engine that starts from filesystem primitives, persists canonical file/directory identity once, and then lets each context perform context-specific processing without re-implementing base discovery.

## Solution

Refactor the existing Vault module into the initial base of a discovery module (incrementally, without a full module move in this session). The discovery typestate processor will become the shared filesystem discovery engine and will:

- Run scoped scans using configurable scan input.
- Compare scan results against persisted views to classify freshness.
- Persist only deltas (new, stale metadata updates, deletions) rather than rewriting all records.
- Return a discovery result contract for context-specific processors.

Context processors (Schema, Note, Template, Config) remain standalone and consume discovery results as their first stage, then continue with context-specific parsing, hashing, validation, and persistence.

## User Stories

1. As a Lithos maintainer, I want one base discovery engine, so that file discovery behavior is consistent across contexts.
2. As a Lithos maintainer, I want to avoid duplicate scan code in Schema and Config, so that refactors are safer and faster.
3. As a Schema processor maintainer, I want discovery to provide canonical file identity, so that SchemaId can be replaced by FileId.
4. As a Note processor maintainer, I want discovery classifications (new/stale/unchanged/deleted), so that note ingestion can skip unnecessary work.
5. As a Template processor maintainer, I want indexed file metadata query support, so that template querying can evolve toward Obsidian-like behavior.
6. As a Config processor maintainer, I want config discovery to run first in orchestrated flows, so that downstream processors use resolved configuration.
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

## Implementation Decisions

### 1. Architecture & Boundaries
- **Discovery First**: Discovery remains an incremental refactor from the current Vault module first; full module renaming/re-homing is deferred.
- **Shared Engine**: The discovery typestate processor lives in the discovery layer and acts as the shared base filesystem discovery engine for all contexts.
- **Standalone Processors**: Context processors remain standalone and consume discovery results as input to their own pipelines, preserving independent evolution and parallel execution.
- **Parsing**: Parsing remains context-specific and is not owned by base discovery.

### 2. Scanning & Classification
- **Scoped Scans**: Discovery processing is scoped by scan input and should support partial or targeted scans.
- **Incremental Capabilities**: Incremental change ingestion will be introduced so that indexing does not always require a full directory traversal (e.g., allowing file/directory updates to trigger lean reprocessing).
- **Freshness Classification**: Discovery includes metadata-based comparison (timestamp AND size) to classify records by freshness; it does not blindly scan-and-write.
- **Result Contract**:
  ```rust
  pub struct DiscoveredFile {
      pub id: FileId,
      pub view: FileView,        // Embedded view with path, metadata, recorded_at
      pub path: FilePath,        // From scan, for immediate reads
      pub status: DiscoveryStatus,
  }

  pub enum DiscoveryStatus {
      New,      // Not in DB
      Fresh,    // Metadata unchanged (timestamp AND size match)
      Stale,    // Metadata changed (timestamp OR size differs)
      Deleted,  // In DB, not on filesystem
  }

  pub struct DiscoveryResult {
      pub files: Vec<DiscoveredFile>,       // New, Fresh, Stale only
      pub deleted_file_ids: Vec<FileId>,    // Separate collection
  }
  ```
- **Staleness Detection**: Uses existing `FileMetadata::is_timestamp_match()` and `FileMetadata::is_size_match()` methods. File is `Fresh` only if BOTH match; `Stale` if EITHER differs.

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

### 4. Persistence & Tables
- **Delta Persistence**: Discovery persists only deltas (new files, stale metadata updates, deletions) and uses batch repository operations for efficient writes/deletes.
- **Indices**: The basename index can be removed from general discovery concerns; retained indexes are path, parent, format, and primary views.

### 5. Hashing & Content Staleness
- **Context Ownership**: File-level content hashing is context-owned for freshness checks; discovery will not force file content hashing in `FileView`.
- **Structured Contexts**: Structured contexts (Schema/Config) require both content hash and entry/property hash indexing in their own view models.
- **Hash Contracts**: Hash capability contracts are crate-private and based on support hash primitives, utilizing traits: `HasContentHash`, `HasContentHashMut`, `HasEntryHashes`, `HasEntryHashesMut`.

### 6. Pipeline Resilience & Restartability

**Decision: Context-Specific Event Sourcing with Shared Infrastructure**

**Architecture Pattern**: Event sourcing enables complete pipeline restartability after crashes, preserving all completed work and providing audit trails for debugging.

#### Core Components

1. **Generic Event Store** (shared infrastructure in `db` module):
   ```rust
   pub struct EventStore<E> {
       db: Database,
       table_def: TableDefinition<u64, &'static [u8]>,
       _event: PhantomData<E>,
   }

   impl<E> EventStore<E> {
       pub fn append(&self, event: &E) -> Result<u64, DbError>;
       pub fn append_batch(&self, events: &[E]) -> Result<Vec<u64>, DbError>;
       pub fn load_all(&self) -> Result<Vec<E>, DbError>;
       pub fn compact(&self, completed_file_ids: &[FileId]) -> Result<(), DbError>;
   }
   ```

2. **Context-Specific Event Tables** (maintains bounded contexts):
   - `DISCOVERY_EVENTS` (vault module) - Discovery scan events
   - `SCHEMA_EVENTS` (schema module) - Schema processing events
   - `NOTE_EVENTS` (note module) - Note processing events
   - `TEMPLATE_EVENTS` (template module) - Template processing events
   - `CONFIG_EVENTS` (config module) - Config processing events

3. **Event Types Per Context** (intermediate typestate transitions):
   ```rust
   // Example: Schema events track full pipeline lifecycle
   pub enum SchemaEvent {
       Discovered { file_id: FileId, path: PathKey, discovered_at: SystemTime },
       ParseStarted { file_id: FileId, started_at: SystemTime },
       Parsed { file_id: FileId, parsed_at: SystemTime },
       PropertyBankReferenceExpanded { file_id: FileId, expanded_at: SystemTime },
       InheritanceResolved { file_id: FileId, parent_count: usize, resolved_at: SystemTime },
       SchemaPersisted { file_id: FileId, persisted_at: SystemTime },
       Completed { file_id: FileId, completed_at: SystemTime },
       Failed { file_id: FileId, error: String, failed_at: SystemTime },
   }
   ```

4. **Projector Pattern** (rehydrates state from events):
   ```rust
   pub struct PendingSchemaState {
       pub pending: HashMap<FileId, PathKey>,
       pub completed: HashSet<FileId>,
       pub failed: HashSet<FileId>,
   }

   impl PendingSchemaState {
       pub fn from_events(events: &[SchemaEvent]) -> Self {
           // Fold events into current state
       }

       pub fn pending_files(&self) -> Vec<FileId> {
           self.pending.keys().copied().collect()
       }
   }
   ```

#### Partial Success & Resumption Flow

**Scenario**: Discovery scans 1000 files → Schema processes 300 files → **CRASH** → Restart

**Recovery Process**:
1. **Rehydrate State**: Load all `SchemaEvent` from `SCHEMA_EVENTS` table
2. **Project State**: `PendingSchemaState::from_events()` identifies:
   - 300 completed files (via `Completed` events)
   - 700 pending files (via `Discovered` but no `Completed/Failed`)
3. **Resume Processing**: Process only the 700 pending files
4. **Emit Events**: Append `Completed/Failed` events as files finish
5. **Compact Log**: After all files complete, delete events for completed/failed files

**Key Benefits**:
- ✅ **Zero Work Lost**: All completed work preserved across crashes
- ✅ **Audit Trail**: Full history of state transitions for debugging
- ✅ **Bounded Context Isolation**: Schema crash cannot corrupt Note event log
- ✅ **Natural Typestate Fit**: Events emitted at typestate transitions

#### Batch Performance

**Discovery**: Batch commits every N=100 files
```rust
const BATCH_SIZE: usize = 100;
for batch in files.chunks(BATCH_SIZE) {
    // Atomic: persist views + append events
    repository.persist_discovery_batch(batch)?;
    event_store.append_batch(batch_events)?;
}
```

**Context Processing**: Per-file event logging
```rust
for file in pending_files {
    match process_schema_file(file) {
        Ok(_) => {
            event_store.append(&SchemaEvent::Completed { file_id })?;
        }
        Err(e) => {
            event_store.append(&SchemaEvent::Failed { file_id, error })?;
        }
    }
}
```

#### Dependency-Aware Cleanup

**Cleanup timing respects context dependencies**:
```
Discovery → Config → {Schema, Note, Template}
```

- **Immediate Cleanup**: Schema, Note, Template event logs (independent contexts)
- **Deferred Cleanup**: Config event log (after all dependents complete)
- **Final Cleanup**: Discovery events (after ALL contexts complete)

```rust
// Schema completes
event_store.compact(&completed_file_ids)?;  // ✅ Immediate

// All contexts complete
clear_config_events()?;       // ✅ After dependents
clear_discovery_events()?;    // ✅ After all
```

#### Performance Characteristics

- **Append-only writes**: Lock-free, fast (1-2ms per 100-file batch)
- **Redb single-writer**: No write contention (sequential by design)
- **Log compaction**: Keeps event tables bounded (delete completed/failed)
- **Rehydration cost**: O(N) where N = pending files (typically small after compaction)

**Reference**: `.scratch/pipeline-restartability-research.md`

## Testing Decisions

- Good tests validate external behavior at module seams (scan/classify/persist/result), not internal implementation detail.
- Discovery tests should cover:
  - Correct classification (new/stale/unchanged/deleted).
  - Delta persistence (batch save/delete only where needed).
  - Path safety and root scoping guarantees.
  - Deterministic ordering and stable result output where expected.
- Context processor tests should cover behavior when fed discovery results (not direct scanner mocks unless needed).
- Hash trait tests should validate:
  - Content-only hash record behavior.
  - Entry/property hash diff behavior.
  - Mutating trait behavior consistency.
- Prior art in codebase includes typestate processor tests, repository seam tests, and scanner/path validation tests; new tests should follow those conventions.

## Out of Scope

- Final renaming or relocation from Vault module to a fully separate discovery module namespace.
- Full redesign of all repository/table architecture across the entire codebase.
- Public API exposure of hash traits beyond crate-private boundaries.
- Immediate removal of all old context discovery code in a single change.
- Any UI/CLI redesign unrelated to orchestration ordering.

## Locked-In Design Summary

### Critical Decisions Resolved (2026-05-28)

1. **Identity Migration** (✅ LOCKED):
   - FileId replaces SchemaId/NoteId everywhere
   - Schema inheritance graph uses FileId (file-to-file relationships)
   - Simpler identity model, fewer index tables

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
   - Generic `EventStore<E>` infrastructure in `db` module
   - Projector pattern for state rehydration
   - Batch commits (N=100 for discovery, per-file for contexts)
   - Dependency-aware log compaction

### Remaining Open Questions

1. **Orchestration Policy**: Config-first execution, parallelization rules
2. **Reindex Policy**: Full vs partial/scoped scan triggers
3. **Table Ownership**: Which tables are discovery-owned vs context-owned post-refactor
4. **ADR**: Architecture Decision Record generation after all decisions locked

### Reference Documentation

- **Cross-Platform Paths**: `.scratch/CROSS_PLATFORM_PATH_FINDINGS.md`
- **Pipeline Restartability**: `.scratch/pipeline-restartability-research.md`
- **Existing Processor Patterns**: GitNexus analysis of current typestate processors
