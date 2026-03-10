//! `Resolver` — assembles fully-resolved [`StoredSchema`] entities from a
//! [`SchemaTree`].
//!
//! # Pipeline position
//!
//! ```text
//! Extender → SchemaTree
//! Resolver        ← here
//! → Vec<StoredSchema>
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

use std::collections::HashMap;

use super::{
    aggregate::{SchemaId, SchemaName},
    error::SchemaError,
    extender::SchemaTree,
    property::{Multiplicity, Optionality, Property, PropertyName},
    storage::{StoredProperty, StoredSchema},
};

/// Maximum allowed inheritance depth to prevent infinite loops.
/// If a schema chain exceeds this depth, resolution fails with
/// [`SchemaError::InheritanceDepthExceeded`].
const INHERITANCE_MAX_DEPTH: usize = 10;

// ─────────────────────────────────────────────────────────────────────────────
//  Resolver
// ─────────────────────────────────────────────────────────────────────────────

/// Assembles fully-resolved [`StoredSchema`] entities from a [`SchemaTree`].
///
/// Stateless: all resolution state is threaded through the arguments.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Loader` instead.
#[doc(hidden)]
#[non_exhaustive]
pub struct Resolver;

impl Resolver {
    /// Resolve all schemas in `tree`, returning them as [`StoredSchema`]
    /// values.
    ///
    /// Walks `tree` in topological order (parents before children).  For each
    /// node:
    ///
    /// 1. Looks up the parent's resolved properties from `resolved_cache` (an
    ///    in-batch parent resolved earlier in the walk), from `known_parents`
    ///    (a DB-fresh parent), or uses an empty slice for root schemas.
    /// 2. Calls private `merge_properties` to produce the final sorted property
    ///    list.
    /// 3. Constructs a [`StoredSchema`] directly.
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
        known_parents: &HashMap<SchemaId, StoredSchema>,
    ) -> Result<Vec<StoredSchema>, SchemaError> {
        let order = tree.nodes();
        let mut resolved_cache: HashMap<SchemaId, StoredSchema> =
            HashMap::with_capacity(order.len());
        let mut results: Vec<StoredSchema> = Vec::with_capacity(order.len());

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
            let parent_props: Vec<Property> =
                if let Some(parent_id) = node.parent_id {
                    resolved_cache
                        .get(&parent_id)
                        .or_else(|| known_parents.get(&parent_id))
                        .map(|stored| {
                            stored
                                .properties
                                .iter()
                                .map(Self::stored_to_property)
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .transpose()?
                        .unwrap_or_else(|| {
                            // This should not happen if Extender worked
                            // correctly - all parents should be in
                            // resolved_cache or known_parents
                            tracing::warn!(
                                schema_id = %id,
                                parent_id = %parent_id,
                                "Parent schema not found in \
                                 resolved_cache or known_parents, using \
                                 empty properties. This may indicate a bug \
                                 in Extender or missing parent in database."
                            );
                            Vec::new()
                        })
                } else {
                    vec![]
                };

            let merged = Self::merge_properties(
                &parent_props,
                &node.properties,
                &node.excludes,
            );

            // Validate name (should always succeed since it passed earlier
            // validation)
            let _name_check = SchemaName::try_new(&node.name)?;

            // Build StoredSchema directly
            let stored_properties: Vec<StoredProperty> = merged
                .into_iter()
                .map(|p| StoredProperty {
                    id: p.id(),
                    name: p.name().as_str().into(),
                    required: p.optionality() == Optionality::Required,
                    multi: p.multiplicity() == Multiplicity::Many,
                    spec: p.spec().clone(),
                })
                .collect();

            let stored = StoredSchema {
                id,
                name: node.name.clone(),
                parent_id: node.parent_id,
                properties: stored_properties,
            };

            resolved_cache.insert(id, stored.clone());
            results.push(stored);
        }

        Ok(results)
    }

    /// Convert a `StoredProperty` to a `Property`.
    ///
    /// Used when retrieving parent properties from the cache.
    fn stored_to_property(
        sp: &StoredProperty,
    ) -> Result<Property, SchemaError> {
        let name = PropertyName::try_new(&sp.name)?;
        let optionality = Optionality::from(sp.required);
        let multiplicity = Multiplicity::from(sp.multi);
        Ok(Property::new(
            sp.id,
            name,
            optionality,
            multiplicity,
            sp.spec.clone(),
        ))
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
        parent: &[Property],
        own: &[Property],
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
                            result.push(c.clone());
                            c_iter.next();
                        }
                        Ordering::Equal => {
                            // Child overrides parent
                            result.push(c.clone());
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
                    result.push(c.clone());
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
        prop: &Property,
        excludes: &[Box<str>],
    ) {
        if !Self::is_excluded(prop.name(), excludes) {
            result.push(prop.clone());
        }
    }

    #[inline]
    fn is_excluded(name: &PropertyName, excludes: &[Box<str>]) -> bool {
        excludes.iter().any(|e| e.as_ref() == name.as_str())
    }

    /// Incrementally resolve affected properties in a schema when PropertyBank
    /// changes.
    ///
    /// This is a performance optimization for the case where only PropertyBank
    /// properties have changed, not the schema file itself. Instead of
    /// re-resolving the entire schema from scratch, this method updates only
    /// the properties that reference changed bank properties.
    ///
    /// # Arguments
    ///
    /// * `schema` - The existing resolved schema to update
    /// * `affected_properties` - Names of properties in this schema that
    ///   reference changed bank properties
    /// * `bank` - The updated PropertyBank with new property definitions
    ///
    /// # Errors
    ///
    /// Returns [`SchemaError`] if any property lookup fails or if the property
    /// is not found in the bank (indicates database inconsistency).
    ///
    /// **Internal API**: Public for benchmarking only.
    #[doc(hidden)]
    #[inline]
    pub fn resolve_affected_properties(
        schema: &StoredSchema,
        affected_properties: &[PropertyName],
        bank: &super::bank::PropertyBank,
    ) -> Result<StoredSchema, SchemaError> {
        if affected_properties.is_empty() {
            return Ok(schema.clone());
        }

        // Build a set for O(1) lookup
        let affected_set: std::collections::HashSet<&PropertyName> =
            affected_properties.iter().collect();

        // Update affected properties with new definitions from bank
        let updated_properties: Result<Vec<StoredProperty>, SchemaError> =
            schema
                .properties
                .iter()
                .map(|stored_prop| {
                    let prop_name = PropertyName::try_new(&stored_prop.name)?;

                    // If this property is affected, look up new definition
                    // from bank
                    if affected_set.contains(&prop_name) {
                        let bank_prop =
                            bank.get(&prop_name).ok_or_else(|| {
                                SchemaError::PropertyRefNotFound(format!(
                                    "property_bank#/{}",
                                    stored_prop.name
                                ))
                            })?;

                        // Update the spec from the bank, keep other fields
                        // (required, multi) Properties in schemas can override
                        // optionality/multiplicity
                        Ok(StoredProperty {
                            id: bank_prop.id(),
                            name: stored_prop.name.clone(),
                            required: stored_prop.required,
                            multi: stored_prop.multi,
                            spec: bank_prop.spec().clone(),
                        })
                    } else {
                        // Property not affected, keep as-is
                        Ok(stored_prop.clone())
                    }
                })
                .collect();

        Ok(StoredSchema {
            id: schema.id,
            name: schema.name.clone(),
            parent_id: schema.parent_id,
            properties: updated_properties?,
        })
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
        aggregate::SchemaId,
        dereferencer::DereferencedSchema,
        error::SchemaError,
        extender::Extender,
        property::{
            Multiplicity, Optionality, Property, PropertyId, PropertyName,
        },
        property_spec::{BoolSpec, PropertySpec},
        storage::StoredSchema,
    };

    mod fixtures {
        use super::*;

        pub fn bool_property(name: &str) -> Result<Property, SchemaError> {
            Ok(Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::try_new(name)?,
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
            let stored = &result[0];
            assert_eq!(stored.name.as_ref(), "root");
            assert_eq!(stored.properties.len(), 1);
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
                .find(|s| s.name.as_ref() == "child")
                .expect("child schema in result");

            let prop_names: Vec<&str> =
                child.properties.iter().map(|p| p.name.as_ref()).collect();
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
                PropertyName::try_new("shared")?,
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
            let child_prop = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::try_new("shared")?,
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
                .find(|s| s.name.as_ref() == "child")
                .expect("child in result");
            let shared = child
                .properties
                .iter()
                .find(|p| p.name.as_ref() == "shared")
                .expect("shared property");
            assert!(
                !shared.required,
                "Child should override parent's optionality (should be \
                 optional/not required)"
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
                .find(|s| s.name.as_ref() == "child")
                .expect("child in result");
            let has_excluded =
                child.properties.iter().any(|p| p.name.as_ref() == "excluded");
            assert!(
                !has_excluded,
                "Excluded property should be absent from child"
            );
            Ok(())
        }

        #[test]
        fn db_fresh_parent_properties_inherited() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(PARENT_ID);
            let child_id = SchemaId::from_uuid(CHILD_ID);

            let parent_prop = fixtures::bool_property("db-prop")?;
            let parent_stored = StoredSchema {
                id: parent_id,
                name: "parent".into(),
                parent_id: None,
                properties: vec![StoredProperty {
                    id: parent_prop.id(),
                    name: parent_prop.name().as_str().into(),
                    required: parent_prop.optionality()
                        == Optionality::Required,
                    multi: parent_prop.multiplicity() == Multiplicity::Many,
                    spec: parent_prop.spec().clone(),
                }],
            };
            let mut known_parents = HashMap::new();
            known_parents.insert(parent_id, parent_stored);

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
                .find(|s| s.name.as_ref() == "child")
                .expect("child in result");
            let has_db_prop =
                child.properties.iter().any(|p| p.name.as_ref() == "db-prop");
            assert!(
                has_db_prop,
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
