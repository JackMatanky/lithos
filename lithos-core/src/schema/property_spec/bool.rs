//! Boolean property constraints.

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::error::SchemaError;

/// Boolean property constraints (marker type).
///
/// This type intentionally has no methods because it carries no data.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Default, Archive, Serialize, Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct BoolSpec;

impl TryFrom<crate::schema::raw::property_spec::RawBoolSpec> for BoolSpec {
    type Error = SchemaError;

    #[inline]
    fn try_from(
        _raw: crate::schema::raw::property_spec::RawBoolSpec,
    ) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::property_spec::PropertySpec;

    #[test]
    fn bool_spec_accepts_true() {
        let spec = PropertySpec::Bool(super::BoolSpec::default());
        let result = spec.validate(&serde_json::Value::Bool(true));

        assert!(
            result.is_ok(),
            "Expected bool validation to succeed, got: {result:?}"
        );
    }

    #[test]
    fn bool_spec_accepts_false() {
        let spec = PropertySpec::Bool(super::BoolSpec::default());
        let result = spec.validate(&serde_json::Value::Bool(false));

        assert!(
            result.is_ok(),
            "Expected bool validation to succeed, got: {result:?}"
        );
    }
}
