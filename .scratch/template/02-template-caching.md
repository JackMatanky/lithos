---
labels: ["needs-triage"]
---

## Parent

None

## What to build

Implement the rigid `TemplateProcessor` typestate pipeline to ensure that templates are effectively cached and only re-parsed when file metadata or contents change. Define the `redb` tables (`RAW_TEMPLATE_VIEWS`, `TEMPLATE_ID_BY_PATH`, `TEMPLATES`) and the zero-copy `RawTemplateView` utilizing `rkyv`. The pipeline should correctly navigate the transitions from `Discovery` to `Comparison` to `Construction`, preventing the execution engine from receiving stale or un-parseable templates.

## Acceptance criteria

- [ ] `redb` table definitions and `RawTemplateView` (with `rkyv`) are implemented in `cache.rs`.
- [ ] `TemplateProcessor` typestate pipeline is implemented with explicit states (`Discovery`, `Comparison`, `Parsed`, `Refresh`, `Construction`).
- [ ] Tests verify that the pipeline correctly transitions to fetch the cached template when `mtime` matches.
- [ ] Tests verify that the pipeline correctly re-parses the template when content hash diverges.

## Blocked by

- .scratch/template/01-basic-markdown-rendering.md
