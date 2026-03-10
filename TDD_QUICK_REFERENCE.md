# TDD Quick Reference for Extraction Refactor

**Use this as a quick checklist during implementation**

---

## The TDD Mantra

```
🔴 RED   → Write a failing test
🟢 GREEN → Make it pass (minimal code)
🔵 REFACTOR → Improve (tests stay green)
```

**Never write production code without a failing test first.**

---

## TDD Cycle Template

### Step 1: RED (Write Failing Test)

```rust
#[test]
fn extracts_simple_heading() {
    let mut extractor = HeadingExtractor::new();  // ← Doesn't exist
    let ctx = ExtractionContext::default();

    // ... process events ...

    assert_eq!(heading.text(), "Title");  // ← Will fail
}
```

**Run**: `cargo test extracts_simple_heading`
**Expect**: ❌ Compilation error or test failure

### Step 2: GREEN (Minimal Implementation)

```rust
pub struct HeadingExtractor {
    current: Option<HeadingBuilder>,
}

impl HeadingExtractor {
    pub fn new() -> Self {
        Self { current: None }
    }
}

// Minimal code to pass test
impl Extractor for HeadingExtractor {
    // ... just enough to make test pass
}
```

**Run**: `cargo test extracts_simple_heading`
**Expect**: ✅ Test passes

### Step 3: REFACTOR (Improve Code)

```rust
impl HeadingExtractor {
    // Extract helper methods
    fn start_heading(&mut self, level: HeadingLevel, position: usize) {
        // ...
    }

    // Add documentation
    /// Accumulates heading text across multiple events.
    fn push_text(&mut self, text: &str) {
        // ...
    }
}
```

**Run**: `cargo test`
**Expect**: ✅ All tests still pass

### Step 4: COMMIT

```bash
git add lithos-core/src/note/adapter/extract_heading.rs
git commit -m "feat(adapter): add basic heading extraction

- Extract H1-H6 headings
- Accumulate text across events
- Add 7 unit tests

TDD: Red → Green → Refactor"
```

---

## TDD Rules (Non-Negotiable)

| Rule | Why |
|------|-----|
| ✅ **Write test FIRST** | Ensures testability |
| ✅ **Run test** (verify red) | Confirms test actually tests something |
| ✅ **Minimal code** to pass | Prevents over-engineering |
| ✅ **Run test** (verify green) | Confirms implementation works |
| ✅ **Refactor** while green | Improves design safely |
| ✅ **Commit on green** | Never commit broken code |

---

## Common TDD Mistakes

| Mistake | Fix |
|---------|-----|
| ❌ Writing code before test | Stop. Delete code. Write test first. |
| ❌ Writing many tests at once | One test at a time |
| ❌ Not running test in red | Always verify test fails first |
| ❌ Over-implementing in green | Write simplest code to pass |
| ❌ Skipping refactor | Always clean up after green |
| ❌ Committing on red | Only commit when all tests pass |

---

## Quick Command Reference

### Run Single Test
```bash
cargo test test_name
```

### Run Tests for Module
```bash
cargo test extract_list
```

### Run Tests with Output
```bash
cargo test test_name -- --nocapture
```

### Run All Tests
```bash
cargo test
# OR
mise run test
```

### Watch Mode (Optional)
```bash
cargo watch -x "test extract_list"
```

---

## Test Structure (Arrange-Act-Assert)

```rust
#[test]
fn test_name() {
    // ARRANGE - Set up test data
    let mut extractor = ListExtractor::new(&config);
    let ctx = ExtractionContext::default();
    let event = Event::Start(CmarkTag::List(None));

    // ACT - Perform action
    let result = extractor.process(&event, text, range, &ctx).unwrap();

    // ASSERT - Verify outcome
    assert!(matches!(result, ExtractionState::Continue));
}
```

---

## TDD Anti-Patterns

### ❌ Implementation-Driven Testing

```rust
// BAD: Testing implementation details
#[test]
fn list_stack_has_correct_size() {
    assert_eq!(extractor.list_stack.len(), 1);  // ← Internal detail
}
```

### ✅ Behavior-Driven Testing

```rust
// GOOD: Testing observable behavior
#[test]
fn emits_list_when_closed() {
    let result = extractor.process(&end_event, ...)?;
    assert!(matches!(result, ExtractionState::Emit(_)));  // ← Behavior
}
```

---

## TDD Checklist (Per Feature)

```markdown
- [ ] Write failing test
- [ ] Run test → verify RED
- [ ] Write minimal implementation
- [ ] Run test → verify GREEN
- [ ] Refactor code
- [ ] Run all tests → verify still GREEN
- [ ] Add documentation
- [ ] Commit with TDD tag
- [ ] Move to next feature
```

---

## Daily TDD Metrics

Track at end of each day:

```markdown
## Day X TDD Report

**Tests First**: ✅/❌ (for each feature)
**Red-Green-Refactor Cycles**: ___ completed
**Average Time in Red**: ___ minutes
**Commits on Green**: ___/___
**Coverage**: ___% (cargo tarpaulin)
**Insights**: What worked well? What was challenging?
```

---

## Example: Complete TDD Session

### Feature: Extract Simple Heading

#### 1. RED (5 min)
```rust
#[test]
fn extracts_h1_heading() {
    let mut extractor = HeadingExtractor::new();
    // ... test code ...
    assert_eq!(heading.text(), "Title");
}
```
**Run**: ❌ Compilation error

#### 2. GREEN (15 min)
```rust
pub struct HeadingExtractor { /* ... */ }
impl Extractor for HeadingExtractor { /* ... */ }
```
**Run**: ✅ Test passes

#### 3. REFACTOR (10 min)
- Extract `start_heading()` helper
- Add documentation
- Improve naming

**Run**: ✅ All tests pass

#### 4. COMMIT (2 min)
```bash
git commit -m "feat: extract H1 headings (TDD)"
```

#### 5. REPEAT
Next test: `extracts_h2_through_h6`

---

## Emergency: If You Get Stuck

1. **Stop writing code**
2. **Run tests** - where are we? (red/green?)
3. **If red**:
   - Is test correct?
   - What's simplest code to pass?
4. **If green**:
   - Refactor what you have
   - Write next test
5. **If tests failing unexpectedly**:
   - Revert to last green commit
   - Start over with smaller step

---

## Success Indicators

You're doing TDD correctly when:

- ✅ You never wonder "will this work?" (tests tell you)
- ✅ You refactor fearlessly (tests protect you)
- ✅ You have 100% test coverage (every line justified)
- ✅ Your tests document behavior (tests = specs)
- ✅ You commit frequently (always on green)
- ✅ You write less code (tests prevent over-engineering)

---

## Remember

> "Test-first is not about testing. It's about **design**."
>
> Tests written first force you to think about:
> - What is the simplest API?
> - What behavior do I need?
> - How will this be used?
>
> This leads to better, simpler, more maintainable code.

**Now go write some tests! 🔴 → 🟢 → 🔵**
