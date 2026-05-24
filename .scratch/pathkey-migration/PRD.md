# PRD: PathKey Migration — Repository Boundary Path Type Unification

## Problem Statement

Developers working on schema discovery, config-to-schema handoff, and repository persistence encounter repeated path-type conversion churn, runtime panics from unchecked path assumptions, and semantic drift between filesystem operations and database keys.

Current issues:
- `SchemaConfigSpec` stores relative paths but exposes both relative and absolute accessors, forcing recomputation with `unreachable!` branches for invariant enforcement
- `DiscoveryEngine` and `Builder` contain ad hoc `strip_prefix` + `RelativePath::try_from` conversion chains at every repository boundary
- Repository trait signatures accept `RelativePath` (platform-specific, filesystem-oriented) for database keys instead of normalized canonical storage keys
- `AbsolutePath` and `RelativePath` serve overlapping roles without clear boundary ownership
- `Config::to_schema_spec()` uses panic-based `expect()` for core path contract assembly instead of fallible constructors

This coupling makes it difficult to reason about path semantics, introduces silent conversion bugs, and prevents clean separation between execution-facing (filesystem I/O) and persistence-facing (database key) path usage.

## Solution

Introduce `PathKey` as the single canonical vault-relative path type for all repository and storage boundaries across contexts, while keeping `FsPath`, `FilePath`, and `DirPath` strictly for filesystem I/O operations.

Key changes:
1. Rename `NormalizedPath` → `PathKey` with explicit semantics as the persistence-key primitive
2. Make all filesystem→key conversions root-scoped and fallible via `PathKey::from_rooted_path(root, path)` and convenience `as_key(root)` methods
3. Redesign `SchemaConfigSpec` to be execution-facing: store typed filesystem paths (`DirPath`, `FilePath`), derive `PathKey` only at repository call boundaries
4. Hard-cut repository trait signatures from `RelativePath` to `PathKey` per context (schema → vault → note/template)
5. Remove `AbsolutePath` and `RelativePath` after migration, with short-lived deprecated aliases and architecture test enforcement

This separates concerns cleanly, eliminates conversion churn, and makes path semantics explicit at every boundary.

## User Stories

1. As a schema loader developer, I want repository methods to accept only canonical database keys, so that I don't have to manually convert filesystem paths to storage keys at every call site
2. As a config developer, I want `SchemaConfigSpec` to expose typed filesystem paths for discovery operations, so that execution-facing code works with `DirPath`/`FilePath` natively
3. As a schema loader developer, I want `SchemaConfigSpec` to provide persistence keys on demand via `directory_key()` / `property_bank_key()`, so that repository calls are explicit and fallible
4. As a discovery engine developer, I want to convert scanned `FilePath`/`DirPath` entries to `PathKey` via `as_key(root)`, so that conversion logic is centralized and root-scoped
5. As a repository implementer, I want all persistence methods to use `PathKey` consistently, so that table key types are uniform and comparable across contexts
6. As a builder developer, I want to remove manual `strip_prefix` + `RelativePath::try_from` chains, so that conversion errors are caught at the right boundary with proper error types
7. As a codebase maintainer, I want `AbsolutePath` removed from config-facing types, so that filesystem paths use the typed `DirPath`/`FilePath` model consistently
8. As a developer, I want `PathKey` to enforce strict normalization (no `.`/`..`, forward slashes, UTF-8 only), so that database keys are deterministic and aliasing bugs are prevented
9. As a config developer, I want `VaultRoot` and `TrustedVaultPath` to wrap `DirPath` instead of raw `PathBuf`, so that invariants are checked at construction time
10. As a schema repository developer, I want batch read operations (`find_raw_schema_views_by_paths`) to accept `&[PathKey]`, so that key semantics are explicit and type-safe
11. As a vault storage developer, I want the same `PathKey` type used for vault file/directory keys, so that path handling is uniform across all contexts
12. As a note/template developer, I want repository boundaries to use `PathKey` for persistence, so that filesystem operations and database keys are clearly separated
13. As a developer adding new repository methods, I want compiler enforcement that filesystem paths cannot be used directly as keys, so that I cannot accidentally introduce semantic drift
14. As a migration owner, I want a transitional architecture test module, so that I can enforce phased deprecation and prevent `RelativePath`/`AbsolutePath` from spreading during migration
15. As a developer, I want `PathKey` conversions to return `Result<PathKey, PathError>` for outside-root cases, so that boundary violations are explicit and mappable to context-specific errors
16. As a test author, I want clear prior art for `PathKey` normalization tests, so that I can verify slash handling, traversal rejection, and UTF-8 enforcement comprehensively
17. As a config author, I want `SchemaConfigSpec::new()` to accept `DirPath`/`FilePath` directly, so that path validation happens at config construction instead of at repository call time
18. As a schema loader, I want `Config::to_schema_spec()` to use fallible `try_from` conversions instead of `expect()`, so that invalid paths are returned as errors instead of panics
19. As a developer reading discovery code, I want to see `entry.as_key(root)` instead of manual `strip_prefix` logic, so that the boundary conversion is obvious and auditable
20. As a repository trait user, I want `get_raw_property_bank_view(&PathKey)` instead of `get_raw_property_bank_view(&RelativePath)`, so that the key type matches table storage semantics
21. As a codebase maintainer, I want `NormalizedPath` to exist only as a short-lived deprecated alias, so that new code cannot accidentally use the old name
22. As a CI/test author, I want architecture tests to fail if new code introduces `RelativePath` in repository signatures, so that regressions are caught immediately
23. As a developer working on schema, vault, or note contexts, I want phased migration with clear exit criteria per context, so that I can complete one context fully before starting the next
24. As a schema discovery developer, I want `DiscoveryEngine::scan_filesystem` to return `Vec<FsEntry>` unchanged but convert to `PathKey` only at repository query boundaries, so that filesystem scanning and persistence logic are decoupled
25. As a builder developer, I want `load_property_bank` to call `entry.path().as_key(root)` for the repository lookup key, so that conversion is explicit and error-handling is correct

