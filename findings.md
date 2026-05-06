# Findings & Decisions

## Requirements
- Migrate away from BMAD method.
- Reduce documentation bloat and improve discoverability.
- Determine what in `docs/` and `_bmad-output/` is still relevant.
- Define `CONTEXT.md` approach per Matt Pocock skills.
- Propose a `docs/` structure that is clear and resistant to bloat.

## Research Findings
- `AGENTS.md` is currently the main repo-level agent context file.
- `docs/agents/` exists and is currently configured for:
  - local markdown issue tracker (`.scratch/<feature>/`),
  - canonical triage labels,
  - multi-context domain docs via `CONTEXT-MAP.md`.
- `docs/adr/` is populated and likely still core architectural source-of-truth.
- `docs/` has many categories: ADRs, references, design docs, CI/testing, operations, plus research and archive-like areas.
- `_bmad-output/` is extensive and appears to contain both:
  - potentially valuable architecture/context docs,
  - and high-volume process artifacts (epics, story reviews, validation reports, retros, ATDD checklists).
- `docs/index.md` is BMAD-linked and points heavily into `_bmad-output`, which makes current navigation brittle/stale.
- High-signal BMAD documents are concentrated in:
  - `_bmad-output/project-context.md`
  - `_bmad-output/planning-artifacts/prd.md`
  - `_bmad-output/planning-artifacts/architecture/03-core-architectural-decisions.md`
  - `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md`
  - `_bmad-output/planning-artifacts/architecture/05-project-structure-boundaries.md`
- High-signal long-term docs in `docs/` are concentrated in:
  - `docs/adr/**`
  - `docs/refs/rust/**`
  - `docs/refs/crates/**`
  - `docs/ci/README.md`
  - `docs/design/README.md` + `docs/design/template.md`
- Most `_bmad-output/implementation-artifacts/**`, `_bmad-output/test-artifacts/**`, and `_bmad-output/planning-artifacts/epics/**` are better treated as historical archive material.
- All module `CONTEXT.md` files now share a consistent refinement pattern: canonical vocabulary, invariants, and explicit non-ownership terms to reduce cross-context drift.

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Use a keep/migrate/archive/delete-candidate classification | Enables cleanup without losing potentially useful historical knowledge |
| Treat `docs/adr/` as authoritative architecture baseline | ADRs are decision records and map well to long-term relevance |
| Preserve BMAD artifacts in an archive boundary initially, not immediate deletion | Reduces risk while enabling a cleaner primary docs experience |
| Use multi-context docs with `docs/CONTEXT-MAP.md` + per-context `docs/contexts/*/CONTEXT.md` | Matches repo architecture and Matt Pocock skill expectations while reducing cognitive load |
| Place `CONTEXT-MAP.md` at project root | Aligns with grill-with-docs conventions for multi-context repositories |
| Place each `CONTEXT.md` inside each module folder | Keeps domain context nearest module boundaries and reduces navigation friction |
| Keep `docs/project-genesis/**` | User explicitly wants early ideation artifacts retained as historical reference |
| Execute in strict order: refine `CONTEXT-MAP.md` + module `CONTEXT.md` first, then reorganize `docs/` | Context boundaries should drive information architecture, not vice versa |
| In `CONTEXT-MAP.md`, list `DB` and `FS` as full contexts, but mark them as infrastructure under Relationships | Preserves explicit ownership while clarifying dependency role |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| Very large documentation surface area | Use phased classification and only deep-read high-signal files first |

## Resources
- `AGENTS.md`
- `docs/adr/`
- `docs/index.md`
- `docs/design/README.md`
- `docs/refs/README.md`
- `_bmad-output/project-context.md`
- `_bmad-output/planning-artifacts/architecture/03-core-architectural-decisions.md`
- `_bmad-output/planning-artifacts/architecture/04-implementation-patterns-consistency-rules.md`
- `_bmad-output/planning-artifacts/architecture/05-project-structure-boundaries.md`
- `docs/ci/README.md`
- `docs/testing/README.md`

## Visual/Browser Findings
- Not applicable in this session.

---
*Update this file after every 2 view/browser/search operations*
*This prevents visual information from being lost*
