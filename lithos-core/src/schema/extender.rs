//! `Extender` — builds a topologically-ordered [`SchemaTree`] from
//! dereferenced schemas and previously-resolved (DB-fresh) parents.
//!
//! # Pipeline position
//!
//! ```text
//! Dereferencer → Vec<(SchemaId, DereferencedSchema)>
//! Extender          ← here
//! → SchemaTree
//! Resolver
//! ```
//!
//! # Design
//!
//! The `Extender` takes:
//! - Stale schemas already dereferenced by [`Dereferencer`] (their `$ref`s
//!   resolved).
//! - A map of fresh (non-stale) schemas loaded from the DB — these may act as
//!   parents.
//!
//! It produces a [`SchemaTree`] whose nodes are in **topological order**
//! (parents before children) so the downstream [`Resolver`] can walk the tree
//! once without back-tracking.
//!
//! [`Dereferencer`]: super::dereferencer::Dereferencer
//! [`Resolver`]: super::resolver::Resolver

use std::collections::{HashMap, HashSet, VecDeque};

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    dereferencer::DereferencedSchema,
    error::SchemaError,
    property::Property,
};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaNode
// ─────────────────────────────────────────────────────────────────────────────

/// A single node in the inheritance tree, ready for property merging.
#[derive(Debug)]
pub(crate) struct SchemaNode {
    /// Schema name string.
    pub name: Box<str>,
    /// Own properties (from `DereferencedSchema`), sorted by name.
    pub own_properties: Vec<Property>,
    /// Property names inherited from the parent that this schema excludes.
    pub excludes: Vec<Box<str>>,
    /// Parent schema identifier, if any.
    pub parent_id: Option<SchemaId>,
    /// Children of this node (populated during `build`).
    pub children: Vec<SchemaId>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaTree
// ─────────────────────────────────────────────────────────────────────────────

/// A topologically-ordered inheritance tree of schemas.
///
/// - `nodes` provides O(1) lookup by `SchemaId`.
/// - `order` is the topological order (`roots` first, leaves last) produced by
///   Kahn's algorithm.
/// - `roots` contains schemas whose `parent_id` is `None` (or whose parent is a
///   DB-fresh known parent rather than an in-batch node).
#[derive(Debug)]
pub(crate) struct SchemaTree {
    /// IDs of root schemas (no in-batch parent).
    roots: Vec<SchemaId>,
    /// All nodes indexed by `SchemaId`.
    nodes: HashMap<SchemaId, SchemaNode>,
    /// Schema IDs in topological order (parents before children).
    order: Vec<SchemaId>,
}

impl SchemaTree {
    /// Returns schema IDs in topological order (parents first).
    ///
    /// Suitable for a single linear walk by [`Resolver`].
    ///
    /// [`Resolver`]: super::resolver::Resolver
    #[inline]
    #[must_use]
    pub(crate) fn nodes(&self) -> &[SchemaId] {
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
type NameIndexes = (HashMap<Box<str>, SchemaId>, HashMap<SchemaId, Box<str>>);

/// Type alias for the `(order, roots)` pair returned by Kahn's algorithm.
type KahnResult = (Vec<SchemaId>, Vec<SchemaId>);

/// Builds a [`SchemaTree`] from dereferenced schemas.
pub(crate) struct Extender;

impl Extender {
    /// Build a [`SchemaTree`] from stale, dereferenced schemas.
    ///
    /// `derefed` — schemas processed by the [`Dereferencer`].
    /// `known_parents` — fresh schemas pre-loaded from the DB; their IDs are
    /// valid parent targets.
    ///
    /// # Errors
    ///
    /// - [`SchemaError::CircularInheritance`] — a cycle was detected.
    /// - [`SchemaError::ParentNotFound`] — a `extends` name refers to a schema
    ///   that is neither in `derefed` nor in `known_parents`.
    /// - [`SchemaError::AlreadyExists`] — two schemas share the same name.
    ///
    /// [`Dereferencer`]: super::dereferencer::Dereferencer
    #[inline]
    pub(crate) fn build(
        derefed: Vec<(SchemaId, DereferencedSchema)>,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<SchemaTree, SchemaError> {
        // Phase 1: build name ↔ id indexes.
        let (name_to_id, id_to_name) =
            Self::build_name_indexes(&derefed, known_parents)?;

        // Phase 2: build node map with resolved parent IDs.
        let mut nodes = Self::build_nodes(derefed, &name_to_id)?;

        // Phase 3: DFS cycle detection.
        Self::detect_cycles(&nodes, known_parents, &id_to_name)?;

        // Phase 4: populate children lists.
        Self::populate_children(&mut nodes);

        // Phase 5: Kahn's topological ordering.
        let (order, roots) = Self::kahn_order(&nodes)?;

        Ok(SchemaTree {
            roots,
            nodes,
            order,
        })
    }

    /// Phase 1 — build owned `name → id` and `id → name` indexes.
    ///
    /// Uses `Box<str>` keys so `derefed` can be consumed in Phase 2 without
    /// lifetime issues.  `Box<str>: Borrow<str>` so `HashMap::get(&str)` works.
    fn build_name_indexes(
        derefed: &[(SchemaId, DereferencedSchema)],
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<NameIndexes, SchemaError> {
        let cap = derefed.len();
        let mut name_to_id: HashMap<Box<str>, SchemaId> =
            HashMap::with_capacity(cap.saturating_add(known_parents.len()));
        let mut id_to_name: HashMap<SchemaId, Box<str>> =
            HashMap::with_capacity(cap);

        // Iterating over a HashMap; order doesn't matter here (all entries
        // are inserted unconditionally).
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Insertion order of DB-fresh parents is irrelevant; all \
                      entries are written to name_to_id unconditionally"
        )]
        for (id, schema) in known_parents {
            name_to_id.insert(schema.name().as_str().into(), *id);
        }
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics on &(SchemaId, DereferencedSchema) \
                      tuples; explicit derefs would add noise without clarity"
        )]
        for (id, deref) in derefed {
            if name_to_id.insert(deref.name.clone(), *id).is_some()
                && !known_parents
                    .values()
                    .any(|s| s.name().as_str() == deref.name.as_ref())
            {
                return Err(SchemaError::AlreadyExists(deref.name.to_string()));
            }
            id_to_name.insert(*id, deref.name.clone());
        }
        Ok((name_to_id, id_to_name))
    }