## Implementation Decisions

### PathKey Type Design

#### Core Structure
```rust
/// Canonical vault-relative path key for repository/storage boundaries.
///
/// This type is the single source of truth for persistence keys across all
/// contexts (schema, vault, note, template, config). It enforces strict
/// normalization and vault-boundary safety.
///
/// # Invariants
/// - UTF-8 only (non-UTF8 paths rejected at conversion)
/// - Forward slashes only (`\` normalized to `/`)
/// - No empty paths
/// - No `.` (current dir) or `..` (parent traversal) components
/// - No duplicate separators (`a//b` → `a/b`)
/// - No trailing separators (`a/b/` → `a/b`)
/// - Shape-agnostic (no file-vs-dir encoding; semantics at I/O boundary)
///
/// # Storage
/// - `Box<str>` for compact immutable ownership (~25% memory reduction vs `PathBuf`)
/// - Suitable for large key sets in DB/index workflows
///
/// # Usage
/// - Repository/storage boundaries: accept `&PathKey` only
/// - Filesystem I/O: use `FsPath`/`FilePath`/`DirPath`
/// - Conversion: always root-scoped via `from_rooted_path` or `as_key(root)`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct PathKey(Box<str>);
```

#### Constructors

**String-based (for tests/deserialization):**
```rust
impl PathKey {
    /// Create a normalized path key from a string.
    ///
    /// Performs structural normalization:
    /// - Converts backslashes to forward slashes
    /// - Removes duplicate separators
    /// - Removes trailing separators
    /// - Trims whitespace
    ///
    /// # Errors
    /// Returns `PathError` if:
    /// - Path is empty after normalization
    /// - Path contains `.` or `..` components
    /// - Path is absolute
    /// - Path contains platform-specific prefixes (Windows `C:\`, etc.)
    pub fn try_new(path: &str) -> Result<Self, PathError> { /* ... */ }
}

impl TryFrom<&str> for PathKey { /* ... */ }
impl TryFrom<String> for PathKey { /* ... */ }
```

**Root-scoped (for filesystem conversions):**
````rust
impl PathKey {
    /// Create a path key from an absolute filesystem path rooted at `root`.
    ///
    /// This is the low-level primitive for all filesystem→key conversions.
    /// Strips `root` prefix, validates result, and normalizes.
    ///
    /// # Errors
    /// Returns `PathError::OutsideRoot` if `path` is not under `root`.
    /// Returns other `PathError` variants if normalized path violates invariants.
    ///
    /// # Examples
    /// ```
    /// let root = DirPath::try_from("/vault")?;
    /// let file = Path::new("/vault/notes/daily.md");
    /// let key = PathKey::from_rooted_path(&root, file)?;
    /// assert_eq!(key.as_str(), "notes/daily.md");
    /// ```
    pub fn from_rooted_path(root: &DirPath, path: &Path) -> Result<Self, PathError> {
        // 1. Strip root prefix
        let relative = path.strip_prefix(root.as_path())
            .map_err(|_| PathError::OutsideRoot { /* ... */ })?;

        // 2. Validate UTF-8
        let utf8 = relative.to_str()
            .ok_or_else(|| PathError::InvalidUtf8 { /* ... */ })?;

        // 3. Normalize and validate via try_new
        Self::try_new(utf8)
    }
}
````

