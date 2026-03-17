# Loader ↔ Ingestor Architecture Review

**Date**: 2026-03-17
**Status**: Analysis Complete - Awaiting Refactoring Plan
**Reviewers**: Critical architectural review of schema loading pipeline

---

## Executive Summary

Critical analysis of `loader.rs` and `ingestor.rs` revealed fundamental architectural inefficiencies:
1. **Double-loop anti-pattern**: Iterating through schemas twice
2. **N+1 query pattern**: Per-file database queries instead of bulk operations
3. **Missing incremental resolution**: Re-expanding all schemas on every property change
4. **Inefficient caching**: Storing compressed strings instead of serialized structs
5. **Unclear abstraction boundaries**: Duplicate metadata across Raw* and Raw*View types

---

## Current Architecture Problems

### Problem 1: PropertyBank Construction Over-Complexity

**Current Flow:**
```rust
// Ingestor
property_bank() -> IngestResult<RawPropertyBank>  // Returns RAW type

// Loader
match result {
    Fresh(raw) => PropertyBank::try_from(raw)?,  // Parse AGAIN from raw
    Stale(raw) => {
        let changed = compare_with_view();  // Extra step
        let bank = PropertyBank::try_from(raw)?;
        repository.save_property_bank(&bank)?;
    }
}
```

**Issues:**
- PropertyBank already exists in DB (resolved domain type)
- Going backwards from PropertyBank → RawPropertyBank → PropertyBank is wasteful
- Not using `PropertyBank.update_properties()` for incremental updates

**Better Flow:**
```rust
if fresh {
    bank = repository.get_property_bank()?;  // Direct load
} else {
    bank = repository.get_property_bank()?.unwrap_or_default();
    let raw = parse_file();
    let changed = view.changed_properties(&raw.metadata);
    bank.update_properties(&raw.properties)?;  // Incremental!
}
```

---

### Problem 2: Duplicate SchemaId Lookups

**Current:**
```rust
// Ingestor.schema() - Line 375
let schema_id = self.repository
    .find_schema_id_by_path(&rel_path)?;  // Query #1

// Loader.load() - Line 111
let name_to_id = self.load_name_to_id_map()?;  // Query #2 (bulk)
```

**Issue:**
RawSchemaView already links path → SchemaId, making `load_name_to_id_map()` redundant.

**Analysis:**
- RawSchemaView uses `file_path` as key
- When we `find_raw_schema_view_by_path()`, we get SchemaId
- SchemaName derived from filename
- The name_to_id map duplicates what's in the views

---

### Problem 3: Inefficient Content Storage

**Current Architecture:**
```rust
pub struct RawFileVersion {
    compressed_content: Option<Vec<u8>>,  // Compressed STRING (JSON/TOML)
    content_hash: [u8; 32],
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
}

// To use cached content:
let content = version.decompress_content()?;  // String
let raw = parse_from_str(&content)?;          // Parse AGAIN!
```

**Issues:**
1. Storing compressed JSON/TOML string instead of serialized struct
2. Must decompress → parse to get `RawSchema`
3. Parsing overhead even when checking freshness by content hash
4. Metadata duplication: `RawFileVersion` duplicates `RawSchemaMetadata`

**Comparison:**

| Approach | Storage | To Use | Performance |
|----------|---------|--------|-------------|
| Current | `zstd(json_string)` | decompress → parse → RawSchema | 2 steps (decompress + parse) |
| Better | `zstd(rkyv(RawSchema))` | decompress → deserialize → RawSchema | 2 steps (decompress + deserialize) |
| Best | `rkyv(RawSchema)` | access_archived → &ArchivedRawSchema | 0-copy! |

**Metadata Duplication Analysis:**
```rust
// RawFileVersion fields:
content_hash: [u8; 32],        // ← Duplicates RawSchemaMetadata.content_hash
property_hashes: BTreeMap,     // ← Duplicates RawSchemaMetadata.property_hashes
created_at: SystemTime,        // ← Duplicates RawSchemaMetadata.created_at
modified_at: SystemTime,       // ← Duplicates RawSchemaMetadata.modified_at

// RawSchema already has:
metadata: RawSchemaMetadata {
    content_hash: Option<[u8; 32]>,
    property_hashes: HashMap<Box<str>, [u8; 32]>,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
}
```

---

### Problem 4: Missing Incremental Resolution

