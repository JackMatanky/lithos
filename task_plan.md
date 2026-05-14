# Criterion 0.8.2 Upgrade Plan

## Goal
Bump criterion from 0.5 to 0.8.2 across the workspace, updating any deprecated/removed APIs.

## Context
- Criteria used in workspace `Cargo.toml` (line 72) with `async_tokio` + `html_reports` features
- lithos-core uses workspace criterion with `html_reports` feature
- 4 benchmark files: `db_storage.rs`, `note_parsing.rs`, `string_construction.rs`, `db_key_handling.rs`
- All use `criterion::black_box` (should migrate to `std::hint::black_box` per 0.6.0 recommendation)
- MSRV 1.92, Rust 1.94 nightly (well above 0.8's MSRV 1.86)

## Research (findings.md)

## Phases

### Phase 1: Research ✓
- [x] Fetch criterion CHANGELOG.md
- [x] Inventory all criterion imports & usages
- [x] Identify breaking changes 0.5 → 0.8.2
- [x] Check MSRV compatibility
- [x] Check `async_tokio` feature still exists in 0.8.x
- [x] Check `html_reports` feature still exists in 0.8.x

### Phase 2: Implement
- [ ] Update version in workspace Cargo.toml: `0.5` → `0.8.2`
- [ ] Update `criterion::black_box` → `std::hint::black_box` in 4 benchmark files
- [ ] Run `cargo build` to verify compilation

### Phase 3: Verify
- [ ] Run `cargo test` to verify tests pass
- [ ] Run `cargo bench -- --quick` to verify benchmarks run
- [ ] Run `mise run fmt` to format
- [ ] Run `mise run lint` for clippy

### Phase 4: Finalize
- [ ] Commit changes in worktree
- [ ] Report summary
