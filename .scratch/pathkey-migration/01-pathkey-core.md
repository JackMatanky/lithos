---
title: "Issue 01: PathKey core type and normalization pipeline"
category: enhancement
label: ready-for-agent
status: completed
date_created: 2026-05-25
date_completed: 2026-05-25
---

# Issue 01: PathKey core type and normalization pipeline

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Implement `PathKey` as the canonical persistence-key type by renaming `NormalizedPath` and formalizing the `trim -> normalize -> validate` pipeline with root-scoped conversion errors.

## Agent Brief

**Category:** enhancement
**Summary:** Establish `PathKey` as canonical repository key type with strict normalization and root-scoped conversion errors.

**Current behavior:**
`NormalizedPath` exists but lacks canonical key semantics, complete normalization guarantees (duplicate/trailing separators), and robust root-scoped conversion error coverage.

**Desired behavior:**
Rename `NormalizedPath` → `PathKey` as the persistence-key primitive. Keep `PathKey` as `Box<str>` internally for compact immutable ownership. Make all filesystem→key conversions root-scoped and fallible via `PathKey::from_rooted_path(root, path)` and convenience `as_key(root)` methods.

**Key interfaces:**

1. **Core Type:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct PathKey(Box<str>);
```

2. **Constructors & Normalization:**
The implementation must follow the **"parse, don't validate"** principle.
```rust
impl PathKey {
    pub fn try_new(path: &str) -> Result<Self, PathError> {
        let trimmed = Self::trim(path);
        let normalized = Self::normalize(trimmed);
        Self::validate(normalized.as_ref())?;
        Ok(Self(normalized.into_owned().into_boxed_str()))
    }

    // - Converts `\` to `/`
    // - Deduplicates `//`
    // - Removes trailing `/`
    // - Preserves leading `/` for absolute path detection
    // - Returns Cow::Borrowed if already canonical
    fn normalize(path: &str) -> Cow<'_, str> { /* ... */ }

    // Checks for empty, absolute (leading `/`), `..`, `.`, platform prefixes
    fn validate(path: &str) -> Result<(), PathError> { /* ... */ }
}
```

3. **Root-Scoped Conversions:**
```rust
impl PathKey {
    pub fn from_rooted_path(root: &DirPath, path: &Path) -> Result<Self, PathError> {
        let relative = path.strip_prefix(root.as_path())
            .map_err(|_| PathError::OutsideRoot { /* ... */ })?;
        let utf8 = relative.to_str()
            .ok_or_else(|| PathError::InvalidUtf8 { /* ... */ })?;
        Self::try_new(utf8)
    }
}

// Implement on FilePath, DirPath, FsPath:
pub fn as_key(&self, root: &DirPath) -> Result<PathKey, PathError>
```

4. **Error Types (Final):**
```rust
pub enum RootScopeError {
    PathOutsideVaultRootBoundary { root: PathBuf, path: PathBuf },
}

