# Findings & Decisions

## Requirements
<!-- User's refactoring objectives -->
1. Refactor `discovery.rs` to have optimal design using available codebase capabilities efficiently
2. Eliminate duplicate capabilities throughout codebase
3. Remove unnecessary components from `discovery.rs`
4. Route all discovery logic for schema module through `DiscoveryEngine`
5. Eliminate discovery phases from `property_bank_processor.rs`
6. Eliminate discovery routing from `schema_processor.rs`
7. Eliminate discovery logic from `builder.rs`

## Research Findings

### Complete Dependency Map & Call Chains

**Builder::load_all() Call Chain (Current):**
```
1. Builder.discover_files() [Lines 158-242]
   ├─ Input: self.config, self.source
   ├─ Uses: DirScanner (DUPLICATE of DiscoveryEngine)
   ├─ Returns: FilesContext { files: Vec<RelativePath> }
   ├─ Callback: on_bank_found(PropertyBankContext)
   └─ Side effect: Sets bank_branch = BankContextBranch::Present/Missing

2. Builder.discover_graph() [Lines 244-258]
   ├─ Input: self.repository
   ├─ Calls: repository.get_topological_graph() (DUPLICATE of DiscoveryEngine)
   └─ Returns: GraphContextBranch { Present { graph } | Missing }

3. Builder.load_property_bank() [Lines 131-156]
   ├─ Input: PropertyBankContext, self.source, self.repository
   ├─ Creates: PropertyBankProcessor::<Discovery, Unknown>
   ├─ Calls: pipeline.discover() [property_bank_processor.rs Lines 217-249]
   │   ├─ Queries: repository.get_raw_property_bank_view(path) (DUPLICATE)
   │   └─ Reads: source.info(config_path) (filesystem)
   └─ Returns: PropertyBank

4. SchemaProcessor::<Discovery, Review>::discover() [Lines 752-794]
   ├─ Input: &FilesContext, &InheritanceGraph, &R, &FsReader
   ├─ Calls: Self::fetch_view_maps() [Lines 799-825]
   │   ├─ Queries: repository.find_raw_schema_views_by_paths() (DUPLICATE)
   │   └─ Queries: repository.find_schema_ids_by_paths() (DUPLICATE)
   ├─ Calls: Self::classify_file_state() [Lines 827-869]
   └─ Returns: DiscoveryBranch

5. SchemaProcessor::<Discovery, NeverSeen>::discover() [Lines 723-748]
   ├─ Input: &FilesContext, &FsReader
   ├─ Creates: NewBatch<InitialScan> with SchemaId::new()
   └─ Returns: DiscoveryBranch::AllMissing
```

**What DiscoveryEngine Already Provides:**
```
DiscoveryEngine::run(spec, repo, vault_root) → DiscoveryOutcome
├─ files: HashMap<RelativePath, DiscoveredFile>
│   ├─ DiscoveredFile.kind: SchemaFileKind (PropertyBank | Schema(SchemaId))
│   ├─ DiscoveredFile.view: Option<DiscoveredView>
│   │   ├─ DiscoveredView::Schema(RawSchemaView)
│   │   └─ DiscoveredView::PropertyBank(RawPropertyBankView)
│   └─ DiscoveredFile.info: FileInfo (created_at, modified_at, size)
├─ graph: Option<InheritanceGraph<()>>
└─ deleted_schemas: Vec<SchemaId>

Methods:
├─ schema_files() → Iterator over schema files only
├─ property_bank() → Option<(&RelativePath, &DiscoveredFile)>
├─ has_schemas() → bool
└─ is_cold_start() → bool
```

**Mapping Current → Target:**
```
CURRENT                                       → TARGET
─────────────────────────────────────────────────  ─────────────────────────────
1. Builder.discover_files()                    → DiscoveryEngine::run()
   ├─ Returns FilesContext { files }            → Use outcome.schema_files()
   └─ Callback with PropertyBankContext      → Use outcome.property_bank()
                                                Returns DiscoveredFile with view

2. Builder.discover_graph()                    → Already in outcome.graph
   └─ Returns graph                           → Remove this method entirely

3. PropertyBankProcessor.discover()           → Remove discovery stage
   ├─ Queries repo for view                   → Receive DiscoveredFile from outcome
   └─ Reads file info                         → DiscoveredFile.info already has this
                                                DiscoveredFile.view has cached view

4. SchemaProcessor::<Discovery, Review>::discover()
   ├─ Input: &FilesContext, &Graph, &Repo    → Input: &DiscoveryOutcome
   ├─ Queries views_by_path                   → outcome.files already has views
   ├─ Queries ids_by_path                     → Extract from DiscoveredFile.kind
   └─ Returns DiscoveryBranch                  → Returns DiscoveryBranch (unchanged)

5. SchemaProcessor::<Discovery, NeverSeen>::discover()
   ├─ Input: &FilesContext                     → Input: &DiscoveryOutcome
   └─ Creates NewBatch with new IDs           → Extract from outcome.files
                                                (SchemaFileKind::Schema(id))
```

### Current State Analysis

**discovery.rs (DiscoveryEngine):**
- Already provides unified atomic discovery: `DiscoveryEngine::run(spec, repo, vault_root)`
- Returns `DiscoveryOutcome` with: `files`, `graph`, `deleted_schemas`
- Uses `DirScanner` to scan filesystem once
- Uses `Repository::with_batch_schema_reader()` for atomic DB queries
- Separates property bank from schemas via `SchemaFileKind` enum
- Provides query methods: `schema_files()`, `property_bank()`, `has_schemas()`, `is_cold_start()`

**builder.rs (Builder):**
- Has `discover_files()` method (lines 158-242) - DUPLICATE filesystem scanning
- Uses `DirScanner` directly with same pattern as DiscoveryEngine
- Has `discover_graph()` method (lines 244-258) - DUPLICATE DB query
- Manually separates property bank from schema files
- Returns custom `FilesContext`, `PropertyBankContext`, `GraphContextBranch` types

