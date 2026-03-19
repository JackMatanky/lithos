# Schema Module Pipeline Review: Complete Analysis for State Machine Redesign

**Date**: 2026-03-19
**Purpose**: Comprehensive review of all pipeline stages to inform typestate pattern implementation
**Scope**: PropertyBank and Schema pipelines in `lithos-core/src/schema/`

---

## Executive Summary

The schema module implements two parallel pipelines:

1. **PropertyBank Pipeline**: Discovery → Branching (NEW/FRESH/STALE) → Construction → Persistence
2. **Schema Pipeline**: File → Raw → Expanded → Tree → Merged → Storage

Both pipelines share infrastructure (Ingestor, Repository) and follow distinct stages with clear inputs/outputs but **lack explicit state machine enforcement**. This leads to:

- **Unorganized orchestration** in `Loader`
- **Implicit state transitions** scattered across modules
- **Difficult-to-track** intermediate data flows
- **Complex staleness detection** interleaved with processing
- **"Validate, then use" anti-pattern** in Raw types (violates "parse, don't validate")

**Recommendation**: Implement **two separate state machines** with prerequisite refactoring:

**Prerequisites**:
1. Refactor `RawPropertyBank` and `RawSchema` to use "parse, don't validate" pattern
   - Move validation into parsing constructors
   - Make fields private, add public accessors
   - Guarantee validity at type level

**State Machines**:
1. `PropertyBankStateMachine` - 7 states, branching based on staleness (NEW/FRESH/STALE paths)
2. `SchemaStateMachine` - 10 states, complex branching with staleness optimizations

---

## 1. PropertyBank Pipeline Stages

### 1.1 State Identification

The PropertyBank follows a **branching pipeline** with 7 distinct states, determined by staleness detection:

```
┌───────────┐
│ Discovery │──────┐
└───────────┘      │
                   │  Query DB for RawPropertyBankView
                   │  Determine staleness (NEW/FRESH/STALE)
                   │
       ┌───────────┴───────────┬─────────────────┬──────────────┐
       │                       │                 │              │
       ▼                       ▼                 ▼              ▼
    ┌─────┐               ┌───────┐         ┌────────┐      ┌───────┐
    │ NEW │               │ FRESH │         │FRESH+TS│      │ STALE │
    └──┬──┘               └───┬───┘         └───┬────┘      └───┬───┘
       │                      │                 │               │
       ▼                      │                 │               ▼
  ┌────────────┐              │                 │         ┌────────────┐
  │FileParsed  │              │                 │         │FileParsed  │
  └─────┬──────┘              │                 │         └─────┬──────┘
        │                     │                 │               │
        ▼                     │                 │               ▼
  ┌──────────────┐            │                 │         ┌──────────────┐
  │BaseConstructed│           │                 │         │PropertyDelta │
  │(from scratch) │           │                 │         └──────┬───────┘
  └──────┬────────┘           │                 │                │
         │                    │                 │                ▼
         │                    │                 │         ┌──────────────┐
         │                    │                 │         │BaseConstructed│
         │                    │                 │         │(from DB)     │
         │                    │                 │         └──────┬───────┘
         │                    │                 │                │
         │                    │                 │                ▼
         │                    │                 │         ┌────────────┐
         │                    │                 │         │DeltaApplied│
         │                    │                 │         └─────┬──────┘
         │                    │                 │               │
         ▼                    ▼                 ▼               ▼
  ┌──────────────────────────────────────────────────────────────┐
  │                      ViewUpdated                             │
  └───────────────────────────┬──────────────────────────────────┘
                              │
                              ▼
                       ┌────────────┐
                       │ Completed  │
                       └────────────┘
```

**State Details**:

| State | Data Structure | Location | Transitions | Notes |
|-------|---------------|----------|-------------|-------|
| **1. Discovery** | `&Path` + `Option<RawPropertyBankView>` | Entry point | → PropertyBankPath enum | Query DB, determine staleness |
| **2. FileParsed** | `RawPropertyBank` (validated) | NEW/STALE paths only | → BaseConstructed or PropertyDelta | File parsed + validated in one step |
| **3. PropertyDelta** | Delta info (new/modified/removed) | STALE path only | → BaseConstructed | Property-level hash comparison |
| **4. BaseConstructed** | `PropertyBank` | All paths converge here | → DeltaApplied or ViewUpdated | Fetched from DB or built from scratch |
| **5. DeltaApplied** | `PropertyBank` (updated) | STALE path only | → ViewUpdated | Incremental updates applied |
| **6. ViewUpdated** | `RawPropertyBankView` | All paths | → Completed | View created/updated with metadata |
| **7. Completed** | `PropertyBank` (persisted) | Terminal state | - | Both bank and view persisted to DB |

**Note**: The "Validated" state from the original analysis has been **eliminated** - validation now happens during parsing (see Section 1.8 for "parse, don't validate" refactoring).

### 1.2 Stage-by-Stage Breakdown

The PropertyBank pipeline branches into **four distinct paths** based on staleness detection:

#### **Discovery Phase** (State 1)

**Operations**:
1. Query `Repository::get_raw_property_bank_view()` → `Option<RawPropertyBankView>`
2. If `None` → **NEW** path
3. If `Some(view)`:
   - Extract file timestamps (`created_at`, `modified_at`)
   - Compare with `view.file_times()`
   - **If timestamps match** → **FRESH** path
   - **If timestamps differ**:
     - Read file content with `FsReader::read_with()`
     - Compute `blake3::hash()` of content
     - Compare with `view.hashes().content_hash()`
     - **If hash matches** → **FRESH+TS** path (timestamp update needed)
     - **If hash differs** → **STALE** path

**Errors**: `SchemaRepositoryError`, `SchemaFileError`
**Location**: `Ingestor::property_bank()` lines 473-510

**Output**: `PropertyBankPath` enum with one of:
- `PropertyBankPath::New(PropertyBankPipeline<FileParsed>)`
- `PropertyBankPath::Fresh(PropertyBankPipeline<BaseConstructed>)`
- `PropertyBankPath::FreshWithTimestampUpdate(PropertyBankPipeline<BaseConstructed>)`
- `PropertyBankPath::Stale(PropertyBankPipeline<FileParsed>)`

---

#### **Path 1: NEW** (No cached view exists)

**Steps**:

**State 1→2: File Parsing**
- **Input**: `&Path` to property bank file
- **Output**: `RawPropertyBank` (validated)
- **Operations**:
  - Read file with `FsReader::read_with()`
  - Parse via `RawPropertyBank::try_from_str()` (parsing + validation in one step)
  - Version check (`$version` field)
  - Property name syntax validation
- **Errors**: `SchemaParseError`, `SchemaVersionError`, `PropertyNameError`
- **Location**: `Ingestor::ingest_new_property_bank()` line 522

**State 2→4: Base Construction (from scratch)**
- **Input**: `RawPropertyBank` (validated)
- **Output**: `PropertyBank`
- **Operations**:
  - `PropertyBank::try_from(RawPropertyBank)` - convert all raw entries to `Property`
  - For each property:
    - Convert `RawPropertySpec` → `PropertySpec`
    - Create `PropertyId::new()` (UUID v7)
    - Create `PropertyName::try_new()`
    - Set `Optionality`, `Multiplicity`
    - Call `PropertyBank::register()`
- **Errors**: `SchemaError::PropertyBank(DuplicatePropertyName)`, `PropertySpec` conversion errors
- **Location**: `PropertyBank::try_from()` in `bank.rs:313-367`

**State 4→6: View Update**
- **Input**: `PropertyBank`, file content, timestamps, hash
- **Output**: `RawPropertyBankView`
- **Operations**:
  - Create `RawPropertyBankView::try_from_with_content()` with all metadata
