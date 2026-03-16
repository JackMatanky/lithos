# Inheritance Storage Design - First Principles Analysis

**Date**: 2026-03-16
**Goal**: Design optimal redb-based storage for schema inheritance, from scratch
**Approach**: Ruthless analysis of entire pipeline to minimize computation

---

## Part 1: Pipeline Analysis - What Actually Happens?

### Current Pipeline (Every Load)

```
Files → Ingestor → RawSchema
          ↓
      RefExpander → RefExpandedSchema (property $refs → PropertyId)
          ↓
      Extender → SchemaTree (6 phases!)
          ↓                  Phase 1: Build name indexes
          ↓                  Phase 2: Build nodes (resolve parent names → IDs)
          ↓                  Phase 3: DFS cycle detection
          ↓                  Phase 4: Populate children lists
          ↓                  Phase 5: BFS depth computation
          ↓                  Phase 6: Kahn's topological sort
          ↓
      Resolver → Schema (merge properties from parent chain)
          ↓
      Repository → redb (persist)
```

### Cost Analysis

**For 1000 schemas, avg depth 3**:

| Phase | Operation | Complexity | Time (est) |
|-------|-----------|------------|------------|
| Ingestor | Parse JSON | O(n) | 20ms |
| RefExpander | Property lookup | O(n × p) where p = avg properties | 10ms |
| **Extender Phase 1** | Build name indexes | O(n) | 5ms |
| **Extender Phase 2** | Resolve parent names | O(n) | 5ms |
| **Extender Phase 3** | DFS cycle detection | O(n × d) | 10ms |
| **Extender Phase 4** | Populate children | O(n) | 5ms |
| **Extender Phase 5** | BFS depth | O(n) | 5ms |
| **Extender Phase 6** | Kahn's topological sort | O(n + e) ≈ O(n) | 10ms |
| **Resolver** | Merge properties | O(n × d × p) | 30ms |
| **Total** | | | **~100ms** |

**Bottleneck**: Extender (40ms) + Resolver (30ms) = **70% of total time**

---

## Part 2: What Can We Precompute & Cache?

### Critical Question: What Changes Rarely?

**Analyzed loader.rs:170-266**:

Staleness categories:
1. **New schemas** → Need full resolution
2. **File changed** → Need full resolution
3. **File unchanged, but PropertyBank changed** → Incremental resolution (line 212-221)

**Key insight**: Step 3 already implements **incremental resolution** for unchanged files!

```rust
// loader.rs:212-219
for schema in &stored_schemas {
    if let Some(affected_props) = affected_map.get(schema.id()) {
        let updated = Resolver::resolve_affected_properties(
            schema,
            affected_props,
            &bank,
        )?;
        resolved.push(updated);
    }
}
```

**This proves**: The architecture ALREADY supports skipping resolution when possible.

**Gap**: No equivalent optimization for inheritance changes.

### What Should We Cache?

**Don't cache**:
- ❌ Properties (change frequently via PropertyBank)
- ❌ Property specs (change with PropertyBank)
- ❌ Resolved schemas (properties change)

**Do cache**:
- ✅ Name → ID mappings (rarely change) ← **Already done** in `list_schema_name_id_pairs()`
- ✅ Parent → ID relationships (change only when schema changes `extends`)
- ✅ Ancestor chains (change only when parent chain changes)
- ✅ Children lists (change only when child changes `extends`)
- ✅ Inheritance depth (change only when parent chain changes)
- ✅ Topological order (change only when inheritance graph changes)

---

## Part 3: Ruthless Analysis - What's Actually Expensive?

### Extender Phase Breakdown

**Phase 1: Build name indexes** (5ms)
```rust
// Build: name → id and id → name hashmaps
for (id, schema) in known_parents {
    name_to_id.insert(schema.name(), *id);
}
```
- **Can we cache this?** YES ✅ Already cached via `list_schema_name_id_pairs()`
- **Savings**: 5ms → 0ms if used

**Phase 2: Resolve parent names → IDs** (5ms)
```rust
// For each schema, look up parent name in name_to_id
let parent_id = schema.extends
    .as_ref()
    .map(|name| name_to_id.get(name))
    .transpose()?;
```
- **Can we cache this?** YES ✅ Store `parent_id: Option<SchemaId>` directly
- **Savings**: 5ms → 0ms

