# Benchmark Task Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor `.mise/tasks/test/bench` for better mise integration and code organization

**Architecture:** Single file with 6 sections (configuration, utilities, validators, executors, handlers, main). Externalize config to mise.toml vars. Use choices enum for mode selection.

**Tech Stack:** Bash, mise task system, critcmp

---

## File Structure

**Modified files:**
- `mise.toml` - Add benchmark vars to [vars] section
- `.mise/tasks/test/bench` - Complete refactor with new structure

**Design spec:** `docs/superpowers/specs/2026-05-15-bench-task-refactor-design.md`

---

## Task 1: Add Benchmark Vars to mise.toml

**Files:**
- Modify: `mise.toml` (add after line 78, in [vars] section)

- [ ] **Step 1: Add benchmark configuration vars**

Add these lines to the `[vars]` section in `mise.toml` (after line 78, after `binary_name = "lithos"`):

```toml
# Benchmark configuration
bench_archive_dir = ".benchmarks/baselines"
bench_package_core = "lithos-core"
bench_package_cli = "lithos-cli"
```

- [ ] **Step 2: Verify TOML syntax**

Run: `mise config ls 2>&1 | head -5`
Expected: No parse errors, shows config paths

- [ ] **Step 3: Verify vars are accessible**

Run: `mise exec -- bash -c 'echo $MISE_VARS_BENCH_ARCHIVE_DIR'`
Expected: `.benchmarks/baselines`

- [ ] **Step 4: Commit mise.toml changes**

```bash
git add mise.toml
git commit -m "feat(bench): externalize config to mise.toml vars

Add benchmark configuration variables to [vars] section:
- bench_archive_dir: baseline archive directory
- bench_package_core: full name for 'core' shorthand
- bench_package_cli: full name for 'cli' shorthand

This enables easy override per-environment and follows mise conventions."
```

---

## Task 2: Create New Bench Script Structure

**Files:**
- Modify: `.mise/tasks/test/bench` - Complete rewrite with new structure

- [ ] **Step 1: Backup current script**

```bash
cp .mise/tasks/test/bench .mise/tasks/test/bench.backup
```

- [ ] **Step 2: Write new script header and configuration section**

Replace entire `.mise/tasks/test/bench` file with this content:

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

set -euo pipefail

#######################################
# SECTION 1: CONFIGURATION
#######################################
ARCHIVE_DIR="${MISE_VARS_BENCH_ARCHIVE_DIR:-.benchmarks/baselines}"
cd "$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"

```

- [ ] **Step 3: Verify script header**

Run: `mise tasks --json | grep -A 5 'test:bench'`
Expected: Shows task with description, alias "tb", sources, outputs

---

## Task 3: Add Domain Utilities Section

**Files:**
- Modify: `.mise/tasks/test/bench` - Add pure functions

- [ ] **Step 1: Add map_package_name function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# SECTION 2: DOMAIN UTILITIES
# Pure functions - no side effects
#######################################

#######################################
# Map shorthand package names to full cargo package names.
# Globals:
#   MISE_VARS_BENCH_PACKAGE_CORE
#   MISE_VARS_BENCH_PACKAGE_CLI
# Arguments:
#   Package shorthand (core|cli)
# Outputs:
#   Full package name
#######################################
map_package_name() {
  case "${1:-}" in
    core)
      echo "${MISE_VARS_BENCH_PACKAGE_CORE:-lithos-core}"
      ;;
    cli)
      echo "${MISE_VARS_BENCH_PACKAGE_CLI:-lithos-cli}"
      ;;
    *)
      echo ""
      ;;
  esac
}

```

- [ ] **Step 2: Add generate_baseline_name function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Generate a deterministic baseline name.
# Outputs: <branch>-<date>-<time>-<sha_short>
#######################################
generate_baseline_name() {
  local branch date_time sha
  branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
  branch="${branch//\//-}"
  date_time=$(date +%Y-%m-%d-%H%M%S)
  sha=$(git rev-parse --short HEAD 2>/dev/null || echo "0000000")
  echo "${branch}-${date_time}-${sha}"
}

```

- [ ] **Step 3: Add discover_bench_targets function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Discover benchmark target names from Cargo.toml files.
# Arguments:
#   Search path (directory, e.g. "lithos-core" or "." for all)
# Outputs:
#   One bench target name per line
#######################################
discover_bench_targets() {
  local search_path=${1:-.}
  find "${search_path}" -maxdepth 2 -name Cargo.toml \
    -not -path '*/target/*' \
    -exec awk '
    /^\[\[bench\]\]$/ { in_bench = 1; next }
    in_bench && /^name = / {
      gsub(/.*name = "/, "")
      gsub(/".*/, "")
      print
      in_bench = 0
    }
  ' {} \;
}

```

