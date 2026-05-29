# PRD Corrections - Centralized Discovery Processor

**Status**: Awaiting user approval before applying to PRD.md
**Created**: 2026-05-29
**Issues Addressed**: 18 critical/moderate/minor + 4 second-pass issues

---

## How to Use This Document

Each issue below contains:
1. **Issue ID & Severity**
2. **Current PRD text** (exact lines)
3. **Problem description**
4. **Proposed correction** (specific text replacement)
5. **Impact assessment**

After your review and approval, I will apply all approved corrections to PRD.md in a single atomic commit.

---

## CATEGORY 1: CRITICAL ARCHITECTURAL FIXES

### Issue 1.1: DiscoveredFile.view Semantics Clarification (HIGH)

**Current PRD (lines 66-84)**:
```rust
pub struct DiscoveredFile {
    pub id: FileId,
    pub view: FileView,        // Embedded view with path, metadata, recorded_at
    pub path: FilePath,        // From scan, for immediate reads
    pub status: DiscoveryStatus,
}
```

**Problem**: Ambiguous whether `view` is the OLD persisted view or the NEW/current view after classification.

**Proposed Correction**:

Replace lines 66-84 with:

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

**Add new subsection after line 86**:

```markdown
- **Freshness Comparison Logic**: Freshness is determined by comparing scanned `FsFile.metadata` (from `DirScanner`) against persisted `FileView.metadata` in the database. The `recorded_at` field in `FileView` is NOT part of freshness comparison—it tracks when the view was last persisted, not when the filesystem entity was modified. Comparison uses existing `FileMetadata::is_timestamp_match()` and `FileMetadata::is_size_match()` methods. A file is `Fresh` only if BOTH match; `Stale` if EITHER differs.
```

**Impact**: ✅ Clarifies view semantics, preserves current structure, aligns with user guidance.

---

### Issue 1.2: DirView Classification & Typestate Processor Pipeline (CRITICAL)

**Current PRD**: No explicit typestate processor pipeline definition. `DirView` mentioned (lines 109-117, 655) but no classification logic.

**Problem**: Directories lack delta persistence strategy, no classification flow defined.

**Proposed Correction**:

**Add new Section 2.5 after line 131**:

```markdown
### 2.5: Filesystem Discovery Typestate Processor

**Decision: Centralized Typestate Processor Replacing Context-Specific Discovery**

The filesystem discovery processor follows the same typestate pattern as `schema/property_bank_processor.rs`, adapted for filesystem-level discovery. It processes BOTH files AND directories through a multi-stage pipeline.

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
```

**Impact**: ✅ Defines complete processor pipeline, aligns with existing patterns, covers directories.

---

### Issue 1.3: Config Location Enums Completion (MODERATE)

**Current PRD (lines 535-557)**:
```rust
pub enum GlobalConfigLocation {
    XdgConfigHome(PathBuf),     // ~/.config/lithos/lithos.toml
    ExplicitOverride(PathBuf),  // --config flag
}

pub enum LocalConfigLocation {
    HiddenRoot(PathBuf),        // <vault>/.lithos/lithos.toml
    RootFile(PathBuf),          // <vault>/lithos.toml
}
```

**Problem**: Missing config discovery candidates from Ascending Discovery (lines 397-403): `.lithos.{toml|json|yaml|yml}` and `.lithos/config.{toml|json|yaml|yml}`.

**Proposed Correction**:

Replace lines 535-557 with:

```rust
/// Provenance of a discovered config file.
///
/// Variants represent logical locations, not resolved paths.
/// Multiple file formats may be candidates at each location.
pub enum GlobalConfigLocation {
    /// Explicit path from `--config` CLI flag.
    ExplicitOverride,
    /// Path from `LITHOS_CONFIG_PATH` environment variable.
    EnvironmentOverride,
    /// `$XDG_CONFIG_HOME/lithos/lithos.{toml,json,yaml,yml}`
    XdgConfig,
    /// `~/.config/lithos/lithos.{toml,json,yaml,yml}` (fallback if XDG unset)
    UserConfig,
    /// `/etc/lithos/lithos.{toml,json,yaml,yml}` (system-wide)
    SystemConfig,
}

pub enum LocalConfigLocation {
    /// `<vault>/lithos.{toml,json,yaml,yml}` (root-level visible config)
    RootConfigFile,
    /// `<vault>/.lithos.{toml,json,yaml,yml}` (root-level hidden config)
    HiddenRootConfigFile,
    /// `<vault>/.lithos/config.{toml,json,yaml,yml}` (config directory)
    ConfigDirectoryFile,
}

/// Combined config location (global or local).
pub enum ConfigLocation {
    Global(GlobalConfigLocation),
    Local(LocalConfigLocation),
}

/// Result of config discovery with provenance.
pub struct ConfigDiscoveryResult {
    pub location: ConfigLocation,
    pub path: PathBuf,  // Resolved absolute path to discovered config file
}
```

**Add clarification after line 403**:

```markdown
**Config Discovery Enumerates Multiple Candidates**:

For each logical location, discovery checks multiple file formats in precedence order:
1. `lithos.toml`
2. `lithos.json`
3. `lithos.yaml`
4. `lithos.yml`

First discovered file wins. If multiple formats exist at the same location, the PRD does NOT specify behavior (implementation may warn, error, or follow precedence).
```

**Impact**: ✅ Aligns with Ascending Discovery, supports multiple formats, clarifies provenance model.

---

### Issue 1.4: Discovery Event Semantics Clarification (MODERATE)

**Current PRD (lines 214-218, 320-332)**: Discovery events shown as `DiscoveryEvent::Discovered { file_id }` but semantics unclear.

**Problem**: "Discovered" is ambiguous (scanned? classified? persisted?). Discovery described as "stateless" but has events.

**Proposed Correction**:

Replace line 214 with:

```rust
// Discovery (vault module) - Filesystem discovery processor events
pub const DISCOVERY_EVENTS: EventTable<&[u8]> = EventTable::new("discovery_events");
```

**Replace lines 320-332 with**:

```rust
/// Filesystem discovery events track typestate transitions for restartability.
///
/// These events enable resuming discovery after crashes without re-scanning
/// completed batches.
#[derive(Archive, Deserialize, Serialize)]
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

// Batch commit example with corrected event semantics
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

**Add clarification after line 156**:

```markdown
**Discovery Event Scope**: Discovery events support restartability of the **filesystem discovery typestate processor** only. They track scanning, classification, and persistence of `FileView`/`DirView` records. They do NOT model events for schema, note, or template context processors (those are out of scope for this PRD).
```

**Impact**: ✅ Clarifies event semantics, aligns with typestate processor, preserves restartability goal.

---

### Issue 1.5: EventId Allocation Strategy Design (CRITICAL)

**Current PRD (lines 189-211)**: `EventStore` trait defines `append_event(...) -> Result<EventId>` but provides no allocation mechanism.

**Problem**: No specification for how `EventId` is generated (scan table? AtomicU64? sequence table?).

**Proposed Correction**:

**Add new subsection 6.4 after line 211**:

```markdown
#### 6.4: EventId Allocation Strategy

**Decision: Per-Context Sequence Tables with Transactional Increment**

Each context maintains its own monotonic sequence in a dedicated table. `EventId` allocation occurs within the same write transaction as event append to ensure atomicity and crash safety.

##### Sequence Table Schema

```rust
/// Per-context event sequence table (stores next available EventId).
///
/// Single-row table: key is context name, value is next u64.
pub const EVENT_SEQUENCES: TableDefinition<'static, &str, u64> =
    TableDefinition::new("event_sequences");
```

##### EventStore Trait (Revised)

```rust
/// Event storage behavior for context-specific event logs.
///
/// Implementations MUST ensure:
/// - EventId allocation is transactional (same txn as append)
/// - EventIds are monotonically increasing per context
/// - Concurrent appends serialize via redb MVCC
pub trait EventStore {
    type Event: Archive;

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

##### Reference Implementation Pattern

```rust
impl EventStore for SchemaRepository {
    type Event = SchemaEvent;

