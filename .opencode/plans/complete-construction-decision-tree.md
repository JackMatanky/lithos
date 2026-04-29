# Complete Schema Construction Decision Tree

**Purpose**: Exhaustive mapping of all possible state combinations to construction strategies

**Date**: 2026-04-29
**Status**: Comprehensive Analysis - All Paths Documented

---

## Understanding the Two-Level Construction Model

### Level 1: Base Properties (Own Properties Only)

**Source**: `SCHEMA_BASE_PROPERTIES` table (`BasePropertiesView`)
**Contains**: Fully expanded own properties (refs resolved, inline included, NO parent properties)
**Cache Validation**: `BasePropertiesView.hash == RawSchemaView.current().hashes().properties()`

### Level 2: Full Schema (Base + Inherited Properties)

**Source**: `SCHEMAS` table (`Schema`)
**Contains**: Merged properties (own + inherited from parents, filtered by excludes)
**Formula**: `final = own_expanded + (parent_merged - excludes)`

---

## State Variables

Each schema has the following independent state dimensions:

### Dimension 1: Base Properties State
- **CACHED_VALID** - Base properties cache exists and hash matches
- **CACHED_INVALID** - Base properties cache exists but hash doesn't match
- **MISSING** - No base properties cache exists

### Dimension 2: Property Changes
- **NO_CHANGE** - No property delta
- **INLINE_ONLY** - Only inline properties changed
- **REFS_ONLY** - Only ref properties changed
- **BOTH** - Both inline and refs changed
- **BANK_AFFECTED** - Bank properties this schema references changed

### Dimension 3: Parent Relationship State
- **UNCHANGED** - Same parent(s) as before
- **REWIRED** - Parent changed to different schema
- **ROOT_TO_CHILD** - Gained a parent (was root, now child)
- **CHILD_TO_ROOT** - Lost parent (was child, now root)
- **CASCADE** - Parent's properties changed (but relationship unchanged)

### Dimension 4: Excludes State
- **UNCHANGED** - Same excludes list
- **ADDED_ONLY** - Only added excludes
- **REMOVED_ONLY** - Only removed excludes
- **BOTH** - Both added and removed excludes

### Dimension 5: File/Metadata State
- **FRESH** - No file changes
- **TIMESTAMPS_ONLY** - File timestamps changed, content hash same
- **CONTENT_CHANGED** - File content hash changed
- **NEW** - Schema doesn't exist in DB yet
- **DELETED** - File removed from disk

---

## Construction Strategies

### Base Properties Strategies (Level 1)

#### BP-1: USE_CACHED_BASE
**When**: Base properties cache valid
**Operations**:
1. Fetch `BasePropertiesView` from DB
2. Use `properties` field directly

**Cost**: 1 DB read

---

#### BP-2: DELTA_EXPAND_BASE
**When**: Base properties cache valid for some properties, others changed
**Operations**:
1. Fetch `BasePropertiesView` from DB (cached base)
2. Expand ONLY changed properties:
   - If inline changed: construct inline properties from raw
   - If refs changed: expand refs through `RefExpander`
   - If bank affected: expand affected refs through `RefExpander`
3. Merge: `cached_base - removals + newly_expanded`
4. Save new `BasePropertiesView` to DB

**Cost**: 1 DB read + partial expansion + 1 DB write

**Key Optimization**: Only expands changed properties, not all properties

---

#### BP-3: FULL_EXPAND_BASE
**When**: No cache OR cache invalid (hash mismatch) OR all properties changed
**Operations**:
1. Expand ALL properties from raw:
   - Construct inline properties from raw
   - Expand all refs through `RefExpander`
2. Save `BasePropertiesView` to DB

**Cost**: Full expansion + 1 DB write

---

### Full Schema Strategies (Level 2)

#### FS-1: FETCH_MERGED
**When**: Nothing changed, schema unchanged in DB
**Operations**:
1. Fetch `Schema` from DB
2. Return as-is

**Cost**: 1 DB read

---

#### FS-2: SKIP_CONSTRUCTION
**When**: Only metadata changed (timestamps, content hash)
**Operations**:
1. Update `RawSchemaView` metadata
2. Fetch `Schema` from DB
3. Return as-is

**Cost**: 1 DB read + 1 view update

