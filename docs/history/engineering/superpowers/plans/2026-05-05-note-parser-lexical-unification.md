# Note Parser Lexical Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify `note/parser`, lexical scanning, and raw note assembly into a single coherent parser pipeline where facts live in parser text context, policy decisions are centralized, and lexical recognition is isolated from policy.

**Architecture:** Replace the current split (`parser` + `extractor` + `scanner`) with a parser-owned 3-phase pipeline: parse (`stream/structure/text`) -> lexical selection + recognition (`parser::lexical` + scanner recognizer) -> raw assembly (`parser::assemble`). Eliminate duplicated scannability logic (`TextNode::is_scannable`, extractor range filtering, scanner policy flags) by making `TextContext` factual and using `SourceByteRangeIndex` as the only lexical scan boundary carrier.

**Tech Stack:** Rust, pulldown-cmark, existing Lithos parser IR (`RangedEvent`, `DocTree`, `TextSequence`), existing scanner state-machine rules.

---

## 0. Scope and Non-Goals

- In-scope:
  - Parser submodule redesign to own lexical + assembly phases.
  - Remove overlap among `parser/text.rs`, `extractor.rs`, `scanner.rs`.
  - Preserve existing externally observable behavior of `MarkdownParser::parse`.
- Out-of-scope:
  - New artifact kinds.
  - Changing RawNote schema.
  - Flavor-specific scanning expansions.

## 1. Target Module Ownership (Final State)

- `note/parser/stream.rs`: event adaptation and normalization only.
- `note/parser/structure.rs`: AST topology only.
- `note/parser/text.rs`: inline projection + factual context flags only.
- `note/parser/lexical.rs` (new): policy + range index builder + scanner invocation only.
- `note/parser/assemble.rs` (new): convert parsed blocks + lexical artifacts into `RawNote` only.
- `note/scanner.rs` (or renamed `artifact_lexer.rs`): lexical recognizer only.
- `note/extractor.rs`: removed.

## 2. Design Contracts to Enforce

1. Facts are parser-owned: text and block contexts are encoded in parser types.
2. Decisions are policy-owned: include/exclude decisions are in one place (`parser::lexical`).
3. Lexical recognizer never decides context eligibility.
4. Assembly never re-scans source.
5. `SourceByteRangeIndex` is the only scan-boundary substrate.

---

### Task 1: Introduce Parser-Owned Factual Context Model

**Files:**
- Modify: `lithos-core/src/note/parser/text.rs`
- Modify: `lithos-core/src/note/parser/mod.rs` (tests that reference scannability semantics)
- Test: `lithos-core/src/note/parser/text.rs` (existing unit tests + new context tests)

- [ ] **Step 1: Replace `TextContext` enum with factual flags**

Use a compact bitflag/newtype for factual context. Minimum flags:

```rust
pub(crate) struct TextContext(u16);

impl TextContext {
    pub(crate) const NONE: Self = Self(0);
    pub(crate) const IN_LINK_LABEL: Self = Self(1 << 0);
    pub(crate) const IN_IMAGE_ALT: Self = Self(1 << 1);
    pub(crate) const IN_CODE_INLINE: Self = Self(1 << 2);
    pub(crate) const IN_MATH_INLINE: Self = Self(1 << 3);
    pub(crate) const IN_MATH_DISPLAY: Self = Self(1 << 4);
    pub(crate) const IN_CODE_BLOCK: Self = Self(1 << 5);
    pub(crate) const IN_FRONTMATTER: Self = Self(1 << 6);
}
```

- [ ] **Step 2: Move style-derived facts into context at node creation time**

In `InlineStyleContext::create_node`, set factual bits from style/depth so facts are co-located on `TextNode`.

- [ ] **Step 3: Remove `TextNode::is_scannable`**

Delete hard-coded scannability rules from `TextNode`. Keep `is_displayable` only if needed for heading display.

- [ ] **Step 4: Add/adjust tests proving factual context generation**

Add tests for link labels, image alt text, code inline, math inline/display.

- [ ] **Step 5: Run focused tests**

Run: `cargo test -q note::parser::text`

Expected: parser text tests pass with no `is_scannable` references.

- [ ] **Step 6: Commit**

```bash
git add lithos-core/src/note/parser/text.rs lithos-core/src/note/parser/mod.rs
git commit -m "refactor(parser): make text context factual and remove node scannability"
```

---

### Task 2: Add Parser Lexical Phase (`parser/lexical.rs`)

**Files:**
- Create: `lithos-core/src/note/parser/lexical.rs`
- Modify: `lithos-core/src/note/parser/mod.rs`
- Modify: `lithos-core/src/note/position.rs` (if needed for index ergonomics)
- Test: `lithos-core/src/note/parser/lexical.rs` (new tests)

