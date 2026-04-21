# Execution Tracker: Schema Processor `PipelinePayload` Migration

## Purpose

Track execution of `.opencode/plans/schema-processor-pipeline-payload-plan.md` with compile-safe checkpoints, verification gates, and commit slices.

## Current Baseline

- [x] `ProcessorNode` contains `status` and `relation`
- [x] `Graph::map_nodes(self, ...)` exists
- [x] `ProcessingGraph::map_payload(self, ...)` exists
- [x] `NewBatch::into_sorted_iter()` exists
- [x] Relation sidecar removed from stage structs
- [ ] Unified `PipelinePayload` not yet implemented
- [ ] Stage graph type unification not yet implemented

## Milestones

### M0 - Planning and checkpoint commit (this commit)

- [x] Create comprehensive migration blueprint
- [x] Create execution tracker
- [ ] Commit all current files before implementation

### M1 - Introduce `PipelinePayload` primitives

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] Add `PipelinePayload` enum
- [ ] Add `variant_name()` helper
- [ ] Add stage invariant error helper
- [ ] Add helper accessors for common mutation paths (`Analysis`)

Gates:

- [ ] `cargo fmt`
- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M2 - Unify stage graph type signatures

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] Update `Present.graph` type
- [ ] Update `Compared.graph` type
- [ ] Update `Parsed.graph` type
- [ ] Update `Graphed.graph` type
- [ ] Update `Analyzed.graph` type
- [ ] Update `Constructed.graph` type
- [ ] Update `NewBuild.graph` type

Gates:

- [ ] `cargo check -p lithos-core`

### M3 - Discovery path migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] `build_present_graph`: emit `PipelinePayload::Present` / `PipelinePayload::Deleted`
- [ ] Keep `NodeStatus` mapping intact
- [ ] Keep default relation assignment intact
- [ ] Ensure discovery branches return unified graph type

Gates:

- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M4 - Compare stage conversion (no graph rebuild)

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] Replace `SchemaGraphBuilder` path in `compare()` with `map_payload`
- [ ] Transform `Present(Found)` -> `Compared`
- [ ] Pass through deleted/tombstone nodes intentionally
- [ ] Preserve `fresh`, `stale_timestamps`, `stale_refs`, `stale` vectors
- [ ] Keep deterministic iteration semantics

Gates:

- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M5 - Parse stage migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] `Compared` parse path: `PipelinePayload::Compared -> PipelinePayload::FileParsed`
- [ ] All-missing path: emit `PipelinePayload::NewParsed`
- [ ] Preserve parse error behavior and metadata assignment

Gates:

- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M6 - Build graph migration (structural phase)

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] Read from `PipelinePayload::FileParsed` and `PipelinePayload::NewParsed`
- [ ] Preserve `build_resolution_index`
- [ ] Preserve `collect_old_parents`
- [ ] Preserve `ExtendsChangeKind` logic
- [ ] Emit `PipelinePayload::Inheritance`

Gates:

- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M7 - Analyze + refresh migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] `analyze_properties`: `Inheritance -> Analysis`
- [ ] Preserve `refresh_ids` and `rebuild_ids` behavior
- [ ] `refresh_metadata`: mutate `Analysis` payload in place only
- [ ] Preserve view persistence semantics

Gates:

- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M8 - Construction + completion migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] `construct_schemas`: consume analysis variant from unified payload
- [ ] `construct_new_schemas`: consume new-parsed variant from unified payload
- [ ] `complete`: keep save + delete + structure persistence behavior unchanged
- [ ] Ensure `InheritanceGraph<()>` persistence path is unchanged

Gates:

- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M9 - Tests and hardening

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [ ] Add unit tests for payload variant transitions
- [ ] Add tests for stage invariant errors
- [ ] Add tests for deleted/tombstone pass-through rules
- [ ] Remove transition adapters and dead code

Gates:

- [ ] `cargo fmt`
- [ ] `cargo check -p lithos-core`
- [ ] `cargo test -p lithos-core schema_processor --lib`

### M10 - Final verification

Tasks:

- [ ] `mise run verify`
- [ ] `mise run test:bench:core`
- [ ] Capture benchmark notes/regression summary

## Commit Slice Checklist

- [ ] C1: add `PipelinePayload` helpers + invariant error helpers
- [ ] C2: unify graph state signatures
- [ ] C3: discovery migration
- [ ] C4: compare migration
- [ ] C5: parse migration
- [ ] C6: build graph migration
- [ ] C7: analyze/refresh migration
- [ ] C8: construction/completion migration
- [ ] C9: tests + cleanup + final verification

## Regression Watchlist

- [ ] `compare()` no longer rebuilds edges
- [ ] `relation` semantics unchanged in incremental construction
- [ ] refresh/rebuild categorization unchanged
- [ ] delete lifecycle still runs in `complete()`
- [ ] structure-only persistence graph remains `InheritanceGraph<()>`