- **Location**: `Ingestor::ingest_new_property_bank()` line 527

**State 6→7: Persist & Complete**
- **Operations**:
  - `Repository::save_property_bank(&bank)`
  - `Repository::save_raw_property_bank_view(&view)`
- **Location**: Lines 530-545

---

#### **Path 2: FRESH** (Timestamps match)

**Steps**:

**State 1→4: Base Construction (from DB)**
- **Input**: None (view timestamps match)
- **Output**: `PropertyBank`
- **Operations**:
  - `Repository::get_property_bank()` - fetch cached bank
- **Errors**: `SchemaStorageError::PropertyBankNotFound` (if DB inconsistent)
- **Location**: `Ingestor::property_bank()` lines 498-505

**State 4→7: Complete (no persistence needed)**
- **Operations**: Return cached `PropertyBank`
- **Note**: No view update, no bank persistence (everything is fresh)

---

#### **Path 3: FRESH+TS** (Content hash matches but timestamps differ)

**Steps**:

**State 1→4: Base Construction (from DB)**
- **Input**: None (content unchanged)
- **Output**: `PropertyBank`
- **Operations**: `Repository::get_property_bank()` - fetch cached bank

**State 4→6: View Update (timestamps only)**
- **Input**: Existing `RawPropertyBankView`, new timestamps
- **Output**: Updated `RawPropertyBankView`
- **Operations**:
  - Clone existing view
  - Update `file_times()` with new timestamps
  - Keep content hash, property hashes, and compressed content unchanged

**State 6→7: Persist View Only**
- **Operations**:
  - `Repository::save_raw_property_bank_view(&updated_view)`
  - **NO** `save_property_bank()` (bank unchanged)

---

#### **Path 4: STALE** (Content hash differs)

**Steps**:

**State 1→2: File Parsing**
- Same as NEW path (read + parse + validate)

**State 2→3: Property Delta Computation**
- **Input**: `RawPropertyBank` (new), `RawPropertyBankView` (cached)
- **Output**: Delta information (new/modified/removed properties)
- **Operations**:
  - Compute per-property hashes for new `RawPropertyBank.properties`
  - Compare with `view.hashes().property_hashes()`
  - Identify:
    - **New properties**: In new but not in cached
    - **Modified properties**: In both but hash differs
    - **Removed properties**: In cached but not in new
- **Location**: `Ingestor::ingest_stale_property_bank()` lines 580-588

**State 3→4: Base Construction (from DB)**
- **Input**: None
- **Output**: `PropertyBank` (base state before updates)
- **Operations**: `Repository::get_property_bank()` - fetch current bank

**State 4→5: Delta Application**
- **Input**: `PropertyBank` (base), Delta info, `RawPropertyBank` (new)
- **Output**: `PropertyBank` (updated)
- **Operations**:
  - `PropertyBank::update_from_raw(&raw, &changed_properties)`
  - For each changed property:
    - If in new raw: update or add property
    - If not in new raw: remove property
  - Increment `BankVersion` if any changes applied
- **Errors**: `SchemaError::PropertyBank`
- **Location**: `bank.rs:255-300`

**State 5→6: View Update**
- **Input**: Updated `PropertyBank`, new file content, timestamps, hashes
- **Output**: New `RawPropertyBankView`
- **Operations**: Create full view with all new metadata

**State 6→7: Persist Both**
- **Operations**:
  - `Repository::save_property_bank(&updated_bank)`
  - `Repository::save_raw_property_bank_view(&new_view)`

### 1.3 Staleness Detection Details

**Four Distinct Paths** based on view existence and content comparison:

| Path | Trigger | File I/O | Parsing | DB Reads | DB Writes | Notes |
|------|---------|----------|---------|----------|-----------|-------|
| **NEW** | No view in DB | ✅ Full read | ✅ Parse + validate | - | Bank + View | First time seeing file |
| **FRESH** | Timestamps match | ❌ None | ❌ None | Bank only | - | Fastest path (cached) |
| **FRESH+TS** | Hash matches, timestamps differ | ✅ Read for hash | ❌ None | Bank only | View only | Clock skew handling |
| **STALE** | Hash differs | ✅ Full read | ✅ Parse + validate | Bank only | Bank + View | Incremental update |

**Two-Tier Staleness Check** (fast path → slow path):

1. **Fast Path** (no file I/O): Compare timestamps
   - `view.file_times().is_timestamp_match(created_at, modified_at)`
   - If match → FRESH path
   - If mismatch → proceed to slow path

2. **Slow Path** (single file read): Compare content hash
   - Read file content with `FsReader::read_with()`
   - Compute `blake3::hash(content.as_bytes())`
   - `view.hashes().is_content_match(content_hash)`
   - If match → FRESH+TS path
   - If mismatch → STALE path

**Key Optimization**: Content hash check prevents unnecessary re-parsing when timestamps change but content is identical (e.g., file copied, touched, or clock skew).

**Location**: `Ingestor::property_bank()` lines 473-510

---

### 1.4 Prerequisite Refactoring: "Parse, Don't Validate"

**Problem**: Current `RawPropertyBank` and `RawSchema` violate the "parse, don't validate" principle:

```rust
// Current anti-pattern: validation is optional
let raw: RawPropertyBank = serde_json::from_str(content)?;  // Can create invalid instance!
raw.validate()?;  // Caller might forget this step
```

**Issues**:
- ❌ Can construct invalid `Raw*` instances
- ❌ Validation is optional (caller can forget to call it)
- ❌ Type system doesn't enforce validity
- ❌ Separate "Validated" state needed in state machine

**Solution**: Transform validation into parsing (make invalid states unrepresentable):

```rust
// Better: Parsing constructor ensures validity
let raw = RawPropertyBank::try_from_str(content, FileFormat::Json, path)?;
// Guaranteed valid! Can't create invalid instance.
```

#### **Refactoring Plan**

**Step 1: Make Fields Private**

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RawPropertyBank {
    /// Property bank format version (validated to be supported)
    #[serde(rename = "$version")]
    version: RawSchemaVersion,  // PRIVATE

    /// Map of property name to property definition (validated)
    properties: HashMap<Box<str>, property::RawPropertyBankEntry>,  // PRIVATE

    #[serde(skip)]
    metadata: RawPropertyBankMetadata,  // PRIVATE
}
```

**Step 2: Add Parsing Constructor**

```rust
impl RawPropertyBank {
    /// Parse and validate property bank from string.
    ///
    /// This is the ONLY way to construct a RawPropertyBank externally,
    /// ensuring all instances are valid.
    ///
    /// # Errors
    /// Returns error if:
    /// - Deserialization fails (syntax error)
    /// - Version is unsupported
    /// - Property names are invalid
    pub fn try_from_str(
        content: &str,
        format: FileFormat,
        path: &str
    ) -> Result<Self, SchemaIngestionError> {
        // Parse via serde
        let raw: Self = match format {
            FileFormat::Json => serde_json::from_str(content)
                .map_err(|e| SchemaParseError::Json { source: e })?,
            FileFormat::Toml => toml::from_str(content)
                .map_err(|e| SchemaParseError::Toml { source: e })?,
            FileFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| SchemaParseError::Yaml { source: e })?,
        };

        // Validate version (fail fast)
        raw.version.validate(path)?;

        // Validate property names (syntax only, not references)
        for name in raw.properties.keys() {
            PropertyName::try_new(name.as_ref())?;
        }