**Phase 3: DFS cycle detection** (10ms)
```rust
fn detect_cycles_dfs(node_id, visited, rec_stack) {
    if rec_stack.contains(node_id) { return Err(Cycle); }
    // Recurse to parent
}
```
- **Can we cache this?** PARTIALLY - if we cache acyclic property, can skip check
- **But**: Need to re-check only schemas with changed `extends`
- **Savings**: 10ms → 2ms (only check changed schemas)

**Phase 4: Populate children** (5ms)
```rust
// For each node, add self to parent's children list
for node in nodes {
    if let Some(parent_id) = node.parent_id {
        nodes[parent_id].children.push(node.id);
    }
}
```
- **Can we cache this?** YES ✅ Store `children: Vec<SchemaId>` per schema
- **Savings**: 5ms → 0ms

**Phase 5: BFS depth** (5ms)
```rust
// BFS from roots, incrementing depth
queue.push_all(roots);
while let Some(id) = queue.pop() {
    for child in node.children {
        depth[child] = depth[id] + 1;
    }
}
```
- **Can we cache this?** YES ✅ Store `depth: u8` per schema
- **Savings**: 5ms → 0ms

**Phase 6: Kahn's topological sort** (10ms)
```rust
// Count in-degrees, process zero-degree nodes
let mut in_degree = HashMap::new();
let mut queue = VecDeque::from(roots);
while let Some(id) = queue.pop() {
    for child in node.children {
        in_degree[child] -= 1;
        if in_degree[child] == 0 { queue.push(child); }
    }
}
```
- **Can we cache this?** YES ✅ Store `topological_order: Vec<SchemaId>` globally
- **Savings**: 10ms → 0ms

**Total Extender savings**: 40ms → ~2ms (95% reduction!)

### Resolver Analysis

**Current** (resolver.rs:73-151):
```rust
for &id in tree.nodes() {  // In topological order
    let parent_props = resolved_cache.get(parent_id)
        .or_else(|| known_parents.get(parent_id))
        .map(Schema::properties);

    let merged = merge_properties(parent_props, own_props, excludes);
}
```

**Cost**: O(n × d × p) where:
- n = schemas to resolve
- d = avg depth
- p = avg properties per schema

**Can we optimize?**

**Key insight**: If a schema's file is unchanged AND its parent chain is unchanged, the resolved properties are IDENTICAL.

**But**: Properties can change via PropertyBank updates (handled by incremental resolution).

**Conclusion**: Can't skip Resolver entirely, but can:
1. Use cached ancestor chain to avoid parent lookups
2. Pre-sorted merge (already done)

**Savings**: Minimal for Resolver itself, but enables skipping Extender entirely.

---

## Part 4: First Principles - What Should We Store?

### Design Principle: Separate Computation from Data

**Three layers**:
1. **Raw layer**: What user wrote in file
2. **Metadata layer**: Precomputed relationships (what we cache)
3. **Resolved layer**: Final schemas with merged properties

### Metadata We Should Cache

**Per-schema metadata** (changes only when schema changes `extends`):

```rust
pub struct SchemaInheritanceMetadata {
    /// Schema ID
    pub schema_id: SchemaId,

    /// Immediate parent ID (None for roots)
    pub parent_id: Option<SchemaId>,

    /// Full ancestor chain: [parent, grandparent, great-grandparent, ...]
    /// Ordered closest-first for easy property merging
    pub ancestors: Vec<SchemaId>,  // Use IDs, not names (smaller, no lookup)

    /// Direct children IDs
    pub children: Vec<SchemaId>,

    /// Inheritance depth (1 = root, 2 = one level deep, etc.)
    pub depth: u8,

    /// Property names to exclude from ancestors
    /// Stored here (not in Schema) because needed for resolution
    pub excludes: Vec<Box<str>>,

    /// Hash of (parent_id + parent.ancestors_hash) for O(1) staleness detection
    pub ancestors_hash: u64,
}
```

**Global metadata** (changes when ANY schema changes `extends`):

```rust
pub struct SchemaGraphMetadata {
    /// Topological order for resolution (roots first, leaves last)
    pub resolution_order: Vec<SchemaId>,

    /// Root schema IDs (no parent)
    pub roots: Vec<SchemaId>,

    /// Hash of entire graph structure for O(1) staleness detection
    pub graph_hash: u64,

    /// When this graph was computed
    pub computed_at: SystemTime,
}
```

