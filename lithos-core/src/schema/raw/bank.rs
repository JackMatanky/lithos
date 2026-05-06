//! Root aggregate for raw property bank parsing.
//!
//! Provides the [`RawPropertyBank`] type which represents a property bank
//! file as parsed from the filesystem.

use crate::{
    fs::FileInfo,
    schema::raw::{
        RawPropertyBankEntry, RawPropertyMap, version::RawSchemaVersion,
    },
};

/// Represents a raw property bank as parsed from a file.
///
/// This structure captures the serialized form of a property bank,
/// including its format version and global property definitions.
///
/// # Field Policy
///
/// - `version`: Format version (defaults to "1.0").
/// - `properties`: Map of shared property definitions.
/// - `info`: File metadata for staleness detection.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct RawPropertyBank {
    /// Property bank format version.
    #[serde(rename = "$version", default)]
    pub version: RawSchemaVersion,

    /// Validated property map (keys are guaranteed valid `PropertyNames`).
    pub properties: RawPropertyMap<RawPropertyBankEntry>,

    /// File metadata for staleness detection.
    ///
    /// Populated during ingestion. Not serialized to TOML.
    #[serde(skip, default = "default_info")]
    pub info: FileInfo,
}

#[inline]
const fn default_info() -> FileInfo {
    FileInfo::new(None, None, 0)
}

impl RawPropertyBank {
    /// Returns the schema version.
    #[inline]
    #[must_use]
    pub fn version(&self) -> &RawSchemaVersion {
        &self.version
    }

    /// Returns the properties map.
    ///
    /// All keys are guaranteed to be valid `PropertyName` instances.
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &RawPropertyMap<RawPropertyBankEntry> {
        &self.properties
    }

    /// Consumes the bank and returns the properties map.
    #[inline]
    #[must_use]
    pub fn into_properties(self) -> RawPropertyMap<RawPropertyBankEntry> {
        self.properties
    }

    /// Returns the file information.
    #[inline]
    #[must_use]
    pub fn info(&self) -> &FileInfo {
        &self.info
    }

    /// Set file information (called by Ingestor after deserialization).
    #[inline]
    #[must_use]
    pub fn with_info(self, info: FileInfo) -> Self {
        Self {
            info,
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
}
