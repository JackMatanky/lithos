# Vault + Note Processor Plan (Detailed)

## Scope

Introduce a vault module at `lithos-core/src/vault/` and implement two
typestate processors:

- `vault::processor::VaultProcessor` for scanning, comparison, routing, and
  pruning across the vault (files + folders).
- `note::processor::NoteProcessor` for per-markdown processing.

Replace `application::vault::Service` with the vault processor as the
entrypoint. Keep the note pipeline localized to the note module. Folder
tracking is required from day one.

## Non-Negotiable Constraints

- Vault module lives at `lithos-core/src/vault/`.
- Separate vault tables from note tables.
- No `*View` types in initial vault or note pipelines.
- No `rename_*` repository methods.
- Hashing is excluded initially; metadata mismatch leads to full reparse.
- Vault processor is a typestate pipeline.
- Note processor is a typestate pipeline.
- Deletions are handled in a distinct `Prune` stage.
- `Prune` runs only on full scans; skipped for partial scans.
- Folder tracking from day one.
- Structured outcome reports for vault and note processing.

## Vault Module Layout

Create `lithos-core/src/vault/` with the following files:

- `mod.rs`
  - Re-exports, table definitions, public API for vault module.
- `model.rs`
  - `VaultPath`, `VaultFile`, `VaultFolder`, `VaultEntry` types.
- `ext.rs`
  - `MarkdownExt` marker/type for extension-based routing.
- `storage.rs`
  - `vault::Repository` trait, redb adapter, batch adapters.
- `processor.rs`
  - `VaultProcessor<Stage, Status>`, `ScanMode`, `VaultProcessReport`.
- `fs.rs` (optional)
  - Thin wrapper around `FileReader` for scanning and metadata extraction.

## Vault Domain Types

### `VaultPath`

- Newtype validated to be vault-relative, no traversal, no absolute paths.
- Follows existing path validation patterns (`note::paths` and `fs` rules).
- `VaultPath::try_new(&str) -> Result<VaultPath, VaultPathError>`.
- Provides `as_str()` and `as_path()` accessors.

### `VaultFile`

Required fields (all owned):

- `path: VaultPath`
- `basename: Box<str>`
- `filename: Box<str>`
- `parent: Box<str>`
- `extension: Option<Box<str>>`
- `size: u64`
- `created_at: Option<SystemTime>`
- `modified_at: Option<SystemTime>`

Construction:

- `VaultFile::try_from_fs(path: &VaultPath, metadata: &Metadata)`
- Derived components (`basename`, `filename`, `parent`, `extension`) are
  computed from path using `Path` utilities.

### `VaultFolder`

Required fields:

- `path: VaultPath`
- `basename: Box<str>`
- `parent: Box<str>`
- `created_at: Option<SystemTime>`
- `modified_at: Option<SystemTime>`

Construction similar to `VaultFile`.

### `MarkdownExt`

- Marker for extensions: `md` and `markdown` (case-insensitive).
- `MarkdownExt::is_supported(path: &VaultPath) -> bool`.

## Vault Repository

### Tables

- `VAULT_FILES_BY_PATH`: key `&str`, value `VaultFile` (rkyv serialized).
- `VAULT_FOLDERS_BY_PATH`: key `&str`, value `VaultFolder` (rkyv serialized).

### Repository Trait (no rename methods)

Read methods:

- `get_file(&self, path: &VaultPath) -> Result<Option<VaultFile>, Error>`
- `list_files(&self) -> Result<Vec<VaultFile>, Error>`
- `get_folder(&self, path: &VaultPath) -> Result<Option<VaultFolder>, Error>`
- `list_folders(&self) -> Result<Vec<VaultFolder>, Error>`

Write methods:

- `save_file(&self, file: &VaultFile) -> Result<(), Error>`
- `delete_file(&self, path: &VaultPath) -> Result<(), Error>`
- `save_folder(&self, folder: &VaultFolder) -> Result<(), Error>`
- `delete_folder(&self, path: &VaultPath) -> Result<(), Error>`

