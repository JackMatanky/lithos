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

---

## Agent Brief

> *This was generated by AI during triage.*

### Summary

Decompose `fs/error.rs` from 3 catch-all types into 6 focused error types with single responsibilities. Implements [ADR 017](../../docs/adr/017-fs-error-type-hierarchy.md). This refactor enables clean consumer migration in Phase 3+ by eliminating the `ParseError::Io` junk drawer and providing semantically clear error types.

### Impact Analysis

**GitNexus Analysis:**
- `ParseError` has **LOW risk** (0 direct callers in knowledge graph)
- However, grep shows **184 usage sites** across fs/, schema/, config/, note/ contexts
- 3 consumer `From<ParseError>` impls require migration (schema, config, note)

**Current Error Usage Map:**

| Error Type       | Variants | Usage Contexts | Role |
|------------------|----------|----------------|------|
| `ParseError`     | 6        | fs/, schema/, config/, note/ | Catch-all (I/O + parsing + path) |
| `PathValidationError` | 9 | fs/validator.rs | Security validation |
| `DirEntryError`  | 2        | fs/entry.rs | Legacy conversions |

**Post-Refactor Error Map:**

| Error Type       | Variants | Usage Contexts | Role |
|------------------|----------|----------------|------|
| `PathError`      | 11       | path.rs, name.rs, scanner.rs | Path construction failures |
| `ReadError`      | 2        | reader.rs | File input access |
| `ScanError`      | 4        | scanner.rs, entry.rs | Directory traversal |
| `ParseError`     | 4        | types.rs, reader.rs | Structured deserialization only |
| `FsError`        | 5 (compositor) | reader.rs (public API) | Module-level compositor |
| `PathValidationError` | 9 | validator.rs | Security validation (unchanged) |

### Critical Path

**Phase 1: Create new error types** (no breaking changes yet)
1. Define `PathError` with 11 variants
2. Define `ReadError` with 2 variants
3. Define `ScanError` with 4 variants (composing `PathError`)
4. Define `FsError` compositor with 5 `#[from]` variants
5. Narrow `ParseError` to 4 deserialization-only variants

**Phase 2: Migrate fs/ module return types** (breaking changes within fs/)
1. `path.rs`: Change constructors from `std::io::Error` → `PathError`
2. `name.rs`: Change `TryFrom` impls from `std::io::Error` → `PathError`
3. `path.rs`: Change `as_relative()` from `ParseError` → `ReadError`
4. `entry.rs`: Change `FsEntry::try_from` from `ParseError` → `ScanError`
5. `scanner.rs`: Change all methods from `ParseError` → `ScanError`
6. `reader.rs`: Change all methods from `ParseError` → `FsError`
7. `types.rs`: Keep `ParseError` (already deserialization-only)
8. Delete `DirEntryError`

**Phase 3: Migrate consumer `From` impls** (breaking changes in schema/, config/, note/)
1. **schema/error.rs**: Split `From<ParseError>` (6 variants) → `From<ParseError>` (4 variants) + `From<ReadError>` (2 variants)
2. **config/error.rs**: Split `From<ParseError>` (6 variants) → `From<ParseError>` (4 variants) + `From<ReadError>` (2 variants)
3. **note/error.rs**: Replace dummy-path hack with `From<ReadError>` (2 variants)

**Phase 4: Update tests and verify**
1. Adapt all existing error tests to new types
2. Run `mise run verify` (fmt + lint + tests + adr:validate)

### Technical Constraints

**Rust Best Practices (Apollo Handbook):**
- **Chapter 4 (Error Handling)**: Use `thiserror` with `#[from]` for auto-conversions; `#[non_exhaustive]` for forward compatibility; `#[error(transparent)]` for compositor variants
- **Chapter 3 (Performance)**: Error types are on cold paths; prioritize clarity over size
- **Chapter 5 (Testing)**: One assertion per test; behavior-focused names; test error Display messages