**Convenience methods (preferred public API):**
````rust
impl FilePath {
    /// Convert this file path to a storage key relative to `root`.
    ///
    /// # Errors
    /// Returns `PathError::OutsideRoot` if this path is not under `root`.
    ///
    /// # Examples
    /// ```
    /// let root = DirPath::try_from("/vault")?;
    /// let file = FilePath::try_from("/vault/notes/daily.md")?;
    /// let key = file.as_key(&root)?;
    /// assert_eq!(key.as_str(), "notes/daily.md");
    /// ```
    pub fn as_key(&self, root: &DirPath) -> Result<PathKey, PathError> {
        PathKey::from_rooted_path(root, self.as_path())
    }
}

impl DirPath {
    /// Convert this directory path to a storage key relative to `root`.
    pub fn as_key(&self, root: &DirPath) -> Result<PathKey, PathError> {
        PathKey::from_rooted_path(root, self.as_path())
    }
}

impl FsPath {
    /// Convert this path to a storage key relative to `root`.
    pub fn as_key(&self, root: &DirPath) -> Result<PathKey, PathError> {
        match self {
            FsPath::File(f) => f.as_key(root),
            FsPath::Dir(d) => d.as_key(root),
        }
    }
}
````

#### Accessors
```rust
impl PathKey {
    /// Returns the normalized path string.
    ///
    /// Guaranteed to be:
    /// - Valid UTF-8
    /// - Forward-slash separated
    /// - Free of `.`/`..` components
    /// - Free of duplicate/trailing separators
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PathKey { /* delegates to as_str */ }
impl std::fmt::Display for PathKey { /* delegates to as_str */ }
```

#### Invariant Implementation Strategy

The implementation follows the **"parse, don't validate"** principle: `PathKey::try_new` normalizes input into canonical form rather than rejecting non-canonical representations of valid paths.

**Normalization Pipeline:**
```
trim → normalize → validate → construct
```

1. **Trim**: Remove leading/trailing whitespace
2. **Normalize**: Convert to canonical form (single allocation if needed)
3. **Validate**: Check invariants on canonical form
4. **Construct**: Wrap validated string in `PathKey`

#### Normalization Algorithm

**Overview:**
- Check upfront if any normalization is needed
- If already canonical, return borrowed string (zero-copy)
- If normalization needed, allocate once and apply all transformations
- Preserves leading `/` for absolute path detection

**Implementation:**
```rust
impl PathKey {
    pub fn try_new(path: &str) -> Result<Self, PathError> {
        // Phase 1: Trim whitespace
        let trimmed = Self::trim(path);

        // Phase 2: Normalize to canonical form
        let normalized = Self::normalize(trimmed);

        // Phase 3: Validate invariants on canonical form
        Self::validate(normalized.as_ref())?;

        // Phase 4: Construct
        Ok(Self(normalized.into_owned().into_boxed_str()))
    }

    /// Trim leading and trailing whitespace.
    #[inline]
    fn trim(path: &str) -> &str {
        path.trim()
    }

    /// Normalize path to canonical form.
    ///
    /// Transformations:
    /// - Backslashes → forward slashes (`\` → `/`)
    /// - Duplicate separators → single separator (`a//b` → `a/b`)
    /// - Trailing separators removed (`a/b/` → `a/b`)
    /// - Preserves leading `/` for absolute path detection
    ///
    /// Returns `Cow::Borrowed` if no normalization needed (common case).
    /// Returns `Cow::Owned` with single allocation if normalization applied.
    fn normalize(path: &str) -> Cow<'_, str> {
        // Check if any normalization is needed
        let needs_slash_norm = path.contains('\\');
        let needs_sep_norm = path.contains("//") || path.ends_with('/');

        if !needs_slash_norm && !needs_sep_norm {
            return Cow::Borrowed(path);
        }

        // Allocate once and apply all normalizations
        let mut normalized = String::with_capacity(path.len());
        let mut prev_was_sep = false;
        let mut is_first = true;

        for ch in path.chars() {
            let is_sep = ch == '/' || ch == '\\';

            if is_sep {
                // Skip duplicate separators, but preserve leading separator
                if !prev_was_sep || is_first {
                    normalized.push('/');
                }
                prev_was_sep = true;
            } else {
                normalized.push(ch);
                prev_was_sep = false;
            }

            is_first = false;
        }

        // Remove trailing separator (unless it's the root `/`)
        if normalized.len() > 1 && normalized.ends_with('/') {
            normalized.pop();
        }

        Cow::Owned(normalized)
    }

    /// Validate invariants on canonical form.
    ///
    /// Checks after normalization:
    /// - Not empty
    /// - Not absolute (no leading `/`)
    /// - No `..` (parent traversal)
    /// - No `.` (current dir)
    /// - No platform prefixes (Windows `C:\`, etc.)
    fn validate(path: &str) -> Result<(), PathError> {
        // Check empty
        if path.is_empty() {
            return Err(PathError::Empty);
        }

        // Check absolute (on normalized form with potential leading `/`)
        let path_buf = PathBuf::from(path);
        if path_buf.is_absolute() {
            return Err(PathError::NotRelative(path_buf));
        }

        // Validate components
        for component in path_buf.components() {
            match component {
                Component::ParentDir => {
                    return Err(PathError::ParentTraversal(path_buf.clone()));
                }
                Component::CurDir => {
                    return Err(PathError::CurrentDirComponent(path_buf.clone()));
                }
                Component::Prefix(_) => {
                    return Err(PathError::PlatformPrefix(path_buf.clone()));
                }
                Component::RootDir | Component::Normal(_) => {}
            }
        }

        Ok(())
    }
}
```

#### Error Types
```rust
// Add to existing PathError enum:
pub enum PathError {
    // ... existing variants ...

