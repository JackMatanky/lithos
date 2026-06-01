---
name: three-tier-path-taxonomy
status: accepted
date_proposed: 2026-06-01
date_decided: 2026-06-01
date_implemented: 2026-06-01
stakeholders: [Core Team]
---

# ADR 020: Three-Tier Path Taxonomy

## Context

Path handling across the application has historically been a source of ambiguity and fragility. The existence of `NormalizedPath` alongside multiple platform-specific and logical path types led to confusion regarding which type should be used at different architectural boundaries. Specifically, there was a migration from the `RelativePath` struct to a `RelativePath` enum, and configuration types were tightened to `RelativeDirPath` and `RelativeFilePath`. At the same time, `PathKey` was established as the sole repository boundary type, replacing the overly generic `NormalizedPath`.

Without a clear taxonomy, developers often face type confusion or unnecessary path conversions across the filesystem I/O boundary, the display/configuration boundary, and the persistent storage boundary. We need a definitive, documented structure mapping path types to these specific domain contexts to ensure type safety and explicit conversions.

## Decision

We will adopt a formal Three-Tier Path Taxonomy that groups our path types into three distinct domains with clear boundary rules and conversion seams.

### Three-Tier Path Taxonomy

| Tier | Types | Where | Example |
|------|-------|-------|---------|
| **Filesystem I/O** | `FsPath`, `FilePath`, `DirPath` | Scanner, reader, writer, vault processor | `DirPath::append_file(&rel_file)` |
| **Display / Config** | `RelativePath` enum, `Relative*Path` | CLI display, config values, serialization | `RelativeDirPath::try_new("schemas")` |
| **Storage Keys** | `PathKey` | Repository traits, DB tables | `fn find_file_view_by_path(&PathKey)` |

### Conversion Seams

To prevent ambiguous casts, conversions between these tiers are explicit and typically fallible where system boundaries are crossed:

| Source → Target | Method | Fallible? |
|----------------|--------|-----------|
| Config value → FS path | `DirPath::append_dir(&RelativeDirPath)` | Yes |
| Config value → FS path | `DirPath::append_file(&RelativeFilePath)` | Yes |
| FS path → Storage key | `file_path.as_key(root)` | Yes |
| FS path → Display | `file_path.as_relative(base) → RelativePath::File(...)` | Yes |
| FS path → Display | `dir_path.as_relative(base) → RelativePath::Dir(...)` | Yes |

## Alternatives Considered

### Alternative 1: Single Universal Path Type
- **Pros**: Reduces the number of types; simpler to pass a single type around.
- **Cons**: Conflates concerns (I/O, presentation, storage). Fails to leverage the compiler to enforce correctness (e.g., trying to write to a declarative, platform-agnostic config path). A universal type usually acts like a string, leading to "stringly-typed" architecture flaws.

### Alternative 2: Maintaining NormalizedPath
- **Pros**: Minimal refactoring required, relies on existing patterns.
- **Cons**: `NormalizedPath` was ambiguous. It was unclear if it was an on-disk relative path, an abstract key, or a display path. Removing it forces correct boundary mapping.

## Technical Validation

### Research Findings
- Using distinct, zero-cost types for different domains pushes path resolution errors to the boundaries (e.g., during config load or DB read) rather than deep within core logic.

### Benchmarks & Prototypes
- The migration to `PathKey` and the `RelativePath` enum has been verified through architectural unit tests, confirming that these tight boundaries prevent cross-domain path leakage.

## Consequences

- **Positive**:
  - The type system strictly enforces correct usage of paths; developers rely on compiler errors and the taxonomy table rather than debugging runtime CI failures.
  - Clarity on where paths map to external representations (JSON configs vs. SQLite keys vs. OS File descriptors).
- **Negative**:
  - Higher cognitive load initially to learn the distinct types and the specific methods required to convert between them.
- **Risks**:
  - Misuse of `as_key()` or `as_relative()` with the wrong base directory can still lead to logical bugs, even though the type signatures align.

## References
- [Issue 10: Document three-tier path taxonomy](.scratch/pathkey-migration/10-three-tier-path-taxonomy-documentation.md)
