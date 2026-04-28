//! Hash metadata for staleness detection and incremental resolution.
//!
//! [`HashRecord`] stores:
//! - Content hash ([`Blake3Hash`]) for fast staleness checks
//! - Per-property hashes for incremental updates and resolution
//!
//! Use [`HashRecord::is_content_match`] for efficient staleness detection
//! without deserializing entire schema data.

use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

use crate::{schema::property::PropertyName, support::hash::Blake3Hash};

// ─────────────────────────────────────────────────────────────────────────────
//  HashRecord
// ─────────────────────────────────────────────────────────────────────────────

/// Content and property hash metadata shared by schema and property bank
/// versions.
///
/// Used for staleness detection (content hash) and incremental resolution
/// (property hashes).
///
/// # Examples
///
/// ```
/// # use lithos_core::schema::views::HashRecord;
/// # use lithos_core::support::hash::Blake3Hash;
/// # use std::collections::HashMap;
/// #
/// let content_hash = Blake3Hash::compute(b"content");
/// let property_hashes = HashMap::new();
/// let record = HashRecord::new(content_hash, property_hashes);
///
/// assert!(record.is_content_match(&content_hash));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct HashRecord {
    /// Blake3 hash of content for staleness detection.
    content: Blake3Hash,

    /// Per-property Blake3 hashes for incremental resolution.
    properties: HashMap<PropertyName, Blake3Hash>,
}

impl HashRecord {
    /// Creates a new hash metadata record.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let content_hash = Blake3Hash::compute(b"content");
    /// let property_hashes = HashMap::new();
    /// let record = HashRecord::new(content_hash, property_hashes);
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        content: Blake3Hash,
        properties: HashMap<PropertyName, Blake3Hash>,
    ) -> Self {
        Self {
            content,
            properties,
        }
    }

    /// Returns the content hash ([`Blake3Hash`]) for staleness detection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let content = Blake3Hash::compute(b"test");
    /// let record = HashRecord::new(content, HashMap::new());
    /// assert_eq!(record.content(), &content);
    /// ```
    #[inline]
    #[must_use]
    pub fn content(&self) -> &Blake3Hash {
        &self.content
    }

    /// Returns per-property hashes for incremental resolution.
    ///
    /// Returns a map of [`PropertyName`] to [`Blake3Hash`] for properties
    /// that need incremental updates.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let record = HashRecord::new(Blake3Hash::compute(b"c"), HashMap::new());
    /// assert!(record.properties().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn properties(&self) -> &HashMap<PropertyName, Blake3Hash> {
        &self.properties
    }

    /// Returns `true` if the provided content hash matches this record.
    ///
    /// Used for fast staleness checks without deserializing the full schema.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::HashRecord;
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// #
    /// let content = Blake3Hash::compute(b"test");
    /// let record = HashRecord::new(content, HashMap::new());
    /// assert!(record.is_content_match(&content));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.content.is_match(hash)
    }
}

impl ArchivedHashRecord {
    /// Returns `true` if the archived content hash matches (zero-copy).
    ///
    /// Used for zero-copy staleness checks without deserialization.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lithos_core::schema::views::hashes::{HashRecord, ArchivedHashRecord};
    /// # use lithos_core::support::hash::Blake3Hash;
    /// # use std::collections::HashMap;
    /// # use rkyv::access;
    /// #
    /// let hash = Blake3Hash::compute(b"test");
    /// let record = HashRecord::new(hash, HashMap::new());
    /// let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&record).unwrap();
    /// let archived =
    ///     rkyv::access::<ArchivedHashRecord, rkyv::rancor::Error>(&bytes)
    ///         .unwrap();
    ///
    /// assert!(archived.is_content_match(&hash));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_content_match(&self, hash: &Blake3Hash) -> bool {
        self.content.is_match(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_record_content_matches() {
        let hash = Blake3Hash::compute(b"test");
        let record = HashRecord::new(hash, HashMap::new());

        assert!(record.is_content_match(&hash));
        assert!(!record.is_content_match(&Blake3Hash::compute(b"other")));
    }
}
