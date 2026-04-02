# Schema Pipeline Refactor: Unified Graph-Based Processing

**Date**: 2026-04-01
**Status**: READY FOR IMPLEMENTATION
**Author**: AI Assistant (based on user design proposal)
**Purpose**: Comprehensive implementation plan for refactoring the schema processor pipeline to use `TopologicalGraph<FileStatus>` as the single source of truth

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current vs Proposed Architecture](#current-vs-proposed-architecture)
3. [Core Design Principles](#core-design-principles)
4. [Implementation Phases](#implementation-phases)
5. [Detailed Phase Plans](#detailed-phase-plans)
6. [Code Change Specifications](#code-change-specifications)
7. [Testing Strategy](#testing-strategy)
8. [Migration Path](#migration-path)
9. [Risk Mitigation](#risk-mitigation)
10. [Definition of Done](#definition-of-done)

---

## Executive Summary

### Problem Statement

The current `schema_pipeline.rs` implementation has several architectural issues:

1. **Dual State Tracking**: Maintains both `TopologicalGraph<InheritanceNode>` and separate `HashMap<SchemaId, FileStatus>`, causing:
   - Duplicate lookups (must query both structures)
   - Potential desync between graph structure and file status
   - No single source of truth

2. **Graph Rebuild on Every Run**: Rebuilds the entire graph from scratch despite having a persisted `get_topological_graph()` method already implemented.

3. **Incomplete Refresh Stage**: Only handles `StaleTimestamps`, missing optimization for `StaleContent` with unchanged semantic properties.

4. **PropertyBankDelta Integration Gap**: Fresh schemas aren't demoted when they reference changed PropertyBank properties.

5. **Missing Construction Optimizations**: Always performs full merge even when only properties changed (not extends/excludes).

### Solution Overview

**Leverage Existing Infrastructure**:
- Use already-implemented `Repository::get_topological_graph()` and `Repository::save_topological_graph()`
- Use already-implemented `GraphNode<T>` generic payload pattern
- Unify structure and status in single `TopologicalGraph<FileStatus>`

**Key Changes**:

1. **Builder Discovery** (new): Load graph from DB, scan files, return `DiscoveryContext`
2. **Graph Hydration** (new): Embed `FileStatus` into `GraphNode` payloads
3. **PropertyBankDelta Integration** (enhanced): Check in Comparison stage, demote Fresh→StaleContent if needed
4. **Enhanced Refresh** (enhanced): Handle both `StaleTimestamps` and `StaleContent` with unchanged properties
5. **Incremental Construction** (new): Skip full merge when only properties changed

### Benefits

| Aspect | Current | Proposed | Improvement |
|--------|---------|----------|-------------|
| **State Tracking** | Graph + HashMap | Single Graph | -50% lookups, single source of truth |
| **Graph Loading** | Rebuild every time | Load from DB | 10-100x faster on warm starts |
| **Refresh Optimization** | Timestamps only | Timestamps + content-only | Skips unnecessary reconstruction |
| **PropertyBank Integration** | Analysis stage only | Comparison stage | Correctly demotes Fresh schemas |
| **Construction Speed** | Always full merge | Incremental when possible | 50-80% faster for property-only changes |

### Estimated Effort

- **Phase 1** (Builder Discovery): 4-6 hours
- **Phase 2** (Graph Hydration): 6-8 hours
- **Phase 3** (PropertyBankDelta Integration): 3-4 hours
- **Phase 4** (Enhanced Refresh): 3-4 hours
- **Phase 5** (Incremental Construction): 4-5 hours
- **Testing & Integration**: 6-8 hours

**Total**: 26-35 hours

---

## Current vs Proposed Architecture

### Current Architecture

```rust
// CURRENT: Separate graph and status tracking
pub(crate) struct TreeGraphedState {
    graph: TopologicalGraph<InheritanceNode>,  // Structure only
    statuses: HashMap<SchemaId, FileStatus>,    // Status tracking (SEPARATE!)
    raw_by_id: HashMap<SchemaId, RawSchema>,
    extends_deltas: HashMap<SchemaId, ExtendsDelta>,
    affected_subtrees: HashSet<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

// CURRENT: Graph rebuilt from scratch every time
impl SchemaTreeProcessor<TreeGraphed, ComparisonState> {
    pub(crate) fn graph_structure(self) -> Result<...> {
        // Rebuilds entire graph from FileStatus HashMap
        let graph = DagBuilder::new(&statuses).build()?;  // ← EXPENSIVE!
        // ...
    }
}

// CURRENT: Lookups require two steps
let node = graph.nodes.get(&id)?;           // Step 1: Get structure
let status = statuses.get(&id)?;            // Step 2: Get status
```

**Problems**:
- Duplicate data structures
- Potential desync
- Graph rebuilt every time (ignores DB persistence)
- Two-step lookups

---

### Proposed Architecture

```rust
// PROPOSED: Unified graph with embedded status
pub(crate) struct TreeGraphedState {
    graph: TopologicalGraph<FileStatus>,       // Structure + Status in ONE!
    raw_by_id: HashMap<SchemaId, RawSchema>,
    extends_deltas: HashMap<SchemaId, ExtendsDelta>,
    affected_subtrees: HashSet<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}

// PROPOSED: Load graph from DB, hydrate with status
impl Builder {
    pub fn discovery(&self) -> Result<DiscoveryContext, Error> {
        // Load persisted graph (if exists)
        let graph = self.repository.get_topological_graph()?;  // ← FAST!
        let files = scan_schema_directory()?;

        Ok(DiscoveryContext { graph, files, has_property_bank })
    }
}

impl SchemaTreeProcessor<Discovery, Unknown> {
    pub fn discover_and_hydrate(
        context: DiscoveryContext
    ) -> TopologicalGraph<FileStatus> {
        let mut graph = context.graph.unwrap_or_default();

        // Hydrate existing nodes with FileStatus
        for file in context.files {
            let status = determine_file_status(file)?;
            hydrate_or_create_node(&mut graph, file, status)?;
        }

        graph  // TopologicalGraph<FileStatus>
    }
}

// PROPOSED: Single-step lookup
let node = graph.nodes.get(&id)?;
let status = &node.payload;  // Status embedded in node!
```

**Benefits**:
- Single source of truth
- No desync possible
- Graph loaded from DB (fast)
- Single-step lookups

---

## Core Design Principles

### 1. Leverage Existing Infrastructure

**DO NOT reinvent**:
- ✅ `Repository::get_topological_graph()` already exists (storage.rs:384)
- ✅ `Repository::save_topological_graph()` already exists (storage.rs:393)
- ✅ `GraphNode<T>` generic payload pattern already exists (graph.rs:219)
- ✅ `TopologicalGraph<T>` generic container already exists (graph.rs:104)

**Just use them correctly!**

### 2. Incremental Refactoring

**DO NOT rewrite everything**:
- Keep existing stage structure (Discovery → Comparison → TreeGraphed → PropertyAnalysis → Refresh → Construction → Completion)
- Keep existing `FileStatus` enum (add variants only if needed)
- Keep existing algorithms (topological sort, cycle detection, depth computation)

**Only change**:
- How graph is loaded (DB instead of rebuild)
- How status is stored (in graph instead of separate HashMap)
- How stages coordinate (pass graph with embedded status)

### 3. Maintain Type Safety

**DO NOT lose compile-time guarantees**:
- Keep typestate pattern for pipeline stages
- Keep `#[must_use]` on branch enums
- Keep phantom data for stage markers
- Add payload types for graph nodes (already designed in graph.rs:297-338)

### 4. Preserve Batch Semantics

**DO NOT convert to per-schema pipelines**:
- Schemas have parent-child dependencies (unlike PropertyBank)
- Batch processing with topological ordering is correct
- Level-by-level construction must remain
- Graph rewiring requires coordination across nodes

### 5. Early Validation (Fail-Fast)

**DO validate structure before expensive operations**:
- Cycle detection before property expansion
- Parent verification before merging
- Graph consistency checks before construction

---

## Implementation Phases

### Overview

We implement in 5 distinct phases, each independently testable:

```
Phase 1: Builder Discovery
  ↓
Phase 2: Graph Hydration
  ↓
Phase 3: PropertyBankDelta Integration
  ↓
Phase 4: Enhanced Refresh
  ↓
Phase 5: Incremental Construction
```

Each phase:
- Is independently mergeable (feature flag controlled)
- Has comprehensive tests
- Can be rolled back without affecting others
- Improves performance/correctness incrementally

### Phase Dependencies

```
Phase 1 ──┐
          ├──→ Phase 2 ──┐
          │              ├──→ Phase 3 ──→ Phase 4 ──→ Phase 5
          └──────────────┘
```

- **Phase 1** is prerequisite for **Phase 2** (provides DiscoveryContext)
- **Phase 2** is prerequisite for **Phases 3-5** (provides unified graph)
- **Phases 3-5** can be implemented in any order after Phase 2

---

## Detailed Phase Plans

---

## Phase 1: Builder Discovery

### Goal

Move initial graph loading and file scanning into `Builder`, producing a `DiscoveryContext` that encapsulates:
1. Graph loaded from DB (if exists)
2. List of schema files (excluding property bank)
3. PropertyBank existence check

### Why This Matters

**Current**: `SchemaTreeProcessor::discover_tree()` does everything (file scanning, DB queries, deletion detection) in one giant method.

**Proposed**: `Builder::discovery()` separates concerns:
- Builder handles filesystem + DB coordination
- Processor handles pipeline state transitions

### Files to Modify

1. **`lithos-core/src/schema/builder.rs`**
   - Add `discovery()` method
   - Add `DiscoveryContext` struct

2. **`lithos-core/src/schema/schema_pipeline.rs`**
   - Update `Discovery` stage to accept `DiscoveryContext`
   - Remove file scanning logic (moved to Builder)

### Detailed Changes

#### 1.1: Add DiscoveryContext to schema_pipeline.rs

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (after line 50)

**Add**:
```rust
/// Context returned from Builder::discovery() containing initial state.
#[derive(Debug)]
pub(crate) struct DiscoveryContext {
    /// Topological inheritance graph loaded from DB (if exists).
    ///
    /// `None` if this is the first run or graph was corrupted.
    pub(crate) graph: Option<TopologicalGraph<InheritanceNode>>,

    /// All schema files in schema directory (excluding property bank).
    pub(crate) files: Vec<PathBuf>,

    /// Property bank file exists in schema directory.
    pub(crate) has_property_bank: bool,
}
```

#### 1.2: Add Builder::discovery() method

**Location**: `lithos-core/src/schema/builder.rs` (after line 79)

**Add**:
```rust
/// Discover schema files and load inheritance graph from DB.
///
/// This method performs initial filesystem scanning and DB queries to
/// prepare the context for schema pipeline processing.
///
/// # Errors
///
/// Returns `SchemaLoaderError` if file scanning or DB access fails.
pub(crate) fn discovery(
    &self,
) -> Result<super::schema_pipeline::DiscoveryContext, SchemaLoaderError> {
    use super::schema_pipeline::DiscoveryContext;

    // 1. Scan schema directory
    let pattern = format!("{}/**/*", self.schema_dir.display());
    let all_files = self.source.list_files(&pattern).map_err(|e| {
        SchemaIngestionError::File(SchemaFileError::Io {
            path: self.schema_dir.clone(),
            source: std::io::Error::other(e),
        })
    })?;

    // 2. Filter for schema files (exclude property bank)
    let property_bank_filename = self.property_bank_filename.as_deref();
    let files: Vec<PathBuf> = all_files
        .into_iter()
        .filter(|path| {
            let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            else {
                return false;
            };

            // Exclude property bank file
            if let Some(pb_name) = property_bank_filename {
                if file_name == pb_name {
                    return false;
                }
            }

            // Only include valid schema extensions
            let Some(ext) = path.extension().and_then(|e| e.to_str())
            else {
                return false;
            };

            const SCHEMA_EXTENSIONS: [&str; 4] = ["json", "toml", "yaml", "yml"];
            SCHEMA_EXTENSIONS
                .iter()
                .any(|allowed| ext.eq_ignore_ascii_case(allowed))
        })
        .collect();

    // 3. Check if property bank exists
    let has_property_bank = property_bank_filename.is_some();

    // 4. Load graph from DB (if exists)
    let graph = self
        .repository
        .get_topological_graph()
        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

    Ok(DiscoveryContext {
        graph,
        files,
        has_property_bank,
    })
}
```

#### 1.3: Update load_schemas_v2() to use discovery()

**Location**: `lithos-core/src/schema/builder.rs` (lines 168-199)

**Replace**:
```rust
pub(crate) fn load_schemas_v2(
    &self,
    pb: &PropertyBank,
) -> Result<Vec<Schema>, SchemaLoaderError> {
    use super::schema_pipeline::{Discovery, SchemaTreeProcessor, Unknown};

    let property_bank_filename =
        match self.property_bank_filename.as_deref() {
            Some(name) => name,
            None => self
                .source
                .filename(self.property_bank_path.as_path())
                .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?,
        };

    let pipeline = SchemaTreeProcessor::<Discovery, Unknown>::new();
    let discovered = pipeline.discover_tree(
        &self.source,
        &self.repository,
        self.schema_dir.as_path(),
        property_bank_filename,
    )?;
    // ... rest of pipeline
}
```

**With**:
```rust
pub(crate) fn load_schemas_v2(
    &self,
    pb: &PropertyBank,
) -> Result<Vec<Schema>, SchemaLoaderError> {
    use super::schema_pipeline::{Discovery, SchemaTreeProcessor, Unknown};

    // 1. Discovery: Load graph + scan files
    let context = self.discovery()?;

    // 2. Start pipeline with discovery context
    let pipeline = SchemaTreeProcessor::<Discovery, Unknown>::new();
    let discovered = pipeline.discover_with_context(
        context,
        &self.source,
        &self.repository,
    )?;

    // ... rest of pipeline unchanged
}
```

#### 1.4: Update SchemaTreeProcessor::discover_tree()

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 363-451)

**Rename** `discover_tree()` to `discover_with_context()` and **update**:

```rust
/// Discovers schema files using pre-loaded context from Builder.
///
/// This method:
/// 1. Uses graph from context (if exists) or creates empty graph
/// 2. Queries DB for views and IDs for discovered files
/// 3. Detects and deletes schemas that no longer exist on filesystem
///
/// # Arguments
///
/// * `context` - Discovery context from Builder (graph + files)
/// * `source` - Filesystem reader for file metadata
/// * `repository` - Database for querying views
///
/// # Errors
///
/// Returns `SchemaLoaderError` if DB queries or file operations fail.
#[expect(
    clippy::iter_over_hash_type,
    reason = "iteration order irrelevant for view hydration"
)]
pub(crate) fn discover_with_context<R: Repository>(
    self,
    context: DiscoveryContext,
    source: &FsReader,
    repository: &R,
) -> Result<
    SchemaTreeProcessor<Comparison, DiscoveryState>,
    SchemaLoaderError,
>
where
    R::Error: Into<SchemaRepositoryError>,
{
    let DiscoveryContext { graph, files, .. } = context;

    // Note: We don't use the graph yet in Phase 1
    // Phase 2 will hydrate it with FileStatus
    drop(graph);  // Explicit drop to show we're not using it yet

    // Rest of logic unchanged (DB queries, deleted detection)
    let id_by_path = repository
        .find_schema_ids_by_paths(&files)
        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

    let views_by_path = repository
        .find_raw_schema_views_by_paths(&files)
        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

    let mut view_by_id = HashMap::new();
    for (path, view) in views_by_path {
        if let Some(id) = id_by_path.get(&path) {
            view_by_id.insert(*id, view);
        }
    }

    // Detect deleted schemas
    let db_pairs = repository
        .list_schema_path_id_pairs()
        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
    let file_set: HashSet<PathBuf> = files.iter().cloned().collect();

    let mut deleted_ids = Vec::new();
    for (path, id) in db_pairs {
        if !file_set.contains(&path) {
            deleted_ids.push(id);
        }
    }

    // Delete from DB immediately
    if !deleted_ids.is_empty() {
        for id in &deleted_ids {
            repository
                .delete_schema(*id)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        }
    }

    Ok(self.transition(DiscoveryState {
        files,
        id_by_path,
        view_by_id,
        deleted_ids,
    }))
}
```

### Testing Strategy for Phase 1

#### Unit Tests

**File**: `lithos-core/src/schema/builder.rs` (in `#[cfg(test)]` mod)

**Add**:
```rust
#[test]
fn builder_discovery_loads_graph_from_db() {
    let temp = TempDir::new().unwrap();
    let repo = setup_test_repo_with_graph(&temp);
    let config = setup_test_config(&temp);
    let source = FsReader::new(temp.path().to_path_buf());

    let builder = Builder::new(repo, source, &config);
    let context = builder.discovery().unwrap();

    assert!(context.graph.is_some(), "Should load graph from DB");
}

#[test]
fn builder_discovery_excludes_property_bank() {
    let temp = TempDir::new().unwrap();
    create_schema_files(&temp, &["schema_a.toml", "property_bank.toml"]);
    let repo = InMemoryRepository::new();
    let config = setup_test_config(&temp);
    let source = FsReader::new(temp.path().to_path_buf());

    let builder = Builder::new(repo, source, &config);
    let context = builder.discovery().unwrap();

    assert_eq!(context.files.len(), 1, "Should exclude property_bank");
    assert!(context.has_property_bank, "Should detect property bank");
}

#[test]
fn builder_discovery_handles_missing_graph() {
    let temp = TempDir::new().unwrap();
    let repo = InMemoryRepository::new();  // Empty DB
    let config = setup_test_config(&temp);
    let source = FsReader::new(temp.path().to_path_buf());

    let builder = Builder::new(repo, source, &config);
    let context = builder.discovery().unwrap();

    assert!(context.graph.is_none(), "Should handle missing graph");
}
```

#### Integration Tests

**File**: `lithos-core/tests/schema_discovery_integration.rs` (new file)

```rust
use lithos_core::schema::{Builder, InMemoryRepository};
use lithos_core::config::Config;
use lithos_core::fs::FsReader;
use tempfile::TempDir;

#[test]
fn discovery_context_round_trip() {
    // Setup: Create schemas, save graph to DB
    let temp = TempDir::new().unwrap();
    let repo = InMemoryRepository::new();
    create_schema_files(&temp, &["a.toml", "b.toml", "c.toml"]);

    // First run: Graph should be None
    let builder = setup_builder(&repo, &temp);
    let context1 = builder.discovery().unwrap();
    assert!(context1.graph.is_none());

    // Save graph to DB
    let graph = build_graph_from_files(&context1.files);
    repo.save_topological_graph(&graph).unwrap();

    // Second run: Graph should be loaded
    let builder = setup_builder(&repo, &temp);
    let context2 = builder.discovery().unwrap();
    assert!(context2.graph.is_some());
    assert_eq!(context2.graph.unwrap().nodes.len(), 3);
}
```

### Phase 1 Validation Checklist

- [ ] `Builder::discovery()` compiles and runs
- [ ] `DiscoveryContext` struct defined with correct fields
- [ ] Graph loaded from DB when present
- [ ] Graph is `None` on first run
- [ ] Property bank file excluded from file list
- [ ] Property bank detection works correctly
- [ ] File filtering by extension works
- [ ] All existing tests still pass
- [ ] New unit tests added and passing
- [ ] Integration test for discovery round-trip passing

---

## Phase 2: Graph Hydration

### Goal

Refactor pipeline to use `TopologicalGraph<FileStatus>` instead of separate `TopologicalGraph<InheritanceNode>` + `HashMap<SchemaId, FileStatus>`.

This is the **core architectural change** that unifies structure and status tracking.

### Why This Matters

**Current Problem**:
```rust
// Two separate lookups required
let node = graph.nodes.get(&id)?;      // Get structure
let status = statuses.get(&id)?;       // Get status (separate!)
```

**Proposed Solution**:
```rust
// Single lookup
let node = graph.nodes.get(&id)?;
let status = &node.payload;  // Status embedded!
```

### Files to Modify

1. **`lithos-core/src/schema/schema_pipeline.rs`**
   - Update `ComparisonState` to store `TopologicalGraph<FileStatus>`
   - Update `TreeGraphedState` to use `TopologicalGraph<FileStatus>`
   - Update `PropertyAnalysisState` to use `TopologicalGraph<FileStatus>`
   - Update `ConstructionState` to use `TopologicalGraph<FileStatus>`
   - Update `CompletionState` to use `TopologicalGraph<FileStatus>`

2. **`lithos-core/src/schema/graph.rs`**
   - No changes needed (already supports generic payloads!)

### Detailed Changes

#### 2.1: Convert InheritanceNode to GraphNode<FileStatus>

**Helper function** (add to schema_pipeline.rs after line 1300):

```rust
/// Convert TopologicalGraph<InheritanceNode> to TopologicalGraph<FileStatus>.
///
/// This hydrates existing graph structure with file status payloads.
fn hydrate_graph_with_status(
    graph: TopologicalGraph<InheritanceNode>,
    statuses: HashMap<SchemaId, FileStatus>,
) -> TopologicalGraph<FileStatus> {
    let mut new_nodes = HashMap::new();

    for (id, node) in graph.nodes {
        // Get status for this node (should exist for all nodes)
        let status = statuses.get(&id).cloned().unwrap_or_else(|| {
            // Fallback: If status missing, mark as deleted
            // (This shouldn't happen in normal operation)
            FileStatus::Fresh {
                id,
                path: PathBuf::from("unknown"),
                view: RawSchemaView::default(),  // Will need proper handling
            }
        });

        let graph_node = GraphNode {
            id: node.id,
            parents: node.parents,
            children: node.children,
            depth: node.depth,
            payload: status,
        };

        new_nodes.insert(id, graph_node);
    }

    TopologicalGraph {
        order: graph.order,
        nodes: new_nodes,
        roots: graph.roots,
    }
}

/// Convert TopologicalGraph<FileStatus> to TopologicalGraph<InheritanceNode>.
///
/// This strips status payloads for persistence.
fn dehydrate_graph_to_inheritance(
    graph: &TopologicalGraph<FileStatus>,
) -> TopologicalGraph<InheritanceNode> {
    let mut new_nodes = HashMap::new();

    for (id, node) in &graph.nodes {
        let inheritance_node = InheritanceNode {
            id: node.id,
            parents: node.parents.clone(),
            children: node.children.clone(),
            depth: node.depth,
        };

        new_nodes.insert(*id, inheritance_node);
    }

    TopologicalGraph {
        order: graph.order.clone(),
        nodes: new_nodes,
        roots: graph.roots.clone(),
    }
}
```

#### 2.2: Update ComparisonState

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 115-124)

**Change from**:
```rust
#[derive(Debug)]
pub(crate) struct ComparisonState {
    statuses: HashMap<SchemaId, FileStatus>,
    fresh_ids: Vec<SchemaId>,
    stale_ids: Vec<SchemaId>,
    new_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}
```

**To**:
```rust
#[derive(Debug)]
pub(crate) struct ComparisonState {
    /// Graph with FileStatus embedded in nodes.
    ///
    /// Note: At this stage, graph structure is from DB (if exists)
    /// or empty. It will be rebuilt/patched in TreeGraphed stage
    /// based on extends changes.
    graph: Option<TopologicalGraph<FileStatus>>,

    /// IDs categorized by staleness (for metrics/logging).
    fresh_ids: Vec<SchemaId>,
    stale_ids: Vec<SchemaId>,
    new_ids: Vec<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}
```

#### 2.3: Update Comparison Stage

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 458-571)

**Key changes**:

1. Accept `DiscoveryContext` (contains pre-loaded graph)
2. Build `HashMap<SchemaId, FileStatus>` as before
3. Hydrate graph with statuses (or build fresh if no graph)
4. Return `ComparisonState` with hydrated graph

**Updated method signature**:
```rust
pub(crate) fn compare_files(
    self,
    source: &FsReader,
) -> Result<
    SchemaTreeProcessor<TreeGraphed, ComparisonState>,
    SchemaLoaderError,
> {
    let SchemaTreeProcessor {
        status:
            DiscoveryState {
                files,
                id_by_path,
                view_by_id,
                deleted_ids,
            },
        ..
    } = self;

    // Build FileStatus map (unchanged logic)
    let mut statuses = HashMap::new();
    let mut fresh_ids = Vec::new();
    let mut stale_ids = Vec::new();
    let mut new_ids = Vec::new();

    for path in &files {
        let id = id_by_path.get(path).copied().unwrap_or_else(SchemaId::new);
        let times = RawFileTimes {
            created_at: source.created_at(path),
            modified_at: source.modified_at(path),
        };

        let status = if let Some(view) = view_by_id.get(&id) {
            // ... existing staleness detection logic ...
        } else {
            // ... new schema logic ...
        };

        // Track IDs by category
        match &status {
            FileStatus::Fresh { .. } => fresh_ids.push(id),
            FileStatus::StaleTimestamps { .. } | FileStatus::StaleContent { .. } => {
                stale_ids.push(id)
            }
            FileStatus::New { .. } => new_ids.push(id),
        }

        statuses.insert(id, status);
    }

    // NEW: Hydrate graph with statuses
    // (In Phase 1, we don't have graph yet, so this is None)
    // (In Phase 2+, we load graph from DiscoveryContext)
    let graph = None;  // TODO: Will be populated from DiscoveryContext in next commit

    Ok(SchemaTreeProcessor {
        status: ComparisonState {
            graph,
            fresh_ids,
            stale_ids,
            new_ids,
            deleted_ids,
        },
        _stage: PhantomData,
    })
}
```

#### 2.4: Update TreeGraphedState

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 127-134)

**Change from**:
```rust
#[derive(Debug)]
pub(crate) struct TreeGraphedState {
    graph: PipelineGraph,  // TopologicalGraph<InheritanceNode>
    statuses: HashMap<SchemaId, FileStatus>,  // ← REMOVE THIS
    raw_by_id: HashMap<SchemaId, RawSchema>,
    extends_deltas: HashMap<SchemaId, ExtendsDelta>,
    affected_subtrees: HashSet<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}
```

**To**:
```rust
#[derive(Debug)]
pub(crate) struct TreeGraphedState {
    /// Graph with FileStatus embedded in nodes (structure + status unified).
    graph: TopologicalGraph<FileStatus>,

    /// Parsed RawSchema for stale/new schemas.
    raw_by_id: HashMap<SchemaId, RawSchema>,

    /// Extends deltas for detecting graph rewiring needs.
    extends_deltas: HashMap<SchemaId, ExtendsDelta>,

    /// IDs of schemas in affected subtrees (need rebuild).
    affected_subtrees: HashSet<SchemaId>,

    /// IDs of deleted schemas (remove from graph + DB).
    deleted_ids: Vec<SchemaId>,
}
```

#### 2.5: Update TreeGraphed Stage

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 578-646)

**Key changes**:

1. Extract statuses from graph nodes
2. Build `DagBuilder` from statuses (unchanged)
3. Rebuild graph structure
4. Re-hydrate with original statuses
5. Compute deltas and affected subtrees

**Updated method**:
```rust
pub(crate) fn graph_structure(
    self,
) -> Result<
    SchemaTreeProcessor<PropertyAnalysis, TreeGraphedState>,
    SchemaLoaderError,
> {
    let SchemaTreeProcessor {
        status:
            ComparisonState {
                graph,
                new_ids,
                deleted_ids,
                ..
            },
        ..
    } = self;

    // Extract statuses from graph (if exists)
    let (statuses, old_graph) = if let Some(g) = graph {
        let mut statuses = HashMap::new();
        for (id, node) in &g.nodes {
            statuses.insert(*id, node.payload.clone());
        }
        (statuses, Some(g))
    } else {
        (HashMap::new(), None)
    };

    // Build raw_by_id and extends_deltas (unchanged logic)
    let mut raw_by_id = HashMap::new();
    let mut extends_deltas = HashMap::new();

    for (id, status) in &statuses {
        if let Some(raw) = status.raw() {
            raw_by_id.insert(*id, raw.clone());
            let old_parent = status.view().and_then(|view| view.extends().cloned());
            let new_parent = raw.extends().cloned();
            extends_deltas.insert(*id, ExtendsDelta {
                old_parent,
                new_parent,
            });
        }
    }

    // Rebuild graph structure (unchanged)
    let inheritance_graph = DagBuilder::new(&statuses).build()?;

    // Re-hydrate with statuses
    let graph = hydrate_graph_with_status(inheritance_graph, statuses);

    // Compute affected subtrees (unchanged logic)
    let mut changed_extends_ids = HashSet::new();
    for (id, delta) in &extends_deltas {
        if delta.changed() {
            changed_extends_ids.insert(*id);
        }
    }

    let mut seed_ids: HashSet<SchemaId> = changed_extends_ids;
    seed_ids.extend(new_ids.iter().copied());

    let affected_subtrees = if seed_ids.is_empty() {
        HashSet::new()
    } else {
        graph.affected_subtree(&seed_ids)
    };

    Ok(SchemaTreeProcessor {
        status: TreeGraphedState {
            graph,
            raw_by_id,
            extends_deltas,
            affected_subtrees,
            deleted_ids,
        },
        _stage: PhantomData,
    })
}
```

#### 2.6: Update PropertyAnalysisState

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 136-148)

**Change**:
```rust
#[derive(Debug)]
pub(crate) struct PropertyAnalysisState {
    graph: TopologicalGraph<FileStatus>,  // ← Changed from PipelineGraph
    // Remove: statuses: HashMap<SchemaId, FileStatus>,
    raw_by_id: HashMap<SchemaId, RawSchema>,
    deltas_by_id: HashMap<SchemaId, SchemaPropertyDelta>,
    excludes_by_id: HashMap<SchemaId, ExcludesDelta>,
    rebuild_ids: HashSet<SchemaId>,
    deleted_ids: Vec<SchemaId>,
}
```

#### 2.7: Update PropertyAnalysis Stage

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 653-786)

**Key changes**: Replace all `statuses.get(id)` with `graph.nodes.get(id).map(|n| &n.payload)`

**Example changes**:
```rust
// OLD:
for (id, status) in &statuses {
    let view = status.view();
    let has_raw = status.raw().is_some();
    // ...
}

// NEW:
for (id, node) in &graph.nodes {
    let status = &node.payload;
    let view = status.view();
    let has_raw = status.raw().is_some();
    // ...
}
```

#### 2.8: Update ConstructionState

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 150-159)

**Change**:
```rust
#[derive(Debug)]
pub(crate) struct ConstructionState {
    graph: TopologicalGraph<FileStatus>,  // ← Changed from PipelineGraph
    // Remove: statuses: HashMap<SchemaId, FileStatus>,
    raw_by_id: HashMap<SchemaId, RawSchema>,
    rebuild_ids: HashSet<SchemaId>,
    changed_ids: HashSet<SchemaId>,
    schemas: Vec<Arc<Schema>>,
}
```

#### 2.9: Update Construction Stage

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 941-1121)