**property_bank_processor.rs (PropertyBankProcessor):**
- Has `Discovery` stage with `discover()` method (lines 217-249)
- Queries repository for cached view: `repository.get_raw_property_bank_view(path)`
- Reads file info from filesystem via `source.info(config_path)`
- Returns `ComparisonBranch` to continue pipeline

**schema_processor.rs (SchemaProcessor):**
- Has `Discovery` stage with two variants: `NeverSeen` and `Review`
- `NeverSeen::discover()` (lines 723-748) - creates new IDs for files
- `Review::discover()` (lines 752-794) - queries DB for views and IDs
- Uses `fetch_view_maps()` to batch query repository
- Uses `classify_file_state()` to separate missing/found/deleted
- Returns `DiscoveryBranch` to route pipeline

### Duplication Analysis

**Filesystem Scanning:**
1. `discovery.rs` - Lines 280-324: DirScanner with pattern + extensions
2. `builder.rs` - Lines 184-200: DirScanner with pattern + extensions (EXACT DUPLICATE)
3. Both use: `DirScanInput::new().with_pattern(&pattern).with_extensions(&SCHEMA_EXTENSIONS)`

**Database Queries:**
1. `discovery.rs` - Lines 215-232: Batch reads (graph, views, IDs) in single transaction
2. `builder.rs` - Lines 247-250: `get_topological_graph()` separately
3. `property_bank_processor.rs` - Line 229: `get_raw_property_bank_view()` separately
4. `schema_processor.rs` - Lines 806-824: `find_raw_schema_views_by_paths()` and `find_schema_ids_by_paths()` separately

**DiscoveryEngine already consolidates all of this!**

### Available Capabilities in DiscoveryEngine

**DiscoveryOutcome Methods:**
- `schema_files()` - Iterator over schema files
- `property_bank()` - Option with property bank discovered file
- `has_schemas()` - Boolean check
- `is_cold_start()` - Boolean check for cached views
- Direct access to: `files`, `graph`, `deleted_schemas`

**DiscoveredFile:**
- `kind: SchemaFileKind` - PropertyBank or Schema(SchemaId)
- `view: Option<DiscoveredView>` - Cached view from DB
- `info: FileInfo` - Filesystem metadata (created, modified, size)
- `is_timestamp_match()` - Compare cached vs current timestamps
- `is_new()` - Check if no cached view exists

**What's Missing:**
- Nothing! DiscoveryEngine already provides everything needed
- Just need to route calls through it instead of duplicating discovery

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Single DiscoveryEngine::run() call in Builder | Atomic discovery eliminates race conditions; single DB transaction is faster |
| Remove discover_files() from Builder | DirScanner already used in DiscoveryEngine; duplicate code |
| Remove discover_graph() from Builder | Graph already loaded in DiscoveryEngine batch read |
| PropertyBankProcessor receives DiscoveredFile | Processor focuses on comparison/parsing; doesn't need I/O |
| SchemaProcessor receives DiscoveryOutcome | Processor focuses on transformation; discovery is infrastructure |
| Keep DiscoveredFile as data carrier | Clean separation: discovery returns data, processors transform it |
| Preserve closure-based with_archived() | Zero-copy pattern is performance-critical |

## Phase 2: Unified API Design (REVISED)

### Design Review & Concerns

**Critical Issues Identified:**
1. **Naming**: `DiscoveryOutcome` is vague; should be `DiscoveryResult` or split into file/graph concerns
2. **Type overlap**: `PropertyBankContext`, `FileDiscovery`, `DiscoveredFile` have unclear boundaries and duplicated data
3. **New file problem**: DiscoveryEngine assigns `SchemaId::new()` too early - IDs should be assigned during processing, not discovery
4. **Data loss**: Converting `FileEntry` → `DiscoveredFile` drops the path from the struct (only in HashMap key)
5. **Unnecessary indirection**: PropertyBankProcessor Discovery stage is redundant if we route directly to Comparison
6. **Not adversarial enough**: Design uses existing structures instead of optimizing from first principles

### Revised Design Principles

**Core Insight:** Discovery should separate **finding files** from **assigning identities**:
- **Discovery phase**: Find what exists on disk + what's in DB
- **Processing phase**: Assign IDs to new files, classify changes

**Data Flow:**
```
Filesystem Scan → FileEntry (path, filename, info)
DB Query → Cached views by path
Combine → Discovered items (file + optional cached state)
```

**Key Distinction:**
- **New files**: Have `FileEntry` but no `SchemaId` yet (ID assigned during processing)
- **Existing files**: Have `FileEntry` + cached view + `SchemaId` from DB
- **Deleted files**: Have `SchemaId` in DB but no `FileEntry`

## Phase 2: Unified API Design

### Revised Type Design

#### 1. Eliminate Redundant Types

**REMOVE (overlapping concerns):**
- ✂️ `FileDiscovery` - Just a HashMap wrapper, no value added
- ✂️ `PropertyBankContext` - Only holds a path; use tuple or inline
- ✂️ `DiscoveredFile` - Confusing name; replace with clearer types

**KEEP (clear purpose):**
- ✅ `FileEntry` - From DirScanner; has path + filename + info
- ✅ `SchemaFileKind` - Distinguishes PropertyBank vs Schema
- ✅ `DiscoveredView` - Polymorphic cached view from DB

#### 2. New Core Types

**`DiscoveryResult`** (renamed from `DiscoveryOutcome`):
```rust
/// Result of atomic discovery combining filesystem scan and DB state.
pub(crate) struct DiscoveryResult {
    /// Discovered schema files (path → file entry + optional cached state).
    pub(crate) schemas: HashMap<RelativePath, SchemaDiscovery>,

    /// Discovered property bank file (if present).
    pub(crate) property_bank: Option<PropertyBankDiscovery>,

    /// Inheritance graph from DB (if exists).
    pub(crate) graph: Option<InheritanceGraph<()>>,

    /// Schema IDs that exist in DB but not on filesystem.
    pub(crate) deleted_ids: Vec<SchemaId>,
}
```