        Ok(raw)
    }
}
```

**Step 3: Add Public Accessors**

```rust
impl RawPropertyBank {
    /// Returns the schema version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &RawSchemaVersion {
        &self.version
    }

    /// Returns the properties map.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<Box<str>, property::RawPropertyBankEntry> {
        &self.properties
    }

    /// Returns the metadata.
    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &RawPropertyBankMetadata {
        &self.metadata
    }
}
```

**Step 4: Remove Old Methods**

Delete:
- `RawPropertyBank::validate()`
- `RawPropertyBank::validate_version()`
- `RawPropertyBank::validated()`

**Step 5: Update Ingestor**

```rust
// Before
let raw: RawPropertyBank = FsReader::parse_structured_from_str(path, content)?;
let raw = raw.validated(&path.to_string_lossy())?;

// After
let raw = RawPropertyBank::try_from_str(
    content,
    FileFormat::from_path(path)?,
    &path.to_string_lossy()
)?;
```

**Step 6: Repeat for `RawSchema`**

Apply the same pattern:
- Make fields private
- Add `RawSchema::try_from_str()`
- Add public accessors
- Remove `validate()`, `validate_version()`, `validated()`

#### **Benefits**

| Before | After |
|--------|-------|
| Can create invalid `RawPropertyBank` | Type guarantees validity |
| Validation is optional | Validation is mandatory (at construction) |
| Two-step process (parse, then validate) | One-step process (parsing = validation) |
| Separate "Validated" state in state machine | No separate state needed |
| Public fields (can mutate to invalid state) | Private fields (immutable after construction) |

**Impact on State Machine**:
- **Eliminates "Validated" state** (parsing = validation)
- **FileParsed state now guarantees validity**
- Cleaner state transitions

**Estimated Effort**: 2-3 hours

**Files to Modify**:
- `lithos-core/src/schema/raw/mod.rs` (RawPropertyBank and RawSchema)
- `lithos-core/src/schema/ingestor.rs` (update to use `try_from_str()`)
- Tests in both files

---

## 2. Schema Pipeline Stages

### 2.1 State Identification

The Schema pipeline is **complex and branching** with 10 primary states:

```
                                  ┌─────────────────┐
                                  │  FileRef List   │
                                  │  (Vec<Path>)    │
                                  └────────┬────────┘
                                           │
                                           ▼
                         ┌─────────────────────────────────┐
                         │    Bulk Staleness Detection     │
                         │    (partition all schemas)      │
                         └─────────┬───────────────────────┘
                                   │
                   ┌───────────────┼───────────────┐
                   ▼               ▼               ▼
           ┌──────────┐    ┌──────────┐    ┌──────────┐
           │   NEW    │    │   STALE  │    │  FRESH   │
           └─────┬────┘    └─────┬────┘    └─────┬────┘
                 │               │               │
                 ├───────────────┘               │
                 ▼                               │
         ┌──────────────┐                        │
         │  RawSchema   │                        │
         │  (parsed)    │                        │
         └──────┬───────┘                        │
                │                                │
                ▼                                │
    ┌────────────────────────┐                   │
    │  RefExpandedSchema     │                   │
    │  (refs resolved)       │                   │
    └────────────┬───────────┘                   │
                 │                               │
                 ▼                               │
         ┌──────────────┐                        │
         │  SchemaTree  │ ←──────────────────────┘
         │  (topology)  │    (known_parents)
         └──────┬───────┘
                │
                ▼
         ┌──────────────┐
         │   Schema     │
         │  (resolved)  │
         └──────┬───────┘
                │
                ▼
         ┌──────────────┐
         │  Persisted   │
         └──────────────┘
