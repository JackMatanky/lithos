# Schema Processor `PipelinePayload` Refactor Blueprint

## 1) Purpose and Scope

This document defines the full migration plan for refactoring
`lithos-core/src/schema/schema_processor.rs` to a unified payload model:

- **Current pattern**: stage-specific graph payload types
- **Target pattern**: one stable graph payload type (`PipelinePayload`) through the pipeline

Primary goal:

1. Support heterogeneous node states naturally (not all nodes share the same payload shape at a given stage).
2. Remove avoidable graph type churn and rebuild patterns.
3. Preserve all existing behavior, persistence contracts, and context boundaries.

This plan is implementation-oriented and includes code regions, invariants,
error semantics, test coverage, and step-by-step migration order.

---

## 2) Ground Truth: Current Architecture and Constraints

### 2.1 Current processor topology (from code)

`schema_processor.rs` currently models the pipeline as:

`Discovery -> Comparison -> FileParsed -> InheritanceGraphed -> PropertyAnalysis -> Refresh -> Construction -> Completed`

with stage-specific graph state structs:

- `Present.graph: ProcessingGraph<ProcessorNode<PresentPayload>>`
- `Compared.graph: ProcessingGraph<ProcessorNode<ComparedPayload>>`
- `Parsed.graph: ProcessingGraph<ProcessorNode<FileParsedBranch>>`
- `Graphed.graph: ProcessingGraph<ProcessorNode<InheritanceBranch>>`
- `Analyzed.graph: ProcessingGraph<ProcessorNode<AnalysisBranch>>`
- `Constructed.graph: ProcessingGraph<ProcessorNode<AnalysisBranch>>`
- `NewBuild.graph: ProcessingGraph<ProcessorNode<NewParsedPayload>>`

### 2.2 Current graph capabilities

Already available and should be leveraged:

- `Graph::map_nodes(self, ...) -> Result<Graph<Id, U>, E>` (consuming transform)
  - `lithos-core/src/graph/core.rs`
- `ProcessingGraph::map_payload(self, ...)` (consuming payload transform)
  - `lithos-core/src/schema/inheritance.rs`
- `ProcessingGraph::node_ids_sorted()` for deterministic traversal
- `NewBatch::into_sorted_iter()` for deterministic batch iteration

### 2.3 Current metadata model

`ProcessorNode<T>` currently contains:

- `status: NodeStatus`
- `relation: ExtendsChangeKind`
- `payload: T`

This is correct and should remain in `schema_processor.rs`.

### 2.4 Non-negotiable constraints (project + user direction)

1. Keep `NodeStatus`, `ExtendsChangeKind`, and `ProcessorNode` in
   `schema_processor.rs`.
2. Keep graph infrastructure generic (no schema-specific leakage into `graph/*`).
3. Keep persistence on `InheritanceGraph<()>` (structure-only persistence graph).
4. Keep explicit DAG validation boundaries (do not enforce DAG globally on processing graph).
5. No architectural drift across context boundaries.

---

## 3) Problem Statement in Concrete Terms

### 3.1 Type churn problem

The graph payload generic type changes at nearly every stage. This makes
transitions heavier than needed and forces conversion logic that treats each
stage as a new graph universe.

### 3.2 Heterogeneous node reality vs homogeneous type assumption

At runtime, nodes are naturally mixed:

- some `fresh`
- some `stale timestamps`
- some `stale content`
- some `deleted`
- some `newly parsed`

Stage-specific graph payload generics cannot model mixed node states cleanly.

### 3.3 Maintainability pressure points

1. Transition duplication (`match` trees repeated across stages).
2. Stage logic split between payload conversion and orchestration concerns.
3. Harder invariant reasoning because stage type and node status can drift.

### 3.4 Performance pressure points

`compare()` still reconstructs a graph via builder instead of pure payload
transition. Structural data is unnecessarily re-walked in some paths.

---

## 4) Target Architecture

## 4.1 Unified payload enum

Introduce in `schema_processor.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PipelinePayload {
    Present(PresentPayload),
    Compared(ComparedPayload),
    FileParsed(FileParsedBranch),
    Inheritance(InheritanceBranch),
    Analysis(AnalysisBranch),
    NewParsed(NewParsedPayload),
    Deleted(DeletedPayload),
}
```

Notes:

- `Deleted` explicit variant is preferred to avoid implicit tombstones hidden in unrelated payload types.
- Existing payload structs/enums are retained initially to minimize risk.

## 4.2 Stable graph type through pipeline

Every graph-bearing stage status uses:

`ProcessingGraph<ProcessorNode<PipelinePayload>>`

Only non-graph stage fields differ (`new_schemas`, id buckets, etc.).

## 4.3 Stage markers remain

Keep stage marker types (`Comparison`, `FileParsed`, etc.) and
`SchemaProcessor<Stage, Status>` generic shape intact for orchestration and
compile-time flow control.

## 4.4 Structural vs non-structural phases

- **Non-structural phases** (compare, parse, analysis, refresh metadata):
  transition payload variants only.
- **Structural phases** (build graph/new graph):
  allowed to rebuild graph structure via `SchemaGraphBuilder`.

---

## 5) Invariants and Error Semantics

## 5.1 Core invariants (before and after)

Must continue to hold:

1. `ProcessorNode.status` is coherent with logical state and branch behavior.
2. `ProcessorNode.relation` reflects extends-change semantics after graph construction.
3. `refresh_ids` and `rebuild_ids` classify analysis outcomes correctly.
4. Persisted topology graph only contains structure (`()` payload).
5. Deleted IDs are removed from repository in completion stage.

## 5.2 New variant invariants

Per-stage expected payloads:

- `compare`: expect `PipelinePayload::Present` (and pass-through `Deleted`)
- `parse`: expect `PipelinePayload::Compared`
- `build_graph`: expect `PipelinePayload::FileParsed` and `PipelinePayload::NewParsed`
- `analyze_properties`: expect `PipelinePayload::Inheritance`
- `refresh_metadata`: expect `PipelinePayload::Analysis`
- `construct_schemas`: consume `PipelinePayload::Analysis`
- `construct_new_schemas`: consume `PipelinePayload::NewParsed`

## 5.3 Invariant violation policy (required)

Do not silently swallow unexpected variants in critical paths.

Add local helper:

```rust
fn stage_variant_error(
    stage: &'static str,
    id: SchemaId,
    expected: &'static str,
    actual: &'static str,
) -> SchemaLoaderError
```

Use this for fail-fast behavior when a node should be transformable in a
stage but has an incompatible payload variant.

Exception:

- explicit `Deleted` pass-through can be intentionally skipped where designed.

---

## 6) Migration Map (Old -> New)

## 6.1 Graph type mapping

- `ProcessingGraph<ProcessorNode<PresentPayload>>`
  -> `ProcessingGraph<ProcessorNode<PipelinePayload>>`
- `ProcessingGraph<ProcessorNode<ComparedPayload>>`
  -> same unified type
- `ProcessingGraph<ProcessorNode<FileParsedBranch>>`
  -> same unified type
- `ProcessingGraph<ProcessorNode<InheritanceBranch>>`
  -> same unified type
- `ProcessingGraph<ProcessorNode<AnalysisBranch>>`
  -> same unified type
- `ProcessingGraph<ProcessorNode<NewParsedPayload>>`
  -> same unified type

## 6.2 Payload transition mapping

1. discovery:
   - found -> `PipelinePayload::Present(PresentPayload::Found)`
   - deleted -> `PipelinePayload::Deleted`
2. compare:
   - `Present(Found)` -> `Compared(...)`
3. parse:
   - `Compared(...)` -> `FileParsed(...)`
4. build graph:
   - `FileParsed(...)` + `NewParsed(...)` -> `Inheritance(...)`
5. analysis:
   - `Inheritance(...)` -> `Analysis(...)`
6. refresh:
   - mutate `Analysis` in place
7. construction:
   - consume `Analysis`
8. completion:
   - persist graph structure only

---

## 7) Detailed Execution Plan (Implementation-Level)

### Phase A - Preparation and Safety Harness

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. Add a short internal section comment: "PipelinePayload migration rules".
2. Add helper methods on `PipelinePayload`:
   - `variant_name(&self) -> &'static str`
   - mutable accessors for analysis branch.
3. Add stage-variant error helper.

Checkpoint:

- `cargo check -p lithos-core`

### Phase B - Type Unification in State Structs

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. Update graph type in state structs:
   - `Present`, `Compared`, `Parsed`, `Graphed`, `Analyzed`, `Constructed`, `NewBuild`.
2. Keep stage fields unchanged otherwise.

Checkpoint:

- fix compile fallout by temporary adapter matches.
- run `cargo check -p lithos-core`.

### Phase C - Discovery Path Migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. `build_present_graph` now emits `PipelinePayload::Present(...)` or `PipelinePayload::Deleted`.
2. Ensure `NodeStatus` setting remains equivalent.
3. Ensure `relation` defaults remain unchanged (`Unchanged`).

Checkpoint:

- `discover` code compiles both review and never-seen paths.

### Phase D - Compare Stage Conversion to Transform-Only

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. Remove `SchemaGraphBuilder` usage in `compare()`.
2. Use `graph.map_payload(...)`.
3. Match node payload:
   - `PipelinePayload::Present(PresentPayload::Found(found))` -> compute and emit `PipelinePayload::Compared(...)`
   - `PipelinePayload::Deleted(_)` -> pass through
4. Preserve id bucket updates exactly.

Critical checks:

1. No edge changes in compare stage.
2. No changes to `deleted_ids` handling.
3. Preserve deterministic behavior.

### Phase E - Parse Stage Migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. In `FileParsed, Compared::parse`:
   - transform `PipelinePayload::Compared` -> `PipelinePayload::FileParsed`.
2. In all-missing parse/new graph path:
   - convert `InitialParsed` nodes to `PipelinePayload::NewParsed`.
3. Keep parsing helper (`parse_new`) semantics unchanged.

### Phase F - Structural Graph Build Stage Migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. `build_graph` reads unified payload variants.
2. Keep helper functions:
   - `build_resolution_index`
   - `collect_old_parents`
3. Compute `ExtendsChangeKind` as currently implemented.
4. Emit `PipelinePayload::Inheritance(...)` for existing and new nodes.

Critical checks:

1. Parent resolution from index unchanged.
2. Relation semantics unchanged.

### Phase G - Analysis and Refresh Migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. `analyze_properties`:
   - transform `PipelinePayload::Inheritance(...)` -> `PipelinePayload::Analysis(...)`
   - keep refresh/rebuild bucket behavior.
2. `refresh_metadata`:
   - mutate analysis payload in place only.
3. maintain relation and status metadata coherence.

### Phase H - Construction and Completion Migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. `construct_schemas` reads analysis variant from unified payload.
2. `construct_new_schemas` reads new-parsed variant from unified payload.
3. `complete` remains persistence step:
   - save changed schemas
   - delete removed schemas
   - persist structure graph as `InheritanceGraph<()>`

Critical checks:

1. `delete_schema` loop preserved.
2. persistence graph still structure-only.

### Phase I - Cleanup and Hardening

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

1. Remove temporary adapter helpers.
2. Remove dead code created by migration.
3. Consolidate repetitive transition code into small helpers.
4. Re-run clippy-sensitive polish (explicit reasons where `#[expect]` is needed).

---

## 8) Function-by-Function Refactor Checklist

Each item is required unless marked optional.

### Discovery

- [ ] `SchemaProcessor<Discovery, NeverSeen>::discover`
- [ ] `SchemaProcessor<Discovery, Review>::discover`
- [ ] `build_present_graph`
- [ ] `classify_file_state` (only if payload assumptions change)

### Comparison

- [ ] `compare`
- [ ] `check_timestamps`
- [ ] `check_content`
- [ ] `status_for_payload` (adapt to `PipelinePayload::Compared` wrapping)

### Parsing

- [ ] `SchemaProcessor<FileParsed, AllMissing>::parse`
- [ ] `SchemaProcessor<FileParsed, Compared>::parse`
- [ ] `parse_new`

### Graph construction

- [ ] `build_graph`
- [ ] `build_resolution_index`
- [ ] `collect_old_parents`
- [ ] `build_new_graph`

### Analysis + refresh

- [ ] `analyze_properties`
- [ ] `bank_changed`
- [ ] `schema_stem`
- [ ] `build_version`
- [ ] `refresh_metadata`

### Construction + completion

- [ ] `construct_schemas`
- [ ] `construct_schema_incremental`
- [ ] `collect_parent_properties`
- [ ] `construct_new_schemas`
- [ ] `complete`

### Helper functions

- [ ] `collect_inline_entries`
- [ ] `diff_excludes`
- [ ] `diff_properties`

---

## 9) Data Ownership and Allocation Strategy

### 9.1 Ownership rules during transitions

Prefer consuming transforms:

- `graph.map_payload(self, mapper)` should move node payloads where possible.
- Avoid clone unless required for dual-use data in same branch.

### 9.2 Expected large payload areas

Potentially large members:

- `RawSchema`
- `RawSchemaView`
- `Box<str>` content buffers
- delta structures with hash maps

Action:

1. During migration, do not prematurely box more variants.
2. After correctness, profile enum size and hot path costs.
3. If needed, selectively box largest variants in `PipelinePayload`.

### 9.3 Determinism requirements

Must preserve deterministic iteration in:

- batch processing (`NewBatch::into_sorted_iter()`)
- node traversal where stage output ordering is compared or persisted.

---

## 10) Test Plan (Detailed)

### 10.1 New unit tests to add in `schema_processor.rs`

1. `pipeline_payload_variant_name_reports_expected`
2. `pipeline_payload_analysis_accessor_none_for_non_analysis`
3. `stage_variant_error_contains_stage_and_variant`
4. `compare_stage_transforms_present_found_to_compared`
5. `compare_stage_preserves_deleted_variant`
6. `parse_stage_transforms_compared_to_file_parsed`
7. `build_graph_stage_transforms_file_parsed_to_inheritance`
8. `analysis_stage_transforms_inheritance_to_analysis`

### 10.2 Existing tests to keep green

Current in-file tests around:

- `ExtendsChangeKind` behavior
- delta emptiness semantics

### 10.3 Integration confidence checks

Must pass:

1. `schema_loader::initial_loading::*`
2. `schema_loader::incremental_loading::*`
3. `schema_loader::inheritance::*`
4. `schema_loader::error_handling::*`

These are already exercised by `mise run verify`.

---

## 11) Verification and Performance Gates

Run after each major phase:

1. `cargo fmt`
2. `cargo check -p lithos-core`
3. `cargo test -p lithos-core schema_processor --lib`

Final required gates:

1. `mise run verify`
2. `mise run test:bench:core`

Metrics to watch during benchmark pass:

- no obvious regressions in schema-related and db/storage benches
- no significant increase in hot read/transform timings attributable to enum inflation

---

## 12) Risk Register and Mitigation

### R1: Variant mismatch bugs during migration

- Symptom: stage sees unexpected payload and silently skips work.
- Mitigation: stage-invariant fail-fast helper; tests for negative cases.

### R2: Memory footprint increase from enum aggregation

- Symptom: increased allocations/cache misses.
- Mitigation: post-migration profiling; selective boxing of heavy variants.

### R3: Behavior drift in relation/status semantics

- Symptom: incorrect rebuild/update decisions.
- Mitigation: preserve existing relation computation algorithm verbatim;
  add targeted assertions around `ExtendsChangeKind` behavior in stage transitions.

### R4: Incomplete migration leaves mixed old/new assumptions

- Symptom: compile churn, fragile adapters.
- Mitigation: strict phase order and compile checkpoint after each phase.

### R5: Regression in deletion lifecycle

- Symptom: orphaned schemas/views not removed.
- Mitigation: verify `complete()` delete loop remains and integration tests stay green.

---

## 13) Commit Strategy (Suggested)

1. `refactor(schema): add PipelinePayload and variant helpers`
2. `refactor(schema): unify processor state graph payload type`
3. `refactor(schema): migrate discovery and new-build payload wrapping`
4. `refactor(schema): convert compare stage to payload transform`
5. `refactor(schema): migrate parse and graph build to unified payload`
6. `refactor(schema): migrate analysis/refresh/construction/completion`
7. `refactor(schema): remove transitional adapters and harden invariants`
8. `test(schema): add pipeline payload transition coverage`

Each commit should compile and keep tests passing for changed scope.

---

## 14) Definition of Done

All must be true:

1. All graph-bearing stage states in `schema_processor.rs` use
   `ProcessingGraph<ProcessorNode<PipelinePayload>>`.
2. `compare()` no longer rebuilds graph structure.
3. Structural edits are constrained to graph-building phases.
4. `complete()` still saves schemas, deletes removed schemas, and persists
   structure-only graph.
5. Invariant errors exist for unexpected payload variants.
6. Unit + integration + doc tests pass via `mise run verify`.
7. Bench suite runs via `mise run test:bench:core`.
8. No movement of `NodeStatus`/`ProcessorNode`/`ExtendsChangeKind` out of
   `schema_processor.rs`.

---

## 15) Optional Follow-Up Work (Not in This Refactor)

1. Revisit `NodeStatus` redundancy once `PipelinePayload` stabilizes.
2. Reduce `Vec<SchemaId>` membership checks in hot paths (potential set/bitmap strategy).
3. Split `schema_processor.rs` into stage modules if file complexity remains high.
4. Add benchmark specifically for payload transition stages.

---

## 16) Quick Reference: Do/Do Not

Do:

- keep processing graph generic
- keep schema-specific payload logic in schema context
- fail fast on stage variant mismatches
- preserve deterministic traversal

Do not:

- add schema concepts to `graph/*`
- alter persistence format or topology storage contract
- silently skip unexpected variants unless explicitly designed tombstone pass-through
