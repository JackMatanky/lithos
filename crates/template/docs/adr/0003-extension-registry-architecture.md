---
name: extension-registry-and-io-capability-architecture
status: accepted
supersedes: []
date_proposed: 2026-07-05
date_decided: 2026-07-05
date_implemented: TBD
stakeholders: [Jack (Developer), Architecture Team]
---

# ADR 0003: Extension Registry and I/O Capability Architecture

## Context

The Traces `template` crate uses MiniJinja as its rendering engine. While the foundation deliberately restricted functionality to standard MiniJinja built-ins, template authors require powerful domain-specific extensions (dates, paths, string manipulations, and file operations).

To support Obsidian Templater-like capabilities, templates must be able to perform side effects mid-render (e.g., dynamically dictating the output path using `file.write_to("custom/path.md")`).

Integrating these extensions presents several architectural challenges:
1. **Hexagonal Integrity**: The `MiniJinjaEngine` is an infrastructure port. Hardcoding domain extensions or I/O capabilities directly into it violates the architecture.
2. **State Management**: Effectful functions must log side-effects (like target paths) dynamically during rendering so the host application can act on them afterwards.
3. **Capability Segregation**: We need to provide I/O capabilities (file reads, future prompts) without creating an unwieldy "God object" (`TemplateHost`) that makes testing tedious.

## Decision

We will implement an **Extension Registry** using MiniJinja's native state-capture features and granular capability traits.

1. **ExtensionRegistry Component**: The Application layer (`TemplateService`) will assemble an `ExtensionRegistry` containing all pure and effectful extensions, injecting it into `MiniJinjaEngine::new`.
2. **Granular Capability Traits**: Effectful extensions will rely on module-specific traits (e.g., `FileProvider`) provided by the Application layer, rather than a monolithic host object.
3. **State Injection via Temps**: Capabilities will be injected into MiniJinja's lock-free temporary object cache (`State::get_or_set_temp_object`). This scopes effectful state exactly to the current render pass, ensuring thread safety for concurrent renders.
4. **Post-Render Side Channels**: Template functions that trigger post-render effects (like `file.write_to()`) will record their commands into the `State` temps. The `TemplateService` will use `render_captured()` to safely extract these commands after the template finishes rendering.
5. **Jinja Semantics**: We will not force a single extension format. We will use Jinja Functions for data generation and side effects, Filters for transformations, and Tests for conditional evaluation.

## Alternatives Considered

### Alternative 1: Monolithic TemplateHost Trait
Provide a single `TemplateHost` trait containing methods for all possible effects (`read_file`, `write_to`, `prompt_user`, `query_metadata`).
* **Why Rejected:** Violates the Interface Segregation Principle. Any test that exercises the template engine would need to mock the entire host, even if it only tests a simple `file.write_to` effect. Granular traits (`FileProvider`, `PromptProvider`) are vastly superior for isolated testing.

### Alternative 2: "Push" Data Model (No Effectful Extensions)
Force the Application layer to pre-load all possible file paths, metadata, and user prompts into the `variables` context before calling `render()`.
* **Why Rejected:** Fundamentally incompatible with dynamic templates. Templates often conditionally determine *which* file to read or *what* to prompt the user for based on previous logic (e.g., `if is_personal { prompt("Task?") }`).

### Alternative 3: Arc<Mutex<Tracker>> Injected as Global Variable
Inject a shared tracking object into the MiniJinja `Environment` globally.
* **Why Rejected:** Clumsy and forces the engine to handle cross-render synchronization. MiniJinja's `State` temps natively provide thread-safe, per-render isolation, automatically tearing down the state when the render finishes.

## Technical Validation

MiniJinja supports the `render_captured()` API, which returns a `Captured` struct containing both the rendered output string and the retained `State`.
By using `state.get_or_set_temp_object("file_commands", || FileTracker::default())`, an extension can seamlessly log a `write_to` command. The host application can then extract it via `captured.state().get_temp("file_commands")` with zero risk of concurrency collisions.

## Consequences

### Positive
- **Hexagonal Purity**: `MiniJinjaEngine` remains a dumb infrastructure port. It simply registers the provided `ExtensionRegistry` without knowing what the extensions do.
- **Thread Safety**: Relying on `State` temps ensures that thousands of templates can be rendered concurrently without locking contention on a global registry.
- **Simplicity for Authors**: Template authors can write `{{ file.write_to("path.md") }}` effortlessly. The extension returns an empty string, meaning they don't have to use verbose `{% do %}` syntax.
- **Testability**: Pure extensions can be unit-tested trivially. Effectful extensions only require mocking granular traits (e.g., `FileProvider`).

### Negative
- **Opaque Errors**: If an effectful capability is missing from the state, the template fails at render-time.
- **Deferred Complexity**: Relying on the "Pull" model paves the way for complex, heavy queries inside templates (which will be tackled in the future `query.*` module PRD).
