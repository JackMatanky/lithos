---
title: 16-cli-bootstrap-config-inspection
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
date_updated: 2026-06-16
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`
- `.scratch/root-config-discovery/discovery-redesign-decisions.md`
- `docs/adr/024-bootstrapper-orchestration.md`

## What to build

Add the initial CLI surface for inspecting Lithos bootstrap/config loading behavior.

This slice introduces:

- `lithos config`
- `lithos config files`
- `lithos doctor`

It also promotes bootstrap/loading controls to top-level CLI options:

- `--vault <DIR>`
- `--config <FILE>`
- `--no-global-config`
- `--format <human|json>`
- `-v, --verbose...`

These commands expose which vault and config files Lithos resolves before normal command execution. They are meant for user-facing inspection and debugging, not for parsing, validating, hashing, or merging config contents inside CLI handlers.

## Command behavior

### `lithos config`

Reports the effective bootstrap/config loading result:

- resolved vault root
- selected vault config file
- selected global config file, if any
- global config suppression status
- relevant discovery/report warnings

### `lithos config files`

Reports the discovered config files/candidates returned by the bootstrap discovery flow, in deterministic precedence order.

This command always exits `0`, even when no vault root is found. It reports discovered candidates only. It must not claim to enumerate every found/not-found path checked unless Discovery exposes a dedicated trace/enumeration report.

### `lithos doctor`

Runs whole-program diagnostics. For this slice, it includes a bootstrap/config section that checks whether vault/config discovery is healthy.

`doctor` is intentionally top-level, not under `config`.

## Flag design

### `--format` vs `--output`

Use `--format <human|json>` (not `--output`) for selecting output format. Per the CLI guidelines, `-o`/`--output` is the conventional name for an output *file* path (used by `sort`, `gcc`, etc.). Using it to select output *format* creates a naming collision that will confuse users.

`--format` is the correct name for a format selector. The `-f` short form may be used if it does not conflict with another top-level flag.

### `-v, --verbose...`

`-v` is stackable (clap's `action = ArgAction::Count`). Each level adds more diagnostic detail to stderr. Do not use `-v` for version — use `--version` (long only).

### Bootstrap flags as top-level options

`--vault`, `--config`, and `--no-global-config` are top-level CLI options, not subcommand-only flags. This is required because they control how every command resolves its runtime context — they belong at the level they are shared (see cli_guide.md: "Use consistent names for multiple levels of subcommand").

## Bootstrapper boundary

`lithos config` and `lithos doctor` both require a full `Bootstrapper::run()` call (discovery + config build).

`lithos config files` requires only the discovery leg. It must call `Bootstrapper::discover()` (or a dedicated facade method) without triggering config parsing. Do not call `Bootstrapper::run()` in the `config files` handler — doing so would parse config as a side effect.

If no discovery-only method exists on `Bootstrapper`, add one as a separate method rather than adding a parameter to `run()` (see ADR 024: "If `run()` grows additional responsibilities [...] each new responsibility should be added as a separate method, not as a parameter to `run()`").

## Acceptance criteria

- [ ] Top-level options are wired: `--vault <DIR>`, `--config <FILE>`, `--no-global-config`, `--format <human|json>`, and `-v, --verbose...`.
- [ ] `--vault`, `--config`, and `--no-global-config` are modeled as top-level bootstrap/loading options, not as config-subcommand-only flags.
- [ ] `--format` is used for output format selection (not `--output`).
- [ ] `lithos config` reports the effective resolved vault/config loading state.
- [ ] `lithos config files` reports discovered vault/global config candidates in deterministic precedence order.
- [ ] `lithos config files` always exits `0`, including when no vault root is found.
- [ ] `lithos config files` does not report unchecked/not-found paths unless the app/discovery layer exposes that data explicitly.
- [ ] `lithos config files` calls only the discovery leg of `Bootstrapper` — it does not trigger config parsing as a side effect.
- [ ] `lithos doctor` includes bootstrap/config diagnostics but is not implemented as `lithos config doctor`.
- [ ] Structured output goes to stdout; verbose diagnostics go to stderr.
- [ ] CLI handlers use an app-level bootstrap/discovery facade and do not import `Discovery` or `Config` directly.
- [ ] CLI handlers do not parse, validate, hash, merge, or process config file contents.
- [ ] CLI tests cover output shape, top-level flag behavior, precedence behavior, and error mapping.

## Exit codes

- [ ] `0` success.
- [ ] `1` no vault/config loading target found when the command requires one (`lithos config` and `lithos doctor` only; `lithos config files` always exits `0`).
- [ ] `2` invalid explicit path or strict diagnostic failure.
- [ ] `3` permission or filesystem access failure.

## Out of scope

- Config editing.
- Config content validation.
- Indexer integration.
- Task execution / `run`.
- Vault initialization / `init`.
- Full CLI redesign beyond this bootstrap/config inspection surface.
- TOML output format (not part of this surface; remove from any prior criteria that referenced it).
- `--strict` flag (was on the old `config check`; not carried forward here — health check strictness is handled by `doctor` exit code `2`).

## Agent Brief

> *This was generated by AI during triage.*

**Category:** enhancement

**Summary:** Add `lithos config`, `lithos config files`, and `lithos doctor` to expose bootstrap/config loading state at the CLI. Promote `--vault`, `--config`, `--no-global-config`, `--format`, and `--verbose` to top-level options.

**Current behavior:**
Discovery and bootstrap loading behavior is not inspectable from the CLI. Users and developers cannot see which vault root or config files were resolved, or verify that discovery is healthy, without reading source code or debug logs.

**Desired behavior:**
Three commands expose bootstrap/config loading state:
- `lithos config` — effective resolved state (vault root, config files, warnings)
- `lithos config files` — discovered candidates in precedence order (always exits 0)
- `lithos doctor` — whole-program health check including bootstrap diagnostics

Top-level flags (`--vault`, `--config`, `--no-global-config`, `--format`, `--verbose`) are available to all commands. Structured output goes to stdout; diagnostics go to stderr.

**Key interfaces:**
- `Bootstrapper::run()` — full bootstrap for `lithos config` and `lithos doctor`
- `Bootstrapper::discover()` (or a new discovery-only facade method) — for `lithos config files`; must not trigger config parsing
- `DiscoveryFlags` (`vault_dir`, `config_file`, `suppress_global`) — maps to `--vault`, `--config`, `--no-global-config`
- `DiscoveryReport` — surfaced by verbose output; never passed to config
- Exit code taxonomy: `0` success, `1` vault not found (requiring commands only), `2` invalid path or diagnostic failure, `3` permission error

**Rust implementation notes:**
- Use `clap` with `ArgAction::Count` for `-v, --verbose...`
- Use `--format` (not `--output`) for format selection — `--output` conventionally means output file
- `lithos config files` handler must call only `Bootstrapper::discover()` — calling `run()` would parse config as a side effect, violating the "discovery-only" contract
- If `Bootstrapper` does not yet expose a discovery-only public method, add `Bootstrapper::run_discovery_only()` as a separate method per ADR 024's guidance against parameterizing `run()`
- Error types: map `BootstrapError::Discovery(InvalidAnchorDirectory | ...)` to exit `1`, `BootstrapError::Discovery(Override*)` to exit `2`, I/O permission errors to exit `3`

**Acceptance criteria (agent-verifiable):**
- [ ] Command outputs are deterministic and script-friendly.
- [ ] Exit codes map cleanly to bootstrap outcomes; `config files` exits `0` unconditionally.
- [ ] `--format` selects `human` or `json` output; no `--output` flag for format.
- [ ] Top-level flags (`--vault`, `--config`, `--no-global-config`) are available before the subcommand, not only under `config`.
- [ ] Precedence behavior is externally verifiable via CLI tests.
- [ ] CLI handlers respect ADR 024: go through `Bootstrapper`, never import `Discovery` or `Config` directly.
- [ ] `config files` handler does not parse config file contents as a side effect.

**Out of scope:**
- Config content parsing/validation
- Indexer integration
- Broader CLI redesign beyond this bootstrap/config inspection surface
- TOML output format
- `--strict` flag / `config check` subcommand

## Blocked by

- `.scratch/root-config-discovery/15-context-docs-alignment.md`

---

## TDD Plan

### Architecture notes

This plan follows hexagonal architecture. The CLI is an inbound adapter; it never imports from `discovery/` or `config/` directly. All bootstrap access goes through `Bootstrapper` in `lithos-core::app`. Exit code mapping and error presentation live entirely in `lithos-cli` (`CliError`), keeping `BootstrapError` clean of adapter concerns.

CLI command handlers accept injectable `impl Write` for stdout and stderr so output shape can be verified in unit tests without capturing process streams.

### Preconditions

- Issue 15 (`15-context-docs-alignment.md`) must be resolved or explicitly deferred before this work begins.
- `clap` `derive` feature must be added to workspace dependencies.

### Required dependency changes

**`Cargo.toml` (workspace):**
```toml
clap = { version = "4.6", features = ["env", "derive"] }
```

**`lithos-cli/Cargo.toml` `[dev-dependencies]`:**
```toml
mockall = { workspace = true }
pretty_assertions = { workspace = true }
tempfile = { workspace = true }
```

### Visibility changes

All symbols listed below are currently `pub(crate)`. They must become `pub`.

**`lithos-core/src/app/bootstrap.rs`:**
- `Bootstrapper<D>` struct
- `Bootstrapper::new()`
- `Bootstrapper::build_context()`
- `Bootstrapper::run()`
- `Bootstrapper::run_discovery_only()` (new — `pub` from creation)
- `Bootstrapper::from_platform()`
- `Bootstrapper::with_global_directories()`

**`lithos-core/src/app/error.rs`:**
- `BootstrapError` enum

**`lithos-core/src/discovery/mod.rs` — add `pub use` re-exports:**

```rust
pub use context::{DiscoveryEnv, DiscoveryFlags};
pub use report::{
    DiscoveryReport, GlobalResolutionSkipReason, LocalTraversalStopReason,
    SkippedCeiling, SkippedCeilingReason,
};
pub use service::{CandidatePath, DiscoveryResult};
```

The underlying types in `context.rs`, `report.rs`, and `service.rs` also need their structs, enums, and public methods changed from `pub(crate)` to `pub` so the re-exports compile. The `discovery` module in `lib.rs` stays `pub(crate)` — the CLI accesses these types via `lithos_core::discovery::*` only through `lithos-core::app`'s dependency chain; the re-exports are for crate-internal use and to support the `Bootstrapper` public API.

> **Note:** If `lib.rs` keeps `pub(crate) mod discovery`, the `pub use` re-exports in `discovery/mod.rs` will still be `pub(crate)` from the crate boundary. Whether `discovery` itself needs to become `pub` in `lib.rs` depends on whether the CLI needs to name `lithos_core::discovery::DiscoveryFlags` explicitly. If `Bootstrapper`'s public API uses `DiscoveryFlags` as a parameter type, Rust requires the type to be reachable. Either change `pub(crate) mod discovery` → `pub mod discovery` in `lib.rs`, or re-export the required types from `lithos_core::app` directly.

### Slice 1 — `Bootstrapper::run_discovery_only()`

**File:** `lithos-core/src/app/bootstrap.rs`

Add a new `pub` method to `Bootstrapper<D: DiscoveryPort>`:

```rust
pub fn run_discovery_only(
    &self,
    flags: Option<DiscoveryFlags>,
    env: Option<DiscoveryEnv<'_>>,
    anchor: &std::path::Path,
) -> Result<(DiscoveryResult, DiscoveryReport), BootstrapError>
```

This calls `build_context` + `discover()` only. It must not call `Builder`. The "does not trigger config parsing" behavior is verified by a test that feeds invalid TOML to the mock — `run()` would return `BootstrapError::Config`, but `run_discovery_only()` must succeed.

**Tests — `mod run_discovery_only` in `lithos-core/src/app/bootstrap.rs`:**

```
returns_discovery_result_when_port_succeeds()
returns_report_when_port_succeeds()
propagates_discovery_error_when_port_fails()
propagates_discovery_error_from_invalid_anchor()
does_not_return_config_error_when_discovery_result_contains_invalid_toml()
```

### Slice 2 — Visibility + `pub use` re-exports

**Files:** `lithos-core/src/app/bootstrap.rs`, `lithos-core/src/app/error.rs`, `lithos-core/src/discovery/mod.rs`, and all affected `context.rs` / `report.rs` / `service.rs` type/method declarations.

No new logic. Changes are visibility only. Verified by: the CLI crate compiles after each new import is added.

Remove the `#[allow(dead_code, ...)]` attributes on types that are now wired to a live caller.

