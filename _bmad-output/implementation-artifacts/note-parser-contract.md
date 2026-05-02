# Note Parser Contract (Pre-Scanner Baseline)

Date: 2026-04-29
Scope: `lithos-core/src/note/parser/` (full module, including `mod.rs`)
Status: active cleanup contract for scanner/extraction/assembly planning.

## Ownership Boundaries

### `stream.rs`

- Owns pulldown-cmark adapter boundary.
- Owns event normalization:
  - break policy transformation,
  - optional adjacent text merge,
  - conversion into `ParserEvent`.
- Owns event-to-range pairing via `EventWithRange`.
- Owns extraction handoff for link reference definitions.
- Is the intended long-term canonical adapter boundary.

### `mod.rs`

- Current state: mixed facade + legacy parser implementation.
- Near-term policy: preserve output behavior while migrating responsibilities
  into canonical modules.
- End-state policy: reduce `mod.rs` to public facade/orchestration only.

### `context.rs`

- Owns parse-once caching of:
  - normalized event stream,
  - normalized reference definitions,
  - source borrow.
- Guarantees downstream parser stages do not re-run pulldown parsing.

### `structure.rs`

- Owns flat-event-to-AST assembly (`DocTree::from_context`).
- Owns stack/tree assembly algorithm and list nesting/task marker semantics.
- Must not re-interpret markdown source directly.

### `block.rs`

- Owns block-domain representation (`Block`, `BlockKind`, leaf/container kinds).
- Owns block-level helpers such as text extraction/scannability checks.
- Must remain storage/scanner agnostic.

### `visitor.rs`

- Owns traversal abstraction only.
- Must not duplicate AST assembly logic.

### `references.rs`

- Owns CommonMark-like reference label normalization and O(1) lookup.

### `config.rs`

- Owns parser stream policy knobs only (`Options`, break policy, merge policy).

## ParserEvent Mapping Contract

### Preserved and mapped into IR

- Block boundaries:
  - `Tag::Paragraph`, `TagEnd::Paragraph`
  - `Tag::Heading`, `TagEnd::Heading`
  - `Tag::BlockQuote`, `TagEnd::BlockQuote`
  - `Tag::List`, `TagEnd::List`
  - `Tag::Item`, `TagEnd::Item`
  - `Tag::CodeBlock`, `TagEnd::CodeBlock`
  - `Tag::MetadataBlock`, `TagEnd::MetadataBlock`
- Inline boundaries/content:
  - emphasis/strong/strikethrough/superscript/subscript start/end
  - link/image start + link/image end
  - text, code span, html/inline html
- Other:
  - task list marker
  - thematic break

### Intentionally dropped by current contract

- `SoftBreak` and `HardBreak` as discrete events after normalization stage
  - either rewritten to text by break policy or omitted from IR
- `FootnoteReference`
- `InlineMath`
- `DisplayMath`
- Table tags/events
- Definition list tags/events
- Footnote definition tags/events
- Html block tags/events

Downstream implication:

- Scanner/extraction must not rely on dropped event classes unless parser contract is explicitly expanded first.

Math-specific implication:

- `Options::ENABLE_MATH` is enabled in parser configuration, but
  `stream.rs` currently drops `Event::InlineMath` and `Event::DisplayMath`
  from the parser IR.
- Until that contract changes, math content is not exposed as dedicated IR
  events to downstream scanner/extractor stages.

## Consolidation Rule

- Where `mod.rs` and modular parser components overlap, modular components are
  the source of truth for future behavior unless an explicit parity exception is
  documented.

## Keep/Remove Decision for Borrowed Block Views

Near-term policy:

- Keep `LeafBlockRef`, `ContainerBlockRef`, `BlockKind::as_leaf`, and
  `BlockKind::as_container` only if scanner/extraction will consume them in the
  immediate next implementation slice.
- If not consumed by scanner/extraction, remove them to reduce dormant API
  surface and dead-code suppressions.

Current recommendation:

- Remove in cleanup unless a scanner design task explicitly chooses borrowed
  views as its traversal API.
