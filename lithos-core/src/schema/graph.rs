//! Directed acyclic graph (DAG) builders, editors, and sorting utilities.
//!
//! This module provides construction and patching workflows for inheritance
//! graphs, along with topological sorting that enforces acyclicity.
//!
//! # Helper Types
//!
//! - **`TopologicalSorter`**: Sorts nodes and detects cycles
//! - **`GraphBuilder`**: Builds graphs from `SchemaId` parent lists
//! - **`GraphEditor`**: Applies scoped updates to existing graphs
//!
//! # Usage
//!
//! ```rust
//! use std::collections::HashMap;
//!
//! use lithos_core::schema::{
//!     aggregate::SchemaId,
//!     graph::TopologicalSorter,
//!     inheritance::{NodeAccessor, NodeDepth},
//! };
//!
//! struct MockNode {
//!     id: SchemaId,
//!     parents: Vec<SchemaId>,
//!     children: Vec<SchemaId>,
//! }
//!
//! impl NodeAccessor for MockNode {
//!     fn id(&self) -> SchemaId {
//!         self.id
//!     }
//!
//!     fn parents(&self) -> &[SchemaId] {
//!         &self.parents
//!     }
//!
//!     fn children(&self) -> &[SchemaId] {
//!         &self.children
//!     }
//!
//!     fn depth(&self) -> NodeDepth {
//!         NodeDepth::ROOT
//!     }
//! }
//!
//! let root_id = SchemaId::new();
//! let child_id = SchemaId::new();
//! let root = MockNode {
//!     id: root_id,
//!     parents: Vec::new(),
//!     children: vec![child_id],
//! };
//! let child = MockNode {
//!     id: child_id,
//!     parents: vec![root_id],
//!     children: Vec::new(),
//! };
//!
//! let nodes = HashMap::from([(root_id, root), (child_id, child)]);
//! let sorter = TopologicalSorter::new(&nodes);
//! let (order, roots) = sorter.sort().expect("acyclic graph");
//! assert_eq!(roots, vec![root_id]);
//! assert_eq!(order, vec![root_id, child_id]);
//! ```

#![expect(
    clippy::missing_inline_in_public_items,
    reason = "public API favors readability over forced inlining"
)]
#![expect(
    clippy::iter_over_hash_type,
    reason = "graph algorithms do not rely on HashMap iteration order"
)]

use std::collections::{HashMap, HashSet, VecDeque};

use crate::schema::{
    aggregate::SchemaId,
    error::{SchemaError, SchemaLoaderError, SchemaResolutionError},
    inheritance::{InheritanceGraph, InheritanceNode, NodeAccessor, NodeDepth},
};

// ═════════════════════════════════════════════════════════════════════════════
//  GRAPH BUILDER (Graph Construction)
// ═════════════════════════════════════════════════════════════════════════════
/// Builder for constructing `InheritanceGraph<InheritanceNode>` from
/// `SchemaId` parent lists.
///
/// Parent lists are normalized (sorted, deduplicated) before building.
pub(crate) struct GraphBuilder {
    parents_by_id: ChildParentsMap,
}

