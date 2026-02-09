# Config Context Update Plan

**Date**: 2026-02-09
**Status**: Phase 0.5 In Progress (2 of 8 tasks complete)
**Objective**: Align config implementation with design specs (001, 002, 003)

---

## Executive Summary

The config implementation has diverged from design specs in minor but important ways. This document consolidates the original implementation plan with comprehensive review findings and verification results.

**Test Status**: ✅ 100/100 config unit tests passing
**Quality Gate**: ✅ `mise run verify` passing (all pre-commit hooks)

---

## Part 1: Implementation Plan (Original)

### Phase 0: Unblock Quality Gate (CRITICAL)

See original plan in config-context-combined-status.md (preserved as historical record).

---

### Phase 0.5: Design Spec Alignment Issues

**Context**: Comprehensive review of implementation vs design specs revealed critical misalignments.

#### 0.5.1) ✅ COMPLETE: ConfigCommandError Extra Variant (RESOLVED)
**Spec**: 002-config-cqrs.md Section 1.4 (UPDATED)
**Status**: ✅ **RESOLVED** - Spec updated to document three-tier error taxonomy

**Rationale**:
- `ConfigIngestError` isolates Figment/adapter errors at boundary
- `ConfigError` for domain validation failures
- `ConfigCommandError` aggregates: Domain | Storage | Ingest

**Verification**:
- [x] Implementation in `error.rs` lines 108-128 matches updated spec
- [x] `From<ConfigIngestError>` impl correctly maps to Ingest variant
- [x] All 100 config tests pass
- [x] `mise run verify` passes

---

#### 0.5.2) ✅ COMPLETE: Bounds<T> Generic Type (COMPLETED)
**Spec**: 003-config-task.md Section 3.2 lines 434-475
**Status**: ✅ **COMPLETED** - Generic `Bounds<T>` created with validation logic

**Implementation** (`lithos-core/src/bounds.rs`):
- Generic `Bounds<T>` enum with Unbounded/Min/Max/Range variants
- `from_options()`, `validate()`, `min()`, `max()` methods
- Full serde serialization support
- 15 comprehensive unit tests

**Rkyv Integration Strategy**:
- `Bounds<T>` is designed for validation logic (no rkyv derives)
- Task.rs retains `IntegerBounds`/`FloatBounds` with rkyv for database storage
- Both types have identical shape for future interoperability

**Verification**:
- [x] All 15 bounds tests pass
- [x] All 100 config tests pass
- [x] `mise run verify` passes

---

#### 0.5.3) MAJOR: Type Inference for RawTaskFieldSpec (PENDING)
**Spec**: 003-config-task.md Section 4.1.5
**Issue**: Uses `#[serde(tag = "type")]`, spec requires `#[serde(untagged)]`

**Current**: Tagged enum requiring explicit `type="Integer"`
**Required**: Untagged enum with type inferred from structure
**Impact**: Major UX improvement - users can write `min = 0` instead of `type = "Integer"`

---

#### 0.5.4) MEDIUM: Compiled Regex in TaskFieldSpec (PENDING)
**Spec**: 003-config-task.md Section 3.2
**Issue**: Stores `Option<String>`, spec requires `Option<Arc<regex::Regex>>`

**Current**: Recompiles regex on every validation
**Required**: Pre-compile regex at config load
**Impact**: 100-10000x speedup for repeated validations

---

#### 0.5.5) MEDIUM: Make validate_raw_value Private (PENDING)
**Spec**: 003-config-task.md Section 3.3
**Issue**: Method is public, spec says private helper
**Fix**: Change `pub fn` to `pub(crate) fn`

---

#### 0.5.6) MEDIUM: TaskFieldKeyword Box<str> (PENDING)
**Spec**: 003-config-task.md Section 3.2
**Issue**: Uses `String`, spec requires `Box<str>`
**Impact**: 33% stack size reduction (never mutated after construction)

---

#### 0.5.7) MEDIUM: TaskTag Box<str> (PENDING)
Same as 0.5.6 but for `TaskTag`

---

