---
title: 07-integration-and-regression-suite
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

Add the full phase-1 integration and contract verification suite for base schema processing.

This slice creates `lithos-core/tests/base_processor.rs` and `lithos-core/tests/base_schema_handoff_contract.rs` using `InMemoryRepository`-first patterns and verifies mixed lifecycle batches plus stale-reference scenarios.

## Acceptance criteria

- [ ] Integration tests cover cold start, mixed incremental run, and property-bank-driven stale references.
- [ ] Contract tests verify deterministic `SchemaId` ordering and delta payload completeness.
- [ ] Tests verify metadata-only normalization to `Fresh` and deletion cleanup semantics.
- [ ] Tests avoid private implementation coupling and assert via public behavior.
- [ ] The phase-1 acceptance checklist in `.scratch/base-schema/PRD.md` is verifiably satisfiable by test outcomes.

## Blocked by

- `.scratch/base-schema/06-lifecycle-handoff-and-deletion-semantics.md`