**Clippy Requirements:**
- `#[non_exhaustive]` on all enums (future-proofing)
- `#[expect(clippy::module_name_repetitions)]` with reason for `ParseError`, `PathError`, etc.
- No `unwrap()`/`expect()` in production code (tests only)

**ADR 017 Decisions:**
- `PathError` has 11 self-documenting variants (no `std::io::Error` strings)
- `ScanError::Traversal` preserves original `io::Error` source (not erased)
- `FsError` is pure compositor (zero direct variants)
- `ParseError` narrowed to 4 deserialization-only variants

### Test-Driven Development Plan

**TDD Workflow (Vertical Slices):**

Each error type follows RED-GREEN-REFACTOR cycles:

1. **`PathError` tracer bullet:**
   - RED: Test `PathError::Empty` Display message
   - GREEN: Define `PathError` enum with `Empty` variant
   - RED: Test `PathError::NotAFile` Display message
   - GREEN: Add `NotAFile(PathBuf)` variant
   - Repeat for all 11 variants

2. **`ReadError` tracer bullet:**
   - RED: Test `ReadError::Io` Display message with path context
   - GREEN: Define `ReadError` enum with `Io` variant
   - RED: Test `ReadError::NotInBase` Display message
   - GREEN: Add `NotInBase` variant

3. **`ScanError` tracer bullet:**
   - RED: Test `ScanError::Traversal` preserves original `io::Error`
   - GREEN: Define `ScanError` with `Traversal` variant
   - RED: Test `ScanError::Path(#[from] PathError)` composition
   - GREEN: Add `Path` variant with `#[from]`
   - Repeat for `InvalidPattern`, `UnsupportedEntryType`

4. **`FsError` compositor:**
   - RED: Test `FsError::Read(#[from] ReadError)` auto-conversion
   - GREEN: Define `FsError` with `Read` variant
   - Repeat for `Scan`, `Parse`, `Path`, `Validation` variants

5. **Narrow `ParseError`:**
   - RED: Test that `ParseError` no longer has `Io` or `NotInBasePath` variants (compile error)
   - GREEN: Remove those variants, keep `Json`, `Toml`, `Yaml`, `UnsupportedFormat`

6. **Migrate fs/ modules:**
   - For each module (path.rs, name.rs, entry.rs, scanner.rs, reader.rs):
     - RED: Change return type, observe compile errors
     - GREEN: Fix error construction sites to use new types
     - Verify: Run `cargo test --lib fs::<module>` after each

7. **Migrate consumer `From` impls:**
   - For each consumer (schema, config, note):
     - RED: Update `From<ParseError>` to handle only 4 variants, observe compile errors
     - GREEN: Add `From<ReadError>` impl for `Io` and `NotInBase` variants
     - Verify: Run `cargo test --lib <context>::error` after each

8. **Delete `DirEntryError`:**
   - RED: Remove `DirEntryError`, observe compile errors in entry.rs
   - GREEN: Replace with `ScanError::Traversal` and `PathError::InvalidUtf8`
   - Verify: Run `cargo test --lib fs::entry`

**Test Organization:**
- `error.rs` tests organized into submodules: `path_error`, `read_error`, `scan_error`, `parse_error`, `fs_error`
- Each submodule tests Display messages (format assertions)
- Test auto-conversions via `#[from]` (e.g., `PathError` → `ScanError`, `ReadError` → `FsError`)
- Test that error sources are preserved (not erased)

**Behavioral Focus (TDD Philosophy):**
- Tests verify public interfaces (Display messages, conversions)
- Tests survive internal refactors (no testing implementation details)
- One assertion per test where possible
- Test names describe user-facing behavior: `formats_io_error_with_path_context`, `preserves_original_source_error`

### Implementation Checklist

**Phase 1: Create New Error Types**
- [ ] Define `PathError` enum with 11 variants, `#[non_exhaustive]`, Display messages tested
- [ ] Define `ReadError` enum with 2 variants, `#[non_exhaustive]`, Display messages tested
- [ ] Define `ScanError` enum with 4 variants, `#[non_exhaustive]`, test `#[from]` composition
- [ ] Define `FsError` compositor with 5 `#[from]` variants, test auto-conversions
- [ ] Narrow `ParseError` to 4 variants (remove `Io`, `NotInBasePath`), update existing tests

