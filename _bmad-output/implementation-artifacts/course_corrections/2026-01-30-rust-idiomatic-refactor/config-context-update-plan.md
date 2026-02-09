# Config Context Update Plan

**Date**: 2026-02-09
**Status**: Phase 0.5 In Progress (2 of 8 tasks complete)
**Objective**: Align config implementation with design specs (001, 002, 003)

---

## Executive Summary

The config implementation has diverged from design specs in minor but important ways. This plan consolidates all review findings, verification results, and implementation tasks into a single actionable document.

**Test Status**: ✅ 100/100 config unit tests passing
**Quality Gate**: ✅ `mise run verify` passing (all pre-commit hooks)

---

## Completed Tasks

### ✅ Task 0.5.1: ConfigCommandError Extra Variant (RESOLVED)

**Status**: ✅ **COMPLETE** - Spec updated to document three-tier error taxonomy

**Rationale**:
- `ConfigIngestError` isolates adapter/boundary failures (Figment/TOML parsing)
- `ConfigError` handles domain validation (business rules)
- `ConfigCommandError` aggregates: Domain | Storage | Ingest

**Three-Tier Error Taxonomy**:
```
ConfigIngestError  → TOML parsing, Figment extraction (adapter)
ConfigError        → Empty paths, invalid enums (domain)
DbError           → Storage layer failures (infrastructure)
ConfigCommandError → Domain | Storage | Ingest (CQRS aggregate)
```

**Verification**:
- [x] Implementation in `error.rs` lines 108-128 matches updated spec
- [x] All 100 config tests pass
- [x] `mise run verify` passes

---

### ✅ Task 0.5.2: Bounds<T> Generic Type (COMPLETED)

**Status**: ✅ **COMPLETE** - Generic `Bounds<T>` created with validation logic

**Implementation** (`lithos-core/src/bounds.rs`):
- Generic `Bounds<T>` enum with Unbounded/Min/Max/Range variants
- `from_options()`, `validate()`, `min()`, `max()` methods
- Full serde serialization support
- 15 comprehensive unit tests

**Rkyv Integration Strategy**:
- `Bounds<T>` is designed for validation logic (no rkyv derives)
- Task.rs retains `IntegerBounds`/`FloatBounds` with rkyv for database storage
- Both types have identical shape for future interoperability
- Migration path: task.rs types can use `Bounds<T>` methods via conversion

**Verification**:
- [x] All 15 bounds tests pass
- [x] All 100 config tests pass
- [x] `mise run verify` passes

---

## Remaining Phase 0.5 Tasks

### Task 0.5.3: Type Inference for RawTaskFieldSpec (MAJOR - UX)

**Spec**: 003-config-task.md Section 4.1.5
**Issue**: Uses `#[serde(tag = "type")]`, spec requires `#[serde(untagged)]`

**Current** (`raw.rs` lines 188-232):
```rust
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RawTaskFieldSpec {
    Integer { keyword: String, min: Option<i64>, max: Option<i64> },
    // ...
}
```

**Required**:
```rust
#[serde(untagged)]
pub enum RawTaskFieldSpec {
    Enum { ... },      // Match first (has values array)
    Integer { ... },   // Match second (has i64 min/max)
    Float { ... },     // Match third (has f64 min/max)
    DateTime { ... },  // Match fourth (has format)
    String { ... },    // Fallback (no distinctive fields)
}
```

**Impact**: Users can write `[task.fields.priority] min = 0 max = 10` instead of requiring `type = "Integer"`

**Checklist**:
- [ ] Change to `#[serde(untagged)]`
- [ ] Reorder variants: Enum → Integer → Float → DateTime → String
- [ ] Add comment explaining untagged matching order
- [ ] Update deserialization tests
- [ ] **Breaking change**: Users must remove `type=` from config files
- [ ] Run `mise run verify`

---

### Task 0.5.4: Compiled Regex in TaskFieldSpec (PERFORMANCE)

**Spec**: 003-config-task.md Section 3.2 lines 497-506
**Issue**: Stores `Option<String>`, spec requires `Option<Arc<regex::Regex>>`

**Current** (`task.rs` lines 255-261):
```rust
String {
    keyword: TaskFieldKeyword,
    pattern: Option<String>,  // ← NOT compiled
}
```

**Problem**: Regex recompiles on every validation (lines 819-842)

**Required**:
```rust
String {
    keyword: TaskFieldKeyword,
    pattern: Option<Arc<regex::Regex>>,  // ← Pre-compiled
}
```

**Performance Impact**:
- Current: ~10-100µs per validation (regex compilation)
- With Arc<Regex>: ~1ns per validation (dereference)
- **100-10000x speedup** for repeated validations

**Checklist**:
- [ ] Change `pattern` type to `Option<Arc<regex::Regex>>`
- [ ] Compile regex in `TaskFieldSpec::from_raw()`
- [ ] Remove regex compilation from `validate_string()`
- [ ] Handle rkyv serialization (Arc<Regex> needs special handling)
- [ ] Run `mise run verify`

---

### Task 0.5.5: Make validate_raw_value Private (API)

**Spec**: 003-config-task.md Section 3.3 lines 719-723
**Issue**: Method is public, spec says private helper

**Current** (`task.rs` line 737):
```rust
pub fn validate_raw_value(&self, value: &serde_json::Value) -> Result<(), ConfigError>
```

**Required**:
```rust
pub(crate) fn validate_raw_value(&self, value: &serde_json::Value) -> Result<(), ConfigError>
```

**Rationale**: Note context should own FieldValue conversion, config provides internal validation only

**Checklist**:
- [ ] Change `pub fn` to `pub(crate) fn`
- [ ] Verify no external crate uses this method
- [ ] Run `mise run verify`

