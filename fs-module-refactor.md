# fs module refactor — second pass

This document tracks the second-pass remediation of `lithos-core/src/fs/`. The first pass
(`fs-module-review.md`) established the module's foundation. This pass removes dead weight,
fixes real defects, tightens public surface, and enforces best practices identified through
critical review against authoritative Rust file-handling standards.

## What this refactor fixes

The review identified three categories of problem:

1. **Dead code and speculative API** — types, methods, and re-exports with zero callers, adding
   surface area and maintenance cost for no benefit.
2. **Real defects** — a panicking constructor, silent error swallowing, a broken atomic write
   implementation, logically misplaced validation, and a lossy type projection.
3. **Organisation and clarity** — internal helpers needlessly split across functions, misleading
   match arms, doc example errors, and a public API that exposes implementation detail.

## Execution rules

- Each step must compile and pass its verify command before the next step begins.
- No `unwrap()`/`panic!`/`assert!` in production code after this refactor.
- `mise run verify` must be 100% green at the end of every step marked with it.
- Legend: `[ ]` pending · `[~]` in progress · `[x]` done

---

## Step 1 — `Cargo.toml`: promote `tempfile` to a regular dependency

**Status:** `[x]`

**Why first:** `atomic_write` (Step 6) uses `tempfile::NamedTempFile` in production code.
`tempfile` is currently a dev-dependency only. It must be promoted before Step 6 touches
`writer.rs`.

**What:**

In `lithos-core/Cargo.toml`, move `tempfile.workspace = true` from `[dev-dependencies]`
to `[dependencies]`.

**Verify:** `cargo check`

---

## Step 2 — `error.rs`: delete `FsError`; add `PathValidationError::RelativeRoot`

**Status:** `[x]`

**Why:** `FsError` is a one-variant passthrough of `std::io::Error` with no callers anywhere in
the codebase. It adds no context, enforces no invariant, and has no justification for existing.
`RelativeRoot` is required by Step 3 (`try_new_strict` returns `Result` instead of panicking).

**What:**

2a. Delete `FsError` entirely from `error.rs` (lines 7–20 including the `#[expect]` attribute).

2b. Add `RelativeRoot` variant to `PathValidationError` in `error.rs`:

```rust
/// Root path provided to strict validator is not absolute.
#[error("Validator root must be absolute, got: {0}")]
RelativeRoot(std::path::PathBuf),
```

**Verify:** `mise run test:unit:fs`

---

## Step 3 — `validator.rs`: all structural and logic fixes

**Status:** `[x]`

**Why:** The validator has the most issues of any single file: a panicking constructor, a
misplaced Windows path check, dead indirection, an opaque match arm, silent hidden-coupling
in boundary checking, redundant UTF-8 conversion, and a broken doc example.

**What:**

3a. **Replace `new_strict` panic with `try_new_strict` returning `Result`.**

Delete `new_strict`. Add:

```rust
/// Creates a strict validator with root boundary enforcement.
///
/// # Errors
///
/// Returns [`PathValidationError::RelativeRoot`] if `root` is not absolute.
/// Callers are responsible for canonicalizing the root before calling this
/// constructor — a non-canonicalized root containing symlinks may cause
/// `resolve_safe_symlink` to reject valid paths.
pub fn try_new_strict(root: PathBuf) -> Result<Self, PathValidationError> {
    if !root.is_absolute() {
        return Err(PathValidationError::RelativeRoot(root));
    }
    Ok(Self { mode: Mode::Strict { root } })
}
```

Update all callers of `new_strict` in tests and production code to use `try_new_strict`
(unwrap is acceptable in test setup; `?` or `expect` at the call site is preferred).

3b. **Remove `#[non_exhaustive]` from `pub(crate) Mode`.**

The attribute only affects code in *other* crates. `Mode` is `pub(crate)` — the attribute
has no effect and misleads readers. Remove it.

3c. **Delete `check_windows_path_bytes` free function; fold into private `Validator` method.**

