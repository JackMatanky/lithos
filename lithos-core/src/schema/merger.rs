//! Schema-level property merging for inheritance.
//!
//! Combines properties from parent and child schemas following inheritance
//! rules:
//! - Child properties override parent properties with same name
//! - Parent properties in excludes list are filtered out
//! - All other parent properties are inherited

use std::collections::HashSet;

use super::property::{PropertyMap, PropertyName};

// ─────────────────────────────────────────────────────────────────────────────
//  Merger
// ─────────────────────────────────────────────────────────────────────────────

/// Merges parent and child properties following inheritance rules.
///
/// **Internal API**: This type is public solely for benchmarking purposes.
/// Do not depend on it in production code - use `Loader` instead.
#[doc(hidden)]
#[non_exhaustive]
pub struct Merger;

impl Merger {
    /// Build a child's resolved properties by inheriting from parent.
    ///
    /// A child schema inherits properties from its parent unless:
    /// - The property is in the `excludes` list
    /// - The child schema already defines the property (child overrides parent)
    ///
    /// Child properties are always kept as-is (full override).
    #[inline]
    #[must_use]
    /// **Internal API**: Public for benchmarking only.
    #[doc(hidden)]
    pub fn inherit_properties(
        parent: &PropertyMap,
        child: &PropertyMap,
        excludes: &[PropertyName],
    ) -> PropertyMap {
        let excluded_names: HashSet<PropertyName> =
            excludes.iter().cloned().collect();

        let mut result = child.clone();

        for (name, prop) in parent.iter_named() {
            if excluded_names.contains(name) || result.contains_key(name) {
                continue;
            }
            result.insert(name.clone(), prop.clone());
        }

        result
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
        error::SchemaError,
        property::{
            Multiplicity, Optionality, Property, PropertyId, PropertyMap,
            PropertyName,
        },
        property_spec::{BoolSpec, PropertySpec},
    };

    mod fixtures {
        use super::*;

        pub fn bool_property(
            name: &str,
        ) -> Result<(PropertyName, Property), SchemaError> {
            let property = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
            Ok((PropertyName::try_new(name)?, property))
        }

        pub fn map_properties(
            props: Vec<(PropertyName, Property)>,
        ) -> PropertyMap {
            PropertyMap::from(props.into_iter().collect::<HashMap<_, _>>())
        }
    }

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions use assert! macros; standard test practice"
    )]
    mod inherit_properties {
        use super::*;

        #[test]
        fn child_inherits_parent_properties() -> Result<(), SchemaError> {
            let parent_prop = fixtures::bool_property("from-parent")?;
            let child_prop = fixtures::bool_property("from-child")?;

            let parent = fixtures::map_properties(vec![parent_prop]);
            let child = fixtures::map_properties(vec![child_prop]);

            let merged = Merger::inherit_properties(&parent, &child, &[]);
            let prop_names: Vec<&str> =
                merged.iter_named().map(|(n, _)| n.as_ref()).collect();
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
            let name = PropertyName::try_new("shared")?;
            let parent_prop = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                Optionality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );
            let child_prop = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                Optionality::Optional,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            );

            let parent =
                fixtures::map_properties(vec![(name.clone(), parent_prop)]);
            let child = fixtures::map_properties(vec![(
                name.clone(),
                child_prop.clone(),
            )]);

            let merged = Merger::inherit_properties(&parent, &child, &[]);
            let stored_prop =
                merged.get(&name).expect("shared property exists");
            assert_eq!(stored_prop.optionality(), Optionality::Optional);
            Ok(())
        }

        #[test]
        fn excludes_filter_parent_properties() -> Result<(), SchemaError> {
            let parent_prop = fixtures::bool_property("skip")?;
            let child_prop = fixtures::bool_property("keep")?;

            let parent = fixtures::map_properties(vec![parent_prop]);
            let child = fixtures::map_properties(vec![child_prop]);

            let excludes = vec![PropertyName::try_new("skip")?];
            let merged = Merger::inherit_properties(&parent, &child, &excludes);
            assert!(merged.contains_key(&PropertyName::try_new("keep")?));
            assert!(!merged.contains_key(&PropertyName::try_new("skip")?));
            Ok(())
        }
    }
}