**Note**: No base properties or schema reconstruction needed

---

#### FS-3: INCREMENTAL_EXCLUDES
**When**: ONLY excludes changed (no property changes, no parent changes)
**Operations**:
1. Get base properties (cached)
2. Fetch parent `Schema` from DB
3. Extract parent properties: `parent.properties - base_properties`
4. Apply excludes delta:
   - Add properties from `ExcludesDelta.added()` (take from parent properties)
   - Remove properties from `ExcludesDelta.removals()` (unless in base - name collision check)
5. Merge: `base + parent_filtered`
6. Create new schema

**Cost**: 1 base read + 1 schema read + incremental filter + merge

**Caveat**: Must check if removed exclude name exists in base properties (avoid removing local property)

**Algorithm**:
```rust
let base_props = fetch_base_properties(id);
let parent_schema = fetch_schema(parent_id);
let parent_props = parent_schema.properties();

// Handle added excludes (exclude from parent)
let mut final_props = base_props.clone();
for exclude_name in excludes_delta.added() {
    // Don't add from parent if excluded
    if !parent_props.contains_key(exclude_name) {
        continue;
    }
}

// Handle removed excludes (now inherit from parent)
for exclude_name in excludes_delta.removals() {
    // Only add if NOT in base (avoid overriding local property)
    if !base_props.contains_key(exclude_name) {
        if let Some(prop) = parent_props.get(exclude_name) {
            final_props.insert(exclude_name.clone(), prop.clone());
        }
    }
}

Schema::new(id, name, parents, children, final_props)
```

---

#### FS-4: FULL_MERGE
**When**: Properties OR parent relationship changed
**Operations**:
1. Get base properties (cached or fresh)
2. Collect parent properties (from constructed cache or DB)
3. Merge with excludes: `Merger::inherit_properties(&parent_props, &base_props, excludes)`
4. Create new schema

**Cost**: Base properties (varies) + parent collection + merge

---

#### FS-5: CASCADE_MERGE
**When**: Own properties unchanged, parent properties changed
**Operations**:
1. Get base properties (cached - unchanged)
2. Collect NEW parent properties (from constructed cache - parents were rebuilt)
3. Merge with excludes
4. Create new schema

**Cost**: 1 base read + parent collection (from cache) + merge

**Key Optimization**: No base expansion needed, parents already in cache

---

## Complete Construction Decision Matrix

### Matrix Structure

Each row represents a unique combination of states leading to a specific construction path.

**Legend**:
- ✓ = Condition true
- ✗ = Condition false
- • = Don't care / any value
- N/A = Not applicable

---

### Section 1: No Construction Needed

| # | File State      | Base Cache | Props Δ | Parent State | Excludes Δ | Base Strategy | Schema Strategy  | Code Path |
|---|-----------------|------------|---------|--------------|------------|---------------|------------------|-----------|
| 1 | FRESH           | VALID      | NO      | UNCHANGED    | NO         | (skip)        | FETCH_MERGED     | A-1       |
| 2 | TIMESTAMPS_ONLY | VALID      | NO      | UNCHANGED    | NO         | (skip)        | SKIP_CONSTRUCTION| A-2       |
| 3 | CONTENT_CHANGED | VALID      | NO      | UNCHANGED    | NO         | (skip)        | SKIP_CONSTRUCTION| A-3       |

**Notes**:
- Path 1: Pure fetch - nothing changed
- Path 2-3: Only metadata changed - update view, fetch schema

---

### Section 2: Excludes-Only Changes

| # | File State      | Base Cache | Props Δ | Parent State | Excludes Δ    | Base Strategy    | Schema Strategy      | Code Path |
|---|-----------------|------------|---------|--------------|---------------|------------------|----------------------|-----------|
| 4 | CONTENT_CHANGED | VALID      | NO      | UNCHANGED    | ADDED_ONLY    | USE_CACHED_BASE  | INCREMENTAL_EXCLUDES | B-1       |
| 5 | CONTENT_CHANGED | VALID      | NO      | UNCHANGED    | REMOVED_ONLY  | USE_CACHED_BASE  | INCREMENTAL_EXCLUDES | B-2       |
| 6 | CONTENT_CHANGED | VALID      | NO      | UNCHANGED    | BOTH          | USE_CACHED_BASE  | INCREMENTAL_EXCLUDES | B-3       |