- [ ] **Step 4: Add resolve_baseline_path function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Resolve a baseline reference to a file path or critcmp name.
# Arguments:
#   Baseline reference string
# Outputs:
#   Resolved path or name
#######################################
resolve_baseline_path() {
  local ref=$1
  if [[ -f "$ref" ]]; then
    echo "$ref"
  elif [[ -f "${ARCHIVE_DIR}/${ref}.json" ]]; then
    echo "${ARCHIVE_DIR}/${ref}.json"
  else
    echo "$ref"
  fi
}

```

---

## Task 4: Add Validators Section

**Files:**
- Modify: `.mise/tasks/test/bench` - Add validation functions

- [ ] **Step 1: Add verify_critcmp function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# SECTION 3: VALIDATORS
# Precondition checks
#######################################

#######################################
# Verify critcmp is installed.
#######################################
verify_critcmp() {
  if ! command -v critcmp &>/dev/null; then
    echo "Error: critcmp is not installed." >&2
    echo "Run: mise install" >&2
    exit 1
  fi
}

```

- [ ] **Step 2: Add ensure_archive_dir function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Ensure the archive directory exists.
#######################################
ensure_archive_dir() {
  mkdir -p "${ARCHIVE_DIR}"
}

```

---

## Task 5: Add Executors Section

**Files:**
- Modify: `.mise/tasks/test/bench` - Add functions with side effects

- [ ] **Step 1: Add build_cargo_args function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# SECTION 4: EXECUTORS
# Functions with side effects
#######################################

#######################################
# Build arguments for cargo bench.
# Arguments:
#   Reference to an array for cargo arguments
#   The resolved package name (empty = all workspace benches)
# Outputs:
#   None (modifies array by reference)
#######################################
build_cargo_args() {
  local -n ref_cargo_args=$1
  local package_name=$2
  local search_path="."

  if [[ -n "${package_name}" ]]; then
    search_path="${package_name}"
  fi

  while IFS= read -r bench_name; do
    ref_cargo_args+=("--bench" "${bench_name}")
  done < <(discover_bench_targets "${search_path}")
}

```

- [ ] **Step 2: Add build_bench_args function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Build arguments for Criterion benchmarks.
# Globals:
#   usage_quick
#   usage_noplot
# Arguments:
#   Reference to an array for benchmark arguments
#   Benchmark filter string (optional)
#   Baseline name
# Outputs:
#   None (modifies array by reference)
#######################################
build_bench_args() {
  local -n ref_bench_args=$1
  local filter=${2:-}
  local baseline_name=$3

  ref_bench_args+=("--save-baseline" "${baseline_name}")

  if [[ -n "${filter}" ]]; then
    ref_bench_args+=("${filter}")
  fi

  if [[ "${usage_quick:-}" == "true" ]]; then
    ref_bench_args+=("--quick")
  fi

  if [[ "${usage_noplot:-}" == "true" ]]; then
    ref_bench_args+=("--noplot")
  fi
}

```

- [ ] **Step 3: Add run_benchmarks function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Run performance benchmarks using cargo bench.
# Arguments:
#   Cargo arguments array (reference)
#   Benchmark arguments array (reference)
# Outputs:
#   Writes benchmark progress to stdout
#######################################
run_benchmarks() {
  local -n cargo_args_ref=$1
  local -n bench_args_ref=$2

  echo "🧪 Running benchmarks..."
  if [[ ${#bench_args_ref[@]} -gt 0 ]]; then
    cargo bench "${cargo_args_ref[@]}" -- "${bench_args_ref[@]}"
  else
    cargo bench "${cargo_args_ref[@]}"
  fi
}

```

- [ ] **Step 4: Add export_baseline function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Export a saved baseline to the archive directory.
# Arguments:
#   Baseline name
#######################################
export_baseline() {
  local name=$1
  local archive_path="${ARCHIVE_DIR}/${name}.json"

  echo "🧪 Archiving baseline '${name}'..."
  critcmp --export "${name}" > "${archive_path}"
  if [[ ! -s "${archive_path}" ]]; then
    echo "Error: critcmp export produced empty file" >&2
    exit 1
  fi
  echo "✅ Archived to ${archive_path}"
}

```

---

## Task 6: Add Mode Handlers Section

**Files:**
- Modify: `.mise/tasks/test/bench` - Add command implementations

- [ ] **Step 1: Add cmd_run function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# SECTION 5: MODE HANDLERS
# Top-level command implementations
#######################################

#######################################
# Run benchmarks and archive the baseline.
# Globals:
#   usage_name, usage_package, usage_filter
#######################################
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

```