```

**State Details**:

| State                | Data Structure                | Location                     | Transitions       | Notes                    |
| -------------------- | ----------------------------- | ---------------------------- | ----------------- | ------------------------ |
| **1. FileRefList**   | `Vec<PathBuf>`                | `Ingestor::ingest_all()`     | → BulkStaleness   | List of all schema files |
| **2. BulkStaleness** | `HashMap<Path, SchemaResult>` | `Ingestor::ingest_all()`     | → NEW/STALE/FRESH | Partition by staleness   |
| **3a. NEW**          | `(SchemaId, RawSchema)`       | `SchemaResult::New`          | → RefExpanded     | First-time file          |
| **3b. STALE**        | `(SchemaId, RawSchema)`       | `SchemaResult::Stale`        | → RefExpanded     | File changed             |
| **3c. FRESH**        | `SchemaId`                    | `SchemaResult::Fresh`        | → (skip pipeline) | File unchanged           |
| **4. RawSchema**     | `RawSchema`                   | After parsing                | → RefExpanded     | Syntax-validated         |
| **5. RefExpanded**   | `RefExpandedSchema`           | `RefExpander::expand_all()`  | → SchemaTree      | Bank refs resolved       |
| **6. SchemaTree**    | `SchemaTree`                  | `Extender::build()`          | → Schema          | Topological order        |
| **7. Schema**        | `Schema`                      | `Merger::resolve()`          | → Persisted       | Fully resolved           |
| **8. Persisted**     | `Schema` (in DB)              | `Repository::save_schemas()` | → Done            | Cached in database       |

### 2.2 Stage-by-Stage Breakdown

#### Stage 1: File Discovery

**Input**: Config paths
**Output**: `Vec<PathBuf>` (all schema files)
**Operations**:

- Scan schemas directory for `**/*.json`, `**/*.toml`, `**/*.yaml`
- Exclude property_bank file
- Return list of paths
  **Errors**: `SchemaFileError::FileSystem`
  **Location**: `Ingestor::list_all_schema_files()` lines 821-849

#### Stage 2: Bulk Staleness Partitioning

**Input**: `Vec<PathBuf>` (file list)
**Output**: `HashMap<PathBuf, SchemaResult>` (partitioned by staleness)
**Operations**:

1. **Bulk queries** (NO N+1):
   - `Repository::find_raw_schema_views_by_paths()` → `HashMap<Path, RawSchemaView>`
   - `Repository::find_schema_ids_by_paths()` → `HashMap<Path, SchemaId>`
2. **Per-file staleness check**:
   - Compare timestamps (fast path, no I/O)
   - If mismatch, read file and compare hash (slow path)
   - Classify as NEW/STALE/FRESH
3. **Result variants**:
   - `SchemaResult::New { id, raw }` - no view in DB
   - `SchemaResult::Stale { id, raw, expanded }` - hash mismatch
   - `SchemaResult::Fresh { id, expanded }` - hash match
     **Errors**: `SchemaRepositoryError`, `SchemaFileError`
     **Location**: `Ingestor::ingest_all()` lines 781-816

#### Stage 3a-c: Schema Result Partitioning

**Input**: `HashMap<PathBuf, SchemaResult>`
**Output**: Three vectors: `needs_expansion`, `cached_expansion`, `fresh_ids`
**Operations** (in `Loader::load()` lines 150-202):

- **Bank Fresh + Schema Fresh** → `fresh_ids` (fully reusable, skip all stages)
- **Bank Stale + Schema Fresh + Has Cache** → `cached_expansion` (skip RefExpander)
- **Bank Stale + Schema Fresh + No Cache** → `needs_expansion` (run full pipeline)
- **File Changed or New** → `needs_expansion` (run full pipeline)
  **Location**: `Loader::load()` lines 150-202

#### Stage 4: Property Reference Expansion

**Input**: `Vec<(SchemaId, RawSchema)>` (needs expansion)
**Output**: `Vec<(SchemaId, RefExpandedSchema)>`
**Operations**:

- For each schema:
  - For each property in `raw.properties`:
    - **Inline property** → Convert `RawPropertySpec` to `PropertySpec` directly
    - **Bank reference** (`$ref`) → Lookup in `PropertyBank`, apply overrides via `Resolver::from_bank_ref()`
  - Store result as `HashMap<PropertyName, Property>`
    **Errors**: `SchemaError::PropertyRef(NotFound)`, `PropertySpec` conversion errors
    **Location**: `RefExpander::expand_all()` in `expander.rs:105-116`

**Key Substeps**:

1. `RefExpander::expand_schema()` - process one schema
2. `RefExpander::expand_property()` - process one property
   - `RawProperty::Inline` → Direct conversion
   - `RawProperty::Ref` → Bank lookup + override application

#### Stage 5: Inheritance Tree Building

**Input**:

- `Vec<(SchemaId, RefExpandedSchema)>` (stale schemas)
- `HashMap<SchemaId, Schema>` (known_parents, from DB)

**Output**: `SchemaTree` (topologically ordered)

**Operations** (6 phases in `Extender::build()` lines 222-250):

**Phase 1**: Build name indexes

- `name_to_id: HashMap<Box<str>, SchemaId>` - forward lookup
- `id_to_name: HashMap<SchemaId, Box<str>>` - reverse lookup
- Includes both stale schemas and known_parents
- Detect duplicate names

**Phase 2**: Build node map

- For each `RefExpandedSchema`:
  - Resolve `extends` name → parent `SchemaId` via `name_to_id`
  - Create `SchemaNode` with:
    - `name`, `properties`, `excludes`
    - `parent_id` (resolved)
    - `children: Vec::new()` (populated in Phase 4)
    - `depth: NodeDepth::root()` (computed in Phase 5)

**Phase 3**: DFS cycle detection

- For each node with `parent_id`:
  - Walk up the parent chain
  - Track visited IDs in `HashSet`
  - If loop detected → `SchemaError::Inheritance(CircularInheritance)`

**Phase 4**: Populate children lists

- Reverse parent→child relationships
- For each node with `parent_id`:
  - Add `child_id` to `parent.children`

**Phase 5**: Compute inheritance depths

- BFS traversal starting from roots
- Root depth = 1
- Child depth = parent depth + 1
- Accounts for both in-batch parents and known_parents

**Phase 6**: Kahn's topological ordering

- Initialize: roots have in-degree 0
- Queue roots, process children when all parents visited
- Result: `Vec<SchemaId>` in topological order (parents before children)

**Errors**:

- `SchemaError::Resolution(DuplicateSchemaName)`
- `SchemaError::Inheritance(CircularInheritance)`
- `SchemaError::Inheritance(ParentNotFound)`

**Location**: `Extender::build()` in `extender.rs:222-250`

#### Stage 6: Property Merging (Schema Resolution)

**Input**:

- `SchemaTree` (topological order)
- `HashMap<SchemaId, Schema>` (known_parents)

**Output**: `Vec<Schema>` (fully resolved)

**Operations** (single linear walk in `Merger::resolve()` lines 69-152):

- Walk tree in topological order (parents before children)
- For each node:
  1. **Depth check**: Verify `node.depth <= INHERITANCE_MAX_DEPTH` (10)
  2. **Get parent properties**:
     - If `node.parent_id.is_some()`:
       - Lookup in `resolved_cache` (in-batch parent, already processed)
       - OR lookup in `known_parents` (DB-fresh parent)
     - If `None` (root schema): use empty `HashMap`
  3. **Merge properties**:
     - Start with child's own properties (child overrides)
     - Add parent properties NOT in excludes list and NOT already in child
     - Result: `HashMap<PropertyName, Property>`
  4. **Construct Schema**:
     - `Schema::new(id, name, parent_id, children, merged_properties)`
  5. **Cache result**: Store in `resolved_cache` for downstream children
  6. **Add to results**: Append to output vector

**Key Merging Rules**:

- Child property with same name **completely replaces** parent property
- Parent properties in `excludes` list are **not inherited**
- All other parent properties are **inherited**

**Errors**:

- `SchemaError::Inheritance(DepthExceeded)`
- `SchemaError::Resolution(MissingNode)`

**Location**: `Merger::resolve()` in `merger.rs:69-152`

#### Stage 7-8: Persistence

**Input**: `Vec<Schema>` (resolved)
**Output**: Persisted to database

**Operations**:

1. `Repository::save_schemas()` - bulk save (lines 396-402)
2. `Repository::save_inheritance_metadata()` - cache inheritance views (lines 417-474)
3. `Repository::save_raw_schema_view()` - save staleness metadata (already done in Stage 2)

**Metadata Persisted** (for future staleness checks):

- `RawSchemaView` - timestamps, hashes, compressed content, expanded properties
- `SchemaInheritanceView` - parent, ancestors, excludes, ancestors_hash, resolved_at

**Errors**: `SchemaRepositoryError::Storage`
**Location**: `Loader::persist_resolved_schemas()`, `Loader::persist_inheritance_metadata()`

### 2.3 Cached Expansion Optimization (Phase 5.2)

**Special Path**: When PropertyBank is stale but Schema file is fresh AND cached expansion exists

**Input**: `Vec<(SchemaId, HashMap<PropertyName, Property>)>` (cached_expansion)
**Output**: `Vec<Schema>` (resolved via cached path)

**Operations** (lines 282-319):

1. For each `(id, cached_props)`:
   - Load `RawSchemaView` from DB
   - Extract `raw.name`, `raw.extends`, `raw.excludes`
   - Construct `RefExpandedSchema` directly from cached properties (skip `RefExpander`!)
2. Call `Extender::build()` with cached expansions + known_parents
3. Call `Merger::resolve()` as normal

**Key Optimization**: Skips property reference expansion when:

- Schema file unchanged (timestamps + hash match)
- PropertyBank changed
- Previous expansion cached in `RawSchemaView.current().expanded_properties()`

**Location**: `Loader::resolve_with_cached_expansion()` lines 282-319

---

## 3. Cross-Cutting Concerns

### 3.1 Staleness Detection Architecture

**Two-Tier Strategy**:

1. **Fast Path**: Timestamp comparison (no I/O)
2. **Slow Path**: Content hash comparison (single file read)

**Metadata Types**:

| Type                    | Purpose                | Storage          | Data                                                              |
| ----------------------- | ---------------------- | ---------------- | ----------------------------------------------------------------- |
| `RawPropertyBankView`   | PropertyBank staleness | Singleton table  | timestamps, content_hash, property_hashes, compressed_content     |
| `RawSchemaView`         | Schema staleness       | Per-schema table | timestamps, content_hash, compressed_content, expanded_properties |
| `SchemaInheritanceView` | Inheritance cache      | Per-schema table | parent, ancestors, excludes, ancestors_hash, resolved_at          |

**Timestamp Fields**:

- `created_at: Option<SystemTime>` - file creation time
- `modified_at: Option<SystemTime>` - file modification time

**Hash Fields**:

- `content_hash: [u8; 32]` - blake3 hash of file content
- `property_hashes: HashMap<PropertyName, [u8; 32]>` - per-property hashes (PropertyBank only)

**Comparison Logic** (in `views/metadata.rs`):

```rust
// Fast path: timestamp match
view.file_times().is_timestamp_match(created_at, modified_at)

// Slow path: hash match
view.hashes().is_content_match(content_hash)
```

### 3.2 Error Taxonomy (Hierarchical)

**Umbrella Errors** (top level):

- `SchemaError` - catch-all for domain errors
- `SchemaIngestionError` - file loading + parsing pipeline
- `SchemaRepositoryError` - database operations
- `SchemaLoaderError` - orchestration failures

**Pipeline Errors** (specific stages):

- `SchemaFileError` - file I/O (Stage 1→2)
- `SchemaParseError` - deserialization (Stage 2→3)
- `SchemaVersionError` - version validation (Stage 3→4)
- `SchemaSyntaxError` - syntax validation (Stage 3→4)
- `SchemaStorageError` - database persistence (Stage 5→6)

**Sub-Domain Errors** (property/schema level):

- `SchemaNameError` - schema name validation
- `PropertyNameError` - property name validation
- `PropertySpecError` - property spec validation
- `PropertyValueError` - property value validation
- `PropertyRefError` - bank reference resolution
- `PropertyBankError` - bank registration
- `SchemaInheritanceError` - inheritance logic
- `SchemaResolutionError` - schema resolution

### 3.3 Repository Interface

**Unified Repository Trait** (`schema::storage::Repository`):

**PropertyBank Operations**:

- `get_property_bank() -> Result<Option<PropertyBank>>`
- `save_property_bank(&PropertyBank) -> Result<()>`
- `get_raw_property_bank_view() -> Result<Option<RawPropertyBankView>>`
- `save_raw_property_bank_view(&RawPropertyBankView) -> Result<()>`

**Schema Operations**:

- `find_schema_by_name(&SchemaName) -> Result<Option<Schema>>`
- `find_schemas_by_ids(&[SchemaId]) -> Result<Vec<Schema>>`
- `find_schema_ids_by_paths(&[PathBuf]) -> Result<HashMap<PathBuf, SchemaId>>`
- `save_schemas(&[Schema]) -> Result<()>`

**View Operations**:

- `get_raw_schema_view(SchemaId) -> Result<Option<RawSchemaView>>`
- `find_raw_schema_view_by_path(&str) -> Result<Option<RawSchemaView>>`
- `find_raw_schema_views_by_paths(&[PathBuf]) -> Result<HashMap<PathBuf, RawSchemaView>>`
- `save_raw_schema_view(SchemaId, &RawSchemaView) -> Result<()>`

**Inheritance Operations**:

- `save_inheritance_metadata(SchemaId, &SchemaInheritanceView) -> Result<()>`

**Implementations**:

- `RedbRepository` - production (redb zero-copy database)
- `InMemoryRepository` - testing (HashMap-based)
- `FakeStorage` - mocking (configurable behavior)

---

## 4. State Machine Design Recommendations

### 4.1 PropertyBank State Machine

**Consolidated States** (7 total):

```rust
// State types (zero-sized markers)
struct Discovery;
struct FileParsed;
struct PropertyDelta;
struct BaseConstructed;
struct DeltaApplied;
struct ViewUpdated;
struct Completed;

// Generic state machine
struct PropertyBankPipeline<S> {
    data: Box<PropertyBankData>,  // Shared data
    _state: PhantomData<S>,        // Zero-sized state marker
}

// Sealed state trait (prevents external state implementations)
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Discovery {}
    impl Sealed for super::FileParsed {}
    impl Sealed for super::PropertyDelta {}
    impl Sealed for super::BaseConstructed {}
    impl Sealed for super::DeltaApplied {}
    impl Sealed for super::ViewUpdated {}
    impl Sealed for super::Completed {}
}
pub trait PropertyBankState: sealed::Sealed {}
impl<T: sealed::Sealed> PropertyBankState for T {}
```

**State-Specific Operations**:

```rust
// Discovery: Query DB and determine branch
impl PropertyBankPipeline<Discovery> {
    pub fn new(path: &Path, repo: &Repository) -> Self {
        Self {
            data: Box::new(PropertyBankData {
                path: path.to_path_buf(),
                repo: repo.clone(),
                // ... other fields
            }),
            _state: PhantomData,
        }
    }

    pub fn discover(self) -> Result<PropertyBankPath, Error> {
        let view = self.data.repo.get_raw_property_bank_view()?;

        match view {
            None => Ok(PropertyBankPath::New(/* transition to FileParsed */)),
            Some(view) => {
                if timestamps_match(&view, &self.data.path)? {
                    Ok(PropertyBankPath::Fresh(/* transition to BaseConstructed */))
                } else {
                    let content = read_file(&self.data.path)?;
                    let hash = blake3::hash(content.as_bytes());

                    if view.hashes().is_content_match(hash.as_bytes()) {
                        Ok(PropertyBankPath::FreshWithTimestampUpdate(
                            /* transition to BaseConstructed */
                        ))
                    } else {
                        Ok(PropertyBankPath::Stale(/* transition to FileParsed */))
                    }
                }
            }
        }
    }
}

