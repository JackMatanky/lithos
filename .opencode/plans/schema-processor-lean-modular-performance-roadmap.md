# Schema Processor Lean/Modular/Performance Roadmap

## 1) Purpose

This document captures a full, critical issue inventory and implementation roadmap
for improving:

- `lithos-core/src/schema/schema_processor.rs`
- `lithos-core/src/schema/builder.rs`
- related support APIs in `raw/property.rs`, `views/metadata.rs`, and
  `inheritance.rs`

The goal is to make the schema pipeline leaner, more modular, and more
performant while preserving behavior and architectural constraints.

This roadmap is intentionally structured so work can be done in different
orders. It also calls out a recommended starting sequence based on risk,
impact, and dependencies.

---

## 2) Constraints and Guardrails

1. Preserve context boundaries (no cross-context business imports).
2. Keep graph infrastructure generic (`graph/*` remains schema-agnostic).
3. Preserve persistence contract on `InheritanceGraph<()>`.
4. Keep stage invariants explicit and fail-fast on invalid variant transitions.
5. Keep deterministic behavior where already relied upon (sorted IDs/topo order).
6. Avoid decomposition that hides flow intent (especially in `Builder::load_all`).

---

## 3) Executive Summary of Current Problems

### 3.1 High-impact problems

1. **Delta logic fragmentation**
   - `schema_processor` and `property_bank_processor` implement similar
     property-delta computations separately.
2. **Fresh-path over-processing**
   - Fresh and stale-timestamp nodes are still touched in analysis/refresh paths
     more than necessary.
3. **Clone-heavy hot paths**
   - Large payload cloning occurs in several loops/stages.
4. **RawPropertyMap ergonomics gap**
   - `ref_entries()` exists, but no symmetric `inline_entries()` or one-pass
     partitioning API.

### 3.2 Medium-impact problems

5. **Membership checks done with repeated `Vec::contains`**
   - O(n) membership checks are repeated in hot loops.
6. **Duplicate graph persistence conversion logic**
   - Unit graph persistence logic appears in multiple places.
7. **State model duplication (`NodeStatus` + stage ID vectors)**
   - Two concurrent representations of node classification increase drift risk.
8. **Overgrown stage methods**
   - `compare`, `analyze_properties`, and `construct_schemas` have grown large.

### 3.3 Lower-impact / architecture hygiene

9. **Builder flow readability pressure**
   - `Builder::load_all` is coherent but monolithic.
10. **Inconsistent utility reuse**
    - Existing graph conversion and metadata helpers are underused in places.

---

## 4) Recommended Starting Sequence (Requested)

The recommended first wave is:

1. `delta.rs` module (centralized property/excludes delta computation)
2. Fresh and stale-timestamp short-circuiting
3. Clone minimization in hot paths
4. RawPropertyMap ergonomics (`inline_entries` + optional partition API)

Rationale:

- This sequence removes duplicated logic first, then cuts unnecessary work,
  then lowers memory churn, then improves API ergonomics that support the
  previous steps.
- It minimizes risk and avoids premature architectural reshaping.

---

## 5) Detailed Issue Inventory and Recommendations

## I-01: Centralize delta logic in `delta.rs`

### Problem

`schema_processor` currently computes deltas via local helpers
(`diff_properties`, `diff_excludes`) while `property_bank_processor` has its own
delta computation (`compute_delta_from_raw_view`).

This duplicates logic and increases drift risk.

### Evidence

- `lithos-core/src/schema/schema_processor.rs`:
  - `diff_excludes(...)`
  - `diff_properties(...)`
- `lithos-core/src/schema/property_bank_processor.rs`:
  - `compute_delta_from_raw_view(...)`

### Recommendation

Create `lithos-core/src/schema/delta.rs` with shared, schema-context utilities.

Suggested contents:

1. `ExcludesDelta`
2. `PropertyDelta` abstraction (or schema/bank-specific wrappers over shared core)
3. canonical hash/diff helpers that both processors call

### Design notes

- Keep output types domain-appropriate where needed (`SchemaPropertyDelta` vs
  `PropertyBankDelta`), but share computation primitives.
