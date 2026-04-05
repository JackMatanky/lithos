# Schema Pipeline Batch Refactor - Complete Implementation Plan

**Date**: 2026-04-05
**Status**: PLANNING
**Estimated Effort**: 3-5 days
**Goal**: Implement batch processing with granular stages and incremental construction

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Problems with Current Implementation](#problems-with-current-implementation)
3. [Proposed Architecture](#proposed-architecture)
4. [Implementation Phases](#implementation-phases)
5. [New Type Definitions](#new-type-definitions)
6. [Stage Specifications](#stage-specifications)
7. [Migration Strategy](#migration-strategy)
8. [Testing Strategy](#testing-strategy)
9. [Definition of Done](#definition-of-done)

---

## Executive Summary

### Current Issues

1. **No batch processing**: Processes files one-by-one in loops instead of batching by status
2. **Rigid pipeline**: Difficult to add new stages or modify flow
3. **Mixed responsibilities**: Builder doesn't own discovery, pipeline does too much
4. **No granular status tracking**: Large enum variants instead of stage-specific payloads
5. **Missing incremental construction**: Always does full merge, can't optimize for property-only changes

### Proposed Solution

1. **Builder handles discovery**: Loads graph from DB, lists files, returns `DiscoveryContext`
2. **Batch processing**: Group schemas by status for efficient processing
3. **Granular stages**: Each stage has specific payload structs
4. **Branching enums**: Type-safe transitions between stages
5. **Incremental construction**: Uses `ExtendsChangeKind` to optimize merges

---

## Problems with Current Implementation

### 1. No Clear Separation of Concerns

**Current**:
```rust
// Builder::load_schemas_v2() - line 250
let context = self.discovery()?;
let discovered = SchemaTreeProcessor::new(Unknown)
    .discover_with_context(context, &self.source, &self.repository)?;
```

**Problem**: Builder creates context, but pipeline does discovery logic

**Proposed**:
```rust
// Builder owns discovery completely
let context = self.discovery()?;  // Loads graph, lists files
let processor = SchemaTreeProcessor::from_context(context);
```

### 2. FileStatus is Too Large

**Current**:
```rust
enum FileStatus {
    Fresh { id, path, view },
    StaleTimestamps { id, path, view, times },
    StaleContent { id, path, view, raw, content_hash, times },
    New { id, path, raw, content_hash, times },
}
```

**Problem**: Single enum with all fields for all stages

**Proposed**: Stage-specific payloads
```rust
// Discovery stage
struct MissingPayload { path, times }

// TimeComparison stage
struct PresentPayload { times, view }

// ContentComparison stage
struct StaleSuspectPayload { times, content_str, view }
```

### 3. No Batch Processing

**Current**:
```rust
for path in &files {
    let status = /* determine status */;
    statuses.insert(id, status);
}
```

**Problem**: Processes one file at a time, can't batch new schemas

**Proposed**:
```rust
// Batch new schemas
let new_schemas: HashMap<SchemaId, NewSchema> = /* ... */;

// Batch stale schemas
let stale_schemas: HashMap<SchemaId, StaleSchema> = /* ... */;

// Process batches separately
```

### 4. No Incremental Construction

**Current**:
```rust
// Always does full merge
let merged = merge_properties(
    Some(&parent_props),
    &expanded.properties,
    &expanded.excludes,
);
```

**Problem**: Can't optimize for property-only changes

**Proposed**:
```rust
match extends_change_kind {
    ExtendsChangeKind::Unchanged => {
        // Fetch from DB, update properties only
    }
    ExtendsChangeKind::Rewired => {
        // Full merge required
    }
    // ...
}
```

---

## Proposed Architecture

### High-Level Flow

```
Builder::discovery()
    ↓
SchemaProcessor<Discovery>
    ↓ (branch)
    ├─→ Missing → FileParsed → InheritanceGraphed → Construction (Full)
    └─→ Present → TimeComparison
                      ↓ (branch)
                      ├─→ Fresh → InheritanceGraphed → Construction (Fetch)
                      └─→ Mismatch → ContentComparison
                                         ↓ (branch)
                                         ├─→ StaleTimestamp → Refresh → Construction (Fetch)
                                         └─→ StaleContent → FileParsed → InheritanceGraphed
                                                                              ↓
                                                                         PropertyAnalysis
                                                                              ↓
                                                                         Construction (Merge/Update)
```

### Stage Ownership

| Stage                | Owner                | Responsibility                          |
| -------------------- | -------------------- | --------------------------------------- |
| Discovery            | Builder              | Load graph, list files                  |
| TimeComparison       | SchemaProcessor      | Check timestamps                        |
| ContentComparison    | SchemaProcessor      | Check content hash                      |
| FileParsed           | SchemaProcessor      | Parse raw schemas                       |
| InheritanceGraphed   | SchemaProcessor      | Build/patch graph, detect cycles       |
| PropertyAnalysis     | SchemaProcessor      | Detect property/excludes changes        |
| Refresh              | SchemaProcessor      | Update views for unchanged semantics    |
| Construction         | SchemaProcessor      | Build schemas (Full/Merge/Update/Fetch) |
| Completion           | SchemaProcessor      | Persist graph and schemas               |

---

## Implementation Phases

### Phase 1: Foundation Types (Day 1 - 4 hours)

**Goal**: Add new payload structs and enums to `graph.rs` and `schema_pipeline.rs`

**Files**:
- `lithos-core/src/schema/graph.rs`
- `lithos-core/src/schema/schema_pipeline.rs`

**Tasks**:
1. Add `ExtendsChangeKind` enum
2. Add `ExcludesDelta` struct
3. Add `SchemaPropertyDelta` struct
4. Add `SchemaPropertyUpserts` struct
5. Add payload structs:
   - `MissingPayload`
   - `PresentPayload`
   - `StaleSuspectPayload`
   - `DeletedPayload`
   - `FreshPayload` (simplified)
   - `StaleTimestampPayload`
   - `StaleContentSuspectPayload`
   - `NewPayload`

**Definition of Done**:
- [ ] All types compile
- [ ] All types have `Debug`, `Clone`, `PartialEq` derives
- [ ] Documentation for each type
- [ ] Unit tests for delta structs

### Phase 2: Builder Discovery (Day 1 - 4 hours)

**Goal**: Move discovery logic to Builder

**Files**:
- `lithos-core/src/schema/builder.rs`

**Tasks**:
1. Implement `Builder::discovery()`:
   - Load graph from DB via `repository.get_topological_graph()`
   - Scan schema directory using `source.list_files()`
   - Filter for schema extensions
   - Detect property bank file
   - Return `DiscoveryContext`
2. Update `load_schemas_v2()` to use new discovery

**Definition of Done**:
- [ ] `discovery()` compiles and has tests
- [ ] Returns graph + file list + has_property_bank flag
- [ ] Existing tests still pass

### Phase 3: Discovery Stage (Day 2 - 6 hours)

**Goal**: Implement Discovery stage with branching

**Files**:
- `lithos-core/src/schema/schema_pipeline.rs`

**Tasks**:
1. Create `Discovery` stage marker
2. Create `DiscoveryState` struct
3. Implement `discover_with_context()`:
   - Query DB for views by paths
   - Classify as Missing, Present, or Deleted
   - Build batches: `HashMap<SchemaId, MissingPayload>`, etc.
   - Embed payloads in graph nodes
4. Create branching enum:
   ```rust
   enum DiscoveryBranch {
       Missing(SchemaProcessor<FileParsed, MissingState>),
       Present(SchemaProcessor<TimeComparison, PresentState>),
   }
   ```

**Definition of Done**:
- [ ] Discovery stage compiles
- [ ] Returns branching enum
- [ ] Tests for Missing/Present/Deleted classification
- [ ] Integration test for full discovery flow

### Phase 4: TimeComparison & ContentComparison Stages (Day 2 - 4 hours)

**Goal**: Implement timestamp and content hash comparison

**Files**:
- `lithos-core/src/schema/schema_pipeline.rs`

**Tasks**:
1. Create `TimeComparison` stage marker
2. Implement `compare_timestamps()`:
   - Check `RawFileTimes` against view
   - Branch to Fresh or StaleSuspect
3. Create `ContentComparison` stage marker
4. Implement `compare_content()`:
   - Hash file content
   - Compare against view
   - Branch to StaleTimestamp or StaleContent

**Definition of Done**:
- [ ] Both stages compile
- [ ] Branching enums defined
- [ ] Tests for timestamp match/mismatch
- [ ] Tests for content hash match/mismatch

### Phase 5: FileParsed Stage (Day 3 - 4 hours)

**Goal**: Parse raw schemas for new/stale content

**Files**:
- `lithos-core/src/schema/schema_pipeline.rs`

**Tasks**:
1. Create `FileParsed` stage marker
2. Implement `parse_files()`:
   - Parse Missing batch (new schemas)
   - Parse StaleContentSuspect batch
   - Expand properties early for new schemas
   - Save `RawSchemaView` for new schemas
3. Update graph nodes with parsed data

**Definition of Done**:
- [ ] FileParsed stage compiles
- [ ] Batch parsing works
- [ ] Tests for new schema parsing
- [ ] Tests for stale schema parsing

### Phase 6: InheritanceGraphed Stage (Day 3 - 6 hours)

**Goal**: Build/patch graph and validate structure

**Files**:
- `lithos-core/src/schema/schema_pipeline.rs`
- `lithos-core/src/schema/graph.rs`

**Tasks**:
1. Create `InheritanceGraphed` stage marker
2. Implement `graph_structure()`:
   - Handle NewGraph branch (all schemas new)
   - Handle PatchGraph branch (graph exists)
   - Detect extends changes via `ExtendsChangeKind`
   - Update graph structure
   - Validate topological sort
   - Detect cycles
3. Embed `ExtendsChangeKind` in payloads

**Definition of Done**:
- [ ] InheritanceGraphed stage compiles
- [ ] NewGraph branch works
- [ ] PatchGraph branch works
- [ ] Extends change detection works
- [ ] Cycle detection tests
- [ ] Integration test for graph patching

### Phase 7: PropertyAnalysis Stage (Day 4 - 6 hours)

**Goal**: Detect semantic changes in properties/excludes

**Files**:
- `lithos-core/src/schema/schema_pipeline.rs`

**Tasks**:
1. Create `PropertyAnalysis` stage marker
2. Implement `analyze_properties()`:
   - Compare excludes against view → `ExcludesDelta`
   - Hash properties and compare → `SchemaPropertyDelta`
   - Check PropertyBankDelta against bank_references
   - Classify as: Unchanged (→ Refresh) or Changed (→ Construction)
3. Embed deltas in payloads

**Definition of Done**:
- [ ] PropertyAnalysis stage compiles
- [ ] Excludes delta detection works
- [ ] Property delta detection works
- [ ] PropertyBank delta checking works
- [ ] Tests for all delta types

### Phase 8: Refresh Stage (Day 4 - 3 hours)

**Goal**: Update views for unchanged semantics

**Files**:
- `lithos-core/src/schema/schema_pipeline.rs`

**Tasks**:
1. Keep existing `Refresh` stage
2. Handle StaleTimestamp batch
3. Handle StaleContent (unchanged) batch
4. Update `RawSchemaView` with new hashes/times
5. Persist views to DB

**Definition of Done**:
- [ ] Refresh stage compiles
- [ ] Batch refresh works
- [ ] Views persisted correctly
- [ ] Tests for view updates

### Phase 9: Construction Stage (Day 5 - 8 hours)

**Goal**: Incremental construction based on `ExtendsChangeKind`

**Files**:
- `lithos-core/src/schema/schema_pipeline.rs`

**Tasks**:
1. Refactor `construct_schemas()`:
   - **Full** branch: New schemas (root or child)
   - **Merge** branch: `ExtendsChangeKind::Rewired` or `RootToChild`
   - **Update** branch: `ExtendsChangeKind::Unchanged` + property delta
   - **Fetch** branch: Fresh schemas
2. Implement batch processing for each branch
3. Walk graph in topological order
4. Cache constructed schemas for children

**Definition of Done**:
- [ ] All construction branches work
- [ ] Batch processing works
- [ ] Tests for Full construction
- [ ] Tests for Merge construction
- [ ] Tests for Update construction
- [ ] Tests for Fetch construction
- [ ] Integration test for full pipeline

### Phase 10: Integration & Testing (Day 5 - 4 hours)

**Goal**: End-to-end testing and cleanup

**Files**:
- `lithos-core/tests/schema_batch_pipeline.rs` (new)
- `lithos-core/src/schema/builder.rs`

**Tasks**:
1. Write integration tests:
   - First run (all new)
   - Second run (all fresh)
   - Property change only
   - Extends change
   - Mixed changes
2. Update Builder integration
3. Clean up old code
4. Update documentation

**Definition of Done**:
- [ ] All 862+ tests pass
- [ ] Zero clippy warnings
- [ ] Integration tests cover all branches
- [ ] Documentation updated
- [ ] ADR created

---

## New Type Definitions

### Core Enums

```rust
/// Tracks how schema inheritance changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExtendsChangeKind {
    /// Parent unchanged.
    Unchanged,
    /// Schema gained a parent (was root).
    RootToChild,
    /// Schema became root (lost parent).
    ChildToRoot,
    /// Schema changed parents.
    Rewired,
}

impl ExtendsChangeKind {
    /// Check if this change requires full merge.
    pub(crate) fn requires_merge(&self) -> bool {
        matches!(self, Self::Rewired | Self::RootToChild)
    }

    /// Check if this change can use incremental update.
    pub(crate) fn can_update(&self) -> bool {
        matches!(self, Self::Unchanged | Self::ChildToRoot)
    }
}
```

### Delta Structs

```rust
/// Delta of excludes list changes.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ExcludesDelta {
    pub(crate) added: Vec<PropertyName>,
    pub(crate) removed: Vec<PropertyName>,
}

impl ExcludesDelta {
    pub(crate) fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

/// Delta of property changes.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyDelta {
    pub(crate) upserts: SchemaPropertyUpserts,
    pub(crate) removed: Vec<PropertyName>,
}

impl SchemaPropertyDelta {
    pub(crate) fn is_empty(&self) -> bool {
        self.upserts.inline.is_empty()
            && self.upserts.refs.is_empty()
            && self.removed.is_empty()
    }
}

/// Property upserts (inline or refs).
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SchemaPropertyUpserts {
    pub(crate) inline: HashMap<PropertyName, RawPropertyInline>,
    pub(crate) refs: HashMap<PropertyName, RawPropertyRef>,
}
```

### Payload Structs

```rust
/// Payload for Missing status (Discovery stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MissingPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
}

/// Payload for Present status (Discovery stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PresentPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) view: RawSchemaView,
}

/// Payload for StaleSuspect status (TimeComparison stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleSuspectPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_str: Box<str>,
    pub(crate) view: RawSchemaView,
}

/// Payload for Deleted status (Discovery stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DeletedPayload {
    pub(crate) is_deleted: bool,
}

/// Payload for Fresh status (TimeComparison stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FreshPayload {
    pub(crate) path: PathBuf,
    pub(crate) view: RawSchemaView,
}

/// Payload for StaleTimestamp status (ContentComparison stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleTimestampPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) view: RawSchemaView,
}

/// Payload for StaleContentSuspect status (ContentComparison stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleContentSuspectPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: RawSchemaView,
}

/// Payload for New status (FileParsed stage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
}

/// Payload for InheritanceGraphed stage.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GraphedPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: Option<RawSchemaView>,
    pub(crate) extends_change: ExtendsChangeKind,
}

/// Payload for PropertyAnalysis stage.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnalyzedPayload {
    pub(crate) path: PathBuf,
    pub(crate) times: RawFileTimes,
    pub(crate) content_hash: [u8; 32],
    pub(crate) raw: RawSchema,
    pub(crate) view: RawSchemaView,
    pub(crate) extends_change: ExtendsChangeKind,
    pub(crate) excludes_delta: Option<ExcludesDelta>,
    pub(crate) property_delta: Option<SchemaPropertyDelta>,
}
```

---

## Stage Specifications

### Discovery Stage

**Input**: `DiscoveryContext` from Builder

**Process**:
1. Query DB for views by paths: `repository.find_raw_schema_views_by_paths(&files)`
2. For each file:
   - If view exists → `Present` (has `RawSchemaView`)
   - If view missing → `Missing` (new schema)
3. For each ID in graph with no corresponding file:
   - Mark as `Deleted`

**Output**: Branching enum

```rust
pub(crate) enum DiscoveryBranch {
    Missing(SchemaProcessor<FileParsed, MissingBatch>),
    Present(SchemaProcessor<TimeComparison, PresentBatch>),
}

pub(crate) struct MissingBatch {
    graph: TopologicalGraph<GraphNode<MissingPayload>>,
    batch: HashMap<SchemaId, MissingPayload>,
}

pub(crate) struct PresentBatch {
    graph: TopologicalGraph<GraphNode<PresentPayload>>,
    batch: HashMap<SchemaId, PresentPayload>,
}
```

### TimeComparison Stage

**Input**: `PresentBatch`

**Process**:
1. For each schema in batch:
   - Compare `times` with `view.current().file_times()`
   - If match → `Fresh`
   - If mismatch → `StaleSuspect` (read file content)

**Output**: Branching enum

```rust
pub(crate) enum TimestampBranch {
    Match(SchemaProcessor<InheritanceGraphed, FreshBatch>),
    Mismatch(SchemaProcessor<ContentComparison, StaleSuspectBatch>),
}
```

### ContentComparison Stage

**Input**: `StaleSuspectBatch`

**Process**:
1. For each schema in batch:
   - Hash `content_str` with blake3
   - Compare with `view.current().hashes().content()`
   - If match → `StaleTimestamp`
   - If mismatch → `StaleContentSuspect`

**Output**: Branching enum

```rust
pub(crate) enum ContentBranch {
    Match(SchemaProcessor<Refresh, StaleTimestampBatch>),
    Mismatch(SchemaProcessor<FileParsed, StaleContentBatch>),
}
```

### FileParsed Stage

**Input**: `MissingBatch` or `StaleContentBatch`

**Process**:
1. For each schema:
   - Parse file content → `RawSchema`
   - For new schemas:
     - Expand properties
     - Build `RawSchemaView`
     - Save view to DB
   - For stale content:
     - Parse to `RawSchema`

**Output**: Proceed to `InheritanceGraphed`

### InheritanceGraphed Stage

**Input**: Mixed batch from `FileParsed` or `Fresh`

**Process**:
1. If no graph exists (NewGraph):
   - Build graph from scratch using `DagBuilder`
   - All schemas marked as `ExtendsChangeKind::Unchanged` (they're new)
2. If graph exists (PatchGraph):
   - For new schemas: Insert into graph
   - For stale content: Detect extends changes
     - Compare `node.parents` with `raw.extends` → `ExtendsChangeKind`
     - Update graph structure if needed
3. Validate topological sort
4. Detect cycles

**Output**: Proceed to `PropertyAnalysis` (for stale) or `Construction` (for new/fresh)

### PropertyAnalysis Stage

**Input**: Batch with `ExtendsChangeKind`

**Process**:
1. For each schema:
   - Compare `raw.excludes` with `view.excludes()` → `ExcludesDelta`
   - Hash `raw.properties` and compare with `view.hashes().properties()` → `SchemaPropertyDelta`
   - Check `PropertyBankDelta` against `view.bank_references()`
2. Classify:
   - If all unchanged → Send to `Refresh`
   - If any changed → Send to `Construction` with deltas

**Output**: Branching to `Refresh` or `Construction`

### Refresh Stage

**Input**: Batch with unchanged semantics

**Process**:
1. For each schema:
   - Build new `SchemaVersion` with updated times/hashes
   - Add version to view
   - Save view to DB

**Output**: Schemas marked as Fresh, send to `Construction` (Fetch branch)

### Construction Stage

**Input**: All schemas, classified by construction type

**Branches**:

1. **Full** (new schemas):
   - Root: Expand properties, build Schema
   - Child: Expand properties, inherit from parents, build Schema

2. **Merge** (`ExtendsChangeKind::Rewired` or `RootToChild`):
   - Expand properties
   - Inherit from new parents
   - Merge with excludes
   - Build Schema

3. **Update** (`ExtendsChangeKind::Unchanged` + property delta):
   - Fetch existing Schema from DB
   - Expand changed properties only
   - Update Schema.properties
   - Save Schema

4. **Fetch** (Fresh schemas):
   - Fetch Schema from DB
   - No changes needed

**Process**:
1. Walk graph in topological order
2. Cache constructed parents for children
3. Save schemas to DB
4. Build `RawSchemaView` for StaleContent

**Output**: Vec<Arc<Schema>>

---

## Migration Strategy

### Step 1: Add New Types Without Breaking Existing Code

- Add all new structs/enums to `graph.rs` and `schema_pipeline.rs`
- Keep existing `FileStatus` enum unchanged
- All new types compile independently

### Step 2: Implement Builder::discovery() Alongside Existing Code

- Add `Builder::discovery()` method
- Keep existing `load_schemas_v2()` working
- Tests can use either path

### Step 3: Implement New Stages One at a Time

- Start with `Discovery` stage
- Can branch back to old code path for testing
- Gradually replace old stages

### Step 4: Integration Point

- Once all stages implemented, switch Builder to use new path
- Remove old code
- Update all tests

### Step 5: Cleanup

- Remove old `FileStatus` enum
- Remove old stage methods
- Update documentation

---

## Testing Strategy

### Unit Tests (Per Phase)

**Phase 1**: Delta struct tests
- `ExcludesDelta::is_empty()`
- `SchemaPropertyDelta::is_empty()`
- `ExtendsChangeKind::requires_merge()`

**Phase 2**: Builder discovery tests
- Graph loaded from DB
- Files listed correctly
- Property bank detected

**Phase 3-9**: Stage tests
- Each stage has tests for:
  - Batch processing
  - Branching logic
  - Payload transformations

### Integration Tests

**File**: `lithos-core/tests/schema_batch_pipeline.rs`

**Scenarios**:
1. **First run** (all new)
   - No graph in DB
   - All schemas go through Full construction
   - Graph persisted

2. **Second run** (all fresh)
   - Graph loaded from DB
   - All schemas marked Fresh
   - Fetch construction only

3. **Property change only**
   - One schema property changed
   - Uses Update construction
   - No graph rebuild

4. **Extends change**
   - Schema inheritance changed
   - Uses Merge construction
   - Graph patched

5. **Mixed changes**
   - Some new, some stale, some fresh
   - All branches exercised

6. **PropertyBank change**
   - PropertyBank updated
   - Affected schemas rebuilt
   - Unaffected schemas stay fresh

### Performance Tests

**Benchmarks**:
- Discovery stage (with/without graph)
- Batch processing vs one-by-one
- Full vs Update construction
- Graph patching vs rebuild

**Targets**:
- Discovery: < 10ms for 100 schemas
- Batch processing: 10x faster than one-by-one
- Update construction: 5x faster than Full

---

## Definition of Done

### Code Quality

- [ ] All 862+ tests pass
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings
- [ ] All public APIs documented
- [ ] All stages have unit tests
- [ ] Integration tests cover all branches

### Functionality

- [ ] Builder::discovery() works
- [ ] All stages implemented
- [ ] Batch processing works
- [ ] Incremental construction works
- [ ] PropertyBank integration works
- [ ] Graph patching works
- [ ] Cycle detection works

### Performance

- [ ] Benchmarks show improvement
- [ ] No regressions in test times
- [ ] Memory usage acceptable

### Documentation

- [ ] ADR created for batch refactor
- [ ] Stage flow diagram added
- [ ] Code comments updated
- [ ] README updated

---

## Next Steps

1. **Review this plan** with user
2. **Start Phase 1**: Add foundation types
3. **Implement phases sequentially**
4. **Test after each phase**
5. **Integrate when all phases complete**

---

## Questions to Resolve

1. Should we use `Box<str>` for file content or `String`?
2. Should batches be `HashMap` or `Vec`?
3. Should we add a status enum to GraphNode for tracking?
4. How to handle deleted schemas in graph?
5. Should PropertyBankDelta checking happen in TimeComparison or PropertyAnalysis?

---

## Estimated Timeline

| Phase | Duration | Deliverable                    |
| ----- | -------- | ------------------------------ |
| 1     | 4 hours  | Foundation types               |
| 2     | 4 hours  | Builder discovery              |
| 3     | 6 hours  | Discovery stage                |
| 4     | 4 hours  | Time/Content comparison        |
| 5     | 4 hours  | FileParsed stage               |
| 6     | 6 hours  | InheritanceGraphed stage       |
| 7     | 6 hours  | PropertyAnalysis stage         |
| 8     | 3 hours  | Refresh stage                  |
| 9     | 8 hours  | Construction stage             |
| 10    | 4 hours  | Integration & testing          |
| **Total** | **49 hours** | **Complete batch refactor** |

**Estimated Calendar Time**: 3-5 days (depending on interruptions and testing cycles)