`check_windows_path_bytes` (lines 73–76) is a two-line function called by exactly one
caller (`is_windows_absolute_path`). Delete the free function. Move the logic directly into
a private associated function:

```rust
#[inline]
#[must_use]
fn is_windows_absolute(path: &str) -> bool {
    let b = path.as_bytes();
    b.first().is_some_and(u8::is_ascii_alphabetic) && b.get(1) == Some(&b':')
}
```

3d. **Move `validate_vault_path` from free function to `impl Validator` as an associated function.**

Delete the `pub fn validate_vault_path(...)` free function (lines 429–458).
Add as `pub fn validate_vault_path` inside `impl Validator`:

```rust
/// Validates a vault-relative path string.
///
/// Bundles common path constraints: non-empty, not a Windows drive path,
/// not absolute, no traversal (`..`), no hidden components, optional
/// extension requirement.
///
/// # Errors
///
/// Returns [`PathValidationError`] if any constraint is violated.
pub fn validate_vault_path(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), PathValidationError> {
    // 1. Empty check
    if path.is_empty() {
        return Err(PathValidationError::EmptyPath);
    }
    // 2. Windows drive-absolute check (must come before Validator::validate
    //    because on Unix, Path::is_absolute("C:\\foo") is false)
    if Self::is_windows_absolute(path) {
        return Err(PathValidationError::AbsolutePathError(
            std::path::PathBuf::from(path),
        ));
    }
    // 3. Standard validation (traversal, hidden files, absolute paths)
    Self::new_flexible().validate(path)?;
    // 4. Extension check
    if let Some(required_ext) = require_extension {
        let has_ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case(required_ext));
        if !has_ext {
            return Err(PathValidationError::InvalidExtension {
                path: std::path::PathBuf::from(path),
                required: required_ext.into(),
            });
        }
    }
    Ok(())
}
```

Note: this also fixes the ordering bug (Issue V2) — the Windows absolute check now runs
*before* the standard validator, not after it.

3e. **Delete `is_windows_absolute_path` public free function** (lines 78–83).

It is now a private method (`Self::is_windows_absolute`) used only within `validate_vault_path`.

3f. **Clarify `check_absolute_path_policy` match structure (Issue V4).**

Replace the combined `Mode::Flexible | Mode::Strict { .. }` arm with two explicit arms
to make the "strict but outside root" case unambiguous:

```rust
match &self.mode {
    Mode::Strict { root } if path.starts_with(root) => Ok(()),
    Mode::Flexible => Err(PathValidationError::AbsolutePathError(path.to_path_buf())),
    Mode::Strict { .. } => Err(PathValidationError::AbsolutePathError(path.to_path_buf())),
}
```

3g. **Inline `get_relative_validation_path` into `validate` (Issue V3).**

Delete the `get_relative_validation_path` method (lines 203–213). Its single call site
in `validate` becomes an inline match expression:

```rust
let check_path = match &self.mode {
    Mode::Strict { root } => path_ref.strip_prefix(root).unwrap_or(path_ref),
    Mode::Flexible => path_ref,
};
Self::validate_core(check_path)?;
```

3h. **Rename and clarify `check_strict_boundary` (Issue V5).**

The function checks boundary AND runs hidden-file validation on the symlink target —
two distinct security concerns. Make both explicit in the name and add a comment at
the call site in `resolve_safe_symlink`:

```rust
/// Verifies that a canonicalized symlink target stays within the strict root
/// and does not resolve to a hidden file within the vault.
fn check_strict_boundary_and_hidden(&self, resolved: &Path) -> Result<(), PathValidationError>
```

Add comment at call site:
```rust
// Enforce boundary (symlink escape) and vault hidden-file policy on the resolved target.
self.check_strict_boundary_and_hidden(&resolved)?;
```

3i. **Fix the broken doc example in `resolve_safe_symlink` (Issue V8).**

