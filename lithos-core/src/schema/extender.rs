//! `Extender` — builds a topologically-ordered [`InheritanceGraph`] from
//! raw schemas with resolved property references.
//!
//! This module resolves inheritance relationships between schemas and produces
//! a sorted execution plan for property merging.
//!
//! # Pipeline Context
//!
//! RefExpander → Extender → InheritanceGraph → Merger
//!
//! # Design
//!
//! The `Extender` takes:
//! - Stale schemas with `$ref`s already resolved by [`RefExpander`].
//! - A map of fresh (non-stale) schemas loaded from the DB — these may act as
//!   parents.
//!
//! It produces a [`InheritanceGraph`] whose nodes are in **topological order**
//! (parents before children) so the downstream [`Merger`] can walk the tree
//! once without back-tracking.
//!
//! [`RefExpander`]: super::expander::RefExpander

use std::collections::{HashMap, HashSet, VecDeque};

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    error::SchemaError,
    property::{Property, PropertyName},
    raw::RawSchema,
};

type ExpandedSchemaInput =
    (SchemaId, RawSchema, HashMap<PropertyName, Property>);

// ─────────────────────────────────────────────────────────────────────────────
//  NodeDepth
// ─────────────────────────────────────────────────────────────────────────────

/// Inheritance depth in the schema tree.
///
/// Depth is 1-indexed: root schemas have depth 1, their children have depth 2,
/// and so on. This is used to prevent infinite recursion from cyclic
/// inheritance and to provide meaningful error messages when the limit is
/// exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NodeDepth(usize);

impl NodeDepth {
    /// Creates a depth for a root node (depth = 1).
    #[inline]
    pub const fn root() -> Self {
        Self(1)
    }

    /// Returns the depth as a raw usize.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "will be used for depth validation in resolver"
        )
    )]
    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Returns the depth of a child node (parent depth + 1).
    #[inline]
    pub fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Returns true if this depth exceeds the given limit.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "will be used for depth validation in resolver"
        )
    )]
    #[inline]
    pub const fn exceeds(self, limit: usize) -> bool {
        self.0 > limit
    }
}

impl Default for NodeDepth {
    fn default() -> Self {
        Self::root()
    }
}

