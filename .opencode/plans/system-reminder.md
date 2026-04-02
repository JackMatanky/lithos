# Schema Pipeline Single State Machine Refactor Plan (Comprehensive)

## Objective
Refactor the schema ingestion pipeline into a single batch state machine with
per-file status enums. Remove `SchemaFileProcessor`, implement graph patching
with affected-subtree recomputation and stable order preservation, and handle
deletions during Discovery. Maintain all existing functionality and align with
the original redesign intent while keeping the system extensible.

## Scope and Constraints
- Scope: `lithos-core/src/schema/` only
- Files likely touched:
  - `lithos-core/src/schema/schema_pipeline.rs`
  - `lithos-core/src/schema/builder.rs`
  - `lithos-core/src/schema/storage.rs` (only if missing APIs)
  - Tests under `lithos-core/src/schema/` and `lithos-core/tests/`
- Keep `TopologicalGraph<NodeStatus>` as the persisted graph type
- Extends delta computed in TreeGraphed
- Excludes delta computed in PropertyAnalysis
- Deletions handled during Discovery (not Completion)
- Stable topo order: preserve order for unaffected nodes

## Non-Goals
- No changes outside schema module
- No new external dependencies
- No reformat/rename unrelated code

## Current Issues to Fix
- `SchemaFileProcessor` is largely unused but still complex
- Graphing redoes comparison and parsing logic
- Extends delta computed too late
- PropertyBank delta not integrated into schema changes
- StaleContent refresh uses cached raw, not newly parsed raw
- Deletions not handled

## Target Architecture

### Single Processor
`SchemaTreeProcessor<Stage, Status>` is the only typestate driver.

### Per-File Status Enum
```rust
enum FileStatus {
    Fresh {
        id: SchemaId,
        path: PathBuf,
        view: RawSchemaView,
    },
    StaleTimestamps {
        id: SchemaId,
        path: PathBuf,
        view: RawSchemaView,
        times: RawFileTimes,
    },
    StaleContent {
        id: SchemaId,
        path: PathBuf,
        view: RawSchemaView,
        raw: RawSchema,
        content_hash: [u8; 32],
        times: RawFileTimes,
    },
    New {
        id: SchemaId,
        path: PathBuf,
        raw: RawSchema,
        content_hash: [u8; 32],
        times: RawFileTimes,
    },
}
```

### Stage Sequence
Discovery -> Comparison -> TreeGraphed -> PropertyAnalysis -> Refresh -> Construction -> Completion

### Stage Payloads
```rust
struct DiscoveryState {
    files: Vec<PathBuf>,
    cached_graph: Option<TopologicalGraph<NodeStatus>>,
    id_by_path: HashMap<PathBuf, SchemaId>,
    view_by_id: HashMap<SchemaId, RawSchemaView>,
    deleted_ids: Vec<SchemaId>,
}

struct ComparisonState {
    statuses: Vec<FileStatus>,
    fresh_ids: Vec<SchemaId>,
    stale_ids: Vec<SchemaId>,
    new_ids: Vec<SchemaId>,
}

struct TreeGraphedState {
    graph: TopologicalGraph<NodeStatus>,
    raw_by_id: HashMap<SchemaId, RawSchema>,
    extends_deltas: HashMap<SchemaId, ExtendsDelta>,
    affected_subtrees: HashSet<SchemaId>,
}

struct PropertyAnalysisState {
    graph: TopologicalGraph<NodeStatus>,
    deltas_by_id: HashMap<SchemaId, SchemaPropertyDelta>,
    excludes_by_id: HashMap<SchemaId, ExcludesDelta>,
}

struct ConstructionState {
    graph: TopologicalGraph<NodeStatus>,
    resolved: Vec<Arc<Schema>>,
    changed_ids: HashSet<SchemaId>,
}

struct CompletionState {
    schemas: Vec<Arc<Schema>>,
    graph: TopologicalGraph<NodeStatus>,
}
```

## Detailed Implementation Steps

### Step 1: Remove SchemaFileProcessor
- Delete the `SchemaFileProcessor` type and its typestates
- Remove local stage enums:
  - `LocalComparisonBranch`, `LocalTimestampBranch`, `LocalContentBranch`, `LocalAnalysisBranch`
  - `Fresh`, `Suspect`, `StaleTimestamps`, `StaleContent`, `Parsed`, `Changed`, `Ready`, etc.
- Remove local pipeline functions:
  - `discover`, `check_timestamps`, `check_content`, `analyze`, `sync_metadata`, `construct`
- Remove any use sites for local types in `schema_pipeline.rs`

### Step 2: Add New Status Types and Stage Payloads
- Add the `FileStatus` enum
- Add the new stage payload structs
- Update `SchemaTreeProcessor` transitions to use new payloads

### Step 3: Discovery Stage (includes deletion cleanup)
**Responsibilities:**
- Scan FS for schema files (exclude property bank filename)
- Load cached graph from DB
- Load path->id pairs from DB
- Load raw schema views in batch
- Compute deleted ids: ids in DB not present on disk

**Deletion cleanup during Discovery:**
- Delete by id:
  - Raw schema views
  - Schemas
  - Inheritance metadata
  - Graph cache nodes
- Ensure deleted ids are excluded from all later steps

**Outputs:**
- `DiscoveryState` with files, cached graph, maps, deleted ids

**Edge cases:**
- Missing cached graph: treat as None
- DB has ids for missing paths: treat as deletions

### Step 4: Comparison Stage (per file)
For each file path:
- Look up SchemaId (existing or new)
- Load view if present
- Collect file times