**`SchemaDiscovery`** (replaces `DiscoveredFile` for schemas):
```rust
/// Discovery data for a single schema file.
pub(crate) struct SchemaDiscovery {
    /// File entry from filesystem scan (always present for discovered files).
    pub(crate) entry: FileEntry,  // Has path, filename, info

    /// Cached state from database (if file was previously ingested).
    pub(crate) cached: Option<SchemaCachedState>,
}

/// Cached state for an existing schema.
pub(crate) struct SchemaCachedState {
    /// Schema ID from previous ingestion.
    pub(crate) id: SchemaId,

    /// Cached view for staleness detection.
    pub(crate) view: RawSchemaView,
}
```

**`PropertyBankDiscovery`** (replaces `DiscoveredFile` for property bank):
```rust
/// Discovery data for the property bank file.
pub(crate) struct PropertyBankDiscovery {
    /// File entry from filesystem scan.
    pub(crate) entry: FileEntry,  // Has path, filename, info

    /// Cached view from database (if previously ingested).
    pub(crate) view: Option<RawPropertyBankView>,
}
```

#### 3. Why This Design?

| Design Choice | Rationale |
|---------------|-----------|
| Split `DiscoveredFile` into `SchemaDiscovery` + `PropertyBankDiscovery` | Different concerns: schemas need ID assignment logic, property bank doesn't |
| Keep `FileEntry` intact | Already has all filesystem data; no need to decompose and lose information |
| `SchemaId` only in `SchemaCachedState` | New files don't have IDs yet; ID assignment is processing concern, not discovery |
| Separate `cached` field | Makes "new vs existing" explicit; easier to route to correct pipeline branch |
| HashMap by `RelativePath` for schemas | O(1) lookup; path is natural key before ID assignment |

### Revised Discovery Flow

**Target Architecture:**
```
Builder::load_all()
  └─ DiscoveryEngine::run(spec, repo, vault_root) → DiscoveryResult
       ├─ schemas: HashMap<RelativePath, SchemaDiscovery>
       │    ├─ entry: FileEntry (path, filename, info)
       │    └─ cached: Option<SchemaCachedState> (id, view)
       ├─ property_bank: Option<PropertyBankDiscovery>
       │    ├─ entry: FileEntry
       │    └─ view: Option<RawPropertyBankView>
       ├─ graph: Option<InheritanceGraph<()>>
       └─ deleted_ids: Vec<SchemaId>
```

### Revised Component Integration

#### 1. DiscoveryResult - Clean Query API

**No redundant helper methods** - direct field access is clearest:
```rust
impl DiscoveryResult {
    /// Returns true if any schema files were discovered.
    #[inline]
    pub(crate) fn has_schemas(&self) -> bool {
        !self.schemas.is_empty()
    }

    /// Returns true if this is a cold-start (no cached data).
    #[inline]
    pub(crate) fn is_cold_start(&self) -> bool {
        self.graph.is_none() &&
        self.schemas.values().all(|s| s.cached.is_none()) &&
        self.property_bank.as_ref().map_or(true, |pb| pb.view.is_none())
    }
}
```

**That's it!** Direct field access for everything else:
- `result.schemas` - iterate or lookup by path
- `result.property_bank` - `Option<PropertyBankDiscovery>`
- `result.graph` - `Option<InheritanceGraph<()>>`
- `result.deleted_ids` - `Vec<SchemaId>`

#### 1. DiscoveryOutcome - New Helper Methods

**Current methods:**
```rust
pub(crate) fn schema_files(&self) -> impl Iterator<Item = (&RelativePath, &DiscoveredFile)>
pub(crate) fn property_bank(&self) -> Option<(&RelativePath, &DiscoveredFile)>
pub(crate) fn has_schemas(&self) -> bool
pub(crate) fn is_cold_start(&self) -> bool
```

**New methods needed:**
```rust
impl DiscoveryOutcome {
    /// Returns the property bank DiscoveredFile, if found.
    pub(crate) fn property_bank_file(&self) -> Option<&DiscoveredFile> {
        self.property_bank().map(|(_, file)| file)
    }

    /// Returns schema file entries with their DiscoveredFile.
    pub(crate) fn schema_file_entries(&self) -> impl Iterator<Item = (&RelativePath, &DiscoveredFile)> {
        self.schema_files()
    }

    /// Returns discovered schema IDs from the filesystem scan.
    pub(crate) fn discovered_schema_ids(&self) -> HashMap<RelativePath, SchemaId> {
        self.schema_files()
            .filter_map(|(path, file)| {
                if let SchemaFileKind::Schema(id) = file.kind {
                    Some((path.clone(), id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns whether a property bank was discovered.
    pub(crate) fn has_property_bank(&self) -> bool {
        self.property_bank().is_some()
    }

    /// Extracts property bank path and DiscoveredFile if present.
    pub(crate) fn take_property_bank(&mut self) -> Option<(RelativePath, DiscoveredFile)> {
        // Return owned data for builder to pass to processor
        // Need to remove from files map or clone
        // Approach: Return references + let builder decide
        todo!("Design decision needed")
    }
}
```

**Design Decision:** DiscoveryOutcome should provide **references**, not owned data. Builder can clone if needed (DiscoveredFile is small).

#### 2. Builder - Simplified load_all()

