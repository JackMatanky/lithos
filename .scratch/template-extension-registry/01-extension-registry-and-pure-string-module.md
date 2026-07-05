---
title: 01-extension-registry-and-pure-string-module
category: feature
label: ready-for-agent
status: open
---

## Parent

`.scratch/template-extension-registry/PRD.md`

## What to build

Introduce the foundational `ExtensionRegistry` in the Application layer (`TemplateService`) and wire it into the Infrastructure layer (`MiniJinjaEngine`). The registry must be able to load pure functions and filters into the engine.

To prove the registry works end-to-end without mocking, implement the `str.*` module and register it. The engine should successfully parse and apply these pure extensions during a normal template render.

*(For the exhaustive list of `str.*` filters to implement, refer to `.scratch/template-extension-registry/planned-extensions.md`)*

## Acceptance criteria

- [ ] `ExtensionRegistry` struct exists in the application layer and is injected into `MiniJinjaEngine::new`
- [ ] `str.*` module is implemented with pure string transformations
- [ ] The engine correctly evaluates `str` filters (e.g., `{{ "text" | str.slugify }}`) in rendered templates
- [ ] Tests verify that the `ExtensionRegistry` correctly loads the `str` extensions into `MiniJinjaEngine`

## Blocked by

None - can start immediately
