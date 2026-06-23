# Mise Configuration Update Design

> **Status**: Draft design for issue creation
> **Scope**: Update all mise configuration and file tasks to match the consolidated workspace crate layout, leveraging newer mise features, and conforming to the Google Shell Style Guide.

---

## 1. Current State Summary

The project consolidated from monolithic `lithos-core` / `lithos-cli` layout into individual workspace crates under `crates/`. All 12 crates now have `trace-*` package names (e.g. `trace-cli`, `trace-settings`, `trace-note`). However, mise configuration still references the legacy names.

### What's Broken/Outdated

| File | Issue |
|---|---|
| `mise.toml` `[vars]` | `core_crate = "lithos-core"`, `cli_crate = "lithos-cli"`, `binary_name = "lithos"`, `bench_package_core/cli` — all wrong |
| `mise.toml` `[env]` | `CORE_CRATE`, `CLI_CRATE` reference legacy vars |
| `mise.toml` `[tasks.build]` | Maps `core → lithos-core`, `cli → lithos`; outputs reference `lithos` binary |
| `mise.toml` per-crate shortcuts | 8 `test:unit:*` + 2 `test:bench:*` tasks hardcode legacy package names |
| `.mise/tasks/test/unit` | `map_package_name()` maps to `lithos-core` / `lithos` |
| `.mise/tasks/test/integration` | Same legacy mapping |
| `.mise/tasks/test/e2e` | Uses `--package lithos`, `binary(lithos)` |
| `.mise/tasks/test/changed` | Greps `^(lithos-core\|lithos-cli)/` — wrong paths |
| `.mise/tasks/test/coverage` | Same legacy mapping as `test/unit` |
| `.mise/tasks/test/bench` | Fallback vars `:-lithos-core`, `:-lithos-cli`; search path `lithos-core/` |

### Google Shell Style Non-Compliance

| File | Issues |
|---|---|
| `clean` | 4-space indent, missing function headers, missing `main()` argument doc |
| `fmt` | 4-space indent, `Arguments: None` boilerplate |
| `dev-setup` | 4-space indent, missing `Arguments:` on `verify_rust_toolchain` |
| `test/unit` | 4-space indent, missing proper function headers |
| `test/integration` | Missing `set -euo pipefail`, 4-space indent, missing shebang |
| `test/e2e` | Missing `set -euo pipefail`, 4-space indent, missing shebang, no function header for `main()` |
| `test/changed` | 4-space indent |
| `test/coverage` | Missing `set -euo pipefail`, 4-space indent |
| `test/watch` | 4-space indent |
| `test/burn-in` | 4-space indent |
| `adr/validate` | 4-space indent, uses `[` instead of `[[` for numeric condition |
| `adr/metrics` | 4-space indent, uses `[` instead of `[[` for numeric condition |

---

## 2. Approach Selected: Full Refactor (Approach 3)

Three changes bundled into one issue:

1. **Dynamic crate discovery** — no hardcoded package name mappings
2. **mise.toml modernization** — remove legacy vars, leverage mise's structured task dependencies
3. **Google Shell Style compliance** — across all 12 file tasks

---

## 3. Design: `mise.toml` Changes

### 3.1 Vars (`[vars]`)

Remove all legacy crate-specific vars. Keep only generic path/tool vars:

```toml
[vars]
docs_dir = "docs"
adr_dir = "docs/adr"
target_dir = "target"
target_doc_dir = "target/doc"
nextest_dir = "target/nextest"
coverage_report = "tarpaulin-report.html"
junit_unit = "nextest-unit.xml"
junit_integration = "nextest-integration.xml"
bench_archive_dir = ".benchmarks"
```

**Removed vars**: `core_crate`, `cli_crate`, `binary_name`, `bench_package_core`, `bench_package_cli`, `test_bench_dir`

**Rationale**: Package/binary names change when crates are renamed. File tasks use `cargo metadata` to discover the actual names dynamically.