**Phase 2: Migrate fs/ Return Types**
- [ ] `path.rs`: `RelativePath/AbsolutePath::try_from` returns `PathError` (was `std::io::Error`)
- [ ] `path.rs`: `FilePath::new`, `DirPath::new` return `PathError` (was `std::io::Error`)
- [ ] `path.rs`: `as_relative(base)` returns `ReadError` (was `ParseError`)
- [ ] `name.rs`: `FileName/BaseName::try_from` returns `PathError` (was `std::io::Error`)
- [ ] `entry.rs`: `FsEntry::try_from(walkdir::DirEntry)` returns `ScanError` (was `ParseError`)
- [ ] `scanner.rs`: All methods return `ScanError` (was `ParseError`)
- [ ] `reader.rs`: All methods return `FsError` (was `ParseError`)
- [ ] `types.rs`: Parsers return narrowed `ParseError` (4 variants)
- [ ] Delete `DirEntryError` enum

**Phase 3: Migrate Consumer `From` Impls**
- [ ] `schema/error.rs`: Split `From<ParseError>` → `From<ParseError>` (4 variants) + `From<ReadError>` (2 variants)
- [ ] `config/error.rs`: Split `From<ParseError>` → `From<ParseError>` (4 variants) + `From<ReadError>` (2 variants)
- [ ] `note/error.rs`: Replace dummy-path hack with `From<ReadError>` (2 variants)

**Phase 4: Finalize**
- [ ] Update `mod.rs` re-exports (add `PathError`, `ReadError`, `ScanError`, `FsError`)
- [ ] Run `mise run verify` — all tests pass, no Clippy warnings, ADRs valid
- [ ] Update issue 08-fs-error-redesign.md acceptance criteria

### Verification Commands

```bash
# Per-module verification (run after each module migration)
cargo test --lib fs::error
cargo test --lib fs::path
cargo test --lib fs::name
cargo test --lib fs::entry
cargo test --lib fs::scanner
cargo test --lib fs::reader
cargo test --lib schema::error
cargo test --lib config::error
cargo test --lib note::error

# Full verification (run at end)
mise run verify  # fmt + lint + tests + adr:validate
```

### Risks & Mitigations

**Risk 1: Over-decomposition**
- *Mitigation*: Each type maps to a single operational concern (path construction, file reads, scanning, parsing). ADR 017 validates this mapping.

**Risk 2: Consumer migration breaks downstream code**
- *Mitigation*: TDD with vertical slices ensures each consumer From impl is tested before moving to next. Compile errors guide migration.

**Risk 3: Missing error construction sites**
- *Mitigation*: Grep shows all 184 usage sites. Compiler will catch all required changes (no silent breakage).

**Risk 4: Test churn due to error type changes**
- *Mitigation*: Tests focus on behavior (Display messages, conversions), not structure. Test names describe capabilities, not implementation.

### Domain Context

**FS Context Invariants:**
- File operations remain constrained to validated vault roots
- Path validation is required before filesystem access
- File access contracts are deterministic and testable

**Error Semantics:**
- `PathError`: Path construction/name extraction failures (pre-I/O validation)
- `ReadError`: File input access failures (I/O + vault boundary)
- `ScanError`: Directory traversal failures (walkdir + path construction)
- `ParseError`: Structured deserialization only (post-read parsing)
- `FsError`: Public API compositor (wraps all child errors)

### Success Criteria

✅ All 6 error types defined with clear, non-overlapping responsibilities
✅ `ParseError` narrowed to 4 deserialization-only variants
✅ All fs/ module return types migrated
✅ All consumer `From` impls updated (schema, config, note)
✅ `DirEntryError` deleted
✅ All tests pass (`mise run verify`)
✅ No Clippy warnings
✅ Error sources preserved (not erased via `io::Error::other()`)

---

## TDD Implementation Plan

