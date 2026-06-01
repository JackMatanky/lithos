# Schema Pipeline Refactor: Implementation Plan

**Date**: 2026-03-23
**Status**: Ready for Implementation
**Branch**: `schema-refactor`
**Estimated Effort**: 8-12 development sessions

---

## Executive Summary

This document provides a **phased implementation plan** for refactoring the `schema` module pipelines using the Rust typestate pattern. The refactor will:

1. **Eliminate redundant abstractions** (remove `Ingestor`, simplify `Loader` to `Builder` facade)
2. **Implement explicit state machines** for PropertyBank (6 states) and Schema (9 states)
3. **Enable fail-fast validation** (early tree construction before property deserialization)
4. **Optimize incremental updates** (fresh schemas skip processing at any level)
5. **Improve code readability** (explicit state transitions, clear data flow)

---

## Phase Overview

| Phase | Focus                               | Estimated Effort | Status  |
| ----- | ----------------------------------- | ---------------- | ------- |
| **0** | Prerequisites                       | ✅ **COMPLETE**  | Done    |
| **1** | InheritanceGraph Simplification     | 3-4 hours        | Pending |
| **2** | PropertyBank State Machine          | 6-8 hours        | Pending |
| **3** | Schema State Machine (Foundation)   | 8-10 hours       | Pending |
| **4** | Schema State Machine (Optimization) | 6-8 hours        | Pending |
| **5** | Builder Facade & Ingestor Removal   | 4-6 hours        | Pending |
| **6** | Integration & Testing               | 6-8 hours        | Pending |
| **7** | Documentation & Cleanup             | 2-3 hours        | Pending |

**Total Estimated Effort**: 35-47 hours (approximately 7-10 work sessions at 5 hours each)

---

## Phase 0: Prerequisites ✅ COMPLETE

### Completed Work

1. ✅ **`RawPropertyMap<T>` wrapper** - Validates property keys during parsing
   - Custom `Deserialize` implementation
   - Automatic `PropertyName` validation
   - Zero-cost abstraction (compiles to `HashMap<PropertyName, T>`)

2. ✅ **`RawPropertyRefPath` type** - Validates `$ref` format during parsing
   - Ensures `#property_bank/` prefix
   - Extracts target property name
   - O(1) access to target via `target_name()` method

3. ✅ **`bank_references` construction** - Built in `SchemaVersion::new`
   - Filters `RawProperty::Ref` variants
   - Extracts target names from validated `RawPropertyRefPath`
   - Stored in `SchemaVersion` for delta detection

4. ✅ **Hash strategy research** - Comprehensive analysis completed
   - 64-bit vs 128-bit vs 256-bit collision probabilities
   - Blake3 truncation safety validated
   - Storage impact calculated

5. ✅ **Pipeline architecture finalized** - All design decisions confirmed
   - Early tree construction optimization
   - Level-by-level expansion + merging
   - Incremental staleness handling
   - `expanded_properties` cache strategy

### Verification

```bash
# All prerequisites implemented and tested
mise run test:unit:schema
```

---

## Phase 1: InheritanceGraph Simplification

### Goal

Simplify the `Extender` to build a lightweight `InheritanceGraph` with just IDs and metadata (no properties).

### Tasks

#### 1.1 Rename `SchemaTree` → `InheritanceGraph` ✅ COMPLETE

- ✅ Rename struct in `extender.rs`
- ✅ Update all references throughout codebase
- ✅ Update documentation

**Files Modified**:

- ✅ `lithos-core/src/schema/extender.rs`
- ✅ `lithos-core/src/schema/merger.rs`
- ✅ `lithos-core/src/schema/views/inheritance.rs`
- ✅ `lithos-core/src/schema/views/inheritance_redesign.rs`

#### 1.2 Create `InheritanceNode` Type

- [ ] Define lightweight node structure (no properties)
- [ ] Fields: `id`, `name`, `parent_id`, `children`, `depth`, `excludes`
- [ ] Remove `properties` field entirely

**Files to Modify**:

- `lithos-core/src/schema/extender.rs` (new type)

#### 1.3 Simplify `Extender::build()`

- [ ] Remove property handling logic
- [ ] Input: `Vec<(SchemaId, SchemaName, Option<SchemaName>, Vec<PropertyName>)>`
- [ ] Output: `InheritanceGraph` (topologically ordered IDs)
- [ ] Keep: cycle detection, depth computation, topological sort

**Files to Modify**:

- `lithos-core/src/schema/extender.rs` (~200 lines)

#### 1.4 Update `Merger` to Accept Graph

- [ ] Update `Merger::resolve()` signature to accept `InheritanceGraph`
- [ ] Properties will come from state machine data, not nodes

**Files to Modify**:

- `lithos-core/src/schema/merger.rs` (lines ~70-100)

### Verification

```bash
# Extender builds graph successfully
mise run test:unit:schema -- extender

# Merger works with new graph structure
mise run test:unit:schema -- merger
```

### Definition of Done

- [ ] `InheritanceGraph` contains only IDs and metadata
- [ ] No property data in graph nodes
- [ ] All extender tests passing
- [ ] Merger tests passing

---

## Phase 2: PropertyBank State Machine

### Goal

Implement the 6-state PropertyBank pipeline as a typestate state machine.

### Tasks

#### 2.1 Define State Types

- [ ] Create `PropertyBankStateMachine<S>` generic struct
- [ ] Define 6 state marker types (ZSTs):
  - `Discovery`
  - `FileParsed`
  - `PropertyDelta`
  - `BaseConstructed`
  - `DeltaApplied`
  - `Completed`

**Files to Create**:

- `lithos-core/src/schema/property_bank_pipeline.rs`

#### 2.2 Define Data Structures

- [ ] Create `PropertyBankData` struct:
  ```rust
  struct PropertyBankData {
      raw: Option<RawPropertyBank>,
      bank: Option<PropertyBank>,
      view: Option<RawPropertyBankView>,
      delta: PropertyDelta,  // Empty HashSet = no changes
  }
  ```

#### 2.3 Implement State Transitions

- [ ] `Discovery::discover()` → `PropertyBankStateMachine<Discovery>`
- [ ] Branching transitions based on staleness:
  - NEW → `FileParsed`
  - FreshTimestamp → `BaseConstructed` (from DB)
  - FreshContent → `BaseConstructed` (update timestamps)
  - STALE → `FileParsed` → `PropertyDelta` → `DeltaApplied`
- [ ] `into_completed()` → `PropertyBankStateMachine<Completed>`

#### 2.4 Implement Transition Logic

- [ ] File parsing (FileReader integration)
- [ ] Delta computation (hash comparison)
- [ ] Delta application (merge old + new properties)
- [ ] DB persistence

**Files to Modify**:

- `lithos-core/src/schema/mod.rs` (add module)
- `lithos-core/src/schema/property_bank_pipeline.rs` (new)

### Verification

```bash
# PropertyBank pipeline tests
mise run test:unit:schema -- property_bank_pipeline

# Integration test with filesystem
mise run test:integration -- property_bank
```

### Definition of Done

- [ ] All 6 states implemented
- [ ] State transitions type-safe (compile-time enforcement)
- [ ] Branching logic based on staleness
- [ ] Unit tests for each transition
- [ ] Integration test with real filesystem

---

## Phase 3: Schema State Machine (Foundation)

### Goal

Implement the basic 9-state Schema pipeline structure (without optimization).

### Tasks

#### 3.1 Define State Types

- [ ] Create `SchemaStateMachine<S>` generic struct
- [ ] Define 9 state marker types:
  - `Discovery`
  - `FileParsed`
  - `SchemaDelta`
  - `TreeConstructed`
  - `RawPropertiesDeserialized`
  - `BankReferenceDelta`
  - `RefsExpanded`
  - `PropertiesMerged`
  - `Persisted`

**Files to Create**:

- `lithos-core/src/schema/schema_pipeline.rs`

#### 3.2 Define Data Structures

