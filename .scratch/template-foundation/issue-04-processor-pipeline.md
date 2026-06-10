# Template Processor Pipeline

Status: ready-for-agent

## Parent

`.scratch/template-foundation/PRD.md`

## What to build

Implement the Template Processor: a typestate ingestion pipeline that scans the configured template directory, compares files against cached `RawTemplateView`s, and produces persisted `Template` aggregates.

Pipeline stages:
1. **Discovery** — scan the template directory for `.md` files using `DirScanner`; produce file paths
2. **Comparison** — load cached `RawTemplateView`s from the repository via batch read; compare by content hash and file metadata to determine fresh/stale/new/deleted
3. **Parsed** — read stale or new files via `FileReader`; produce `RawTemplate` DTOs
4. **Refresh** — update `RawTemplateView` records for changed files
5. **Construction** — resolve or generate `TemplateId` (look up existing by path, or `TemplateId::new()` for new templates); construct `Template` aggregates
6. **Completed** — persist `Template` and updated `RawTemplateView`s to the repository; pipeline ends here

The processor stops at `Completed`. There is no `Compiled` or `Validated` stage — compilability is a live, on-demand engine check, not an ingestion state.

`TemplateId` is resolved once in the Construction stage and carried through to `Completed`, eliminating redundant repository lookups.

## Acceptance criteria

- [ ] Processor typestate stages are defined: `Discovery`, `Comparison`, `Parsed`, `Refresh`, `Construction`, `Completed`
- [ ] No `Compiled` or `Validated` stage exists
- [ ] `TemplateId` is resolved exactly once (Construction stage) and not looked up again in `Completed`
- [ ] File reads use `FileReader`, not raw `std::fs`
- [ ] Directory scanning uses `DirScanner`, scoped to `.md` files
- [ ] Tests cover: fresh (no-op), new file (full construction path), stale content (refresh + re-construction), stale timestamp only (metadata-only refresh without re-construction), deleted-cache entry (removal from repository), batch path comparison correctness

## Blocked by

- `issue-02-config-spec.md`
- `issue-03-repository-traits.md`
