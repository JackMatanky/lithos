# PRD: Filesystem Type Consolidation and Inode-Based Vault Architecture

**Status**: completed
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
- Utilities: `FileReader`, `FsWriter`, `DirScanner`, `PathValidator`

**vault/ (Domain - inode-based tracking):**
- Identifiers: `FileId(UuidV7)`, `DirId(UuidV7)`
- Storage entities: `FileView`, `DirView`, `FsEntryView` (compose fs/ primitives)
- Database keys: `NormalizedPath` (vault-relative, forward-slash normalized)

### File Organization (Granular - Option A)

```
lithos-core/src/fs/
├── mod.rs          # Re-exports all public types
├── path.rs         # RelativePath, AbsolutePath, FilePath, DirPath, FsPath, ParentDir<'a>
├── name.rs         # FileName, DirName, BaseName (owned) + FileNameRef, DirNameRef, BaseNameRef<'a> (borrowed)
├── metadata.rs     # FsTimes, FileMetadata, DirMetadata, FsMetadata (unified enum)
├── entry.rs        # FsFile, FsDir, FsEntry (unified enum)
├── format.rs       # FileFormat (public, expanded from FormatKind) + FileExtensionRef<'a>
├── scanner.rs      # DirScanner, DirScanInput (paths()→Vec<FsPath>, entries()→Vec<FsEntry>)
├── reader.rs       # FileReader (metadata() returns FsMetadata, delete info())
├── error.rs        # FsError, ReadError, ScanError, ParseError, PathError, PathValidationError (see ADR 017)
└── validator.rs    # PathValidator (unchanged)
```

**Deleted files:**
- `fs/file.rs` → Contents split into name.rs, metadata.rs, entry.rs
- `fs/types.rs` → Contents moved to format.rs

### Reader and Scanner API Changes

**Final API (after migration):**

```rust
// DirScanner methods (returns ScanError — see ADR 017)
impl DirScanner {
    pub fn paths(&self, input: DirScanInput) -> Result<Vec<FsPath>, ScanError>;  // File or Dir
    pub fn entries(&self, input: DirScanInput) -> Result<Vec<FsEntry>, ScanError>;  // File or Dir
}

// FileReader methods (returns FsError compositor — see ADR 017)
impl FileReader {
    // Path-based filters (returns typed paths)
    pub fn filter_paths(&self, pattern: &str) -> Result<Vec<FsPath>, FsError>;  // Both files and dirs
    pub fn filter_file_paths(&self, pattern: &str) -> Result<Vec<FilePath>, FsError>;  // Files only
    pub fn filter_dir_paths(&self, pattern: &str) -> Result<Vec<DirPath>, FsError>;  // Directories only

    // Entry-based filters (returns typed paths)
    pub fn filter_entries(&self, pattern: &str) -> Result<Vec<FsEntry>, FsError>;  // Both files and dirs
    pub fn filter_file_entries(&self, pattern: &str) -> Result<Vec<FsFile>, FsError>;  // Files only
    pub fn filter_dir_entries(&self, pattern: &str) -> Result<Vec<FsDir>, FsError>;  // Directories only

    // Single-item metadata (unified File or Dir)
    pub fn metadata(&self, path: &Path) -> Result<FsMetadata, FsError>;
}
```

`FsPath` provides type-safe path representation:
```rust
pub enum FsPath {
    File(FilePath),
    Dir(DirPath),
}

impl FsPath {
    pub fn is_file(&self) -> bool;
    pub fn is_dir(&self) -> bool;
    pub fn as_file(&self) -> Option<&FilePath>;
    pub fn as_dir(&self) -> Option<&DirPath>;
    pub fn as_relative(&self, base: &Path) -> Result<RelativePath, ReadError>;  // **REVISED 2026-05-13**: Returns ReadError (ADR 017)
}

// **REVISION 2026-05-12**: as_relative() now requires base path parameter.
// Returns Result to handle cases where path is outside base directory.
// Storage layer calls this during FileView/DirView creation to normalize paths.
```

**Migration approach:**
- Phase 2: Add new methods (filter_entries, filter_file_entries, filter_dir_entries, filter_paths, filter_file_paths, filter_dir_paths, metadata_typed) alongside existing methods
- Phase 4: Delete old methods (entries(), info()), then rename *typed methods to remove "typed" suffix

`FsMetadata` provides type-level distinction between files and directories:
```rust
pub enum FsMetadata {
    File(FileMetadata),  // For regular files
    Dir(DirMetadata),     // For directories
}

impl FsMetadata {
    pub fn is_file(&self) -> bool;
    pub fn is_dir(&self) -> bool;
    pub fn as_file(&self) -> Option<&FileMetadata>;
    pub fn as_dir(&self) -> Option<&DirMetadata>;
}
```

