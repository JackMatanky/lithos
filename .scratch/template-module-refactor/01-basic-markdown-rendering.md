---
labels: ["needs-triage"]
---

## Parent

None

## What to build

Remove obsolete CQRS abstractions (`command.rs`, `query.rs`, `events.rs`, `block.rs`) from the template module. Implement the basic `Template` aggregate to act as the domain boundary. Build a lightweight frontmatter extractor that isolates the raw Markdown body. Create the `minijinja` engine adapter, configured specifically for raw Markdown (auto-escaping disabled, strict undefined behavior). Finally, establish the core end-to-end flow from string input to rendered string output, proving the engine generates clean Markdown.

## Acceptance criteria

- [ ] Obsolete CQRS files are deleted from `@lithos-core/src/template/`.
- [ ] `Template` aggregate is defined and decoupled from `minijinja`.
- [ ] A local extractor cleanly splits Markdown body from frontmatter (ignoring frontmatter data for now).
- [ ] `minijinja` engine adapter is implemented and does not auto-escape Markdown characters (like `<` or `>`).
- [ ] End-to-end test successfully compiles and renders a basic Markdown template.

## Blocked by

None - can start immediately
