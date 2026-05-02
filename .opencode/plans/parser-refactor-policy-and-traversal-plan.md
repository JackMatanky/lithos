# Parser Refactor Plan: Policy-Driven IR + Iterator Traversal

## Status

- Requested before implementation.
- Legacy `lithos-core/src/note/parser/mod.rs` remains compatibility-oriented; new structural work targets `structure.rs`/`types.rs` first.
- Traversal choice is locked to **Option 2**: iterator/callback API instead of trait-heavy visitor.
- Stage 0-4.5b are complete (policy/IR/text projection migration + scanner boundary fixes).
- Stage 5a topology hardening is complete (typed mismatch/underflow errors + focused regressions).
- Remaining Stage 5 work is `structure.rs`-centric: variant payload processing nodes, list-only depth tracking, and tree-owned exact end matching.

## Executive Summary

This refactor will make parser behavior explicit, non-lossy by contract, and backend-isolated:

1. Introduce a declarative extension contract (`CmarkExtensionsPolicy` + `EventRetentionPolicy`).
2. Add parser-owned neutral IR/types (`parser/types.rs`) with zero pulldown leakage.
3. Refactor `stream.rs` to policy-enforced mapping (no implicit `None` drops).
4. Decouple `block.rs` from stream internals and pulldown types.
5. Rework `structure.rs` stack invariants with `ProcessingBlockTree` as correctness owner.
6. Replace `visitor.rs` with traversal iterator/callback API.
7. Add capability alignment matrix + contract tests for CommonMark and pulldown extensions.

Math is preserved now as first-class IR (`InlineMath`, `DisplayMath`).

---

## Problem Statement

### Current flaws (critical)

1. Parser domain leaks pulldown types across `block.rs`, `structure.rs`, `visitor.rs`.
2. Enabled features can be silently dropped (math, break events, optional extension tags).
3. Structure builder allows weak invariants (wide processing bags, silent invalid attach path).
4. Visitor API has high surface area and weak ROI relative to debt.

### Why this must be fixed

- Enabled feature loss is correctness debt.
- Tight pulldown coupling blocks future parser flexibility.
- Weak stack invariants reduce diagnostic quality and can mask topology bugs.
- Overgrown traversal interface hinders maintainability.

---

## Design Principles

1. Only adapter boundary (`stream.rs` + option synthesis) may know pulldown.
2. Enabled feature must never vanish silently.
3. Stack correctness belongs to `ProcessingBlockTree` (CommonMark Appendix A strategy).
4. Variant payloads over wide “bag structs”.
5. Traversal should be composable and minimal (iterator/callback).
6. Incremental migration with contract tests at every phase.
7. IR terminology should follow CommonMark or well-known markdown terms.

---

## 1) Introduce declarative extension contract (yes, do this)

Create a policy model that separates:

1. Parse capabilities enabled in pulldown
2. IR retention behavior (must not silently drop)
3. Downstream support expectation (structure/scanner/semantic phases)

### Suggested policy design

- `CmarkExtensionsPolicy`
  - `task_lists: Enabled | Disabled`
  - `wikilinks: Enabled | Disabled`
  - `math: Disabled | PreserveAsMathEvents | DegradeToText | Reject`
  - `metadata_blocks: Disabled | YamlOnly | YamlAndToml`
  - `strikethrough: Enabled | Disabled`
  - `tables: Disabled | Preserve`
  - `definition_lists: Disabled | Preserve`
  - `footnotes: Disabled | Preserve`

- `EventRetentionPolicy`
  - `unknown_block: Reject | DropWithDiagnostic`
  - `unknown_inline: Reject | DropWithDiagnostic`
  - `breaks: PreserveEvents | NormalizeText`

### Key invariant

If option enabled in pulldown, mapper must either:

- emit typed IR, or
- transform via declared policy, or
- emit structured ingest error.

Never implicit `None`.

Additional invariant from review:

- Policy model round-trips must be fidelity-safe (`from_options` + `to_options` cannot enable options that were not present in the source profile).

### Recommended defaults