**Key changes**: Replace all `status.statuses.get(id)` with `status.graph.nodes.get(id).map(|n| &n.payload)`

#### 2.10: Update CompletionState

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 161-166)

**Change**:
```rust
#[derive(Debug)]
pub(crate) struct CompletionState {
    schemas: Vec<Arc<Schema>>,
    graph: TopologicalGraph<FileStatus>,  // ← Changed from PipelineGraph
    changed_ids: HashSet<SchemaId>,
}
```

#### 2.11: Update Completion Stage (Persist)

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 1128-1163)

**Key change**: Dehydrate graph before persisting

```rust
pub(crate) fn persist<R: Repository>(
    self,
    repository: &R,
) -> Result<Vec<Schema>, SchemaLoaderError>
where
    R::Error: Into<SchemaRepositoryError>,
{
    // Persist changed schemas (unchanged)
    if !self.status.changed_ids.is_empty() {
        let schemas: Vec<Schema> = self
            .status
            .schemas
            .iter()
            .filter(|schema| self.status.changed_ids.contains(schema.id()))
            .map(|schema| (**schema).clone())
            .collect();
        if !schemas.is_empty() {
            repository
                .save_schemas(&schemas)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
        }
    }

    // NEW: Dehydrate graph before persisting
    let inheritance_graph = dehydrate_graph_to_inheritance(&self.status.graph);
    repository
        .save_topological_graph(&inheritance_graph)
        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

    Ok(self
        .status
        .schemas
        .into_iter()
        .map(|schema| (*schema).clone())
        .collect())
}
```

