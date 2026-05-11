# PRD: Filesystem Type Consolidation and Inode-Based Vault Architecture

**Status**: ready-for-agent
**Created**: 2026-05-09
**Context**: Refactor fs/ and vault/ modules to eliminate duplication, establish clear separation of concerns, and implement inode-based file tracking with stable identity across renames.

---

## Problem Statement

The current fs/ and vault/ modules have overlapping responsibilities and duplicate types, leading to confusion about ownership and maintenance burden:

1. **Type duplication**: `VaultFile`/`VaultFolder` duplicate `FileInfo` timestamp/size tracking and path decomposition logic already available via std::path
2. **Path-as-identity limitation**: Current path-based storage cannot efficiently handle file renames, move operations, or maintain stable references across path changes
3. **Missing zero-copy primitives**: No borrowed path component views (filename, extension, parent) for efficient extraction without allocation
4. **Format detection scattered**: `FormatKind` is internal to fs/types.rs but needed for vault storage queries (e.g., "list all markdown files")
5. **Unclear module boundaries**: vault/ contains both infrastructure (file discovery) and domain logic (metadata tracking), violating separation of concerns

Users need efficient wikilink resolution by basename, folder-based filtering by parent path, and file type queries (markdown vs images vs other) - all with stable identity that survives renames.

---

## Solution

Consolidate filesystem primitives into fs/ as infrastructure building blocks, and refactor vault/ to use an inode-based architecture with UUID identity and multi-index storage:

1. **fs/ provides infrastructure primitives**: Path types, metadata extraction, runtime entities, zero-copy views, format detection
2. **vault/ uses fs/ primitives**: Composes them into domain types (StoredFile/StoredDir) with inode identity, parent links, and query-optimized indexes
3. **Inode-based storage**: Files/directories get stable UUID identity, path becomes a secondary index, renames update index only
4. **Multi-index queries**: Separate multimap indexes for basename (wikilink resolution), parent (folder filtering), and format (markdown/image queries)

---

## User Stories

1. As a developer, I want a single source of truth for file metadata extraction, so that timestamp/size comparison logic is consistent across all contexts
2. As a developer, I want zero-copy path component views, so that I can efficiently extract basename/extension/parent without allocating strings
3. As a developer, I want clear ownership of path validation, so that I know whether fs/ or vault/ is responsible for security checks
4. As a developer, I want file format detection available as a public type, so that I can query files by type (markdown, images, etc.) without string parsing
5. As a developer, I want to rename a top-level directory, so that I don't need to update thousands of child file records (instant rename via path index update)
6. As a user, I want wikilinks to remain valid when files are renamed, so that my knowledge graph stays connected (stable FileId references)
7. As a developer, I want to resolve wikilinks by basename, so that `[[my-note]]` finds all files named "my-note" regardless of parent directory
8. As a developer, I want to filter files by parent directory, so that I can implement folder-based views efficiently
9. As a developer, I want to list all markdown files, so that I can implement Obsidian-like `vault.getMarkdownFiles()` API
10. As a developer, I want separate owned and borrowed filename types, so that I can avoid allocations in hot paths (zero-copy extraction) while still storing filenames in domain models
11. As a developer, I want file and directory entities to be distinguishable at the type level, so that I cannot accidentally use a file path where a directory is required
12. As a developer, I want parent directory lookups to be O(1), so that directory tree traversal during ingestion is performant
13. As a developer, I want content hashing to be optional per file, so that binary files (images, PDFs) don't incur hashing overhead unnecessarily
14. As a developer, I want directory metadata to exclude size fields, so that directory storage is minimal (no file size makes sense for dirs)
15. As a developer, I want walkdir iteration order guarantees to be explicit, so that parent directories are always processed before children (enables depth-first cache strategy)
16. As a developer, I want path normalization rules to be documented, so that I understand when to use forward slashes, how case is handled, and what UTF-8 constraints exist
17. As a developer, I want migration from VaultFile to StoredFile to be type-safe, so that the compiler catches any missed conversions
18. As a developer, I want existing tests to guide refactoring, so that I can verify behavior is preserved during the migration
19. As a developer, I want FileFormat to support future extension types, so that adding PDF/video/archive support doesn't require table schema changes
20. As a developer, I want staleness detection to work identically across schema, config, and vault contexts, so that file change detection is consistent

