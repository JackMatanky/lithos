//! Schema identifier types: Name and ID.
//!
//! This module provides the core identity types for the schema system:
//! - [`SchemaId`] - UUID-based schema identifier
//! - [`SchemaName`] - Validated schema name value object
//!
//! Separating identifiers from aggregates allows for cleaner dependency
//! management and targeted indexing.

use std::{
    borrow::Borrow,
    fmt::Display,
    sync::{Arc, LazyLock},
};

use regex::Regex;
use rkyv::{Archive, Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    fs::RelativePath,
    schema::error::{SchemaError, SchemaNameError, SchemaSyntaxError},
    support::uuid::UuidV7,
};

// ============================================================================
// Schema Identifier
// ============================================================================

/// Unique identifier for a schema.
///
/// Wraps a UUID v7 (time-ordered) identifier for schemas.
///
/// # Examples
/// ```
/// use lithos_core::schema::identifier::SchemaId;
///
/// let id = SchemaId::new();
/// let _uuid = id.as_uuid_v7().into_uuid();
/// ```
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq))]
pub struct SchemaId(UuidV7);

impl SchemaId {
    /// Creates a new UUID v7-based `SchemaId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::identifier::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _ = id.as_uuid_v7();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(UuidV7::new())
    }

    /// Returns the inner `UuidV7` reference.
    #[inline]
    #[must_use]
    pub const fn as_uuid_v7(&self) -> &UuidV7 {
        &self.0
    }
}

impl From<UuidV7> for SchemaId {
    #[inline]
    fn from(value: UuidV7) -> Self {
        Self(value)
    }
}

impl TryFrom<Uuid> for SchemaId {
    type Error = crate::support::UuidV7Error;

    #[inline]
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        Ok(Self(UuidV7::try_from(value)?))
    }
}

impl Default for SchemaId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SchemaId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Schema Name
// ============================================================================

/// Validated schema name value object.
///
/// Enforces invariants:
/// - Non-empty
/// - Max 64 characters
/// - Matches regex `^[a-z0-9_-]+$` (lowercase alphanumeric, underscores,
///   dashes)
///
/// # Examples
///
/// ```
/// use lithos_core::schema::identifier::SchemaName;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let name = SchemaName::try_new("project-note")?;
/// assert_eq!(name.as_str(), "project-note", "Schema name should match");
///
/// let name2 = SchemaName::try_new("daily_note")?;
/// assert_eq!(name2.as_str(), "daily_note", "Schema name should match");
///
/// let name3 = SchemaName::try_new("myschema")?;
/// assert_eq!(name3.as_str(), "myschema", "Schema name should match");
///
/// let invalid = SchemaName::try_new("");
/// assert!(invalid.is_err(), "Empty name should be rejected");
/// # Ok(())
/// # }
/// ```
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
#[derive(serde::Serialize)]
pub struct SchemaName(Arc<str>);

impl SchemaName {
    /// Schema name validation pattern: lowercase letters, numbers, underscores,
    /// and hyphens.
    ///
    /// Pattern: `^[a-z0-9_-]+$`.
    ///
    /// # Examples
    /// - Valid: `daily-note`, `project_schema`, `schema123`
    /// - Invalid: `MySchema`, `invalid name`, `name!`
    const PATTERN: &'static str = "^[a-z0-9_-]+$";

    /// Create a new `SchemaName` with validation.
    ///
    /// # Errors
    /// Returns `SchemaError` if validation fails.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::identifier::SchemaName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::try_new("test")?;
    /// assert_eq!(name.as_str(), "test");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_new(name: &str) -> Result<Self, SchemaError> {
        Self::validate(name)?;
        Ok(Self(Arc::from(name)))
    }

    #[inline]
    fn validate(name: &str) -> Result<(), SchemaError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(SchemaName::PATTERN));

        if name.is_empty() {
            return Err(SchemaNameError::Empty.into());
        }
        if name.len() > 64 {
            return Err(SchemaNameError::TooLong {
                len: name.len(),
                max: 64,
            }
            .into());
        }

        let re =
            RE.as_ref().map_err(|error| SchemaNameError::InvalidRegex {
                reason: error.to_string().into(),
            })?;

        if !re.is_match(name) {
            return Err(SchemaNameError::InvalidFormat {
                name: name.into(),
            }
            .into());
        }
        Ok(())
    }

    /// Returns the inner string slice.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::identifier::SchemaName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::try_new("project")?;
    /// assert_eq!(name.as_str(), "project");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SchemaName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for SchemaName {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for SchemaName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SchemaName> for String {
    #[inline]
    fn from(val: SchemaName) -> Self {
        val.0.to_string()
    }
}

impl TryFrom<&str> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(&value)
    }
}

impl TryFrom<Box<str>> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(Arc::from(value)))
    }
}

impl TryFrom<Arc<str>> for SchemaName {
    type Error = SchemaError;

    #[inline]
    fn try_from(value: Arc<str>) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

impl TryFrom<&RelativePath> for SchemaName {
    type Error = SchemaError;

    /// Derives `SchemaName` from a path's file stem (basename without
    /// extension).
    ///
    /// # Examples
    /// ```
    /// use lithos_core::{fs::RelativePath, schema::identifier::SchemaName};
    ///
    /// let path = RelativePath::try_from("schemas/user-profile.json").unwrap();
    /// let name = SchemaName::try_from(&path).unwrap();
    /// assert_eq!(name.as_str(), "user-profile");
    /// ```
    #[inline]
    fn try_from(path: &RelativePath) -> Result<Self, Self::Error> {
        let filename = path.filename().ok_or_else(|| {
            SchemaError::Syntax(SchemaSyntaxError::SchemaName(
                SchemaNameError::InvalidFormat {
                    name: format!("Path has no filename: {path}").into(),
                },
            ))
        })?;

        let name = filename.basename();
        Self::try_new(name)
    }
}

impl TryFrom<RelativePath> for SchemaName {
    type Error = SchemaError;

    /// Derives `SchemaName` from a path's file stem (basename without
    /// extension).
    ///
    /// # Examples
    /// ```
    /// use lithos_core::{fs::RelativePath, schema::identifier::SchemaName};
    ///
    /// let path = RelativePath::try_from("schemas/user-profile.json").unwrap();
    /// let name = SchemaName::try_from(path).unwrap();
    /// assert_eq!(name.as_str(), "user-profile");
    /// ```
    #[inline]
    fn try_from(path: RelativePath) -> Result<Self, Self::Error> {
        Self::try_from(&path)
    }
}

#[expect(
    clippy::missing_trait_methods,
    reason = "deserialize_in_place is not applicable for this wrapper"
)]
impl<'de> serde::Deserialize<'de> for SchemaName {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Arc::<str>::deserialize(deserializer)?;
        Self::try_from(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_id_new_creates_unique_ids() {
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn schema_id_exposes_uuid_v7_view() {
        let id = SchemaId::new();
        assert_eq!(
            id.as_uuid_v7().as_uuid().get_version(),
            Some(uuid::Version::SortRand)
        );
    }

    #[test]
    fn schema_name_try_from_path() {
        let path = RelativePath::try_from("schemas/daily-note.toml").unwrap();
        let name = SchemaName::try_from(path).unwrap();
        assert_eq!(name.as_str(), "daily-note");
    }

    #[test]
    fn schema_name_try_from_invalid_path_fails() {
        let path = RelativePath::try_from("schemas/.json").unwrap();
        let result = SchemaName::try_from(path);
        result.unwrap_err();
    }
}