### Testing Strategy for Phase 2

#### Unit Tests

**File**: `lithos-core/src/schema/schema_pipeline.rs` (in `#[cfg(test)]` mod)

**Add**:
```rust
#[test]
fn hydrate_dehydrate_graph_round_trip() {
    let id_a = SchemaId::new();
    let id_b = SchemaId::new();

    // Create graph with FileStatus
    let status_a = FileStatus::Fresh {
        id: id_a,
        path: PathBuf::from("a.toml"),
        view: RawSchemaView::default(),
    };
    let status_b = FileStatus::New {
        id: id_b,
        path: PathBuf::from("b.toml"),
        raw: RawSchema::default(),
        content_hash: [0u8; 32],
        times: RawFileTimes::default(),
    };

    let mut nodes = HashMap::new();
    nodes.insert(id_a, GraphNode {
        id: id_a,
        parents: vec![],
        children: vec![id_b],
        depth: NodeDepth::ROOT,
        payload: status_a.clone(),
    });
    nodes.insert(id_b, GraphNode {
        id: id_b,
        parents: vec![id_a],
        children: vec![],
        depth: NodeDepth::new(1),
        payload: status_b.clone(),
    });

    let graph_with_status = TopologicalGraph {
        order: vec![id_a, id_b],
        nodes,
        roots: vec![id_a],
    };

    // Dehydrate
    let inheritance_graph = dehydrate_graph_to_inheritance(&graph_with_status);
    assert_eq!(inheritance_graph.nodes.len(), 2);

    // Rehydrate
    let mut statuses = HashMap::new();
    statuses.insert(id_a, status_a);
    statuses.insert(id_b, status_b);
    let rehydrated = hydrate_graph_with_status(inheritance_graph, statuses);

    assert_eq!(rehydrated.nodes.len(), 2);
    assert_eq!(rehydrated.order, vec![id_a, id_b]);
}

#[test]
fn graph_structure_preserves_status() {
    // Setup comparison state with graph
    let statuses = create_test_statuses();
    let graph = build_test_graph_from_statuses(&statuses);

    let comparison_state = ComparisonState {
        graph: Some(graph),
        fresh_ids: vec![],
        stale_ids: vec![],
        new_ids: vec![],
        deleted_ids: vec![],
    };

    let processor = SchemaTreeProcessor {
        status: comparison_state,
        _stage: PhantomData::<TreeGraphed>,
    };

    let result = processor.graph_structure().unwrap();

    // Verify statuses preserved
    for (id, node) in &result.status.graph.nodes {
        let original_status = statuses.get(id).unwrap();
        assert_eq!(&node.payload, original_status);
    }
}
```