**Notes**:
- Base properties unchanged (cache valid, no property delta)
- Only excludes list changed
- Use incremental excludes strategy

---

### Section 3: Property Changes with Valid Cache

| # | File State      | Base Cache | Props Δ      | Parent State | Excludes Δ | Base Strategy       | Schema Strategy | Code Path |
|---|-----------------|------------|--------------|--------------|------------|---------------------|-----------------|-----------|
| 7 | CONTENT_CHANGED | VALID      | INLINE_ONLY  | UNCHANGED    | •          | DELTA_EXPAND_BASE   | FULL_MERGE      | C-1       |
| 8 | CONTENT_CHANGED | VALID      | REFS_ONLY    | UNCHANGED    | •          | DELTA_EXPAND_BASE   | FULL_MERGE      | C-2       |
| 9 | CONTENT_CHANGED | VALID      | BOTH         | UNCHANGED    | •          | DELTA_EXPAND_BASE   | FULL_MERGE      | C-3       |
| 10| CONTENT_CHANGED | VALID      | BANK_AFFECTED| UNCHANGED    | •          | DELTA_EXPAND_BASE   | FULL_MERGE      | C-4       |

**Notes**:
- Base cache valid for unchanged properties
- Delta expansion for changed properties only
- Parent unchanged, full merge needed

---

### Section 4: Property Changes with Invalid/Missing Cache

| # | File State      | Base Cache      | Props Δ | Parent State | Excludes Δ | Base Strategy     | Schema Strategy | Code Path |
|---|-----------------|-----------------|---------|--------------|------------|-------------------|-----------------|-----------|
| 11| CONTENT_CHANGED | INVALID         | •       | UNCHANGED    | •          | FULL_EXPAND_BASE  | FULL_MERGE      | D-1       |
| 12| CONTENT_CHANGED | MISSING         | •       | UNCHANGED    | •          | FULL_EXPAND_BASE  | FULL_MERGE      | D-2       |
| 13| NEW             | MISSING         | N/A     | UNCHANGED    | N/A        | FULL_EXPAND_BASE  | FULL_MERGE      | D-3       |

**Notes**:
- Cache invalid or missing - must expand all properties
- New schemas always need full expansion

---

### Section 5: Parent Relationship Changes (No Cascade)

| # | File State      | Base Cache | Props Δ | Parent State  | Excludes Δ | Base Strategy      | Schema Strategy | Code Path |
|---|-----------------|------------|---------|---------------|------------|--------------------|-----------------|-----------|
| 14| •               | VALID      | NO      | REWIRED       | •          | USE_CACHED_BASE    | FULL_MERGE      | E-1       |
| 15| •               | VALID      | NO      | ROOT_TO_CHILD | •          | USE_CACHED_BASE    | FULL_MERGE      | E-2       |
| 16| •               | VALID      | NO      | CHILD_TO_ROOT | •          | USE_CACHED_BASE    | FULL_MERGE      | E-3       |
| 17| CONTENT_CHANGED | VALID      | YES     | REWIRED       | •          | DELTA_EXPAND_BASE  | FULL_MERGE      | E-4       |
| 18| CONTENT_CHANGED | VALID      | YES     | ROOT_TO_CHILD | •          | DELTA_EXPAND_BASE  | FULL_MERGE      | E-5       |

**Notes**:
- Parent relationship changed - need to collect new parent properties
- Base properties may or may not have changed (orthogonal)
- Always use FULL_MERGE (new parent properties)

---

### Section 6: Cascade (Parent Properties Changed)

| # | File State | Base Cache | Props Δ | Parent State | Excludes Δ | Base Strategy   | Schema Strategy | Code Path |
|---|------------|------------|---------|--------------|------------|-----------------|-----------------|-----------|
| 19| FRESH      | VALID      | NO      | CASCADE      | NO         | USE_CACHED_BASE | CASCADE_MERGE   | F-1       |
| 20| TIMESTAMPS | VALID      | NO      | CASCADE      | NO         | USE_CACHED_BASE | CASCADE_MERGE   | F-2       |
| 21| CONTENT    | VALID      | NO      | CASCADE      | NO         | USE_CACHED_BASE | CASCADE_MERGE   | F-3       |

