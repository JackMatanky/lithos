# Worktree Merge Plan - Event Sourcing Foundation

## Objective

Merge `feat/eventstore-contract-transactional-append` back into `main` while preserving all changes on both sides since divergence.

## Scope

- Planning-only artifacts in this phase.
- No merge execution until explicit approval.

## Inputs

- Current worktree: `.worktrees/eventstore-contract-transactional-append`
- Divergence base: `f6473a499fd7e8662d8ea10dbb42e1bb8f6840ac`
- Updated issue source: `.scratch/event-sourcing-foundation/03-eventstore-contract-and-transactional-append-semantics.md`

## Phases

1. Gather divergence + file delta from both sides. (`complete`)
2. Identify overlap/conflict risk and manual intervention points. (`complete`)
3. Define merge sequence, validation, and rollback procedures. (`complete`)
4. Present findings for approval. (`in_progress`)
5. Post-approval execution (stage/commit plan artifacts, merge, validate). (`pending`)

## Errors Encountered

- None.

## Approval Gate

- Required before any merge commands are run.
