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

use crate::support::error::UuidV7Error;

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
        let uuid = Uuid::try_parse(input).map_err(UuidV7Error::Parse)?;
        Self::try_from(uuid)
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

    /// Returns the UUID as a 16-byte array (zero-copy).
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }

    /// Consumes the UUID and returns the raw 16 bytes.
    #[inline]
    #[must_use]
    pub fn into_bytes(self) -> [u8; 16] {
        self.0.into_bytes()
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
    fn try_from(uuid: Uuid) -> Result<Self, Self::Error> {
        if uuid.get_version() == Some(Version::SortRand) {
            return Ok(Self(uuid));
        }
        Err(UuidV7Error::WrongVersion {
            got: uuid.get_version(),
        })
    }
}

impl From<UuidV7> for Uuid {
    #[inline]
    fn from(value: UuidV7) -> Self {
        value.0
    }
}

impl TryFrom<[u8; 16]> for UuidV7 {
    type Error = UuidV7Error;

    #[inline]
    fn try_from(bytes: [u8; 16]) -> Result<Self, Self::Error> {
        let uuid = Uuid::from_bytes(bytes);
        Self::try_from(uuid)
    }
}

impl TryFrom<&[u8; 16]> for UuidV7 {
    type Error = UuidV7Error;

    #[inline]
    fn try_from(bytes: &[u8; 16]) -> Result<Self, Self::Error> {
        let uuid = Uuid::from_bytes_ref(bytes);
        Self::try_from(*uuid)
    }
}

impl TryFrom<&[u8]> for UuidV7 {
    type Error = UuidV7Error;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let uuid =
            Uuid::from_slice(bytes).map_err(UuidV7Error::InvalidBytes)?;
        Self::try_from(uuid)
    }
}

impl FromStr for UuidV7 {
    type Err = UuidV7Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
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
        let err = UuidV7::parse("not-a-uuid").unwrap_err();
        assert!(matches!(err, UuidV7Error::Parse(_)));
    }

    #[test]
    fn try_from_rejects_non_v7() {
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"lithos");
        assert!(matches!(
            UuidV7::try_from(uuid),
            Err(UuidV7Error::WrongVersion { .. })
        ));
    }

    #[test]
    fn roundtrip_into_from_uuid() {
        let id = UuidV7::new();
        let raw: Uuid = id.into();
        let id2 = UuidV7::try_from(raw).expect("roundtrip should work");
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

    #[test]
    fn as_bytes_returns_16_bytes() {
        let id = UuidV7::new();
        let bytes = id.as_bytes();
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn into_bytes_ownership_transfer() {
        let id = UuidV7::new();
        let bytes: [u8; 16] = id.into_bytes();
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn try_from_bytes_accepts_valid_v7() {
        let id = UuidV7::new();
        let bytes = id.into_bytes();
        let id2: UuidV7 =
            bytes.try_into().expect("valid v7 bytes should succeed");
        assert_eq!(id, id2);
    }

    #[test]
    fn try_from_bytes_rejects_non_v7() {
        let non_v7_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"lithos");
        let non_v7_bytes: [u8; 16] = non_v7_uuid.into_bytes();
        let result: Result<UuidV7, _> = non_v7_bytes.try_into();
        assert!(matches!(result, Err(UuidV7Error::WrongVersion { .. })));
    }

    #[test]
    fn try_from_slice_accepts_valid_v7() {
        let id = UuidV7::new();
        let bytes: &[u8] = id.as_bytes();
        let id2: UuidV7 =
            bytes.try_into().expect("valid v7 bytes should succeed");
        assert_eq!(id, id2);
    }

    #[test]
    fn try_from_slice_rejects_wrong_length() {
        let short_bytes: &[u8] = b"short";
        let result: Result<UuidV7, _> = short_bytes.try_into();
        assert!(matches!(result, Err(UuidV7Error::InvalidBytes(_))));
    }
}
