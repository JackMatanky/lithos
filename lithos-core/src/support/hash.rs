use rkyv::{Archive, Deserialize, Serialize};

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

    /// Computes a BLAKE3 hash of the provided value as JSON.
    ///
    /// Uses JSON serialization to ensure consistent hashing across all
    /// property types and variants.
    #[inline]
    #[must_use]
    pub fn compute_json<T: serde::Serialize + std::fmt::Debug>(
        value: &T,
    ) -> Self {
        if let Ok(json) = serde_json::to_string(value) {
            Self::compute(json.as_bytes())
        } else {
            // Fallback: use debug representation
            Self::compute(format!("{value:?}").as_bytes())
        }
    }

    /// Returns the underlying bytes.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl ArchivedBlake3Hash {
    /// Check if archived hash matches (for zero-copy staleness checks).
    #[inline]
    #[must_use]
    pub fn is_match(&self, hash: &Blake3Hash) -> bool {
        self.0
            .iter()
            .zip(hash.as_bytes().iter())
            .all(|(left, right)| left == right)
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
