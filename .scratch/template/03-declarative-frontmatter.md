---
labels: ["needs-triage"]
---

## Parent

None

## What to build

Expand the lightweight parser to fully deserialize the `TemplateFrontmatter` block, specifically extracting `schemas` and `inputs`. Update the typestate pipeline to capture this data and validate these static dependencies. Expose these declarative inputs into the `minijinja` render context so they can be referenced inside the template body.

## Acceptance criteria

- [ ] `TemplateFrontmatter` and `InputSpec` data models are defined.
- [ ] Parser correctly deserializes YAML frontmatter into the domain structures.
- [ ] The typestate pipeline securely stores and loads these dependencies.
- [ ] Render context is successfully injected with default/static values from the frontmatter.
- [ ] Test verifies that a template referencing a declared input renders correctly.

## Blocked by

- .scratch/template/02-template-caching.md
