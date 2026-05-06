# Phase 5: Discovery Stage - Task Checklist

## Quick Reference

- **Total Tasks**: 8
- **Estimated Time**: 16-23 hours
- **Priority**: High (blocks all downstream stages)
- **Dependencies**: None (foundation complete)

## Task Progress

- [ ] Task 8: PropertyBankDelta type (~2h)
- [ ] Task 1: DiscoveryContext struct (~2h)
- [ ] Task 2: File system scanning (~4h)
- [ ] Task 3: Database batch query (~4h)
- [ ] Task 4: Deleted schema detection (~3h)
- [ ] Task 5: Per-schema branching (~4h)
- [ ] Task 6: Main discover() method (~4h)
- [ ] Task 7: Builder orchestration (~3h)

---

## Task 8: PropertyBankDelta Type ⚡ START HERE

**Priority**: P0 (blocks Task 1)
**Estimated Time**: 2 hours
**Complexity**: Simple

### Definition

```rust
/// Delta from PropertyBank processor indicating which properties changed.
///
/// Used by schema pipeline to determine which schemas need re-expansion.
#[derive(Debug, Clone, Default)]
pub struct PropertyBankDelta {
    /// Properties that were added, removed, or modified in PropertyBank.
    pub changed: HashSet<PropertyName>,
}

impl PropertyBankDelta {
    /// Creates an empty delta (no changes).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a delta with the given changed properties.
    pub fn with_changed(changed: HashSet<PropertyName>) -> Self {
        Self { changed }
    }

    /// Returns true if any properties changed.
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty()
    }

    /// Returns true if the given property changed.
    pub fn contains(&self, property: &PropertyName) -> bool {
        self.changed.contains(property)
    }
}
```

### Location
`lithos-core/src/schema/schema_pipeline.rs` after `SchemaPropertyUpserts`

### Tests
```rust
#[test]
fn property_bank_delta_empty() {
    let delta = PropertyBankDelta::empty();
    assert!(delta.is_empty());
}

#[test]
fn property_bank_delta_with_changes() {
    let mut changed = HashSet::new();
    changed.insert("prop1".try_into().unwrap());
    let delta = PropertyBankDelta::with_changed(changed);
    assert!(!delta.is_empty());
    assert!(delta.contains(&"prop1".try_into().unwrap()));
}
```

### Definition of Done
- [ ] Struct defined with documentation
- [ ] Helper methods implemented
- [ ] Unit tests pass
- [ ] No clippy warnings

---

## Task 1: DiscoveryContext Struct

**Priority**: P0 (blocks Task 6)
**Estimated Time**: 2 hours
**Complexity**: Simple
**Dependencies**: Task 8

### Definition

```rust
/// Global context produced by Discovery stage for batch coordination.
///
/// Contains indexes and metadata needed by downstream stages (TreeGraphed,
/// PropertyAnalysis, Construction).
#[derive(Debug)]
pub struct DiscoveryContext {
    /// Map from schema name to SchemaId for parent lookups.
    ///
    /// Built from existing database views. Used by TreeGraphed stage to
    /// resolve parent references.
    pub name_to_id: HashMap<SchemaName, SchemaId>,

    /// Map from SchemaId to schema name for error messages.
    ///
    /// Reverse index of `name_to_id` for better diagnostics.
    pub id_to_name: HashMap<SchemaId, SchemaName>,

    /// Schemas that exist in DB but not on filesystem.
    ///
    /// These may need cleanup depending on orchestration strategy.
    pub deleted_schema_ids: Vec<SchemaId>,

    /// PropertyBank delta from upstream processor.
    ///
    /// If present, indicates which properties changed in PropertyBank,
    /// allowing schemas to determine if they need re-expansion.
    pub property_bank_delta: Option<PropertyBankDelta>,
}

impl DiscoveryContext {
    /// Creates a new discovery context.
    pub fn new(
        name_to_id: HashMap<SchemaName, SchemaId>,
        id_to_name: HashMap<SchemaId, SchemaName>,
        deleted_schema_ids: Vec<SchemaId>,
        property_bank_delta: Option<PropertyBankDelta>,
    ) -> Self {
        Self {
            name_to_id,
            id_to_name,
            deleted_schema_ids,
            property_bank_delta,
        }
    }

    /// Returns the SchemaId for a given schema name.
    pub fn lookup_id(&self, name: &SchemaName) -> Option<SchemaId> {
        self.name_to_id.get(name).copied()
    }

    /// Returns the schema name for a given SchemaId.
    pub fn lookup_name(&self, id: SchemaId) -> Option<&SchemaName> {
        self.id_to_name.get(&id)
    }
}
```