    /// Path is outside the specified root directory.
    OutsideRoot {
        root: PathBuf,
        path: PathBuf,
    },

    /// Path contains invalid UTF-8.
    InvalidUtf8 {
        path: PathBuf,
    },
}
```

### Core Type Design (Summary)

- Rename `NormalizedPath` to `PathKey` to reflect role as canonical persistence-key primitive
- Keep `PathKey` as `Box<str>` internally for compact immutable ownership
- Enforce strict invariants: UTF-8 only, forward slashes, no `.`/`..`, no empty, structural normalization (dedupe/trailing separators)
- Make `PathKey` shape-agnostic: no file-vs-dir encoding; kind is enforced at filesystem-facing APIs

### Conversion API

- Low-level primitive: `PathKey::from_rooted_path(root: &DirPath, path: &Path) -> Result<PathKey, PathError>`
- Preferred public API: `as_key(&self, root: &DirPath) -> Result<PathKey, PathError>` on `FilePath`, `DirPath`, `FsPath`
- String-based constructors: `PathKey::try_new(&str)`, `TryFrom<&str>`, `TryFrom<String>` for tests/deserialization
- No `TryFrom<PathBuf>` to force root-scoped conversions from filesystem paths
- All conversions are fallible and explicit; no infallible `From` implementations

### Config Layer Changes

- Redesign `SchemaConfigSpec` to store only typed filesystem paths:
  - `root: VaultRoot` (thin wrapper over `DirPath`)
  - `directory: DirPath`
  - `property_bank: FilePath`
- Replace `directory_relative()` / `property_bank_relative()` accessors with `directory_key()` / `property_bank_key()` computed via `as_key(root)`
- Convert `VaultRoot` from raw `PathBuf` wrapper to thin newtype over `DirPath`
- Convert `TrustedVaultPath` from `AbsolutePath` wrapper to thin newtype over `DirPath`
- Split config path parsing by usage semantics:
  - Filesystem-target settings (vault root, trusted vaults) parse to `DirPath`
  - Persisted/indexed keys derive `PathKey` only at repository boundaries

### Repository Layer Hard Cuts

- Update `schema::repository::ReadRepository` / `WriteRepository` trait signatures:
  - Replace all `&RelativePath` parameters with `&PathKey`
  - Affected methods: `find_raw_schema_view_by_path`, `find_raw_schema_views_by_paths`, `get_raw_property_bank_view`, `find_schema_id_by_path`, `find_schema_ids_by_paths`
- Update `schema::storage` table key types from `RelativePath` to `PathKey`
- Remove all `RelativePath` usage in schema repository signatures in one atomic PR

### Discovery and Builder Cleanup

- In `DiscoveryEngine::separate_property_bank`, replace manual `strip_prefix` + `RelativePath::try_from` with `file.path().as_key(spec.root())`
- In `Builder::load_property_bank`, replace `strip_prefix` logic with `entry.path().as_key(root)` for repository key derivation
- Update `query_cached_state` to accept `PathKey` for property bank lookups

### Sequenced Context Migration

1. **Schema context first**: hard-cut repository signatures + discovery/builder cleanup
2. **Vault context second**: same pattern applied to vault repository/storage
3. **Note/template contexts third**: ensure uniform `PathKey` usage at all persistence boundaries
4. **Config context fourth**: finalize `SchemaConfigSpec`, `VaultRoot`, `TrustedVaultPath` thin-wrapper conversions

### Deprecation and Enforcement

- Create short-lived deprecated type alias: `pub type NormalizedPath = PathKey` with `#[deprecated(note = "...")]`
- Remove `AbsolutePath` from all production code paths after `DirPath` migration
- Mark `RelativePath` deprecated with phase-based removal plan
- Create transitional architecture test module (`lithos-core/tests/path_migration_architecture.rs`) with:
  - Phase 1: forbid `AbsolutePath` outside FS module
  - Phase 2: forbid `RelativePath` in schema repository signatures
  - Phase 3: forbid `RelativePath` in vault repository signatures
  - Phase 4: forbid `RelativePath` in note/template repository signatures
  - Phase 5: forbid `NormalizedPath` alias usage entirely