---

## Implementation Decisions

### Module Responsibilities

**fs/ (Infrastructure - building blocks):**
- Path types: `AbsolutePath`, `RelativePath`, `FilePath`, `DirPath`
- Zero-copy views: `FileNameRef<'a>`, `DirNameRef<'a>`, `BaseNameRef<'a>`, `FileExtensionRef<'a>`, `ParentDir<'a>`
- Owned components: `FileName`, `DirName`, `BaseName` (using suffix pattern: no suffix = owned, `Ref` suffix = borrowed)
- Metadata: `FsTimes`, `FileMetadata`, `DirMetadata` (split from FileInfo)
- Runtime entities: `FsFile`, `FsDir`, `FsEntry`
- Format detection: `FileFormat` (refactored from FormatKind, public, rkyv-enabled)
- Utilities: `FsReader`, `FsWriter`, `DirScanner`, `PathValidator`

**vault/ (Domain - inode-based tracking):**
- Identifiers: `FileId(UuidV7)`, `DirId(UuidV7)`
- Storage entities: `FileView`, `DirView`, `FsEntryView` (compose fs/ primitives)
- Database keys: `NormalizedPath` (vault-relative, forward-slash normalized)
- Repository: CRUD + query methods (by path, basename, parent, format)
- Processor: File discovery pipeline (FsReader → FileView/DirView → save)

**Deleted types (redundant):**
- `vault::VaultPath` → Use `NormalizedPath` for storage keys, `RelativePath` for validation
- `vault::VaultFile` → Replace with `FileView`
- `vault::VaultFolder` → Replace with `DirView`
- `fs::FileEntry` → Replace with `FsFile`
- `fs::FileInfo` → Rename to `FileMetadata` (file-specific), extract `FsTimes`

### Type Hierarchy

```rust
// fs/path.rs
pub struct AbsolutePath(PathBuf);           // Validated absolute path
pub struct RelativePath(PathBuf);           // Validated relative path (no .., no absolute)
pub struct FilePath(RelativePath);          // Vault-relative file path
pub struct DirPath(RelativePath);           // Vault-relative directory path

// fs/path.rs - Zero-copy borrowed views
pub struct FileNameRef<'a>(&'a OsStr);      // Borrowed filename view
pub struct DirNameRef<'a>(&'a OsStr);       // Borrowed dirname view
pub struct BaseNameRef<'a>(&'a OsStr);      // Borrowed basename view (file stem)
pub struct FileExtensionRef<'a>(&'a OsStr); // Borrowed extension view
pub enum ParentDir<'a> { Root, Path(&'a Path) }

// fs/file.rs - Owned components
pub struct FileName(Box<str>);              // Owned filename (UTF-8)
pub struct DirName(Box<str>);               // Owned dirname (UTF-8)
pub struct BaseName(Box<str>);              // Owned basename (UTF-8, Obsidian terminology)

// fs/file.rs - Metadata components
pub struct FsTimes {
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
}

pub struct FileMetadata {
    times: FsTimes,
    size: u64,
    is_symlink: bool,
}

pub struct DirMetadata {
    times: FsTimes,
    is_symlink: bool,
}

// fs/file.rs - Runtime entities
pub struct FsFile {
    path: FilePath,
    metadata: FileMetadata,
}

pub struct FsDir {
    path: DirPath,
    metadata: DirMetadata,
}

pub enum FsEntry {
    File(FsFile),
    Dir(FsDir),
}

// fs/types.rs - Format detection
pub enum FileFormat {
    Json, Toml, Yaml, Markdown,
    Image, Document, Archive, Binary, Unknown
}

// vault/model.rs - Identifiers
pub struct FileId(UuidV7);
pub struct DirId(UuidV7);

// vault/model.rs - Storage entities (View suffix matches codebase pattern)
pub struct FileView {
    id: FileId,
    parent_id: Option<DirId>,
    name: String,                           // Just filename (no path)
    format: FileFormat,
    metadata: FileMetadata,
    content_hash: Option<Blake3Hash>,
}

pub struct DirView {
    id: DirId,
    parent_id: Option<DirId>,
    name: String,                           // Just dirname (no path)
    metadata: DirMetadata,
}

// vault/model.rs - Unified entry view
pub enum FsEntryView {
    File(FileView),
    Dir(DirView),
}

impl FsEntryView {
    pub fn id_bytes(&self) -> &[u8; 16];
    pub fn parent_id(&self) -> Option<DirId>;
    pub fn name(&self) -> &str;
    pub fn is_file(&self) -> bool;
    pub fn is_dir(&self) -> bool;
}

// vault/model.rs - Database key
pub struct NormalizedPath(String);          // Vault-relative, forward slashes
```

