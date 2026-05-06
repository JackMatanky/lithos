# Wave B Implementation Plan: P0 Performance Optimizations

**Status**: Planning (Read-only mode)
**Date**: 2026-04-29
**Prerequisites**: Wave A complete (delta centralization, clone reduction)

---

## Executive Summary

Wave B focuses on two P0 optimizations based on the schema processor roadmap:

1. **I-05: Batch Membership Optimization** - Replace O(n) `Vec::contains()` loops with O(1) lookups using `IndexSet`
2. **I-06: Graph Construction Deduplication** - Extract duplicate unit-graph building code into reusable helper

**Expected Impact**:
- **I-05**: 10-50% faster processing for large schemas (500+ files) during incremental updates
- **I-06**: Eliminates ~30 lines of duplicate code, improves maintainability

**Complexity**: Low (I-05: add dependency + type change; I-06: extract method)

---

## I-05: Batch Membership Optimization

### Problem Statement

Current code uses `Vec<SchemaId>` with `.contains()` in tight loops, resulting in O(n) membership checks:

**Hotspot 1: Line 2003** (analyze_properties stage)
```rust
for id in topo_order {
    if affected.contains(&id) && !rebuild_ids.contains(&id) {  // O(n) + O(n)
        rebuild_ids.push(id);
    }
}
```

**Hotspot 2: Lines 2243-2248** (refresh_metadata stage)
```rust
let mut fetch_ids = refresh_ids.clone();
for id in &stale_timestamp_ids {
    if !fetch_ids.contains(id) {  // O(n) lookup
        fetch_ids.push(*id);
    }
}
```

**Hotspot 3: Lines 2272-2275** (refresh_metadata stage)
```rust
for id in update_ids {
    if !fetch_ids.contains(&id) {  // O(n) lookup
        fetch_ids.push(id);
    }
}
```

**Impact**: With 500 schemas in topological order, this creates 250,000 comparisons (500 × 500) per load.

### Research-Backed Solution

Use **`IndexSet<SchemaId>`** from the `indexmap` crate instead of `Vec<SchemaId>`:

**Why IndexSet over Vec + HashSet dual containers**:
- ✅ Single source of truth (no synchronization bugs)
- ✅ O(1) membership checks via hash table
- ✅ Deterministic insertion-order iteration (critical for tests/reproducibility)
- ✅ ~1.5x memory overhead vs Vec (better than 2x for dual containers)
- ✅ Widely adopted in production Rust (rust-analyzer, petgraph)

**Research Reference**: See Wave B research document section 1 for detailed analysis.

### Implementation Plan

#### Step 1: Add Dependency

**File**: `Cargo.toml` (workspace level)

```toml
[workspace.dependencies]
# ... existing deps ...
indexmap = "2.8"  # Ordered hash map/set with O(1) lookups
```

**File**: `lithos-core/Cargo.toml`

```toml
[dependencies]
# ... existing deps ...
indexmap.workspace = true
```

#### Step 2: Create Domain Type Wrapper (Optional but Recommended)

**File**: `lithos-core/src/schema/identifier.rs`

Add after `SchemaId` definition:

```rust
use indexmap::IndexSet;

/// Ordered set of schema IDs with fast membership testing.
///
/// Provides O(1) contains() checks while preserving insertion order
/// for deterministic iteration (critical for test stability).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaIdSet(IndexSet<SchemaId>);

impl SchemaIdSet {
    /// Creates an empty set.
    #[must_use]
    pub fn new() -> Self {
        Self(IndexSet::new())
    }

    /// Creates a set with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(IndexSet::with_capacity(capacity))
    }

    /// Inserts an ID into the set.
    ///
    /// Returns `true` if the ID was newly inserted, `false` if it was already present.
    pub fn insert(&mut self, id: SchemaId) -> bool {
        self.0.insert(id)
    }

    /// Checks if the set contains an ID (O(1) operation).
    #[must_use]
    pub fn contains(&self, id: &SchemaId) -> bool {
        self.0.contains(id)
    }

    /// Returns an iterator over IDs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &SchemaId> {
        self.0.iter()
    }

    /// Returns the number of IDs in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts into a `Vec` of IDs in insertion order.
    #[must_use]
    pub fn into_vec(self) -> Vec<SchemaId> {
        self.0.into_iter().collect()
    }
}

impl FromIterator<SchemaId> for SchemaIdSet {
    fn from_iter<T: IntoIterator<Item = SchemaId>>(iter: T) -> Self {
        Self(IndexSet::from_iter(iter))
    }
}

impl From<Vec<SchemaId>> for SchemaIdSet {
    fn from(vec: Vec<SchemaId>) -> Self {
        vec.into_iter().collect()
    }
}
```