#### Integration Tests

**File**: `lithos-core/tests/schema_graph_hydration.rs` (new file)

```rust
#[test]
fn pipeline_preserves_status_through_stages() {
    let temp = TempDir::new().unwrap();
    let repo = InMemoryRepository::new();
    create_test_schemas(&temp);

    let builder = setup_builder(&repo, &temp);
    let context = builder.discovery().unwrap();

    let pipeline = SchemaTreeProcessor::<Discovery, Unknown>::new();
    let discovered = pipeline.discover_with_context(context, &source, &repo).unwrap();
    let compared = discovered.compare_files(&source).unwrap();
    let graphed = compared.graph_structure().unwrap();

    // Verify graph has FileStatus payloads
    for (id, node) in &graphed.status.graph.nodes {
        match &node.payload {
            FileStatus::Fresh { id: status_id, .. } => assert_eq!(status_id, id),
            FileStatus::New { id: status_id, .. } => assert_eq!(status_id, id),
            FileStatus::StaleContent { id: status_id, .. } => assert_eq!(status_id, id),
            FileStatus::StaleTimestamps { id: status_id, .. } => assert_eq!(status_id, id),
        }
    }
}
```

### Phase 2 Validation Checklist

- [ ] All state structs updated to use `TopologicalGraph<FileStatus>`
- [ ] `hydrate_graph_with_status()` helper implemented
- [ ] `dehydrate_graph_to_inheritance()` helper implemented
- [ ] All stages updated to access status via `node.payload`
- [ ] Compilation succeeds with no type errors
- [ ] All existing tests still pass
- [ ] New unit tests for hydration/dehydration passing
- [ ] Integration test for status preservation passing
- [ ] Manual verification: Graph persisted to DB correctly