impl GraphBuilder {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            parents_by_id: ChildParentsMap::new(),
        }
    }

    #[inline]
    #[expect(dead_code, reason = "reserved for future builder inputs")]
    pub(crate) fn with_nodes(nodes: ChildParentsMap) -> Self {
        Self {
            parents_by_id: nodes,
        }
    }

    #[inline]
    pub(crate) fn insert_node(&mut self, id: SchemaId, parents: Vec<SchemaId>) {
        self.parents_by_id.insert(id, parents);
    }

    /// Build a validated graph with order and roots populated.
    ///
    /// # Errors
    ///
    /// Returns an error if any parent ID is missing or the graph is cyclic.
    pub(crate) fn build(
        self,
    ) -> Result<InheritanceGraph<InheritanceNode>, SchemaLoaderError> {
        let mut parents_by_id = self.parents_by_id;
        parents_by_id.normalize_all();
        let mut nodes = Self::build_nodes(parents_by_id);
        Self::validate_parents_exist(&nodes)?;
        Self::build_children(&mut nodes);
        Self::compute_depths(&mut nodes);

        let sorter = TopologicalSorter::new(&nodes);
        let (order, roots) = sorter.sort().map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;

        let graph = Self::assemble(nodes, order, roots);

        #[cfg(debug_assertions)]
        debug_assert!(
            graph.validate_consistency().is_ok(),
            "GraphBuilder produced inconsistent graph"
        );

        Ok(graph)
    }

    fn build_nodes(
        parents_by_id: ChildParentsMap,
    ) -> HashMap<SchemaId, InheritanceNode> {
        let parents_by_id = parents_by_id.into_inner();
        let mut nodes = HashMap::with_capacity(parents_by_id.len());
        for (id, parents) in parents_by_id {
            let node = InheritanceNode::new_child(id, parents, NodeDepth::ROOT);
            nodes.insert(id, node);
        }
        nodes
    }

    fn validate_parents_exist(
        nodes: &HashMap<SchemaId, InheritanceNode>,
    ) -> Result<(), SchemaLoaderError> {
        for node in nodes.values() {
            for parent_id in node.parents() {
                if !nodes.contains_key(parent_id) {
                    return Err(SchemaLoaderError::Resolution(
                        SchemaError::Resolution(
                            SchemaResolutionError::MissingNode {
                                id: *parent_id,
                            },
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn build_children(nodes: &mut HashMap<SchemaId, InheritanceNode>) {
        let parent_to_children: HashMap<SchemaId, Vec<SchemaId>> = nodes
            .values()
            .flat_map(|node| {
                node.parents().iter().map(move |&parent| (parent, node.id()))
            })
            .fold(
                HashMap::new(),
                |mut acc: HashMap<SchemaId, Vec<SchemaId>>, (parent, child)| {
                    acc.entry(parent).or_default().push(child);
                    acc
                },
            );

        for (id, node) in nodes.iter_mut() {
            let mut children =
                parent_to_children.get(id).cloned().unwrap_or_default();
            children.sort();
            let parents = node.parents().to_vec();
            node.set_edges(parents, children);
        }
    }

    fn compute_depths(nodes: &mut HashMap<SchemaId, InheritanceNode>) {
        let mut depths: HashMap<SchemaId, usize> = HashMap::new();
        let mut queue: VecDeque<SchemaId> = nodes
            .values()
            .filter(|node| node.is_root())
            .map(InheritanceNode::id)
            .collect();

        for &root_id in &queue {
            depths.insert(root_id, 0);
        }

        while let Some(id) = queue.pop_front() {
            let Some(node) = nodes.get(&id) else {
                continue;
            };
            for &child_id in node.children() {
                if let Some(child) = nodes.get(&child_id)
                    && let Some(depth) =
                        Self::calculate_node_depth(child, &depths)
                {
                    depths.insert(child_id, depth);
                    queue.push_back(child_id);
                }
            }
        }

        for (id, depth) in depths {
            if let Some(node) = nodes.get_mut(&id) {
                node.set_depth(NodeDepth::new(depth));
            }
        }
    }

    #[inline]
    fn calculate_node_depth(
        node: &InheritanceNode,
        depths: &HashMap<SchemaId, usize>,
    ) -> Option<usize> {
        let mut max_parent_depth = 0;
        for parent_id in node.parents() {
            let &depth = depths.get(parent_id)?;
            max_parent_depth = max_parent_depth.max(depth);
        }
        Some(max_parent_depth.saturating_add(1))
    }

    fn assemble(
        nodes: HashMap<SchemaId, InheritanceNode>,
        order: Vec<SchemaId>,
        roots: Vec<SchemaId>,
    ) -> InheritanceGraph<InheritanceNode> {
        InheritanceGraph::new(nodes, order, roots)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  GRAPH EDITOR (Scoped Patching)
// ═════════════════════════════════════════════════════════════════════════════
/// Editor for scoped updates to `InheritanceGraph<InheritanceNode>`.
///
/// Updates are applied to the affected subtree; unaffected nodes keep their
/// relative order.
pub struct GraphEditor {
    graph: InheritanceGraph<InheritanceNode>,
    changed_ids: HashSet<SchemaId>,
    deleted_ids: HashSet<SchemaId>,
}

impl GraphEditor {
    /// Create a new editor from an existing graph.
    #[must_use]
    pub fn from_graph(graph: &InheritanceGraph<InheritanceNode>) -> Self {
        Self {
            graph: graph.clone(),
            changed_ids: HashSet::new(),
            deleted_ids: HashSet::new(),
        }
    }

    /// Queue a change to a node's parents.
    pub fn apply_change(&mut self, id: SchemaId, parents: Vec<SchemaId>) {
        let mut parents = parents;
        ChildParentsMap::normalize_parents(&mut parents);

        if let Some(node) = self.graph.as_mut_nodes().get_mut(&id) {
            let old_parents = node.parents().to_vec();
            for parent_id in old_parents {
                self.changed_ids.insert(parent_id);
            }
            for parent_id in &parents {
                self.changed_ids.insert(*parent_id);
            }
            let children = node.children().to_vec();
            node.set_edges(parents, children);
            node.set_depth(NodeDepth::ROOT);
        } else {
            let node = if parents.is_empty() {
                InheritanceNode::new_root(id)
            } else {
                for parent_id in &parents {
                    self.changed_ids.insert(*parent_id);
                }
                InheritanceNode::new_child(id, parents, NodeDepth::ROOT)
            };
            self.graph.as_mut_nodes().insert(id, node);
        }

        self.changed_ids.insert(id);
    }

    /// Queue a node deletion.
    pub fn delete_node(&mut self, id: SchemaId) {
        if let Some(node) = self.graph.as_mut_nodes().remove(&id) {
            for &parent_id in node.parents() {
                self.changed_ids.insert(parent_id);
            }
            for &child_id in node.children() {
                self.changed_ids.insert(child_id);
            }
        }

        self.changed_ids.insert(id);
        self.deleted_ids.insert(id);
        self.graph.as_mut_order().retain(|entry| *entry != id);
        self.graph.as_mut_roots().retain(|entry| *entry != id);
    }

    /// Apply queued changes and return the patched graph.
    ///
    /// # Errors
    ///
    /// Returns an error if patching introduces a cycle.
    pub fn patch(
        mut self,
    ) -> Result<InheritanceGraph<InheritanceNode>, SchemaLoaderError> {
        Self::apply_deletes_cleanup(&mut self.graph, &self.deleted_ids);

        let affected = self.graph.affected_subtree(&self.changed_ids);
        Self::rebuild_children(&mut self.graph, &affected);

        let sorter = TopologicalSorter::new(self.graph.nodes());
        let affected_order = sorter.sort_scoped(&affected).map_err(|e| {
            SchemaLoaderError::Resolution(SchemaError::Resolution(e))
        })?;

        Self::recompute_depths(&mut self.graph, &affected);
        Self::splice_order(&mut self.graph, &affected_order, &affected)?;
        Self::rebuild_roots(&mut self.graph);

        #[cfg(debug_assertions)]
        debug_assert!(
            self.graph.validate_consistency().is_ok(),
            "GraphEditor produced inconsistent graph"
        );

        Ok(self.graph)
    }

    fn apply_deletes_cleanup(
        graph: &mut InheritanceGraph<InheritanceNode>,
        deleted: &HashSet<SchemaId>,
    ) {
        if deleted.is_empty() {
            return;
        }

        for node in graph.as_mut_nodes().values_mut() {
            let mut parents = node.parents().to_vec();
            let mut children = node.children().to_vec();
            parents.retain(|id| !deleted.contains(id));
            children.retain(|id| !deleted.contains(id));
            node.set_edges(parents, children);
        }

        let deleted_ids: Vec<SchemaId> = deleted.iter().copied().collect();
        Self::prune(graph, &deleted_ids);
    }

    fn prune(
        graph: &mut InheritanceGraph<InheritanceNode>,
        deleted: &[SchemaId],
    ) {
        for id in deleted {
            graph.as_mut_nodes().remove(id);
        }
        let remaining: HashSet<SchemaId> =
            graph.nodes().keys().copied().collect();
        graph.as_mut_order().retain(|id| remaining.contains(id));
        graph.as_mut_roots().retain(|id| remaining.contains(id));
    }

    fn rebuild_children(
        graph: &mut InheritanceGraph<InheritanceNode>,
        affected: &HashSet<SchemaId>,
    ) {
        if affected.is_empty() {
            return;
        }

        let mut update_ids: HashSet<SchemaId> = HashSet::new();
        for &id in affected {
            update_ids.insert(id);
            if let Some(node) = graph.nodes().get(&id) {
                for parent_id in node.parents() {
                    update_ids.insert(*parent_id);
                }
            }
        }

        let mut parent_to_children: HashMap<SchemaId, Vec<SchemaId>> =
            HashMap::new();
        for node in graph.nodes().values() {
            for parent in node.parents() {
                if update_ids.contains(parent) {
                    parent_to_children
                        .entry(*parent)
                        .or_default()
                        .push(node.id());
                }
            }
        }

        for id in update_ids {
            if let Some(node) = graph.as_mut_nodes().get_mut(&id) {
                let mut children =
                    parent_to_children.remove(&id).unwrap_or_default();
                children.sort();
                let parents = node.parents().to_vec();
                node.set_edges(parents, children);
            }
        }
    }

    fn recompute_depths(
        graph: &mut InheritanceGraph<InheritanceNode>,
        affected: &HashSet<SchemaId>,
    ) {
        if affected.is_empty() {
            return;
        }

        let mut depths: HashMap<SchemaId, usize> = HashMap::new();
        let mut queue: VecDeque<SchemaId> = VecDeque::new();

        for &id in affected {
            if let Some(node) = graph.nodes().get(&id)
                && node.parents().iter().all(|p| !affected.contains(p))
            {
                let depth = Self::get_max_parent_depth(graph, node);
                depths.insert(id, depth);
                queue.push_back(id);
            }
        }

        while let Some(id) = queue.pop_front() {
            let Some(node) = graph.nodes().get(&id) else {
                continue;
            };
            for &child_id in node.children() {
                if !affected.contains(&child_id) {
                    continue;
                }
                if let Some(child) = graph.nodes().get(&child_id)
                    && let Some(depth) =
                        Self::calculate_node_depth(child, &depths)
                {
                    depths.insert(child_id, depth);
                    queue.push_back(child_id);
                }
            }
        }

        for (id, depth) in depths {
            if let Some(node) = graph.as_mut_nodes().get_mut(&id) {
                node.set_depth(NodeDepth::new(depth));
            }
        }
    }

    #[inline]
    fn calculate_node_depth(
        node: &InheritanceNode,
        depths: &HashMap<SchemaId, usize>,
    ) -> Option<usize> {
        let mut max_parent_depth = 0;
        for parent_id in node.parents() {
            let &depth = depths.get(parent_id)?;
            max_parent_depth = max_parent_depth.max(depth);
        }
        Some(max_parent_depth.saturating_add(1))
    }

    fn get_max_parent_depth(
        graph: &InheritanceGraph<InheritanceNode>,
        node: &InheritanceNode,
    ) -> usize {
        node.parents()
            .iter()
            .filter_map(|p| {
                graph.nodes().get(p).map(|pn| pn.depth().as_usize())
            })
            .max()
            .map_or(0, |d| d.saturating_add(1))
    }

    fn splice_order(
        graph: &mut InheritanceGraph<InheritanceNode>,
        affected_order: &[SchemaId],
        affected: &HashSet<SchemaId>,
    ) -> Result<(), SchemaLoaderError> {
        let mut anchor_map: HashMap<Option<SchemaId>, Vec<SchemaId>> =
            HashMap::new();

        for &id in affected_order {
            let anchor = Self::nearest_unaffected_ancestor(graph, id, affected);
            anchor_map.entry(anchor).or_default().push(id);
        }

        let capacity = graph.order().len().saturating_add(affected.len());
        let mut new_order = Vec::with_capacity(capacity);
        for id in
            graph.order().iter().copied().filter(|id| !affected.contains(id))
        {
            new_order.push(id);
            if let Some(mut list) = anchor_map.remove(&Some(id)) {
                new_order.append(&mut list);
            }
        }
        if let Some(mut list) = anchor_map.remove(&None) {
            new_order.append(&mut list);
        }
        for mut list in anchor_map.into_values() {
            new_order.append(&mut list);
        }

        if new_order.len() != graph.nodes().len() {
            return Err(SchemaLoaderError::Resolution(
                SchemaError::Resolution(SchemaResolutionError::CycleDetected {
                    schemas: Vec::new(),
                }),
            ));
        }

        *graph.as_mut_order() = new_order;
        Ok(())
    }

    fn nearest_unaffected_ancestor(
        graph: &InheritanceGraph<InheritanceNode>,
        id: SchemaId,
        affected: &HashSet<SchemaId>,
    ) -> Option<SchemaId> {
        let node = graph.nodes().get(&id)?;

        for &parent_id in node.parents() {
            if !affected.contains(&parent_id) {
                return Some(parent_id);
            }
            if let Some(ancestor) =
                Self::nearest_unaffected_ancestor(graph, parent_id, affected)
            {
                return Some(ancestor);
            }
        }
        None
    }

    fn rebuild_roots(graph: &mut InheritanceGraph<InheritanceNode>) {
        let roots: Vec<SchemaId> = graph
            .nodes()
            .values()
            .filter(|node| node.parents().is_empty())
            .map(InheritanceNode::id)
            .collect();
        *graph.as_mut_roots() = roots;
    }
}

type TopologicalOrder = (Vec<SchemaId>, Vec<SchemaId>);

/// Sorter for producing a topological order of nodes.
pub struct TopologicalSorter<'graph, T> {
    nodes: &'graph HashMap<SchemaId, T>,
}

impl<'graph, T: NodeAccessor> TopologicalSorter<'graph, T> {
    /// Create a sorter for the provided graph nodes.
    #[must_use]
    pub fn new(nodes: &'graph HashMap<SchemaId, T>) -> Self {
        Self {
            nodes,
        }
    }

    /// Compute topological order using Kahn's algorithm.
    ///
    /// # Errors
    ///
    /// Returns `CycleDetected` if the graph contains a cycle.
    #[expect(
        clippy::excessive_nesting,
        reason = "graph traversal keeps nesting explicit for clarity"
    )]
    pub fn sort(&self) -> Result<TopologicalOrder, SchemaResolutionError> {
        let mut in_degree: HashMap<SchemaId, usize> = self
            .nodes
            .values()
            .map(|node| (node.id(), node.parents().len()))
            .collect();

        let mut queue: VecDeque<SchemaId> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        let roots: Vec<SchemaId> = queue.iter().copied().collect();

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);

            if let Some(node) = self.nodes.get(&node_id) {
                for &child_id in node.children() {
                    if let Some(deg) = in_degree.get_mut(&child_id) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(child_id);
                        }
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: Vec::new(),
            });
        }

        Ok((order, roots))
    }

    /// Topological sort for only the affected subtree.
    ///
    /// # Errors
    ///
    /// Returns `CycleDetected` if the affected subtree contains a cycle.
    #[expect(
        clippy::excessive_nesting,
        reason = "scoped traversal keeps nesting explicit for clarity"
    )]
    pub fn sort_scoped(
        &self,
        affected: &HashSet<SchemaId>,
    ) -> Result<Vec<SchemaId>, SchemaResolutionError> {
        let mut in_degree: HashMap<SchemaId, usize> = HashMap::new();

        for &id in affected {
            if let Some(node) = self.nodes.get(&id) {
                let parent_in_scope_count = node
                    .parents()
                    .iter()
                    .filter(|pid| affected.contains(pid))
                    .count();
                in_degree.insert(id, parent_in_scope_count);
            }
        }

        let mut queue: VecDeque<SchemaId> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::with_capacity(affected.len());

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);

            if let Some(node) = self.nodes.get(&node_id) {
                for &child_id in node.children() {
                    if !affected.contains(&child_id) {
                        continue;
                    }

                    if let Some(deg) = in_degree.get_mut(&child_id) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(child_id);
                        }
                    }
                }
            }
        }

        if order.len() != affected.len() {
            return Err(SchemaResolutionError::CycleDetected {
                schemas: Vec::new(),
            });
        }

        Ok(order)
    }
}