    fn append_event(
        &self,
        txn: &mut WriteTransaction,
        event: &Self::Event,
    ) -> Result<EventId, DbError> {
        // 1. Read current sequence (transactional)
        let sequences = txn.open_table(EVENT_SEQUENCES)?;
        let next_id = sequences
            .get("schema")?
            .map(|guard| guard.value())
            .unwrap_or(0);

        // 2. Serialize event
        let event_bytes = event.to_bytes()?;

        // 3. Write event with allocated ID
        let events = txn.open_table(SCHEMA_EVENTS.definition())?;
        events.insert(EventId(next_id), event_bytes.as_slice())?;

        // 4. Increment sequence (same transaction)
        sequences.insert("schema", next_id + 1)?;

        Ok(EventId(next_id))
    }

    fn compact_events(
        &self,
        txn: &mut WriteTransaction,
        completed_event_ids: &[EventId],
    ) -> Result<(), DbError> {
        let events = txn.open_table(SCHEMA_EVENTS.definition())?;
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

AtomicU64 increments are NOT transactional. If a transaction allocates ID=42 but crashes before commit, the sequence is advanced but event 42 never exists (gap in log). Transactional sequence tables ensure atomicity: either BOTH sequence increment AND event append succeed, or NEITHER does.

##### Scope Per Context

Each context has its own sequence key in `EVENT_SEQUENCES`:
- `"discovery"` → discovery event IDs
- `"schema"` → schema event IDs
- `"note"` → note event IDs
- `"template"` → template event IDs
- `"config"` → config event IDs

This prevents EventId collisions across contexts and allows independent compaction.
```

**Impact**: ✅ Defines complete allocation strategy, ensures transactional atomicity, aligns with redb constraints, crash-safe.

---

## CATEGORY 2: TECHNICAL SPECIFICATION CORRECTIONS

### Issue 2.1-2.4: Reference Existing Types (MODERATE)

**Current PRD**: Treats `FileFormat`, `FileName`, `DirName`, `DirScanInput` as new undefined types.

**Problem**: These types already exist in `fs/` module with well-defined semantics.

**Proposed Correction**:

**Add new subsection 3.5 after line 131**:

```markdown
### 3.5: Existing Filesystem Types (Reference)

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
```

**Impact**: ✅ Eliminates undefined type ambiguity, references existing implementations, clarifies batch size policy.

---

## CATEGORY 3: ORCHESTRATION CLARIFICATIONS

### Issue 3.1: Phase 3 State Rehydration Scope Fix (HIGH)

**Current PRD (lines 416-419)**:
```rust
// Phase 3: State Rehydration (database)
let db = Database::open(config.db_path())?;
let schema_events = schema_repo.load_all_events()?;
let pending_state = PendingSchemaState::from_events(&schema_events);
```

**Problem**: Phase 3 shows schema-specific rehydration, but schema runs in Phase 5. Should show discovery rehydration.

**Proposed Correction**:

Replace lines 416-419 with:

```rust
// Phase 3: Database & Discovery State Rehydration
let db = Database::open(config.db_path())?;
let discovery_repo = discovery::Repository::new(&db);
let discovery_events = discovery_repo.load_all_events()?;
let pending_discovery = PendingDiscoveryState::from_events(&discovery_events);
```

**Add new struct definition after line 255**:

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
}
```

**Update Section 6 title (line 144)**: Change "Pipeline Resilience & Restartability" to "Discovery Processor Restartability (Event Sourcing Infrastructure)"

**Add clarification after line 156**:

```markdown
**Scope Limitation**: This PRD defines event sourcing infrastructure (`EventId`, `EventTable`, `EventStore` trait) and applies it ONLY to the filesystem discovery processor. Schema, note, template, and config processor event modeling is OUT OF SCOPE and will be addressed in a separate PRD.
```

**Impact**: ✅ Fixes incorrect schema reference, aligns with discovery-scoped PRD, defines correct rehydration model.

---

### Issue 3.2: DiscoveryEngine::run Replacement (HIGH)

**Current PRD (line 424)**:
```rust
let discovery_result = DiscoveryEngine::run(&discovery_spec, scope, repository)?;
```

**Problem**: `DiscoveryEngine::run` implies stateless function, contradicts typestate processor design.

**Proposed Correction**:

Replace line 424 with:

```rust
// Phase 4: Filesystem Discovery (typestate processor)
let processor = FsDiscoveryProcessor::new(&discovery_spec, scope);
let discovery_result = if pending_discovery.has_pending_work() {
    // Resume from rehydrated state
    processor.resume(pending_discovery, &discovery_repo)?
} else {
    // Fresh discovery run
    processor.run(&discovery_repo)?
};
```

**Update lines 55-56**:

OLD:
```markdown
- **Shared Engine**: The discovery typestate processor lives in the discovery layer and acts as the shared base filesystem discovery engine for all contexts.
```

NEW:
```markdown
- **Centralized Processor**: The `FsDiscoveryProcessor` typestate processor lives in the discovery layer (incrementally refactored from `vault/`) and replaces context-specific stateless discovery functions.
```

**Impact**: ✅ Aligns with typestate design, clarifies processor API, supports resumption.

---

### Issue 3.3: Config Repository Boundary (MODERATE)

**Current PRD (line 414)**: `let config = ConfigBuilder::load(vault_root, repository)?;`

**Problem**: Which repository? Config has its own views (`GlobalConfigFileView`, `LocalConfigFileView`) that need persistence.

**Proposed Correction**:

Replace line 414 with:

```rust
// Phase 2: Config Hydration (FAIL-FAST, uses config repository)
let config_repo = config::Repository::new(&db);
let config = ConfigBuilder::new(vault_root)
    .discover(&config_repo)?   // Discover config files (Ascending Discovery)
    .load(&config_repo)?        // Parse and validate config
    .build()?;                  // Freeze config for discovery handoff
```

**Add to Section 9.1 after line 661**:

```rust
// Config (config module)
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

**Impact**: ✅ Clarifies config owns its own repository, shows persistence boundary, aligns with Section 8.3 views.

---

### Issue 3.4: Discovery Result Routing Model (HIGH)

**Current PRD (lines 427-431)**:
```rust
rayon::scope(|s| {
    s.spawn(|_| SchemaProcessor::process(discovery_result.schemas(), config, repository));
    s.spawn(|_| NoteProcessor::process(discovery_result.notes(), config, repository));
    s.spawn(|_| TemplateProcessor::process(discovery_result.templates(), config, repository));
});
```

**Problem**: `DiscoveryResult` (lines 81-84) has no `.schemas()`, `.notes()`, `.templates()` methods. Routing logic undefined.

**Proposed Correction**:

**Replace lines 427-431 with**:

```rust
// Phase 5: Context Processing (SEQUENTIAL until parallel execution model resolved)
//
// IMPORTANT: Parallel execution deferred pending redb write contention analysis.
// See Section 7.5 for parallel execution design alternatives.
let router = ContextRouter::new(&config);
let routed_files = router.route(&discovery_result)?;

SchemaProcessor::process(routed_files.schemas, &config, &schema_repo)?;
NoteProcessor::process(routed_files.notes, &config, &note_repo)?;
TemplateProcessor::process(routed_files.templates, &config, &template_repo)?;
```

**Add new subsection 7.5 after line 483**:

```markdown
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
            // Determine context from config-defined directory boundaries
            if self.is_schema_file(&file)? {
                routed.schemas.push(file.clone());
            } else if self.is_note_file(&file)? {
                routed.notes.push(file.clone());
            } else if self.is_template_file(&file)? {
                routed.templates.push(file.clone());
            }
            // Files not matching any context boundary are ignored (e.g., property bank files)
        }

