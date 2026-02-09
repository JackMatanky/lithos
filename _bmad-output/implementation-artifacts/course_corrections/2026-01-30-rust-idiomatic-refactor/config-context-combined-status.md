# Config Context Status & Plan (Combined)

**Date**: 2026-02-08
**Status**: In Progress - Phase 0-6 Complete (Core & Persistence Logic Implemented)

This document combines the Gap Analysis and Implementation Plan for the Config Context refactoring.

---

# Part 1: Gap Analysis

## Executive Summary

The config implementation has diverged from the design specs. Key gaps:
1. **Legacy type exists**: `SettingValue` should not exist per design spec
2. **Clippy failures blocking progress**: Pattern matching issues with `ref` patterns
3. **Missing module-level lint suppressions**: rkyv-generated types trigger exhaustive lint

## Current State vs Design Spec

### ✅ Completed (Matches Design)

1. **File Structure** - All required files exist:
   - `config/raw.rs` - Raw input types
   - `config/ingest.rs` - Figment boundary
   - `config/task.rs` - Task configuration
   - `config/ports.rs`, `command.rs`, `query.rs` - CQRS
   - `config/types.rs` - Shared types
   - `config/aggregate.rs`, `global.rs`, `vault.rs` - Domain models

2. **TrustedVaults Enum** - Implemented as `enum { List, Map }`
3. **TaskConfig** - Fully implemented with all specs
4. **Newtypes** - Most validators in place:
   - `TaskTag`, `TaskFieldKeyword`, `StatusName`, `StatusSymbol`
   - `FrontmatterKey`, `LogLevel`, `SchemasDir`, `TemplatesDir`
5. **CQRS Ports** - Split into `CommandPort` and `QueryPort`
6. **Split Errors** - `ConfigCommandError` and `ConfigQueryError` implemented
7. **Versioning** - `ConfigVersion` and `merged_config_versions` persistence implemented
8. **Vault Identity** - `VaultId` types exist and are used in persistence

### ❌ Gaps (Not Yet Implemented)

#### **Critical - Blocks Quality Gate**

1. **`SettingValue` exists but shouldn't** (Design Section 1.1)
   - **Location**: `lithos-core/src/config/types.rs:631`
   - **Why it's wrong**: Design explicitly states "no universal value type"
   - **Action**: Remove `SettingValue` enum and all usages
   - **Impact**: May break tests/code that depends on it

2. **Clippy failures** (ref patterns, pattern_type_mismatch)
   - **Location**: `global.rs`, `task.rs`, `types.rs`
   - **Why it's blocking**: Cannot pass `pre-commit run --all-files`
   - **Action**: Fix pattern matching to use `&` patterns instead of `ref`
   - **Status**: Partially fixed but syntax errors introduced

3. **Module-level lint suppressions**
   - **Location**: `types.rs` - needs `#![expect(clippy::exhaustive_enums)]` for rkyv
   - **Why it's failing**: rkyv derives trigger exhaustive lints
   - **Action**: Add module-level expect with clear reason
   - **Status**: Attempted but unfulfilled expectation errors

#### **Important - Missing from Design Spec**

None remaining. Previously identified gaps (VaultId, Versioning, Split Errors) have been verified as implemented.

#### **Nice to Have - Future Work**

8. **Schema property_bank_path** returns String not PathBuf
9. **SchemaVersion** implements Deref (design says use `as_str()`)
10. **Validation-in-types pattern** not fully applied

## Immediate Action Plan

### Phase 0: Unblock Quality Gate (THIS FIRST)

**Objective**: Get `pre-commit run --all-files` passing

1. **Fix clippy pattern errors** (30 min)
   - Remove `ref` patterns, use `&` patterns correctly
   - Fix syntax errors from previous attempts
   - Test: `cargo clippy --all-targets`

2. **Add module-level lint suppressions** (10 min)
   - Add `#![expect(clippy::exhaustive_enums)]` to `types.rs`
   - Add reason: "rkyv::Archive derive generates exhaustive archived enums"
   - Test: `cargo clippy --all-targets`

3. **Remove SettingValue** (1 hour)
   - Search for all usages: `rg "SettingValue" lithos-core/`
   - Delete enum definition
   - Update/remove tests that use it
   - Test: `mise run test:unit:config`