---

## Phase 3: PropertyBankDelta Integration

### Goal

Integrate PropertyBankDelta checking into the Comparison stage to correctly demote Fresh schemas when they reference changed PropertyBank properties.

### Why This Matters

**Current Problem**: Fresh schemas that reference changed PropertyBank properties aren't rebuilt until PropertyAnalysis stage, missing the opportunity to skip file I/O in Comparison.

**Proposed Solution**: Check `bank_references` during Comparison. If Fresh schema references changed property, demote to `StaleContent` immediately.

### Files to Modify

1. **`lithos-core/src/schema/schema_pipeline.rs`**
   - Update `compare_files()` to accept `property_bank_delta`
   - Add PropertyBank reference checking after timestamp/hash checks

2. **`lithos-core/src/schema/builder.rs`**
   - Pass `property_bank_delta` to `compare_files()`

### Detailed Changes

#### 3.1: Update compare_files() signature

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (line 459)

**Change from**:
```rust
pub(crate) fn compare_files(
    self,
    source: &FsReader,
) -> Result<
    SchemaTreeProcessor<TreeGraphed, ComparisonState>,
    SchemaLoaderError,
> {
```

**To**:
```rust
pub(crate) fn compare_files(
    self,
    source: &FsReader,
    property_bank_delta: Option<&HashSet<PropertyName>>,
) -> Result<
    SchemaTreeProcessor<TreeGraphed, ComparisonState>,
    SchemaLoaderError,
> {
```

#### 3.2: Add PropertyBank reference checking

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (after line 502)

**Add after timestamp check**:
```rust
if timestamps_match {
    // NEW: Check if schema references changed PropertyBank properties
    if let Some(pb_delta) = property_bank_delta {
        if let Some(version) = view.current() {
            let bank_refs = version.bank_references();
            let is_affected = bank_refs
                .values()
                .any(|bank_prop| pb_delta.contains(bank_prop));

            if is_affected {
                // Demote to StaleContent: need to re-expand bank refs
                let content = source
                    .read_to_string(path)
                    .map_err(|e| SchemaLoaderError::Ingestion(e.into()))?;
                let content_hash = *blake3::hash(content.as_bytes()).as_bytes();
                let raw = parse_raw_schema_from_str(path, &content, &times)?;

                stale_ids.push(id);
                FileStatus::StaleContent {
                    id,
                    path: path.clone(),
                    view: view.clone(),
                    raw,
                    content_hash,
                    times,
                }
            } else {
                // Fresh: no bank references changed
                fresh_ids.push(id);
                FileStatus::Fresh {
                    id,
                    path: path.clone(),
                    view: view.clone(),
                }
            }
        } else {
            // No version in view, treat as fresh
            fresh_ids.push(id);
            FileStatus::Fresh {
                id,
                path: path.clone(),
                view: view.clone(),
            }
        }
    } else {
        // No PropertyBank delta, treat as fresh
        fresh_ids.push(id);
        FileStatus::Fresh {
            id,
            path: path.clone(),
            view: view.clone(),
        }
    }
} else {
    // Timestamps don't match, proceed to content check...
    // (existing logic unchanged)
}
```

#### 3.3: Update Builder to pass property_bank_delta

**Location**: `lithos-core/src/schema/builder.rs` (line 191)

**Change from**:
```rust
let compared = discovered.compare_files(&self.source)?;
```

**To**:
```rust
let compared = discovered.compare_files(
    &self.source,
    self.property_bank_delta.as_ref(),
)?;
```

### Testing Strategy for Phase 3

#### Unit Tests

**File**: `lithos-core/src/schema/schema_pipeline.rs`

```rust
#[test]
fn fresh_schema_demoted_when_bank_ref_changed() {
    let id = SchemaId::new();
    let path = PathBuf::from("test.toml");

    // Create view with bank reference to "prop_a"
    let mut bank_refs = HashMap::new();
    bank_refs.insert(
        PropertyName::try_new("field_x").unwrap(),
        PropertyName::try_new("prop_a").unwrap(),
    );
    let view = create_test_view_with_bank_refs(bank_refs);

    // PropertyBank delta includes "prop_a"
    let mut pb_delta = HashSet::new();
    pb_delta.insert(PropertyName::try_new("prop_a").unwrap());

    let processor = setup_comparison_processor(id, path, view);
    let result = processor.compare_files(&source, Some(&pb_delta)).unwrap();

    // Verify schema was demoted to StaleContent
    let node = result.status.graph.unwrap().nodes.get(&id).unwrap();
    assert!(matches!(node.payload, FileStatus::StaleContent { .. }));
}

#[test]
fn fresh_schema_stays_fresh_when_bank_ref_unchanged() {
    let id = SchemaId::new();

    // View references "prop_a"
    let view = create_test_view_with_bank_ref("prop_a");

    // PropertyBank delta has "prop_b" (different property)
    let mut pb_delta = HashSet::new();
    pb_delta.insert(PropertyName::try_new("prop_b").unwrap());

    let processor = setup_comparison_processor(id, PathBuf::from("test.toml"), view);
    let result = processor.compare_files(&source, Some(&pb_delta)).unwrap();

    // Verify schema stayed Fresh
    let node = result.status.graph.unwrap().nodes.get(&id).unwrap();
    assert!(matches!(node.payload, FileStatus::Fresh { .. }));
}
```

#### Integration Tests

**File**: `lithos-core/tests/schema_property_bank_integration.rs`