### Storage Design (redb tables)

**Table 1: schema_inheritance** (per-schema metadata)
```rust
// Key: SchemaId (as UUID string)
// Value: SchemaInheritanceMetadata (rkyv bytes)
pub const SCHEMA_INHERITANCE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_inheritance");
```

**Table 2: schema_graph** (global metadata, singleton)
```rust
// Key: Constant "graph"
// Value: SchemaGraphMetadata (rkyv bytes)
pub const SCHEMA_GRAPH: TableDefinition<&str, &[u8]> =
    TableDefinition::new("schema_graph");
```

**Why this design?**

1. **Single source of truth**: Each piece of data stored once
2. **Minimal redundancy**: Only `ancestors` somewhat duplicates parent chain
3. **Fast staleness detection**: Compare hashes, not entire structures
4. **Efficient queries**:
   - Get ancestors: O(1) table lookup
   - Get children: O(1) table lookup
   - Get resolution order: O(1) singleton lookup

---

## Part 5: Optimal Algorithm - Using Cached Metadata

### Staleness Detection (O(1) per schema)

```rust
fn is_inheritance_stale(&self, schema_id: SchemaId, raw: &RawSchema) -> Result<bool> {
    // Load cached metadata
    let cached = self.repo.get_inheritance_metadata(schema_id)?;
    let Some(cached) = cached else {
        return Ok(true);  // No cache: must rebuild
    };

    // Check if parent changed (name comparison)
    let current_parent_name = raw.extends.as_ref();
    let cached_parent = cached.parent_id
        .map(|id| self.name_to_id.get_by_id(&id))
        .transpose()?;

    if current_parent_name != cached_parent {
        return Ok(true);  // Parent changed
    }

    // Check if excludes changed
    if raw.excludes != cached.excludes {
        return Ok(true);  // Excludes changed
    }

    // Check if parent's ancestors changed (transitive staleness)
    if let Some(parent_id) = cached.parent_id {
        let parent_metadata = self.repo.get_inheritance_metadata(parent_id)?
            .ok_or_else(|| SchemaError::MissingMetadata)?;

        let expected_hash = Self::compute_ancestors_hash(
            Some(parent_id),
            &parent_metadata.ancestors_hash,
        );

        if expected_hash != cached.ancestors_hash {
            return Ok(true);  // Parent's ancestors changed
        }
    }

    Ok(false)  // Fresh!
}
```

### Fast Path: Resolve From Cache (O(n × p))

```rust
fn resolve_from_cache(&self, schemas: Vec<(SchemaId, RawSchema)>) -> Result<Vec<Schema>> {
    let mut result = Vec::new();

    for (schema_id, raw) in schemas {
        // Load inheritance metadata (O(1))
        let metadata = self.repo.get_inheritance_metadata(schema_id)?
            .ok_or_else(|| SchemaError::MissingMetadata)?;

        // Collect ancestor properties (use cached ancestor IDs)
        let mut ancestor_props = Vec::new();
        for &ancestor_id in metadata.ancestors.iter().rev() {  // Reverse for merge order
            let ancestor = self.repo.find_schema_by_id(ancestor_id)?
                .ok_or_else(|| SchemaError::NotFound)?;
            ancestor_props.extend(ancestor.properties().iter().cloned());
        }

        // Expand own properties (RefExpander)
        let expanded = RefExpander::new(&self.bank).expand_single(raw.clone())?;

        // Merge (using cached excludes)
        let merged = Resolver::merge_properties(
            &ancestor_props,
            &expanded.properties,
            &metadata.excludes,
        );

        // Build schema (using cached parent_id and children)
        let schema = Schema::new(
            schema_id,
            raw.name,
            metadata.parent_id,
            metadata.children.clone(),
            merged,
        );

        result.push(schema);
    }

    Ok(result)
}
```

**Cost**: O(n × p) - just property merging, no tree building!

### Slow Path: Rebuild Metadata (O(n log n))

