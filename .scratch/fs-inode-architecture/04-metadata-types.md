---
title: 04-fs-metadata-types
category: enhancement
label: completed
status: completed
date_created: 2026-05-11
date_completed: 2026-05-12
---

## Type

AFK

## Labels

- completed

## What to build

Create fs/metadata.rs: FsTimes (created_at, modified_at), FileMetadata (times, size, is_symlink), DirMetadata (times, is_symlink), FsMetadata enum (File/Dir variants).

Split from existing FileInfo. FsMetadata provides type-level distinction between files and directories.

## Acceptance criteria

- [x] FsTimes with created_at, modified_at (Option<SystemTime>, rkyv with AsUnixTime)
- [x] FsTimes::is_match() for staleness detection
- [x] FileMetadata: times (FsTimes), size (u64), is_symlink (bool)
- [x] DirMetadata: times (FsTimes), is_symlink (bool) - no size (not meaningful for dirs)
- [x] FsMetadata enum: File(FileMetadata), Dir(DirMetadata)
- [x] FsMetadata helpers: is_file(), is_dir(), as_file(), as_dir()
- [x] rkyv archived type support
- [x] Tests for timestamp extraction, metadata construction
- [x] Update fs/mod.rs exports

## Blocked by

None - can start immediately

## Agent Brief

**Category:** enhancement
**Summary:** Separate filesystem metadata into specialized types for files and directories.

**Current behavior:**
The project uses a unified `FileInfo` struct that contains timestamps and file size. However, "size" is not a meaningful or consistent property for directories across all platforms, and using the same struct for both creates ambiguity in the domain model.

**Desired behavior:**
Extract timestamp logic into a reusable `FsTimes` struct. Create distinct `FileMetadata` (which includes `size`) and `DirMetadata` (which does not) types. Unify these in an `FsMetadata` enum to allow for type-safe metadata handling.

**Key interfaces:**
- `FsTimes` — wraps `created_at` and `modified_at` (`Option<SystemTime>`)
- `FileMetadata` — contains `FsTimes`, `size` (u64), and `is_symlink`
- `DirMetadata` — contains `FsTimes` and `is_symlink`
- `FsMetadata` — enum with `File(FileMetadata)`, `Dir(DirMetadata)` variants

**Acceptance criteria:**
- [ ] `FsTimes` implements `is_match()` for staleness detection (comparing timestamps)
- [ ] `FsMetadata` provides helper methods: `is_file()`, `is_dir()`, `as_file()`, `as_dir()`
- [ ] Types are `rkyv`-compatible using `AsUnixTime` for `SystemTime` serialization
- [ ] `DirMetadata` explicitly excludes the `size` field
- [ ] Tests verify correct extraction from `std::fs::Metadata`

**Out of scope:**
- Content hashing (reserved for Vault module)
- Migrating existing `FileInfo` consumers (reserved for Issue 08)

## Implementation Notes

**File:** `lithos-core/src/fs/metadata.rs`

**Implemented Types:**

**FsTimes:**
- `created_at: Option<SystemTime>` - Creation time (platform-dependent)
- `modified_at: Option<SystemTime>` - Modification time
- `rkyv` serialization with `AsUnixTime` wrapper for cross-platform compatibility
- `is_match(&other) -> bool` - Staleness detection by timestamp comparison

**FileMetadata:**
- `times: FsTimes` - Creation and modification timestamps
- `size: u64` - File size in bytes
- `is_symlink: bool` - Symbolic link indicator
- Type-safe file-specific metadata (includes size)

**DirMetadata:**
- `times: FsTimes` - Creation and modification timestamps
- `is_symlink: bool` - Symbolic link indicator
- **No size field** - Size is not meaningful/portable for directories

**FsMetadata enum:**
```rust
pub enum FsMetadata {
    File(FileMetadata),
    Dir(DirMetadata),
}
```

**Key Methods:**
- `FsTimes::new(created, modified)` - Constructor
- `FsTimes::is_match(&other)` - Compare timestamps for staleness detection
- `FileMetadata::new(times, size, is_symlink)` - Constructor
- `DirMetadata::new(times, is_symlink)` - Constructor (no size)
- `FsMetadata::is_file()`, `is_dir()` - Variant discrimination
- `FsMetadata::as_file()`, `as_dir()` - Safe variant access
- `TryFrom<std::fs::Metadata>` for `FsMetadata` - Convert from stdlib

