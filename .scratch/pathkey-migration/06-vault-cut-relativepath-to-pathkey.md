---
title: "Issue 06: Vault context hard cut from RelativePath to PathKey"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-25
date_completed: null
---

# Issue 06: Vault context hard cut from RelativePath to PathKey

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Complete the PathKey migration for vault context with four architectural improvements:

1. **Implement `redb::Key` and `redb::Value` for PathKey** - Type safety at DB boundary
2. **Add `PathUuidTable` and `UuidPathTable` wrappers** - Eliminate repetition, prevent String leakage
3. **Fix filesystem layer violation in vault/processor.rs** - Use typed FS paths before PathKey conversion
4. **Add runtime tests for db/table.rs** - Verify serialization round-trips

## Agent Brief

**Category:** enhancement
**Summary:** Harden vault PathKey migration with redb trait implementations, table wrappers, and filesystem layer fixes.

### Current State

**Discovery:** Vault repository and storage already use `PathKey` throughout:
- ✅ All 9 repository methods use `&PathKey` parameters
- ✅ All 9 storage tables use `PathTable<V>` (internally `String`)
- ✅ No `RelativePath` in repository trait signatures

**However, four architectural issues remain:**

1. **PathKey lacks redb traits** - Storage layer manually converts `PathKey → String → PathKey`:
   ```rust
   // lithos-core/src/vault/storage/write.rs:86
   path_table.insert(path.as_str().to_owned(), &file.id())?;  // ❌ Manual conversion
   ```

2. **Table wrappers missing** - Pattern repeated 4× (files, dirs) without type safety:
   ```rust
   pub const FILE_ID_BY_PATH: PathTable<FileId> = ...;
   pub const PATH_BY_FILE_ID: UuidTable<FileId, String> = ...;  // ❌ String, not PathKey
   ```

3. **Filesystem layer violated** - `vault/processor.rs` converts `RelativePath → PathKey` directly:
   ```rust
   // lithos-core/src/vault/processor.rs:866
   PathKey::try_new(raw).map_err(...)  // ❌ Bypasses FilePath/DirPath
   ```
   **Correct:** `FilePath::try_new(path)?.as_key(root)?`

4. **Test coverage gaps** - `db/table.rs` has only 2 compile-time tests, missing runtime validation.

### Desired Behavior

**After refactor:**

1. **Type-safe database boundary:**
   ```rust
   impl redb::Value for PathKey { ... }
   impl redb::Key for PathKey { ... }

   path_table.insert(path, &file.id())?;  // ✅ Direct PathKey insertion
   ```

2. **Self-documenting table wrappers:**
   ```rust
   pub const FILE_ID_BY_PATH: PathUuidTable<FileId> = PathUuidTable::new("...");
   pub const PATH_BY_FILE_ID: UuidPathTable<FileId> = UuidPathTable::new("...");
   ```

3. **Proper filesystem layer separation:**
   ```rust
   let file_path = FilePath::try_new(absolute_path)?;
   let key = file_path.as_key(root)?;  // ✅ Typed FS → PathKey
   ```

4. **Comprehensive test coverage:**
   - PathKey serialization round-trips
   - Table wrapper runtime behavior
   - Filesystem conversion contracts

### Key Interfaces

**New trait implementations:**
- `lithos-core/src/db/path.rs` (new file):
  - `impl redb::Value for PathKey`
  - `impl redb::Key for PathKey`

**New table wrappers:**
- `lithos-core/src/db/table.rs`:
  - `PathUuidTable<V: UuidV7DbType>` - PathKey → UUID forward index
  - `UuidPathTable<K: UuidV7DbType>` - UUID → PathKey reverse index

**Updated implementations:**
- `vault/storage/tables.rs` - Use new wrappers
- `vault/storage/read.rs` - Remove `.to_owned()` conversions
- `vault/storage/write.rs` - Remove `.to_owned()` conversions
- `vault/processor.rs` - Fix filesystem layer violation

### Acceptance Criteria