### Storage Tables

**Primary inode tables:**
```rust
Table<FileId, FileView>                     // [u8; 16] → FileView bytes
Table<DirId, DirView>                       // [u8; 16] → DirView bytes
```

**Path index tables:**
```rust
Table<NormalizedPath, FileId>               // "2024/daily/note.md" → FileId
Table<NormalizedPath, DirId>                // "2024/daily" → DirId
```

**Query optimization indexes (multimap):**
```rust
MultimapTable<BaseName, FileId>             // "note" → [FileId, ...]
MultimapTable<Parent, FileId>               // "2024/daily" → [FileId, ...]
MultimapTable<FileFormat, FileId>           // Markdown → [FileId, ...]
```

**Deleted tables:**
- `VAULT_FILES_BY_PATH` → Replaced by FileId primary table + NormalizedPath index
- `VAULT_FOLDERS_BY_PATH` → Replaced by DirId primary table + NormalizedPath index

### Ingestion Pipeline

**Single-pass walkdir scan with lazy parent creation:**

1. Iterate WalkDir with default depth-first ordering (parents before children)
2. Maintain in-memory `HashMap<PathBuf, DirId>` cache for parent lookups
3. For each entry:
   - Extract vault-relative path
   - Lookup parent DirId from cache (O(1))
   - If directory: create DirView, save to DB, cache DirId
   - If file: detect format, optionally hash content, create FileView, save to DB
4. Update all indexes (path, basename, parent, format) during save

**Performance characteristics:**
- O(1) parent lookups (cache hit)
- Zero database queries during scan (pure writes)
- Single transaction for entire scan (atomic vault update)

### Path Normalization Rules

