# Design: Benchmark Task Refactor

**Date:** 2026-05-15
**Status:** Approved
**Goal:** Refactor `.mise/tasks/test/bench` for better mise integration and code organization

## Summary

Refactor the benchmark task script from a working-but-improvable 347-line monolith into a well-organized single file that leverages mise features (vars, sources/outputs, choices enum) and follows clear separation of concerns.

## Background

### Current State

The existing `.mise/tasks/test/bench` script (347 lines) is functional but has improvement opportunities:

1. **Hardcoded configuration** - Paths and mappings embedded in bash code
2. **Mixed concerns** - Validation, execution, and archival blended together
3. **Limited mise integration** - Not using vars, sources/outputs tracking
4. **Boolean flag soup** - `--compare`, `--list`, `--open` flags instead of mode argument

### User Goals

1. **Better mise integration** - Leverage vars, env, sources/outputs, task features
2. **Better code organization** - Clear separation of concerns, better function structure

## Design Decisions

### File Structure

**Decision:** Keep as single file with improved organization

**Rationale:**
- Compare mode is only ~50 lines, not worth separate file
- Simpler to maintain and understand
- Still achieves organization goals through clear sections

**Alternatives considered:**
- 5-file split (run/archive/compare/list/open) - rejected as over-engineering
- 2-file split (main + compare) - rejected as unnecessary complexity

### Mode Selection

**Decision:** Use choices enum instead of boolean flags

**Before:**
```bash
mise run test:bench --compare a b
mise run test:bench --list
mise run test:bench --open
```

**After:**
```bash
mise run test:bench compare a b
mise run test:bench list
mise run test:bench open
mise run test:bench         # run mode (default)
```

**Rationale:**
- Cleaner UX (positional argument vs flag)
- Mutually exclusive by design
- Better tab completion
- Follows mise best practices

### Configuration Externalization

**Decision:** Move hardcoded config to `mise.toml` vars

**Before (hardcoded):**
```bash
ARCHIVE_DIR=".benchmarks/baselines"
case "${usage_package:-}" in
  core) echo "lithos-core" ;;
  cli)  echo "lithos-cli" ;;
esac
```

**After (mise vars):**
```toml
[vars]
bench_archive_dir = ".benchmarks/baselines"
bench_package_core = "lithos-core"
bench_package_cli = "lithos-cli"
```

```bash
ARCHIVE_DIR="${MISE_VARS_BENCH_ARCHIVE_DIR:-.benchmarks/baselines}"
```

**Rationale:**
- Configuration visible in mise.toml
- Easy to override per-environment
- Follows mise conventions

### Code Organization

**Decision:** Organize by domain/responsibility, not execution order

**Structure (6 sections):**

1. **Configuration** - Source mise vars, setup constants
2. **Domain Utilities** - Pure functions (no side effects)
3. **Validators** - Precondition checks
4. **Executors** - Functions with side effects
5. **Mode Handlers** - Top-level command implementations
6. **Main Entry Point** - Dispatcher

**Rationale:**
- Clear mental model (find functions by purpose, not when they run)
- Separates pure from impure functions
- Easy to test and maintain
- Follows Google Shell Style Guide

## Detailed Design

### Mise Configuration Changes

Add to `mise.toml`:

```toml
[vars]
bench_archive_dir = ".benchmarks/baselines"
bench_package_core = "lithos-core"
bench_package_cli = "lithos-cli"
```

### Task Metadata

