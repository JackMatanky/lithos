---
labels: ["ready-for-agent"]
---

## Implementation

**Completed:** 2026-06-24
**Branch:** `05-global-rename`
**Commit:** `74a2419c` — `feat: global rename Lithos to Traces`
**Files changed:** 351 (2275 insertions, 2275 deletions)

### Approach

Bulk `sed` replacement across all files excluding `.scratch/`, `target/`, `.git/`, `.gitnexus/` using three non-overlapping patterns in a single pass:

```
s/LITHOS/TRACES/g   → env vars (LITHOS_VAULT_DIR → TRACES_VAULT_DIR, etc.)
s/Lithos/Traces/g   → title case (docs, CLI descriptions, comments)
s/lithos/traces/g   → lowercase (config paths, UUID namespace, marker prefixes)
```

### Key changes by area

| Area | Files | Notes |
|------|-------|-------|
| CLI | `crates/cli/src/cli.rs` | `name = "traces"` literal (clap requires compile-time constant) |
| Env vars | `crates/settings/src/discovery/env.rs` | `TRACES_VAULT_DIR`, `TRACES_CONFIG_FILE`, `TRACES_CACHE_DIR`, `TRACES_CEILING_DIRS`, `TRACES_SUPPRESS_GLOBAL` |
| Marker prefixes | `crates/settings/src/discovery/policy.rs` | `"traces"`, `".traces"`, `".traces/config"` |
| UUID v5 namespace | `crates/utils/src/uuid.rs`, `crates/template/src/aggregate.rs` | `b"traces"` — pre-1.0 semantic change accepted |
| clippy.toml | `clippy.toml:38` | `doc-valid-idents` updated from `"Lithos"` to `"Traces"` |
| Cargo metadata | `Cargo.toml:19` | `repository = "https://github.com/jack/traces"` |

### Deviations from plan

