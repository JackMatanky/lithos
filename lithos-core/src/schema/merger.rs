//! Schema-level property merging for inheritance.
//!
//! Combines properties from parent and child schemas following inheritance
//! rules:
//! - Child properties override parent properties with same name
//! - Parent properties in excludes list are filtered out
//! - All other parent properties are inherited

use std::collections::{HashMap, HashSet};

use super::property::{Property, PropertyName};

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
        parent: &HashMap<PropertyName, Property>,
        child: &HashMap<PropertyName, Property>,
        excludes: &[PropertyName],
    ) -> HashMap<PropertyName, Property> {
        let excluded_names: HashSet<PropertyName> =
            excludes.iter().cloned().collect();

        let mut result = child.clone();

        #[expect(
            clippy::iter_over_hash_type,
            reason = "HashMap iteration is intentional for property \
                      inheritance"
        )]
        for (name, prop) in parent {
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
            Multiplicity, Optionality, Property, PropertyId, PropertyName,
        },
        property_spec::{BoolSpec, PropertySpec},
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

        pub fn map_properties(
            props: Vec<Property>,
        ) -> HashMap<PropertyName, Property> {
            props.into_iter().map(|p| (p.name().clone(), p)).collect()
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
                merged.values().map(|p| p.name().as_ref()).collect();
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

            let parent = fixtures::map_properties(vec![parent_prop]);
            let child = fixtures::map_properties(vec![child_prop.clone()]);

            let merged = Merger::inherit_properties(&parent, &child, &[]);
            let stored_prop = merged
                .get(&PropertyName::try_new("shared")?)
                .expect("shared property exists");
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
