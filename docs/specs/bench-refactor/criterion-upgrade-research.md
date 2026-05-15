# Criterion 0.8.2 Upgrade - Research Findings

## Breaking Changes from 0.5 → 0.8.2

### 0.6.0 (2025-05-17)
- **MSRV bumped to 1.80** → Not blocking (ours is 1.92)
- **`criterion::black_box()` → `std::hint::black_box()`** → Not breaking (still re-exported), but recommended migration
- **clap unpinned** → No impact

### 0.7.0 (2025-07-25)
- Just version alignment with criterion-plot, no API changes

### 0.8.0 (2025-11-29) — BREAKING
- **Drop async-std support** → We use `async_tokio`, no impact
- **MSRV to 1.86** → Not blocking
- `Throughput::ElementsAndBytes` added (new feature, not needed)
- alloca-based memory layout randomization (internal)

### 0.8.1 (2025-12-07)
- Fix homepage link, typo

### 0.8.2 (2026-02-04) — target version
- Fix panic with uniform iteration durations
- Fix alloca on unsupported targets

## Inventory

### Workspace Cargo.toml (line 72)
```toml
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
```
→ Change to: `version = "0.8.2"`

### lithos-core/Cargo.toml (line 59)
```toml
criterion = { workspace = true, features = ["html_reports"] }
```
→ No version change needed (workspace inheritance)

### Benchmark files - all import `black_box` from criterion
All 4 benches import `black_box` from `criterion`. In 0.8.2, `criterion::black_box` is still exported as a re-export of `std::hint::black_box`. The 0.6.0 changelog recommends switching to `std::hint::black_box()`.

### Features check
- `html_reports` ✓ Still available in 0.8.x
- `async_tokio` ✓ Still available in 0.8.x (only async-std was dropped)

## Verdict
Clean upgrade - no API breakage for our usage. Only changes needed:
1. Version bump `0.5` → `0.8.2` in workspace Cargo.toml
2. Optional: migrate `criterion::black_box` → `std::hint::black_box` across 4 bench files

---

# Phase 2 Research: Automated Benchmark Result Tracking

## Goal
Replace most of the manual `lithos-core/benches/RESULTS.md` with automated output from criterion's built-in tools, while tracking all benchmark runs historically (not just the latest).

## Current State of RESULTS.md
The file is **524 lines** of hand-maintained content covering:
- **Performance tables** per benchmark (4 suites: db_storage, note_parsing, string_construction, db_key_handling)
- **Detailed analysis & interpretation** (breakdowns, root causes, scaling behavior)
- **Performance vs targets** (pass/fail against requirements)
- **Production recommendations**
- **Optimization history** (what changed, by how much, per commit)
- **Regression detection guidelines** (critical/significant/noise thresholds)
- **Running benchmarks instructions**
- **Document history** (change log)

## Toolchain Survey

### 1. `cargo-criterion` — Cargo extension for benchmark lifecycle
- Repo: https://github.com/criterion-rs/cargo-criterion
- Install: `cargo install cargo-criterion`
- Use: `cargo criterion` (replaces `cargo bench`)
- **Key feature**: Generates **historical reports** showing function performance over time in `target/criterion/reports/`
- **Key feature**: `--message-format=json` → machine-readable JSON output (one JSON object per line)
- JSON includes: `reason: "benchmark-complete"` with `id`, `report_directory`, iteration data, confidence intervals (typical/mean/median/median_abs_dev/slope), change detection
- JSON also includes: `reason: "group-complete"` with group name, member benchmarks
- Configurable via `Criterion.toml` (output format, colors, plotting backend)
- **Limitation**: Does NOT support baselines (managed separately by criterion native)
- **Limitation**: Last release was v1.1.0 (2020) — may need compatibility check with criterion 0.8.2