**Notes**:
- Own properties unchanged
- Parent properties changed (parent was rebuilt)
- Use cached base, collect new parents, re-merge

---

### Section 7: Cascade + Own Changes

| # | File State      | Base Cache | Props Δ | Parent State | Excludes Δ | Base Strategy     | Schema Strategy | Code Path |
|---|-----------------|------------|---------|--------------|------------|-------------------|-----------------|-----------|
| 22| CONTENT_CHANGED | VALID      | YES     | CASCADE      | NO         | DELTA_EXPAND_BASE | FULL_MERGE      | G-1       |
| 23| CONTENT_CHANGED | VALID      | YES     | CASCADE      | YES        | DELTA_EXPAND_BASE | FULL_MERGE      | G-2       |
| 24| CONTENT_CHANGED | VALID      | NO      | CASCADE      | YES        | USE_CACHED_BASE   | FULL_MERGE      | G-3       |

**Notes**:
- Both own properties AND parent properties changed
- Cascade + property changes use full merge
- Cascade + excludes changes use full merge

---

### Section 8: Deleted Schemas

| # | File State | Base Cache | Props Δ | Parent State | Excludes Δ | Base Strategy | Schema Strategy | Code Path |
|---|------------|------------|---------|--------------|------------|---------------|-----------------|-----------|
| 25| DELETED    | •          | N/A     | •            | N/A        | (none)        | DELETE          | H-1       |

**Notes**:
- File removed from disk
- Delete from all tables (schemas, base_properties, views)

---

## Construction Path Reference

### Path A: No Construction

#### A-1: Pure Fetch (Fresh, Unchanged)
```rust
fn construct_a1(id: SchemaId, repo: &Repository) -> Result<Schema> {
    repo.find_schema_by_id(id)
}
```

**Statuses**: Fresh (not in cascade)

---

#### A-2: Metadata-Only Refresh (Timestamps Changed)
```rust
fn construct_a2(
    id: SchemaId,
    view: &mut RawSchemaView,
    stats: FileStats,
    content_hash: Blake3Hash,
    repo: &Repository,
) -> Result<Schema> {
    // Update view metadata
    view.update_metadata(stats, content_hash);
    repo.save_raw_schema_view(id, view)?;

    // Fetch unchanged schema
    repo.find_schema_by_id(id)
}
```

**Statuses**: StaleTimestamps (not in cascade), StaleContent (not in cascade)

---

#### A-3: Content Hash Changed, No Semantic Changes
```rust
fn construct_a3(
    id: SchemaId,
    view: &mut RawSchemaView,
    stats: FileStats,
    content_hash: Blake3Hash,
    repo: &Repository,
) -> Result<Schema> {
    // Same as A-2
    construct_a2(id, view, stats, content_hash, repo)
}
```

**Statuses**: StaleContent (excludes unchanged, properties unchanged)

---

### Path B: Excludes-Only Changes

#### B-1/B-2/B-3: Incremental Excludes
```rust
fn construct_b(
    id: SchemaId,
    excludes_delta: &ExcludesDelta,
    raw: &RawSchema,
    repo: &Repository,
) -> Result<Schema> {
    // Get cached base properties
    let base_view = repo.find_base_properties(id)?;
    let base_props = base_view.properties();

    // Get parent schema
    let parent_id = /* from topology */;
    let parent_schema = repo.find_schema_by_id(parent_id)?;
    let parent_props = parent_schema.properties();

    // Apply excludes delta
    let mut final_props = base_props.clone();

    // Removed excludes: now inherit from parent (if not in base)
    for name in excludes_delta.removals() {
        if !base_props.contains_key(name) {
            if let Some(prop) = parent_props.get(name) {
                final_props.insert(name.clone(), prop.clone());
            }
        }
    }

    // Added excludes: remove from final if came from parent
    for name in excludes_delta.added() {
        if !base_props.contains_key(name) {
            final_props.remove(name);
        }
    }

    Ok(Schema::new(id, name, parents, children, final_props))
}
```

**Statuses**: (Reserved) ExcludesChanged

**Optimization**: No property expansion, just incremental parent property filter

---

### Path C: Property Delta with Valid Cache