pub enum PathError {
    RootScope(#[from] RootScopeError),
    InvalidUtf8(PathBuf),
    // ...existing path invariants
}
```

5. **Deprecation Alias:**
```rust
#[deprecated(note = "Use PathKey instead")]
pub type NormalizedPath = PathKey;
```

**Acceptance criteria:**
- [x] `PathKey` exists and implements the `trim -> normalize -> validate` pipeline utilizing "parse, don't validate", preserving leading `/`, and optimizing with `Cow` (zero-copy when canonical, single-allocation otherwise).
- [x] Normalization explicitly removes trailing separators (except root `/`) and deduplicates separators.
- [x] `from_rooted_path` and `as_key` methods are implemented and fallible.
- [x] Root boundary and UTF-8 failures are surfaced when converting from filesystem paths (`RootScopeError::PathOutsideVaultRootBoundary`, `PathError::InvalidUtf8`).
- [x] Comprehensive tests cover normalization rules, traversal rejection, UTF-8 validation, and outside-root cases.

**Out of scope:**
- Repository signature migration (handled in subsequent slices).

## TDD & Implementation Plan

### 1. Planning & Design
**Deep Modules / Testability:**
- `PathKey` should encapsulate all normalization logic. The public interface only exposes `try_new` and `from_rooted_path`.
- Internally, use `Cow<'_, str>` to achieve zero-cost abstractions for already-canonical paths (Rust Best Practice: Performance).
- **Test Structure:** We will use **Structure A** from `unit-naming.md` within `lithos-core/src/fs/path.rs`.
- **Test Modules:** `constructor` (for `try_new`), `normalization` (for slashes/duplicates), `validation` (for invalid paths), and `conversions` (for `from_rooted_path` and `as_key`).

**Behaviors to Test (Prioritized):**
1. System accepts canonical paths without allocation.
2. System normalizes non-canonical paths (duplicate/trailing slashes).
3. System rejects invalid structures (empty, absolute, traversals).
4. System derives valid root-scoped keys from absolute paths, rejecting outside-root paths.

### 2. Tracer Bullet: Canonical Path Acceptance
**Behavior:** System accepts canonical paths without allocation.
- **RED:** In `mod pathkey { mod constructor { ... } }`, write test `accepts_canonical_paths_without_allocation` asserting `PathKey::try_new("a/b")` succeeds.
- **GREEN:** Implement `PathKey::try_new` pipeline (`trim` -> `normalize` -> `validate`). Use `Box<str>` for compact immutable ownership.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 3. Incremental Loop: Normalization
**Behavior:** System normalizes non-canonical paths (duplicate/trailing slashes).
- **RED:** In `mod pathkey { mod normalization { ... } }`, write tests `normalizes_backslashes_to_forward_slashes` (`a\\b` -> `a/b`), `normalizes_duplicate_slashes` (`a//b` -> `a/b`), `removes_trailing_slashes` (`a/b/` -> `a/b`).
- **GREEN:** Expand `normalize` to handle these. Ensure `Cow::Owned` is used only when modification is needed.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 4. Incremental Loop: Invariant Rejection
**Behavior:** System rejects invalid structures (empty, absolute, traversals).
- **RED:** In `mod pathkey { mod validation { ... } }`, write tests `rejects_empty_paths`, `rejects_absolute_paths` (`/a`), `rejects_parent_traversals` (`a/../b`).
- **GREEN:** Expand `validate` to return `PathError` variants using `thiserror` (Rust Best Practice: Error Handling). Add `OutsideRoot` to `PathError`.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 5. Incremental Loop: Root-Scoped Derivation
**Behavior:** System derives valid root-scoped keys from absolute paths, rejecting outside-root paths.
- **RED:** In `mod pathkey { mod conversions { ... } }`, write tests `returns_key_when_path_is_within_root` and `returns_error_when_path_is_outside_root`.
- **GREEN:** Implement `from_rooted_path` using `strip_prefix`.
**Checklist:**
- [ ] Test describes behavior, not implementation
- [ ] Test uses public interface only
- [ ] Test would survive internal refactor
- [ ] Code is minimal for this test
- [ ] No speculative features added

### 6. Refactor
- [x] Review borrowing and ownership: `try_new` takes `&str` instead of `String`.
- [x] Ensure `PathError` uses `thiserror`.
- [x] Add `///` doc comments for public APIs (Rust Best Practice: Documentation).

## Implementation Notes (2026-05-25)

- **Unification**: `PathKey` validation is now unified with the `RelativePathValidator` infrastructure.
- **Validation-First Pipeline**: `PathKey::try_new` was refactored to validate raw input *before* normalization. This ensures safety invariants (no `..`, no absolute paths) are enforced on the original string, preventing dangerous paths from being "cleaned" into a valid-looking state.
- **Canonical Double-Check**: A second validation pass is performed on the normalized (canonical) form as a safety net.
- **Logic Consolidation**:
  - `RelativePathValidator` (ZST) owns the "Strict Relative" policy.
  - `PathValidationContext::analyze` owns the path property analysis (absolute, traversal, etc.).
- **Consistency**: `PathKey` now shares the same `split('/')` dot-component detection as config types, improving security against platform-specific separator tricks.
- **Normalization Internals**:
  - `collect_normalization_context`
  - `apply_separator_canonicalization`
  - `apply_trailing_separator_policy`
- **Verification**:
  - Added `rejects_messy_dangerous_path_early` to `mod pathkey::validation`.
  - All 17 `pathkey` tests passing.