### 2. `criterion-table` — Generate markdown tables from cargo-criterion JSON
- Repo: https://github.com/nu11ptr/criterion-table
- Install: `cargo install criterion-table`
- Pipeline: `cargo criterion --message-format=json | criterion-table > BENCHMARKS.md`
- Output: GitHub Flavored Markdown (GFM) comparison tables
- **Requirement**: Benchmark IDs must have 2-3 sections separated by `/` (group/function/value or column/row)
  - Our current benchmarks use simple names like `read/zero_copy` — some may need restructuring
- Optional `tables.toml` config file for inline commentary between tables
- **Limitation**: Last release v0.4.2 (2022), unmaintained? Compatibility with newer cargo-criterion JSON format unverified
- **Limitation**: Only outputs GFM (not raw markdown for non-GitHub)

### 3. `critcmp` — Compare criterion baselines across runs
- Repo: https://github.com/BurntSushi/critcmp
- Install: `cargo install critcmp`
- Usage: `critcmp before change` (compares two saved baselines)
- Can export baselines to standalone JSON for permanent archiving outside `target/`
- Supports regex grouping for cross-benchmark comparison
- Threshold filtering (e.g., `-t 5` to hide <5% changes)
- `--list` view for many-comparison scenarios
- **Primary use**: Interactive CLI diffing of benchmark results

### 4. Criterion Native Features (already available)
- `--save-baseline <name>` — Store current results under a named baseline
- `--baseline <name>` — Compare against named baseline without overwriting
- `--baseline-lenient <name>` — Same but won't fail if baseline missing benchmarks (CI-friendly)
- `--load-baseline <name>` — Swap which run is "previous"
- `raw.csv` in `target/criterion/$BENCHMARK/new/` — Stable machine-readable CSV format with columns: group, function, value, throughput_num, throughput_type, sample_measured_value, unit, iteration_count
- HTML reports in `target/criterion/reports/index.html` (via `html_reports` feature, already enabled)

### 5. Bencher (bencher.dev) — External continuous benchmarking SaaS
- URL: https://bencher.dev
- `bencher run --adapter rust_criterion "cargo bench"` — wraps criterion and sends data to Bencher Cloud
- Web console for tracking results over time by branch, testbed, benchmark, measure
- State-of-the-art analytics for regression detection (CI integration)
- Self-hosted option available (open source)
- **Pros**: Full historical tracking, web UI, CI-native
- **Cons**: External dependency, vendor lock-in for Cloud tier, operational overhead for self-hosted

### 6. GitHub Actions Integration (already configured)
- `.github/workflows/ci.yml` uses `criterion-compare-action@v3`
- Currently compares PR benchmarks against `main` branch baseline
- Could be extended to archive baseline JSON artifacts for long-term storage

## Recommended Architecture

### Tier 1: Automated Table Generation (highest value / least effort)
```
cargo criterion --message-format=json | criterion-table > BENCHMARKS.md
```
- Replaces the manual performance number tables in RESULTS.md
- Requires: installing `cargo-criterion` + `criterion-table`, possibly restructuring benchmark IDs
- Run as part of a new mise task (e.g., `mise run bench-report`)

### Tier 2: Historical Baseline Archiving
```
cargo bench -- --save-baseline $(date +%Y-%m-%d)
critcmp --export $(date +%Y-%m-%d) > .baselines/$(date +%Y-%m-%d).json
```
- Archive each run's baseline as a JSON file in the repo (git-tracked or LFS)
- Enables: `critcmp 2026-02-11 2026-05-14` to compare any two historical runs
- Baseline files stored under `lithos-core/.baselines/`

### Tier 3: CI-Integrated Historical Tracking
Options:
1. **GitHub Actions**: Save baseline JSON as a CI artifact per run, add step to upload to gh-pages report
2. **Bencher**: Full web console with zero self-hosted infra if using Bencher Cloud
3. **cargo-criterion historical reports**: Already generates `target/criterion/reports/` with per-function historical charts