- [ ] PathKey implements `redb::Value` and `redb::Key` traits
- [ ] `PathUuidTable` and `UuidPathTable` wrappers exist in `db/table.rs`
- [ ] `PathTable<V>` definition uses `PathKey` instead of `String`
- [ ] Vault storage tables use new wrappers (4 tables updated)
- [ ] No `.to_owned()` calls on PathKey in storage layer
- [ ] `vault/processor.rs` uses typed FS paths (`FilePath`/`DirPath`) before `as_key(root)`
- [ ] No direct `PathKey::try_new()` calls in vault scanning code
- [ ] Comprehensive tests for PathKey serialization (~15 new tests)
- [ ] All vault integration and unit tests pass
- [ ] `mise run verify` passes (fmt + lint + tests)

### Out of Scope

- Schema context filesystem layer violations (defer to issue 05 or new issue)
- Note/template context migration (covered in issue 07)
- Unicode normalization or case-folding in PathKey

---

## TDD & Implementation Plan

### Planning & Design

**Deep Modules / Testability:**
- PathKey redb traits: small interface (`Value` + `Key`), deep implementation (normalization)
- Table wrappers: follow existing `UuidTable` pattern for consistency
- Filesystem layer: enforce typed conversion at processor boundary

**Interface Design Principles (Rust Best Practices Ch. 1):**
- Prefer `&PathKey` over `String` in public APIs (zero-copy)
- Use `.expect()` with context messages (better than `.unwrap()`)
- Panic in `from_bytes()` is acceptable (matches redb ecosystem, no Result alternative)

**Behaviors to Test (Prioritized):**
1. PathKey serialization round-trips through redb
2. Table wrappers compile and store/retrieve correctly
3. Vault storage uses PathKey directly (no String conversions)
4. Filesystem paths convert to PathKey via typed layer

---

## Phase 1: PathKey redb Traits (Priority 1)

**Goal:** Implement `redb::Key` and `redb::Value` for PathKey

**Why First:** Prerequisite for all other changes, highest impact

**Files:**
- **New:** `lithos-core/src/db/path.rs`
- **Modified:** `lithos-core/src/db/mod.rs` (add `pub mod path;`)
- **Modified:** `lithos-core/src/db/table.rs` (update `PathTable` definition)

### Cycle 1.1: Tracer Bullet - redb::Value Implementation

**Location:** `lithos-core/src/db/path.rs` (new file)

**RED:**
```rust
//! PathKey redb trait implementations.

use redb::{Key, Value};
use crate::fs::path::PathKey;

#[cfg(test)]
mod tests {
    use super::*;

    mod serialization {
        use super::*;

        #[test]
        fn preserves_value_across_redb_roundtrip() {
            let original = PathKey::try_new("notes/daily.md").expect("valid key");

            // Serialize via redb::Value
            let bytes = PathKey::as_bytes(&original);

            // Deserialize via redb::Value
            let deserialized = PathKey::from_bytes(bytes);

            assert_eq!(original, deserialized);
        }
    }
}
```

**GREEN:**
```rust
impl redb::Value for PathKey {
    type SelfType<'a> = PathKey where Self: 'a;
    type AsBytes<'a> = &'a [u8] where Self: 'a;

    fn fixed_width() -> Option<usize> {
        None  // Variable-length UTF-8 strings
    }

    /// Deserialize PathKey from database bytes.
    ///
    /// # Panics
    /// Panics if stored data is not valid UTF-8 or violates PathKey normalization
    /// invariants. This indicates database corruption.
    ///
    /// This panic behavior matches redb ecosystem patterns (String, &str) and is
    /// required by the trait signature (no Result return type).
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where Self: 'a
    {
        let s = std::str::from_utf8(data)
            .expect("PathKey data from database must be valid UTF-8");

        PathKey::try_new(s)
            .expect("PathKey data from database must be normalized")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> &'a [u8]
    where Self: 'a, Self: 'b
    {
        value.as_str().as_bytes()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("lithos::PathKey")
    }
}
```

**Verification:** `cargo test -p lithos-core db::path::tests::serialization`

**Checklist:**
- [x] Tests through public `PathKey::try_new` interface
- [x] Uses redb traits (not internal serialization)
- [x] Would survive PathKey internal refactors
- [x] Uses `.expect()` with context (Rust BP Ch. 4.2)
- [x] Documents panic conditions

### Cycle 1.2: redb::Key Implementation

