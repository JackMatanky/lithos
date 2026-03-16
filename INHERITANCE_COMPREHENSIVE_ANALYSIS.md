# Comprehensive Inheritance Analysis: MetadataMenu vs Lithos

**Date**: 2026-03-16
**Purpose**: Single source of truth for inheritance implementation comparison
**Status**: In Progress - Research Phase

---

## Executive Summary

This document provides a thorough comparison between:

1. **MetadataMenu** (Obsidian plugin) - Reference implementation
2. **Lithos** (Rust schema system) - Current implementation

**Key Question**: Does Lithos already support the same inheritance features as MetadataMenu, or are there gaps?

---

## Part 1: MetadataMenu Implementation (Source Code Analysis)

### 1.1 Ancestor Chain Caching

**File**: `src/index/FieldIndex.ts` (lines 264-291)

```typescript
// GLOBAL CACHING: Built once during index, reused for all resolutions
fileClassesAncestors: Map<string, string[]>; // "physics" → ["course", "base"]
```

**Algorithm**:

1. **Initialize** - Read immediate parent from each fileClass's `extends` field
2. **Recursive expansion** - Walk up the chain via `getAncestorsRecursively()`
3. **Cycle detection** - Stop when `lastAncestorParent === fileClassName`

**Result**: Full ancestor chain stored globally, computed **once per index build**.

```typescript
private getFileClassesAncestors(): void {
    // 1. Init with immediate parent
    this.indexableFileClasses().forEach(f => {
        const parent = metadataCache.getFileCache(f)?.frontmatter?.extends
        if (parent && parentFileExists) {
            this.fileClassesAncestors.set(fileClassName, [parent])
        } else {
            this.fileClassesAncestors.set(fileClassName, [])
        }
    });

    // 2. Expand recursively
    [...this.fileClassesAncestors].forEach(([fileClassName, ancestors]) => {
        if (ancestors.length > 0) {
            this.getAncestorsRecursively(fileClassName)
        }
    })
}

private getAncestorsRecursively(fileClassName: string) {
    const ancestors = this.fileClassesAncestors.get(fileClassName)
    const lastAncestor = ancestors.last();
    const lastAncestorParent = this.fileClassesAncestors.get(lastAncestor)?.[0];

    // Cycle detection: stop if parent points back to self
    if (lastAncestorParent && lastAncestorParent !== fileClassName) {
        this.fileClassesAncestors.set(fileClassName, [...ancestors, lastAncestorParent]);
        this.getAncestorsRecursively(fileClassName);  // Recursive call
    }
}
```

### 1.2 Exclude Scope

**File**: `src/fileClass/fileClass.ts` (lines 144-183)

**Critical Finding**: Excludes apply to **ALL ancestors**, not just immediate parent.

```typescript
public getAttributes(): void {
    const ancestors = this.plugin.fieldIndex.fileClassesAncestors.get(this.name);
    let excludedFields = getExcludedFieldsFromFrontmatter(this.frontmatter.excludes);

    // 1. Collect attributes from THIS fileClass
    const ancestorsAttributes = new Map();
    ancestorsAttributes.set(this.name, getFileClassAttributes(this, excludedFields))

    // 2. Collect attributes from EACH ancestor (grandparent, great-grandparent, etc.)
    ancestors?.forEach(ancestorName => {
        const ancestor = new FileClass(this.plugin, ancestorName);
        ancestorsAttributes.set(ancestorName, getFileClassAttributes(ancestor, excludedFields))

        // 3. ACCUMULATE excludes from each ancestor
        const ancestorExcludes = ancestorFile.frontmatter?.excludes
        excludedFields.push(...getExcludedFieldsFromFrontmatter(ancestorExcludes));
    })

    // 4. Merge: child fields override parent fields (name-based deduplication)
    for (const [fileClassName, fileClassAttributes] of ancestorsAttributes) {
        this.attributes.push(...fileClassAttributes.filter(attr =>
            !this.attributes.map(_attr => _attr.name).includes(attr.name)
        ))
    }
}
```

**Exclude Behavior**:

- `excludedFields` passed to `getFileClassAttributes()` filters at EACH level
- Excludes accumulate: child's excludes + parent's excludes + grandparent's excludes
- **Result**: Can exclude fields from ANY ancestor (grandparent, great-grandparent, etc.)

### 1.3 Property Merge Strategy

**Merge Order** (from `getAttributes()`):

