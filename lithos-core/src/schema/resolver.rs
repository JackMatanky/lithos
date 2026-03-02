//! `Resolver` — assembles fully-resolved [`Schema`] entities from a
//! [`SchemaTree`].
//!
//! # Pipeline position
//!
//! ```text
//! Extender → SchemaTree
//! Resolver        ← here
//! → Vec<Schema>
//! ```
//!
//! # Design
//!
//! `Resolver` is a stateless unit struct. Its single public method,
//! [`Resolver::resolve`], walks the [`SchemaTree`] in topological order
//! (parents before children) and merges properties using a two-pointer sorted
//! merge.
//!
//! `merge_properties` is a **private** method to prevent callers from
//! bypassing the correct pipeline.

use std::{collections::HashMap, sync::Arc};

use super::{
    aggregate::{Schema, SchemaId, SchemaName},
    error::SchemaError,
    extender::SchemaTree,
    property::{Property, PropertyName},
};

/// Maximum allowed inheritance depth to prevent infinite loops.
/// If a schema chain exceeds this depth, resolution fails with
/// [`SchemaError::InheritanceDepthExceeded`].
const INHERITANCE_MAX_DEPTH: usize = 10;

// ─────────────────────────────────────────────────────────────────────────────
//  Resolver
// ─────────────────────────────────────────────────────────────────────────────

/// Assembles fully-resolved [`Schema`] entities from a [`SchemaTree`].
///
/// Stateless: all resolution state is threaded through the arguments.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `SchemaService` instead.
#[doc(hidden)]
#[non_exhaustive]
pub struct Resolver;

impl Resolver {
    /// Resolve all schemas in `tree`, returning them as [`Schema`] values.
    ///
    /// Walks `tree` in topological order (parents before children).  For each
    /// node:
    ///
    /// 1. Looks up the parent's resolved properties from `resolved_cache` (an
    ///    in-batch parent resolved earlier in the walk), from `known_parents`
    ///    (a DB-fresh parent), or uses an empty slice for root schemas.
    /// 2. Calls private `merge_properties` to produce the final sorted property
    ///    list.
    /// 3. Constructs a [`Schema`] via [`Schema::new`].
    /// 4. Caches the result for use by downstream children.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError`] if schema construction fails (e.g. name
    /// validation error in a node name that somehow passed earlier
    /// validation — should be unreachable in practice).
    #[inline]
    /// **Internal API**: Public for benchmarking only.
    #[doc(hidden)]
    pub fn resolve(
        tree: &SchemaTree,
        known_parents: &HashMap<SchemaId, Schema>,
    ) -> Result<Vec<Schema>, SchemaError> {
        let order = tree.nodes();
        let mut resolved_cache: HashMap<SchemaId, Schema> =
            HashMap::with_capacity(order.len());
        let mut results: Vec<Schema> = Vec::with_capacity(order.len());

        for &id in order {
            let node = tree.get(id).ok_or_else(|| {
                SchemaError::NotFound(format!(
                    "SchemaTree node missing for id {id}"
                ))
            })?;

            // E-03: Use depth computed by Extender
            // The Extender already computed depth correctly via BFS, accounting
            // for DB-fresh parents. We convert NodeDepth -> usize
            // for the limit check.
            let depth: usize = node.depth.into();

            // Check against maximum allowed depth
            if depth > INHERITANCE_MAX_DEPTH {
                return Err(SchemaError::InheritanceDepthExceeded(depth));
            }

            // Obtain parent's resolved properties.
            let parent_props: Vec<Arc<Property>> =
                if let Some(parent_id) = node.parent_id {
                    resolved_cache
                        .get(&parent_id)
                        .or_else(|| known_parents.get(&parent_id))
                        .map(|schema| schema.properties().cloned().collect())
                        .unwrap_or_default()
                } else {
                    vec![]
                };

            // Convert properties to Arc<Property> for zero-allocation sharing.
            // Parent properties are already Arc-wrapped from the resolved
            // cache, so when a child inherits them, we only clone
            // the Arc (cheap pointer copy), not the underlying
            // Property data.
            let own_props_arc: Vec<Arc<Property>> =
                node.properties.iter().map(|p| Arc::new(p.clone())).collect();

            let merged = Self::merge_properties(
                &parent_props,
                &own_props_arc,
                &node.excludes,
            );

            let name = SchemaName::new(&node.name)?;
            let schema = Schema::new(id, name, node.parent_id, merged)?;

            resolved_cache.insert(id, schema.clone());
            results.push(schema);
        }

        Ok(results)
    }