**RED:**
```rust
#[test]
fn orders_keys_lexicographically() {
    let key1 = PathKey::try_new("a/file.md").expect("valid");
    let key2 = PathKey::try_new("b/file.md").expect("valid");

    let bytes1 = key1.as_str().as_bytes();
    let bytes2 = key2.as_str().as_bytes();

    let result = PathKey::compare(bytes1, bytes2);

    assert_eq!(result, std::cmp::Ordering::Less);
}
```

**GREEN:**
```rust
impl redb::Key for PathKey {
    /// Compare PathKey bytes lexicographically.
    ///
    /// UTF-8 byte comparison is valid because PathKey enforces UTF-8 at construction.
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}
```

**Verification:** Same test file as 1.1

**Checklist:**
- [x] Simple byte comparison (UTF-8 safe)
- [x] Doc comment explains safety assumption

### Cycle 1.3: Update PathTable Definition

**Location:** `lithos-core/src/db/table.rs`

**RED:**
```rust
#[cfg(test)]
mod tests {
    mod path_table {
        use crate::fs::path::PathKey;

        #[test]
        fn accepts_pathkey_as_key_type() {
            use redb::TableDefinition;

            const TABLE: PathTable<u64> = PathTable::new("test");

            // This should compile with PathKey as key type
            let _def: TableDefinition<'static, PathKey, u64> = TABLE.definition();
        }
    }
}
```

**GREEN:**
```rust
/// Table with PathKey keys, typically representing vault-relative file paths.
///
/// Uses `PathKey` directly as the redb key type (requires `PathKey` to implement
/// `redb::Key` and `redb::Value`). This enforces type safety: only normalized,
/// validated paths can be stored.
///
/// # Design Note
///
/// Earlier versions used `String` keys, requiring manual `.to_owned()` conversions.
/// With PathKey implementing redb traits, we can store and retrieve paths directly
/// without string allocation.
pub struct PathTable<V: Value + 'static> {
    definition: TableDefinition<'static, PathKey, V>,
}
```

**Verification:** `cargo test -p lithos-core db::table::tests::path_table`

**Checklist:**
- [x] Updated doc comment explains design change
- [x] Type signature enforces PathKey
- [x] Test verifies compilation

### Cycle 1.4: Update Vault Storage Layer

**RED:** Run existing vault tests - should fail with type errors after PathTable change

**GREEN:**

**File:** `lithos-core/src/vault/storage/write.rs`
```rust
// Line 86: Before
path_table.insert(path.as_str().to_owned(), &file.id())?;

// After
path_table.insert(path, &file.id())?;

// Line 88: Before
reverse_path_table.insert(&file.id(), path.as_str().to_owned())?;

// After (note: reverse table still uses String until Phase 2)
reverse_path_table.insert(&file.id(), path.as_str().to_owned())?;
```

**File:** `lithos-core/src/vault/storage/read.rs`
```rust
// Line 89: Before
path_table.get(path.as_str().to_owned())?

// After
path_table.get(path)?
```

**Similar changes in:**
- `find_dir_view_by_path` (line 118)
- `list_file_paths` (line 263-282)
- `list_dir_paths` (similar pattern)

**Verification:** `cargo test -p lithos-core vault::`

**Checklist:**
- [x] All vault tests pass
- [x] Removed `.to_owned()` on path insertions
- [x] Direct PathKey usage at storage boundary

---

## Phase 2: Table Wrappers (Priority 2)

**Goal:** Add `PathUuidTable` and `UuidPathTable` wrappers

**Why Second:** Depends on Phase 1 (PathKey as redb type), eliminates repetition

**Files:**
- **Modified:** `lithos-core/src/db/table.rs`
- **Modified:** `lithos-core/src/vault/storage/tables.rs`

### Cycle 2.1: PathUuidTable Tracer Bullet

**Location:** `lithos-core/src/db/table.rs`

