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

// Phase 2: Config Hydration (stateless I/O, FAIL-FAST)
let config = ConfigBuilder::load(vault_root, repository)?;

// Phase 3: State Rehydration (database)
let db = Database::open(config.db_path())?;
let pending_state = EventStore::rehydrate_pending_work(&db)?;

// Phase 4: Filesystem Discovery (uses frozen config)
let discovery_spec = config.to_discovery_spec();
let scope = cli.discovery_scope();  // ← Runtime parameter
let discovery_result = DiscoveryEngine::run(&discovery_spec, scope, repository)?;

// Phase 5: Context Processing (PARALLEL, MVCC commits)
rayon::scope(|s| {
    s.spawn(|_| SchemaProcessor::process(discovery_result.schemas(), config, repository));
    s.spawn(|_| NoteProcessor::process(discovery_result.notes(), config, repository));
    s.spawn(|_| TemplateProcessor::process(discovery_result.templates(), config, repository));
});
```

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

#### 7.4: Parallelization (MVCC-Based Decentralized Commits)

**Execution**: Schema/Note/Template run in full parallel isolation using redb MVCC.

**Event Logging Pattern** (embedded in typestate transitions):
```rust
impl SchemaProcessor<Parsed, Review> {
    pub fn analyze(self) -> Result<SchemaProcessor<Analyzed, Review>, Error> {
        // 1. CPU-bound work (no lock)
        let analysis_result = self.analyze_properties()?;

        // 2. Short-lived write transaction
        let txn = self.event_store.begin_write()?;
        txn.append(&SchemaEvent::Analyzed {
            file_id: self.file_id,
            analyzed_at: SystemTime::now(),
        })?;
        txn.commit()?;  // ← Immediate lock release

        // 3. Typestate transition
        Ok(SchemaProcessor::new_analyzed(analysis_result))
    }
}
```

**Key Properties**:
- ✅ No scatter-gather bottleneck
- ✅ redb MVCC handles write contention
- ✅ Event log append = single write per transition
- ✅ Crash recovery via event replay

#### 7.5: Config Error Propagation (Strict Fail-Fast)

**Rule**: Config errors are fatal, non-recoverable.

```rust
let config = ConfigBuilder::new(vault_root, repository)
    .load()
    .map_err(PipelineError::ConfigLoadFailed)?;  // ❌ HALT HERE
```

**NO fallback to defaults. NO silent degradation.**

---

### 8. Reindex Policy

**Decision: Freshness Checking by Default, Explicit Full Scan Triggers**

#### 8.1: Terminology (Strict Definitions)

**BANNED**: "Incremental" (overloaded term), "Schema Migration" (ambiguous)

**PRECISE TERMS**:

| Term | Definition |
|------|------------|
| **Freshness Checking** | Built-in DirScanner metadata comparison against FILE_VIEWS |
| **Full Scan (Vault)** | Bypass freshness checks globally across entire vault |
| **Full Scan (Context)** | Bypass freshness checks within specific context directory |
| **Targeted Scan** | Scan specific directory subtree (e.g., `notes/daily/`) |
| **Event-Driven Scan** | Process specific FileEvent list (skip traversal, future file watcher) |

**Internal Architecture Changes**:
- **Schema Context Update**: User modifies `.md` schema files → standard processor update
- **Meta-Schema Migration**: Changes to `.schema.json` validation schemas
- **Object Model Migration**: Changes to Rust struct shapes/types
- **Internal Database Migration**: Changes to redb table definitions/binary format

#### 8.2: Full Scan Triggers

**Default**: Freshness checking (metadata comparison via `FileView.recorded_at` + size)

**Full Scan Overrides**:

| Trigger | Scope | Detection | Example |
|---------|-------|-----------|---------|
| **Uninitialized DB** | Full Vault | Automatic | Empty FILE_VIEWS table → first run |
| **Explicit --force** | Vault OR Context | User CLI flag | `lithos index --force` (vault)<br>`lithos schema --force` (context) |
| **Database Corruption** | Full Vault | Automatic | redb integrity check fails |
| **Internal Database Migration** | Full Vault | Automatic | Version table mismatch vs binary |
| **Config Boundary Changes** | Vault OR Context | Automatic | DiscoveryConfigSpec boundary hash changed |

#### 8.3: Config Processing & Boundary Detection

**Config Identity Model** (decoupled from vault FileView):

```rust
// NEW: Config-specific views (NOT using vault FileView)
pub struct GlobalConfigFileView {
    pub location: GlobalConfigLocation,
    pub metadata: FileMetadata,        // ✅ Shared fs::metadata logic
    pub hash_state: ConfigHashView,    // Embedded hash state
}

