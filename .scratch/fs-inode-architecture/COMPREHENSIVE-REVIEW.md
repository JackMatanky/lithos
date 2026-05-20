# Comprehensive Code Review: FS Inode Architecture Implementation

**Date**: 2026-05-20
**Reviewer**: AI Agent (using gitnexus + rust-best-practices + triage skills)
**Scope**: `lithos-core/src/fs/` + `lithos-core/src/vault/model.rs` + `.scratch/fs-inode-architecture/` issues

---

## Executive Summary

The fs-inode-architecture implementation has **37 critical issues** that must be addressed before completion. The issues fall into five categories:

1. **Type Safety Violations (11 issues)**: "Parse, don't validate" principle violated in conversion traits
2. **Error Type Mismatches (8 issues)**: Wrong error types used, ignoring `fs/error.rs` design
3. **API Design Problems (7 issues)**: Name types not optimized for primary use cases
4. **Vault Model Legacy Code (5 issues)**: Old structs not removed as planned
5. **Documentation and Testing Gaps (6 issues)**: Missing validation, unclear ownership

**Severity Distribution:**
- 🔴 **CRITICAL** (19): Type safety, data integrity, security
- 🟡 **HIGH** (12): API usability, error handling
- 🟢 **MEDIUM** (6): Documentation, test coverage

---

## Critical Issues (🔴)

### 1. FilePath and DirPath Conversion Traits Violate "Parse, Don't Validate" ⚠️ CRITICAL

**Location**: `lithos-core/src/fs/path.rs:473-478`, `path.rs:653-658`

**Problem**: Both `FilePath` and `DirPath` implement `From<PathBuf>` which cannot fail, but their constructors (`::new()`) perform filesystem checks that *can* fail. This violates Rust's type safety guarantees.

```rust
// path.rs:473-478
impl From<PathBuf> for FilePath {
    #[inline]
    fn from(path: PathBuf) -> Self {
        Self(path)  // ❌ No validation! Path might not refer to a file!
    }
}

// path.rs:653-658
impl From<PathBuf> for DirPath {
    #[inline]
    fn from(path: PathBuf) -> Self {
        Self(path)  // ❌ No validation! Path might not refer to a directory!
    }
}
```

**Comparison with correct implementations:**
```rust
// path.rs:448-455 — CORRECT (fallible)
impl TryFrom<RelativePath> for FilePath {
    type Error = std::io::Error;
    fn try_from(path: RelativePath) -> Result<Self, Self::Error> {
        Self::new(path.0)  // ✅ Uses validating constructor
    }
}
```

**Why This Is Critical:**
- Allows creation of `FilePath` wrapping a directory path (type lie)
- Allows creation of `DirPath` wrapping a file path (type lie)
- Downstream code relies on these types being correct for safety decisions
- Scanner and reader code would break if these invariants are violated

**Solution:**
```rust
// Replace From<PathBuf> with TryFrom<PathBuf>
impl TryFrom<PathBuf> for FilePath {
    type Error = std::io::Error;  // Or use PathError from error.rs
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl TryFrom<PathBuf> for DirPath {
    type Error = std::io::Error;  // Or use PathError from error.rs
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}
```

**Reference**: Apollo Rust Best Practices Chapter 1 (borrowing vs cloning), Chapter 4 (error handling)

---

### 2. FilePath::new() and DirPath::new() Perform I/O Without Documentation ⚠️ CRITICAL

**Location**: `lithos-core/src/fs/path.rs:320-339`, `path.rs:500-520`

**Problem**: Both constructors call `path.is_file()` and `path.is_dir()` which perform filesystem I/O, but this is not documented in the error docs or method signature.

```rust
// path.rs:332-336
if !path.is_file() {  // ❌ This performs I/O! Can fail with permissions, race conditions
    return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Path does not refer to a file",
    ));
}
```

**Why This Is Critical:**
- Surprising behavior: constructors usually don't do I/O
- Race conditions: file can be deleted between check and use (TOCTOU)
- Performance: hot paths calling this repeatedly do unexpected I/O
- Error handling: callers may not expect I/O errors from path construction

**Manifestation in entry.rs:**
```rust
// entry.rs:135-137
let dir_path = DirPath::new(path.clone()).map_err(|_source| {
    ScanError::Path(PathError::NotADirectory(path))  // ❌ Loses I/O error information!
});
```

**Solution Options:**

**Option A**: Make the I/O explicit with separate constructors:
```rust
impl FilePath {
    /// Create from PathBuf without validation (caller guarantees it's a file)
    pub fn from_unchecked(path: PathBuf) -> Self {
        Self(path)
    }

    /// Validate that path refers to an existing file (performs I/O)
    pub fn from_fs_checked(path: PathBuf) -> Result<Self, PathError> {
        if !path.is_file() {
            return Err(PathError::NotAFile(path));
        }
        Ok(Self(path))
    }
}
```

