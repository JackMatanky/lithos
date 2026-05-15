# Session Log

## Phase 1: Criterion 0.5 → 0.8.2 ✓
- Found criterion at workspace `Cargo.toml:72` with `html_reports` + `async_tokio` features
- lithos-core uses workspace criterion with `html_reports` feature
- 4 benchmark files: db_storage, note_parsing, string_construction, db_key_handling
- All use `criterion::black_box`
- MSRV 1.92, Rust 1.94 nightly
- Created worktree at `.worktrees/chore/bump-criterion-0.8.2/`
- No API breakage for our usage (async-std dropped, we use async_tokio)
- Committed as `850b98d4` on `chore/bump-criterion-0.8.2`

## Phase 2: Automated Benchmark Tracking — Research 🔬
- Read `docs/refs/crates/criterion.md` (~2200 lines) thoroughly
- Surveyed toolchain: cargo-criterion, criterion-table, critcmp, Bencher
- Analyzed RESULTS.md (524 lines) to map automatable (~90 lines) vs manual (~434 lines) content
- Researched internet for criterion-table (nu11ptr), critcmp (BurntSushi), Bencher (bencher.dev), cargo-criterion historical reports, baseline dir conventions
- **Key decision: Recommend against cargo-criterion** — semi-abandoned (last release 2021, issue #64), no baseline support, format compat with 0.8.2 uncertain
- **Key pivot — Bencher replaces critcmp + manual archiving**: Bencher provides historical tracking, web console, CI regression detection, and PR comments in a single tool. Free for open source public projects.
- **`.benchmarks/` probably unnecessary** if using Bencher Cloud (persistence handled server-side)
- **Proposed `.mise/tasks/bench/` task group**: run (local dev), bencher (CI integration wrapper)
- **Remaining questions**: Bencher Cloud vs Self-Hosted? Who creates the API token? Keep `test:bench` or migrate?