1. **Own fields first** - Added to `this.attributes`
2. **Parent fields** - Added if name not already present
3. **Grandparent fields** - Added if name not already present
4. **...**

**Override Semantics**:

- Name-based matching: `attr.name === _attr.name`
- Full replacement (not partial merge)
- Child wins in name conflicts

### 1.4 Cycle Detection

**Location**: `getAncestorsRecursively()`

**Method**: Simple name comparison

```typescript
if (lastAncestorParent !== fileClassName) {
  // Continue recursion
} else {
  // Stop: cycle detected
}
```

**Limitation**: Only detects **direct cycles** (A → B → A), not complex patterns.

---

## Part 2: Lithos Implementation (Rust Source Code Analysis)

### 2.1 Pipeline Overview

```text
File → Ingestor → RawSchema
         ↓
     RefExpander → RefExpandedSchema (property $refs resolved)
         ↓
     Extender → SchemaTree (topologically sorted, cycle-free)
         ↓
     Resolver → Schema (properties merged, overrides applied)
         ↓
     Repository → Persist to redb
```

### 2.2 Cycle Detection

**File**: `lithos-core/src/schema/extender.rs` (lines 393-435)

**Method**: **DFS + Kahn's algorithm** (more robust than MetadataMenu)

```rust
// Phase 3: DFS cycle detection
fn detect_cycles_dfs(&self, node_id: SchemaId, visited: &mut HashSet<SchemaId>,
                     rec_stack: &mut HashSet<SchemaId>) -> Result<(), SchemaError> {
    visited.insert(node_id);
    rec_stack.insert(node_id);

    if let Some(parent_id) = self.nodes.get(&node_id).and_then(|n| n.parent_id) {
        if rec_stack.contains(&parent_id) {
            // Cycle detected!
            return Err(SchemaError::CyclicInheritance(...));
        }
        if !visited.contains(&parent_id) {
            self.detect_cycles_dfs(parent_id, visited, rec_stack)?;
        }
    }

    rec_stack.remove(&node_id);
    Ok(())
}

// Phase 6: Kahn's topological sort (verifies acyclic + gives resolution order)
```

**Verdict**: Lithos is **MORE robust** than MetadataMenu.

### 2.3 Depth Tracking

**File**: `lithos-core/src/schema/extender.rs` (lines 474-507)

```rust
// Phase 5: BFS depth computation
pub struct NodeDepth(usize);  // 1-indexed: roots=1, children=2, etc.

fn compute_depths_bfs(&mut self) -> Result<(), SchemaError> {
    let mut queue: VecDeque<SchemaId> = VecDeque::new();

    // Start with roots (depth 1)
    for &root_id in &self.roots {
        self.nodes.get_mut(&root_id).unwrap().depth = NodeDepth::root();
        queue.push_back(root_id);
    }

    // BFS to children
    while let Some(parent_id) = queue.pop_front() {
        let parent_depth = self.nodes[&parent_id].depth;
        for &child_id in &self.nodes[&parent_id].children {
            self.nodes.get_mut(&child_id).unwrap().depth = parent_depth.increment();
            queue.push_back(child_id);
        }
    }
}
```

**Verdict**: Lithos **BETTER** than MetadataMenu (explicit depth tracking + max limit of 10).

### 2.4 Property Resolution

**File**: `lithos-core/src/schema/resolver.rs` (lines 73-151)

**CRITICAL ANALYSIS** - Does excludes work on ancestors?

```rust
pub fn resolve(tree: &SchemaTree, known_parents: &HashMap<SchemaId, Schema>)
    -> Result<Vec<Schema>, SchemaError> {

    let order = tree.nodes();  // Topologically sorted (parents before children)
    let mut resolved_cache: HashMap<SchemaId, Schema> = HashMap::new();

    for &id in order {
        let node = tree.get(id)?;

        // Get parent's RESOLVED properties
        let parent_props: &[Property] = if let Some(parent_id) = node.parent_id {
            resolved_cache.get(&parent_id)
                .or_else(|| known_parents.get(&parent_id))
                .map(Schema::properties)  // ← This is the FULLY RESOLVED parent
                .unwrap_or(&[])
        } else {
            &[]
        };

        // Merge parent props + own props, applying excludes
        let merged = Self::merge_properties(parent_props, &node.properties, &node.excludes);

        let schema = Schema::new(id, name, parent_id, children, merged);
        resolved_cache.insert(id, schema.clone());  // ← Cache for children
    }
}
```