**Target (clean & direct):**
```rust
pub fn load_all(&mut self) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
    // Single atomic discovery
    let result = DiscoveryEngine::run(
        &self.config.paths().schema,
        &self.repository,
        self.source.root(),
    )?;

    // Load property bank if discovered (route directly to Comparison stage)
    let property_bank = if let Some(ref pb_discovery) = result.property_bank {
        Some(self.load_property_bank_direct(pb_discovery)?)
    } else {
        self.property_bank_delta = None;
        None
    };

    // Early return if no schemas
    if !result.has_schemas() {
        return Ok(Vec::new());
    }

    let property_bank = property_bank.unwrap_or_else(PropertyBank::new);

    // Route to schema processor (no separate discover() call needed)
    let branch = SchemaProcessor::from_discovery_result(
        &result,
        &self.source,
    )?;

    // Rest of pipeline unchanged
    match branch {
        DiscoveryBranch::AllMissing(missing) => {
            let parsed = missing.parse(&self.source)?;
            parsed.build_new_graph()?.construct_new_schemas(&self.repository, &property_bank)
        }
        DiscoveryBranch::HasPresent(present) => {
            let compared = present.compare(&self.source, self.property_bank_delta.as_ref())?;
            let parsed = compared.parse(&self.source)?;
            let graphed = parsed.build_graph()?;
            let analyzed = graphed.analyze_properties(&self.source, &property_bank, self.property_bank_delta.as_ref())?;
            let refreshed = analyzed.refresh_metadata(&self.repository)?;
            let constructed = refreshed.construct_schemas(&self.repository, &property_bank)?;
            constructed.complete(&self.repository)?.into_schemas()
        }
    }
}
```

**Key simplifications:**
1. No more `BankContextBranch` enum - just use `Option<PropertyBankDiscovery>`
2. No more callback pattern - direct field access
3. No more `discover_files()` or `discover_graph()` methods
4. PropertyBank routes directly to processing (no Discovery stage)

#### 3. PropertyBankProcessor - Direct to Comparison

**✂️ REMOVE Discovery Stage Entirely**

PropertyBankProcessor should start at Comparison stage. Discovery data is already available.

**Builder integration:**
```rust
impl Builder {
    fn load_property_bank_direct(
        &mut self,
        discovery: &PropertyBankDiscovery,
    ) -> Result<PropertyBank, SchemaLoaderError> {
        let path = RelativePath::try_from(discovery.entry.path.clone())?;

        // Route directly to Comparison based on cached view presence
        let branch = if let Some(ref view) = discovery.view {
            // Has cached view - check timestamps
            PropertyBankProcessor::from_comparison_present(
                discovery.entry.info,
                view.clone(),
            )
        } else {
            // No cached view - parse and create
            PropertyBankProcessor::from_comparison_missing(
                discovery.entry.info,
            )
        };

        // Continue through pipeline
        let (completed, delta) = match branch {
            ComparisonBranch::Missing(p) => {
                self.handle_missing(p, &path, discovery.entry.path.as_path())?
            }
            ComparisonBranch::Present(p) => {
                self.handle_present(p, &path, discovery.entry.path.as_path())?
            }
        };

        self.property_bank_delta = delta;
        Ok(completed)
    }
}
```

**New PropertyBankProcessor constructors:**
```rust
impl PropertyBankProcessor<Comparison, Present> {
    /// Create processor from discovery data (has cached view).
    pub(crate) fn from_discovery_present(
        info: FileInfo,
        view: RawPropertyBankView,
    ) -> Self {
        Self {
            status: Present { info, view },
            _stage: PhantomData,
        }
    }
}

impl PropertyBankProcessor<Comparison, Missing> {
    /// Create processor from discovery data (no cached view).
    pub(crate) fn from_discovery_missing(info: FileInfo) -> Self {
        Self {
            status: Missing { info },
            _stage: PhantomData,
        }
    }
}
```

**Benefits:**
1. No Discovery stage in PropertyBankProcessor
2. No I/O in processor construction
3. Clear entry points based on discovery state
4. FileEntry data preserved (not decomposed unnecessarily)

#### 4. SchemaProcessor - Direct from DiscoveryResult

**Replace separate discover() methods with unified constructor:**

```rust
impl SchemaProcessor {
    /// Create processor from discovery result (replaces discover() methods).
    pub(crate) fn from_discovery_result(
        result: &DiscoveryResult,
        source: &FsReader,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        // Classify schemas by cached state
        let mut found = HashMap::new();
        let mut missing = NewBatch::new();

        for (path, schema_disc) in &result.schemas {
            match &schema_disc.cached {
                Some(cached) => {
                    // Existing schema - has ID and view
                    found.insert(cached.id, FoundPayload {
                        path: path.clone(),
                        info: schema_disc.entry.info,
                        view: cached.view.clone(),
                    });
                }
                None => {
                    // New schema - assign fresh ID during processing
                    let id = SchemaId::new();
                    missing.insert(id, InitialScan {
                        path: path.clone(),
                        info: schema_disc.entry.info,
                    });
                }
            }
        }

        // Route to appropriate branch
        if found.is_empty() {
            // All schemas are new
            Ok(DiscoveryBranch::AllMissing(Self::transition(
                FileParsed,
                AllMissing { new_schemas: missing },
            )))
        } else {
            // Some schemas exist - use graph and classify
            let graph = result.graph.as_ref()
                .ok_or(SchemaLoaderError::Ingestion(/* ... */))?;

            let present_graph = Self::build_present_graph(
                graph,
                &found,
                &result.deleted_ids,
            );

            Ok(DiscoveryBranch::HasPresent(Self::transition(
                Comparison,
                Present {
                    graph: present_graph,
                    new_schemas: missing,
                    deleted_ids: result.deleted_ids.clone(),
                },
            )))
        }
    }
}
```

**Key improvements:**
1. ✂️ Remove `Discovery` stage entirely from SchemaProcessor
2. ✂️ Remove `fetch_view_maps()` - data already in `result.schemas`
3. ✂️ Remove `classify_file_state()` - done during iteration
4. ✅ Direct construction from discovery data
5. ✅ Clear routing based on `cached` field presence
6. ✅ ID assignment happens in processing, not discovery

