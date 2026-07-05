---
title: 03-effectful-infrastructure-and-file-module
category: feature
label: ready-for-agent
status: open
---

## Parent

`.scratch/template-extension-registry/PRD.md`

## What to build

Deliver the "Pull" model for effectful operations. Define the granular `FileProvider` capability trait in the Application layer and inject an implementation of it into MiniJinja's `State` temporary object cache during rendering.

Modify the template evaluation pipeline to use `render_captured()` so that Side-Channel Commands (such as `file.write_to`) are retained and can be extracted post-render. Implement the `file.*` module, ensuring that `file.write_to` operates silently (returns an empty string) while successfully queuing the target output path for the `TemplateService` to execute.

*(For the specific `file.*` extensions to implement, refer to `.scratch/template-extension-registry/planned-extensions.md`)*

## Acceptance criteria

- [ ] `FileProvider` trait defined in the application layer
- [ ] `TemplateService` injects capability traits into MiniJinja's `State` temps via the `ExtensionRegistry`
- [ ] `TemplateService` extracts Side-Channel Commands post-render using `render_captured()`
- [ ] `file.write_to("path")` evaluates silently (returns `""`) but successfully overrides the artifact's target output path
- [ ] `file.read("path")` successfully reads content using the injected `FileProvider`
- [ ] Missing capabilities or failing effectful operations result in an immediate engine error that aborts rendering

## Blocked by

- `.scratch/template-extension-registry/01-extension-registry-and-pure-string-module.md`
