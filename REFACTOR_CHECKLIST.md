# Extraction Refactor - Quick Checklist

**Methodology**: TDD (Red → Green → Refactor)
**Estimated**: 4.5 days
**Status**: [ ] Not Started

---

## Phase 1: Foundation ✅

### Task 1.1: EmbedType::from_extension (60 min) ✅
- [x] RED: Write 7 tests for extension detection (15m)
- [x] GREEN: Implement from_extension method (20m)
- [x] REFACTOR: Add docs, inline hints (15m)
- [x] COMMIT: "feat(note): add EmbedType::from_extension"
- [x] VERIFY: `cargo test -p lithos-core --lib note::link::tests`

### Task 1.2: FieldValue Conversion (90 min) ✅
- [x] RED: Write 8 round-trip tests (30m)
- [x] GREEN: Implement to_yaml_value and to_toml_value (40m)
- [x] REFACTOR: Polish error handling (10m)
- [x] COMMIT: "feat(note): add FieldValue bidirectional conversion"
- [x] VERIFY: `cargo test conversion_tests`

### Task 1.3: Extraction Protocol (60 min) ✅
- [x] RED: Write protocol tests with mock extractor (20m)
- [x] GREEN: Add ExtractionContext, ExtractionState, Extractor (25m)
- [x] REFACTOR: Add comprehensive documentation (10m)
- [x] COMMIT: "feat(adapter): add extraction protocol"
- [x] VERIFY: `cargo test protocol_tests`

**Phase 1 Complete**: [x] All tests green, 3 commits

---

## Phase 2: Core Extractors ✅ / 🔴 / ⏸️

### Task 2.1: List Extractor (3 hours)

#### Cycle 1: Basic Lists (90 min)
- [ ] RED: Write plain list tests (30m)
- [ ] GREEN: Implement basic ListExtractor (45m)
- [ ] REFACTOR: Extract helpers, docs (15m)
- [ ] VERIFY: `cargo test extract_list`

#### Cycle 2: Checkboxes (50 min)
- [ ] RED: Write checkbox tests (20m)
- [ ] GREEN: Add checkbox handling (30m)
- [ ] VERIFY: `cargo test extract_list`

#### Cycle 3: Task Promotion (80 min)
- [ ] RED: Write promotion tests (30m)
- [ ] GREEN: Implement promotion logic (45m)
- [ ] REFACTOR: Clean up (20m)
- [ ] COMMIT: "feat(adapter): add list extractor with task promotion"
- [ ] VERIFY: `cargo test extract_list` (10+ tests pass)

### Task 2.2: Link Extractor (3 hours)

#### Cycle 1: Wiki-links (60 min)
- [x] RED: Write wiki-link tests (20m)
- [x] GREEN: Implement link state machine (30m)
- [x] REFACTOR: Extract anchor parsing (10m)
- [x] VERIFY: Tests pass

#### Cycle 2: Anchors (60 min)
- [x] RED: Write anchor tests (heading + block-ref) (20m)
- [x] GREEN: Implement anchor parsing (30m)
- [x] REFACTOR: Use domain EmbedType (10m)
- [x] VERIFY: Tests pass

#### Cycle 3: Markdown Links (60 min)
- [x] RED: Write markdown link + embed tests (20m)
- [x] GREEN: Implement markdown links (30m)
- [x] REFACTOR: Extract URL validation (10m)
- [ ] COMMIT: "feat(adapter): add link extractor"
- [x] VERIFY: `cargo test extract_link` (10+ tests pass)

### Task 2.3: Heading Extractor (2 hours)

#### Cycle 1: H1-H6 (60 min)
- [x] RED: Write heading level tests (20m)
- [x] GREEN: Implement heading extraction (30m)
- [x] REFACTOR: Extract helpers (10m)
- [x] VERIFY: Tests pass

#### Cycle 2: Text Accumulation (60 min)
- [x] RED: Write text + break tests (20m)
- [x] GREEN: Implement text accumulation (30m)
- [x] REFACTOR: Polish (10m)
- [ ] COMMIT: "feat(adapter): add heading extractor"
- [x] VERIFY: `cargo test extract_heading` (7+ tests pass)

**Phase 2 Complete**: [ ] 3 extractors, 27+ tests, all green

---

## Phase 3: Secondary Extractors ✅ / 🔴 / ⏸️

### Task 3.1: Section Extractor (2 hours)
- [x] RED: Write section boundary tests (30m)
- [x] GREEN: Implement block tracking (60m)
- [x] REFACTOR: Add heading association (30m)
- [ ] COMMIT: "feat(adapter): add section extractor"
- [x] VERIFY: `cargo test extract_section` (6+ tests pass)

### Task 3.2: Frontmatter Extractor (2 hours)
- [x] RED: Write YAML/TOML tests (30m)
- [x] GREEN: Implement metadata parsing (60m)
- [x] REFACTOR: Use FieldValue::from_yaml (30m)
- [ ] COMMIT: "feat(adapter): add frontmatter extractor"
- [x] VERIFY: `cargo test extract_frontmatter` (8+ tests pass)

### Task 3.3: Tag Extractor (2 hours)
- [ ] RED: Write tag scanning tests (30m)
- [ ] GREEN: Implement tag extraction (60m)
- [ ] REFACTOR: Add context awareness (30m)
- [ ] COMMIT: "feat(adapter): add tag extractor"
- [ ] VERIFY: `cargo test extract_tag` (10+ tests pass)