**Target - Accept DiscoveryOutcome directly:**
```rust
impl SchemaProcessor<Discovery, Review> {
    pub(crate) fn discover_with_outcome(
        outcome: &DiscoveryOutcome,
        repository: &R,  // Still needed for some operations?
        source: &FsReader,
    ) -> Result<DiscoveryBranch, SchemaLoaderError> {
        // Use outcome.files directly (already has views and IDs)
        // Use outcome.graph directly
        // Use outcome.deleted_schemas directly
        // NO MORE REPOSITORY QUERIES!

        let graph = outcome.graph.as_ref().unwrap();
        let deleted_ids = &outcome.deleted_schemas;

        // Build Present graph from outcome.files
        let found: HashMap<SchemaId, FoundPayload> = outcome
            .schema_files()
            .filter_map(|(path, file)| {
                if let Some(DiscoveredView::Schema(ref view)) = file.view {
                    let id = if let SchemaFileKind::Schema(id) = file.kind {
                        id
                    } else {
                        return None;
                    };
                    Some((id, FoundPayload {
                        path: path.clone(),
                        info: file.info,
                        view: view.clone(),
                    }))
                } else {
                    None
                }
            })
            .collect();

        // Build missing from files without views
        let missing: NewBatch<InitialScan> = outcome
            .schema_files()
            .filter(|(_, file)| file.view.is_none())
            .map(|(path, file)| {
                let id = if let SchemaFileKind::Schema(id) = file.kind {
                    id
                } else {
                    SchemaId::new() // Fallback
                };
                (id, InitialScan {
                    path: path.clone(),
                    info: file.info,
                })
            })
            .collect();

        // Rest of logic uses found/missing/deleted_ids directly
        // ...
    }
}
```

### Revised Data Flow

```
DiscoveryEngine::run() → DiscoveryResult
  ├─ schemas: HashMap<RelativePath, SchemaDiscovery>
  │    ├─ entry: FileEntry (path, filename, info)  ← Preserves all FileEntry data
  │    └─ cached: Option<SchemaCachedState>
  │         ├─ id: SchemaId
  │         └─ view: RawSchemaView
  ├─ property_bank: Option<PropertyBankDiscovery>
  │    ├─ entry: FileEntry (path, filename, info)
  │    └─ view: Option<RawPropertyBankView>
  ├─ graph: Option<InheritanceGraph<()>>
  └─ deleted_ids: Vec<SchemaId>

Builder::load_all()
  ├─ DiscoveryEngine::run() [SINGLE CALL]
  ├─ load_property_bank_direct() [No Discovery stage]
  │    └─ PropertyBankProcessor starts at Comparison stage
  └─ SchemaProcessor::from_discovery_result() [No discover() call]

Eliminated Types:
  ✂️ FileDiscovery (just HashMap wrapper)
  ✂️ PropertyBankContext (just holds path)
  ✂️ DiscoveredFile (split into specific types)
  ✂️ FilesContext (just Vec<RelativePath>)
  ✂️ BankContextBranch (use Option directly)
  ✂️ GraphContextBranch (use Option directly)

Eliminated Methods:
  ✂️ Builder.discover_files()
  ✂️ Builder.discover_graph()
  ✂️ PropertyBankProcessor<Discovery, Unknown>::discover()
  ✂️ SchemaProcessor<Discovery, *>::discover()
  ✂️ SchemaProcessor::fetch_view_maps()
  ✂️ SchemaProcessor::classify_file_state()
```

### FileEntry Optimization

**Question:** Should FileEntry have a method to convert to HashMap?

**Answer:** No need - DirScanner already returns `Vec<FileEntry>`. The conversion happens once in DiscoveryEngine:

```rust
impl DiscoveryEngine {
    fn scan_filesystem(...) -> Result<Vec<FileEntry>, Error> {
        scanner.entries(input)
    }

    fn run(...) -> Result<DiscoveryResult, Error> {
        let entries = Self::scan_filesystem(spec, vault_root)?;

        // Convert to HashMap once (O(n))
        let mut schemas = HashMap::new();
        let mut property_bank = None;

        for entry in entries {
            let path = RelativePath::try_from(entry.path.clone())?;

            if path == spec.property_bank() {
                property_bank = Some(PropertyBankDiscovery {
                    entry,
                    view: /* query DB */,
                });
            } else {
                schemas.insert(path, SchemaDiscovery {
                    entry,
                    cached: /* query DB */,
                });
            }
        }

        Ok(DiscoveryResult { schemas, property_bank, /* ... */ })
    }
}
```

**FileEntry stays intact** - no decomposition, no data loss.

### Revised Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Naming** | `DiscoveryResult` (not Outcome) | Result indicates output; clearer intent |
| **Type structure** | Split into `SchemaDiscovery` + `PropertyBankDiscovery` | Different concerns; schemas need ID assignment logic |
| **FileEntry preservation** | Keep intact in discovery types | Already has all data; no need to decompose |
| **SchemaId assignment** | Only in `SchemaCachedState` | New files get IDs during processing, not discovery |
| **PropertyBank routing** | Direct to Comparison stage | Discovery stage is redundant when data is already discovered |
| **SchemaProcessor entry** | `from_discovery_result()` constructor | Replace discover() with direct construction |
| **Intermediate types** | Eliminate wrappers | `FilesContext`, `PropertyBankContext`, etc. add no value |
| **HashMap conversion** | Single conversion in DiscoveryEngine | O(n) once instead of multiple passes |
| **Branch routing** | Based on `cached` field presence | Makes new vs existing explicit |

### Critical Improvements Over Initial Design

1. **No premature ID assignment**: New schemas don't get `SchemaId::new()` in discovery
2. **No data loss**: FileEntry preserved intact (path, filename, info)
3. **No redundant types**: Eliminated 6 wrapper types that added no value
4. **Clear ownership**: `cached` field separates new from existing explicitly
5. **Direct routing**: PropertyBank goes straight to Comparison
6. **Minimal API surface**: Only 2 query methods on DiscoveryResult

## Method Decomposition Opportunities in discovery.rs

### Current DiscoveryEngine::run() Analysis