- [ ] Create `SchemaData` struct:
  ```rust
  struct SchemaData {
      raw: Option<RawSchema>,
      view: Option<RawSchemaView>,
      property_delta: Option<SchemaPropertyDelta>,
      extends_delta: Option<ExtendsDelta>,
      excludes_delta: Option<ExcludesDelta>,
  }
  ```
- [ ] Create `InheritanceGraphData` for tree state
- [ ] Create `ExpandedSchemaData` for expansion state

#### 3.3 Implement Discovery → TreeConstructed

- [ ] `Discovery::discover()` - Query DB, determine staleness
- [ ] `FileParsed::parse()` - Parse TOML/JSON/YAML (only for NEW/STALE)
- [ ] `SchemaDelta::compute()` - Hash comparison (only for STALE)
- [ ] **`TreeConstructed::build_graph()`** - Early tree construction
  - Build `InheritanceGraph` with just IDs
  - Cycle detection
  - Parent verification
  - **FAIL FAST** if structural errors

#### 3.4 Implement TreeConstructed → Persisted (Basic)

- [ ] `RawPropertiesDeserialized::deserialize()` - Only for schemas needing processing
- [ ] `BankReferenceDelta::compute()` - Intersect with PropertyBank delta
- [ ] `RefsExpanded::expand()` - Expand $refs (basic, all schemas)
- [ ] `PropertiesMerged::merge()` - Merge with parents (basic, all schemas)
- [ ] `Persisted::persist()` - Save to DB

**Files to Modify**:

- `lithos-core/src/schema/mod.rs` (add module)
- `lithos-core/src/schema/schema_pipeline.rs` (new, ~800 lines)

### Verification

```bash
# Basic pipeline tests
mise run test:unit:schema -- schema_pipeline

# Integration test (simple schemas, no optimization)
mise run test:integration -- schema_basic
```

### Definition of Done

- [ ] All 9 states implemented
- [ ] Basic transition logic working
- [ ] Early tree construction functioning
- [ ] Fail-fast validation operational
- [ ] Unit tests for each state
- [ ] Integration test with simple schemas

---

## Phase 4: Schema State Machine (Optimization)

### Goal

Add incremental update optimizations (level-by-level processing, staleness skipping).

### Tasks

#### 4.1 Implement Level-by-Level Expansion

- [ ] Modify `RefsExpanded::expand()` to process schemas by topological level
- [ ] For each level:
  - FRESH schemas: Skip expansion (retrieve from DB)
  - STALE/NEW schemas: Expand local properties only
- [ ] Update `expanded_properties` cache in `SchemaVersion`

#### 4.2 Implement Level-by-Level Merging

- [ ] Modify `PropertiesMerged::merge()` to process by level
- [ ] For each level:
  - FRESH schemas (with FRESH parents): Skip merging (retrieve from DB)
  - FRESH schemas (with STALE parents): Merge cached expanded + parent merged
  - STALE/NEW schemas: Merge freshly expanded + parent merged
- [ ] Cache merged results for child levels

#### 4.3 Implement Staleness Propagation

- [ ] Track which schemas are fresh/stale
- [ ] Propagate staleness down the tree (if parent stale, children need re-merge)
- [ ] Use `ancestors_hash` for transitive staleness detection

#### 4.4 Add Performance Optimizations

- [ ] Early exit when all schemas fresh
- [ ] Skip deserialization for fresh schemas
- [ ] Batch DB queries for fresh schema retrieval

**Files to Modify**:

- `lithos-core/src/schema/schema_pipeline.rs` (lines ~400-600)

### Verification

```bash
# Optimization tests
mise run test:unit:schema -- schema_pipeline_optimization

# Integration test with mixed fresh/stale schemas
mise run test:integration -- schema_incremental

# Benchmark against old implementation
mise run test:bench:core -- schema_load
```

### Definition of Done