/// Map of child schema IDs to their parent IDs.
#[derive(Debug, Clone, Default)]
pub(crate) struct ChildParentsMap(HashMap<SchemaId, Vec<SchemaId>>);

impl ChildParentsMap {
    #[inline]
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    #[inline]
    #[expect(dead_code, reason = "reserved for future builder inputs")]
    pub(crate) fn with_entries(map: HashMap<SchemaId, Vec<SchemaId>>) -> Self {
        Self(map)
    }

    #[inline]
    pub(crate) fn insert(&mut self, id: SchemaId, parents: Vec<SchemaId>) {
        self.0.insert(id, parents);
    }

    #[inline]
    pub(crate) fn into_inner(self) -> HashMap<SchemaId, Vec<SchemaId>> {
        self.0
    }

    fn normalize_parents(parents: &mut Vec<SchemaId>) {
        parents.sort();
        parents.dedup();
    }

    fn normalize_all(&mut self) {
        for parents in self.0.values_mut() {
            Self::normalize_parents(parents);
        }
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn build_diamond_graph()
    -> (InheritanceGraph<InheritanceNode>, Vec<SchemaId>) {
        let id_a = SchemaId::new();
        let id_b = SchemaId::new();
        let id_c = SchemaId::new();
        let id_d = SchemaId::new();

        let mut node_a = InheritanceNode::new_root(id_a);
        let mut node_b =
            InheritanceNode::new_child(id_b, vec![id_a], NodeDepth::ROOT);
        let mut node_c =
            InheritanceNode::new_child(id_c, vec![id_a], NodeDepth::ROOT);
        let node_d =
            InheritanceNode::new_child(id_d, vec![id_b, id_c], NodeDepth::ROOT);

        node_a.set_edges(Vec::new(), vec![id_b, id_c]);
        node_b.set_edges(vec![id_a], vec![id_d]);
        node_c.set_edges(vec![id_a], vec![id_d]);

        let nodes = HashMap::from([
            (id_a, node_a),
            (id_b, node_b),
            (id_c, node_c),
            (id_d, node_d),
        ]);

        let graph = InheritanceGraph::new(nodes, Vec::new(), Vec::new());

        (graph, vec![id_a, id_b, id_c, id_d])
    }

    #[test]
    fn computes_depths_for_diamond_inheritance() {
        let (mut graph, ids) = build_diamond_graph();
        let id_a = *ids.first().expect("id a");
        let id_d = *ids.get(3).expect("id d");

        GraphBuilder::compute_depths(graph.as_mut_nodes());

        let depth_a = graph.nodes().get(&id_a).expect("node a").depth();
        let depth_d = graph.nodes().get(&id_d).expect("node d").depth();

        assert_eq!(depth_a, NodeDepth::ROOT);
        assert_eq!(depth_d, NodeDepth::new(2));
    }

    #[test]
    fn topological_order_respects_all_parents() {
        let (mut graph, ids) = build_diamond_graph();
        GraphBuilder::compute_depths(graph.as_mut_nodes());

        let sorter = TopologicalSorter::new(graph.nodes());
        let (order, _roots) = sorter.sort().expect("topo sort");

        let position: HashMap<SchemaId, usize> =
            order.iter().copied().enumerate().map(|(i, id)| (id, i)).collect();

        let id_a = *ids.first().expect("id a");
        let id_b = *ids.get(1).expect("id b");
        let id_c = *ids.get(2).expect("id c");
        let id_d = *ids.get(3).expect("id d");

        let pos_a = *position.get(&id_a).expect("position a");
        let pos_b = *position.get(&id_b).expect("position b");
        let pos_c = *position.get(&id_c).expect("position c");
        let pos_d = *position.get(&id_d).expect("position d");

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn detects_cycles_in_multi_parent_graph() {
        let id_a = SchemaId::new();
        let id_b = SchemaId::new();
        let id_c = SchemaId::new();

        let node_a =
            InheritanceNode::new_child(id_a, vec![id_c], NodeDepth::ROOT);
        let node_b =
            InheritanceNode::new_child(id_b, vec![id_a], NodeDepth::ROOT);
        let node_c =
            InheritanceNode::new_child(id_c, vec![id_b], NodeDepth::ROOT);

        let nodes =
            HashMap::from([(id_a, node_a), (id_b, node_b), (id_c, node_c)]);
        let sorter = TopologicalSorter::new(&nodes);

        sorter.sort().unwrap_err();
    }

    #[test]
    fn patch_updates_bidirectional_links() {
        let id_a = SchemaId::new();
        let id_b = SchemaId::new();

        let mut node_a = InheritanceNode::new_root(id_a);
        node_a.set_edges(Vec::new(), vec![id_b]);
        let node_b =
            InheritanceNode::new_child(id_b, vec![id_a], NodeDepth::ROOT);

        let graph = InheritanceGraph::new(
            HashMap::from([(id_a, node_a), (id_b, node_b)]),
            vec![id_a, id_b],
            vec![id_a],
        );

        let mut editor = GraphEditor::from_graph(&graph);
        editor.apply_change(id_b, Vec::new());
        let patched_graph = editor.patch().expect("patch graph");

        #[cfg(debug_assertions)]
        patched_graph.validate_consistency().expect("consistent graph");

        let parent = patched_graph.nodes().get(&id_a).expect("node a");
        let child = patched_graph.nodes().get(&id_b).expect("node b");
        assert!(parent.children().is_empty());
        assert!(child.parents().is_empty());
    }
}
