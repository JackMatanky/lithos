# Schema Construction Strategies & Batch-Based Pipeline Architecture

**Purpose**: Two-part analysis for schema processor redesign

1. Map all possible statuses → construction strategies
2. Explore batch-based pipeline alternative to graph-centric processing
   **Date**: 2026-04-29
   **Status**: Analysis & Planning Phase

---

## PART 1: Complete Status → Construction Strategy Matrix

### Current Construction Strategies (As Implemented)

The construction stage currently uses **5 distinct strategies**:

#### Strategy 1: SKIP (No Construction)

**When**: Schema unchanged, not in any rebuild/refresh list
**Operations**: None - schema already correct in DB
**Code**: Lines 2374-2391

```rust
else {
    repository.find_schema_by_id(schema_id)
}
```

**Statuses Leading Here**:

- `Fresh` + NOT in `rebuild_ids` + NOT in `refresh_ids`
  **Cost**: 1 DB read per schema

---

#### Strategy 2: FETCH_ONLY (Use DB Schema As-Is)

**When**: View refreshed but schema semantics unchanged
**Operations**:

1. Fetch schema from DB
2. Return without modification
   **Code**: Lines 2347-2359

```rust
if refresh_ids.contains(&schema_id) || stale_timestamp_ids.contains(&schema_id) {
    fetched_by_id.remove(&schema_id)
}
```

**Statuses Leading Here**:

- `StaleTimestamps` + in `refresh_ids`
- `StaleContent` + in `refresh_ids`
  **Cost**: 1 batch DB read (pre-fetched at line 2278-2285)
  **View Update**: Metadata refreshed separately in `refresh_metadata()` stage (lines 2064-2215)

---

#### Strategy 3: DELTA_UPDATE (Apply Property Delta)

**When**: Properties changed, extends unchanged
**Operations**:

1. Fetch schema from DB
2. Expand changed properties from raw
3. Apply delta (upserts + removals)
4. Create new schema with updated properties
   **Code**: Lines 2468-2516

```rust
(ExtendsChangeKind::Unchanged, Some(delta)) => {
    let schema = fetched_by_id.get(&id).cloned()...;
    let mut properties = schema.properties().clone();
    for (name, prop) in expanded {
        if delta.contains_upsert(name) {
            properties.insert(name.clone(), prop.clone());
        }
    }
    for name in delta.removals() {
        properties.remove(name);
    }
    Schema::new(id, name, parents, children, properties)
}
```

**Statuses Leading Here**:

- `Stale` + has `property_delta` + `ExtendsChangeKind::Unchanged`
- (Reserved) `PropertiesChanged` + `ExtendsChangeKind::Unchanged`
  **Cost**: 1 batch DB read + expand changed properties only + delta application
  **Key Optimization**: Doesn't re-expand ALL properties, only changed ones

---

#### Strategy 4: EXPAND_AND_MERGE (Full Rebuild with Inheritance)

**When**: Extends relationship changed OR excludes changed
**Operations**:

1. Expand ALL properties from raw (refs + inline)
2. Collect parent properties (from cache or DB)
3. Merge parent properties with own properties
4. Apply excludes filter
5. Create new schema
   **Code**: Lines 2519-2559 (Rewired/RootToChild), 2586-2631 (Unchanged fallback)

```rust
(ExtendsChangeKind::Rewired | ExtendsChangeKind::RootToChild, _) => {
    let expanded = expanded_by_id.get(&id)?;
    let parent_props = Self::collect_parent_properties(...);
    let merged = Merger::inherit_properties(&parent_props, expanded, raw.excludes());
    Schema::new(id, name, parents, children, merged)
}
```

**Statuses Leading Here**:

- `Stale` + `ExtendsChangeKind::Rewired`
- `Stale` + `ExtendsChangeKind::RootToChild`
- `StaleBankReferences` + `ExtendsChangeKind::Rewired`
- `StaleBankReferences` + `ExtendsChangeKind::RootToChild`
- `Fresh` + in `rebuild_ids` (cascade) + `ExtendsChangeKind::Rewired`
- `StaleTimestamps` + in `rebuild_ids` (cascade) + `ExtendsChangeKind::Rewired`
- `New` (all new schemas use this)
  **Cost**: Expand all properties + collect parent properties + merge