- [ ] Level-by-level processing working
- [ ] Fresh schemas skip expansion/merging
- [ ] Staleness propagates correctly
- [ ] Performance improvement measurable (>50% faster for fresh schemas)
- [ ] Unit tests for optimization paths
- [ ] Integration test with realistic staleness scenarios

---

## Phase 5: Builder Facade & Ingestor Removal

### Goal

Refactor `Loader` into a thin `Builder` facade and remove `Ingestor` entirely.

### Tasks

#### 5.1 Create Builder Facade

- [ ] Create `lithos-core/src/schema/builder.rs`
- [ ] Thin facade (~50 lines):

  ```rust
  pub struct Builder<'config, R> {
      config: &'config Config,
      source: FileReader,
      repository: R,
      property_delta: PropertyDelta,
  }

  impl<'config, R: Repository> Builder<'config, R> {
      pub fn load_all(self) -> Result<(), SchemaError> {
          // 1. Run PropertyBank pipeline
          let property_bank = PropertyBankStateMachine::run(...)?;

          // 2. Run Schema pipeline
          let schemas = SchemaStateMachine::run(...)?;

          Ok(())
      }
  }
  ```

#### 5.2 Remove Ingestor

- [ ] Delete `lithos-core/src/schema/ingestor.rs`
- [ ] Move filesystem logic into state machine transitions
- [ ] Move DB queries into state machine transitions

#### 5.3 Update Public API

- [ ] Update `lithos-core/src/schema/mod.rs` exports
- [ ] Remove `Loader` struct
- [ ] Export `Builder` as primary API

#### 5.4 Update Callsites

- [ ] Update `lithos-cli` to use `Builder`
- [ ] Update tests to use `Builder`

**Files to Create**:

- `lithos-core/src/schema/builder.rs`

**Files to Delete**:

- `lithos-core/src/schema/ingestor.rs`
- `lithos-core/src/schema/loader.rs` (most of it)

**Files to Modify**:

- `lithos-core/src/schema/mod.rs`
- `lithos-cli/src/commands/*.rs`

### Verification

```bash
# Builder tests
mise run test:unit:schema -- builder

# CLI integration tests
mise run test:e2e

# Full verification
mise run verify
```

### Definition of Done

- [ ] `Builder` facade implemented and working
- [ ] `Ingestor` deleted
- [ ] All CLI commands working
- [ ] All tests passing
- [ ] No clippy warnings

---

## Phase 6: Integration & Testing

### Goal

Comprehensive testing of the refactored pipelines.

### Tasks

#### 6.1 Unit Test Coverage

- [ ] PropertyBank pipeline: all 6 states
- [ ] Schema pipeline: all 9 states
- [ ] State transitions
- [ ] Error handling
- [ ] Edge cases (empty bank, circular inheritance, missing parents)

#### 6.2 Integration Tests

- [ ] Full pipeline end-to-end
- [ ] Mixed fresh/stale scenarios
- [ ] PropertyBank changes triggering schema updates
- [ ] Schema inheritance changes
- [ ] File deletion handling

#### 6.3 Performance Benchmarks

- [ ] Compare against old implementation:
  - Full load (all schemas stale)
  - Incremental load (some fresh, some stale)
  - PropertyBank-only update
  - Schema-only update
- [ ] Target: ≥ 50% improvement for incremental scenarios

#### 6.4 Error Path Testing

- [ ] Cycle detection
- [ ] Missing parent
- [ ] Invalid $ref
- [ ] File I/O errors
- [ ] DB errors

**Files to Create**:

- `lithos-core/tests/schema_pipeline_integration.rs`
- `lithos-core/benches/schema_pipeline.rs`

### Verification

```bash
# Full test suite
mise run test

# Coverage check
mise run test:coverage

# Benchmarks
mise run test:bench
```

### Definition of Done

- [ ] > 90% test coverage for new code
- [ ] All integration tests passing
- [ ] Benchmarks show performance improvement
- [ ] Error paths tested
- [ ] No flaky tests

---

## Phase 7: Documentation & Cleanup

### Goal

Update all documentation and clean up any technical debt.

### Tasks

#### 7.1 Code Documentation

