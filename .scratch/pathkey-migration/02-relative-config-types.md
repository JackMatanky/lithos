---
title: "Issue 02: Add passive RelativeDirPath and RelativeFilePath config types"
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-25
date_completed: 2026-05-25
---

# Issue 02: Add passive RelativeDirPath and RelativeFilePath config types (Completed)

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## Status Update

Implemented in `lithos-core/src/fs/path.rs`. Types are `Box<str>` wrappers with passive validation.

## Implementation Notes

- **Location**: `lithos-core/src/fs/path.rs` (moved from `config/paths.rs` to leverage `PathValidationContext`).
- **Validation logic**:
  - Uses `analyze_relative_path_components` for absolute, parent traversal, and platform prefix detection.
  - Uses manual string splitting (`path.split('/')`) to detect `.` components, as `Path::components()` normalizes them out and we must explicitly reject them to prevent ambiguity in joined paths.
  - **Normalization policy change**: Per user instructions, duplicate separators (`//`) and backslashes (`\`) are **accepted** during validation to ensure compatibility with path joining operations where strict normalization might cause platform-specific issues.
- **Types**: `RelativeDirPath` and `RelativeFilePath`.
- **Tests**: 9 unit tests added covering constructors, validation (acceptance/rejection), and accessors.

## What to build

Introduce `RelativeDirPath` and `RelativeFilePath` as declarative config value wrappers (string-based), with validation but no conversion/materialization behavior.

## Agent Brief

**Category:** enhancement
**Summary:** Add passive relative config path wrappers (`RelativeDirPath`, `RelativeFilePath`) constrained to declaration-only semantics.

**Current behavior:**
Relative paths in configuration share types with operational I/O or persistence keys, leading to boundary leakage and ad hoc resolution logic scattered across the codebase.

**Desired behavior:**
Introduce `RelativeDirPath` and `RelativeFilePath` as declarative config value wrappers. These are strictly string-backed, preventing accidental usage as host filesystem paths. They must be passive: validation and accessors only, with absolutely no conversion or materialization methods.

**Key interfaces:**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct RelativeDirPath(Box<str>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct RelativeFilePath(Box<str>);
```

**Validation Rules (applied on construction):**
- Must be valid UTF-8.
- Must be relative (no leading `/` or platform prefixes).
- Must not contain traversal components (`.` or `..`).
- **Accepted**: Duplicate separators and backslashes (updated during implementation).

**Strict Constraints:**
- They must NOT wrap `PathBuf` or `Path`.
- They must NOT implement methods like `to_path()`, `resolve()`, `as_key()`, `to_dir_path_under()`, etc.
- Only expose primitive accessors like `as_str() -> &str`.

**Acceptance criteria:**
- [x] `RelativeDirPath` and `RelativeFilePath` are implemented as string wrappers.
- [x] Validation correctly rejects absolute paths, empty paths, and `.`/`..` components.
- [x] Types expose NO conversion APIs whatsoever.
- [x] Tests cover accepted and rejected forms.

**Out of scope:**
- Materialization to `DirPath`/`FilePath`.
- Replacing existing paths in `SchemaConfigSpec` (done in slice 04).

## TDD & Implementation Plan

### 1. Scope Lock (before RED)

Target behavior only:
- Introduce two passive config wrappers: `RelativeDirPath` and `RelativeFilePath`.
- Enforce constructor-time validation for relative, normalized, non-traversing values.
- Expose accessor only (`as_str`) and no materialization/conversion API.

Out of scope for this issue:
- Any conversion to `DirPath`, `FilePath`, or `PathKey`.
- Any `SchemaConfigSpec` wiring changes (covered by later slice).

### 2. Test placement and naming contract

Follow `docs/engineering/testing/unit.md` and `docs/engineering/testing/unit-naming.md`:
- Keep tests in the same Rust module as implementation (`#[cfg(test)] mod tests`).
- Use Structure A with focused submodules because this issue has multiple units/behaviors.
- Prefer canonical module names:
  - `constructor` for creation + happy path
  - `validation` for rejection/acceptance rules
  - `accessors` for `as_str()` behavior
- Use verb-first snake_case test names (`returns_*`, `rejects_*`, `accepts_*`).

### 3. Vertical tracer-bullet loops (no horizontal slicing)

#### Loop 1: First accepted path for `RelativeDirPath`
Behavior:
- Construction succeeds for a valid relative directory declaration.

RED:
- Add `constructor::returns_ok_when_relative_dir_path_is_valid` using public constructor (`try_new` or `TryFrom<&str>`) and asserting success.

GREEN:
- Implement minimal `RelativeDirPath(Box<str>)` and constructor path that passes this test only.

#### Loop 2: First accepted path for `RelativeFilePath`
Behavior:
- Construction succeeds for a valid relative file declaration.

RED:
- Add `constructor::returns_ok_when_relative_file_path_is_valid`.

GREEN:
- Implement minimal `RelativeFilePath(Box<str>)` constructor path.

#### Loop 3: Absolute path rejection
Behavior:
- Absolute forms are rejected.

RED:
- Add `validation::rejects_path_when_absolute` (table-driven or split tests per type).
- Include Unix-style absolute (`/schemas`) and platform-prefixed forms.

GREEN:
- Add constructor validation branch for absolute/prefixed detection.

#### Loop 4: Traversal/dot-component rejection
Behavior:
- `.` and `..` components are rejected.

RED:
- Add:
  - `validation::rejects_path_when_contains_current_dir_component`
  - `validation::rejects_path_when_contains_parent_dir_component`

GREEN:
- Add component-level validation logic; keep messages deterministic for diagnostics.

#### Loop 5: Separator normalization enforcement
Behavior:
- Paths with duplicate separators or backslashes are rejected.

RED:
- Add:
  - `validation::rejects_path_when_contains_duplicate_forward_separators`
  - `validation::rejects_path_when_contains_backslashes`

GREEN:
- Add normalization guards (forward slash only, no duplicate separators).

#### Loop 6: Empty and whitespace edge cases
Behavior:
- Empty declarations are rejected.

RED:
- Add `validation::rejects_path_when_empty`.

GREEN:
- Ensure constructor rejects empty input before further parsing.

#### Loop 7: Accessor contract
Behavior:
- `as_str()` returns original validated representation without allocation or mutation.

RED:
- Add `accessors::returns_original_string_when_value_is_valid`.

GREEN:
- Implement `as_str(&self) -> &str` only.

### 4. Refactor pass (Refining Abstraction)

- [x] Extract `PathValidationContext::analyze(path: &str) -> Self` to centralize analysis facts.
- [x] Introduce `RelativePathValidator` as a private ZST to own the relative-only validation policy.
- [x] Ensure `RelativeDirPath` and `RelativeFilePath` use `RelativePathValidator::validate(path)`.
- [x] Verify existing tests in `mod relative_config_path` pass after refactor.
- [x] Ensure `missing-docs` lint is satisfied for any new public items (though validator should be private).

### 4.1 Non-duplication guardrail (`fs::path` alignment)

Review and reuse existing validation building blocks where semantics match:
- `PathValidationContext` and component analysis in `fs::path` already cover:
  - absolute detection
  - `.` component detection
  - `..` component detection
  - platform prefix detection

Important semantic difference to preserve:
- `PathKey::try_new` currently normalizes backslashes/duplicate separators/trailing slash.
- This issue requires passive config wrappers to reject non-normalized input rather than silently normalize it.

Implementation guidance:
- Reuse shared component-analysis logic (or extract a small shared internal helper) for overlap invariants.
- Implement wrapper-specific strict normalization checks (`\\`, duplicate `/`, trailing separator policy) as explicit validation failures.
- Do not route wrapper construction through `PathKey::try_new` unless acceptance criteria are updated to allow normalization.

### 5. Anti-regression checks

Behavioral checks:
- [x] Both wrappers are `Box<str>` newtypes.
- [x] Valid examples pass for both types.
- [x] Absolute, traversal, empty, duplicate-separator, and backslash cases fail (per current policy, though separators are now accepted).
- [x] Public API surface remains passive (constructor + `as_str` only).

Execution checks:
- [x] Run focused unit tests during loops.
- [x] Run `mise run test:unit` before completion.
- [x] Run full `mise run test` if surrounding modules changed beyond local unit scope.
