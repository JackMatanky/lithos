# fs module review

This document captures the fs module review, including the full set of problems and the agreed solutions after clarifying the read/parse pipeline, boundary rules, and public surface constraints.

## Scope and intent

The fs module must provide capabilities beyond std::fs by centralizing:
- root scoping and path validation policy
- deterministic discovery (glob-based, stable order)
- a well-defined read pipeline (validation → classify → read → parse)
- safe write orchestration (atomic replace)
- consistent metadata access

Domain types must not import fs. Context-specific parsing (notably Markdown options) remains a context concern.

## Problems and solutions

### 1) Path validation split and context boundary violations

Problem:
- `NotePath::new()` imports `crate::fs::validate_vault_path` (infrastructure dependency in a domain type).
- `validate_vault_path` and `PathValidator` duplicate logic and disagree on `..` handling.

Solutions:
- Inline domain-level validation in `NotePath::new()` (relative, `.md`, no `..`, no dotfiles). No fs import.
- Move `validate_vault_path` into `validator.rs` and make it delegate to `PathValidator`.
- Fix `validate_vault_path` to check `Component::ParentDir` rather than substring `".."`.
- Fix Windows drive-relative detection (reject `C:relative` and `C:\absolute`).

### 2) Read vs parse pipeline ambiguity

Problem:
- Reading and parsing were conflated, leading to confusion over responsibilities and public APIs.

Solutions:
- Define the pipeline explicitly in `FsReader`:
  1) `validate_path`
  2) `classify(path)`
  3) `read_bytes` / `read_to_string`
  4) `parse_structured<T>` (JSON/TOML/YAML only)
  5) `read<T>` convenience method that dispatches via classification and parsing
- Keep parsing for JSON/TOML/YAML in fs, but allow context-specific Markdown parsing through a closure hook.

### 3) Format identification should be explicit and type-safe

Problem:
- A generic “Structured” bucket is too coarse, and risks parsing JSON as TOML or YAML.

Solutions:
- Keep Json/Toml/Yaml structs and rely on existing `is_supported(path)` methods.
- `FsReader::classify(path)` uses the public type-level predicates (e.g., `Json::is_supported(path)`), and each type’s `parse` must call its own `is_supported` guard to prevent mismatches.
- Ensure `parse_structured<T>` checks the file type before parsing to prevent mismatches.

### 4) Parsing should not force context-specific Markdown decisions

Problem:
- Markdown parsing is context-dependent (Obsidian options), and fs should not own those policies.

Solutions:
- FsReader exposes `read_with<T>(path, f)` where `f` is a closure `(path, text) -> Result<T, ParseError>`.
- Contexts inject their own Markdown parsing without fs depending on pulldown-cmark.
- FsReader can still classify Markdown by extension for pipeline dispatch.

### 5) FsReader/FsWriter split and value beyond std

Problem:
- The current `FileSource` is a thin wrapper and not worth its maintenance cost.

Solutions:
- Rename `FileSource` to `FsReader` in `reader.rs`.
- Introduce `FsWriter` in `writer.rs` with safe write orchestration.
- Ensure fs adds value beyond std via:
  - root-scoped validated paths for all operations
  - deterministic list ordering
  - glob-based discovery with root-relative patterns
  - metadata access via a stable `FileMetadata` struct
  - atomic replace for safe writes

### 6) Glob discovery and traversal correctness

Problem:
- `list_files` compiles the glob per entry and silently drops errors.
- `walkdir` is unnecessary for simple globbing.

Solutions:
- Use `glob::glob()` with a single compiled pattern.
- Return errors on invalid patterns.
- Make ordering deterministic by sorting results.
- Use root-relative patterns and strip the root from results for consistency.

### 7) Symlink policy must be context-dependent

Problem:
- Hard-coded symlink exclusion is incorrect for some contexts (e.g., config files often symlinked).

Solutions:
- Default to **including symlinks** in discovery.
- Add a policy hook or configuration for stricter contexts later.
- Document the future policy option in fs docs.

### 8) Atomic write durability decision

Problem:
- `rename` is not a full durability guarantee; parent-dir fsync may be required for crash safety.

Recommendation:
- Default to a lean, safe approach:
  - write temp file with `create_new`
  - `sync_all` on the file
  - `rename` to target
- Avoid parent-directory fsync by default for performance, but keep API space for a stricter mode later if needed.

### 9) Tests must use tempfile only

Problem:
- In-memory sources diverge from real filesystem behavior.

Solutions:
- Remove in-memory implementations.
- Replace tests with `tempfile::TempDir` integration tests.
- Ensure tests cover glob patterns, symlink handling, and read/parse pipeline dispatch.

## File-level plan (summary)

### fs/reader.rs (renamed from source.rs)
- Rename `FileSource` -> `FsReader`.
- Add pipeline methods: `validate_path`, `classify`, `read_bytes`, `read_to_string`, `parse_structured<T>`.
- Add convenience `read<T>` that dispatches by file type.
- Add `read_with<T>` closure hook for Markdown parsing.
- Add `metadata()`.
- Fix `list_files()` to use `glob::glob()` with deterministic ordering.

