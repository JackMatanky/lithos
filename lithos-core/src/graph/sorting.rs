//! Internal topological sorting helpers for the graph module.
//!
//! Uses Kahn's algorithm with a deterministic queue to produce stable orderings
//! and detect cycles for DAG validation.

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

use crate::graph::GraphError;

/// Topological sort result: (order, roots).
type TopoResult<Id> = Result<(Vec<Id>, Vec<Id>), GraphError<Id>>;

/// Computes topological order using Kahn's algorithm.
///
/// # Errors
/// Returns `GraphError::CycleDetected` if graph has cycles.
pub(crate) fn topological_sort_with_nodes<Id>(
    parents: &HashMap<Id, Vec<Id>>,
    nodes: impl IntoIterator<Item = Id>,
) -> TopoResult<Id>
where
    Id: Copy + Eq + Hash + Ord,
{
    let mut indegree = init_indegree(parents, nodes)?;
    let children = build_children(parents, &mut indegree);
    let roots = collect_roots(&indegree);
    let (order, indegree) = kahn_order(&children, indegree);

    if order.len() != indegree.len() {
        let remaining = indegree
            .into_iter()
            .filter_map(|(id, degree)| (degree > 0).then_some(id))
            .collect();
        return Err(GraphError::CycleDetected {
            nodes: remaining,
        });
    }

    Ok((order, roots))
}

fn init_indegree<Id>(
    parents: &HashMap<Id, Vec<Id>>,
    nodes: impl IntoIterator<Item = Id>,
) -> Result<HashMap<Id, usize>, GraphError<Id>>
where
    Id: Copy + Eq + Hash + Ord,
{
    let mut indegree = HashMap::new();

    for id in nodes {
        indegree.entry(id).or_insert(0);
    }

    #[expect(
        clippy::iter_over_hash_type,
        reason = "Iteration order doesn't affect validation logic"
    )]
    for (child, parent_ids) in parents {
        if !indegree.contains_key(child) {
            return Err(GraphError::MissingNode {
                id: *child,
            });
        }
        for parent in parent_ids {
            if !indegree.contains_key(parent) {
                return Err(GraphError::MissingNode {
                    id: *parent,
                });
            }
        }
    }

    Ok(indegree)
}

fn build_children<Id>(
    parents: &HashMap<Id, Vec<Id>>,
    indegree: &mut HashMap<Id, usize>,
) -> HashMap<Id, Vec<Id>>
where
    Id: Copy + Eq + Hash + Ord,
{
    let mut children: HashMap<Id, Vec<Id>> = HashMap::new();
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Iteration order doesn't affect adjacency list construction"
    )]
    for (child, parent_ids) in parents {
        // INVARIANT: child IDs validated in init_indegree
        if let Some(entry) = indegree.get_mut(child) {
            *entry = entry.saturating_add(parent_ids.len());
        }

        for parent in parent_ids {
            children.entry(*parent).or_default().push(*child);
        }
    }

    #[expect(
        clippy::iter_over_hash_type,
        reason = "Iteration order doesn't affect edge normalization"
    )]
    for child_list in children.values_mut() {
        child_list.sort();
        child_list.dedup();
    }

    children
}

fn collect_roots<Id>(indegree: &HashMap<Id, usize>) -> Vec<Id>
where
    Id: Copy + Eq + Hash + Ord,
{
    let mut roots: Vec<Id> = indegree
        .iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(*id))
        .collect();
    roots.sort();
    roots
}

fn kahn_order<Id>(
    children: &HashMap<Id, Vec<Id>>,
    indegree: HashMap<Id, usize>,
) -> (Vec<Id>, HashMap<Id, usize>)
where
    Id: Copy + Eq + Hash + Ord,
{
    let mut heap: BinaryHeap<Reverse<Id>> = BinaryHeap::new();
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Iteration order doesn't affect heap initialization (heap \
                  re-sorts)"
    )]
    for (id, degree) in &indegree {
        if *degree == 0 {
            heap.push(Reverse(*id));
        }
    }

    let mut order = Vec::with_capacity(indegree.len());
    let mut indegree = indegree;

    while let Some(Reverse(id)) = heap.pop() {
        order.push(id);
        let Some(children) = children.get(&id) else {
            continue;
        };

        for child in children {
            // INVARIANT: child IDs validated in init_indegree
            let Some(entry) = indegree.get_mut(child) else {
                continue;
            };
            *entry = entry.saturating_sub(1);
            if *entry == 0 {
                heap.push(Reverse(*child));
            }
        }
    }

    (order, indegree)
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::shadow_unrelated,
    reason = "Test code uses simplified patterns for clarity"
)]
mod tests {
    use super::*;