```rust
fn rebuild_inheritance_metadata(&self, changed_schemas: Vec<(SchemaId, RawSchema)>)
    -> Result<()>
{
    // Load all schemas (changed + fresh)
    let all_schemas = self.load_all_schemas_with_changes(changed_schemas)?;

    // Build SchemaTree (6 phases)
    let expanded = RefExpander::new(&self.bank).expand_all(all_schemas.clone())?;
    let tree = Extender::build(expanded, &HashMap::new())?;

    // Extract metadata from tree
    for &schema_id in tree.nodes() {
        let node = tree.get(schema_id)?;

        // Build ancestor chain
        let ancestors = self.build_ancestor_chain_from_tree(schema_id, &tree)?;

        // Compute hash
        let parent_metadata = node.parent_id
            .and_then(|pid| self.repo.get_inheritance_metadata(pid).ok().flatten());
        let ancestors_hash = Self::compute_ancestors_hash(
            node.parent_id,
            parent_metadata.map(|m| m.ancestors_hash).unwrap_or(0),
        );

        // Save metadata
        let metadata = SchemaInheritanceMetadata {
            schema_id,
            parent_id: node.parent_id,
            ancestors,
            children: node.children.clone(),
            depth: node.depth.into(),
            excludes: node.excludes.clone(),
            ancestors_hash,
        };

        self.repo.save_inheritance_metadata(schema_id, &metadata)?;
    }

    // Save global graph metadata
    let graph_metadata = SchemaGraphMetadata {
        resolution_order: tree.order().to_vec(),
        roots: tree.roots().to_vec(),
        graph_hash: Self::compute_graph_hash(&tree),
        computed_at: SystemTime::now(),
    };
    self.repo.save_graph_metadata(&graph_metadata)?;

    Ok(())
}
```

---

## Part 6: Critical Analysis - Why This Is Optimal

### What We Eliminated

1. ❌ **Redundant tables**: Only 2 tables (not 3+)
   - `schema_inheritance` (per-schema)
   - `schema_graph` (singleton)

2. ❌ **Redundant data**: Each field stored once
   - No duplication of `excludes` across tables
   - No duplication of `resolved_at` (use `computed_at` in graph metadata)
   - `ancestors` stored as `Vec<SchemaId>` (not names - smaller, no lookup)

3. ❌ **Unnecessary multimap**: Use simple Vec in metadata
   - Redb multimap forces multiple deserializations for "get all children"
   - Our design: Single deserialize of `SchemaInheritanceMetadata` gives children

4. ❌ **Name lookups during resolution**: Store IDs, not names
   - `ancestors: Vec<SchemaId>` not `Vec<SchemaName>`
   - Saves HashMap lookup per ancestor

### What We Optimized

1. ✅ **O(1) staleness detection** via hashes
2. ✅ **O(1) ancestor lookup** via precomputed chain
3. ✅ **O(1) children lookup** via cached list
4. ✅ **Skip entire Extender** when inheritance unchanged (40ms → 0ms)
5. ✅ **Single table scan** to get all metadata (not N queries)

### Performance Model

**Common case: File changed, inheritance unchanged** (95% of loads)
- Current: 100ms (full pipeline)
- Optimized: 30ms (skip Extender, just Resolver)
- **Speedup: 3.3x**

**Rare case: Inheritance changed** (5% of loads)
- Current: 100ms
- Optimized: 110ms (rebuild metadata overhead)
- **Slowdown: 10%** (acceptable trade-off)

**Amortized**: 0.95 × 30ms + 0.05 × 110ms = **34ms avg** (vs 100ms)
- **Overall speedup: 2.9x**

---

## Part 7: Rust Best Practices Applied

### 1. Zero-Copy Where Possible

```rust
// Don't copy ancestors during merge
fn resolve_with_cached_metadata(&self, metadata: &SchemaInheritanceMetadata) {
    for &ancestor_id in metadata.ancestors.iter().rev() {
        // Use &SchemaId, not clone
    }
}
```

### 2. Minimize Allocations

```rust
// ancestors: Vec<SchemaId> not Vec<SchemaName>
// SchemaId = 16 bytes (UUID)
// SchemaName = 24+ bytes (Box<str> + heap allocation)
// For 3 ancestors: 48 bytes vs 72+ bytes = 33% savings
```

### 3. Leverage rkyv Zero-Copy Deserialization

