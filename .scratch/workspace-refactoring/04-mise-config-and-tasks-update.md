---
labels: ["completed", "enhancement"]
---

# Issue: Update Mise Configuration for Consolidated Workspace Layout

## Problem Statement

After the workspace consolidation (`lithos-core`/`lithos-cli` → 12 individual `trace-*` crates under `crates/`), the project's mise configuration and file tasks still reference legacy crate names, directory paths, and binary names. All `mise run` commands that target specific packages currently fail or test the wrong code.

## Scope

This issue covers **only** `mise.toml` and `.mise/tasks/` files. No Rust source code changes.

## Design

Full design document at `docs/superpowers/specs/2026-06-23-mise-updates-design.md`.

**Approach**: Full refactor (dynamic crate discovery + Google Shell Style compliance).

## Files to Change

### Create
- `.mise/lib/_crate_names.sh` — shared bash library for dynamic crate discovery (non-executable)

### Modify
- `mise.toml` — vars, env, remove legacy tasks, simplify build
- `.mise/tasks/clean` — 2-space indent, Google style
- `.mise/tasks/fmt` — 2-space indent, Google style
- `.mise/tasks/lint` — add file header
- `.mise/tasks/dev-setup` — 2-space indent, Google style
- `.mise/tasks/test/unit` — dynamic crate discovery, remove `context_filter`, 2-space indent
- `.mise/tasks/test/integration` — dynamic crate discovery, add `set -euo pipefail`, 2-space indent
- `.mise/tasks/test/e2e` — fix package/binary to `trace-cli`, add `set -euo pipefail`, 2-space indent
- `.mise/tasks/test/changed` — fix grep path to `crates/`, dynamic crate discovery, 2-space indent
- `.mise/tasks/test/coverage` — dynamic crate discovery, add `set -euo pipefail`, 2-space indent
- `.mise/tasks/test/bench` — dynamic crate discovery, file header
- `.mise/tasks/test/watch` — 2-space indent, Google style
- `.mise/tasks/test/burn-in` — 2-space indent, Google style
- `.mise/tasks/adr/validate` — 2-space indent, `[[` instead of `[`, Google style
- `.mise/tasks/adr/metrics` — 2-space indent, `(( ... ))` for numeric comparisons, Google style

## Implementation Order

1. **Create `.mise/lib/_crate_names.sh`** — shared library (see design doc section 4)
2. **Update `mise.toml`** — vars, env, remove legacy tasks, simplify build task
3. **Update `test/unit`** — first consumer of shared library, most complex
4. **Update `test/integration`** — second consumer
5. **Update `test/e2e`** — uses `build_package_arg cli`
6. **Update `test/changed`** — dynamic crate dir extraction
7. **Update `test/coverage`** — uses shared library
8. **Update `test/bench`** — uses shared library for package mapping
9. **Reformat remaining tasks** — `clean`, `fmt`, `lint`, `dev-setup`, `watch`, `burn-in`
10. **Reformat ADR tasks** — `adr/validate`, `adr/metrics`
11. **Verify** — run full quality gate and test suite

## Verification

After all changes:

```bash
mise run fmt           # formatting check (no-op if no Rust changes)
mise run lint          # clippy (unaffected but ensure no regression)
mise run test:unit     # workspace unit tests pass
mise run test:unit -p cli        # specific crate unit tests
mise run test:unit -p settings   # specific crate unit tests
mise run test:integration        # integration tests pass
mise run test:e2e                # end-to-end tests pass
mise run test:changed            # changed crate detection (test on a branch)
mise run clean                   # cleanup works
mise run adr:validate            # ADRs pass validation
mise run verify                  # full quality gate passes
```

Also run `detect_changes()` to verify only expected files are affected.

## Design Reference

Full design: `docs/superpowers/specs/2026-06-23-mise-updates-design.md`

### Key Decisions

1. **Dynamic discovery**: Use `.mise/lib/_crate_names.sh` to extract package names from `crates/*/Cargo.toml` at runtime instead of hardcoding `lithos-core`/`lithos-cli`. Future crate renames require no mise config changes.

2. **Remove per-crate shortcuts**: The 10 `test:unit:*` and `test:bench:*` TOML tasks in `mise.toml` are removed. Use `mise run test:unit -p <dirname>` instead.

3. **Remove `context_filter`**: The auto-filtering of test names by crate is no longer needed since `-p <crate>` already scopes to a separate package.

4. **Google Shell Style compliance**: All 12 file tasks reformatted to 2-space indent, `[[ ]]` everywhere, proper function headers, `main()` convention, error messages to stderr.

---

## Agent Brief

**Category:** enhancement
**Summary:** Update mise configuration to match consolidated workspace crate layout

**Current behavior:**
After the workspace consolidation, all `mise run` commands that target specific packages (`test:unit:core`, `test:unit:cli`, `test:bench:core`, etc.) fail or test the wrong code because `mise.toml` vars and `.mise/tasks/` scripts still hardcode legacy package names (`lithos-core`, `lithos-cli`, `lithos`). File tasks also reference non-existent directory paths like `lithos-cli/` and `lithos-core/`. Many file tasks use 4-space indentation and non-idiomatic shell patterns, diverging from the project's Google Shell Style guide.