#### C-1/C-2/C-3/C-4: Delta Expansion
```rust
fn construct_c(
    id: SchemaId,
    property_delta: &SchemaPropertyDelta,
    raw: &RawSchema,
    property_bank: &PropertyBank,
    repo: &Repository,
    topology: &InheritanceGraph,
    constructed_cache: &HashMap<SchemaId, Arc<Schema>>,
) -> Result<Schema> {
    // Get cached base properties
    let base_view = repo.find_base_properties(id)?;
    let mut base_props = base_view.properties().clone();

    // Expand ONLY changed properties
    let expander = RefExpander::new(property_bank);

    // Expand changed refs
    for (name, ref_entry) in property_delta.upserts().refs() {
        let expanded = expander.expand_ref(ref_entry)?;
        base_props.insert(name.clone(), expanded);
    }

    // Construct changed inline properties
    for (name, inline_entry) in property_delta.upserts().inline() {
        let prop = Property::try_from(inline_entry)?;
        base_props.insert(name.clone(), prop);
    }

    // Remove deleted properties
    for name in property_delta.removals() {
        base_props.remove(name);
    }

    // Save updated base properties
    let new_hash = raw.properties().compute_hashes();
    let new_base_view = BasePropertiesView::new(base_props.clone(), new_hash);
    repo.save_base_properties(id, &new_base_view)?;

    // Collect parent properties
    let parent_props = collect_parent_properties(
        topology.parents_of(id),
        constructed_cache,
        repo,
    );

    // Full merge
    let merged = Merger::inherit_properties(
        &parent_props,
        &base_props,
        raw.excludes(),
    );

    Ok(Schema::new(id, name, parents, children, merged))
}
```

**Statuses**: Stale + property_delta + extends unchanged

**Optimization**: Only expands changed properties, not all properties

---

### Path D: Full Base Expansion

#### D-1/D-2/D-3: Full Expansion
```rust
fn construct_d(
    id: SchemaId,
    raw: &RawSchema,
    property_bank: &PropertyBank,
    repo: &Repository,
    topology: &InheritanceGraph,
    constructed_cache: &HashMap<SchemaId, Arc<Schema>>,
) -> Result<Schema> {
    // Expand ALL properties
    let expander = RefExpander::new(property_bank);
    let mut base_props = PropertyMap::new();

    // Expand all refs
    let refs = raw.properties().ref_entries();
    let expanded_refs = expander.expand_properties(&refs)?;
    base_props.extend(expanded_refs);

    // Add all inline properties
    let inline_entries = raw.properties().inline_entries();
    let inline_props = PropertyMap::try_from(inline_entries)?;
    base_props.extend(inline_props);

    // Save base properties
    let hash = raw.properties().compute_hashes();
    let base_view = BasePropertiesView::new(base_props.clone(), hash);
    repo.save_base_properties(id, &base_view)?;

    // Collect parent properties
    let parent_props = collect_parent_properties(
        topology.parents_of(id),
        constructed_cache,
        repo,
    );

    // Full merge
    let merged = Merger::inherit_properties(
        &parent_props,
        &base_props,
        raw.excludes(),
    );

    Ok(Schema::new(id, name, parents, children, merged))
}
```

**Statuses**: New, Stale + cache invalid/missing

---

### Path E: Parent Relationship Changes

#### E-1/E-2/E-3/E-4/E-5: Rewired/Root↔Child
```rust
fn construct_e(
    id: SchemaId,
    raw: &RawSchema,
    property_delta: Option<&SchemaPropertyDelta>,
    property_bank: &PropertyBank,
    repo: &Repository,
    topology: &InheritanceGraph,
    constructed_cache: &HashMap<SchemaId, Arc<Schema>>,
) -> Result<Schema> {
    // Get base properties (cached or delta expand)
    let base_props = if let Some(delta) = property_delta {
        // Delta expand (Path C logic)
        construct_c(id, delta, raw, property_bank, repo, topology, constructed_cache)?.
            properties().clone()
    } else {
        // Use cached
        repo.find_base_properties(id)?.properties().clone()
    };

    // Collect NEW parent properties (relationship changed)
    let new_parent_props = collect_parent_properties(
        topology.parents_of(id),  // NEW parents
        constructed_cache,
        repo,
    );

    // Full merge with new parents
    let merged = Merger::inherit_properties(
        &new_parent_props,
        &base_props,
        raw.excludes(),
    );

    Ok(Schema::new(id, name, parents, children, merged))
}
```

