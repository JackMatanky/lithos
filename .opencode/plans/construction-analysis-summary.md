# Schema Construction Analysis - Summary & Next Steps

**Purpose**: Executive summary of complete construction analysis and recommended path forward

**Date**: 2026-04-29
**Documents**: A (Decision Tree), B (Base Properties), C (Bank Delta), D (Batch Architecture - pending)

---

## What We've Accomplished

### Document A: Complete Construction Decision Tree ✅

**File**: `complete-construction-decision-tree.md`

**Contents**:

- **25 unique construction paths** mapped exhaustively
- **4 base properties strategies** (skip, use cached, delta expand, full expand)
- **6 schema construction strategies** (fetch, skip, incremental excludes, full merge, cascade merge, delete)
- **Complete state matrix** covering all combinations of:
  - File state (fresh, timestamps, content, new, deleted)
  - Base cache state (valid, invalid, missing)
  - Property delta (none, inline, refs, both, bank-affected)
  - Parent state (unchanged, rewired, root↔child, cascade)
  - Excludes delta (unchanged, added, removed, both)

**Key Finding**: Current implementation likely missing **delta expansion optimization** for bank changes

---

### Document B: Base Properties Integration Design ✅

**File**: `base-properties-integration-design.md`

**Contents**:

- Database schema for `SCHEMA_BASE_PROPERTIES` table
- Repository trait extensions (`save`, `find`, `delete` methods)
- Pipeline integration points (4 stages affected)
- Cache validation logic (`BasePropertiesView::is_current()`)
- Migration plan (4 phases)
- Testing strategy and metrics

**Key Benefits**:

- **50-80% reduction** in expansion overhead for incremental updates
- Clear separation of own vs inherited properties
- Enables cascade merge optimization (no re-expansion needed)

**Implementation**: ~10-16 hours total effort

---

### Document C: Bank Reference Delta Implementation ✅

**File**: `bank-reference-delta-implementation.md`

**Contents**:

- New payload type: `StaleBankReferencesPayload` with `affected_refs`
- Comparison stage: `compute_affected_refs()` method
- Analysis stage: `create_bank_affected_delta()` method
- Data flow from comparison → construction
- Edge case handling (deleted bank props, local overrides)
- Testing strategy

**Key Benefits**:

- **90% reduction** in expansion overhead when few bank refs affected
- Precision tracking of what actually changed
- Reuses existing delta expansion path (no new construction logic)

**Implementation**: ~7 hours total effort

---

## Critical Insights Gained

### Insight 1: Two-Level Construction Model

**Discovery**: Construction operates on TWO independent levels:

1. **Base Properties Level**: Own properties only (refs expanded, inline included)
   - Stored in: `SCHEMA_BASE_PROPERTIES` table
   - Strategies: Skip, use cached, delta expand, full expand

2. **Full Schema Level**: Base + inherited properties (merged with excludes)
   - Stored in: `SCHEMAS` table
   - Strategies: Fetch, skip construction, incremental excludes, full merge, cascade merge

**Implication**: These are **orthogonal** - can have any combination

---

### Insight 2: Cache Invalidation is Hash-Based Only

**Discovery**: `BasePropertiesView` validity is determined ONLY by hash match with `RawSchemaView.current().hashes().properties()`

**Why bank changes don't invalidate cache directly**:

- Bank changes are detected via `bank_references` map
- Affected refs are added to property delta
- Cache invalidation happens via normal property delta flow

**Implication**: No special bank change cache logic needed

---

### Insight 3: Excludes-Only Changes Can Be Incremental

**Discovery**: When ONLY excludes change:

```rust
final_props = base_props
  + (parent_props filtered by ExcludesDelta.removals())
  - (parent_props filtered by ExcludesDelta.added())
```

**Caveat**: Must check for name collisions (local property same name as excluded parent property)

**Benefit**: No property expansion needed at all!

---

### Insight 4: Cascade Rebuilds Don't Need Expansion

**Discovery**: When parent changes but own properties unchanged:

```rust
base_props = fetch_cached()  // No expansion!
new_parent_props = collect_from_cache()  // Parents already rebuilt
merged = Merger::inherit_properties(new_parent_props, base_props, excludes)
```

**Current Gap**: Likely doing full re-expansion unnecessarily

**Opportunity**: Major optimization for cascade scenarios

---

## Recommended Implementation Priority

### Phase 1: Foundation (Immediate - Required for Optimization)

**Tasks**:

1. ✅ Complete analysis (DONE - Documents A, B, C)
2. ⬜ Review analysis with team
3. ⬜ Implement `SCHEMA_BASE_PROPERTIES` table (Document B, Phase 1)
4. ⬜ Add repository methods for base properties
5. ⬜ Add `BasePropertiesView::is_current()` method

**Effort**: ~4-6 hours
**Benefit**: Infrastructure for all optimizations

---

### Phase 2: Bank Delta Integration (High Value, Low Risk)

**Tasks** (from Document C):

1. ⬜ Add `StaleBankReferencesPayload` type
2. ⬜ Implement `compute_affected_refs()` in comparison stage
3. ⬜ Implement `create_bank_affected_delta()` in analysis stage
4. ⬜ Update parse/graphing stages to carry `affected_refs`
5. ⬜ Write unit tests