#### 0.5.8) MEDIUM: RawTaskDates Extra Fields (PENDING - DECISION NEEDED)
**Spec**: 4 fields (`due`, `created`, `reminder`, `completed`)
**Current**: 6 fields (adds `scheduled`, `start`)
**Decision**: Keep for Obsidian compatibility or remove to match spec?

---

## Part 2: Review Findings

### Comprehensive Review Summary

**Date**: 2026-02-09
**Files Reviewed**: All 15 files in `lithos-core/src/config/`

#### What Was Done RIGHT

1. **Clean Raw Types Pattern** (`raw.rs`)
2. **Option Overlay Merge** (`aggregate.rs`)
3. **LogLevel IS an Enum** (`logging.rs`)
4. **Path Types Use PathBuf** (`paths.rs`)
5. **CQRS Implementation** (`command.rs`, `query.rs`, `ports.rs`)
6. **VaultId Stable Identity** (`vault.rs`)
7. **ConfigVersion Monotonic** (`aggregate.rs`)
8. **Figment Properly Isolated** (`ingest.rs`)

#### Critical Issues Identified

**Issue #1: Whole-Struct vs Field-Level Overrides**
- Frontmatter/logging use whole-struct replacement
- Schema/template use field-level overrides
- **Question**: Intentional design or gap?

**Issue #2: Figment Layering**
- Figment merges within layers (correct)
- Domain merges across layers (correct)
- Spec needs clarification

#### Figment Usage Analysis

**Verdict**: ✅ **Already optimal**

All Figment best practices validated:
- ✅ `Serialized::defaults` for programmatic defaults
- ✅ `merge` for overrides
- ✅ No `#[serde(flatten)]`
- ✅ Handle missing files gracefully
- ✅ Extract into Raw types

---

## Part 3: Verification Results

### Phase 0.5 Issues Verification

**Status**: ✅ **ALL 8 CLAIMS VERIFIED**

| Issue | Status | Severity |
|-------|--------|----------|
| 0.5.1 Error Variant | ✅ RESOLVED | Medium |
| 0.5.2 Bounds<T> | ✅ COMPLETE | High |
| 0.5.3 Type Inference | ⏳ PENDING | **High** |
| 0.5.4 Regex Compile | ⏳ PENDING | **High** |
| 0.5.5 Public Method | ⏳ PENDING | Low |
| 0.5.6 Box<str> | ⏳ PENDING | Low |
| 0.5.7 Box<str> | ⏳ PENDING | Low |
| 0.5.8 Extra Fields | ⏳ PENDING | Medium |

**Critical Path**: 0.5.3 → 0.5.4 → (0.5.5-0.5.8 in any order)

---

## Part 4: Action Items Summary

### Immediate (Critical Path)

1. **Task 0.5.3**: Type inference with `#[serde(untagged)]` (UX)
2. **Task 0.5.4**: Compiled regex with `Arc<Regex>` (performance)
3. **Task 0.5.8**: Decision on extra date fields (design)

### Can Defer

- 0.5.5: Private method
- 0.5.6/0.5.7: Box<str> optimizations

### Design Decisions Needed

1. **Whole-struct vs field-level overrides**: Intentional?
2. **Extra date fields**: Keep for Obsidian compatibility?

---

## Commands Reference

```bash
# Run all quality gates
mise run verify

# Run config tests only
mise run test:unit:config

# Check clippy
mise run lint

# Format code
mise run fmt
```

---

## Historical Documents Preserved

This consolidated plan references the following preserved documents:

1. **config-context-combined-status.md** - Original implementation plan (Phases 0-8)
2. **config-design-review-findings.md** - Figment usage analysis and design spec review
3. **CRITICAL-REVIEW-CORRECTION.md** - Correction of my initial assessment errors
4. **HONEST-COMPREHENSIVE-REVIEW.md** - Thorough review of all 15 config files
5. **PHASE-0.5-VERIFIED.md** - Detailed verification of all 8 Phase 0.5 issues

These documents are preserved as a record of the review process and findings.

---

*Document created: 2026-02-09*
*Consolidates: Implementation plan + Review findings + Verification results*
