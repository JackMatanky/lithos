---
title: "Testing Commands"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
scope: "Authorized test entry points via mise"
---

# Testing Commands

All test execution is orchestrated through `mise`.

## Primary commands

- `mise run test` - all tests
- `mise run test:unit` - unit tests
- `mise run test:unit:core` - core crate unit tests
- `mise run test:unit:cli` - CLI crate unit tests
- `mise run test:unit:config` - config context unit tests
- `mise run test:unit:note` - note context unit tests
- `mise run test:unit:schema` - schema context unit tests
- `mise run test:unit:template` - template context unit tests
- `mise run test:unit:db` - db context unit tests
- `mise run test:unit:fs` - fs context unit tests
- `mise run test:integration` - integration tests
- `mise run test:e2e` - end-to-end tests
- `mise run test:coverage` - coverage report
- `mise run test:bench` - benchmarks
- `mise run test:bench:core` - core benchmarks
- `mise run test:bench:cli` - CLI benchmarks
- `mise run test:watch` - watch mode
- `mise run test:burn-in` - repeated runs for flake detection
- `mise run test:changed` - changed-scope testing
- `mise run verify` - full quality gate

## Usage guidance

- Use `test:changed` for fast PR feedback loops.
- Use `verify` before merge-sensitive work.
- Use `burn-in` when diagnosing flakiness.

## Minimal local pre-merge sequence

```bash
mise run quality
mise run test:changed
mise run test
mise run verify
```
