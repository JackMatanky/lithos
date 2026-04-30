# Parser Contract Design Note

## Purpose

This note defines parser-layer contracts for the ongoing refactor in
`lithos-core/src/note/parser/*`.

It exists to prevent semantic drift and silent behavior changes during staged
migration from pulldown-cmark adapter output to parser-owned IR.

## Scope

- In scope:
  - parser event adaptation (`stream.rs`)
  - parser IR (`types.rs`)
  - structure builder contracts (`structure.rs`)
  - derived text projection (`text.rs`)
- Out of scope:
  - legacy `parser/mod.rs` behavior as source of truth
  - renderer-specific semantics

## Canonical Terms

Use CommonMark/well-known names only:

- `BlockStart`, `BlockEnd`
- `Paragraph`, `Heading`, `BlockQuote`, `List`, `ListItem`, `CodeBlock`, `Frontmatter`, `ThematicBreak`
- `InlineToken`, `InlineCode`, `LineBreak`, `InlineMath`, `DisplayMath`

Avoid introducing project-specific aliases for these concepts.

## IR Boundary Contract

1. `stream.rs` is the only pulldown-aware adapter layer.
2. All downstream parser components consume parser-owned IR types.
3. No parser-owned type may expose `pulldown_cmark::*` in public fields.
4. Prefer reusing `types::BlockStart` / `types::BlockEnd` for structure
   correctness checks; avoid introducing redundant close-kind enums unless IR
   expressiveness gaps are demonstrated.

## Feature Policy Contract

For each enabled parser option, mapper behavior must be explicit:

- Emit typed IR, or
- Apply documented transformation, or
- Emit structured parse error.

Silent drop is forbidden.

Policy fidelity requirement:

- Policy <-> options conversion must be non-expansive and round-trip-safe.
- `from_options(...).to_options()` must not enable additional parser options that were absent in the input profile.

## Current Baseline (documented known gaps)

- Footnotes: currently not emitted as first-class parser IR token.
- Tables: currently not emitted as first-class parser IR block structure.
- Legacy `parser/mod.rs` remains compatibility-oriented and is not parser source of truth.

These are transitional states and must be covered by contract tests.

## Payload Contract: Inline Code and Math

Based on pulldown-cmark behavior:

- `InlineCode` payload is content-only (no backtick delimiters).
- `InlineMath`/`DisplayMath` payloads are content-only (no `$`/`$$` delimiters).

Contract tests must lock this behavior.

Additionally, policy-enforcement integration tests must lock unknown-event behavior:

- `Reject` returns typed parse error with range context.
- `DropWithDiagnostic` degrades deterministically without silent ambiguity.

## Range Contract

- Parser event wrappers carry `SourceByteRange` for diagnostics.
- Derived text nodes carry content ranges.
- Delimiter positions remain in semantic IR, not duplicated in text projection by default.

## Text Projection Contract

- `text.rs` is a layer above parser IR.
- `TextNode` and `TextSequence` are derived views for scanner ergonomics.
- Semantic IR remains available and authoritative.
- `text.rs` owns canonical projection from parser IR to derived text nodes.
- Scan/link/plain inclusion policy is consumer-owned (extractor/parser boundary)
  and must not fork into competing projection pipelines.

Current migration caveat:

- Legacy compatibility code paths in `parser/mod.rs` should be minimized;
  new structural work should target `structure.rs` + parser IR contracts.

## Error Taxonomy Direction

Prefer specific typed parser errors over free-form reason strings.

Target examples:

- stack underflow/mismatch
- invalid transition/topology
- unsupported enabled extension
- policy violation with range context

Keep existing variants for compatibility during migration; stop adding new
stringly-typed parser errors.

## Placement Guidance

- Keep this design note as a durable artifact under `.opencode/plans/`.
- Also include a concise summary in module docs (`parser/types.rs`, `parser/stream.rs`, `parser/text.rs`) so contracts stay visible at implementation sites.

## Stage 5 Design Direction

- `ProcessingBlockTree` should own structural correctness (start/end legality,
  stack underflow/mismatch, and attach legality) per CommonMark Appendix A
  parsing strategy.
- If processing-node kind splits are introduced, favor placing the minimal
  discriminants in `types.rs` when it reduces duplication across structure
  components without leaking builder internals.