## What Stays Manual
- **Detailed analysis & interpretation** (why performance changed, root causes)
- **Performance vs targets** (business rules, pass/fail assessment)
- **Production recommendations** (domain expertise required)
- **Optimization history narrative** (what was done and why)
- **Regression threshold adjustments** (policy decisions)
- **Benchmark methodology changes** (new suites, parameter changes)

## Estimated Automation Coverage
| RESULTS.md Section | Lines | Automatable? | Tool |
|---|---|---|---|
| Performance tables | ~90 | ✅ Yes | criterion-table from cargo-criterion JSON |
| Detailed analysis | ~110 | ❌ No | Domain expertise |
| Performance vs targets | ~15 | ❌ No | Business rules |
| Production recommendations | ~20 | ❌ No | Domain expertise |
| Optimization history | ~55 | ⚠️ Partial | Baseline archive + manual narrative |
| Regression guidelines | ~25 | ❌ No | Policy decisions |
| Running benchmarks | ~40 | ✅ Generated once | Static doc, not per-run |
| Performance context | ~30 | ❌ No | Domain expertise |

## Benchmark ID Format Analysis (criterion-table Compatibility)

criterion-table requires benchmark IDs in 2-3 `/`-separated sections:
- 2-section: `column_name/row_name`
- 3-section: `group_name/function_name/variant`

### Current Format

| Benchmark Group | Function Names | Format | Sections | Compatible? |
|---|---|---|---|---|
| `read_zero_copy` | `get_zero_copy` | `read_zero_copy/get_zero_copy` | 2 | ✅ Yes |
| `read_deserialize` | `get_owned` | `read_deserialize/get_owned` | 2 | ✅ Yes |
| `write_single` | `put_single` | `write_single/put_single` | 2 | ✅ Yes |
| `write_batch` | `BenchmarkId::from_parameter` | `write_batch/100` | 2 | ✅ Yes |
| `delete` | `delete_single` | `delete/delete_single` | 2 | ✅ Yes |
| `cache_effectiveness` | `hot_read`, `cold_read` | `cache_effectiveness/hot_read` | 2 | ✅ Yes |
| `transaction_overhead` | `individual_txns`, `batch_txn` | `transaction_overhead/individual_txns` | 2 | ✅ Yes |
| `scan_range` | `range_query_100_matches`, `full_scan_filter_100_matches` | `scan_range/range_query_100_matches` | 2 | ✅ Yes |
| `uuid_handling` | `get_preformatted_key`, ... | `uuid_handling/get_preformatted_key` | 2 | ✅ Yes |
| `key_formatting` | `get_with_string_key`, `put_with_string_key` | `key_formatting/get_with_string_key` | 2 | ✅ Yes |
| `numeric_formatting` | `format_integers_itoa`, ... | `numeric_formatting/format_integers_itoa` | 2 | ✅ Yes |
| `constructor_apis` | `schema_name_from_str`, ... | `constructor_apis/schema_name_from_str` | 2 | ✅ Yes |
| `aggregate_workflow` | `complete_optimized_workflow` | `aggregate_workflow/complete_optimized_workflow` | 2 | ✅ Yes |
| `note_parsing` | `ingest_markdown/simple` (has `/`) | `note_parsing/ingest_markdown/simple` | 3 | ⚠️ Slash in function name |
| `note_parsing_parse_only` | `parse_markdown/simple` (has `/`) | `note_parsing_parse_only/parse_markdown/simple` | 3 | ⚠️ Slash in function name |

**Issue**: `note_parsing.rs` uses `/` inside function names (e.g., `ingest_markdown/simple`), resulting in 3-section IDs. criterion-table treats 3-section IDs as `group/function/variant`, which may cause incorrect table layout for these benchmarks. Either:
- Restructure note_parsing benchmarks to use `BenchmarkId` instead of `/` in function name, OR
- In criterion-table, `note_parsing/ingest_markdown/simple` → column="ingest_markdown", row="simple" (actually usable for sub-tables)