- [ ] **Step 1: Define policy surface and artifact kind**

```rust
pub(crate) enum ArtifactKind { Tag, InlineField, BlockRef }

pub(crate) trait ScanPolicy: Send + Sync {
    fn allow(&self, artifact: ArtifactKind, ctx: TextContext) -> bool;
}

pub(crate) struct DefaultScanPolicy;
```

- [ ] **Step 2: Implement `DefaultScanPolicy` rules**

Exclude artifacts when context includes any of:
- link label
- image alt
- code inline
- math inline/display
- frontmatter
- code block

- [ ] **Step 3: Implement scan boundary builder using `SourceByteRangeIndex`**

```rust
pub(crate) fn build_scan_index(
    projection: &TextSequence,
    policy: &dyn ScanPolicy,
) -> SourceByteRangeIndex
```

This function must:
- iterate text nodes,
- apply policy,
- append approved ranges,
- preserve source order.

- [ ] **Step 4: Add lexical phase entrypoint**

```rust
pub(crate) fn scan_projection<'a>(
    scanner: &NoteScanner,
    source: &'a str,
    projection: &TextSequence,
    policy: &dyn ScanPolicy,
) -> Result<ScannedRawArtifacts<'a>, NoteError>
```

This should call scanner with `SourceByteRangeIndex` directly.

- [ ] **Step 5: Add tests for policy + index generation**

Test cases:
- link labels excluded,
- code/math excluded,
- normal paragraph included,
- ordering preserved.

- [ ] **Step 6: Run focused tests**

Run: `cargo test -q note::parser::lexical`

Expected: lexical tests pass.

- [ ] **Step 7: Commit**

```bash
git add lithos-core/src/note/parser/lexical.rs lithos-core/src/note/parser/mod.rs lithos-core/src/note/position.rs
git commit -m "feat(parser): add lexical phase with centralized scan policy"
```

---

### Task 3: Purify Scanner into Recognizer-Only API

**Files:**
- Modify: `lithos-core/src/note/scanner.rs`
- Modify: `lithos-core/src/note/parser/lexical.rs`
- Test: `lithos-core/src/note/scanner.rs`

- [ ] **Step 1: Change `scan_ranges` signature to accept `SourceByteRangeIndex`**

```rust
pub(crate) fn scan_ranges<'source>(
    &self,
    text: &'source str,
    ranges: &SourceByteRangeIndex,
) -> Result<ScannedRawArtifacts<'source>, NoteError>
```

- [ ] **Step 2: Remove `_include_task_marker` parameter and any latent policy hooks**

Scanner must not carry contextual eligibility logic.

- [ ] **Step 3: Keep cursor/rule engine unchanged except input iteration adapter**

Use `ranges.iter()` and `range.as_usize_range()`; preserve behavior and offsets.

- [ ] **Step 4: Update scanner unit tests to use `SourceByteRangeIndex`**

Rewrite existing disjoint-range tests using index builder fixtures.

- [ ] **Step 5: Run focused tests**

Run: `cargo test -q note::scanner`

Expected: scanner tests pass with same semantics.

- [ ] **Step 6: Commit**

```bash
git add lithos-core/src/note/scanner.rs lithos-core/src/note/parser/lexical.rs
git commit -m "refactor(scanner): make scanner recognizer-only with range index input"
```

---

### Task 4: Move Raw Assembly into Parser (`parser/assemble.rs`)

**Files:**
- Create: `lithos-core/src/note/parser/assemble.rs`
- Modify: `lithos-core/src/note/parser/mod.rs`
- Modify: `lithos-core/src/note/extractor.rs` (deprecate / thin wrapper during migration)
- Test: `lithos-core/src/note/parser/mod.rs` tests

- [ ] **Step 1: Copy `BlockExtractor` logic into parser-owned assembler type**

Create `RawAssembler` (or `ParserAssembler`) in `parser/assemble.rs` with clear API:

```rust
pub(crate) struct RawAssembler<'source> { ... }

impl<'source> RawAssembler<'source> {
    pub(crate) fn new(source: &'source str, scanner: NoteScanner) -> Self;
    pub(crate) fn process_doc_tree(
        &mut self,
        tree: &DocTree<'source, Complete>,
        policy: &dyn ScanPolicy,
    ) -> Result<(), NoteIngestError>;
    pub(crate) fn finish(self) -> RawNote<'source>;
}
```

- [ ] **Step 2: Replace direct projection scanning with `parser::lexical::scan_projection`**

All tag/field/blockref extraction from text projections must go through lexical phase.

- [ ] **Step 3: Keep link extraction in assembler phase only**

Do not move link extraction into scanner; links are parser-semantic, not lexical token matches.

