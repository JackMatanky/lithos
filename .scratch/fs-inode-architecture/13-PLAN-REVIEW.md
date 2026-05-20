# Issue 13 Implementation Plan Review

**Date**: 2026-05-20
**Reviewer**: AI Agent (using gitnexus-impact-analysis, tdd, rust-best-practices skills)
**Issue File**: `.scratch/fs-inode-architecture/13-vault-model-types.md`

---

## Executive Summary

The implementation plan for moving `NormalizedPath` from `vault/model.rs` to `fs/path.rs` is **mostly sound** but has **4 critical gaps** and **7 refinement opportunities**. The plan correctly identifies the module boundary violation and proposes a reasonable approach, but misses important side effects around error handling, rkyv compatibility, and test coverage.

**Risk Level**: **MEDIUM** (simple move, but error type changes + vault export decision could cause downstream breakage)

**Estimated Effort**: 1.5-2 hours (up from original 1 hour estimate due to error type migration)

---

## Critical Gaps (Must Address)

### 1. Error Type Migration Strategy Missing

**Problem**: The plan mentions "update error handling" but doesn't specify the migration path from `VaultPathError` to fs-context errors.

**Current State**:
- `NormalizedPath::try_new()` returns `Result<Self, VaultPathError>`
- `VaultPathError` wraps `PathValidationError` (from fs/)
- After move, should return fs-context error type

**What's Missing**:
1. **Which fs error type?** Options:
   - `PathError` (constructor errors like `Empty`, `NotRelative`)
   - `PathValidationError` (security checks like `..`, `.`, platform prefix)
   - New error variant in `PathError`

2. **Call site impact**: vault/processor.rs has 2 functions using `NormalizedPath::try_new()`:
   ```rust
   // processor.rs:855
   fn normalize_path(...) -> Result<NormalizedPath, VaultFileError>

   // processor.rs:867
   fn normalize_parent(...) -> Result<Option<NormalizedPath>, VaultFileError>
   ```

   Both wrap `NormalizedPath::try_new()` errors into `VaultFileError::InvalidPath`. After the move, these will need to convert from the new fs error type.