### Location
`lithos-core/src/schema/schema_pipeline.rs` after `PropertyBankDelta`

### Tests
```rust
#[test]
fn discovery_context_lookup_id() {
    let mut name_to_id = HashMap::new();
    let name: SchemaName = "test".try_into().unwrap();
    let id = SchemaId::new_v4();
    name_to_id.insert(name.clone(), id);

    let ctx = DiscoveryContext::new(
        name_to_id,
        HashMap::new(),
        Vec::new(),
        None,
    );

    assert_eq!(ctx.lookup_id(&name), Some(id));
}

#[test]
fn discovery_context_lookup_name() {
    let mut id_to_name = HashMap::new();
    let name: SchemaName = "test".try_into().unwrap();
    let id = SchemaId::new_v4();
    id_to_name.insert(id, name.clone());

    let ctx = DiscoveryContext::new(
        HashMap::new(),
        id_to_name,
        Vec::new(),
        None,
    );

    assert_eq!(ctx.lookup_name(id), Some(&name));
}
```

### Definition of Done
- [ ] Struct defined with documentation
- [ ] Helper methods implemented
- [ ] Unit tests pass
- [ ] No clippy warnings

---

## Task 2: File System Scanning

**Priority**: P1 (needed for Task 6)
**Estimated Time**: 4 hours
**Complexity**: Medium
**Dependencies**: None

### Implementation

Helper function to scan schema directory:

```rust
/// Scans the schema directory and returns file paths with timestamps.
///
/// Excludes the `property_bank` file as it's not a schema.
///
/// # Errors
/// Returns error if directory read fails or file metadata unavailable.
fn scan_schema_files(
    source: &FsReader,
    schemas_dir: &Path,
) -> Result<Vec<(PathBuf, RawFileTimes)>, SchemaLoaderError> {
    // TODO: Implement
    todo!()
}
```

### Requirements
- Use `FsReader::list_files()` or similar
- Extract file modified/created times
- Filter out `property_bank` file
- Return `(PathBuf, RawFileTimes)` pairs
- Handle I/O errors with proper context

### Tests
- Scans directory with multiple files
- Excludes property_bank file
- Returns empty vec for empty directory
- Extracts correct timestamps
- Handles missing directory
- Handles permission errors

### Definition of Done
- [ ] Function implemented
- [ ] All tests pass
- [ ] Errors have proper context
- [ ] No clippy warnings

---

## Task 3: Database Batch Query

**Priority**: P1 (needed for Task 6)
**Estimated Time**: 4 hours
**Complexity**: Medium
**Dependencies**: None

### Implementation

```rust
/// Queries database for existing views and builds name/ID indexes.
///
/// Returns:
/// - HashMap of file path → (SchemaId, RawSchemaView)
/// - name_to_id index
/// - id_to_name index
///
/// # Errors
/// Returns error if database query fails.
fn query_existing_views<R: Repository>(
    repository: &R,
    file_paths: &[PathBuf],
) -> Result<(
    HashMap<PathBuf, (SchemaId, RawSchemaView)>,
    HashMap<SchemaName, SchemaId>,
    HashMap<SchemaId, SchemaName>,
), SchemaLoaderError>
where
    R::Error: Into<SchemaRepositoryError>,
{
    // TODO: Implement
    todo!()
}
```

### Requirements
- Call `repository.find_raw_schema_views_by_paths(file_paths)`
- For each view, extract SchemaId and SchemaName
- Build both name_to_id and id_to_name indexes
- Handle missing views (return None in HashMap)
- Proper error conversion

### Tests
- Queries multiple paths successfully
- Builds correct indexes
- Handles missing views gracefully
- Returns empty maps for no files
- Handles repository errors

