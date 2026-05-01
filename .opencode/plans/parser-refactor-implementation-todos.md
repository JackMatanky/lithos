# Parser Refactor Implementation Todos

## Purpose

Execution checklist for the policy-driven parser refactor.

- Source plan: `.opencode/plans/parser-refactor-policy-and-traversal-plan.md`
- Scope: `lithos-core/src/note/parser/*` (legacy `mod.rs` is compatibility-only; refactor work should target `structure.rs`/`types.rs` unless safety fixes are required)
- Constraint: preserve math events and remove silent drops.

## Execution Rules

- Complete stages strictly in order (`0 -> 9`).
- At each stage boundary:
  - run the listed validation commands,
  - fix failures before moving to the next stage,
  - commit only the stage scope.
- If a stage requires temporary compatibility shims, add explicit follow-up todo in Stage 9 cleanup.

## Pre-Flight Gaps Confirmed From Re-Review

- `config.rs` now has extension + retention modeling, but critical integration gaps remain:
  - retention policy semantics are not yet wired into `stream.rs` unknown-event paths
  - metadata policy round-trip is currently lossy for pluses-only metadata options
  - default profile is defined in multiple places and needs single-source deduplication
- parser config review (subagent) identified additional cleanup/test gaps:
  - broad module-level dead-code suppression in `config.rs` hides useful unused-path signals
  - unknown-event policy diagnostics are too generic for triage/debugging
  - missing integration tests proving unknown-event enforcement in stream mapping
  - missing regression test for metadata round-trip fidelity
  - missing constructor/default coherence tests
- `stream.rs` still silently drops `SoftBreak`, `HardBreak`, `InlineMath`, `DisplayMath`, footnotes, and extension tags in mapper branches.
- `stream.rs` still exposes reverse conversion helpers (`as_start_tag`, `as_end_tag`) via block IR types.
- `block.rs` still imports pulldown types and stream internals.
- `structure.rs` still imports pulldown types and depends on `stream::BlockType`; stack invariants are split and one invalid attach path is silent.
- `visitor.rs` is trait-heavy and pulldown-leaking; `DocStructure::walk` is coupled to it.

---

## Stage 0 - Baseline and Safety Rails

- [x] Capture baseline parser behavior with focused tests for current losses:
  - [x] math event currently dropped
  - [x] break behavior under each `BreakPolicy`
  - [x] optional extension tags dropped
- [x] Add/expand parser error variants needed by new policy/stack contract.
- [x] Add a short design note in docs/artifacts describing new parser contract boundaries.

Acceptance criteria:

- Baseline tests exist and explicitly document current behavior vs target behavior.
- Error variants compile and are imported where needed.

Validation:

- `mise run test:unit:note`

---

## Stage 1 - Introduce Policy Layer in `config.rs`

Files:

- `lithos-core/src/note/parser/config.rs`

Tasks:

- [x] Add `CmarkExtensionsPolicy` with explicit fields:
  - [x] `task_lists`
  - [x] `wikilinks`
  - [x] `math`
  - [x] `metadata_blocks`
  - [x] `strikethrough`
  - [x] `tables`
  - [x] `definition_lists`
  - [x] `footnotes`
- [x] Add `EventRetentionPolicy`:
  - [x] `unknown_block`
  - [x] `unknown_inline`
  - [x] `breaks`
- [x] Add typed policy enums (`Enabled/Disabled`, `Preserve/Reject/Degrade` variants as specified).
- [x] Refactor `EventStreamConfig` to carry policy fields.
- [x] Add option synthesis method from policy to pulldown `Options`.
- [x] Define explicit default profile with `math: PreserveAsMathEvents`.
- [x] Keep compatibility constructors if needed for phased migration.

Acceptance criteria:

- Policy model is complete and documented.
- Defaults are deterministic and tested.

Validation:

- `mise run lint`
- `mise run test:unit:note`

---

## Stage 2 - Add Neutral Parser Types Module

Files:

- `lithos-core/src/note/parser/types.rs` (new)
- `lithos-core/src/note/parser/text.rs` (new)
- `lithos-core/src/note/parser/*` imports updated

Tasks:

- [ ] Add parser-owned enums/structs:
- [x] Add parser-owned enums/structs:
  - [x] `BlockStart`, `BlockEnd`
  - [x] `InlineDelimiterStart`, `InlineDelimiterEnd`, `InlineToken`
  - [x] `ParserEvent`
  - [x] `HeadingLevel`, `ListKind`, `FrontmatterFormat`, `LinkKind`, `LineBreakKind`, `MathKind`
  - [x] `RangedEvent` for event + source range
  - [x] `TextStyle` and `TextNode`/`TextSequence` as derived inline text model in `text.rs`
- [x] Ensure zero `pulldown_cmark::*` references in `types.rs`.
- [x] Define conversion helpers local to adapter boundary only (not in domain types).
- [x] Add unit tests for neutral type invariants and simple constructors.

Acceptance criteria:

- Neutral type layer compiles and is imported by stream/structure/block paths.
- No pulldown leakage in neutral type definitions.

Validation:

- `mise run lint`
- `cargo test -p lithos-core note::parser::types::tests -- --nocapture`
- `cargo test -p lithos-core note::parser::text::tests -- --nocapture`

---

## Stage 3 - Refactor `stream.rs` to Policy-Enforced Mapping

Files:

- `lithos-core/src/note/parser/stream.rs`

Tasks:

- [x] Replace current IR mappings to emit neutral types from `types.rs`.
- [x] Remove reverse conversion methods (`as_start_tag`, `as_end_tag`).
- [x] Replace every silent-drop path with policy-gated behavior:
  - [x] `SoftBreak`
  - [x] `HardBreak`
  - [x] `InlineMath`
  - [x] `DisplayMath`
  - [x] `FootnoteReference`
  - [x] table tags/events
  - [x] definition list tags/events
- [x] Keep reference extraction logic intact.
- [x] Preserve source range mapping for all emitted events.
- [x] Wire `EventRetentionPolicy` enforcement for unknown block/inline events; no silent unknown drop.
- [x] Improve policy violation diagnostics with concrete observed event context.
- [x] Define delimiter normalization contract for inline code/math:
  - [x] decide whether `InlineToken::InlineCode` and `InlineToken::Math` store raw payload or delimiter-stripped payload
  - [x] codify decision in mapper docs and tests
- [x] Restrict `TextMergeWithOffset` usage when policies require boundary-sensitive retention.
  - [x] Add explicit guard/tests for math boundary safety.

Acceptance criteria:

- No enabled extension is silently dropped.
- Mapping outcomes are deterministic under policy.

Validation:

- `mise run test:unit:note`
- Focused stream tests for each policy profile
- `cargo test -p lithos-core note::parser::stream::tests::event_adapter_iter_extension_drop_baseline -- --nocapture`

---

## Stage 4 - Decouple `block.rs` From Pulldown and Stream Internals

Files:

- `lithos-core/src/note/parser/block.rs`

Tasks:

- [x] Remove pulldown imports (`CowStr`, `MetadataBlockKind`).
- [x] Remove direct dependency on `stream::{ParserEvent, InlineEvent, EventWithRange}`.
- [x] Switch to parser-owned `types::{ParserEvent, InlineToken, RangedEvent}` and `text` projection as needed.
- [x] Replace leaf payload types:
  - [x] code language -> `Option<Box<str>>`
  - [x] frontmatter format -> `FrontmatterFormat`
- [x] Replace heading conversion impl from pulldown with parser-owned conversion at adapter boundary.
- [x] Introduce AST-owned inline representation strategy:
  - [x] Option A: store `InlineToken` vectors in leaf blocks
  - [ ] Option B: store materialized text payloads with optional rich inline storage
- [x] Update `block.text()` and related helpers to use AST-owned structures only.

Acceptance criteria:

- `block.rs` has zero `pulldown_cmark` imports.
- `block.rs` has zero direct `stream` IR coupling.

Validation:

- `mise run lint`
- `mise run test:unit:note`

---

## Stage 5 - Rework Structure Builder Invariants (`structure.rs`)

Precondition note (must complete before Stage 5 invariants work):

- Parser text projection is still split across legacy fragment flow and
  `text.rs`.
- Implement the canonical `text.rs` migration slice below first, then proceed
  to Stage 5 topology/invariant hardening.