**Current Pipeline:**
```
RawSchema (file)
  ↓ [RefExpander - EXPENSIVE!]
ExpandedSchema (properties fully specified) ← THROWN AWAY
  ↓ [Resolver - inheritance merge]
Schema (final resolved)
```

**What We Store:**
- ✅ RawSchema (in Raw*View as compressed content)
- ❌ ExpandedSchema (not stored - always recomputed!)
- ✅ Schema (final resolved type)

**Impact:**
```rust
// Every load, even if PropertyBank is fresh:
for schema in all_schemas {
    RefExpander.expand(schema, &bank)?;  // Re-expand EVERYTHING
}
```

**Better Approach:**
```rust
// If PropertyBank fresh:
for schema in all_schemas {
    let expanded = repository.get_expanded_schema(id)?;  // Load from cache!
    // Skip expansion entirely
}

// If PropertyBank changed:
let changed_props = bank_result.changed_properties();
for schema in schemas_using(changed_props) {
    RefExpander.expand(schema, &bank)?;  // Only re-expand affected schemas
}
```

**Benefit:** Massive performance improvement when PropertyBank is fresh (common case).

---

### Problem 5: Double-Loop Anti-Pattern

**Current:**
```rust
// LOOP #1: In ingestor.all_schemas()
for ext in SCHEMA_EXTENSIONS {
    for path in files {
        let result = self.schema(&path)?;  // Individual processing
        results.push(result);              // Flat list
    }
}

// LOOP #2: In loader.load()
for result in raw_schema_results {  // Loop over same data AGAIN!
    match result {
        IngestResult::Fresh(raw) => {
            let name = SchemaName::try_new(&raw.name)?;
            if let Some(id) = name_to_id.get(&name) {
                fresh_ids.push(*id);  // Partition
            }
        }
        IngestResult::Stale(raw) => {
            let name = SchemaName::try_new(&raw.name)?;
            let id = name_to_id.get(&name).copied()
                .unwrap_or_else(SchemaId::new);
            stale_schemas.push((id, raw));  // Partition
        }
    }
}
```

**Issue:** Partitioning should happen during ingestion, not after.

---

### Problem 6: N+1 Database Query Pattern

**Current (in `ingestor.schema()` - called for EACH file):**
```rust
// Query #1: Get view
let cached_view = self.repository
    .find_raw_schema_view_by_path(&rel_path)?;

// ... staleness check ...

// Query #2: Get schema ID
let schema_id = self.repository
    .find_schema_id_by_path(&rel_path)?;

// Multiply by N schemas = 2N queries!
```

**Better (bulk queries upfront):**
```rust
pub fn all_schemas(&self) -> Result<...> {
    let paths = list_all_schema_files();

    // SINGLE bulk query for all views
    let views: HashMap<PathBuf, RawSchemaView> =
        self.repository.find_raw_schema_views_by_paths(&paths)?;

    // SINGLE bulk query for all schema IDs
    let ids: HashMap<PathBuf, SchemaId> =
        self.repository.find_schema_ids_by_paths(&paths)?;

    // Now process with no DB queries per file
    for path in paths {
        let view = views.get(&path);
        let id = ids.get(&path);
        // ... process ...
    }
}
```

---

## Agreed-Upon Redesign Decisions

### Decision 1: Keep Raw*View Types with Enhancements

**Keep the types** but enhance them significantly:

#### 1.1: Split Version Types and Store Serialized Raw* Types

**Current Issue:** `RawFileVersion` mixes multiple concerns (hashes, times, cached content)

**Solution:** Split into three specialized version types:

**Type 1: File Metadata (shared)**
```rust
/// File timestamp metadata - shared by both schema and property bank views
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct FileVersionMetadata {
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
    recorded_at: SystemTime,  // When this version was recorded in DB
}
```

**Type 2: Schema Version (specific to RawSchemaView)**
```rust
/// A single version of a schema file with cached data
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// File timestamp metadata
    metadata: FileVersionMetadata,

    /// Content hash for staleness detection
    content_hash: [u8; 32],

    /// Per-property hashes for incremental resolution
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,

    /// Serialized RawSchema (optionally compressed)
    /// Format: rkyv(RawSchema) or zstd(rkyv(RawSchema))
    archived_schema: Vec<u8>,

    /// Cached expanded properties (from RefExpander)
    /// Enables skipping expansion when PropertyBank is fresh
    expanded_properties: Option<HashMap<PropertyName, Property>>,
}
```