- [ ] **Step 2: Add cmd_compare function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Compare two baselines using critcmp.
# Globals:
#   usage_threshold, usage_group, usage_baseline_a, usage_baseline_b
#######################################
cmd_compare() {
  local a b
  a=$(resolve_baseline_path "${usage_baseline_a?Error: baseline_a required}")
  b=$(resolve_baseline_path "${usage_baseline_b?Error: baseline_b required}")

  local critcmp_args=()
  if [[ -n "${usage_threshold:-}" ]]; then
    critcmp_args+=("--threshold" "${usage_threshold}")
  fi
  if [[ -n "${usage_group:-}" ]]; then
    critcmp_args+=("-g" "${usage_group}")
  fi

  verify_critcmp
  critcmp "${critcmp_args[@]}" "$a" "$b"
}

```

- [ ] **Step 3: Add cmd_list function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# List available baselines (local + archived).
#######################################
cmd_list() {
  echo "=== Local baselines (target/criterion/) ==="
  if command -v critcmp &>/dev/null; then
    critcmp --list 2>/dev/null || echo "  (none)"
  else
    echo "  (install critcmp to query local baselines)"
  fi

  echo ""
  echo "=== Archived baselines (${ARCHIVE_DIR}/) ==="
  if ls "${ARCHIVE_DIR}"/*.json &>/dev/null 2>&1; then
    for f in "${ARCHIVE_DIR}"/*.json; do
      local name
      name=$(basename "$f" .json)
      echo "  ${name}"
    done
  else
    echo "  (none)"
  fi
}

```

- [ ] **Step 4: Add cmd_open function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# Open the Criterion HTML report.
#######################################
cmd_open() {
  local report="target/criterion/reports/index.html"
  if [[ -f "$report" ]]; then
    echo "🧪 Opening report..." >&2
    if [[ "$(uname -s)" == "Darwin" ]]; then
      open "$report"
    else
      xdg-open "$report"
    fi
  else
    echo "No report found. Run benchmarks first." >&2
  fi
}

```

---

## Task 7: Add Main Entry Point

**Files:**
- Modify: `.mise/tasks/test/bench` - Add dispatcher and main call

- [ ] **Step 1: Add main function**

Append to `.mise/tasks/test/bench`:

```bash
#######################################
# SECTION 6: MAIN ENTRY POINT
#######################################

#######################################
# Main entry point.
#######################################
main() {
  local mode="${usage_mode:-run}"

  case "$mode" in
    run)
      cmd_run
      ;;
    compare)
      cmd_compare
      ;;
    list)
      cmd_list
      ;;
    open)
      cmd_open
      ;;
    *)
      echo "Error: Unknown mode: $mode" >&2
      exit 1
      ;;
  esac
}

main "$@"
```

- [ ] **Step 2: Verify script is executable**

Run: `chmod +x .mise/tasks/test/bench`

- [ ] **Step 3: Verify script syntax**

Run: `bash -n .mise/tasks/test/bench`
Expected: No output (syntax OK)

- [ ] **Step 4: Verify mise can parse the task**

Run: `mise tasks --json | jq '.[] | select(.name == "test:bench") | {name, description, alias}'`
Expected: Shows task with name "test:bench", description, alias "tb"

---

## Task 8: Test All Modes

**Files:**
- No files modified (testing only)

- [ ] **Step 1: Test run mode (default)**

Run: `mise run test:bench run -p core -q --name test-refactor-1`
Expected:
- Discovers 4 bench targets (db_storage, db_key_handling, string_construction, note_parsing)
- Runs cargo bench with --quick flag
- Saves baseline as "test-refactor-1"
- Archives to `.benchmarks/baselines/test-refactor-1.json`
- Shows "✅ Baseline 'test-refactor-1' complete"

- [ ] **Step 2: Test list mode**

Run: `mise run test:bench list`
Expected:
- Shows "=== Local baselines ===" with critcmp --list output
- Shows "=== Archived baselines ===" with test-refactor-1 in list

- [ ] **Step 3: Test run mode again (for comparison baseline)**

Run: `mise run test:bench run --name test-refactor-2 -q`
Expected: Creates second baseline "test-refactor-2"

- [ ] **Step 4: Test compare mode**

Run: `mise run test:bench compare test-refactor-1 test-refactor-2`
Expected: Shows critcmp comparison output (likely all "No difference" since code unchanged)

- [ ] **Step 5: Test open mode**

Run: `mise run test:bench open`
Expected: Opens `target/criterion/reports/index.html` in browser

- [ ] **Step 6: Test filter flag**

Run: `mise run test:bench run -f note_parsing --name test-filter -q`
Expected: Only runs note_parsing benchmark

- [ ] **Step 7: Clean up test baselines**

Run:
```bash
rm -f .benchmarks/baselines/test-refactor-*.json
rm -f .benchmarks/baselines/test-filter.json
```

---

## Task 9: Verify Backward Compatibility Break is Documented

**Files:**
- Read: `docs/superpowers/specs/2026-05-15-bench-task-refactor-design.md`

- [ ] **Step 1: Verify breaking change is documented in design spec**

Run: `grep -A 5 "Breaking change" docs/superpowers/specs/2026-05-15-bench-task-refactor-design.md`
Expected: Shows documentation of UX change from flags to positional arg

- [ ] **Step 2: Verify old usage no longer works**

Run: `mise run test:bench --compare 2>&1 | head -3`
Expected: Error or usage message (flag --compare no longer exists)

- [ ] **Step 3: Verify new usage works**

Run: `mise run test:bench 2>&1 | head -1`
Expected: Starts running benchmarks (run mode is default)

---

## Task 10: Remove Backup and Commit Final Changes

**Files:**
- Delete: `.mise/tasks/test/bench.backup`
- Modify: `.mise/tasks/test/bench` (final version)

- [ ] **Step 1: Verify all tests passed**

Manually confirm all tests from Task 8 passed successfully.

- [ ] **Step 2: Remove backup file**

Run: `rm .mise/tasks/test/bench.backup`

- [ ] **Step 3: Compare line counts**

Run:
```bash
echo "New script line count:"
wc -l .mise/tasks/test/bench
```
Expected: ~320 lines (similar to original 347)

- [ ] **Step 4: Verify shellcheck passes**

Run: `shellcheck .mise/tasks/test/bench || echo "shellcheck not installed, skipping"`
Expected: No errors (or skipped if shellcheck not available)

- [ ] **Step 5: Commit refactored bench script**

```bash
git add .mise/tasks/test/bench
git commit -m "refactor(bench): reorganize script with better mise integration

