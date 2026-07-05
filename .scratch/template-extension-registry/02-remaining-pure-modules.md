---
title: 02-remaining-pure-modules
category: feature
label: ready-for-agent
status: open
---

## Parent

`.scratch/template-extension-registry/PRD.md`

## What to build

Flesh out the rest of the pure standard library in the `ExtensionRegistry`. Implement the remaining pure modules: `date.*`, `path.*`, `num.*`, and `id.*`.

This slice proves that the registry can horizontally scale to support multiple pure namespaces effortlessly.

*(For the exhaustive list of functions, filters, and tests required for each module, refer to `.scratch/template-extension-registry/planned-extensions.md`. Note that the `id.*` module for UUID generation must also be included based on the PRD).*

## Acceptance criteria

- [ ] `id.uuid()` generates valid unique identifiers
- [ ] `date.*` module functions and filters evaluate correctly
- [ ] `path.*` module functions and filters evaluate correctly
- [ ] `num.*` module filters evaluate correctly
- [ ] Integration tests verify that templates using multiple pure namespaces render successfully

## Blocked by

- `.scratch/template-extension-registry/01-extension-registry-and-pure-string-module.md`
