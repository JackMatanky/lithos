---
title: 05-stalereferences-targeted-reexpand-and-id-stability
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-06-01
---

## Type

AFK

## Labels

- base-schema
- ready-for-agent

## Parent

- `.scratch/base-schema/PRD.md`

## What to build

Add `StaleReferences` handling that reacts to property bank deltas by re-expanding only affected schema properties.

This slice uses `SchemaVersion.bank_references` and `changed_bank_references(&bank_delta)` to identify impacted schema properties and preserves `PropertyId` stability by name during targeted updates.

## Acceptance criteria

- [ ] Processor computes affected schema properties using `RawSchemaView.current()` and `SchemaVersion.changed_bank_references(...)`.
- [ ] `StaleReferences` is treated as orthogonal to file staleness (including fresh file + changed bank path).
- [ ] Targeted re-expansion updates only affected properties; unaffected properties are not rebuilt.
- [ ] `PropertyId` values are preserved by property name across targeted re-expansion.
- [ ] Structural reference conflict path escalates to full rebuild fallback.
- [ ] Tests cover fresh+staleRefs, staleTimestamps+staleRefs, staleContent+staleRefs, and property ID stability.

## Blocked by

- `.scratch/base-schema/04-base-processor-stale-analysis-and-normalization.md`