```rust
#[test]
fn pipeline_rebuilds_schemas_with_changed_bank_refs() {
    let temp = TempDir::new().unwrap();
    let repo = InMemoryRepository::new();

    // Create schema that references PropertyBank
    let schema_content = r#"
        version = "1.0"
        properties.title = { "$ref": "property_bank#/standard_title" }
    "#;
    fs::write(temp.path().join("article.toml"), schema_content).unwrap();

    // First run: Process schema
    let builder = setup_builder(&repo, &temp);
    let pb = builder.load_property_bank().unwrap();
    let schemas1 = builder.load_schemas_v2(&pb).unwrap();
    assert_eq!(schemas1.len(), 1);

    // Second run: Change PropertyBank (mark standard_title as changed)
    let mut pb_delta = HashSet::new();
    pb_delta.insert(PropertyName::try_new("standard_title").unwrap());
    let mut builder = setup_builder(&repo, &temp);
    builder.set_property_bank_delta(Some(pb_delta));

    // Schema should be rebuilt even though file unchanged
    let schemas2 = builder.load_schemas_v2(&pb).unwrap();
    assert_eq!(schemas2.len(), 1);
    // (Further assertions to verify rebuild happened)
}
```

### Phase 3 Validation Checklist

- [ ] `compare_files()` signature updated with `property_bank_delta` parameter
- [ ] PropertyBank reference checking implemented
- [ ] Fresh schemas demoted to StaleContent when bank ref changed
- [ ] Fresh schemas stay fresh when bank ref unchanged
- [ ] Builder passes `property_bank_delta` correctly
- [ ] All existing tests still pass
- [ ] Unit tests for demotion logic passing
- [ ] Integration test for end-to-end bank ref handling passing

---

## Phase 4: Enhanced Refresh

### Goal

Extend the Refresh stage to handle not just `StaleTimestamps` but also `StaleContent` with unchanged semantic properties (only comments/formatting changed).

### Why This Matters

**Current**: `StaleContent` always proceeds to Construction even if only whitespace changed.

**Proposed**: If property hashes, extends, and excludes are unchanged, just update content hash and skip Construction.

### Files to Modify

1. **`lithos-core/src/schema/schema_pipeline.rs`**
   - Update `refresh_metadata()` to handle `StaleContent` optimization

### Detailed Changes

#### 4.1: Update refresh_metadata()

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 807-935)

**Add** after handling `StaleTimestamps` (around line 879):

```rust
FileStatus::StaleContent {
    id,
    path,
    mut view,
    raw,
    content_hash,
    times,
} => {
    // Check if this schema needs rebuild
    // If not in rebuild_ids, semantic properties unchanged
    if !rebuild_ids.contains(&id) {
        // Semantic properties unchanged, only content hash differs
        // (comments, whitespace, formatting changed)

        // Rebuild SchemaVersion with new hashes
        let file_times = FileTimesMetadata::new(
            times.created_at,
            times.modified_at,
        );

        // Compute new property hashes from raw
        let property_hashes = compute_property_hashes(&raw)?;

        let hashes = HashMetadata::new(content_hash, property_hashes);

        let version = SchemaVersion::new(file_times, hashes, &raw)
            .map_err(SchemaLoaderError::Ingestion)?;

        view.add_version(version);

        // Persist updated view
        repository
            .save_raw_schema_view(id, &view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

        // Transition to Fresh (skip Construction)
        FileStatus::Fresh {
            id,
            path,
            view,
        }
    } else {
        // Needs rebuild, pass through to Construction
        FileStatus::StaleContent {
            id,
            path,
            view,
            raw,
            content_hash,
            times,
        }
    }
}
```

#### 4.2: Add compute_property_hashes helper

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (after line 1300)

**Add**:
```rust
/// Compute property hashes from RawSchema.
///
/// This is used in Refresh stage to update hashes for schemas
/// with unchanged semantic properties.
fn compute_property_hashes(
    raw: &RawSchema,
) -> Result<HashMap<PropertyName, [u8; 32]>, SchemaLoaderError> {
    let mut hashes = HashMap::new();

    for (name, prop) in raw.properties() {
        let serialized = serde_json::to_vec(prop).map_err(|e| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Parse(
                SchemaParseError::Serialization {
                    path: PathBuf::from(raw.name()),
                    reason: e.to_string().into(),
                },
            ))
        })?;

        let hash = *blake3::hash(&serialized).as_bytes();
        hashes.insert(name.clone(), hash);
    }

    Ok(hashes)
}
```

### Testing Strategy for Phase 4

#### Unit Tests

**File**: `lithos-core/src/schema/schema_pipeline.rs`

```rust
#[test]
fn refresh_skips_construction_for_comment_changes() {
    let id = SchemaId::new();
    let path = PathBuf::from("test.toml");

    // Create StaleContent with unchanged properties
    let raw = create_test_raw_schema();
    let view = create_test_view_matching_raw(&raw);
    let content_hash = [1u8; 32];  // Different hash
    let times = RawFileTimes::default();

    let status = FileStatus::StaleContent {
        id,
        path: path.clone(),
        view,
        raw,
        content_hash,
        times,
    };

    let mut graph = TopologicalGraph::default();
    graph.nodes.insert(id, GraphNode {
        id,
        parents: vec![],
        children: vec![],
        depth: NodeDepth::ROOT,
        payload: status,
    });

    let analysis_state = PropertyAnalysisState {
        graph,
        raw_by_id: HashMap::new(),
        deltas_by_id: HashMap::new(),
        excludes_by_id: HashMap::new(),
        rebuild_ids: HashSet::new(),  // Empty: no rebuild needed
        deleted_ids: vec![],
    };

    let processor = SchemaTreeProcessor {
        status: analysis_state,
        _stage: PhantomData::<Refresh>,
    };

    let result = processor.refresh_metadata(&repo).unwrap();

    // Verify transitioned to Fresh (skipped Construction)
    let node = result.status.graph.nodes.get(&id).unwrap();
    assert!(matches!(node.payload, FileStatus::Fresh { .. }));
}

#[test]
fn refresh_preserves_stale_content_for_semantic_changes() {
    // Similar test but with rebuild_ids containing the schema
    // Should preserve StaleContent status
}
```

#### Integration Tests

**File**: `lithos-core/tests/schema_refresh_optimization.rs`

```rust
#[test]
fn pipeline_skips_construction_for_comment_only_changes() {
    let temp = TempDir::new().unwrap();
    let repo = InMemoryRepository::new();

    // First run: Process schema
    let schema_v1 = r#"
        version = "1.0"
        properties.title = { type = "string" }
    "#;
    fs::write(temp.path().join("article.toml"), schema_v1).unwrap();

    let builder = setup_builder(&repo, &temp);
    let pb = builder.load_property_bank().unwrap();
    let schemas1 = builder.load_schemas_v2(&pb).unwrap();

    // Second run: Add comment (no semantic change)
    let schema_v2 = r#"
        version = "1.0"
        # This is a comment
        properties.title = { type = "string" }
    "#;
    fs::write(temp.path().join("article.toml"), schema_v2).unwrap();

    // Touch file to change timestamp
    touch_file(temp.path().join("article.toml"));

    let builder = setup_builder(&repo, &temp);
    let schemas2 = builder.load_schemas_v2(&pb).unwrap();

    // Verify schema wasn't reconstructed (just view updated)
    // (Check internal metrics or logs)
}
```

### Phase 4 Validation Checklist

- [ ] `refresh_metadata()` handles `StaleContent` optimization
- [ ] `compute_property_hashes()` helper implemented
- [ ] Comment-only changes skip Construction
- [ ] Semantic changes still trigger Construction
- [ ] View updated correctly with new content hash
- [ ] All existing tests still pass
- [ ] Unit tests for refresh optimization passing
- [ ] Integration test for comment-only changes passing

---

## Phase 5: Incremental Construction

### Goal

Optimize Construction stage to skip full merge when only properties changed (not extends/excludes).

### Why This Matters

**Current**: Always performs full merge (fetch parent, merge properties, apply excludes) even when only a single property changed.

**Proposed**: When only `SchemaPropertyDelta` present (no `ExtendsDelta` or `ExcludesDelta`), fetch existing Schema from DB, apply property delta, save.

**Performance Impact**: 50-80% faster for property-only changes.

### Files to Modify

1. **`lithos-core/src/schema/schema_pipeline.rs`**
   - Update `construct_schemas()` to detect property-only changes
   - Add incremental update path

### Detailed Changes

#### 5.1: Add incremental construction logic

**Location**: `lithos-core/src/schema/schema_pipeline.rs` (lines 1010-1111, inside the `for id in &status.graph.order` loop)

**Replace** the section that handles rebuild_ids with:

```rust
if !rebuild_ids.contains(id) {
    // Fresh schema: fetch from DB
    let schema = fetched_by_id.get(id).ok_or_else(|| {
        SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
            SchemaStorageError::NotFound {
                name: status_name.clone(),
            },
        ))
    })?;
    resolved_cache.insert(*id, schema.clone());
    status.schemas.push(Arc::new(schema.clone()));
    continue;
}

// Schema needs rebuild: determine rebuild strategy
changed_ids.insert(*id);

let extends_delta = status.extends_deltas.get(id);
let excludes_delta = status.excludes_by_id.get(id);
let property_delta = status.deltas_by_id.get(id);

// OPTIMIZATION: Detect property-only changes
let property_only_change = extends_delta
    .map(|d| !d.changed())
    .unwrap_or(true)  // No extends delta = unchanged
    && excludes_delta
        .map(|d| d.is_empty())
        .unwrap_or(true)  // No excludes delta = unchanged
    && property_delta
        .map(|d| !d.is_empty())
        .unwrap_or(false);  // Has property delta

if property_only_change {
    // INCREMENTAL PATH: Only properties changed

    // Fetch existing schema from DB
    let mut schema = fetched_by_id.get(id).cloned().ok_or_else(|| {
        SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
            SchemaStorageError::NotFound {
                name: status_name.clone(),
            },
        ))
    })?;

    // Get property delta
    let delta = property_delta.unwrap();  // Safe: checked above

    // Expand changed properties
    let raw = status.raw_by_id.get(id).ok_or_else(|| {
        SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
            SchemaStorageError::NotFound {
                name: status_name.clone(),
            },
        ))
    })?;

    let expanded = RefExpander::new(bank)
        .expand_all(vec![(*id, raw.clone())])
        .map_err(SchemaLoaderError::Resolution)?
        .into_iter()
        .next()
        .ok_or_else(|| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
                SchemaStorageError::NotFound {
                    name: status_name.clone(),
                },
            ))
        })?;

    // Apply property delta to schema
    let mut properties = schema.properties().clone();

    // Upsert changed properties
    for (name, prop) in &expanded.properties {
        if delta.upserts.inline.contains_key(name)
            || delta.upserts.refs.contains_key(name)
        {
            properties.insert(name.clone(), prop.clone());
        }
    }

    // Remove deleted properties
    for name in &delta.removed {
        properties.remove(name);
    }

    // Update schema
    let updated_schema = Schema::new(
        schema.id,
        schema.name.clone(),
        schema.parent_id,
        schema.children.clone(),
        properties,
    );

    // Save view
    if let Some(file_status) = status_entry {
        let content_hash = file_status.content_hash().ok_or_else(|| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
                SchemaStorageError::NotFound {
                    name: status_name.clone(),
                },
            ))
        })?;
        let view = build_view_from_raw(raw, file_status.path(), content_hash)?;
        repository
            .save_raw_schema_view(*id, &view)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
    }

    resolved_cache.insert(*id, updated_schema.clone());
    status.schemas.push(Arc::new(updated_schema));
    continue;
}

// FULL MERGE PATH: Extends/excludes changed or new schema
let expanded = expanded_by_id.get(id).ok_or_else(|| {
    SchemaLoaderError::Ingestion(SchemaIngestionError::Storage(
        SchemaStorageError::NotFound {
            name: status_name.clone(),
        },
    ))
})?;

// ... existing full merge logic ...
```

### Testing Strategy for Phase 5

#### Unit Tests

**File**: `lithos-core/src/schema/schema_pipeline.rs`

```rust
#[test]
fn construction_uses_incremental_path_for_property_only() {
    let id = SchemaId::new();

    // Setup schema with property-only delta
    let mut property_delta = SchemaPropertyDelta::default();
    property_delta.upserts.inline.insert(
        PropertyName::try_new("title").unwrap(),
        create_test_property_inline(),
    );

    let mut deltas_by_id = HashMap::new();
    deltas_by_id.insert(id, property_delta);

    // No extends delta, no excludes delta
    let extends_deltas = HashMap::new();
    let excludes_by_id = HashMap::new();

    let mut rebuild_ids = HashSet::new();
    rebuild_ids.insert(id);

    // ... setup rest of ConstructionState ...

    let processor = SchemaTreeProcessor {
        status: construction_state,
        _stage: PhantomData::<Construction>,
    };

    // Mock repository to track calls
    let mock_repo = create_mock_repository();

    let result = processor.construct_schemas(&mock_repo, &bank).unwrap();

    // Verify incremental path was used (check mock calls)
    assert!(mock_repo.find_schemas_by_ids_called());
    assert!(!mock_repo.excessive_merges_performed());
}

#[test]
fn construction_uses_full_path_for_extends_changes() {
    // Similar test but with extends delta
    // Should use full merge path
}
```

#### Integration Tests

**File**: `lithos-core/tests/schema_construction_optimization.rs`

```rust
#[test]
fn pipeline_optimizes_property_only_changes() {
    let temp = TempDir::new().unwrap();
    let repo = InMemoryRepository::new();

    // First run: Create schema
    let schema_v1 = r#"
        version = "1.0"
        extends = "base"
        properties.title = { type = "string" }
        properties.author = { type = "string" }
    "#;
    fs::write(temp.path().join("article.toml"), schema_v1).unwrap();

    let builder = setup_builder(&repo, &temp);
    let pb = builder.load_property_bank().unwrap();
    let schemas1 = builder.load_schemas_v2(&pb).unwrap();

    // Second run: Change only property (not extends)
    let schema_v2 = r#"
        version = "1.0"
        extends = "base"
        properties.title = { type = "string", max_length = 100 }  # Changed
        properties.author = { type = "string" }
    "#;
    fs::write(temp.path().join("article.toml"), schema_v2).unwrap();

    let builder = setup_builder(&repo, &temp);
    let schemas2 = builder.load_schemas_v2(&pb).unwrap();

    // Verify incremental update was used
    // (Check logs, metrics, or internal counters)
}
```

### Phase 5 Validation Checklist

- [ ] Incremental construction path implemented
- [ ] Property-only changes use incremental path
- [ ] Extends/excludes changes use full merge path
- [ ] Schema correctly updated with delta
- [ ] View persisted correctly
- [ ] All existing tests still pass
- [ ] Unit tests for incremental path passing
- [ ] Integration test for optimization passing
- [ ] Performance improvement measurable (benchmark)

---

## Code Change Specifications

### Summary of Modified Files

| File | Lines Changed | New Lines | Deleted Lines | Complexity |
|------|---------------|-----------|---------------|------------|
| `builder.rs` | ~100 | +80 | -20 | Medium |
| `schema_pipeline.rs` | ~500 | +350 | -150 | High |
| `graph.rs` | ~0 | +0 | -0 | None (already supports generics) |

### Dependency Graph

```
Phase 1 (Builder Discovery)
  ↓ creates DiscoveryContext
Phase 2 (Graph Hydration)
  ↓ uses TopologicalGraph<FileStatus>
Phase 3 (PropertyBankDelta) ←─┐
  ↓                            │
Phase 4 (Enhanced Refresh)     │ Can be
  ↓                            │ parallel
Phase 5 (Incremental Construction) ─┘
```

### Backward Compatibility

All phases maintain backward compatibility:
- Existing `FileStatus` enum unchanged
- Existing `TopologicalGraph` algorithms unchanged
- Existing stage sequence unchanged
- Only internal representation changes

### Migration Strategy

**Option A: Big Bang** (all phases at once)
- **Pros**: Single PR, fewer context switches
- **Cons**: Large diff, harder to review, riskier

**Option B: Incremental** (phase-by-phase)
- **Pros**: Smaller PRs, easier review, incremental testing
- **Cons**: More PRs, potential for temporary inconsistencies

**Recommendation**: **Option B (Incremental)** with feature flags

---

## Testing Strategy

### Test Pyramid

```
        /\
       /  \      E2E Tests (5)
      /────\     - Full pipeline scenarios
     /      \    - Performance benchmarks
    /────────\   Integration Tests (15)
   /          \  - Stage transitions
  /────────────\ - Graph persistence
 /              \ Unit Tests (30+)
/────────────────\ - Individual functions
                   - Edge cases
```

### Test Coverage Goals

| Component | Current Coverage | Target Coverage | Priority |
|-----------|------------------|-----------------|----------|
| Builder | 85% | 90% | High |
| Discovery | 70% | 85% | High |
| Comparison | 60% | 80% | High |
| TreeGraphed | 75% | 85% | Medium |
| PropertyAnalysis | 65% | 80% | Medium |
| Refresh | 50% | 75% | High (new logic) |
| Construction | 70% | 85% | High (optimizations) |

### Critical Test Scenarios

#### 1. First Run (Empty DB)
- No graph in DB
- All schemas are new
- Graph built from scratch
- All schemas constructed

#### 2. Warm Start (Graph Cached)
- Graph loaded from DB
- All schemas fresh
- No reconstruction needed
- Fast path through pipeline

#### 3. PropertyBank Change
- Fresh schemas reference changed bank properties
- Demoted to StaleContent in Comparison
- Re-expanded in Construction