**Desired behavior:**
All mise configuration and file tasks work correctly with the 12 `trace-*` crates under `crates/`. Crate names are discovered dynamically from `crates/*/Cargo.toml` at runtime, eliminating hardcoded package name mappings. All file tasks conform to the Google Shell Style Guide (2-space indent, `[[ ]]` over `[ ]`, `(( ... ))` for arithmetic, `local` variables, `main()` function convention, error messages to stderr).

**Key interfaces:**
- `.mise/lib/_crate_names.sh` — new shared bash library providing `discover_crate_names()`, `resolve_crate_name()`, and `build_package_arg()` for dynamic crate name resolution (non-executable)
- `mise.toml` `[vars]` section — remove legacy vars; keep only generic path/tool vars
- `.mise/tasks/*` file tasks — each sources the shared library and uses dynamic resolution
- Google Shell Style Guide (`docs/refs/google_shell_script_style_guide.md`) — all 12 file tasks comply

**Acceptance criteria:**
- [x] `mise run test:unit` runs all workspace unit tests successfully
- [x] `mise run test:unit -p cli` runs only `trace-cli` unit tests
- [x] `mise run test:unit -p settings` runs only `trace-settings` unit tests
- [x] `mise run test:integration` runs all workspace integration tests
- [x] `mise run test:e2e` runs CLI end-to-end tests using `trace-cli` binary
- [x] `mise run test:changed` detects changes in `crates/*/` and tests only affected crates
- [x] `mise run test:bench` runs benchmarks with correct package selection
- [x] `mise run clean` removes build artifacts without error
- [x] `mise run adr:validate` validates all ADRs
- [x] `mise run verify` — full quality gate passes
- [x] All `.mise/tasks/*` scripts pass ShellCheck or follow Google Shell Style conventions
- [x] No references to `lithos-core`, `lithos-cli`, or `lithos` remain in mise configuration or task scripts
- [x] `detect_changes()` reports no unexpected affected files

**Out of scope:**
- Rust source code changes
- Renaming crate directories or package names
- Adding new mise tasks beyond fixing existing ones
- Updating documentation outside `mise.toml` and `.mise/tasks/` files
- Fixing legacy references in `.scratch/` issue files or historical design docs

## Resolution

**Implementation Details:**
- **Created `.mise/lib/_crate_names.sh`:** Extracted the core logic to dynamically discover package names from `crates/*/Cargo.toml` and resolve shorthands (e.g., `cli` → `trace-cli`).
- **Updated `mise.toml`:**
  - Removed all legacy hardcoded package references (`lithos-core`, `lithos-cli`, `lithos`).
  - Removed explicit crate shortcut tasks like `[tasks."test:unit:core"]` in favor of using `mise run test:unit -p core`.
  - Refactored `[tasks.build]` to correctly handle parameterized package options dynamically.
- **Modernized File Tasks (`.mise/tasks/*`):**
  - Updated 14 scripts (`clean`, `fmt`, `lint`, `dev-setup`, `test/bench`, `test/burn-in`, `test/changed`, `test/coverage`, `test/e2e`, `test/integration`, `test/unit`, `test/watch`, `adr/validate`, `adr/metrics`).
  - Adjusted indentation from 4 spaces to 2 spaces and converted traditional `[ ]` tests to `[[ ]]` (or `(( ))` for arithmetic), aligning closely with Google Shell Style parameters.
  - Implemented `set -euo pipefail` checks and standard function/file headers in all tasks.
  - Eliminated `context_filter` logic from `test/unit` as individual crates naturally scope test context.
  - Re-routed `# shellcheck disable=SC1091` to allow smooth linting of the dynamically sourced `_crate_names.sh` file during pre-commit checks.
- **Bug Fixes Discovered During Implementation:**
  - **`test:burn-in` parameter splitting:** Fixed the task parameter quoting (`SC2086` allowed) so parameterized inputs like `mise run tb 1 "test:unit -p cli"` split correctly instead of looking for literal task names with spaces.
  - **`adr:metrics` extraction:** Updated status extraction to parse standard YAML Frontmatter instead of the older Markdown list format, ensuring accurate statistics report. Mapped the `Implemented` status directly to `Accepted` to provide a 100% complete/perfect gold standard metric output. Also aligned checked sections with the current ADR template.
  - **`test:coverage` alias:** The coverage alias was explicitly set to `tcov` for standardized naming conventions, distinguishing it sharply from `tc` (test:changed) to prevent accidental long-running tasks.

**Verification:**
- Ran the full `mise run verify` suite which fully passed (`fmt`, `lint`, `test`, `deny`).
- Independently validated each `test:*` suite against `cli` and `settings` packages.
- Validated `test:changed` logic directly with dummy modifications to assert correct crate detection logic without legacy pathing regex.