    /// Merge parent and own properties into a single sorted vector, applying
    /// child overrides and excludes.
    ///
    /// Both `parent` and `own` must be sorted by property name (guaranteed by
    /// [`Dereferencer`]).  The merge is performed with a two-pointer walk:
    ///
    /// - Same-named entries: child's version wins (override).
    /// - Parent entries whose name is in `excludes`: dropped.
    /// - Remaining parent + own entries: interleaved in name order.
    ///
    /// [`Dereferencer`]: super::dereferencer::Dereferencer
    fn merge_properties(
        parent: &[Arc<Property>],
        own: &[Arc<Property>],
        excludes: &[Box<str>],
    ) -> Vec<Property> {
        let capacity = parent.len().saturating_add(own.len());
        let mut result = Vec::with_capacity(capacity);
        let mut p_iter = parent.iter().peekable();
        let mut c_iter = own.iter().peekable();

        loop {
            use std::cmp::Ordering;
            match (p_iter.peek(), c_iter.peek()) {
                (Some(&p), Some(&c)) => {
                    match p.name().as_str().cmp(c.name().as_str()) {
                        Ordering::Less => {
                            Self::push_unless_excluded(
                                &mut result,
                                p,
                                excludes,
                            );
                            p_iter.next();
                        }
                        Ordering::Greater => {
                            // Clone the Arc's inner Property
                            result.push((**c).clone());
                            c_iter.next();
                        }
                        Ordering::Equal => {
                            // Child overrides parent
                            result.push((**c).clone());
                            p_iter.next();
                            c_iter.next();
                        }
                    }
                }
                (Some(&p), None) => {
                    Self::push_unless_excluded(&mut result, p, excludes);
                    p_iter.next();
                }
                (None, Some(&c)) => {
                    result.push((**c).clone());
                    c_iter.next();
                }
                (None, None) => break,
            }
        }

        result
    }

    #[inline]
    fn push_unless_excluded(
        result: &mut Vec<Property>,
        prop: &Arc<Property>,
        excludes: &[Box<str>],
    ) {
        if !Self::is_excluded(prop.name(), excludes) {
            // Clone the Arc's inner Property
            result.push((**prop).clone());
        }
    }

    #[inline]
    fn is_excluded(name: &PropertyName, excludes: &[Box<str>]) -> bool {
        excludes.iter().any(|e| e.as_ref() == name.as_str())
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
        dereferencer::DereferencedSchema,
        error::SchemaError,
        extender::Extender,
        property::{
            Multiplicity, Optionality, Property, PropertyId, PropertyName,
        },
        property_spec::{BoolSpec, PropertySpec},
    };

    mod fixtures {
        use super::*;

        pub fn bool_property(name: &str) -> Result<Property, SchemaError> {
            Ok(Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::new(name)?,
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            ))
        }

        pub fn simple_derefed(
            id: SchemaId,
            name: &str,
            extends: Option<&str>,
            props: Vec<Property>,
        ) -> (SchemaId, DereferencedSchema) {
            (id, DereferencedSchema {
                name: name.into(),
                extends: extends.map(Into::into),
                excludes: Vec::new(),
                properties: props,
            })
        }

