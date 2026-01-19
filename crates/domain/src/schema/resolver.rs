//! `Resolver` domain service for schema resolution.
//!
//! Resolves raw schemas into fully resolved Schema entities by merging parent properties,
//! applying excludes, and resolving $ref pointers through the `PropertyBank`.

use std::collections::{HashMap, HashSet};

use super::{
    aggregate::{PropertyBank, Schema},
    property::{Property, PropertyName},
    raw::{RawProperty, RawPropertyRef, RawSchema},
};
use crate::errors::DomainError;

/// Domain Service: Resolves a raw schema into a final Schema entity.
///
/// Merges parent properties, applies excludes, and resolves `$ref` pointers.
///
/// # Examples
///
/// ```
/// # use lithos_domain::{RawSchema, SchemaName, PropertyBank, SchemaResolver};
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
/// let schema = SchemaResolver::resolve(raw, None, &bank).unwrap();
/// assert_eq!(schema.name().as_str(), "test");
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
    /// Merges properties from parent, applies excludes, and resolves references.
    ///
    /// # Arguments
    /// * `raw` - The raw schema definition.
    /// * `parent` - The fully resolved parent schema (if any).
    /// * `bank` - The property bank for resolving references.
    ///
    /// # Errors
    /// Returns `DomainError` if resolution fails (e.g. property not found).
    #[inline]
    pub fn resolve(
        raw: RawSchema,
        parent: Option<&Schema>,
        bank: &PropertyBank,
    ) -> Result<Schema, DomainError> {
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
        final_props.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));

        // Create the Schema entity using the identity of its raw definition
        Schema::new(raw.id, raw.name, final_props)
    }

    fn resolve_own_properties(
        resolved_props: &mut HashMap<String, Property>,
        raw_properties: Vec<RawProperty>,
        bank: &PropertyBank,
    ) -> Result<(), DomainError> {
        for raw_prop in raw_properties {
            let prop = Self::resolve_single_property(raw_prop, bank)?;
            resolved_props.insert(prop.name().to_string(), prop);
        }
        Ok(())
    }

    fn resolve_single_property(
        raw_prop: RawProperty,
        bank: &PropertyBank,
    ) -> Result<Property, DomainError> {
        match raw_prop {
            RawProperty::Inline(inline) => {
                let name = PropertyName::new(inline.name)?;
                Ok(Property::new(
                    inline.id,
                    name,
                    inline.required,
                    inline.array,
                    inline.spec,
                )?)
            }
            RawProperty::Ref(RawPropertyRef {
                ref_path,
            }) => {
                let lookup =
                    ref_path.strip_prefix("#/properties/").unwrap_or(&ref_path);
                bank.get_by_name(lookup).cloned().ok_or_else(|| {
                    DomainError::PropertyNotFound(ref_path.clone())
                })
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Unit tests use expect for readability"
)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::{BoolSpec, PropertySpec};

    #[test]
    fn resolves_ref_property_with_json_pointer_prefix() {
        // GIVEN a property bank with a property
        let mut bank = PropertyBank::new();
        let property = Property::new(
            Uuid::now_v7(),
            PropertyName::new("status".to_owned()).expect("valid name"),
            true,
            false,
            PropertySpec::Bool(BoolSpec::default()),
        )
        .expect("valid property");
        bank.register(property.clone()).expect("register property");

        let raw = RawProperty::Ref(RawPropertyRef {
            ref_path: "#/properties/status".to_owned(),
        });

        // WHEN resolving the ref
        let resolved =
            Resolver::resolve_single_property(raw, &bank).expect("resolve ref");

        // THEN it finds the property by name
        assert_eq!(resolved.name().as_str(), "status");
    }

    #[test]
    fn resolves_ref_property_with_plain_name() {
        // GIVEN a property bank with a property
        let mut bank = PropertyBank::new();
        let property = Property::new(
            Uuid::now_v7(),
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

        // WHEN resolving the ref
        let resolved =
            Resolver::resolve_single_property(raw, &bank).expect("resolve ref");

        // THEN it finds the property by name
        assert_eq!(resolved.name().as_str(), "status");
    }
}