**Current structure (93 lines, lines 193-270):**
```rust
pub(crate) fn run<R>(spec, repo, vault_root) -> Result<DiscoveryOutcome> {
    // Step 1: Scan filesystem (1 call)
    let mut discovered = Self::scan_filesystem(spec, vault_root)?;

    // Step 2: Extract property bank (inline, 3 lines)
    let property_bank_path = spec.property_bank();
    let property_bank_entry = discovered.extract_property_bank(property_bank_path);

    // Step 3: Extract schema paths (inline, 2 lines)
    let schema_paths: Vec<_> = discovered.iter().map(...).collect();

    // Step 4: Batch DB queries (inline closure, 17 lines)
    let (graph, bank_view, mut views_by_path, mut ids_by_path) = repo
        .with_batch_schema_reader(|batch_reader| {
            // ... 13 lines of query logic
        })?;

    // Step 5: Build result (inline, 26 lines)
    let mut files = HashMap::new();
    // ... property bank construction (8 lines)
    // ... schema files construction (12 lines)

    // Step 6: Detect deletions (1 call)
    let deleted_schemas = Self::detect_deleted_schemas(...);

    Ok(DiscoveryOutcome { ... })
}
```

### Decomposition Strategy

**Goals:**
1. Extract clear, single-responsibility methods
2. Maintain performance (avoid unnecessary allocations/clones)
3. Keep hot path inline (batch DB query)
4. Improve testability (smaller units)

**Proposed decomposition:**

```rust
impl DiscoveryEngine {
    /// Main entry point - orchestrates discovery pipeline.
    pub(crate) fn run<R>(
        spec: &SchemaConfigSpec,
        repo: &R,
        vault_root: &std::path::Path,
    ) -> Result<DiscoveryResult, SchemaLoaderError> {
        // Step 1: Scan filesystem
        let entries = Self::scan_filesystem(spec, vault_root)?;

        // Step 2: Separate property bank from schemas (O(n) single pass)
        let (property_bank_entry, schema_entries) =
            Self::separate_property_bank(entries, spec.property_bank());

        // Step 3: Query DB for all cached state (single transaction)
        let cached_state = Self::query_cached_state(
            repo,
            &property_bank_entry,
            &schema_entries,
            spec.property_bank(),
        )?;

        // Step 4: Combine filesystem + DB state into result
        Self::build_result(
            property_bank_entry,
            schema_entries,
            cached_state,
        )
    }

    // ─────────────────────────────────────────────────────────────────────
    // Filesystem Operations
    // ─────────────────────────────────────────────────────────────────────

    /// Scans filesystem for schema files.
    ///
    /// Returns Vec<FileEntry> (not HashMap) for efficient processing.
    fn scan_filesystem(
        spec: &SchemaConfigSpec,
        vault_root: &std::path::Path,
    ) -> Result<Vec<FileEntry>, SchemaLoaderError> {
        const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];

        let schema_dir = spec.directory();
        let pattern = format!("{}/**/*", schema_dir.as_path().display());

        DirScanner::new(vault_root)
            .entries(
                DirScanInput::new()
                    .with_pattern(&pattern)
                    .with_extensions(&SCHEMA_EXTENSIONS),
            )
            .map_err(|e| SchemaLoaderError::Ingestion(/* ... */))
    }

    /// Separates property bank from schema files (O(n) single pass).
    ///
    /// Performance: No HashMap allocation; returns owned FileEntry values.
    fn separate_property_bank(
        entries: Vec<FileEntry>,
        property_bank_path: &RelativePath,
    ) -> (Option<FileEntry>, Vec<(RelativePath, FileEntry)>) {
        let mut property_bank = None;
        let mut schemas = Vec::with_capacity(entries.len());

        for entry in entries {
            let Ok(path) = RelativePath::try_from(entry.path.clone()) else {
                continue;
            };

            if path == *property_bank_path {
                property_bank = Some(entry);
            } else {
                schemas.push((path, entry));
            }
        }

        (property_bank, schemas)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Database Operations
    // ─────────────────────────────────────────────────────────────────────

    /// Queries all cached state from DB in single transaction.
    ///
    /// Performance: Single batch read with closure (hot path stays inline).
    fn query_cached_state<R>(
        repo: &R,
        property_bank_entry: &Option<FileEntry>,
        schema_entries: &[(RelativePath, FileEntry)],
        property_bank_path: &RelativePath,
    ) -> Result<CachedState, SchemaLoaderError>
    where
        R: crate::schema::storage::Repository,
        R::Error: Into<SchemaRepositoryError>,
    {
        let schema_paths: Vec<_> = schema_entries.iter()
            .map(|(path, _)| path.clone())
            .collect();

        repo.with_batch_schema_reader(|batch_reader| {
            let graph = batch_reader.get_topological_graph()?;

            let property_bank_view = property_bank_entry
                .as_ref()
                .and_then(|_| {
                    batch_reader
                        .get_raw_property_bank_view(property_bank_path)
                        .ok()
                });

            let schema_views = batch_reader
                .find_raw_schema_views_by_paths(&schema_paths)?;
            let schema_ids = batch_reader
                .find_schema_ids_by_paths(&schema_paths)?;

            Ok(CachedState {
                graph,
                property_bank_view,
                schema_views,
                schema_ids,
            })
        })
        .map_err(|e| SchemaLoaderError::Repository(e.into()))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Result Construction
    // ─────────────────────────────────────────────────────────────────────

    /// Builds final DiscoveryResult from filesystem + DB data.
    ///
    /// Performance: Single pass over schema_entries; no intermediate allocations.
    fn build_result(
        property_bank_entry: Option<FileEntry>,
        schema_entries: Vec<(RelativePath, FileEntry)>,
        cached: CachedState,
    ) -> Result<DiscoveryResult, SchemaLoaderError> {
        // Build property bank discovery
        let property_bank = property_bank_entry.map(|entry| {
            PropertyBankDiscovery {
                entry,
                view: cached.property_bank_view,
            }
        });

        // Build schema discoveries with cached state lookup
        let mut schemas = HashMap::with_capacity(schema_entries.len());
        let mut filesystem_ids = HashSet::with_capacity(schema_entries.len());

        for (path, entry) in schema_entries {
            let cached = if let Some(view) = cached.schema_views.get(&path) {
                let id = cached.schema_ids.get(&path).copied()
                    .unwrap_or_else(SchemaId::new);
                filesystem_ids.insert(id);
                Some(SchemaCachedState {
                    id,
                    view: view.clone(),
                })
            } else {
                None
            };

            schemas.insert(path, SchemaDiscovery { entry, cached });
        }

        // Detect deleted schemas
        let deleted_ids = Self::detect_deleted_schemas(
            cached.graph.as_ref(),
            &filesystem_ids,
        );

        Ok(DiscoveryResult {
            schemas,
            property_bank,
            graph: cached.graph,
            deleted_ids,
        })
    }

    /// Detects schemas deleted from filesystem but still in DB.
    ///
    /// Performance: O(n) where n = graph size.
    fn detect_deleted_schemas(
        graph: Option<&InheritanceGraph<()>>,
        filesystem_ids: &HashSet<SchemaId>,
    ) -> Vec<SchemaId> {
        let Some(graph) = graph else {
            return Vec::new();
        };

        graph.topo_order()
            .filter(|id| !filesystem_ids.contains(id))
            .copied()
            .collect()
    }
}

/// Cached state from database (result of batch query).
struct CachedState {
    graph: Option<InheritanceGraph<()>>,
    property_bank_view: Option<RawPropertyBankView>,
    schema_views: HashMap<RelativePath, RawSchemaView>,
    schema_ids: HashMap<RelativePath, SchemaId>,
}
```