**Type 3: PropertyBank Version (specific to RawPropertyBankView)**
```rust
/// A single version of the property bank file with cached data
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct PropertyBankVersion {
    /// File timestamp metadata
    metadata: FileVersionMetadata,

    /// Content hash for staleness detection
    content_hash: [u8; 32],

    /// Per-property hashes for incremental updates
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,

    /// Serialized RawPropertyBank (optionally compressed)
    /// Format: rkyv(RawPropertyBank) or zstd(rkyv(RawPropertyBank))
    archived_property_bank: Vec<u8>,
}
```

**Updated View Types:**
```rust
pub struct RawSchemaView {
    file_path: Box<str>,
    extends: Option<SchemaName>,
    excludes: Vec<PropertyName>,

    /// Version history (ring buffer, max 5 versions, newest first)
    versions: VecDeque<SchemaVersion>,  // Changed from RawFileVersion
}

pub struct RawPropertyBankView {
    /// Version history (ring buffer, max 5 versions, newest first)
    versions: VecDeque<PropertyBankVersion>,  // Changed from RawFileVersion
}
```

**Benefits:**
- Each version type has exactly what it needs (no unused fields)
- Serialized structs stored IN the version (versioned properly)
- Expanded properties stored IN SchemaVersion (versioned with the schema)
- Property hashes stored IN each version for staleness detection
- Clear separation of concerns

#### 1.2: Remove RawSchemaMetadata

**Current Problem:**
```rust
// Duplication - these are in BOTH places:
pub struct RawSchemaMetadata {
    content_hash: Option<[u8; 32]>,        // Also in SchemaVersion
    property_hashes: HashMap<Box<str>, [u8; 32]>,  // Also in SchemaVersion
    created_at: Option<SystemTime>,        // Also in FileVersionMetadata
    modified_at: Option<SystemTime>,       // Also in FileVersionMetadata
}

pub struct RawSchema {
    name: Box<str>,
    extends: Option<Box<str>>,
    excludes: Vec<Box<str>>,
    properties: HashMap<Box<str>, RawProperty>,
    metadata: RawSchemaMetadata,  // ← REDUNDANT!
}
```

**Solution: Remove RawSchemaMetadata entirely**
```rust
pub struct RawSchema {
    name: Box<str>,
    extends: Option<Box<str>>,
    excludes: Vec<Box<str>>,
    properties: HashMap<Box<str>, RawProperty>,
    // NO metadata field - it's all in SchemaVersion now!
}

pub struct RawPropertyBank {
    properties: HashMap<Box<str>, RawProperty>,
    // NO metadata field - it's all in PropertyBankVersion now!
}
```

**Why This Works:**
- File timestamps → `FileVersionMetadata` (in version)
- Content hash → `content_hash` field (in version)
- Property hashes → `property_hashes` field (in version)
- RawSchema/RawPropertyBank are now pure data structures

**Migration Path:**
When parsing a file, compute metadata on the fly and store in the version:
```rust
// Parse file
let raw_schema: RawSchema = parse_file(content)?;

// Compute metadata for version
let content_hash = blake3::hash(content.as_bytes());
let property_hashes = compute_property_hashes(&raw_schema.properties);

// Create version with metadata
let version = SchemaVersion {
    metadata: FileVersionMetadata {
        created_at: file.created_at,
        modified_at: file.modified_at,
        recorded_at: SystemTime::now(),
    },
    content_hash: *content_hash.as_bytes(),
    property_hashes,
    archived_schema: rkyv::to_bytes(&raw_schema)?,
    expanded_properties: None,  // Will be filled after expansion
};
```

#### 1.3: Add Helper Method for Property Hash Computation

**Add to SchemaVersion and PropertyBankVersion:**
```rust
impl SchemaVersion {
    /// Compute property hashes from RawSchema properties
    pub fn compute_property_hashes(
        properties: &HashMap<Box<str>, RawProperty>
    ) -> BTreeMap<PropertyName, [u8; 32]> {
        properties
            .iter()
            .filter_map(|(name, prop)| {
                let hash = Self::hash_property(prop);
                PropertyName::try_new(name.as_ref())
                    .ok()
                    .map(|pn| (pn, hash))
            })
            .collect()
    }

    fn hash_property(prop: &RawProperty) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        // Hash the property definition
        hasher.update(prop.spec.as_bytes());
        hasher.update(&[prop.multi as u8]);
        *hasher.finalize().as_bytes()
    }
}
```

