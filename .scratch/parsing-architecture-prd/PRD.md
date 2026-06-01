---
labels: ["ready-for-agent"]
---

## Problem Statement

Lithos parsing responsibilities are split across seams that currently blur context ownership: Vault Root file access, structured decoding, and markdown ingestion primitives are coupled in ways that reduce locality and make change riskier than necessary. In practice, parsing behavior is spread between FS and context-specific modules, with structured parsing entry points tied to `FileReader`, and markdown ingestion behavior embedded in Note parser internals. This makes it harder to evolve parsing policy, improve diagnostics, and reuse frontmatter/offset logic for Template Asset ingestion without duplicating logic.

From the user perspective, this increases implementation friction and confidence cost when adding parsing features, evolving error messages, or introducing shared markdown behavior for Note and Template contexts.

## Solution

Introduce a parser-owned architecture that deepens parsing modules while preserving context boundaries:

- Keep FS focused on Vault Root-safe file operations (Path Validation + reads).
- Move parse policy into a dedicated parser module with clear submodules for structured and markdown parsing.
- Define parser-owned parseable format contracts and parser-owned errors with precise suberrors.
- Share only markdown ingestion primitives (frontmatter/body split, offset-aware event adaptation) between contexts.
- Keep Note and Template semantics context-local (no semantic unification).

This yields a clean `File Source -> Parser -> Domain Construction -> Projection` flow aligned with current type-driven design goals ("parse, don't validate") and with pulldown-cmark event-stream design in ADR 008.

## User Stories

1. As a developer, I want File Source reads to remain isolated from parse policy, so that filesystem safety and parsing evolution can change independently.
2. As a developer, I want structured parsing to have a parser-owned interface, so that Schema and Config flows do not depend on `FileReader` parsing helpers.
3. As a developer, I want parseable format classification to be explicit, so that unsupported-format behavior is deterministic and easy to test.
4. As a developer, I want Markdown included as a parseable format family, so that markdown ingestion can be represented consistently alongside JSON/TOML/YAML.
5. As a developer, I want parser errors split into structured and markdown suberrors, so that diagnostics are precise and operationally actionable.
6. As a developer, I want crate-native parser errors preserved as sources, so that upstream parse details are available without duplicating parser internals.
7. As a developer, I want parser error variants to carry path/format/range context, so that troubleshooting does not require reproducing failures with instrumentation.
8. As a Note maintainer, I want Note semantics (tags/tasks/link extraction) to remain in Note context, so that shared parser modules do not leak domain behavior.
9. As a Template maintainer, I want to reuse markdown frontmatter/body extraction, so that Template Asset ingestion does not reimplement pulldown behavior.
10. As a performance-conscious developer, I want pulldown-based markdown ingestion to remain offset-aware and linear, so that large vault ingestion stays efficient.
11. As a developer, I want byte offset mapping behavior to remain stable through refactor, so that position-derived features and diagnostics remain correct.
12. As an architect, I want `StructuredFileFormat` ownership to align with parse policy rather than file IO, so that discovery precedence logic lives at the right seam.
13. As a developer, I want migration to be staged with compatibility delegates, so that high-blast-radius schema flows are not broken in one cutover.
14. As a developer, I want old parsing entry points deprecated only after caller migration, so that refactor risk remains controlled.
15. As a reviewer, I want seam-level tests to pin behavior before and after migration, so that architectural change is measurable and safe.
16. As a user of Lithos, I want parsing failures to explain what failed and where, so that invalid files can be corrected quickly.
17. As a future contributor, I want parser modules to be deep and discoverable, so that implementation and AI navigation are easier.
18. As a developer, I want context boundaries preserved while sharing primitives, so that code reuse does not collapse domain separation.
19. As a maintainer, I want minimal duplicate frontmatter logic across Note and Template, so that fixes land once.
20. As a maintainer, I want migration docs and deprecation path clarity, so that follow-up work does not stall on architectural ambiguity.

## Implementation Decisions

