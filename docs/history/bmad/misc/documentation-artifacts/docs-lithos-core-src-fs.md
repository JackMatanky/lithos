---
project: lithos-rust
target: lithos-core/src/fs
date: 2026-02-19
mode: create
components: modules=5, structs=8, enums=5, functions=35, traits=0
compliance: rfc1574
---

# Generated Rustdoc Documentation

## Crate Documentation

Target is a module directory (`lithos-core/src/fs`) within the `lithos-core`
crate, so crate-level docs are not generated here.

## Module Documentation

### `lithos-core/src/fs/mod.rs`

//! Filesystem-related utilities and infrastructure.
//!
//! This module contains file system infrastructure for security validation,
//! deterministic discovery, read pipelines, and safe write orchestration.
//!
//! ## Security-Critical Modules
//!
//! - **validator**: Path traversal protection and security validation.
//!   - Prevents path traversal attacks, symlink escapes, and arbitrary file
//!     access.
//!   - Re-exported as `PathValidator` for ergonomic imports.
//!
//! ## Data Processing Modules
//!
//! - **reader**: Root-scoped file access with validation and classification.
//!   - Read pipeline: validate → classify → read → parse.
//! - **types**: TOML/JSON/YAML parsing helpers with explicit format guards.
//! - **writer**: Root-scoped writes with atomic replace.

### `lithos-core/src/fs/error.rs`

//! Error types for filesystem and parsing operations.

### `lithos-core/src/fs/reader.rs`

//! File system abstraction for testable file I/O.
//!
//! This module provides the [`Reader`] concrete type for scoped filesystem
//! access. The reader keeps path resolution anchored to a root directory so
//! adapters can perform deterministic file access without leaking filesystem
//! details into domain logic.

### `lithos-core/src/fs/types.rs`

//! File type markers and parsing helpers for structured formats.
//!
//! This module defines file type helpers used to classify and parse
//! structured configuration files. JSON/TOML/YAML expose detect + parse
//! helpers; Markdown is represented as a file type without detect/parse
//! support.

### `lithos-core/src/fs/validator.rs`

//! **Security-Critical Path Validation Utilities.**
//!
//! This module provides path validation to prevent **path traversal attacks**,
//! **arbitrary file access**, and **symlink escape vulnerabilities**.
//!
//! **SECURITY REQUIREMENT**: All file I/O operations in adapters MUST use these
//! validation utilities before accessing the filesystem. Bypassing these checks
//! creates critical security vulnerabilities.

### `lithos-core/src/fs/writer.rs`

//! Filesystem writer utilities for safe writes.
//!
//! The writer keeps all paths scoped to a root directory and validates inputs
//! before touching the filesystem. This preserves adapter safety guarantees
//! while providing atomic replace semantics for file updates.

## Type Documentation

### Enums

- `FsError`: File system error types (I/O failures).
- `ParseError`: Structured parse errors with file path and line/column context.
- `PathValidationError`: Path validation failures (empty, absolute, traversal,
  restricted, extension, symlink escape, or I/O errors).
- `FormatKind`: Internal read-pipeline classification (json/toml/yaml/markdown,
  binary, unknown).
- `Mode` (crate-private): Validator mode (flexible vs strict).

### Structs

- `Reader`: Root-scoped filesystem reader for adapter ingestion.
- `FileMetadata`: Lightweight metadata (modified time, size, symlink flag).
- `Writer`: Root-scoped filesystem writer with atomic replace support.
- `Validator`: Path validator with strict/flexible security modes.
- `Json`, `Toml`, `Yaml`: Structured format parsers with extension guards.
- `Markdown`: File type marker for markdown extensions.

### Type Aliases

- `FileReader`: Alias for `Reader` used at adapter call sites.
- `FsWriter`: Alias for `Writer` used at adapter call sites.

## Function Documentation

### Free Functions

- `is_windows_absolute_path`: Checks whether a string is a Windows absolute or
  drive-relative path.
- `validate_vault_path`: Validates vault-relative paths for emptiness, relative
  form, traversal, and optional extension constraint, standardizing checks for
  adapter call sites.

### `Reader` methods

- `new`: Creates a reader scoped to a root directory.
- `root`: Returns the root directory.
- `classify`: Classifies a path by extension/heuristics into `FormatKind`.
- `exists`: Returns whether a file exists at the given path.
- `list_files`: Globs files relative to the root.
- `metadata`: Reads symlink-aware metadata for a path.
- `parse_structured`: Parses JSON/TOML/YAML based on extension.
- `read_bytes`: Reads a file as bytes.
- `read_to_string`: Reads a file as UTF-8 text.
- `read_with`: Reads text and delegates parsing to a caller closure.
- `validate_path`: Validates a path with the flexible validator.

### `Writer` methods

- `new`: Creates a writer scoped to a root directory.
- `create_dir_all`: Creates directories for a given path.
- `write_file`: Writes bytes to a file (create/truncate).
- `atomic_write`: Writes via temp file + rename for atomic replace.
- `rename`: Renames a file.
- `remove_file`: Removes a file.

### `Validator` methods

- `new_flexible`: Allows external symlinks while still preventing traversal.
- `new_strict`: Enforces root boundary and rejects escaping symlinks.
- `resolve_safe_symlink`: Canonicalizes and enforces boundary checks.
- `validate`: Validates a path for traversal, absolute, and restricted checks.

### Structured format helpers

- `Json::detect`, `Toml::detect`, `Yaml::detect`: Heuristics for content shape.
- `Json::is_supported`, `Toml::is_supported`, `Yaml::is_supported`:
  Extension guards.
- `Json::parse`, `Toml::parse`, `Yaml::parse`: Parse into `T` with rich errors.
- `Markdown::is_supported`: Extension guard for `.md`/`.markdown`.

## Trait Documentation

No public traits in this module.

## Validation Report

- RFC 1574 conventions used for module/enum/struct docs.
- Examples use `?` and return `Ok(())` where applicable.
- All public `Result`-returning APIs document `# Errors`.
- Unsafe functions are not present.

## Application Instructions

1. Copy doc comments into your source files (already applied).
2. Run `cargo doc --no-deps` to preview HTML.
3. Run `cargo test --doc` to verify examples.