pub struct LocalConfigFileView {
    pub location: LocalConfigLocation,
    pub metadata: FileMetadata,
    pub hash_state: ConfigHashView,
}

pub enum GlobalConfigLocation {
    XdgConfigHome(PathBuf),     // ~/.config/lithos/lithos.toml
    ExplicitOverride(PathBuf),  // --config flag
}

pub enum LocalConfigLocation {
    HiddenRoot(PathBuf),        // <vault>/.lithos/lithos.toml
    RootFile(PathBuf),          // <vault>/lithos.toml
}

// NEW: Config hash state (NO separate DiscoveryMetadata table)
pub struct ConfigHashView {
    pub content_hash: Blake3Hash,           // Full file hash
    pub entry_hashes: BTreeMap<String, Blake3Hash>,  // Per-entry granular hashes
}

impl HasContentHash for ConfigHashView { /* ... */ }
impl HasEntryHashes for ConfigHashView { /* ... */ }
```

**Phase 2 Execution**:
1. Freshness check against `FileMetadata` (timestamp + size)
2. If stale, parse into `RawGlobalConfig`/`RawVaultConfig`
3. Hash raw config contents → compare against stored `ConfigHashView`
4. **Boundary change detection**: Compare `DiscoveryConfigSpec` boundary hashes against specific entry hashes in `ConfigHashView`
5. If boundaries changed (extensions/exclusions), trigger appropriate Full Scan (Vault or Context)

**Rationale**:
- Global config outside vault → cannot use vault-relative PathKey
- Embedded FileMetadata → shares filesystem logic without duplication
- ConfigHashView → granular entry-level change detection
- Boundary hashes → automatic Full Scan trigger on config changes

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

**Discovery (Vault Module)**:
```rust
// Read operations (contexts query this)
pub trait DiscoveryReadRepository {
    fn find_file_by_path(&self, path: &PathKey) -> Result<Option<FileView>>;
    fn find_file_by_id(&self, id: FileId) -> Result<Option<FileView>>;
    fn find_files_by_basename(&self, name: &str) -> Result<Vec<FileView>>;
    fn find_files_by_parent(&self, parent: DirId) -> Result<Vec<FileView>>;
    fn find_files_by_format(&self, format: FileFormat) -> Result<Vec<FileView>>;
    fn find_dir_by_path(&self, path: &PathKey) -> Result<Option<DirView>>;
}

// Write operations (ONLY discovery processor)
pub trait DiscoveryWriteRepository {
    fn persist_file_views(&self, files: &[FileView]) -> Result<()>;
    fn persist_dir_views(&self, dirs: &[DirView]) -> Result<()>;
    fn delete_files(&self, ids: &[FileId]) -> Result<()>;
    fn delete_dirs(&self, ids: &[DirId]) -> Result<()>;
}

// Unified
pub trait DiscoveryRepository: DiscoveryReadRepository + DiscoveryWriteRepository {}
```

**Context Pattern** (Schema/Note/Template):
```rust
pub trait SchemaReadRepository: DiscoveryReadRepository {
    fn find_by_file_id(&self, file_id: FileId) -> Result<Option<Schema>>;
    fn find_by_name(&self, name: &str) -> Result<Option<Schema>>;
    // ... context-specific queries
}

pub trait SchemaWriteRepository {
    fn save(&self, schema: &Schema, file_id: FileId) -> Result<()>;
    fn save_many(&self, schemas: &[(Schema, FileId)]) -> Result<()>;
    fn delete_by_file_id(&self, file_id: FileId) -> Result<()>;
}

