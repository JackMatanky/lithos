---
title: "Issue 04: Remove Discovery Dead Stage"
labels: needs-triage
status: draft
created: 2026-05-27
---

# Issue 04: Remove Discovery Dead Stage

**Parent:** `.scratch/property-bank-processor-deepening/PRD.md` (Candidate 3)

## What to build

Remove the `Discovery` stage marker from `PropertyBankProcessor`. The `Discovery` stage only has `new()` and `Default::default()` — no `discover()` method. `DiscoveryEngine::run()` already does all discovery. Remove the stage; processor starts at `Comparison`.

- `PropertyBankProcessor<Discovery, Unknown>` → `PropertyBankProcessor<Comparison, Unknown>` — no `Discovery` stage
- `Discovery` mod removed from `property_bank_processor.rs`
- All code paths start at `Comparison` — no TWO stage parameters

## Acceptance criteria

- [ ] No `PropertyBankProcessor<Discovery, Unknown>` exists in any code path
- [ ] `Discovery` stage marker removed from `property_bank_processor.rs`
- [ ] `PropertyBankProcessor::new()` starts at `Comparison` — not `Discovery`
- [ ] `builder.rs` does not reference `Discovery` stage

## Blocked by

- `.scratch/property-bank-processor-deepening/issues/01-fsfile-processor-root.md` (needs `FsFile` root)
- `.scratch/property-bank-processor-deepening/issues/03-property-bank-discovery-fsfile.md` (needs `FsFile` in discovery)

## Implementation notes

- `Discovery` stage has no methods — only `new()` and `Default::default()` (both no-ops)
- `DiscoveryEngine::run()` does all discovery — the stage is redundant
- After removal: `PropertyBankProcessor` has 1 stage parameter (`Comparison` → `Analysis` → `Refresh` → `Construction` → `Completed`)
- Simplifies the dual-typestate from 2 stage params to 1

## Test

Verify no code path references `PropertyBankProcessor<Discovery, Unknown>` after removal.