**Alternative (Simpler)**: Skip the wrapper and use `IndexSet<SchemaId>` directly. Trade-off:
- ✅ Less code, faster implementation
- ❌ Less encapsulation, harder to add domain-specific methods later

#### Step 3: Update schema_processor.rs Hotspots

**Location 1: Line 2003** (analyze_properties stage)

```rust
// Before:
let mut rebuild_ids: Vec<SchemaId> = Vec::new();
// ... later ...
for id in topo_order {
    if affected.contains(&id) && !rebuild_ids.contains(&id) {
        rebuild_ids.push(id);
    }
}

// After:
use indexmap::IndexSet;
let mut rebuild_ids: IndexSet<SchemaId> = IndexSet::new();
// ... later ...
for id in topo_order {
    if affected.contains(&id) {
        rebuild_ids.insert(id);  // O(1) insertion, automatically deduplicates
    }
}
```

**Location 2: Lines 2243-2248** (refresh_metadata stage)

```rust
// Before:
let mut fetch_ids = refresh_ids.clone();
for id in &stale_timestamp_ids {
    if !fetch_ids.contains(id) {
        fetch_ids.push(*id);
    }
}

// After:
let mut fetch_ids: IndexSet<SchemaId> = refresh_ids.iter().copied().collect();
for id in &stale_timestamp_ids {
    fetch_ids.insert(*id);  // O(1), auto-deduplicates
}
```

**Location 3: Lines 2272-2275** (refresh_metadata stage)

```rust
// Before:
for id in update_ids {
    if !fetch_ids.contains(&id) {
        fetch_ids.push(id);
    }
}

// After:
for id in update_ids {
    fetch_ids.insert(id);  // O(1), auto-deduplicates
}
```

#### Step 4: Update Type Signatures

**File**: `lithos-core/src/schema/schema_processor.rs`

Update struct field types:

```rust
pub(crate) struct Analyzed {
    pub(crate) graph: ProcessingGraph<AnalysisBranch>,
    pub(crate) refresh_ids: IndexSet<SchemaId>,       // Changed from Vec
    pub(crate) stale_timestamp_ids: IndexSet<SchemaId>,  // Changed from Vec
    pub(crate) rebuild_ids: IndexSet<SchemaId>,       // Changed from Vec
    pub(crate) deleted_ids: Vec<SchemaId>,            // Keep as Vec (no contains() usage)
}
```

**Important**: Check all downstream usages:
- If code iterates: `.iter()` works the same
- If code converts to `Vec`: Use `.into_iter().collect()` or `into_vec()`
- If code clones: `.clone()` still works (IndexSet is Clone)

#### Step 5: Testing Strategy

**Unit Tests** (add to `schema_processor.rs` test module):

```rust
#[cfg(test)]
mod batch_membership_tests {
    use super::*;
    use indexmap::IndexSet;

    #[test]
    fn test_indexset_preserves_insertion_order() {
        let mut set = IndexSet::new();
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();
        let id3 = SchemaId::new();

        set.insert(id1);
        set.insert(id2);
        set.insert(id3);

        let collected: Vec<_> = set.iter().copied().collect();
        assert_eq!(collected, vec![id1, id2, id3]);
    }

    #[test]
    fn test_indexset_deduplicates() {
        let mut set = IndexSet::new();
        let id = SchemaId::new();

        assert!(set.insert(id));  // First insert returns true
        assert!(!set.insert(id)); // Second insert returns false
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_indexset_contains_is_fast() {
        let mut set = IndexSet::with_capacity(1000);
        let ids: Vec<_> = (0..1000).map(|_| SchemaId::new()).collect();

        for id in &ids {
            set.insert(*id);
        }

        // O(1) lookup (hash-based)
        assert!(set.contains(&ids[500]));
        assert!(!set.contains(&SchemaId::new()));
    }
}
```

