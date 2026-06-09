# Template Interactive Prompt Extension PRD

Status: draft

## Problem Statement

The Template foundation is intentionally non-interactive, and the extension registry should not absorb blocking terminal behavior. Users will eventually need Templates that request text input or selections during rendering, but prompt behavior introduces UI dependencies, blocking execution, cancellation paths, and render-mode constraints.

Lithos needs a dedicated interactive prompt extension phase so prompt behavior can be designed around explicit interaction ports rather than leaking terminal UI dependencies into Template domain logic.

## Solution

Introduce a prompt extension module that provides controlled `prompt.*` functions for text input, single selection, and multi-selection. The extension uses interaction ports owned outside the Template repository traits, with CLI adapters implementing those ports through terminal UI dependencies.

Prompt extensions are only available in interactive render modes. Non-interactive foundation rendering remains available and must fail clearly if a Template requires prompts without an interactive provider.

## User Stories

1. As a Lithos user, I want a Template to ask for text input, so that generated notes can include one-off user-provided values.
2. As a Lithos user, I want a Template to offer a single-choice selection, so that generated notes can use constrained options.
3. As a Lithos user, I want a Template to offer multi-select choices, so that generated notes can include multiple selected values.
4. As a Lithos user, I want prompt defaults where supported, so that repeated generation is faster.
5. As a Lithos user, I want prompt cancellation to stop rendering safely, so that partial or unwanted output is not committed.
6. As a Lithos user, I want clear errors when prompts are used in non-interactive mode, so that automation does not hang unexpectedly.
7. As a Lithos user, I want prompts grouped under `prompt.*`, so that interactive behavior is obvious in Template source.
8. As a Lithos user, I want prompt behavior to happen before commit, so that cancelled renders do not write files.
9. As a Lithos user, I want prompt choices to be deterministic within a render, so that Template output matches selected values.
10. As a developer, I want interaction ports outside the Template repository traits, so that persistence and UI concerns stay separate.
11. As a developer, I want `InputProvider` for text input, so that prompt text can be tested without terminal UI.
12. As a developer, I want `SelectionProvider` for single and multi-select input, so that selection behavior can be tested independently.
13. As a developer, I want `lithos-cli` to own terminal UI dependencies, so that `lithos-core` remains free of terminal-specific dependencies.
14. As a developer, I want prompt functions registered through the extension registry, so that prompt availability is a render-mode capability.
15. As a developer, I want prompt failures to map to Template errors, so that service callers can handle cancellation and unavailable providers.
16. As a developer, I want prompt extensions tested with fake providers, so that automated tests do not require terminal interaction.
17. As a maintainer, I want prompt functions to be explicitly effectful, so that they cannot be mistaken for pure transformations.
18. As a maintainer, I want prompt behavior deferred from foundation, so that non-interactive rendering remains simple and automatable.
19. As a maintainer, I want the interaction module name settled, so that future features share consistent vocabulary.
20. As a maintainer, I want prompt extension docs to warn about automation behavior, so that CI and scripts use non-interactive render modes safely.

## Implementation Decisions

- Build this phase after Template foundation and after or alongside the extension registry.
- Keep prompt behavior out of Template repository traits.
- Place interaction ports in a top-level interaction context, with exact module name still to be decided.
- Define an `InputProvider` port for text input.
- Define a `SelectionProvider` port for single and multi-select choices.
- Implement terminal adapters in the CLI layer using terminal UI dependencies.
- Register prompt functions through the Template Extension Registry.
- Treat prompt functions as effectful and render-mode gated.
- Ensure prompt functions cannot commit output directly; commit remains Template Artifact/Template Service behavior.
- Support at least `prompt.text`, `prompt.select`, and `prompt.multi_select` as planned functions.
- Do not introduce prompt filters; prompts generate data through direct function calls.

## Testing Decisions

- Interaction port tests should use fake providers.
- Prompt extension tests should verify text input, single selection, multi-selection, defaults, cancellation, and unavailable provider behavior.
- Non-interactive render tests should verify prompt use fails clearly rather than blocking.
- Template Service tests should verify cancelled prompt rendering does not produce a committed artifact.
- CLI tests should verify prompt adapter wiring without relying on brittle terminal rendering details.
- Error tests should verify cancellation and provider errors map to Template errors.

## Out of Scope

- Foundation Template domain, repositories, processor, and single-output commit pipeline.
- Non-prompt extension modules such as date, string, path, file, numeric, query, and frontmatter.
- Query-powered option generation unless provided by a later query extension.
- Long-running async UI flows, background prompts, or prompt persistence.
- Arbitrary scripts or hooks.
- Multi-file template packs.

## Further Notes

- This PRD is a draft and should be revisited after the extension registry defines render-mode capabilities.
- The planned location name remains open: `interact` versus `prompt`.
- Prompt functions must be designed so automation can opt out cleanly.
