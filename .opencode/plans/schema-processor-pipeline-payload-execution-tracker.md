# Execution Tracker: Schema Processor `PipelinePayload` Migration

## Purpose

Track execution of `.opencode/plans/schema-processor-pipeline-payload-plan.md` with compile-safe checkpoints, verification gates, and commit slices.

## Current Baseline

- [x] `ProcessorNode` contains `status` and `relation`
- [x] `Graph::map_nodes(self, ...)` exists
- [x] `ProcessingGraph::map_payload(self, ...)` exists
- [x] `NewBatch::into_sorted_iter()` exists
- [x] Relation sidecar removed from stage structs
- [x] Unified `PipelinePayload` implemented
- [x] Stage graph type unification implemented

## Milestones

### M0 - Planning and checkpoint commit (this commit)

- [x] Create comprehensive migration blueprint
- [x] Create execution tracker
- [x] Commit all current files before implementation

### M1 - Introduce `PipelinePayload` primitives

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] Add `PipelinePayload` enum
- [x] Add `variant_name()` helper
- [x] Add stage invariant error helper
- [x] Add helper accessors for common mutation paths (`Analysis`)

Gates:

- [x] `cargo fmt`
- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M2 - Unify stage graph type signatures

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] Update `Present.graph` type
- [x] Update `Compared.graph` type
- [x] Update `Parsed.graph` type
- [x] Update `Graphed.graph` type
- [x] Update `Analyzed.graph` type
- [x] Update `Constructed.graph` type
- [x] Update `NewBuild.graph` type

Gates:

- [x] `cargo check -p lithos-core`

### M3 - Discovery path migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] `build_present_graph`: emit `PipelinePayload::Present` / `PipelinePayload::Deleted`
- [x] Keep `NodeStatus` mapping intact
- [x] Keep default relation assignment intact
- [x] Ensure discovery branches return unified graph type

Gates:

- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M4 - Compare stage conversion (no graph rebuild)

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] Replace `SchemaGraphBuilder` path in `compare()` with `map_payload`
- [x] Transform `Present(Found)` -> `Compared`
- [x] Pass through deleted/tombstone nodes intentionally
- [x] Preserve `fresh`, `stale_timestamps`, `stale_refs`, `stale` vectors
- [x] Keep deterministic iteration semantics

Gates:

- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M5 - Parse stage migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] `Compared` parse path: `PipelinePayload::Compared -> PipelinePayload::FileParsed`
- [x] All-missing path: emit `PipelinePayload::NewParsed`
- [x] Preserve parse error behavior and metadata assignment

Gates:

- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M6 - Build graph migration (structural phase)

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] Read from `PipelinePayload::FileParsed` and `PipelinePayload::NewParsed`
- [x] Preserve `build_resolution_index`
- [x] Preserve `collect_old_parents`
- [x] Preserve `ExtendsChangeKind` logic
- [x] Emit `PipelinePayload::Inheritance`

Gates:

- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M7 - Analyze + refresh migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] `analyze_properties`: `Inheritance -> Analysis`
- [x] Preserve `refresh_ids` and `rebuild_ids` behavior
- [x] `refresh_metadata`: mutate `Analysis` payload in place only
- [x] Preserve view persistence semantics

Gates:

- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M8 - Construction + completion migration

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] `construct_schemas`: consume analysis variant from unified payload
- [x] `construct_new_schemas`: consume new-parsed variant from unified payload
- [x] `complete`: keep save + delete + structure persistence behavior unchanged
- [x] Ensure `InheritanceGraph<()>` persistence path is unchanged

Gates:

- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M9 - Tests and hardening

Files:

- `lithos-core/src/schema/schema_processor.rs`

Tasks:

- [x] Add unit tests for payload variant transitions
- [x] Add tests for stage invariant errors
- [x] Add tests for deleted/tombstone pass-through rules
- [x] Remove transition adapters and dead code

Gates:

- [x] `cargo fmt`
- [x] `cargo check -p lithos-core`
- [x] `cargo test -p lithos-core schema_processor --lib`

### M10 - Final verification

Tasks:

- [x] `mise run verify`
- [x] `mise run test:bench:core`
- [x] Capture benchmark notes/regression summary

Notes:

- Latest `mise run test:bench:core` completed successfully; Criterion reported mixed results with several `db_storage`/`db_key_handling` regressions and some improvements. These changes are test-only in `schema_processor.rs`, so no production-path perf changes are expected from this patch set.

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

- [x] `compare()` no longer rebuilds edges
- [x] `relation` semantics unchanged in incremental construction
- [x] refresh/rebuild categorization unchanged
- [x] delete lifecycle still runs in `complete()`
- [x] structure-only persistence graph remains `InheritanceGraph<()>`