    mod topological_sort_with_nodes {
        use super::*;

        #[test]
        fn returns_root_for_linear_chain() {
            let parents =
                HashMap::from([(2i32, vec![1i32]), (3i32, vec![2i32])]);
            let (_order, roots) =
                topological_sort_with_nodes(&parents, [1i32, 2i32, 3i32])
                    .unwrap();

            assert_eq!(roots, vec![1i32], "expected root [1], got {roots:?}");
        }

        #[test]
        fn orders_parent_before_child_for_first_edge() {
            let parents =
                HashMap::from([(2i32, vec![1i32]), (3i32, vec![2i32])]);
            let (order, _roots) =
                topological_sort_with_nodes(&parents, [1i32, 2i32, 3i32])
                    .unwrap();

            let pos: HashMap<_, _> = order
                .iter()
                .copied()
                .enumerate()
                .map(|(i, id)| (id, i))
                .collect();

            assert!(
                pos[&1i32] < pos[&2i32],
                "expected 1 before 2, positions: {pos:?}"
            );
        }

        #[test]
        fn orders_parent_before_child_for_second_edge() {
            let parents =
                HashMap::from([(2i32, vec![1i32]), (3i32, vec![2i32])]);
            let (order, _roots) =
                topological_sort_with_nodes(&parents, [1i32, 2i32, 3i32])
                    .unwrap();

            let pos: HashMap<_, _> = order
                .iter()
                .copied()
                .enumerate()
                .map(|(i, id)| (id, i))
                .collect();

            assert!(
                pos[&2i32] < pos[&3i32],
                "expected 2 before 3, positions: {pos:?}"
            );
        }

        #[test]
        fn returns_error_when_cycle_detected() {
            let parents = HashMap::from([
                (2i32, vec![1i32]),
                (3i32, vec![2i32]),
                (1i32, vec![3i32]),
            ]);
            let result =
                topological_sort_with_nodes(&parents, [1i32, 2i32, 3i32]);

            assert!(
                matches!(&result, Err(GraphError::CycleDetected { .. })),
                "expected cycle error, got {result:?}"
            );
        }

        #[test]
        fn returns_error_when_parent_missing() {
            let parents = HashMap::from([(2i32, vec![1i32])]);
            let result = topological_sort_with_nodes(&parents, [2i32]);

            assert!(
                matches!(&result, Err(GraphError::MissingNode { .. })),
                "expected missing-node error, got {result:?}"
            );
        }

        #[test]
        fn includes_isolated_node_in_order() {
            let parents: HashMap<u8, Vec<u8>> = HashMap::new();
            let (order, _roots) =
                topological_sort_with_nodes(&parents, [1, 2]).unwrap();

            assert!(
                order.contains(&2),
                "expected order to include isolated node 2, got {order:?}"
            );
        }

        #[test]
        fn includes_isolated_node_in_roots() {
            let parents: HashMap<u8, Vec<u8>> = HashMap::new();
            let (_order, roots) =
                topological_sort_with_nodes(&parents, [1, 2]).unwrap();

            assert!(
                roots.contains(&2),
                "expected roots to include isolated node 2, got {roots:?}"
            );
        }

        #[test]
        fn returns_empty_order_for_empty_graph() {
            let parents: HashMap<u8, Vec<u8>> = HashMap::new();
            let (order, _roots) =
                topological_sort_with_nodes(&parents, []).unwrap();

            assert!(
                order.is_empty(),
                "expected empty order for empty graph, got {order:?}"
            );
        }

        #[test]
        fn returns_deterministic_order_for_parallel_children() {
            let parents = HashMap::from([
                (2i32, vec![1i32]),
                (3i32, vec![1i32]),
                (4i32, vec![1i32]),
            ]);

            let (order1, _roots) =
                topological_sort_with_nodes(&parents, [1i32, 2i32, 3i32, 4i32])
                    .unwrap();
            let (order2, _roots) =
                topological_sort_with_nodes(&parents, [1i32, 2i32, 3i32, 4i32])
                    .unwrap();

            assert_eq!(
                order1, order2,
                "expected deterministic order, got {order1:?} vs {order2:?}"
            );
        }
    }
}
