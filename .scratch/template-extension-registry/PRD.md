# Template Extension Registry PRD

Status: draft

## Problem Statement

The Template foundation intentionally supports MiniJinja built-ins only. Users will need Lithos-owned functions and filters for dates, strings, paths, files, numbers, and future prompt/query modules, but adding those directly to foundation would couple extension policy to core rendering before extension boundaries are understood.

Lithos needs a dedicated extension registry design that can classify pure and effectful extensions, define module namespaces, preserve Template Engine isolation, and prepare first-party extension modules without turning Templates into arbitrary scripts.

## Solution

Design and implement a Template Extension Registry that registers Lithos-owned functions and filters with the configured Template Engine through an adapter-local boundary. The registry defines first-party namespaces, extension capabilities, pure versus effectful behavior, and rules for which render modes can use which extensions.

The first implementation should focus on safe, deterministic, non-interactive extension modules that build on foundation rendering. Prompt extensions and query/frontmatter extensions remain separate follow-up PRDs.

## User Stories

1. As a Lithos user, I want date helpers in Templates, so that generated notes can include current and derived dates.
2. As a Lithos user, I want string filters in Templates, so that generated note text can be normalized without external scripts.
3. As a Lithos user, I want path helpers in Templates, so that paths can be inspected and formatted safely.
4. As a Lithos user, I want numeric filters in Templates, so that simple calculations and formatting can happen during rendering.
5. As a Lithos user, I want file helpers only when explicitly allowed, so that Templates do not gain surprising side effects.
6. As a Lithos user, I want extension names grouped by module namespace, so that functions and filters are discoverable.
7. As a Lithos user, I want consistent function call syntax, so that extension behavior is predictable.
8. As a Lithos user, I want consistent filter syntax, so that transformations compose naturally in MiniJinja pipelines.
9. As a Lithos user, I want unsupported extensions to fail clearly, so that broken Templates are easy to fix.
10. As a Lithos user, I want extension behavior documented, so that Templates can be written without reading implementation code.
11. As a developer, I want extension registration separate from Template foundation, so that foundation remains a minimal rendering slice.
12. As a developer, I want `TemplateExtension` or equivalent abstractions designed around Lithos use cases, so that MiniJinja registration APIs do not leak into service code.
13. As a developer, I want pure extensions separated from effectful extensions, so that render modes can enforce safety boundaries.
14. As a developer, I want module namespaces such as `date`, `str`, `path`, `file`, and `num`, so that first-party extensions stay organized.
15. As a developer, I want extension signatures specified, so that validation can catch incorrect usage where possible.
16. As a developer, I want extension registration to be adapter-local, so that Template domain models do not depend on MiniJinja extension types.
17. As a developer, I want extension errors to map into Template error types, so that users receive Rust-native error chains.
18. As a developer, I want extension tests independent from Template Service tests, so that extension behavior can be validated in isolation.
19. As a maintainer, I want first-party modules to avoid arbitrary hooks and script execution, so that Templates stay safe.
20. As a maintainer, I want effectful extensions to be explicit, so that future render modes can restrict them.
21. As a maintainer, I want prompt and query modules deferred, so that this phase does not absorb interactive or data-query concerns.
22. As a maintainer, I want compatibility with MiniJinja built-ins, so that custom extensions do not duplicate behavior unnecessarily.
23. As a maintainer, I want stable names for first-party modules, so that Templates remain portable across Lithos versions.
24. As a maintainer, I want extension registry decisions documented, so that future modules follow the same rules.

## Implementation Decisions

- Build this phase after Template foundation.
- Introduce an extension registry boundary dedicated to Template Engine configuration.
- Keep extension registration out of Template domain models, repository traits, service requests, and service responses.
- Preserve MiniJinja as an adapter detail where registration happens.
- Define functions as direct calls that generate data or perform controlled side effects.
- Define filters as transformations that take the piped value as their first input.
- Group operations into modules with explicit namespaces.
- Start with deterministic pure modules where possible: date, string, path, and numeric helpers.
- Treat file helpers as effectful or capability-gated unless a specific helper is pure metadata inspection.
- Defer prompt helpers to the interactive prompt extension PRD.
- Defer query and frontmatter helpers to the query extension PRD.
- Prefer wrappers around MiniJinja built-ins only when Lithos needs stable naming, constraints, or future documentation.
- Do not add arbitrary script execution or hooks.

## Testing Decisions

- Tests should assert extension behavior through Template Engine rendering, not MiniJinja internals.
- Registry tests should verify enabled modules register the expected function/filter names.
- Pure function/filter tests should use deterministic inputs and outputs.
- Date tests should isolate current-time behavior behind an injectable clock or deterministic test seam where needed.
- Path tests should verify safe normalization and platform expectations.
- File tests should verify capability and FS context boundaries before any effectful behavior is allowed.
- Error tests should verify unsupported names, invalid argument shapes, and extension failures map to Template errors.
- Compatibility tests should ensure MiniJinja built-ins still work after registry configuration.

## Out of Scope

- Template foundation ingestion, repositories, artifact commit pipeline, and CLI vertical slice.
- Prompt interaction and blocking UI behavior.
- Query runtime objects, Dataview-style queries, and frontmatter handlers.
- Multi-file template packs.
- Arbitrary scripts, hooks, shell execution, or plugin loading from user code.
- Rich diagnostics beyond Rust error types and logging.

## Further Notes

- This PRD is a draft and should be grilled after foundation implementation pressure is known.
- The planned module list comes from the existing function registry planning surface and may be trimmed before implementation.
- The registry should make later prompt/query modules possible without deciding their behavior here.