```rust
#[derive(Archive, Serialize, Deserialize)]
pub struct SchemaInheritanceMetadata {
    pub ancestors: Vec<SchemaId>,  // rkyv can deserialize as &[SchemaId]
}

// Access without full deserialize
fn get_ancestors_count(&self, id: SchemaId) -> Result<usize> {
    self.repo.with_inheritance_metadata(id, |archived| {
        archived.ancestors.len()  // No deserialize, just read length!
    })
}
```

### 4. Smart Hash Function

```rust
fn compute_ancestors_hash(parent_id: Option<SchemaId>, parent_hash: u64) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    parent_id.hash(&mut hasher);
    parent_hash.hash(&mut hasher);  // Recursive hash for transitive staleness
    hasher.finish()
}
```

**Why this works**: If parent's hash changes, child's hash changes automatically.

---

## Part 8: Implementation Comparison

### Current Design (views/inheritance.rs)

```rust
// TWO tables, DUPLICATED data
pub struct ParentSchemaView {
    pub parent_id: Option<SchemaId>,
    pub excludes: Vec<Box<str>>,       // DUPLICATE 1
    pub resolved_at: SystemTime,       // DUPLICATE 1
}

pub struct ChildSchemaView {
    pub child_id: SchemaId,
    pub excludes: Vec<Box<str>>,       // DUPLICATE 2
    pub resolved_at: SystemTime,       // DUPLICATE 2
}
```

**Issues**:
- ❌ Data duplicated across 2 tables
- ❌ Multimap for children (expensive iteration)
- ❌ No ancestor chain (requires O(depth) queries)
- ❌ No depth cache (requires BFS each time)
- ❌ No graph metadata (requires topological sort each time)

### Proposed Design (optimal)

```rust
// ONE table per-schema, ZERO duplication
pub struct SchemaInheritanceMetadata {
    pub schema_id: SchemaId,
    pub parent_id: Option<SchemaId>,
    pub ancestors: Vec<SchemaId>,      // Precomputed chain
    pub children: Vec<SchemaId>,       // Precomputed list
    pub depth: u8,                     // Precomputed depth
    pub excludes: Vec<Box<str>>,       // Stored once
    pub ancestors_hash: u64,           // For staleness
}

// ONE singleton table for graph
pub struct SchemaGraphMetadata {
    pub resolution_order: Vec<SchemaId>,  // Precomputed topological order
    pub roots: Vec<SchemaId>,
    pub graph_hash: u64,
    pub computed_at: SystemTime,
}
```

**Benefits**:
- ✅ Single source of truth
- ✅ All inheritance data in one lookup
- ✅ O(1) staleness detection
- ✅ Skip entire Extender when fresh
- ✅ Smaller storage (no duplication)

---

## Part 9: Revised Views Structure

### File: `schema/views/inheritance.rs` (complete rewrite)