// FileParsed: File read and validated in one step
impl PropertyBankPipeline<FileParsed> {
    pub fn parse_file(path: &Path) -> Result<Self, Error> {
        let content = FsReader::read_to_string(path)?;
        let raw = RawPropertyBank::try_from_str(
            &content,
            FileFormat::from_path(path)?,
            &path.to_string_lossy()
        )?;  // Parsing = validation!

        Ok(Self {
            data: Box::new(PropertyBankData { raw: Some(raw), /* ... */ }),
            _state: PhantomData,
        })
    }

    // NEW path: Build from scratch
    pub fn build_from_scratch(self) -> Result<PropertyBankPipeline<BaseConstructed>, Error> {
        let raw = self.data.raw.expect("raw exists in FileParsed");
        let bank = PropertyBank::try_from(raw)?;

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData { bank: Some(bank), /* ... */ }),
            _state: PhantomData,
        })
    }

    // STALE path: Compute delta
    pub fn compute_delta(
        self,
        cached_view: &RawPropertyBankView
    ) -> Result<PropertyBankPipeline<PropertyDelta>, Error> {
        let new_hashes = compute_property_hashes(&self.data.raw);
        let delta = cached_view.hashes().compare(&new_hashes);

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData { delta: Some(delta), /* ... */ }),
            _state: PhantomData,
        })
    }
}

// PropertyDelta: Delta computed (STALE path only)
impl PropertyBankPipeline<PropertyDelta> {
    pub fn fetch_base(self, repo: &Repository) -> Result<PropertyBankPipeline<BaseConstructed>, Error> {
        let bank = repo.get_property_bank()?.expect("bank exists for STALE");

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData { bank: Some(bank), /* ... */ }),
            _state: PhantomData,
        })
    }
}

// BaseConstructed: PropertyBank available
impl PropertyBankPipeline<BaseConstructed> {
    // STALE path: Apply delta
    pub fn apply_delta(
        mut self,
        delta: &PropertyDelta
    ) -> Result<PropertyBankPipeline<DeltaApplied>, Error> {
        let bank = self.data.bank.as_mut().expect("bank exists");
        bank.update_from_raw(&self.data.raw, &delta.changed)?;

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }

    // FRESH, FRESH+TS, NEW paths: Create/update view
    pub fn create_view(self) -> Result<PropertyBankPipeline<ViewUpdated>, Error> {
        let view = RawPropertyBankView::try_from_with_content(
            &self.data.raw,
            &self.data.content
        )?;

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData { view: Some(view), /* ... */ }),
            _state: PhantomData,
        })
    }

    // FRESH+TS path: Update timestamps only
    pub fn update_timestamps(self, view: RawPropertyBankView) -> Result<PropertyBankPipeline<ViewUpdated>, Error> {
        let mut updated_view = view.clone();
        updated_view.update_file_times(self.data.created_at, self.data.modified_at);

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData { view: Some(updated_view), /* ... */ }),
            _state: PhantomData,
        })
    }
}

// DeltaApplied: Incremental updates applied (STALE path only)
impl PropertyBankPipeline<DeltaApplied> {
    pub fn update_view(self) -> Result<PropertyBankPipeline<ViewUpdated>, Error> {
        let view = RawPropertyBankView::try_from_with_content(
            &self.data.raw,
            &self.data.content
        )?;

        Ok(PropertyBankPipeline {
            data: Box::new(PropertyBankData { view: Some(view), /* ... */ }),
            _state: PhantomData,
        })
    }
}