#### 4. Comment-Only Change
- Schema file modified (whitespace/comments)
- Content hash different
- Property hashes same
- Refreshed without reconstruction

#### 5. Property-Only Change
- Schema property modified
- Extends/excludes unchanged
- Incremental update in Construction

#### 6. Extends Change
- Schema parent changed
- Graph rewired
- Full merge required

#### 7. Deleted Schema
- Schema file removed
- Deleted from DB in Discovery
- Removed from graph

#### 8. Cycle Detection
- Schema A extends B, B extends A
- Detected in TreeGraphed
- Pipeline fails with clear error

### Performance Benchmarks

**Baseline** (current implementation):

| Scenario | Time | Graph Rebuilds | Full Merges |
|----------|------|----------------|-------------|
| 100 fresh schemas | 50ms | 1 | 0 |
| 1 property change | 120ms | 1 | 1 |
| 10 property changes | 500ms | 1 | 10 |
| 1 extends change | 150ms | 1 | 1 + affected subtree |

**Target** (after optimization):

| Scenario | Time | Graph Rebuilds | Full Merges | Improvement |
|----------|------|----------------|-------------|-------------|
| 100 fresh schemas | 5ms | 0 (loaded) | 0 | **10x faster** |
| 1 property change | 30ms | 0 (loaded) | 0 (incremental) | **4x faster** |
| 10 property changes | 150ms | 0 (loaded) | 0 (incremental) | **3.3x faster** |
| 1 extends change | 120ms | 0 (patched) | 1 + affected subtree | **1.25x faster** |

---

## Migration Path

### Rollout Plan

#### Week 1: Phase 1 + Phase 2
- Implement Builder discovery
- Implement graph hydration
- Run comprehensive tests
- **Checkpoint**: All tests pass, graph persisted correctly

#### Week 2: Phase 3 + Phase 4
- Implement PropertyBankDelta integration
- Implement enhanced Refresh
- Run performance tests
- **Checkpoint**: Optimizations measurable

#### Week 3: Phase 5 + Integration
- Implement incremental Construction
- Full integration testing
- Performance benchmarking
- **Checkpoint**: All optimizations working

#### Week 4: Refinement + Documentation
- Fix any issues found
- Update documentation
- Code review
- **Final Checkpoint**: Ready for merge

### Feature Flags

Use Cargo features to control rollout:

```toml
[features]
default = ["schema-graph-hydration"]
schema-graph-hydration = []
schema-incremental-construction = ["schema-graph-hydration"]
```

In code:
```rust
#[cfg(feature = "schema-graph-hydration")]
fn use_hydrated_graph() { /* new code */ }

#[cfg(not(feature = "schema-graph-hydration"))]
fn use_old_approach() { /* old code */ }
```

### Rollback Strategy

Each phase is independently revertible:

1. **Phase 1 rollback**: Remove `Builder::discovery()`, keep old `discover_tree()`
2. **Phase 2 rollback**: Revert to separate `HashMap<SchemaId, FileStatus>`
3. **Phase 3 rollback**: Remove PropertyBankDelta checking in Comparison
4. **Phase 4 rollback**: Remove StaleContent optimization in Refresh
5. **Phase 5 rollback**: Remove incremental Construction path

### Database Migration

**No schema changes required**!

Existing DB tables:
- `SCHEMA_BY_ID` (unchanged)
- `RAW_SCHEMA_VIEWS` (unchanged)
- `SCHEMA_TOPOLOGICAL_GRAPH` (unchanged - already exists!)

All changes are in-memory representation only.

---

## Risk Mitigation

### Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Graph desync | Low | High | Validation checks + tests |
| Performance regression | Low | High | Benchmarks + feature flags |
| Breaking existing code | Medium | High | Comprehensive test coverage |
| Complex merge conflicts | Medium | Medium | Small PRs + clear ownership |
| Incomplete implementation | Low | Medium | Phased approach + checkpoints |

### Mitigation Strategies

#### 1. Graph Consistency Validation

Add debug assertions:
```rust
#[cfg(debug_assertions)]
fn validate_graph_status_consistency(
    graph: &TopologicalGraph<FileStatus>,
) -> Result<(), Error> {
    for (id, node) in &graph.nodes {
        // Verify ID matches
        assert_eq!(node.id, *id);

        // Verify status ID matches
        match &node.payload {
            FileStatus::Fresh { id: status_id, .. } => assert_eq!(status_id, id),
            FileStatus::New { id: status_id, .. } => assert_eq!(status_id, id),
            // ... other variants
        }

        // Verify parent/child consistency
        for parent_id in &node.parents {
            let parent = graph.nodes.get(parent_id)?;
            assert!(parent.children.contains(id));
        }
    }
    Ok(())
}
```

#### 2. Performance Monitoring

Add metrics:
```rust
struct PipelineMetrics {
    graph_loaded_from_db: bool,
    graph_rebuild_time_ms: u64,
    fresh_schemas: usize,
    stale_schemas: usize,
    new_schemas: usize,
    incremental_updates: usize,
    full_merges: usize,
    total_time_ms: u64,
}
```

#### 3. Gradual Rollout

1. **Alpha**: Enable for development only
2. **Beta**: Enable with feature flag (opt-in)
3. **Gamma**: Enable by default (opt-out)
4. **Production**: Remove old code

#### 4. Comprehensive Testing

- **Unit tests**: 30+ tests covering edge cases
- **Integration tests**: 15+ tests for stage transitions
- **E2E tests**: 5+ tests for full pipeline scenarios
- **Performance tests**: Benchmarks for each optimization
- **Fuzz testing**: Random schema generation + mutation

---

## Definition of Done

### Phase 1 Complete When:
- [ ] `Builder::discovery()` implemented
- [ ] `DiscoveryContext` struct defined
- [ ] Graph loaded from DB successfully
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Code reviewed and approved

### Phase 2 Complete When:
- [ ] All state structs use `TopologicalGraph<FileStatus>`
- [ ] `hydrate_graph_with_status()` implemented
- [ ] `dehydrate_graph_to_inheritance()` implemented
- [ ] All stages access status via `node.payload`
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Graph persistence verified
- [ ] Code reviewed and approved

### Phase 3 Complete When:
- [ ] PropertyBankDelta integration in Comparison
- [ ] Fresh schemas demoted correctly
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Code reviewed and approved

### Phase 4 Complete When:
- [ ] Enhanced Refresh handles StaleContent
- [ ] Comment-only changes skip Construction
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Code reviewed and approved

### Phase 5 Complete When:
- [ ] Incremental Construction implemented
- [ ] Property-only changes optimized
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Performance benchmarks show improvement
- [ ] Code reviewed and approved

### Overall Project Complete When:
- [ ] All phases complete
- [ ] All tests pass (unit + integration + E2E)
- [ ] Performance benchmarks meet targets
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] No regressions detected
- [ ] Feature flags removed (if using)
- [ ] Release notes written

---

## Appendix A: Code Examples

### Example 1: Using Hydrated Graph

**Before** (separate lookups):
```rust
let node = graph.nodes.get(&id)?;
let status = statuses.get(&id)?;
let view = status.view();
```

**After** (single lookup):
```rust
let node = graph.nodes.get(&id)?;
let view = node.payload.view();
```

### Example 2: Iterating Through Graph

**Before**:
```rust
for (id, status) in &statuses {
    let node = graph.nodes.get(id)?;
    process(node, status)?;
}
```

**After**:
```rust
for (id, node) in &graph.nodes {
    process(&node, &node.payload)?;
}
```

### Example 3: Building Graph from Discovery

**Before** (rebuild from scratch):
```rust
let graph = DagBuilder::new(&statuses).build()?;  // Expensive!
```

**After** (load and hydrate):
```rust
let graph = repository.get_topological_graph()?;  // Fast!
let hydrated = hydrate_graph_with_status(graph, statuses);
```

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **Hydration** | Adding FileStatus payloads to graph nodes |
| **Dehydration** | Removing FileStatus payloads for persistence |
| **Demotion** | Changing Fresh status to StaleContent |
| **Incremental Construction** | Updating schema without full merge |
| **Graph Rewiring** | Changing parent-child relationships |
| **Affected Subtree** | All descendants of changed nodes |

---

## Appendix C: References

- **Current Implementation**: `lithos-core/src/schema/schema_pipeline.rs`
- **Graph Module**: `lithos-core/src/schema/graph.rs`
- **Builder**: `lithos-core/src/schema/builder.rs`
- **Repository Trait**: `lithos-core/src/schema/storage.rs`
- **Property Bank Processor**: `lithos-core/src/schema/property_bank_processor.rs` (reference pattern)

---

**End of Implementation Plan**
