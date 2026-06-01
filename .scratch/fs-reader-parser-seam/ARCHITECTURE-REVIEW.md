# Lithos Parsing Architecture Review

## 1. Executive Summary

Lithos already has two distinct parsing concerns that should not share ownership in `fs`: (1) structured document decoding (JSON/TOML/YAML) and (2) markdown semantic extraction for note and template workflows. The current implementation places structured parsing entry points on `FileReader`, while markdown parsing is a separate parser pipeline in `note::parser`.

Recommendation: create a dedicated `parser` context that owns all content parsing interfaces and implementations, while `fs` remains responsible only for file access, path validation, and format hints. Within `parser`, keep two deep modules with a common seam:

- `parser::structured` for JSON/TOML/YAML classification + decoding
- `parser::markdown` for pulldown-cmark event processing + projections

For markdown, prefer:

`Markdown -> pulldown-cmark -> Lossless IR -> Reusable projections -> Domain models`

This gives the best long-term leverage for note extraction, template frontmatter/body extraction, lexical phases, diagnostics, caching, and future interactive runtime features.

## 2. Current-State Architecture Review

### Ownership today

- `fs::reader::FileReader` owns I/O (`read_to_string`, `exists`) and also structured parsing entry points (`parse_structured`, `parse_structured_from_str`, `classify_path`): `lithos-core/src/fs/reader.rs`.
- `fs::format` owns `FileFormat`, extension classification, structured sniffing, and parser implementations for JSON/TOML/YAML (`JsonParser`, `TomlParser`, `YamlParser`): `lithos-core/src/fs/format.rs`.
- `note::parser` owns a full markdown pipeline (event stream adapter, parser IR, structure tree, lexical scan, assembly to `RawNote`): `lithos-core/src/note/parser/`.

### Current parsing responsibilities

- Structured file parsing (JSON/TOML/YAML): in `fs`.
- Markdown parsing for note ingestion: in `note`.
- Frontmatter extraction in note flow: represented as `Frontmatter` block in parser IR (`BlockStart::Frontmatter`), then assembled into `RawFrontmatter`.

### Existing abstractions

- Structured classification abstractions:
  - `FileFormat`
  - `StructuredFileFormat`
  - `sniff_structured_format()`
- Markdown abstractions:
  - `MarkdownEventStream` adapter over `pulldown-cmark`
  - `RangedEvent` / `ParserEvent` IR
  - `DocTree` structural AST
  - `ArtifactLexer` lexical extraction
  - `RawAssembler` domain projection

### Coupling analysis

- I/O <-> parsing coupling exists in structured flows because parse APIs are methods/associated functions on `FileReader` despite pure parsing logic living in `fs::format`.
- Parsing <-> domain coupling is low-to-moderate in note parser:
  - low in stream/types/structure layers (mostly parser-owned IR)
  - higher in `assemble.rs`, which projects directly into `RawNote` domain types.
- `note::processor` invokes parser and then domain conversion (`Note::try_from`), which is good staging.

### Duplication and shallow spots

- Format classification exists near file concerns (`fs`) but is consumed by non-fs domains (schema/config/template direction), creating a shallow seam.
- Future template parsing draft proposes local extractor, risking duplication with note frontmatter handling.
- `FileReader::classify_path` is an orchestration helper that likely belongs to parser classification, not file access.

## 3. Pulldown-cmark Capability Analysis

Based on the internal guide and current API usage:

- Event stream is first-class (`Parser` iterator), enabling transform pipelines without building full AST.
- `OffsetIter` provides byte range tracking, already leveraged by Lithos (`RangedEvent` / `SourceByteRange`).
- Reference definitions are directly available via `parser.reference_definitions()`, already used in `MarkdownEventStream`.
- Extensions/options are controlled via `Options` flags; Lithos already centralizes defaults via `EventStreamConfig::default_options()`.
- Metadata/frontmatter support is available as parser events/tags (`Frontmatter` path in Lithos IR indicates this support is already integrated).
- `TextMergeWithOffset` supports adjacent text merge while preserving ranges.

Implications:

- You can avoid custom hand-rolled markdown tokenization for most block/inline grammar.
- Offset-aware event pipelines can support lexical pass handoff without re-parsing.
- Frontmatter extraction can be done via early projection over event stream, not bespoke line scanners.

## 4. External Project Survey

Note: sources were sampled from available upstream/docs pages and repository source files; confidence is high for rustdoc, Zola, and rumdl where implementation files were inspectable.

### mdBook

- Parsing boundary: markdown parser is infrastructure; book/domain orchestration stays outside.
- IR strategy: pulldown events are transformed before final render.
- Event-stream handling: stream-centric and renderer-oriented.
- Lesson: keep markdown mechanics independent from book/domain state.

### rustdoc