### 3.2 Env (`[env]`)

Remove `CORE_CRATE` and `CLI_CRATE` (no file task uses these). Keep `PROJECT_ROOT`, `TEST_THREADS`, `CARGO_TEST_ARGS`, `TEST_OUTPUT_DIR`, `CI`.

```toml
[env]
CODEX_HOME = ".codex"
TEST_THREADS = "4"
CARGO_TEST_ARGS = "--lib --bins"
TEST_OUTPUT_DIR = "{{config_root}}/test-output"
CI = "${GITHUB_ACTIVES:-false}"
PROJECT_ROOT = "{{config_root}}"
```

### 3.3 TOML Tasks (`[tasks.*]`)

**Remove all per-crate shortcuts** — the file tasks already accept `-p <package>`:

- `test:unit:core`, `test:unit:cli`, `test:unit:config`, `test:unit:note`, `test:unit:schema`, `test:unit:template`, `test:unit:db`, `test:unit:fs`
- `test:bench:core`, `test:bench:cli`

**Simplify `[tasks.build]`** — remove the hardcoded package choices. Default to `--workspace`. If custom package selection is needed, add it as a passthrough arg:

```toml
[tasks.build]
description = "Build all workspace crates"
alias = "b"
depends = ["fmt"]
sources = ["**/*.rs", "Cargo.toml", "rustfmt.toml", "rust-toolchain.toml"]
outputs = ["target/debug/trace-cli"]
usage = '''
flag "-r --release" help="Build in release mode"
'''
run = '''
#!/usr/bin/env bash
set -euo pipefail
args=()
if [[ "${usage_release:-}" == "true" ]]; then
  args+=("--release")
fi
cargo build --workspace "${args[@]}"
'''
```

**Keep** (no changes needed): `test`, `ci`, `timing`, `_setup`, `_cleanup`, `clean:*`, `quality`, `verify`, `deny`, `doc`

**Update `[tasks.build]` outputs** to reference the actual binary path `target/debug/trace-cli`.

### 3.4 Dependency Chain Alignment

The `verify` task depends on `deny`. The `deny` task should ideally be part of the fmt → lint → test chain. Confirm whether `deny` is intended to block tests or run in parallel.

---

## 4. Design: Shared Crate Discovery Library

The core innovation is a shared bash library (`scripts/_crate_utils.sh`) that all file tasks source. This replaces the redundant `map_package_name` functions currently duplicated across 5+ tasks.

**Location**: `scripts/_crate_utils.sh` — outside mise's task discovery paths, so it won't be accidentally loaded as a task. Made non-executable (`chmod -x`).

### 4.1 `scripts/_crate_utils.sh`

A library file (not executable, per Google convention). Provides:

```bash
#!/usr/bin/env bash
#
# _crate_utils.sh — Shared crate discovery and mapping utilities.
#
# Provides functions for dynamically resolving workspace crate names
# from the filesystem, eliminating hardcoded package name mappings.
#
# Usage: source "$(dirname "$0")/_crate_utils.sh"
#
# Functions:
#   discover_crate_names    — Lists all workspace crate package names
#   resolve_crate_dir       — Maps a shorthand to a crate directory
#   CrateNameMapping        — A global associative array for lookups

#######################################
# Discover all workspace crate package names from the filesystem.
# Reads crates/*/Cargo.toml and extracts the `name` field.
# Globals:
#   None
# Arguments:
#   None
# Outputs:
#   Writes one package name per line to stdout
# Returns:
#   0 if at least one crate is found, 1 otherwise
#######################################
discover_crate_names() {
  local project_root
  project_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "${MISE_PROJECT_ROOT:-.}")"

  local crates_dir="${project_root}/crates"
  if [[ ! -d "${crates_dir}" ]]; then
    return 1
  fi

  local found=0
  local cargo_file
  for cargo_file in "${crates_dir}"/*/Cargo.toml; do
    if [[ -f "${cargo_file}" ]]; then
      local pkg_name
      pkg_name="$(awk '/^name = / { gsub(/.*name = "/, ""); gsub(/".*/, ""); print; exit }' "${cargo_file}")"
      if [[ -n "${pkg_name}" ]]; then
        echo "${pkg_name}"
        found=1
      fi
    fi
  done

  if [[ "${found}" -eq 0 ]]; then
    return 1
  fi
}

#######################################
# Resolve a shorthand or directory name to a cargo package name.
# Tries: exact match, crates/<name>/ match, prefix match.
# Globals:
#   None
# Arguments:
#   $1 - Crate shorthand (e.g., "cli", "settings", "trace-cli")
# Outputs:
#   Writes the resolved package name to stdout, or empty string
#######################################
resolve_crate_name() {
  local shorthand="$1"
  [[ -z "${shorthand}" ]] && return 1

  # Direct match against known crate dirs
  local project_root
  project_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "${MISE_PROJECT_ROOT:-.}")"

  local cargo_file="${project_root}/crates/${shorthand}/Cargo.toml"
  if [[ -f "${cargo_file}" ]]; then
    awk '/^name = / { gsub(/.*name = "/, ""); gsub(/".*/, ""); print; exit }' "${cargo_file}"
    return 0
  fi

  # It might already be a full package name — verify it exists
  if discover_crate_names | grep -qxF "${shorthand}"; then
    echo "${shorthand}"
    return 0
  fi

  return 1
}

#######################################
# Build cargo --package arguments for one or more crates.
# If no shorthand is given, output --workspace.
# Globals:
#   None
# Arguments:
#   $1 - Optional crate shorthand (empty = workspace)
# Outputs:
#   Writes "--package <name>" or "--workspace" to stdout
#######################################
build_package_arg() {
  local shorthand="$1"
  if [[ -z "${shorthand}" ]]; then
    echo "--workspace"
    return 0
  fi

  local resolved
  resolved="$(resolve_crate_name "${shorthand}")"
  if [[ -n "${resolved}" ]]; then
    echo "--package"
    echo "${resolved}"
  else
    echo "Error: Unknown crate '${shorthand}'" >&2
    return 1
  fi
}
```

### 4.2 Integration Contract

Each file task sources the library at the top:

```bash
# shellcheck source=scripts/_crate_utils.sh
source "$(git rev-parse --show-toplevel)/scripts/_crate_utils.sh"
```

**Alternative** (if `git` is unavailable in the execution context): Use `"${MISE_PROJECT_ROOT}/scripts/_crate_utils.sh"` or construct a relative path from `$(dirname "$0")`. For `.mise/tasks/test/unit`, the relative path is `"$(dirname "$0")/../../scripts/_crate_utils.sh"`.

Prefer `git rev-parse --show-toplevel` since it is already used consistently in existing file tasks (e.g., `test/bench`, `fmt`).

---

## 5. Design: Per-File Task Changes

### 5.1 `.mise/tasks/clean`

- Fix: 4-space → 2-space indent
- Fix: Add descriptive file header comment per Google style
- Fix: Remove redundant `Arguments: None` entries
- Already has: Proper `main()` function, `case` statement, `set -euo pipefail`
- Keep: Same logic, just reformat

### 5.2 `.mise/tasks/fmt`

- Fix: 4-space → 2-space indent
- Fix: Add proper file header
- Keep: Same logic (no crate-reference issues)

### 5.3 `.mise/tasks/lint`

- Already: 2-space indent, good function structure, arrays, `main()`
- Fix: Add file header comment
- Keep: No crate-reference issues

### 5.4 `.mise/tasks/dev-setup`

- Fix: 4-space → 2-space indent
- Fix: Add file header comment
- Fix: Remove redundant `Arguments: None` boilerplate
- Keep: Same logic

### 5.5 `.mise/tasks/test/unit`