```rust
// Before (wrong — join with absolute path discards current_dir() entirely)
let root = std::env::current_dir()?.join("/path/to/vault");

// After
let root = PathBuf::from("/path/to/vault");
let validator = Validator::try_new_strict(root)?;
```

3j. **Update `validate_vault_path` tests.**

The test module currently tests the free function. Update tests to call
`Validator::validate_vault_path(...)` instead. Test that the Windows absolute
path check fires *before* the standard validator (verifies the ordering fix).

**Verify:** `mise run test:unit:fs`

---

## Step 4 — `mod.rs`: remove deleted/privatised re-exports; update docs

**Status:** `[x]`

**Why:** `mod.rs` is the public surface of the `fs` module. After Steps 2 and 3 it re-exports
types that no longer exist and exposes free functions that are now methods. This step cleans
the surface.

**What:**

4a. **Remove these re-exports entirely** (types deleted or moved to methods):

```rust
// Remove:
pub type FsError = error::FsError;            // FsError deleted in Step 2
pub fn is_windows_absolute_path(...) { ... }  // now private method on Validator
pub fn validate_vault_path(...) { ... }       // now Validator::validate_vault_path
```

4b. **Demote these from `pub` to `pub(crate)`** (no external callers):

```rust
pub(crate) type FsWriter = writer::Writer;
pub(crate) type Json     = types::Json;
pub(crate) type Toml     = types::Toml;
pub(crate) type Yaml     = types::Yaml;
pub(crate) type Markdown = types::Markdown;
```

4c. **Update module doc comment** to reflect:
- `validate_vault_path` is now accessed as `PathValidator::validate_vault_path`
- Remove references to the deleted `FsError` type
- Remove the re-exported free function entries from the module overview

**Verify:** `mise run test:unit:fs`

---

## Step 5 — `types.rs`: make `pub(crate)`; wire `detect()` into `classify_path`

**Status:** `[x]`

**Why:** `Json`, `Toml`, `Yaml`, `Markdown` have no external callers — they are implementation
detail of `parse_structured`. The `detect()` methods exist but are never called; the
content-sniffing fallback described in the module doc does not actually exist.

**What:**

5a. **Make all type structs `pub(crate)`:**

```rust
pub(crate) struct Json;
pub(crate) struct Toml;
pub(crate) struct Yaml;
pub(crate) struct Markdown;
```

Remove the `#[non_exhaustive]` attribute from each — it only applies to `pub` types
visible outside the crate.

5b. **Wire `detect()` into `classify_path` as a content-sniffing fallback.**

`classify_path` currently returns `FormatKind::Unknown` for files with no recognised
extension. Change the signature to accept optional content and use `detect()` as a
fallback when the extension is absent or unknown:

The pipeline in `Reader::parse_structured` already has the content string before calling
`classify`. Pass it through:

```rust
// classify_path becomes:
fn classify_path(path: &Path, content: Option<&str>) -> FormatKind {
    // 1. Extension-first (fast, zero allocation)
    if Json::is_supported(path)     { return FormatKind::Json; }
    if Toml::is_supported(path)     { return FormatKind::Toml; }
    if Yaml::is_supported(path)     { return FormatKind::Yaml; }
    if Markdown::is_supported(path) { return FormatKind::Markdown; }
    if is_likely_binary(path)       { return FormatKind::Binary; }

    // 2. Content-sniffing fallback (extension-less or unknown files)
    if let Some(content) = content {
        if Json::detect(content)    { return FormatKind::Json; }
        if Yaml::detect(content)    { return FormatKind::Yaml; }
        if Toml::detect(content)    { return FormatKind::Toml; }
    }
    FormatKind::Unknown
}
```

Note: detection order for content sniffing is JSON → YAML → TOML. JSON is
unambiguous (`{`/`[`). YAML before TOML because YAML's `---` separator is
unambiguous; TOML's heuristic (`=` without `:`) is most likely to produce
false positives.

Update `Reader::classify` and `Reader::parse_structured` to pass content:

```rust
pub fn classify(&self, path: &Path, content: Option<&str>) -> FormatKind {
    classify_path(path, content)
}
```

5c. **Add tests for the content-sniffing fallback** in `types.rs` tests:
- Extension-less file with JSON content → `FormatKind::Json`
- Extension-less file with YAML content → `FormatKind::Yaml`
- Extension-less file with TOML content → `FormatKind::Toml`
- Extension-less file with no recognisable content → `FormatKind::Unknown`

**Verify:** `mise run test:unit:fs`

---

## Step 6 — `reader.rs`: delete `FileMetadata`; store `Validator`; fix all reader defects

**Status:** `[ ]`

**Why:** Multiple independent defects in `reader.rs` are addressed together because several
interact (e.g., storing `Validator` affects `validate_path` deletion; the `classify` signature
change affects `parse_structured`).

**What:**

6a. **Delete `FileMetadata` struct. Change `Reader::metadata` to return `std::fs::Metadata`.**

`FileMetadata` is a lossy projection of `std::fs::Metadata` — it silently swallows the error
from `.modified()` and exposes fewer fields. Return the real type:

```rust
/// Read metadata for a path without following symlinks.
///
/// Uses `symlink_metadata` to avoid following symlinks, ensuring the caller
/// sees the symlink itself rather than its target.
///
/// # Errors
///
/// Returns an error if metadata cannot be read.
pub fn metadata(&self, path: &Path) -> Result<std::fs::Metadata, io::Error> {
    std::fs::symlink_metadata(self.resolve_path(path))
}
```

Update the schema ingestor call site (the only production caller):

```rust
// Before
.and_then(|meta| meta.modified)

// After
.and_then(|meta| meta.modified().ok())
```

Update reader tests: the two metadata tests currently assert `metadata.size` and
`metadata.is_symlink`. Update to use `metadata.len()` and
`metadata.file_type().is_symlink()`.

6b. **Store `Validator` on `Reader`; add `Reader::new_strict`; delete `Reader::validate_path`.**

`validate_path` is hardcoded to `Flexible` mode regardless of context and has zero callers
outside the fs module. Delete it. Store the validator at construction time:

```rust
pub struct Reader {
    root: PathBuf,
    validator: Validator,
}

impl Reader {
    /// Creates a reader with flexible path validation (allows external symlinks).
    pub fn new(root: &Path) -> Self {
        Self { root: root.to_path_buf(), validator: Validator::new_flexible() }
    }

    /// Creates a reader with strict path validation (enforces root boundary).
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError::RelativeRoot`] if `root` is not absolute.
    pub fn new_strict(root: PathBuf) -> Result<Self, PathValidationError> {
        let validator = Validator::try_new_strict(root.clone())?;
        Ok(Self { root, validator })
    }
    // validate_path: deleted
}
```

6c. **Fix `list_files` to propagate per-entry glob errors.**

Silent error swallowing hides real failures (permission denied, broken symlinks) from
the ingestion pipeline. Replace `entry.ok()?` with proper error propagation:

```rust
let mut paths: Vec<PathBuf> = glob::glob(pattern_str)
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
    .map(|entry| -> io::Result<Option<PathBuf>> {
        let path = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        if !path.is_file() && !path.is_symlink() {
            return Ok(None);
        }
        Ok(path.strip_prefix(&self.root).ok().map(Path::to_path_buf))
    })
    .filter_map(|r| r.transpose())
    .collect::<io::Result<_>>()?;
```

6d. **Fix `is_likely_binary` to use `eq_ignore_ascii_case` (eliminates allocation).**

```rust
// Before: allocates String on every call
matches!(ext.to_ascii_lowercase().as_str(), "png" | "jpg" | ...)

// After: zero allocation, consistent with rest of module
ext.eq_ignore_ascii_case("png")
    || ext.eq_ignore_ascii_case("jpg")
    || ext.eq_ignore_ascii_case("jpeg")
    || ext.eq_ignore_ascii_case("gif")
    || ext.eq_ignore_ascii_case("pdf")
    || ext.eq_ignore_ascii_case("mp3")
    || ext.eq_ignore_ascii_case("mp4")
    || ext.eq_ignore_ascii_case("zip")
    || ext.eq_ignore_ascii_case("wasm")