4. **Run full verification** (5 min)
   - `mise run verify`
   - Fix any remaining issues
   - Commit checkpoint

### Phase 1: Design Alignment (After Quality Gate Passes)

Follow the updated implementation plan (see below).

## Risk Assessment

**High Risk**:
- Removing `SettingValue` may break downstream code (unknown usage)
- Pattern matching fixes may introduce regressions

**Medium Risk**:
- Integration with Note/CLI might expose missing API surface

**Low Risk**:
- Module-level lint suppressions are cosmetic

## Recommendations

1. **Stop all new features** until quality gate passes
2. **Remove SettingValue immediately** - it contradicts design
3. **Fix clippy before anything else** - blocks commits
4. **Follow implementation plan phase-by-phase** - no shortcuts
5. **Run `mise run verify` after every phase** - frequent checkpoints

---

# Part 2: Implementation Plan

Purpose: Track tasks required to implement the config design specs in
`docs/design/001-config-models.md`, `docs/design/002-config-cqrs.md`, and
`docs/design/003-config-task.md`, with test updates and frequent pre-commit
checks.

**CURRENT STATUS**: Phase 0-6 Complete. Focus is on Phase 0 (cleanup) and Phase 7 (Integration).

Conventions:
- Each task includes explicit test updates and a pre-commit check.
- Use `mise run <task>` for tooling consistency.
- Pre-commit hooks must pass at each checkpoint (run via `mise run verify`).
- **CRITICAL**: Run `mise run verify` after EVERY checkbox - no exceptions

---

## Phase 0: Unblock Quality Gate (IN PROGRESS - CRITICAL)

**Objective**: Get `pre-commit run --all-files` passing with zero errors/warnings

**Context**: Implementation has diverged from design. Must clean up before proceeding.
See `config-context-gap-analysis.md` for detailed gap assessment.

### 0.1) Fix Clippy Pattern Errors
- [ ] Read `lithos-core/src/config/global.rs` lines 295-325 (TrustedVaults::validate)
- [ ] Read `lithos-core/src/config/task.rs` lines 647-705 (TaskFieldSpec methods)
- [ ] Read `lithos-core/src/config/types.rs` lines 702-730 (SettingValue Debug impl)
- [ ] Fix all `ref` pattern errors by using `&` patterns correctly
- [ ] Remove duplicate closing braces from previous broken edits
- [ ] Run `cargo clippy --all-targets` until zero errors
- [ ] Run `mise run verify` (REQUIRED)

### 0.2) Add Module-Level Lint Suppressions
- [ ] Add `#![expect(clippy::exhaustive_enums, reason = "rkyv::Archive derive generates exhaustive archived enums")]` to `types.rs`
- [ ] Add `#![expect(clippy::struct_field_names, reason = "Frontmatter fields share '_key' suffix by design (flagged by rkyv::Archive)")]` to `types.rs`
- [ ] Verify no unfulfilled lint expectations
- [ ] Run `cargo clippy --all-targets` until zero errors
- [ ] Run `mise run verify` (REQUIRED)

### 0.3) Remove SettingValue (Design Violation)
- [ ] Search all usages: `rg "SettingValue" lithos-core/src/config/`
- [ ] **CRITICAL**: `SettingValue` violates design spec Section 1.1 - must be removed
- [ ] Delete `pub enum SettingValue` from `types.rs` (line ~631)
- [ ] Delete `impl std::fmt::Debug for SettingValue` from `types.rs` (line ~702)
- [ ] Delete `impl From<bool>` and other From impls for SettingValue
- [ ] Remove SettingValue from module exports in `types.rs`
- [ ] Update/delete any tests that reference SettingValue
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.4) Final Quality Gate Checkpoint
- [ ] Run `mise run verify` (full check: fmt + lint + tests + adr)
- [ ] Confirm zero clippy warnings
- [ ] Confirm all tests pass
- [ ] Stage all changes: `git add lithos-core/src/config/`
- [ ] Commit checkpoint: `git commit -m "fix(config): remove SettingValue, fix clippy patterns, add lint suppressions"`

**STOP**: Do not proceed to Phase 1 until Phase 0 is 100% complete and committed.

