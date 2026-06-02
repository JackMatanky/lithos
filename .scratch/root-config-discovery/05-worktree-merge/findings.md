# Findings: Worktree 05 Merge Analysis

## Divergence Analysis
- **Common Ancestor:** `6b6bf0a2a2e23ffceaef1b7f56d86698f46fb29d`
- **Branch:** `05-move-discovery-module-boundary`
- **Base:** `main`

## Changes in `05-move-discovery-module-boundary`
- `15551228`: Rust best practices fixes (doc comments, better tests, clippy suppressions).
- Enforced `Discovery -> Config -> Indexer` boundary.
- Full boundary documentation in `discovery/mod.rs`.
- Moved Config-owned taxonomy from `discovery` to `config/`.
- Reinstated `config/discovery.rs` as Config-owned orchestration.

## Changes in `main` (since divergence)
- `a8b8ef70`: Added `lithos-core/src/schema/base_processor.rs` and updated `schema/mod.rs`.
- `schema/discovery.rs` exists and follows the old "Discovery" pattern (Scan + DB).
- `config/discovery.rs` exists and follows the old pattern.

## Conflict Analysis
### Physical Conflicts
- **None**: File sets are disjoint.

### Logical/Semantic Conflicts (GitNexus)
- **`discovery/mod.rs`**: `main` has a minimal version, worktree has full boundary docs.
- **`config/location.rs`**: Divergent `allow`/`expect` attributes and reason strings.
- **`schema/discovery.rs` and `config/discovery.rs`**: Both in `main` still use the old "Discovery" pattern. The worktree refactor's goal was to move this "find paths" logic to the top-level `discovery` context.
- **`config/mod.rs`**: Divergent documentation regarding local `discovery` vs `crate::discovery`.

## Rust Best Practices & Conventions
- The worktree uses `#[allow(dead_code)]` and `#[expect(dead_code)]` with reason strings, following `rust-best-practices`.
- `main` has started adopting this but is less consistent.