**RED:**
```rust
#[cfg(test)]
mod tests {
    mod path_uuid_table {
        use crate::fs::path::PathKey;
        use crate::utils::UuidV7;

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct TestId(UuidV7);
        impl_redb_uuid!(TestId);

        #[test]
        fn const_construction_works() {
            const TABLE: PathUuidTable<TestId> = PathUuidTable::new("test");
            let _def = TABLE.definition();
        }

        #[test]
        fn inserts_and_retrieves_pathkey_to_uuid() {
            let db = redb::Database::create(":memory:").expect("db");
            const TABLE: PathUuidTable<TestId> = PathUuidTable::new("test");

            let key = PathKey::try_new("notes/test.md").expect("key");
            let id = TestId(UuidV7::new());

            // Write
            let tx = db.begin_write().expect("tx");
            {
                let mut table = tx.open_table(TABLE.definition()).expect("open");
                table.insert(&key, &id).expect("insert");
            }
            tx.commit().expect("commit");

            // Read
            let tx = db.begin_read().expect("tx");
            let table = tx.open_table(TABLE.definition()).expect("open");
            let retrieved = table.get(&key).expect("get").expect("value");

            assert_eq!(retrieved.value(), id);
        }

        #[test]
        fn returns_none_when_key_not_found() {
            let db = redb::Database::create(":memory:").expect("db");
            const TABLE: PathUuidTable<TestId> = PathUuidTable::new("test");

            let key = PathKey::try_new("nonexistent.md").expect("key");

            let tx = db.begin_read().expect("tx");
            let table = tx.open_table(TABLE.definition()).expect("open");
            let result = table.get(&key).expect("get");

            assert!(result.is_none());
        }
    }
}
```

**GREEN:**
```rust
/// Table mapping PathKey → UUID (forward index).
///
/// Use this for path-based lookups where filesystem paths are the query key.
/// Typical use case: finding entity IDs by their vault-relative paths.
///
/// # Examples
///
/// ```
/// use lithos_core::db::PathUuidTable;
/// use lithos_core::vault::FileId;
///
/// const FILE_ID_BY_PATH: PathUuidTable<FileId> = PathUuidTable::new("file_id_by_path");
/// ```
pub struct PathUuidTable<V: UuidV7DbType + 'static> {
    definition: TableDefinition<'static, PathKey, V>,
}

impl<V: UuidV7DbType + 'static> PathUuidTable<V> {
    /// Create a new PathKey → UUID table definition.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> TableDefinition<'static, PathKey, V> {
        self.definition
    }
}
```

**Verification:** `cargo test -p lithos-core db::table::tests::path_uuid_table`

**Checklist:**
- [x] Follows `UuidTable` pattern
- [x] Doc comments explain use case
- [x] Tests cover compile-time and runtime behavior
- [x] Missing key case tested

### Cycle 2.2: UuidPathTable Implementation

**RED:**
```rust
mod uuid_path_table {
    #[test]
    fn inserts_and_retrieves_uuid_to_pathkey() {
        let db = redb::Database::create(":memory:").expect("db");
        const TABLE: UuidPathTable<TestId> = UuidPathTable::new("test");

        let id = TestId(UuidV7::new());
        let key = PathKey::try_new("notes/test.md").expect("key");

        // Write
        let tx = db.begin_write().expect("tx");
        {
            let mut table = tx.open_table(TABLE.definition()).expect("open");
            table.insert(&id, &key).expect("insert");
        }
        tx.commit().expect("commit");

        // Read
        let tx = db.begin_read().expect("tx");
        let table = tx.open_table(TABLE.definition()).expect("open");
        let retrieved = table.get(&id).expect("get").expect("value");

        assert_eq!(retrieved.value(), key);
    }

    #[test]
    fn supports_path_recovery_for_deletes() {
        // Demonstrates reverse index use case
        let db = redb::Database::create(":memory:").expect("db");
        const TABLE: UuidPathTable<TestId> = UuidPathTable::new("test");

        let id = TestId(UuidV7::new());
        let key = PathKey::try_new("notes/daily.md").expect("key");

        // Store
        let tx = db.begin_write().expect("tx");
        {
            let mut table = tx.open_table(TABLE.definition()).expect("open");
            table.insert(&id, &key).expect("insert");
        }
        tx.commit().expect("commit");

        // Recover path by ID (for delete operations)
        let tx = db.begin_read().expect("tx");
        let table = tx.open_table(TABLE.definition()).expect("open");
        let recovered_path = table.get(&id).expect("get").expect("value").value();

        assert_eq!(recovered_path, key);
    }
}
```

**GREEN:**
```rust
/// Table mapping UUID → PathKey (reverse index).
///
/// Use this for ID-to-path lookups, enabling O(1) path recovery during delete
/// operations and ID-based queries.
///
/// # Examples
///
/// ```
/// use lithos_core::db::UuidPathTable;
/// use lithos_core::vault::FileId;
///
/// const PATH_BY_FILE_ID: UuidPathTable<FileId> = UuidPathTable::new("path_by_file_id");
/// ```
pub struct UuidPathTable<K: UuidV7DbType + 'static> {
    definition: TableDefinition<'static, K, PathKey>,
}

