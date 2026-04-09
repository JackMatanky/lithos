//! Raw schema version types.

/// Schema format version.
///
/// Represents the version string from schema and property bank files.
/// Validates against the supported version constant during deserialization.
///
/// # Design Note
///
/// This type uses `serde(try_from)` to validate the version string during
/// parsing, following the "parse, don't validate" principle.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct RawSchemaVersion(Box<str>);

impl RawSchemaVersion {
    /// Current supported schema version.
    pub const SUPPORTED: &'static str = "1.0";

    /// Returns the version string.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl serde::Serialize for RawSchemaVersion {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl TryFrom<String> for RawSchemaVersion {
    type Error = String;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s != Self::SUPPORTED {
            return Err(format!(
                "unsupported schema version '{}', expected '{}'",
                s,
                Self::SUPPORTED
            ));
        }
        Ok(Self(s.into()))
    }
}

impl Default for RawSchemaVersion {
    #[inline]
    fn default() -> Self {
        Self(Self::SUPPORTED.into())
    }
}

impl TryFrom<&str> for RawSchemaVersion {
    type Error = String;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_owned())
    }
}

impl TryFrom<Box<str>> for RawSchemaVersion {
    type Error = String;

    #[inline]
    fn try_from(s: Box<str>) -> Result<Self, Self::Error> {
        Self::try_from(s.into_string())
    }
}

impl AsRef<str> for RawSchemaVersion {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_defaults_to_supported() {
        let version = RawSchemaVersion::default();
        assert_eq!(version.as_str(), RawSchemaVersion::SUPPORTED);
    }

    #[test]
    fn version_validates_supported() {
        let version = RawSchemaVersion::try_from("1.0").unwrap();
        assert_eq!(version.as_str(), "1.0");
    }

    #[test]
    fn version_rejects_unsupported() {
        let result = RawSchemaVersion::try_from("2.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported schema version"));
    }

    #[test]
    fn version_roundtrips_json() {
        let version = RawSchemaVersion::default();
        let json = serde_json::to_string(&version).unwrap();
        assert_eq!(json, "\"1.0\"");
        let back: RawSchemaVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(back, version);
    }
}