### Test Organization Structure

```
fs/error.rs tests:
├── path_error/
│   ├── display_messages/
│   │   ├── formats_empty_error
│   │   ├── formats_not_a_file_with_path
│   │   ├── formats_not_a_directory_with_path
│   │   ├── formats_not_relative_with_path
│   │   ├── formats_not_absolute_with_path
│   │   ├── formats_parent_traversal_with_path
│   │   ├── formats_current_dir_component_with_path
│   │   ├── formats_platform_prefix_with_path
│   │   ├── formats_invalid_utf8_with_path
│   │   ├── formats_no_filename_with_path
│   │   └── formats_no_stem_with_path
│   └── properties/
│       └── preserves_path_in_variants
├── read_error/
│   ├── display_messages/
│   │   ├── formats_io_error_with_path_context
│   │   └── formats_not_in_base_with_paths
│   └── conversions/
│       └── constructs_from_io_error_with_path
├── scan_error/
│   ├── display_messages/
│   │   ├── formats_traversal_with_path
│   │   ├── formats_invalid_pattern_with_pattern
│   │   ├── formats_unsupported_entry_type_with_path
│   │   └── formats_composed_path_error
│   ├── conversions/
│   │   └── converts_from_path_error_automatically
│   └── source_preservation/
│       └── preserves_original_io_error_in_traversal
├── parse_error/
│   ├── display_messages/
│   │   ├── formats_json_error_with_location
│   │   ├── formats_toml_error_with_location
│   │   ├── formats_yaml_error_with_location
│   │   └── formats_unsupported_format_with_supported_list
│   └── narrowing/
│       └── no_longer_has_io_or_not_in_base_variants  # Compile-time test
└── fs_error/
    ├── conversions/
    │   ├── converts_from_read_error_automatically
    │   ├── converts_from_scan_error_automatically
    │   ├── converts_from_parse_error_automatically
    │   ├── converts_from_path_error_automatically
    │   └── converts_from_validation_error_automatically
    └── composition/
        └── preserves_inner_error_source
```

### Vertical Slice Sequence

**Slice 1: PathError Foundation**
- RED: Write test for `PathError::Empty` Display message
- GREEN: Define `PathError` enum skeleton with `Empty` variant
- RED: Write test for `PathError::NotAFile(PathBuf)` Display message
- GREEN: Add `NotAFile` variant with Display impl
- Repeat for remaining 9 variants
- REFACTOR: Extract Display message formatting helpers if duplication emerges

**Slice 2: ReadError Foundation**
- RED: Test `ReadError::Io` Display includes path and source
- GREEN: Define `ReadError` with `Io` variant
- RED: Test `ReadError::NotInBase` Display includes both paths
- GREEN: Add `NotInBase` variant

**Slice 3: ScanError with Composition**
- RED: Test `ScanError::Traversal` preserves original `io::Error`
- GREEN: Define `ScanError` with `Traversal` variant
- RED: Test `ScanError::Path(#[from] PathError)` conversion
- GREEN: Add `Path` variant with `#[from]` attribute
- RED: Test `PathError` → `ScanError` automatic conversion via `?`
- GREEN: Verify conversion compiles
- Repeat for `InvalidPattern`, `UnsupportedEntryType`

**Slice 4: FsError Compositor**
- RED: Test `ReadError` → `FsError` automatic conversion
- GREEN: Define `FsError` with `Read(#[from] ReadError)` variant
- RED: Test `?` operator auto-converts in function returning `FsError`
- GREEN: Verify
- Repeat for `Scan`, `Parse`, `Path`, `Validation` variants

**Slice 5: Narrow ParseError**
- RED: Remove `Io` and `NotInBasePath` variants from `ParseError`
- Observe compile errors in existing tests
- GREEN: Update existing `parse_error` tests to only cover 4 variants
- VERIFY: `cargo test --lib fs::error::parse_error`