```bash
#!/usr/bin/env bash
#MISE description="Benchmark tasks: run (default), compare, list, open report"
#MISE alias="tb"
#MISE sources=["**/*.rs", "Cargo.toml", "benches/**/*"]
#MISE outputs=[".benchmarks/baselines/*.json"]
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

**Key additions:**
- `sources` - Tracks Rust source files, Cargo.toml, bench files
- `outputs` - Tracks archived baselines
- `mode` argument with choices enum
- Preserved all existing flags

### Script Structure

```
┌─────────────────────────────────────┐
│ SECTION 1: CONFIGURATION            │
│ - Source mise vars                  │
│ - Set constants                     │
│ - Change to repo root               │
│ (~20 lines)                         │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│ SECTION 2: DOMAIN UTILITIES         │
│ - map_package_name()                │
│ - generate_baseline_name()          │
│ - discover_bench_targets()          │
│ - resolve_baseline_path()           │
│ (~80 lines, pure functions)         │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│ SECTION 3: VALIDATORS               │
│ - verify_critcmp()                  │
│ - ensure_archive_dir()              │
│ (~20 lines)                         │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│ SECTION 4: EXECUTORS                │
│ - build_cargo_args()                │
│ - build_bench_args()                │
│ - run_benchmarks()                  │
│ - export_baseline()                 │
│ (~80 lines, side effects)           │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│ SECTION 5: MODE HANDLERS            │
│ - cmd_run()                         │
│ - cmd_compare()                     │
│ - cmd_list()                        │
│ - cmd_open()                        │
│ (~80 lines)                         │
└─────────────────────────────────────┘
                 ↓
┌─────────────────────────────────────┐
│ SECTION 6: MAIN ENTRY POINT         │
│ - Mode dispatcher                   │
│ - main() call                       │
│ (~20 lines)                         │
└─────────────────────────────────────┘
```

**Total:** ~300 lines (similar to current, but better organized)

### Section Details

#### Section 1: Configuration

```bash
set -euo pipefail

ARCHIVE_DIR="${MISE_VARS_BENCH_ARCHIVE_DIR:-.benchmarks/baselines}"
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"
```

**Purpose:** Initialize environment, source mise vars, setup constants

#### Section 2: Domain Utilities (Pure Functions)

```bash
#######################################
# Map shorthand package names to full cargo package names.
# Globals: MISE_VARS_BENCH_PACKAGE_*
# Arguments: Package shorthand (core|cli)
# Outputs: Full package name
#######################################
map_package_name() {
  case "${1:-}" in
    core) echo "${MISE_VARS_BENCH_PACKAGE_CORE:-lithos-core}" ;;
    cli)  echo "${MISE_VARS_BENCH_PACKAGE_CLI:-lithos-cli}" ;;
    *)    echo "" ;;
  esac
}

generate_baseline_name() { ... }
discover_bench_targets() { ... }
resolve_baseline_path() { ... }
```

**Purpose:** Reusable logic with no side effects (easy to test)

#### Section 3: Validators

```bash
verify_critcmp() {
  if ! command -v critcmp &>/dev/null; then
    echo "Error: critcmp not installed. Run: mise install" >&2
    exit 1
  fi
}

ensure_archive_dir() {
  mkdir -p "${ARCHIVE_DIR}"
}
```

**Purpose:** Precondition checks

#### Section 4: Executors

```bash
build_cargo_args() { ... }
build_bench_args() { ... }
run_benchmarks() { ... }
export_baseline() { ... }
```

**Purpose:** Functions with side effects (execution, file I/O)

#### Section 5: Mode Handlers

```bash
cmd_run() {
  local resolved_package baseline_name filter
  resolved_package=$(map_package_name "${usage_package:-}")
  baseline_name="${usage_name:-$(generate_baseline_name)}"
  filter="${usage_filter:-}"

  ensure_archive_dir
  verify_critcmp

  local cargo_args=()
  build_cargo_args cargo_args "${resolved_package}"

  local bench_args=()
  build_bench_args bench_args "${filter}" "${baseline_name}"

  run_benchmarks cargo_args bench_args
  export_baseline "${baseline_name}"

  echo "✅ Baseline '${baseline_name}' complete"
}

