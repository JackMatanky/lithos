## Findings

### Repository/Branch State

- Worktree branch: `feat/note-storage-refactor` at `b01c282c`
- Main worktree: `main` at `cf762b7c`
- Merge base: `8335dc32`
- Divergence is significant on both sides.

### Feature Branch Delta (main..feature)

- 28 feature-only commits.
- Includes:
  - note storage split into modular files
  - vault storage split into modular files
  - O(1) reverse path index optimization
  - broad test/doc refactors in note/vault storage

### Main Branch Delta (merge-base..main)

- 30 commits.
- Includes:
  - `NormalizedPath` -> `PathKey` migration in active areas
  - `fs/path.rs` structural changes (traits/helpers)
  - downstream test updates (`note_reader.rs`)

### Dry-Run Merge Result

Conflicts observed when merging `feat/note-storage-refactor` into `main`:

1. `lithos-core/src/fs/path.rs` (content)
2. `lithos-core/src/note/storage.rs` (modify/delete)
3. `lithos-core/src/vault/storage.rs` (modify/delete)
4. `lithos-core/tests/note_reader.rs` (content)

### Key Technical Finding

Feature branch modular vault storage currently still references `NormalizedPath` in many locations (`read.rs`, `write.rs`, `testing.rs`).

Implication:

- Conflict resolution alone is not enough; a post-merge compatibility sweep is required to align modular storage with `PathKey` naming used by `main`.

### Preservation-Sensitive Areas

- Main-only behavior tweak in legacy `note/storage.rs` tests (temp-dir-backed vault root instead of hardcoded `/vault`).
- PathKey API consistency in `tests/note_reader.rs` and fs path types.
- Feature-only split architecture files replacing monoliths.

### Recommended Resolution Policy by File

- `lithos-core/src/fs/path.rs`
  - Prefer main as base implementation.
  - Reintroduce only feature changes proven necessary by compile/tests.

- `lithos-core/src/note/storage.rs`
  - Keep deletion (monolith retired).
  - Port any test-stability improvements into split test modules.

- `lithos-core/src/vault/storage.rs`
  - Keep deletion (monolith retired).
  - Ensure split files cover all repository APIs and now use `PathKey`.

- `lithos-core/tests/note_reader.rs`
  - Keep `PathKey` signatures and constructors from main.
  - Preserve feature test scenarios around partial scans/pruning.

### Verification Requirements

- Run full project quality tasks via mise (`fmt`, `lint`, `test`).
- Run focused storage tests for note and vault modules.
- Validate no unintended flow spread via GitNexus detect changes after merge resolution.