**Verdict**: Most benchmarks are 2-section and fully compatible. note_parsing may need minor restructuring or produce sub-tables.

## cargo-criterion v1.1.0 Compatibility Assessment

- **Last release**: 2020 (no new releases since)
- **Does NOT link against criterion** — reads criterion's output files from `target/criterion/`
- **JSON message format** is produced by cargo-criterion itself (not criterion), so it's versioned with cargo-criterion
- **Dependencies**: clap 2.33, serde_cbor 0.11, toml 0.5 — old but will compile with Rust 1.94 (these are stable)
- **Critical concern**: Criterion.rs explicitly states its internal file formats (estimates.json, sample.json) are "private implementation details... structure may change at any time without warning." cargo-criterion reads these files, so v1.1.0 may NOT be compatible with criterion 0.8.2's file format.
- **Alternative**: cargo-criterion's `--message-format=json` invokes benchmark executables and captures their stdout. This pipeline is more format-stable than reading internal files.

### Recommendation
Install cargo-criterion in the implementation phase and run a quick test: `cargo criterion --message-format=json` on our 0.8.2 benchmarks. If it works, proceed with the pipeline. If not, alternatives:
1. Use criterion native baselines + critcmp for comparison (no cargo-criterion needed)
2. Write a small script to parse `raw.csv` (stable format) and generate markdown
3. Skip automated table generation; use critcmp for CLI comparisons + keep manual tables

## Directory Naming Convention for Archived Baselines

**Research question**: Is there a standard naming convention for storing criterion benchmark baseline files outside of `target/`?

**Findings**: No universal convention exists. Surveyed patterns across Rust ecosystem:

| Pattern | Used by | Notes |
|---|---|---|
| `benchmarks/baselines/<machine>/<workload>.json` | sochdb-benchmarks | Non-hidden, project-level benchmarks workspace member |
| `.zenbench/baselines/<name>.json` | zenbench | Hidden, tool-specific (zenbench), within project root |
| `target/criterion/<bench>/<baseline>/` | Criterion (native) | Default location, inside build artifacts (not git-tracked) |
| `.critcmp/baselines/` | N/A (hypothetical) | No known usage — critcmp doesn't prescribe a directory |

**Key observations from ecosystem**:
- **Criterion's native baselines** live inside `target/criterion/<benchmark>/<baseline_name>/` — these are ephemeral (cleared by `cargo clean`) and NOT suitable for git tracking
- **critcmp** works with either Criterion's `target/` baselines OR exported JSON files at any path — it doesn't prescribe a convention
- **sochdb-benchmarks** uses a non-hidden `benchmarks/baselines/` dir at workspace member level, organized by machine + workload
- **zenbench** uses a hidden `.zenbench/baselines/` dir at project root
- No project in the surveyed ecosystem uses `.benchmarks/` as a hidden directory
- The closest analog is `.scratch/` which this project already uses for issue tracking (per AGENTS.md)

**Recommendation**: Use `.benchmarks/` (as user proposed) or `.baselines/`. `.benchmarks/` is:
- Consistent with this project's hidden-dir convention (`.scratch/` for issues)
- Self-documenting (unambiguous purpose vs `target/criterion/`)
- Future-extensible (can hold baseline JSONs, report configs, etc.)
- Not conflicting with any established Rust ecosystem convention (since none exists)

**Proposed structure**:
```
.benchmarks/
├── baselines/           # Archived criterion baseline exports (critcmp JSON)
│   ├── 2026-02-11.json  # Baseline from a specific date
│   ├── 2026-05-14.json
│   └── main.json        # "main" branch reference baseline (updated via CI)
├── reports/             # Generated markdown tables & comparison summaries
│   ├── latest.md
│   └── comparison-main.md
└── README.md            # Documentation of what's stored and how to use
```

## cargo-criterion v1.1.0 Updated Compatibility

