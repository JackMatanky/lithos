---
title: "Issue 02: Unified persist() HashRecord Source"
labels: needs-triage
status: draft
created: 2026-05-27
---

# Issue 02: Unified persist() HashRecord Source

**Parent:** `.scratch/property-bank-processor-deepening/PRD.md` (Candidate 2)

## What to build

Unify `persist()` so `HashRecord` is always computed from the same `RawPropertyBank` source, regardless of which pipeline path (`New` or `Changed`) created it.

Currently:
- `New::persist` computes from `self.status.raw.properties()` (un-analyzed)
- `Changed::persist` uses `self.status.raw_hash` (analysis-derived)

Move `persist()` to a generic `impl<P, S> PropertyBankProcessor<P, S>` method that takes `(&RawPropertyBank, &PathKey, &WriteRepository)`. Both `New::persist` and `Changed::persist` call the same `persist()` with `&self.status.raw`.

## Acceptance criteria

- [ ] `persist()` is generic over any stage — not tied to `New` or `Changed`
- [ ] `New::persist` and `Changed::persist` both call same `persist()` with `&self.status.raw`
- [ ] Cache view cannot diverge — same `RawPropertyBank` → same `HashRecord`
- [ ] Test: `HashRecord` from `New::persist` matches `HashRecord` from `Changed::persist` for same `RawPropertyBank`

## Blocked by

- `.scratch/property-bank-processor-deepening/issues/01-fsfile-processor-root.md` (needs `file: FsFile` to pass `&RawPropertyBank` uniformly without `config_path` args)

## Implementation notes

- `persist()` signature: `fn persist(&self, raw: &RawPropertyBank, key: &PathKey, repo: &mut impl WriteRepository) -> Result<HashRecord, _>`
- `New::persist` passes `self.status.raw.as_ref()` — the un-analyzed `RawPropertyBank`
- `Changed::persist` passes `self.status.raw` (already present) — not `self.status.raw_hash`
- `HashRecord` computation algorithm does NOT change — only the **source** changes

## Test

One test: `New::persist(raw_property_bank)` and `Changed::persist(raw_property_bank)` produce identical `HashRecord`.