### Decomposition Benefits

| Method | Responsibility | Lines | Testable | Performance Impact |
|--------|---------------|-------|----------|-------------------|
| `run()` | Orchestration | ~15 | Integration test | None (delegates) |
| `scan_filesystem()` | FS scan | ~15 | Unit test | None (existing code) |
| `separate_property_bank()` | Classify files | ~15 | Unit test | None (single pass) |
| `query_cached_state()` | DB batch read | ~25 | Unit test | None (same batch query) |
| `build_result()` | Combine data | ~30 | Unit test | None (single pass) |
| `detect_deleted_schemas()` | Find deletions | ~8 | Unit test | None (existing code) |

**Total:** ~108 lines (vs 93 currently) - slight increase for clarity and testability.

### Performance Guarantees

1. ✅ **Single filesystem scan** - `scan_filesystem()` returns `Vec<FileEntry>`
2. ✅ **Single DB transaction** - `query_cached_state()` uses batch reader
3. ✅ **Single pass over entries** - `separate_property_bank()` is O(n)
4. ✅ **Single pass for result** - `build_result()` is O(n)
5. ✅ **No extra allocations** - Intermediate `CachedState` struct is stack-local
6. ✅ **Iterator for deletions** - `detect_deleted_schemas()` uses iterator (no temp vec)

### Testing Improvements

**Before:** Only integration tests possible (entire `run()` method)

**After:** Unit tests for each responsibility:
- `scan_filesystem()` - mock DirScanner
- `separate_property_bank()` - pure function
- `query_cached_state()` - mock Repository
- `build_result()` - pure function
- `detect_deleted_schemas()` - pure function

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| Three locations duplicate DirScanner logic | Consolidate to single DiscoveryEngine::run() call |
| Three locations query DB separately | Already solved - DiscoveryEngine uses batch reader |
| Builder manually separates property bank | Use DiscoveryOutcome::property_bank() instead |
| Processors have discovery stages | Refactor to receive discovered data as input |

## Resources
- **Core Files:**
  - `lithos-core/src/schema/discovery.rs` - DiscoveryEngine (central orchestrator)
  - `lithos-core/src/schema/builder.rs` - Builder (pipeline orchestrator)
  - `lithos-core/src/schema/property_bank_processor.rs` - PropertyBank typestate pipeline
  - `lithos-core/src/schema/schema_processor.rs` - Schema typestate pipeline

- **Infrastructure:**
  - `lithos-core/src/fs/scanner.rs` - DirScanner for filesystem operations
  - `lithos-core/src/schema/storage.rs` - Repository trait with batch operations

- **Patterns:**
  - Unified Repository pattern (single trait per context)
  - Typestate processors for pipeline stages
  - Zero-copy via `with_archived()` closure pattern
  - Batch operations via `with_batch_schema_reader()`

## Architecture Insights

**Current Flow (Fragmented):**
```
Builder.load_all()
  ├─ Builder.discover_files() [DirScanner]
  ├─ Builder.discover_graph() [Repository]
  ├─ Builder.load_property_bank()
  │   └─ PropertyBankProcessor::discover() [Repository]
  └─ SchemaProcessor::discover() [Repository]
      └─ fetch_view_maps() [Repository]
```

**Target Flow (Unified):**
```
Builder.load_all()
  └─ DiscoveryEngine::run() [DirScanner + Repository batch]
      ├─ Returns DiscoveryOutcome
      ├─ Builder uses outcome.property_bank()
      ├─ PropertyBankProcessor receives DiscoveredFile
      └─ SchemaProcessor receives DiscoveryOutcome
```

**Benefits:**
1. Single filesystem scan (not 1-3 scans)
2. Single DB transaction (not 3-4 queries)
3. Atomic consistency (all data from same snapshot)
4. Clear separation: discovery is infrastructure, processors are transformations
5. No duplicate code across builder, property_bank_processor, schema_processor

---

## Phase 4 Implementation Plan: Refactor Builder

### Current State Analysis