        Ok(routed)
    }

    fn is_schema_file(&self, file: &DiscoveredFile) -> Result<bool, Error> {
        // Check if file.view.path is under config.schema.directory
        Ok(file.view.path().starts_with(self.config.schema().directory()))
    }

    // ... similar for notes, templates
}

/// Files partitioned by context.
#[derive(Default)]
pub struct RoutedFiles {
    pub schemas: Vec<DiscoveredFile>,
    pub notes: Vec<DiscoveredFile>,
    pub templates: Vec<DiscoveredFile>,
}
```

##### Routing Rules

1. **Schema files**: Under `config.schema.directory` (e.g., `schemas/`)
2. **Note files**: Under `config.note.directory` (e.g., `notes/`)
3. **Template files**: Under `config.template.directory` (e.g., `templates/`)
4. **Excluded files**: Property bank files filtered via config (e.g., `schemas/properties/`)
5. **Overlapping boundaries**: PRD does NOT specify behavior (implementation may error or use precedence)

##### Execution Model (DEFERRED DECISION)

**Current Status**: Sequential execution pending parallel execution analysis.

The PRD originally proposed parallel context processing (rayon) but this requires resolving redb write contention constraints. Section "Second Pass 1: Parallel Execution Model" below analyzes alternatives.

**Placeholder**: For now, contexts run sequentially. Future PRD or implementation may switch to parallel CPU work with serialized writes.
```