// ViewUpdated: View created/updated
impl PropertyBankPipeline<ViewUpdated> {
    pub fn persist(self, repo: &Repository) -> Result<PropertyBankPipeline<Completed>, Error> {
        // Persist bank (if modified)
        if self.data.bank_modified {
            repo.save_property_bank(&self.data.bank.as_ref().unwrap())?;
        }

        // Persist view (always)
        repo.save_raw_property_bank_view(&self.data.view.as_ref().unwrap())?;

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}

// Completed: Terminal state
impl PropertyBankPipeline<Completed> {
    pub fn into_bank(self) -> PropertyBank {
        self.data.bank.expect("bank exists in Completed")
    }

    pub fn bank(&self) -> &PropertyBank {
        self.data.bank.as_ref().expect("bank exists in Completed")
    }
}
```

**Branching Enum** (returned from Discovery):

```rust
enum PropertyBankPath {
    New(PropertyBankPipeline<FileParsed>),
    Fresh(PropertyBankPipeline<BaseConstructed>),
    FreshWithTimestampUpdate(PropertyBankPipeline<BaseConstructed>),
    Stale(PropertyBankPipeline<FileParsed>),
}

impl PropertyBankPath {
    pub fn into_completed(self, repo: &Repository) -> Result<PropertyBankPipeline<Completed>, Error> {
        match self {
            PropertyBankPath::New(pipeline) => {
                pipeline
                    .build_from_scratch()?
                    .create_view()?
                    .persist(repo)
            }
            PropertyBankPath::Fresh(pipeline) => {
                // Already BaseConstructed, no persistence needed
                pipeline.persist_noop()
            }
            PropertyBankPath::FreshWithTimestampUpdate(pipeline) => {
                let view = repo.get_raw_property_bank_view()?.unwrap();
                pipeline
                    .update_timestamps(view)?
                    .persist(repo)
            }
            PropertyBankPath::Stale(pipeline) => {
                let cached_view = repo.get_raw_property_bank_view()?.unwrap();
                pipeline
                    .compute_delta(&cached_view)?
                    .fetch_base(repo)?
                    .apply_delta(&pipeline.data.delta)?
                    .update_view()?
                    .persist(repo)
            }
        }
    }
}
```

**Benefits**:

- ✅ Compile-time guarantee of correct ordering
- ✅ Branch-specific paths type-safe
- ✅ Clear state transitions (can't skip stages)
- ✅ Self-documenting API
- ✅ "Validated" state eliminated (parsing = validation)
- ✅ Error types can be state-specific

**Key Improvements Over Original**:
- **7 states instead of 10** (consolidated related operations)
- **No separate "Validated" state** (prerequisite refactoring eliminates it)
- **Branching explicit** via `PropertyBankPath` enum
- **Discovery state** encapsulates staleness detection

### 4.2 Schema State Machine

**Typestate Pattern Application** (more complex):

```rust
// Primary states
struct FileList;          // Vec<PathBuf>
struct BulkStaleness;     // HashMap<Path, SchemaResult>
struct Partitioned;       // Three vectors split
struct Expanded;          // RefExpandedSchema
struct Tree;              // SchemaTree
struct Resolved;          // Vec<Schema>
struct Persisted;         // Saved to DB
struct Completed;         // Done

// Generic state machine
struct SchemaPipeline<S> {
    data: Box<SchemaData>,
    _state: PhantomData<S>,
}

// State-specific operations
impl SchemaPipeline<FileList> {
    pub fn new(paths: Vec<PathBuf>) -> Self { /* ... */ }
    pub fn check_staleness(self, repo: &Repository)
        -> Result<SchemaPipeline<BulkStaleness>, Error> { /* ... */ }
}

impl SchemaPipeline<BulkStaleness> {
    pub fn partition(self, bank_is_fresh: bool)
        -> SchemaPipeline<Partitioned> { /* ... */ }
}

impl SchemaPipeline<Partitioned> {
    pub fn expand_refs(self, bank: &PropertyBank)
        -> Result<SchemaPipeline<Expanded>, Error> { /* ... */ }

    pub fn skip_expansion_if_cached(self)
        -> Result<SchemaPipeline<Expanded>, Error> { /* ... */ }
}

impl SchemaPipeline<Expanded> {
    pub fn build_tree(self, known_parents: &HashMap<SchemaId, Schema>)
        -> Result<SchemaPipeline<Tree>, Error> { /* ... */ }
}

impl SchemaPipeline<Tree> {
    pub fn resolve(self, known_parents: &HashMap<SchemaId, Schema>)
        -> Result<SchemaPipeline<Resolved>, Error> { /* ... */ }
}

impl SchemaPipeline<Resolved> {
    pub fn persist(self, repo: &Repository)
        -> Result<SchemaPipeline<Persisted>, Error> { /* ... */ }
}

impl SchemaPipeline<Persisted> {
    pub fn complete(self) -> SchemaPipeline<Completed> { /* ... */ }
}

impl SchemaPipeline<Completed> {
    pub fn schemas(&self) -> &[Schema] { /* ... */ }
}
```

**Branch Handling**:

```rust
// Use enums for branching states
enum PartitionedSchemas {
    NeedsExpansion {
        needs_expansion: Vec<(SchemaId, RawSchema)>,
        cached_expansion: Vec<(SchemaId, HashMap<PropertyName, Property>)>,
        fresh_ids: Vec<SchemaId>,
    },
}

impl SchemaPipeline<Partitioned> {
    pub fn into_branches(self) -> PartitionedSchemas {
        // Extract three vectors
    }
}

// Process each branch through its path
```

### 4.3 Loader Orchestration with State Machines

**Before** (current code in `loader.rs:136-271`):

```rust
pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
    let results = self.ingestor.ingest_all()?;
    let bank = results.property_bank.bank();
    let bank_is_fresh = results.property_bank.is_fresh();

    // Complex partitioning logic (50 lines)
    let mut needs_expansion = Vec::new();
    let mut cached_expansion = Vec::new();
    let mut fresh_ids = Vec::new();
    // ... partitioning ...

    // Load known parents
    let parent_schemas = /* ... */;

    // Process needs_expansion
    if !needs_expansion.is_empty() {
        let expanded = RefExpander::new(bank).expand_all(needs_expansion.clone())?;
        self.store_expanded_properties(&expanded)?;
        let tree = Extender::build(expanded, &known_parents)?;
        let full_resolved = Merger::resolve(&tree, &known_parents)?;
        resolved.extend(full_resolved);
    }

    // Process cached_expansion
    if !cached_expansion.is_empty() {
        let cached_resolved = self.resolve_with_cached_expansion(/* ... */)?;
        resolved.extend(cached_resolved);
    }

    // Persist
    if !resolved.is_empty() {
        self.persist_resolved_schemas(&resolved)?;
        // ...
    }

