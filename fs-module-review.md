# fs module review

This document captures the fs module review, including the full set of problems and the agreed solutions after clarifying the read/parse pipeline, boundary rules, and public surface constraints.

## Scope and intent

The fs module must provide capabilities beyond std::fs by centralizing:
- root scoping and path validation policy
- deterministic discovery (glob-based, stable order)
- a well-defined read pipeline (validation → classify → read → parse)
- safe write orchestration (atomic replace)
- consistent metadata access

Domain types must not import fs. Context-specific parsing (notably Markdown options) remains a context concern.

## Problems and solutions

### 1) Path validation split and context boundary violations

Problem:
- `NotePath::new()` imports `crate::fs::validate_vault_path` (infrastructure dependency in a domain type).
- `validate_vault_path` and `PathValidator` duplicate logic and disagree on `..` handling.

Solutions:
- Inline domain-level validation in `NotePath::new()` (relative, `.md`, no `..`, no dotfiles). No fs import.
- Move `validate_vault_path` into `validator.rs` and make it delegate to `PathValidator`.
- Fix `validate_vault_path` to check `Component::ParentDir` rather than substring `".."`.
- Fix Windows drive-relative detection (reject `C:relative` and `C:\absolute`).

### 2) Read vs parse pipeline ambiguity

Problem:
- Reading and parsing were conflated, leading to confusion over responsibilities and public APIs.

Solutions:
- Define the pipeline explicitly in `FsReader`:
  1) `validate_path`
  2) `classify(path)`
  3) `read_bytes` / `read_to_string`
  4) `parse_structured<T>` (JSON/TOML/YAML only)
  5) `read<T>` convenience method that dispatches via classification and parsing
- Keep parsing for JSON/TOML/YAML in fs, but allow context-specific Markdown parsing through a closure hook.

### 3) Format identification should be explicit and type-safe

Problem:
- A generic “Structured” bucket is too coarse, and risks parsing JSON as TOML or YAML.

Solutions:
- Keep Json/Toml/Yaml structs and rely on existing `is_supported(path)` methods.
- `FsReader::classify(path)` uses the public type-level predicates (e.g., `Json::is_supported(path)`), and each type’s `parse` must call its own `is_supported` guard to prevent mismatches.
- Ensure `parse_structured<T>` checks the file type before parsing to prevent mismatches.

### 4) Parsing should not force context-specific Markdown decisions

Problem:
- Markdown parsing is context-dependent (Obsidian options), and fs should not own those policies.

Solutions:
- FsReader exposes `read_with<T>(path, f)` where `f` is a closure `(path, text) -> Result<T, ParseError>`.
- Contexts inject their own Markdown parsing without fs depending on pulldown-cmark.
- FsReader can still classify Markdown by extension for pipeline dispatch.

### 5) FsReader/FsWriter split and value beyond std

Problem:
- The current `FileSource` is a thin wrapper and not worth its maintenance cost.

Solutions:
- Rename `FileSource` to `FsReader` in `reader.rs`.
- Introduce `FsWriter` in `writer.rs` with safe write orchestration.
- Ensure fs adds value beyond std via:
  - root-scoped validated paths for all operations
  - deterministic list ordering
  - glob-based discovery with root-relative patterns
  - metadata access via a stable `FileMetadata` struct
  - atomic replace for safe writes

### 6) Glob discovery and traversal correctness

Problem:
- `list_files` compiles the glob per entry and silently drops errors.
- `walkdir` is unnecessary for simple globbing.

Solutions:
- Use `glob::glob()` with a single compiled pattern.
- Return errors on invalid patterns.
- Make ordering deterministic by sorting results.
- Use root-relative patterns and strip the root from results for consistency.

### 7) Symlink policy must be context-dependent

Problem:
- Hard-coded symlink exclusion is incorrect for some contexts (e.g., config files often symlinked).

Solutions:
- Default to **including symlinks** in discovery.
- Add a policy hook or configuration for stricter contexts later.
- Document the future policy option in fs docs.

### 8) Atomic write durability decision

Problem:
- `rename` is not a full durability guarantee; parent-dir fsync may be required for crash safety.

Recommendation:
- Default to a lean, safe approach:
  - write temp file with `create_new`
  - `sync_all` on the file
  - `rename` to target
- Avoid parent-directory fsync by default for performance, but keep API space for a stricter mode later if needed.

### 9) Tests must use tempfile only

Problem:
- In-memory sources diverge from real filesystem behavior.

Solutions:
- Remove in-memory implementations.
- Replace tests with `tempfile::TempDir` integration tests.
- Ensure tests cover glob patterns, symlink handling, and read/parse pipeline dispatch.

## File-level plan (summary)

### fs/reader.rs (renamed from source.rs)
- Rename `FileSource` -> `FsReader`.
- Add pipeline methods: `validate_path`, `classify`, `read_bytes`, `read_to_string`, `parse_structured<T>`.
- Add convenience `read<T>` that dispatches by file type.
- Add `read_with<T>` closure hook for Markdown parsing.
- Add `metadata()`.
- Fix `list_files()` to use `glob::glob()` with deterministic ordering.

### fs/types.rs (renamed from parsers.rs)
- Keep Json/Toml/Yaml structs only (no FsParser).
- Use existing `is_supported(path)` in each type’s `parse` method (no separate helpers).
- Keep parse methods for each struct and ensure they validate the file type.

### fs/writer.rs (new)
- Add `FsWriter` and `OsFsWriter`.
- Implement `atomic_write` (no parent-dir fsync by default).

### fs/validator.rs
- Move `validate_vault_path` here and delegate to `PathValidator`.
- Fix Windows drive-relative check.

### note/aggregate.rs
- Inline `NotePath::new` domain validation with no fs dependency.

## Open questions resolved by this review

1) **Format classification**: `FsReader::classify(path)` with type helpers in `types.rs`.
2) **Convenience read**: provide `read<T>(path)` that dispatches to the correct read/parse path.
3) **Markdown parsing hook**: use closure-based `read_with<T>` to avoid coupling to pulldown-cmark.
4) **Public surface**: keep FormatKind internal where possible; prefer `FsReader` as the main entry point.
