# PRD: Decouple FileReader and DirScanner

**Status**: drafting
**Created**: 2026-05-26
**Context**: The `FsReader` has become a shallow module that mixes file I/O operations with directory traversal. This PRD outlines the strategy to deepen the architecture by splitting these responsibilities into distinct seams: `FileReader` for I/O and `DirScanner` for traversal.

---

## Problem Statement

Currently, `FsReader` provides an interface that is nearly as complex as its implementation. It violates single-responsibility by managing both File I/O (reading bytes/strings and fetching metadata) and Directory Scanning (glob-based filtering and traversal like `filter_entries`).

This creates significant architectural friction:
- **Poor Leverage**: Consumers like `VaultProcessor` and `SchemaProcessor` must depend on the entire `FsReader` surface area even if they only need one capability.
- **Poor Locality**: Traversal bugs and I/O bugs all funnel back to `fs/reader.rs`.
- **Testing Friction**: Mocking or instantiating `FsReader` requires setting up both I/O boundaries and traversal trees, even when only one behavior is under test.

## Solution

We will decouple I/O and traversal into two distinct boundaries:

1. **`FileReader` (I/O)**: A stripped-down version of `FsReader` that only handles reading file contents and extracting metadata scoped to a validated root. It retains the `Validator` to ensure paths passed to it are safe.
2. **`DirScanner` (Traversal)**: A standalone component dedicated to discovering files and directories. It traverses a root path and yields `FsEntry` objects containing strongly-typed `FsPath` instances (which are guaranteed to exist).

Processors like `VaultProcessor` and `SchemaProcessor` will be updated to accept `FileReader` and `DirScanner` as separate dependencies. Discovery engines will instantiate `DirScanner` to find files, and pass the results to processors which then use `FileReader` for I/O.

---

## User Stories

1. As a system architect, I want file reading and directory scanning to be separate adapters, so that I can mock them independently in tests.
2. As a system architect, I want `VaultProcessor` and `SchemaProcessor` to accept `FileReader` and `DirScanner` as distinct dependencies, so that I have fine-grained control over their injection.
3. As a developer, I want `DirScanner` to guarantee the existence and type of the files it discovers (via `FsPath`), so that `FileReader` can trust the paths it receives.
4. As a developer, I want discovery engines (`schema/discovery.rs` and `config/discovery.rs`) to instantiate `DirScanner` and pass the results downward, so that processors don't have to manage their own discovery traversal.

---

## Implementation Decisions

### 1. `FileReader` Refactor
- Rename `FsReader` to `FileReader` in `src/fs/reader.rs`.
- Strip all `filter_*` methods (`filter_entries`, `filter_file_paths`, `filter_dir_paths`, etc.) from `FileReader`.
- Retain the `Validator` within `FileReader` to enforce strict/flexible boundary security on arbitrary path strings.

### 2. Dependency Injection Updates
- Update `VaultProcessor` (`scan_views`) to accept a `DirScanner` for discovering vault files, removing its reliance on the reader for traversal.
- Update all other processors (`SchemaProcessor`, `NoteProcessor`, `ConfigBuilder`, etc.) to accept `FileReader` instead of `FsReader`. They already do not use any scanning methods, so this is a strict type rename for them.
- Continue using `DirScanner` natively in `schema/discovery.rs` and `config/discovery.rs`.

### 3. Type State and Validation
- Rely on `FsPath` (and its variants `FilePath`/`DirPath`) guarantees: since `DirScanner` constructs these types, any function accepting an `FsPath` can safely assume it is structurally valid.
- The `Validator` inside `FileReader` acts as a fail-safe security layer for any paths constructed outside of `DirScanner` (like direct string paths from config).

## Testing Decisions

- **Deletion Test**: Verify that replacing `FsReader` with `FileReader` + `DirScanner` isolates traversal complexity to the discovery modules, leaving the processors focused purely on domain logic.
- **Modules Tested**: Unit tests in `src/fs/reader.rs` will be pruned of `filter_*` tests (as these are effectively tested in `scanner.rs`).
- **Prior Art**: We will rely heavily on existing integration tests in `tests/note_reader.rs`, `tests/schema_loader.rs`, and the processor test modules to ensure end-to-end behavior remains identical.

## Out of Scope

- Extraction of the `DocumentParser` (handling format classification and Markdown extraction). This is covered in a separate PRD (`.scratch/fs-reader-parser-seam/PRD.md`).
- A centralized Application Service `FsContext`. We decided against this in favor of explicit, targeted dependency injection.
- Removing I/O from `FilePath`/`DirPath` constructors. We will retain `.is_file()` / `.is_dir()` checks to uphold their type-state guarantees.

## Further Notes

This refactor lays the groundwork for a future **centralized discovery processor**. By decoupling traversal from reading now, a future discovery phase can completely own `DirScanner`, build the file manifest, and pass only the needed files and a `FileReader` down to the domain processors.