- Avoid `unwrap_or_default()` in hash paths; errors should be explicit or routed
  through one controlled fallback policy.
- Prefer the existing canonical hash flow in `views::metadata` as the
  consistency baseline.

### Deliverables

- New module + tests
- Both processors migrated to shared diff core
- Old duplicated helpers removed or reduced to thin wrappers

### Priority

P0

---

## I-02: Fresh and stale-timestamp short-circuiting

### Problem

Nodes classified as fresh (or timestamp-only stale) can still be touched by
analysis/refresh logic more than necessary.

### Target behavior

1. **Fresh + not bank-affected**: no property-analysis work.
2. **Stale-timestamp + not bank-affected**: metadata sync only, no parse/rebuild.
3. **Bank-affected cases**: only affected properties/paths should be processed.

### Recommendation

Refactor stage boundaries so unaffected fresh nodes bypass expensive analysis.

### Design notes

- Keep pipeline invariants explicit.
- Do not hide behavior behind implicit fall-through branches.
- Ensure this does not break refresh metadata commitments.

### Deliverables

- Explicit short-circuit paths
- Tests proving unaffected fresh nodes are not re-read/re-parsed
- Bench comparison on mixed fresh/stale corpus

### Priority

P0

---

## I-03: Clone minimization

### Problem

Large payload clones occur in stage transitions and construction paths,
including `RawSchema`-heavy variants.

### Recommendation

Reduce cloning by:

1. Matching by reference where ownership is not required.
2. Moving only required fields out of payloads.
3. Consolidating helper APIs to return borrowed views where possible.

### Design notes

- This should be guided by allocation-heavy hotspots first.
- Preserve readability; avoid borrow complexity that obscures intent.

### Deliverables

- Reduced clone count in critical loops
- No behavior changes
- Optional microbench snapshots before/after

### Priority

P0

---

## I-04: RawPropertyMap ergonomics

### Problem

`RawPropertyMap<RawProperty>` offers `ref_entries()` but no equivalent
`inline_entries()` or one-pass partition helper.

This leads to ad-hoc inline extraction in `schema_processor`
(`collect_inline_entries`).

### Recommendation

Add API in `raw/property.rs`:

1. `inline_entries()`
2. optional one-pass split method, e.g. `split_entries()` returning
   `(inline_map, ref_map)`

### Design notes

- Keep `ref_entries()` for call sites that only need refs.
- Use one-pass split only where both classes are needed.
- Remove local extraction helpers once call sites migrate.

### Deliverables

- New RawPropertyMap methods + tests
- `schema_processor` migrated off `collect_inline_entries`

### Priority

P0

---

## I-05: Batch membership performance (`Vec` + `HashSet` pair)

### Problem

Stage IDs are often stored as `Vec<SchemaId>` and repeatedly queried with
`contains`, creating avoidable O(n) checks in loops.

### Recommendation

For frequently queried batches, store:

```text
{ ordered: Vec<SchemaId>, set: HashSet<SchemaId> }
```

- `ordered` for deterministic iteration
- `set` for O(1)-ish membership checks

### Important clarification

This does **not** replace topological order from the graph; it complements it.

- Topo order solves execution ordering.
- Batch index/set solves fast classification membership.

### Priority

P1

---

## I-06: Graph persistence conversion dedup + `TryFrom` usage

### Problem

There are duplicate code blocks that rebuild a unit graph for persistence.

### Recommendation

Create one shared helper that persists `InheritanceGraph<()>` from the
processing graph.

Potential approach:

1. transform payload graph to unit payload graph
2. call `TryFrom<ProcessingGraph<T>> for InheritanceGraph<T>`
3. persist once via helper

### Clarification on "why not using TryFrom now"

`TryFrom` is available, but existing code likely retained ownership of the
processing graph for return paths. A helper can resolve ownership decisions
cleanly without duplication.

### Priority

P1

---

## I-07: State model duplication (`NodeStatus` vs stage batches)

### Problem

Current flow carries both per-node `NodeStatus` and stage-level ID batches.
This duplicates classification state and can drift.