```

6e. **Generalise `read_with` error type.**

The closure is currently forced to return `ParseError`. This couples every caller to the
fs error type even when they want to produce a domain error directly. Change to:

```rust
pub fn read_with<T, E, F>(&self, path: &Path, f: F) -> Result<T, E>
where
    F: FnOnce(&Path, &str) -> Result<T, E>,
    E: From<ParseError>,
{
    let content = self.read_to_string(path).map_err(|error| {
        E::from(ParseError::Io { path: path.to_path_buf(), source: error })
    })?;
    f(path, &content)
}
```

The existing `NoteReader::parse` call site continues to compile unchanged:
`read_with(path, |_, content| Ok(content.to_owned()))` — `Ok` produces any `E`.

6f. **Make `Reader::classify`, `FormatKind`, and `Reader::read_bytes` `pub(crate)`.**

No external callers exist. These are implementation details:

```rust
pub(crate) enum FormatKind { ... }
// Reader::classify: pub → pub(crate)
// Reader::read_bytes: pub → pub(crate)
```

Update `Reader::classify` to pass `content: Option<&str>` to `classify_path` (from Step 5).

6g. **Remove redundant `FileMetadata` re-export from `mod.rs`.**

```rust
// Remove:
pub type FileMetadata = reader::FileMetadata;
```

**Verify:** `mise run test:unit:fs && mise run test:unit:schema && mise run test:unit:note`

---

## Step 7 — `writer.rs`: store `Validator`; fix `atomic_write`; make `pub(crate)`; add tests

**Status:** `[ ]`

**Why:** `Writer` has three independent problems: its validator is reconstructed per-call,
its `atomic_write` uses a fragile hand-rolled temp file, and it has zero unit tests for
its production code.

**What:**

7a. **Store `Validator` on `Writer`; remove per-call construction.**

```rust
pub(crate) struct Writer {
    root: PathBuf,
    validator: Validator,
}

impl Writer {
    pub(crate) fn new(root: &Path) -> Self {
        Self { root: root.to_path_buf(), validator: Validator::new_flexible() }
    }

    fn validate_path(&self, path: &Path) -> io::Result<()> {
        self.validator
            .validate(path)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }
}
```

7b. **Replace `atomic_write` hand-rolled temp file with `tempfile::NamedTempFile::new_in`.**

The current implementation generates a nanosecond-timestamp temp name, which is not
unique under concurrent access and orphans the temp file if the process is killed between
`open` and `rename`. Replace:

```rust
pub(crate) fn atomic_write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    self.validate_path(path)?;
    let target = self.resolve(path);
    let parent = target.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "path has no parent")
    })?;

    // NamedTempFile::new_in uses O_CREAT|O_EXCL with a cryptographically
    // unique name. Its Drop impl deletes the temp file if persist() is never
    // called, preventing orphaned temp files on panic or early return.
    let mut tmp = NamedTempFile::new_in(parent)?;
    tmp.write_all(contents)?;
    tmp.as_file().sync_all()?; // durability: flush data before rename
    tmp.persist(&target).map_err(|e| e.error)?;
    Ok(())
}
```

7c. **Make `Writer` and `FsWriter` alias `pub(crate)`** (no external callers; will be
promoted to `pub` when the template module provides a real caller).

7d. **Add unit tests for `Writer`** covering all public methods. `writer.rs` currently has
zero tests — a clear violation of the project's Definition of Done.

Required test cases:

- `write_file`: creates file with correct content
- `write_file`: overwrites existing file (truncates)
- `write_file`: rejects path traversal (`../escape`)
- `write_file`: rejects hidden file paths (`.secret`)
- `atomic_write`: content is correct after successful write
- `atomic_write`: no orphaned temp file remains in parent dir after success
- `atomic_write`: rejects path traversal
- `create_dir_all`: creates nested directories
- `rename`: file appears at new path; old path is gone
- `remove_file`: file is deleted
- `remove_file`: returns error for non-existent file

**Verify:** `mise run test:unit:fs`

---

## Step 8 — `template/adapter/filters.rs`: update `validate_vault_path` call site

**Status:** `[x]`

**Why:** The only external caller of `validate_vault_path` uses it as a free function from
`crate::fs`. After Step 3 it is an associated function on `Validator`.

**What:**

Update `template/adapter/filters.rs:216`:

```rust
// Before
crate::fs::validate_vault_path(&path, None).map_err(|e| { ... })?;

