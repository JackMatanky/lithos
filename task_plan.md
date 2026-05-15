# Criterion Upgrade + Automated Benchmark Tracking Plan

## Goal
1. Bump criterion from 0.5 to 0.8.2 across the workspace (✅ DONE)
2. Research and design automated benchmark result tracking to replace manual RESULTS.md (🔬 IN PROGRESS)

## Context
- Criterion used in workspace `Cargo.toml` with `async_tokio` + `html_reports` features
- 4 benchmark files: `db_storage.rs`, `note_parsing.rs`, `string_construction.rs`, `db_key_handling.rs`
- Manual RESULTS.md (524 lines) maintained per optimization round
- Need persistent historical tracking across all runs, not just latest

## Research (findings.md)
- `findings.md` contains full detail on criterion 0.5→0.8.2 breaking changes + automated tracking toolchain survey

## Phases

### Phase 1: Criterion Version Bump ✓
- [x] Fetch criterion CHANGELOG.md
- [x] Inventory all criterion imports & usages
- [x] Identify breaking changes 0.5 → 0.8.2
- [x] Check MSRV compatibility
- [x] Check `async_tokio` feature still exists in 0.8.x
- [x] Check `html_reports` feature still exists in 0.8.x
- [x] Update version in workspace Cargo.toml: `0.5` → `0.8.2`
- [x] Update `criterion::black_box` → `std::hint::black_box` in 4 benchmark files
- [x] Verify compilation, tests, fmt, lint all pass
- [x] Commit changes

### Phase 2: Automated Result Tracking — Research & Analysis (IN PROGRESS)
- [x] Read criterion.md docs thoroughly
- [x] Survey available tools (cargo-criterion, criterion-table, critcmp, Bencher)
- [x] Analyze RESULTS.md to identify automatable vs manual sections
- [x] Research internet for additional resources (criterion-table, critcmp, Bencher)
- [x] Research benchmark baseline directory naming conventions across Rust ecosystem
- [x] Assess cargo-criterion v1.1.0 compatibility (semi-abandoned; recommend against)
- [x] Propose recommended architecture + toolchain (critcmp for archival, criterion native for analysis)
- [x] Propose `.benchmarks/` directory structure
- [x] Propose `.mise/tasks/bench/` task group design
- [x] Finalize toolchain decision: critcmp for baseline archival + comparison
- [x] Present findings to user for sign-off

### Phase 3: Automated Result Tracking — Implementation (FUTURE)
- [ ] Install critcmp: `cargo install critcmp`
- [ ] Create `.benchmarks/` directory structure (baselines/, reports/)
- [ ] Add `.benchmarks/` to `.gitignore` or decide git-tracked items
- [ ] Create `.mise/tasks/bench/run` — migrate from `test:bench`, add `--save-baseline` support
- [ ] Create `.mise/tasks/bench/archive` — run + export baseline to `.benchmarks/baselines/`
- [ ] Create `.mise/tasks/bench/compare` — critcmp wrapper
- [ ] Create `.mise/tasks/bench/list` — list available baselines
- [ ] Create `.mise/tasks/bench/report` — open HTML report
- [ ] Update TOML convenience tasks in `mise.toml`
- [ ] Run a full archive cycle: benchmark → save baseline → critcmp --export → verify
- [ ] Verify with `mise run verify`