### Slice 3 — `CliError` in `lithos-cli`

**New file:** `lithos-cli/src/error.rs`

```rust
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(crate) enum CliError {
    #[error(transparent)]
    Bootstrap(#[from] BootstrapError),
}
```

`CliError` owns exit code derivation. It must not call `std::process::exit` (forbidden by lint). The pattern: `main()` returns `miette::Result<()>`; on error, `miette` renders the diagnostic and the process exits 1. For commands requiring exit codes 2 and 3, a custom `Termination` impl or a wrapping runner is required — this detail must be resolved before Slice 5.

**Tests — `lithos-cli/src/error.rs`:**

```
mod conversions {
    converts_bootstrap_error_to_cli_error()
}
mod exit_code {
    returns_1_when_vault_not_found()
    returns_2_when_explicit_path_is_invalid()
    returns_3_when_permission_denied()
}
```

Exit code mapping rules (in `CliError`, not `BootstrapError`):
- `BootstrapError::Discovery(DiscoveryError::InvalidAnchorDirectory { .. })` → 1
- `BootstrapError::Discovery(DiscoveryError::Flag(_) | DiscoveryError::Env(_))` → 2
- `BootstrapError::Discovery(DiscoveryError::ReadDirectory { .. })` → 3
- `BootstrapError::Discovery(DiscoveryError::CanonicalizePath { source, .. })` where `source.kind() == PermissionDenied` → 3
- All others → 2