### fs/types.rs (renamed from parsers.rs)
- Keep Json/Toml/Yaml structs only (no FsParser).
- Use existing `is_supported(path)` in each type’s `parse` method (no separate helpers).
- Keep parse methods for each struct and ensure they validate the file type.

### fs/writer.rs (new)
- Add `FsWriter` and `OsFsWriter`.
- Implement `atomic_write` (no parent-dir fsync by default).

### fs/validator.rs
- Move `validate_vault_path` here and delegate to `PathValidator`.
- Fix Windows drive-relative check.

### note/aggregate.rs
- Inline `NotePath::new` domain validation with no fs dependency.

## Open questions resolved by this review

1) **Format classification**: `FsReader::classify(path)` with type helpers in `types.rs`.
2) **Convenience read**: provide `read<T>(path)` that dispatches to the correct read/parse path.
3) **Markdown parsing hook**: use closure-based `read_with<T>` to avoid coupling to pulldown-cmark.
4) **Public surface**: keep FormatKind internal where possible; prefer `FsReader` as the main entry point.

---

## Cross-module impact analysis

A full codebase search was performed to find every file outside `lithos-core/src/fs/` that
references symbols changed by this refactor. Results are listed with their impact.

### Files outside fs/ that are affected

| File | Affected symbols | Requires code change? |
|---|---|---|
| `lithos-core/src/note/aggregate.rs` | `crate::fs::validate_vault_path` (line 369) | **Yes** — step 4 replaces with inline validation |
| `lithos-core/src/note/parser.rs` | `crate::fs::MarkdownParser` (line 23) | **No** — `pub(crate)` is visible within the same crate; no change required |
| `lithos-core/src/schema/adapter/ingestor.rs` | `FileSource`, `parse_file` (lines 7, 54, 67, 105) | **Yes** — steps 5h and 6i |
| `lithos-core/src/schema/error.rs` | `From<ParseError>` impl (line 254) — destructures `Box<Path>` and `Vec<&'static str>` | **No** — `PathBuf::to_string_lossy()` and `<[_]>::join()` both compile after step 1 changes |
| `lithos-core/src/template/adapter/filters.rs` | `crate::fs::validate_vault_path` (line 216) | **No** — uses `format!("{e}")` which works for any `Display` type; compiles after step 3 changes return type |
| `lithos-core/src/application/mod.rs` | doc comment mentions `FileSource` (line 31) | **Minor** — update in step 10b, add to files-changed table |
| `lithos-core/tests/architecture.rs` | Uses `FileSource` only in assertion message strings | **No** — not a Rust import; unaffected |
| `lithos-cli/src/` | Zero usages of any affected symbol | **No** |

### Gap: steps 3 and 4 must be treated as an atomic unit

After step 3 changes `validate_vault_path` to return `Result<(), PathValidationError>`,
the existing code in `note/aggregate.rs:370` will **fail to compile**:

```rust
// Currently compiles: NoteError::InvalidPath(String) ← e: String
crate::fs::validate_vault_path(&normalized, Some("md"))
    .map_err(|e| NoteError::InvalidPath(e.clone()))?;

// After step 3: NoteError::InvalidPath(String) ← e: PathValidationError ← COMPILE ERROR
```

`PathValidationError` is `Clone`, so `.clone()` is fine, but `NoteError::InvalidPath` takes
`String`, not `PathValidationError`. Therefore:

- **Step 3 verify command `mise run test:unit:fs` is insufficient** — it does not compile the note
  module. A `cargo check` after step 3 alone would fail.
- **Mitigation**: treat steps 3 and 4 as a single atomic commit. Complete both before running
  `cargo check` or `mise run verify`. The verify command listed in step 4
  (`mise run test:unit:note`) covers both.

### Gap: step 10a wording is incorrect

Step 10a says "Remove `pub mod markdown;` from `mod.rs`". This is wrong — removing the `mod`
declaration entirely would break `note/parser.rs` which imports `crate::fs::MarkdownParser`.

**Correct action**: change `pub mod markdown;` to `pub(crate) mod markdown;` and change the
type aliases in `mod.rs`:

```rust
// before
pub type MarkdownParser = markdown::MarkdownParser;
pub type MarkdownOffsetIter<'markdown> = markdown::MarkdownOffsetIter<'markdown>;

// after
pub(crate) type MarkdownParser = markdown::MarkdownParser;
pub(crate) type MarkdownOffsetIter<'markdown> = markdown::MarkdownOffsetIter<'markdown>;
```

`note/parser.rs` accesses `crate::fs::MarkdownParser` — `pub(crate)` is sufficient since both
files are in the same crate. **No change to `note/parser.rs` is required.**

### Gap: `ParseError::UnsupportedFormat.supported` type change