---

#### Strategy 5: EXPAND_ONLY (Root Schema with No Parents)

**When**: Extends changed from child → root (removed parent)
**Operations**:

1. Expand ALL properties from raw
2. Create schema with NO parent properties
3. No merge needed
   **Code**: Lines 2562-2583

```rust
(ExtendsChangeKind::ChildToRoot, _) => {
    let expanded = expanded_by_id.get(&id)?;
    Schema::new(id, name, Vec::new(), children, expanded.clone())
}
```

**Statuses Leading Here**:

- Any status + `ExtendsChangeKind::ChildToRoot`
  **Cost**: Expand all properties only (no parent collection, no merge)

---

### Missing/Potential Construction Strategies

Based on the analysis, here are **additional strategies** that could be implemented:

#### Strategy 6: VIEW_ONLY_REFRESH (No Schema Construction)

**When**: Only view metadata changed (timestamps, content hash)
**Operations**:

1. Update view with new metadata
2. Save view to DB
3. Skip schema construction entirely
   **Current Implementation**: Partially implemented in `refresh_metadata()` stage
   **Gap**: Views are refreshed, but schemas are STILL fetched from DB in construction
   **Potential Optimization**:

```rust
if refresh_ids.contains(&schema_id) && !rebuild_ids.contains(&schema_id) {
    // Skip construction - view already updated
    continue;
}
```

**Statuses That Could Use This**:

- `StaleTimestamps` + NOT in cascade
- `StaleContent` + NOT in cascade
  **Benefit**: Avoid DB fetch for pure metadata updates

---

#### Strategy 7: INLINE_ONLY_UPDATE

**When**: Only inline properties changed (no refs changed)
**Operations**:

1. Fetch schema from DB
2. Expand ONLY inline properties (skip ref expansion)
3. Apply delta
4. Create new schema
   **Not Currently Implemented**
   **Potential Use**:

```rust
if property_delta.only_inline_changes() && extends_unchanged {
    // Expand inline only, skip ref expansion
}
```

**Statuses That Could Use This**:

- `Stale` + `property_delta.only_inline` + `ExtendsChangeKind::Unchanged`
  **Benefit**: Skip expensive ref expansion if only inline properties changed

---

#### Strategy 8: EXCLUDES_ONLY_UPDATE

**When**: Only excludes list changed (no property changes)
**Operations**:

1. Fetch schema from DB
2. Get parent properties (already expanded)
3. Re-apply NEW excludes filter to parent properties
4. Create new schema (properties unchanged)
   **Not Currently Implemented** (would use reserved `ExcludesChanged` status)
   **Potential Use**:

```rust
if excludes_delta.is_some() && property_delta.is_none() && extends_unchanged {
    let schema = fetched_by_id.get(&id)?;
    let parent_props = Self::collect_parent_properties(...);
    // Re-filter parent props with new excludes
    let filtered = apply_excludes(&parent_props, raw.excludes());
    Schema::new(id, name, parents, children, filtered)
}
```

**Statuses That Could Use This**:

- (Reserved) `ExcludesChanged` + `ExtendsChangeKind::Unchanged`
  **Benefit**: Skip property expansion entirely if only excludes changed

---

#### Strategy 9: CASCADE_REBUILD_FROM_CACHE

**When**: Schema unchanged but parent changed (cascade)
**Operations**:

1. Fetch schema from DB
2. Collect NEW parent properties (from constructed cache)
3. Re-merge with existing expanded properties
4. Create new schema
   **Partially Implemented** but could be optimized
   **Current Behavior**: Falls through to EXPAND_AND_MERGE (re-expands everything)
   **Potential Optimization**:

```rust
if in_cascade && property_delta.is_none() && excludes_unchanged {
    let schema = fetched_by_id.get(&id)?;
    let new_parent_props = Self::collect_parent_properties(...constructed_cache...);
    // Re-merge without re-expanding own properties
    let merged = Merger::inherit_properties(&new_parent_props, &schema.own_properties(), excludes);
}
```

**Statuses That Could Use This**:

- `Fresh` + in `rebuild_ids` (cascade only)
- `StaleTimestamps` + in `rebuild_ids` (cascade only)
  **Benefit**: Avoid re-expansion for schemas that only need new parent properties
  **Blocker**: Current `Schema` doesn't track "own properties" separately from inherited

---

### Complete Status → Strategy Mapping Table

| Final Status          | Extends Change | Property Δ? | Excludes Δ? | In rebuild_ids? | In refresh_ids? | Strategy                 | Code Lines |
| --------------------- | -------------- | ----------- | ----------- | --------------- | --------------- | ------------------------ | ---------- |
| `Fresh`               | `Unchanged`    | No          | No          | No              | No              | SKIP                     | 2374-2391  |
| `Fresh`               | `Unchanged`    | No          | No          | Yes (cascade)   | No              | EXPAND_AND_MERGE         | 2586-2631  |
| `Fresh`               | `Rewired`      | (any)       | (any)       | Yes (cascade)   | No              | EXPAND_AND_MERGE         | 2519-2559  |
| `Deleted`             | N/A            | N/A         | N/A         | N/A             | N/A             | (deletion in completion) | 2826-2830  |
| `StaleTimestamps`     | `Unchanged`    | No          | No          | No              | Yes             | FETCH_ONLY               | 2347-2359  |
| `StaleTimestamps`     | `Unchanged`    | No          | No          | Yes (cascade)   | No              | EXPAND_AND_MERGE         | 2586-2631  |
| `StaleContent`        | `Unchanged`    | No          | No          | No              | Yes             | FETCH_ONLY               | 2347-2359  |
| `StaleContent`        | `Unchanged`    | No          | No          | Yes (cascade)   | No              | EXPAND_AND_MERGE         | 2586-2631  |
| `Stale`               | `Unchanged`    | Yes         | No          | Yes             | No              | DELTA_UPDATE             | 2468-2516  |
| `Stale`               | `Unchanged`    | No          | Yes         | Yes             | No              | EXPAND_AND_MERGE         | 2586-2631  |
| `Stale`               | `Unchanged`    | Yes         | Yes         | Yes             | No              | EXPAND_AND_MERGE         | 2586-2631  |
| `Stale`               | `Rewired`      | (any)       | (any)       | Yes             | No              | EXPAND_AND_MERGE         | 2519-2559  |
| `Stale`               | `RootToChild`  | (any)       | (any)       | Yes             | No              | EXPAND_AND_MERGE         | 2519-2559  |
| `Stale`               | `ChildToRoot`  | (any)       | (any)       | Yes             | No              | EXPAND_ONLY              | 2562-2583  |
| `StaleBankReferences` | `Unchanged`    | Yes         | No          | Yes             | No              | DELTA_UPDATE             | 2468-2516  |
| `StaleBankReferences` | `Unchanged`    | No          | (any)       | Yes             | No              | EXPAND_AND_MERGE         | 2586-2631  |
| `StaleBankReferences` | `Rewired`      | (any)       | (any)       | Yes             | No              | EXPAND_AND_MERGE         | 2519-2559  |
| `StaleBankReferences` | `RootToChild`  | (any)       | (any)       | Yes             | No              | EXPAND_AND_MERGE         | 2519-2559  |
| `StaleBankReferences` | `ChildToRoot`  | (any)       | (any)       | Yes             | No              | EXPAND_ONLY              | 2562-2583  |
| `New`                 | `Unchanged`    | N/A         | N/A         | Yes             | No              | EXPAND_AND_MERGE         | 2669-2798  |

## **Total Unique Strategy Paths**: 19

### Construction Strategy Decision Tree

```
┌─────────────────────────────────────┐
│ Is schema in rebuild_ids?           │
└─────────┬───────────────────────────┘
          │
          ├─→ NO ──┐
          │        │
          │        ├─→ In refresh_ids? ──→ YES ──→ FETCH_ONLY
          │        │
          │        └─→ In refresh_ids? ──→ NO ───→ SKIP
          │
          └─→ YES ─┐
                   │
                   ├─→ ExtendsChangeKind = ChildToRoot ──→ EXPAND_ONLY
                   │
                   ├─→ ExtendsChangeKind = Rewired ──────→ EXPAND_AND_MERGE
                   │
                   ├─→ ExtendsChangeKind = RootToChild ──→ EXPAND_AND_MERGE
                   │
                   └─→ ExtendsChangeKind = Unchanged ──┐
                                                        │
                                                        ├─→ property_delta = Some(Δ) ──→ DELTA_UPDATE
                                                        │
                                                        └─→ property_delta = None ──────→ EXPAND_AND_MERGE (cascade)
```