---

## Phase 0.5: Design Spec Alignment Issues (FROM DESIGN REVIEW 2026-02-09)

**Context**: Comprehensive review of implementation vs design specs revealed critical misalignments.
**Source**: Design specs 001-config-models.md, 002-config-cqrs.md, 003-config-task.md

### 0.5.1) CRITICAL: Fix ConfigCommandError Extra Variant
**Spec**: 002-config-cqrs.md Section 1.4
**Issue**: `ConfigCommandError` has `Ingest` variant not in spec (only Domain/Storage allowed)
- [ ] Read `lithos-core/src/config/error.rs` lines 108-128
- [ ] **Decision needed**: Remove `Ingest` variant OR update design spec to justify it
- [ ] If removing: Map ingest errors to `Domain(ConfigError)` in command.rs
- [ ] If keeping: Document rationale and update spec
- [ ] Update tests in `error.rs` if error variants change
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.2) CRITICAL: Consolidate Bounds Types (DRY Violation)
**Spec**: 003-config-task.md Section 3.2 lines 434-475, Decision 4.1.8
**Issue**: Implementation has separate `IntegerBounds`/`FloatBounds`, spec requires generic `Bounds<T>`
- [ ] Read `lithos-core/src/config/task.rs` lines 145-203 (current separate types)
- [ ] Create generic `Bounds<T>` enum with Unbounded/Min/Max/Range variants
- [ ] Replace `IntegerBounds` with `Bounds<i64>`
- [ ] Replace `FloatBounds` with `Bounds<f64>`
- [ ] Update `TaskFieldSpec::Integer` and `TaskFieldSpec::Float` to use `Bounds<T>`
- [ ] Update validation methods to use shared `bounds.validate(value)` logic
- [ ] Update all tests referencing `IntegerBounds` or `FloatBounds`
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.3) MAJOR: Implement Type Inference for RawTaskFieldSpec
**Spec**: 003-config-task.md Section 4.1.5 Decision, Section 2.1 lines 117-140
**Issue**: Uses `#[serde(tag = "type")]` requiring explicit `type=` key, spec requires `#[serde(untagged)]`
- [ ] Read `lithos-core/src/config/raw.rs` lines 188-232
- [ ] Change `#[serde(tag = "type", rename_all = "lowercase")]` to `#[serde(untagged)]`
- [ ] Reorder enum variants for correct matching priority: Enum → Integer → Float → DateTime → String
- [ ] Add comment explaining untagged matching order
- [ ] Update deserialization tests to verify type inference works (no `type=` key needed)
- [ ] **Breaking change**: Users must remove `type=` from config files
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.4) MEDIUM: Store Compiled Regex in TaskFieldSpec
**Spec**: 003-config-task.md Section 3.2 lines 497-506, Decision 4.1.3
**Issue**: Stores `Option<String>`, spec requires `Option<Arc<regex::Regex>>` for performance
- [ ] Read `lithos-core/src/config/task.rs` lines 240-276 (TaskFieldSpec enum)
- [ ] Change `String { pattern: Option<String> }` to `String { pattern: Option<Arc<regex::Regex>> }`
- [ ] Update `TaskFieldSpec::from_raw` to compile regex and wrap in Arc
- [ ] Remove regex compilation from `validate_string` method (use pre-compiled)
- [ ] Update rkyv derives (Arc<Regex> needs manual serialization or skip)
- [ ] **Consider**: Store both compiled Arc<Regex> and pattern string for serialization
- [ ] Update validation tests to verify compiled regex is used
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.5) MEDIUM: Make validate_raw_value Private
**Spec**: 003-config-task.md Section 3.3 lines 719-723 (marked as "private helper")
**Issue**: Method is public, spec explicitly says private
- [ ] Read `lithos-core/src/config/task.rs` line 737 (`pub fn validate_raw_value`)
- [ ] Change `pub fn validate_raw_value` to `pub(crate) fn validate_raw_value`
- [ ] **Rationale**: Only Note context should validate; users shouldn't call directly
- [ ] Verify no external crate uses this method
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.6) MEDIUM: Change TaskFieldKeyword Backing to Box<str>
**Spec**: 003-config-task.md Section 3.2 lines 362-387
**Issue**: Uses `String`, spec requires `Box<str>` (immutable, no growth capacity)
- [ ] Read `lithos-core/src/config/task.rs` lines 129-143
- [ ] Change `pub struct TaskFieldKeyword(String)` to `pub struct TaskFieldKeyword(Box<str>)`
- [ ] Update `try_new` to convert `value.into(): String` to `value.into(): Box<str>`
- [ ] Update `From<TaskFieldKeyword> for String` to use `.into_string()` or clone
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.7) MEDIUM: Change TaskTag Backing to Box<str>
**Spec**: 003-config-task.md Section 3.2 lines 336-360
**Issue**: Uses `String`, spec requires `Box<str>` (same as TaskFieldKeyword)
- [ ] Read `lithos-core/src/config/task.rs` lines 113-127
- [ ] Change `pub struct TaskTag(String)` to `pub struct TaskTag(Box<str>)`
- [ ] Update `try_new` to convert to `Box<str>`
- [ ] Update `From<TaskTag> for String` implementation
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.8) MEDIUM: Document or Remove Extra RawTaskDates Fields
**Spec**: 003-config-task.md Section 3.2 lines 593-607 (only 4 fields: due/created/reminder/completed)
**Issue**: Implementation adds `scheduled` and `start` fields not in spec
- [ ] Read `lithos-core/src/config/raw.rs` lines 159-174
- [ ] **Decision needed**: Keep extra fields (update spec) OR remove them (match spec)
- [ ] If keeping: Add to spec documentation and explain use case
- [ ] If removing: Delete `scheduled` and `start` fields
- [ ] Update tests if fields are removed
- [ ] Run `mise run test:unit:config`
- [ ] Run `mise run verify` (REQUIRED)