**Benefits:**
- Single source of truth for hash computation
- Reusable across schema and property bank
- No duplication of hash logic

---

### Decision 2: Store Expanded Properties in RawSchemaView

**As covered in Decision 1.2 above.**

**Reconstruction Logic:**
```rust
impl RawSchemaView {
    pub fn to_ref_expanded_schema(&self) -> Option<RefExpandedSchema> {
        let expanded_props = self.expanded_properties.as_ref()?;

        Some(RefExpandedSchema {
            id: self.schema_id(),  // Derived or stored
            name: self.schema_name(),  // From file_path
            file_path: self.file_path.clone(),
            extends: self.extends.clone(),
            excludes: self.excludes.clone(),
            properties: expanded_props.clone(),
        })
    }
}
```

**Usage in Loader:**
```rust
// If PropertyBank is fresh:
let expanded_schemas: Vec<RefExpandedSchema> = schema_views
    .iter()
    .filter_map(|view| view.to_ref_expanded_schema())
    .collect();

// Skip RefExpander entirely!
let tree = Extender::build(expanded_schemas, &known_parents)?;
let resolved = Resolver::resolve(&tree, &known_parents)?;
```

---

### Decision 3: PropertyBank Incremental Updates

#### 3.1: Return PropertyBank Directly
```rust
pub enum PropertyBankResult {
    New(PropertyBank),    // First time seeing property bank
    Fresh(PropertyBank),  // File unchanged - loaded from DB
    Stale {
        bank: PropertyBank,           // Updated incrementally
        changed: Vec<PropertyName>,   // What changed
    },
}
```

**Three Paths:**

**Path 1: New (no previous property bank)**
```rust
impl PropertyBank {
    pub fn from_raw(raw: RawPropertyBank) -> Result<Self, Error> {
        // Full conversion (existing TryFrom logic)
        Self::try_from(raw)
    }
}

// Usage:
let raw = parse_property_bank_file()?;
PropertyBankResult::New(PropertyBank::from_raw(raw)?)
```

**Path 2: Fresh (file unchanged)**
```rust
// Check timestamps/hash match cached RawPropertyBankView
if view.is_fresh(&file_metadata) {
    let bank = repository.get_property_bank()?
        .expect("PropertyBank must exist if view exists");
    return Ok(PropertyBankResult::Fresh(bank));
}
```

**Path 3: Stale (file changed)**
```rust
impl PropertyBank {
    /// Update properties incrementally using changed properties from raw bank.
    pub fn update_from_raw(
        &mut self,
        raw: &RawPropertyBank,
        changed: &[PropertyName],
    ) -> Result<(), Error> {
        // Use existing update_properties() method
        let updates: Vec<_> = changed
            .iter()
            .filter_map(|name| {
                raw.properties.get(name.as_ref())
                    .map(|raw_prop| (name.clone(), raw_prop.clone()))
            })
            .collect();

        self.update_properties(&updates)
    }
}

// Usage:
let mut bank = repository.get_property_bank()?.unwrap_or_default();
let raw = parse_property_bank_file()?;
let changed = compute_changed_properties(&cached_view, &raw);
bank.update_from_raw(&raw, &changed)?;
PropertyBankResult::Stale { bank, changed }
```

**Question:** Do we need the `New` variant or can we treat it as `Stale` with all properties changed?

**Answer:** Keep `New` for semantic clarity:
- `New`: First load, no incremental resolution possible
- `Stale`: Incremental update, can track what changed
- `Fresh`: No changes, skip everything

---

### Decision 4: Compress Serialized Structs

**Store in RawSchemaView:**
```rust
pub struct RawSchemaView {
    // ...

    /// Serialized RawSchema (optionally compressed)
    ///
    /// Storage format options:
    /// 1. rkyv(RawSchema) - Fast, zero-copy deserialize
    /// 2. zstd(rkyv(RawSchema)) - Smaller, needs decompress
    ///
    /// Trade-off: Speed vs disk space
    archived_schema: Option<Vec<u8>>,
}

impl RawSchemaView {
    /// Deserialize cached RawSchema.
    pub fn to_raw(&self) -> Option<RawSchema> {
        let bytes = self.archived_schema.as_ref()?;

        // Option 1: Direct rkyv deserialize (if uncompressed)
        rkyv::from_bytes(bytes).ok()

        // Option 2: Decompress then deserialize (if compressed)
        // let decompressed = zstd::decode_all(bytes.as_slice()).ok()?;
        // rkyv::from_bytes(&decompressed).ok()
    }
}
```

