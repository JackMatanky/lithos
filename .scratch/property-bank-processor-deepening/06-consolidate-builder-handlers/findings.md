# Findings & Analysis: Consolidate Builder Handler Chain

## Requirements

- Move builder's property bank orchestration into `PropertyBankProcessor` as a `run()` method
- Apply TDD with vertical slices through each pipeline path
- Follow rust-best-practices (typestate discipline, interface design)
- Remove the builder's 7 handler methods
- Collapse builder import from 15 types to 1
- Do NOT touch SchemaProcessor (out of scope)

## Blast Radius (GitNexus Impact Analysis)

**Risk: LOW** across all handler methods. No external consumers.

| Symbol | Kind | Direct Callers | Depth | Risk |
|--------|------|---------------|-------|------|
| `load_property_bank` | Method | 1 (`load_all`) | d=2 (test) | LOW |
| `handle_present` | Method | 1 (`load_property_bank`) | d=3 (test) | LOW |
| `handle_content_mismatch` | Method | 1 (`handle_present`) | d=3 (test) | LOW |
| `handle_analysis_branch` | Method | 1 (`handle_content_mismatch`) | d=3 (test) | LOW |
| `handle_missing` | Method | 1 (`load_property_bank`) | d=3 (test) | LOW |
| `fetch_fresh` | Method | 1 (`handle_present`, `sync_and_fetch_*`) | d=2..3 | LOW |
| `sync_and_fetch_timestamps` | Method | 1 (`handle_content_mismatch`) | d=3 | LOW |
| `sync_and_fetch_content` | Method | 1 (`handle_analysis_branch`) | d=3 | LOW |

Key finding: the handler chain is a **linear call chain with zero fan-out**. Each handler has exactly one caller (the next method up). The chain is purely internal to the builder module — no test outside `builder.rs` imports any handler directly.

No execution flows beyond the builder test are affected. Risk assessment: **LOW** (<5 symbols, 1 test process).

### PropertyBankProcessor Upstream Consumers

The processor struct itself has incoming calls from:
- `transition_from_parts` (internal)
- Two test functions in the processor's own test module

No external modules consume processor internals directly — all external access goes through the Builder.

---

## Architecture Vocabulary (per LANGUAGE.md)

**Module**: `PropertyBankProcessor` — the typestate pipeline engine.
**Interface**: Currently includes ~15 exported types + per-(Stage,Status) methods. Callers must know about `Comparison`, `Parsed`, `Present`, `Missing`, `Suspect`, `TimestampBranch`, `ContentBranch`, `AnalysisBranch`, `Refresh`, `StaleTimestamps`, `StaleContent`, `Construction`, `Fresh`, `Changed`, `New`, `Completed`, and 3 completed-ready variants.
**Depth**: The module is internally deep (complex typestate) but its interface is shallow — the builder must orchestrate each transition explicitly.
**Seam**: The `pub(crate)` visibility boundary at `property_bank_processor.rs`.
**Adapter**: `Builder` — one adapter for the processor seam. Per LANGUAGE.md: "One adapter = hypothetical seam." Adding `run()` doesn't change the adapter count; it deepens the module.

---

## Design Options Considered

### Option A: Two `run()` Methods (on `(Comparison, Present)` + `(Parsed, Missing)`)

```
Builder still branches on view:
  match view {
    Some(v) → processor.transition(Comparison, Present::new(v)).run(source, repo)
    None    → processor.transition(Parsed, Missing).run(source, repo)
  }
```

Pros:
- Stage-specific logic stays on the right impl block
- No `Option` parameter needed — the branching is explicit at the call site

Cons:
- Builder still imports 13 of 15 processor types (`Comparison`, `Present`, `Parsed`, `Missing`, `Transition`)
- The branching is still in the builder — the module doesn't get deeper from the builder's perspective
- Two separate `run()` methods to test, document, and maintain

### Option B: Single `run()` on `(Init, Unknown)` taking `Option<&RawPropertyBankView>`

```
Builder:
  let processor = PropertyBankProcessor::from_discovery(file, root)?;
  let (bank, delta) = processor.run(bank_discovery.view(), &self.source, &self.repository)?;

Processor internally:
  match view {
    Some(v) → self.transition(Comparison, Present::new(v.clone())).run_present(source, repo)
    None    → self.transition(Parsed, Missing).run_missing(source, repo)
  }
```

Pros:
- Builder imports collapse: 15 types → **1** (`PropertyBankProcessor`)
- The branching is internal to the processor — module gets deeper
- `RawPropertyBankView` is already imported in the processor (line 122), so no new coupling
- The `Option<&RawPropertyBankView>` parameter honestly conveys the branching nature at the entry point

Cons:
- `Option<&RawPropertyBankView>` is a runtime choice when the typestate was designed for compile-time branching
  - Mitigation: this is the same runtime choice the builder was already making — just relocated closer to the types
- `run_present()` and `run_missing()` need to be private helpers on their respective impl blocks
  - Per DEEPENING.md: "A deep module can have internal seams" — this is fine

### Option C: Single `run()` on `(Init, Unknown)` taking `&PropertyBankDiscovery`

```
Builder:
  processor.run(&bank_discovery, &self.source, &self.repository)
```

Cons:
- Couples the processor to `PropertyBankDiscovery` from the discovery module
- The processor is currently a pure typestate engine that knows nothing about discovery types
- Creates a cross-module dependency for minimal benefit over Option B
- **Rejected**

