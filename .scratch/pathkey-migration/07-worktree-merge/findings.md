# Findings: Worktree Merge Analysis — `07/pathkey-note-template` → `main`

## Divergence

- **Merge base**: `6e951e49` — "docs: update centralized discovery prd"
- **main** (3 commits past base): `66b90411` (root-config-discovery issues), `b69b91f5` (mermaid skill), `79bd1325` (schema split consistency pass)
- **Worktree** (1 commit past base): `1c96ca7f` (issue 07 agent brief update)

## File Sets

### Files on `main` only (no overlap with worktree)
- `.agents/skills/mermaid-diagrams/` — 5 new files, +1,819 lines (new skill)
- `.scratch/base-schema/` — PRD + 8 new issues, +377 lines
- `.scratch/schema-processor-split/PRD.md` — updated, +14/-41 lines
- `AGENTS.md` — +2 lines (mermaid skill reference)
- `skills-lock.json` — +6 lines

### Files on `07/pathkey-note-template` only (no overlap with main)
- `lithos-core/src/note/paths.rs` — -49 net lines
- `lithos-core/src/note/storage/read.rs` — -5 net lines
- `lithos-core/src/note/storage/write.rs` — -27 net lines
- `lithos-core/src/vault/processor.rs` — -2 net lines
- `.scratch/pathkey-migration/07-*.md` — +114/-38 lines

## Overlap Analysis
**Zero overlapping files.** All 4 source files modified in the worktree have zero edits on `main` since `6e951e49`. All 21 files modified on `main` are documentation/skills/config — none are source code or note-context files.

## Rust Best Practices Assessment

### What was done well:
- **Borrowing over cloning (§1.1)**: `as_path_key()` returns `&PathKey` — borrows the inner key without copying
- **Performance (§3.2)**: Eliminated 6 redundant `PathKey::try_new` allocations + validations. The old code re-parsed already-validated paths through the validation pipeline
- **Error handling (§4)**: `path_error_to_invalid_path` explicitly matches all known `PathError` variants. Wildcard `_` provides future-proofing. No `unwrap()`/`expect()` in production code
- **Type safety (§1)**: `TryFrom<PathKey>` for `NotePath` preserves the type chain. `From<PathKey>` for `FolderPath` is infallible (correct — folders impose no extension constraint)
- **API Preservation**: Public API unchanged. All existing conversion impls (`TryFrom<&str>`, `TryFrom<String>`) preserved
- **Testing (§5)**: All 1433 unit + 152 doc + 36 integration + 1 e2e tests pass unchanged
- **Formatting/Linting**: Clippy + rustfmt clean with zero warnings

### Deviation noted:
- **AC 3**: Used `TryFrom<PathKey>` for vault processor instead of typed FS chain (`FilePath`/`DirPath` + `from_rooted_path`). Rationale documented in issue file and TODO: path is already a validated DB key; typed FS chain would add ~4 redundant allocations. This is a pre-centrilization bridge — to be removed when centralized discovery processor handles all path construction.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Merge conflict in source files | None | N/A | No overlapping files |
| rkyv incompatibility | None | N/A | Same `Box<str>` binary format |
| Test regression | Low | Medium | Full test suite validation post-merge |
| CI/CD pipeline issues | Low | Low | No dependency or config changes |
| Index staleness | Low | Low | Run `npx gitnexus analyze` after merge |
