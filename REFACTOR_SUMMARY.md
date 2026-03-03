# Extraction Refactor - Executive Summary

**Status**: Ready for TDD Implementation
**Created**: 2026-03-03
**Estimated Effort**: 4.5 days with TDD
**Methodology**: Test-Driven Development (Red → Green → Refactor)

---

## What We're Doing

Replacing a **2,186-line god-object** (`reader.rs`) with **7 focused extractors** (~1,100 lines total) following the **Extractor pattern**.

### Before
```
reader.rs (2,186 lines)
├── ParseState (12 fields - all coupled)
├── 7 Collectors (interdependent)
├── Tag scanning (duplicated)
├── Task parsing (tightly coupled)
└── Frontmatter conversion (150 lines duplicated)
```

**Problems**:
- ❌ Untestable in isolation
- ❌ Cross-collector coupling
- ❌ Parse all or nothing
- ❌ Impossible to extend
- ❌ 2000+ lines to understand

### After
```
reader.rs (200 lines - protocol + orchestration)
├── ExtractionContext (shared state)
├── ExtractionState (result enum)
└── Extractor trait (protocol)

extract_list.rs (300 lines)
extract_link.rs (150 lines)
extract_heading.rs (100 lines)
extract_section.rs (150 lines)
extract_frontmatter.rs (120 lines)
extract_tag.rs (80 lines)
```

**Benefits**:
- ✅ Unit testable in isolation
- ✅ Zero cross-extractor coupling
- ✅ Composable (extract subsets)
- ✅ Easy to extend (add new extractor)
- ✅ 48% code reduction

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Extractor pattern** | Clean protocol for event → entity transformation |
| **Flat structure** | 7 files < 10 threshold for subdirectory |
| **Protocol in reader.rs** | Reader orchestrates; extractors implement |
| **`pub(super)` visibility** | Protocol visible to siblings, not external |
| **Unified list extraction** | Checkboxes ARE list items (no duplication) |
| **`CowStr<'_>` everywhere** | Zero-copy text from pulldown-cmark |
| **Test-Driven Development** | 100% test-first coverage |

---

## TDD Methodology

Every feature follows strict **Red → Green → Refactor**:

1. **RED**: Write failing test first (5-30 min)
2. **GREEN**: Minimal code to pass (15-60 min)
3. **REFACTOR**: Improve while green (10-20 min)
4. **COMMIT**: Git commit on green only

### TDD Rules (Non-Negotiable)

✅ **Write test BEFORE code**
✅ **Run test to verify RED**
✅ **Write simplest code to pass**
✅ **Run test to verify GREEN**
✅ **Refactor while staying green**
✅ **Commit only on green**
✅ **Never commit failing tests**

---

## Implementation Phases

### Phase 1: Foundation (Day 1 Morning - 4 hours)
**TDD**: Write tests first for all domain enhancements

- [ ] Add `EmbedType::from_extension` (TDD: 60 min)
- [ ] Add `FieldValue` conversion methods (TDD: 90 min)
- [ ] Create extraction protocol (TDD: 60 min)

**Deliverables**: 22+ tests, 3 commits, all green

---

### Phase 2: Core Extractors (Day 1 Afternoon - Day 2 Morning - 8 hours)
**TDD**: Multiple red-green-refactor cycles per extractor

#### List Extractor (3 hours)
- RED → GREEN → REFACTOR: Basic lists (1.5 hours)
- RED → GREEN: Checkbox support (50 min)
- RED → GREEN → REFACTOR: Task promotion (1.5 hours)
- **Deliverables**: 10+ tests, ListExtractor complete

#### Link Extractor (3 hours)
- RED → GREEN → REFACTOR: Wiki-links (1 hour)
- RED → GREEN → REFACTOR: Anchors (1 hour)
- RED → GREEN: Markdown links & embeds (1 hour)
- **Deliverables**: 10+ tests, LinkExtractor complete

#### Heading Extractor (2 hours)
- RED → GREEN → REFACTOR: H1-H6 extraction (1 hour)
- RED → GREEN: Text accumulation (1 hour)
- **Deliverables**: 7+ tests, HeadingExtractor complete

---

### Phase 3: Secondary Extractors (Day 2-3 - 6 hours)

#### Section Extractor (2 hours)
- TDD cycles for block tracking and heading association
- **Deliverables**: 6+ tests

#### Frontmatter Extractor (2 hours)
- TDD cycles for YAML/TOML parsing
- **Deliverables**: 8+ tests

#### Tag Extractor (2 hours)
- TDD cycles for tag scanning
- **Deliverables**: 10+ tests

---

### Phase 4: Reader Orchestration (Day 3 - 3 hours)
**TDD**: Integration tests before wiring

- RED: Write integration test (30 min)
- GREEN: Wire all extractors (1.5 hours)
- REFACTOR: Extract helpers, cleanup (1 hour)
- **Deliverables**: 5+ integration tests, reader.parse_str() complete

---

### Phase 5: Integration & Performance (Day 4 - 6 hours)

#### Characterization Tests (2 hours)
- Document current behavior with tests
- Verify new implementation matches exactly

#### Property-Based Tests (2 hours)
- Use proptest for fuzzing
- Verify invariants hold

#### Performance Verification (2 hours)
- Run existing `benches/note_parsing.rs`
- Compare before/after with baseline
- Ensure no regression >10%