`FsEntry` provides a unified representation for both files and directories:
```rust
pub enum FsEntry {
    File(FsFile),  // File with path and metadata
    Dir(FsDir),    // Directory with path and metadata
}

impl FsEntry {
    pub fn is_file(&self) -> bool;
    pub fn is_dir(&self) -> bool;
    pub fn as_file(&self) -> Option<&FsFile>;
    pub fn as_dir(&self) -> Option<&FsDir>;
    pub fn path(&self) -> &FsPath;
}
```

`FsEntryView` (vault module) provides persisted storage for both files and directories:
```rust
pub enum FsEntryView {
    File(FileView),  // Persisted file with ID, parent, format, hash
    Dir(DirView),    // Persisted directory with ID and parent
}

impl FsEntryView {
    pub fn id_bytes(&self) -> &[u8; 16];       // UUID bytes for the entry
    pub fn parent_id(&self) -> Option<DirId>;  // Parent directory ID (None for root)
    pub fn name(&self) -> &str;                // Filename or dirname (no path)
    pub fn is_file(&self) -> bool;
    pub fn is_dir(&self) -> bool;
}
```

**New:**
```rust
pub fn metadata(&self, path: &Path) -> Result<FsMetadata, FsError>;  // Unified File or Dir (ADR 017)
// Delete info() method entirely
```
- Processor: File discovery pipeline (FileReader → FileView/DirView → save)

**Deleted types (NO ALIASES - direct replacement):**
- `vault::VaultPath` → Use `NormalizedPath` for storage keys, `RelativePath` for validation
- `vault::VaultFile` → Replace with `FileView`
- `vault::VaultFolder` → Replace with `DirView`
- `fs::FileEntry` → Replace with `FsEntry` (unified file/dir enum)
- `fs::FileInfo` → Replace with `FileMetadata` (no alias)
- `fs::FormatKind` → Replace with `FileFormat` (no alias)
- `fs/file.rs` → Delete (split into name.rs, metadata.rs, entry.rs)
- `fs/types.rs` → Delete (contents moved to format.rs)

### Type Hierarchy