### 0.5.9) Quality Gate for Phase 0.5
- [ ] Run `mise run verify` (full quality gate)
- [ ] Confirm all spec alignment issues resolved
- [ ] Document any deviations from spec with rationale
- [ ] Stage changes: `git add lithos-core/src/config/`
- [ ] Commit checkpoint: `git commit -m "fix(config): align implementation with design specs (bounds, errors, task fields)"`

**STOP**: Do not proceed to Phase 7 until Phase 0.5 is complete and all design alignment issues are resolved.

---

## Phase 1: Raw Input Types + Ingest Boundary (Figment) - COMPLETED

Status: ✅ Files exist, ❌ Not fully validated against spec

- [x] Add `lithos-core/src/config/raw.rs` with `RawGlobal`, `RawVault`, and `RawTaskConfig` shapes.
- [x] Add `lithos-core/src/config/ingest.rs` with Figment provider wiring and `ingest_global`/`ingest_vault`.
- [x] Implement `TryFrom<Raw*>` -> validated domain types; keep Figment out of domain modules.
- [x] **Refactored**: Removed redundant `RawSchemaPaths` in favor of validated overrides (2026-02-08).

---

## Phase 2: Domain Type Refactor (Models + Newtypes) - COMPLETED

Status: ✅ Core types exist and are used in persistence/aggregate.

- [x] Replace empty-string sentinels with `Option<T>` overlays in vault overrides.
- [x] Introduce newtypes per spec: `VaultPathKey`, `FrontmatterKey`, `LogLevel`, etc.
- [x] `VaultId`, `VaultRoot`, `VaultPathKey` exist.
- [ ] Remove `SchemaVersion` deref; add `as_str()` and `Display`.
- [x] Convert `TrustedVaults` to `enum TrustedVaults { List, Map }` with `#[serde(untagged)]`.
- [ ] Update `Schema::property_bank_path()` to return `PathBuf` using join semantics.

---

## Phase 3: Task Config Schema (Cross-Cutting Infrastructure) - COMPLETED

Status: ✅ Fully implemented, matches design spec

- [x] Add `lithos-core/src/config/task.rs` with `TaskConfig`, `TaskTag`, `TaskFieldKeyword`, `StatusName`, `StatusSymbol`.
- [x] Implement `Bounds<T>`, `DateFieldSpec`, `TaskFieldSpec` with validation + regex compile + chrono format checks.
- [x] Add `TaskConfig::from_raw` and default config matching current checkbox behavior.
- [x] Add validation and parsing helpers (`field_spec`, `parse_date_value`, status mapping lookups).
- [x] Add unit tests for task tags, status mapping, bounds, regex, date parsing, and indexed fields.