The plan uses `supported: &["json"]` (a `&'static [&'static str]`) in new code (steps 5e, 6d),
but the current `error.rs` field is `supported: Vec<&'static str>`. Step 1 must also change
this field:

```rust
// step 1 change to error.rs
UnsupportedFormat {
    path: PathBuf,                      // was Box<Path>
    supported: &'static [&'static str], // was Vec<&'static str>
}
```

The `From<ParseError>` impl in `schema/error.rs:295` uses `supported.join(", ")` — this
compiles unchanged because `<[&str]>::join()` exists for slices.

### Gap: step 5h — `Ingestor` temporary bridge needs explicit struct change

Step 5h adds a "temporary bridge" with inline `std::fs::read_to_string` calls. But
`Ingestor<'config, S>` has a bound `S: FileSource<Error = io::Error>` at line 54 that
references the now-deleted `FileSource` trait. After step 5 removes `FileSource`, the struct
bound must be removed entirely and the `source` field must either:

- Be removed and replaced with direct `std::fs` calls in the impl body, OR
- Keep a `PhantomData<S>` placeholder if `S` is still needed

**Minimum change for step 5**: remove the `source: S` field and `impl<S: FileSource...>` bound
from `Ingestor`. The struct becomes:

```rust
pub struct Ingestor<'config> {
    root: PathBuf,   // temporary: hardcoded from config.paths()
    config: &'config Config,
}
```

Step 6i then replaces this with `source: S where S: FsReader`, restoring the abstraction.

This must be reflected in step 5h and step 6i in the plan.

---

## Refactoring plan

This plan is executable in strict step order. Each step compiles and passes
tests before the next begins. Steps are grouped by dependency: earlier steps
unblock later ones. Completion status is tracked inline.

Legend: `[ ]` pending · `[x]` done

---

### Step 1 — `error.rs`: fix `ParseError` path types and `UnsupportedFormat.supported` type

**Status:** `[ ]`

**What:**
- Replace `Box<std::path::Path>` with `PathBuf` in all four `ParseError`
  variants (`Io`, `Json`, `Toml`, `Yaml`, `UnsupportedFormat`).
- Change `UnsupportedFormat.supported` from `Vec<&'static str>` to
  `&'static [&'static str]` (a static slice). This aligns with the new
  pipeline code in `reader.rs` and is required for consistency.
  Note: `schema/error.rs:295` uses `supported.join(", ")` which compiles
  unchanged for slices — no change to `schema/error.rs` required.
- Change `path: path.into()` at every construction site in `parsers.rs`
  (lines 236, 283, 328, 373, 202) from `Box<Path>` to `PathBuf`.
  Both use `.into()` so only the field type changes.
- Add a `# Note` comment on `PathValidationError::IoError(String)` explaining
  that `io::Error` cannot be stored directly due to the required
  `Clone + Eq` bounds on `PathValidationError`.

**Why before other steps:**
`ParseError` is constructed inside `parsers.rs`; fixing the type here avoids
a second touch when `parsers.rs` is restructured in step 3.

**Verify:** `mise run test:unit:fs`

---

### Step 2 — `validator.rs`: fix `validate()` return type, visibility, tracing, and `CwdGuard`

**Status:** `[ ]`

**What:**

2a. Change `validate()` signature from
```rust
pub fn validate<'path, PathType>(&self, path: &'path PathType)
    -> Result<Cow<'path, Path>, PathValidationError>
```
to
```rust
pub fn validate<PathType>(&self, path: &PathType)
    -> Result<(), PathValidationError>
```
Update the single internal caller at `validator.rs:325`:
`let _validated: Cow<'_, Path> = self.validate(path_ref)?;`
becomes `self.validate(path_ref)?;`
Remove the `use std::borrow::Cow;` import if no longer used.

2b. Change `pub enum Mode` to `pub(crate) enum Mode` (line 78).
Remove `pub type ValidationMode = Mode;` (line 91) — it has zero callers.

2c. Remove `tracing::warn!` (line 145) and `tracing::debug!` (line 403).
Remove the `use tracing::{debug, warn};` import (line 72).

2d. Fix `CwdGuard::drop` (line 833):
```rust
// before
if std::env::set_current_dir(&self.previous).is_err() {}
// after
if let Err(e) = std::env::set_current_dir(&self.previous) {
    eprintln!("CwdGuard: failed to restore CWD: {e}");
}
```

**Why before step 3:**
Step 3 moves `validate_vault_path` into `validator.rs` and calls
`self.validate()`; the return type must be settled first.

**Verify:** `mise run test:unit:fs`

---

### Step 3 — `validator.rs` + `mod.rs`: move path helpers, fix `validate_vault_path`

