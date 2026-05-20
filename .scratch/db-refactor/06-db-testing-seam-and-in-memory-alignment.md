---
title: 06-db-testing-seam-and-in-memory-alignment
category: enhancement
label: needs-triage
status: open
date_created: 2026-05-20
---

## Type

AFK

## Labels

- needs-triage

## What to build

Establish a shared DB testing seam that standardizes in-memory testing
infrastructure across contexts while keeping context Repository Adapters local.

This slice is complete when Schema, Note, Template, and Config migrations can
rely on the same testing primitives and error semantics for in-memory adapters.

## Agent Brief (v1 - 2026-05-20)

**Category:** enhancement
**Summary:** Add `db::testing` primitives and alignment constraints for
context-local in-memory Repository Adapters.

**Current behavior:**
In-memory testing adapters are inconsistent across contexts (Schema,
Config, Template, Note), increasing risk of semantic drift in lock handling,
instrumentation, and failure injection.

**Desired behavior:**
1. Add a `db::testing` module with infra-only test primitives (no context
   business semantics).
2. Introduce `InMemoryDbError` for shared in-memory testing failures.
3. Provide failure injection and operation counters usable by all context
   in-memory adapters.
4. Document and enforce the rule: contexts own their in-memory Repository
   Adapter semantics; `db::testing` provides shared testing infrastructure.

**Key interfaces:**
- `db::testing` module
- `InMemoryDbError`
- Context-local in-memory Repository Adapters

**Acceptance criteria:**
- [ ] `db::testing` module exists with reusable infra primitives.
- [ ] `InMemoryDbError` exists and is used for shared in-memory testing
      failures.
- [ ] Failure injection and operation counters are available to all contexts.
- [ ] Context-level adapter guidance is documented and referenced by follow-up
      migration slices.

## Acceptance criteria

- [ ] Shared in-memory testing infra is available in `db::testing`.
- [ ] Context adapters (Schema/Note/Template/Config) can consume the shared
      infra without moving context invariants into DB.
- [ ] Cross-context guidance for in-memory adapter shape is documented.

## Blocked by

- `05-cross-context-interface-depth-review.md`