- `math: PreserveAsMathEvents`
- `task_lists`, `wikilinks`, `metadata_blocks`, `strikethrough`: enabled
- `tables`, `definition_lists`, `footnotes`: explicit preserve/reject policy
- breaks default to `NormalizeText` only if scanner requires; otherwise `PreserveEvents`

---

## 2) Define neutral parser IR types (new shared types module)

Add a neutral type layer (`lithos-core/src/note/parser/types.rs`) used by `stream`, `structure`, `block`, and traversal.
Keep derived styled-text projection types in `lithos-core/src/note/parser/text.rs`.

### Naming rule (explicit)

- Prefer CommonMark names: `block`, `inline`, `list item`, `thematic break`,
  `metadata block`, `line break`.
- Avoid introducing new project-specific terms when CommonMark names already
  exist.

### Core types

- `BlockStart` (no pulldown types):
  - `Paragraph`
  - `Heading { level: HeadingLevel }`
  - `BlockQuote`
  - `List { kind: ListKind }`
  - `ListItem`
  - `CodeBlock { info_string: Option<Box<str>> }`
  - `Frontmatter { format: FrontmatterFormat }`

- `BlockEnd`:
  - matched end variants (`Paragraph`, `Heading`, `BlockQuote`, etc.)

- `InlineDelimiterStart` / `InlineDelimiterEnd`:
  - emphasis/strong/strikethrough/superscript/subscript
  - link/image payloads with parser-owned `LinkKind` + `destination/title/label`

- `InlineToken`:
  - `Text`, `InlineCode`, `Html`, `LineBreak(Soft|Hard)`, `Math(Inline|Display)`

- `ParserEvent`:
  - `BlockStart(BlockStart)`
  - `BlockEnd(BlockEnd)`
  - `Inline(InlineToken)`
  - `TaskListMarker(bool)`
  - `ThematicBreak`

- Parser-owned enums:
  - `FrontmatterFormat`, `LinkKind`, `HeadingLevel`, `ListKind`, `LineBreakKind`, `MathKind`

- Range wrapper:
  - `RangedEvent` (event + source byte range)

Then:

- `stream.rs` maps pulldown -> neutral types.
- No neutral type mentions `pulldown_cmark::*`.

---

## 3) Fix stream contract issues

In `stream.rs`:

- Remove `BlockType::as_start_tag` / `as_end_tag`.
- Replace all drop branches for:
  - `SoftBreak`, `HardBreak`, `InlineMath`, `DisplayMath`
  - `Table*`, `DefinitionList*`, `Footnote*`
- Route by policy:
  - preserve as explicit IR events, or
  - map to text via explicit policy, or
  - return structured error (`UnsupportedEnabledExtension`, etc).

Also:

- keep reference extraction logic
- keep source ranges
- make inline code/math payload contract explicit (content-only vs delimiter-retaining)
- keep merge behavior only when semantically safe (no merging across preserved math boundaries)

---

## 4) Decouple block AST from stream internals

`block.rs` changes:

- Remove `use pulldown_cmark::{CowStr, MetadataBlockKind}`.
- Remove stream event imports for `inline_events_text`.
- Introduce AST-owned inline representation for leaf text extraction:
  - either store canonical `InlineToken` slices,
  - or store materialized text payloads where needed.
- Replace:
  - `LeafBlockKind::CodeBlock { language: Option<CowStr> }` -> `Option<Box<str>>`
  - `Frontmatter { format: MetadataBlockKind }` -> `FrontmatterFormat`
- Replace `impl From<pulldown_cmark::HeadingLevel>` with adapter-local conversion.

---

## 5) Rework structure builder invariants

### Stage 5a sequencing lock (must run first)

Before broader `structure.rs` redesign, harden topology correctness in the
active ingestion path (`parser/mod.rs`) and eliminate all silent mismatch
no-ops:

- `mod.rs` frame finalization must return typed mismatch errors (no silent drop
  on leaf/container role mismatch).
- `structure.rs` attach/finalize paths must return typed topology errors with
  range context (no silent attach-to-leaf behavior).
- Add focused regression tests for these mismatch paths.