- Replace: `map_package_name()` → source `_crate_utils.sh` and call `resolve_crate_name`/`build_package_arg`
- **Remove**: `context_filter()` — no longer needed. Previously filtered tests like `config::` within monolithic `lithos-core`. Now `-p settings` already scopes to the `trace-settings` crate. Users pass positional filter args directly (e.g., `mise run test:unit -p settings config::`).
- Update: `-p choices` in usage spec to list all 12 crate directories: `cli`, `settings`, `note`, `schema`, `template`, `db`, `fs`, `app`, `support`, `indexer`, `vault`, `utils`
- Fix: 4-space → 2-space indent
- Fix: Add file header, proper function docs
- Keep: Same test flow (nextest + doc-tests)
- **Package choice mapping**: `cli` → `trace-cli`, `settings` → `trace-settings`, `note` → `trace-note`, etc. All resolved via `_crate_utils.sh`.

### 5.6 `.mise/tasks/test/integration`

- Replace: `map_package_name()` → source `_crate_utils.sh`
- Fix: Add `set -euo pipefail`, shebang, file header
- Fix: 4-space → 2-space indent
- Update: `-p choices` usage spec to reflect current crate names: `cli`, `settings`, `note`, `schema`, `template`, `db`, `fs`, `app`, `support`, `indexer`, `vault`, `utils`

### 5.7 `.mise/tasks/test/e2e`

- Replace: Hardcoded `--package lithos` → source `_crate_utils.sh`, use `build_package_arg cli`
- Replace: `binary(lithos)` → `binary(trace-cli)`
- Fix: Add `set -euo pipefail`, shebang, file header, `main()` function with proper docs
- Fix: 4-space → 2-space indent
- Update: `sources` from `lithos-cli/**/*.rs` → `crates/cli/**/*.rs`

### 5.8 `.mise/tasks/test/changed`

- Replace: Legacy grep `^(lithos-core|lithos-cli)/` → `^crates/`
- Replace: Legacy directory mapping (`lithos-core` → `mise run test:unit -p core`) → source `_crate_utils.sh` and call  `mise run test:unit -p <dirname>` directly (e.g., `mise run test:unit -p settings`)
- Fix: 4-space → 2-space indent
- Fix: File header and function docs
- **New logic**: Extract changed crate dirs from `crates/<name>/` paths, then use the directory name as shorthand for `resolve_crate_name`

### 5.9 `.mise/tasks/test/coverage`

- Replace: `map_package_name()` → source `_crate_utils.sh`
- Replace: Case filter (`config | note | schema | ...`) → use `resolve_crate_name`
- Fix: Add `set -euo pipefail`
- Fix: 4-space → 2-space indent
- Fix: Add shebang, file header, `main()` function

### 5.10 `.mise/tasks/test/bench`

- Replace: `map_package_name()` → source `_crate_utils.sh`
- Replace: Legacy fallback vars `:-lithos-core`, `:-lithos-cli` → dynamic discovery
- Replace: `discover_bench_targets` search path from `lithos-core` -> resolve dynamically
- Already: 2-space indent, good structure
- Fix: Add file header comment
- Fix: Ensure proper quoting and error handling

### 5.11 `.mise/tasks/test/watch`

- Fix: 4-space → 2-space indent
- Fix: Add file header
- Keep: Same logic

### 5.12 `.mise/tasks/test/burn-in`

- Fix: 4-space → 2-space indent
- Fix: Add file header, function docs
- Keep: Same logic

### 5.13 `.mise/tasks/adr/validate`

- Fix: 4-space → 2-space indent
- Fix: Replace `if [ $EXIT_CODE -eq 0 ]` → `if [[ ${EXIT_CODE} -eq 0 ]]` (or `if (( EXIT_CODE == 0 ))`)
- Fix: Add file header
- Keep: Same validation logic

### 5.14 `.mise/tasks/adr/metrics`

