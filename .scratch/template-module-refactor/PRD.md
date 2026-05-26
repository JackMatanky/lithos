## Problem Statement

The current template module in `@lithos-core` is weighed down by legacy CQRS/event-driven abstractions that no longer fit the project's file-centric, deterministic asset model. It unnecessarily duplicates functionality natively provided by `minijinja` (like template inheritance and blocks) while simultaneously fighting the engine by intermingling domain abstractions with rendering logic.

Crucially, the current template system lacks support for structured interactions (such as prompts and suggesters) and database queries, which are core requirements for interactive, Obsidian-Templater style workflows. Furthermore, there is no strict validation pipeline ensuring that templates are properly cached, validated, and safely parsed before hitting the execution engine.

## Solution

The Template context will be completely redesigned as a file-centric, query-aware system driven by a rigid typestate pipeline.

The redesign embraces a "Limited Hybrid" execution model: static dependencies and options are declared in YAML frontmatter, while the template body drives dynamic interaction mid-render through a dedicated `TemplateRuntime` object (the `li` variable). The rendering engine (`minijinja`) is decoupled and isolated behind a thin adapter boundary, acting strictly as a Markdown string generator with auto-escaping disabled.

To avoid blocking the project on a full parser generalization, the template module will use its own lightweight `pulldown-cmark` extractor to cleanly split frontmatter from the Markdown body without rebuilding full ASTs.

## User Stories

1. As a note-taker, I want to define templates using standard Markdown files with YAML frontmatter, so that I can easily edit them in any text editor.
2. As a template author, I want to declare my template's schema dependencies in the frontmatter, so that the system can validate them before rendering.
3. As a template author, I want to declare static input variables in my frontmatter, so that the template prompts the user for required fields before rendering.
4. As a template author, I want to use standard `minijinja` syntax for logic and variables, so that I don't have to learn a proprietary templating language.
5. As a template author, I want to issue database queries directly from the template body using a structured builder API (e.g., `li.query("project").where(...)`), so that I can dynamically fetch related notes.
6. As a template author, I want to pause rendering to prompt the user to select an option from a list (e.g., `li.suggester(...)`), so that I can build interactive generation workflows.
7. As a system architect, I want the rendering engine to be completely decoupled from the domain models, so that the domain logic remains pure and engine-agnostic.
8. As a system architect, I want templates to be cached effectively in `redb` using `rkyv`, so that we avoid re-parsing unchanged templates.
9. As a system architect, I want a strict typestate pipeline (Discovery -> Comparison -> Parsed -> Construction) for template loading, so that I can guarantee at compile-time that un-validated or stale templates are never rendered.
10. As a performance engineer, I want the system to check file timestamps and content hashes before parsing, so that parsing is only performed when a template file is actually modified.
11. As a template user, I want the output to be perfectly formatted Markdown without injected HTML escaping, so that my generated notes are clean and readable.

## Implementation Decisions

- **Typestate Pipeline:** Modeled after the `schema` context, `TemplateProcessor` will use `Stage` and `Status` generics to enforce transitions. A template goes from `Discovery` to `Comparison` (mtime/hash checks) to `Parsed` to `Construction`.
- **Database Caching:** We will use `redb` tables `RAW_TEMPLATE_VIEWS` (for fast mtime/hash comparison), `TEMPLATE_ID_BY_PATH` (identity mapping), and `TEMPLATES` (the parsed domain aggregate).
- **Template Identity:** Templates will use `UuidV7` via `TemplateId`.
- **Data Model:** The core aggregate is `Template`, which contains `schemas`, `inputs`, and the raw `body` string. Frontmatter is parsed into `TemplateFrontmatter` and `InputSpec` objects.
- **Parsing Strategy:** We will defer a full parser refactoring. Instead, `template::parser::extract_template_asset` will use `pulldown-cmark` locally to consume only the YAML block and yield the raw Markdown body string.
- **Engine Adapter:** `minijinja` will be isolated to `template/engine/minijinja.rs`. It will be explicitly configured with `AutoEscape::None` and `UndefinedBehavior::Strict`. We will not cache the compiled AST on disk.
- **Runtime Environment:** The `TemplateRuntime` struct will implement `minijinja::Object` and be exposed as the `li` variable in templates. It will provide methods for `li.query` and `li.suggester`.
- **Query Representation:** Queries will use a structured builder pattern (`QueryBuilder`) mimicking SQL/Dataview rather than raw strings.
- **Abstractions Removed:** All files related to CQRS commands, queries, events, and native block inheritance strategies will be deleted.

## Testing Decisions

Tests must verify external behavior and state transitions, not implementation details.

- **`TemplateProcessor` (Typestate):** Comprehensive tests validating that the pipeline branches correctly on missing files, timestamp matches, timestamp mismatches, and content hash mismatches.
- **`Template Parser`:** Tests ensuring that frontmatter is cleanly split from the body without consuming or altering the body's Markdown content.
- **`Template Engine Adapter`:** Tests verifying that `minijinja` compiles successfully and does *not* auto-escape Markdown characters (e.g., `<` or `>`).
- **`Query API`:** Tests validating that the `QueryBuilder` correctly tracks schemas and filters.
- **`TemplateRuntime (li)`:** Tests utilizing a mock `InteractiveHost` to verify that `li.suggester` successfully pauses, prompts, and returns the selected option to the template engine.

## Out of Scope

- A full refactoring of `@lithos-core/src/note/parser/` to extract a generic infrastructure toolkit. (Deferred to maintain focus on the template system).
- Implementation of the actual CLI/TUI interactive prompt UI. (The template module will define the `InteractiveHost` trait, but the implementation belongs in the CLI crate).
- Advanced runtime schema discovery beyond children of frontmatter-declared schemas.
- Dynamic cross-template cache invalidation via a dependency graph table.

## Further Notes

The decision to use a "Limited Hybrid" schema resolution model ensures that we can validate the root dependencies of a template at load time, while still allowing the template author flexibility to drill down into child schemas interactively at runtime.
