---
title: "Issue 01: PathKey core type and normalization pipeline"
category: "enhancement"
label: "ready-for-agent"
status: "ready-for-agent"
date_created: "2026-05-25"
date_completed: null
---

# Issue 01: PathKey core type and normalization pipeline

Labels: `ready-for-agent`
Type: AFK

## Parent

- `.scratch/pathkey-migration/PRD.md`

## What to build

Implement `PathKey` as the canonical persistence-key type by renaming `NormalizedPath` and formalizing the `trim -> normalize -> validate` pipeline with root-scoped conversion errors.

## Agent Brief

**Category:** enhancement
**Summary:** Establish `PathKey` as canonical repository key type with strict normalization and root-scoped conversion errors.

**Current behavior:**
`NormalizedPath` exists but lacks canonical key semantics, complete normalization guarantees (duplicate/trailing separators), and robust root-scoped conversion error coverage.

**Desired behavior:**
`PathKey` replaces `NormalizedPath` as the persistence-key primitive. Normalization strictly follows a `trim -> normalize -> validate` pipeline utilizing "parse, don't validate", preserving leading `/`, and optimizing with `Cow` (zero-copy when canonical, single-allocation otherwise). Filesystem-to-key conversion is root-scoped and fallible.

**Key interfaces:**
- `PathKey::try_new(path: &str) -> Result<Self, PathError>`
- `PathKey::from_rooted_path(root: &DirPath, path: &Path) -> Result<Self, PathError>`
- `FilePath::as_key(root: &DirPath)`, `DirPath::as_key(root: &DirPath)`
- `PathError::OutsideRoot`, `PathError::InvalidUtf8`
- `NormalizedPath` (deprecated type alias)

**Acceptance criteria:**
- [ ] `PathKey` successfully parses valid strings and rejects traversals (`..`), current dir (`.`), and absolute paths post-normalization.
- [ ] Normalization pipeline deduplicates separators, removes trailing separators, and enforces forward slashes in one pass (single allocation).
- [ ] `PathError::OutsideRoot` and `PathError::InvalidUtf8` are surfaced when converting from filesystem paths.
- [ ] Traceable to PRD User Stories: #8, #15, #16, #21.

**Out of scope:**
- Repository signature migration (handled in subsequent slices).
- Unicode normalization (e.g., NFC).
