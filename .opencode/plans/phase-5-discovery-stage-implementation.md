# Phase 5: Discovery Stage Implementation Plan

## Overview

Implement the Discovery stage, which is the entry point for the schema pipeline. This stage scans the filesystem, queries the database, builds global indexes, and branches each schema into the appropriate pipeline path.

## Goals

1. Implement `SchemaProcessor<Discovery, Unknown>::discover()` method
2. Create global context structures for batch coordination
3. Implement file system scanning logic
4. Implement database batch query logic
5. Implement per-schema branching logic
6. Handle deleted schemas
7. Add comprehensive tests

## Current State

### What Exists
- ✅ Stage marker: `Discovery` struct defined
- ✅ Status types: `Unknown`, `Missing`, `Present` defined
- ✅ Branching enum: `DiscoveryBranch` defined
- ✅ Repository methods: `find_raw_schema_views_by_paths()` exists
- ✅ Filesystem abstraction: `FileReader` available
- ✅ Method signature: `discover()` exists with `todo!()`

### What's Needed
- ❌ Discovery orchestration logic
- ❌ Global context structures
- ❌ File scanning implementation
- ❌ Name-to-ID index building
- ❌ Deleted schema detection
- ❌ Per-schema branching logic
- ❌ Integration with PropertyBank delta
- ❌ Comprehensive tests

## Design Specification

From `schema-pipeline-typestate-redesign.md`:

### Stage 1: Discovery (Batch Start, Per-Schema Branch)

**Purpose**: Initialize batch processing, query DB, detect deletions, branch schemas into pipelines

**Scope**: Batch operation producing per-schema pipelines

**Operations**:
1. Scan schema directory (excluding `property_bank` file)
2. Batch query: Load all `RawSchemaView`s from DB (`find_raw_schema_views_by_paths`)
3. Build global indexes: `name_to_id`, `id_to_name` from DB data
4. Detect deleted schemas: Schemas in DB but not on filesystem
5. Check PropertyBank staleness (from upstream `PropertyBankProcessor`)
6. For each schema file: timestamp check, determine branch path
7. Produce: `Vec<SchemaProcessor<Comparison, Status>>` (one per file)

**Outputs**:
- `Vec<DiscoveryBranch>` (one per schema)
- Global context: `name_to_id`, `id_to_name`, deleted schema IDs, PropertyBank delta

**Errors**: `SchemaRepositoryError`, `SchemaFileError`

## Implementation Plan

### Task 1: Define Global Context Structures

Create structures to hold batch-level information that needs to be shared across stages.

```rust
/// Global context produced by Discovery stage for batch coordination.
#[derive(Debug)]
pub(crate) struct DiscoveryContext {
    /// Map from schema name to SchemaId (for parent lookups).
    pub(crate) name_to_id: HashMap<SchemaName, SchemaId>,

    /// Map from SchemaId to schema name (for error messages).
    pub(crate) id_to_name: HashMap<SchemaId, SchemaName>,

    /// Schemas that exist in DB but not on filesystem (for cleanup).
    pub(crate) deleted_schema_ids: Vec<SchemaId>,

    /// PropertyBank delta from upstream processor (if available).
    pub(crate) property_bank_delta: Option<PropertyBankDelta>,
}
```

**Where**: Add to `lithos-core/src/schema/schema_pipeline.rs` after delta structures

**Tests**: Unit tests for struct creation and field access

### Task 2: Implement File System Scanning

Implement logic to scan the schema directory and collect all schema file paths.

**Requirements**:
- Use `FileReader` abstraction (already available)
- Exclude `property_bank` file
- Collect file paths and timestamps
- Handle I/O errors gracefully

**Signature**:
```rust
fn scan_schema_files(
    source: &FileReader,
) -> Result<Vec<(PathBuf, RawFileTimes)>, SchemaLoaderError>;
```

**Where**: Helper function in `schema_pipeline.rs` or separate `discovery.rs` module

**Tests**:
- Scans directory successfully
- Excludes property_bank file
- Returns empty vec for empty directory
- Handles I/O errors

### Task 3: Implement Database Batch Query

Query the database for all existing `RawSchemaView`s matching the file paths.

**Requirements**:
- Call `repository.find_raw_schema_views_by_paths()`
- Build `name_to_id` and `id_to_name` indexes from results
- Handle missing views gracefully
- Return both views and indexes

