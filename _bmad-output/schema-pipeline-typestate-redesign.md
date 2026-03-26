# Schema Pipeline Typestate Redesign (Draft)

**Date**: 2026-03-26
**Status**: Draft for review
**Purpose**: Define the schema pipeline typestate model using learnings from `property_bank_processor.rs`, with an explicit Graphed stage and incremental graph updates.

---

## Goals

1. Make the schema pipeline a first-class typestate state machine (stage + status dimensions).
2. Preserve incremental processing: avoid full recomputation when staleness is narrow.
3. Keep orchestration explicit through branching enums (like PropertyBank).
4. Add a dedicated Graphed stage between Comparison and Analysis with incremental graph maintenance.
5. Preserve the stale timestamps vs stale content distinction for schema files.

---

## Stage Taxonomy (Order Matters)

1. **Discovery**: find schema files, load cached views, track deletions, initialize indexes.
2. **Comparison**: compare timestamps, then content hashes; retain content when needed.
3. **Inheritance Analysis**: compute `ExtendsDelta` and `ExcludesDelta` to plan graph work.
4. **Graphed**: build or update `InheritanceGraph` with fail-fast structural validation.
5. **Analysis**: compute schema deltas and bank reference deltas.
6. **Refresh**: update view metadata only (timestamps/content hash updates).
7. **Construction**: expand refs + merge properties level-by-level; fetch cached results where fresh.
8. **Completed**: persist updated views and deliver resolved schemas.

---

## Status Axis (Knowledge Carriers)

Use status types to carry invariants and data, mirroring the PropertyBank model.

### Discovery
- `Unknown`
- `Missing` (no cached view)
- `Present` (has `RawSchemaView`)

### Comparison
- `Fresh` (timestamps + content hash match)
- `Suspect` (timestamps mismatch; content retained for hash)
- `StaleTimestamps` (content hash matches; timestamps differ)
- `StaleContent` (content hash differs)

### Inheritance Analysis
- `Unchanged` (no extends changes; no new schemas)
- `Changed` (extends changes or new schemas)
- `ExtendsDelta` (old/new extends per schema)

### Graphed
- `GraphFresh` (reuse graph from DB)
- `GraphPatched` (graph updated incrementally)

### Analysis
- `Unchanged` (no schema delta and no PB ref delta)
- `Changed` (carries schema delta + bank reference delta + affected ids)
- `ExcludesDelta` (old/new excludes per schema)

### Refresh
- `StaleTimestamps` (timestamps updated; content hash unchanged)
- `StaleContent` (timestamps + content hash updated)

### Construction
- `Fresh` (schema result fetched from DB)
- `Full` (expanded/merged in this run)
- `Merge` (merge-only path using cached expanded properties)

### Completed
- `Ready` (final resolved schemas)

---

## Branching Enums (Orchestration Surface)

Expose branching enums at each decision point, as done in `property_bank_processor.rs`.
This keeps transitions explicit and prevents implicit control flow.

Proposed enums (names placeholder):

- `DiscoveryBranch::Missing | Present`
- `ComparisonBranch::Fresh | Suspect`
- `ContentBranch::StaleTimestamps | StaleContent`
- `InheritanceBranch::Unchanged | Changed`
- `GraphBranch::GraphFresh | GraphPatched`
- `AnalysisBranch::Unchanged | Changed`
- `ConstructionBranch::Fresh | Full | Merge`

All branch enums should be `#[must_use = "branch outcomes must be handled"]`.

---

## Inheritance Analysis Stage (Global)

The inheritance analysis stage computes deltas needed to plan graph work and later merges.

### Outputs
- `ExtendsDelta` (drives graph reuse/patch/insert)
- `InheritanceBranch::Unchanged | Changed`

---

## Graphed Stage (Global, Incremental)

The graph is built once per run, but the strategy depends on `extends` freshness per schema.

### Inputs
- `ExtendsDelta` (defines graph reuse/patch/insert strategy)
- detection of new schemas (no cached view)

### Cases

1. **Graph reuse (fastest)**
   - Condition: `InheritanceBranch::Unchanged`.
   - Action: load `InheritanceGraph` from DB.

2. **Graph insertion (new schema)**
   - Condition: at least one new schema.
   - Action: insert new node(s), update parent/child edges, update order and depths.
   - Must revalidate only the affected subtree where a new schema is added.

3. **Graph patch (extends changed)**
   - Condition: `InheritanceBranch::Changed` and no new schemas.
   - Action: rewire edges for the schema and update only affected branches:
     - detach from old parent branch
     - attach to new parent branch
     - recompute depths and order for affected subtrees
   - Must recheck for cycles only in the modified region (fail-fast).

### Output
- `InheritanceGraph` plus metadata describing which subtrees are affected.

---

## Stale Timestamps vs Stale Content (Schema)

Follow the PropertyBank model:

1. **Timestamp check**
   - If match → `Fresh`
   - If mismatch → retain content and check hash

2. **Content hash check**
   - If match → `StaleTimestamps`
   - If mismatch → `StaleContent`

Only `StaleContent` must enter Analysis and Construction. `StaleTimestamps` can go to Refresh.

---

## Suggested Orchestration Pattern

### Builder-driven branching (default)
Use explicit branching enums at each stage, mirroring PropertyBank. This keeps the control
flow readable and testable at each decision point.

### For level-by-level merge

**Option A (explicit branching per level)**
- For each level, branch on freshness and PB delta.
- Pros: explicit, testable, deterministic.
- Cons: many branch points in orchestration.

**Option B (single construction pass with internal branching)**
- `Construction::merge_levels()` internally handles per-level branches.
- Pros: reduces nested branching in Builder.
- Cons: hides decisions and reduces test granularity.

**Recommendation**: Start with Option A (explicit branching enums).
It matches the typestate model and preserves clarity while the pipeline is still evolving.
Once stable, consider a thin helper to reduce Builder nesting without hiding branches.

---

## Incremental Optimization: Merge-Only When Refs Unaffected

If a schema's properties are unchanged **and** its cached `bank_references` do **not**
intersect the `PropertyBank` `PropertyDelta`, then ref expansion is unnecessary.
In that case, the pipeline can:

1. Skip ref expansion entirely.
2. Reuse cached `expanded_properties` from the view.
3. Proceed directly to merge with the parent (merge-only path).

This optimization applies even when the PropertyBank is stale, as long as the schema's
own properties are unchanged and none of its referenced bank properties changed.

---

## Implications for Schema Pipeline Design

1. Replace the existing linear "9 state" model with stage + status axes.
2. Treat Graphed as a distinct stage with its own branching logic and data.
3. Keep branching enums for schema decisions, including per-level merge decisions.
4. Ensure all stage transitions consume `self` and return a new typestate.
5. Add `#[must_use]` on branch enums to force handling.

---

## Open Questions

1. Exact data carried in Graphed status (affected subtree list, updated order, etc.).
2. How to represent graph patch operations in type-safe transitions.
3. Whether to store graph deltas in DB for incremental rebuilds.

---

## Next Steps

1. Use this document to update `_bmad-output/schema-pipeline-review.md`.
2. Update `_bmad-output/IMPLEMENTATION_PLAN.md` to include Graphed stage and branching enums.
3. Implement schema typestate design once the doc is approved.
