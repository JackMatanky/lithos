# Note Parser Cleanup Working Checklist

Date: 2026-04-29
Scope: `lithos-core/src/note/parser/` (full module, including `mod.rs`)
Constraint: no intentional runtime behavior change unless explicitly documented.

## Baseline Health (Current)

- `mise run test:unit:note` passes.
- `mise run lint` completes with parser warnings present (dead code and lint expectations in parser modules).
- `mise run verify` passes end-to-end.

Recorded warning hotspots to resolve during consolidation:

- `lithos-core/src/note/parser/block.rs`
  - wildcard enum match arm in `Block::text`
  - pattern type mismatch in `Block::text`
  - unused methods: `is_scannable`, `HeadingLevel::as_u8`
- `lithos-core/src/note/parser/config.rs`
  - `trivially_copy_pass_by_ref` on `EventStreamConfig` accessors
- `lithos-core/src/note/parser/stream.rs`
  - dead enum variants: `InlineTag::{InlineMath, DisplayMath, Html}`
  - dead enum variants: `InlineTagEnd::{InlineMath, DisplayMath, Html}`
- `lithos-core/src/note/parser/visitor.rs`
  - unfulfilled `dead_code` lint expectation at module level

## Parser Invariants To Preserve

1. Source range fidelity:
   - Every emitted parser event keeps a valid `SourceByteRange` mapped from original markdown bytes.
2. Parser context caching contract:
   - Parse once in `ParserContext`, reuse cached events and reference definitions.
3. Break policy semantics:
   - `BreakPolicy` replacement behavior for soft/hard breaks remains unchanged.
4. Reference normalization semantics:
   - Case folding, whitespace collapse, and backslash unescaping remain stable.
5. AST assembly semantics:
   - `DocTree::from_context` stack assembly keeps current list depth/task marker/parent span behavior.
6. Visitor traversal semantics:
   - Pre-order traversal and depth propagation behavior remain stable.
7. Context isolation:
   - Parser changes stay within note context and do not introduce cross-context imports.

## Known Bloat/Overlap Targets

- Duplicate text flattening logic in `block.rs` and `structure.rs`.
- Staged/unused APIs with `dead_code` suppressions (`stream.rs`, `block.rs`).
- Potential stale test model drift in `structure.rs` relative to current block/event IR.
- Overlapping coverage between `context.rs` tests and `context_integration_test.rs`.
- Legacy parser implementation remains embedded in `mod.rs` and duplicates modular responsibilities in `stream.rs`, `references.rs`, and `structure.rs`.

## Phase-0 Parity Checklist (Must Preserve)

- Link/reference behavior:
  - first reference definition wins
  - case-insensitive reference lookup
  - whitespace-collapsed labels resolve correctly
  - backslash-escaped labels resolve correctly
  - multiline reference destinations resolve correctly
  - references inside frontmatter are ignored
  - references inside fenced code are ignored
- Link target behavior:
  - fragments preserved for `obsidian://`, `file://`, and `s3://` targets
- Metadata extraction behavior:
  - tags in headings are captured
  - tags inside links are ignored
  - block refs in paragraph tail are captured
  - block refs inside links are ignored
  - inline fields are extracted
- List semantics:
  - nested list depth tracked correctly
  - checkbox marker state captured correctly
- Frontmatter semantics:
  - frontmatter captured when present at start

## Change Control Rules For This Cleanup

- Prefer deletions and consolidation over new abstractions.
- Keep behavior same first, then simplify implementation.
- Any behavior delta must include:
  - explicit rationale,
  - targeted test updates,
  - note in final readiness report.

## Validation Plan During Cleanup

- Fast loop: targeted note parser tests where possible.
- Milestones: `mise run test:unit:note` and `mise run lint`.
- Final gate: `mise run verify` (or report baseline blocker if scanner module issue remains unresolved).
