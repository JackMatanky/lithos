# Task Plan: Documentation Rationalization and Context Migration

## Goal
Identify still-relevant documentation across `docs/` and `_bmad-output/`, define what to keep/archive/remove, and produce a clear target docs structure plus `CONTEXT.md`/`CONTEXT-MAP.md` plan aligned with Matt Pocock skills.

## Current Phase
Phase 5

## Phases

### Phase 1: Inventory and Discovery
- [x] Capture top-level documentation landscape
- [x] Confirm current skill configuration expectations (`docs/agents/*.md`)
- [x] Enumerate markdown files in `docs/` and `_bmad-output/`
- **Status:** complete

### Phase 2: Relevance Classification
- [x] Classify docs into keep / migrate / archive / delete candidates
- [x] Separate operational references from historical BMAD artifacts
- [x] Identify minimum authoritative sources for architecture, process, and standards
- **Status:** complete

### Phase 3: Context Model Design
- [x] Propose `CONTEXT-MAP.md` and per-context `CONTEXT.md` layout
- [x] Map current authoritative docs into each context
- [x] Define consumer rules for skills and humans
- **Status:** complete

### Phase 4: Docs Folder Information Architecture
- [x] Propose a target `docs/` taxonomy that reduces bloat
- [x] Define migration plan from current paths to target paths
- [x] Define archival policy for `_bmad-output/` and stale docs
- **Status:** complete

### Phase 5: Delivery Artifacts
- [x] Provide concise keep/archive matrix
- [x] Provide concrete first-pass cleanup checklist
- [x] Offer to implement file moves/creation in a follow-up step
- **Status:** in_progress

### Phase 6: Context Refinement First (Execution Order Lock)
- [x] Refine `CONTEXT-MAP.md` with canonical context names and relationships
- [x] Refine module `CONTEXT.md` files with vocabulary, invariants, and boundaries
- [x] Confirm infrastructure designation under Relationships (`DB`, `FS`)
- **Status:** complete

### Phase 7: Docs Reorganization Second
- [ ] Reorganize `docs/` guided by refined context boundaries
- [ ] Preserve `docs/project-genesis/**` as historical-only with clear markers
- [ ] Update indexes and links to point at the refined context model
- **Status:** in_progress

## Key Questions
1. Which docs are authoritative now vs historical snapshots?
2. What should be first-class for day-to-day engineering navigation?
3. How should multi-context docs map to the Rust workspace and boundaries?

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use `planning-with-files` working memory files in repo root | Task spans many files and requires persistent analysis context |
| Treat this step as analysis/design first, no destructive file moves yet | Safer migration path; user asked for help identifying relevance first |
| Execution order is fixed: refine context docs first, then reorganize `docs/` | Prevents rework and ensures cleanup uses stable domain boundaries |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| Session catchup script produced no output | 1 | Proceeded with direct repository inventory and logging |

## Notes
- Update phase status as analysis and recommendations firm up.
- Keep proposals reversible and low-risk.