**Effort**: ~7 hours
**Benefit**: 90% reduction in expansion overhead for bank changes
**Risk**: Low - additive change only

---

### Phase 3: Base Properties Population (Enables All Optimizations)

**Tasks** (from Document B, Phases 2-3):

1. ⬜ Update construction stage to save base properties after expansion
2. ⬜ Update analysis stage to check cache validity
3. ⬜ Implement delta expansion using cached base
4. ⬜ Implement cascade merge using cached base
5. ⬜ Run full ingestion to populate cache

**Effort**: ~6-8 hours
**Benefit**: 50-80% reduction in expansion overhead overall
**Risk**: Medium - affects core construction logic

---

### Phase 4: Incremental Excludes (Optional Polish)

**Tasks**:

1. ⬜ Implement `ExcludesChanged` status (reserved)
2. ⬜ Implement incremental excludes strategy
3. ⬜ Add name collision detection
4. ⬜ Write tests

**Effort**: ~4-6 hours
**Benefit**: Skip expansion for excludes-only changes
**Risk**: Low - rare use case

---

### Phase 5: Validation & Monitoring (Quality Assurance)

**Tasks**:

1. ⬜ Add cache hit/miss metrics
2. ⬜ Add expansion timing metrics
3. ⬜ Integration tests for all 25 paths
4. ⬜ Performance benchmarks
5. ⬜ Validate expected improvements

**Effort**: ~4-6 hours
**Benefit**: Confidence in optimizations
**Risk**: None - pure validation

---

## Total Implementation Effort

| Phase                   | Hours     | Complexity | Risk   | Value  |
| ----------------------- | --------- | ---------- | ------ | ------ |
| Phase 1: Foundation     | 4-6       | Medium     | None   | High   |
| Phase 2: Bank Delta     | 7         | Low        | Low    | High   |
| Phase 3: Base Props     | 6-8       | High       | Medium | High   |
| Phase 4: Incr. Excludes | 4-6       | Medium     | Low    | Low    |
| Phase 5: Validation     | 4-6       | Low        | None   | Medium |
| **TOTAL**               | **25-33** | -          | -      | -      |

**Recommended Minimum**: Phases 1-3 + 5 = ~21-28 hours for core optimizations

---

## Expected Performance Improvements

### Scenario 1: Incremental Property Changes

**Before**: Full expansion (all refs + inline)
**After**: Delta expansion (changed properties only)

**Typical case**: 10% of properties changed
**Improvement**: **90% reduction** in expansion time

---

### Scenario 2: Bank Property Changes

**Before**: Full expansion of all refs (even if schema uses 1 out of 100 bank props)
**After**: Delta expansion of affected refs only

**Typical case**: 1-2 affected refs out of 10 total refs
**Improvement**: **80-90% reduction** in expansion time

---

### Scenario 3: Cascade (Parent Changed)

**Before**: Full expansion of all properties
**After**: Use cached base, re-merge only

**Typical case**: 100% of cascade schemas
**Improvement**: **100% elimination** of expansion (just merge)

---

### Scenario 4: Excludes-Only Changes

**Before**: Full expansion (unnecessary)
**After**: Incremental filter of parent properties

**Typical case**: Rare but happens
**Improvement**: **100% elimination** of expansion

---

### Overall Expected Improvement

**Workload mix**:

- 60% unchanged (no construction)
- 20% incremental property changes
- 10% bank changes
- 5% cascade
- 5% excludes/other

**Weighted average**:

- Before: 100% baseline
- After: ~30-40% of baseline time

**Overall**: **60-70% reduction** in construction time for typical workloads

---

## Open Questions for Decision

### Q1: Should we implement all phases or subset?

**Option A**: Minimum viable (Phases 1-3 + 5)

- Core optimizations only
- ~21-28 hours
- 50-80% improvement

**Option B**: Complete (All phases)

- All optimizations
- ~25-33 hours
- 60-70% improvement

**Recommendation**: Option A first, Option B if time permits

---

### Q2: Should we batch-refactor the pipeline architecture?

**Current**: Graph-centric (single graph, evolving payloads)
**Alternative**: Batch-centric (separate batches per status, shared topology)

**Pros of batch refactor**:

- Clearer construction strategy separation
- Better parallelization potential
- Easier to add new strategies

**Cons**:

- Major refactor (~40-60 hours)
- Risk of introducing bugs
- Benefits unclear without implementation

**Recommendation**: **Defer batch refactor** - focus on optimizations first, reconsider later

---

### Q3: How to handle missing bank properties?

**Scenario**: Schema references bank property that was deleted

**Options**:

- **A**: Error (strict) - force user to fix schema
- **B**: Remove property (lenient) - auto-fix schema
- **C**: Keep old value (cached) - no change until manual fix

**Recommendation**: **Option A (error)** - fail fast, force explicit fix

---

### Q4: Should we add cache eviction?

**Current**: Cache persists forever (until schema deleted)

**Alternative**: Evict old caches (e.g., >30 days unused)