- **Parser module introduction**: Introduce a parser-owned module with separate submodules for format contracts, structured parsing, markdown parsing, and parser errors.
- **FS context narrowing**: FS remains owner of Vault Root-safe reads and path safety. FS no longer owns parse policy after migration.
- **ParseableFormat contract**: Add `ParseableFormat` with `Json`, `Toml`, `Yaml`, and `Markdown` to represent parser-level format families explicitly.
- **Structured parsing seam**: Structured parsing APIs handle only structured decode behavior (JSON/TOML/YAML), and reject non-structured parse attempts through precise parser errors.
- **Markdown parsing seam**: Shared markdown module exposes frontmatter/body extraction and offset-aware event adaptation primitives only; no Note/Template semantics included.
- **Error hierarchy**: Parser errors are split into suberrors:
  - `StructuredParserError` for structured decode and format-policy failures.
  - `MarkdownParserError` for frontmatter/offset/event adaptation failures.
  - Optional small wrapper (`ParserError`) to normalize top-level propagation.
- **Crate-native error source preservation**: JSON/TOML/YAML parse errors are preserved as sources and wrapped with Lithos context (path/format/range) when needed.
- **Structured format ownership migration**: `StructuredFileFormat` migrates toward parser ownership with compatibility re-exports during transition to avoid wide breakage in discovery flows.
- **No speculative trait now**: Do not add a `StructuredDecoder` trait yet. Use concrete APIs first; introduce a trait only if a second real adapter emerges.
- **Position ownership decision**: Keep Note position types in Note context for now because they carry Note-domain invariants and broad downstream usage.
- **Offset seam strategy**: Shared markdown module may use parser-neutral ranges internally and convert to Note position types at context seam boundaries.
- **Compatibility stage**: Existing `FileReader` structured parsing helpers become transitional delegates to parser module before deprecation/removal.
- **Blast-radius-aware rollout**: Migrate high-dependency structured parsing callers incrementally (especially schema/property bank flows), then retire legacy entry points.
- **Architectural alignment**:
  - Align with current type-driven design goals (`File Source -> Parse -> Construct Domain -> Project`) without relying on ADR 010 as authoritative.
  - Align with ADR 008 (pulldown event-stream + offset-based processing).
- **Rust best-practices constraints**:
  - Prefer concrete static dispatch initially.
  - Use typed errors (`thiserror`) and avoid panic-driven control flow.
  - Keep borrowing/ownership efficient in markdown event paths and avoid unnecessary cloning in hot parsing loops.

## Testing Decisions

- **Test quality definition**: Tests validate externally observable parsing behavior and diagnostics contracts, not internal module topology.
- **Behavior pinning before migration**:
  - Pin structured parse outcomes and error reporting for JSON/TOML/YAML malformed inputs.
  - Pin extension/content classification precedence behavior.
  - Pin markdown frontmatter extraction behavior and exact body preservation.
  - Pin offset-to-range validity expectations for event adaptation.
- **Seam test suites**:
  - FS seam tests verify Vault Root and read behavior remain unchanged.
  - Structured parser seam tests verify format handling and precise suberror behavior.
  - Markdown parser seam tests verify frontmatter and offset adaptation behavior.
  - Note integration tests verify no semantic drift in extracted note artifacts.
  - Template integration tests (new parser use) verify reusable frontmatter/body behavior without Note semantic leakage.
- **Error tests**:
  - Verify source-chain preservation for crate-native errors.
  - Verify added context fields (path/format/range) are present and accurate.
- **Performance safety checks**:
  - Re-run existing parse benchmarks and ensure no regression in hot parsing paths.
- **Prior art in codebase**:
  - Existing FS format and reader tests for structured parsing behavior.
  - Existing Note parser tests around frontmatter and reference handling.
  - Existing note parsing benchmark suites for ingest/parse performance baselines.

## Out of Scope

- Full semantic unification of Note and Template parsing logic.
- Moving all Note position primitives into a shared utility context immediately.
- Redesigning downstream domain validation semantics for Note/Schema/Template.
- Reworking repository traits, persistence model, or database projection contracts.
- Implementing unrelated architectural candidates previously marked irrelevant.

## Further Notes

- Migration should proceed in narrow, reviewable slices to reduce risk on schema/property-bank ingestion flows.
- Deprecation should be explicit and time-boxed after caller migration is complete.
- If future adapters create real seam multiplicity, revisit trait extraction with evidence.
- If parser-neutral position invariants stabilize across contexts, a later proposal can evaluate promoting shared position primitives.
- ADR 010 is treated as historical context only and is not normative for this effort.
