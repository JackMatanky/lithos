//! Raw property types.
//!
//! Defines the property-level structures:
//! - Property variants (Inline vs Ref)
//! - Property bank entries
//! - Overridable fields

use super::property_spec::{
    RawDateSpec, RawFileSpec, RawNumberSpec, RawPropertySpec, RawStringSpec,
};

/// Raw property for schema properties map.
///
/// Used in `RawSchema.properties` where the name is the map key.
/// Discriminated by presence of `$ref` field. Ref is tried first because
/// it has a required `$ref` field that Inline never has.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawBoolSpec, RawProperty, RawPropertyInline, RawPropertySpec,
/// };
///
/// let property = RawProperty::Inline(RawPropertyInline {
///     required: false,
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// });
/// match property {
///     RawProperty::Inline(_) => {}
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum RawProperty {
    /// A reference to a property in the property bank with optional overrides.
    Ref(RawPropertyRef),
    /// An inline property definition.
    Inline(RawPropertyInline),
}

/// Inline variant of a raw property.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawBoolSpec, RawPropertyInline, RawPropertySpec,
/// };
///
/// let inline = RawPropertyInline {
///     required: false,
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// };
/// let _ = inline;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyInline {
    /// Whether property is required.
    #[serde(default)]
    pub required: bool,
    /// Whether property accepts multiple values.
    #[serde(default)]
    pub multi: bool,
    /// Type-specific validation constraints.
    #[serde(flatten)]
    pub spec: RawPropertySpec,
}

/// Reference variant of a raw property with optional overrides.
///
/// Override fields are grouped by type via flattened `Raw*Spec` structs.
/// All override fields are `Option<T>` — `None` means "don't override".
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{
///     RawDateSpec, RawFileSpec, RawNumberSpec, RawPropertyRef, RawStringSpec,
/// };
///
/// let reference = RawPropertyRef {
///     ref_path: "property_bank#/flag".into(),
///     required: None,
///     multi: None,
///     number: RawNumberSpec::default(),
///     string: RawStringSpec::default(),
///     date: RawDateSpec::default(),
///     file: RawFileSpec::default(),
/// };
/// let _ = reference;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyRef {
    /// The reference path (e.g., `property_bank#/date_iso_8601`).
    #[serde(rename = "$ref")]
    pub ref_path: Box<str>,
    /// Override whether property is required.
    pub required: Option<bool>,
    /// Override whether property accepts multiple values.
    pub multi: Option<bool>,
    /// Number-type overrides (min, max, step).
    #[serde(flatten)]
    pub number: RawNumberSpec,
    /// String-type overrides (options, pattern).
    #[serde(flatten)]
    pub string: RawStringSpec,
    /// Date-type overrides (format).
    #[serde(flatten)]
    pub date: RawDateSpec,
    /// File-type overrides (directory, `file_class`).
    #[serde(flatten)]
    pub file: RawFileSpec,
}

/// Entry in the raw property bank.
///
/// The property name is the map key, not a field here.
/// `required` is not present because the bank is schema-agnostic.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::{RawBoolSpec, RawPropertyBankEntry, RawPropertySpec};
///
/// let entry = RawPropertyBankEntry {
///     multi: false,
///     spec: RawPropertySpec::Bool(RawBoolSpec),
/// };
/// let _ = entry;
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct RawPropertyBankEntry {
    /// Whether property accepts multiple values.
    #[serde(default)]
    pub multi: bool,
    /// Type-specific validation constraints.
    #[serde(flatten)]
    pub spec: RawPropertySpec,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_property_inline_variant_constructs() {
        use super::super::property_spec::RawBoolSpec;

        let inline = RawPropertyInline {
            required: false,
            multi: false,
            spec: RawPropertySpec::Bool(RawBoolSpec),
        };
        let inline_variant = RawProperty::Inline(inline);

        assert!(
            matches!(inline_variant, RawProperty::Inline(_)),
            "RawProperty should be Inline variant"
        );
    }

    #[test]
    fn raw_property_ref_variant_constructs() {
        let reference = RawPropertyRef {
            ref_path: "property_bank#/status".into(),
            required: None,
            multi: None,
            number: RawNumberSpec::default(),
            string: RawStringSpec::default(),
            date: RawDateSpec::default(),
            file: RawFileSpec::default(),
        };
        let reference_variant = RawProperty::Ref(reference);

        assert!(
            matches!(reference_variant, RawProperty::Ref(_)),
            "RawProperty should be Ref variant"
        );
    }
}
