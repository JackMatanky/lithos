//! Blake3 hash types for content addressing.

use std::fmt;

use rkyv::{Archive, Deserialize, Serialize};

/// Blake3 hash (32 bytes).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Archive, Serialize, Deserialize,
)]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    /// Compute Blake3 hash of bytes.
    #[inline]
    #[must_use]
    pub fn compute(bytes: &[u8]) -> Self {
        let hash = blake3::hash(bytes);
        Self(*hash.as_bytes())
    }

    /// Create a zero hash (all bytes are zero).
    ///
    /// Useful for tests and placeholder values when the actual hash is not
    /// available.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Get hash as byte slice.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for Blake3Hash {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl std::str::FromStr for Blake3Hash {
    type Err = hex::FromHexError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(Self(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blake3_compute() {
        let data = b"hello world";
        let hash1 = Blake3Hash::compute(data);
        let hash2 = Blake3Hash::compute(data);
        assert_eq!(hash1, hash2, "Same input produces same hash");
    }

    #[test]
    fn blake3_display_parse() {
        let hash = Blake3Hash::compute(b"test");
        let hex_str = hash.to_string();
        let parsed: Blake3Hash = hex_str.parse().unwrap();
        assert_eq!(hash, parsed, "Display/parse roundtrip works");
    }
}