**Phase 3 Complete**: [ ] 3 more extractors, 24+ tests, all green

---

## Phase 4: Reader Orchestration ✅ / 🔴 / ⏸️

### Task 4.1: Orchestrate Extractors (3 hours)
- [ ] RED: Write integration test (30m)
- [ ] GREEN: Wire extractors in parse_str (90m)
- [ ] REFACTOR: Extract update_context helper (60m)
- [ ] COMMIT: "feat(adapter): orchestrate extraction in reader"
- [ ] VERIFY: `cargo test -p lithos-core --lib note::adapter::reader`

**Phase 4 Complete**: [ ] Full pipeline works, 5+ integration tests

---

## Phase 5: Testing & Performance ✅ / 🔴 / ⏸️

### Task 5.1: Characterization Tests (2 hours)
- [ ] Capture baseline: `cargo bench --bench note_parsing -- --save-baseline before_refactor`
- [ ] RED: Document current behavior with tests (60m)
- [ ] GREEN: Verify new impl matches (45m)
- [ ] REFACTOR: Add new isolation tests (15m)
- [ ] COMMIT: "test: add characterization tests"

### Task 5.2: Property-Based Tests (2 hours)
- [ ] RED: Write 4+ proptest properties (45m)
- [ ] GREEN: Fix any failures (60m)
- [ ] REFACTOR: Add more properties (15m)
- [ ] COMMIT: "test: add property-based tests"

### Task 5.3: Performance Verification (2 hours)
- [ ] Run benchmark: `cargo bench --bench note_parsing -- --baseline before_refactor`
- [ ] Verify simple ≤15µs (±10%)
- [ ] Verify medium ≤21µs (±10%)
- [ ] Verify complex ≤53µs (±10%)
- [ ] If regression >10%: Profile and optimize
- [ ] Update benchmark docs if needed

**Phase 5 Complete**: [ ] 70+ tests total, no performance regression

---

## Phase 6: Cleanup ✅ / 🔴 / ⏸️

### Task 6.1: Delete Old Code (30 min)
- [ ] Delete: `tag_scanner.rs`
- [ ] Delete: `task_parser.rs`
- [ ] Update: `mod.rs` (remove old module refs)
- [ ] COMMIT: "refactor: remove old extraction code"

### Task 6.2: Update Documentation (60 min)
- [ ] Update: `note/adapter/mod.rs` module docs
- [ ] Update: `ARCHITECTURE.md` extraction section
- [ ] Update: `CHANGELOG.md` add refactor entry
- [ ] COMMIT: "docs: update extraction architecture"

### Task 6.3: Final Verification (30 min)
- [ ] Run: `mise run verify` → 100% pass
- [ ] Run: `cargo test --doc` → doctests pass
- [ ] Run: `cargo clippy` → no warnings
- [ ] Run: `cargo fmt -- --check` → formatted
- [ ] Search: `rg "tag_scanner|task_parser"` → no refs
- [ ] COMMIT: "chore: final cleanup"

**Phase 6 Complete**: [ ] Ready to merge

---

## Pre-Merge Checklist ✅ / 🔴

- [ ] All tests pass: `mise run test`
- [ ] No clippy warnings: `mise run lint`
- [ ] Code formatted: `mise run fmt`
- [ ] Benchmarks green: `cargo bench --bench note_parsing`
- [ ] Documentation complete
- [ ] CHANGELOG.md updated
- [ ] All commits on green
- [ ] Squash/organize commits if needed
- [ ] Write merge commit message

---

## TDD Metrics (Track Daily)

### Day 1
- Tests written first: ___% (target: 100%)
- Red-Green-Refactor cycles: ___ (target: 10+)
- Average time in red: ___ min (target: <5)
- Commits on green: ___/___ (target: 100%)

### Day 2
- Tests written first: ___% (target: 100%)
- Red-Green-Refactor cycles: ___ (target: 10+)
- Average time in red: ___ min (target: <5)
- Commits on green: ___/___ (target: 100%)

### Day 3
- Tests written first: ___% (target: 100%)
- Red-Green-Refactor cycles: ___ (target: 8+)
- Average time in red: ___ min (target: <5)
- Commits on green: ___/___ (target: 100%)

### Day 4
- Tests written first: ___% (target: 100%)
- Red-Green-Refactor cycles: ___ (target: 5+)
- Average time in red: ___ min (target: <5)
- Commits on green: ___/___ (target: 100%)

---

## Quick Commands

```bash
# Run specific test
cargo test test_name

# Run module tests
cargo test extract_list

# Run all tests
mise run test

# Run benchmarks
cargo bench --bench note_parsing

# Save baseline
cargo bench --bench note_parsing -- --save-baseline before_refactor

# Compare to baseline
cargo bench --bench note_parsing -- --baseline before_refactor

# Profile
cargo flamegraph --bench note_parsing

# Watch mode
cargo watch -x "test extract_list"
```

---

## Status Legend

- ✅ Complete
- 🔴 In Progress
- ⏸️ Blocked
- ⏭️ Skipped

---

**Last Updated**: 2026-03-03 (Phase 1 complete ✅)
**Current Phase**: Phase 2 - Core Extractors (Task 2.1 next)
**Blockers**: None
