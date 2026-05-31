# Findings - Worktree Merge Analysis

## Environment Verification

- Active path: `/Users/jack/Documents/41_personal/lithos/.worktrees/feat/eventid-core-type-redb-contract`
- Git top-level: `/Users/jack/Documents/41_personal/lithos/.worktrees/feat/eventid-core-type-redb-contract`
- Active branch: `feat/eventid-core-type-redb-contract`

## Divergence

- Merge-base (`main` vs feature): `a11bd95490e989cc6e5146effd2479290ab2200d`

## Commits since divergence

### Feature-side commits

1. `d87a43330` docs(scratch): finalize eventid issue contract
2. `8758ebf62` feat(db): add EventId redb contract
3. `dc6e69b17` docs(scratch): improve eventid invariant contract
4. `1fe6030b4` refactor(db): reorder event id module

### Main-side commits (selected from divergence onward)

- `619227b7e` docs(scratch): update refactor plan for configbuilder metadata threading
- `c5db922c1` docs(scratch): add approved design decisions and agent brief for issue 06
- `b7f7847f1` feat(support): define HasHashIndex and HasHashIndexMut traits
- `6bf78550f` feat(core): implement HasHashIndex traits on wrapper types
- `ed338c5d5` refactor(config): decouple traversal from IO and thread discovery metadata
- `...` additional docs/planning commits
- `1d6ddc745` feat(fs): add structured file format selector
- `73bd5947e` test(fs): normalize format suite and rename rank

## File-level changes since divergence

### Feature branch

- `M .scratch/event-sourcing-foundation/01-eventid-core-type-and-redb-contract.md`
- `A lithos-core/src/db/events.rs`
- `M lithos-core/src/db/mod.rs`

### Main branch

- Multiple `.scratch/*` planning/issue docs for issues 04/06/17
- `M lithos-core/src/config/builder.rs`
- `M lithos-core/src/config/discovery.rs`
- `M lithos-core/src/config/processor.rs`
- `M lithos-core/src/fs/format.rs`
- `M lithos-core/src/fs/mod.rs`
- `M lithos-core/src/schema/discovery.rs`
- `M lithos-core/src/schema/views/hashes.rs`
- `M lithos-core/src/support/hash_index.rs`
- `M lithos-core/src/support/mod.rs`

## Overlap and conflict analysis

- Exact file overlap (both sides changed same path since divergence): **none**
- Expected textual merge conflicts: **low probability**
- Potential semantic conflict zones: **low**, because affected subsystems differ:
  - Feature: `db/events` event-id foundation slice
  - Main: fs/config/schema/support evolutions

## Rust/GitNexus review notes

- Rust best-practices alignment in feature slice:
  - Derive ordering and file ordering improved.
  - `thiserror` typed error model retained.
  - clippy-compatibility preserved with reasoned lint annotations where needed.
- GitNexus impact limitation:
  - `gitnexus_impact` on `EventId`/`EventIdAllocator` returned unresolved target (`Target '' not found`), so blast-radius data is incomplete.
  - Fallback used: git-level divergence and path-based impact reasoning.

## Required migrations/manual interventions

1. **Visibility/re-export invariants**
   - Keep `EventId`, `EventIdAllocator`, `EventIdError` at `pub(crate)` for this slice.
   - Keep `lithos-core/src/db/mod.rs` re-export as `pub(crate) use ...` to avoid `E0365` and clippy `unused_imports` failures.

2. **Lint policy consistency**
   - Maintain reason strings on lint allowances due to `clippy::allow-attributes-without-reason` deny policy.

3. **Working tree hygiene before merge**
   - There is currently an unstaged `AGENTS.md` modification in this worktree. Exclude from merge commit scope unless explicitly intended.

## Validation requirements

- Minimum: `mise run fmt`, `mise run lint`, `mise run test:unit`
- Recommended: `mise run verify` for full quality gate after merge execution.