impl<K: UuidV7DbType + 'static> UuidPathTable<K> {
    /// Create a new UUID → PathKey table definition.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> TableDefinition<'static, K, PathKey> {
        self.definition
    }
}
```

**Verification:** `cargo test -p lithos-core db::table::tests::uuid_path_table`

**Checklist:**
- [x] Mirror of PathUuidTable (reverse direction)
- [x] Doc comments explain reverse index use case
- [x] Tests demonstrate delete recovery pattern

### Cycle 2.3: Update Vault Table Definitions

**Location:** `lithos-core/src/vault/storage/tables.rs`

**RED:** Change table types - existing tests should still pass

**GREEN:**
```rust
// Before:
pub const FILE_ID_BY_PATH: PathTable<FileId> = PathTable::new("file_id_by_path");
pub const PATH_BY_FILE_ID: UuidTable<FileId, String> = UuidTable::new("path_by_file_id");

pub const DIR_ID_BY_PATH: PathTable<DirId> = PathTable::new("dir_id_by_path");
pub const PATH_BY_DIR_ID: UuidTable<DirId, String> = UuidTable::new("path_by_dir_id");

// After:
pub const FILE_ID_BY_PATH: PathUuidTable<FileId> = PathUuidTable::new("file_id_by_path");
pub const PATH_BY_FILE_ID: UuidPathTable<FileId> = UuidPathTable::new("path_by_file_id");

pub const DIR_ID_BY_PATH: PathUuidTable<DirId> = PathUuidTable::new("dir_id_by_path");
pub const PATH_BY_DIR_ID: UuidPathTable<DirId> = UuidPathTable::new("path_by_dir_id");
```

**Update imports:**
```rust
use crate::db::{PathUuidTable, UuidPathTable, UuidTable, ...};
```

**Verification:** `cargo test -p lithos-core vault::`

**Checklist:**
- [x] 4 table definitions updated
- [x] All vault tests pass
- [x] No String in PathKey-related tables
- [x] Self-documenting table names

---

## Phase 3: Filesystem Layer Fix (Priority 3)

**Goal:** Fix `vault/processor.rs` to use typed FS paths before PathKey conversion

**Why Third:** Depends on Phase 1 (`as_key()` method works), isolated scope

**Files:**
- **Modified:** `lithos-core/src/vault/processor.rs`

### Cycle 3.1: Update Filesystem Scanning

**Problem:** Lines 527, 560, 866 convert `RelativePath` directly to `PathKey` via string

**RED:**
```rust
#[cfg(test)]
mod tests {
    mod path_conversion {
        use super::*;
        use tempfile::TempDir;

        #[test]
        fn converts_dir_paths_via_typed_fs_layer() {
            let temp = TempDir::new().expect("temp");
            let root = DirPath::try_new(temp.path().to_path_buf()).expect("root");
            let notes_dir = temp.path().join("notes");
            std::fs::create_dir_all(&notes_dir).expect("create dir");

            let dir_path = DirPath::try_new(notes_dir).expect("dir path");
            let key = dir_path.as_key(&root).expect("key");

            assert_eq!(key.as_str(), "notes");
        }

