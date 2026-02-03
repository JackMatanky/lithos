//! `Resolver` domain service for schema resolution.
//!
//! Resolves raw schemas into fully resolved Schema entities by merging parent
//! properties, applying excludes, and resolving $ref pointers through the
//! `PropertyBank`.

use std::collections::{HashMap, HashSet};

use super::{
    aggregate::{PropertyBank, Schema},
    error::SchemaError,
    property::{Property, PropertyName},
    raw::{RawProperty, RawPropertyRef, RawSchema},
};

/// Domain Service: Resolves a raw schema into a final Schema entity.
///
/// Merges parent properties, applies excludes, and resolves `$ref` pointers.
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::raw::RawSchema;
/// # use lithos_core::schema::aggregate::{SchemaName, PropertyBank};
/// # use lithos_core::schema::resolver::Resolver;
/// # use std::collections::HashSet;
/// # use uuid::Uuid;
/// let bank = PropertyBank::new();
/// let raw = RawSchema::new(
///     Uuid::now_v7(),
///     SchemaName::new("test".into()).unwrap(),
///     None,
///     HashSet::new(),
///     Vec::new(),
/// );
///
/// let schema = Resolver::resolve(raw, None, &bank).unwrap();
/// assert_eq!(&schema.name().0, "test");
/// ```
#[non_exhaustive]
pub struct Resolver;

impl Resolver {
    fn merge_parent_properties(
        resolved_props: &mut HashMap<String, Property>,
        parent: Option<&Schema>,
        excludes: &HashSet<PropertyName>,
    ) {
        if let Some(p) = parent {
            for prop in p.properties() {
                if !excludes.contains(prop.name()) {
                    resolved_props
                        .insert(prop.name().to_string(), prop.clone());
                }
            }
        }
    }

    /// Resolve a `RawSchema` into a fully resolved Schema.
    ///
    /// Merges properties from parent, applies excludes, and resolves
    /// references.
    ///
    /// # Arguments
    /// * `raw` - The raw schema definition.
    /// * `parent` - The fully resolved parent schema (if any).
    /// * `bank` - The property bank for resolving references.
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails (e.g. property not found).
    #[inline]
    pub fn resolve(
        raw: RawSchema,
        parent: Option<&Schema>,
        bank: &PropertyBank,
    ) -> Result<Schema, SchemaError> {
        let mut resolved_props = HashMap::new();

        Self::merge_parent_properties(
            &mut resolved_props,
            parent,
            &raw.excludes,
        );
        Self::resolve_own_properties(
            &mut resolved_props,
            raw.properties,
            bank,
        )?;

        let mut final_props: Vec<Property> =
            resolved_props.into_values().collect();
        // Sort for determinism
        final_props.sort_by(|a, b| a.name().0.cmp(&b.name().0));

        // Create the Schema entity using the identity of its raw definition
        Schema::new(raw.id, raw.name, final_props)
    }

    fn resolve_own_properties(
        resolved_props: &mut HashMap<String, Property>,
        raw_properties: Vec<RawProperty>,
        bank: &PropertyBank,
    ) -> Result<(), SchemaError> {
        for raw_prop in raw_properties {
            let prop = Self::resolve_single_property(raw_prop, bank)?;
            resolved_props.insert(prop.name().to_string(), prop);
        }
        Ok(())
    }

