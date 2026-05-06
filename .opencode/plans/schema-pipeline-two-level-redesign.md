# Schema Pipeline: Two-Level Typestate Redesign

**Date**: 2026-03-30
**Status**: **PLANNING**
**Purpose**: Redesign the schema ingestion pipeline from a per-schema typestate to a global pipeline with per-node states.

## 1. Global Pipeline Architecture

The `SchemaPipeline<Stage>` state machine manages the entire directory-level process. It coordinates a `TopologicalGraph<FileNode>` through a sequence of batch and parallel operations.

### Global Stages
1.  **Discovery**: Scan FS/DB, identify all files, detect deletions, and create the initial `TopologicalGraph<FileNode>`.
2.  **Comparison**: [PARALLEL] Hashing and timestamp checks for all nodes. Early metadata refresh for stale-timestamp-only files.
3.  **Graphing**: [BATCH] structural patching of edges based on `extends` changes, depth calculation, and fail-fast cycle/depth validation.
4.  **Analysis**: [PARALLEL] Compute `SchemaPropertyDelta` and `ExcludesDelta` for all nodes.
5.  **Construction**: [TOPOLOGICAL] Level-by-level $ref expansion and property merging.
6.  **Completion**: [BATCH] Final persistence and cleanup.

---

## 2. Per-Node State (Node Lifecycle)

Each node in the `TopologicalGraph<T>` carries its own lifecycle state. This allows the pipeline to process fresh, changed, and new schemas together.

### `NodeStatus` Transitions
1.  **Discovered**: Basic metadata (ID, name, path, times, and cached `RawSchemaView`).
2.  **Compared**: Result of timestamp/hash checks. Categorized as `Fresh`, `StaleTimestamps`, or `StaleContent`.
3.  **Graphed**: Carries the `ExtendsDelta` and parsed `RawSchema` (if content changed).
4.  **Analyzed**: Carries `SchemaPropertyDelta` and `ExcludesDelta`.
5.  **Constructed**: The final resolved `Schema` and an `is_changed` flag.

---

## 3. Key Optimizations

- **Rayon Integration**: Independent hashing and analysis use `par_iter()` across nodes.
- **Topological Pass-Through**: During `Construction`, a schema is only rebuilt if it is `StaleContent` **OR** if its parent was changed (`parent.is_changed`).
- **Fail-Fast Structural Check**: We perform structural validation (cycle detection, 10-level max depth) during the `Graphing` stage before any expensive property work.
- **Lean Graph**: The graph remains structural (IDs and names only). Properties and excludes are property-level concerns.

---

## 4. Storage Pattern

- `SCHEMA_TOPOLOGICAL_GRAPH`: Singleton singleton `TopologicalGraphCache` (sorted order/roots/count).
- `SCHEMA_CHILDREN_BY_PARENT`: Multimap `ParentId -> Vec<ChildMetadata>` (child IDs + depths).
- `SCHEMA_ANCESTORS_BY_ID`: Table `SchemaId -> AncestorMetadata` (ancestry chain + hash).
