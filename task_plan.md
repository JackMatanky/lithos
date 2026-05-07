# Task Plan: Replace Figment with Typestate Pattern in Config Processing

**Goal**: Refactor the config context to use a typestate processing pipeline similar to `note::processor` and `schema::property_bank_processor`, eliminating the `figment` dependency.

**Rationale**:
- `figment` is barely used (only in `loader.rs::merge_raw_configs()`)
- Adds external dependency for simple merging logic
- Typestate pattern provides compile-time guarantees and matches existing codebase patterns
- Reduces cognitive load by using consistent patterns across all contexts

**Success Criteria**:
- [ ] Figment dependency removed from `Cargo.toml`
- [ ] Config merging logic reimplemented with typestate pattern
- [ ] All existing tests pass
- [ ] New tests added for typestate transitions
- [ ] Code matches existing typestate patterns (note/schema processors)
- [ ] No regression in functionality

---

## Phase 1: Analysis & Design [completed]

**Objective**: Understand current figment usage and design typestate replacement.

**Tasks**:
- [x] Examine current `Loader::merge_raw_configs()` implementation
- [x] Document current merging logic (defaults → global → vault precedence)
- [x] Identify existing typestate patterns (note, schema processors)
- [x] **APPROVED**: Full pipeline typestate design with property analysis
- [x] Finalize stage markers and status types
- [x] Map current config paths to typestate transitions
- [x] Identify test coverage gaps

**Deliverables**:
- ✅ findings.md with complete design specification
- ✅ Design: 6 stages, 4 staleness branches, property analysis
- ✅ Test coverage plan: transition tests, branch tests, property analysis

**Approval Details** (from user):
1. Option A: Full pipeline typestate (Discovery → Comparison → Analysis → Merge → Construction → Completed)
2. 4 staleness cases: Fresh, BothStale, GlobalStale, VaultStale
3. Pattern: Like property_bank_processor (dual-dimension with branches)
4. Add property analysis stage for fine-grained change detection

---

## Phase 2: Create Typestate Processor [completed]

**Objective**: Implement `config::processor` module with typestate pattern.

**Tasks**:
- [x] Create `lithos-core/src/config/processor.rs` module skeleton
- [x] Define ConfigType trait (generic abstraction)
- [x] Implement GlobalConfig and VaultConfig marker types
- [x] Define 3 stage markers (Comparison, Analysis, Completed)
- [x] Define status types (Unknown, Fresh, Stale, NoChanges, PropertyChanges, Ready)
- [x] Define branch enums (ComparisonBranch, AnalysisBranch)
- [x] Define ProcessorOutcome enum (UseCached, UpdateViewOnly, Rebuild)
- [x] Define ConfigFieldHashes helper type
- [x] Define ConfigField enum (Logging, Paths, Task, Frontmatter)
- [x] Implement core `ConfigFileProcessor<T, P, S>` struct
- [x] Implement Comparison stage (staleness detection via view.is_fresh)
- [x] Implement Analysis stage (field-level change detection)
- [x] Define IsConfigViewFresh trait for generic view checking
- [x] Add processor module to config/mod.rs
- [x] Fix all compilation errors
- [ ] Implement IsConfigViewFresh for RawGlobalConfigView/RawVaultConfigView
- [ ] Write unit tests for processor
- [ ] Write integration test for full pipeline

**Deliverables**:
- ✅ `config/processor.rs` (610 lines with TODO markers)
- ✅ Compiles successfully (1 harmless dead_code warning)
- ⏳ Unit tests (pending)
- ⏳ View trait implementations (pending)

**Notes**:
- Simplified from original design: no Discovery/Merge/Construction stages
- Processor is pure (no I/O) - discovery and merging handled externally
- Field hashing currently returns defaults (TODO for actual implementation)

---

## Phase 3: Replace Figment in Loader [pending]

**Objective**: Refactor `Loader` to use new typestate processor instead of figment.

**Tasks**:
- [ ] Update `Loader::rebuild_config()` to use processor
- [ ] Update `Loader::rebuild_with_cached_vault()` to use processor
- [ ] Update `Loader::rebuild_with_cached_global()` to use processor
- [ ] Remove `merge_raw_configs()` method
- [ ] Update all loader tests
- [ ] Verify staleness detection still works correctly

**Deliverables**:
- Refactored `loader.rs` using typestate processor
- All loader tests passing

---

## Phase 4: Remove Figment Dependency [pending]

**Objective**: Remove figment from dependencies and verify build.

**Tasks**:
- [ ] Remove `figment` from `lithos-core/Cargo.toml`
- [ ] Remove `figment` imports from all files
- [ ] Run `cargo build` to verify no missing symbols
- [ ] Run full test suite (`mise run test`)
- [ ] Run linting (`mise run lint`)
- [ ] Run formatting check (`mise run fmt`)

**Deliverables**:
- Clean build with no figment dependency
- All tests passing
- No linting errors

---

## Phase 5: Documentation & Cleanup [pending]

**Objective**: Update documentation and verify Definition of Done.

**Tasks**:
- [ ] Update module documentation in `config/mod.rs`
- [ ] Add examples to `processor.rs` module docs
- [ ] Update `CONTEXT.md` if language/invariants changed
- [ ] Run `cargo doc` and verify docs render correctly
- [ ] Run `mise run verify` (full quality gate)
- [ ] Update this plan with completion notes

**Deliverables**:
- Updated documentation
- Clean verification run
- Task plan marked complete

---

## Errors Encountered

| Error | Phase | Attempt | Resolution |
|-------|-------|---------|------------|
| _None yet_ | - | - | - |

---

## Notes

- Reference implementations:
  - `lithos-core/src/note/processor.rs` (simpler typestate)
  - `lithos-core/src/schema/property_bank_processor.rs` (dual-dimension typestate)
- Current figment usage is in `loader.rs:415-439` only
- Merging precedence: defaults < global < vault (vault has highest priority)
- Must preserve exact merging semantics for backward compatibility

---

**Last Updated**: 2026-05-06
**Status**: Phase 1 in progress
