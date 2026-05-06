# Schema Pipeline: Two-Level Typestate Architecture (Final)

**Date**: 2026-03-30
**Status**: **IMPLEMENTATION READY**

## Overview

This design implements a two-level typestate state machine for schema ingestion:
1. **Global Level**: `SchemaProcessor<Stage, Status>` - Directory-wide orchestration
2. **Local Level**: `SchemaFileProcessor<Stage, Status>` - Individual file lifecycle

## Global Processor: `SchemaProcessor<Stage, Status>`

### Stages
- `Discovery`: Tree and file identification
- `Graphing`: Building or patching the topological graph
- `Analysis`: Property-level delta coordination
- `Construction`: Final topological merge
- `Completion`: Persistence and delivery

### Statuses (Discovery Stage)
```rust
pub struct Unknown; // Entry state (replaces Booting)

pub struct TreeFound {
    pub path_index: HashMap<PathBuf, SchemaId>,
    pub tree: TopologicalGraph<SchemaFileProcessor<Discovery, Unknown>>,
}

pub struct TreeMissing {
    pub files: Vec<PathBuf>,
}
```

### Statuses (Other Stages)
```rust
pub struct TreeGraphed {
    pub graph: TopologicalGraph<SchemaFileProcessor<PropertyAnalysis, ...>>,
}

pub struct TreeAnalyzed {
    pub graph: TopologicalGraph<SchemaFileProcessor<Construction, ...>>,
}

pub struct TreeConstructed {
    pub schemas: Vec<Arc<Schema>>,
}
```

## Local Processor: `SchemaFileProcessor<Stage, Status>`

### Stages
1. `Discovery`: Determine file identity
2. `Comparison`: Timestamp/hash staleness checks
3. `PropertyAnalysis`: Parse and compute deltas (extends, excludes, properties)
4. `Refresh`: Early-commit metadata updates
5. `Construction`: Build final `Schema`

### Statuses

**Discovery Stage:**
```rust
pub struct Unknown;

pub struct FilePresent {
    pub times: RawFileTimes,
    pub view: RawSchemaView,
}

pub struct FileMissing {
    pub times: RawFileTimes,
}

pub struct FileNew {
    pub times: RawFileTimes,
}
```

**Comparison Stage:**
```rust
pub struct Suspect {
    pub times: RawFileTimes,
    pub view: RawSchemaView,
    pub content: String,
}

pub struct StaleTimestamps {
    pub times: RawFileTimes,
    pub view: RawSchemaView,
}

pub struct Fresh; // Can fetch from DB
```

**PropertyAnalysis Stage:**
```rust
pub struct Parsed {
    pub raw: RawSchema,
    pub times: RawFileTimes,
    pub view: Option<RawSchemaView>,
}

// Analysis outputs
pub struct StaleContent {
    pub times: RawFileTimes,
    pub view: RawSchemaView,
    pub content_hash: [u8; 32],
}

pub struct Changed {
    pub raw: RawSchema,
    pub extends_delta: ExtendsDelta,
    pub property_delta: SchemaPropertyDelta,
    pub excludes_delta: ExcludesDelta,
}
```

**Construction Stage:**
```rust
pub struct Ready {
    pub schema: Arc<Schema>,
    pub is_changed: bool,
}
```

## Processing Flow

### Phase 1: Global Discovery
```
SchemaProcessor::<Discovery, Unknown>::new()
  → discover_tree(source, repo, path)
    ├─ TreeFound: Has graph cache
    │   → initiate_local_discovery()
    │   → TopologicalGraph<SchemaFileProcessor<Discovery, Unknown>>
    └─ TreeMissing: No graph cache
        → skip to full build
```

### Phase 2: Global Graphing
```
SchemaProcessor::<Graphing, TreeFound/TreeMissing>
  → graph_structure()
    ├─ TreeFound: Patch existing graph (parallel Comparison on nodes)
    └─ TreeMissing: Build from scratch (all nodes are New)
  → Result: TreeGraphed with validated structure (cycles, depth)
```

### Phase 3: Global Analysis
```
SchemaProcessor::<Analysis, TreeGraphed>
  → analyze_properties()
    → Parallel PropertyAnalysis on all nodes
    → Refresh metadata for stale nodes
  → Result: TreeAnalyzed
```

### Phase 4: Global Construction
```
SchemaProcessor::<Construction, TreeAnalyzed>
  → construct_schemas()
    → Topological iteration (parents before children)
    → Local Construction on each node
  → Result: TreeConstructed
```

## Key Design Points

1. **No InheritanceAnalysis Stage**: `extends` is extracted during PropertyAnalysis alongside properties.
2. **Shared Parsing**: `RawSchema` is parsed once and passed through `Parsed` → `Changed` → `Construction`.
3. **Parallel Execution**: `Comparison` and `PropertyAnalysis` use `rayon::par_iter()`.
4. **Early Refresh**: `StaleTimestamps` and `StaleContent` nodes persist metadata immediately to avoid re-parsing on retry.
5. **Fail-Fast Validation**: Cycle detection and depth limits are enforced in the `Graphing` stage before any property work.

## Delta Structures

```rust
pub struct ExtendsDelta {
    pub old_parent: Option<SchemaName>,
    pub new_parent: Option<SchemaName>,
}

pub struct SchemaPropertyDelta {
    pub upserts: HashMap<PropertyName, RawPropertyInline | RawPropertyRef>,
    pub removed: Vec<PropertyName>,
}

pub struct ExcludesDelta {
    pub added: Vec<PropertyName>,
    pub removed: Vec<PropertyName>,
}
```

## Implementation Notes

- Follow the `PropertyBankProcessor` pattern: use `#[must_use]` on all transition methods.
- Use branching enums (e.g., `DiscoveryBranch::Found | Missing`) for fan-out logic.
- Store the `TopologicalGraph` inside the global processor's status structs.
- Ensure all `Arc<Schema>` references are skipped during `rkyv` serialization for DB caching.
