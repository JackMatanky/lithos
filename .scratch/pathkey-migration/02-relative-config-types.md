---
title: "Issue 02: Add passive RelativeDirPath and RelativeFilePath config types"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 02: Add passive RelativeDirPath and RelativeFilePath config types

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

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
- Must have normalized separators (forward slashes only, no duplicates).

**Strict Constraints:**
- They must NOT wrap `PathBuf` or `Path`.
- They must NOT implement methods like `to_path()`, `resolve()`, `as_key()`, `to_dir_path_under()`, etc.
- Only expose primitive accessors like `as_str() -> &str`.

**Acceptance criteria:**
- [ ] `RelativeDirPath` and `RelativeFilePath` are implemented as string wrappers.
- [ ] Validation correctly rejects absolute paths, empty paths, and `.`/`..` components.
- [ ] Types expose NO conversion APIs whatsoever.
- [ ] Tests cover accepted and rejected forms.

**Out of scope:**
- Materialization to `DirPath`/`FilePath`.
- Replacing existing paths in `SchemaConfigSpec` (done in slice 04).

## TDD & Implementation Plan

### 1. Planning & Design
**Deep Modules / Testability:**
- `RelativeDirPath` and `RelativeFilePath` encode the "unresolved, relative config state" using the Type State Pattern.
- They must expose *zero* conversion or materialization methods.

**Behaviors to Test (Prioritized):**
1. Configuration loader accepts valid relative paths as declarative wrappers.
2. Configuration loader rejects absolute or traversal paths.

### 2. Tracer Bullet: Valid Declaration
**Behavior:** Configuration loader accepts valid relative paths as declarative wrappers.
- **RED:** Write `test_valid_relative_dir` asserting `RelativeDirPath::try_new("config/dir")` succeeds.
- **GREEN:** Implement `RelativeDirPath(Box<str>)` with minimal validation (UTF-8, no leading `/`, no `.`/`..`).
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added

### 3. Incremental Loop: Invariant Rejection
**Behavior:** Configuration loader rejects absolute or traversal paths.
- **RED:** Write `test_reject_absolute` (`/config`) and `test_reject_traversal` (`../config`).
- **GREEN:** Expand validation logic.
**Checklist:**
- [x] Test describes behavior, not implementation
- [x] Test uses public interface only
- [x] Test would survive internal refactor
- [x] Code is minimal for this test
- [x] No speculative features added

### 4. Refactor
- [ ] Verify *no* conversion methods (`to_path`, etc.) are implemented (Rust Best Practice: Type State Pattern).
- [ ] Ensure `Box<str>` is used instead of `PathBuf` for optimal memory layout.
