# Architectural & Component Decisions

This file captures decisions made so far. It is not exhaustive; unresolved items are listed under **Open Decisions**.

## Hexagonal Architecture
- **Template ports**: Template-specific repository traits are defined in `lithos-core/src/template/` and must follow ADR 016's segregated repository pattern: `ReadRepository`, `WriteRepository`, and `Repository`.
- **Interaction ports**: `InputProvider` and `SelectionProvider` do not belong in `template`. They are planned for a new top-level `lithos-core/src/interact/` or `lithos-core/src/prompt/` context/module.
- **Adapters (Implementations)**: Defined at the edges. Example: `InquireAdapter` in `lithos-cli` using the `inquire` crate.
- **Core Purity**: `lithos-core` remains free of terminal-specific or UI-specific dependencies.

## Registry Pattern
- **TemplateExtension Trait**: Modules (File, Path, Date, etc.) implement a `register(&self, env: &mut minijinja::Environment)` method.
- **Modularity**: This allows for a pluggable standard library and easy extension for future interactive or query-based modules.

## Stateful Side-Effects
- **Execution Context**: The `TemplateEngine` maintains a buffered context during rendering.
- **Post-Render Execution**: Side-effects like `file.write(path)` are recorded during the render pass and executed only after a successful render to ensure determinism.

## Persistence & Caching
- **Database**: Using `redb` for identity and caching.
- **Zero-Copy**: Using `rkyv` for efficient serialization/deserialization.
- **Change Detection**: Using `Blake3HashIndex` and `Blake3Hash` from `lithos-core/src/support/` for `RawTemplateView`.
- **Design order**: Domain models and DTOs must be designed before the typestate processor, because processor stages should move typed domain data rather than ad-hoc strings/maps.

## Future-Proofing
- **Vault Move**: The initial template processor is designed with the intent that it will eventually migrate to the `vault` module as part of the filesystem indexer. This ensures file freshness logic is consistent across the project.
- **Filesystem freshness**: Template-specific freshness checks should be a small, replaceable seam. The future vault filesystem indexer is expected to own general file freshness.

## Project Phases
1. **template-foundation**: Registry pattern, Standard Library (File, Path, Date, String, Math), and Typestate Processor. (Current focus)
2. **template-user-interaction**: Parity with Templater system modules (prompt, suggester) using `inquire`.
3. **template-query**: Structured query builder (Dataview-style) and the `frontmatter` handler.

## Open Decisions
- Exact location name for interaction ports: `lithos-core/src/interact/` vs `lithos-core/src/prompt/`.
- Exact shape of `Template`, `RawTemplate`, `RawTemplateView`, and related DTOs.
- Exact repository trait methods for template reads/writes.
- Whether template file reads and writes call existing FS context ports directly or through template-specific ports that delegate to FS adapters.
- How `RawTemplateView` keys its `Blake3HashIndex` before the vault filesystem indexer exists.