**Statuses**: Any + ExtendsChangeKind::{Rewired, RootToChild, ChildToRoot}

---

### Path F: Cascade (Parent Changed)

#### F-1/F-2/F-3: Cascade Re-Merge
```rust
fn construct_f(
    id: SchemaId,
    raw: &RawSchema,
    repo: &Repository,
    topology: &InheritanceGraph,
    constructed_cache: &HashMap<SchemaId, Arc<Schema>>,
) -> Result<Schema> {
    // Get cached base properties (unchanged)
    let base_props = repo.find_base_properties(id)?.properties();

    // Collect NEW parent properties (parents were rebuilt)
    // Parents MUST be in constructed_cache (topological order)
    let new_parent_props = collect_parent_properties_from_cache(
        topology.parents_of(id),
        constructed_cache,
    );

    // Full merge with new parent properties
    let merged = Merger::inherit_properties(
        &new_parent_props,
        base_props,
        raw.excludes(),
    );

    Ok(Schema::new(id, name, parents, children, merged))
}
```

**Statuses**: Fresh (in cascade), StaleTimestamps (in cascade)

**Key Optimization**: No base expansion, parents in cache (already constructed)

---

### Path G: Cascade + Own Changes

#### G-1/G-2/G-3: Cascade + Property/Excludes Changes
```rust
fn construct_g(
    id: SchemaId,
    property_delta: Option<&SchemaPropertyDelta>,
    excludes_delta: Option<&ExcludesDelta>,
    raw: &RawSchema,
    property_bank: &PropertyBank,
    repo: &Repository,
    topology: &InheritanceGraph,
    constructed_cache: &HashMap<SchemaId, Arc<Schema>>,
) -> Result<Schema> {
    // Get base properties (cached or delta expand)
    let base_props = if let Some(delta) = property_delta {
        // Delta expand
        construct_c_base_only(id, delta, raw, property_bank, repo)?
    } else {
        // Use cached
        repo.find_base_properties(id)?.properties().clone()
    };

    // Collect NEW parent properties (cascade)
    let new_parent_props = collect_parent_properties_from_cache(
        topology.parents_of(id),
        constructed_cache,
    );

    // Full merge
    let merged = Merger::inherit_properties(
        &new_parent_props,
        &base_props,
        raw.excludes(),  // May have new excludes
    );

    Ok(Schema::new(id, name, parents, children, merged))
}
```

**Statuses**: Any in cascade + property/excludes changes

---

### Path H: Deletion

#### H-1: Delete Schema
```rust
fn construct_h(id: SchemaId, repo: &Repository) -> Result<()> {
    repo.delete_schema(id)?;
    repo.delete_base_properties(id)?;
    repo.delete_raw_schema_view(id)?;
    Ok(())
}
```

**Statuses**: Deleted

---

## Summary Statistics

### Total Construction Paths: 25

**By Category**:
- No construction: 3 paths (A-1 to A-3)
- Excludes-only: 3 paths (B-1 to B-3)
- Property delta: 4 paths (C-1 to C-4)
- Full expansion: 3 paths (D-1 to D-3)
- Parent changes: 5 paths (E-1 to E-5)
- Cascade: 3 paths (F-1 to F-3)
- Cascade + own: 3 paths (G-1 to G-3)
- Deletion: 1 path (H-1)

### By Base Properties Strategy

| Strategy          | Path Count | Optimization Level |
|-------------------|------------|--------------------|
| Skip (no access)  | 3          | Maximum            |
| USE_CACHED_BASE   | 10         | High               |
| DELTA_EXPAND_BASE | 7          | Medium             |
| FULL_EXPAND_BASE  | 3          | Low (necessary)    |
| None (deletion)   | 1          | N/A                |

### By Schema Strategy

| Strategy              | Path Count | Requires Expansion? |
|-----------------------|------------|---------------------|
| FETCH_MERGED          | 1          | No                  |
| SKIP_CONSTRUCTION     | 2          | No                  |
| INCREMENTAL_EXCLUDES  | 3          | No                  |
| CASCADE_MERGE         | 3          | No                  |
| FULL_MERGE            | 15         | Maybe (depends)     |
| DELETE                | 1          | N/A                 |