**Compression Benchmark Needed:**
```
Test with real schemas:
1. Size: rkyv vs zstd(rkyv) vs zstd(json)
2. Speed: deserialize vs decompress+deserialize vs decompress+parse
3. Memory: zero-copy rkyv vs allocated decompressed buffer
```

**Hypothesis:**
- `rkyv`: Fastest (zero-copy), larger size
- `zstd(rkyv)`: Good compression, fast deserialize (faster than parsing)
- `zstd(json)`: Best compression, slowest (parse overhead)

---

### Decision 5: Structured IngestorResults

```rust
pub struct IngestorResults {
    pub property_bank: PropertyBankResult,
    pub schemas: HashMap<PathBuf, SchemaIngestResult>,
}

pub enum SchemaIngestResult {
    Fresh {
        id: SchemaId,
        expanded: Option<RefExpandedSchema>,  // If available from view
    },
    Stale {
        id: SchemaId,
        raw: RawSchema,
        expanded: Option<RefExpandedSchema>,  // If available from view
    },
}

impl Ingestor {
    pub fn ingest_all(&self) -> Result<IngestorResults, Error> {
        let property_bank_result = self.ingest_property_bank()?;

        let paths = self.list_all_schema_files()?;

        // Bulk queries upfront
        let views = self.repository.find_raw_schema_views_by_paths(&paths)?;
        let ids = self.repository.find_schema_ids_by_paths(&paths)?;

        let mut schemas = HashMap::new();

        for path in paths {
            let view = views.get(&path);
            let id = ids.get(&path).copied().unwrap_or_else(SchemaId::new);

            let result = self.process_schema_with_view(&path, id, view)?;
            schemas.insert(path, result);
        }

        Ok(IngestorResults {
            property_bank: property_bank_result,
            schemas,
        })
    }
}
```

**Benefits:**
1. **No double loop**: Partitioning happens during ingestion
2. **Clear structure**: HashMap makes Fresh/Stale lookups O(1)
3. **Bulk operations**: All DB queries upfront (no N+1)
4. **Type safety**: Can't accidentally loop twice

**Loader Usage:**
```rust
let results = ingestor.ingest_all()?;

let bank = match results.property_bank {
    PropertyBankResult::Fresh(bank) => bank,
    PropertyBankResult::Stale { bank, .. } => bank,
    PropertyBankResult::New(bank) => bank,
};

// Partition schemas (O(N) single pass)
let mut fresh_with_expanded = Vec::new();
let mut stale_to_resolve = Vec::new();

for (path, result) in results.schemas {
    match result {
        SchemaIngestResult::Fresh { id, expanded: Some(exp) } => {
            fresh_with_expanded.push(exp);  // Can skip RefExpander!
        }
        SchemaIngestResult::Stale { id, raw, .. } => {
            stale_to_resolve.push((id, raw));
        }
        // ... handle other cases
    }
}

// Resolution pipeline (only on stale schemas)
let expanded = RefExpander::new(&bank).expand_all(stale_to_resolve)?;
// ... rest of pipeline
```

---

## Proposed Refactoring Phases

### Phase 1: Raw*View Structure Changes
**Goal:** Add serialized storage and expanded properties

**Changes:**
1. Add `archived_schema` field to `RawSchemaView`
2. Add `archived_property_bank` field to `RawPropertyBankView`
3. Add `expanded_properties` field to `RawSchemaView`
4. Add `to_property_hashes()` helper to `RawFileVersion`
5. Remove metadata duplication from `RawFileVersion`

**Tests:**
- Serialization round-trip tests
- Compression benchmarks
- Expanded properties reconstruction

---

### Phase 2: PropertyBank Incremental Updates
**Goal:** Return PropertyBank directly with incremental updates

**Changes:**
1. Add `PropertyBankResult` enum
2. Add `PropertyBank::update_from_raw()` method
3. Update `Ingestor::property_bank()` to return `PropertyBankResult`
4. Update Loader to handle three paths (New/Fresh/Stale)

**Tests:**
- New property bank creation
- Fresh property bank load from DB
- Stale property bank incremental update
- Changed properties tracking