impl std::fmt::Display for NodeDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NodeDepth> for usize {
    #[inline]
    fn from(depth: NodeDepth) -> Self {
        depth.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaNode
// ─────────────────────────────────────────────────────────────────────────────

/// A single node in the inheritance tree, ready for property merging.
#[derive(Debug)]
pub(crate) struct SchemaNode {
    /// Validated schema name.
    pub name: SchemaName,
    /// Properties defined by this schema (`HashMap` for O(1) lookup).
    pub properties: HashMap<PropertyName, Property>,
    /// Validated property names inherited from the parent that this schema
    /// excludes.
    pub excludes: Vec<PropertyName>,
    /// Parent schema identifier, if any.
    pub parent_id: Option<SchemaId>,
    /// Children of this node (populated during `build`).
    pub children: Vec<SchemaId>,
    /// Inheritance depth in the tree (1 for roots, increments with each
    /// level).
    pub depth: NodeDepth,
}

// ─────────────────────────────────────────────────────────────────────────────
//  InheritanceGraph
// ─────────────────────────────────────────────────────────────────────────────

/// A topologically-ordered inheritance tree of schemas.
///
/// - `nodes` provides O(1) lookup by `SchemaId`.
/// - `order` is the topological order (`roots` first, leaves last) produced by
///   Kahn's algorithm.
/// - `roots` contains schemas whose `parent_id` is `None` (or whose parent is a
///   DB-fresh known parent rather than an in-batch node).
#[derive(Debug)]
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Loader` instead.
#[doc(hidden)]
pub struct InheritanceGraph {
    /// IDs of root schemas (no in-batch parent).
    roots: Vec<SchemaId>,
    /// All nodes indexed by `SchemaId`.
    nodes: HashMap<SchemaId, SchemaNode>,
    /// Schema IDs in topological order (parents before children).
    order: Vec<SchemaId>,
}

impl InheritanceGraph {
    /// Returns schema IDs in topological order (parents first).
    ///
    /// Suitable for a single linear walk by [`Resolver`].
    ///
    /// [`Resolver`]: super::merger::Merger
    #[inline]
    #[must_use]
    /// **Internal API**: Public for benchmarking only.
    #[doc(hidden)]
    pub fn nodes(&self) -> &[SchemaId] {
        &self.order
    }

    /// Returns the `SchemaNode` for the given ID, or `None` if not found.
    #[inline]
    #[must_use]
    pub(crate) fn get(&self, id: SchemaId) -> Option<&SchemaNode> {
        self.nodes.get(&id)
    }

    /// Returns root schema IDs (those with no in-batch parent).
    #[inline]
    #[must_use]
    pub(crate) fn roots(&self) -> &[SchemaId] {
        &self.roots
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Extender
// ─────────────────────────────────────────────────────────────────────────────

/// Type alias for the pair of name indexes produced by Phase 1.
type NameIndexes =
    (HashMap<SchemaName, SchemaId>, HashMap<SchemaId, SchemaName>);

/// Type alias for the `(order, roots)` pair returned by Kahn's algorithm.
type KahnResult = (Vec<SchemaId>, Vec<SchemaId>);

/// Builds a [`InheritanceGraph`] from ref-expanded schemas.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Loader` instead.
#[doc(hidden)]
#[non_exhaustive]
pub struct Extender;

impl Extender {
    /// Build a [`InheritanceGraph`] from stale, ref-expanded schemas.
    ///
    /// `expanded` — schemas processed by the [`RefExpander`].
    /// `known_parents` — fresh schemas pre-loaded from the DB; their IDs are
    /// valid parent targets.
    ///
    /// # Errors
    ///
    /// - `SchemaError::Inheritance(SchemaInheritanceError::CircularInheritance)`
    ///   — a cycle was detected.
    /// - `SchemaError::Inheritance(SchemaInheritanceError::ParentNotFound)` — a
    ///   `extends` name refers to a schema that is neither in `expanded` nor in
    ///   `known_parents`.
    /// - `SchemaError::Resolution(SchemaResolutionError::DuplicateSchemaName)`
    ///   — two schemas share the same name.
    ///
    /// [`RefExpander`]: super::expander::RefExpander
    #[inline]
    /// **Internal API**: Public for benchmarking only.
    #[doc(hidden)]
    pub fn build(
        expanded: Vec<ExpandedSchemaInput>,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<InheritanceGraph, SchemaError> {
        // Phase 1: build name ↔ id indexes.
        let (name_to_id, id_to_name) =
            Self::build_name_indexes(&expanded, known_parents)?;

        // Phase 2: build node map with resolved parent IDs.
        let mut nodes = Self::build_nodes(expanded, &name_to_id)?;

        // Phase 3: DFS cycle detection.
        Self::detect_cycles(&nodes, known_parents, &id_to_name)?;

        // Phase 4: populate children lists.
        Self::populate_children(&mut nodes);

        // Phase 5: compute inheritance depths.
        Self::compute_depths(&mut nodes, known_parents);

        // Phase 6: Kahn's topological ordering.
        let (order, roots) = Self::kahn_order(&nodes)?;

        let graph = InheritanceGraph {
            roots,
            nodes,
            order,
        };
        let roots_ref = graph.roots();
        let _roots_len = roots_ref.len();
        Ok(graph)
    }

    /// Phase 1 — build owned `name → id` and `id → name` indexes.
    ///
    /// Uses `Box<str>` keys so `expanded` can be consumed in Phase 2 without
    /// lifetime issues.  `Box<str>: Borrow<str>` so `HashMap::get(&str)` works.
    fn build_name_indexes(
        expanded: &[ExpandedSchemaInput],
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<NameIndexes, SchemaError> {
        let cap = expanded.len();
        let mut name_to_id: HashMap<SchemaName, SchemaId> =
            HashMap::with_capacity(cap.saturating_add(known_parents.len()));
        let mut id_to_name: HashMap<SchemaId, SchemaName> =
            HashMap::with_capacity(cap);

        // Iterating over a HashMap; order doesn't matter here (all entries
        // are inserted unconditionally).
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Insertion order of DB-fresh parents is irrelevant; all \
                      entries are written to name_to_id unconditionally"
        )]
        for (id, schema) in known_parents {
            name_to_id.insert(schema.name().clone(), *id);
        }
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics on &(SchemaId, RawSchema, ..) tuples"
        )]
        for (id, raw, _) in expanded {
            let name = SchemaName::try_new(raw.name())?;
            if name_to_id.insert(name.clone(), *id).is_some()
                && !known_parents.values().any(|s| s.name() == &name)
            {
                return Err(SchemaError::Resolution(
                    super::error::SchemaResolutionError::DuplicateSchemaName {
                        name: name.as_ref().into(),
                    },
                ));
            }
            id_to_name.insert(*id, name);
        }
        Ok((name_to_id, id_to_name))
    }