### Recommendation

Move toward payload-encoded state + stage indexes as primary classification.

- Keep `NodeStatus` only if it carries uniquely required semantics not otherwise
  representable.
- Otherwise remove it gradually after payload/status mapping is explicit.

### Risk

This is structurally impactful and should follow P0/P1 cleanup for safer
migration.

### Priority

P2

---

## I-08: Overgrown stage methods (decomposition)

### Problem

Some methods have become too large and hard to reason about.

### Recommendation

Decompose by branch semantics, not by micro-helper noise.

Good decomposition rule:

- keep top-level stage function as linear orchestration
- extract only meaningful branch handlers (fresh/stale/bank-affected/rebuild)
- keep data flow explicit in function signatures

### Priority

P2

---

## I-09: Builder orchestration readability without masked indirection

### Problem

`Builder::load_all` is coherent but dense.

### Recommendation

Refactor only into phase-level helpers with strong intent names:

1. discovery and preconditions
2. property bank load
3. run new-flow pipeline
4. run incremental-flow pipeline
5. completion mapping

Avoid fragmented tiny helpers that obscure control flow.

### Priority

P2

---

## 6) Dependency and Reordering Matrix

| Issue | Can Start Immediately | Depends On | Notes |
| --- | --- | --- | --- |
| I-01 delta.rs | Yes | None | Best first step |
| I-02 fresh short-circuit | Yes | I-01 (recommended) | Can proceed independently if needed |
| I-03 clone minimization | Yes | None | Safe in small PR slices |
| I-04 RawPropertyMap ergonomics | Yes | None | Enables cleaner I-01/I-02 call sites |
| I-05 batch membership | Yes | None | Localized change, low risk |
| I-06 graph persistence helper | Yes | None | Mostly dedup/hygiene |
| I-07 state model unification | Later | I-01..I-05 | Structural, higher risk |
| I-08 method decomposition | Later | Optional prior cleanup | Do not over-fragment |
| I-09 builder readability | Later | Optional prior cleanup | Keep top-level linear |

---

## 7) Suggested Implementation Waves

## Wave A (Start Here)

1. I-01 `delta.rs`
2. I-04 RawPropertyMap ergonomics
3. I-02 fresh/stale timestamp short-circuit
4. I-03 clone minimization in touched paths

## Wave B

5. I-05 batch membership structure (`Vec` + `HashSet` pair)
6. I-06 graph persistence helper + conversion cleanup

## Wave C

7. I-08 targeted decomposition of overgrown methods
8. I-09 builder phase-level readability refactor

## Wave D (Architecture-level cleanup)

9. I-07 state model unification and potential `NodeStatus` retirement

---

## 8) Verification Requirements per Wave

For each wave:

1. unit tests for changed helpers and branch behavior
2. stage invariant tests (unexpected variant -> fail-fast)
3. regression tests for deleted/fresh/stale paths
4. performance checks on representative mixed corpus
5. full quality gate (`mise run verify`)

Additional checks after Wave A/B:

- no new unnecessary clone regressions in hot loops
- unaffected fresh nodes are not re-parsed/rebuilt

---

## 9) Open Design Questions (to finalize before implementation)

1. Should `PropertyDelta` be generic and shared directly across schema/property
   bank processors, or should there be one internal shared core with
   processor-specific wrapper types?
2. Should one-pass `split_entries()` be canonical, with `inline_entries()` and
   `ref_entries()` implemented as thin projections?
3. For stage batch indexing, do we introduce a reusable type (e.g. `IdBatch`) or
   keep per-stage explicit `{ ordered, set }` fields for transparency?
4. In state-model unification, which semantics currently carried by
   `NodeStatus` must be represented explicitly in payload variants?

---

## 10) Definition of Done for This Roadmap

This roadmap is complete when:

1. all issues are either implemented or explicitly deferred with rationale,
2. duplicated delta logic is centralized,
3. fresh-path over-processing is eliminated,
4. clone-heavy hotspots are reduced materially,
5. RawPropertyMap supports symmetric property extraction ergonomics,
6. method decomposition improves readability without masking pipeline intent.
