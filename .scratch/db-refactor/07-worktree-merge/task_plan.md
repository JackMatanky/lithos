## Goal

Merge `feat/note-storage-refactor` into `main` without losing any work done on either side after worktree split.

## Scope

- Include all feature-branch commits related to note/vault storage refactor.
- Preserve `main` branch PathKey migration and related fs/test updates.
- Resolve merge conflicts with explicit, audited decisions.

## Inputs Collected

- Feature branch head: `b01c282c`
- Main branch head: `cf762b7c`
- Merge base: `8335dc32`
- Feature-only commits: 28
- Main-only commits since merge base: 30
- Dry-run merge conflicts: 4 files

## Conflict Inventory

1. `lithos-core/src/fs/path.rs` (content conflict)
2. `lithos-core/src/note/storage.rs` (modify/delete conflict)
3. `lithos-core/src/vault/storage.rs` (modify/delete conflict)
4. `lithos-core/tests/note_reader.rs` (content conflict)

## High-Level Merge Strategy

1. Update feature branch with latest main first (`merge main -> feature`) to resolve conflicts in isolation.
2. Keep the modular storage architecture from feature branch:
   - Keep `lithos-core/src/note/storage/{mod,read,write,tables,testing}.rs`
   - Keep `lithos-core/src/vault/storage/{mod,read,write,tables,testing}.rs`
   - Delete legacy monoliths only after salvaging any main-only fixes.
3. Port PathKey naming from main across all new vault/note modules where needed.
4. Re-run full verification gates before final merge back to main.

## Phase Plan

### Phase 1: Pre-merge safety snapshot

- [ ] Tag current tips (`main` and `feat/note-storage-refactor`) with temporary safety tags.
- [ ] Export file lists changed on each side since merge base.
- [ ] Save conflict baseline (`git merge --no-commit --no-ff feat/note-storage-refactor` from main, then abort).

### Phase 2: Integrate main into feature branch

- [ ] Checkout `feat/note-storage-refactor` worktree.
- [ ] Merge `main` into feature branch.
- [ ] Resolve 4 known conflicts using decisions below.

### Phase 3: Conflict resolution decisions

- [ ] `fs/path.rs`: start from `main` version, then re-apply feature-required behavior only.
- [ ] `note/storage.rs`: accept delete, but transfer main test fixture robustness (temp dir config) into new split test modules if still relevant.
- [ ] `vault/storage.rs`: accept delete, then ensure new split modules use `PathKey` (not `NormalizedPath`).
- [ ] `tests/note_reader.rs`: keep latest PathKey API usage from `main`, retain feature branch scenario coverage.

### Phase 4: Rust/API compatibility sweep

- [ ] Replace remaining `NormalizedPath` references in migrated vault storage with `PathKey`.
- [ ] Confirm trait signatures align with current `fs/path.rs` and repository interfaces.
- [ ] Validate error boundaries still follow repository/domain split.

### Phase 5: Verification gates

- [ ] `mise run fmt`
- [ ] `mise run lint`
- [ ] `mise run test`
- [ ] Optional targeted: `cargo test --package lithos-core --lib note::storage`
- [ ] Optional targeted: `cargo test --package lithos-core --lib vault::storage`

### Phase 6: Final merge to main

- [ ] Fast-forward or regular merge `feat/note-storage-refactor` into `main` (based on branch policy).
- [ ] Run `gitnexus_detect_changes(scope:"all")` and confirm expected execution flows only.
- [ ] Remove temporary safety tags once validated.

## Risk Assessment (GitNexus-informed)

- Risk level: HIGH (broad refactor in core storage paths + API type migration overlap).
- Directly sensitive areas:
  - Note repository and processor persistence flow
  - Vault repository read/write and path indexing
  - FS path core types (`PathKey` migration)
- Primary failure modes:
  - Silent type mismatch (`NormalizedPath` remnants)
  - Lost main-only changes during monolith-to-modular conflict resolution
  - Behavioral drift in note reader partial scan tests

## Non-Negotiable Preservation Rules

1. Do not drop any feature split-module files during conflict resolution.
2. Do not revert main PathKey migration semantics in shared code.
3. Do not resolve modify/delete conflicts by blindly taking one side; salvage required behavior first.
4. Do not finalize merge until fmt/lint/test are green.

## Exit Criteria

- Merge commit exists with all 4 conflicts resolved intentionally.
- No `NormalizedPath` usage remains in migrated vault/note storage paths that should use `PathKey`.
- Full quality gates pass.
- GitNexus change impact is consistent with expected storage and processor flows.