**Signature**:
```rust
fn query_existing_views<R: Repository>(
    repository: &R,
    file_paths: &[PathBuf],
) -> Result<(HashMap<PathBuf, (SchemaId, RawSchemaView)>, HashMap<SchemaName, SchemaId>, HashMap<SchemaId, SchemaName>), SchemaLoaderError>
where
    R::Error: Into<SchemaRepositoryError>;
```

**Where**: Helper function in `schema_pipeline.rs`

**Tests**:
- Queries views for multiple paths
- Builds correct indexes
- Handles missing views (returns None in map)
- Handles empty input

### Task 4: Implement Deleted Schema Detection

Detect schemas that exist in the database but not on the filesystem.

**Requirements**:
- Compare DB views against filesystem paths
- Collect SchemaIds for deletion
- Return list of deleted IDs

**Signature**:
```rust
fn detect_deleted_schemas(
    db_views: &HashMap<PathBuf, (SchemaId, RawSchemaView)>,
    file_paths: &[PathBuf],
) -> Vec<SchemaId>;
```

**Where**: Helper function in `schema_pipeline.rs`

**Tests**:
- Detects schemas in DB but not filesystem
- Returns empty vec when all schemas present
- Handles multiple deletions

### Task 5: Implement Per-Schema Branching

For each schema file, determine the appropriate branch path (Missing vs Present).

**Requirements**:
- For each file, check if view exists in DB
- If missing → create `Missing` status, generate new SchemaId
- If present → create `Present` status with cached view
- Wrap in `DiscoveryBranch` enum
- Collect timestamps for each file

**Signature**:
```rust
fn branch_schema(
    file_path: PathBuf,
    file_times: RawFileTimes,
    db_view: Option<(SchemaId, RawSchemaView)>,
) -> DiscoveryBranch;
```

**Where**: Helper function in `schema_pipeline.rs`

**Tests**:
- Branches to Missing when no DB view
- Branches to Present when DB view exists
- Generates unique SchemaId for new schemas
- Preserves file times correctly

### Task 6: Implement Main `discover()` Method

Orchestrate all the above steps in the main discover method.

**Requirements**:
- Accept `FileReader` and `Repository` as inputs
- Accept optional PropertyBank delta
- Call scan → query → detect → branch in sequence
- Return `Vec<DiscoveryBranch>` + `DiscoveryContext`
- Handle all errors with proper context

**Current Signature** (needs update):
```rust
pub(crate) fn discover<R: Repository>(
    self,
    source: &FileReader,
    repository: &R,
) -> Result<DiscoveryBranch, SchemaLoaderError>
where
    R::Error: Into<SchemaRepositoryError>;
```

**New Signature**:
```rust
pub(crate) fn discover<R: Repository>(
    self,
    source: &FileReader,
    repository: &R,
    property_bank_delta: Option<PropertyBankDelta>,
) -> Result<(Vec<DiscoveryBranch>, DiscoveryContext), SchemaLoaderError>
where
    R::Error: Into<SchemaRepositoryError>;
```

**Implementation Steps**:
1. Scan filesystem for schema files
2. Query DB for existing views
3. Build name_to_id and id_to_name indexes
4. Detect deleted schemas
5. For each file, branch into Missing/Present
6. Construct DiscoveryContext
7. Return branches + context

**Tests**:
- Discovers new schemas (Missing branches)
- Discovers existing schemas (Present branches)
- Builds correct name_to_id index
- Detects deleted schemas
- Handles mixed scenarios (new + existing + deleted)
- Handles empty directory
- Propagates I/O errors
- Propagates repository errors

### Task 7: Update Builder Orchestration

Update `builder.rs` to call the new `discover()` method with proper inputs.

**Requirements**:
- Remove `todo!()` placeholder
- Call `discover()` with correct parameters
- Handle `DiscoveryContext` for downstream stages
- Store context for use in TreeGraphed stage

**Where**: `lithos-core/src/schema/builder.rs::load_schemas_v2()`

**Tests**: Integration tests in `tests/schema_loader.rs`

### Task 8: Add PropertyBankDelta Type

Define the PropertyBankDelta type to pass from PropertyBankProcessor to schema pipeline.

**Requirements**:
- Define structure for changed property names
- Match PropertyBankProcessor output format
- Keep lightweight (just property names, not full properties)

**Signature**:
```rust
#[derive(Debug, Clone)]
pub(crate) struct PropertyBankDelta {
    /// Properties added or changed in PropertyBank.
    pub(crate) changed: HashSet<PropertyName>,
}
```

**Where**: `lithos-core/src/schema/schema_pipeline.rs` (or separate file)

**Tests**: Unit tests for construction and access

## Implementation Order

Recommended order (each task builds on previous):

