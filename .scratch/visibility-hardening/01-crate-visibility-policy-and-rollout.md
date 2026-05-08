---
title: 01-crate-visibility-policy-and-rollout
category: enhancement
state: needs-triage
---

# Visibility hardening: crate policy and rollout

## Parent

General visibility hardening refactor for the codebase.

## Related

- `.scratch/hash-index-refactor/issue-001-hash-index-visibility-propagation.md` (hash-specific visibility propagation, implemented)
- `.scratch/hash-index-refactor/issue-002-raw-property-map-emits-hash-index.md`
- `.scratch/hash-index-refactor/issue-003-rename-and-adapt-view-hash-index-wrapper.md`

## What to build

Define and roll out crate-wide visibility guidelines for `lithos-core`, then apply them incrementally across contexts and infrastructure. This issue is policy and governance focused; hash-surface propagation work has been split into hash-index-refactor.

## Acceptance criteria

- [ ] Visibility policy documented with clear defaults and escalation rules:
  - private by default,
  - `pub(crate)` for crate-internal seams,
  - `pub` only for intentional external API.
- [ ] Re-export policy documented (what can be re-exported from module roots and what must stay internal).
- [ ] Review pass completed for context boundaries (`config`, `note`, `schema`, `template`) and infrastructure modules (`db`, `fs`, `support`).
- [ ] Incremental implementation issues created for any violations discovered during review.
- [ ] `cargo clippy -p lithos-core --all-targets -- -D warnings` and `mise run verify` pass after applied changes.

## Out of scope

- Hash-index seam refactors already tracked in `.scratch/hash-index-refactor/`.
- Broad API redesign unrelated to visibility boundaries.

## Implementation plan

1. Draft visibility guideline doc in `docs/` (and ADR if policy decisions are architectural).
2. Audit current exports and signatures by module/context.
3. File focused follow-up issues per area (small, independently shippable slices).
4. Apply changes incrementally with lint/test verification at each slice.

## Follow-up validation

- [ ] Ensure each follow-up issue links back to this parent policy issue.
- [ ] Re-run full `mise run verify` after final rollout slice merges.