**Recommendation**:
- **Use `PathValidationError`** directly (it's what `NormalizedPath` currently uses internally)
- `NormalizedPath::try_new()` should return `Result<Self, PathValidationError>`
- vault/processor.rs call sites already convert to `VaultFileError`, so they just need to update the `map_err` closure
- Add this to Slice 1 (before moving the code)

**Rust Best Practice Violation**:
- Current plan might introduce `unwrap()` or lossy error conversion
- Apollo Rust Best Practices Ch 4.2: "Avoid `unwrap`/`expect` in Production"
- Apollo Rust Best Practices Ch 4.3: Use `#[from]` for error hierarchies

### 2. Incomplete GitNexus Impact Analysis

**Problem**: Manual grep found 63 uses of `NormalizedPath`, but GitNexus found 0 upstream dependencies. The plan relies on incomplete data.

**Actual Usage** (from grep):
- **vault/mod.rs**: Public export (line 33)
- **vault/model.rs**: Definition + 2 tests (lines 136, 372, 375)
- **vault/processor.rs**:
  - Import (line 25)
  - 4 struct fields using `NormalizedPath` (lines 92, 99, 100, 108)
  - 3 function signatures (lines 126, 132)
  - 2 function parameters (lines 341, 389)
  - 1 HashMap key type (line 394)
  - 2 function return types (lines 855, 868)
  - 3 constructor calls in tests (lines 983, 992, 999)
- **vault/storage.rs**:
  - Import (line 15)
  - 8 trait method parameters (lines 50, 60, 90, 166, 178, 281, 310, 369)
  - 2 trait method return types (lines 142, 156)
  - 4 impl method parameters (lines 579, 629)
  - 2 `Vec<NormalizedPath>` construction sites (lines 522, 567)
  - 20 test uses (lines 896-1442)

**What's Missing**:
1. **Storage trait contract changes**: `Repository` trait uses `NormalizedPath` in 10 method signatures
2. **Type parameter in collections**: `HashSet<NormalizedPath>`, `HashMap<NormalizedPath, DirId>`, `Vec<NormalizedPath>`
3. **Struct field types**: 4 fields in `VaultProcessor` structs use `NormalizedPath`

**Recommendation**:
- Document all 63 usage sites in the agent brief
- Add verification step: "Run `cargo check` after each slice to catch import errors early"
- Add Slice 0 (pre-flight): "Use grep/ripgrep to catalog all NormalizedPath uses, verify against GitNexus"

### 3. rkyv Compatibility Not Verified

**Problem**: The plan mentions "ensure rkyv derives work" but doesn't verify that moving the type won't break archived data compatibility.

**Current State**:
- `NormalizedPath` has `#[derive(Archive, Serialize, Deserialize)]` and `#[rkyv(derive(Debug))]`
- Stored in redb tables via `Repository` trait methods
- Archive format includes fully-qualified type path (might include module path)

**Risk**:
- If archived format includes module path (`vault::model::NormalizedPath`), moving it to `fs::path::NormalizedPath` breaks deserialization of existing data
- redb persists rkyv-archived data — schema migration might be required

**What's Missing**:
1. **Archive format verification**: Does rkyv include module path in serialized format?
2. **Backward compatibility test**: Read archived data with old path, verify it works with new path
3. **Migration plan**: If incompatible, need a one-time data migration or dual-path support

**Recommendation**:
- Add Slice 0 verification: "Check if rkyv `Archive` includes module path in serialized format"
- If YES: Add migration strategy to agent brief (either re-export at old path temporarily, or write migration script)
- If NO: Document that archive format is module-agnostic and safe to move
- Consult project's rkyv usage patterns (check `docs/refs/crates/rkyv.md` if exists)

**Rust Best Practice Note**:
- Apollo Rust Best Practices Ch 3: "Always benchmark with `--release` flag" — but for schema changes, need backward-compat tests

### 4. Test Coverage Gap: No Tests for NormalizedPath

**Problem**: The plan assumes tests exist ("Move tests to fs/path.rs test module") but grep shows only 2 inline `expect("ok")` assertions in vault/model.rs tests (lines 372, 375).

**Actual Test Coverage**:
```rust
// vault/model.rs:372-375 (in normalize() tests)
NormalizedPath::try_new("notes\\daily\\today.md").expect("ok");
assert!(NormalizedPath::try_new("../outside.md").is_err());
```

These are incidental tests in the `normalize()` function tests, not dedicated `NormalizedPath` tests.

**What's Missing**:
1. **Behavior tests for validation rules**:
   - Forward slash normalization
   - Vault-relative constraint (no `..`, no `.`)
   - Invalid UTF-8 handling
   - Empty string rejection
   - Platform prefix rejection (Windows `C:`, UNC paths)
2. **rkyv roundtrip tests**:
   - Archive → bytes → deserialize equality
   - Archived access patterns (if `NormalizedPath` is accessed in archived form)
3. **Integration tests**:
   - Used as HashMap key (Hash + Eq invariants)
   - Used in struct fields (derive macro interactions)

**TDD Anti-Pattern Violation**:
- TDD skill: "Test describes behavior, not implementation"
- TDD skill: "Integration-style: Test through real interfaces, not mocks"
- Current "tests" are just smoke tests embedded in other function tests

**Recommendation**:
- **Add Slice 0 (RED)**: Write comprehensive behavior tests for `NormalizedPath` **before moving it**
  - Test in vault/model.rs first (establish baseline behavior)
  - Then move tests to fs/path.rs with the type
  - This follows TDD: tests anchor the behavior contract before refactor
- Test plan (7 tests):
  1. `normalized_path_accepts_forward_slashes`
  2. `normalized_path_converts_backslashes_to_forward_slashes`
  3. `normalized_path_rejects_parent_traversal`
  4. `normalized_path_rejects_current_dir_component`
  5. `normalized_path_rejects_empty_string`
  6. `normalized_path_rejects_absolute_paths`
  7. `normalized_path_rkyv_roundtrip_preserves_value`

---

## Refinement Opportunities (Should Consider)

### 5. Slice Order Violates TDD Principles

**Problem**: Current slice order is:
1. Copy to fs/path.rs
2. Add fs re-export
3. Update vault/mod.rs exports
4. Delete from vault/model.rs
5. Move tests

**TDD Violation**:
- TDD skill: "Never refactor while RED. Get to GREEN first."
- Copying the type (Slice 1) without tests means we're in an ambiguous state
- We should establish GREEN (all tests pass) before moving code

**Recommendation**:
- **Reorder slices**:
  1. **Slice 0 (RED → GREEN)**: Write NormalizedPath tests in vault/model.rs, verify they pass
  2. **Slice 1 (REFACTOR)**: Copy NormalizedPath + tests to fs/path.rs (keeping original)
  3. **Slice 2 (GREEN)**: Update imports in vault to use fs::NormalizedPath
  4. **Slice 3 (GREEN)**: Update vault/mod.rs export strategy
  5. **Slice 4 (GREEN)**: Delete original from vault/model.rs
  6. **Slice 5 (GREEN)**: Run full verify, fix any stragglers

- Each slice ends with `cargo test` passing
- Never in RED state during the move

### 6. Export Strategy Decision Lacks Clear Guidance

**Problem**: Agent brief lists two export options but doesn't provide decision criteria:
- Option A: Re-export from fs (`pub use crate::fs::NormalizedPath;`)
- Option B: Remove from vault exports (breaking change)

**Impact Analysis**:
- `vault/mod.rs` currently exports `NormalizedPath` (line 33)
- If external crates use `lithos_core::vault::NormalizedPath`, Option B breaks them
- If no external usage, Option B is cleaner (no redundant export)

**What's Missing**:
1. **External usage check**: Does any code outside `lithos-core/src/vault/` import `vault::NormalizedPath`?
2. **API stability policy**: Is `vault::NormalizedPath` part of the public API contract?
3. **Migration guide**: If breaking change, how should users migrate?

**Recommendation**:
- **Check for external usage**:
  ```bash
  rg "vault::NormalizedPath|vault::\{[^}]*NormalizedPath" lithos-core/src --glob '!vault/**'
  rg "use.*vault.*NormalizedPath" lithos-core/src --glob '!vault/**'
  ```
- **If no external usage**: Choose Option B (remove from vault exports) — cleaner boundary
- **If external usage exists**: Choose Option A (re-export) + add deprecation comment:
  ```rust
  #[deprecated(since = "0.x.0", note = "Import from crate::fs::NormalizedPath instead")]
  pub use crate::fs::NormalizedPath;
  ```
- Document decision in agent brief with rationale

**Rust Best Practice**:
- Apollo Rust Best Practices Ch 8.2: "Public API changes need migration path and docs"

### 7. NormalizedPath vs RelativePath Redundancy Not Addressed

**Problem**: Agent brief mentions the redundancy but defers it to "future work". This creates technical debt.

**Overlap Analysis**:
| Feature | NormalizedPath | RelativePath |
|---------|----------------|--------------|
| Storage | `Box<str>` | `PathBuf` |
| Accessor | `as_str() -> &str` | `as_path() -> &Path`, `as_str() -> Option<&str>` |
| Validation | `PathValidator::validate_vault_path` | Custom `validate()` function |
| Slash Normalization | YES (forward slashes) | NO (preserves OS slashes) |
| Use Case | Vault storage keys (string-based) | General vault-relative paths |

**Key Differences**:
1. **NormalizedPath** guarantees forward slashes (cross-platform storage keys)
2. **RelativePath** preserves platform slashes (filesystem operations)
3. **NormalizedPath** returns `&str` directly (no UTF-8 check needed)
4. **RelativePath** returns `Option<&str>` (UTF-8 not guaranteed)

**What's Missing**:
1. **Clarify when to use which**: Both enforce vault-relative constraints, but for different purposes
2. **Document the distinction**: Why do we have both?
3. **Consider converging**: Could `RelativePath` have a `.normalized() -> NormalizedPath` method?

**Recommendation**:
- **Add documentation** to both types explaining the distinction:
  ```rust
  /// Normalized vault-relative path using forward slashes.
  ///
  /// Use this for:
  /// - Storage keys (database, cache)
  /// - Cross-platform path serialization
  /// - String-based path comparison
  ///
  /// Use [`RelativePath`] for filesystem operations that need platform-native paths.
  pub struct NormalizedPath(Box<str>);
  ```
- **Add conversion method** (follow-up issue):
  ```rust
  impl RelativePath {
      pub fn normalized(&self) -> Result<NormalizedPath, PathValidationError> { ... }
  }
  ```
- **Document in CONTEXT.md** under "Language" section:
  - **NormalizedPath**: Cross-platform storage key (forward slashes, string-based)
  - **RelativePath**: Filesystem path (platform slashes, PathBuf-based)

### 8. Missing Verification of PathValidator Import

**Problem**: `NormalizedPath::try_new()` calls `PathValidator::validate_vault_path(...)`, but the plan doesn't verify that `PathValidator` is accessible from `fs/path.rs`.

**Current State** (from vault/model.rs:148):
```rust
PathValidator::validate_vault_path(normalized, None)
    .map_err(VaultPathError::from)?;
```

**Questions**:
1. Where is `PathValidator` defined? Is it in `fs/` already?
2. If it's in `vault/`, does `fs/path.rs` need to import from vault (circular dependency)?
3. Does `validate_vault_path` belong in `fs/` or `vault/`?

**Recommendation**:
- Add pre-flight check: "Verify PathValidator is accessible from fs/path.rs without circular imports"
- If `PathValidator` is in `vault/`: **This is a blocker** — path validation logic should move to `fs/` first
- If `PathValidator` is in `fs/`: Update the agent brief to document the import path

**Architecture Pattern**:
- FS context owns path validation (per CONTEXT.md: "Path validation is required before filesystem access")
- If `PathValidator` is in `vault/`, it's a **module boundary violation** (same issue as `NormalizedPath`)
- This might need a separate issue: "Move PathValidator to fs/ context"

### 9. No Rollback Plan

**Problem**: If the move breaks something subtle (rkyv compat, storage query performance, external API users), there's no rollback strategy.

**What's Missing**:
1. **Git strategy**: Should this be a single commit or multiple commits (one per slice)?
2. **Revert procedure**: If we discover a problem after merging, how do we safely revert?
3. **Feature flag**: Should we keep both paths temporarily with a feature flag?

**Recommendation**:
- **Single atomic commit**: All 5 slices in one commit (easier to revert)
- **Before merging**: Run full test suite + any integration tests
- **After merging**: Monitor for any issues in downstream contexts (note, schema, template)
- If rkyv compat is a concern: Keep temporary re-export at old path for one release cycle:
  ```rust
  // vault/model.rs - temporary backward compat
  #[deprecated(since = "0.x.0", note = "Moved to crate::fs::NormalizedPath")]
  pub use crate::fs::NormalizedPath;
  ```

### 10. Missing Context Boundary Documentation Update

**Problem**: The plan moves code but doesn't update context documentation to reflect the new boundary.

**Required Updates**:
1. **fs/CONTEXT.md**: Add `NormalizedPath` to "Language" section
2. **vault/CONTEXT.md**: Remove `NormalizedPath` from domain language (if documented)
3. **docs/agents/domain.md**: Update context map if it references path types

**Recommendation**:
- Add Slice 6: "Update context documentation"
  - Add to fs/CONTEXT.md under "Language":
    ```markdown
    **Normalized Path**:
    A vault-relative path normalized to forward slashes for cross-platform storage.
    _Avoid_: platform-specific path, absolute storage key
    ```
  - Update vault/CONTEXT.md "Not Owned Here":
    ```markdown
    - Path validation and normalization (owned by FS context)
    ```

### 11. Test Naming Doesn't Follow Project Convention

**Problem**: Current TDD plan has generic test function names, but the project likely has naming conventions (based on AGENTS.md mentioning `mise run test:unit`).

**Rust Best Practice**:
- Apollo Rust Best Practices Ch 5.1: "Test naming should be descriptive"
- Convention: `<functionality>_<scenario>_<expected_result>`

**Example**:
```rust
// BAD (current plan suggestion)
#[test]
fn test_normalized_path() { ... }

// GOOD (descriptive)
#[test]
fn normalized_path_try_new_accepts_forward_slashes() { ... }

#[test]
fn normalized_path_try_new_converts_backslashes() { ... }

#[test]
fn normalized_path_try_new_rejects_parent_traversal() { ... }
```

**Recommendation**:
- Update TDD plan with specific test names following project convention
- Each test name should read as a specification: "NormalizedPath::try_new() rejects parent traversal"

---

## Revised TDD Plan

### Pre-flight (Verification)

1. **Verify rkyv archive format**:
   ```bash
   # Check if module path is included in archived format
   rg "Archive.*NormalizedPath" lithos-core/src/vault
   # Check project docs for rkyv schema migration guidance
   cat docs/refs/crates/rkyv.md 2>/dev/null || echo "No rkyv docs found"
   ```

2. **Verify PathValidator location**:
   ```bash
   rg "PathValidator" lithos-core/src/fs
   rg "PathValidator" lithos-core/src/vault
   # Ensure no circular dependency risk
   ```

3. **Catalog all NormalizedPath usage**:
   ```bash
   rg "NormalizedPath" lithos-core/src --count-matches
   # Expected: 63 matches across vault/mod.rs, vault/model.rs, vault/processor.rs, vault/storage.rs
   ```

4. **Check external usage** (for export decision):
   ```bash
   rg "vault::NormalizedPath|use.*vault.*NormalizedPath" lithos-core/src --glob '!vault/**'
   # If matches: re-export needed; if none: clean removal
   ```

5. **Run baseline tests**:
   ```bash
   mise run test:unit
   # Establish GREEN state before refactor
   ```

### Slice 0: Write NormalizedPath Behavior Tests (RED → GREEN)

**Goal**: Establish behavior contract before moving code.

**RED**:
1. Create test module in `vault/model.rs`:
   ```rust
   #[cfg(test)]
   mod normalized_path_tests {
       use super::*;

       #[test]
       fn normalized_path_try_new_accepts_forward_slashes() {
           let path = NormalizedPath::try_new("notes/daily/today.md");
           assert!(path.is_ok());
           assert_eq!(path.unwrap().as_str(), "notes/daily/today.md");
       }

       #[test]
       fn normalized_path_try_new_converts_backslashes_to_forward_slashes() {
           let path = NormalizedPath::try_new("notes\\daily\\today.md");
           assert!(path.is_ok());
           assert_eq!(path.unwrap().as_str(), "notes/daily/today.md");
       }

       #[test]
       fn normalized_path_try_new_rejects_parent_traversal() {
           let path = NormalizedPath::try_new("../outside.md");
           assert!(path.is_err());
       }

       #[test]
       fn normalized_path_try_new_rejects_current_dir_component() {
           let path = NormalizedPath::try_new("./notes/file.md");
           assert!(path.is_err());
       }

       #[test]
       fn normalized_path_try_new_rejects_empty_string() {
           let path = NormalizedPath::try_new("");
           assert!(path.is_err());
       }

       #[test]
       fn normalized_path_try_new_rejects_absolute_paths() {
           let path = NormalizedPath::try_new("/usr/local/file.md");
           assert!(path.is_err());
       }

       #[test]
       fn normalized_path_rkyv_roundtrip_preserves_value() {
           let original = NormalizedPath::try_new("notes/test.md").unwrap();
           let bytes = rkyv::to_bytes::<_, 256>(&original).unwrap();
           let archived = rkyv::check_archived_root::<NormalizedPath>(&bytes).unwrap();
           let deserialized: NormalizedPath = archived.deserialize(&mut rkyv::Infallible).unwrap();
           assert_eq!(original, deserialized);
       }
   }
   ```

**GREEN**:
1. Run `cargo test normalized_path` — all tests should pass (implementation already exists)
2. If any test fails: Fix the issue before moving forward (this is our baseline)

**REFACTOR**:
1. Clean up test names if needed
2. Add doc comments to test module explaining coverage

**Verification**:
```bash
cargo test normalized_path_tests
mise run test:unit
```

### Slice 1: Change NormalizedPath Error Type (RED → GREEN)

**Goal**: Migrate from `VaultPathError` to `PathValidationError` before moving.

**RED**:
1. Change `NormalizedPath::try_new()` signature:
   ```rust
   // Before:
   pub fn try_new(path: &str) -> Result<Self, VaultPathError>

   // After:
   pub fn try_new(path: &str) -> Result<Self, PathValidationError>
   ```

2. Update error handling:
   ```rust
   // Before:
   PathValidator::validate_vault_path(normalized, None)
       .map_err(VaultPathError::from)?;

   // After:
   PathValidator::validate_vault_path(normalized, None)?;
   ```

3. Compiler will error on call sites in `vault/processor.rs`

**GREEN**:
1. Update `vault/processor.rs` call sites:
   ```rust
   // processor.rs:855
   fn normalize_path(...) -> Result<NormalizedPath, VaultFileError> {
       // Before:
       NormalizedPath::try_new(raw).map_err(|error| VaultFileError::InvalidPath { ... })

       // After:
       NormalizedPath::try_new(raw).map_err(|error| VaultFileError::InvalidPath {
           path: raw.into(),
           reason: error.to_string().into(),
       })
   }
   ```

2. Do the same for `normalize_parent()` at line 867

**REFACTOR**:
1. Check if error messages are still clear and actionable
2. Consider if `VaultFileError` should have a `#[from] PathValidationError` variant instead of string conversion

**Verification**:
```bash
cargo check
cargo test
```

### Slice 2: Copy NormalizedPath to fs/path.rs (REFACTOR)

**Goal**: Duplicate the type in new location (keeping original).

**GREEN**:
1. Copy `NormalizedPath` struct and impl from `vault/model.rs` to `fs/path.rs`:
   - Insert after `RelativePath` impl (around line 200)
   - Include doc comments
   - Verify rkyv derives still work

2. Copy test module from `vault/model.rs` to `fs/path.rs`:
   - Place in existing `#[cfg(test)] mod tests { ... }` section
   - Update imports if needed

3. Add re-export to `fs/mod.rs`:
   ```rust
   pub use path::{..., NormalizedPath, ...};
   ```

**Verification**:
```bash
cargo check
cargo test normalized_path_tests  # Should pass in both modules
```

### Slice 3: Update vault imports to use fs::NormalizedPath (GREEN)

**Goal**: Point all vault code at new location.

**GREEN**:
1. Update `vault/model.rs`:
   ```rust
   // Add at top of file:
   use crate::fs::NormalizedPath;
   ```

2. Update `vault/processor.rs`:
   ```rust
   // Before:
   use model::{DirId, DirView, FileId, FileView, NormalizedPath};

   // After:
   use crate::fs::NormalizedPath;
   use model::{DirId, DirView, FileId, FileView};
   ```

3. Update `vault/storage.rs`:
   ```rust
   // Before:
   use model::{..., NormalizedPath, ...};

   // After:
   use crate::fs::NormalizedPath;
   use model::{...};
   ```

**Verification**:
```bash
cargo check
cargo test
```

### Slice 4: Update vault/mod.rs export strategy (GREEN)

**Goal**: Decide and implement export strategy based on pre-flight check.

**Decision Point** (from pre-flight):
- **If external usage found**: Re-export with deprecation
- **If no external usage**: Remove from exports

**Option A - Re-export** (if external usage exists):
```rust
// vault/mod.rs
#[deprecated(
    since = "0.x.0",
    note = "Moved to crate::fs::NormalizedPath. Import from fs module instead."
)]
pub use crate::fs::NormalizedPath;
```

**Option B - Remove** (if no external usage):
```rust
// vault/mod.rs - just remove NormalizedPath from pub use model::{...}
pub use model::{
    DirId, DirView, FileId, FileView, FsEntryView,
    // NormalizedPath removed
};
```

**Verification**:
```bash
cargo check
cargo test
# If Option A: run clippy and verify deprecation warning appears
cargo clippy -- -D warnings
```

### Slice 5: Delete original from vault/model.rs (GREEN)

**Goal**: Remove duplicate definition.

**GREEN**:
1. Delete `NormalizedPath` struct (lines ~136-156 in vault/model.rs)
2. Delete `NormalizedPath` impl
3. Delete `normalized_path_tests` module (moved to fs/path.rs)
4. Keep the `use crate::fs::NormalizedPath;` import added in Slice 3

**Verification**:
```bash
cargo check  # Should have no compilation errors
cargo test   # All tests should pass
grep -n "struct NormalizedPath" lithos-core/src/vault/model.rs  # Should return nothing
```

### Slice 6: Update context documentation (GREEN)

**Goal**: Reflect new module boundary in documentation.

**GREEN**:
1. Update `lithos-core/src/fs/CONTEXT.md`:
   ```markdown
   ## Language

   ... existing entries ...

   **Normalized Path**:
   A vault-relative path normalized to forward slashes for cross-platform storage keys.
   Use [`NormalizedPath`] for database keys and serialized path storage.
   Use [`RelativePath`] for filesystem operations.
   _Avoid_: platform-specific path, absolute storage key
   ```

2. Update `lithos-core/src/vault/CONTEXT.md`:
   ```markdown
   ## Not Owned Here

   - Note/schema/template business semantics and validation rules.
   - Persistence transaction semantics and archived read strategy.
   - CLI command intent and user-facing output behavior.
   - Path validation and normalization (owned by FS context).  # NEW
   ```

3. Add doc comment to `NormalizedPath` in `fs/path.rs`:
   ```rust
   /// Normalized vault-relative path using forward slashes.
   ///
   /// This type enforces vault-relative path constraints and normalizes all
   /// paths to use forward slashes (`/`) for cross-platform compatibility.
   ///
   /// # Use Cases
   ///
   /// - Database storage keys (consistent across platforms)
   /// - Serialized path representation in rkyv archives
   /// - Path comparison and hashing (HashMap keys, HashSet members)
   ///
   /// # Comparison with [`RelativePath`]
   ///
   /// - [`NormalizedPath`]: Forward slashes, `Box<str>` storage, `as_str() -> &str`
   /// - [`RelativePath`]: Platform slashes, `PathBuf` storage, `as_path() -> &Path`
   ///
   /// Use [`RelativePath`] for filesystem operations; use [`NormalizedPath`]
   /// for storage and serialization.
   ```

**Verification**:
```bash
# Verify doc comments render correctly
cargo doc --no-deps --open
# Navigate to NormalizedPath and verify docs are clear
```

### Final Verification

**Checklist**:
- [ ] `cargo check` passes with no warnings
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `mise run test:unit` passes (all unit tests)
- [ ] `mise run verify` passes (fmt + lint + tests + adr:validate)
- [ ] `grep "struct NormalizedPath" lithos-core/src/vault/model.rs` returns nothing
- [ ] `grep "NormalizedPath" lithos-core/src/fs/path.rs` shows definition and tests
- [ ] `rg "use.*vault.*NormalizedPath" lithos-core/src` returns 0 matches (or only deprecation re-export)
- [ ] Doc comments for `NormalizedPath` explain when to use it vs `RelativePath`
- [ ] CONTEXT.md files updated to reflect new boundary

**Regression Tests** (manual verification):
1. Run vault storage tests: `cargo test vault::storage`
2. Run vault processor tests: `cargo test vault::processor`
3. Verify rkyv roundtrip still works (test in slice 0 covers this)
4. Check that existing serialized data still deserializes (if rkyv includes module path)

---

## Side Effects Analysis

### Downstream Impacts (Low Risk)

1. **note context**: Uses `vault::Repository` trait, but doesn't directly use `NormalizedPath` ✅
2. **schema context**: Uses `vault::Repository` trait, but doesn't directly use `NormalizedPath` ✅
3. **template context**: Uses `vault::Repository` trait, but doesn't directly use `NormalizedPath` ✅
4. **CLI commands**: Use vault processor, but don't directly use `NormalizedPath` ✅

All downstream contexts use the vault API through trait methods, so the internal move is transparent to them.

### Cross-Context Compilation (Medium Risk)

If any downstream context has:
```rust
use lithos_core::vault::NormalizedPath;  // This will break if we choose Option B
```

**Mitigation**: Pre-flight check for external usage determines export strategy.

### Performance Implications (Low Risk)

- `NormalizedPath` representation unchanged (`Box<str>`)
- Hash + Eq implementations unchanged
- rkyv archive format unchanged (if module-agnostic)
- No additional allocations introduced

### Future Refactor Opportunities

After this issue:
1. **Issue 13.1**: Consolidate `NormalizedPath` and `RelativePath` (add `.normalized()` conversion)
2. **Issue 13.2**: Move `PathValidator` to `fs/` if it's currently in `vault/`
3. **Issue 13.3**: Add `NormalizedPath -> RelativePath` conversion (for filesystem operations)

---

## Recommendations Summary

### Must Fix (Before Starting)

1. ✅ **Add Slice 0**: Write comprehensive NormalizedPath tests before moving
2. ✅ **Add Slice 1**: Change error type from `VaultPathError` to `PathValidationError` before moving
3. ✅ **Pre-flight**: Verify rkyv archive format doesn't include module path
4. ✅ **Pre-flight**: Verify PathValidator location and accessibility from fs/

### Should Fix (During Implementation)

5. ✅ **Reorder slices**: Follow TDD discipline (tests first, never in RED state)
6. ✅ **Document export decision**: Add rationale for re-export vs removal
7. ✅ **Add Slice 6**: Update context documentation
8. ✅ **Add verification steps**: `cargo check` after each slice

### Nice to Have (Follow-up Issues)

9. ⚠️ **Document NormalizedPath vs RelativePath distinction** in both type doc comments
10. ⚠️ **Create follow-up issue**: Consolidate path types (add conversion methods)
11. ⚠️ **Git strategy**: Single atomic commit for easier rollback

---

## Conclusion

The original implementation plan is **structurally sound** but has **4 critical gaps** that must be addressed:

1. **Error type migration** (missing strategy, will cause compilation errors)
2. **Incomplete impact analysis** (63 usages not documented)
3. **rkyv compatibility** (not verified, potential data loss risk)
4. **Test coverage** (no dedicated tests, violates TDD principles)

**Revised Effort Estimate**: 1.5-2 hours (up from 1 hour)
- +15 min: Write tests (Slice 0)
- +15 min: Error type migration (Slice 1)
- +15 min: Documentation updates (Slice 6)

**Risk Level**: **MEDIUM** → **LOW** (after addressing critical gaps)

The revised TDD plan above addresses all gaps and follows:
- ✅ GitNexus impact analysis discipline (catalog all usage, verify blast radius)
- ✅ TDD principles (tests first, never in RED, refactor in GREEN)
- ✅ Rust best practices (error hierarchies, no unwrap, descriptive tests, doc comments)

**Recommendation**: Use the revised TDD plan above instead of the original plan in the issue file.
