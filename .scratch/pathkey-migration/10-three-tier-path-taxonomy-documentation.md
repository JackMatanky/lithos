---
title: "Issue 10: Document three-tier path taxonomy (ADR + doc comments)"
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
date_completed: null
---

# Issue 10: Document three-tier path taxonomy

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Capture the completed path taxonomy as an ADR and in module-level doc comments so the design is durable and navigable without architecture tests.

### ADR

Write a new ADR (`docs/adr/020-three-tier-path-taxonomy.md`) recording the decision:

- **Context**: The migration from `RelativePath` (struct) → `RelativePath` (enum), config types tightened to `RelativeDirPath`/`RelativeFilePath`, `PathKey` established as sole repository boundary type, `NormalizedPath` removed
- **Decision**: Three-tier path taxonomy with clear boundary rules and conversion seams
- **Consequences**: Type system enforces correct usage; developers consult the taxonomy table rather than CI failures

Include a taxonomy reference table:

| Tier | Types | Where | Example |
|------|-------|-------|---------|
| **Filesystem I/O** | `FsPath`, `FilePath`, `DirPath` | Scanner, reader, writer, vault processor | `DirPath::append_file(&rel_file)` |
| **Display / Config** | `RelativePath` enum, `Relative*Path` | CLI display, config values, serialization | `RelativeDirPath::try_new("schemas")` |
| **Storage Keys** | `PathKey` | Repository traits, DB tables | `fn find_file_view_by_path(&PathKey)` |

And the conversion seam reference:

| Source → Target | Method | Fallible? |
|----------------|--------|-----------|
| Config value → FS path | `DirPath::append_dir(&RelativeDirPath)` | Yes |
| Config value → FS path | `DirPath::append_file(&RelativeFilePath)` | Yes |
| FS path → Storage key | `file_path.as_key(root)` | Yes |
| FS path → Display | `file_path.as_relative(base) → RelativePath::File(...)` | Yes |
| FS path → Display | `dir_path.as_relative(base) → RelativePath::Dir(...)` | Yes |

### Doc comments

Update module-level doc comments in:

- **`lithos-core/src/fs/path.rs`**: Add a module-level documentation block explaining the three-tier taxonomy, type choice guidance, and conversion seam reference. Mark the `RelativePath` enum with its intended use case (display, serialization — not FS I/O, not storage keys).
- **`lithos-core/src/config/paths.rs`**: Update the module doc to note that config stores declarative types (`RelativeDirPath`/`RelativeFilePath`) rather than filesystem-validated or storage-key types.
- **`lithos-core/src/fs/CONTEXT.md`**: Update "Normalized Path" language entry to reference the taxonomy and clarify the role of `RelativePath` enum.
- **`lithos-core/src/config/CONTEXT.md`**: Add note about config using `RelativeDirPath`/`RelativeFilePath` for declarative path values.

## AGENT-BRIEF
**Category:** enhancement
**Summary:** Document the three-tier path taxonomy (FS I/O, Display/Config, Storage Keys).

**Context:** The codebase has moved away from `NormalizedPath` towards a clearer three-tier system:
1. **Filesystem I/O**: `FsPath`, `FilePath`, `DirPath` (rooted, exist on disk)
2. **Display / Config**: `RelativePath`, `Relative*Path` (declarative, platform-agnostic)
3. **Storage Keys**: `PathKey` (normalized, forward-slash, DB-ready)

**Key Task:** Update all architectural and module documentation to reflect this taxonomy and provide a clear conversion reference for developers.

## TDD PLAN
1. **ADR Creation**: Generate `docs/adr/020-three-tier-path-taxonomy.md` using the taxonomy and conversion tables provided in the issue. Use `docs/adr/template.md`.
2. **Module Doc Updates**:
    - `lithos-core/src/fs/path.rs`: Module-level docs with taxonomy table and conversion reference. Improve existing doc comments for clarity and precision.
    - `lithos-core/src/config/paths.rs`: Module-level docs mentioning declarative path types.
3. **Context Alignment**:
    - `lithos-core/src/fs/CONTEXT.md`: Update "Normalized Path" language entry to reference the taxonomy and clarify roles. Do NOT mention removal of `NormalizedPath`.
4. **Verification**:
    - `cargo doc --no-deps` to ensure no documentation warnings or broken links.
    - `cargo test --doc` to verify code examples.
    - `mise run test:unit` for sanity check.

## Acceptance criteria

- [ ] `docs/adr/020-three-tier-path-taxonomy.md` exists with context, decision, taxonomy table, and conversion seams
- [ ] `lithos-core/src/fs/path.rs` module doc explains the taxonomy and type choice guidance
- [ ] `lithos-core/src/config/paths.rs` module doc mentions declarative path types
- [ ] `lithos-core/src/fs/CONTEXT.md` updated with taxonomy reference
- [ ] `lithos-core/src/config/CONTEXT.md` updated with declarative path types note
- [ ] All doc comments pass `cargo doc` without warnings

### Blocked by

- `.scratch/pathkey-migration/09-relativepath-enum-and-config-migration.md` (the taxonomy must exist before documenting it)