---

### Phase 3: Structured IngestorResults
**Goal:** Eliminate double-loop and N+1 queries

**Changes:**
1. Add `IngestorResults` struct
2. Add bulk query methods to Repository trait
3. Update `Ingestor::ingest_all()` to use bulk queries
4. Update Loader to consume structured results

**Tests:**
- Bulk query correctness
- Single-pass partitioning
- Fresh vs Stale handling

---

### Phase 4: Incremental Resolution
**Goal:** Skip RefExpander when PropertyBank is fresh

**Changes:**
1. Store expanded properties when resolving schemas
2. Update resolution pipeline to use cached expanded schemas
3. Add logic to skip expansion for fresh schemas

**Tests:**
- Fresh PropertyBank + Fresh schemas → no expansion
- Stale PropertyBank → re-expand only affected schemas
- Schema file change → re-expand that schema only

---

## Open Questions

### Q1: Compression Strategy
**Question:** Should we compress the serialized structs?
- Option A: `rkyv(T)` - Faster, larger
- Option B: `zstd(rkyv(T))` - Slower, smaller
- Option C: Configurable per deployment

**Need:** Benchmark with real data before deciding.

---

### Q2: RawSchemaMetadata Fate
**Question:** Remove entirely or rename to RawFileTimes?
- Option A: Remove, move timestamps to RawFileVersion
- Option B: Rename to RawFileTimes, keep structure

**Recommendation:** Remove (Option A) for cleaner separation.

---

### Q3: SchemaId Storage
**Question:** Should RawSchemaView store SchemaId directly?

**Current:** SchemaId derived from queries or reconstructed from name
**Alternative:** Store SchemaId in RawSchemaView

**Trade-off:**
- Pro: Faster lookups, no need for name_to_id map
- Con: Must update view when ID changes (rare but possible)

**Recommendation:** Store it - ID changes are extremely rare.

---

### Q4: PropertyBankResult::New Necessity
**Question:** Do we need separate `New` variant or treat as `Stale` with all properties changed?

**Analysis:**
```rust
// Option A: Three variants
PropertyBankResult::New(bank)  // First time
PropertyBankResult::Fresh(bank)  // Unchanged
PropertyBankResult::Stale { bank, changed }  // Changed

// Option B: Two variants
PropertyBankResult::Fresh(bank)  // Unchanged
PropertyBankResult::Stale { bank, changed }  // Changed or new
  // If new: changed = all properties
```

**Recommendation:** Keep three variants for semantic clarity and explicit handling of first load.

---

### Q5: Backward Compatibility
**Question:** Can we migrate existing Raw*View data or do we need to rebuild cache?

**Answer:** Likely need cache rebuild since structure is changing significantly.

**Migration Strategy:**
1. Add feature flag for new storage format
2. Support reading old format during transition
3. Gradually migrate on access (lazy migration)
4. Or: Provide migration tool to rebuild all views

---

## Performance Impact Predictions

### Current Performance (Baseline)
```
Load 100 schemas (all fresh):
- File stat calls: 100 (check timestamps)
- DB queries: 200 (100 views + 100 IDs)
- Decompression: 100
- Parsing: 100
- RefExpander runs: 100
- Total time: ~500ms (estimated)
```

### After Phase 1-2 (Serialized Storage + PropertyBank)
```
Load 100 schemas (all fresh):
- File stat calls: 100
- DB queries: 2 (bulk views + IDs)
- Deserialization: 100 (faster than parsing)
- RefExpander runs: 100
- Total time: ~300ms (estimated, -40%)
```

### After Phase 3-4 (Structured Results + Incremental Resolution)
```
Load 100 schemas (all fresh, PropertyBank fresh):
- File stat calls: 100
- DB queries: 2 (bulk views + IDs)
- Deserialization: 100
- RefExpander runs: 0 (skipped!)
- Total time: ~100ms (estimated, -80%)
```

**Key Insight:** Biggest win is skipping RefExpander when PropertyBank is fresh (common case).

---

## Next Steps

1. **Review and approve this analysis**
2. **Decide on open questions** (compression, metadata removal, etc.)
3. **Create detailed refactoring plan** for Phase 1
4. **Write tests for new behavior** before changing code
5. **Implement phases incrementally** with all tests passing between phases

---

## Appendix: Code Samples

### A1: Split Version Types