> ⚠️ **Atomicity note**: Steps 3 and 4 must be committed together. After step 3
> changes `validate_vault_path` to return `Result<(), PathValidationError>`, the
> call at `note/aggregate.rs:370` — `NoteError::InvalidPath(e.clone())` where
> `NoteError::InvalidPath(String)` — will fail to compile because `e` is now
> `PathValidationError`, not `String`. Step 4 fixes this by replacing the call
> entirely. Do not run `cargo check` on a step-3-only tree; run it only after
> step 4 is complete. The verify command is the step 4 command:
> `mise run test:unit:fs && mise run test:unit:note`.

**Status:** `[ ]`

**What:**

3a. Move from `mod.rs` into `validator.rs` (make private):
- `check_windows_separator` — inline its body directly into
  `check_windows_path_bytes` (it is a one-line predicate);
  delete the separate function.
- `check_windows_path_bytes` — keep as `fn` (private).
- `is_windows_absolute_path` — keep as `pub fn`.

3b. Fix `is_windows_absolute_path` / `check_windows_path_bytes` to also catch
drive-relative paths (e.g., `C:relative` where position 2 is neither `/`
nor `\`). New logic:
```rust
fn check_windows_path_bytes(bytes: &[u8]) -> bool {
    // Matches C:\ C:/ C:relative — any path starting with [alpha]:
    bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes.get(1) == Some(&b':')
}
```
This correctly rejects all Windows absolute AND drive-relative paths.

3c. Add `validate_vault_path` to `validator.rs` as a `pub fn`:
```rust
pub fn validate_vault_path(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), PathValidationError>
```
Implementation:
- Reject empty string → `PathValidationError::AbsolutePathError` (reuse
  or add a dedicated `EmptyPath` variant — prefer dedicated variant).
- Call `Validator::new_flexible().validate(path)?` for traversal, hidden
  file, absolute path, and encoding checks (replaces the duplicate substring
  `..` check and the `starts_with('/')` check).
- Call `is_windows_absolute_path(path)` and return
  `PathValidationError::AbsolutePathError` if true (covers the drive-relative
  case not caught by `Component` iteration).
- Check extension if `require_extension` is `Some`, using
  `Path::new(path).extension()` as today.
- Return type is `Result<(), PathValidationError>` (not `String`).

3d. In `mod.rs`:
- Remove the four functions that moved to `validator.rs`.
- Add `pub use validator::validate_vault_path;` and
  `pub use validator::is_windows_absolute_path;` re-exports so existing
  call sites (`crate::fs::validate_vault_path`) still compile.
- Fix `MarkdownOffsetIter` double alias: currently
  `mod.rs:43` aliases `markdown::MarkdownOffsetIter` which itself aliases
  `pulldown_cmark::OffsetIter`. Define it once directly in `mod.rs`:
  `pub type MarkdownOffsetIter<'a> = pulldown_cmark::OffsetIter<'a, pulldown_cmark::DefaultBrokenLinkCallback>;`
  and remove the redundant alias from `markdown.rs`.
- Remove `pub type ValidationMode = Mode;` re-export (dead alias removed in
  step 2b, so this is already gone).

**Verify:** `mise run test:unit:fs`

---

### Step 4 — `note/aggregate.rs`: remove `crate::fs` domain boundary violation

**Status:** `[ ]`

**What:**
Replace the call to `crate::fs::validate_vault_path` at
`note/aggregate.rs:369` with inline domain-level checks using only `std::path`:

```rust
fn validate_note_path(path: &str) -> Result<(), NoteError> {
    use std::path::{Component, Path};

    if path.is_empty() {
        return Err(NoteError::InvalidPath("path cannot be empty".into()));
    }
    let p = Path::new(path);
    // must be relative
    if p.is_absolute() {
        return Err(NoteError::InvalidPath("path must be relative".into()));
    }
    // no .. traversal, no dotfile components
    for component in p.components() {
        match component {
            Component::ParentDir => {
                return Err(NoteError::InvalidPath(
                    "path traversal not allowed".into(),
                ));
            }
            Component::Normal(s)
                if s.to_str().is_some_and(|s| s.starts_with('.')) =>
            {
                return Err(NoteError::InvalidPath(
                    "hidden path components not allowed".into(),
                ));
            }
            _ => {}
        }
    }
    // must have .md extension
    if !p
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("md"))
    {
        return Err(NoteError::InvalidPath(
            "path must have .md extension".into(),
        ));
    }
    Ok(())
}
```

Add the comment:
```rust
// TODO(future): filesystem-level validation (symlink resolution, vault
// boundary checking) belongs in the note ingestion adapter using
// PathValidator::new_strict(vault_root).
```

Remove `crate::fs` import from `note/aggregate.rs`.
`NoteError::InvalidPath(String)` stays unchanged.

**Verify:** `mise run test:unit:note`

---

### Step 5 — `parsers.rs` → `types.rs`: rename and refocus

**Status:** `[ ]`

**What:**

5a. Rename the file: `lithos-core/src/fs/parsers.rs` →
`lithos-core/src/fs/types.rs`.

5b. Remove `Dispatcher` struct and its `impl` block entirely (lines 80–206).

5c. Remove the `parse_file` free function (lines 368–379) and its
`use super::source::FileSource` import (line 69) and `use std::io` import
(line 64). The `FileSource` coupling is eliminated here.

5d. Remove `tracing::{debug, error}` import (line 67) and all `debug!()` /
`error!()` calls within `Dispatcher::parse`.

5e. In each type's `parse` method, add an `is_supported` guard at the top
that returns `ParseError::UnsupportedFormat` if the path extension does not
match, preventing cross-format misparse:
```rust
// Example for Json::parse
pub fn parse<T: DeserializeOwned>(path: &Path, content: &str)
    -> Result<T, ParseError>
{
    if !Self::is_supported(path) {
        return Err(ParseError::UnsupportedFormat {
            path: path.to_path_buf(),  // PathBuf after step 1
            supported: &["json"],
        });
    }
    serde_json::from_str(content).map_err(|e| ParseError::Json { ... })
}
```
Do the same for `Toml::parse` and `Yaml::parse`.

5f. Keep `Json`, `Toml`, `Yaml` structs and their `is_supported`, `detect`,
and `parse` methods. Keep all existing tests, updating module path from
`parsers` to `types` in `mod.rs`.

5g. Update `mod.rs`:
- Change `pub mod parsers;` to `pub mod types;`.
- Change `pub type FormatDispatcher = parsers::Dispatcher;` — remove this
  alias entirely (Dispatcher is gone).
- Add re-exports: `pub use types::{Json, Toml, Yaml};`

5h. Update `schema/adapter/ingestor.rs` (lines 7, 54, 67, 105):
- Remove `use crate::fs::parsers::parse_file;` import (line 7).
- Remove `use crate::fs::source::FileSource;` import (line 7).
- The `impl<S: FileSource<Error = std::io::Error>> Ingestor<'_, S>` bound (line 54)
  references the deleted `FileSource` trait. Remove the generic parameter `S` from
  `Ingestor` entirely. The struct becomes:
  ```rust
  pub struct Ingestor<'config> {
      config: &'config Config,
  }
  impl<'config> Ingestor<'config> {
      pub const fn new(config: &'config Config) -> Self { Self { config } }
  }
  ```
  Note: the `source: S` field is dropped; I/O is done directly via `std::fs`.