1. **Task 8**: PropertyBankDelta type (simple struct, no dependencies)
2. **Task 1**: DiscoveryContext struct (simple struct, references PropertyBankDelta)
3. **Task 2**: File system scanning (standalone helper)
4. **Task 3**: Database batch query (standalone helper)
5. **Task 4**: Deleted schema detection (uses Task 3 output)
6. **Task 5**: Per-schema branching (uses Task 3 output)
7. **Task 6**: Main discover() method (orchestrates Tasks 2-5)
8. **Task 7**: Builder orchestration update (uses Task 6)

## Testing Strategy

### Unit Tests (Per Task)
- Test each helper function independently
- Use in-memory repository for DB operations
- Use test fixtures for filesystem operations
- Focus on edge cases and error handling

### Integration Tests
- End-to-end Discovery stage execution
- Real filesystem + in-memory DB
- Test scenarios:
  - Empty vault (no schemas)
  - New vault (all schemas new)
  - Existing vault (mixed new/existing/deleted)
  - Error scenarios (missing files, DB errors)

### Test Files
- `lithos-core/src/schema/schema_pipeline.rs` (unit tests in `#[cfg(test)]` module)
- `lithos-core/tests/schema_discovery.rs` (integration tests)

## Dependencies & Prerequisites

### External Dependencies (Already Available)
- `FileReader` - filesystem abstraction
- `Repository` trait - database operations
- `RawSchemaView` - cached view structure
- `SchemaId` - UUID type for schemas
- `SchemaName` - validated name type
- `RawFileTimes` - file metadata structure

### Internal Dependencies (Already Implemented)
- Stage markers (Discovery, Comparison)
- Status types (Unknown, Missing, Present)
- Branching enum (DiscoveryBranch)
- Error types (SchemaLoaderError)

### Prerequisites
- PropertyBankProcessor must be run first (provides PropertyBankDelta)
- Schema directory must exist (can be empty)
- Database must be initialized (can be empty)

## Success Criteria

✅ **Functional**:
- Discovery stage produces correct branches for all schemas
- Global context includes accurate name_to_id indexes
- Deleted schemas are correctly identified
- All file system operations use FileReader abstraction
- All database operations use Repository trait

✅ **Quality**:
- All unit tests pass
- All integration tests pass
- No clippy warnings
- Code formatted with rustfmt
- Proper error handling with context
- Documentation for all public/crate functions

✅ **Design**:
- Follows typestate pattern consistently
- Matches PropertyBank orchestration style
- Clean separation of concerns (scan/query/branch)
- Minimal allocations (use references where possible)

## Estimated Effort

- **Simple Tasks** (1-2, 8): ~2-3 hours
  - Struct definitions, basic logic

- **Medium Tasks** (3-5): ~4-6 hours
  - Helper functions with some complexity

- **Complex Tasks** (6-7): ~4-6 hours
  - Orchestration logic, error handling, integration

- **Testing**: ~6-8 hours
  - Unit tests, integration tests, edge cases

**Total**: ~16-23 hours of implementation time

## Open Questions

1. **SchemaId Generation**: How should we generate SchemaIds for new schemas?
   - Option A: Use UUID v4 (random)
   - Option B: Use UUID v5 (deterministic based on name)
   - **Recommendation**: UUID v4 for simplicity (matches current pattern)

2. **Deleted Schema Handling**: Should Discovery stage delete them or just report them?
   - Option A: Just report (let builder decide)
   - Option B: Delete immediately
   - **Recommendation**: Just report (separation of concerns)

3. **PropertyBankDelta Format**: What exact shape should this take?
   - Option A: Just changed property names (HashSet)
   - Option B: Full delta with added/removed/changed
   - **Recommendation**: Start simple with Option A

4. **Error Recovery**: How should we handle partial failures (some files unreadable)?
   - Option A: Fail entire batch
   - Option B: Skip bad files, continue with rest
   - **Recommendation**: Option A for now (fail-fast)

## Next Steps After Phase 5

Once Discovery is complete, the logical next phases are:

**Phase 6**: Refresh Stage Implementation
- Simple metadata updates
- No complex logic
- Good warm-up for batch operations

**Phase 7**: TreeGraphed Stage Implementation (Complex)
- Graph building and patching
- Cycle detection
- Topological sorting

**Phase 8**: PropertyAnalysis Stage Implementation (Complex)
- Delta computation
- Bank reference tracking
- Excludes analysis

**Phase 9**: Construction Stage Implementation (Most Complex)
- Level-by-level processing
- Property merging
- Inheritance resolution
