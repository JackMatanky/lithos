---
name: pathkey-repository-boundary-type
status: accepted
date_proposed: 2026-05-24
date_decided: 2026-05-24
date_implemented: null
stakeholders: [Core Team]
---

# ADR 019: PathKey as Sole Repository Boundary Type

## Context

Repository trait signatures currently accept `RelativePath` (a platform-specific, filesystem-oriented type) for database keys instead of normalized canonical storage keys. This forces repeated ad hoc `strip_prefix` + `RelativePath::try_from` conversion chains in `DiscoveryEngine`, `Builder`, and config-to-schema handoff logic. `SchemaConfigSpec` stores relative paths but exposes both relative and absolute accessors with `unreachable!` branches for invariant enforcement. `AbsolutePath` and `RelativePath` serve overlapping roles without clear boundary ownership, and `Config::to_schema_spec()` uses panic-based `expect()` for core path assembly.

This coupling creates semantic drift between filesystem I/O paths (`FsPath`, `FilePath`, `DirPath`) and persistence keys, introduces conversion bugs, and prevents clean separation of execution-facing vs persistence-facing path types.

## Decision

1. **Rename `NormalizedPath` → `PathKey`** to reflect its role as the canonical persistence-key primitive
2. **Make `PathKey` the only type accepted at repository/storage boundaries** across all contexts
3. **Require root-scoped, fallible conversions** for all filesystem→key operations:
   - Core primitive: `PathKey::from_rooted_path(root: &DirPath, path: &Path) -> Result<PathKey, PathError>`
   - Public convenience: `as_key(&self, root: &DirPath) -> Result<PathKey, PathError>` on `FilePath`, `DirPath`, `FsPath`
   - No infallible conversions; no `TryFrom<PathBuf>` (forces root context)
4. **Redesign `SchemaConfigSpec` as execution-facing**: store only typed filesystem paths (`root: VaultRoot`, `directory: DirPath`, `property_bank: FilePath`); derive `PathKey` via `directory_key()` / `property_bank_key()` only at repository call boundaries
5. **Hard-cut repository signatures per context** in sequence:
   - Schema first: replace all `&RelativePath` with `&PathKey` in `schema::repository` traits + storage
   - Vault second: same pattern for `vault::storage`
   - Note/template third: uniform `PathKey` usage at all persistence boundaries
6. **Remove `AbsolutePath` and deprecate `RelativePath`** after migration:
   - `AbsolutePath` replaced by `DirPath` in config-facing types (`VaultRoot`, `TrustedVaultPath` become thin wrappers over `DirPath`)
   - `RelativePath` deprecated with short-lived alias, removed after phased hard cuts
7. **Enforce via architecture tests**: transitional test module (`lithos-core/tests/path_migration_architecture.rs`) bans `AbsolutePath`, `RelativePath`, and `NormalizedPath` alias usage per context phase with explicit exit criteria

## References
- [fs-inode-architecture issue 01](.scratch/fs-inode-architecture/01-path-types.md) - Post-implementation refactors for path types
- [fs-inode-architecture issue 02](.scratch/fs-inode-architecture/02-name-types.md) - Name type design decisions
- [PathKey Migration PRD](.scratch/pathkey-migration/PRD.md) - Comprehensive implementation plan

## Alternatives Considered

### Alternative 1: Keep dual path semantics (`RelativePath` for repositories, `FilePath`/`DirPath` for I/O)
- **Pros**: No migration cost, existing code continues to work
- **Cons**: Perpetuates boundary ambiguity and conversion churn; `RelativePath` semantics (platform-specific path representation) don't match persistence-key needs (canonical, normalized, cross-platform string keys)
- **Rejected because**: This fails to address the core problem of semantic drift between filesystem I/O paths and persistence keys

### Alternative 2: Use separate wrapper types (`FilePathKey`, `DirPathKey`) over `PathKey`
- **Pros**: Type system encodes file-vs-dir at key level
- **Cons**: Key-type explosion without meaningful safety benefit; file-vs-dir semantics already enforced at filesystem-facing APIs (`FilePath`, `DirPath`)
- **Rejected because**: Encoding shape in the key type creates coupling without preventing misuse at repository boundaries

### Alternative 3: Allow rootless `PathKey` conversion (e.g., `PathKey::from_absolute(path)`)
- **Pros**: More ergonomic for simple cases, less boilerplate
- **Cons**: Loses vault-boundary safety; allows cross-root key generation bugs
- **Rejected because**: Explicit root requirement at conversion boundaries prevents ambiguous absolute path handling and bakes safety into the type

### Alternative 4: Do hard cuts across all contexts simultaneously in one mega-PR
- **Pros**: Faster to complete, no transitional state
- **Cons**: Blast radius too high for review and rollback
- **Rejected because**: Sequencing by context (schema → vault → note/template) keeps each PR atomic and testable while still avoiding long-lived dual semantics

## Technical Validation

### Research Findings
- GitNexus impact analysis showed `SchemaConfigSpec` is central orchestration seam with upstream usage from schema load flow
- `scan_filesystem` and `query_cached_state` both directly depend on `SchemaConfigSpec` path accessor split
- Grep analysis revealed 40+ instances of `strip_prefix` + `RelativePath::try_from` conversion chains across schema/builder/discovery modules
- Current `NormalizedPath` already implements strict normalization (UTF-8, forward slashes, no traversal) but is underutilized

### Benchmarks & Prototypes
- `Box<str>` storage provides ~25% memory reduction vs `PathBuf` for large key sets (based on existing `NormalizedPath` usage patterns in vault storage)
- Fallible conversion overhead is negligible compared to I/O costs at repository boundaries

## Consequences

- **Positive**:
  - Single canonical repository boundary type eliminates conversion churn and semantic drift
  - Root-scoped fallible conversions make boundary safety explicit and prevent cross-root key bugs
  - `SchemaConfigSpec` becomes execution-facing with clear filesystem vs persistence path separation
  - Architecture tests enforce monotonic progress and prevent regression during migration
  - Error boundaries become explicit (outside-root cases return typed errors, not panics)
  - Removes panic-based `expect()` calls in `Config::to_schema_spec()` and `unreachable!` branches in `SchemaConfigSpec` accessors

- **Negative**:
  - Short-term migration cost across schema/vault/note/template repository signatures (~40+ call sites based on grep analysis)
  - Deprecated `NormalizedPath` alias creates temporary naming ambiguity until removed
  - Developers must learn new conversion contract (`as_key(root)` instead of ad hoc `strip_prefix` chains)
  - Adds transitional architecture test module surface area (though explicitly retired post-migration)

- **Risks**:
  - Phased migration requires discipline to prevent `RelativePath` regression during transitional period
  - If exit criteria are not enforced per phase, deprecated alias could persist longer than intended
  - Repository signature changes could break external consumers (though repository traits are internal to crate currently)