```rust
//! Inheritance metadata views for schema caching.
//!
//! These types cache precomputed inheritance graph data to avoid
//! rebuilding SchemaTree on every load.

use std::time::SystemTime;
use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};
use crate::{db::DbError, schema::aggregate::SchemaId};

/// Per-schema inheritance metadata.
///
/// **Storage pattern:**
/// - Table: `schema_inheritance` (regular table)
/// - Key: SchemaId (as UUID string)
/// - Value: `SchemaInheritanceMetadata` (rkyv-serialized bytes)
///
/// **Purpose**: Cache inheritance relationships to skip SchemaTree rebuilding.
///
/// **Staleness**: Rebuild when:
/// - Schema changes `extends` field
/// - Schema changes `excludes` field
/// - Parent schema's ancestors change (detected via `ancestors_hash`)
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaInheritanceMetadata {
    /// Schema ID (redundant for validation)
    pub schema_id: SchemaId,

    /// Immediate parent ID, or None for root schemas
    pub parent_id: Option<SchemaId>,

    /// Full ancestor chain: [parent, grandparent, great-grandparent, ...]
    /// Ordered closest-first for efficient property merging (merge in reverse).
    /// Uses IDs (not names) for:
    /// - Smaller storage (16 bytes vs 24+ bytes per ancestor)
    /// - No name lookup needed during resolution
    pub ancestors: Vec<SchemaId>,

    /// Direct children IDs (cached for descendant queries)
    pub children: Vec<SchemaId>,

    /// Inheritance depth: 1 = root, 2 = one parent, etc.
    /// Max value enforced during tree building (typically 10).
    pub depth: u8,

    /// Property names this schema excludes from ancestors.
    /// Stored here (not in Schema aggregate) because needed for resolution.
    pub excludes: Vec<Box<str>>,

    /// Hash of (parent_id || parent.ancestors_hash) for O(1) staleness detection.
    /// Recursively incorporates ancestor changes for transitive staleness.
    pub ancestors_hash: u64,
}

/// Global inheritance graph metadata (singleton).
///
/// **Storage pattern:**
/// - Table: `schema_graph` (regular table, singleton)
/// - Key: Constant "graph"
/// - Value: `SchemaGraphMetadata` (rkyv-serialized bytes)
///
/// **Purpose**: Cache topological order and roots to avoid Kahn's algorithm.
///
/// **Staleness**: Rebuild when ANY schema changes `extends` field.
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaGraphMetadata {
    /// Topological ordering: roots first, leaves last.
    /// Order for resolution: parent must be processed before child.
    pub resolution_order: Vec<SchemaId>,

    /// Root schema IDs (parent_id = None)
    pub roots: Vec<SchemaId>,

    /// Hash of entire graph structure for O(1) staleness detection.
    /// Computed from all (schema_id, parent_id) pairs.
    pub graph_hash: u64,

    /// When this graph metadata was computed
    #[rkyv(with = AsUnixTime)]
    pub computed_at: SystemTime,
}

impl SchemaInheritanceMetadata {
    /// Serialize to bytes for storage.
    pub fn to_bytes(&self) -> Result<Vec<u8>, DbError> {
        rkyv::to_bytes(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| DbError::Serialization(e.to_string()))
    }

    /// Check if this metadata is stale relative to a new parent.
    ///
    /// Returns true if parent changed or parent's ancestors changed.
    pub fn is_stale(&self, current_parent_id: Option<SchemaId>, parent_metadata: Option<&Self>) -> bool {
        // Parent changed
        if self.parent_id != current_parent_id {
            return true;
        }

        // Parent's ancestors changed (transitive staleness)
        if let (Some(parent_id), Some(parent_meta)) = (current_parent_id, parent_metadata) {
            let expected_hash = Self::compute_hash(Some(parent_id), parent_meta.ancestors_hash);
            if expected_hash != self.ancestors_hash {
                return true;
            }
        }

        false
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

impl SchemaGraphMetadata {
    /// Serialize to bytes for storage.
    pub fn to_bytes(&self) -> Result<Vec<u8>, DbError> {
        rkyv::to_bytes(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|e| DbError::Serialization(e.to_string()))
    }
}
```

---

## Part 10: Migration Plan

### Phase 1: Add New Tables (2-3 hours)

1. Add table definitions to `schema/mod.rs`
2. Add `SchemaInheritanceMetadata` and `SchemaGraphMetadata` to `views/inheritance.rs`
3. Add repository methods for read/write

### Phase 2: Build Metadata During Resolution (3-4 hours)

1. Extract metadata from `SchemaTree` after `Extender::build()`
2. Save to `schema_inheritance` table
3. Build and save `SchemaGraphMetadata`

### Phase 3: Use Cached Metadata (4-5 hours)

1. Add staleness detection for inheritance
2. Implement fast path: `resolve_from_cache()`
3. Fallback to slow path when stale

### Phase 4: Delete Old Tables (1 hour)

1. Remove `SCHEMA_PARENT` and `SCHEMA_CHILDREN` tables
2. Delete old `ParentSchemaView` and `ChildSchemaView` types
3. Update tests

**Total**: 10-13 hours for full implementation

---

## Conclusion

**Optimal design**:
- **2 tables** (not 3): `schema_inheritance` + `schema_graph`
- **Zero duplication**: Each field stored once
- **IDs not names**: 33% storage savings, no lookups
- **Recursive hashing**: O(1) transitive staleness detection
- **Precomputed everything**: ancestors, children, depth, order

**Performance gain**: 2.9x average speedup (3.3x common case)

**Rust best practices**:
- Zero-copy via rkyv
- Minimal allocations (Vec<SchemaId> not Vec<SchemaName>)
- Single source of truth
- Leverages redb efficiently (no multimap overhead)

**Key insight**: Don't just cache parent-child relationships - cache the ENTIRE GRAPH STRUCTURE because that's what's expensive to compute.
