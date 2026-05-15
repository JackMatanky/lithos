# Bench Task Scripts — Final Design (One Script)

## Conventions

- `#!/usr/bin/env bash`
- `set -euo pipefail`
- `cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"` for dir safety
- `#MISE description=...` then `#USAGE` lines
- Comment blocks with `####` documenting Globals/Arguments/Outputs
- `main() { ... }; main "$@"`
- Nameref arrays (`local -n ref=$1`)
- `"${usage_var:-}"` defaults, `"${usage_var?}"` required

---

## File: `.mise/tasks/bench`

### Header / USAGE

```
#MISE description="Benchmark tasks: run (default), compare, list, open report"
#MISE alias="b"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE arg "[baseline_a]" help="First baseline (compare mode only)"
#USAGE arg "[baseline_b]" help="Second baseline (compare mode only)"
#USAGE flag "-c --compare" help="Compare two baselines"
#USAGE flag "-l --list" help="List available baselines"
#USAGE flag "-o --open" help="Open HTML report"
#USAGE flag "-p --package <package>" help="Run benchmarks for a specific package" { choices "core" "cli" }
#USAGE flag "-f --filter <filter>" help="Filter benchmarks by name (run mode)"
#USAGE flag "-q --quick" help="Run with quick mode (lower statistical guarantees)"
#USAGE flag "-n --noplot" help="Disable plot generation"
#USAGE flag "--name <name>" help="Override auto-generated baseline name"
#USAGE flag "-t --threshold <pct>" help="Hide comparisons below this percentage"
#USAGE flag "-g --group <regex>" help="Group benchmarks by regex capturing group"
```

### Behavioral Contract

**Default mode** (no `-c`/`-l`/`-o`): Run benchmarks + archive baseline.

1. Generate baseline name: `<branch>-<date>-<time>-<sha>` (e.g., `main-2026-05-14-143022-a1b2c3d`)
2. Run `cargo bench --benches [--package <pkg>] -- --save-baseline <name> [filter] [--quick] [--noplot]`
3. Export via `critcmp --export <name> > .benchmarks/baselines/<name>.json`
4. Verify exported file is non-empty

**Compare mode** (`-c <a> <b>`): Run critcmp with path resolution.

**List mode** (`-l`): Show local (target/criterion/) + archived (.benchmarks/baselines/) baselines.

**Open mode** (`-o`): Open HTML report in default browser.

### Key Functions

| Function | Purpose |
|----------|---------|
| `map_package_name()` | Shorthand → cargo package name |
| `generate_baseline_name()` | `<branch>-<date>-<time>-<sha>` |
| `build_cargo_args()` | `--benches` + `--package`/`--workspace` |
| `build_bench_args()` | Always `--save-baseline`, optional filter/quick/noplot |
| `ensure_archive_dir()` | `mkdir -p .benchmarks/baselines/` |
| `verify_critcmp()` | Error if critcmp not installed |
| `export_baseline()` | `critcmp --export` + non-empty verify |
| `run_benchmarks()` | `cargo bench [cargo_args] -- [bench_args]` |
| `resolve_baseline_path()` | File path → archived `.json` → critcmp name |
| `cmd_compare()` | Run critcmp with resolved paths |
| `cmd_list()` | Show local + archived baselines |
| `cmd_open()` | Open HTML report (macOS/Linux) |
| `cmd_run()` | Run + archive flow |
| `validate_args()` | Mutually exclusive mode flags + compare requires 2 baselines |

### Edge Cases

| Case | Handling |
|------|----------|
| Branch name with `/` | Replaced with hyphens |
| Same commit run twice | Overwrites archive (deterministic name) |
| `--name` collision | Warning then overwrite |
| critcmp not installed | Error with install instructions, exit 1 before benchmarks run |
| `.benchmarks/` missing | Auto-created |
| `cargo bench` fails | `set -e` propagates, no export attempted |
| Compare w/ 0/1 baselines | Validation error |
| Conflicting modes (-c + -l) | Validation error |
| `usage_baseline_a` in run mode | Treated as benchmark filter (backward compat) |
| No report file | Friendly message, exit 0 |
| Outside git repo | Falls back to `unknown` branch / `0000000` sha / `.` dir |

---

## Migration: `test:bench`

Replaced with 4-line delegation:

```bash
#!/usr/bin/env bash
#MISE description="Run performance benchmarks (delegates to bench)"
#MISE alias="tb"
exec mise run bench "$@"
```

Old `test:bench my_filter` → `bench my_filter` → `usage_baseline_a=my_filter` → treated as filter in run mode.

---

## TOML Updates

Old `test:bench:core` / `test:bench:cli` replaced with `bench:core` / `bench:cli`:

```toml
[tasks."bench:core"]
alias = "bc"
run = "mise run bench -p core"

[tasks."bench:cli"]
alias = "bcli"
run = "mise run bench -p cli"
```

---

## Files Created / Modified

| File | Action |
|------|--------|
| `.mise/tasks/bench` | Created (320 lines) |
| `.mise/tasks/test/bench` | Replaced with delegation |
| `mise.toml` | Updated alias blocks |
| `.benchmarks/baselines/.gitkeep` | Created |
| `bench-tasks-design.md` | This document |
