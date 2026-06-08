# Issue 07: Delete `lithos-core::vault`

**Status**: ready-for-human
**Created**: 2026-06-09

## What to build

Remove `lithos-core::vault` once the Indexer storage adapter has passed full
integration tests (issue 03 complete) and the Indexer application service is
proven (issue 04 complete). The Vault module is prior art and migration source;
it is not the long-term API. Keeping it any longer than necessary risks
confusion about which module owns filesystem node state.

The agent should:

1. Audit every import of `lithos-core::vault` across the workspace and
   replace each with the equivalent Indexer type or repository call.
2. Remove the `vault` module declaration from `lithos-core/src/lib.rs`.
3. Delete `lithos-core/src/vault/`.
4. Confirm all tests pass and no dead-code warnings remain.
5. Open the PR for human review before merging — this is a destructive
   migration and warrants explicit sign-off.

## Acceptance criteria

- [ ] `lithos-core::vault` module and all its files are deleted.
- [ ] No remaining imports of `lithos_core::vault` anywhere in the workspace.
- [ ] All 1588+ tests pass (`mise run test`).
- [ ] No clippy warnings (`mise run lint`).
- [ ] PR description explains what each replaced call site was doing and what
      it was replaced with.
- [ ] Human reviewer has approved the PR before merge.

## Blocked by

- issue-03-ports-and-adapters.md
- issue-04-application-service.md
