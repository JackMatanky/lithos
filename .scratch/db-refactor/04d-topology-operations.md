---
title: 04d-topology-operations
category: enhancement
label: ready-for-agent
status: open
date_created: 2026-05-12
---

## Type

AFK

## Labels

- ready-for-agent

## What to build

Migrate Schema inheritance topology graph operations from v1 to v2.

The topological graph singleton tracks schema inheritance relationships in a DAG structure, enabling efficient inheritance resolution.

## Operations to Migrate

### Read Operations (→ `SchemaReadRepository`)
1. **`get_topological_graph()`** - Get the inheritance graph singleton

### Write Operations (→ `SchemaWriteRepository`)
2. **`save_topological_graph(graph: &InheritanceGraph<()>)`** - Save the inheritance graph singleton

## Tables Required

Add to `storage_v2/tables.rs`:
```rust
/// Topological inheritance graph singleton
/// (key: singleton string, value: serialized InheritanceGraph<()>)
pub const SCHEMA_TOPOLOGICAL_GRAPH: SingletonTable<&[u8]> =
    SingletonTable::new("schema_topological_graph_v2", "graph_singleton");
```

**Note**: If `SingletonTable` doesn't exist, use `PathTable` with a constant key like v1 does.

## TDD Implementation Plan

### Phase 1: Read Path
1. RED: Test `get_topological_graph()` returns None when not saved
2. GREEN: Implement in `read.rs`
3. RED: Test `get_topological_graph()` returns saved graph
4. GREEN: Implement (will pass after Phase 2)

### Phase 2: Write Path
1. RED: Test `save_topological_graph(graph)` persists graph
2. GREEN: Implement in `write.rs`
3. RED: Test overwrite - saving new graph replaces old one
4. GREEN: Verify (should pass with singleton semantics)

### Phase 3: Persistence
1. RED: Test graph persists across store reopens
2. GREEN: Verify passes
3. RED: Test graph structure integrity (nodes, edges preserved)
4. GREEN: Verify `InheritanceGraph` serialization/deserialization

## Acceptance Criteria

- [ ] `get_topological_graph()` added to `SchemaReadRepository`
- [ ] `save_topological_graph(graph)` added to `SchemaWriteRepository`
- [ ] `SCHEMA_TOPOLOGICAL_GRAPH` table added to `storage_v2/tables.rs`
- [ ] Implementation in `storage_v2/read.rs` and `storage_v2/write.rs`
- [ ] Unit tests verify:
  - None returned when not saved
  - Saved graph retrievable
  - Overwrite replaces previous graph
  - Graph structure preserved (nodes, edges)
  - Persistence across store reopens
- [ ] `InheritanceGraph<()>` is rkyv-serializable
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code formatted

## Blocked by

- `04a-property-bank-migration.md` (proves singleton pattern)

## Blocks

- `04e-remaining-schema-operations.md`

## Notes

- Singleton pattern similar to Property Bank (04a)
- `InheritanceGraph<()>` should already be rkyv-serializable from v1
- No multi-table atomicity concerns (single table)
- Graph is rebuilt/patched when inheritance relationships change (outside this issue's scope)