---

### Task 0.5.6: TaskFieldKeyword Box<str> (MEMORY)

**Spec**: 003-config-task.md Section 3.2 lines 362-387
**Issue**: Uses `String`, spec requires `Box<str>`

**Current** (`task.rs` lines 129-143):
```rust
pub struct TaskFieldKeyword(String);  // 24 bytes (ptr+len+cap)
```

**Required**:
```rust
pub struct TaskFieldKeyword(Box<str>);  // 16 bytes (ptr+len)
```

**Impact**: 33% stack size reduction (8 bytes per keyword, never mutated after construction)

**Checklist**:
- [ ] Change backing store to `Box<str>`
- [ ] Update `try_new` to convert to `Box<str>`
- [ ] Update `From<TaskFieldKeyword> for String`
- [ ] Run `mise run verify`

---

### Task 0.5.7: TaskTag Box<str> (MEMORY)

**Spec**: 003-config-task.md Section 3.2 lines 336-360
**Issue**: Uses `String`, spec requires `Box<str>`

Same as Task 0.5.6 but for `TaskTag` (`task.rs` lines 113-127)

**Checklist**:
- [ ] Change backing store to `Box<str>`
- [ ] Update `try_new` to convert to `Box<str>`
- [ ] Update `From<TaskTag> for String`
- [ ] Run `mise run verify`

---

### Task 0.5.8: RawTaskDates Extra Fields (DESIGN DECISION)

**Spec**: 003-config-task.md Section 3.2 lines 593-607
**Issue**: Implementation has 6 fields, spec shows 4

**Current** (`raw.rs` lines 159-174):
```rust
pub struct RawTaskDates {
    pub due: Option<RawDateFieldSpec>,
    pub scheduled: Option<RawDateFieldSpec>,  // ← EXTRA
    pub start: Option<RawDateFieldSpec>,      // ← EXTRA
    pub completed: Option<RawDateFieldSpec>,
    pub created: Option<RawDateFieldSpec>,
    pub reminder: Option<RawDateFieldSpec>,
}
```

**Spec** (4 fields):
- `due`, `created`, `reminder`, `completed`

**Options**:
1. **Remove** `scheduled` and `start` (match spec strictly)
2. **Keep** and update spec to document use case

**Decision Needed**: Are these intentional for Obsidian compatibility?

**Checklist**:
- [ ] Decision: Keep or remove?
- [ ] If removing: Delete fields, update tests
- [ ] If keeping: Update spec documentation
- [ ] Run `mise run verify`

---

## Task Priority Summary

| Task | Status | Severity | Effort | UX Impact |
|------|--------|----------|--------|-----------|
| 0.5.1 Error Variant | ✅ Complete | Medium | Low | None |
| 0.5.2 Bounds<T> | ✅ Complete | High | Medium | None |
| 0.5.3 Type Inference | ⏳ Pending | **High** | Low | **Major** |
| 0.5.4 Regex Compile | ⏳ Pending | **High** | Medium | None |
| 0.5.5 Private Method | ⏳ Pending | Low | Trivial | None |
| 0.5.6 Box<str> | ⏳ Pending | Low | Low | None |
| 0.5.7 Box<str> | ⏳ Pending | Low | Low | None |
| 0.5.8 Date Fields | ⏳ Pending | Medium | Low | Design Decision |

**Critical Path**: 0.5.3 → 0.5.4 → (0.5.5-0.5.8 in any order)

---

## Architecture Patterns

### Three-Shape Serialization Pattern
```
Raw* → TryFrom → Domain → [Stored*]
```

- **Raw types**: Dumb data (serde derives only)
- **TryFrom**: Explicit validation boundary
- **Domain types**: Validation-in-construction
- **Stored types**: Only when profiling shows need

### Error Taxonomy
- **ConfigIngestError**: Adapter boundary (Figment/TOML)
- **ConfigError**: Domain validation
- **DbError**: Storage layer
- **ConfigCommandError**: CQRS aggregate (Domain | Storage | Ingest)
- **ConfigQueryError**: CQRS query (Storage | Corruption)

### Figment Usage
- ✅ **Correct**: Figment merges within layers (file1 + file2 + env → RawGlobal)
- ✅ **Correct**: Domain merges across layers (Global + Vault → Config)
- ❌ **Not possible**: Figment cannot merge different schemas (Global vs Vault)

---

## Design Decisions to Confirm

### 1. Whole-Struct vs Field-Level Overrides

**Current**:
- `frontmatter: Option<Frontmatter>` (all-or-nothing replacement)
- `logging: Option<Logging>` (all-or-nothing)
- `task: Option<TaskConfig>` (all-or-nothing)

**Question**: Should frontmatter use field-level overrides like schema?
```rust
// Current (whole-struct)
vault.frontmatter.cloned().or_else(|| global.cloned())

// Alternative (field-level)
title_key: vault.title_key.clone().or_else(|| global.title_key.clone())
```

### 2. RawTaskDates Extra Fields

**Question**: Keep `scheduled` and `start` for Obsidian compatibility, or remove to match spec?

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

# Full CI simulation
mise run ci
```

---

## Notes

- **STOP** if `mise run verify` fails - fix before proceeding
- Run verification after EVERY task completion
- Commit checkpoint after each task with conventional commit format
- Update this document as tasks are completed
- Document any intentional spec deviations with rationale

---

*Document consolidated from: config-context-combined-status.md, config-design-review-findings.md, CRITICAL-REVIEW-CORRECTION.md, HONEST-COMPREHENSIVE-REVIEW.md, PHASE-0.5-VERIFIED.md*
*Consolidation date: 2026-02-09*