cmd_compare() { ... }
cmd_list() { ... }
cmd_open() { ... }
```

**Purpose:** Top-level command implementations, orchestrate utilities/executors

#### Section 6: Main Entry Point

```bash
main() {
  local mode="${usage_mode:-run}"

  case "$mode" in
    run)     cmd_run ;;
    compare) cmd_compare ;;
    list)    cmd_list ;;
    open)    cmd_open ;;
    *)       echo "Error: Unknown mode: $mode" >&2; exit 1 ;;
  esac
}

main "$@"
```

**Purpose:** Dispatch to appropriate mode handler

## Migration Strategy

### Refactoring Phases

1. **Extract configuration to mise.toml vars** (low risk)
2. **Add sources/outputs metadata** (no logic change)
3. **Reorganize functions into sections** (move code, preserve logic)
4. **Switch from boolean flags to choices enum** (UX change, test carefully)
5. **Update function headers with clear comments** (documentation)

### Backward Compatibility

**Breaking change:** Mode selection UX

**Before:**
```bash
mise run test:bench --compare a b
```

**After:**
```bash
mise run test:bench compare a b
```

**Mitigation:** Document in commit message, update any CI scripts

### Testing Plan

Test all modes with all flag combinations:

**Run mode:**
- `mise run test:bench` (default, all packages)
- `mise run test:bench run -p core` (specific package)
- `mise run test:bench run -f note_parsing` (filter)
- `mise run test:bench run -q` (quick mode)
- `mise run test:bench run --name custom-baseline` (custom name)

**Compare mode:**
- `mise run test:bench compare baseline1 baseline2`
- `mise run test:bench compare baseline1 baseline2 -t 5` (threshold)
- `mise run test:bench compare baseline1 baseline2 -g "(.+)"` (grouping)

**List mode:**
- `mise run test:bench list`

**Open mode:**
- `mise run test:bench open`

**Verify:**
- All 4 benchmarks run (db_storage, db_key_handling, string_construction, note_parsing)
- Baselines saved to target/criterion/
- Baselines archived to .benchmarks/baselines/
- critcmp comparison output correct
- HTML report opens in browser

## Benefits

### Mise Integration Improvements

1. **Configuration externalized** - Easy to override archive_dir, package names
2. **Sources/outputs tracking** - Mise can skip unchanged benchmarks (useful in CI)
3. **Better UX** - Choices enum clearer than boolean flags
4. **Follows mise conventions** - Uses [vars] section correctly

### Code Organization Improvements

1. **Clear sections** - Easy to find functions by purpose
2. **Separation of concerns** - Pure functions isolated from side effects
3. **Better maintainability** - Each section has single responsibility
4. **Improved readability** - Function headers document contracts
5. **Easier testing** - Pure functions can be tested in isolation

### No Regression

1. **Same file** - No file management overhead
2. **Same functionality** - All modes, flags, options preserved
3. **Same line count** - ~300 lines (just reorganized)

## Trade-offs

### Pros

- ✅ Better mise integration (vars, sources/outputs)
- ✅ Clearer code organization (6 sections by domain)
- ✅ Single file (simple, no overhead)
- ✅ Cleaner UX (choices enum)
- ✅ Follows Google Shell Style Guide
- ✅ Easy to maintain and extend

### Cons

- ⚠️ Breaking change to UX (flag → positional arg for mode)
- ⚠️ Need to update any scripts/docs that use old flags

## Future Enhancements

Ideas for later (out of scope for this refactor):

1. **Task dependencies** - Could split archive into separate task: `test:bench:run` depends on `test:bench:archive`
2. **More export formats** - CSV, JSON output in addition to critcmp
3. **Baseline management** - `mise run test:bench clean --older-than 30d`
4. **CI integration** - `mise run test:bench ci` mode with stricter thresholds

## References

- [mise task configuration](https://mise.jdx.dev/tasks/task-configuration.html)
- [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html)
- Current implementation: `.mise/tasks/test/bench` (347 lines)
- Research findings: `refactor_findings.md`
