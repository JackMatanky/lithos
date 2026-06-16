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
