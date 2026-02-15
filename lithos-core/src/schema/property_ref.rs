//! Typed property references for schema resolution.

use uuid::Uuid;

use super::{
    error::SchemaError,
    property::{PropertyId, PropertyName},
};

/// Typed reference to a property definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PropertyRef {
    /// Reference by property id.
    ById(PropertyId),
    /// Reference by property name.
    ByName(PropertyName),
}

impl PropertyRef {
    /// Parse a reference string into a typed property reference.
    ///
    /// Accepted forms:
    /// - "#/properties/<name>" (by name)
    /// - "$bank:<uuid>" (by id)
    /// - "<name>" (by name)
    /// - "<uuid>" (by id)
    ///
    /// # Errors
    /// Returns `SchemaError` when the reference is not valid.
    #[inline]
    pub fn parse(reference: &str) -> Result<Self, SchemaError> {
        if let Some(name) = reference.strip_prefix("#/properties/") {
            return Ok(Self::ByName(PropertyName::try_from(name)?));
        }

        if let Some(id_str) = reference.strip_prefix("$bank:") {
            let id = Uuid::parse_str(id_str).map_err(|error| {
                SchemaError::ValidationFailed(format!(
                    "Invalid property id reference: {error}"
                ))
            })?;
            return Ok(Self::ById(PropertyId::from_uuid(id)));
        }

        if let Ok(id) = Uuid::parse_str(reference) {
            return Ok(Self::ById(PropertyId::from_uuid(id)));
        }

        Ok(Self::ByName(PropertyName::try_from(reference)?))
    }
}

impl TryFrom<&str> for PropertyRef {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}