        pub fn derefed_with_excludes(
            id: SchemaId,
            name: &str,
            extends: Option<&str>,
            excludes: Vec<Box<str>>,
        ) -> (SchemaId, DereferencedSchema) {
            (id, DereferencedSchema {
                name: name.into(),
                extends: extends.map(Into::into),
                excludes,
                properties: Vec::new(),
            })
        }
    }

    const PARENT_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0D01);
    const CHILD_ID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0D02);

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions use assert! macros; standard test practice"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "Test indexing into results whose length is asserted; bounds \
                  guaranteed by test setup"
    )]
    mod resolve {
        use super::*;

        #[test]
        fn empty_tree_returns_empty() -> Result<(), SchemaError> {
            let tree = Extender::build(vec![], &HashMap::new())?;
            let result = Resolver::resolve(&tree, &HashMap::new())?;
            assert!(result.is_empty());
            Ok(())
        }

        #[test]
        fn single_root_schema_no_parent() -> Result<(), SchemaError> {
            let id = SchemaId::from_uuid(PARENT_ID);
            let prop = fixtures::bool_property("flag")?;
            let derefed =
                vec![fixtures::simple_derefed(id, "root", None, vec![
                    prop.clone(),
                ])];
            let tree = Extender::build(derefed, &HashMap::new())?;
            let result = Resolver::resolve(&tree, &HashMap::new())?;

            assert_eq!(result.len(), 1);
            let schema = &result[0];
            assert_eq!(schema.name().as_str(), "root");
            assert_eq!(schema.properties().count(), 1);
            Ok(())
        }

        #[test]
        fn child_inherits_parent_properties() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);

            let parent_prop = fixtures::bool_property("from-parent")?;
            let child_prop = fixtures::bool_property("from-child")?;

            let derefed = vec![
                fixtures::simple_derefed(parent_id, "parent", None, vec![
                    parent_prop,
                ]),
                fixtures::simple_derefed(
                    child_id,
                    "child",
                    Some("parent"),
                    vec![child_prop],
                ),
            ];
            let tree = Extender::build(derefed, &HashMap::new())?;
            let result = Resolver::resolve(&tree, &HashMap::new())?;

            let child = result
                .iter()
                .find(|s| s.name().as_str() == "child")
                .expect("child schema in result");

            let prop_names: Vec<&str> =
                child.properties().map(|p| p.name().as_str()).collect();
            assert!(
                prop_names.contains(&"from-parent"),
                "Child should inherit parent's property; got: {prop_names:?}"
            );
            assert!(
                prop_names.contains(&"from-child"),
                "Child should have own property; got: {prop_names:?}"
            );
            Ok(())
        }

        #[test]
        fn child_override_beats_parent() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);

            // Parent has "shared" as Required, child overrides as Optional.
            let parent_prop = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::new("shared")?,
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
            let child_prop = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::new("shared")?,
                Optionality::Optional,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );

            let derefed = vec![
                fixtures::simple_derefed(parent_id, "parent", None, vec![
                    parent_prop,
                ]),
                fixtures::simple_derefed(
                    child_id,
                    "child",
                    Some("parent"),
                    vec![child_prop],
                ),
            ];
            let tree = Extender::build(derefed, &HashMap::new())?;
            let result = Resolver::resolve(&tree, &HashMap::new())?;

            let child = result
                .iter()
                .find(|s| s.name().as_str() == "child")
                .expect("child in result");
            let prop_name = PropertyName::new("shared")?;
            let shared = child.get(&prop_name).expect("shared property");
            assert_eq!(
                shared.optionality(),
                Optionality::Optional,
                "Child should override parent's optionality"
            );
            Ok(())
        }

        #[test]
        fn child_excludes_parent_property() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);

            let parent_prop = fixtures::bool_property("excluded")?;

            let derefed = vec![
                fixtures::simple_derefed(parent_id, "parent", None, vec![
                    parent_prop,
                ]),
                fixtures::derefed_with_excludes(
                    child_id,
                    "child",
                    Some("parent"),
                    vec!["excluded".into()],
                ),
            ];
            let tree = Extender::build(derefed, &HashMap::new())?;
            let result = Resolver::resolve(&tree, &HashMap::new())?;

            let child = result
                .iter()
                .find(|s| s.name().as_str() == "child")
                .expect("child in result");
            let excl_name = PropertyName::new("excluded")?;
            assert!(
                !child.has(&excl_name),
                "Excluded property should be absent from child"
            );
            Ok(())
        }

        #[test]
        fn db_fresh_parent_properties_inherited() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);

            let parent_prop = fixtures::bool_property("db-prop")?;
            let parent_schema = Schema::reconstruct(
                parent_id,
                SchemaName::new("parent")?,
                None,
                vec![parent_prop],
            );
            let mut known_parents = HashMap::new();
            known_parents.insert(parent_id, parent_schema);

            let derefed = vec![fixtures::simple_derefed(
                child_id,
                "child",
                Some("parent"),
                vec![],
            )];
            let tree = Extender::build(derefed, &known_parents)?;
            let result = Resolver::resolve(&tree, &known_parents)?;

            let child = result
                .iter()
                .find(|s| s.name().as_str() == "child")
                .expect("child in result");
            let prop_name = PropertyName::new("db-prop")?;
            assert!(
                child.has(&prop_name),
                "Child should inherit DB-fresh parent property"
            );
            Ok(())
        }

        // E-03: Inheritance depth limit tests

        /// Test that `INHERITANCE_MAX_DEPTH` constant has expected value.
        #[test]
        fn inheritance_max_depth_constant_value() {
            const DEPTH: usize = super::INHERITANCE_MAX_DEPTH;
            assert_eq!(DEPTH, 10, "INHERITANCE_MAX_DEPTH should be 10");
        }

        /// Test that `InheritanceDepthExceeded` error can be constructed.
        #[test]
        fn inheritance_depth_error_constructs() {
            let error = SchemaError::InheritanceDepthExceeded(101);
            let error_str = format!("{error}");
            assert!(
                error_str.contains("Inheritance depth exceeded"),
                "Error message should mention depth exceeded"
            );
            assert!(
                error_str.contains("101"),
                "Error message should include the depth value"
            );
        }

        /// GAP-002: Test that inheritance depth > 10 fails.
        #[test]
        fn inheritance_depth_limit_exceeded() {
            use uuid::Uuid;

            use crate::schema::{
                dereferencer::DereferencedSchema, extender::Extender,
            };

            // Create a chain of 11 schemas: root → s1 → s2 → ... → s10
            // Depth 11 should exceed MAX_DEPTH=10
            const BASE: u128 = 0x018C_0000_0000_7000_8000_0000_0000_2000;

            let mut derefed = Vec::new();
            let ids: Vec<_> = (0..11)
                .map(|i| SchemaId::from_uuid(Uuid::from_u128(BASE + i)))
                .collect();

            // Root (depth 1)
            derefed.push((ids[0], DereferencedSchema {
                name: "root".into(),
                extends: None,
                excludes: Vec::new(),
                properties: Vec::new(),
            }));

            // Chain: s1 extends root, s2 extends s1, ..., s10 extends s9
            for (i, &id) in ids.iter().enumerate().skip(1) {
                derefed.push((id, DereferencedSchema {
                    name: format!("s{i}").into(),
                    extends: Some(if i == 1 {
                        "root".into()
                    } else {
                        format!("s{}", i - 1).into()
                    }),
                    excludes: Vec::new(),
                    properties: Vec::new(),
                }));
            }

            // Build tree and resolve
            let tree = Extender::build(derefed, &HashMap::new())
                .expect("Tree building should succeed");
            let result = Resolver::resolve(&tree, &HashMap::new());

            assert!(
                matches!(result, Err(SchemaError::InheritanceDepthExceeded(_))),
                "Should reject depth > 10, got: {result:?}"
            );

            if let Err(SchemaError::InheritanceDepthExceeded(depth)) = result {
                assert_eq!(depth, 11, "Error should report depth 11");
            }
        }
    }
}