- [ ] **Step 4: Update `MarkdownParser::parse` orchestration to call parser-owned assembler**

`parser/mod.rs` should no longer import `note::extractor::BlockExtractor`.

- [ ] **Step 5: Run parser integration tests**

Run: `cargo test -q note::parser`

Expected: existing parser behavior tests pass (including link/tag exclusion scenarios).

- [ ] **Step 6: Commit**

```bash
git add lithos-core/src/note/parser/assemble.rs lithos-core/src/note/parser/mod.rs lithos-core/src/note/extractor.rs
git commit -m "refactor(parser): move raw note assembly into parser submodule"
```

---

### Task 5: Remove Overlap and Delete Legacy Extractor

**Files:**
- Delete: `lithos-core/src/note/extractor.rs`
- Modify: `lithos-core/src/note/mod.rs` (if module re-exports exist)
- Modify: `lithos-core/src/note/parser/mod.rs` docs/comments
- Test: parser + note integration suites

- [ ] **Step 1: Remove final callsites to `note::extractor`**

Ensure all assembly is parser-owned.

- [ ] **Step 2: Delete legacy extractor module**

Remove file and module declarations cleanly.

- [ ] **Step 3: Update architecture docs in parser module header**

Revise phase list to reflect real pipeline:
1) stream adapter
2) structure builder
3) lexical policy + scanner recognizer
4) artifact assembly

- [ ] **Step 4: Run full note-focused tests**

Run:
- `cargo test -q note::parser`
- `cargo test -q note::scanner`
- `cargo test -q note::task`

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor(note): remove legacy extractor and finalize parser-owned pipeline"
```

---

### Task 6: Hardening Invariants and Regression Coverage

**Files:**
- Modify: `lithos-core/src/note/parser/mod.rs` tests
- Modify: `lithos-core/src/note/parser/text.rs` tests
- Modify: `lithos-core/src/note/scanner.rs` tests
- Create: `lithos-core/tests/note_lexical_policy_integration.rs`

- [ ] **Step 1: Add invariant tests for factual context + policy matrix**

Matrix rows:
- normal text
- link label
- image alt
- inline code
- inline/display math
- frontmatter
- code block

Columns:
- tag
- inline field
- block ref

Assertions: allow/deny exactly per policy.

- [ ] **Step 2: Add regression tests for non-leakage**

Cases:
- tag inside link label not emitted globally,
- field inside code/math not emitted,
- list child content not leaking into parent item extraction.

- [ ] **Step 3: Add range-order determinism test**

Ensure same lexical artifacts are emitted regardless of traversal path ordering assumptions for equivalent text coverage.

- [ ] **Step 4: Run integration tests**

Run: `cargo test -q note_lexical_policy_integration`

Expected: all invariants hold.

- [ ] **Step 5: Commit**

```bash
git add lithos-core/tests/note_lexical_policy_integration.rs lithos-core/src/note/parser/mod.rs lithos-core/src/note/parser/text.rs lithos-core/src/note/scanner.rs
git commit -m "test(note): add lexical policy invariants and non-leakage regressions"
```

---

### Task 7: Final Verification and Hygiene

**Files:**
- Modify: any touched files for final clippy/rustfmt/doc cleanup

- [ ] **Step 1: Run full quality gates**

Run: `mise run verify`

Expected: fmt + clippy + unit + integration + docs checks pass.

- [ ] **Step 2: Run docs tests if public docs changed**

Run: `cargo test --doc`

Expected: doctests pass.

- [ ] **Step 3: Architectural conformance spot-check**

Run: `cargo test -q architecture`

Expected: architecture boundary tests pass.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "refactor(parser): complete lexical-policy unification and pipeline cleanup"
```

---

## Spec Coverage Check

- Unify parser/scanner/extractor into one parser-owned pipeline: covered by Tasks 2, 4, 5.
- Facts as `in_*` context in parser text model: covered by Task 1.
- Use `SourceByteRangeIndex` over ad-hoc span type: covered by Tasks 2 and 3.
- Remove overlap and enforce parsimonious ownership: covered by Tasks 3 and 5.
- Full implementation detail with affected components: all tasks include exact files and APIs.

## Risk Notes and Mitigations

- Risk: behavior drift during extractor relocation.
  - Mitigation: move code first, then delete legacy module, with integration tests at each step.
- Risk: context flags becoming inconsistent with existing style stack.
  - Mitigation: generate context facts at node creation and test matrix directly.
- Risk: scanner offset regressions with new index input.
  - Mitigation: preserve existing cursor logic and adapt only iteration layer; retain disjoint-range tests.

## Rollout Strategy

1. Land factual model.
2. Land lexical phase + policy.
3. Land scanner API change.
4. Move assembly.
5. Remove extractor.
6. Add invariant suite.
7. Full verify.
