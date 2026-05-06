---
title: "E2E Testing"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "CLI end-to-end behavior and user journeys"
---

# E2E Testing

## Scope

- Validate complete CLI user journeys.
- Assert exit codes, core output behavior, and key filesystem outcomes.

## Primary tooling

- `assert_cmd` for invoking binaries
- `predicates` for robust output assertions
- `tempfile::TempDir` for isolated test vault/workdir

## Patterns

- Use CLI test tooling such as `assert_cmd` and output predicates.
- Build each test with isolated temporary vault/workdir setup.
- Cover critical flows, not every low-level edge case.

## Canonical journeys to cover

- command help/version behavior
- vault initialization and note creation workflows
- indexing/search flows through CLI commands
- clear user-facing failure modes for invalid command usage

## Do not use E2E for

- Internal algorithm edge-case validation (unit scope).
- Fine-grained adapter contract checks (integration scope).