- Replace `parse_file(&self.source, &path)` calls with equivalent inline calls
  using `std::fs::read_to_string` + `Json::parse`/`Yaml::parse`/`Toml::parse`:
  ```rust
  use crate::fs::types::{Json, Toml, Yaml};
  let content = std::fs::read_to_string(&path)
      .map_err(|e| SchemaIngestionError::Io {
          path: path.to_string_lossy().into(),
          reason: e.to_string().into(),
      })?;
  let raw = if Json::is_supported(&path) {
      Json::parse(&path, &content).map_err(SchemaIngestionError::from)?
  } else if Toml::is_supported(&path) {
      Toml::parse(&path, &content).map_err(SchemaIngestionError::from)?
  } else if Yaml::is_supported(&path) {
      Yaml::parse(&path, &content).map_err(SchemaIngestionError::from)?
  } else {
      return Err(SchemaIngestionError::UnsupportedFormat {
          path: path.to_string_lossy().into(),
          supported: "json, toml, yaml, yml".into(),
      });
  };
  ```
- Replace `self.source.list_files(&pattern)` with `glob::glob(&pattern)` inline.
  This is a **temporary bridge** — the abstraction is restored in step 6i via `FsReader`.
- Update all call sites in tests to use `Ingestor::new(&config)` (no `source` arg).
  Tests that previously used `InMemoryFileSource` will be rewritten in step 9c.

**Verify:** `mise run test:unit:fs && mise run test:unit:schema`

---

### Step 6 — `source.rs` → `reader.rs`: introduce `FsReader` with full pipeline

**Status:** `[ ]`

**What:**

6a. Rename the file: `lithos-core/src/fs/source.rs` →
`lithos-core/src/fs/reader.rs`.

6b. Rename `FileSource` → `FsReader` and `FsFileSource` → `OsFsReader`
throughout the file. Remove `InMemoryFileSource` entirely
(its tests are replaced in step 9).

6c. Add `FormatKind` as a `pub(crate)` enum in `reader.rs`:
```rust
pub(crate) enum FormatKind {
    Json,
    Toml,
    Yaml,
    Markdown,
    Binary,
    Unknown,
}
```

6d. Add `FileMetadata` as a `pub` struct in `reader.rs`:
```rust
pub struct FileMetadata {
    pub modified: Option<std::time::SystemTime>,
    pub size: u64,
    pub is_symlink: bool,
}
```

6e. Add to the `FsReader` trait:

```rust
pub trait FsReader: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    // --- existing, kept ---
    fn exists(&self, path: &Path) -> bool;
    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error>;
    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error>;

    // --- new ---
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, Self::Error>;
    fn metadata(&self, path: &Path) -> Result<FileMetadata, Self::Error>;

    // --- pipeline methods (provided via default impls using above) ---

    fn validate_path(&self, path: &Path) -> Result<(), PathValidationError> {
        Validator::new_flexible().validate(path)
    }

    fn classify(&self, path: &Path) -> FormatKind {
        if Json::is_supported(path)     { return FormatKind::Json; }
        if Toml::is_supported(path)     { return FormatKind::Toml; }
        if Yaml::is_supported(path)     { return FormatKind::Yaml; }
        if is_markdown(path)            { return FormatKind::Markdown; }
        if is_likely_binary(path)       { return FormatKind::Binary; }
        FormatKind::Unknown
    }

    fn parse_structured<T: DeserializeOwned>(
        &self,
        path: &Path,
    ) -> Result<T, ParseError>
    where
        Self::Error: Into<std::io::Error>,
    {
        let content = self.read_to_string(path)
            .map_err(|e| ParseError::Io { path: path.to_path_buf(), source: e.into() })?;
        match self.classify(path) {
            FormatKind::Json => Json::parse(path, &content),
            FormatKind::Toml => Toml::parse(path, &content),
            FormatKind::Yaml => Yaml::parse(path, &content),
            _ => Err(ParseError::UnsupportedFormat {
                path: path.to_path_buf(),
                supported: &["json", "toml", "yaml", "yml"],
            }),
        }
    }

    fn read_with<T, F>(&self, path: &Path, f: F) -> Result<T, ParseError>
    where
        F: FnOnce(&Path, &str) -> Result<T, ParseError>,
        Self::Error: Into<std::io::Error>,
    {
        let content = self.read_to_string(path)
            .map_err(|e| ParseError::Io { path: path.to_path_buf(), source: e.into() })?;
        f(path, &content)
    }
}
```

`is_markdown(path)` is a private `fn` in `reader.rs` checking extensions
`["md", "markdown"]`. `is_likely_binary(path)` is a private `fn` checking
common binary extensions (`["png", "jpg", "jpeg", "gif", "pdf", "mp3", "mp4",
"zip", "wasm"]`).

6f. Fix `OsFsReader::list_files`:
- Replace `walkdir::WalkDir` + per-entry `glob::Pattern::new()` with a
  single `glob::glob()` call:
  ```rust
  fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, io::Error> {
      let full = self.root.join(pattern);
      let full_str = full.to_str().ok_or_else(|| {
          io::Error::new(io::ErrorKind::InvalidInput, "non-UTF-8 pattern")
      })?;
      let mut paths: Vec<PathBuf> = glob::glob(full_str)
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
          .filter_map(|entry| {
              let p = entry.ok()?;
              // include only files (symlinks to files included by default)
              if !p.is_file() && !p.is_symlink() { return None; }
              p.strip_prefix(&self.root).ok().map(Path::to_path_buf)
          })
          .collect();
      paths.sort();
      Ok(paths)
  }
  ```
  Document: symlinks are included by default; policy tightening is deferred.

6g. Implement `read_bytes` and `metadata` on `OsFsReader`:
```rust
fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
    std::fs::read(self.resolve_path(path))
}

fn metadata(&self, path: &Path) -> Result<FileMetadata, io::Error> {
    let full = self.resolve_path(path);
    let m = std::fs::symlink_metadata(&full)?;
    Ok(FileMetadata {
        modified: m.modified().ok(),
        size: m.len(),
        is_symlink: m.file_type().is_symlink(),
    })
}
```

6h. Update `mod.rs`:
- Change `pub mod source;` to `pub mod reader;`.
- Remove `pub type FsFileSource = source::FsFileSource;`
  and `pub type InMemoryFileSource = source::InMemoryFileSource;`.
- Add:
  ```rust
  pub use reader::{FsReader, OsFsReader, FileMetadata};
  pub(crate) use reader::FormatKind;
  ```

6i. Update `schema/adapter/ingestor.rs`:
- Restore the generic parameter `S: FsReader` on `Ingestor`:
  ```rust
  pub struct Ingestor<'config, S> {
      source: S,
      config: &'config Config,
  }
  impl<'config, S> Ingestor<'config, S> {
      pub const fn new(source: S, config: &'config Config) -> Self { ... }
  }
  impl<S: FsReader<Error = io::Error>> Ingestor<'_, S> {
      // use self.source.parse_structured(&path)?
  }
  ```
- Replace the temporary `std::fs` + `glob` bridge from step 5h with
  `self.source.parse_structured(&path)` and `self.source.list_files(&pattern)`.
- The `// File modification time not available from FileSource trait` comment
  at line 108 can now be removed and replaced with
  `self.source.metadata(&path).ok().and_then(|m| m.modified)`.
- Update `Ingestor::new` call sites in non-test code to pass an `OsFsReader`.

**Verify:** `mise run test:unit:fs && mise run test:unit:schema`

---

### Step 7 — `writer.rs`: introduce `FsWriter` and `OsFsWriter`

**Status:** `[ ]`

**What:**

