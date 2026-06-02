# Merge Analysis: feature/02-base-repository-contracts-and-storage → main

## 1. Divergence Point

| Detail | Value |
|--------|-------|
| Merge base | `3d255769` — docs(scratch): complete merge cleanup and record worktree removal |
| Feature branch | `feature/02-base-repository-contracts-and-storage` |
| Target branch | `main` |
| Feature commits | 7 (f9345a26..2a1f38eb) |
| Main commits since divergence | 11 (dd17a3e9..4184cb63) |

## 2. Changes by Branch

### Main (config discovery)
| Commit | Description |
|--------|-------------|
| dd17a3e9 | docs: refine issue 04 triage decisions |
| e0b1e216 | docs: record issue 04 implementation checkpoint |
| ca65773a | feat(config): add vault root resolver |
| 94be53ce | refactor(config): use iterator root discovery |
| 956f1791 | test: refactor discovery suite naming and structure |
| cfbd6d67 | refactor(config): reorganize discovery resolver for ordering discipline |
| bde459f1 | docs(config): add rustdoc for phase-1 vault root resolver |
| 1d9ed132 | docs: complete phase-1 vault root resolution issue |
| 8289a324 | docs: planning artifacts for worktree 04 merge analysis |
| f85d346d | feat(discovery): merge phase 1 vault root resolution implementation |
| 4184cb63 | chore: organize config discovery files |

**Files:** `lithos-core/src/config/discovery/` only (6 files)

### Feature (06-base-repository-contracts-and-storage)
| Commit | Description |
|--------|-------------|
| f9345a26 | docs(scratch): update issue 02 with review findings and TDD plan |
| 6268921a | feat(schema): add BaseSchema repository contracts and storage |
| d0776577 | refactor(schema): iterator pre-serialization, fix BASE_SCHEMA_BY_ID doc |
| d643cd2e | docs(scratch): add implementation log and review notes to issue 02 |
| 1bee3c13 | docs(scratch): close issue 02 |
| 48358a0a | docs(scratch): add date_completed to issue 02 frontmatter |
| 2a1f38eb | docs(schema): remove phase-1 from doc comments |

**Files:** `lithos-core/src/schema/` only (8 files)

## 3. File Overlap Analysis

| Path | Main | Feature | Conflict? |
|------|------|---------|-----------|
| `lithos-core/src/config/discovery/candidates.rs` | ✅ 386 insertions | — | None |
| `lithos-core/src/config/discovery/contracts.rs` | ✅ 204 insertions | — | None |
| `lithos-core/src/config/discovery/diagnostics.rs` | ✅ 153 insertions (new) | — | None |
| `lithos-core/src/config/discovery/location.rs` | ✅ 140 insertions | — | None |
| `lithos-core/src/config/discovery/mod.rs` | ✅ 2 insertions | — | None |
| `lithos-core/src/config/discovery/resolver.rs` | ✅ 1100 insertions (new) | — | None |
| `lithos-core/src/schema/base.rs` | — | ✅ 4 insertions | None |
| `lithos-core/src/schema/discovery.rs` | — | ✅ 19 insertions | None |
| `lithos-core/src/schema/mod.rs` | — | ✅ 2 insertions | None |
| `lithos-core/src/schema/repository.rs` | — | ✅ 99 insertions (new) | None |
| `lithos-core/src/schema/storage/read.rs` | — | ✅ 170 insertions | None |
| `lithos-core/src/schema/storage/tables.rs` | — | ✅ 10 insertions | None |
| `lithos-core/src/schema/storage/testing.rs` | — | ✅ 322 insertions (new) | None |
| `lithos-core/src/schema/storage/write.rs` | — | ✅ 178 insertions | None |

**Result: Zero overlapping files. Zero expected merge conflicts.**

## 4. Risk Assessment

| Factor | Rating | Rationale |
|--------|--------|-----------|
| File overlap | None | Config discovery vs schema — entirely disjoint modules |
| API surface | Low | New types are downstream of existing consumers; no upstream breakage |
| Test breakage | Low | All tests pass on both branches independently |
| Build breakage | Low | No dependency or feature-gate changes on either side |
| Migration needed | None | No schema changes on main, no config changes on feature |

## 5. Recommended Merge Strategy

**Strategy: Standard `git merge` (fast-forward or 3-way)**

```
main:   ... A --- B --- C --- D --- E --- F
                \
feature:          G --- H --- I --- J --- K
```

Since:
- No overlapping files exist
- Main and feature touch different crate modules
- No interface contracts bridge config ↔ schema modules
- Main has not modified any schema/ files since divergence

**Merge sequence:**
1. While on `feature/02-base-repository-contracts-and-storage` worktree, merge main into feature (synchronization, optional)
2. Push feature branch to origin
3. Checkout main → merge feature/02-base-repository-contracts-and-storage → push

**Or simpler (no sync needed):**
1. Checkout main in main worktree
2. `git merge feature/02-base-repository-contracts-and-storage`
3. Run quality gates
4. Push

## 6. Validation Checklist

After merge:
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --all-targets -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — formatting clean
- [ ] Doc tests pass (`cargo test --doc`)

## 7. Rollback Procedure

```bash
# If merge is on main and NOT pushed:
git reset --hard ORIG_HEAD

# If merge is on main and PUSHED:
git revert -m 1 HEAD
git push origin main
```