---

## PART 2: Batch-Based Pipeline Architecture

### Current Graph-Centric Approach

**Core Data Structure**:

```rust
ProcessingGraph<ProcessorNode<PipelinePayload>>
```

**Characteristics**:

- Single graph flows through all stages
- Nodes carry stage-specific payloads (Present → Compared → FileParsed → etc.)
- Graph topology preserved throughout pipeline
- Status and payload evolve per-node
  **Advantages**:
- Topological order built-in (for inheritance dependencies)
- Single data structure to pass between stages
- Clear node-to-node relationships
  **Disadvantages**:
- Graph operations (map_payload) allocate new graph each stage
- Large payload enums carry unused data through stages
- Status semantics change across stages (semantic overload)
- Hard to batch operations by status (must iterate full graph each time)

---

### Alternative: Batch-Based Container Architecture

**Core Concept**: Group schemas by **status/action** into batches instead of maintaining single graph

#### Proposed Data Structures

```rust
/// Batch container for schemas at same stage with same action
pub(crate) struct SchemaBatch<T> {
    schemas: HashMap<SchemaId, T>,
}
/// Pipeline state organized by batches
pub(crate) struct BatchedPipeline {
    // Discovery/Comparison results
    fresh: SchemaBatch<FreshPayload>,
    stale_timestamps: SchemaBatch<StaleTimestampsPayload>,
    stale_content: SchemaBatch<StaleContentPayload>,
    stale_properties: SchemaBatch<StalePropertiesPayload>,
    stale_full: SchemaBatch<StaleFullPayload>,
    deleted: SchemaBatch<DeletedPayload>,
    new: SchemaBatch<NewPayload>,

    // Relationship graph (shared by all batches)
    topology: InheritanceGraph<()>,
}
```

#### Pipeline Flow with Batches

```
Discovery
    ↓
Classify into batches
    ↓
┌────────────────────────────────────────────┐
│ fresh: {...}                               │
│ stale_timestamps: {...}                    │
│ stale_content: {...}                       │
│ stale_properties: {...}                    │
│ new: {...}                                 │
│ deleted: {...}                             │
│ topology: InheritanceGraph<()>             │
└────────────────────────────────────────────┘
    ↓
Parse (only batches that need parsing)
    ↓
Expand Properties (only batches that need expansion)
    ↓
Construct (each batch uses appropriate strategy)
    ↓
Save Results
```

---

### Batch-Based Construction Strategies

#### Batch 1: Fresh Schemas

**Size**: Potentially large (most schemas unchanged)
**Strategy**: SKIP
**Operations**: None (or cascade check)

```rust
impl SchemaBatch<FreshPayload> {
    fn construct(&self, topology: &InheritanceGraph, cascade_roots: &HashSet<SchemaId>)
        -> Vec<SchemaId>
    {
        // Filter: only schemas in affected subtree need rebuild
        let affected = affected_subtree(topology, cascade_roots);
        self.schemas.keys()
            .filter(|id| affected.contains(id))
            .copied()
            .collect()
    }
}
```

---

#### Batch 2: Stale Timestamps Schemas

**Size**: Small-medium
**Strategy**: VIEW_ONLY_REFRESH + FETCH_ONLY
**Operations**:

1. Update views (batch operation)
2. Fetch schemas (batch DB read)
3. Return as-is

```rust
impl SchemaBatch<StaleTimestampsPayload> {
    fn refresh_views(&mut self, repo: &Repository) -> Result<()> {
        for (id, payload) in &mut self.schemas {
            payload.view.update_metadata(payload.stats);
            repo.save_raw_schema_view(*id, &payload.view)?;
        }
        Ok(())
    }

    fn fetch_schemas(&self, repo: &Repository) -> Result<HashMap<SchemaId, Schema>> {
        let ids: Vec<_> = self.schemas.keys().copied().collect();
        repo.find_schemas_by_ids(&ids)
            .map(|schemas| schemas.into_iter().map(|s| (*s.id(), s)).collect())
    }
}
```