// After
crate::fs::PathValidator::validate_vault_path(&path, None).map_err(|e| { ... })?;
```

No other changes to this file.

**Verify:** `mise run test:unit:template`

---

## Step 9 — Extract shared test fixture

**Status:** `[ ]`

**Why:** `validator.rs` has a 108-line `Workspace` test fixture. `reader.rs` has a `write_file`
helper. Both provide filesystem setup for tests but are duplicated. A shared `#[cfg(test)]`
module eliminates the duplication.

**What:**

9a. Create `lithos-core/src/fs/test_support.rs` (behind `#[cfg(test)]`) with:

- `Workspace` struct (from `validator.rs`): `new()`, `create_file()`, `create_symlink()`
- `write_file(root: &Path, relative: &str, contents: &[u8]) -> PathBuf` (from `reader.rs`)

9b. In `validator.rs` tests: replace inline `Workspace` definition with
`use super::test_support::Workspace;`.

9c. In `reader.rs` tests: replace inline `write_file` with
`use super::test_support::write_file;`.

9d. Add `#[cfg(test)] mod test_support;` in `mod.rs`.

**Verify:** `mise run test:unit:fs`

---

## Step 10 — Final verification

**Status:** `[ ]`

**What:**

10a. Run the full quality gate: `mise run verify`

10b. Run doc tests: `cargo test --doc`

10c. Check that no public items in `lithos-core::fs` lack documentation
(`cargo doc --no-deps 2>&1 | grep "missing"` — must be empty).

10d. Confirm the Definition of Done checklist:

- [ ] `FsError` deleted — no references remain
- [ ] `FileMetadata` deleted — no references remain
- [ ] `validate_vault_path` and `is_windows_absolute_path` accessible only
      via `PathValidator::validate_vault_path`
- [ ] `new_strict` replaced by `try_new_strict` — no `assert!`/`panic!` in
      production validation code
- [ ] `atomic_write` uses `NamedTempFile::new_in`, not hand-rolled temp name
- [ ] `detect()` wired into `classify_path` as content-sniffing fallback
- [ ] `Writer` has tests covering all methods including `atomic_write`
- [ ] Items demoted from `pub` to `pub(crate)`:
      `Writer`/`FsWriter`, `Json`, `Toml`, `Yaml`, `Markdown`,
      `FormatKind`, `Reader::classify`, `Reader::read_bytes`
- [ ] `list_files` propagates per-entry errors
- [ ] `is_likely_binary` uses `eq_ignore_ascii_case` — no allocation
- [ ] `read_with` closure error type is generic `E: From<ParseError>`
- [ ] Shared test fixture extracted; no duplication between `validator.rs`
      and `reader.rs` tests
- [ ] `#[non_exhaustive]` removed from `pub(crate) Mode`
- [ ] No `unwrap()`/`panic!`/`assert!` in production code
- [ ] No string allocation anti-patterns introduced
- [ ] `mise run verify` 100% green
- [ ] `cargo test --doc` passes

**Verify:** `mise run verify && cargo test --doc`

---

## Execution order

