---
labels: ["needs-triage"]
---

## Parent

None

## What to build

Bridge the declarative template domain with imperative runtime execution by introducing the `TemplateRuntime` object. This object will be injected into `minijinja` as the `li` variable. Implement the `li.suggester()` capability to allow the rendering engine to pause and delegate interactive prompts to a mockable `InteractiveHost`, proving that mid-render dynamic interactions work.

## Acceptance criteria

- [ ] `InteractiveHost` trait is defined to abstract prompt/UI logic.
- [ ] `TemplateRuntime` implements `minijinja::Object` and exposes the `suggester` method.
- [ ] Engine adapter successfully injects the `TemplateRuntime` as `li`.
- [ ] Test with a mock `InteractiveHost` proves that `{{ li.suggester(['a', 'b']) }}` pauses execution, resolves the mock choice, and correctly outputs the selected string.

## Blocked by

- .scratch/template/03-declarative-frontmatter.md