Only after Stage 5a is green should deeper `structure.rs` model reshaping
proceed.

`structure.rs` improvements:

- Remove pulldown imports.
- Consume only neutral IR types.
- Keep `ProcessingContainer`, but redesign with payload variants:
  - `ProcessingBlockQuote { ... }`
  - `ProcessingList { kind, ... }`
  - `ProcessingListItem { depth, parent_span, is_checked, ... }`
- Split depth tracking:
  - `list_depth` (list hierarchy only)
  - non-list container nesting must not mutate list hierarchy depth
- Enforce exact start/end matching:
  - Start heading must end heading, etc. (reuse `types::BlockEnd` semantics)
- Replace silent leaf-parent attach no-op with hard error.

### Appendix A alignment

`ProcessingBlockTree` becomes authoritative for:

- `start_block(...)`
- `end_block(...)`
- parent-child legality checks
- stack underflow/mismatch diagnostics

Note: avoid introducing redundant close-kind enums unless IR gaps require it;
prefer matching directly on `types::BlockEnd` with helper mappings.

`StructureBuilder` becomes orchestration thin layer only.

---

## 6) Visitor and API cleanup (chosen path: Option 2)

### Decision

Replace trait-heavy `visitor.rs` with traversal iterator/callback APIs.

### Proposed traversal API

- `DocTree::walk_preorder() -> impl Iterator<Item = TraversalEvent<'_>>`
- `TraversalEvent`:
  - `EnterBlock { block: &Block<'source>, depth: u32 }`
  - optional `ExitBlock { block: &Block<'source>, depth: u32 }`
- helpers:
  - `for_each_block(|block, depth| ...)`
  - optional typed filters

### Temporary compatibility

- Optional short-lived visitor adapter shim if migration needs it.
- Any retained signatures must use parser-owned types only:
  - `Option<&str>`/`Option<&Box<str>>` for language
  - `FrontmatterFormat`, `HeadingLevel`, `ListKind`

Traversal order remains pre-order, depth semantics unchanged.

---

## 7) CommonMark + pulldown capability alignment matrix (explicit)

Add capability contract doc and tests for:

- CommonMark core:
  - headings, paragraphs, lists, blockquote, code, thematic break, links/images, emphasis, html, soft/hard breaks
- pulldown extensions:
  - task lists, wikilinks, metadata blocks, strikethrough, math, tables, definition lists, footnotes

For each feature document and test:

`Enabled option -> IR behavior -> Structure behavior -> Scanner/semantic behavior -> Unsupported behavior`

This prevents future enabled-then-dropped regressions.

---

## 8) Test strategy (gates + new tests)

Update/add tests in `stream.rs`, `structure.rs`, `context.rs`.

### Contract tests

- extension enabled + policy preserve -> IR contains expected events
- policy reject -> typed parse error
- policy degrade -> deterministic transform

### Break policy tests

- `PreserveEvents` keeps break events
- `NormalizeText` produces deterministic text mapping

### Structure integrity tests

- mismatched end tags fail
- invalid parent attach fails (not silent)

### Integration tests

- `ParserContext` end-to-end for multiple policy profiles

### Quality gates

- `mise run lint`
- `mise run test:unit:note`
- `mise run verify`

---

## Addendum: Canonical text projection migration

Review findings after Stage 4 indicate text/scanner behavior remains split
between:

- legacy fragment flow in `parser/mod.rs` + `extractor.rs`, and
- derived projection types in `parser/text.rs`.

To avoid semantic drift and duplicated scan-policy logic, the next
implementation slice must establish `text.rs` as the only projection layer from
parser IR (`types.rs`) to scanner-friendly text/ranges.

Required effects:

1. Remove `TextFragment` / `FragmentPool` from parser orchestration.
2. Make sink and extractor consume event/range-based projection outputs.
3. Keep `text.rs` projection policy-agnostic; scanner/link/plain inclusion
   rules live at consumer boundaries (extractor/parser adapters).
4. Preserve existing observable extraction behavior with contract tests.
5. Treat disjoint scanner ranges as isolated lexical segments (no mode/alnum
   carry across excluded gaps).