### Stage 4.5 - Canonical Text Projection Migration (`text.rs`)

Files:

- `lithos-core/src/note/parser/text.rs`
- `lithos-core/src/note/parser/block.rs`
- `lithos-core/src/note/parser/structure.rs`
- `lithos-core/src/note/parser/mod.rs`
- `lithos-core/src/note/extractor.rs`
- `lithos-core/src/note/parser/context.rs` (tests/doc alignment)

Tasks:

- [x] Promote `text.rs` to canonical projection engine:
  - [x] `TextSequence::from_events(&[RangedEvent])`
  - [x] style stack handling from delimiter start/end tokens
  - [x] `as_plain_text()` and source-covering range helper
- [x] Remove legacy parser fragment path from `mod.rs`:
  - [x] remove `TextFragment`
  - [x] remove `FragmentPool`
  - [x] store and propagate parser events/ranges for leaf completion
- [x] Update sink API to consume event-derived projection input (not fragments).
- [x] Keep `text.rs` policy-agnostic; scannable filtering owned by extractor/scanner boundary.
- [x] Keep parser IR authoritative in `types.rs`; no pulldown leakage.
- [x] Remove/retire `Block::text()` and `Block::is_scannable()` if no longer
      needed by callers.
- [x] Stage 4.5b projection model cleanup:
  - [x] remove `ProjectionPolicy` from `text.rs` (consumer-owned filtering)
  - [x] keep `TextNode`/`TextSequence` names and add `TextContext`
  - [x] represent code/math as `TextStyle` variants
  - [x] move scanner/link inclusion decisions to extractor/parser boundaries
  - [x] fix empty projection fallback range to block-anchored span
  - [x] fix scanner disjoint-range lexical boundary semantics (no cross-range mode/prev_alnum carry)

Acceptance criteria:

- Single source of truth for text projection exists in `text.rs`.
- No duplicated text/scannable logic remains in parser/extractor paths.
- Heading/paragraph/list/frontmatter extraction behavior remains unchanged.

Validation:

- `mise run test:unit:note`
- `cargo test -p lithos-core note::parser:: -- --nocapture`

Files:

- `lithos-core/src/note/parser/mod.rs`
- `lithos-core/src/note/parser/structure.rs`

Tasks:

- [x] Stage 5a parser/topology correctness hardening (production path first):
  - [x] remove silent frame-role mismatch no-ops in `mod.rs` finalization
  - [x] return typed topology mismatch errors (`EventStackMismatch`) in `mod.rs`
  - [x] populate mismatch diagnostics with meaningful payload fields (`depth`, labels)
  - [x] remove silent attach-to-leaf no-op in `structure.rs`
  - [x] return typed topology/underflow errors with range context in `structure.rs`
  - [x] add focused failing-then-passing regression tests for mismatch paths

Remaining Stage 5 scope (structure-first, minimal legacy churn):
- [x] Remove pulldown imports.
- [x] Consume only neutral `types.rs` events/tags.
- [x] Keep `ProcessingContainer`, but redesign with variant payload structs:
  - [x] `ProcessingBlockQuote`
  - [x] `ProcessingList`
  - [x] `ProcessingListItem`
- [ ] Refactor `ProcessingLeaf` similarly if needed for symmetry and invariants.
      (deferred: optional, not required for Stage 5 completion)
- [x] Split depth tracking:
  - [x] `list_depth`
  - [x] ensure non-list nesting does not mutate list hierarchy depth
- [x] Move structural correctness into `ProcessingBlockTree`:
  - [x] legal `start_block` (no-start-inside-leaf enforcement)
  - [x] exact `end_block` matching (reuse `types::BlockEnd` semantics)
  - [x] legal parent-child attach rules
  - [x] no silent no-op on invalid attach
- [x] Ensure end-tag mismatch yields typed errors with range context.

Acceptance criteria:

- [x] Structure builder fails fast on topology violations.
- [x] No silent attach failures remain.

**Stage 5: COMPLETE**

Validation:

- `mise run test:unit:note`

---

## Stage 6 - Replace Visitor Trait With Traversal Iterator/Callback API

Files:

- `lithos-core/src/note/parser/visitor.rs` (replace or compatibility shim)
- `lithos-core/src/note/parser/structure.rs` traversal methods