**Key Insight**:

- `parent_props` comes from `resolved_cache.get(&parent_id).map(Schema::properties)`
- `resolved_cache` stores **already-resolved** schemas (processed earlier due to topological order)
- Therefore, `parent_props` already contains properties from **all ancestors** (grandparent, great-grandparent, etc.)

**Example Walk-Through**:

```
Hierarchy: base → course → physics
base properties: [id, created_at, internal_ref]
course properties: [title]
course excludes: []
physics properties: []
physics excludes: [internal_ref]
```

**Resolution Order** (topological):

1. **base** (depth 1, processed first):
   - `parent_props = []`
   - `merged = [id, created_at, internal_ref]`
   - `resolved_cache["base"] = Schema { properties: [id, created_at, internal_ref] }`

2. **course** (depth 2, processed second):
   - `parent_props = resolved_cache["base"].properties = [id, created_at, internal_ref]`
   - `node.properties = [title]`
   - `node.excludes = []`
   - `merged = [created_at, id, internal_ref, title]` (sorted merge)
   - `resolved_cache["course"] = Schema { properties: [created_at, id, internal_ref, title] }`

3. **physics** (depth 3, processed last):
   - `parent_props = resolved_cache["course"].properties = [created_at, id, internal_ref, title]`
   - `node.properties = []`
   - `node.excludes = ["internal_ref"]`
   - `merged = merge_properties([created_at, id, internal_ref, title], [], ["internal_ref"])`
   - **Result**: `[created_at, id, title]` ✅ **internal_ref excluded!**

### 2.5 Merge Properties Implementation

**File**: `lithos-core/src/schema/resolver.rs` (lines 167-232)

```rust
fn merge_properties(
    parent: &[Property],       // Already contains all inherited props
    own: &[Property],
    excludes: &[Box<str>],
) -> Vec<Property] {
    let mut result = Vec::new();
    let mut p_iter = parent.iter().peekable();
    let mut c_iter = own.iter().peekable();

    loop {
        match (p_iter.peek(), c_iter.peek()) {
            (Some(&p), Some(&c)) => {
                match p.name().cmp(c.name()) {
                    Ordering::Less => {
                        // Parent property: add if NOT excluded
                        Self::push_unless_excluded(&mut result, p, excludes);
                        p_iter.next();
                    }
                    Ordering::Greater => {
                        // Own property: always add
                        result.push(c.clone());
                        c_iter.next();
                    }
                    Ordering::Equal => {
                        // Same name: child overrides parent
                        result.push(c.clone());
                        p_iter.next();
                        c_iter.next();
                    }
                }
            }
            (Some(&p), None) => {
                // Remaining parent properties
                Self::push_unless_excluded(&mut result, p, excludes);
                p_iter.next();
            }
            (None, Some(&c)) => {
                // Remaining own properties
                result.push(c.clone());
                c_iter.next();
            }
            (None, None) => break,
        }
    }
    result
}

fn push_unless_excluded(result: &mut Vec<Property>, prop: &Property, excludes: &[Box<str>]) {
    if !Self::is_excluded(prop.name(), excludes) {
        result.push(prop.clone());
    }
}

fn is_excluded(name: &PropertyName, excludes: &[Box<str>]) -> bool {
    excludes.iter().any(|e| e.as_ref() == name.as_str())
}
```

**Analysis**:

- Two-pointer sorted merge (O(n + m) complexity)
- Excludes checked via `push_unless_excluded()` for **every** parent property
- Since `parent` contains all inherited properties, excludes apply to **all ancestors**

**Verdict**: ✅ **Lithos ALREADY supports excluding ancestor properties**

### 2.6 Ancestor Chain Storage

**Current State**:

- NO explicit `ancestors: Vec<SchemaName>` field
- NO global `fileClassesAncestors` map cache

**How It Works Instead**:

- Topological ordering ensures parents resolved before children
- Parent's resolved properties **implicitly** contain full ancestor chain
- Tradeoff: **Recompute each load** vs MetadataMenu's **cache once**

---

## Part 3: Comparison Matrix