**Integration Tests**: No changes needed - existing tests should pass with IndexSet (same iteration order).

**Performance Benchmark** (optional, for validation):

```rust
// File: lithos-core/benches/batch_membership.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lithos_core::schema::identifier::SchemaId;
use indexmap::IndexSet;

fn bench_vec_contains(c: &mut Criterion) {
    let ids: Vec<SchemaId> = (0..1000).map(|_| SchemaId::new()).collect();
    let search_ids: Vec<SchemaId> = (0..100).map(|_| SchemaId::new()).collect();

    c.bench_function("vec_contains_1000", |b| {
        b.iter(|| {
            for id in &search_ids {
                black_box(ids.contains(id));
            }
        });
    });
}

fn bench_indexset_contains(c: &mut Criterion) {
    let ids: IndexSet<SchemaId> = (0..1000).map(|_| SchemaId::new()).collect();
    let search_ids: Vec<SchemaId> = (0..100).map(|_| SchemaId::new()).collect();

    c.bench_function("indexset_contains_1000", |b| {
        b.iter(|| {
            for id in &search_ids {
                black_box(ids.contains(id));
            }
        });
    });
}

criterion_group!(benches, bench_vec_contains, bench_indexset_contains);
criterion_main!(benches);
```

### Rollout Strategy

**Phase 1** (Low Risk): Replace only the 3 identified hotspots
- Lines 2003, 2243-2248, 2272-2275 in `schema_processor.rs`
- Minimal API surface change
- Run full test suite to verify behavior preservation

**Phase 2** (Optional): Audit entire codebase for similar patterns
- Search for `Vec<SchemaId>` with `.contains()` usage
- Consider `affected` HashSet usage (already optimal)

**Rollback Plan**: If tests fail, revert type changes (IndexSet → Vec) and add `.to_vec()` calls.

### Definition of Done

- [ ] `indexmap` dependency added to workspace and lithos-core
- [ ] `SchemaIdSet` wrapper created (or decision to use IndexSet directly)
- [ ] 3 hotspots updated with IndexSet
- [ ] All 868 tests passing (835 unit + 33 integration)
- [ ] Zero clippy warnings
- [ ] Code formatted
- [ ] Optional: Benchmark shows >10% improvement for 1000-element contains() loop

---

## I-06: Graph Construction Deduplication

### Problem Statement

Duplicate code exists at two locations in `schema_processor.rs`:

**Location 1: Lines 2778-2795** (build stage completion)
```rust
let mut persist_builder = SchemaGraphBuilder::<()>::new();
for (id, _node) in graph.graph().iter() {
    persist_builder.add_node(id, ());
}
for (child_id, _) in graph.graph().iter() {
    for &parent_id in graph.graph().parents_of(child_id) {
        persist_builder.add_parent(child_id, parent_id);
    }
}
let inheritance_graph = InheritanceGraph::try_from(persist_builder.build())
    .map_err(|e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)))?;
repository.save_topological_graph(&inheritance_graph).map_err(|e| {
    let repo_err: SchemaRepositoryError = e.into();
    SchemaLoaderError::Repository(repo_err)
})?;
```

**Location 2: Lines 2834-2852** (complete stage)
```rust
// Build unit-payload graph for persistence (structure only)
let mut persist_builder = SchemaGraphBuilder::<()>::new();
for (id, _node) in graph.graph().iter() {
    persist_builder.add_node(id, ());
}
for (child_id, _) in graph.graph().iter() {
    for &parent_id in graph.graph().parents_of(child_id) {
        persist_builder.add_parent(child_id, parent_id);
    }
}

let inheritance_graph = InheritanceGraph::try_from(persist_builder.build())
    .map_err(|e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)))?;

repository.save_topological_graph(&inheritance_graph).map_err(|e| {
    let repo_err: SchemaRepositoryError = e.into();
    SchemaLoaderError::Repository(repo_err)
})?;
```

