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
#MISE alias="bench"
#MISE sources=["**/*.rs", "Cargo.toml", "benches/**/*"]
#MISE outputs=[".benchmarks/*.json"]
#USAGE arg "[mode]" help="Operation mode" {
#USAGE   choices "run" "compare" "list" "open"
#USAGE   default "run"
#USAGE }
#USAGE arg "[baseline_a]" help="First baseline (compare mode)"
#USAGE arg "[baseline_b]" help="Second baseline (compare mode)"
#USAGE flag "-p --package <package>" help="Run benchmarks for specific package" {
#USAGE   choices "core" "cli"
#USAGE }
#USAGE flag "-f --filter <filter>" help="Filter benchmarks by name (run mode)"
#USAGE flag "-q --quick" help="Quick mode (run mode)"
#USAGE flag "-n --noplot" help="Disable plots (run mode)"
#USAGE flag "--name <name>" help="Override baseline name (run mode)"
#USAGE flag "-t --threshold <pct>" help="Hide comparisons below threshold (compare mode)"
#USAGE flag "-g --group <regex>" help="Group by regex (compare mode)"
```

### Behavioral Contract

**Run mode** (default or explicit `run`): Run benchmarks + archive baseline.

1. Generate baseline name: `<branch>-<date>-<time>-<sha>` (e.g., `main-2026-05-14-143022-a1b2c3d`)
2. Run `cargo bench --bench <target>... [--package <pkg>] -- --save-baseline <name> [filter] [--quick] [--noplot]`
3. Export via `critcmp --export <name> > .benchmarks/<name>.json`
4. Verify exported file is non-empty

**Compare mode** (`compare <a> <b>`): Run critcmp with path resolution.

**List mode** (`list`): Show archived baselines in `.benchmarks/`.

**Open mode** (`open`): Open HTML report in default browser.

### Key Functions

| Function | Purpose |
|----------|---------|
| `map_package_name()` | Shorthand → cargo package name |
| `generate_baseline_name()` | `<branch>-<date>-<time>-<sha>` |
| `discover_bench_targets()` | Find `[[bench]]` target names in Cargo.toml files |
| `build_cargo_args()` | `--bench <target>...` + `--package` (via discovery) |
| `build_bench_args()` | Always `--save-baseline`, optional filter/quick/noplot |
| `ensure_archive_dir()` | `mkdir -p .benchmarks/` |
| `verify_critcmp()` | Error if critcmp not installed |
| `export_baseline()` | `critcmp --export` + non-empty verify |
| `run_benchmarks()` | `cargo bench [cargo_args] -- [bench_args]` |
| `resolve_baseline_path()` | File path → archived `.json` → critcmp name |
| `cmd_compare()` | Run critcmp with resolved paths |
| `cmd_list()` | Show archived baselines |
| `cmd_open()` | Open HTML report (macOS/Linux) |
| `cmd_run()` | Run + archive flow |

### Edge Cases

| Case | Handling |
|------|----------|
| Branch name with `/` | Replaced with hyphens |
| Same commit run twice | Overwrites archive (deterministic name) |
| `--name` collision | Warning then overwrite |
| critcmp not installed | Error with install instructions, exit 1 (compare mode) |
| `.benchmarks/` missing | Auto-created |
| `cargo bench` fails | `set -e` propagates, no export attempted |
| Compare w/ 0/1 baselines | Error message with usage |
| No bench targets found | `cargo bench` runs without `--bench` flags |
| No report file (open mode) | Error message, exit 1 |
| Outside git repo | Falls back to `unknown` branch / `0000000` sha / `.` dir |

---

## Implementation: `.mise/tasks/test/bench`

Current implementation (352 lines) with positional mode argument:

```bash
#!/usr/bin/env bash
#MISE description="Benchmark tasks: run (default), compare, list, open report"
#MISE alias="tb"
#USAGE arg "[mode]" help="Operation mode" {
#USAGE   choices "run" "compare" "list" "open"
#USAGE   default "run"
#USAGE }
```

Usage examples:
- `mise run test:bench` → run mode (default)
- `mise run test:bench compare baseline-a baseline-b` → compare mode
- `mise run test:bench list` → list mode
- `mise run test:bench open` → open report

---

## Package-Specific Shortcuts

Package-specific bench tasks can be defined via TOML:

```toml
[tasks."test:bench:core"]
alias = "tbc"
run = "mise run test:bench run -p core"

[tasks."test:bench:cli"]
alias = "tbcli"
run = "mise run test:bench run -p cli"
```

---

## Files Referenced

| File | Description |
|------|-------------|
| `.mise/tasks/test/bench` | Main bench task script (352 lines) |
| `.benchmarks/*.json` | Archived baseline exports |
| `target/criterion/` | Criterion output directory |
| `bench-tasks-design.md` | This design document |