| Feature                | MetadataMenu                         | Lithos                    | Winner?                     |
| ---------------------- | ------------------------------------ | ------------------------- | --------------------------- |
| **Exclude Scope**      | ✅ All ancestors                     | ✅ All ancestors          | **TIE**                     |
| **Exclude Method**     | Filter during collect                | Filter during merge       | Different but equivalent    |
| **Cycle Detection**    | ⚠️ Name comparison only              | ✅ DFS + Kahn's           | **Lithos**                  |
| **Depth Tracking**     | ❌ Not explicit                      | ✅ `NodeDepth` type       | **Lithos**                  |
| **Depth Limit**        | ❌ No limit                          | ✅ Max 10 levels          | **Lithos**                  |
| **Ancestor Caching**   | ✅ Global `fileClassesAncestors` map | ❌ Rebuild each load      | **MetadataMenu**            |
| **Merge Algorithm**    | Name-based filter loop               | Two-pointer sorted merge  | **Lithos** (more efficient) |
| **Override Semantics** | Full replacement                     | Full replacement          | **TIE**                     |
| **Property Sorting**   | Not guaranteed                       | Guaranteed (sorted merge) | **Lithos**                  |

---

## Part 4: Identified Gaps & Recommendations

### Gap 1: No Ancestor Caching (Performance)

**Problem**: Lithos rebuilds inheritance tree on every load via `Extender::build()`.

**MetadataMenu approach**: Cache `fileClassesAncestors` globally, invalidate on fileClass changes.

**Impact**:

- Small vaults (< 100 schemas): Negligible
- Large vaults (1000+ schemas): Potentially 100-500ms overhead per load

**Recommendation**:

- **Priority**: Medium
- **Effort**: 6-8 hours
- **Approach**: Add `ancestors: Vec<SchemaName>` to `RawSchemaView`, recompute only when parent chain changes

### Gap 2: ~~Exclude Scope Limited~~

**RESOLVED**: Lithos ALREADY supports excluding ancestor properties via topological resolution.

**Test Evidence**: Test `exclude_grandparent_property()` passes (see resolver.rs:550).

### Gap 3: Lenient Parent Resolution

**Problem**: Lithos errors when parent not found; MetadataMenu warns but continues.

**Impact**:

- Breaking change if parent file temporarily missing
- Blocks vault sync if parent committed after child

**Recommendation**:

- **Priority**: Low
- **Effort**: 1-2 hours
- **Approach**: Add config flag `tolerate_missing_parents: bool`

---

## Part 5: Test Coverage Verification

### Tests to Add

1. ✅ **`exclude_grandparent_property()`** - ADDED (lines 550-621)
2. ✅ **`exclude_great_grandparent_property()`** - ADDED (lines 623-682)
3. ✅ **`mixed_excludes_at_multiple_levels()`** - ADDED (lines 684-748)

### Test Results

All tests **PASS** ✅, confirming that Lithos already correctly handles excludes on all ancestors.

---

## Part 6: Conclusion

### Summary of Findings

1. **Lithos inheritance is MORE robust than initially thought**
   - Cycle detection via DFS (better than MetadataMenu)
   - Depth tracking + limits (MetadataMenu lacks this)
   - Sorted property merge (MetadataMenu order not guaranteed)

2. **Excludes work correctly on ancestors**
   - Topological ordering means parent properties include full chain
   - Tests confirm grandparent/great-grandparent excludes work

3. **Only performance optimization missing**
   - Ancestor caching could reduce repeated tree builds
   - Not a correctness gap, just efficiency

### Next Steps

**Immediate**:

- [ ] Update `metadatamenu-reference.md` with accurate comparison
- [ ] Document Lithos's approach in architecture docs
- [ ] Archive outdated comparison documents

**Future Enhancements** (if needed):

- [ ] Implement ancestor caching (8 hours, medium priority)
- [ ] Add lenient parent resolution flag (2 hours, low priority)

---

## Appendix A: Source Files Reviewed

### MetadataMenu

- `src/fileClass/fileClass.ts` (lines 1-560)
- `src/index/FieldIndex.ts` (lines 264-291, full file)

### Lithos

- `lithos-core/src/schema/aggregate.rs` (lines 68-166)
- `lithos-core/src/schema/extender.rs` (lines 220-525, full algorithm)
- `lithos-core/src/schema/resolver.rs` (lines 73-232, full merge logic)
- `lithos-core/src/schema/views/inheritance.rs` (lines 42-108)

---

**Document Status**: ✅ Research Complete
**Verification**: Tests written and passing
**Next**: Update reference documentation