**Impact**: ✅ Defines routing model, preserves generic `DiscoveryResult`, defers parallel execution decision.

---

## CATEGORY 4: TERMINOLOGY UNIFICATION

### Issue 4.1-4.3: Consistent Terminology (MODERATE)

**Problems**:
- "Discovery engine" vs "discovery processor" vs "typestate processor"
- "Context processing" vs "Context Processing phase"
- "Vault module" vs "Discovery module" (renaming deferred but inconsistent)

**Proposed Correction**:

**Global find-and-replace rules** (apply after all other corrections):

1. **Discovery terminology**:
   - Replace: "discovery typestate processor" → `FsDiscoveryProcessor` (when referring to type)
   - Replace: "discovery engine" → "filesystem discovery processor" (when generic)
   - Replace: "shared filesystem discovery engine" → "centralized filesystem discovery processor"
   - Keep: "DiscoveryEngine" only in "Out of Scope" sections when referring to OLD implementation

2. **Context terminology**:
   - Keep: "context processor" (lowercase) when referring to concept
   - Keep: "Context Processing" (title case) only in Phase 5 pipeline phase names
   - Replace: "Parallel context processing" → "Sequential context processing (parallel execution deferred)"

3. **Module terminology**:
   - Replace: "Vault module" → "discovery module (vault/ directory)" on first mention in each section
   - Replace: "(vault module)" comment annotations → "(discovery module)" in code blocks
   - Add footnote after line 58: "**Module Location**: Discovery code currently lives in `lithos-core/src/vault/` and will be renamed to `lithos-core/src/discovery/` as part of this PRD's implementation. References to 'discovery module' refer to this target location."

**Impact**: ✅ Eliminates terminology confusion, aligns with implementation plan, preserves legacy references.

---

## CATEGORY 5: EVENT SOURCING PATTERN COMPLETENESS

### Issue 5.1: Transactional Event Compaction (MODERATE)

**Current PRD (lines 206-209)**:
```rust
fn compact_events(
    &self,
    completed_file_ids: &[FileId],
) -> Result<(), DbError>;
```

**Problem**: No transaction parameter, unclear when compaction is safe, no concurrent safety guarantees.

**Proposed Correction**:

**Already corrected in Issue 1.5 (EventStore trait revision)** — compaction now takes `&mut WriteTransaction`.

**Add new subsection after line 376**:

```markdown
#### Compaction Safety Rules

**When to Compact**:
- Discovery events: After Phase 5 completes (all contexts finished consuming `DiscoveryResult`)
- Context events: After context processor completes (e.g., all schemas persisted)
- Never compact mid-pipeline (rehydration requires full event log)

**Orchestrator Responsibility**:

```rust
// After Phase 5: All contexts complete
discovery_repo.write(|txn| {
    let completed_file_ids: Vec<FileId> = discovery_result.files
        .iter()
        .map(|f| f.id)
        .collect();
    discovery_repo.compact_events(txn, &completed_file_ids)?;
    Ok(())
})?;
```

**Concurrent Safety**:
- Compaction uses a write transaction (exclusive lock)
- No concurrent appends can occur during compaction
- redb MVCC ensures readers see pre-compaction snapshot until commit
```