---

## Phase 4: CQRS Refactor (Ports, Errors, Commands, Queries) - COMPLETED

Status: ✅ Ports split, Errors split, Implementation exists.

- [x] Update `ports.rs` to split `ConfigCommandPort` and `ConfigQueryPort` with GATs.
- [x] Add `ConfigCommandError` and `ConfigQueryError` (structured storage/domain split).
- [x] Update `command.rs` and `query.rs` to be generic over ports and return split errors.
- [x] Implement command-side `save_global`, `save_vault`, `load_global`, `load_vault`.
- [x] Implement query-side `get(vault_id)` (merged read model only).
- [x] Add unit tests for command/query behavior and error mapping.

---

## Phase 5: Versioned Merged Config Read Model - COMPLETED

Status: ✅ Versioning types and persistence logic implemented.

- [x] Add `ConfigVersion`, `MergedConfigRecord`, `ActiveMergedConfig` types.
- [x] Implement `rebuild_merged`, `activate_version`, and optional `rollback` in command.
- [x] Add DB table mapping: `vault_id_by_path`, `vault_path_by_id`, `merged_config_versions`, `merged_config_active`.
- [x] Update adapters in `lithos-core/src/db/config_adapter.rs` if needed.
- [x] Add tests for version creation, activation, and rollback behavior.

---

## Phase 6: Aggregate Build and Merge Updates - COMPLETED

Status: ✅ Build logic uses `VaultId` and proper merge precedence.

- [x] Update `Config::build` signature and metadata to use `VaultId` + `VaultRoot` (per spec).
- [x] Ensure merge precedence is explicit and deterministic (vault > global > defaults).
- [x] Ensure config events remain valid and contain structured source when required.
- [x] Update tests that assume fixed vault path or string-based metadata.

---

## Phase 7: Integration Touchpoints (Note/CLI) - IN PROGRESS

- [x] **Core API Integration**: Validated end-to-end CQRS flow with Redb integration test (`lithos-core/tests/config_integration.rs`).
- [ ] Wire TaskConfig into config loading and note parsing interfaces (no context cross-imports).
- [ ] Update any CLI or adapter boundaries that depend on old config APIs.
- [ ] Add integration tests (if applicable) under `lithos-core/tests/`.
- [ ] Run `mise run test:integration` (if integration tests changed).
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

---

## Phase 8: Final Quality Gate and Checkpoint Commit - NOT REACHED

- [ ] Run `mise run verify` (full pre-commit hooks + tests + adr checks).
- [ ] Confirm all updated tests pass and no clippy warnings.
- [ ] Stage and commit checkpoint with a concise message (if requested).

---

## Critical Path

```
Phase 0 (Quality Gate) → MUST COMPLETE FIRST
  ↓
Phase 7 (Integration) → Wire to Note/CLI
  ↓
Phase 8 (Final Gate) → Ship it
```

**Phases 1-6 are already complete (Core Library Logic).**

---

## Notes

- Pre-commit checks should be run frequently via `mise run verify` so hooks
  pass before each checkpoint commit.
- If any on-disk format changes are introduced, record migration notes or ADRs.
- **STOP and fix immediately** if `mise run verify` fails - do not proceed to next phase.
- See `config-context-gap-analysis.md` for detailed gap assessment vs design specs.

## Architectural Patterns

**Three-Shape Serialization Pattern**: The config context implements the canonical **Raw* → TryFrom → Domain → [Stored*]** pattern for parsing, validation, and optional storage optimization. See [`_bmad-output/planning-artifacts/architecture/implementation-patterns-consistency-rules.md`](../../../_bmad-output/planning-artifacts/architecture/implementation-patterns-consistency-rules.md#three-shape-serialization-pattern) for:
- Rationale for zero-method Raw types (dumb data)
- TryFrom as explicit validation boundary
- When to create Stored* types (rarely, profiling only)
- Testing approaches and anti-patterns

This pattern is **canonical** and codified in the project architecture document. All external input validation must follow this three-layer approach.
