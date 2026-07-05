# Template Extension Registry PRD

Status: ready

## Problem Statement

Traces needs a robust templating system to generate notes based on Markdown files. While the Template foundation deliberately uses MiniJinja built-ins only, users require an ecosystem of domain-specific helpers (dates, paths, files, strings, numbers) to automate note generation. Because some operations are pure data transformations and others perform side effects (like defining where a file is saved), injecting these directly into the engine without a formal architecture risks coupling the infrastructure layer to application logic, violating Hexagonal Architecture.

## Solution

Design and implement a Template Extension Registry that registers Traces-owned functions, filters, and tests into the Template Engine. The registry cleanly separates "pure" extensions (which require no I/O) from "effectful" extensions (which require host capabilities like file access).

To preserve the Template Engine as an isolated infrastructure port, the Application layer (`TemplateService`) will assemble the `ExtensionRegistry` and its capability traits (e.g., `FileProvider`), and inject them into the `MiniJinjaEngine`. Effectful commands will be handled natively via MiniJinja's lock-free `State` temps and extracted post-render using `render_captured()`, allowing complex templating (such as `file.write_to`) without exposing internal state complexity to the template author.

## User Stories

1. As a Traces user, I want a `date.*` module, so that I can insert and manipulate current and future dates into my templates.
2. As a Traces user, I want a `str.*` module, so that I can format, split, replace, and normalize text pipelines seamlessly.
3. As a Traces user, I want a `path.*` module, so that I can manipulate file paths (getting basenames, extensions, joining paths) predictably.
4. As a Traces user, I want a `num.*` module, so that I can perform basic math and numeric coercion inside my templates.
5. As a Traces user, I want to use `{{ file.write_to("custom.md") }}`, so that my template can dynamically define where its output should be saved.
6. As a Traces user, I want functions like `date.now()` to be callable without `do` syntax silently, so that my templates remain clean and readable.
7. As a Traces user, I want to use standard `{% set %}` to capture the results of effectful functions, so that I can reuse them later in my template.
8. As a Traces user, I want `if` blocks to naturally allow variable mutation without restrictive scoping tricks, so that I can easily build conditionally generated metadata.
9. As a Traces user, I want clear and distinct syntax for functions (`date.now()`), filters (`| str.slugify`), and tests (`is path.is_file`), so that template operations feel idiomatic to the Jinja ecosystem.
10. As a Traces user, I want templates that error out when an effectful operation fails, so that partial renders don't corrupt my vault.
11. As a developer, I want an `ExtensionRegistry`, so that the `MiniJinjaEngine` infrastructure port remains generic and agnostic of domain operations.
12. As a developer, I want granular capability traits (e.g., `FileProvider`), so that I don't have to mock a monolithic `TemplateHost` when writing tests.
13. As a developer, I want effectful state tracking handled via MiniJinja's `State` temps and `render_captured()`, so that the template rendering remains strictly thread-safe and lock-free.
14. As a developer, I want pure extensions implemented separately from effectful ones, so that future render modes can easily restrict or mock side-effects.
15. As a developer, I want template evaluation errors to fail fast and abort rendering, so that I can reliably catch template logic bugs during test validation.

## Implementation Decisions

- **Extension Categories**: Operations are strictly defined as either Functions (generate data/effects), Filters (transform data), or Tests (evaluate conditions), aligning with native Jinja semantics.
- **Namespacing**: Extensions belong to core modules (`date`, `str`, `path`, `num`, `file`). MiniJinja natively supports dot-syntax, so functions will use `module.function()` and filters will use `| module.function` syntax.
- **Classification**:
  - *Pure Extensions* perform no I/O and rely purely on input data.
  - *Effectful Extensions* require I/O capabilities and rely on "State Injection".
- **Interface Segregation**: Effectful capabilities are delivered through granular Application layer traits (e.g., `FileProvider`) rather than a single massive God-object.
- **Side-Channel Tracking**: Effectful commands (like `file.write_to`) log their instructions directly into MiniJinja's `State` temporary object cache. The `TemplateService` will execute `render_captured()` to safely extract these commands after the template renders.
- **Variable State**: We will rely entirely on native MiniJinja syntax (`{% set %}`) for caching data across the template. No custom global `var.*` module will be built.
- **Output of side effects**: Void functions like `file.write_to()` will return an empty string rather than requiring `{% do %}` syntax, lowering the friction for template authors.

## Testing Decisions

- **Pure Extension Tests**: Standard unit tests against an isolated `MiniJinjaEngine` containing the pure modules, passing deterministic inputs and asserting deterministic outputs.
- **Effectful Extension Tests**: Integration tests against `TemplateService` using in-memory implementations of `FileProvider` or similar traits to verify the side-channel queue correctly logs `file.write_to` operations.
- **State Seams**: Because we use `State` temps, we only need a single seam (the injected `Provider` traits) to test effectful extensions. We will not mock MiniJinja internals.

## Out of Scope

- **Prompt Extensions (`prompt.*`)**: Blocking user interaction is deferred to a future PRD, as it requires a UI orchestrator.
- **Query Extensions (`query.*`)**: Vault metadata querying (replacing Obsidian's `metadataCache`) is deferred to a future PRD.
- **User Plugins**: This slice covers first-party Traces extensions only. Dynamic user scripts (WASM/Lua/JS) are strictly out of scope.
- **Multi-file Template Packs**: Advanced template generation orchestrations spanning multiple output files.

## Further Notes

- The architecture avoids a "Push" model (pre-loading everything before rendering) in favor of a "Pull" model where templates request I/O capabilities dynamically.
- The use of `State::get_or_set_temp_object` means the engine can run fully concurrent renders across multiple threads with absolute state isolation.
