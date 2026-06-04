# task_plan.md — Delete `@lithos-core/src/template/` Module

## Goal
Fully delete the existing `lithos-core/src/template/` module (all 14 sub-modules) and all external references, so the module can be rebuilt from scratch per `docs/superpowers/specs/2026-05-07-template-redesign-design.md`.

## Phase 1 — Research & Impact Assessment
- [x] Explore template module structure (14 entries: adapter/, aggregate.rs, block.rs, catalog.rs, command.rs, CONTEXT.md, error.rs, events.rs, mod.rs, query.rs, raw.rs, repository.rs, storage/, value.rs)
- [x] Find all external Rust references (mod declaration, import statements)
- [x] Find all doc/config references (CONTEXT-MAP.md, ADRs, specs)
- [x] Check execution flows in GitNexus

## Phase 2 — Dependency Resolution
- [ ] Remove `pub mod template;` from `lithos-core/src/lib.rs`
- [ ] Remove template benchmark code from `lithos-core/benches/string_construction.rs`
- [ ] Update `CONTEXT-MAP.md` — mark Template context entries as `_(planned)_`
- [ ] Clean up any remaining doc references to the old module structure

## Phase 3 — Deletion
- [ ] Delete `lithos-core/src/template/` directory entirely
- [ ] Run `cargo check` to verify no broken references
- [ ] Run `mise run test` to ensure tests pass
- [ ] Run `mise run fmt && mise run lint`

## Decisions
| Decision | Rationale |
|----------|-----------|
| Delete entire directory (not keep files) | Design spec calls for clean-slate rebuild; existing code carries CQRS debt |
| Remove bench code using Template/TemplateName | Bench uses old aggregate types that won't exist after rebuild |
| Update CONTEXT-MAP.md | Must reflect deletion of Template bounded context |

## Errors Encountered
(none yet)