**Impact**: ✅ Defines compaction timing, ensures transactional safety, clarifies orchestrator role.

---

### Issue 5.2: Cross-Context Coordination (LOW - DEFER)

**Current PRD**: Mentions "dependency-aware cleanup" (lines 359-376) but doesn't define coordination.

**Proposed Correction**:

**Add disclaimer after line 376**:

```markdown
**DEFERRED: Cross-Context Completion Tracking**

This PRD does NOT define a mechanism for tracking when all contexts have completed processing. The orchestrator (CLI entrypoint) currently handles this implicitly by running contexts sequentially and compacting discovery events after Phase 5.

Future enhancements (parallel execution, long-running pipelines, background workers) will require explicit completion tracking, likely via:
- Orchestrator-level event log (not context-specific)
- Completion marker table (Context → Status)
- Dependency graph traversal

This is OUT OF SCOPE for this PRD.
```

**Impact**: ✅ Defers unresolved design, clarifies current sequential assumption, preserves future options.

---

## SECOND PASS: CRITICAL DESIGN RE-EVALUATIONS

### Second Pass 1: Parallel Execution Model (CRITICAL)

**Current PRD (lines 427-431, 464-471)**: Proposes parallel context processing with rayon + MVCC.

**Problem**: redb write transactions cannot overlap (exclusive lock). Parallel CPU work requires sequential write coordination, adding complexity.

**Proposed Correction**:

**Add new Section 7.6 after routing model**:

```markdown
#### 7.6: Parallel vs Sequential Execution Analysis

**DECISION REQUIRED: Choose Execution Model**

The PRD originally proposed parallel context processing, but redb's write transaction model requires careful analysis.

##### Alternative 1: Fully Sequential (SIMPLEST)

```rust
SchemaProcessor::process(routed_files.schemas, &config, &schema_repo)?;
NoteProcessor::process(routed_files.notes, &config, &note_repo)?;
TemplateProcessor::process(routed_files.templates, &config, &template_repo)?;
```

**Pros**:
- Simplest orchestration (no coordination)
- No write contention
- Deterministic ordering
- Easy to reason about restartability

**Cons**:
- No CPU parallelism (slower on multi-core)
- Schemas block notes block templates (sequential bottleneck)

##### Alternative 2: Parallel CPU + Sequential Writes

```rust
// Phase 5a: Parallel CPU-bound work (parsing, validation)
let (parsed_schemas, parsed_notes, parsed_templates) = rayon::join(
    || SchemaParser::parse_batch(routed_files.schemas),
    || NoteParser::parse_batch(routed_files.notes),
    || TemplateParser::parse_batch(routed_files.templates),
);

// Phase 5b: Sequential writes (one context at a time)
schema_repo.write(|txn| {
    for schema in parsed_schemas {
        schema_repo.save_schema(txn, &schema)?;
        schema_repo.append_event(txn, &SchemaEvent::Completed { file_id: schema.file_id })?;
    }
    Ok(())
})?;

note_repo.write(|txn| { /* ... */ })?;
template_repo.write(|txn| { /* ... */ })?;
```

**Pros**:
- Parallel CPU work (faster parsing/validation)
- Batch writes per context (fewer transactions)
- Clear separation: compute vs I/O

**Cons**:
- Memory buffering required (hold parsed results)
- Loss of per-file atomic commits (batch-level atomicity)
- More complex restartability (can't resume mid-batch)

##### Alternative 3: Per-File MVCC with Small Transactions

```rust
rayon::scope(|s| {
    s.spawn(|_| {
        for schema_file in routed_files.schemas {
            let schema = parse_schema(schema_file)?;
            schema_repo.write(|txn| {
                schema_repo.save_schema(txn, &schema)?;
                schema_repo.append_event(txn, &SchemaEvent::Completed { file_id: schema.file_id })?;
                Ok(())
            })?; // Exclusive write lock (1-2ms), then release
        }
    });
    // ... parallel note/template processing
});
```

**Pros**:
- True parallelism (rayon thread pool)
- Per-file atomicity (granular restartability)
- redb MVCC serializes writes automatically

**Cons**:
- Write transactions serialize (not truly parallel writes)
- Contention if many small transactions (overhead)
- Requires redb MVCC performance validation

##### Alternative 4: Discovery-First Sequential Persistence

```rust
// Phase 4: Discovery persists all FileView/DirView (complete)
processor.run(&discovery_repo)?;

