---
title: 08-fs-error-type-redesign
category: enhancement
label: ready-for-agent
status: pending
date_created: 2026-05-13
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Redesign `fs/error.rs` into six focused error types before Phase 3 consumer migration begins. Implements [ADR 017](../../docs/adr/017-fs-error-type-hierarchy.md).

### New types to create

**`PathError`** (11 variants) — path construction and name extraction:
- `Empty` — path string is empty
- `NotAFile(PathBuf)` — expected a file, path does not refer to one
- `NotADirectory(PathBuf)` — expected a directory, path does not refer to one
- `NotRelative(PathBuf)` — expected relative, got absolute
- `NotAbsolute(PathBuf)` — expected absolute, got relative
- `ParentTraversal(PathBuf)` — contains `..` component
- `CurrentDirComponent(PathBuf)` — contains `.` component
- `PlatformPrefix(PathBuf)` — contains platform-specific prefix (e.g. `C:`)
- `InvalidUtf8(PathBuf)` — path is not valid UTF-8
- `NoFileName(PathBuf)` — path has no filename component
- `NoStem(PathBuf)` — path has no stem (basename without extension)

**`ReadError`** (2 variants) — file input access:
- `Io { path: PathBuf, source: std::io::Error }` — file read or metadata access failed
- `NotInBase { path: PathBuf, base: PathBuf }` — path is outside vault root boundary

**`ScanError`** (4 variants) — directory traversal:
- `Traversal { path: PathBuf, source: std::io::Error }` — walkdir entry or metadata read failed
- `InvalidPattern { pattern: Box<str>, source: Box<str> }` — glob pattern is syntactically invalid
- `UnsupportedEntryType(PathBuf)` — filesystem entry is neither file nor directory
- `Path(#[from] PathError)` — path construction failed during scan

**`ParseError`** (4 variants, narrowed from 6) — structured deserialization only:
- `Json { path, message, line, column }`
- `Toml { path, message, line, column }`
- `Yaml { path, message, line, column }`
- `UnsupportedFormat { path, supported }`

**`FsError`** (5 compositor variants) — module-level public error:
- `Read(#[from] ReadError)`
- `Scan(#[from] ScanError)`
- `Parse(#[from] ParseError)`
- `Path(#[from] PathError)`
- `Validation(#[from] PathValidationError)`

**`PathValidationError`** — unchanged (9 variants).

### Types to delete

- `DirEntryError` — absorbed into `ScanError::Traversal` and `PathError::InvalidUtf8`

### Module return type changes

| Module | Method | Old return | New return |
|---|---|---|---|
| path.rs | `RelativePath/AbsolutePath::try_from` | `std::io::Error` | `PathError` |
| path.rs | `FilePath::new`, `DirPath::new` | `std::io::Error` | `PathError` |
| path.rs | `as_relative(base)` | `ParseError` | `ReadError` |
| name.rs | `FileName/BaseName::try_from` | `std::io::Error` | `PathError` |
| entry.rs | `FsEntry::try_from(walkdir::DirEntry)` | `ParseError` | `ScanError` |
| scanner.rs | `paths`, `entries`, `*_typed` | `ParseError` | `ScanError` |
| reader.rs | `read_to_string`, `metadata`, etc. | `ParseError` | `FsError` |
| reader.rs | `filter_*`, `list_*` | `ParseError` | `FsError` |
| reader.rs | `parse_structured` | `ParseError` | `FsError` |
| types.rs | `Json/Toml/Yaml::parse` | `ParseError` | `ParseError` |

### Consumer `From` impl migration

Existing `From<ParseError>` impls in consumer contexts need splitting:

- **schema/error.rs**: `From<ParseError>` (4 variants) + `From<ReadError>` (2 variants) for `SchemaIngestionError`
- **config/error.rs**: `From<ParseError>` (4 variants) + `From<ReadError>` (2 variants) for `ConfigIngestError`
- **note/error.rs**: `From<ReadError>` for `NoteIngestError` (replaces the dummy-path hack)

## Acceptance criteria

- [ ] `PathError` defined with 11 self-documenting variants, all `#[non_exhaustive]`
- [ ] `ReadError` defined with `Io` and `NotInBase` variants
- [ ] `ScanError` defined with 4 variants, composing `PathError` via `#[from]`
- [ ] `ParseError` narrowed to 4 deserialization-only variants (remove `Io`, `NotInBasePath`)
- [ ] `FsError` defined as pure compositor (5 `#[from]` variants, zero direct)
- [ ] `DirEntryError` deleted
- [ ] `path.rs` constructors return `PathError` instead of `std::io::Error`
- [ ] `name.rs` `TryFrom` impls return `PathError` instead of `std::io::Error`
- [ ] `path.rs` `as_relative()` returns `ReadError` instead of `ParseError`
- [ ] `entry.rs` `FsEntry::try_from` returns `ScanError` instead of `ParseError`
- [ ] `scanner.rs` methods return `ScanError` instead of `ParseError`
- [ ] `reader.rs` methods return `FsError` (wrapping child errors via `?`)
- [ ] `types.rs` parsers return narrowed `ParseError`
- [ ] Consumer `From` impls updated in schema/, config/, note/
- [ ] `mod.rs` re-exports updated
- [ ] All existing tests adapted to new error types
- [ ] Run `mise run verify` — no compile errors, all tests pass

## Blocked by

- 07-fsreader-methods

## Blocks

- 09-consumer-fileinfo-to-metadata
- 10-consumer-formatkind-to-fileformat
- 11-consumer-fileentry-to-fsentry
- 12-phase-4-cleanup