---

## 9) Migration plan (safe sequence)

1. Add neutral types + policy types (no behavior change yet).
2. Adapt `stream.rs` to emit neutral IR while maintaining current defaults.
3. Port `structure.rs` to neutral IR + strict invariants.
4. Port `block.rs` AST payloads to neutral types.
5. Replace/update `visitor.rs` APIs and call sites.
6. Update `context.rs` tests and contract tests.
7. Tighten defaults:
   - choose math policy explicitly (`PreserveAsMathEvents` recommended).
8. Run full verify + benchmark parser hot paths.

---

## TextStyle / TextNode/TextSequence design (requested detail)

Use a derived style model in `parser/text.rs` for extraction ergonomics while retaining semantic IR.

### Context modeling decision

- Primary approach: embed derived text components directly under block/inline structures.
- Include a lightweight `TextContext` (`Normal | LinkLabel | ImageAlt`) to
  avoid duplicating link/image boundary checks across consumers.

Rationale:

- Embedding keeps the model concrete and easy to reason about.
- `TextContext` is useful as a unifying abstraction across origins (paragraph text,
  link label text, image alt text, inline code, inline/display math), but it is
  not required for correctness in the first iteration.

### Types

- `TextStyle`:
  - `Emphasis`, `Strong`, `Strikethrough`, `Code`, `MathInline`,
    `MathDisplay`
- `TextNode`:
  - `text: Box<str>`
  - `styles: Vec<TextStyle>` (or smallvec)
  - `context: TextContext`
  - `range: SourceByteRange`

- `TextSequence`:
  - ordered collection of `TextNode` values

### Pipeline placement

- Stream emits semantic IR.
- `text.rs` derives `TextNode`/`TextSequence` from inline delimiters + text
  tokens with context preserved.
- Consumer boundaries (scanner/link/plain extract) decide inclusion based on
  `TextStyle` + `TextContext`.

Rationale:

- Basalt-like ergonomics for styled text operations.
- No semantic precision loss for downstream behavior.

---

## Error model additions

- `UnsupportedEnabledExtension { feature, policy }`
- `UnexpectedBlockEnd { expected, got, range }`
- `StructuralStackUnderflow { tag, range }`
- `IllegalChildAttachment { parent_kind, child_kind, range }`
- `PolicyViolation { rule, range }`

All errors must preserve enough context for actionable diagnostics.

---

## Position handling scope

- `position.rs` hardening (byte-span helpers, UTF-8-safe conversions, span query
  helpers) is recommended and should be tracked as a follow-up immediately after
  parser-layer refactor completion.
- During current parser refactor, keep existing `SourceByteRange` usage and avoid
  broad `position.rs` redesign unless blocked by correctness issues.

---

## Risks and mitigations

### Risks

- Scope expansion while covering all extension paths.
- Temporary churn while replacing traversal APIs.
- Performance regressions from richer IR.

### Mitigations

- Strict phased commits.
- Capability-matrix contract tests early.
- Hot-path benchmarks after stream/structure migration.
- Temporary adapter shims with explicit removal tasks.

---

## Definition of Done

1. `block.rs`, `structure.rs`, traversal API surface are pulldown-free.
2. Math is preserved and tested in IR.
3. No enabled feature is silently dropped.
4. `ProcessingBlockTree` owns and enforces structural correctness.
5. Traversal API is iterator/callback based; visitor debt removed or shimed with sunset.
6. All parser quality gates green:
   - `mise run fmt`
   - `mise run lint`
   - `mise run test:unit:note`
   - `mise run verify`

---

## Suggested implementation commits

1. `docs(parser): detail policy-driven parser refactor blueprint`
2. `refactor(parser): add extension policy and neutral parser types`
3. `refactor(parser): enforce policy-driven stream event mapping`
4. `refactor(parser): enforce stack invariants in structure block tree`
5. `refactor(parser): decouple block ast from pulldown and stream internals`
6. `refactor(parser): replace visitor trait with traversal iterator api`
7. `test(parser): add capability matrix and contract coverage`