**Correction from initial research**: Last release was **2021-07-28** (not 2020). Latest repo push was 2025-05-17.

**Critical limitation confirmed**: cargo-criterion v1.1.0 **does NOT support baselines** — it's incompatible with `--save-baseline` and `--baseline` flags. This means:
- `cargo criterion` ≠ `cargo bench` when using baselines
- We would need to keep `cargo bench` for baseline-aware runs
- `cargo criterion` would be used only for the `--message-format=json` pipeline

**Compatibility concerns**:
- cargo-criterion v1.1.0 uses **edition 2018** — compiles on 1.94 nightly but may have issues
- cargo-criterion reads criterion's internal data files (CBOR in `target/criterion/`) for historical reports — these files are explicitly documented as "private implementation details... structure may change at any time without warning" (see criterion docs)
- Open issue **#64 "Status of project?"** (2024) suggests the project is unmaintained
- Open issue **#68 "JSON format appears to contain truncated keys"** (2025) suggests bugs with newer criterion versions

**Updated recommendation**: Do not adopt cargo-criterion. The project is semi-abandoned, baseline support is missing, and format compatibility with 0.8.2 is uncertain. Criterion already provides all the analysis we need natively — the only gap is **persistence** of historical results beyond the default two-run window. The solution:
1. Use **`cargo bench`** with native baselines (`--save-baseline`, `--baseline`) for per-run comparison — criterion already handles statistical change detection, confidence intervals, and regression/improvement signals
2. Use **`critcmp`** for CLI diffing across any two archived baselines (cross-run, cross-branch)
3. Export baseline JSONs with **`critcmp --export`** for permanent archiving in `.benchmarks/baselines/` — this is the only thing criterion doesn't do natively
4. **No custom parser needed** — criterion's CLI output and HTML reports (`html_reports` feature, already enabled) already display comparison data with change percentages, confidence intervals, and regression detection

## Mise Task Organization for Benchmark Operations

### Current State
- Bench task exists at `.mise/tasks/test/bench` → invoked as `mise run test:bench`
- TOML convenience aliases in `mise.toml`: `test:bench:core`, `test:bench:cli`
- Task supports: package filtering (`-p core|cli`), name filtering, `--quick`, `--noplot` flags

### Proposed Architecture: `.mise/tasks/bench/` Group
Per mise docs, file tasks in subdirectories auto-prefix: `.mise/tasks/bench/foo` → `bench:foo`.

**Option A**: New `bench/` directory (recommended — cleaner naming, separates from `test:`):
```
.mise/tasks/
└── bench/
    ├── run            # bench:run  — run benchmarks (migrate from test:bench)
    ├── archive        # bench:archive — save baseline + export to .benchmarks/
    ├── compare        # bench:compare — critcmp between two baselines
    ├── report         # bench:report — generate markdown tables from raw.csv / cargo-criterion JSON
    ├── history        # bench:history — open cargo-criterion historical report
    └── list           # bench:list — show available baselines
```

**Option B**: Extend inside `test/` (consistent with current location):
```
.mise/tasks/test/
├── bench/             # becomes test:bench: prefix
│   ├── run            # test:bench:run (replaces test:bench)
│   ├── archive        # test:bench:archive
│   ├── compare        # test:bench:compare
│   ├── report         # test:bench:report
│   └── list           # test:bench:list
└── bench              # existing, deprecated → delegates to bench:run
```

**Option A is preferred** because:
1. Cleaner naming: `bench:compare main 2026-05-14` vs `test:bench:compare`
2. Benchmark ops are distinct from testing — different concerns, different tooling
3. Existing `test:bench` task would remain as an alias/delegate for backward compat
4. TOML aliases in `mise.toml` would update: `test:bench:core` → `bench:run -p core`

### Proposed Task Descriptions