```rust
// fs/path.rs
pub struct AbsolutePath(PathBuf);           // Validated absolute path
pub struct RelativePath(PathBuf);           // Validated relative path (no .., no absolute)
pub struct FilePath(PathBuf);               // **REVISED**: Wraps PathBuf (absolute or relative)
pub struct DirPath(PathBuf);                // **REVISED**: Wraps PathBuf (absolute or relative)
pub enum FsPath { File(FilePath), Dir(DirPath) }  // Unified path enum

// **REVISION 2026-05-12**: FilePath/DirPath no longer constrained to RelativePath.
// They wrap PathBuf directly and can represent absolute or relative paths.
// Conversion to vault-relative happens explicitly via `as_relative(base)` method.
// Rationale: DirScanner produces absolute paths from walkdir; conversion to
// relative is deferred to storage layer (FileView/DirView) where vault root
// is available. This eliminates need for base path in DirScanner constructor.

// fs/path.rs - Zero-copy parent view
pub enum ParentDir<'a> { Root, Path(&'a Path) }

// fs/name.rs - Owned filename components
pub struct FileName(Box<str>);              // Owned filename (UTF-8)
pub struct DirName(Box<str>);               // Owned dirname (UTF-8)
pub struct BaseName(Box<str>);              // Owned basename (Obsidian terminology)

// fs/name.rs - Zero-copy borrowed views
pub struct FileNameRef<'a>(&'a OsStr);      // Borrowed filename view
pub struct DirNameRef<'a>(&'a OsStr);       // Borrowed dirname view
pub struct BaseNameRef<'a>(&'a OsStr);      // Borrowed basename view (file stem)

// fs/format.rs - Format detection and borrowed extension view
pub enum FileFormat {
    Json, Toml, Yaml, Markdown,
    Image,  // png, jpg, jpeg, gif, webp, svg, bmp, ico
    Pdf,    // pdf - binary/compiled format
    Document, // doc, docx, odt, rtf, txt - text-based documents
    Archive,  // zip, tar, gz, rar, 7z, wasm
    Binary,   // fallback for other binary formats
    Unknown,  // no extension or unrecognized
}
pub struct FileExtensionRef<'a>(&'a OsStr); // Borrowed extension view

// fs/metadata.rs - Timestamp and metadata types
pub struct FsTimes {
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
}

impl FsTimes {
    pub const fn created_at(&self) -> Option<SystemTime>;
    pub const fn modified_at(&self) -> Option<SystemTime>;
    pub fn is_match(&self, created_at: Option<SystemTime>, modified_at: Option<SystemTime>) -> bool;
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

pub enum FsMetadata {
    File(FileMetadata),
    Dir(DirMetadata),
}

// fs/entry.rs - Runtime entities
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

**Phase 1: Create new types and files in fs/ (no breaking changes)**
- Create fs/name.rs: FileName, DirName, BaseName + FileNameRef, DirNameRef, BaseNameRef<'a>
- Create fs/metadata.rs: FsTimes, FileMetadata, DirMetadata, FsMetadata
- Create fs/entry.rs: FsFile, FsDir, FsEntry (unified enum)
- Create fs/format.rs: FileFormat (public, expanded from FormatKind), FileExtensionRef<'a>
- Add FilePath(RelativePath), DirPath(RelativePath) to fs/path.rs
- Add ParentDir to fs/path.rs
- Keep old fs/file.rs and fs/types.rs temporarily for backward compat

**Phase 2: Add new methods to DirScanner and FileReader**
- **REVISED**: Add DirScanner.paths_typed() returning Vec<FsPath> (keeps existing paths() during transition)
- **REVISED**: Add DirScanner.entries_typed() returning Vec<FsEntry> (keeps existing entries() during transition)
- Add FileReader methods: filter_entries_typed, filter_file_entries_typed, filter_dir_entries_typed, filter_paths_typed, filter_file_paths_typed, filter_dir_paths_typed
- Add FileReader.metadata_typed(path: &Path) -> Result<FsMetadata, ParseError>
- Keep old methods (paths() returning Vec<PathBuf>, entries() returning Vec<FileEntry>, info()) for backward compat during transition

**REVISION 2026-05-12**: Use *_typed() suffix for new methods to avoid breaking existing call sites.
This allows gradual migration in Phase 3 before removing old methods in Phase 4.

**Phase 2a: Redesign fs/error.rs (Issue 08 — prerequisite for Phase 3)**
- Implement ADR 017: decompose into FsError, ReadError, ScanError, ParseError, PathError
- Update path.rs, name.rs constructors to return PathError
- Update scanner.rs, entry.rs to return ScanError
- Update reader.rs to return FsError
- Narrow ParseError to deserialization-only (remove Io, NotInBasePath variants)
- Delete DirEntryError
- Update consumer From impls in schema/, config/, note/
- Run `mise run verify` to confirm

**Phase 3: Update all consumers (split into subphases)**

**Phase 3a: Update FileInfo → FileMetadata**
- Straightforward rename across all contexts
- Update schema/, config/, and any other consumers
- Run `mise run verify` to confirm

**Phase 3b: Update FormatKind → FileFormat**
- Make FileFormat public (was pub(crate))
- Add new format variants (Image, Document, Archive)
- Update all FormatKind usages to FileFormat

**Phase 3c: Update FileEntry → FsEntry**
- More complex: FsEntry is a unified enum (File or Dir), FileEntry was file-only
- Update DirScanner.entries() to return Vec<FsEntry> (replace old method)
- Update all consumers to handle FsEntry::File vs FsEntry::Dir variants
- Update FileReader.list_entries() return type

**Phase 4: Delete old files, methods, and rename new methods (breaking change)**
- ONLY after all Phase 3 subphases complete
- Delete fs/file.rs (contents moved to name.rs, metadata.rs, entry.rs)
- Delete fs/types.rs (contents moved to format.rs)
- Delete old DirScanner.paths() returning Vec<PathBuf>
- Delete old DirScanner.entries() returning Vec<FileEntry>
- Delete old FileReader.info() method
- **REVISED**: Rename all *_typed() methods to remove suffix:
  - DirScanner.paths_typed() → paths()
  - DirScanner.entries_typed() → entries()
  - FileReader.filter_entries_typed() → filter_entries() (and all filter variants)
  - FileReader.metadata_typed() → metadata()
- Update all remaining consumers
- Run `mise run verify` to confirm no broken imports

**Phase 5: Vault module refactoring**
- Implement FileId, DirId, FileView, DirView, FsEntryView, NormalizedPath
- Create new storage tables (FILES_BY_ID, DIRS_BY_ID, etc.)
- Implement Repository trait with new signatures (including get_entry)
- Implement RedbRepository adapter
- Add multimap indexes incrementally
- Refactor vault processor pipeline to use new types
- Update save logic to populate all indexes
- Delete VaultPath, VaultFile, VaultFolder
- Remove old VAULT_FILES_BY_PATH, VAULT_FOLDERS_BY_PATH tables
- Verify existing tests pass

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

**fs/name.rs:**
- FileName/DirName/BaseName owned component creation
- FileNameRef/DirNameRef/BaseNameRef borrowed view extraction
- Conversions between owned and borrowed types

**fs/metadata.rs:**
- FsTimes timestamp extraction from std::fs::Metadata
- FileMetadata/DirMetadata construction
- FsMetadata enum variants and helper methods

**fs/entry.rs:**
- FsFile/FsDir from DirEntry conversion
- FsEntry enum variants and helper methods
- Path access via FsPath

**fs/format.rs:**
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
- Existing: `fs/file.rs` tests for FileInfo from std::fs::Metadata (to be replaced)
- Pattern: tempfile-based tests creating real files
- Extend with FileMetadata/DirMetadata in fs/metadata.rs

**Storage tests:**
- Existing: `schema/storage.rs` tests for RedbRepository
- Pattern: tempfile DB, save/load round-trip, index consistency
- Mirror for vault/storage.rs with FileView/DirView/FsEntryView

**Processor tests:**
- Existing: `vault/processor.rs` has typestate transition tests
- Pattern: Mock FileReader, verify state machine transitions
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