7a. Create `lithos-core/src/fs/writer.rs` with:

```rust
pub trait FsWriter: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn create_dir_all(&self, path: &Path) -> Result<(), Self::Error>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), Self::Error>;
    fn atomic_write(&self, path: &Path, contents: &[u8]) -> Result<(), Self::Error>;
    fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::Error>;
    fn remove_file(&self, path: &Path) -> Result<(), Self::Error>;
}
```

7b. Implement `OsFsWriter`:
```rust
pub struct OsFsWriter { root: PathBuf }

impl OsFsWriter {
    pub fn new(root: &Path) -> Self { Self { root: root.to_path_buf() } }
    fn resolve(&self, path: &Path) -> PathBuf { self.root.join(path) }
}

impl FsWriter for OsFsWriter {
    type Error = io::Error;

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(self.resolve(path))
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        std::fs::write(self.resolve(path), contents)
    }

    fn atomic_write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        let target = self.resolve(path);
        let parent = target.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path has no parent")
        })?;
        // Create temp file in same directory to guarantee same filesystem
        // (rename across filesystems is not atomic on any platform).
        let mut tmp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(parent.join(tmp_name(&target)))?;
        tmp.write_all(contents)?;
        tmp.sync_all()?;   // durability guarantee on the data
        drop(tmp);
        std::fs::rename(parent.join(tmp_name(&target)), &target)
        // Note: parent directory fsync is omitted for performance.
        // Add OsFsWriter::atomic_write_strict() later if durability of the
        // directory entry itself is required.
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(self.resolve(from), self.resolve(to))
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(self.resolve(path))
    }
}
```
`tmp_name` generates a unique temp filename in the same directory using the
target filename + a random suffix (`format!(".{}.tmp", rand_suffix)`).
Use `std::time::SystemTime` nanos or a simple counter for the suffix — no
external crate needed.

7c. Add `PathValidator` enforcement to every `OsFsWriter` method:
call `Validator::new_flexible().validate(path)?` before every resolve,
converting `PathValidationError` to `io::Error` via `io::Error::other(e)`.

7d. Add `pub mod writer;` and `pub use writer::{FsWriter, OsFsWriter};` to
`mod.rs`.

**Verify:** `mise run test:unit:fs`

---

### Step 8 — `validator.rs`: remaining fixes

**Status:** `[ ]`

**What:**

8a. Add a new `PathValidationError::EmptyPath` variant:
```rust
#[error("Path cannot be empty")]
EmptyPath,
```
Used by `validate_vault_path` (step 3c) and documented for external callers.

8b. Ensure `new_strict` `# Panics` doc reads:
> Panics if `root` is not an absolute path. The caller is responsible for
> canonicalizing the root with `std::fs::canonicalize` before calling this
> constructor. Passing a non-canonicalized path containing symlinks may cause
> `resolve_safe_symlink` to reject valid paths.

**Verify:** `mise run test:unit:fs`

---

### Step 9 — Tests: replace in-memory source with `tempfile`

**Status:** `[ ]`

**What:**

9a. Delete all `InMemoryFileSource` unit tests in `source.rs` (now
`reader.rs`). These are fully replaced.

9b. Add integration tests in `reader.rs` using `tempfile::TempDir` covering:
- `list_files("**/*.json")` — returns only json files, sorted, relative.
- `list_files("*.md")` — flat directory, correct files only.
- `list_files("[invalid")` — returns `InvalidInput` error.
- `read_to_string` — reads file correctly, root-relative path.
- `read_bytes` — reads binary content.
- `metadata` — returns correct size, `is_symlink = false` for regular file.
- Symlink: `metadata` returns `is_symlink = true` for a symlink.
- Symlink: `list_files` includes symlinked files (default policy).
- `parse_structured<T>` — reads and parses a `.json` file into a `serde_json::Value`.
- `parse_structured<T>` — returns `UnsupportedFormat` for `.xml` file.
- `read_with` — invokes closure with content, closure result propagated.

9c. Update `schema/adapter/ingestor.rs` tests (lines 129–439):
- Remove `use fs::source::InMemoryFileSource;` import.
- Replace all `InMemoryFileSource::new()` / `source.insert(...)` patterns
  with a helper that creates a `TempDir`, writes files to disk, and
  constructs `OsFsReader::new(temp_dir.path())`.

9d. Remove `walkdir` from `lithos-core/Cargo.toml` `[dependencies]`
(it was only used in `list_files`, now replaced by `glob::glob()`).
Confirm with `cargo check` before committing.

**Verify:** `mise run verify`

---

### Step 10 — `mod.rs` + `markdown.rs`: final cleanup

**Status:** `[ ]`

**What:**

10a. Change `pub mod markdown;` to `pub(crate) mod markdown;` in `mod.rs`.
Do NOT remove the `mod` declaration — `note/parser.rs:23` imports
`crate::fs::MarkdownParser` and must continue to compile.

