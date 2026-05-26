---
title: "Issue 01: FsFile in PropertyBankProcessor Root"
labels: needs-triage
status: draft
created: 2026-05-27
---

# Issue 01: FsFile in PropertyBankProcessor Root

**Parent:** `.scratch/property-bank-processor-deepening/PRD.md` (Candidate 1)

## What to build

Add `file: FsFile` as a root-level field to `PropertyBankProcessor<P, S>`. All status methods derive file identity and metadata from the processor root instead of receiving them as external arguments:

- `self.file.path().as_path()` replaces `config_path: &Path` args
- `self.file.path().as_key(&vault_root)` produces `PathKey` once at construction
- `self.file.metadata()` provides `FileMetadata` for content-freshness comparison

`PropertyBankProcessor::from_fs_file(fs_file)` is the public constructor. `transition()` stays internal for state transitions only.

### Status structs

`PropertyBankProcessor` status structs (`Missing`, `Present`, `Suspect`, `Stale*`, `New`, `Changed`, etc.) keep `FileMetadata` for content-freshness comparison only — they do NOT carry `FilePath`. File identity (`FilePath`) lives at the processor root.

## Acceptance criteria

- [ ] `PropertyBankProcessor<P, S>` gains `file: FsFile` private field
- [ ] `PathKey` is derived once at construction via `file.path().as_key()` — never `PathKey::try_new()`
- [ ] `config_path: &Path` is removed from all method signatures (every `persist`, `parse`, `create`, `update`, `fetch`, `sync_metadata`)
- [ ] `path_key: &PathKey` is removed from all method signatures
- [ ] All `builder.rs` integration tests pass unchanged (lines 301–394)
- [ ] `transition()` retains its internal-only signature — no `file` arg added
- [ ] `from_fs_file()` is the only public constructor

## Blocked by

None — can start immediately.

## Implementation notes

- `FsFile` is 24–40 bytes — pass by `&self.file`, never clone
- `transition()` creates `PhantomData<P>` stage, not `file` — `file` is set once at construction, never during transitions
- All status methods: `&self.file.path()` instead of `config_path`, `&self.file.metadata()` instead of receiving `FileMetadata`
- `PathKey` stored once — not per-method
- `FileMetadata` stays in status for content freshness; `FilePath` is identity, lives at processor root