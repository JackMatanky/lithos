# Config Context Status & Plan (Combined)

**Date**: 2026-02-08
**Status**: In Progress - Phase 0 & 1 Complete (Raw/Overrides Refactor)

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

4. **VaultId not implemented** (Design Section 3.2 "Vault identity")
   - **Required types**: `VaultId(Uuid)`, `VaultRoot(PathBuf)`, `VaultPathKey(Box<str>)`
   - **Current**: `Metadata` uses strings for vault path
   - **Impact**: Cannot support vault moves/renames

5. **Versioned Merged Config** (Design Section 3.2 "Versioned merged config read model")
   - **Required types**: `ConfigVersion(u64)`, `MergedConfigRecord`, `ActiveMergedConfig`
   - **Current**: No versioning, no rollback support
   - **Impact**: Cannot cache or rollback configs

6. **Split CQRS Errors** (CQRS Spec Section 1.4)
   - **Required**: `ConfigCommandError`, `ConfigQueryError`
   - **Current**: Single `ConfigError` type
   - **Impact**: Cannot distinguish storage vs domain errors

7. **Empty-string sentinels** (Design Section 3.2 "Remove empty-string sentinels")
   - **Location**: Merge logic in `aggregate.rs`
   - **Current**: Uses `choose_value` with empty-string checks
   - **Action**: Replace with `Option` overlays

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
- VaultId changes require storage migrations
- Versioned config adds complexity

**Low Risk**:
- Module-level lint suppressions are cosmetic
- Empty-string → Option refactor is mechanical

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

**CURRENT STATUS**: Phase 0 (Unblock Quality Gate) - MUST COMPLETE BEFORE PROCEEDING

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

## Phase 1: Raw Input Types + Ingest Boundary (Figment) - COMPLETED

Status: ✅ Files exist, ❌ Not fully validated against spec

- [x] Add `lithos-core/src/config/raw.rs` with `RawGlobal`, `RawVault`, and `RawTaskConfig` shapes.
- [x] Add `lithos-core/src/config/ingest.rs` with Figment provider wiring and `ingest_global`/`ingest_vault`.
- [x] Implement `TryFrom<Raw*>` -> validated domain types; keep Figment out of domain modules.
- [ ] **TODO**: Verify unit tests for raw deserialization and conversion errors (missing/invalid fields, unknown keys policy).
- [ ] **TODO**: Run `mise run test:unit:config`.
- [ ] **TODO**: Run `mise run verify` to ensure pre-commit hooks pass.

---

## Phase 2: Domain Type Refactor (Models + Newtypes) - PARTIALLY COMPLETE

Status: ✅ Most newtypes exist, ❌ Empty-string sentinels still in use, ❌ VaultId missing

- [ ] Replace empty-string sentinels with `Option<T>` overlays in vault overrides.
  - **Current**: `choose_value` in `aggregate.rs` uses empty-string checks
  - **Target**: `vault.or(global).unwrap_or(default)` pattern
- [x] Introduce newtypes per spec: `VaultPathKey`, `FrontmatterKey`, `LogLevel`, etc.
- [ ] **MISSING**: `VaultId`, `VaultRoot`, `VaultPathKey` (Design Section 3.2)
- [ ] Remove `SchemaVersion` deref; add `as_str()` and `Display`.
- [x] Convert `TrustedVaults` to `enum TrustedVaults { List, Map }` with `#[serde(untagged)]`.
- [ ] Update `Schema::property_bank_path()` to return `PathBuf` using join semantics.
  - **Current**: Returns `String` with string formatting
  - **Target**: `PathBuf::join` semantics
- [ ] Update config aggregate/build logic to use Option overlays (no empty-string checks).
- [ ] Update and add unit tests for newtypes, validation, and merge precedence.
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

---

## Phase 3: Task Config Schema (Cross-Cutting Infrastructure) - COMPLETED

Status: ✅ Fully implemented, matches design spec

