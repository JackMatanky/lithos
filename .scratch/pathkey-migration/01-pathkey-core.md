---
title: "Issue 01: PathKey core type and normalization pipeline"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
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

4. **Error Types (Add to PathError):**
```rust
pub enum PathError {
    OutsideRoot { root: PathBuf, path: PathBuf },
    InvalidUtf8 { path: PathBuf },
}
```

5. **Deprecation Alias:**
```rust
#[deprecated(note = "Use PathKey instead")]
pub type NormalizedPath = PathKey;
```

**Acceptance criteria:**
- [ ] `PathKey` exists and implements the `trim -> normalize -> validate` pipeline utilizing "parse, don't validate", preserving leading `/`, and optimizing with `Cow` (zero-copy when canonical, single-allocation otherwise).
- [ ] Normalization explicitly removes trailing separators (except root `/`) and deduplicates separators.
- [ ] `from_rooted_path` and `as_key` methods are implemented and fallible.
- [ ] `PathError::OutsideRoot` and `PathError::InvalidUtf8` are surfaced when converting from filesystem paths.
- [ ] Comprehensive tests cover normalization rules, traversal rejection, UTF-8 validation, and outside-root cases.

**Out of scope:**
- Repository signature migration (handled in subsequent slices).