- Parsing boundary: dedicated markdown module (`html/markdown.rs`) isolates parser options + event transforms.
- IR strategy: event stream adapters (`CodeBlocks`, link replacers, heading processors), then HTML emission.
- Event-stream handling: strong iterator pipeline with specialized adapters.
- Lesson: compose many focused transforms on event streams; do not fuse with I/O.

### Zola

- Parsing boundary: markdown module handles parser configuration and event rewriting; templating/runtime outside.
- IR strategy: mutate pulldown events for links, anchors, footnotes, shortcodes, then render.
- Event-stream handling: parse once, transform events, project to output.
- Lesson: event-level reusable transforms scale across features.

### pulldown-cmark-to-cmark

- Parsing boundary: takes `Event` streams and serializes back to markdown.
- IR strategy: event stream as canonical interchange.
- Event-stream handling: stateful serializer with optional source-range preservation.
- Lesson: event streams are robust enough to support round-trip and alternate projections.

### cargo-about

- Not markdown-centric in sampled code; no major pulldown parser architecture signal for this decision.
- Lesson: low relevance.

### markedit

- Parsing boundary: parse markdown to events; rewriting engine is separate.
- IR strategy: matcher/rewriter abstractions over pulldown events.
- Event-stream handling: streaming rewrites with minimal buffering.
- Lesson: reusable event-level utilities are high leverage and easy to compose.

### pullup

- Parsing boundary: conversion framework across markup formats.
- IR strategy: generic `ParserEvent` abstraction between format-specific adapters.
- Event-stream handling: adapter/converter model.
- Lesson: a neutral parser IR unlocks multi-consumer projections.

### rumdl

- Parsing boundary: a dedicated parsing context (`lint_context`) owns markdown parsing and normalization, while rule modules consume parsed artifacts (`src/lint_context/mod.rs`).
- IR strategy: parser-owned line and token structures (`LineInfo`, `ParsedLink`, `ParsedImage`, footnotes, heading metadata, list metadata, skip ranges) plus workspace-level `FileIndex` / `WorkspaceIndex` for cross-file analysis (`src/workspace_index.rs`).
- Event-stream handling strategy:
  - Uses `pulldown-cmark` for links/images/reference/broken-link extraction and offset ranges in `parse_links_images_pulldown()` (`src/lint_context/link_parser.rs`).
  - Augments pulldown with targeted regex fallback passes and flavor-aware post-processing for cases where pulldown coverage is intentionally incomplete or flavor-specific.
  - Performs many precomputed skip-range and context passes (code blocks, HTML blocks, template ranges, flavor-specific blocks) before rule evaluation.
- Separation between parsing and domain logic:
  - Parsing and context derivation are centralized in `lint_context`.
  - Rule behavior (domain policy) lives in independent rule modules (example: `MD051` in `src/rules/md051_link_fragments.rs`) and consumes parser output.
  - Cross-file validation is decoupled into `cross_file_scope`, `contribute_to_index`, and `cross_file_check` with persistent workspace indexing.
- Relevant lessons for Lithos:
  - A dedicated parser module with reusable artifacts scales better than embedding parse logic into each consumer.
  - pulldown should be the core parser, but practical systems still need targeted projection/fallback layers.
  - Offset-aware parsing plus cached indices unlock robust cross-file and incremental workflows.
  - Flavor/feature variability is best handled in parser/context normalization layers, not scattered across downstream domain modules.

## 5. Architectural Patterns Identified

- **Deep module pattern**: parser module owns complexity of markdown grammar/event normalization; callers consume stable interface.
- **Two-stage pipeline**: syntax extraction first, domain projection second.
- **Event adapter seam**: pulldown specifics are isolated behind internal IR.
- **Offset-carrying IR**: enables diagnostics, lexical scanning, and exact slicing without reparsing.
- **Projection fan-out**: one parse result serving multiple consumers (notes, templates, diagnostics, indexing).

## 6. Parser Ownership Recommendation

### Recommendation

Own parsing in a dedicated `parser` context, not in `fs` and not inside each domain module.

### Rationale

- `fs` context language and invariants focus on path safety and vault-scoped access, not content semantics.
- Parsing is a cross-domain concern used by schema/config/note/template and should be isolated from storage seams.
- Current `note::parser` already proves value of a dedicated parser module; extend this model systematically.

### Ownership split

- `fs` owns: file discovery, path validation, byte/string reading, format hints by extension.
- `parser` owns: classification policies, content sniffing, structured decoding, markdown event/IR/projections.
- domain contexts own: validation and business semantics over parsed outputs.

## 7. Structured-File Parsing Recommendation

### For TOML/JSON/YAML