### Definition of Done
- [ ] Function implemented
- [ ] All tests pass
- [ ] Indexes correct
- [ ] No clippy warnings

---

## Task 4: Deleted Schema Detection

**Priority**: P1 (needed for Task 6)
**Estimated Time**: 3 hours
**Complexity**: Simple
**Dependencies**: Task 3

### Implementation

```rust
/// Detects schemas that exist in database but not on filesystem.
///
/// Returns list of SchemaIds for schemas that should be cleaned up.
fn detect_deleted_schemas(
    db_views: &HashMap<PathBuf, (SchemaId, RawSchemaView)>,
    file_paths: &HashSet<PathBuf>,
) -> Vec<SchemaId> {
    // TODO: Implement
    todo!()
}
```

### Requirements
- Compare db_views keys against file_paths
- Collect SchemaIds where path not in file_paths
- Return as Vec for easy iteration

### Tests
- Detects single deleted schema
- Detects multiple deletions
- Returns empty when all present
- Handles empty inputs

### Definition of Done
- [ ] Function implemented
- [ ] All tests pass
- [ ] Logic correct
- [ ] No clippy warnings

---

## Task 5: Per-Schema Branching

**Priority**: P1 (needed for Task 6)
**Estimated Time**: 4 hours
**Complexity**: Medium
**Dependencies**: Task 3

### Implementation

```rust
/// Branches a schema into Missing or Present based on DB view existence.
///
/// Generates new SchemaId for missing schemas.
fn branch_schema(
    file_path: PathBuf,
    file_times: RawFileTimes,
    db_view: Option<(SchemaId, RawSchemaView)>,
) -> DiscoveryBranch {
    match db_view {
        None => {
            let id = SchemaId::new_v4();
            DiscoveryBranch::Missing(
                SchemaProcessor::transition(
                    Comparison,
                    Missing { /* fields */ },
                )
            )
        }
        Some((id, view)) => {
            DiscoveryBranch::Present(
                SchemaProcessor::transition(
                    Comparison,
                    Present { id, times: file_times, view },
                )
            )
        }
    }
}
```

### Requirements
- Accept file path, times, optional view
- Generate UUID for new schemas
- Create appropriate status struct
- Wrap in DiscoveryBranch enum
- Transition to Comparison stage

### Tests
- Branches to Missing when no view
- Branches to Present when view exists
- Generates unique IDs
- Preserves file times
- Preserves view data

### Definition of Done
- [ ] Function implemented
- [ ] All tests pass
- [ ] Branches correct
- [ ] No clippy warnings

---

## Task 6: Main discover() Method

**Priority**: P0 (core functionality)
**Estimated Time**: 4 hours
**Complexity**: Medium-High
**Dependencies**: Tasks 1-5

### Implementation

Update the signature and implement:

```rust
impl SchemaProcessor<Discovery, Unknown> {
    pub(crate) fn discover<R: Repository>(
        self,
        source: &FsReader,
        repository: &R,
        schemas_dir: &Path,
        property_bank_delta: Option<PropertyBankDelta>,
    ) -> Result<(Vec<DiscoveryBranch>, DiscoveryContext), SchemaLoaderError>
    where
        R::Error: Into<SchemaRepositoryError>,
    {
        // 1. Scan filesystem
        let file_data = scan_schema_files(source, schemas_dir)?;
        let file_paths: Vec<PathBuf> = file_data.iter()
            .map(|(p, _)| p.clone())
            .collect();

        // 2. Query database
        let (db_views, name_to_id, id_to_name) =
            query_existing_views(repository, &file_paths)?;

        // 3. Detect deletions
        let file_paths_set: HashSet<PathBuf> = file_paths.into_iter().collect();
        let deleted_schema_ids = detect_deleted_schemas(&db_views, &file_paths_set);

        // 4. Branch each schema
        let branches = file_data
            .into_iter()
            .map(|(path, times)| {
                let view = db_views.get(&path).cloned();
                branch_schema(path, times, view)
            })
            .collect();

        // 5. Build context
        let context = DiscoveryContext::new(
            name_to_id,
            id_to_name,
            deleted_schema_ids,
            property_bank_delta,
        );

        Ok((branches, context))
    }
}
```