---

#### Batch 3: Stale Content Schemas

**Size**: Small
**Strategy**: VIEW_ONLY_REFRESH + FETCH_ONLY
**Operations**: Same as Stale Timestamps

---

#### Batch 4: Stale Properties Schemas

**Size**: Medium
**Strategy**: DELTA_UPDATE
**Operations**:

1. Batch fetch existing schemas
2. Batch expand changed properties only
3. Apply deltas
4. Construct new schemas

```rust
impl SchemaBatch<StalePropertiesPayload> {
    fn construct_with_delta(
        &self,
        repo: &Repository,
        property_bank: &PropertyBank,
    ) -> Result<Vec<Schema>> {
        // Batch fetch
        let ids: Vec<_> = self.schemas.keys().copied().collect();
        let existing = repo.find_schemas_by_ids(&ids)?;

        // Batch expand (only changed properties)
        let expander = RefExpander::new(property_bank);
        let mut results = Vec::new();

        for (id, payload) in &self.schemas {
            let schema = existing.get(id)?;
            let expanded = expander.expand_properties(&payload.changed_refs)?;

            // Apply delta
            let mut properties = schema.properties().clone();
            for (name, prop) in expanded {
                if payload.delta.contains_upsert(&name) {
                    properties.insert(name, prop);
                }
            }
            for name in payload.delta.removals() {
                properties.remove(name);
            }

            results.push(Schema::new(*id, schema.name(), parents, children, properties));
        }

        Ok(results)
    }
}
```

---

#### Batch 5: Stale Full Schemas

**Size**: Small-medium
**Strategy**: EXPAND_AND_MERGE
**Operations**:

1. Batch expand ALL properties
2. Collect parent properties (topological order)
3. Merge
4. Construct new schemas

```rust
impl SchemaBatch<StaleFullPayload> {
    fn construct_with_merge(
        &self,
        topology: &InheritanceGraph,
        property_bank: &PropertyBank,
        constructed_cache: &HashMap<SchemaId, Arc<Schema>>,
    ) -> Result<Vec<Schema>> {
        let expander = RefExpander::new(property_bank);
        let mut results = Vec::new();

        // Process in topological order
        let topo_order = topology.topo_order()?;

        for id in topo_order {
            if let Some(payload) = self.schemas.get(&id) {
                // Expand all properties
                let expanded = expander.expand_all(&payload.raw)?;

                // Collect parent properties
                let parents = topology.parents_of(id);
                let parent_props = Self::collect_parent_properties(parents, constructed_cache);

                // Merge
                let merged = Merger::inherit_properties(&parent_props, &expanded, payload.raw.excludes());

                results.push(Schema::new(id, name, parents, children, merged));
            }
        }

        Ok(results)
    }
}
```

---

#### Batch 6: New Schemas

**Size**: Variable
**Strategy**: EXPAND_AND_MERGE
**Operations**: Same as Stale Full, but no existing schemas to fetch

---

#### Batch 7: Deleted Schemas

**Size**: Small
**Strategy**: DELETE
**Operations**: Batch delete from DB

```rust
impl SchemaBatch<DeletedPayload> {
    fn delete_all(&self, repo: &Repository) -> Result<()> {
        let ids: Vec<_> = self.schemas.keys().copied().collect();
        repo.delete_schemas(&ids)
    }
}
```

---

### Batch-Based Pipeline Stages

#### Stage 1: Discovery & Classification

**Input**: File list + existing DB state
**Output**: `BatchedPipeline`

