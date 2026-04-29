# Base Properties Integration Design (Document B)

**Purpose**: Complete design for `SCHEMA_BASE_PROPERTIES` table integration into schema processor pipeline

**Date**: 2026-04-29
**Status**: Design Specification

---

## Overview

The `SCHEMA_BASE_PROPERTIES` table stores cached, fully-expanded **own properties** (refs resolved, inline included) separately from the merged schema. This enables:

1. **Delta expansion optimization**: Only re-expand changed properties
2. **Cascade optimization**: Re-merge without re-expansion when parent changes
3. **Clear separation**: Own properties vs inherited properties

---

## Database Schema

### Table: `schema_base_properties`

**Primary Key**: `schema_id` (UUID)

| Column        | Type              | Description                                      |
|---------------|-------------------|--------------------------------------------------|
| schema_id     | UUID              | Foreign key to schemas table                     |
| properties    | BLOB (rkyv)       | Archived `PropertyMap` (fully expanded)          |
| hash          | BLOB (rkyv)       | Archived `RawPropertyMapHash` (property hashes)  |
| recorded_at   | INTEGER (unix ms) | When this cache entry was recorded               |

**Indexes**:
- Primary key on `schema_id` (implicit)
- No additional indexes needed (always accessed by primary key)

**Relationships**:
- `schema_id` references `schemas(id)` ON DELETE CASCADE
- `hash` must match `raw_schema_views.current().hashes().properties()` for validity

---

## Repository Trait Extension

### New Methods on `Repository` Trait

```rust
pub trait Repository {
    // ... existing methods

    /// Saves base properties for a schema.
    fn save_base_properties(
        &self,
        id: SchemaId,
        view: &BasePropertiesView,
    ) -> Result<(), Self::Error>;

    /// Finds base properties for a schema.
    ///
    /// Returns `None` if no cache exists for this schema.
    fn find_base_properties(
        &self,
        id: SchemaId,
    ) -> Result<Option<BasePropertiesView>, Self::Error>;

    /// Finds base properties for multiple schemas (batch operation).
    fn find_base_properties_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<HashMap<SchemaId, BasePropertiesView>, Self::Error>;

    /// Deletes base properties for a schema.
    fn delete_base_properties(
        &self,
        id: SchemaId,
    ) -> Result<(), Self::Error>;

    /// Deletes base properties for multiple schemas (batch operation).
    fn delete_base_properties_by_ids(
        &self,
        ids: &[SchemaId],
    ) -> Result<(), Self::Error>;
}
```

---

## BasePropertiesView API Extensions

### Current Implementation (from properties.rs)

```rust
pub struct BasePropertiesView {
    properties: PropertyMap,
    hash: RawPropertyMapHash,
    recorded_at: SystemTime,
}

impl BasePropertiesView {
    pub fn new(properties: PropertyMap, hash: RawPropertyMapHash) -> Self;
    pub const fn properties(&self) -> &PropertyMap;
    pub const fn hash(&self) -> &RawPropertyMapHash;
    pub const fn recorded_at(&self) -> &SystemTime;
}
```

### Proposed Extensions

```rust
impl BasePropertiesView {
    /// Returns true if this cache matches the current raw schema snapshot.
    ///
    /// Compares `self.hash` with `RawSchemaView.current().hashes().properties()`.
    pub fn is_current(&self, raw_view: &RawSchemaView) -> bool {
        raw_view.current()
            .map(|v| v.hashes().properties() == self.hash())
            .unwrap_or(false)
    }

    /// Returns true if this cache was recorded within the given duration.
    ///
    /// Useful for cache freshness heuristics (e.g., warn on old caches).
    pub fn is_fresh(&self, max_age: std::time::Duration) -> bool {
        SystemTime::now()
            .duration_since(*self.recorded_at())
            .map(|age| age < max_age)
            .unwrap_or(false)
    }

    /// Consumes self and returns owned properties.
    pub fn into_properties(self) -> PropertyMap {
        self.properties
    }
}
```

---

## Pipeline Integration Points

### Stage 1: Comparison

**Action**: Detect if base properties cache needs invalidation

**When**:
- If property delta detected → cache will be updated in construction
- If bank references changed → cache will be updated in construction
- If only excludes changed → cache stays valid

**No direct base properties access** - just detect what needs updating

---

### Stage 2: Analysis

**Action**: Determine which construction path to use

**Logic**:
```rust
// In analyze_properties stage
let base_cache_status = if let Some(base_view) = repo.find_base_properties(id)? {
    if base_view.is_current(&raw_view) {
        BaseCacheStatus::Valid(base_view)
    } else {
        BaseCacheStatus::Invalid(base_view)
    }
} else {
    BaseCacheStatus::Missing
};

// Store in payload for construction stage
AnalysisBranch::Rebuild(RebuildNodePayload {
    // ... existing fields
    base_cache_status,  // NEW
})
```

---

### Stage 3: Refresh (NEW Responsibility)