        #[test]
        fn converts_file_paths_via_typed_fs_layer() {
            let temp = TempDir::new().expect("temp");
            let root = DirPath::try_new(temp.path().to_path_buf()).expect("root");
            let file_abs = temp.path().join("notes/daily.md");
            std::fs::create_dir_all(file_abs.parent().unwrap()).expect("dirs");
            std::fs::write(&file_abs, "# test").expect("write");

            let file_path = FilePath::try_new(file_abs).expect("file path");
            let key = file_path.as_key(&root).expect("key");

            assert_eq!(key.as_str(), "notes/daily.md");
        }
    }
}
```

**GREEN:**

**Step 1:** Remove old helper function (lines 859-870):
```rust
// DELETE THIS:
fn normalized_path_from_relative(
    relative: &Path,
) -> Result<PathKey, VaultFileError> {
    let raw = relative.to_str().ok_or_else(...)?;
    PathKey::try_new(raw).map_err(...)
}
```

**Step 2:** Update `scan_views` to use `FsPath` directly:

The key insight: `FsReader::filter_dir_entries()` and `filter_file_entries()` already return
typed `FsDir` and `FsFile` entries with `DirPath` and `FilePath` internally. We just need to
call `.as_key(root)` on them.

```rust
fn scan_views(source: &FsReader) -> Result<ScanViews, VaultFileError> {
    // Directories
    let mut dir_entries: Vec<(PathKey, crate::fs::entry::FsDir)> = source
        .filter_dir_entries("**/*")
        .map_err(|error| VaultFileError::ReadFailed {
            path: "<vault>".into(),
            message: error.to_string().into(),
        })?
        .into_iter()
        .map(|entry| {
            // Use typed DirPath → as_key(root) → PathKey
            let key = entry.path().as_key(source.root())
                .map_err(|error| VaultFileError::InvalidPath {
                    path: error.to_string().into(),
                    reason: "path conversion failed".into(),
                })?;
            Ok((key, entry))
        })
        .collect::<Result<Vec<_>, VaultFileError>>()?;

    // Sort by depth
    dir_entries.sort_by(|(key_a, _), (key_b, _)| {
        let depth_a = key_a.as_str().split('/').count();
        let depth_b = key_b.as_str().split('/').count();
        depth_a.cmp(&depth_b).then_with(|| key_a.as_str().cmp(key_b.as_str()))
    });

    let mut dirs = Vec::with_capacity(dir_entries.len());
    let mut dir_ids_by_path = HashMap::with_capacity(dir_entries.len());

    for (key, entry) in dir_entries {
        let parent = parent_path(&key)?;
        let parent_id = parent.as_ref()
            .and_then(|parent_key| dir_ids_by_path.get(parent_key))
            .copied();

        let dir = ScannedDir {
            path: key.clone(),
            view: DirView::new(
                DirId::new(),
                parent_id,
                DirName::new(last_component_from_key(&key)?),
                entry.metadata().clone(),
            ),
        };

        dir_ids_by_path.insert(key, dir.view.id());
        dirs.push(dir);
    }

    // Files (similar pattern)
    let file_entries = source.filter_file_entries("**/*")
        .map_err(|error| VaultFileError::ReadFailed {
            path: "<vault>".into(),
            message: error.to_string().into(),
        })?;

    let mut files = Vec::with_capacity(file_entries.len());
    for file_entry in file_entries {
        let key = file_entry.path().as_key(source.root())
            .map_err(|error| VaultFileError::InvalidPath {
                path: error.to_string().into(),
                reason: "path conversion failed".into(),
            })?;

        let parent = parent_path(&key)?;
        let parent_id = parent.as_ref()
            .and_then(|parent_key| dir_ids_by_path.get(parent_key))
            .copied();

        let format = file_entry.path().extension_ref()
            .map_or(FileFormat::Unknown, FileFormat::from_extension);

        let file = ScannedFile {
            path: key.clone(),
            view: FileView::new(
                FileId::new(),
                parent_id,
                FileName::new(last_component_from_key(&key)?),
                format,
                file_entry.metadata().clone(),
                [0u8; 32],
            ),
        };
        files.push(file);
    }

    Ok(ScanViews { dirs, files })
}

// Helper: extract last component from PathKey
fn last_component_from_key(key: &PathKey) -> Result<Box<str>, VaultFileError> {
    Path::new(key.as_str())
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.into())
        .ok_or_else(|| VaultFileError::InvalidPath {
            path: key.as_str().into(),
            reason: "missing terminal path component".into(),
        })
}
```

**Verification:** `cargo test -p lithos-core vault::processor`

**Checklist:**
- [x] No `PathKey::try_new()` in scanning logic
- [x] Uses `entry.path().as_key(root)?` pattern
- [x] Existing vault tests pass
- [x] Filesystem layer properly separated

---

## Phase 4: Additional Tests (Priority 4)

**Goal:** Add any missing edge case tests

**Why Last:** Hardens implementation, not blocking

### Tests Already Covered

From Phases 1-3:
- ✅ PathKey serialization round-trip
- ✅ PathKey lexicographic ordering
- ✅ PathUuidTable insert/retrieve
- ✅ PathUuidTable missing key
- ✅ UuidPathTable insert/retrieve
- ✅ UuidPathTable delete recovery
- ✅ Filesystem typed conversion (files, dirs)

### Additional Tests (if needed)

Add only if gaps found during implementation:

```rust
#[test]
fn pathkey_handles_unicode_correctly() {
    let key = PathKey::try_new("notes/日本語.md").expect("valid");
    let bytes = PathKey::as_bytes(&key);
    let deserialized = PathKey::from_bytes(bytes);
    assert_eq!(deserialized, key);
}

