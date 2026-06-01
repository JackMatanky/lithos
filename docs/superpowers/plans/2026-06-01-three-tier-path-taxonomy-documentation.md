# Three-Tier Path Taxonomy Documentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Formally document the three-tier path taxonomy in an ADR and module-level doc comments.

**Architecture:** Update architectural and module documentation to reflect the FS I/O, Display/Config, and Storage Key tiers.

**Tech Stack:** Rust, Cargo, Markdown

---

### Task 1: ADR Creation

**Files:**
- Create: `docs/adr/020-three-tier-path-taxonomy.md`
- Reference: `docs/adr/template.md`

- [ ] **Step 1: Create the ADR file from template**

```markdown
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
```

- [ ] **Step 2: Verify ADR format**

Run: `mise run adr:validate`
Expected: PASS

- [ ] **Step 3: Commit ADR**

```bash
git add docs/adr/020-three-tier-path-taxonomy.md
git commit -m "docs: create ADR 020 for three-tier path taxonomy"
```

---

### Task 2: Update Module Documentation in `lithos-core/src/fs/path.rs`

**Files:**
- Modify: `lithos-core/src/fs/path.rs`

- [ ] **Step 1: Replace module-level doc comment**

Replace lines 1-19 with the new taxonomy-focused documentation.

```rust
//! Three-tier path taxonomy for the Lithos core library.
//!
//! This module provides a hierarchy of type-safe path wrappers that enforce
//! filesystem invariants and security policies. The path system is organized into
//! three distinct tiers based on their purpose and validation rules.
//!
//! # Path Taxonomy
//!
//! | Tier | Types | Purpose | Validation |
//! |------|-------|---------|------------|
//! | **Filesystem I/O** | [`FsPath`], [`FilePath`], [`DirPath`] | Rooted, platform-native paths for direct I/O. | Exist on disk, scoped to vault. |
//! | **Display / Config** | [`RelativePath`], [`RelativeDirPath`], [`RelativeFilePath`] | Declarative, platform-agnostic paths for config. | Lexical validation, no I/O. |
//! | **Storage Keys** | [`PathKey`] | Normalized, forward-slash keys for DB storage. | Forward-slash only, UTF-8. |
//!
//! # Type Choice Guidance
//!
//! - **Use FS I/O types** when performing actual filesystem operations (read, write, metadata).
//! - **Use Display/Config types** for values coming from configuration files, CLI arguments, or when sending paths to a UI.
//! - **Use Storage Keys** for database indices, serialized metadata, or cross-platform identifiers.
//!
//! # Conversion Seams
//!
//! | Source → Target | Method | Description |
//! |----------------|--------|-------------|
//! | Config → FS path | `DirPath::append_dir(&RelativeDirPath)` | Resolves a declarative path against a rooted dir. |
//! | FS path → Key | `file_path.as_key(root)` | Normalizes a rooted path into a storage key. |
//! | FS path → Display | `file_path.as_relative(base)` | Converts a rooted path back to a relative view. |
```

- [ ] **Step 2: Update `RelativePath` enum documentation**

```rust
/// Unified view of a declarative relative path.
///
/// This enum is intended for **Display** and **Configuration** purposes. It provides
//! a platform-agnostic way to represent paths before they are resolved against a
//! filesystem root.
//!
//! For filesystem operations, use [`FsPath`]. For storage, use [`PathKey`].
```

- [ ] **Step 3: Verify docs and tests**

Run: `cargo doc --no-deps && cargo test --doc`
Expected: PASS

- [ ] **Step 4: Commit changes**

```bash
git add lithos-core/src/fs/path.rs
git commit -m "docs: overhaul path module documentation with three-tier taxonomy"
```

---

### Task 3: Update `lithos-core/src/config/paths.rs`

**Files:**
- Modify: `lithos-core/src/config/paths.rs`

- [ ] **Step 1: Enhance module documentation**

```rust
//! Validated path configuration management.
//!
//! This module defines how Lithos manages its filesystem locations (cache,
//! schemas, templates). Configuration values use **declarative path types**
//! ([`RelativeDirPath`] and [`RelativeFilePath`]) which are platform-agnostic
//! and only lexically validated. They must be resolved against a vault root
//! into Filesystem I/O types ([`DirPath`], [`FilePath`]) before use.
```

- [ ] **Step 2: Verify and Commit**

```bash
git add lithos-core/src/config/paths.rs
git commit -m "docs: update config paths documentation to mention declarative types"
```

---

### Task 4: Context Alignment

**Files:**
- Modify: `lithos-core/src/fs/CONTEXT.md`
- Modify: `lithos-core/src/config/CONTEXT.md`

- [ ] **Step 1: Update `fs/CONTEXT.md`**

Replace "Normalized Path" definition:

```markdown
**Normalized Path**:
A vault-relative path normalized to forward slashes for cross-platform storage keys.
The path system follows a three-tier taxonomy:
1. **Filesystem I/O**: `FsPath`, `DirPath`, `FilePath` (rooted, validated).
2. **Display/Config**: `RelativePath` (enum), `RelativeDirPath`, `RelativeFilePath` (declarative).
3. **Storage Keys**: `PathKey` (normalized, forward-slash).
Use [`PathKey`] for database keys and serialized path storage.
Use [`FsPath`], [`DirPath`], and [`FilePath`] for filesystem operations.
_Avoid_: platform-specific path, absolute storage key
```

- [ ] **Step 2: Update `config/CONTEXT.md`**

Add note to "Invariants" or "Language":

```markdown
**Declarative Paths**:
Configuration values use declarative relative paths (`RelativeDirPath`, `RelativeFilePath`) which represent intended locations without requiring they exist on disk at configuration time.
```

- [ ] **Step 3: Verify and Commit**

```bash
git add lithos-core/src/fs/CONTEXT.md lithos-core/src/config/CONTEXT.md
git commit -m "docs: align context files with three-tier path taxonomy"
```

---

### Task 5: Final Verification

- [ ] **Step 1: Run full verification**

Run: `mise run verify` (if available) or `cargo doc --no-deps && cargo test --doc && mise run test:unit`
Expected: PASS
