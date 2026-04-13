//! Graph infrastructure performance benchmarks.
//!
//! # Summary
//!
//! Validates performance improvements in the new graph module
//! (`lithos_core::graph`) compared to the old schema-specific implementation
//! (`lithos_core::schema::graph`).
//!
//! # Motivation
//!
//! The schema graph redesign (Phase 1) introduced a new generic graph module to
//! eliminate ID duplication, simplify construction patterns, and improve query
//! performance. This benchmark suite validates the ≥10% performance improvement
//! target for construction and adjacency queries.
//!
//! # Scope
//!
//! **Included**:
//! - Graph construction (old `Graph::add_node`/`add_edge` vs new
//!   `GraphBuilder`)
//! - Adjacency queries (`parents_of`, `children_of`)
//! - Multiple graph sizes (10, 100, 1000 nodes)
//!
//! **Excluded**:
//! - Full pipeline performance (schema loading, validation)
//! - Memory allocation measurement (uses latency as proxy)
//! - Topological sorting (tested separately in unit tests)
//!
//! # Benchmark Style
//!
//! - **Micro-benchmarks**: Isolated operation measurement
//! - **Comparative**: Old vs new implementation for same workload
//! - **Single-threaded**: No concurrent access scenarios
//! - **Parameterized**: Multiple graph sizes to detect scaling behavior
//!
//! # Methodology
//!
//! - **Harness**: Criterion.rs (default configuration)
//! - **Black-boxing**: All IDs and results passed through `black_box()`
//! - **Input generation**: Deterministic node/edge creation
//! - **Graph structure**: Linear chain (1→2→3→...→N) for construction, balanced
//!   tree for queries
//!
//! # Input Model
//!
//! **Construction benchmarks**:
//! - **Sizes**: 10, 100, 1000 nodes
//! - **Structure**: Linear chain (each node has one parent except root)
//! - **Payload**: `Box<str>` with node index as content
//!
//! **Adjacency benchmarks**:
//! - **Size**: 1000 nodes
//! - **Structure**: Balanced binary tree (predictable parent/child counts)
//! - **Queries**: Full iteration over all nodes (measures average case)
//!
//! # Expected Characteristics
//!
//! **Target improvements (from schema-graph-redesign.md)**:
//! - **Construction**: New ≥10% faster than old
//! - **Adjacency queries**: O(1) lookups (old required `AdjacencyMap`
//!   construction)
//! - **Memory**: New uses ≤60% of old (ID deduplication)
//!
//! **Scaling expectations**:
//! - Construction: O(N) for both implementations
//! - Adjacency queries: O(1) for new, O(N) setup + O(1) query for old
//!
//! # Interpreting Results
//!
//! **Success criteria**:
//! - New construction ≥10% faster than old across all sizes
//! - Adjacency queries show minimal overhead (no map construction)
//! - Ratios stable across graph sizes (no unexpected scaling)
//!
//! **Warning signs**:
//! - New slower than old: Critical regression
//! - Improvement <10%: Target not met
//! - Non-linear scaling: Algorithm complexity issue
//!
//! # Maintenance Contract
//!
//! **Update when**:
//! - Graph module API changes (construction or query methods)
//! - Schema module graph implementation changes
//! - Performance regression reported in real-world usage
//!
//! **Adding benchmarks**:
//! - Always pair old and new implementations for comparison
//! - Use consistent graph structures across implementations
//! - Document expected improvement ratios
//!
//! # Known Limitations
//!
//! - Does not measure actual memory allocation (use dhat for that)
//! - Linear chain may not represent real schema inheritance patterns
//! - No concurrent access patterns (single-threaded only)
//! - Does not test DAG validation overhead (separate concern)

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::excessive_nesting,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use criterion::{
    BenchmarkId, Criterion, black_box, criterion_group, criterion_main,
};
use lithos_core::schema::aggregate::SchemaId;

// ----------------------------------------------------------- //
//                  Graph Construction                         //
// ----------------------------------------------------------- //