// Phase 5: Contexts read from DB, no writes (pure transformation)
let schemas = SchemaBuilder::from_views(&discovery_repo, routed_files.schemas)?;
let notes = NoteBuilder::from_views(&discovery_repo, routed_files.notes)?;
let templates = TemplateBuilder::from_views(&discovery_repo, routed_files.templates)?;

// Phase 6: Batch context writes (one txn per context)
schema_repo.write(|txn| {
    schema_repo.save_all(txn, &schemas)?;
    Ok(())
})?;
// ... notes, templates
```

**Pros**:
- Discovery completes before contexts start
- Contexts can parallelize reads from discovery tables
- Batch writes minimize transactions

**Cons**:
- Separates discovery persistence from context processing
- Requires buffering all context results in memory
- Loses typestate-driven embedded commits pattern

##### RECOMMENDATION: Start with Alternative 1 (Sequential)

**Rationale**:
- Simplest to implement and reason about
- Preserves per-file atomic commits (restartability)
- Avoids premature optimization (measure first)
- Can refactor to Alternative 2/3 after profiling shows CPU bottleneck

**Defer Parallel Execution**: This PRD will implement sequential context processing. A future PRD will analyze profiling data and choose a parallel execution model if warranted.
```

**Update line 426**: Change "Context Processing (PARALLEL, MVCC commits)" to "Context Processing (SEQUENTIAL)"

**Impact**: ✅ Forces explicit design decision, analyzes tradeoffs, recommends conservative approach, preserves future options.

---

### Second Pass 2: Config Boundary Change Behavior (MODERATE)

**Current PRD (lines 522-530)**: Mentions "config boundary changes" trigger full scan but doesn't distinguish add vs remove.

**Problem**: Adding exclusions (remove files from scope) vs removing exclusions (add files to scope) require different behaviors.

**Proposed Correction**:

**Add new subsection 8.3.1 after line 587**:

```markdown
#### 8.3.1: Config Boundary Change Detection & Behavior

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
```

**Impact**: ✅ Defines precise behavior for each config change type, avoids unnecessary full vault scans, clarifies delete-only vs scan-required cases.

---

### Second Pass 3: Missing vs Malformed Config Behavior (MODERATE)

**Current PRD (lines 472-483)**: "Config errors are fatal, non-recoverable" but doesn't distinguish missing vs malformed.

**Problem**: Missing config during Ascending Discovery has different implications than invalid TOML syntax.

**Proposed Correction**:

**Replace lines 472-483 with**:

```markdown
#### 7.4: Config Error Propagation & Missing Config Behavior

**Rule: Malformed Config is Fatal, Missing Config is Conditional**

##### Malformed Config (Fatal)

Invalid TOML/JSON/YAML syntax, missing required fields, or semantic validation errors ALWAYS halt the pipeline:

```rust
let config = ConfigBuilder::new(vault_root, &config_repo)
    .discover()?  // Find config files
    .load()?      // Parse + validate (FAIL-FAST here)
    .build()?;
```

**NO fallback to defaults. NO silent degradation.**

Errors:
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
```

**Impact**: ✅ Separates missing vs malformed cases, defines current vs future behavior, clarifies optional global config.

---

### Second Pass 4: Event Serialization Codec Alignment (MODERATE)

**Current PRD (lines 220, 783-787)**: Shows manual `rkyv::to_bytes::<_, 256>(event)` but codebase has `ArchivedEntity` trait in `db/codec.rs`.

**Problem**: Inconsistent with existing codec infrastructure, bypasses validation layer.

