---
title: "Lithos Testing Docs"
status: "active"
owner: "engineering"
last_updated: "2026-05-06"
source_of_truth:
  - "_bmad-output/test-developer-guide.md"
scope: "Canonical index for active testing documentation"
---

# Lithos Testing Docs

This folder is the active, single-purpose testing documentation surface for Lithos.

## Canonical testing docs

- [Testing Policy](./policy.md)
- [Testing Commands](./commands.md)
- [Unit Testing](./unit.md)
- [Integration Testing](./integration.md)
- [E2E Testing](./e2e.md)
- [Safety and Determinism](./safety-determinism.md)
- [Coverage](./coverage.md)
- [Benchmarks](./benchmarks.md)

## Scope boundaries

- Active guidance lives in this folder.
- `docs/testing/cqrs.md` and `docs/testing/event.md` are legacy and non-authoritative.
- Historical strategy artifacts remain under `_bmad-output/` and are reference-only.

## Maintenance rule

Update this folder first when testing practices change. Do not create parallel active testing guides elsewhere.
