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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let bank = PropertyBank::new();
/// let raw = RawSchema::new(
///     Uuid::now_v7(),
///     SchemaName::new("test".into())?,
///     None,
///     HashSet::new(),
///     Vec::new(),
/// );
///
/// let schema = Resolver::resolve(raw, None, &bank)?;
/// assert_eq!(&schema.name().0, "test");
/// # Ok(())
/// # }
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
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
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

    #[expect(
        clippy::disallowed_methods,
        reason = "Fixture helpers use expect for deterministic setup."
    )]
    mod fixtures {
        use super::*;

        pub fn parent_property() -> Property {
            let name =
                PropertyName::new("parent".to_owned()).expect("valid name");
            Property::new(
                TEST_PROPERTY_ID_PARENT,
                name,
                true,
                false,
                PropertySpec::Bool(BoolSpec::default()),
            )
            .expect("valid property")
        }

        pub fn status_property() -> Property {
            let name =
                PropertyName::new("status".to_owned()).expect("valid name");
            Property::new(
                TEST_PROPERTY_ID_STATUS,
                name,
                true,
                false,
                PropertySpec::Bool(BoolSpec::default()),
            )
            .expect("valid property")
        }

        pub fn excluded_property() -> Property {
            let name = PropertyName::new("p".to_owned()).expect("valid name");
            Property::new(
                TEST_PROPERTY_ID_EXCLUDE,
                name,
                true,
                false,
                PropertySpec::Bool(BoolSpec::default()),
            )
            .expect("valid property")
        }

        pub fn parent_schema_with_property(property: Property) -> Schema {
            let name = SchemaName::new("parent".to_owned())
                .expect("valid schema name");
            Schema::new(TEST_SCHEMA_ID_PARENT, name, vec![property])
                .expect("valid schema")
        }

        pub fn child_raw_schema() -> RawSchema {
            let name =
                SchemaName::new("child".to_owned()).expect("valid schema name");
            RawSchema::new(
                TEST_SCHEMA_ID_CHILD,
                name,
                None,
                HashSet::new(),
                Vec::new(),
            )
        }

        pub fn child_raw_schema_with_excludes(
            exclude_name: PropertyName,
        ) -> RawSchema {
            let name =
                SchemaName::new("child".to_owned()).expect("valid schema name");
            let mut excludes = HashSet::new();
            excludes.insert(exclude_name);
            RawSchema::new(
                TEST_SCHEMA_ID_CHILD,
                name,
                None,
                excludes,
                Vec::new(),
            )
        }

        pub fn property_bank_with(property: Property) -> PropertyBank {
            let mut bank = PropertyBank::new();
            bank.register(property).expect("register property should succeed");
            bank
        }

        pub fn resolved_schema_with_parent_property() -> Schema {
            let bank = PropertyBank::new();
            let property = parent_property();
            let parent_schema = parent_schema_with_property(property.clone());
            let raw = child_raw_schema();
            Resolver::resolve(raw, Some(&parent_schema), &bank)
                .expect("resolve schema")
        }

        pub fn resolved_ref_property() -> Property {
            let property = status_property();
            let bank = property_bank_with(property);
            let raw = RawProperty::Ref(RawPropertyRef {
                ref_path: "status".to_owned(),
            });
            Resolver::resolve_single_property(raw, &bank).expect("resolve ref")
        }

        pub fn resolved_schema_with_excludes() -> Schema {
            let bank = PropertyBank::new();
            let property = excluded_property();
            let parent_schema = parent_schema_with_property(property);
            let exclude_name =
                PropertyName::new("p".to_owned()).expect("valid name");
            let raw = child_raw_schema_with_excludes(exclude_name);
            Resolver::resolve(raw, Some(&parent_schema), &bank)
                .expect("resolve schema")
        }
    }
    mod resolve {
        use super::*;

        #[test]
        fn includes_parent_properties() {
            let schema = fixtures::resolved_schema_with_parent_property();
            assert!(schema.has("parent"));
        }

        #[test]
        fn excludes_properties_listed_in_child() {
            let schema = fixtures::resolved_schema_with_excludes();
            assert!(!schema.has("p"));
        }
    }

    mod resolve_single_property {
        use super::*;

        #[test]
        fn resolves_ref_property_by_plain_name() {
            let property = fixtures::resolved_ref_property();
            assert_eq!(&property.name().0, "status");
        }

        #[test]
        fn returns_error_for_missing_ref() {
            let bank = PropertyBank::new();
            let raw = RawProperty::Ref(RawPropertyRef {
                ref_path: "missing".to_owned(),
            });

            let result = Resolver::resolve_single_property(raw, &bank);

            assert!(
                matches!(result, Err(SchemaError::PropertyNotFound(_))),
                "Missing property reference should be detected, got: \
                 {result:?}"
            );
        }
    }
}