---

## Rust Best Practices Guidance

### Chapter 7 — Type State Pattern

The processor uses the typestate pattern extensively and correctly. Key considerations:

**§7.5 — When to use**: "Use when it saves bugs, increases safety or simplifies logic."
- Adding `run()` **simplifies** the builder interface (15 types → 1)
- Adding `run()` does NOT reduce safety — the internal helpers still use typestate correctly
- Adding `run()` does NOT save new bugs — the existing handler chain already works

**§7.3 — Entry point pattern**: The `FileNotOpened → FileOpened` example shows a single `open()` entry point that returns the correct state. `run()` on `Init, Unknown` mirrors this — it's the single entry point that returns a terminal result, not a new stage.

**§7.5 — Avoid when**: "Writing trivial states like enums" / "Don't need type-safety" / "When it leads to overcomplicated generics."
- `run()` does none of these — it's a natural extension of the existing typestate, not a trivialization

### Chapter 1 — Interface Design

"Prefer `&T` over `.clone()` unless ownership transfer is required."
- `run(view: Option<&RawPropertyBankView>, source: &FileReader, repository: &R)` follows this — the method borrows all external resources
- The internal clone for `Present::new(v.clone())` is the only clone, and it's necessary because `Present` owns its data

### Chapter 3 — Performance

- The `run()` method reorganizes existing call chains; no new allocations or clones on the hot path
- The `source` parameter is used conditionally (only on mismatch) — matching existing `check_timestamps` behavior

### Chapter 5 — Testing

"One assertion per test when possible."
- Each pipeline path gets its own test (Missing, Present→Match, Present→Mismatch→ContentMatch, etc.)
- Each test asserts on the shape of the output (bank presence, delta presence)

---

## TDD Plan Detail

The TDD skill emphasizes **vertical slices** (tracer bullets) — not horizontal layering. Each cycle targets ONE pipeline path:

```
WRONG (horizontal):
  RED:   test_missing, test_fresh, test_content_match, test_content_mismatch
  GREEN: impl all paths

RIGHT (vertical):
  RED→GREEN: test_missing → impl None branch
  RED→GREEN: test_fresh → impl Present→Match→Fetch
  RED→GREEN: test_content_match → impl Present→Mismatch→ContentMatch→Sync→Fetch
  RED→GREEN: test_content_mismatch_empty → impl Present→Mismatch→ContentMismatch→Parse→Analyze→Empty
  RED→GREEN: test_content_mismatch_delta → impl ...Delta...
  RED→GREEN: test_content_mismatch_corrupt → impl ...Corrupt...
```

### Per-cycle checklist (from TDD skill):

```
[ ] Test describes behavior, not implementation
[ ] Test uses public interface only
[ ] Test would survive internal refactor
[ ] Code is minimal for this test
[ ] No speculative features added
```

### Test Design (per path)

Each test:
1. Creates a fixture (file, source, repository, optionally pre-seeded view)
2. Calls `PropertyBankProcessor::from_discovery(file, root)`
3. Calls `processor.run(view, &source, &repository)`
4. Asserts on the shape of the output

The `view` parameter controls which path is exercised:
- `None` → Missing path (parse from scratch)
- `Some(view)` where timestamps match → Fresh path (fetch cached)
- `Some(view)` where timestamps mismatch, content matches → Refresh path
- `Some(view)` where content mismatches, analysis empty → Refresh path (content)
- `Some(view)` where content mismatches, analysis has delta → Update path
- `Some(view)` where content mismatches, analysis corrupt → Create path

---

## Future Candidates (Deferred)

These were identified in earlier analysis but are NOT part of this phase:

1. **Unify Completed into-bank impls** — `FreshReady`, `NewReady`, `StaleReady` have near-identical `into_bank()` methods. Not actionable without a trait or macro — Rust doesn't support field access on generic `S`. Deferred until a stronger pattern emerges.

2. **Duplicate error construction in builder** — `SchemaLoaderError::Ingestion(SchemaIngestionError::File(...))` appears in both `load_all` and `load_property_bank`. Will become a single remaining instance after this refactor (the `load_all` copy). Worth a private helper, but only 1 duplication remaining — borderline.

3. **Test struct-literal construction** — Processor tests bypass the `transition()` constructor by using struct literal syntax. This is a testing gap but not a design concern for this phase. Noted for future cleanup.

---

## Resources

- Relevant files:
  - `lithos-core/src/schema/property_bank_processor.rs` — target module
  - `lithos-core/src/schema/builder.rs` — module to slim down
  - `lithos-core/src/schema/discovery.rs` — `PropertyBankDiscovery` (provider of `view()`)
  - `lithos-core/src/schema/error.rs` — `SchemaLoaderError` variants
- Previous analysis:
  - `.scratch/property-bank-processor-deepening/05-impl-tightening/findings.md`
  - `.scratch/property-bank-processor-deepening/05-impl-tightening/task_plan.md`
  - `.scratch/property-bank-processor-deepening/PRD.md`
- Skills:
  - `.claude/skills/rust-best-practices/references/chapter_07.md` — Type State Pattern
  - `.agents/skills/improve-codebase-architecture/DEEPENING.md`
  - `.agents/skills/improve-codebase-architecture/INTERFACE-DESIGN.md`
  - `.agents/skills/improve-codebase-architecture/LANGUAGE.md`
  - `.agents/skills/tdd/deep-modules.md`
  - `.agents/skills/tdd/interface-design.md`
