---
title: 08-cli-discovery-subcommands
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
---

## Type

AFK

## Labels

- root-config-discovery
- ready-for-agent

## Parent

- `.scratch/root-config-discovery/PRD.md`

## What to build

Add a minimal discovery-focused CLI surface in `lithos-cli` to exercise and verify root/config discovery behavior directly during development.

This slice introduces `config where`, `config list-sources`, and `config check` with deterministic outputs, warning visibility, and scripting-friendly exit semantics.

## Acceptance criteria

- [ ] `lithos config where` reports resolved vault root, discovered config files, source/location metadata, and warnings.
- [ ] `lithos config list-sources` enumerates all candidate paths checked (found/not-found) in deterministic order and always exits 0.
- [ ] `lithos config check` validates discovery outcomes and supports strict warning escalation mode.
- [ ] Flags are wired: `--vault`, `--config`, `--no-global-config`, `--format`, `--verbose`, `--strict`.
- [ ] Exit codes are deterministic: success, root-not-found, explicit-path-invalid, permission-error.
- [ ] Human-readable and JSON output modes are supported for discovery commands.
- [ ] Verbose/trace diagnostics are emitted to stderr while structured output remains stdout-friendly.
- [ ] CLI tests validate command behavior, exit codes, and precedence semantics.

## Blocked by

- `.scratch/root-config-discovery/06-phase-2-local-config-discovery.md`