**Proposed Correction**:

**Replace line 220 with**:

```markdown
   **Serialization**: All event tables use **rkyv** via the `ArchivedEntity` trait (`db/codec.rs`). Events must derive `Archive + Serialize + Deserialize` with `Portable` archived form and `CheckBytes` validation. The `ArchivedEntity` trait handles alignment, validation, and serialization automatically.
```

**Update EventStore trait append_event implementation (line 199)**:

```rust
fn append_event(
    &self,
    txn: &mut WriteTransaction,
    event: &Self::Event,
) -> Result<EventId, DbError>
where
    Self::Event: ArchivedEntity;  // Bound ensures codec compatibility
```

**Update reference implementation in Issue 1.5 correction**:

```rust
impl EventStore for SchemaRepository {
    type Event = SchemaEvent;

    fn append_event(
        &self,
        txn: &mut WriteTransaction,
        event: &Self::Event,
    ) -> Result<EventId, DbError> {
        // 1. Read current sequence (transactional)
        let sequences = txn.open_table(EVENT_SEQUENCES)?;
        let next_id = sequences
            .get("schema")?
            .map(|guard| guard.value())
            .unwrap_or(0);

        // 2. Serialize event via ArchivedEntity trait
        let event_bytes = event.to_bytes()?;  // Uses db/codec.rs automatically

        // 3. Write event with allocated ID
        let events = txn.open_table(SCHEMA_EVENTS.definition())?;
        events.insert(EventId(next_id), event_bytes.as_slice())?;

        // 4. Increment sequence (same transaction)
        sequences.insert("schema", next_id + 1)?;

        Ok(EventId(next_id))
    }
}
```

**Replace lines 783-787 with**:

```markdown
**Serialization Strategy**:

All event tables use **rkyv** with strict validation via `db/codec.rs`:

```rust
#[derive(Archive, Deserialize, Serialize)]
#[rkyv(derive(CheckBytes))]  // Required for validation
pub enum SchemaEvent {
    Discovered { file_id: FileId, path: PathKey },
    // ...
}
```

Events are serialized via `ArchivedEntity::to_bytes()` and deserialized via `ArchivedEntity::from_bytes()`, ensuring:
- 16-byte alignment (`AlignedVec`)
- Bytecheck validation (prevents corrupt reads)
- Zero-copy archived access (`ArchivedEntity::with_archived()`)

**Reference**: `db/codec.rs` (lines 1-100) for complete trait documentation.
```

**Impact**: ✅ Aligns with existing codec infrastructure, ensures validation, simplifies implementation.

---

## APPLICATION PLAN

**After your review and approval**:

1. Create backup: `cp PRD.md PRD.md.backup-$(date +%Y%m%d-%H%M%S)`
2. Apply all corrections systematically (Category 1 → Second Pass 4)
3. Run global find-and-replace for terminology (Issue 4.1-4.3)
4. Validate PRD structure (headings, code blocks, references)
5. Commit with detailed message listing all 18+4 issues resolved

**Estimated application time**: 45-60 minutes (careful surgical edits)

---

## UNRESOLVED DESIGN QUESTIONS (Require User Decision)

1. **Parallel Execution**: Accept Sequential (Alternative 1) or choose Alternative 2/3/4?
2. **Config Discovery**: Should missing local config prompt for creation (future) or always error (current)?
3. **Overlapping Context Boundaries**: If schema directory overlaps note directory, error or precedence?
4. **Multiple Config Formats**: If both `lithos.toml` and `lithos.json` exist at same location, error or precedence?

Please provide guidance on these 4 decisions to finalize all corrections.

---

## SUMMARY

**Total Corrections**: 22 (18 original + 4 second-pass)
- **Critical**: 5 (must-fix before implementation)
- **Moderate**: 11 (clarity/correctness improvements)
- **Minor**: 6 (polish/reference corrections)

**Approval Request**: Please review each correction and indicate:
- ✅ Approve as-is
- 🔄 Modify (provide specific changes)
- ❌ Reject (provide rationale)

I am ready to apply all approved corrections immediately.