    fn resolve_single_property(
        raw_prop: RawProperty,
        bank: &PropertyBank,
    ) -> Result<Property, SchemaError> {
        match raw_prop {
            RawProperty::Inline(inline) => {
                let name = PropertyName::new(inline.name)?;
                let spec = inline.spec.try_into_validated()?;
                Ok(Property::new(
                    inline.id,
                    name,
                    inline.required,
                    inline.array,
                    spec,
                )?)
            }
            RawProperty::Ref(RawPropertyRef {
                ref_path,
            }) => {
                let lookup =
                    ref_path.strip_prefix("#/properties/").unwrap_or(&ref_path);
                bank.get_by_name(lookup).cloned().ok_or_else(|| {
                    SchemaError::PropertyNotFound(ref_path.clone())
                })
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Test module uses Result::expect() for ergonomic arrangement and \
              assertions. Acceptable in test-only code paths."
)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::SchemaName,
        property_spec::{BoolSpec, PropertySpec},
    };

    const TEST_SCHEMA_ID_PARENT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0501);
    const TEST_SCHEMA_ID_CHILD: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0502);
    const TEST_PROPERTY_ID_PARENT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0503);
    const TEST_PROPERTY_ID_STATUS: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0504);
    const TEST_PROPERTY_ID_EXCLUDE: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0505);

    #[test]
    fn resolve_includes_parent_properties() {
        // GIVEN: a parent schema with a property
        let bank = PropertyBank::new();
        let parent_prop = Property::new(
            TEST_PROPERTY_ID_PARENT,
            PropertyName::new("parent".to_owned()).expect("valid name"),
            true,
            false,
            PropertySpec::Bool(BoolSpec::default()),
        )
        .expect("valid property");
        let mut bank_with_prop = PropertyBank::new();
        bank_with_prop
            .register(parent_prop.clone())
            .expect("register property");
        let parent_schema = Schema::new(
            TEST_SCHEMA_ID_PARENT,
            SchemaName::new("parent".to_owned()).expect("valid name"),
            vec![parent_prop],
        )
        .expect("valid schema");

        // WHEN: resolving a child raw schema
        let raw = RawSchema::new(
            TEST_SCHEMA_ID_CHILD,
            SchemaName::new("child".to_owned()).expect("valid name"),
            None,
            HashSet::new(),
            Vec::new(),
        );
        let schema = Resolver::resolve(raw, Some(&parent_schema), &bank)
            .expect("resolve schema");

        // THEN: parent property is retained
        assert!(schema.has("parent"));
    }

    #[test]
    fn resolves_ref_property_with_plain_name() {
        // GIVEN: a property bank with a property
        let mut bank = PropertyBank::new();
        let property = Property::new(
            TEST_PROPERTY_ID_STATUS,
            PropertyName::new("status".to_owned()).expect("valid name"),
            true,
            false,
            PropertySpec::Bool(BoolSpec::default()),
        )
        .expect("valid property");
        bank.register(property.clone()).expect("register property");

        let raw = RawProperty::Ref(RawPropertyRef {
            ref_path: "status".to_owned(),
        });

        // WHEN: resolving the ref
        let resolved =
            Resolver::resolve_single_property(raw, &bank).expect("resolve ref");

        // THEN: it finds the property by name
        assert_eq!(&resolved.name().0, "status");
    }

    #[test]
    fn resolve_handles_excludes() {
        // GIVEN: a parent schema with a property
        let bank = PropertyBank::new();
        let prop = Property::new(
            TEST_PROPERTY_ID_EXCLUDE,
            PropertyName::new("p".to_owned()).unwrap(),
            true,
            false,
            PropertySpec::Bool(BoolSpec::default()),
        )
        .unwrap();
        let parent = Schema::new(
            TEST_SCHEMA_ID_PARENT,
            SchemaName::new("parent".into()).unwrap(),
            vec![prop],
        )
        .unwrap();

        // AND: a child schema that excludes that property
        let mut excludes = HashSet::new();
        excludes.insert(PropertyName::new("p".to_owned()).unwrap());
        let raw = RawSchema::new(
            TEST_SCHEMA_ID_CHILD,
            SchemaName::new("child".into()).unwrap(),
            None,
            excludes,
            vec![],
        );

        // WHEN: resolving
        let resolved = Resolver::resolve(raw, Some(&parent), &bank).unwrap();

        // THEN: the property is excluded
        assert!(!resolved.has("p"));
    }

    #[test]
    fn resolve_returns_error_for_missing_ref() {
        // GIVEN: an empty property bank
        let bank = PropertyBank::new();
        let raw = RawProperty::Ref(RawPropertyRef {
            ref_path: "missing".to_owned(),
        });

        // WHEN: resolving a missing ref
        let result = Resolver::resolve_single_property(raw, &bank);

        // THEN: it returns a PropertyNotFound error
        assert!(
            matches!(result, Err(SchemaError::PropertyNotFound(_))),
            "Missing property reference should be detected, got: {result:?}"
        );
    }
}