**Conversions:**
- `TryFrom<std::fs::Metadata>` - Extracts timestamps, size, and symlink status
- Automatically discriminates between file and directory variants
- Returns `io::Error` if metadata is neither file nor directory

**Tests:** 17 tests covering:
- FsTimes construction with various timestamp combinations
- Staleness detection with `is_match()`
- FileMetadata construction and field access
- DirMetadata construction without size field
- FsMetadata variant discrimination and accessors
- Conversion from `std::fs::Metadata` for files and directories
- Edge cases: missing timestamps, None values

**Module Integration:**
- Registered in `fs/mod.rs`
- Exported: `FsTimes`, `FileMetadata`, `DirMetadata`, `FsMetadata`

**Status:** ✅ Complete - All acceptance criteria met

## Additional Implementation Notes (2026-05-13)

### `FsMetadata::from_path()` API Addition

Added `FsMetadata::from_path<P: AsRef<Path>>(path: P) -> io::Result<Self>` to provide an idiomatic, stdlib-style API for creating metadata from filesystem paths.

**Design Rationale:**
- **Matches stdlib idiom**: Mirrors `std::fs::metadata(path)` pattern (free function-style API)
- **Type ownership**: Type owns its construction logic, not the caller (`FileReader`)
- **Ergonomics**: Generic `<P: AsRef<Path>>` accepts `&Path`, `PathBuf`, `&str` without explicit conversion
- **Discoverability**: `FsMetadata::from_path(path)` is more explicit than `path.try_into()`

**Considered Alternative: `impl TryFrom<&Path> for FsMetadata`**

Rejected because:
- Less discoverable than explicit `from_path()` method
- `TryFrom` trait better suited for type conversions, not filesystem I/O
- Generic flexibility (`AsRef<Path>`) provides better ergonomics
- Domain clarity: "from_path" signals filesystem operation

**Implementation:**
```rust
pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    let std_meta = std::fs::metadata(path.as_ref())?;
    Self::try_from(std_meta)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
```

**Existing `TryFrom<Metadata>` preserved:**
- Different use case: raw metadata → typed metadata conversion
- `from_path()` delegates to it internally
- Useful when caller already has `std::fs::Metadata`

**Delegation Pattern:**
`FileReader::metadata()` now delegates to `FsMetadata::from_path()`:
```rust
pub fn metadata(&self, path: &Path) -> Result<FsMetadata, ParseError> {
    let full_path = self.root.join(path);
    FsMetadata::from_path(&full_path).map_err(|e| ParseError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}
```

Benefits:
- Single Responsibility: `FileReader` handles path resolution, `FsMetadata` handles construction
- Testability: Can test `from_path()` independently of `FileReader`
- Reusability: Direct usage for absolute paths without `FileReader`

### Test Suite Reorganization

Reorganized test suite following Apollo Rust Best Practices (Chapter 5: Automated Testing).

**Before:**
- 23 tests in flat `tests` module
- Mix of naming conventions
- No logical grouping

**After:**
```rust
#[cfg(test)]
mod tests {
    mod fs_times {
        mod is_match { ... }
    }
    mod file_metadata { ... }
    mod dir_metadata { ... }
    mod fs_metadata {
        mod as_file { ... }
        mod as_dir { ... }
        mod try_from { ... }
        mod from_path { ... }
    }
}
```

**Benefits:**
- ✅ Logical grouping by type (`fs_times`, `file_metadata`, `dir_metadata`, `fs_metadata`)
- ✅ Behavior submodules (`is_match`, `as_file`, `as_dir`, `try_from`, `from_path`)
- ✅ IDE support: Each module gets a ▶️ run button
- ✅ Readable test output: `fs_metadata::from_path::constructs_file_metadata_for_file`
- ✅ Naming consistency: Behavior-first naming (`returns_true_for_identical_timestamps`)

**Test Coverage:**
- 23 tests total (6 `fs_times`, 3 `file_metadata`, 3 `dir_metadata`, 11 `fs_metadata`)
- All tests follow "one behavior per test" principle
- Descriptive names matching `unit_of_work::expected_behavior_when_state` pattern

**Verification:**
- ✅ 1124 unit tests passing
- ✅ 190 doctests passing (155 run, 35 ignored)
- ✅ Clippy clean with strict lints
- ✅ Follows Rust best practices for test organization