```rust
fn discover_and_classify(
    files: &[RelativePath],
    repo: &Repository,
    property_bank_delta: Option<&HashSet<PropertyName>>,
) -> Result<BatchedPipeline> {
    let mut fresh = SchemaBatch::new();
    let mut stale_timestamps = SchemaBatch::new();
    let mut stale_content = SchemaBatch::new();
    let mut stale_properties = SchemaBatch::new();
    let mut stale_full = SchemaBatch::new();
    let mut new = SchemaBatch::new();
    let mut deleted = SchemaBatch::new();

    // Classify each file
    for file in files {
        let classification = classify_file(file, repo, property_bank_delta)?;
        match classification {
            FileClassification::Fresh(payload) => fresh.insert(id, payload),
            FileClassification::StaleTimestamps(payload) => stale_timestamps.insert(id, payload),
            // ... etc
        }
    }

    // Load topology
    let topology = repo.get_topological_graph()?;

    Ok(BatchedPipeline {
        fresh,
        stale_timestamps,
        stale_content,
        stale_properties,
        stale_full,
        new,
        deleted,
        topology,
    })
}
```

---

#### Stage 2: Parse (Selective)

**Input**: `BatchedPipeline`
**Output**: Updated batches with parsed data

```rust
impl BatchedPipeline {
    fn parse(mut self, source: &FileReader) -> Result<Self> {
        // Only parse batches that need parsing
        self.stale_full.parse_all(source)?;
        self.new.parse_all(source)?;
        // Fresh, stale_timestamps, stale_content skip parsing
        Ok(self)
    }
}
```

---

#### Stage 3: Property Expansion (Selective)

**Input**: Parsed batches
**Output**: Batches with expanded properties

```rust
impl BatchedPipeline {
    fn expand_properties(mut self, property_bank: &PropertyBank) -> Result<Self> {
        let expander = RefExpander::new(property_bank);

        // Only expand what's needed
        self.stale_properties.expand_changed(&expander)?;
        self.stale_full.expand_all(&expander)?;
        self.new.expand_all(&expander)?;

        Ok(self)
    }
}
```

---

#### Stage 4: Construction (Strategy-Specific)

**Input**: Expanded batches
**Output**: Constructed schemas

```rust
impl BatchedPipeline {
    fn construct(self, repo: &Repository, property_bank: &PropertyBank)
        -> Result<Vec<Arc<Schema>>>
    {
        let mut all_schemas = Vec::new();
        let mut cache = HashMap::new();

        // Process batches in optimal order

        // 1. Fetch-only batches (can run in parallel)
        let fetched_timestamps = self.stale_timestamps.fetch_schemas(repo)?;
        let fetched_content = self.stale_content.fetch_schemas(repo)?;
        cache.extend(fetched_timestamps);
        cache.extend(fetched_content);

        // 2. Delta update batch (needs fetched schemas)
        let updated = self.stale_properties.construct_with_delta(repo, property_bank)?;
        cache.extend(updated.iter().map(|s| (*s.id(), Arc::new(s.clone()))));

        // 3. Full rebuild batches (topological order, needs cache)
        let rebuilt_stale = self.stale_full.construct_with_merge(&self.topology, property_bank, &cache)?;
        cache.extend(rebuilt_stale.iter().map(|s| (*s.id(), Arc::new(s.clone()))));

        let new_schemas = self.new.construct_with_merge(&self.topology, property_bank, &cache)?;
        cache.extend(new_schemas.iter().map(|s| (*s.id(), Arc::new(s.clone()))));

        // 4. Cascade check for fresh batch
        let cascade_roots = self.find_cascade_roots();
        let fresh_rebuild = self.fresh.construct(&self.topology, &cascade_roots)?;
        // ... rebuild fresh schemas in cascade

        // 5. Deletions
        self.deleted.delete_all(repo)?;

        Ok(all_schemas)
    }
}
```

---

### Batch-Based vs Graph-Based Comparison

| Aspect                      | Graph-Based (Current)                  | Batch-Based (Proposed)                   |
| --------------------------- | -------------------------------------- | ---------------------------------------- |
| **Data Structure**          | Single graph with evolving payloads    | Multiple batches + shared topology       |
| **Stage Transitions**       | Graph cloned/mapped each stage         | Batches updated in-place or transformed  |
| **Status Semantics**        | Single enum, meaning changes per stage | Batch type = explicit status             |
| **Batch Operations**        | Iterate full graph, filter by status   | Direct batch access by type              |
| **Memory Overhead**         | Large payload enums throughout         | Minimal payload per batch type           |
| **Parallelization**         | Must process in topo order             | Independent batches can run in parallel  |
| **Construction Strategies** | Switch on status + extends change      | Batch type determines strategy           |
| **Cascade Detection**       | Implicit (rebuild_ids list)            | Explicit (filter fresh batch by subtree) |
| **Code Complexity**         | Complex payload matching               | Simple batch-specific methods            |
| **Type Safety**             | Payload variants can be wrong stage    | Batch type guarantees correct stage      |
| **Testing**                 | Must test all stage transitions        | Can test each batch type independently   |