### Slice 4 — CLI argument structure

**Files:** `lithos-cli/src/main.rs` (replaced) and new `lithos-cli/src/cli.rs`

Use `clap` derive API throughout. Top-level struct:

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, global = true)]
    vault: Option<PathBuf>,
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[arg(long, global = true)]
    no_global_config: bool,
    #[arg(long, global = true, default_value = "human")]
    format: OutputFormat,
    #[arg(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Config(ConfigArgs),
    Doctor,
}

#[derive(Args)]
struct ConfigArgs {
    #[command(subcommand)]
    subcommand: Option<ConfigSubcommand>,
}

#[derive(Subcommand)]
enum ConfigSubcommand {
    Files,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}
```

**Tests — `mod arg_parsing` in `lithos-cli/src/cli.rs`:**

```
returns_vault_flag_value_when_provided()
returns_config_flag_value_when_provided()
returns_no_global_config_when_flag_present()
returns_format_human_by_default()
returns_format_json_when_explicitly_set()
rejects_unknown_format_value()
returns_verbose_count_zero_by_default()
returns_verbose_count_incremented_per_flag()
routes_config_subcommand()
routes_config_files_subcommand()
routes_doctor_subcommand()
vault_flag_is_available_to_config_files_subcommand()
vault_flag_is_available_to_doctor_subcommand()
```

Tests use `Cli::try_parse_from(["lithos", ...])` — no process I/O required.

### Slice 5 — `lithos config` handler

**New file:** `lithos-cli/src/commands/config.rs`

Handler signature:

```rust
pub(crate) fn run_config(
    bootstrapper: &Bootstrapper<DiscoveryService>,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    verbose: u8,
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<(), CliError>
```

Handler calls `Bootstrapper::run()`. Structured output → `out`. Verbose diagnostics (skipped ceilings, stop reason) → `err`.

**Tests — `mod config_handler` in `lithos-cli/src/commands/config.rs`:**

```
mod fixtures { /* mock bootstrapper helpers */ }

mod config_handler {
    returns_resolved_vault_root_in_human_format()
    returns_resolved_vault_root_in_json_format()
    includes_global_config_suppression_status_when_no_global_config_set()
    writes_skipped_ceiling_warning_to_stderr_when_verbose()
    writes_structured_output_to_stdout_writer()
    writes_verbose_diagnostics_to_stderr_writer()
    honours_vault_flag_override()
    honours_config_flag_override()
    returns_err_when_vault_not_found()
    returns_err_when_explicit_vault_path_invalid()
    returns_err_when_permission_denied()
}
```

Handler tests use `MockDiscoveryPort` (same pattern as `bootstrap.rs` tests). Output shape is asserted on captured `Vec<u8>` writers.

### Slice 6 — `lithos config files` handler

**New file:** `lithos-cli/src/commands/config_files.rs`

Handler signature:

```rust
pub(crate) fn run_config_files(
    bootstrapper: &Bootstrapper<DiscoveryService>,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    out: &mut impl Write,
) -> Result<(), CliError>
```

Handler calls `Bootstrapper::run_discovery_only()`. Always returns `Ok(())` — errors are written to `out` as empty/warning output, not propagated. Output lists `vault` candidates then `global` candidates in that order.

**Tests — `mod config_files_handler` in `lithos-cli/src/commands/config_files.rs`:**

```
returns_vault_candidates_in_precedence_order()
returns_global_candidates_after_vault_candidates()
returns_empty_output_when_no_candidates_found()
always_returns_ok_when_no_vault_found()
always_returns_ok_when_discovery_error_occurs()
calls_run_discovery_only_not_run()
returns_candidates_in_json_format_when_format_json()
honours_vault_flag_override()
```

The `calls_run_discovery_only_not_run()` test is enforced structurally: the handler does not have access to `run()` by design (it only receives a reference that exposes `run_discovery_only`), or is verified by injecting a mock that panics if `run()` is called.

### Slice 7 — `lithos doctor` handler

**New file:** `lithos-cli/src/commands/doctor.rs`

Handler signature:

```rust
pub(crate) fn run_doctor(
    bootstrapper: &Bootstrapper<DiscoveryService>,
    flags: Option<DiscoveryFlags>,
    anchor: &Path,
    format: OutputFormat,
    verbose: u8,
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<(), CliError>
```

Handler calls `Bootstrapper::run()`. Reports a bootstrap/config section summarising vault root, config files, and any warnings. `doctor` is registered as a top-level subcommand, not under `config`.

**Tests — `mod doctor_handler` in `lithos-cli/src/commands/doctor.rs`:**

```
reports_healthy_when_bootstrap_succeeds()
reports_vault_not_found_section_when_no_vault_root()
writes_bootstrap_section_to_output()
returns_err_when_bootstrap_fails_vault_not_found()
returns_err_when_bootstrap_fails_invalid_explicit_path()
returns_err_when_bootstrap_fails_permission_denied()
is_registered_as_top_level_subcommand_not_under_config()
```

### Slice 8 — Wire `main()`

**File:** `lithos-cli/src/main.rs`

`main()` parses `Cli`, constructs `Bootstrapper::from_platform()`, reads CWD, builds `DiscoveryFlags` from parsed top-level args, then dispatches to the appropriate handler. Errors are returned as `miette::Result` for diagnostic rendering.

`main()` itself is kept minimal — all logic lives in handler functions. The testing dead zone is bounded to construction and dispatch only.

### Test coverage matrix

| Acceptance criterion | Slice |
| --- | --- |
| Top-level flags wired (`--vault`, `--config`, `--no-global-config`, `--format`, `--verbose`) | 4 |
| `--format` not `--output` | 4 |
| `lithos config` reports resolved state | 5 |
| `lithos config files` reports candidates in order | 6 |
| `lithos config files` always exits 0 | 6 |
| `lithos config files` no unchecked paths | 6 |
| `lithos config files` calls discovery-only leg | 1 + 6 |
| `lithos doctor` is top-level, not under `config` | 4 + 7 |
| Structured output → stdout; verbose → stderr | 5 + 7 |
| Handlers don't import `Discovery` or `Config` directly | Compile-time |
| No config content parsing as side effect | 1 + 6 |
| Exit codes 0 / 1 / 2 / 3 | 3 |
| Output shape verifiable | 5 + 6 + 7 |

### Definition of done

- [ ] All slices implemented in RED → GREEN order (no horizontal slicing).
- [ ] `mise run test` passes.
- [ ] `mise run lint` passes with no `#[allow]` without `reason`.
- [ ] `mise run fmt` passes.
- [ ] All public APIs have doc comments with `# Errors` sections.
- [ ] No `unwrap()`/`expect()` outside test `Arrange` phases.
- [ ] `#[allow(dead_code, ...)]` removed from all newly-wired types.