    /// Phase 2 — build the node map, resolving each `extends` name to a
    /// `SchemaId`.
    fn build_nodes(
        derefed: Vec<(SchemaId, DereferencedSchema)>,
        name_to_id: &HashMap<Box<str>, SchemaId>,
    ) -> Result<HashMap<SchemaId, SchemaNode>, SchemaError> {
        let mut nodes = HashMap::with_capacity(derefed.len());
        for (id, deref) in derefed {
            let parent_id = Self::resolve_parent(&deref, name_to_id)?;
            nodes.insert(id, SchemaNode {
                name: deref.name,
                own_properties: deref.properties,
                excludes: deref.excludes,
                parent_id,
                children: Vec::new(),
            });
        }
        Ok(nodes)
    }

    /// Resolve the optional `extends` string to a `SchemaId`.
    fn resolve_parent(
        deref: &DereferencedSchema,
        name_to_id: &HashMap<Box<str>, SchemaId>,
    ) -> Result<Option<SchemaId>, SchemaError> {
        let Some(parent_name) = deref.extends.as_ref() else {
            return Ok(None);
        };
        SchemaName::validate(parent_name.as_ref())?;
        // `Box<str>: Borrow<str>` so `.get(&str)` works here.
        name_to_id
            .get(parent_name.as_ref())
            .copied()
            .map(Some)
            .ok_or_else(|| SchemaError::ParentNotFound(parent_name.to_string()))
    }