- Each phase has explicit exit criteria: no banned types in scoped modules

### Error Handling

- Keep canonical low-level `PathError` with "outside root" variant
- Map `PathError` to context-specific errors (`SchemaLoaderError`, `VaultFileError`, etc.) at orchestration boundaries
- All `PathKey` conversions return `Result<PathKey, PathError>`; no panics, no `expect()` in production paths

## Testing Decisions

### What Makes a Good Test

- Test external behavior (public API contracts), not implementation details (internal normalization steps)
- Test error boundaries explicitly (outside-root rejection, invalid UTF-8, traversal attempts)
- Use property-based testing for normalization rules where applicable (slash variants, duplicate separators)
- Integration tests should verify end-to-end key round-trips through repository read/write

### Modules to Test

1. **`fs::path::PathKey`**:
   - Normalization rules (forward slashes, dedupe/trailing separators, empty rejection, UTF-8 validation)
   - Traversal rejection (`.`/`..` forbidden)
   - String-based constructors (`try_new`, `TryFrom<&str>`)
   - Prior art: existing `NormalizedPath` tests in `lithos-core/src/fs/path.rs`

2. **`PathKey::from_rooted_path` + `as_key(root)` integration**:
   - Success cases (file/dir under root)
   - Outside-root rejection (file above root, sibling root)
   - Error propagation to context-specific error types
   - Prior art: path validation tests in `lithos-core/src/fs/path.rs`

3. **`config::paths::SchemaConfigSpec`**:
   - Construction with typed filesystem paths
   - `directory_key()` / `property_bank_key()` derive correct `PathKey`s
   - Error handling when key derivation fails
   - Prior art: existing `SchemaConfigSpec` tests in `lithos-core/src/config/paths.rs`

4. **`schema::repository` signature migration**:
   - `PathKey` round-trips through `find_raw_schema_views_by_paths` and other batch operations
   - Key matching behavior unchanged after migration
   - Prior art: existing repository integration tests in `lithos-core/src/schema/storage/read.rs`

5. **`schema::discovery::DiscoveryEngine`**:
   - Discovery correctly converts scanned filesystem paths to `PathKey` for repository lookups
   - No manual `strip_prefix` chains remain
   - Prior art: existing discovery tests in `lithos-core/src/schema/discovery.rs`

6. **Architecture test module** (`lithos-core/tests/path_migration_architecture.rs`):
   - Phase-gated enforcement of `AbsolutePath`, `RelativePath`, `NormalizedPath` bans
   - Exit criteria validation per context phase
   - Prior art: existing architecture tests in `lithos-core/tests/architecture.rs`

## Out of Scope

- Unicode normalization (e.g., NFC) — deferred for future ADR if needed
- Case-folding or case-insensitive key matching — preserve exact casing
- Filesystem I/O path validation beyond current `FilePath`/`DirPath` invariants
- Migration of non-repository path usage (e.g., CLI display paths, user-facing messages)
- Conversion of existing persisted keys in database (migration assumes key format is forward-compatible)
- Performance optimization of `PathKey` normalization (functional correctness first)

## Further Notes

- This migration is a prerequisite for completing the `SchemaConfigSpec` redesign blocked in `.scratch/fs-inode-architecture/01-path-types.md` and `.scratch/fs-inode-architecture/02-name-types.md`
- The architectural decision (single canonical repository boundary type, root-scoped conversions, deprecate `RelativePath`/`AbsolutePath`) will be captured in a separate system-level ADR
- The transitional architecture test module should be retired after all phases complete and the deprecated alias is removed
- Exit criteria per phase ensure monotonic progress and prevent regression of path-type coupling