Batch adapters (mirroring note storage):

- `with_batch_read` and `with_batch_write`.

### Repository Error Type

- `VaultRepositoryError` with variants:
  - `Storage(DbError)`
  - `ConstraintViolation { message: Box<str> }`
- `Error: From<VaultRepositoryError> + std::error::Error`.

## Vault Processor Typestate

### Public API

- `VaultProcessor<Discovery, Unknown>::new(source: FileReader, repo: &R)`
- `process_full(&self) -> Result<VaultProcessReport, VaultProcessError>`
- `process_partial(paths: &[VaultPath]) -> Result<VaultProcessReport, VaultProcessError>`

### Stage Types

- `Discovery`
- `Comparison`
- `Routing`
- `Prune`
- `Completed`

### Status Types

- `Unknown` (entry)
- `Scanned` (validated path + metadata captured)
- `Present` (repo entry exists)
- `Missing` (repo entry missing)
- `Fresh` (metadata match)
- `StaleMetadata` (size/mtime mismatch)
- `Routed` (markdown routed)
- `Ready` (final report)

### Scan Modes

- `ScanMode::Full`: builds a complete path set, must run `Prune`.
- `ScanMode::Partial`: only processes given paths, skips `Prune`.

### Core Invariants

- `Discovery` establishes a validated path set.
- `Comparison` operates on `Scanned` entries only.
- `Routing` allowed only after `Comparison` results.
- `Prune` allowed only if `ScanMode::Full` and path set is complete.
- `Completed` only after routing + (if full) pruning.

### Processor Flow

1. `Discovery`
   - Use `FileReader` to list files and folders.
   - Validate paths and normalize to `VaultPath`.
   - Build `VaultFile` and `VaultFolder` records.
   - Build `HashSet<VaultPath>` for pruning.

2. `Comparison`
   - For each scanned file:
     - `get_file` from repo.
     - If none -> `Missing` and `save_file`.
     - If present and metadata matches -> `Fresh`.
     - If present and metadata differs -> `StaleMetadata` and `save_file`.
   - For each scanned folder:
     - Same logic with folders.

3. `Routing`
   - For `VaultFile` entries where `MarkdownExt::is_supported`:
     - If `Missing` or `StaleMetadata` -> invoke note processor.
     - If `Fresh` -> skip note pipeline.
   - For non-markdown files -> no routing.

4. `Prune` (full scans only)
   - Compare repo file paths against `HashSet<VaultPath>`.
   - Delete missing files from vault repo.
   - If missing file is markdown, invoke note deletion handler.
   - Repeat for folders.

5. `Completed`
   - Emit structured `VaultProcessReport`.

### Structured Outcome

`VaultProcessReport` fields:

- `files_scanned`
- `files_added`
- `files_updated`
- `files_fresh`
- `files_deleted`
- `folders_scanned`
- `folders_added`
- `folders_updated`
- `folders_deleted`
- `markdown_routed`
- `notes_created_or_updated`
- `notes_deleted`
- `errors: Vec<VaultProcessErrorSummary>` (optional)

### Error Types

- `VaultProcessError`:
  - `File(VaultFileError)`
  - `Repository(VaultRepositoryError)`
  - `Note(NoteProcessError)`
- `VaultFileError`:
  - `InvalidPath` (bad UTF-8 or traversal)
  - `MetadataFailed`
  - `ReadFailed`

## Note Processor Typestate

### Public API

- `NoteProcessor<Discovery, Unknown>::new(repo: &R, config: &Config)`
- `process_file(&self, file: &VaultFile) -> Result<NoteProcessReport, NoteProcessError>`
- `record_deleted(path: &NotePath) -> Result<NoteProcessReport, NoteProcessError>`

### Stage Types

- `Discovery`
- `Comparison`
- `Analysis`
- `Construction`
- `Completed`

### Status Types