- [ ] Add module-level docs to all new files
- [ ] Document all public APIs
- [ ] Add examples to complex functions
- [ ] Document state machine design patterns

#### 7.2 Architecture Documentation

- [ ] Update `_bmad-output/ARCHITECTURE_DECISIONS.md`
- [ ] Update `docs/adr/` with new ADRs:
  - ADR: Typestate Pattern for Pipelines
  - ADR: Early Tree Construction Optimization
  - ADR: Hash Size Selection
- [ ] Update `README.md` with new architecture

#### 7.3 Migration Guide

- [ ] Create migration guide for API changes
- [ ] Document breaking changes
- [ ] Provide before/after examples

#### 7.4 Cleanup

- [ ] Remove dead code
- [ ] Remove TODO comments
- [ ] Fix all clippy warnings
- [ ] Format all code

**Files to Create**:

- `docs/adr/0XX-typestate-pattern.md`
- `docs/adr/0XX-early-tree-construction.md`
- `docs/adr/0XX-hash-size-selection.md`
- `docs/migration/schema-refactor.md`

### Verification

```bash
# Documentation builds
mise run doc

# All quality gates pass
mise run verify

# ADR validation
mise run adr:validate
```

### Definition of Done

- [ ] All code documented
- [ ] ADRs created for major decisions
- [ ] Migration guide complete
- [ ] No TODOs or FIXMEs
- [ ] All quality gates green

---

## Risk Assessment

### High Risk

1. **Breaking API Changes**
   - **Risk**: Existing code depends on `Loader` API
   - **Mitigation**: Create `Builder` with similar interface; update callsites incrementally
   - **Contingency**: Provide compatibility shim if needed

2. **Performance Regression**
   - **Risk**: New implementation slower than old
   - **Mitigation**: Benchmark early and often; optimize hot paths
   - **Contingency**: Revert optimization, keep simpler version

### Medium Risk

3. **State Machine Complexity**
   - **Risk**: Typestate pattern adds cognitive overhead
   - **Mitigation**: Excellent documentation; clear naming; ASCII diagrams
   - **Contingency**: Simplify state count if too complex

4. **Test Coverage Gaps**
   - **Risk**: Missing edge cases in new implementation
   - **Mitigation**: Comprehensive integration tests; property-based testing
   - **Contingency**: Add tests as bugs discovered

### Low Risk

5. **Hash Collisions**
   - **Risk**: Shorter hashes cause false cache invalidation
   - **Mitigation**: Mathematical analysis shows negligible probability
   - **Contingency**: Easy to switch back to 256-bit if needed (data migration)

---

## Success Criteria

### Functional

- [ ] All existing functionality preserved
- [ ] PropertyBank and Schema pipelines working end-to-end
- [ ] Incremental updates functional
- [ ] Error handling comprehensive

### Performance

- [ ] ≥ 50% improvement for fresh schema loads
- [ ] ≤ 10% overhead for full stale loads
- [ ] Memory usage comparable or better

### Code Quality

- [ ] > 90% test coverage
- [ ] Zero clippy warnings
- [ ] All ADRs documented
- [ ] Clear, readable code

### Maintainability

- [ ] State transitions explicit and type-safe
- [ ] Easy to add new states if needed
- [ ] Clear separation of concerns
- [ ] Excellent documentation

---

## Rollout Plan

### 1. Development (This Branch)

- Complete Phases 0-8 on `schema-refactor` branch
- Continuous integration testing
- Incremental commits with clear messages

### 2. Internal Testing

- [ ] Full `mise run verify` passing
- [ ] Manual testing with real-world schemas
- [ ] Performance benchmarks meet criteria

### 3. Code Review

- [ ] Self-review using checklist
- [ ] Peer review (if applicable)
- [ ] Address all feedback

### 4. Merge to Main

- [ ] Squash commits into logical units
- [ ] Write comprehensive merge commit message
- [ ] Merge via PR with full CI/CD

### 5. Monitor

