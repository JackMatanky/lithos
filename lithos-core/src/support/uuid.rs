//! UUID v7 support primitives shared across contexts.
//!
//! This module centralizes version-checked UUID handling so domain wrappers can
//! enforce a single identity policy (UUID v7) without duplicating validation.
//! Use [`UuidV7`] at trust boundaries where UUIDs are parsed or accepted from
//! external inputs, then wrap in context-specific ID newtypes.

#![expect(
    clippy::module_name_repetitions,
    reason = "UuidV7 naming is explicit and intentional in uuid module"
)]

use std::{fmt::Display, str::FromStr};

use rkyv::{Archive, Deserialize, Serialize};
use uuid::{Uuid, Version};

/// A UUID constrained to version 7 (time-ordered UUID).
///
/// This type guarantees at construction time that the wrapped UUID uses the
/// v7 format (`uuid::Version::SortRand`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
)]
#[rkyv(derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct UuidV7(Uuid);

impl UuidV7 {
    /// Creates a new UUID v7.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Parses a UUID string and validates that it is version 7.
    ///
    /// # Errors
    ///
    /// Returns [`UuidV7Error`] when parsing fails or the UUID version is not
    /// v7.
    #[inline]
    pub fn parse(input: &str) -> Result<Self, UuidV7Error> {
        let uuid = Uuid::parse_str(input).map_err(UuidV7Error::Parse)?;
        Self::try_from_uuid(uuid)
    }

    /// Validates that a UUID is version 7.
    ///
    /// # Errors
    ///
    /// Returns [`UuidV7Error::WrongVersion`] when `uuid` is not version 7.
    #[inline]
    pub fn try_from_uuid(uuid: Uuid) -> Result<Self, UuidV7Error> {
        if uuid.get_version() == Some(Version::SortRand) {
            return Ok(Self(uuid));
        }
        Err(UuidV7Error::WrongVersion {
            got: uuid.get_version(),
        })
    }

    /// Wraps a UUID without validating version.
    ///
    /// Prefer [`Self::try_from_uuid`] for untrusted inputs.
    /// Use this method only when the caller already guarantees the UUID is v7.
    #[inline]
    #[must_use]
    pub const fn from_uuid_unchecked(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID by reference.
    #[inline]
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Returns the inner UUID by value.
    #[inline]
    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for UuidV7 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Display for UuidV7 {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<Uuid> for UuidV7 {
    type Error = UuidV7Error;

    #[inline]
    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        Self::try_from_uuid(value)
    }
}

impl From<UuidV7> for Uuid {
    #[inline]
    fn from(value: UuidV7) -> Self {
        value.0
    }
}

impl FromStr for UuidV7 {
    type Err = UuidV7Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Errors for UUID v7 validation and parsing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UuidV7Error {
    /// Parsing the UUID string failed.
    #[error("failed to parse UUID: {0}")]
    Parse(#[source] uuid::Error),

    /// The UUID is not version 7.
    #[error("expected UUID version 7, got {got:?}")]
    WrongVersion {
        /// Actual UUID version observed from parsed/provided UUID.
        got: Option<Version>,
    },
}

#[cfg(test)]
mod tests {
    use uuid::{Uuid, Version};

    use super::{UuidV7, UuidV7Error};

    #[test]
    fn new_creates_v7_uuid() {
        let id = UuidV7::new();
        assert_eq!(id.as_uuid().get_version(), Some(Version::SortRand));
    }

    #[test]
    fn parse_accepts_valid_v7() {
        let raw = Uuid::now_v7().to_string();
        let parsed = UuidV7::parse(&raw).expect("v7 uuid should parse");
        assert_eq!(parsed.as_uuid().get_version(), Some(Version::SortRand));
    }

    #[test]
    fn parse_rejects_non_v7() {
        let raw = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"lithos").to_string();
        assert!(matches!(
            UuidV7::parse(&raw),
            Err(UuidV7Error::WrongVersion { .. })
        ));
    }

    #[test]
    fn parse_rejects_invalid_string() {
        UuidV7::parse("not-a-uuid").unwrap_err();
    }

    #[test]
    fn try_from_uuid_rejects_non_v7() {
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"lithos");
        assert!(matches!(
            UuidV7::try_from_uuid(uuid),
            Err(UuidV7Error::WrongVersion { .. })
        ));
    }

    #[test]
    fn roundtrip_into_from_uuid() {
        let id = UuidV7::new();
        let raw: Uuid = id.into();
        let id2 = UuidV7::try_from_uuid(raw).expect("roundtrip should work");
        assert_eq!(id, id2);
    }

    #[test]
    fn display_matches_inner_uuid() {
        let id = UuidV7::new();
        assert_eq!(format!("{id}"), id.as_uuid().to_string());
    }

    #[test]
    fn default_is_v7() {
        let id = UuidV7::default();
        assert_eq!(id.as_uuid().get_version(), Some(Version::SortRand));
    }
}
