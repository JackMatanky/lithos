# findings.md — Template Module Deletion Research

## External Rust References (Code Dependencies)

| File | Reference | Action |
|------|-----------|--------|
| `lithos-core/src/lib.rs:27` | `pub mod template;` | Remove line |
| `lithos-core/benches/string_construction.rs:171` | imports `template::aggregate::{Template, TemplateName}` | Remove bench cases |
| `lithos-core/benches/string_construction.rs:177-178` | `TEMPLATES_TABLE` constant | Remove with cases |

**All 12 `use crate::template::*` references are internal to the module itself** — no external crate imports template types.

## GitNexus Findings

- **Processes**: 0 execution flows involve the template module (it's entirely self-contained)
- **Impact analysis**: On `Template` struct — no upstream consumers outside template module
- **Index**: 20,439 nodes, 300 flows; template module appears as "Template" cluster

## Domain Docs Dependencies

| File | Content | Action |
|------|---------|--------|
| `CONTEXT-MAP.md:9` | `[Template](./lithos-core/src/template/CONTEXT.md)` entry | Mark `_(planned)_` |
| `CONTEXT-MAP.md:19` | `Template -> Schema` relationship | Mark `_(planned)_` |
| `CONTEXT-MAP.md:23` | `Config -> Template` relationship | Mark `_(planned)_` |
| `CONTEXT-MAP.md:24` | `CLI -> ... Template` relationship | Mark `_(planned)_` |
| `CONTEXT-MAP.md:26-27` | `... Template -> DB, ... Template -> FS` | Mark `_(planned)_` |
| `CONTEXT-MAP.md:30` | `... Template, ... -> Utils` | Mark `_(planned)_` |
| `CONTEXT-MAP.md:40` | Segregated Repository includes Template | Mark `_(planned)_` |

## Docs That Reference Template Concept (NOT module code — keep these)

These mention "templates" as a product feature, not the Rust module:
- `docs/adr/discovery/*.md`, `docs/architecture/*.md`, `docs/specs/prd.md`, `docs/superpowers/specs/*` — refer to templating system concept
- `schema/examples/config-*.toml` — config keys (`templates_dir`)

## Key Reference

`docs/superpowers/specs/2026-05-07-template-redesign-design.md` — the authoritative design doc for the rebuild. Spec says:
- Delete: `command.rs`, `query.rs`, `events.rs`, `block.rs` (outdated abstractions)
- Rebuild around: file-centric, typestate pipeline, minijinja delegation, interactive runtime
- Keep: `TemplateName`, `InputName` domain constraints, zero-copy intent