**NormalizedPath canonical form:**
- Vault-relative (strip vault root prefix)
- Forward slashes only (`\` → `/`)
- Case-preserved (no lowercase conversion)
- UTF-8 enforced (reject invalid UTF-8 paths)
- No trailing slashes for directories

**Rationale:**
- Case-sensitive storage matches Unix semantics
- Forward slash normalization enables cross-platform storage
- Case-insensitive queries handled at query time (not storage time)

### Query Methods

```rust
impl Repository {
    // Exact lookups (O(1) hash)
    fn find_file_by_path(&self, path: &NormalizedPath) -> Option<FileId>;
    fn get_file(&self, id: FileId) -> Result<FileView, Error>;
    fn get_dir(&self, id: DirId) -> Result<DirView, Error>;
    fn get_entry(&self, id_bytes: &[u8; 16]) -> Result<FsEntryView, Error>;

    // Indexed queries (O(1) multimap lookup)
    fn find_files_by_basename(&self, basename: &str) -> Vec<FileId>;
    fn find_files_by_parent(&self, parent: &str) -> Vec<FileId>;
    fn list_markdown_files(&self) -> Vec<FileId>;
    fn list_files_by_format(&self, format: FileFormat) -> Vec<FileId>;

    // Full scans (O(n))
    fn list_all_files(&self) -> Vec<FileView>;
    fn list_all_dirs(&self) -> Vec<DirView>;
}
```

### Migration Strategy

**Phase 1: Create new types in fs/ (no breaking changes)**
- Add FilePath(RelativePath), DirPath(RelativePath) to fs/path.rs
- Add FsFile, FsDir, FsEntry to fs/file.rs
- Add zero-copy views: FileNameRef, DirNameRef, BaseNameRef, FileExtensionRef, ParentDir
- Add owned components: FileName, DirName, BaseName
- Split FileInfo → FsTimes + FileMetadata + DirMetadata
- Refactor FormatKind → FileFormat (public, rkyv derives)
- Keep old types temporarily for backward compat

**Phase 2: Create vault storage layer (parallel to existing)**
- Implement FileId, DirId, FileView, DirView, FsEntryView, NormalizedPath
- Create new storage tables (FILES_BY_ID, DIRS_BY_ID, etc.)
- Implement Repository trait with new signatures (including get_entry)
- Implement RedbRepository adapter
- Add multimap indexes incrementally

**Phase 3: Update vault processor (cut over)**
- Refactor processor pipeline to use new types
- Update FsReader → FsFile/FsDir conversion
- Update save logic to populate all indexes
- Verify existing tests pass with new implementation

**Phase 4: Delete old types (breaking change)**
- Remove VaultPath, VaultFile, VaultFolder
- Remove old VAULT_FILES_BY_PATH, VAULT_FOLDERS_BY_PATH tables
- Remove FileEntry
- Rename FileInfo → FileMetadata (keep alias temporarily)

**Phase 5: Update dependent contexts (if needed)**
- schema/ - Already uses FileInfo, should work with FileMetadata rename
- config/ - May need minor adjustments for renamed types
- note/ - Should be unaffected (doesn't import vault/)

---

## Testing Decisions

### What Makes a Good Test

**Test external behavior, not implementation details:**
- Test that `FileView::from_fs` correctly extracts metadata from FsFile
- Test that `Repository::find_files_by_basename("note")` returns all matching FileIds
- Test that parent directory renames only update path index (not FileView records)
- Test that `FsEntryView::id_bytes()` returns correct bytes for both File and Dir variants
- **Don't test**: Internal HashMap structure, walkdir iteration order, cache hit rates

**Test domain invariants:**
- FileId/DirId uniqueness (no collisions)
- Parent links form valid tree (no cycles, no orphans)
- Path index consistency (NormalizedPath → FileId matches FileView.name)
- Format detection determinism (same extension → same FileFormat)
- FsEntryView correctly distinguishes File vs Dir variants

**Test edge cases:**
- Empty directories (no files)
- Files without extensions
- Non-UTF-8 paths (should error gracefully)
- Symlinks (metadata.is_symlink correctly set)
- Vault root files (parent_id = None)

### Modules to Test

**fs/path.rs:**
- FilePath/DirPath wrapping RelativePath (vault-scoped validation)
- RelativePath validation (no .., no absolute paths)
- Zero-copy view extraction (FileNameRef, BaseNameRef, ParentDir)
- Conversions between path types

**fs/file.rs:**
- FsTimes timestamp extraction from std::fs::Metadata
- FileMetadata/DirMetadata construction
- FsFile/FsDir from DirEntry conversion
- FileName/DirName/BaseName owned component creation

**fs/types.rs:**
- FileFormat::from_extension coverage (all supported extensions)
- Format detection case-insensitivity
- is_markdown(), is_structured() helpers

**vault/model.rs:**
- FileId/DirId generation (UuidV7 monotonicity)
- FileView::from_fs conversion preserves all metadata
- FsEntryView enum correctly wraps FileView and DirView
- FsEntryView helper methods (id_bytes, parent_id, name, is_file, is_dir)
- NormalizedPath normalization rules (slash conversion, case preservation)
- NormalizedPath::basename(), parent() extraction

**vault/storage.rs:**
- Repository CRUD operations (save, get, delete)
- Repository::get_entry returns correct FsEntryView variant
- Index consistency (path index matches primary table)
- Multimap queries (basename, parent, format)
- Transaction rollback on error
- Batch operations (save multiple files atomically)

**vault/processor.rs:**
- Full vault scan produces complete FileView/DirView set
- Parent DirIds correctly linked (child.parent_id points to parent)
- Walkdir ordering guarantees parent-before-child
- Empty directory handling
- Markdown file routing to note processor (existing behavior)

### Prior Art for Tests

**Path validation tests:**
- Existing: `fs/path.rs` tests for RelativePath/AbsolutePath
- Pattern: rstest parametric tests for valid/invalid cases
- Extend with FilePath/DirPath validation

**Metadata extraction tests:**
- Existing: `fs/file.rs` tests for FileInfo from std::fs::Metadata
- Pattern: tempfile-based tests creating real files
- Extend with FileMetadata/DirMetadata construction

**Storage tests:**
- Existing: `schema/storage.rs` tests for RedbRepository
- Pattern: tempfile DB, save/load round-trip, index consistency
- Mirror for vault/storage.rs with FileView/DirView/FsEntryView

**Processor tests:**
- Existing: `vault/processor.rs` has typestate transition tests
- Pattern: Mock FsReader, verify state machine transitions
- Extend to verify new FileView/DirView creation

---

## Out of Scope

**Not included in this refactoring:**

1. **File watching/live updates**: This refactoring establishes storage architecture; file system watching (inotify, FSEvents) is future work
2. **Wikilink resolution implementation**: Provides query infrastructure (basename index) but doesn't implement link parsing/resolution
3. **Rename operation API**: Provides instant-rename capability via path index but doesn't expose user-facing rename command
4. **Content hashing strategy changes**: Keeps existing Blake3 hashing, doesn't evaluate alternatives
5. **Symlink following policy**: Preserves existing behavior (walkdir follow_symlinks setting), doesn't change link handling
6. **Cross-vault references**: Single vault only; multi-vault support is future work
7. **File history/versioning**: Stable identity enables history tracking but doesn't implement version storage
8. **Incremental sync**: Full vault scan only; delta/incremental updates are future optimization
9. **Note/schema processor changes**: Vault refactoring only; routing to note/schema processors unchanged
10. **CLI command changes**: Internal refactoring only; no new user-facing commands

---

## Further Notes

### Naming Conventions

**Suffix pattern for owned vs borrowed:**
- No suffix = owned (`FileName`, `DirName`, `BaseName`)
- `Ref` suffix = borrowed (`FileNameRef<'a>`, `DirNameRef<'a>`, `BaseNameRef<'a>`)

This follows Rust conventions (e.g., `Path` vs `PathBuf`, `str` vs `String`) while making ownership explicit through naming.

**Terminology choices:**
- `BaseName` follows **Obsidian terminology** (filename without extension) rather than Rust's `file_stem()`, since this is domain-aligned and more intuitive for users
- `*View` suffix for storage entities follows existing codebase pattern (`RawSchemaView`, `ListView`)

### Performance Implications

**Memory overhead:**
- UUIDs add 16 bytes per file/directory (vs path-based identity)
- In-memory cache during ingestion: O(directory count) = ~100KB for 10K dirs
- Multimap indexes: ~50 bytes per entry (basename string + FileId)

**Query performance:**
- Basename lookup: O(1) multimap → O(log n) iteration
- Parent filtering: O(1) multimap → O(children count) iteration
- Format filtering: O(1) multimap → O(file count for format) iteration
- Path rename: O(1) update to path index (vs O(n) update to all child records)

**Trade-off**: Higher storage cost for better query performance and rename efficiency.

### Relationship to Obsidian API

This refactoring provides infrastructure for Obsidian-compatible APIs:

- `Vault.getFiles()` → `Repository::list_all_files()`
- `Vault.getMarkdownFiles()` → `Repository::list_markdown_files()`
- `Vault.getAbstractFileByPath(path)` → `Repository::find_file_by_path(path)`
- `TFile.parent` → `FileView.parent_id` → `Repository::get_dir(parent_id)`
- `TFile.basename` → Extracted from `FileView.name` via `BaseName` type
- `TFile.extension` → `FileFormat` enum (richer than string extension)

Future work can build higher-level abstractions on this storage layer.

### ADR References

This refactoring implements principles from:
- **ADR-0009**: Type-driven design (FilePath vs DirPath distinction)
- **ADR-0011**: Zero-copy patterns (FilenameRef, ParentDir borrowed views)
- **ADR-0015**: Repository trait pattern (unified read/write interface)

Creates new architectural decisions:
- **Inode-based identity**: UUIDs vs path-based keys
- **Multi-index storage**: Path + basename + parent + format indexes
- **Metadata split**: FsTimes + FileMetadata + DirMetadata separation