**Action**: Ensure base properties cache is saved for view updates

**When**:
- `StaleTimestamps`: Cache valid, no action needed
- `StaleContent`: Cache valid (if no property changes), no action needed
- `StaleBankReferences`: Cache will be updated in construction

**Current refresh stage** (lines 2047-2216) only updates views.

**Proposed**: No changes needed - base properties saved in construction

---

### Stage 4: Construction (PRIMARY Integration Point)

**Action**: Fetch/update/create base properties as needed per construction path

#### Integration Point 1: Fetch Cached Base Properties

**Paths**: C (delta expand), E (parent change), F (cascade), G (cascade + own)

**Location**: Beginning of `construct_schema_incremental` or batch construction

**Code**:
```rust
// Fetch base properties if cache expected
let base_props = if needs_base_properties(node) {
    match node.payload().base_cache_status() {
        BaseCacheStatus::Valid(cached) => cached.properties().clone(),
        BaseCacheStatus::Invalid(_) | BaseCacheStatus::Missing => {
            // Will be expanded below
            PropertyMap::new()
        }
    }
} else {
    PropertyMap::new()
};
```

#### Integration Point 2: Delta Expand Base Properties

**Paths**: C (property delta with valid cache)

**Location**: After determining delta expansion is needed

**Code**:
```rust
// In delta expansion path
let mut base_props = fetch_cached_base_or_default(id, repo)?;

// Expand ONLY changed properties
let expander = RefExpander::new(property_bank);

for (name, ref_entry) in property_delta.upserts().refs() {
    let expanded = expander.expand_ref(ref_entry)?;
    base_props.insert(name.clone(), expanded);
}

for (name, inline_entry) in property_delta.upserts().inline() {
    let prop = Property::try_from(inline_entry)?;
    base_props.insert(name.clone(), prop);
}

for name in property_delta.removals() {
    base_props.remove(name);
}

// Save updated base properties
let new_hash = raw.properties().compute_hashes();
let base_view = BasePropertiesView::new(base_props.clone(), new_hash);
repo.save_base_properties(id, &base_view)?;
```

#### Integration Point 3: Full Expand Base Properties

**Paths**: D (full expansion), E (parent change + property change), some G paths

**Location**: New schema or cache invalid

**Code**:
```rust
// In full expansion path
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
```

#### Integration Point 4: Skip Base Properties Update

**Paths**: A (fetch merged), B (excludes only), F (cascade - cache valid)

**Location**: When base properties unchanged

**Code**: No base properties access or update needed

---

### Stage 5: Completion

**Action**: Delete base properties for deleted schemas

**When**: Schema deleted from disk

**Code**:
```rust
// In completion stage (lines 2806-2860)
for id in &deleted_ids {
    repository.delete_schema(*id)?;
    repository.delete_base_properties(*id)?;  // NEW
    repository.delete_raw_schema_view(*id)?;
}
```

---

## Cache Validation Logic

### Primary Validation: Hash Match

**Method**: Compare `BasePropertiesView.hash` with `RawSchemaView.current().hashes().properties()`

**Reason**: Property hashes change when raw properties change (inline or refs)

**Implementation**:
```rust
impl BasePropertiesView {
    pub fn is_current(&self, raw_view: &RawSchemaView) -> bool {
        raw_view.current()
            .map(|v| v.hashes().properties() == self.hash())
            .unwrap_or(false)
    }
}
```

---

### No Bank Delta Check Needed

**Why**: Bank changes are detected via `bank_references` and added to property delta

**Bank change flow**:
1. Comparison stage: Detect bank change → mark as `StaleBankReferences`
2. Comparison stage: Compute affected refs → store in `affected_refs`
3. Analysis stage: Add `affected_refs` to property delta
4. Construction stage: Delta expand (includes affected refs)

**Therefore**: No need to check bank delta during cache validation - it's handled via property delta

---

## Batch Operations

### Batch Fetch Base Properties

**When**: Construction stage needs base properties for multiple schemas

**Benefits**:
- Single DB query for all schemas in batch
- Pre-populate cache for topological iteration

**Implementation**:
```rust
// Before topological iteration
let base_props_batch = if !requires_base_props.is_empty() {
    repository.find_base_properties_by_ids(&requires_base_props)?
} else {
    HashMap::new()
};

// Later, during iteration
let base_props = base_props_batch.get(&id)
    .map(|v| v.properties().clone())
    .unwrap_or_default();
```

---

### Batch Save Base Properties

**When**: Multiple schemas expanded in same construction run

**Benefits**:
- Reduce DB write overhead
- Atomic batch transaction

**Implementation**:
```rust
// Accumulate during construction
let mut base_props_to_save: HashMap<SchemaId, BasePropertiesView> = HashMap::new();

// ... during construction
base_props_to_save.insert(id, base_view);

// After all schemas constructed
repository.save_base_properties_batch(&base_props_to_save)?;
```

**Optional**: Depends on repository implementation capability