**Slice 6: Migrate path.rs**
- RED: Change `RelativePath::try_from` return type to `PathError`
- Observe compile errors at call sites
- GREEN: Update error construction from `io::Error::new(...)` to `PathError::NotRelative(...)`
- VERIFY: `cargo test --lib fs::path`
- Repeat for `AbsolutePath`, `FilePath`, `DirPath`
- RED: Change `as_relative()` return type to `ReadError`
- GREEN: Update error construction
- VERIFY: `cargo test --lib fs::path`

**Slice 7: Migrate name.rs**
- RED: Change `FileName::try_from` return type to `PathError`
- GREEN: Update error construction
- VERIFY: `cargo test --lib fs::name`
- Repeat for `BaseName`

**Slice 8: Migrate entry.rs**
- RED: Change `FsEntry::try_from(walkdir::DirEntry)` return type to `ScanError`
- GREEN: Update error construction (use `ScanError::Traversal`, `ScanError::Path`)
- VERIFY: `cargo test --lib fs::entry`
- RED: Delete `DirEntryError` usage
- GREEN: Replace with `ScanError::Traversal` and `PathError::InvalidUtf8`
- VERIFY: `cargo test --lib fs::entry`

**Slice 9: Migrate scanner.rs**
- RED: Change all method return types from `ParseError` to `ScanError`
- GREEN: Update all error construction sites
- VERIFY: `cargo test --lib fs::scanner`

**Slice 10: Migrate reader.rs**
- RED: Change all method return types from `ParseError` to `FsError`
- GREEN: Update error construction (use `ReadError`, `ParseError` via `?`)
- VERIFY: `cargo test --lib fs::reader`

**Slice 11: Migrate schema/error.rs**
- RED: Update `From<ParseError>` to handle only 4 variants
- Observe compile errors for `Io` and `NotInBasePath` arms
- GREEN: Add `From<ReadError>` impl handling those 2 variants
- VERIFY: `cargo test --lib schema::error`

**Slice 12: Migrate config/error.rs**
- RED: Update `From<ParseError>` to handle only 4 variants
- GREEN: Add `From<ReadError>` impl
- VERIFY: `cargo test --lib config::error`

**Slice 13: Migrate note/error.rs**
- RED: Remove dummy-path hack in `From<ParseError>`
- GREEN: Replace with `From<ReadError>` impl using real path
- VERIFY: `cargo test --lib note::error`

**Slice 14: Delete DirEntryError**
- RED: Delete `DirEntryError` enum from error.rs
- Observe compile errors (should be zero if Slice 8 was complete)
- GREEN: Remove dead code
- VERIFY: `cargo test --lib fs::error`

**Slice 15: Finalize**
- Update `mod.rs` re-exports
- Run `mise run verify`
- Mark all acceptance criteria complete

### Behavioral Test Examples

**Good (behavior-focused):**
```rust
#[test]
fn formats_io_error_with_path_context() {
    let error = ReadError::Io {
        path: PathBuf::from("vault/note.md"),
        source: std::io::Error::new(ErrorKind::NotFound, "not found"),
    };
    let msg = format!("{error}");
    assert!(msg.contains("vault/note.md"), "Expected path in message");
    assert!(msg.contains("not found"), "Expected source error message");
}
```

**Bad (implementation-focused):**
```rust
#[test]
fn read_error_has_two_fields() {  // ❌ Tests structure, not behavior
    let error = ReadError::Io { ... };
    assert_eq!(std::mem::size_of_val(&error.path), 24);  // ❌ Meaningless
}
```

### Integration with Definition of Done

Before marking this issue complete:
- [ ] All tests pass (`mise run test`)
- [ ] Code formatted (`mise run fmt`)
- [ ] No clippy warnings (`mise run lint`)
- [ ] All public error types have tests (Display messages, conversions)
- [ ] Tests cover critical paths (auto-conversions via `#[from]`, source preservation)
- [ ] No `unwrap()`/`panic!` in production code
- [ ] Documentation updated (doc comments for all new public error types)
- [ ] ADR 017 implementation date filled in
- [ ] No string allocation anti-patterns in error construction (use `Box<str>` where appropriate)