**Deliverables**: 15+ tests, performance verified

---

### Phase 6: Cleanup (Day 4-5 - 2 hours)

- [ ] Delete old files (tag_scanner.rs, task_parser.rs)
- [ ] Update mod.rs
- [ ] Update documentation
- [ ] Final `mise run verify` → 100% pass

---

## Success Metrics

### Functional
- [ ] All existing tests pass
- [ ] All new tests pass (70+ new tests)
- [ ] No functionality regression
- [ ] 100% test-first coverage

### Performance
- [ ] No regression >10% in any benchmark
- [ ] `note_parsing/simple` ≤ 15 µs
- [ ] `note_parsing/medium` ≤ 21 µs
- [ ] `note_parsing/complex` ≤ 53 µs
- [ ] Scaling remains O(n)

### Code Quality
- [ ] 48% code reduction (2,186 → 1,100 lines)
- [ ] Cyclomatic complexity ≤10 per function
- [ ] Zero cross-extractor coupling
- [ ] Each extractor testable in isolation

### TDD Discipline
- [ ] 100% test-first (every line justified by failing test)
- [ ] All commits on green
- [ ] Average time in red <5 min/cycle
- [ ] Documentation complete

---

## Key Files

### Implementation Plan
📄 **REFACTOR_PLAN_EXTRACTION.md** (2,941 lines)
- Complete TDD cycles for every component
- Detailed test examples
- Edge case documentation
- Migration checklist (80+ items)

### TDD Reference
📄 **TDD_QUICK_REFERENCE.md** (390 lines)
- Quick TDD cycle template
- Common mistakes and fixes
- Daily tracking templates
- Emergency procedures

### Existing Benchmark
📄 **lithos-core/benches/note_parsing.rs** (458 lines)
- Already comprehensive
- 3 benchmarks (simple/medium/complex)
- Baseline comparison strategy
- Performance regression detection

---

## Risk Mitigation

### High Risk: Performance Regression
**Mitigation**:
- Save baseline before refactor
- Run benchmarks after each phase
- Profile with flamegraph if issues
- Rollback if >10% regression

### Medium Risk: Behavior Changes
**Mitigation**:
- Characterization tests document current behavior
- Property tests catch edge cases
- Integration tests verify full pipeline

### Low Risk: TDD Overhead
**Mitigation**:
- TDD adds 0.5 days but prevents debugging later
- Tests = living documentation
- Confident refactoring

---

## Rollback Plan

If integration tests fail:
1. Revert Phase 4 commits (keep extractors)
2. Debug in isolation
3. Re-integrate with more logging

If performance regresses >10%:
1. Profile with `cargo flamegraph`
2. Identify hot path
3. Optimize (reduce clones, pre-allocate)
4. If unfixable, revert and redesign

---

## Quick Start

### Day 1 Morning (Start Here)

```bash
# 1. Create feature branch
git checkout -b refactor/extraction-architecture

# 2. Start with Phase 1, Task 1.1
# Open: REFACTOR_PLAN_EXTRACTION.md
# Navigate to: Phase 1 → Task 1.1
# Follow: RED → GREEN → REFACTOR cycle

# 3. Keep TDD reference open
# Open: TDD_QUICK_REFERENCE.md

# 4. Track progress
# Check off items in Migration Checklist
```

### After Each Feature

```bash
# Verify tests pass
cargo test

# Commit on green
git add .
git commit -m "feat: [description]

TDD: Red → Green → Refactor"
```

### End of Day

```bash
# Run full verification
mise run verify

# Update TDD metrics
# See TDD_QUICK_REFERENCE.md → Daily TDD Metrics
```

---

## Expected Timeline

| Day | Phase | Hours | Deliverables |
|-----|-------|-------|--------------|
| 1 | Foundation + List Extractor | 8 | Protocol + List extraction working |
| 2 | Link + Heading + Section | 8 | 3 more extractors complete |
| 3 | Frontmatter + Tag + Reader | 8 | All extraction working end-to-end |
| 4 | Testing + Performance | 6 | All tests pass, no regression |
| 4.5 | Cleanup + Docs | 2 | Ready to merge |

**Total**: 4.5 days with TDD discipline

---

## Questions?

1. **Why TDD?**
   Tests-first prevents over-engineering, ensures testability, provides living documentation, enables confident refactoring.

2. **Why 0.5 days extra for TDD?**
   Writing tests first adds time upfront but saves debugging time later. Net positive ROI.

3. **What if I get stuck?**
   See TDD_QUICK_REFERENCE.md → Emergency section. Revert to last green commit and take smaller steps.

4. **Can I skip tests for "simple" features?**
   **No.** 100% test-first is non-negotiable. If it's simple, test will be fast to write.

5. **What if tests are hard to write?**
   That's the design telling you something. Hard to test = hard to use. Simplify the API.

---

## Final Checklist Before Starting

- [ ] Read TDD Framework section in main plan
- [ ] Read TDD_QUICK_REFERENCE.md
- [ ] Understand Red → Green → Refactor cycle
- [ ] Know how to run tests (`cargo test`)
- [ ] Have criterion installed for benchmarks
- [ ] Feature branch created
- [ ] Ready to write first test (not code!)

**Remember**: Test-first is not about testing. It's about **design**.

---

**Now go write some tests!** 🔴 → 🟢 → 🔵