    Ok(resolved)
}
```

**After** (with state machines):

```rust
pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
    // PropertyBank pipeline
    let bank_pipeline = PropertyBankPipeline::new(&bank_path)
        .read_file()?
        .parse()?
        .validate()?
        .to_domain()?
        .persist(&self.repository)?
        .complete();
    let bank = bank_pipeline.bank();

    // Schema pipeline
    let schema_pipeline = SchemaPipeline::new(file_list)
        .check_staleness(&self.repository)?
        .partition(bank_pipeline.is_fresh());

    // Branch based on partition results
    let branches = schema_pipeline.into_branches();

    // Process needs_expansion branch
    let expanded = SchemaPipeline::from_needs_expansion(branches.needs_expansion)
        .expand_refs(bank)?
        .build_tree(&known_parents)?
        .resolve(&known_parents)?
        .persist(&self.repository)?
        .complete();

    // Process cached_expansion branch
    let cached = SchemaPipeline::from_cached_expansion(branches.cached_expansion)
        .build_tree(&known_parents)?  // Skip expand_refs!
        .resolve(&known_parents)?
        .persist(&self.repository)?
        .complete();

    // Combine results
    let mut all_schemas = Vec::new();
    all_schemas.extend(expanded.schemas());
    all_schemas.extend(cached.schemas());

    Ok(all_schemas)
}
```

**Benefits**:

- **Clear state progression**: Each step is explicit
- **Compile-time safety**: Can't skip stages or call operations in wrong order
- **Self-documenting**: Type signatures show the pipeline
- **Testable**: Each state transition can be unit tested independently
- **Maintainable**: Adding new states/transitions is straightforward

---

## 5. Critical Implementation Details

### 5.1 Data Ownership in State Machines

**Challenge**: State transitions consume `self`, but we need to access data across states.

**Solution**: Store shared data in `Box<T>` or `Arc<T>`, carry it through transitions:

```rust
struct PropertyBankData {
    path: PathBuf,
    config: Config,
    repository: Repository,
    // Mutable data fields
    raw_content: Option<String>,
    raw_bank: Option<RawPropertyBank>,
    validated_bank: Option<RawPropertyBank>,
    domain_bank: Option<PropertyBank>,
}

struct PropertyBankPipeline<S> {
    data: Box<PropertyBankData>,
    _state: PhantomData<S>,
}

impl PropertyBankPipeline<Unloaded> {
    pub fn read_file(mut self) -> Result<PropertyBankPipeline<RawFile>, Error> {
        let content = std::fs::read_to_string(&self.data.path)?;
        self.data.raw_content = Some(content);

        Ok(PropertyBankPipeline {
            data: self.data,  // Move data to new state
            _state: PhantomData,
        })
    }
}
```

**Alternative**: Use `Option<T>` fields and `take()` to move data between states:

```rust
impl PropertyBankPipeline<RawFile> {
    pub fn parse(mut self) -> Result<PropertyBankPipeline<Parsed>, Error> {
        let content = self.data.raw_content.take()
            .expect("raw_content should exist in RawFile state");

        let raw_bank: RawPropertyBank = parse_content(&content)?;
        self.data.raw_bank = Some(raw_bank);

        Ok(PropertyBankPipeline {
            data: self.data,
            _state: PhantomData,
        })
    }
}
```

### 5.2 Error Handling in State Machines

**Approach 1**: State-specific error types

```rust
pub enum ReadFileError {
    Io(std::io::Error),
    InvalidPath(PathBuf),
}

pub enum ParseError {
    Json(serde_json::Error),
    Toml(toml::de::Error),
    Yaml(serde_yaml::Error),
}

impl PropertyBankPipeline<Unloaded> {
    pub fn read_file(self) -> Result<PropertyBankPipeline<RawFile>, ReadFileError> { /* ... */ }
}

impl PropertyBankPipeline<RawFile> {
    pub fn parse(self) -> Result<PropertyBankPipeline<Parsed>, ParseError> { /* ... */ }
}
```

**Approach 2**: Unified error type with context

```rust
pub enum PropertyBankError {
    ReadFile { source: std::io::Error, path: PathBuf },
    Parse { source: Box<dyn Error>, format: FileFormat },
    Validation { source: SchemaError },
    Domain { source: SchemaError },
    Storage { source: SchemaRepositoryError },
}

impl PropertyBankPipeline<Unloaded> {
    pub fn read_file(self) -> Result<PropertyBankPipeline<RawFile>, PropertyBankError> { /* ... */ }
}
```

**Recommendation**: Use Approach 2 (unified error) with `From` conversions for ergonomics.

### 5.3 Testing State Machines

**Unit Test Each Transition**:

```rust
#[test]
fn pipeline_read_file_success() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("property_bank.json");
    std::fs::write(&path, r#"{"properties": {}}"#).unwrap();

    let pipeline = PropertyBankPipeline::<Unloaded>::new(&path);
    let result = pipeline.read_file();

    assert!(result.is_ok());
    // Can call methods on PropertyBankPipeline<RawFile>
}

#[test]
fn pipeline_read_file_failure() {
    let path = PathBuf::from("/nonexistent/path");
    let pipeline = PropertyBankPipeline::<Unloaded>::new(&path);
    let result = pipeline.read_file();

    assert!(result.is_err());
}
```

**Integration Test Full Pipeline**:

```rust
#[test]
fn property_bank_full_pipeline() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("property_bank.json");
    std::fs::write(&path, r#"{"$version": "1.0", "properties": {"title": {"type": "string"}}}"#).unwrap();

    let repo = InMemoryRepository::new();
    let result = PropertyBankPipeline::<Unloaded>::new(&path)
        .read_file().unwrap()
        .parse().unwrap()
        .validate().unwrap()
        .to_domain().unwrap()
        .persist(&repo).unwrap()
        .complete();

    assert_eq!(result.bank().all().count(), 1);
}
```

### 5.4 Staleness Detection Integration

**Option 1**: Separate staleness state machine that produces initial state

```rust
enum PropertyBankInitialState {
    Fresh(PropertyBank),                        // Skip pipeline entirely
    NeedsProcessing(PropertyBankPipeline<Unloaded>),
}

fn detect_staleness(path: &Path, repo: &Repository)
    -> Result<PropertyBankInitialState, Error>
{
    let view = repo.get_raw_property_bank_view()?;

    match view {
        None => Ok(PropertyBankInitialState::NeedsProcessing(
            PropertyBankPipeline::new(path)
        )),
        Some(view) => {
            if view.is_fresh(path)? {
                let bank = repo.get_property_bank()?.unwrap();
                Ok(PropertyBankInitialState::Fresh(bank))
            } else {
                Ok(PropertyBankInitialState::NeedsProcessing(
                    PropertyBankPipeline::new(path)
                ))
            }
        }
    }
}
```

**Option 2**: Make staleness checking part of the state machine

```rust
impl PropertyBankPipeline<Unloaded> {
    pub fn check_staleness(self, repo: &Repository)
        -> Result<PropertyBankStaleness, Error>
    {
        // Return enum indicating Fresh vs NeedsProcessing
    }
}

enum PropertyBankStaleness {
    Fresh(PropertyBank),
    NeedsProcessing(PropertyBankPipeline<Unloaded>),
}
```

**Recommendation**: Option 1 (separate function) for clarity and testability.

### 5.5 Wrapping State Machines in Enums

When storing in parent structs, wrap in enum:

```rust
enum PropertyBankStatus {
    Unloaded(PropertyBankPipeline<Unloaded>),
    RawFile(PropertyBankPipeline<RawFile>),
    Parsed(PropertyBankPipeline<Parsed>),
    Validated(PropertyBankPipeline<Validated>),
    Domain(PropertyBankPipeline<Domain>),
    Persisted(PropertyBankPipeline<Persisted>),
    Completed(PropertyBankPipeline<Completed>),
}

