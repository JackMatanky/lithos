use rkyv::{
    Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
};
use serde::{Deserialize, Serialize};

/// A 32-byte BLAKE3 hash.
///
/// Newtype wrapper around `[u8; 32]` to provide type safety and
/// centralised hashing policy across the project.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[expect(
    clippy::module_name_repetitions,
    reason = "Blake3Hash is descriptive and clear"
)]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    /// Creates a new hash from raw bytes.
    #[inline]
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Computes a BLAKE3 hash of the provided data.
    #[inline]
    #[must_use]
    pub fn compute(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    /// Returns the underlying bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for Blake3Hash {
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8; 32]> for Blake3Hash {
    #[inline]
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}
