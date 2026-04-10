//! Raw property bank definitions.

use super::{
    metadata::RawFileTimes,
    property::{RawPropertyBankEntry, RawPropertyMap},
    version::RawSchemaVersion,
};

/// Raw property bank loaded from vault files.
///
/// Property names are validated during deserialization via `RawPropertyMap`,
/// ensuring all keys are valid `PropertyName` instances.
///
/// Bank entries use inline property DTOs; `required` is accepted at this layer
/// and overridden with a warning during domain construction.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::raw::RawPropertyBank;
///
/// // Properties are validated during deserialization
/// let toml = r#"
/// [properties.my_property]
/// type = "bool"
/// "#;
/// let bank: RawPropertyBank = toml::from_str(toml)?;
/// // bank.properties() returns HashMap<PropertyName, RawPropertyBankEntry>
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawPropertyBank {
    /// Property bank format version (defaults to "1.0" if not specified).
    /// Validated during deserialization via `RawSchemaVersion`'s custom
    /// Deserialize impl.
    #[serde(rename = "$version", default)]
    version: RawSchemaVersion,

    /// Validated property map (keys are guaranteed valid `PropertyNames`).
    properties: RawPropertyMap<RawPropertyBankEntry>,

    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip)]
    file_times: RawFileTimes,
}

impl RawPropertyBank {
    /// Returns the schema version.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// assert_eq!(bank.version().as_str(), "1.0");
    /// ```
    #[inline]
    #[must_use]
    pub fn version(&self) -> &RawSchemaVersion {
        &self.version
    }

    /// Returns the properties map.
    ///
    /// All keys are guaranteed to be valid `PropertyName` instances.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// for (name, entry) in bank.properties().iter() {
    ///     // name is &PropertyName - already validated
    ///     println!("{}: {:?}", name.as_str(), entry);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &RawPropertyMap<RawPropertyBankEntry> {
        &self.properties
    }

    /// Returns the file timestamps.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::RawPropertyBank;
    /// # let bank: RawPropertyBank = unimplemented!();
    /// let file_times = bank.file_times();
    /// println!("Created: {:?}", file_times.created_at);
    /// ```
    #[inline]
    #[must_use]
    pub fn file_times(&self) -> &RawFileTimes {
        &self.file_times
    }

    /// Set file timestamps (called by Ingestor after deserialization).
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::raw::{RawPropertyBank, RawFileTimes};
    /// # let bank: RawPropertyBank = unimplemented!();
    /// # let file_times: RawFileTimes = unimplemented!();
    /// let bank = bank.with_file_times(file_times);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_file_times(self, file_times: RawFileTimes) -> Self {
        Self {
            file_times,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_valid() {
        let json = serde_json::json!({
            "$version": "1.0",
            "properties": {
                "title": {
                    "multi": false,
                    "type": "string"
                }
            }
        });
        let _: RawPropertyBank = serde_json::from_value(json).unwrap();
    }

    #[test]
    fn deserializes_empty() {
        let json = serde_json::json!({
            "$version": "1.0",
            "properties": {}
        });
        let _: RawPropertyBank = serde_json::from_value(json).unwrap();
    }

    #[test]
    fn validate_invalid_property_name() {
        // Property name validation now happens during deserialization
        // Invalid property names cannot be constructed via RawPropertyMap
        let json = serde_json::json!({
            "$version": "1.0",
            "properties": {
                "Invalid Name!": {  // Space + special char
                    "multi": false,
                    "type": "bool"
                }
            }
        });
        // Deserialization should fail
        serde_json::from_value::<RawPropertyBank>(json).unwrap_err();
    }
}