Tasks:

- [ ] Implement `walk_preorder()` iterator API returning traversal events.
- [ ] Add convenience traversal helpers:
  - [ ] `for_each_block`
  - [ ] optional typed filters
- [ ] Update structure docs/examples to use iterator/callback model.
- [ ] Remove pulldown types from traversal API.
- [ ] Decide whether to:
  - [ ] delete `BlockVisitor` trait entirely, or
  - [ ] keep temporary adapter shim with deprecation note.
- [ ] Port existing visitor tests to traversal event tests.

Acceptance criteria:

- Traversal semantics (pre-order, depth) preserved.
- Technical debt from default no-op trait methods removed or isolated to temporary adapter.

Validation:

- `mise run test:unit:note`

---

## Stage 7 - Structure/Text Contract Hardening

Files:

- `lithos-core/src/note/parser/structure.rs`
- `lithos-core/src/note/parser/text.rs`
- `lithos-core/src/note/parser/context.rs`

Tasks:

- [ ] Add contract tests for `TextContext` + `TextStyle` filtering boundaries:
  - [ ] scanner excludes link/image/code/math nodes by intended rules
  - [ ] link display includes intended label/code nodes and excludes math
- [ ] Add range-contract tests for `TextSequence::covering_range()` in nested/mixed inline scenarios.
- [ ] Verify no duplicated projection/filtering logic drifts back into parser/extractor.
- [ ] Add malformed-event topology tests that assert typed errors (no stringly fallbacks).

Acceptance criteria:

- Projection/scanning contracts are enforced by tests and remain policy-consistent.
- No silent topology/policy regressions pass parser test gates.

Validation:

- new unit tests for nested style combinations and range correctness

---

## Stage 8 - Capability Matrix and Contract Tests

Files:

- `lithos-core/src/note/parser/stream.rs` tests
- `lithos-core/src/note/parser/structure.rs` tests
- `lithos-core/src/note/parser/context.rs` tests
- capability doc artifact under `.opencode/plans` or project docs

Tasks:

- [ ] Add matrix doc rows for each feature:
  - [ ] option enabled state
  - [ ] stream IR behavior
  - [ ] structure behavior
  - [ ] scanner/semantic expectation
  - [ ] unsupported fallback/error path
- [ ] Add automated tests per matrix row.

Acceptance criteria:

- Any new enabled feature without handler fails tests.
- No future silent drops can pass CI.

Validation:

- `mise run test:unit:note`
- `mise run lint`

---

## Stage 9 - Final Integration and Quality Gates

Tasks:

- [ ] Remove obsolete compatibility code and stale clippy `#[expect]` blocks.
- [ ] Verify docs/examples reflect new traversal and policy APIs.
- [ ] Run full quality suite.
- [ ] Run parser hot-path benchmark comparison (before/after).
- [ ] Create follow-up task for `position.rs` hardening (byte-span utilities and UTF-8-safe conversions) after parser refactor merges.

Validation commands:

- `mise run fmt`
- `mise run lint`
- `mise run test:unit:note`
- `mise run verify`

Acceptance criteria:

- All gates pass.
- Behavior deltas are intentional and documented.
- Performance regressions (if any) are understood and accepted.

---

## Cross-Stage Watch Items (Do Not Miss)

- [ ] Minimal module wiring may still be needed for new `types.rs` declaration; treat as mechanical compile plumbing only.
- [ ] Keep source range fidelity through every transformation.
- [ ] Keep `ReferenceDefinitions` behavior unchanged unless explicitly scoped.
- [ ] Ensure `ParserContext` remains the canonical cache boundary after traversal API changes.
- [ ] Avoid introducing new string allocation anti-patterns in hot paths.

## Suggested Commit Slices

- [ ] `refactor(parser): add extension policy model`
- [ ] `refactor(parser): introduce parser neutral types module`
- [ ] `refactor(parser): enforce stream mapping policy and preserve math`
- [ ] `refactor(parser): decouple block ast from pulldown`
- [ ] `refactor(parser): enforce structure stack invariants`
- [ ] `refactor(parser): replace visitor trait with traversal iterator`
- [ ] `test(parser): add capability matrix contract coverage`
