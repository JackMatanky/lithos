---
name: three-tier-path-taxonomy
status: accepted
date_proposed: 2026-06-01
date_decided: 2026-06-01
date_implemented: 2026-06-01
stakeholders: [Architecture Team]
---

# ADR 020: Three-Tier Path Taxonomy

## Context
The project has evolved its path handling logic to resolve ambiguity between filesystem operations, storage representation, and user-facing configuration. Previously, `RelativePath` (struct) and `NormalizedPath` served multiple roles, leading to confusion and potential security risks regarding path scoping and normalization.

## Decision
We have formalized a three-tier path taxonomy that distinguishes between Filesystem I/O, Display/Configuration, and Storage Keys.

### Taxonomy Reference

| Tier | Types | Where | Example |
|------|-------|-------|---------|
| **Filesystem I/O** | `FsPath`, `FilePath`, `DirPath` | Scanner, reader, writer, vault processor | `DirPath::append_file(&rel_file)` |
| **Display / Config** | `RelativePath` enum, `Relative*Path` | CLI display, config values, serialization | `RelativeDirPath::try_new("schemas")` |
| **Storage Keys** | `PathKey` | Repository traits, DB tables | `fn find_file_view_by_path(&PathKey)` |

### Conversion Seams

| Source → Target | Method | Fallible? |
|----------------|--------|-----------|
| Config value → FS path | `DirPath::append_dir(&RelativeDirPath)` | Yes |
| Config value → FS path | `DirPath::append_file(&RelativeFilePath)` | Yes |
| FS path → Storage key | `file_path.as_key(root)` | Yes |
| FS path → Display | `file_path.as_relative(base) → RelativePath::File(...)` | Yes |
| FS path → Display | `dir_path.as_relative(base) → RelativePath::Dir(...)` | Yes |

## Alternatives Considered

### Alternative 1: Unified Path Type
- **Pros**: Fewer types for developers to learn.
- **Cons**: Failed to enforce boundary security at the type level; normalization was inconsistent across platform boundaries.

### Alternative 2: Structural Subtyping (Traits)
- **Pros**: Flexible polymorphism.
- **Cons**: Increased generic complexity; lacked the strict validation guarantees provided by concrete newtypes like `PathKey` and `DirPath`.

## Technical Validation

### Research Findings
- The three-tier system mirrors successful patterns in large-scale asset pipelines where declarative paths are separated from resolved FS handles.
- Security audits of the previous `NormalizedPath` showed potential for confusion between storage keys and relative FS paths.

### Benchmarks & Prototypes
- Prototyped in `lithos-core::fs` showing that the compiler now catches incorrect path usage at repository boundaries.

## Consequences

- **Positive**:
    - Type-level enforcement of path purpose.
    - Clear boundaries for where filesystem validation occurs.
    - Simplified auditing of security-critical path scoping.
    - Consistent storage format (`PathKey`).
- **Negative**:
    - Increased type count in `lithos-core::fs`.
    - Explicit conversions required at context boundaries.

## References
- [Issue 10: Document three-tier path taxonomy](.scratch/pathkey-migration/10-three-tier-path-taxonomy-documentation.md)
- [ADR 019: PathKey as Repository Boundary Type](019-pathkey-repository-boundary-type.md)