- `Unknown`
- `Missing`
- `Present`
- `Suspect` (metadata mismatch + content loaded)
- `Fresh`
- `New` (parsed raw note)
- `Changed` (parsed raw note)
- `Ready`

### Core Invariants

- `Present` guarantees stored note exists (metadata available).
- `Suspect` guarantees markdown content is loaded.
- `New`/`Changed` guarantee `RawNote` ready for conversion.

### Processor Flow

1. `Discovery`
   - `find_by_path` in note repository.
   - If none -> `Missing`.
   - If some -> `Present`.

2. `Comparison`
   - Compare stored note `source_bytes` and `modified_at` with `VaultFile`.
   - If match -> `Fresh` -> `Completed` with `Unchanged`.
   - If mismatch -> `Suspect`.

3. `Analysis`
   - Read markdown content using `FileReader` (vault root + file path).
   - Parse via `MarkdownParser` to `RawNote`.
   - Transition to `New` or `Changed`.

4. `Construction`
   - Normalize into `Note` aggregate (`Note::try_from`).
   - Persist via note repository (`save`).

5. `Completed`
   - Emit `NoteProcessReport`.

### Note Outcome

`NoteProcessReport` fields:

- `note_id: NoteId`
- `path: NotePath`
- `action: Created | Updated | Unchanged | Deleted`
- `reason: Fresh | MetadataChanged | Missing`
- `errors: Vec<NoteProcessErrorSummary>` (optional)

### Note Errors

- `NoteProcessError`:
  - `Ingest(NoteIngestError)`
  - `Load(NoteLoadError)`
  - `Repository(NoteRepositoryError)`
  - `File(NoteFileError)`

## Integration Plan

### Entry Point Changes

- Replace `application::vault::Service::load` with a vault processor entry.
- `lithos-cli` and any tests call the new vault processor.
- Keep `application::vault` module only if required for API compatibility.

### Configuration

- `VaultProcessor` uses `Config` for vault root and parsing specs.
- `NoteProcessor` uses `Config` for frontmatter/task specs.
- Use `FileReader::new(config.vault_metadata().root().as_path())`.

### Path Mapping

- `VaultPath` to `NotePath` conversion for markdown routing.
- Validate that `VaultPath` satisfies `NotePath` invariants.

### Deletion Handling

- `Prune` identifies missing paths and deletes vault repo entries.
- For markdown paths:
  - Call note processor `record_deleted` (or loader deletion) to remove note.
- For folders:
  - Delete folder entries only; note deletions handled per-file.

## Test Plan

### Unit Tests

- `VaultPath` validation rules (relative, no traversal, UTF-8).
- `VaultFile` and `VaultFolder` construction from `Path` + `Metadata`.
- `MarkdownExt` detection (case-insensitive extensions).
- Vault repository adapters: save/get/list/delete for files and folders.
- Note processor transitions (fresh, stale, missing).

### Integration Tests

- Full scan with markdown and non-markdown files:
  - Expect vault repo entries for all files/folders.
  - Expect note entries only for markdown files.
- Delete a markdown file and run full scan:
  - Expect vault repo entry removed.
  - Expect note entry removed.

### Regression Tests

- Ensure no `rename_*` methods introduced.
- Ensure no `*View` types added.
- Ensure note processor only called from vault routing.

## Sequenced Implementation Tasks

1. Add `lithos-core/src/vault/` module skeleton and `mod.rs` exports.
2. Implement `VaultPath`, `VaultFile`, `VaultFolder`, `MarkdownExt`.
3. Add vault tables and repository trait + redb adapter.
4. Implement `VaultProcessor` typestate and reports.
5. Add `note::processor` typestate and reports.
6. Wire vault processor into application entrypoints/tests.
7. Remove or deprecate `application::vault::Service` usage.
8. Add tests and update existing ones to use new entrypoint.

## Notes

- No content hash in initial design; metadata mismatch triggers parse.
- Folder tracking must be included from day one.
- Avoid cross-context imports; vault is infrastructure, note remains isolated.
