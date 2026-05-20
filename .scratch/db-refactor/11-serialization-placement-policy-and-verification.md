---
title: 11-serialization-placement-policy-and-verification
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

Define and validate a final policy for serialization placement relative to
write transaction closures in Repository Adapters.

This slice is complete when the policy is explicit, implemented where needed,
and verified against correctness and performance concerns after cross-context
migration.

## Agent Brief (v1 - 2026-05-20)

**Category:** enhancement
**Summary:** Finalize serialization placement policy and verification.

**Current behavior:**
Some write paths (for example schema batch save) perform serialization inside
`Store::write` closures. Behavior is correct, but placement policy is not yet
standardized across contexts.

**Desired behavior:**
1. Define clear serialization-placement policy for write paths.
2. Apply policy consistently across migrated context Repository Adapters.
3. Validate correctness (atomicity/invariants) and performance impact.

**Key interfaces:**
- `db/codec.rs` (`ArchivedEntity`)
- `db/core.rs` (`Store::write`)
- Context write adapters in schema/note/template/config

**Acceptance criteria:**
- [ ] Serialization placement policy is documented with rationale.
- [ ] Policy is reflected in code where required.
- [ ] Verification includes correctness and performance considerations.

## Acceptance criteria

- [ ] Policy for serialization placement is documented and approved.
- [ ] Context write adapters align with policy.
- [ ] Final verification confirms no invariant regressions.

## Blocked by

- `10-cross-context-verification-and-legacy-cleanup.md`