| Task | Command | Purpose |
|---|---|---|---|
| `bench:run` | `cargo bench [--bench <filter>] [-- --save-baseline <name>]` | Run benchmarks (migrate from test:bench, add baseline support) |
| `bench:archive` | `cargo bench -- --save-baseline <name> && critcmp --export <name> > .benchmarks/baselines/<name>.json` | Run + archive baseline permanently |
| `bench:compare` | `critcmp <a> <b> [--list] [--threshold <pct>]` | Compare two baselines (local or archived) |
| `bench:list` | List available baselines (local in target/ + archived in .benchmarks/) | Discover baselines for comparison |
| `bench:report` | Open `target/criterion/reports/index.html` | View HTML report with charts |

### TOML Config Updates in `mise.toml`
```toml
[tasks."bench:run:core"]
description = "Run core crate benchmarks"
alias = "brcore"
run = "mise run bench:run -p core"

[tasks."bench:run:cli"]
description = "Run CLI crate benchmarks"
alias = "brcli"
run = "mise run bench:run -p cli"
```

### Migration Path
1. Keep existing `test:bench` as-is during transition
2. Create `bench/` directory with initial tasks
3. Update `test:bench` to delegate: `exec mise run bench:run "$@"` (or via `run` field)
4. Update TOML aliases once stable
5. Phase out `test:bench` only when all consumers updated

## Existing Mise File Task Conventions

After reading all `.mise/tasks/` scripts, the consistent conventions are:

### Header Block
```
#!/usr/bin/env bash
#MISE description="..."
#MISE alias="x"
#MISE sources=["**/*.rs", "Cargo.toml"]
#MISE outputs=["target/some.stamp"]
#USAGE arg "[name]" help="..."
#USAGE flag "-x --flag <val>" help="..." { choices "a" "b" }
```

### Shell Settings
```
set -euo pipefail
```
**Note**: `test/bench` is the only file missing this — likely an oversight.

### Structure
- Small focused functions with `#######################################` comment blocks
- Each block documents: `# Globals:`, `# Arguments:`, `# Outputs:`
- `main()` function at the bottom, called via `main "$@"`
- Nameref arrays: `local -n ref_name=$1`
- Usage vars accessed via `"${usage_var:-}"` (optional), `"${usage_var?}"` (required), `"${usage_var:-false}"` (booleans)
- Boolean patterns: both `== "true"` (test/bench) and `== "1"` (build, fmt, lint) are used

### Echo Style
- `echo "🧪 Running benchmarks..."` — emoji prefix per task
- `echo "✅ Task complete"` for success
- `echo "Error: ..." >&2` for errors (via stderr)

### Key Patterns for New Tasks
- `#USAGE arg "[filter]"` for optional positional args
- `#USAGE flag "-p --package <package>"` with choices block
- Package resolution using `map_package_name()` case statement
- Cargo args built into a local array via nameref helper functions
- Use `--` to separate cargo args from bench args

## Updated Questions for Implementation Phase
1. Does critcmp install/compile on our Rust 1.94 nightly toolchain?
2. Does critcmp correctly read criterion 0.8.2 baseline format?
3. `.benchmarks/` — should git track only `main.json` reference baselines, or all named baselines?
4. Should `test/bench` be migrated to `bench/run` in Phase 3, or kept as `test:bench` with `bench:run` added alongside?

## Final Toolchain Decision

| Component | Status | Purpose |
|---|---|---|
| `cargo bench --save-baseline` | ✅ Core mechanism | Per-run baseline snapshots (statistical analysis built-in) |
| `critcmp` | ✅ Add for archival | `critcmp --export` into `.benchmarks/baselines/` for permanent storage |
| criterion HTML reports | ✅ Already enabled | Local visual reports via `target/criterion/reports/index.html` |
| `cargo-criterion` | ❌ Skip | Semi-abandoned (2021), no baseline support, format risk |
| `Bencher` | ⏸️ Defer | Cloud solution; revisit if git-tracked baselines prove insufficient |
| Custom parsers | ❌ Not needed | Criterion CLI + HTML already show all analysis |