**Impact**: ~30 lines of duplicate code, risk of drift during maintenance.

### Research-Backed Solution

Extract as **method on `ProcessingGraph<T>`** (not free function or trait):

**Why method over free function**:
- ✅ Idiomatic Rust (methods on types, Rust API Guidelines)
- ✅ Easy to discover (in type's impl block)
- ✅ Clear ownership (borrows source graph)
- ✅ Type-safe signature shows payload erasure: `ProcessingGraph<T> -> InheritanceGraph<()>`

**Why borrow (`&self`) not consume (`self`)**:
- ✅ Allows reuse of source graph after conversion
- ✅ Graph persistence is "observation" not "transformation"

**Research Reference**: See Wave B research document section 3 for detailed analysis.

### Implementation Plan

#### Step 1: Add Method to ProcessingGraph<T>

**File**: `lithos-core/src/schema/inheritance.rs`

Add to `impl<T> ProcessingGraph<T>` block (after existing methods):

```rust
impl<T> ProcessingGraph<T> {
    // ... existing methods (graph(), node_ids_sorted(), etc.) ...

    /// Converts this processing graph to a unit-payload inheritance graph
    /// for persistence.
    ///
    /// This strips all node payloads and retains only the graph structure
    /// (nodes and parent-child relationships). The resulting graph can be
    /// serialized and stored in the repository.
    ///
    /// # Errors
    ///
    /// Returns `SchemaInheritanceError::CycleDetected` if the graph contains
    /// a cycle. This should never happen if the processing graph was
    /// validated during construction via `InheritanceGraph::try_from`.
    ///
    /// # Example
    ///
    /// ```
    /// # use lithos_core::schema::{
    /// #     inheritance::{ProcessingGraph, SchemaGraphBuilder, InheritanceGraph},
    /// #     identifier::SchemaId,
    /// # };
    /// let mut builder = SchemaGraphBuilder::new();
    /// let id = SchemaId::new();
    /// builder.add_node(id, "some payload");
    /// let processing = builder.build();
    ///
    /// let inheritance: InheritanceGraph<()> = processing.to_inheritance_graph()?;
    /// assert_eq!(inheritance.node_ids_sorted(), &[id]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub(crate) fn to_inheritance_graph(
        &self,
    ) -> Result<InheritanceGraph<()>, crate::schema::error::SchemaInheritanceError>
    {
        let mut builder = SchemaGraphBuilder::<()>::new();

        // Copy nodes with unit payload (structure only)
        for (id, _node) in self.graph().iter() {
            builder.add_node(id, ());
        }

        // Copy edges (parent-child relationships)
        for (child_id, _) in self.graph().iter() {
            for &parent_id in self.graph().parents_of(child_id) {
                builder.add_parent(child_id, parent_id);
            }
        }

        InheritanceGraph::try_from(builder.build())
    }
}
```

**Design Notes**:
- **Visibility**: `pub(crate)` - used within `schema` module, not public API
- **Naming**: `to_inheritance_graph()` follows Rust naming taxonomy (`to_*` for expensive conversions)
- **Error handling**: Returns `Result` for fallible conversion (cycle detection)
- **Doc comments**: Includes `# Errors` section per Lithos coding standards

#### Step 2: Update Call Sites in schema_processor.rs

**Location 1: Lines 2778-2795 replacement**

```rust
// Before (17 lines):
let mut persist_builder = SchemaGraphBuilder::<()>::new();
for (id, _node) in graph.graph().iter() {
    persist_builder.add_node(id, ());
}
for (child_id, _) in graph.graph().iter() {
    for &parent_id in graph.graph().parents_of(child_id) {
        persist_builder.add_parent(child_id, parent_id);
    }
}
let inheritance_graph =
    InheritanceGraph::try_from(persist_builder.build()).map_err(
        |e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)),
    )?;
repository.save_topological_graph(&inheritance_graph).map_err(|e| {
    let repo_err: SchemaRepositoryError = e.into();
    SchemaLoaderError::Repository(repo_err)
})?;

// After (4 lines):
let inheritance_graph = graph.to_inheritance_graph()
    .map_err(|e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)))?;
repository.save_topological_graph(&inheritance_graph)
    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
```