- [ ] Watch for issues post-merge
- [ ] Collect performance metrics
- [ ] Gather user feedback

---

## Dependencies

### External Dependencies

- None (all Rust std and existing crate dependencies)

### Internal Dependencies

- ✅ `RawPropertyMap<T>` (Phase 0 - Complete)
- ✅ `RawPropertyRefPath` (Phase 0 - Complete)
- Blake3 crate (existing)
- redb crate (existing)
- rkyv crate (existing)

### Blockers

- **None** - Ready to start Phase 1

---

## Next Steps

**Immediate Actions**:

1. **Start Phase 1** (Hash Strategy Updates)
   - Complete task 1.3 (Update Ingestor hash computation)
   - Run verification
   - Commit changes

2. **Begin Phase 2** (InheritanceGraph Simplification)
   - Create `InheritanceNode` type
   - Simplify `Extender::build()`
   - Update `Merger`

3. **Track Progress**
   - Update this document with completion dates
   - Mark tasks as done with checkboxes
   - Document any deviations from plan

**Communication**:

- Update user on progress after each phase
- Raise any blockers immediately
- Request clarification if design decisions unclear

---

## Appendix A: File Change Summary

### Files to Create (New)

- `lithos-core/src/schema/property_bank_pipeline.rs`
- `lithos-core/src/schema/schema_pipeline.rs`
- `lithos-core/src/schema/builder.rs`
- `lithos-core/tests/schema_pipeline_integration.rs`
- `lithos-core/benches/schema_pipeline.rs`
- `docs/adr/0XX-typestate-pattern.md`
- `docs/adr/0XX-early-tree-construction.md`
- `docs/adr/0XX-hash-size-selection.md`
- `docs/migration/schema-refactor.md`

### Files to Delete

- `lithos-core/src/schema/ingestor.rs` (~1500 lines)
- `lithos-core/src/schema/loader.rs` (~1000 lines)

### Files to Modify (Major Changes)

- `lithos-core/src/schema/extender.rs` (~200 lines changed)
- `lithos-core/src/schema/merger.rs` (~100 lines changed)
- `lithos-core/src/schema/views/metadata.rs` (✅ Complete)
- `lithos-core/src/schema/views/inheritance_redesign.rs` (✅ Complete)
- `lithos-core/src/schema/mod.rs` (~50 lines changed)

### Files to Modify (Minor Changes)

- `lithos-cli/src/commands/*.rs` (update to use Builder)
- Various test files (update to use Builder)

### Net Line Change Estimate

- **Lines Added**: ~2500
- **Lines Deleted**: ~2500
- **Net Change**: ~0 (refactor, not expansion)

---

## Appendix B: Testing Checklist

### Unit Tests

- [ ] PropertyBank state transitions (all paths)
- [ ] Schema state transitions (all paths)
- [ ] InheritanceGraph construction
- [ ] Delta computation
- [ ] Hash truncation functions
- [ ] Staleness detection logic

### Integration Tests

- [ ] Full pipeline (PropertyBank + Schema)
- [ ] Incremental updates (fresh schemas)
- [ ] PropertyBank-triggered schema updates
- [ ] Inheritance changes
- [ ] File deletions
- [ ] Multiple schema formats (TOML, JSON, YAML)

### Error Path Tests

- [ ] Cycle detection
- [ ] Missing parent
- [ ] Invalid $ref
- [ ] File not found
- [ ] Invalid TOML/JSON/YAML
- [ ] DB connection failure
- [ ] Serialization errors

### Performance Tests

- [ ] Full load (all stale)
- [ ] Incremental load (mixed)
- [ ] PropertyBank-only update
- [ ] Schema-only update
- [ ] Large schema count (100+)
- [ ] Deep inheritance (10 levels)

### End-to-End Tests

- [ ] CLI: load schemas from filesystem
- [ ] CLI: update single schema
- [ ] CLI: update property bank
- [ ] CLI: delete schema
- [ ] CLI: detect cycles
- [ ] CLI: handle errors gracefully

---

**END OF IMPLEMENTATION PLAN**