```
Step 1  (Cargo.toml: tempfile promotion)
Step 2  (error.rs: delete FsError, add RelativeRoot)     ← needs Step 1 to be coherent
Step 3  (validator.rs: all structural fixes)              ← needs Step 2 (RelativeRoot)
Step 4  (mod.rs: remove/demote re-exports)               ← needs Step 3 (types deleted/moved)
Step 5  (types.rs: pub(crate), wire detect())            ← independent, can follow Step 4
Step 6  (reader.rs: all reader fixes)                    ← needs Steps 3, 4, 5
Step 7  (writer.rs: Validator storage, atomic_write, tests) ← needs Steps 3, 4
Step 8  (filters.rs: call site update)                   ← needs Step 3
Step 9  (shared test fixture)                            ← needs Steps 6, 7 (tests exist)
Step 10 (final verification)                             ← needs all prior steps
```

Steps 7 and 8 can be done in parallel with Step 6 once Steps 3 and 4 are complete.

---

## Files changed (complete list)

| File                                              | Step(s)    | Change summary                                                                                                           |
| ------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------ |
| `lithos-core/Cargo.toml`                          | 1          | `tempfile` promoted from `[dev-dependencies]` to `[dependencies]`                                                        |
| `lithos-core/src/fs/error.rs`                     | 2          | Delete `FsError`; add `PathValidationError::RelativeRoot`                                                                |
| `lithos-core/src/fs/validator.rs`                 | 3          | Delete `check_windows_path_bytes` free fn; delete `is_windows_absolute_path` pub fn; delete `new_strict`; add `try_new_strict`; add `Validator::validate_vault_path`; add private `Self::is_windows_absolute`; fix ordering; clarify match arms; inline `get_relative_validation_path`; rename `check_strict_boundary`; fix doc example; remove `#[non_exhaustive]` from `Mode` |
| `lithos-core/src/fs/mod.rs`                       | 4          | Remove `FsError`, `FileMetadata`, `is_windows_absolute_path`, `validate_vault_path` re-exports; demote `FsWriter`, `Json`, `Toml`, `Yaml`, `Markdown` to `pub(crate)`; update module doc |
| `lithos-core/src/fs/types.rs`                     | 5          | Make all structs `pub(crate)`; remove `#[non_exhaustive]`; wire `detect()` into `classify_path`; add content-sniffing tests |
| `lithos-core/src/fs/reader.rs`                    | 6          | Delete `FileMetadata`; `metadata()` returns `std::fs::Metadata`; store `Validator` on `Reader`; add `Reader::new_strict`; delete `Reader::validate_path`; fix `list_files` error propagation; fix `is_likely_binary` allocation; generalise `read_with` error type; make `classify`, `FormatKind`, `read_bytes` `pub(crate)`; update `classify` signature for content arg; update tests |
| `lithos-core/src/fs/writer.rs`                    | 7          | Store `Validator` on `Writer`; replace `atomic_write` with `NamedTempFile`; make `Writer`/`FsWriter` `pub(crate)`; add comprehensive unit tests |
| `lithos-core/src/fs/test_support.rs`              | 9          | New `#[cfg(test)]` module: shared `Workspace` and `write_file` fixtures                                                  |
| `lithos-core/src/schema/adapter/ingestor.rs`      | 6          | Update `metadata` call site: `meta.modified` → `meta.modified().ok()`                                                    |
| `lithos-core/src/template/adapter/filters.rs`     | 8          | Update `validate_vault_path` call: `crate::fs::validate_vault_path` → `crate::fs::PathValidator::validate_vault_path`    |

## Files that do NOT need changes

| File                                              | Reason                                                                                     |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `lithos-core/src/note/adapter/reader.rs`          | `read_with` call site compiles unchanged after generalisation (closure returns `Ok(...)`)   |
| `lithos-core/src/note/` (all other files)         | No fs symbols used directly                                                                |
| `lithos-core/src/config/` (all files)             | No affected symbols                                                                        |
| `lithos-core/tests/architecture.rs`               | No affected imports                                                                        |
| `lithos-cli/src/`                                 | No fs symbols used                                                                         |