    /// Phase 2 — build the node map, resolving each `extends` name to a
    /// `SchemaId`.
    fn build_nodes(
        expanded: Vec<ExpandedSchemaInput>,
        name_to_id: &HashMap<SchemaName, SchemaId>,
    ) -> Result<HashMap<SchemaId, SchemaNode>, SchemaError> {
        let mut nodes = HashMap::with_capacity(expanded.len());
        for (id, raw, properties) in expanded {
            let name = SchemaName::try_new(raw.name())?;
            let parent_id = Self::resolve_parent(raw.extends(), name_to_id)?;
            nodes.insert(id, SchemaNode {
                name,
                properties,
                excludes: raw.excludes().to_vec(),
                parent_id,
                children: Vec::new(),
                // Depth computed in Phase 5
                depth: NodeDepth::root(),
            });
        }
        Ok(nodes)
    }

    /// Resolve the optional `extends` string to a `SchemaId`.
    fn resolve_parent(
        parent_name: Option<&SchemaName>,
        name_to_id: &HashMap<SchemaName, SchemaId>,
    ) -> Result<Option<SchemaId>, SchemaError> {
        let Some(parent_name) = parent_name else {
            return Ok(None);
        };
        // SchemaName already validated - no need to re-validate
        // SchemaName: Borrow<str> so `.get(parent_name.as_ref())` works with
        // HashMap<SchemaName, _>
        name_to_id.get(parent_name.as_ref()).copied().map(Some).ok_or_else(
            || {
                SchemaError::Inheritance(
                    super::error::SchemaInheritanceError::ParentNotFound {
                        name: parent_name.as_ref().into(),
                    },
                )
            },
        )
    }

    /// Phase 3 — DFS cycle detection over in-batch nodes.
    fn detect_cycles(
        nodes: &HashMap<SchemaId, SchemaNode>,
        known_parents: &HashMap<SchemaId, Schema>,
        id_to_name: &HashMap<SchemaId, SchemaName>,
    ) -> Result<(), SchemaError> {
        let mut checker = CycleChecker {
            nodes,
            known_parents,
            id_to_name,
            visited: HashSet::with_capacity(nodes.len()),
            in_progress: HashSet::new(),
        };
        // Iteration order doesn't matter — DFS with visited set handles
        // any ordering.
        let ids: Vec<SchemaId> = nodes.keys().copied().collect();
        for id in ids {
            checker.visit(id)?;
        }
        Ok(())
    }

    /// Phase 4 — populate each node's `children` list.
    fn populate_children(nodes: &mut HashMap<SchemaId, SchemaNode>) {
        let pairs: Vec<(SchemaId, Option<SchemaId>)> =
            nodes.iter().map(|(&id, n)| (id, n.parent_id)).collect();
        for (id, parent_id) in pairs {
            if let Some(pid) = parent_id
                && let Some(parent_node) = nodes.get_mut(&pid)
            {
                parent_node.children.push(id);
            }
        }
    }