Change the type aliases in `mod.rs` from `pub` to `pub(crate)`:
```rust
// before
pub type MarkdownParser = markdown::MarkdownParser;
pub type MarkdownOffsetIter<'markdown> = markdown::MarkdownOffsetIter<'markdown>;

// after
pub(crate) type MarkdownParser = markdown::MarkdownParser;
pub(crate) type MarkdownOffsetIter<'markdown> = markdown::MarkdownOffsetIter<'markdown>;
```

No change to `note/parser.rs` is needed — `pub(crate)` is fully visible
within the same crate.

Rationale: `MarkdownParser` and its Obsidian options belong to the note
context. Making the module `pub(crate)` prevents external crates from
depending on it while deferring the full move to a follow-up ADR.

10b. Update `mod.rs` module doc comment to reflect the new structure:
- Remove references to `parsers`, `source`, `FormatDispatcher`,
  `InMemoryFileSource`, `FsFileSource`.
- Add descriptions of `reader`, `writer`, `types`.
- Document the read pipeline order.

10b2. Update `application/mod.rs` doc comment (line 31): change
`FileSource` reference to `FsReader`.

10c. Remove `#[expect(clippy::module_name_repetitions)]` crate-wide
suppression from `mod.rs` if individual suppressions in each file are
sufficient (they are; check with `mise run lint`).

**Verify:** `mise run verify`

---

### Execution order summary

```
Step 1  (error.rs types)
Step 2  (validator.rs cleanup)
Step 3  (validator.rs + mod.rs path helpers)        ← needs step 2
Step 4  (note/aggregate.rs boundary fix)             ← needs step 3 (re-export)
Step 5  (parsers.rs → types.rs)                      ← needs step 1
Step 6  (source.rs → reader.rs FsReader pipeline)    ← needs steps 2, 3, 5
Step 7  (writer.rs new)                              ← needs step 2
Step 8  (validator.rs remaining fixes)               ← needs step 3
Step 9  (tests tempfile migration)                   ← needs step 6
Step 10 (mod.rs + markdown.rs final cleanup)         ← needs all prior steps
```

---

### Files changed (complete list)

| File | Step(s) | Change |
|---|---|---|
| `lithos-core/src/fs/error.rs` | 1, 8 | `Box<Path>` → `PathBuf`; `Vec<&'static str>` → `&'static [&'static str]` in `UnsupportedFormat`; add `EmptyPath` variant |
| `lithos-core/src/fs/validator.rs` | 2, 3, 8 | `validate()` return; `Mode` visibility; tracing removed; path helpers moved in; Windows fix; `validate_vault_path` added; `new_strict` doc fixed |
| `lithos-core/src/fs/mod.rs` | 3, 5, 6, 10 | Remove moved fns; update re-exports; update docs; `pub` → `pub(crate)` for markdown types |
| `lithos-core/src/fs/parsers.rs` → `types.rs` | 5 | Rename; remove `Dispatcher`, `parse_file`, tracing; add `is_supported` guard in `parse` methods |
| `lithos-core/src/fs/source.rs` → `reader.rs` | 6 | Rename; `FileSource` → `FsReader`; `FsFileSource` → `OsFsReader`; remove `InMemoryFileSource`; add `FormatKind`, `FileMetadata`, pipeline methods; fix `list_files` |
| `lithos-core/src/fs/writer.rs` | 7 | New file: `FsWriter`, `OsFsWriter`, `atomic_write` |
| `lithos-core/src/fs/markdown.rs` | 10 | Change to `pub(crate)` module; keep in place; `note/parser.rs` unchanged |
| `lithos-core/Cargo.toml` | 9 | Remove `walkdir` from `[dependencies]` |
| `lithos-core/src/note/aggregate.rs` | 4 | Remove `crate::fs` import; inline domain validation |
| `lithos-core/src/schema/adapter/ingestor.rs` | 5, 6, 9 | Step 5: remove `S` generic + `FileSource` bound, inline `std::fs`; step 6: restore `S: FsReader` abstraction; step 9: replace in-memory tests with `tempfile` |
| `lithos-core/src/application/mod.rs` | 10 | Doc comment only: `FileSource` → `FsReader` |

### Files that do NOT need changes despite referencing affected symbols

| File | Symbol referenced | Reason no change needed |
|---|---|---|
| `lithos-core/src/template/adapter/filters.rs` | `validate_vault_path` (line 216) | Uses only `format!("{e}")` — `PathValidationError: Display`; compiles after step 3 |
| `lithos-core/src/schema/error.rs` | `From<ParseError>` (line 254) | `PathBuf::to_string_lossy()` and `<[_]>::join()` work for changed types |
| `lithos-core/src/note/parser.rs` | `crate::fs::MarkdownParser` (line 23) | `pub(crate)` is visible within the crate; no change needed |
| `lithos-core/tests/architecture.rs` | `FileSource` in string literals only | Not a Rust import; assertion messages only |
| `lithos-cli/src/` | None | No fs symbols used |