---

### Hybrid Approach: Batch-Graph Combination

**Idea**: Use batches for processing, but maintain graph for topology

```rust
pub(crate) struct HybridPipeline {
    // Processing batches
    batches: StatusBatches,

    // Relationship tracking
    topology: InheritanceGraph<NodeMetadata>,
}
pub(crate) struct NodeMetadata {
    status: NodeStatus,
    extends_change: ExtendsChangeKind,
    batch_ref: BatchRef,  // Points to entry in appropriate batch
}
pub(crate) enum BatchRef {
    Fresh(SchemaId),
    StaleTimestamps(SchemaId),
    StaleContent(SchemaId),
    // ... etc
}
```

**Benefits**:

- Topology queries still O(1) on graph
- Processing operates on compact batches
- Metadata in graph is minimal (just status + ref)
- No large payload cloning

---

## PART 3: Questions & Tradeoffs

### Question 1: Is Batch-Based Worth It?

**Pros**:

- Clearer construction strategy separation
- Better parallelization potential
- Reduced memory overhead
- Type-safe stage guarantees
- Easier to add new strategies (just add new batch type)
  **Cons**:
- Major refactor required
- Topology queries need separate graph (hybrid approach)
- More types to maintain
- Cascade logic more complex (need to cross-reference batches)
  **Recommendation**: Consider **hybrid approach** first
- Keep topology in graph
- Use batches for payload data
- Graph nodes hold minimal metadata + batch refs

---

### Question 2: How to Handle Cascades in Batch System?

**Option A**: Pre-compute affected set, promote schemas to different batch

```rust
let cascade_roots = find_cascade_roots(&batches);
let affected = affected_subtree(&topology, &cascade_roots);
// Move affected fresh schemas to stale_full batch
for id in affected {
    if let Some(payload) = batches.fresh.remove(&id) {
        batches.stale_full.insert(id, promote_to_stale(payload));
    }
}
```

**Option B**: Keep in fresh batch, flag for rebuild

```rust
batches.fresh.mark_for_cascade_rebuild(&affected_ids);
// Construction checks flag
```

**Option C**: Separate cascade batch

```rust
struct BatchedPipeline {
    fresh: SchemaBatch<Fresh>,
    cascade_rebuild: SchemaBatch<CascadeRebuild>,
    // ...
}
```

---

### Question 3: Should Construction Strategies Be Traits?

**Current**: Functions on `SchemaProcessor` impl
**Batch-Based**: Methods on each batch type
**Alternative**: Strategy pattern with traits

```rust
trait ConstructionStrategy {
    type Payload;
    fn construct(
        &self,
        batch: &SchemaBatch<Self::Payload>,
        context: &ConstructionContext,
    ) -> Result<Vec<Schema>>;
}
struct DeltaUpdateStrategy;
impl ConstructionStrategy for DeltaUpdateStrategy {
    type Payload = StalePropertiesPayload;
    fn construct(...) -> Result<Vec<Schema>> { ... }
}
struct ExpandAndMergeStrategy;
impl ConstructionStrategy for ExpandAndMergeStrategy {
    type Payload = StaleFullPayload;
    fn construct(...) -> Result<Vec<Schema>> { ... }
}
```

**Benefits**:

- Strategies fully decoupled
- Easy to test independently
- Can swap strategies at runtime
- Clear separation of concerns
  **Costs**:
- More indirection
- Trait complexity
- May be over-engineering for 5-9 strategies

---

### Question 4: What About View Refresh?

**Current**: Separate `refresh_metadata()` stage before construction
**Batch-Based**: Could be integrated into batch methods