- Keep decoders specialized per format (serde_json, toml, serde_yaml).
- Move classification + parse orchestration from `FileReader` surface into `parser::structured`.
- Keep extension-hint path + optional content sniffing policy explicit in API.

### Proposed shared abstractions

- `StructuredKind` (`Json | Toml | Yaml`)
- `StructuredClassifier` (from path hint + optional content)
- `StructuredParser` (decode by explicit kind)

### Why

- Better locality: all structured parse rules in one module.
- Better leverage: fs/reader stops being a pass-through parser host.
- Better testability: pure parsing tested without filesystem adapter.

## 8. Markdown Parsing Recommendation

Evaluate options:

1. `Markdown -> pulldown-cmark -> Domain models`
   - Pros: shortest path
   - Cons: hard to reuse, hard to evolve lexical phases, domain coupling high

2. `Markdown -> pulldown-cmark -> Lossless IR -> Domain projections`
   - Pros: strong separation, supports multiple consumers
   - Cons: projection duplication risk if projections are ad hoc

3. `Markdown -> pulldown-cmark -> Lossless IR -> Reusable projections -> Domain models`
   - Pros: best extensibility/testability; lexical and future analyzers slot naturally
   - Cons: slightly higher upfront module design cost

### Recommendation

Choose option 3.

### Why for Lithos

- Note and template both need frontmatter/body handling but diverge in downstream semantics.
- Future lexical-analysis phases need neutral artifacts before domain coupling.
- Offset-rich IR supports persistence metadata, diagnostics, and incremental pipelines.

## 9. Intermediate Representation Recommendation

Adopt a layered IR model:

- **Layer A: Event IR (lossless-ish)**
  - current `ParserEvent` + `RangedEvent` baseline
  - preserve source ranges and core markdown semantics
- **Layer B: Structural IR**
  - current `DocTree` block hierarchy
- **Layer C: Projection utilities**
  - frontmatter/body projection
  - plain text projection
  - link/reference projection
  - lexical scan index projection

Guideline: domain models (`RawNote`, `Template` aggregate inputs) should be built from projections, not directly from pulldown events.

## 10. Note/Template Integration Strategy

### Shared

- Shared markdown parser core (`parser::markdown`) and shared event/structure IR.
- Shared projection for frontmatter extraction + body preservation.
- Shared lower-level lexical utility hooks where semantics overlap (range-aware scanning utilities).

### Separate

- Note-specific lexical rules and domain assembly remain in note context adapters/projections.
- Template-specific frontmatter schema, MiniJinja runtime bindings, and interaction semantics remain in template context.

### Net

- Share parser + reusable projections.
- Do not force a single unified domain projection type for note and template.

## 11. Proposed Module Boundaries

Suggested target shape:

- `lithos-core/src/parser/mod.rs`
  - `structured/`
    - `classify.rs`
    - `decode.rs`
    - `types.rs`
  - `markdown/`
    - `stream.rs` (pulldown adapter)
    - `types.rs` (neutral IR)
    - `structure.rs` (doc tree)
    - `projections/`
      - `frontmatter.rs`
      - `text.rs`
      - `references.rs`
      - `lexical_index.rs`

Context adapters:

- `note` consumes markdown projections + note-specific lexical/domain projection.
- `template` consumes frontmatter/body projection + template-specific domain mapping.
- `fs::reader` delegates parsing helpers to parser context (or stops exposing them).

## 12. Migration Strategy

1. Introduce parser context as additive module; keep existing call sites working.
2. Move structured classification/parse APIs from `FileReader` surface to parser APIs with compatibility shims.
3. Extract reusable markdown projections from current note parser internals.
4. Rewire note parser assembly to consume projection seams (no behavior change).
5. Implement template parser on shared markdown frontmatter/body projection.
6. Remove compatibility shims after downstream migration.

## 13. Risks and Tradeoffs

- **Risk: over-generalization early**
  - Mitigation: design projections around known consumers (note/template) first.
- **Risk: duplicated IR if note internals and new parser diverge**
  - Mitigation: migrate existing note parser IR into parser context rather than parallel rebuild.
- **Risk: perf regressions from extra projection passes**
  - Mitigation: keep iterator-based streaming + borrow-heavy data; benchmark parse+projection hot paths.
- **Risk: boundary churn across many modules**
  - Mitigation: compatibility shims and staged migration.

## 14. Final Recommendation

Adopt a dedicated parser context with two deep modules: `structured` and `markdown`.

For markdown, standardize on:

`Markdown -> pulldown-cmark -> Lossless IR -> Reusable projections -> Domain models`

Keep `fs` focused on filesystem concerns only. Use shared parser infrastructure for note and template ingestion, while preserving separate domain projection layers per context. This architecture maximizes locality, leverage, and future extensibility (especially lexical-analysis phases) without coupling parser evolution to `FileReader` or any single domain.