Decision logic:
- No view: parse once -> `New`
- Timestamps match: `Fresh`
- Timestamps mismatch:
  - Read content once
  - Compute content hash
  - Hash match -> `StaleTimestamps`
  - Hash mismatch -> parse once -> `StaleContent`

**Outputs:**
- `ComparisonState` with `statuses` and id sets

**Performance:**
- Parse at most once per stale file
- Avoid re-reading content later by carrying raw/hash

### Step 5: TreeGraphed Stage (graph patching, stable order)

**Inputs:**
- `cached_graph`
- `raw_by_id` for `New` and `StaleContent`
- `deleted_ids`

**Compute ExtendsDelta:**
- For each `New` or `StaleContent`:
  - `old_parent`: from view.current().extends()
  - `new_parent`: from raw.extends()
  - Save to `extends_deltas`

**Change sets:**
- `changed_extends_ids` where delta.changed()
- `new_ids`
- `affected_roots = union(changed_extends_ids, new_ids)`

**Graph patching algorithm:**
1) Start from cached graph if available, else build baseline from current files
2) Remove deleted nodes from graph, roots, order, nodes
3) Insert new nodes (parent resolved from name->id map)
4) Rewire parent edges for changed extends

**Children adjacency:**
- Build `children_by_parent` map from updated graph

**Affected subtree discovery:**
- BFS from each `affected_root` over `children_by_parent`
- Collect into `affected_subtrees`

**Scoped validation:**
- Cycle detection limited to `affected_subtrees` + their immediate parents
- Depth recompute only in `affected_subtrees`
  - Root depth = parent.depth + 1 (or 0 if no parent)
  - Propagate to descendants

**Stable topo order patch:**
- Let `old_order` = cached graph order
- Remove all affected ids from `old_order`
- Compute a topo order for the affected subgraph only
- Splice affected order into `old_order` at earliest parent position:
  - If node has parent in `old_order`, insert after last parent index
  - If root, insert at end
- Validate final order length == node count

**Outputs:**
- `TreeGraphedState` with patched graph, raw_by_id, extends_deltas, affected_subtrees

### Step 6: PropertyAnalysis (batch)

**Compute ExcludesDelta:**
- Compare `view.current().excludes()` vs `raw.excludes()`

**Compute SchemaPropertyDelta:**
- Compare raw property hashes to cached hashes
- Build `SchemaPropertyUpserts { inline, refs }` and `removed`

**Integrate PropertyBank delta:**
- Load bank references from view
- Intersect with PropertyBank changed set
- Add affected refs to `SchemaPropertyDelta.upserts.refs`

**Outputs:**
- `PropertyAnalysisState` with `deltas_by_id`, `excludes_by_id`, and graph

### Step 7: Refresh (per-file)
- For `StaleTimestamps`:
  - Update `file_times` in view
  - Persist view
- For `StaleContent`:
  - Rebuild `SchemaVersion` from `raw` (not cached raw)
  - Update file_times + content hash
  - Persist view
- Convert refreshed schemas to `Fresh` for construction

### Step 8: Construction (batch)

**Inputs:**
- Patched graph order
- Property deltas and excludes
- Expanded properties from PropertyBank for changed schemas

**Process topo order:**
- For each id:
  - If schema is Fresh and parent is Fresh -> fetch from DB
  - Else expand changed properties (inline + refs)
  - Merge parent properties + apply excludes
  - Construct Schema
  - Cache in `resolved_cache` for children

**Outputs:**
- `ConstructionState` with resolved schemas and changed ids

### Step 9: Completion
- Save changed/new schemas
- Save `TopologicalGraph<NodeStatus>` cache
- Save inheritance metadata
- Return `Vec<Schema>`

### Step 10: Cleanups
- Remove dead helpers and old branch enums
- Ensure no unused imports remain

## Repository API Needs
Verify presence of:
- `list_schema_path_id_pairs`
- `get_raw_schema_view`
- `save_raw_schema_view`
- `save_schemas`
- `find_schemas_by_ids`
- `save_topological_graph`
- `get_inheritance_metadata`
- `save_inheritance_metadata`
- Deletion APIs:
  - `delete_schema`
  - `delete_raw_schema_view`
  - `delete_inheritance_metadata`

If missing, add minimal methods in `storage.rs` and trait.

## Test Plan
Add/extend tests for:
- Deletion handling during Discovery
- Stale timestamps refresh updates view but not schema
- Stale content refresh rebuilds view from raw
- Graph patching:
  - extends change rewires edges
  - stable order preserves unaffected ids
  - affected subtree recompute only
- PropertyBank delta re-expands refs
- Full pipeline on mixed fresh/stale/new schemas

## Validation Commands
- `mise run fmt`
- `mise run lint`
- `mise run test:unit:schema`
- `mise run verify`

## Risks and Mitigations
- **Risk:** Partial topo reorder breaks parent-before-child.
  - Mitigation: validate order length and parent position constraints in tests.
- **Risk:** Missing delete APIs.
  - Mitigation: add minimal delete operations to repository trait.
- **Risk:** Complexity of patching.
  - Mitigation: instrument with helper functions and focused unit tests.

## Deliverables Checklist
- `SchemaFileProcessor` removed
- Single pipeline in `schema_pipeline.rs`
- Deletion cleanup in Discovery
- Graph patching with affected subtree recompute and stable order
- PropertyBank delta integration
- Refresh uses raw for StaleContent
- Tests updated and passing
