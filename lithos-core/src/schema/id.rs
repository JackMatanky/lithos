//! Schema identifier value objects.
//!
//! Core identifier types for the schema domain: `SchemaId` and `SchemaName`.
//! These types were previously part of the Schema aggregate but are now
//! standalone value objects used across the schema system.

use std::{borrow::Borrow, fmt::Display, sync::LazyLock};

use regex::Regex;
use uuid::Uuid;

use super::error::SchemaError;

/// Unique identifier for a schema.
///
/// Wraps a UUID v7 (time-ordered) identifier for schemas.
///
/// # Examples
/// ```
/// use lithos_core::schema::id::SchemaId;
///
/// let id = SchemaId::new();
/// let uuid = id.into_uuid();
/// ```
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct SchemaId(Uuid);

impl std::fmt::Display for SchemaId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SchemaId {
    /// Creates a new UUID v7-based `SchemaId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::id::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Wraps a UUID into a `SchemaId`.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::id::SchemaId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::now_v7();
    /// let id = SchemaId::from_uuid(uuid);
    /// assert_eq!(*id.as_uuid(), uuid);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID reference.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::id::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _ = id.as_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Returns the inner UUID by value.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::id::SchemaId;
    ///
    /// let id = SchemaId::new();
    /// let _uuid = id.into_uuid();
    /// ```
    #[inline]
    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for SchemaId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

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
/// use lithos_core::schema::id::SchemaName;
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
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct SchemaName(Box<str>);

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
    /// use lithos_core::schema::id::SchemaName;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let name = SchemaName::try_new("test")?;
    /// assert_eq!(name.as_str(), "test");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_new(name: &str) -> Result<Self, SchemaError> {
        Self::validate(name)?;
        Ok(Self(name.into()))
    }

    #[inline]
    fn validate(name: &str) -> Result<(), SchemaError> {
        static RE: LazyLock<Result<Regex, regex::Error>> =
            LazyLock::new(|| Regex::new(SchemaName::PATTERN));

        if name.is_empty() {
            return Err(SchemaError::EmptySchemaName);
        }
        if name.len() > 64 {
            return Err(SchemaError::SchemaNameTooLong(name.len()));
        }

        let re = RE.as_ref().map_err(|error| {
            SchemaError::ValidationFailed(format!(
                "Invalid schema name regex: {error}"
            ))
        })?;

        if !re.is_match(name) {
            return Err(SchemaError::InvalidSchemaName(name.into()));
        }
        Ok(())
    }

    /// Returns the inner string slice.
    ///
    /// # Examples
    /// ```
    /// use lithos_core::schema::id::SchemaName;
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
        val.0.into()
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
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_id_new_creates_unique_ids() {
        let id1 = SchemaId::new();
        let id2 = SchemaId::new();
        assert_ne!(id1, id2, "Each call to new() should create a unique ID");
    }

    #[test]
    fn schema_id_roundtrip() {
        let uuid = Uuid::now_v7();
        let id = SchemaId::from_uuid(uuid);
        assert_eq!(id.into_uuid(), uuid);
    }

    #[test]
    fn schema_name_valid() {
        SchemaName::try_new("test").unwrap();
        SchemaName::try_new("test-name").unwrap();
        SchemaName::try_new("test_name").unwrap();
        SchemaName::try_new("test123").unwrap();
        SchemaName::try_new("a").unwrap();
    }

    #[test]
    fn schema_name_empty_rejected() {
        SchemaName::try_new("").unwrap_err();
    }

    #[test]
    fn schema_name_too_long_rejected() {
        let long_name = "a".repeat(65);
        SchemaName::try_new(&long_name).unwrap_err();
    }

    #[test]
    fn schema_name_invalid_chars_rejected() {
        SchemaName::try_new("Test").unwrap_err(); // Uppercase
        SchemaName::try_new("test name").unwrap_err(); // Space
        SchemaName::try_new("test!").unwrap_err(); // Special char
        SchemaName::try_new("test.name").unwrap_err(); // Period
    }

    #[test]
    fn schema_name_as_str() {
        let name = SchemaName::try_new("test").unwrap();
        assert_eq!(name.as_str(), "test");
        assert_eq!(name.as_ref(), "test");
    }
}