    /// Phase 3 — DFS cycle detection over in-batch nodes.
    fn detect_cycles(
        nodes: &HashMap<SchemaId, SchemaNode>,
        known_parents: &HashMap<SchemaId, Schema>,
        id_to_name: &HashMap<SchemaId, Box<str>>,
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

    /// Phase 5 — Kahn's algorithm; returns `(order, roots)`.
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
                .map_or("", |n| &n.name)
                .cmp(nodes.get(b).map_or("", |n| &n.name))
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
            return Err(SchemaError::CircularInheritance(
                "cycle detected during topological ordering".into(),
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
                .map_or("", |n| &n.name)
                .cmp(nodes.get(b).map_or("", |n| &n.name))
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
    id_to_name: &'graph HashMap<SchemaId, Box<str>>,
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
            return Err(SchemaError::CircularInheritance(name));
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
        aggregate::{Schema, SchemaId, SchemaName},
        error::SchemaError,
    };

    mod fixtures {
        use super::*;
        use crate::schema::dereferencer::DereferencedSchema;

        pub fn simple_derefed(
            id: SchemaId,
            name: &str,
            extends: Option<&str>,
        ) -> (SchemaId, DereferencedSchema) {
            (id, DereferencedSchema {
                name: name.into(),
                extends: extends.map(Into::into),
                excludes: Vec::new(),
                properties: Vec::new(),
            })
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
            let derefed = vec![fixtures::simple_derefed(id, "root", None)];
            let tree = Extender::build(derefed, &HashMap::new())?;
            assert_eq!(tree.nodes(), &[id]);
            assert_eq!(tree.roots(), &[id]);
            assert!(tree.get(id).is_some());
            Ok(())
        }

        #[test]
        fn parent_before_child_in_order() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);
            let derefed = vec![
                fixtures::simple_derefed(child_id, "child", Some("parent")),
                fixtures::simple_derefed(parent_id, "parent", None),
            ];
            let tree = Extender::build(derefed, &HashMap::new())?;
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

            let parent_schema = Schema::reconstruct(
                parent_id,
                SchemaName::new("parent")?,
                Vec::new(),
            );
            let mut known_parents = HashMap::new();
            known_parents.insert(parent_id, parent_schema);

            let derefed = vec![fixtures::simple_derefed(
                child_id,
                "child",
                Some("parent"),
            )];
            let tree = Extender::build(derefed, &known_parents)?;
            assert_eq!(tree.nodes(), &[child_id]);
            Ok(())
        }

        #[test]
        fn missing_parent_returns_error() {
            let child_id = SchemaId::from_uuid(CHILD_ID);
            let derefed = vec![fixtures::simple_derefed(
                child_id,
                "child",
                Some("nonexistent"),
            )];
            let result = Extender::build(derefed, &HashMap::new());
            assert!(
                matches!(result, Err(SchemaError::ParentNotFound(_))),
                "Expected ParentNotFound, got: {result:?}"
            );
        }

        #[test]
        fn cycle_detection_returns_error() {
            use crate::schema::dereferencer::DereferencedSchema;
            let id = SchemaId::from_uuid(ORPHAN_ID);
            // Schema "self" extends "self" — a self-loop.
            let derefed = vec![(id, DereferencedSchema {
                name: "self".into(),
                extends: Some("self".into()),
                excludes: Vec::new(),
                properties: Vec::new(),
            })];
            let result = Extender::build(derefed, &HashMap::new());
            assert!(
                matches!(
                    result,
                    Err(SchemaError::CircularInheritance(_)
                        | SchemaError::ParentNotFound(_))
                ),
                "Expected cycle or missing parent error, got: {result:?}"
            );
        }

        #[test]
        fn children_wired_on_in_batch_parent() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);
            let derefed = vec![
                fixtures::simple_derefed(parent_id, "parent", None),
                fixtures::simple_derefed(child_id, "child", Some("parent")),
            ];
            let tree = Extender::build(derefed, &HashMap::new())?;
            let parent_node = tree.get(parent_id).expect("parent node");
            assert!(
                parent_node.children.contains(&child_id),
                "Parent node should list child as a child"
            );
            Ok(())
        }
    }
}
