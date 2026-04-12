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

/// Computes topological order using Kahn's algorithm.
///
/// # Errors
/// Returns `GraphError::CycleDetected` if graph has cycles.
pub(crate) fn topological_sort_with_nodes<Id>(
    parents: &HashMap<Id, Vec<Id>>,
    nodes: impl IntoIterator<Item = Id>,
) -> Result<(Vec<Id>, Vec<Id>), GraphError<Id>>
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
    for (child, parent_ids) in parents {
        let entry =
            indegree.get_mut(child).expect("child IDs validated to exist");
        *entry = entry.saturating_add(parent_ids.len());

        for parent in parent_ids {
            children.entry(*parent).or_default().push(*child);
        }
    }

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
    for (id, degree) in &indegree {
        if *degree == 0 {
            heap.push(Reverse(*id));
        }
    }

    let mut order = Vec::with_capacity(indegree.len());
    let mut indegree = indegree;

    while let Some(Reverse(id)) = heap.pop() {
        order.push(id);
        if let Some(children) = children.get(&id) {
            for child in children {
                let entry = indegree
                    .get_mut(child)
                    .expect("child IDs validated to exist");
                *entry = entry.saturating_sub(1);
                if *entry == 0 {
                    heap.push(Reverse(*child));
                }
            }
        }
    }

    (order, indegree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topo_sort_respects_dependencies() {
        let parents = HashMap::from([(2, vec![1]), (3, vec![2])]);
        let (order, roots) =
            topological_sort_with_nodes(&parents, [1, 2, 3]).unwrap();

        assert_eq!(roots, vec![1]);

        let pos: HashMap<_, _> =
            order.iter().copied().enumerate().map(|(i, id)| (id, i)).collect();

        assert!(pos[&1] < pos[&2]);
        assert!(pos[&2] < pos[&3]);
    }

    #[test]
    fn topo_sort_detects_cycles() {
        let parents = HashMap::from([(2, vec![1]), (3, vec![2]), (1, vec![3])]);
        let result = topological_sort_with_nodes(&parents, [1, 2, 3]);

        assert!(matches!(result, Err(GraphError::CycleDetected { .. })));
    }

    #[test]
    fn topo_sort_deterministic() {
        let parents = HashMap::from([(2, vec![1]), (3, vec![1])]);

        let (order1, _) =
            topological_sort_with_nodes(&parents, [1, 2, 3]).unwrap();
        let (order2, _) =
            topological_sort_with_nodes(&parents, [1, 2, 3]).unwrap();

        assert_eq!(order1, order2);
    }
}