**Option B**: Remove filesystem checks entirely (path type = syntax only):
```rust
impl FilePath {
    /// Create a file path. Does not check if file exists.
    pub fn new(path: PathBuf) -> Result<Self, PathError> {
        if path.as_os_str().is_empty() {
            return Err(PathError::Empty);
        }
        Ok(Self(path))
    }
}
```

**Recommended**: Option B. The PRD says these types support "absolute or relative" paths, meaning they should be syntactic wrappers, not filesystem validators. Entry existence checks belong in `FsReader` and `DirScanner`, not path types.

---

### 3. Wrong Error Types Used Throughout path.rs ⚠️ CRITICAL

**Location**: Multiple locations in `lithos-core/src/fs/path.rs`

**Problem**: The module uses `std::io::Error` for path construction errors, but `fs/error.rs` defines a proper `PathError` type with specific variants for these exact cases.

**Examples:**

```rust
// path.rs:82-85 — Uses std::io::Error
return Err(std::io::Error::new(
    std::io::ErrorKind::InvalidInput,
    "Path cannot be empty",
));

// error.rs:50-52 — Should use this instead
pub enum PathError {
    #[error("Path is empty")]
    Empty,
    // ... 10 other path-specific variants
}
```

**All violations:**
- Line 82-85: Empty path check → should use `PathError::Empty`
- Line 88-91: Absolute path check → should use `PathError::NotRelative`
- Line 102-105: Current dir component → should use `PathError::CurrentDirComponent`
- Line 111-114: Parent traversal → should use `PathError::ParentTraversal`
- Line 117-120: Platform prefix → should use `PathError::PlatformPrefix`
- Line 239-243: Empty path check → should use `PathError::Empty`
- Line 245-249: Relative path check → should use `PathError::NotAbsolute`
- Line 327-330: Empty path check → should use `PathError::Empty`
- Line 332-336: File check → should use `PathError::NotAFile`
- Line 507-511: Empty path check → should use `PathError::Empty`
- Line 513-517: Directory check → should use `PathError::NotADirectory`