**Pros**: Reduce DB size for rarely-used schemas
**Cons**: Cache misses on next access, re-expansion overhead

**Recommendation**: **No eviction initially** - reevaluate if DB size becomes issue

---

## Next Steps

### Immediate Actions (Today)

1. **Review this summary** with team
2. **Read Documents A, B, C** for details
3. **Decide on implementation scope** (Q1: all phases or subset?)
4. **Decide on batch refactor** (Q2: now or later?)
5. **Answer open questions** (Q3, Q4)

---

### Week 1: Foundation + Bank Delta

**If approved**, start implementation:

**Days 1-2**: Foundation (Phase 1)

- Implement `SCHEMA_BASE_PROPERTIES` table
- Add repository methods
- Add cache validation method

**Days 3-4**: Bank Delta (Phase 2)

- Implement `StaleBankReferencesPayload`
- Update comparison stage
- Update analysis stage
- Write unit tests

**Day 5**: Testing & validation

- Integration tests
- Manual testing
- Code review

---

### Week 2: Base Properties Integration

**Days 1-3**: Base Properties (Phase 3)

- Update construction to save base properties
- Update analysis to check cache
- Implement delta expansion
- Implement cascade merge

**Days 4-5**: Validation (Phase 5)

- Add metrics
- Integration tests
- Performance benchmarks
- Verify improvements

---

### Week 3: Polish & Monitoring (Optional)

**If time permits**:

- Implement incremental excludes (Phase 4)
- Additional edge case tests
- Performance tuning
- Documentation updates

---

## How to Use These Documents

### For Implementation

1. **Start with Document A** (Decision Tree)
   - Identify which construction path you're implementing
   - Reference the code path number (e.g., C-1, E-4)
   - Use the pseudocode as implementation guide

2. **Reference Document B** for base properties
   - Use database schema for table creation
   - Copy repository method signatures
   - Follow pipeline integration points

3. **Reference Document C** for bank delta
   - Copy new payload types
   - Implement helper methods
   - Follow data flow diagram

---

### For Code Review

1. **Verify construction path** matches decision tree
2. **Check cache validation** logic matches Document B
3. **Verify bank delta flow** matches Document C
4. **Confirm all 25 paths** are reachable (no dead code)

---

### For Testing

1. **Use decision tree** to generate test cases
2. **Test each of 25 paths** independently
3. **Verify cache hit/miss** behavior per Document B
4. **Test bank delta** scenarios per Document C

---

## Risk Mitigation

### Risk 1: Cache Corruption

**Mitigation**:

- Hash-based validation on every cache read
- Log warnings on hash mismatch
- Fall back to full expansion on cache miss

---

### Risk 2: Performance Regression

**Mitigation**:

- Benchmarks before/after each phase
- Metrics tracking (cache hit rate, expansion time)
- Rollback capability (feature flag)

---

### Risk 3: Data Inconsistency

**Mitigation**:

- Base properties derive from raw + bank (deterministic)
- Cache can be regenerated anytime
- Integration tests verify consistency

---

### Risk 4: Breaking Changes

**Mitigation**:

- All changes are additive (new table, new fields)
- Existing code paths still work
- Gradual rollout (phase by phase)

---

## Success Criteria

### Phase 1-2 Success (Week 1)

- ✅ `SCHEMA_BASE_PROPERTIES` table exists
- ✅ Repository methods work (round-trip test passes)
- ✅ Bank delta integration complete
- ✅ Unit tests pass (>95% coverage)
- ✅ No performance regression

---

### Phase 3 Success (Week 2)

- ✅ Base properties cache populated for all schemas
- ✅ Cache hit rate >70% for incremental updates
- ✅ Delta expansion working (verified by metrics)
- ✅ Cascade merge working (no re-expansion)
- ✅ 50-80% reduction in expansion time (benchmarked)

---

### Phase 5 Success (Week 2-3)

- ✅ All 25 construction paths tested
- ✅ Integration tests pass
- ✅ Performance benchmarks show expected improvements
- ✅ No bugs in production

---

## Conclusion

We've completed a **comprehensive analysis** of the schema construction system, identifying:

- **25 unique construction paths** (all combinations)
- **10 optimization opportunities** (delta expansion, cascade merge, etc.)
- **2 major missing implementations** (base properties table, bank delta)
- **Expected 60-70% performance improvement** with all optimizations

**Recommended next step**: Review Documents A, B, C and decide on implementation scope.

---

## Document Map

| Document | File Name                                      | Purpose                         | Status |
| -------- | ---------------------------------------------- | ------------------------------- | ------ |
| A        | `complete-construction-decision-tree.md`       | All 25 construction paths       | ✅     |
| B        | `base-properties-integration-design.md`        | Base properties table design    | ✅     |
| C        | `bank-reference-delta-implementation.md`       | Bank delta integration          | ✅     |
| D        | `batch-architecture-proposal.md`               | Batch-based pipeline (optional) | ⬜     |
| Summary  | `construction-analysis-summary.md` (this file) | Executive summary & next steps  | ✅     |

**Total Pages**: ~50 pages of detailed analysis and design

---

**END OF SUMMARY**