```rust
/// File timestamp metadata - shared by both schema and property bank versions
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct FileVersionMetadata {
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,

    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,

    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

/// A single version of a schema file with cached data
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// File timestamp metadata
    metadata: FileVersionMetadata,

    /// Content hash for staleness detection (kept in version, not in RawSchema)
    content_hash: [u8; 32],

    /// Per-property hashes for incremental resolution (kept in version)
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,

    /// Serialized RawSchema (optionally compressed)
    /// Format: rkyv(RawSchema) or zstd(rkyv(RawSchema))
    archived_schema: Vec<u8>,

    /// Cached expanded properties (from RefExpander)
    /// Enables skipping expansion when PropertyBank is fresh
    expanded_properties: Option<HashMap<PropertyName, Property>>,
}

impl SchemaVersion {
    /// Deserialize cached RawSchema
    pub fn to_raw(&self) -> Result<RawSchema, Error> {
        // Option 1: If uncompressed
        rkyv::from_bytes(&self.archived_schema)

        // Option 2: If compressed
        // let decompressed = zstd::decode_all(self.archived_schema.as_slice())?;
        // rkyv::from_bytes(&decompressed)
    }

    /// Compute property hashes from RawSchema properties
    pub fn compute_property_hashes(
        properties: &HashMap<Box<str>, RawProperty>
    ) -> BTreeMap<PropertyName, [u8; 32]> {
        properties
            .iter()
            .filter_map(|(name, prop)| {
                let hash = Self::hash_property(prop);
                PropertyName::try_new(name.as_ref())
                    .ok()
                    .map(|pn| (pn, hash))
            })
            .collect()
    }

    fn hash_property(prop: &RawProperty) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        // Hash the property definition (spec + multiplicity)
        hasher.update(prop.spec.to_string().as_bytes());
        hasher.update(&[prop.multi as u8]);
        *hasher.finalize().as_bytes()
    }
}

/// A single version of the property bank file with cached data
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct PropertyBankVersion {
    /// File timestamp metadata
    metadata: FileVersionMetadata,

    /// Content hash for staleness detection
    content_hash: [u8; 32],

    /// Per-property hashes for incremental updates
    property_hashes: BTreeMap<PropertyName, [u8; 32]>,

    /// Serialized RawPropertyBank (optionally compressed)
    archived_property_bank: Vec<u8>,
}

impl PropertyBankVersion {
    /// Deserialize cached RawPropertyBank
    pub fn to_raw(&self) -> Result<RawPropertyBank, Error> {
        rkyv::from_bytes(&self.archived_property_bank)
    }
}
```

### A2: Enhanced RawSchemaView Structure

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaView {
    /// File path relative to vault root (lookup key)
    file_path: Box<str>,

    /// Parent schema name (from `extends` field)
    extends: Option<SchemaName>,

    /// Property names to exclude from parent
    excludes: Vec<PropertyName>,

    /// Version history (ring buffer, max 5 versions, newest first)
    /// Each version contains serialized RawSchema + expanded properties
    versions: VecDeque<SchemaVersion>,  // Changed from RawFileVersion
}

impl RawSchemaView {
    /// Get current version
    pub fn current(&self) -> Option<&SchemaVersion> {
        self.versions.front()
    }

    /// Deserialize cached RawSchema from current version
    pub fn to_raw(&self) -> Option<RawSchema> {
        self.current()?.to_raw().ok()
    }

    /// Reconstruct RefExpandedSchema from cached data
    pub fn to_ref_expanded_schema(&self, id: SchemaId) -> Option<RefExpandedSchema> {
        let version = self.current()?;
        let expanded_props = version.expanded_properties.as_ref()?;

        Some(RefExpandedSchema {
            id,
            name: self.derive_schema_name(),
            file_path: self.file_path.clone(),
            extends: self.extends.clone(),
            excludes: self.excludes.clone(),
            properties: expanded_props.clone(),
        })
    }