**Location 2: Lines 2834-2852 replacement**

```rust
// Before (19 lines with comment):
// Build unit-payload graph for persistence (structure only)
let mut persist_builder = SchemaGraphBuilder::<()>::new();
for (id, _node) in graph.graph().iter() {
    persist_builder.add_node(id, ());
}
for (child_id, _) in graph.graph().iter() {
    for &parent_id in graph.graph().parents_of(child_id) {
        persist_builder.add_parent(child_id, parent_id);
    }
}

let inheritance_graph =
    InheritanceGraph::try_from(persist_builder.build()).map_err(
        |e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)),
    )?;

repository.save_topological_graph(&inheritance_graph).map_err(|e| {
    let repo_err: SchemaRepositoryError = e.into();
    SchemaLoaderError::Repository(repo_err)
})?;

// After (3 lines):
let inheritance_graph = graph.to_inheritance_graph()
    .map_err(|e| SchemaLoaderError::Resolution(SchemaError::Inheritance(e)))?;
repository.save_topological_graph(&inheritance_graph)
    .map_err(|e| SchemaLoaderError::Repository(e.into()))?;
```

**Net Impact**: Removes 30 lines of duplicate code, replaces with 2× single method call.

#### Step 3: Testing Strategy

**Unit Tests** (add to `inheritance.rs` test module):

```rust
#[cfg(test)]
mod processing_graph_tests {
    use super::*;

    #[test]
    fn test_to_inheritance_graph_preserves_structure() {
        let mut builder = SchemaGraphBuilder::new();
        let root = SchemaId::new();
        let child = SchemaId::new();

        builder.add_node(root, "root payload");
        builder.add_node(child, "child payload");
        builder.add_parent(child, root);

        let processing = builder.build();
        let inheritance = processing.to_inheritance_graph()
            .expect("valid DAG");

        // Structure preserved
        assert_eq!(inheritance.node_ids_sorted().len(), 2);
        assert!(inheritance.node_ids_sorted().contains(&root));
        assert!(inheritance.node_ids_sorted().contains(&child));
        assert_eq!(inheritance.roots(), &[root]);
        assert_eq!(inheritance.topo_order(), &[root, child]);
    }

    #[test]
    fn test_to_inheritance_graph_strips_payloads() {
        let mut builder = SchemaGraphBuilder::new();
        let id = SchemaId::new();
        builder.add_node(id, vec![1, 2, 3, 4, 5]);  // Large payload

        let processing = builder.build();
        let inheritance = processing.to_inheritance_graph()
            .expect("valid DAG");

        // Payload is unit type (structure only)
        // This is verified by type system: InheritanceGraph<()>
        assert_eq!(inheritance.node_ids_sorted(), &[id]);
    }

    #[test]
    fn test_to_inheritance_graph_cycle_detection() {
        // Note: Building a cycle requires bypassing SchemaGraphBuilder validation
        // This test documents expected behavior but may not be easily testable
        // given current APIs. Consider if cycle detection is already covered
        // by InheritanceGraph::try_from tests.
    }

    #[test]
    fn test_to_inheritance_graph_allows_reuse() {
        let mut builder = SchemaGraphBuilder::new();
        let id = SchemaId::new();
        builder.add_node(id, "payload");

        let processing = builder.build();

        // First conversion
        let _inheritance1 = processing.to_inheritance_graph()
            .expect("valid DAG");

        // Second conversion (processing graph still usable)
        let _inheritance2 = processing.to_inheritance_graph()
            .expect("valid DAG");

        // Both conversions succeed (borrow, not consume)
    }
}
```

**Integration Tests**: No changes needed - existing tests cover end-to-end graph persistence.

**Regression Check**: Verify call sites compile and pass existing integration tests:
- `lithos-core/tests/schema_loader.rs` - incremental loading tests
- All graph persistence code paths exercised

#### Step 4: Documentation

Update `ProcessingGraph<T>` module-level docs in `inheritance.rs`:

```rust
//! # Usage
//!
//! ```rust
//! use lithos_core::schema::{
//!     identifier::SchemaId,
//!     inheritance::{InheritanceGraph, SchemaGraphBuilder},
//! };
//!
//! // Build processing graph with payloads
//! let mut builder = SchemaGraphBuilder::new();
//! builder.add_node(root_id, "payload");
//! let processing = builder.build();
//!
//! // Convert to persistence format (strips payloads)
//! let inheritance = processing.to_inheritance_graph()?;
//! repository.save_topological_graph(&inheritance)?;
//! ```
```

### Definition of Done

- [ ] `to_inheritance_graph()` method added to `ProcessingGraph<T>`
- [ ] Both call sites updated (lines 2778-2795, 2834-2852)
- [ ] Unit tests added to `inheritance.rs`
- [ ] All 868 tests passing (835 unit + 33 integration)
- [ ] Zero clippy warnings
- [ ] Code formatted
- [ ] Module docs updated with usage example

---

## Wave B Sequencing

### Recommended Order

**Option A (Parallel - Recommended)**:
- Both I-05 and I-06 are independent
- Can be implemented in parallel or any order
- Low risk of conflicts

**Option B (Sequential - Conservative)**:
1. I-06 first (simpler, pure refactor, no dependency changes)
2. I-05 second (adds dependency, more test validation needed)

### Estimated Effort

| Task | Complexity | Time Estimate | Risk |
|------|------------|---------------|------|
| I-05: IndexSet migration | Low-Medium | 1-2 hours | Low (well-tested pattern) |
| I-06: Extract helper | Low | 30-60 min | Very Low (pure refactor) |
| **Total Wave B** | Low | 2-3 hours | Low |

### Quality Gates

All must pass before marking Wave B complete:

- [ ] `mise run verify` (fmt + lint + tests + adr:validate) 100% green
- [ ] All 868 tests passing (no regressions)
- [ ] Zero clippy warnings (including new code)
- [ ] Code formatted (rustfmt)
- [ ] ADR validation passed (if architectural decision needed)
- [ ] Performance validated (optional benchmark for I-05)
- [ ] Documentation updated (method docs, module docs)

---

## Open Questions for User

Before proceeding with implementation, please clarify:

### 1. IndexSet Wrapper vs Direct Usage

**Option A**: Create `SchemaIdSet` newtype wrapper (more encapsulation, cleaner API)
**Option B**: Use `IndexSet<SchemaId>` directly (faster, less code)

**Recommendation**: Option B (direct usage) - simpler for now, can wrap later if needed.

### 2. Performance Benchmark

Should we create a benchmark for I-05 to validate the improvement?

**Pros**: Quantifies benefit, catches regressions
**Cons**: Adds ~30 min to implementation, may not be needed for obvious win

**Recommendation**: Skip benchmark initially (add later if performance regression suspected).

### 3. Broader Audit

Should we audit the entire codebase for similar patterns (Vec + contains) beyond the 3 identified hotspots?

**Recommendation**: Implement the 3 hotspots first, then decide based on results.

### 4. Wave B Naming

The roadmap labels these as I-05 (P1) and I-06 (P1). Should we rename to match the actual priority we're giving them (P0 in Wave B)?

**Recommendation**: Keep issue numbers as-is for traceability, note priority elevation in commit messages.

---

## Next Steps

Once user confirms preferences:

1. **Disable read-only mode** (exit planning phase)
2. **Implement I-06** (graph deduplication - simpler, no dependencies)
3. **Implement I-05** (IndexSet migration - requires dependency add)
4. **Run full quality gates** (`mise run verify`)
5. **Update roadmap** (mark Wave B complete)
6. **Commit with detailed messages** (reference this plan)

---

## References

- **Research Document**: See research agent output in this conversation
- **Roadmap**: `.opencode/plans/schema-processor-lean-modular-performance-roadmap.md`
- **Naming Taxonomy**: `docs/refs/rust/naming-taxonomy.md`
- **Rust Idioms**: `docs/refs/rust/idioms.md`
- **IndexSet Docs**: https://docs.rs/indexmap/latest/indexmap/set/struct.IndexSet.html
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