    /// Phase 5 — Compute inheritance depth for each node.
    ///
    /// Root nodes (no in-batch parent) get depth 1. Nodes with an in-batch
    /// parent get `parent_depth + 1`. Nodes whose parent is a DB-fresh schema
    /// are treated as roots (depth 1) since we don't track depth for DB
    /// schemas.
    fn compute_depths(
        nodes: &mut HashMap<SchemaId, SchemaNode>,
        known_parents: &HashMap<SchemaId, Schema>,
    ) {
        // Build depth map via BFS from roots
        let mut depths = HashMap::with_capacity(nodes.len());

        // Identify roots: nodes with no parent or parent is DB-fresh
        let mut queue: VecDeque<SchemaId> = nodes
            .iter()
            .filter(|&(_, node)| {
                node.parent_id.is_none()
                    || node
                        .parent_id
                        .is_some_and(|pid| known_parents.contains_key(&pid))
            })
            .map(|(&id, _)| id)
            .collect();

        // BFS to compute depths
        for &root_id in &queue {
            depths.insert(root_id, NodeDepth::root());
        }

        while let Some(id) = queue.pop_front() {
            let current_depth =
                depths.get(&id).copied().unwrap_or(NodeDepth::root());
            let child_depth = current_depth.increment();

            let Some(node) = nodes.get(&id) else {
                continue;
            };

            for &child_id in &node.children {
                if let std::collections::hash_map::Entry::Vacant(e) =
                    depths.entry(child_id)
                {
                    e.insert(child_depth);
                    queue.push_back(child_id);
                }
            }
        }

        // Apply computed depths to nodes
        #[expect(
            clippy::iter_over_hash_type,
            reason = "depths is a worklist; HashMap iteration order does not \
                      affect correctness (we apply depth to each node \
                      independently)"
        )]
        for (id, depth) in depths {
            if let Some(node) = nodes.get_mut(&id) {
                node.depth = depth;
            }
        }
    }

    /// Phase 6 — Kahn's algorithm; returns `(order, roots)`.
    #[expect(
        clippy::iter_over_hash_type,
        reason = "in_degree is a worklist; HashMap insertion order does not \
                  affect correctness (we sort before enqueuing)"
    )]
    fn kahn_order(
        nodes: &HashMap<SchemaId, SchemaNode>,
    ) -> Result<KahnResult, SchemaError> {
        let mut in_degree: HashMap<SchemaId, usize> =
            HashMap::with_capacity(nodes.len());
        for (&id, node) in nodes {
            let deg = node
                .parent_id
                .map_or(0, |pid| usize::from(nodes.contains_key(&pid)));
            in_degree.insert(id, deg);
        }

        // Seed queue with zero-in-degree nodes sorted by name for
        // determinism.
        let mut initial: Vec<SchemaId> = in_degree
            .iter()
            .filter_map(|(&id, &deg)| deg.eq(&0).then_some(id))
            .collect();
        initial.sort_by(|a, b| {
            nodes
                .get(a)
                .map_or("", |n| n.name.as_ref())
                .cmp(nodes.get(b).map_or("", |n| n.name.as_ref()))
        });

        let mut queue: VecDeque<SchemaId> = initial.into();
        let mut order: Vec<SchemaId> = Vec::with_capacity(nodes.len());

        while let Some(id) = queue.pop_front() {
            order.push(id);
            Self::decrement_and_enqueue(
                Self::sorted_children(nodes, id),
                &mut in_degree,
                &mut queue,
            );
        }

        if order.len() != nodes.len() {
            return Err(SchemaError::Inheritance(
                super::error::SchemaInheritanceError::CircularInheritance {
                    name: "cycle detected during topological ordering".into(),
                },
            ));
        }

        let roots = order
            .iter()
            .copied()
            .filter(|id| {
                nodes.get(id).is_none_or(|n| {
                    n.parent_id.is_none_or(|pid| !nodes.contains_key(&pid))
                })
            })
            .collect();

        Ok((order, roots))
    }

    /// Decrement in-degree for each child; enqueue those that reach zero.
    fn decrement_and_enqueue(
        children: Vec<SchemaId>,
        in_degree: &mut HashMap<SchemaId, usize>,
        queue: &mut VecDeque<SchemaId>,
    ) {
        for child_id in children {
            let Some(deg) = in_degree.get_mut(&child_id) else {
                continue;
            };
            *deg = deg.saturating_sub(1);
            if *deg == 0 {
                queue.push_back(child_id);
            }
        }
    }

    /// Returns children of `id` sorted by name for deterministic output.
    fn sorted_children(
        nodes: &HashMap<SchemaId, SchemaNode>,
        id: SchemaId,
    ) -> Vec<SchemaId> {
        let mut children: Vec<SchemaId> =
            nodes.get(&id).map_or(&[][..], |n| n.children.as_slice()).to_vec();
        children.sort_by(|a, b| {
            nodes
                .get(a)
                .map_or("", |n| n.name.as_ref())
                .cmp(nodes.get(b).map_or("", |n| n.name.as_ref()))
        });
        children
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  CycleChecker — DFS helper (Phase 3)
// ─────────────────────────────────────────────────────────────────────────────

/// DFS cycle checker holding shared references to avoid >5 arguments.
struct CycleChecker<'graph> {
    nodes: &'graph HashMap<SchemaId, SchemaNode>,
    known_parents: &'graph HashMap<SchemaId, Schema>,
    id_to_name: &'graph HashMap<SchemaId, SchemaName>,
    visited: HashSet<SchemaId>,
    in_progress: HashSet<SchemaId>,
}