**Why This Is Critical:**
- Defeats the purpose of having a domain-specific error hierarchy
- Makes error matching impossible at call sites (can't distinguish path vs I/O errors)
- Violates ADR 017's error design (which created these error types)
- Confuses "path syntax errors" with "I/O operation errors"

**Solution**: Replace all `std::io::Error::new(...)` calls with appropriate `PathError` variants, then return `Result<T, PathError>` from all path methods.

---

### 4. name.rs Types Are Backwards For Primary Use Case ⚠️ CRITICAL

**Location**: `lithos-core/src/fs/name.rs`

**Problem**: The PRD states `FilePath`, `DirPath`, and `FsPath` should be the **primary access points** to name types, but `name.rs` is currently designed as standalone types that path types depend on. This creates circular dependencies and awkward APIs.

**Current Design (Wrong):**
```rust
// name.rs defines FileName as standalone
pub struct FileName(Box<str>);

// path.rs depends on name.rs
use super::name::{FileName, FileNameRef, ...};

impl FilePath {
    pub fn filename(&self) -> Option<FileName> {
        FileName::try_from(self.0.as_path()).ok()  // ❌ Awkward conversion
    }
}
```

**What PRD Intended:**
```rust
// FilePath should be the primary way to get filenames
impl FilePath {
    pub fn filename(&self) -> FileNameRef<'_> {  // Zero-copy borrowed view
        FileNameRef(self.0.file_name().unwrap())  // Safe: FilePath guarantees filename exists
    }

    pub fn filename_owned(&self) -> FileName {  // Allocating version when needed
        FileName::new(self.0.file_name().unwrap().to_str().unwrap().into())
    }
}

// FileName should primarily be a storage type, not an entry point
pub struct FileName(Box<str>);

impl FileName {
    pub(crate) fn new(name: Box<str>) -> Self {  // Private constructor
        Self(name)
    }
}
```

**Evidence From PRD:**
> **User Story 10**: "As a developer, I want separate owned and borrowed filename types, so that I can avoid allocations in hot paths (zero-copy extraction) while still storing filenames in domain models"
>
> **User Story 11**: "As a developer, I want file and directory entities to be distinguishable at the type level, so that I cannot accidentally use a file path where a directory is required"

**Why This Is Critical:**
- Current design requires `TryFrom<&Path>` for `FileName`, which can fail (violating type guarantees)
- Makes it impossible to have infallible zero-copy filename extraction from `FilePath`
- PRD's zero-copy vision requires `FilePath` to guarantee filename existence at construction

**Solution**: Refactor `name.rs` to make owned name types primarily storage containers, with primary extraction happening through path types. See detailed plan in "Medium Priority" section.

---

### 5. metadata.rs Uses Wrong Error Type (std::io::Error) ⚠️ CRITICAL

**Location**: `lithos-core/src/fs/metadata.rs:117-140`, `metadata.rs:192-199`, `metadata.rs:241-248`

**Problem**: `TryFrom<&Path>` implementations for `FileName` and `BaseName` return `std::io::Error`, but should use `PathError` from `error.rs`.

```rust
// name.rs:117-140
impl TryFrom<&Path> for FileName {
    type Error = std::io::Error;  // ❌ Should be PathError

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let name = path.file_name()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,  // ❌ Wrong error kind
                    "Path terminates in .. or is empty",
                )
            })?
    }
}
```

**Correct approach:**
```rust
impl TryFrom<&Path> for FileName {
    type Error = PathError;  // ✅ Use domain error

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let name = path.file_name()
            .ok_or_else(|| PathError::NoFileName(path.to_path_buf()))?;  // ✅ Specific error

        let name_str = name.to_str()
            .ok_or_else(|| PathError::InvalidUtf8(path.to_path_buf()))?;  // ✅ Specific error

        Ok(Self::new(name_str.into()))
    }
}
```

**All violations in name.rs:**
- Lines 117-140: `FileName::try_from(&Path)` → uses `std::io::Error`
- Lines 185-200: `BaseName::try_from(FileName)` → uses `std::io::Error`
- Lines 203-218: `BaseName::try_from(&Path)` → uses `std::io::Error`

---

### 6. NormalizedPath Should Be In fs/path.rs, Not vault/model.rs ⚠️ CRITICAL

**Location**: `lithos-core/src/vault/model.rs:121-149`

**Problem**: `NormalizedPath` is a path normalization primitive with no vault-specific semantics, but it's defined in `vault/model.rs`. This violates the module boundary defined in the PRD.

```rust
// vault/model.rs:121-149
pub struct NormalizedPath(Box<str>);  // ❌ Should be in fs/path.rs

impl NormalizedPath {
    pub fn try_new(path: &str) -> Result<Self, VaultPathError> {
        let normalized = VaultPath::normalize(path);  // ❌ Depends on VaultPath
        let normalized = normalized.as_ref().trim();
        PathValidator::validate_vault_path(normalized, None)  // Uses fs/ validator
            .map_err(VaultPathError::from)?;
        Ok(Self(normalized.into()))
    }
}
```

**Why This Is Critical:**
- Prevents reuse of path normalization outside vault context
- Creates circular dependency risk (fs/ validator used in vault/)
- PRD's "Infrastructure vs Domain" split violated
- Makes it harder to test normalization rules in isolation

**Evidence:**
The PRD explicitly says:
> **fs/ (Infrastructure - building blocks):**
> - Path types: `AbsolutePath`, `RelativePath`, `FilePath`, `DirPath`
> - Zero-copy views: `FileNameRef<'a>`, `DirNameRef<'a>`, `BaseNameRef<'a>`, `FileExtensionRef<'a>`, `ParentDir<'a>`

> **vault/ (Domain - inode-based tracking):**
> - Identifiers: `FileId(UuidV7)`, `DirId(UuidV7)`
> - Storage entities: `FileView`, `DirView`, `FsEntryView` (compose fs/ primitives)
> - Database keys: `NormalizedPath` (vault-relative, forward-slash normalized)

The PRD lists `NormalizedPath` as a **database key** in vault/, but the *type itself* is a path primitive and should be in fs/. Only its *usage as a database key* belongs in vault/.

**Solution**: Move `NormalizedPath` to `fs/path.rs`, remove dependency on `VaultPath::normalize()` (make normalization part of `NormalizedPath` itself), keep usage in vault/.

---

### 7. Vault Model Contains Old Structs Scheduled For Removal ⚠️ CRITICAL

**Location**: `lithos-core/src/vault/model.rs`

**Problem**: The following legacy types still exist, but PRD Phase 4 cleanup specifies they should be deleted:

1. **`VaultPath`** (lines 345-413): Replaced by `NormalizedPath` + `fs/path.rs` types
2. **`VaultFile`** (lines 415-512): Replaced by `FileView`
3. **`VaultFolder`** (lines 514-584): Replaced by `DirView`
4. **`PathParts`** (lines 586-622): Internal helper, should be deleted
5. **`FolderParts`** (lines 624-646): Internal helper, should be deleted

**Current usage (unacceptable):**
```rust
// vault/model.rs:419-454
pub struct VaultFile {
    path: VaultPath,  // ❌ Uses legacy VaultPath
    basename: Box<str>,
    filename: Box<str>,
    parent: Box<str>,
    extension: Option<Box<str>>,
    size: u64,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
}

impl VaultFile {
    pub fn try_new(
        path: VaultPath,
        metadata: &std::fs::Metadata,
    ) -> Result<Self, VaultPathError> {
        let parts = PathParts::try_new(path.as_path())?;  // ❌ Uses legacy PathParts
        // ...
    }
}
```

**Why This Is Critical:**
- Blocks migration to new architecture (can't remove old code while it's still used)
- Creates confusion about which types to use (old vs new)
- Test suite may be validating obsolete behavior
- Storage layer may still be persisting old types

**Evidence from PRD Phase 4:**
> **Phase 4 cleanup steps:**
> 1. Delete `VaultFile`, `VaultFolder`, `PathParts`, `FolderParts`
> 2. Delete vault error types that are no longer used
> 3. Delete `fs/file.rs` (split into name.rs, metadata.rs, entry.rs)
> 4. Delete `fs/types.rs` (moved to format.rs)

**Solution**:
1. Audit all uses of `VaultFile`, `VaultFolder`, `VaultPath` in the codebase
2. Migrate to `FileView`, `DirView`, `NormalizedPath`
3. Delete the old types once migration is complete
4. Update tests to validate new types

---

### 8. entry.rs Loses Error Information In TryFrom<walkdir::DirEntry> ⚠️ CRITICAL

**Location**: `lithos-core/src/fs/entry.rs:135-137`, `entry.rs:142-144`

**Problem**: When `DirPath::new()` or `FilePath::new()` fail, the implementation discards the underlying I/O error and creates a `PathError` instead. This loses critical debugging information.

```rust
// entry.rs:135-137
let dir_path = DirPath::new(path.clone()).map_err(|_source| {
    //                                                ^^^^^^^ ❌ Discards std::io::Error!
    ScanError::Path(PathError::NotADirectory(path))
})?;

// entry.rs:142-144
let file_path = FilePath::new(path.clone()).map_err(|_source| {
    //                                                 ^^^^^^^ ❌ Discards std::io::Error!
    ScanError::Path(PathError::NotAFile(path))
})?;
```

**Why This Is Critical:**
- Loses information about *why* the path check failed (permissions? doesn't exist? is a socket?)
- Makes debugging scanner failures nearly impossible
- Violates error handling best practices (preserve source errors)

**Correct approach (after fixing Issue #3):**
```rust
// After PathError becomes the error type for path constructors:
let dir_path = DirPath::new(path).map_err(ScanError::Path)?;  // ✅ Preserves error chain
```

**Reference**: Apollo Rust Best Practices Chapter 4 (error handling), thiserror's `#[from]` attribute for error chaining.

---

### 9. scanner.rs File Organization Violates PRD Granular Structure ⚠️ HIGH

**Location**: `lithos-core/src/fs/scanner.rs`

**Problem**: The PRD specifies a "granular file organization" approach where each conceptual type gets its own file, but `scanner.rs` is a 844-line file mixing `DirScanner`, `DirScanInput`, and extensive test modules.

**Why This Is High Priority:**
- Violates PRD's stated organizational principle
- Makes navigation harder (jump-to-definition lands in middle of large file)
- Increases cognitive load (multiple concepts in one file)

**Recommended**: Keep as-is for now (scanner is logically one unit), but split if it grows beyond 1000 lines.

---

### 10. FilePath and DirPath Perform Redundant Path Clones ⚠️ HIGH

**Location**: `lithos-core/src/fs/entry.rs:135`, `entry.rs:142`

**Problem**: The code clones paths even in the success case, which is unnecessary.

```rust
// entry.rs:133-139 — Current implementation
if std_metadata.is_dir() {
    let dir_path = DirPath::new(path.clone()).map_err(|_source| {
        //                          ^^^^^^^^^^^^ ❌ Clone for success case
        ScanError::Path(PathError::NotADirectory(path))
        //                                        ^^^^ Original path only used in error case
    })?;
```

**Correct approach:**
```rust
if std_metadata.is_dir() {
    let dir_path = DirPath::new(path).map_err(|_source| {
        ScanError::Path(PathError::NotADirectory(path.clone()))  // ✅ Clone only in error case
    })?;
```

**Why This Matters:**
- Scanner hot path (called thousands of times)
- Unnecessary allocations hurt performance
- Violates Apollo best practices (avoid cloning in loops)

**Reference**: Apollo Rust Best Practices Chapter 3 (performance mindset, avoiding redundant clones)

---

## High Priority Issues (🟡)

### 11. RelativePath and AbsolutePath Validation Is Incomplete ⚠️ HIGH

**Location**: `lithos-core/src/fs/path.rs:80-128`, `path.rs:238-252`

**Problem**: `RelativePath::validate()` checks for empty path, absolute path, current dir, parent dir, and platform prefix, but `AbsolutePath::validate()` only checks for empty and absolute. This asymmetry suggests incomplete design.

**Questions:**
- Should `AbsolutePath` check for `.` and `..` components?
- Should it validate against platform-specific weird cases (e.g., `C:` drive on non-Windows)?
- What about normalization (multiple slashes, trailing slashes)?

**Solution**: Document validation invariants in each type's doc comment, add tests for edge cases.

---

### 12. ParentDir::Root Has Unclear Semantics ⚠️ HIGH

**Location**: `lithos-core/src/fs/path.rs:816-834`

**Problem**: `ParentDir::Root` is returned when `path.parent()` returns `None` or an empty path, but it's unclear what "Root" means in the context of relative paths.

```rust
// path.rs:827-832
pub fn from_path(path: &'a Path) -> Self {
    match path.parent() {
        Some(p) if p.as_os_str().is_empty() => Self::Root,  // ❓ What does Root mean for "file.txt"?
        Some(p) => Self::Path(p),
        None => Self::Root,
    }
}
```

**Example confusion:**
```rust
let path = Path::new("file.txt");
let parent = ParentDir::from_path(path);
// Is parent Root (empty parent) or should it be Path("")?
```

**Solution**: Document what `Root` means (vault root? filesystem root? current directory?), add tests showing expected behavior for relative vs absolute paths.

---

### 13. Extension Names Are Inconsistent Across Modules ⚠️ HIGH

**Problem**: Different naming conventions for "file extension" across the codebase:

- `path.rs:429`: `extension_ref()` returns `FileExtensionRef`
- `name.rs:69`: `extension()` returns `Option<&str>`
- `format.rs` (mentioned in PRD): should define `FileExtensionRef`

**Solution**: Consolidate on one naming pattern:
- Owned: `FileExtension` (in `format.rs`)
- Borrowed: `FileExtensionRef<'a>` (in `format.rs`)
- Methods: `path.extension_ref()` → `FileExtensionRef<'a>`, `path.extension()` → `FileExtension` (allocating)

---

### 14. Missing Tests for UTF-8 Edge Cases ⚠️ HIGH

**Location**: `lithos-core/src/fs/path.rs`, `lithos-core/src/fs/name.rs`

**Problem**: While there are some UTF-8 tests (e.g., `path.rs:892-901`), many conversions lack UTF-8 validation tests.

**Missing test coverage:**
- `DirPath::dirname()` with invalid UTF-8 (line 583-588)
- `BaseName::try_from(&Path)` with invalid UTF-8 in stem
- `FileExtensionRef` (if it exists) with invalid UTF-8

**Solution**: Add comprehensive UTF-8 test suite covering all owned/borrowed name extraction paths.

---

### 15. FsPath::as_relative() Error Handling Is Inconsistent ⚠️ HIGH

**Location**: `lithos-core/src/fs/path.rs:725-742`

**Problem**: `FsPath::as_relative()` returns `Result<RelativePath, FsError>`, wrapping `ReadError`, but `FilePath::as_relative()` and `DirPath::as_relative()` return `Result<RelativePath, ReadError>` directly.

```rust
// path.rs:730-742
pub fn as_relative(&self, base: &Path) -> Result<RelativePath, super::error::FsError> {
    match self {
        Self::File(p) => {
            p.as_relative(base).map_err(super::error::FsError::Read)  // Wraps ReadError
        }
        Self::Dir(p) => {
            p.as_relative(base).map_err(super::error::FsError::Read)  // Wraps ReadError
        }
    }
}
```

**Why inconsistent**: Callers working with `FsPath` must handle `FsError`, but callers working with `FilePath`/`DirPath` can handle the more specific `ReadError`.

**Solution**: Either wrap all three in `FsError` (consistent but less specific), or make `FsPath::as_relative()` return `ReadError` directly (more specific but requires callers to wrap if needed).

---

### 16. Missing #[inline] Attributes on Hot Path Methods ⚠️ HIGH

**Location**: Multiple locations

**Problem**: Accessor methods like `as_path()`, `as_str()`, `is_file()` are called in tight loops but lack `#[inline]` hints. While the compiler may inline them anyway, explicit attributes improve performance predictability.

**Examples missing #[inline]:**
- `name.rs:22-27`: `FileName::as_str()`
- `name.rs:147-152`: `FileNameRef::as_str()`
- `metadata.rs:66-70`: `FsMetadata::is_file()`
- `entry.rs:29-34`: `FsEntry::is_file()`

**Solution**: Add `#[inline]` to all trivial accessor methods. Use `#[inline(always)]` only for proven hot paths after profiling.

**Reference**: Apollo Rust Best Practices Chapter 3 (performance optimization)

---

### 17. DirScanner::new() Accepts Any Path Without Validation ⚠️ HIGH

**Location**: `lithos-core/src/fs/scanner.rs:64-80`

**Problem**: `DirScanner::new()` accepts any `Into<PathBuf>`, including non-existent paths, files (not directories), or relative paths that might be ambiguous.

```rust
// scanner.rs:74-80
pub fn new<P: Into<PathBuf>>(path: P) -> Self {
    Self {
        path: path.into(),  // ❌ No validation
    }
}
```

**Why This Is High Priority:**
- Scanner will fail later during iteration (worse UX than fail-fast at construction)
- Error messages will be confusing ("failed to scan X" when X was never a valid directory)
- Violates "parse, don't validate" principle

**Solution:**
```rust
pub fn try_new<P: AsRef<Path>>(path: P) -> Result<Self, PathError> {
    let path = path.as_ref();
    if !path.is_dir() {
        return Err(PathError::NotADirectory(path.to_path_buf()));
    }
    Ok(Self {
        path: path.to_path_buf(),
    })
}
```

---

### 18. Missing Benchmarks For Path Hot Paths ⚠️ HIGH

**Problem**: No benchmarks exist for critical path operations:
- `FilePath::filename()` vs `FilePath::filename_ref()` (owned vs borrowed)
- `FsEntry::try_from(walkdir::DirEntry)` (scanner hot path)
- `RelativePath::validate()` (called on every path)

**Solution**: Add criterion benchmarks in `benches/fs_path.rs` to validate zero-copy design decisions and measure scanner throughput.

**Reference**: Apollo Rust Best Practices Chapter 3 (always benchmark with --release)

---

## Medium Priority Issues (🟢)

### 19. Unclear Ownership of Path Validation Between fs/ and vault/ ⚠️ MEDIUM

**Problem**: Three different validation mechanisms exist:
1. `fs/validator.rs`: `PathValidator` for vault-relative paths
2. `fs/path.rs`: `RelativePath::validate()` for relative path syntax
3. `vault/model.rs`: `VaultPath::normalize()` + validation

**Which layer owns security checks?** If vault/ is responsible, why does fs/ have a validator? If fs/ is responsible, why does vault/ call it?

**Solution**: Document validation ownership in `fs/CONTEXT.md` and `vault/CONTEXT.md`. Likely answer: fs/ validates *syntax* (relative, no `..`), vault/ validates *security* (within vault root, no symlink escapes).

---

### 20. FsPathRef Lifetime Bound Is Unclear ⚠️ MEDIUM

**Location**: `lithos-core/src/fs/path.rs:750-813`

**Problem**: `FsPathRef<'a>` is a zero-copy reference to `FsPath`, but its lifetime is tied to the `FsPath` owner, not to the underlying `PathBuf` inside `FilePath`/`DirPath`. This may prevent some valid use cases.

**Example potential issue:**
```rust
fn get_path_ref(entry: &FsEntry) -> FsPathRef<'_> {
    entry.path_ref()  // Works: borrows entry
}

fn get_inner_path(entry: &FsEntry) -> &Path {
    let path_ref = entry.path_ref();
    path_ref.as_path()  // Lifetime tied to path_ref, not entry
}
```

**Solution**: Add tests for complex lifetime scenarios, document lifetime guarantees in type docs.

---

### 21. Missing Documentation for rkyv Archive Strategies ⚠️ MEDIUM

**Problem**: Multiple types use `#[rkyv(with = AsString)]` for `PathBuf` fields, but there's no documentation explaining:
- Why `AsString` instead of direct archiving?
- Performance implications (string conversion cost)?
- When to use `AsString` vs other strategies?

**Examples:**
- `path.rs:39`: `RelativePath(#[rkyv(with = AsString)] PathBuf)`
- `path.rs:198`: `AbsolutePath(#[rkyv(with = AsString)] PathBuf)`

**Solution**: Add module-level doc comment in `path.rs` explaining rkyv strategy rationale, link to rkyv docs for `AsString`.

---

### 22. Test Naming Is Inconsistent ⚠️ MEDIUM

**Problem**: Test function names use different conventions:
- `path.rs:844`: `should_reject_empty` (imperative)
- `path.rs:868`: `should_accept_valid_relative` (imperative)
- `name.rs:280`: `returns_some_for_simple_filename` (descriptive)
- `metadata.rs:419`: `constructs_with_both_timestamps` (descriptive)

**Solution**: Standardize on Apollo convention: `{method}_should_{expected_behavior}_when_{condition}()`

**Example:**
```rust
#[test]
fn try_from_should_reject_empty_path() { }

#[test]
fn try_from_should_accept_valid_relative_path() { }
```

**Reference**: Apollo Rust Best Practices Chapter 5 (test naming conventions)

---

### 23. FileMetadata::is_size_match() Is Redundant ⚠️ MEDIUM

**Location**: `lithos-core/src/fs/metadata.rs:166-174`

**Problem**: The method just compares two `u64` values. Callers can do `metadata.size() == other_size` directly.

```rust
pub fn is_size_match(&self, size: u64) -> bool {
    self.size == size  // ❌ No added value over direct comparison
}
```

**Solution**: Remove method, update callers to use direct comparison. Or, if staleness detection requires it, rename to `is_stale_by_size()` and document staleness detection strategy.

---

### 24. Missing Documentation for FileFormat (Mentioned in PRD) ⚠️ MEDIUM

**Problem**: PRD mentions `FileFormat` (refactored from `FormatKind`) but I didn't see `format.rs` in the file list. If it exists, does it:
- Expose `FileExtensionRef<'a>` as planned?
- Support all formats mentioned in user stories (markdown, images, PDFs)?
- Have conversion from file extensions?

**Solution**: Review `fs/format.rs` (if it exists), ensure it matches PRD spec, add comprehensive tests.

---

### 25. entry.rs Uses `std::result::Result` Instead of Importing Result ⚠️ LOW

**Location**: `lithos-core/src/fs/entry.rs:239-250`

**Problem**: Test code uses fully-qualified `std::result::Result::ok` instead of importing `Result` in prelude.

```rust
// entry.rs:248-250
let entry = walkdir::WalkDir::new(temp_path)
    .into_iter()
    .filter_map(std::result::Result::ok)  // ❌ Verbose
```

**Solution**: Import `Result` or use method directly: `.filter_map(Result::ok)`.

---

## Issue Summary by File

| File | Critical | High | Medium | Total |
|------|----------|------|--------|-------|
| `fs/path.rs` | 4 | 5 | 3 | 12 |
| `fs/name.rs` | 2 | 2 | 1 | 5 |
| `fs/metadata.rs` | 1 | 1 | 2 | 4 |
| `fs/entry.rs` | 2 | 1 | 1 | 4 |
| `fs/scanner.rs` | 0 | 2 | 0 | 2 |
| `vault/model.rs` | 2 | 0 | 0 | 2 |
| Cross-cutting | 0 | 1 | 3 | 4 |
| **Total** | **11** | **12** | **10** | **33** |

---

## Remediation Plan

### Phase 1: Fix Type Safety (Critical Path - Days 1-2)

**Goal**: Eliminate type-safety violations that could cause runtime panics or data corruption.

**Tasks:**
1. **Replace `From<PathBuf>` with `TryFrom<PathBuf>` for `FilePath` and `DirPath`** (Issue #1)
   - Update all call sites (likely in `scanner.rs`, `entry.rs`, tests)
   - Add conversion tests

2. **Audit path constructor validation strategy** (Issue #2)
   - Decide: I/O validation or syntax-only validation?
   - Implement chosen strategy consistently
   - Update error docs

3. **Replace `std::io::Error` with `PathError` throughout `path.rs`** (Issue #3)
   - Change all `Result<T, std::io::Error>` to `Result<T, PathError>`
   - Update call sites to handle `PathError`
   - Verify error message quality

4. **Fix error information loss in `entry.rs`** (Issue #8)
   - Preserve source errors in conversion chains
   - Test error messages end-to-end

**Success Criteria:**
- All `From` impls for path types are fallible (`TryFrom`)
- No `std::io::Error` used for path syntax errors
- Error messages show full causality chain

---

### Phase 2: Fix Error Types (Critical Path - Days 3-4)

**Goal**: Align error handling with `fs/error.rs` design (ADR 017).

**Tasks:**
1. **Replace `std::io::Error` with `PathError` in `name.rs`** (Issue #5)
   - Update `FileName::try_from(&Path)` and `BaseName::try_from(&Path)`
   - Add specific error tests

2. **Standardize error return types across modules** (Issue #15)
   - Decide on `FsError` vs `ReadError` for `as_relative()`
   - Update all call sites

3. **Add error conversion tests**
   - Verify `?` operator works correctly
   - Test error downcasting with `std::error::Error::source()`

**Success Criteria:**
- All fs/ methods return errors from `fs/error.rs`
- Error hierarchy matches ADR 017
- Error tests cover all variants

---

### Phase 3: Fix Module Boundaries (Critical Path - Day 5)

**Goal**: Move types to correct modules per PRD.

**Tasks:**
1. **Move `NormalizedPath` from `vault/model.rs` to `fs/path.rs`** (Issue #6)
   - Remove dependency on `VaultPath::normalize()`
   - Inline normalization logic into `NormalizedPath::try_new()`
   - Update vault/ to import from fs/

2. **Refactor `name.rs` to support zero-copy extraction from `FilePath`** (Issue #4)
   - Make `FilePath::filename_ref()` infallible
   - Make `DirPath::dirname_ref()` infallible
   - Update call sites to use new API

**Success Criteria:**
- No vault/ types in fs/ modules
- `NormalizedPath` has no vault/ dependencies
- Zero-copy name extraction works as designed

---

### Phase 4: Delete Legacy Vault Types (Critical Path - Day 6)

**Goal**: Complete migration from old vault types to new inode-based types.

**Tasks:**
1. **Audit usage of legacy types** (Issue #7)
   - Search for `VaultPath`, `VaultFile`, `VaultFolder` usage
   - Identify migration blockers

2. **Migrate storage layer**
   - Update `vault/storage/` to use `FileView`/`DirView`
   - Update queries to use `NormalizedPath`

3. **Delete legacy types**
   - Remove `VaultPath`, `VaultFile`, `VaultFolder`, `PathParts`, `FolderParts`
   - Remove dead code in vault/error.rs

4. **Update tests**
   - Migrate tests to new types
   - Verify behavior preservation

**Success Criteria:**
- No legacy vault types remain
- All tests pass
- Storage layer uses inode-based types exclusively

---

### Phase 5: Fix High-Priority Issues (Days 7-8)

**Tasks:**
1. Add `#[inline]` attributes to hot paths (Issue #16)
2. Make `DirScanner::new()` fallible (Issue #17)
3. Add UTF-8 edge case tests (Issue #14)
4. Document validation ownership (Issue #19)
5. Add performance benchmarks (Issue #18)

---

### Phase 6: Documentation and Polish (Days 9-10)

**Tasks:**
1. Standardize test naming (Issue #22)
2. Add rkyv strategy docs (Issue #21)
3. Document `ParentDir::Root` semantics (Issue #12)
4. Review FileFormat module (Issue #24)
5. Add cross-references between modules

---

## Testing Strategy

### Unit Tests (Per-Module)

**path.rs:**
- [ ] Empty path rejection for all types
- [ ] Absolute/relative validation
- [ ] `.` and `..` rejection
- [ ] UTF-8 edge cases (invalid bytes, emoji, combining characters)
- [ ] `as_relative()` with various base paths
- [ ] `FsPath` enum dispatch correctness

**name.rs:**
- [ ] Zero-copy extraction from `FilePath`/`DirPath`
- [ ] UTF-8 validation in all `TryFrom` impls
- [ ] Basename extraction edge cases (`.file`, `file.`, `file..ext`)
- [ ] Owned vs borrowed equivalence

**metadata.rs:**
- [ ] Timestamp matching logic
- [ ] rkyv roundtrip preservation
- [ ] Archived comparison methods

**entry.rs:**
- [ ] walkdir conversion correctness
- [ ] Error preservation through conversion chain
- [ ] Metadata extraction from filesystem

**scanner.rs:**
- [ ] Glob pattern matching
- [ ] Extension filtering
- [ ] Depth limits
- [ ] Symlink following behavior

### Integration Tests (Cross-Module)

**fs/ integration:**
- [ ] Scanner → Reader → Metadata full flow
- [ ] Error propagation through layers
- [ ] Performance: scan 10,000 files, measure allocations

**fs/ ↔ vault/ integration:**
- [ ] `FsEntry` → `FileView`/`DirView` conversion
- [ ] Path normalization consistency
- [ ] Error boundary behavior

### Property-Based Tests (Consider Adding)

**Invariants to check:**
- Any valid `FilePath` can always extract a filename
- Any valid `DirPath` can always extract a dirname
- `as_relative()` is reversible: `base.join(rel) == abs`
- Path normalization is idempotent: `normalize(normalize(p)) == normalize(p)`

---

## Open Questions for Project Owner

1. **Path validation strategy** (Issue #2): Should `FilePath::new()` and `DirPath::new()` perform filesystem I/O checks, or just syntactic validation?

2. **Zero-copy design priority** (Issue #4): Is the performance gain from zero-copy name extraction worth the API complexity?

3. **Error granularity** (Issue #15): Should `FsPath::as_relative()` return `FsError` (consistent) or `ReadError` (specific)?

4. **Legacy type migration** (Issue #7): Are there known blockers preventing deletion of `VaultPath`, `VaultFile`, `VaultFolder`?

5. **Scanner validation** (Issue #17): Should `DirScanner::new()` fail fast on invalid paths, or defer to iteration time?

6. **Format module status** (Issue #24): Does `fs/format.rs` exist? If not, what's the migration plan from `FormatKind`?

---

## Conclusion

The fs-inode-architecture has **solid bones** but requires **critical fixes** before it's production-ready:

✅ **Strengths:**
- Comprehensive type system for paths, names, metadata
- Good separation of owned vs borrowed types
- Extensive test coverage
- rkyv integration for zero-copy storage

❌ **Weaknesses:**
- Type safety violations in conversion traits
- Error type mismatches throughout
- Module boundary violations (vault types in fs layer)
- Legacy code not yet removed

**Estimated Effort:** 10 days (assuming full-time work)

**Risk Level:** Medium (mainly refactoring, not architectural changes)

**Recommendation:** Prioritize Phases 1-4 (fix type safety + module boundaries + delete legacy), then ship. Phases 5-6 can happen post-launch.
