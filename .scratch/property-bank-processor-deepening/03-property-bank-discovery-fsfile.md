---
title: "Issue 03: PropertyBankDiscovery entry → FsFile"
labels: needs-triage
status: draft
created: 2026-05-27
---

# Issue 03: PropertyBankDiscovery entry → FsFile

**Parent:** `.scratch/property-bank-processor-deepening/PRD.md` (Supporting change)

## What to build

Change `PropertyBankDiscovery` (`lithos-core/src/schema/discovery.rs:78`) so its `entry` field is `FsFile` instead of `FsEntry`. All callers get a compile-time guarantee that the discovery result is a file — no runtime `.as_file()` check.

- `PropertyBankDiscovery.entry: FsEntry` → `FsFile`
- `PropertyBankDiscovery.entry()` returns `&FsFile` — no `Option::unwrap` or `.as_file()` needed
- `builder.rs:170` changes from `bank_discovery.entry().metadata().clone()` to `bank_discovery.entry().metadata()` — same result, no clone

## Acceptance criteria

- [ ] `PropertyBankDiscovery.entry` is `FsFile` (not `FsEntry`)
- [ ] `entry()` returns `&FsFile` (not `Option<&FsFile>`)
- [ ] No `.as_file()` call exists in any builder code path
- [ ] `builder.rs` tests pass with no runtime check

## Blocked by

- `.scratch/property-bank-processor-deepening/issues/01-fsfile-processor-root.md` (needs `PropertyBankProcessor` to prove `FsFile` is the correct root file type)

## Implementation notes

- `FsFile` already exists in the codebase — no new type
- `PropertyBankDiscovery` is the **only** type that uses `FsEntry` at this boundary
- `FsEntry::as_file()` → `Option<&FsFile>` — this was the runtime check; after this change it's `&FsFile` directly
- `DiscoveryEngine::run()` already constructs `PropertyBankDiscovery` with a file — it just needs to wrap `FsFile` instead of `FsEntry`

## Test

No new tests — existing `builder.rs` tests verify behavior. `PropertyBankDiscovery` is a discovery-only type.