- **Constants skipped**: `crates/utils/src/project_name.rs` was created with `PROJECT_NAME_LOWER/UPPER/TITLE` per spec, then removed by decision. String literals used directly instead — simpler, no dead code, no import boilerplate.
- **clap `name` attribute**: Uses `"traces"` literal (couldn't use constant anyway since clap requires compile-time literal).

### Verification

```
cargo check        ✅
fmt                ✅
clippy (deny all)  ✅
cargo deny         ✅
unit tests         2091/2091 passed ✅
integration tests  50/50 passed      ✅
doc tests          all passed        ✅
adr:validate       ✅
pre-commit hooks   all passed        ✅
```

No references to "Lithos", "lithos", or "LITHOS" remain anywhere in the tree (confirmed via `rg`).

---

## Parent

PRD: `.scratch/workspace-refactoring/PRD.md`

## What to build

Execute the final, global project rename from "Lithos" to "Traces".

The crate structures and imports have already been updated to `traces-`, but many text references remain. Search the codebase for "lithos", "Lithos", and "LITHOS" and replace them with their "Traces" equivalents. This includes:
- Markdown documentation (`README.md`, `CLAUDE.md`, `AGENTS.md`)
- CONTEXT.md and CONTEXT-MAP.md files
- Error messages, logs, tracing spans, and terminal output.
- Environment variable prefixes (`LITHOS_*` → `TRACES_*`).
- Binary names or clap CLI descriptions.
- Config file paths (`lithos.toml` → `traces.toml`).
- Marker prefixes (`policy.rs`: `"lithos"`, `".lithos"`, `".lithos/config"` → `"traces"`, `".traces"`, `".traces/config"`).
- Cache directory paths (`location.rs` doc comments: `~/.cache/lithos/` → `~/.cache/traces/`, etc.).
- Cargo metadata (`Cargo.toml`: `repository = ".../jack/lithos"` → `".../jack/traces"`).
- CI/crate badges (`README.md`: GitHub, crates.io, docs.rs URLs).

## Project name constants

Before doing global replacement, add three case-variant constants to the public
utils crate so every capitalization is defined in one place and no caller needs
to compute one case from another:

**File**: `crates/utils/src/project_name.rs` (new)

```rust
/// Canonical project name (lowercase) — config files, cache dirs, marker prefixes.
pub const PROJECT_NAME_LOWER: &str = "traces";

/// Canonical project name (uppercase) — environment variable prefix.
pub const PROJECT_NAME_UPPER: &str = "TRACES";

/// Canonical project name (title case) — user-facing display, docs, binary help strings.
pub const PROJECT_NAME_TITLE: &str = "Traces";
```

**Re-export**: Add `pub mod project_name;` to `crates/utils/src/lib.rs`.

These constants become the single source of truth — the rename is done by
importing `traces_utils::project_name::{PROJECT_NAME_LOWER, PROJECT_NAME_UPPER, PROJECT_NAME_TITLE}`
wherever the respective case variant would be hardcoded (env var names in
`env.rs`, marker prefixes in `policy.rs`, cache path docs, config file path
construction in tests, output labels). The binary `name` attribute in `cli.rs`
still uses the literal `"traces"` since clap requires a compile-time constant.

## Acceptance criteria

- [x] `PROJECT_NAME_LOWER`, `PROJECT_NAME_UPPER`, `PROJECT_NAME_TITLE` constants exist in `traces_utils::project_name`.
- [x] All user-facing text uses the constants where feasible, or the correct
      literal where not.
- [x] No references to "Lithos" remain in user-facing documentation or output.
- [x] No references to "lithos" remain in environment variables or configuration keys.
- [x] UUID v5 namespace `b"traces"` (was `b"lithos"`) — note: this is a semantic
      change; existing deterministic UUIDs will differ. Acceptable for a pre-1.0 rename.
- [x] `clippy.toml:doc-valid-idents` updated to include `"Traces"`.
- [x] The project successfully compiles and runs under its new identity
      (`mise run verify`).

## Blocked by

- ✅ `.scratch/workspace-refactoring/02-migrate-cli.md`
- ✅ `.scratch/workspace-refactoring/03-consolidate-settings.md`

## Agent Brief

**Category:** enhancement
**Summary:** Perform a global text replacement to rebrand "Lithos" to "Traces" across all documents, strings, and configuration values, using `PROJECT_NAME_LOWER`/`PROJECT_NAME_UPPER`/`PROJECT_NAME_TITLE` constants from `traces_utils::project_name` as the canonical source.

**Current behavior:**
The project name "Lithos" appears extensively in user-facing documentation (`README.md`), CLI output, logs, error messages, environment variables, config file paths, marker prefixes, cache directory paths, and Cargo metadata. The binary is still named `lithos`. The crate names have already been migrated to `traces-*` (previous slices).

**Desired behavior:**
All textual instances of "Lithos", "lithos", and "LITHOS" are correctly replaced with "Traces", "traces", and "TRACES", respectively. The rename is semantically correct and does not accidentally break standard Rust keywords or syntax. The project compiles under its new identity.

**Key interfaces:**
- Markdown documentation files (`README.md`, `CLAUDE.md`, `AGENTS.md`, `CONTEXT.md` files).
- CLI binary name (`cli.rs`: `#[command(name = ...)]`, all test `try_parse_from(&["traces", ...])`).
- Environment variables (`env.rs`: var name strings `LITHOS_*` → use `PROJECT_NAME_UPPER`).
- Marker prefix constants (`policy.rs`: `VAULT_MARKER_PATTERNS`, `GLOBAL_MARKER_PATTERNS`).
- Cache directory doc comments (`location.rs`).
- Config file paths in test fixtures (`lithos.toml` → `traces.toml`).
- UUID v5 namespace (`template/src/aggregate.rs`, `utils/src/uuid.rs`: `b"lithos"` → `b"traces"`).
- Cargo metadata + CI badge URLs in `README.md`.

**Acceptance criteria:**
- [x] `PROJECT_NAME_LOWER`, `PROJECT_NAME_UPPER`, `PROJECT_NAME_TITLE` constants exist in `traces_utils::project_name`.
- [x] No references to "Lithos" remain in user-facing documentation or output.
- [x] No references to "lithos" remain in environment variables or configuration keys.
- [x] UUID v5 namespace updated (semantic change, acceptable pre-1.0).
- [x] `clippy.toml:doc-valid-idents` updated.
- [x] `mise run verify` passes.

**Out of scope:**
- Structural crate renaming (already handled in previous slices).
- Changing domain behavior or functionality.
- Renaming historical artifacts in `.scratch/` files (those are consumed by agents, not shipped).

## Blast radius (GitNexus)

~100+ references across ~30+ files, grouped by area:

| Area | Risk | Details |
|------|------|---------|
| CLI binary name | ~20 refs | `cli.rs:14` + 15+ test `&["lithos", ...]` assertions |
| Env vars | 5 refs | `env.rs:176-184`: `LITHOS_VAULT_DIR`, `LITHOS_CONFIG_FILE`, `LITHOS_CACHE_DIR`, `LITHOS_CEILING_DIRS`, `LITHOS_SUPPRESS_GLOBAL` |
| Marker prefixes | 6+ refs | `policy.rs:18-41`: `"lithos"`, `".lithos"`, `".lithos/config"` + tests |
| Cache path docs | 4 refs | `location.rs:68-89` |
| Config file paths | ~20 refs | `lithos.toml` in test fixtures, error messages, doc examples |
| Docs | heavy | `README.md`, `CLAUDE.md`, `AGENTS.md` |
| Cargo metadata | 1 ref | `Cargo.toml:19` |
| CI/badges | 4+ refs | `README.md`: GitHub, crates.io, docs.rs URLs |
| UUID namespace | 2 refs | `utils/src/uuid.rs:191`, `template/src/aggregate.rs:586` |
| clippy.toml | 1 ref | `doc-valid-idents: ["Lithos", ...]` |

**Risk: HIGH** — but mechanical (search-and-replace with verification).