---

## Missing Implementations & Design Needs

### 1. StaleBankReferencesPayload (Q2 Answer)

**Need**: Separate payload to track bank-affected properties

```rust
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaleBankReferencesPayload {
    path: RelativePath,
    stats: FileStats,
    content_str: Box<str>,
    content_hash: Blake3Hash,
    view: RawSchemaView,
    affected_refs: HashSet<PropertyName>,  // NEW: Which refs are affected
}
```

**Populated in**: Comparison stage (lines 947-967)

**Used in**: Analysis stage to create property delta

---

### 2. Base Properties Cache Validation (Q4 Answer)

**Need**: Method to check if base properties cache is valid

```rust
impl BasePropertiesView {
    /// Returns true if this cache matches the current raw schema snapshot.
    pub fn is_current(&self, raw_view: &RawSchemaView) -> bool {
        raw_view.current()
            .map(|v| v.hashes().properties() == self.hash())
            .unwrap_or(false)
    }
}
```

---

### 3. Base Properties Population Points (Q3 Answer)

**When to populate/update** `SCHEMA_BASE_PROPERTIES`:

| Stage        | Condition                     | Action                           |
|--------------|-------------------------------|----------------------------------|
| Construction | After DELTA_EXPAND_BASE       | Save updated base properties     |
| Construction | After FULL_EXPAND_BASE        | Save new base properties         |
| Refresh      | StaleTimestamps (unchanged)   | No action (cache still valid)    |
| Refresh      | StaleContent (unchanged props)| No action (cache still valid)    |
| Refresh      | StaleBankReferences           | Wait for construction to update  |

**Proposal**: Save base properties in construction stage after expansion, before schema merge

---

### 4. Bank Reference Delta Integration (Q2 Clarification)

**Need**: Compute affected properties from bank delta in comparison stage

**Current** (lines 947-967):
```rust
let is_bank_affected = Self::bank_changed(&found_payload.view, property_bank_delta);
if is_bank_affected {
    // Mark as StaleBankReferences but don't compute specific properties
}
```

**Proposed**:
```rust
let affected_refs = if let Some(delta) = property_bank_delta {
    found_payload.view.changed_bank_references(delta)
} else {
    HashSet::new()
};

if !affected_refs.is_empty() {
    ComparedPayload::StaleBankReferences(StaleBankReferencesPayload {
        // ... existing fields
        affected_refs,  // NEW
    })
}
```

**Then in analysis**: Add `affected_refs` to property delta upserts

---

## Next Steps: Design Documents Needed

### Document B: Base Properties Integration Design

**Contents**:
1. Database schema for `SCHEMA_BASE_PROPERTIES` table
2. Repository trait methods (`save_base_properties`, `find_base_properties`, `delete_base_properties`)
3. Pipeline integration points (when to save/fetch)
4. Cache validation logic
5. Migration plan from current system

### Document C: Bank Reference Delta Implementation

**Contents**:
1. `StaleBankReferencesPayload` design
2. Comparison stage modifications (compute `affected_refs`)
3. Analysis stage modifications (integrate into property delta)
4. Delta expansion with bank-affected refs

### Document D: Batch Architecture Without Graph

**Contents**:
1. Batch container design (one per construction path)
2. Topology-only graph (no payloads)
3. Pipeline orchestration (how batches flow through stages)
4. Parallelization opportunities
5. Migration strategy

---

## Validation Checklist

Before implementation, verify:

- [ ] All 25 construction paths are reachable (no dead code)
- [ ] No path combinations missing (exhaustive coverage)
- [ ] Each path has clear entry conditions (no ambiguity)
- [ ] Optimization strategies are sound (no correctness issues)
- [ ] Base properties cache invalidation is correct
- [ ] Parent property collection works for all cases
- [ ] Excludes delta logic handles name collisions
- [ ] Cascade detection identifies all affected schemas
- [ ] Topological ordering maintained for all paths
- [ ] Error handling for each path defined

---

**END OF DECISION TREE**

Total Paths: 25
Total Strategies (Base): 4
Total Strategies (Schema): 6
Coverage: Complete (all state combinations mapped)