impl PropertyBankStatus {
    pub fn advance(self) -> Result<Self, Error> {
        match self {
            PropertyBankStatus::Unloaded(p) => {
                Ok(PropertyBankStatus::RawFile(p.read_file()?))
            }
            PropertyBankStatus::RawFile(p) => {
                Ok(PropertyBankStatus::Parsed(p.parse()?))
            }
            // ... other transitions
        }
    }
}
```

This allows step-by-step execution while maintaining type safety.

---

## 6. Migration Strategy

### 6.1 Incremental Refactoring Plan

**Phase 0: Prerequisite - "Parse, Don't Validate"** (Week 1: 2-3 hours)

1. Make `RawPropertyBank` fields private
2. Add `RawPropertyBank::try_from_str()` parsing constructor
3. Add public accessor methods (`version()`, `properties()`, `metadata()`)
4. Remove `validate()`, `validate_version()`, `validated()` methods
5. Update `Ingestor` to use `try_from_str()`
6. Repeat for `RawSchema`
7. Run full test suite to ensure no regressions
8. **Deliverable**: No "Validated" state needed in state machine

**Phase 1: PropertyBank State Machine** (Weeks 1-2)

1. Create `PropertyBankPipeline<S>` with 7 consolidated states
2. Implement `Discovery` state with branching logic
3. Implement `PropertyBankPath` enum for branch handling
4. Implement each branch path:
   - NEW: FileParsed → BaseConstructed → ViewUpdated → Completed
   - FRESH: BaseConstructed → Completed (no persistence)
   - FRESH+TS: BaseConstructed → ViewUpdated → Completed
   - STALE: FileParsed → PropertyDelta → BaseConstructed → DeltaApplied → ViewUpdated → Completed
5. Move existing logic from `Ingestor::property_bank()` into state transitions
6. Update `Loader` to use new state machine
7. Add unit tests for each state transition
8. Add integration tests for all four branch paths
9. **Deliverable**: Fully functional PropertyBank state machine

**Phase 2: Schema State Machine** (Weeks 3-5)

1. Create `SchemaPipeline<S>` with sealed states
2. Implement linear stages (FileList → BulkStaleness → Partitioned)
3. Implement branching logic for partitioned state
4. Move existing logic from `Loader::load()` into state transitions
5. Refactor `RefExpander`, `Extender`, `Merger` to work with state machine
6. Add unit tests for each transition
7. Add integration tests for all branches
8. **Deliverable**: Fully functional Schema state machine

**Phase 3: Error Handling & Polish** (Week 6)

1. Consolidate error types (state-specific vs unified)
2. Improve error messages with state context
3. Add extensive rustdoc documentation
4. Performance benchmarking (ensure no regressions)
5. Update AGENTS.md with new patterns
6. Create ADR documenting the refactoring decision
7. **Deliverable**: Production-ready state machines with docs

### 6.2 Backward Compatibility

**Keep Existing API**: Wrap state machine in facade

```rust
impl Loader<'_, R> {
    pub fn load(&self) -> Result<Vec<Schema>, SchemaLoaderError> {
        // Internal: Use state machines
        let bank = self.load_property_bank_internal()?;
        let schemas = self.load_schemas_internal(&bank)?;
        Ok(schemas)
    }

    fn load_property_bank_internal(&self)
        -> Result<PropertyBank, SchemaLoaderError>
    {
        let path = self.config.paths().property_bank_path();
        let repo = self.ingestor.repository();

        // Discovery phase
        let pipeline = PropertyBankPipeline::<Discovery>::new(&path, repo);
        let branch = pipeline.discover()?;

        // Process based on branch
        let completed = branch.into_completed(repo)?;

        Ok(completed.into_bank())
    }

    fn load_schemas_internal(&self, bank: &PropertyBank)
        -> Result<Vec<Schema>, SchemaLoaderError>
    {
        // Schema state machine logic
        // ...
    }
}
```

**External API unchanged**, internal refactored to use state machines.

### 6.3 Testing Strategy

**Preserve All Existing Tests**: Ensure no regressions

**Add New Tests**:

1. **Per-state unit tests**: Each transition in isolation
2. **Invalid transition tests**: Verify compile-time errors
3. **Branch coverage**: All paths through partitioned state
4. **Error propagation**: Errors bubble up correctly with context
5. **Staleness integration**: Cached paths work correctly
6. **Performance benchmarks**: No slowdowns from abstraction

---

## 7. Open Questions & Next Steps

### 7.1 Questions for Discussion

1. **Granularity**: Should we have more fine-grained states (e.g., separate "Deserialized" and "Validated" states)?
2. **Shared Data**: What's the best way to pass `Repository` and `Config` through the pipeline?
3. **Staleness**: Should staleness detection be part of the state machine or a separate pre-check?
4. **Error Handling**: State-specific errors vs unified error type?
5. **Testability**: How to test state transitions without full integration tests?
6. **Performance**: Any concerns about the overhead of state machines?
7. **Documentation**: How to document the state machine pattern in rustdoc?

### 7.2 Next Steps

**Immediate**:

1. **Review this document**: Confirm understanding of all pipeline stages
2. **Decide on state machine design**: PropertyBank vs Schema, granularity, error handling
3. **Prototype**: Implement PropertyBank state machine as proof-of-concept
4. **Validate**: Ensure no regressions, measure performance

**Follow-up**:

1. Implement Schema state machine
2. Refactor Loader orchestration
3. Update documentation
4. Add comprehensive tests
5. Create ADR documenting the refactoring decision

---

## Appendices

### A. File Locations

**Pipeline Orchestration**:

- `loader.rs` - Main orchestration (lines 1-1006)
- `ingestor.rs` - File I/O and staleness detection (lines 1-1466+)

**Pipeline Stages**:

- `expander.rs` - Property reference expansion (lines 1-442)
- `extender.rs` - Inheritance tree building (lines 1-855)
- `merger.rs` - Property merging (lines 1-879)
- `resolver.rs` - Property conflict resolution (lines 1-759)

**Domain Types**:

- `aggregate.rs` - Schema aggregate (lines 1-553)
- `bank.rs` - PropertyBank aggregate (lines 1-959)
- `property.rs` - Property domain types
- `raw/mod.rs` - Raw input types (lines 1-617)

**Views & Metadata**:

- `views/raw.rs` - RawPropertyBankView, RawSchemaView
- `views/metadata.rs` - Hash and timestamp metadata
- `views/inheritance.rs` - SchemaInheritanceView

**Storage**:

- `storage.rs` - Repository trait and implementations

### B. Dependency Graph

```
┌──────────────────────────────────────────────────────┐
│                       Loader                         │
│                   (orchestration)                    │
└────────┬─────────────────────────────────────────────┘
         │
         ├─── Ingestor (file I/O + staleness)
         │      │
         │      ├─── FsReader (filesystem abstraction)
         │      └─── Repository (database operations)
         │
         ├─── RefExpander (property reference expansion)
         │      │
         │      └─── PropertyBank (lookup)
         │
         ├─── Extender (inheritance tree building)
         │      │
         │      └─── SchemaTree (topological order)
         │
         └─── Merger (property merging)
                │
                └─── Resolver (conflict resolution)
```

### C. Key Constants

- `SCHEMA_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml"]`
- `INHERITANCE_MAX_DEPTH: usize = 10`
- `PROPERTY_BANK_KEY: &str = "singleton"`
- `RAW_PROPERTY_BANK_KEY: &str = "property-bank"`

### D. Glossary

- **RefExpanded**: Schema with `$ref` pointers resolved to concrete `Property` instances
- **SchemaTree**: Topologically ordered inheritance tree (parents before children)
- **Known Parents**: Fresh schemas loaded from DB, used as parent references
- **Staleness**: Whether file content has changed since last load (timestamp + hash based)
- **Incremental Resolution**: Updating only changed properties instead of full re-resolution
- **Cached Expansion**: Reusing previously expanded properties when only PropertyBank changed

---

**End of Document**

_Total Pipeline Stages Identified_:

- **PropertyBank**: 7 distinct states
- **Schema**: 10 primary states (with 3 branch paths)
- **Cross-cutting**: 2 staleness detection strategies, 5 error hierarchies

_Confidence Level_: **High** - Based on thorough code review of 15+ source files and 8000+ lines of code.