pub trait SchemaRepository: SchemaReadRepository + SchemaWriteRepository {}
```

**Ownership Rules**:
- ✅ Discovery = ONLY writer to FILE_VIEWS, DIR_VIEWS, path indexes, basename/parent/format indexes
- ✅ Contexts = read-only access via DiscoveryReadRepository
- ✅ Contexts = write their own aggregate tables (SCHEMAS, NOTES, etc.)

#### 9.2: Identity Resolution (FileId as Universal Foreign Key)

**❌ BANNED**: Context-specific path indexes
- ~~SCHEMA_ID_BY_PATH~~
- ~~NOTE_ID_BY_PATH~~
- ~~TEMPLATE_ID_BY_PATH~~

**✅ REQUIRED**: Central path resolution via discovery

```rust
// Step 1: Query discovery's path index
let file_id = discovery_repo.find_file_by_path(&path)?
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
pub const DISCOVERY_EVENTS: Table<u64, &[u8]> = Table::new("discovery_events");

// Schema (schema module)
pub const SCHEMA_EVENTS: Table<u64, &[u8]> = Table::new("schema_events");

// Note (note module)
pub const NOTE_EVENTS: Table<u64, &[u8]> = Table::new("note_events");

// Template (template module)
pub const TEMPLATE_EVENTS: Table<u64, &[u8]> = Table::new("template_events");

// Config (config module)
pub const CONFIG_EVENTS: Table<u64, &[u8]> = Table::new("config_events");
```

**Primary Key Semantics**:
- **Key Type**: `u64` (monotonically increasing sequence number)
- **Ordering**: Strict chronological ordering for deterministic replay
- **Uniqueness**: Auto-increment per context (no shared sequence)

**EventStore Generic Implementation**:
```rust
pub struct EventStore<E> {
    db: Database,
    table_def: TableDefinition<u64, &'static [u8]>,
    next_seq: AtomicU64,  // ✅ Monotonic sequence generator
    _event: PhantomData<E>,
}

impl<E: Serialize + DeserializeOwned> EventStore<E> {
    pub fn append(&self, event: &E) -> Result<u64, DbError> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let bytes = bincode::serialize(event)?;

        self.db.write(|txn| {
            let mut table = txn.open_table(self.table_def)?;
            table.insert(seq, bytes.as_slice())?;
            Ok(seq)
        })
    }
}
```

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
   - Generic `EventStore<E>` infrastructure in `db` module
   - Projector pattern for state rehydration
   - Batch commits (N=100 for discovery, per-file for contexts)
   - Dependency-aware log compaction
   - Event table primary key: u64 monotonic sequence numbers

5. **Orchestration Policy** (✅ LOCKED):
   - 5-phase pipeline: Context Resolution → Config Hydration → State Rehydration → Discovery → Context Processing
   - Config is prerequisite lens (NOT discovery consumer)
   - Ascending Discovery algorithm for vault root resolution
   - Parallel context processing with MVCC-based decentralized commits
   - Config errors are fatal (strict fail-fast, no fallback)

6. **Reindex Policy** (✅ LOCKED):
   - Default: Freshness checking (metadata comparison)
   - Full Scan triggers: Uninitialized DB, explicit `--force`, DB corruption, Internal Database Migration, config boundary changes
   - Config-specific identity model (GlobalConfigFileView/LocalConfigFileView with embedded ConfigHashView)
   - Granular entry-level hash comparison for boundary change detection
   - Scan terminology: Freshness Checking, Full Scan (Vault/Context), Targeted Scan, Event-Driven Scan
   - DiscoveryScope compiles to DirScanInput (uses existing infrastructure)

7. **Table Ownership** (✅ LOCKED):
   - Discovery owns ALL vault tables (FILE_VIEWS, DIR_VIEWS, path indexes, basename/parent/format indexes)
   - Contexts are read-only consumers via DiscoveryReadRepository
   - FileId is universal foreign key (NO context-specific path indexes)
   - Segregated repository pattern: [Context]ReadRepository + [Context]WriteRepository
   - Event tables: Context-owned with u64 monotonic sequence keys

### ALL ARCHITECTURAL DECISIONS COMPLETE

**Status**: Design fully locked, ready for ADR generation and implementation.

### Reference Documentation

- **Architecture Decision Records**: `docs/adr/discovery/` (6 ADRs documenting all architectural decisions)
- **Cross-Platform Paths**: `.scratch/CROSS_PLATFORM_PATH_FINDINGS.md`
- **Pipeline Restartability**: `.scratch/pipeline-restartability-research.md`
- **Existing Processor Patterns**: GitNexus analysis of current typestate processors