    /// Check if current version is fresh (matches file metadata)
    pub fn is_fresh(
        &self,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> bool {
        self.current()
            .is_some_and(|v| {
                v.metadata.created_at == created_at
                    && v.metadata.modified_at == modified_at
            })
    }

    /// Get changed properties by comparing with new property hashes
    pub fn changed_properties(
        &self,
        new_hashes: &BTreeMap<PropertyName, [u8; 32]>,
    ) -> Vec<PropertyName> {
        let Some(current) = self.current() else {
            return Vec::new();
        };

        current.property_hashes
            .keys()
            .chain(new_hashes.keys())
            .filter(|name| {
                current.property_hashes.get(name) != new_hashes.get(name)
            })
            .cloned()
            .collect()
    }
}
```

### A3: PropertyBank Incremental Update

```rust
impl PropertyBank {
    /// Update properties incrementally from raw property bank.
    ///
    /// Only updates the properties specified in `changed`.
    /// More efficient than rebuilding entire bank.
    pub fn update_from_raw(
        &mut self,
        raw: &RawPropertyBank,
        changed: &[PropertyName],
    ) -> Result<(), Error> {
        let updates: Vec<_> = changed
            .iter()
            .filter_map(|name| {
                raw.properties
                    .get(name.as_ref())
                    .map(|raw_prop| (name.clone(), raw_prop.clone()))
            })
            .collect();

        self.update_properties(&updates)
    }
}
```

### A4: Structured Ingestor API

```rust
pub struct IngestorResults {
    pub property_bank: PropertyBankResult,
    pub schemas: HashMap<PathBuf, SchemaIngestResult>,
}

pub enum PropertyBankResult {
    New(PropertyBank),
    Fresh(PropertyBank),
    Stale {
        bank: PropertyBank,
        changed: Vec<PropertyName>,
    },
}

pub enum SchemaIngestResult {
    Fresh {
        id: SchemaId,
        expanded: Option<RefExpandedSchema>,
    },
    Stale {
        id: SchemaId,
        raw: RawSchema,
        expanded: Option<RefExpandedSchema>,
    },
}

impl Ingestor {
    pub fn ingest_all(&self) -> Result<IngestorResults, Error> {
        // 1. Ingest property bank
        let property_bank = self.ingest_property_bank()?;

        // 2. List all schema files
        let paths = self.list_all_schema_files()?;

        // 3. Bulk queries (no N+1!)
        let views = self.repository.find_raw_schema_views_by_paths(&paths)?;
        let ids = self.repository.find_schema_ids_by_paths(&paths)?;

        // 4. Process each schema (single loop)
        let mut schemas = HashMap::new();
        for path in paths {
            let view = views.get(&path);
            let id = ids.get(&path).copied().unwrap_or_else(SchemaId::new);

            let result = self.process_schema(&path, id, view)?;
            schemas.insert(path, result);
        }

        Ok(IngestorResults {
            property_bank,
            schemas,
        })
    }
}
```

### A5: Optimized Loader Flow

```rust
impl Loader {
    pub fn load(&self) -> Result<Vec<Schema>, Error> {
        // Single ingestion call
        let results = self.ingestor.ingest_all()?;

        // Extract property bank (already in desired form)
        let bank = match results.property_bank {
            PropertyBankResult::Fresh(bank) => bank,
            PropertyBankResult::Stale { bank, .. } => bank,
            PropertyBankResult::New(bank) => bank,
        };

        // Partition schemas (single pass, no double loop!)
        let mut fresh_expanded = Vec::new();
        let mut stale_to_expand = Vec::new();

        for (path, result) in results.schemas {
            match result {
                SchemaIngestResult::Fresh { expanded: Some(exp), .. } => {
                    fresh_expanded.push(exp);
                }
                SchemaIngestResult::Stale { id, raw, .. } => {
                    stale_to_expand.push((id, raw));
                }
                _ => { /* handle other cases */ }
            }
        }

        // Only expand stale schemas (if PropertyBank fresh, fresh_expanded already done!)
        let mut all_expanded = fresh_expanded;
        if !stale_to_expand.is_empty() {
            let newly_expanded = RefExpander::new(&bank).expand_all(stale_to_expand)?;
            all_expanded.extend(newly_expanded);
        }

        // Rest of resolution pipeline
        let tree = Extender::build(all_expanded, &known_parents)?;
        let resolved = Resolver::resolve(&tree, &known_parents)?;

        Ok(resolved)
    }
}
```

---

## References

- Current implementation: `lithos-core/src/schema/loader.rs`
- Current implementation: `lithos-core/src/schema/ingestor.rs`
- View types: `lithos-core/src/schema/views/raw.rs`
- PropertyBank: `lithos-core/src/schema/bank.rs`
- RefExpander: `lithos-core/src/schema/expander.rs`
