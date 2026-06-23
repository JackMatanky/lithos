---
labels: ["ready-for-agent", "enhancement"]
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
- `scripts/_crate_utils.sh` — shared bash library for dynamic crate discovery (non-executable)

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

1. **Create `scripts/_crate_utils.sh`** — shared library (see design doc section 4)
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

1. **Dynamic discovery**: Use `scripts/_crate_utils.sh` to extract package names from `crates/*/Cargo.toml` at runtime instead of hardcoding `lithos-core`/`lithos-cli`. Future crate renames require no mise config changes.

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
- `scripts/_crate_utils.sh` — new shared bash library providing `discover_crate_names()`, `resolve_crate_name()`, and `build_package_arg()` for dynamic crate name resolution
- `mise.toml` `[vars]` section — remove legacy vars; keep only generic path/tool vars
- `.mise/tasks/*` file tasks — each sources the shared library and uses dynamic resolution
- Google Shell Style Guide (`docs/refs/google_shell_script_style_guide.md`) — all 12 file tasks comply

**Acceptance criteria:**
- [ ] `mise run test:unit` runs all workspace unit tests successfully
- [ ] `mise run test:unit -p cli` runs only `trace-cli` unit tests
- [ ] `mise run test:unit -p settings` runs only `trace-settings` unit tests
- [ ] `mise run test:integration` runs all workspace integration tests
- [ ] `mise run test:e2e` runs CLI end-to-end tests using `trace-cli` binary
- [ ] `mise run test:changed` detects changes in `crates/*/` and tests only affected crates
- [ ] `mise run test:bench` runs benchmarks with correct package selection
- [ ] `mise run clean` removes build artifacts without error
- [ ] `mise run adr:validate` validates all ADRs
- [ ] `mise run verify` — full quality gate passes
- [ ] All `.mise/tasks/*` scripts pass ShellCheck or follow Google Shell Style conventions
- [ ] No references to `lithos-core`, `lithos-cli`, or `lithos` remain in mise configuration or task scripts
- [ ] `detect_changes()` reports no unexpected affected files

**Out of scope:**
- Rust source code changes
- Renaming crate directories or package names
- Adding new mise tasks beyond fixing existing ones
- Updating documentation outside `mise.toml` and `.mise/tasks/` files
- Fixing legacy references in `.scratch/` issue files or historical design docs