**Builder.load_all() Current Flow (lines 58-127):**
1. Call `discover_files()` → Returns `FilesContext` + callback with `PropertyBankContext`
2. Call `discover_graph()` → Returns `GraphContextBranch`
3. If bank present: call `load_property_bank()`
4. Route to SchemaProcessor based on graph presence
5. Process through pipeline

**Methods to Remove/Replace:**
1. **`discover_files()` (lines 158-242)**:
   - Uses DirScanner (duplicate of DiscoveryEngine)
   - Returns FilesContext { files: Vec<RelativePath> }
   - Callback with PropertyBankContext { path: RelativePath }
   - REPLACE WITH: DiscoveryEngine::run()

2. **`discover_graph()` (lines 244-258)**:
   - Queries repository.get_topological_graph() (duplicate)
   - Returns GraphContextBranch
   - REPLACE WITH: Data from DiscoveryResult

**Context Types to Remove:**
- `FilesContext` - Replace with iterator over DiscoveryResult.schemas
- `PropertyBankContext` - Replace with DiscoveryResult.property_bank
- `BankContextBranch` - Replace with Option from DiscoveryResult.property_bank
- `GraphContextBranch` - Replace with DiscoveryResult.graph (already Option)

### Target State

**New Builder.load_all() Flow:**
```rust
pub fn load_all(&mut self) -> Result<Vec<Arc<Schema>>, SchemaLoaderError> {
    // 1. Single discovery call
    let discovery = DiscoveryEngine::run(
        &self.config.to_schema_spec(),
        &self.repository,
        self.source.root()
    )?;

    // 2. Load property bank if present
    let property_bank = if let Some(bank_discovery) = &discovery.property_bank {
        Some(self.load_property_bank(bank_discovery)?)
    } else {
        self.property_bank_delta = None;
        None
    };

    // 3. Early exit if no schemas
    if !discovery.has_schemas() {
        return Ok(Vec::new());
    }

    let property_bank = property_bank.unwrap_or_else(PropertyBank::new);

    // 4. Route to SchemaProcessor based on graph presence
    let branch = match &discovery.graph {
        Some(graph) => SchemaProcessor::<Discovery, Review>::from_discovery_result(
            &discovery,
            graph,
            &self.repository,
            &self.source,
        )?,
        None => SchemaProcessor::<Discovery, NeverSeen>::from_discovery_result(
            &discovery,
            &self.source,
        )?,
    };

    // 5. Process through pipeline (unchanged)
    match branch { /* ... */ }
}
```

**New load_property_bank() signature:**
```rust
fn load_property_bank(
    &mut self,
    bank_discovery: &PropertyBankDiscovery,
) -> Result<PropertyBank, SchemaLoaderError>
```

### Implementation Steps

**Step 1: Add DiscoveryEngine import**
- Add `DiscoveryEngine` to imports
- Add `DiscoveryResult`, `PropertyBankDiscovery` to imports

**Step 2: Update load_all() method**
- Replace `discover_files()` + `discover_graph()` with single `DiscoveryEngine::run()`
- Replace `FilesContext` with `&DiscoveryResult`
- Replace `BankContextBranch` with `Option<&PropertyBankDiscovery>`
- Replace `GraphContextBranch` with `Option<&InheritanceGraph<()>>`
- Update property bank loading call
- Update SchemaProcessor routing

**Step 3: Update load_property_bank() signature**
- Change parameter from `&PropertyBankContext` to `&PropertyBankDiscovery`
- Update PropertyBankProcessor::discover() call to pass bank_discovery data
  (Note: PropertyBankProcessor changes will be in Phase 5)

**Step 4: Remove obsolete methods and types**
- Remove `discover_files()` method
- Remove `discover_graph()` method
- Remove `FilesContext` type
- Remove `PropertyBankContext` type
- Remove `BankContextBranch` enum
- Remove `GraphContextBranch` enum

**Step 5: Update tests**
- `builder_discovery_loads_graph_from_db()` → Test via load_all() or remove
- `builder_discovery_excludes_property_bank()` → Test via load_all() or remove
- `builder_discovery_handles_missing_graph()` → Test via load_all() or remove
- `builder_discovery_filters_by_extension()` → Test moved to DiscoveryEngine
- `builder_constructs()` → Should still pass (tests load_all())

### Critical Considerations

**Config Method:**
- Config has `to_schema_spec()` method (config/aggregate.rs:111)
- Returns `SchemaConfigSpec` with validated paths
- Use: `self.config.to_schema_spec()`

**Error Handling:**
- DiscoveryEngine returns `Result<DiscoveryResult, SchemaLoaderError>`
- Errors should propagate correctly (same error type)

**PropertyBankProcessor Integration:**
- Phase 4 only changes Builder's signature
- PropertyBankProcessor.discover() still expects old signature (will update in Phase 5)
- Need to extract path from PropertyBankDiscovery: `bank_discovery.entry.path()`

**SchemaProcessor Integration:**
- Will need new constructor methods (Phase 6):
  - `SchemaProcessor::<Discovery, Review>::from_discovery_result()`
  - `SchemaProcessor::<Discovery, NeverSeen>::from_discovery_result()`
- For now, will keep existing discover() methods and adapt the call

### Testing Strategy

1. Run existing Builder tests after each change
2. Ensure `builder_constructs()` still passes (integration test)
3. Verify discovery tests are moved/removed appropriately
4. Run full schema test suite: `mise run test:unit:schema`

### Risk Mitigation

- **Small commits**: Commit after each working step
- **Tests first**: Ensure tests run before removing code
- **Incremental changes**: Don't remove old code until new code works
- **Verification**: Run tests after each modification

### Success Criteria

- [ ] Builder.load_all() uses DiscoveryEngine::run() (single call)
- [ ] No duplicate DirScanner usage in builder.rs
- [ ] No duplicate repository.get_topological_graph() call
- [ ] Context types removed (FilesContext, PropertyBankContext, etc.)
- [ ] All builder tests pass
- [ ] All schema unit tests pass
- [ ] Code is cleaner and more maintainable
