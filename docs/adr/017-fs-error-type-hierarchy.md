---
name: fs-error-type-hierarchy
status: accepted
date_proposed: 2026-05-13
date_decided: 2026-05-13
date_implemented:
stakeholders: [Engineering]
---

# ADR 017: FS Error Type Hierarchy

## Context

The FS context's error module (`fs/error.rs`) uses three error types: `ParseError` (catch-all for I/O, parsing, and path operations), `PathValidationError` (security validation), and `DirEntryError` (legacy conversions). `ParseError` serves as a junk drawer — its `Io` variant absorbs walkdir failures, glob errors, metadata reads, path encoding, and file reads by wrapping them in `std::io::Error::other(...)`, erasing the original error semantics.

The fs-inode-architecture refactoring (Phase 3+) requires migrating consumer error types across schema/, config/, and note/ contexts. Before that migration, the error module needs a clear decomposition so consumers adopt well-defined types rather than inheriting the current catch-all.

## Decision

We will decompose the FS error module into six types, each with a single responsibility:

1. **`PathError`** (11 variants) — Path construction and name extraction failures. Used by `TryFrom` impls in `path.rs` and `name.rs`. Self-documenting variants (`NotAFile`, `NotRelative`, `NoFileName`, etc.) replace `std::io::Error::new(InvalidInput, "...")` strings.

2. **`ReadError`** (2 variants) — File input access failures. `Io` for read/metadata I/O errors with path context; `NotInBase` for vault root boundary violations. Scoped to input operations; a future `WriteError` will handle output.

3. **`ScanError`** (4 variants) — Directory traversal failures. `Traversal` preserves the original `io::Error` as a source (not erased); `InvalidPattern` for glob errors; `UnsupportedEntryType` for non-file/non-directory entries (sockets, fifos); `Path` composes `PathError` for construction failures during scanning.

4. **`ParseError`** (4 variants) — Structured file deserialization only. `Json`, `Toml`, `Yaml` for format-specific parse failures; `UnsupportedFormat` for unrecognized extensions. No I/O — reading is `ReadError`'s concern.

5. **`PathValidationError`** (unchanged, 9 variants) — Security validation for path traversal, symlink escape, restricted paths.

6. **`FsError`** (5 `#[from]` variants) — Pure compositor. Wraps all five child types via `#[error(transparent)]` with `#[from]`. Zero direct variants. Serves as the module's public-facing return type on `FsReader` methods.

### Composition hierarchy

```text
FsError (compositor)
 ├── Read(ReadError)
 ├── Scan(ScanError)
 │    └── Path(PathError)
 ├── Parse(ParseError)
 ├── Path(PathError)
 └── Validation(PathValidationError)
```

### Deleted types

- `DirEntryError` — absorbed into `ScanError::Traversal` and `PathError::InvalidUtf8`.

## Alternatives Considered

### Alternative 1: Single expanded FsError enum

All variants in one enum (I/O, scan, parse, path, validation).
- **Pros**: One type to import; simple `?` propagation.
- **Cons**: 30+ variants in a single enum. Consumers cannot match on a subset. `ParseError` consumers (config, schema) would need to handle scan/path variants they don't care about. Rejected because it violates interface segregation and makes consumer `From` impls unwieldy.

### Alternative 2: Keep ParseError as catch-all, add FsError as compositor

Keep `ParseError` with its current dual role (parsing + I/O) and add `FsError` on top.
- **Pros**: Minimal migration.
- **Cons**: Perpetuates the `ParseError::Io` junk drawer. Consumers continue matching on 6 ParseError variants where only 4 are relevant. The name "ParseError" continues to mislead about non-parse failures. Rejected because it doesn't solve the semantic confusion.

## Technical Validation

### Research Findings

- **Error site inventory**: 60+ error creation sites catalogued across all fs/ modules. Every site maps to exactly one of the six proposed types.
- **Consumer analysis**: `schema/error.rs`, `config/error.rs`, and `note/error.rs` all implement `From<ParseError>`. The current `From` impls destructure all 6 ParseError variants; the new design splits these into separate `From<ParseError>` (4 variants) and `From<ReadError>` (2 variants) impls, each focused on its concern.
- **Rust best practices**: `thiserror` with `#[from]` for automatic conversions; `#[non_exhaustive]` on all enums for forward compatibility; `#[error(transparent)]` for compositor variants.

## Consequences

- **Positive**:
    - Each error type has a clear, non-overlapping responsibility.
    - Call sites become simpler: `PathError::NotAFile(path)` vs `io::Error::new(InvalidInput, "Path does not refer to a file")`.
    - Consumer `From` impls become focused — no irrelevant variant arms.
    - `ReadError` is scoped to input; `WriteError` can be added later without touching existing types.
    - Error sources are preserved (`ScanError::Traversal` keeps the original `io::Error`) instead of being erased.
- **Negative**:
    - Six types instead of three — more to learn for new contributors.
    - Consumer migration required across schema/, config/, note/ contexts.
- **Risks**:
    - Over-decomposition if future changes introduce types that blur the boundaries. Mitigated by the clear operational mapping (path construction → PathError, file reads → ReadError, scanning → ScanError, deserialization → ParseError).

## References

- [ADR 005: Error Handling and Diagnostics Framework](./005-error-handling.md) — Establishes thiserror + miette as the error handling stack.
- [Apollo GraphQL Rust Best Practices: Error Handling](https://github.com/apollographql/rust-best-practices) — Error hierarchy patterns with `#[from]`.
- [Microsoft Rust Patterns: Error Handling](https://microsoft.github.io/RustTraining/rust-patterns-book/ch10-error-handling-patterns.html) — thiserror vs anyhow guidance.