```rust
impl SchemaBatch<StaleTimestampsPayload> {
    fn refresh_and_fetch(&mut self, repo: &Repository) -> Result<HashMap<SchemaId, Schema>> {
        // Refresh views
        for (id, payload) in &mut self.schemas {
            payload.view.update_metadata(payload.stats);
            repo.save_raw_schema_view(*id, &payload.view)?;
        }

        // Then fetch schemas
        let ids: Vec<_> = self.schemas.keys().copied().collect();
        repo.find_schemas_by_ids(&ids)
    }
}
```

## **Benefit**: Single batch operation instead of separate stage

## PART 4: Recommendations

### Immediate Actions (No Refactor Required)

1. **Document Current Strategies**: Add comments to construction code mapping statuses → strategies
2. **Extract Strategy Functions**: Pull out delta-update, expand-merge, expand-only into separate well-named functions
3. **Add Strategy Tests**: Test each construction path independently

### Short-Term Improvements (Minor Refactor)

1. **Implement VIEW_ONLY_REFRESH**: Skip schema fetch for pure metadata updates
2. **Add INLINE_ONLY_UPDATE**: Optimize for inline-only property changes
3. **Implement Reserved Statuses**: Use `ExcludesChanged` and `PropertiesChanged` with dedicated strategies

### Long-Term Exploration (Major Refactor)

1. **Prototype Hybrid Approach**: Graph for topology + batches for payloads
2. **Measure Performance**: Compare graph-based vs batch-based on real workloads
3. **Incremental Migration**: Start with one batch type (e.g., Fresh) and expand

---

## PART 5: Next Steps

### What We Need to Decide

1. **Should we pursue batch-based architecture?**
   - Full batch-based
   - Hybrid (graph + batches)
   - Stay with graph-based but optimize
2. **Which construction strategies to prioritize?**
   - VIEW_ONLY_REFRESH (easy win)
   - INLINE_ONLY_UPDATE (medium complexity)
   - EXCLUDES_ONLY_UPDATE (requires reserved status)
   - CASCADE_REBUILD_FROM_CACHE (requires schema refactor)
3. **How to handle cascades in batch system?**
   - Pre-compute and promote batches
   - Flag-based within batch
   - Separate cascade batch
4. **Should construction strategies be traits?**
   - Yes - full strategy pattern
   - No - keep as methods
   - Hybrid - traits for complex strategies only

### Suggested Discussion Flow

1. Review Part 1 (Status → Strategy mapping) - is this complete and accurate?
2. Discuss Part 2 (Batch architecture) - does this align with your vision?
3. Answer questions in Part 3 - what are your preferences?
4. Decide on recommendations in Part 4 - what should we do next?

---

## Appendix: Construction Strategy Cost Analysis

| Strategy             | DB Reads            | Property Expansion     | Parent Collection | Merge     | Delta Apply |
| -------------------- | ------------------- | ---------------------- | ----------------- | --------- | ----------- |
| SKIP                 | 1 (single fetch)    | None                   | None              | None      | None        |
| FETCH_ONLY           | 1 (batch pre-fetch) | None                   | None              | None      | None        |
| VIEW_ONLY_REFRESH    | 0 (proposed)        | None                   | None              | None      | None        |
| DELTA_UPDATE         | 1 (batch pre-fetch) | Changed props only     | None              | None      | Yes         |
| INLINE_ONLY_UPDATE   | 1 (batch pre-fetch) | Inline only (proposed) | None              | None      | Yes         |
| EXCLUDES_ONLY_UPDATE | 1 (batch pre-fetch) | None (proposed)        | Yes               | Re-filter | None        |
| EXPAND_ONLY          | 0                   | All (refs + inline)    | None              | None      | None        |
| EXPAND_AND_MERGE     | 0                   | All (refs + inline)    | Yes               | Yes       | None        |
| CASCADE_FROM_CACHE   | 1 (batch pre-fetch) | None (proposed)        | Yes (from cache)  | Re-merge  | None        |

**Key Observations**:

- Most expensive: EXPAND_AND_MERGE (full expansion + parent collection + merge)
- Cheapest: VIEW_ONLY_REFRESH (no DB read, no expansion)
- Best optimization target: Cascaded Fresh schemas (currently use EXPAND_AND_MERGE, could use CASCADE_FROM_CACHE)

---

**END OF ANALYSIS**