- [x] Add `lithos-core/src/config/task.rs` with `TaskConfig`, `TaskTag`, `TaskFieldKeyword`, `StatusName`, `StatusSymbol`.
- [x] Implement `Bounds<T>`, `DateFieldSpec`, `TaskFieldSpec` with validation + regex compile + chrono format checks.
- [x] Add `TaskConfig::from_raw` and default config matching current checkbox behavior.
- [x] Add validation and parsing helpers (`field_spec`, `parse_date_value`, status mapping lookups).
- [x] Add unit tests for task tags, status mapping, bounds, regex, date parsing, and indexed fields.
- [ ] **TODO**: Run `mise run test:unit:config` (may need fixes after Phase 0).
- [ ] **TODO**: Run `mise run verify` to ensure pre-commit hooks pass.

---

## Phase 4: CQRS Refactor (Ports, Errors, Commands, Queries) - PARTIALLY COMPLETE

Status: ✅ Ports split, ❌ Split errors missing

- [x] Update `ports.rs` to split `ConfigCommandPort` and `ConfigQueryPort` with GATs.
- [ ] **MISSING**: Add `ConfigCommandError` and `ConfigQueryError` (structured storage/domain split).
  - **Current**: Single `ConfigError` type
  - **Target**: Per CQRS Spec Section 1.4
- [ ] Update `command.rs` and `query.rs` to be generic over ports and return split errors.
- [ ] Implement command-side `save_global`, `save_vault`, `load_global`, `load_vault`.
- [ ] Implement query-side `get(vault_id)` (merged read model only).
- [ ] Add unit tests for command/query behavior and error mapping.
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

---

## Phase 5: Versioned Merged Config Read Model - NOT STARTED

Status: ❌ No versioning types exist

- [ ] Add `ConfigVersion`, `MergedConfigRecord`, `ActiveMergedConfig` types.
- [ ] Implement `rebuild_merged`, `activate_version`, and optional `rollback` in command.
- [ ] Add DB table mapping: `vault_id_by_path`, `vault_path_by_id`, `merged_config_versions`, `merged_config_active`.
- [ ] Update adapters in `lithos-core/src/db/config_adapter.rs` if needed.
- [ ] Add tests for version creation, activation, and rollback behavior.
- [ ] Run `mise run test:unit:config` and `mise run test:unit:db` if adapter changes.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

---

## Phase 6: Aggregate Build and Merge Updates - PARTIALLY COMPLETE

Status: ✅ Build/merge logic exists, ❌ VaultId not used, ❌ Empty-string checks remain

- [ ] Update `Config::build` signature and metadata to use `VaultId` + `VaultRoot` (per spec).
  - **Current**: Uses string vault path
  - **Blocked by**: Phase 2 (VaultId types)
- [ ] Ensure merge precedence is explicit and deterministic (vault > global > defaults).
- [ ] Ensure config events remain valid and contain structured source when required.
- [ ] Update tests that assume fixed vault path or string-based metadata.
- [ ] Run `mise run test:unit:config`.
- [ ] Run `mise run verify` to ensure pre-commit hooks pass.

---

## Phase 7: Integration Touchpoints (Note/CLI) - NOT STARTED

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
Phase 1 (Raw Types) → Already done, verify tests
  ↓
Phase 2 (Domain Refactor) → VaultId + empty-string → Option
  ↓
Phase 4 (CQRS Errors) → Split error types
  ↓
Phase 5 (Versioned Config) → Rollback support
  ↓
Phase 6 (Merge Updates) → Use VaultId
  ↓
Phase 7 (Integration) → Wire to Note/CLI
  ↓
Phase 8 (Final Gate) → Ship it
```

**Phase 3 (Task Config)** is already complete and independent.

---

## Notes

- Pre-commit checks should be run frequently via `mise run verify` so hooks
  pass before each checkpoint commit.
- If any on-disk format changes are introduced, record migration notes or ADRs.
- **STOP and fix immediately** if `mise run verify` fails - do not proceed to next phase.
- See `config-context-gap-analysis.md` for detailed gap assessment vs design specs.