#[test]
fn path_uuid_table_handles_concurrent_reads() {
    // Multi-threaded read test if redb supports it
}
```

---

## Refactor Checklist

After all tests pass:

- [ ] **Review borrowing** (Rust BP Ch. 1):
  - No unnecessary `.to_owned()` in storage layer
  - `&PathKey` used consistently in APIs
  - No gratuitous String allocations

- [ ] **Check clippy** (Rust BP Ch. 2):
  - `cargo clippy -p lithos-core -- -D clippy::perf`
  - Watch for `redundant_clone`, `needless_collect`
  - No `large_enum_variant` warnings

- [ ] **Verify doc comments** (Rust BP Ch. 8):
  - PathKey redb traits document panic conditions
  - Table wrappers explain use cases
  - Examples compile and demonstrate patterns

- [ ] **Test naming** (unit-naming.md):
  - Use Structure A (submodules)
  - Canonical module names: `serialization`, `path_uuid_table`, `uuid_path_table`
  - Verb-first function names: `returns_*`, `preserves_*`, `supports_*`

---

## Summary of Changes

### Files Modified

| File | Change Type | Lines | Risk |
|------|-------------|-------|------|
| `lithos-core/src/db/path.rs` | **New file** | ~80 | LOW |
| `lithos-core/src/db/mod.rs` | Add module | ~1 | LOW |
| `lithos-core/src/db/table.rs` | Add wrappers, change PathTable | ~100 | LOW |
| `lithos-core/src/vault/storage/tables.rs` | Update 4 tables | ~4 | LOW |
| `lithos-core/src/vault/storage/read.rs` | Remove `.to_owned()` | ~10 | LOW |
| `lithos-core/src/vault/storage/write.rs` | Remove `.to_owned()` | ~4 | LOW |
| `lithos-core/src/vault/processor.rs` | Fix FS layer | ~50 | MEDIUM |

### Tests Added

- **Phase 1:** 4 tests (PathKey redb traits)
- **Phase 2:** 6 tests (PathUuidTable, UuidPathTable)
- **Phase 3:** 2 tests (Typed FS conversion)
- **Phase 4:** 0-3 tests (Edge cases if needed)

**Total:** ~12-15 new tests

### Risk Assessment

- **LOW:** Trait implementations, table wrappers (additive changes)
- **MEDIUM:** vault/processor.rs refactor (changes scanning logic, but isolated scope)

### Rollback Strategy

- Each phase commits independently
- Phase 1 can be reverted without affecting other work
- Phase 2-3 depend on Phase 1, but can be individually reverted

---

## Definition of Done

- [ ] All tests pass (`mise run test`)
- [ ] Code formatted (`mise run fmt`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] PathKey implements `redb::Value` and `redb::Key`
- [ ] `PathUuidTable` and `UuidPathTable` exist with tests
- [ ] `PathTable` uses `PathKey` instead of `String`
- [ ] Vault storage uses new table wrappers
- [ ] No `.to_owned()` on PathKey in storage
- [ ] vault/processor.rs uses typed FS paths
- [ ] All public APIs have doc comments
- [ ] Tests cover critical paths and edge cases
- [ ] No `unwrap()` in production code (except redb traits)
- [ ] Full verification passes (`mise run verify`)

---

## Related Issues

**Upstream:**
- Issue 05: Schema context migration - may have similar filesystem layer violations

**Downstream:**
- Issue 07: Note/template migration - will follow same patterns (redb traits, table wrappers)

**Notes to Add:**
- Issue 07: Check for filesystem layer violations in note/template processor
- Issue 05 or new issue: Audit schema context for `PathKey::try_new()` bypass patterns
