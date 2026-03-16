# Schema Inheritance Caching - Complete Design & Implementation Plan

**Date**: 2026-03-16
**Purpose**: Optimize schema resolution by caching inheritance metadata
**Goal**: Eliminate redundant SchemaTree rebuilds when inheritance unchanged

---

## Table of Contents

1. [Current State Analysis](#part-1-current-state-analysis)
2. [Performance Bottleneck](#part-2-performance-bottleneck)
3. [Design Solution](#part-3-design-solution)
4. [Storage Design](#part-4-storage-design)
5. [Integration Design](#part-5-integration-design)
6. [Implementation Plan](#part-6-implementation-plan)

---

## Part 1: Current State Analysis

### Pipeline Overview

**File**: `lithos-core/src/schema/loader.rs`

```
load_schemas()
  ├─ Step 1-4: Load existing state, check staleness
  ├─ Step 5-6: Partition schemas, handle incremental resolution
  └─ Step 7: Full resolution for stale schemas
      ├─ RefExpander::expand_all() → Resolve property $refs
      ├─ Extender::build() → Build SchemaTree (6 phases) ← BOTTLENECK
      └─ Resolver::resolve() → Merge properties
```

**Extender::build() - 6 Phases** (`extender.rs:220-248`):

```rust
pub fn build(
    expanded: Vec<(SchemaId, RefExpandedSchema)>,
    known_parents: &HashMap<SchemaId, Schema>,
) -> Result<SchemaTree, SchemaError> {
    // Phase 1: build name ↔ id indexes
    let (name_to_id, id_to_name) = Self::build_name_indexes(&expanded, known_parents)?;

    // Phase 2: build node map with resolved parent IDs
    let mut nodes = Self::build_nodes(expanded, &name_to_id)?;

    // Phase 3: DFS cycle detection
    Self::detect_cycles(&nodes, known_parents, &id_to_name)?;

    // Phase 4: populate children lists
    Self::populate_children(&mut nodes);

    // Phase 5: compute inheritance depths
    Self::compute_depths(&mut nodes, known_parents);

    // Phase 6: Kahn's topological ordering
    let (order, roots) = Self::kahn_order(&nodes)?;

    Ok(SchemaTree { roots, nodes, order })
}
```

### What Gets Computed

**Per schema** (in SchemaNode):

- `parent_id: Option<SchemaId>` - Resolved from parent name
- `children: Vec<SchemaId>` - Populated by scanning all nodes
- `depth: NodeDepth` - Computed via BFS from roots
- `excludes: Vec<Box<str>>` - Copied from RawSchema

**Global** (in SchemaTree):

- `order: Vec<SchemaId>` - Topological ordering (Kahn's algorithm)
- `roots: Vec<SchemaId>` - Schemas with no parent

### Current Behavior

**Every load** (even if zero schemas changed inheritance):

1. Parse all stale schema files → `RawSchema`
2. Expand property refs → `RefExpandedSchema`
3. **Build entire SchemaTree** (all 6 phases)
4. Resolve properties for all schemas

**Problem**: Steps 3-4 are **redundant** if inheritance unchanged.

---

## Part 2: Performance Bottleneck

### Empirical Cost Analysis

**Test environment**: 1000 schemas, average depth 3, average 5 properties per schema

| Phase                | Operation               | Complexity   | Time (est) |
| -------------------- | ----------------------- | ------------ | ---------- |
| Ingestor             | Parse JSON files        | O(n)         | 20ms       |
| RefExpander          | Property ref lookup     | O(n × p)     | 10ms       |
| **Extender Phase 1** | Build name indexes      | O(n)         | 5ms        |
| **Extender Phase 2** | Resolve parent names    | O(n)         | 5ms        |
| **Extender Phase 3** | DFS cycle detection     | O(n × d)     | 10ms       |
| **Extender Phase 4** | Populate children       | O(n)         | 5ms        |
| **Extender Phase 5** | BFS depth               | O(n)         | 5ms        |
| **Extender Phase 6** | Kahn's topological sort | O(n + e)     | 10ms       |
| **Resolver**         | Merge properties        | O(n × d × p) | 30ms       |
| **Total**            |                         |              | **100ms**  |

**Bottleneck**: Extender (40ms) + Resolver (30ms) = **70% of total load time**

### Why Extender Is Expensive

**For each schema**, we:

1. Look up parent name in HashMap
2. Walk parent chain to check cycles
3. Count in-degrees for topological sort
4. BFS to compute depths

**Key observation**: If a schema's `extends` field hasn't changed, all of this is **redundant**.

### Existing Optimization Patterns

**PropertyBank caching** (`loader.rs:110-128`):

```rust
// Already implements caching!
let bank_view = self.repository.get_raw_property_bank_view()?;
if let Some(view) = bank_view {
    if view.is_fresh(&bank_path, bank_metadata)? {
        // Use cached bank (skip file parsing)
        bank = self.repository.get_property_bank()?.unwrap();
    }
}
```

**Incremental resolution** (`loader.rs:188-222`):

```rust
// Already skips full resolution when possible!
if !existing_file_unchanged.is_empty() && !changed_properties.is_empty() {
    let affected_map = self.repository
        .find_schemas_using_properties(&changed_properties)?;

    // Only resolve schemas affected by PropertyBank changes
    for schema in &stored_schemas {
        if let Some(affected_props) = affected_map.get(schema.id()) {
            let updated = Resolver::resolve_affected_properties(...)?;
        }
    }
}
```

**Gap**: No equivalent optimization for inheritance metadata.

---

## Part 3: Design Solution

### Core Idea

**Cache precomputed inheritance metadata** to skip Extender when inheritance unchanged.

### What to Cache

Based on analysis of what Extender computes:

**Per-schema metadata**:

- `parent_id: Option<SchemaId>` - Resolved parent (from name → ID lookup)
- `ancestors: Vec<SchemaId>` - **Full ancestor chain** (computed by walking parents)
- `children: Vec<SchemaId>` - Direct children (computed by scanning all schemas)
- `excludes: Vec<Box<str>>` - Property exclusions (from RawSchema)
- `ancestors_hash: u64` - **Hash for O(1) staleness detection**

**NOT caching**:

- ❌ Depth - Can derive from `ancestors.len() + 1`
- ❌ Topological order - Can reconstruct from metadata if needed
- ❌ Properties - Change frequently via PropertyBank

### Staleness Detection

**A schema's inheritance metadata is stale if**:

1. Schema changes `extends` field (parent name changed)
2. Schema changes `excludes` field
3. Parent's ancestor chain changed (**transitive staleness**)

**Hash-based detection** (O(1)):

```rust
// Compute hash recursively
fn compute_hash(parent_id: Option<SchemaId>, parent_hash: u64) -> u64 {
    hash(parent_id || parent_hash)
}

// Check staleness
fn is_stale(cached: &Metadata, current_parent_id: SchemaId, parent_metadata: &Metadata) -> bool {
    let expected_hash = compute_hash(Some(current_parent_id), parent_metadata.ancestors_hash);
    expected_hash != cached.ancestors_hash
}
```

**Why this works**: If grandparent changes, parent's hash changes, so child's expected hash changes.

### Fast Path vs Slow Path

**Fast path** (inheritance unchanged):

```
load_schemas()
  ├─ Check inheritance staleness (hash comparison)
  ├─ All fresh? → resolve_from_metadata_cache()
  │   ├─ Load cached ancestors for each schema
  │   ├─ Collect ancestor properties (no tree building!)
  │   └─ Resolver::merge_properties()
  └─ Save resolved schemas
```

**Slow path** (inheritance changed):

```
load_schemas()
  ├─ Check inheritance staleness
  ├─ Some stale? → Full rebuild
  │   ├─ Extender::build() (rebuild tree)
  │   ├─ Resolver::resolve()
  │   └─ update_inheritance_metadata() ← Update cache
  └─ Save resolved schemas + metadata
```

---

## Part 4: Storage Design

### View Type: SchemaInheritanceMetadata

**File**: `lithos-core/src/schema/views/inheritance.rs`

```rust
/// Per-schema inheritance metadata.
///
/// **Storage pattern:**
/// - Table: `schema_inheritance` (regular table)
/// - Key: SchemaId (as UUID string)
/// - Value: `SchemaInheritanceMetadata` (rkyv-serialized bytes)
///
/// **Purpose**: Cache inheritance graph to avoid rebuilding SchemaTree.
///
/// **Staleness**: Rebuild when:
/// - Schema changes `extends` field
/// - Schema changes `excludes` field
/// - Parent schema's ancestors change (detected via `ancestors_hash`)
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaInheritanceMetadata {
    /// Immediate parent ID, or None for root schemas.
    pub parent_id: Option<SchemaId>,

    /// Full ancestor chain: [parent, grandparent, great-grandparent, ...].
    /// Ordered closest-first for efficient property merging (merge in reverse).
    /// Uses IDs (not names) for:
    /// - Smaller storage (16 bytes vs 24+ bytes per ancestor)
    /// - No name lookup needed during resolution
    pub ancestors: Vec<SchemaId>,

    /// Direct children IDs (cached for resolver + descendant queries).
    pub children: Vec<SchemaId>,

    /// Property names this schema excludes from ancestors.
    /// Stored here (not in Schema aggregate) because needed for resolution.
    pub excludes: Vec<Box<str>>,

    /// Hash of (parent_id || parent.ancestors_hash) for O(1) staleness detection.
    /// Recursively incorporates ancestor changes for transitive staleness.
    pub ancestors_hash: u64,
}

impl SchemaInheritanceMetadata {
    /// Serialize to bytes for storage.
    pub fn to_bytes(&self) -> Result<Vec<u8>, DbError> {
        rkyv::to_bytes(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| DbError::Serialization(e.to_string()))
    }

    /// Compute ancestors hash from parent ID and parent's hash.
    ///
    /// Hash is recursive: child_hash = hash(parent_id || parent_hash)
    /// This enables O(1) transitive staleness detection.
    pub fn compute_hash(parent_id: Option<SchemaId>, parent_hash: u64) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        parent_id.hash(&mut hasher);
        parent_hash.hash(&mut hasher);
        hasher.finish()
    }
}
```

**Size analysis**:

- `parent_id`: 17 bytes (Option + UUID)
- `ancestors`: 24 bytes stack + heap (Vec<SchemaId>)
- `children`: 24 bytes stack + heap (Vec<SchemaId>)
- `excludes`: 24 bytes stack + heap (Vec<Box<str>>)
- `ancestors_hash`: 8 bytes
- **Total**: ~97 bytes stack + heap allocations

**Design decisions**:

1. **Why not store `schema_id`?**
   → It's the table key - redundant to store in value

2. **Why not store `depth`?**
   → Can derive from `ancestors.len() + 1`

3. **Why store `ancestors: Vec<SchemaId>`?**
   → Saves O(depth) table lookups during resolution (30µs per schema)

4. **Why store `children: Vec<SchemaId>`?**
   → Needed for `Schema.children` field + descendant queries (avoids O(n) scan)

5. **Why store `excludes`?**
   → Needed during resolution, can't be in `Schema` (already applied)

6. **Why `Vec<SchemaId>` not `Vec<SchemaName>`?**
   → 33% smaller (16 bytes vs 24+ bytes), no HashMap lookup during resolution

### Table Design

**Table**: `schema_inheritance` (regular table, NOT multimap)

```rust
// File: lithos-core/src/schema/mod.rs
pub(crate) const SCHEMA_INHERITANCE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_inheritance");
```

**Why regular table, not multimap?**

| Aspect            | Multimap        | Regular Table + Vec               |
| ----------------- | --------------- | --------------------------------- |
| Write 3 children  | 3 inserts       | 1 insert                          |
| Read all children | 3 deserializes  | 1 deserialize                     |
| Update            | 1 insert/delete | 1 read + modify + write           |
| Our workload      | N/A             | **Read-heavy** (every resolution) |

**Decision**: Regular table - better for read-heavy workload.

---

## Part 5: Integration Design

### Repository Trait Extensions

**File**: `lithos-core/src/schema/storage.rs`

```rust
pub trait Repository {
    // ... existing methods ...

    /// Get inheritance metadata for a schema.
    ///
    /// Returns None if schema has no cached metadata (needs rebuild).
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the query fails.
    fn get_inheritance_metadata(
        &self,
        id: SchemaId,
    ) -> Result<Option<SchemaInheritanceMetadata>, Self::Error>;

    /// Save inheritance metadata for a schema.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the save fails.
    fn save_inheritance_metadata(
        &self,
        id: SchemaId,
        metadata: &SchemaInheritanceMetadata,
    ) -> Result<(), Self::Error>;

    /// Delete inheritance metadata for a schema.
    ///
    /// Used during invalidation when schema changes `extends`.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the delete fails.
    fn delete_inheritance_metadata(
        &self,
        id: SchemaId,
    ) -> Result<(), Self::Error>;

    /// Batch save inheritance metadata for multiple schemas.
    ///
    /// Called after tree building to update entire graph.
    ///
    /// # Errors
    ///
    /// Returns storage-specific error if the batch save fails.
    fn save_inheritance_metadata_batch(
        &self,
        metadata: &[(SchemaId, SchemaInheritanceMetadata)],
    ) -> Result<(), Self::Error>;
}
```

### Loader Extensions

**File**: `lithos-core/src/schema/loader.rs`

#### Method 1: Check Inheritance Staleness

```rust
/// Check which schemas have stale inheritance metadata.
///
/// Returns list of schema IDs that need tree rebuild.
fn check_inheritance_staleness(
    &self,
    schemas: &[(SchemaId, RawSchema)],
) -> Result<Vec<SchemaId>, SchemaLoaderError> {
    let mut stale = Vec::new();

    for (schema_id, raw) in schemas {
        if self.is_inheritance_stale(*schema_id, raw)? {
            stale.push(*schema_id);
        }
    }

    Ok(stale)
}
```

#### Method 2: Per-Schema Staleness Check

```rust
/// Check if a single schema's inheritance metadata is stale.
///
/// Checks:
/// 1. Parent name changed (extends field)
/// 2. Excludes changed
/// 3. Parent's ancestors changed (transitive staleness)
fn is_inheritance_stale(
    &self,
    schema_id: SchemaId,
    raw: &RawSchema,
) -> Result<bool, SchemaLoaderError> {
    // Load cached metadata
    let cached = self.repository
        .get_inheritance_metadata(schema_id)
        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

    let Some(cached) = cached else {
        return Ok(true);  // No cache: must rebuild
    };

    // Check if extends changed
    let current_parent_id = raw.extends.as_ref()
        .map(|name| self.name_to_id.get(name.as_ref()))
        .transpose()
        .map_err(|_| SchemaLoaderError::ParentNotFound(raw.name.clone()))?
        .copied();

    if cached.parent_id != current_parent_id {
        return Ok(true);  // Parent changed
    }

    // Check if excludes changed
    if raw.excludes != cached.excludes {
        return Ok(true);  // Excludes changed
    }

    // Check if parent's ancestors changed (transitive)
    if let Some(parent_id) = current_parent_id {
        let parent_metadata = self.repository
            .get_inheritance_metadata(parent_id)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or_else(|| SchemaLoaderError::MissingMetadata(parent_id))?;

        let expected_hash = SchemaInheritanceMetadata::compute_hash(
            Some(parent_id),
            parent_metadata.ancestors_hash,
        );

        if expected_hash != cached.ancestors_hash {
            return Ok(true);  // Parent's ancestors changed
        }
    }

    Ok(false)  // Fresh!
}
```

#### Method 3: Fast Path Resolution

```rust
/// Resolve schemas using cached inheritance metadata (fast path).
///
/// Skips Extender entirely, directly merges properties using cached ancestors.
fn resolve_from_metadata_cache(
    &self,
    schemas: Vec<(SchemaId, RawSchema)>,
) -> Result<Vec<Schema>, SchemaLoaderError> {
    let mut result = Vec::with_capacity(schemas.len());

    for (schema_id, raw) in schemas {
        // Load cached metadata
        let metadata = self.repository
            .get_inheritance_metadata(schema_id)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
            .ok_or_else(|| SchemaLoaderError::MissingMetadata(schema_id))?;

        // Expand own properties (still need RefExpander)
        let expanded = RefExpander::new(&self.bank).expand_single(raw.clone())?;

        // Collect ancestor properties using cached ancestor IDs
        let mut ancestor_props = Vec::new();
        for &ancestor_id in metadata.ancestors.iter().rev() {
            let ancestor = self.repository
                .find_schema_by_id(ancestor_id)
                .map_err(|e| SchemaLoaderError::Repository(e.into()))?
                .ok_or_else(|| SchemaLoaderError::AncestorNotFound(ancestor_id))?;

            ancestor_props.extend(ancestor.properties().iter().cloned());
        }

        // Merge properties using cached excludes
        let merged = Resolver::merge_properties(
            &ancestor_props,
            &expanded.properties,
            &metadata.excludes,
        );

        // Build schema using cached parent_id and children
        let schema = Schema::new(
            schema_id,
            expanded.name,
            metadata.parent_id,
            metadata.children.clone(),
            merged,
        );

        result.push(schema);
    }

    Ok(result)
}
```

#### Method 4: Update Metadata Cache

```rust
/// Update inheritance metadata cache after tree building.
///
/// Extracts metadata from SchemaTree and saves to database.
fn update_inheritance_metadata(
    &self,
    tree: &SchemaTree,
) -> Result<(), SchemaLoaderError> {
    let mut batch = Vec::with_capacity(tree.nodes().len());

    for &schema_id in tree.nodes() {
        let node = tree.get(schema_id)
            .map_err(SchemaError::from)
            .map_err(SchemaLoaderError::Schema)?;

        // Build ancestor chain from tree
        let ancestors = self.build_ancestor_chain(schema_id, tree)?;

        // Compute hash
        let parent_metadata = node.parent_id
            .and_then(|pid| {
                self.repository
                    .get_inheritance_metadata(pid)
                    .ok()
                    .flatten()
            });

        let ancestors_hash = SchemaInheritanceMetadata::compute_hash(
            node.parent_id,
            parent_metadata.map(|m| m.ancestors_hash).unwrap_or(0),
        );

        // Create metadata
        let metadata = SchemaInheritanceMetadata {
            parent_id: node.parent_id,
            ancestors,
            children: node.children.clone(),
            excludes: node.excludes.clone(),
            ancestors_hash,
        };

        batch.push((schema_id, metadata));
    }

    // Batch save
    self.repository
        .save_inheritance_metadata_batch(&batch)
        .map_err(|e| SchemaLoaderError::Repository(e.into()))?;

    Ok(())
}
```

#### Method 5: Build Ancestor Chain

```rust
/// Build ancestor chain for a schema from SchemaTree.
fn build_ancestor_chain(
    &self,
    schema_id: SchemaId,
    tree: &SchemaTree,
) -> Result<Vec<SchemaId>, SchemaLoaderError> {
    let mut ancestors = Vec::new();
    let mut current_id = schema_id;

    while let Some(node) = tree.get(current_id).ok() {
        if let Some(parent_id) = node.parent_id {
            ancestors.push(parent_id);
            current_id = parent_id;
        } else {
            break;  // Root reached
        }
    }

    Ok(ancestors)
}
```

#### Method 6: Invalidate Metadata

```rust
/// Invalidate inheritance metadata for a schema and all descendants.
///
/// Used when a schema changes its `extends` field.
fn invalidate_inheritance_metadata(
    &self,
    schema_id: SchemaId,
) -> Result<(), SchemaLoaderError> {
    // Get all descendants (BFS)
    let descendants = self.get_all_descendants(schema_id)?;

    // Delete metadata for schema + descendants
    for id in std::iter::once(schema_id).chain(descendants) {
        self.repository
            .delete_inheritance_metadata(id)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
    }

    Ok(())
}

/// Get all descendants of a schema (transitive children).
fn get_all_descendants(
    &self,
    parent_id: SchemaId,
) -> Result<Vec<SchemaId>, SchemaLoaderError> {
    let mut descendants = Vec::new();
    let mut queue = VecDeque::from([parent_id]);
    let mut visited = HashSet::new();

    while let Some(current_id) = queue.pop_front() {
        if !visited.insert(current_id) {
            continue;  // Already processed
        }

        // Load metadata
        let Some(metadata) = self.repository
            .get_inheritance_metadata(current_id)
            .map_err(|e| SchemaLoaderError::Repository(e.into()))?
        else {
            continue;  // No metadata (stale or root)
        };

        // Add children to queue
        for &child_id in &metadata.children {
            descendants.push(child_id);
            queue.push_back(child_id);
        }
    }

    Ok(descendants)
}
```

### Modified Load Flow

**File**: `lithos-core/src/schema/loader.rs:load_schemas()`

**Current** (line 240-249):

```rust
// Step 7: Full resolution for new + file-changed schemas
if !schemas_for_full_resolution.is_empty() {
    let expanded = RefExpander::new(&bank).expand_all(schemas)?;
    let tree = Extender::build(expanded, &known_parents)?;
    let resolved = Resolver::resolve(&tree, &known_parents)?;
    resolved.extend(full_resolved);
}
```

**Optimized**:

```rust
// Step 7: Check inheritance staleness
let inheritance_stale = self.check_inheritance_staleness(&schemas_for_full_resolution)?;

if inheritance_stale.is_empty() {
    // Fast path: Use cached metadata (skip Extender!)
    let resolved_from_cache = self.resolve_from_metadata_cache(schemas_for_full_resolution)?;
    resolved.extend(resolved_from_cache);
} else {
    // Slow path: Rebuild tree + update cache
    let expanded = RefExpander::new(&bank).expand_all(schemas_for_full_resolution.clone())?;
    let tree = Extender::build(expanded, &known_parents)?;
    let full_resolved = Resolver::resolve(&tree, &known_parents)?;

    // Update inheritance metadata cache
    self.update_inheritance_metadata(&tree)?;

    resolved.extend(full_resolved);
}
```

---

## Part 6: Implementation Plan

### Phase 1: Add View Type and Table (2 hours)

**Files to modify**:

1. `lithos-core/src/schema/views/inheritance.rs` - Add `SchemaInheritanceMetadata` struct
2. `lithos-core/src/schema/views/mod.rs` - Export new type
3. `lithos-core/src/schema/mod.rs` - Add `SCHEMA_INHERITANCE` table definition

**Tasks**:

- [ ] Define `SchemaInheritanceMetadata` struct with rkyv derives
- [ ] Implement `to_bytes()` method
- [ ] Implement `compute_hash()` static method
- [ ] Add table constant to mod.rs
- [ ] Write unit tests for hash computation

**Verification**:

```bash
mise run test:unit:schema
```

### Phase 2: Add Repository Methods (2 hours)

**Files to modify**:

1. `lithos-core/src/schema/storage.rs` - Add trait methods
2. `lithos-core/src/schema/storage.rs` - Implement for RedbStorage

**Tasks**:

- [ ] Add 4 methods to Repository trait
- [ ] Implement `get_inheritance_metadata()` for RedbStorage
- [ ] Implement `save_inheritance_metadata()` for RedbStorage
- [ ] Implement `delete_inheritance_metadata()` for RedbStorage
- [ ] Implement `save_inheritance_metadata_batch()` for RedbStorage
- [ ] Add stub implementations for InMemoryStorage, FakeStorage

**Verification**:

```bash
mise run test:unit:schema
```

### Phase 3: Add Loader Staleness Detection (2 hours)

**Files to modify**:

1. `lithos-core/src/schema/loader.rs` - Add staleness methods

**Tasks**:

- [ ] Implement `check_inheritance_staleness()`
- [ ] Implement `is_inheritance_stale()`
- [ ] Write tests for staleness detection
  - Test: Parent changed (name)
  - Test: Excludes changed
  - Test: Transitive staleness (grandparent changed)

**Verification**:

```bash
mise run test:unit:schema
```

### Phase 4: Implement Fast Path (2-3 hours)

**Files to modify**:

1. `lithos-core/src/schema/loader.rs` - Add fast path resolution

**Tasks**:

- [ ] Implement `resolve_from_metadata_cache()`
- [ ] Implement `build_ancestor_chain()`
- [ ] Update `load_schemas()` to use fast path
- [ ] Write tests for cached resolution
  - Test: Single schema resolved from cache
  - Test: Deep inheritance (3+ levels) from cache
  - Test: Excludes applied correctly from cache

**Verification**:

```bash
mise run test:unit:schema
cargo run --bin lithos-cli -- schema list  # Manual smoke test
```

### Phase 5: Implement Cache Update (1-2 hours)

**Files to modify**:

1. `lithos-core/src/schema/loader.rs` - Add metadata update

**Tasks**:

- [ ] Implement `update_inheritance_metadata()`
- [ ] Call from `load_schemas()` after tree building
- [ ] Write tests for cache update
  - Test: Metadata saved after tree build
  - Test: Hash computed correctly
  - Test: Ancestors chain extracted correctly

**Verification**:

```bash
mise run test:unit:schema
```

### Phase 6: Implement Invalidation (1 hour)

**Files to modify**:

1. `lithos-core/src/schema/loader.rs` - Add invalidation

**Tasks**:

- [ ] Implement `invalidate_inheritance_metadata()`
- [ ] Implement `get_all_descendants()`
- [ ] Write tests for invalidation
  - Test: Invalidate schema + descendants
  - Test: BFS traversal correctness

**Verification**:

```bash
mise run test:unit:schema
```

### Phase 7: Integration Testing (1-2 hours)

**Files to create/modify**:

1. `lithos-core/tests/schema_inheritance_caching.rs` - New integration test

**Tasks**:

- [ ] Test: Load → modify extends → reload (should rebuild)
- [ ] Test: Load → modify properties → reload (should use cache)
- [ ] Test: Load → modify grandparent extends → reload (transitive stale)
- [ ] Test: Verify performance gain with benchmark

**Verification**:

```bash
mise run test:integration
```

### Phase 8: Documentation (1 hour)

**Files to create/modify**:

1. `docs/adr/NNNN-schema-inheritance-caching.md` - New ADR

**Tasks**:

- [ ] Document decision to cache inheritance metadata
- [ ] Document hash-based staleness detection
- [ ] Document trade-offs (complexity vs performance)

**Verification**:

```bash
mise run adr:validate
```

---

## Total Estimate: 12-15 hours

| Phase                  | Effort | Dependencies |
| ---------------------- | ------ | ------------ |
| 1. View type + table   | 2h     | None         |
| 2. Repository methods  | 2h     | Phase 1      |
| 3. Staleness detection | 2h     | Phase 2      |
| 4. Fast path           | 2-3h   | Phase 3      |
| 5. Cache update        | 1-2h   | Phase 4      |
| 6. Invalidation        | 1h     | Phase 4      |
| 7. Integration tests   | 1-2h   | Phases 4-6   |
| 8. Documentation       | 1h     | All phases   |

---

## Expected Performance Impact

**Common case: File changed, inheritance unchanged** (estimated 95% of loads):

- **Before**: 100ms (Extender 40ms + Resolver 30ms + overhead 30ms)
- **After**: 60ms (skip Extender, just Resolver 30ms + metadata lookups 30ms)
- **Speedup**: 1.67x

**Rare case: Inheritance changed** (estimated 5% of loads):

- **Before**: 100ms
- **After**: 115ms (rebuild tree 40ms + update metadata 15ms + Resolver 30ms + overhead 30ms)
- **Slowdown**: 15% (acceptable - rare case)

**Amortized average**: 0.95 × 60ms + 0.05 × 115ms = **62.75ms** (vs 100ms)

- **Overall speedup**: 1.59x

---

## Risks and Mitigations

### Risk 1: Cache Invalidation Bugs

**Symptom**: Stale metadata used, incorrect property inheritance

**Mitigation**:

- Comprehensive test coverage for staleness detection
- Hash-based transitive staleness (automatic propagation)
- Integration tests that modify schemas and verify reload

### Risk 2: Hash Collisions

**Symptom**: Different ancestor chains produce same hash (false cache hit)

**Mitigation**:

- Use cryptographic hash (SHA256) instead of DefaultHasher
- Probability of collision: negligible for realistic vault sizes

### Risk 3: Storage Overhead

**Symptom**: Database size increases

**Analysis**:

- Per schema: ~100 bytes stack + ~50 bytes heap (avg 3 ancestors, 2 children, 2 excludes)
- For 1000 schemas: ~150KB total
- **Negligible** compared to property data

### Risk 4: Complexity

**Symptom**: Code harder to maintain

**Mitigation**:

- Clear separation: staleness logic in Loader, storage logic in Repository
- Comprehensive documentation in ADR
- Integration tests demonstrate correct behavior

---

## Appendix: MetadataMenu Comparison

**See**: `docs/refs/obsidian/metadatamenu-reference.md` Appendix C

**Key differences**:

1. **MetadataMenu**: Caches ancestor chain globally as `Map<string, string[]>`
2. **Lithos design**: Caches per-schema metadata with hash-based staleness

**Why different**:

- MetadataMenu has global index rebuild trigger
- Lithos has per-schema staleness detection (more granular)
- Lithos needs to integrate with existing Repository pattern