---

## Error Handling

### Cache Miss vs Error

**Cache miss** (`None` returned):
- Not an error - schema may be new or cache not yet populated
- Fall back to full expansion

**Database error** (I/O failure):
- Propagate as `SchemaRepositoryError`
- Abort pipeline

**Cache corruption** (hash mismatch when expected valid):
- Log warning
- Treat as cache miss, re-expand

---

### Inconsistency Detection

**Scenario**: Base properties exist but raw view missing

**Detection**:
```rust
let raw_view = repo.find_raw_schema_view(id)?.ok_or(...)?;
let base_view = repo.find_base_properties(id)?;

if base_view.is_some() && !base_view.unwrap().is_current(&raw_view) {
    warn!("Base properties cache inconsistent for schema {id}, will re-expand");
}
```

**Resolution**: Re-expand and save new cache

---

## Migration Plan

### Phase 1: Add Table and Repository Methods

**Tasks**:
1. Add `schema_base_properties` table to database schema
2. Implement repository methods (`save`, `find`, `delete`)
3. Add unit tests for repository methods

**No pipeline changes yet** - just infrastructure

---

### Phase 2: Populate Cache During Construction

**Tasks**:
1. Update construction stage to save base properties after expansion
2. Update completion stage to delete base properties for deleted schemas
3. Run full ingestion to populate cache for existing schemas

**Result**: Cache populated but not yet used for optimization

---

### Phase 3: Enable Cache-Based Optimizations

**Tasks**:
1. Update analysis stage to check cache validity
2. Update construction stage to use cached base properties
3. Implement delta expansion strategy
4. Implement cascade merge strategy

**Result**: Full optimization enabled

---

### Phase 4: Validation and Metrics

**Tasks**:
1. Add metrics for cache hit/miss rates
2. Add validation checks for cache consistency
3. Monitor performance improvements

**Expected**: 50-80% reduction in property expansion overhead for incremental updates

---

## Testing Strategy

### Unit Tests

**Test Cases**:
1. `BasePropertiesView::is_current()` returns true when hash matches
2. `BasePropertiesView::is_current()` returns false when hash differs
3. Repository save/find round-trip works correctly
4. Repository batch operations work correctly
5. Deletion cascades to base properties table

---

### Integration Tests

**Test Cases**:
1. New schema → base properties created
2. Property change → base properties updated
3. Excludes-only change → base properties unchanged
4. Cascade → base properties reused (no update)
5. Schema deletion → base properties deleted

---

### Performance Tests

**Metrics**:
- Cache hit rate (should be >70% for typical workloads)
- Expansion time with cache vs without
- Memory overhead of cache
- DB query count reduction

---

## Monitoring and Observability

### Metrics to Track

```rust
struct BasePropertiesCacheMetrics {
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_invalidations: AtomicU64,
    expansion_time_saved_ms: AtomicU64,
    total_expansions: AtomicU64,
}

impl BasePropertiesCacheMetrics {
    fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let total = hits + self.cache_misses.load(Ordering::Relaxed);
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }
}
```

---

### Debug Logging

**Log Events**:
- Cache hit: `debug!("Base properties cache HIT for schema {id}")`
- Cache miss: `debug!("Base properties cache MISS for schema {id}, will expand")`
- Cache save: `debug!("Saved base properties cache for schema {id}")`
- Cache invalidation: `warn!("Base properties cache invalid for schema {id}, hash mismatch")`

---

## Open Questions

### Q1: Cache Eviction Policy?

**Current**: Cache never evicted (stays until schema deleted)

**Alternative**: Evict old/unused caches based on `recorded_at`

**Decision Needed**: Is cache eviction needed or do we keep forever?

---

### Q2: Cache Warming?

**Current**: Cache populated on-demand during construction

**Alternative**: Background job to pre-populate cache for all schemas

**Decision Needed**: Is cache warming needed for initial load performance?

---

### Q3: Cache Compression?

**Current**: rkyv-archived PropertyMap (already compact)

**Alternative**: Additional compression (e.g., zstd) for very large property maps

**Decision Needed**: Is compression needed or is rkyv sufficient?

---

## Summary

### Key Benefits

1. **Performance**: 50-80% reduction in expansion overhead for incremental updates
2. **Clarity**: Clear separation between own and inherited properties
3. **Optimization**: Enables delta expansion and cascade merge strategies
4. **Scalability**: Batch operations reduce DB overhead

### Implementation Effort

- **Phase 1** (Infrastructure): ~2-4 hours
- **Phase 2** (Population): ~2-3 hours
- **Phase 3** (Optimization): ~4-6 hours
- **Phase 4** (Validation): ~2-3 hours

**Total**: ~10-16 hours

### Risk Assessment

- **Low Risk**: Additive change, doesn't break existing code
- **Rollback**: Can disable optimization and fall back to full expansion
- **Data Loss**: No risk - cache is derived data, can be regenerated

---

**END OF DOCUMENT B**