impl CycleChecker<'_> {
    /// Visit `id`, recursing into its in-batch parent if present.
    ///
    /// Only traverses in-batch nodes; DB-fresh parents are terminal.
    fn visit(&mut self, id: SchemaId) -> Result<(), SchemaError> {
        if self.visited.contains(&id) {
            return Ok(());
        }
        if !self.in_progress.insert(id) {
            let name = self
                .id_to_name
                .get(&id)
                .map_or_else(|| id.to_string(), ToString::to_string);
            return Err(SchemaError::Inheritance(
                super::error::SchemaInheritanceError::CircularInheritance {
                    name: name.into(),
                },
            ));
        }

        if let Some(node) = self.nodes.get(&id)
            && let Some(parent_id) = node.parent_id
            && !self.known_parents.contains_key(&parent_id)
        {
            self.visit(parent_id)?;
        }

        self.in_progress.remove(&id);
        self.visited.insert(id);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures before sub-modules for readability"
)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::{Schema, SchemaId},
        error::SchemaError,
    };

    mod fixtures {
        use super::*;

        pub fn simple_expanded(
            id: SchemaId,
            name: &str,
            extends: Option<&str>,
        ) -> (SchemaId, RawSchema, HashMap<PropertyName, Property>) {
            let mut json = serde_json::json!({
                "$version": "1.0",
                "properties": {}
            });
            if let Some(parent) = extends
                && let Some(obj) = json.as_object_mut()
            {
                obj.insert("extends".into(), serde_json::json!(parent));
            }
            let raw = serde_json::from_value::<RawSchema>(json)
                .expect("valid schema JSON")
                .with_name(name.into());
            (id, raw, HashMap::new())
        }
    }

    const PARENT_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0E01);
    const CHILD_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0E02);
    const ORPHAN_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0E03);

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions use assert! macros; standard test practice"
    )]
    mod build {
        use super::*;

        #[test]
        fn empty_input_returns_empty_tree() -> Result<(), SchemaError> {
            let tree = Extender::build(vec![], &HashMap::new())?;
            assert!(tree.nodes().is_empty());
            assert!(tree.roots().is_empty());
            Ok(())
        }

        #[test]
        fn single_root_schema() -> Result<(), SchemaError> {
            let id = SchemaId::from_uuid(PARENT_ID);
            let expanded = vec![fixtures::simple_expanded(id, "root", None)];
            let tree = Extender::build(expanded, &HashMap::new())?;
            assert_eq!(tree.nodes(), &[id]);
            assert_eq!(tree.roots(), &[id]);
            assert!(tree.get(id).is_some());
            Ok(())
        }

        #[test]
        fn parent_before_child_in_order() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);
            let expanded = vec![
                fixtures::simple_expanded(child_id, "child", Some("parent")),
                fixtures::simple_expanded(parent_id, "parent", None),
            ];
            let tree = Extender::build(expanded, &HashMap::new())?;
            let order = tree.nodes();
            let parent_pos = order
                .iter()
                .position(|&x| x == parent_id)
                .expect("parent in order");
            let child_pos = order
                .iter()
                .position(|&x| x == child_id)
                .expect("child in order");
            assert!(
                parent_pos < child_pos,
                "Parent must appear before child in topological order"
            );
            Ok(())
        }

        #[test]
        fn external_db_parent_is_accepted() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);

            let parent_schema = Schema::new(
                parent_id,
                SchemaName::try_new("parent")?,
                None,
                Vec::new(),
                HashMap::new(),
            );
            let mut known_parents = HashMap::new();
            known_parents.insert(parent_id, parent_schema);

            let expanded = vec![fixtures::simple_expanded(
                child_id,
                "child",
                Some("parent"),
            )];
            let tree = Extender::build(expanded, &known_parents)?;
            assert_eq!(tree.nodes(), &[child_id]);
            Ok(())
        }

        #[test]
        fn missing_parent_returns_error() {
            let child_id = SchemaId::from_uuid(CHILD_ID);
            let expanded = vec![fixtures::simple_expanded(
                child_id,
                "child",
                Some("nonexistent"),
            )];
            let result = Extender::build(expanded, &HashMap::new());
            assert!(
                matches!(
                    result,
                    Err(SchemaError::Inheritance(
                        crate::schema::error::SchemaInheritanceError::ParentNotFound { .. }
                    ))
                ),
                "Expected ParentNotFound, got: {result:?}"
            );
        }

        #[test]
        fn cycle_detection_returns_error() {
            let id = SchemaId::from_uuid(ORPHAN_ID);
            // Schema "self" extends "self" — a self-loop.
            let expanded =
                vec![fixtures::simple_expanded(id, "self", Some("self"))];
            let result = Extender::build(expanded, &HashMap::new());
            assert!(
                matches!(
                    result,
                    Err(SchemaError::Inheritance(
                        crate::schema::error::SchemaInheritanceError::CircularInheritance { .. }
                            | crate::schema::error::SchemaInheritanceError::ParentNotFound { .. }
                    ))
                ),
                "Expected cycle or missing parent error, got: {result:?}"
            );
        }

        /// GAP-001: Test multi-node circular inheritance (A→B→C→A).
        #[test]
        fn cycle_detection_multi_node_cycle() {
            const ID_A: Uuid =
                Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_1001);
            const ID_B: Uuid =
                Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_1002);
            const ID_C: Uuid =
                Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_1003);

            let id_a = SchemaId::from_uuid(ID_A);
            let id_b = SchemaId::from_uuid(ID_B);
            let id_c = SchemaId::from_uuid(ID_C);

            // A extends C, B extends A, C extends B → cycle!
            let expanded = vec![
                fixtures::simple_expanded(id_a, "a", Some("c")),
                fixtures::simple_expanded(id_b, "b", Some("a")),
                fixtures::simple_expanded(id_c, "c", Some("b")),
            ];

            let result = Extender::build(expanded, &HashMap::new());
            assert!(
                matches!(
                    result,
                    Err(SchemaError::Inheritance(
                        crate::schema::error::SchemaInheritanceError::CircularInheritance { .. }
                    ))
                ),
                "Should detect multi-node cycle, got: {result:?}"
            );
        }

        #[test]
        fn children_wired_on_in_batch_parent() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);
            let expanded = vec![
                fixtures::simple_expanded(parent_id, "parent", None),
                fixtures::simple_expanded(child_id, "child", Some("parent")),
            ];
            let tree = Extender::build(expanded, &HashMap::new())?;
            let parent_node = tree.get(parent_id).expect("parent node");
            assert!(
                parent_node.children.contains(&child_id),
                "Parent node should list child as a child"
            );
            Ok(())
        }
    }

    mod node_depth_tests {
        use super::*;

        #[test]
        fn node_depth_root() {
            let depth = NodeDepth::root();
            assert_eq!(depth.get(), 1);
        }

        #[test]
        fn node_depth_increment() {
            let depth = NodeDepth::root().increment();
            assert_eq!(depth.get(), 2);
            let depth = depth.increment();
            assert_eq!(depth.get(), 3);
        }

        #[test]
        fn node_depth_exceeds() {
            let depth = NodeDepth::root().increment().increment(); // 3
            assert!(!depth.exceeds(10));
            assert!(!depth.exceeds(3));
            assert!(depth.exceeds(2));
        }

        #[test]
        fn node_depth_display() {
            let depth = NodeDepth::root().increment().increment();
            assert_eq!(format!("{depth}"), "3");
        }

        #[test]
        fn node_depth_saturates_on_increment() {
            // Construct depth manually at max value
            let max_depth = NodeDepth(usize::MAX);
            let result = max_depth.increment();
            assert_eq!(result.get(), usize::MAX);
        }
    }
}