Refactor .mise/tasks/test/bench for better mise integration and code
organization following approved design spec.

Changes:
- Add sources/outputs tracking for mise caching
- Use choices enum for mode selection (breaking change)
- Externalize config to mise.toml vars
- Organize into 6 clear sections by domain/responsibility
- Separate pure functions from side effects
- Add comprehensive function header comments

Breaking change: Mode selection UX
- Old: mise run test:bench --compare a b
- New: mise run test:bench compare a b

All existing functionality preserved:
- run mode: benchmarks + auto-archive
- compare mode: critcmp comparison with threshold/grouping
- list mode: show local + archived baselines
- open mode: open HTML report
- All flags: -p, -f, -q, -n, --name, -t, -g

Implementation follows design spec:
docs/superpowers/specs/2026-05-15-bench-task-refactor-design.md"
```

---

## Task 11: Update Documentation (if needed)

**Files:**
- Read: `AGENTS.md`, `bench-tasks-design.md` (check if references need updating)

- [ ] **Step 1: Check if AGENTS.md references old bench usage**

Run: `grep -n "test:bench" AGENTS.md`
Expected: Shows references to bench task (if any)

- [ ] **Step 2: Check if bench-tasks-design.md needs updating**

Run: `grep -n "\-\-compare\|\-\-list\|\-\-open" bench-tasks-design.md`
Expected: Shows old flag-based usage (needs updating)

- [ ] **Step 3: Update bench-tasks-design.md with new usage**

If grep found old usage, update `bench-tasks-design.md` to reflect new choices enum approach:

```markdown
## Usage

### Run Mode (default)
mise run test:bench              # all packages
mise run test:bench run          # explicit
mise run test:bench run -p core  # specific package

### Compare Mode
mise run test:bench compare baseline1 baseline2
mise run test:bench compare baseline1 baseline2 -t 5

### List Mode
mise run test:bench list

### Open Mode
mise run test:bench open
```

- [ ] **Step 4: Commit documentation updates (if changes made)**

```bash
git add bench-tasks-design.md
git commit -m "docs(bench): update bench-tasks-design.md with new usage

Update documentation to reflect new choices enum mode selection.

Old: --compare, --list, --open flags
New: positional mode argument (run|compare|list|open)"
```

---

## Completion Checklist

All tasks complete when:

- [x] mise.toml has benchmark vars in [vars] section
- [x] .mise/tasks/test/bench refactored with 6-section structure
- [x] All 4 modes tested and working (run, compare, list, open)
- [x] All flags tested and working (-p, -f, -q, -n, --name, -t, -g)
- [x] Sources/outputs metadata added for mise caching
- [x] Choices enum working for mode selection
- [x] Breaking change documented
- [x] Backup file removed
- [x] All changes committed with descriptive messages
- [x] Documentation updated (if applicable)

**Final verification:**

Run: `git log --oneline --graph -5`
Expected: Shows commits for vars, refactor, and docs (if updated)

Run: `mise run test:bench list`
Expected: Works with new mode syntax