- Fix: 4-space → 2-space indent
- Fix: Replace `if [ $TOTAL_ADRS -eq 0 ]` → `if (( TOTAL_ADRS == 0 ))`
- Fix: Replace `if [ $missing -eq 0 ]` → `if (( missing == 0 ))`
- Fix: Replace `if [ $COMPLETENESS -lt 100 ]` → `if (( COMPLETENESS < 100 ))`
- Fix: Add file header, function docs
- Fix: Remove `exit 0` at end of main (unnecessary, `main` returns naturally)

---

## 6. Google Shell Style Guide Compliance Checklist

Every file task must conform to these rules:

| Rule | Check |
|---|---|
| `#!/usr/bin/env bash` shebang | All executable tasks |
| `set -euo pipefail` at top | All executable tasks |
| 2-space indentation, no tabs | All files |
| `[[ ... ]]` over `[ ... ]` or `test` | All conditionals |
| `(( ... ))` for numeric comparisons | Arithmetic conditions |
| `$(...)` over backticks | All command substitutions |
| `"${var}"` quoting (curlies + quotes) | All variable expansions |
| `$()` for command substitution | All cases |
| `local` on all function variables | All functions |
| `main()` function for scripts > 1 function | All multi-function scripts |
| `main "$@"` call at bottom | All scripts with `main` |
| File header comment (description) | All files |
| Function header comments for non-obvious functions | Complex functions |
| Error messages to stderr (`>&2`) | All error paths |
| Return value checking on critical commands | mv, cp, rm, mkdir |
| `readonly` for constants | Where applicable |
| Lowercase function/variable names | All user-defined names |
| Screaming case only for constants/env | Only environment/constants |
| Pipeline splitting (one per line) | Long pipelines |
| Arrays for argument lists | cargo args, etc. |

---

## 7. Implementation Ordering

The proposed implementation order minimizes disruption:

1. **Create `_crate_utils.sh`** — shared library (no existing code changes)
2. **Update `mise.toml`** — vars, env, remove legacy tasks, simplify build task
3. **Update `test/unit`** — first consumer of the shared library, most complex
4. **Update `test/integration`** — second consumer
5. **Update `test/e2e`** — uses `build_package_arg cli`
6. **Update `test/changed`** — uses dynamic crate dir extraction
7. **Update `test/coverage`** — uses shared library
8. **Update `test/bench`** — uses shared library for package mapping
9. **Reformat remaining tasks** — `clean`, `fmt`, `lint`, `dev-setup`, `watch`, `burn-in` (indentation/style only)
10. **Reformat ADR tasks** — `adr/validate`, `adr/metrics`
11. **Verify** — run `mise run verify`, individual test tasks, and `detect_changes()`

---

## 8. Verification Gate

After all changes:

1. `mise run fmt` — formatting check
2. `mise run lint` — clippy (unaffected but ensure no regression)
3. `mise run test:unit` — workspace unit tests pass
4. `mise run test:unit -p cli` — specific crate unit tests
5. `mise run test:unit -p settings` — specific crate unit tests
6. `mise run test:integration` — integration tests pass
7. `mise run test:e2e` — end-to-end tests pass
8. `mise run test:changed` — changed crate detection works (test on a branch with crate changes)
9. `mise run clean` — cleanup works
10. `mise run adr:validate` — ADRs pass validation
11. `mise run verify` — full quality gate passes
12. GitNexus `detect_changes()` — verify only expected files affected

---

## 9. Risk Assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| `resolve_crate_name` fails for edge case shorthand | Low | Test with each shorthand before committing |
| `_crate_utils.sh` sourcing path wrong for deep tasks | Low | Use `$(dirname "$0")` relative pathing; test each depth |
| Dynamic crate discovery breaks in CI | Low | CI runs from project root; `git rev-parse` handles this |
| A crate rename breaks `_crate_utils.sh` | Low by design — auto-discovers from Cargo.toml |
| Google style changes introduce bugs | Low | Indent-only changes are mechanical; test after each task |