/// Benchmarks graph construction: old mutable API vs new builder pattern.
///
/// # Purpose
///
/// Validates that the new `GraphBuilder` pattern is ≥10% faster than the old
/// `Graph::add_node`/`add_edge` API for constructing linear chain graphs.
///
/// # What is Measured
///
/// - **Metric**: Time to construct graph with N nodes in linear chain
///   (1→2→3→...→N)
/// - **Variants**: Old (mutable Graph) vs new (`GraphBuilder`)
/// - **Sizes**: 10, 100, 1000 nodes
///
/// # Inputs
///
/// - **Nodes**: Sequential `SchemaIds` with `Box<str>` payload ("node-0000"
///   format)
/// - **Edges**: Parent→child edges forming linear chain
/// - **Deterministic**: Same structure for both implementations
///
/// # Expected Characteristics
///
/// - **New (builder)**: Should be ≥10% faster than old
/// - **Scaling**: Both O(N), but new should have lower constant factor
/// - **Bottleneck**: `HashMap` insertions and Vec pushes
///
/// # Interpreting Changes
///
/// - **New slower**: Critical regression in builder pattern
/// - **Improvement <10%**: Performance target not met
/// - **Non-linear scaling**: Unexpected algorithm complexity
fn bench_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    for size in [10, 100, 1000] {
        // Old implementation benchmark
        group.bench_with_input(
            BenchmarkId::new("old", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    let mut graph =
                        lithos_core::schema::graph::Graph::<Box<str>, ()>::new(
                        );
                    let mut ids = Vec::with_capacity(s);

                    for i in 0..s {
                        let id = SchemaId::new();
                        let payload: Box<str> = format!("node-{i:04}").into();
                        graph.add_node(black_box(id), payload);
                        ids.push(id);

                        if i > 0 {
                            graph.add_edge(ids[i - 1], id);
                        }
                    }

                    black_box(graph)
                });
            },
        );

        // New implementation benchmark
        group.bench_with_input(
            BenchmarkId::new("new", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    let mut builder = lithos_core::graph::GraphBuilder::new();
                    let mut ids = Vec::with_capacity(s);

                    for i in 0..s {
                        let id = SchemaId::new();
                        let payload: Box<str> = format!("node-{i:04}").into();
                        builder.add_node(black_box(id), payload);
                        ids.push(id);

                        if i > 0 {
                            // add_parent(child, parent) - child extends parent
                            builder.add_parent(id, ids[i - 1]);
                        }
                    }

                    let graph = builder.build();
                    black_box(graph)
                });
            },
        );
    }

    group.finish();
}

// ----------------------------------------------------------- //
//                  Adjacency Queries                          //
// ----------------------------------------------------------- //

/// Benchmarks adjacency queries: old `AdjacencyMap` vs new direct lookups.
///
/// # Purpose
///
/// Validates that the new graph provides O(1) adjacency lookups without
/// requiring separate `AdjacencyMap` construction.
///
/// # What is Measured
///
/// - **Metric**: Time to query parents and children for all nodes
/// - **Variants**: Old (requires `AdjacencyMap`) vs new (direct lookup)
/// - **Size**: 1000 nodes in balanced binary tree
///
/// # Inputs
///
/// - **Structure**: Balanced binary tree (each node has 0-2 children)
/// - **Queries**: Full iteration over all nodes (average case)
/// - **Deterministic**: Same tree structure for both implementations
///
/// # Expected Characteristics
///
/// - **New**: O(1) lookup per query
/// - **Old**: O(N) `AdjacencyMap` construction + O(1) lookup
/// - **Improvement**: Should be significantly faster (no map construction)
///
/// # Interpreting Changes
///
/// - **New slower**: Critical regression in lookup mechanism
/// - **Both slow**: General `HashMap` performance issue
#[expect(
    clippy::integer_division,
    clippy::integer_division_remainder_used,
    reason = "Integer division for binary tree parent index is intentional"
)]
fn bench_adjacency_queries(c: &mut Criterion) {
    const SIZE: usize = 1000;

    // Build new graph for queries
    let mut builder = lithos_core::graph::GraphBuilder::new();
    let mut ids = Vec::with_capacity(SIZE);

    for i in 0..SIZE {
        let id = SchemaId::new();
        let payload: Box<str> = format!("node-{i:04}").into();
        builder.add_node(id, payload);
        ids.push(id);

        // Create binary tree structure (child extends parent)
        if i > 0 {
            let parent_idx = (i - 1) / 2;
            builder.add_parent(id, ids[parent_idx]);
        }
    }

    let graph = builder.build();

    // Build old graph for comparison
    let mut old_graph =
        lithos_core::schema::graph::Graph::<Box<str>, ()>::new();
    let mut old_ids = Vec::with_capacity(SIZE);

    for i in 0..SIZE {
        let id = SchemaId::new();
        let payload: Box<str> = format!("node-{i:04}").into();
        old_graph.add_node(id, payload);
        old_ids.push(id);

        if i > 0 {
            let parent_idx = (i - 1) / 2;
            old_graph.add_edge(old_ids[parent_idx], id);
        }
    }

    let adjacency =
        lithos_core::schema::graph::AdjacencyMap::from_graph(&old_graph);

    let mut group = c.benchmark_group("adjacency");

    // Old implementation: requires AdjacencyMap construction + queries
    group.bench_function("old_parents_of", |b| {
        b.iter(|| {
            for &id in &old_ids {
                black_box(adjacency.parents_of(black_box(id)));
            }
        });
    });

    group.bench_function("old_children_of", |b| {
        b.iter(|| {
            for &id in &old_ids {
                black_box(adjacency.children_of(black_box(id)));
            }
        });
    });

    // New implementation: direct lookups
    group.bench_function("new_parents_of", |b| {
        b.iter(|| {
            for &id in &ids {
                black_box(graph.parents_of(black_box(id)));
            }
        });
    });

    group.bench_function("new_children_of", |b| {
        b.iter(|| {
            for &id in &ids {
                black_box(graph.children_of(black_box(id)));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_graph_construction, bench_adjacency_queries);
criterion_main!(benches);