### Requirements
- Orchestrate all helper functions
- Handle errors with proper context
- Return branches + context
- Clear, readable flow

### Tests
- Discovers new schemas
- Discovers existing schemas
- Mixed scenario (new + existing + deleted)
- Empty directory
- I/O errors propagate
- Repository errors propagate

### Definition of Done
- [ ] Method implemented
- [ ] All tests pass
- [ ] Orchestration correct
- [ ] No clippy warnings

---

## Task 7: Builder Orchestration Update

**Priority**: P1 (integration point)
**Estimated Time**: 3 hours
**Complexity**: Medium
**Dependencies**: Task 6

### Implementation

Update `builder.rs::load_schemas_v2()`:

```rust
pub(crate) fn load_schemas_v2(
    &self,
    pb: &PropertyBank,
    property_bank_delta: Option<PropertyBankDelta>,
) -> Result<Vec<Schema>, SchemaLoaderError> {
    use schema_pipeline::{Discovery, SchemaProcessor, Unknown};

    // Start Discovery
    let pipeline = SchemaProcessor::<Discovery, Unknown>::new();
    let (branches, context) = pipeline.discover(
        &self.source,
        &self.repository,
        &self.config.paths().schema.dir(), // TODO: Fix path method
        property_bank_delta,
    )?;

    // TODO: Process branches through Comparison stage
    // TODO: Batch processing (TreeGraphed → PropertyAnalysis → Construction)

    todo!("complete pipeline orchestration")
}
```

### Requirements
- Remove old todo!()
- Call discover() with correct params
- Store context for downstream stages
- Update method signature (add PropertyBankDelta param)

### Tests
- Integration test: Discovery + Comparison flow
- Test with empty directory
- Test with new schemas
- Test with existing schemas

### Definition of Done
- [ ] Builder updated
- [ ] Method signature correct
- [ ] Discovery called correctly
- [ ] Integration tests pass

---

## Testing Strategy

### Unit Tests
- [ ] PropertyBankDelta tests
- [ ] DiscoveryContext tests
- [ ] scan_schema_files tests
- [ ] query_existing_views tests
- [ ] detect_deleted_schemas tests
- [ ] branch_schema tests
- [ ] discover() tests

### Integration Tests
Create `lithos-core/tests/schema_discovery.rs`:
- [ ] End-to-end Discovery execution
- [ ] Empty vault scenario
- [ ] New vault scenario (all new schemas)
- [ ] Existing vault scenario (mixed)
- [ ] Error scenarios

### Test Coverage Target
- [ ] All public functions tested
- [ ] All error paths tested
- [ ] Edge cases covered
- [ ] Integration tests pass

---

## Progress Tracking

### Daily Checklist

**Day 1** (4-6h):
- [ ] Complete Task 8 (PropertyBankDelta)
- [ ] Complete Task 1 (DiscoveryContext)
- [ ] Start Task 2 (File scanning)

**Day 2** (6-8h):
- [ ] Complete Task 2 (File scanning)
- [ ] Complete Task 3 (DB query)
- [ ] Complete Task 4 (Deleted detection)

**Day 3** (6-8h):
- [ ] Complete Task 5 (Branching)
- [ ] Complete Task 6 (discover() method)
- [ ] Complete Task 7 (Builder update)

**Day 4** (2-4h):
- [ ] Write integration tests
- [ ] Fix any issues
- [ ] Final review and cleanup

---

## Definition of Done (Phase 5)

- [ ] All 8 tasks completed
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Documentation complete
- [ ] Committed to git
- [ ] Ready for Phase 6

---

## Notes & Decisions

### Open Questions
1. Path API: How to get schema directory from config? ✅ Use `config.paths().schema`
2. SchemaId generation: UUID v4 confirmed ✅
3. Error handling: Fail-fast confirmed ✅

### Deferred Items
- Parallel processing (can add later)
- Advanced error recovery (not needed yet)
- Performance optimization (premature)

### Helpful Commands

```bash
# Run all tests
mise run test

# Run just Discovery tests
cargo test schema_pipeline::tests::discover

# Run integration tests
cargo test --test schema_discovery

# Check clippy
mise run lint

# Format code
mise run fmt

# Full verification
mise run verify
```
