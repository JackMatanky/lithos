//! Raw file storage types for versioned schema files.

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize};

use super::{compression, hash::Blake3Hash};

/// A single version of a raw file (content + metadata + hash).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawFileVersion {
    /// Compressed file content (zstd level 3).
    compressed_content: Vec<u8>,
    /// Blake3 hash of uncompressed content.
    content_hash: Blake3Hash,
    /// File creation timestamp (from filesystem).
    created_at: Option<SystemTime>,
    /// File modification timestamp (from filesystem).
    modified_at: Option<SystemTime>,
    /// When this version was recorded in the database.
    recorded_at: SystemTime,
}

impl RawFileVersion {
    /// Create a new file version from content and metadata.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let compressed_content = compression::compress(content)?;
        let content_hash = Blake3Hash::compute(content.as_bytes());
        let recorded_at = SystemTime::now();

        Ok(Self {
            compressed_content,
            content_hash,
            created_at,
            modified_at,
            recorded_at,
        })
    }

    /// Get the Blake3 hash of the content.
    #[inline]
    #[must_use]
    pub fn content_hash(&self) -> &Blake3Hash {
        &self.content_hash
    }

    /// Get file creation timestamp.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Get file modification timestamp.
    #[inline]
    #[must_use]
    pub fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Get database recording timestamp.
    #[inline]
    #[must_use]
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Decompress and return file content.
    ///
    /// # Errors
    /// Returns error if decompression fails.
    #[inline]
    pub fn content(&self) -> Result<String, compression::DecompressionError> {
        compression::decompress(&self.compressed_content)
    }

    /// Get compressed size in bytes.
    #[inline]
    #[must_use]
    pub fn compressed_size(&self) -> usize {
        self.compressed_content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_file_version_roundtrip() {
        let content = "schema: test\nproperties: []";
        let version = RawFileVersion::new(content, None, None)
            .expect("failed to create version");

        let decompressed = version.content().expect("failed to decompress");
        assert_eq!(content, decompressed);
    }

    #[test]
    fn raw_file_version_hash() {
        let content = "test content";
        let version1 = RawFileVersion::new(content, None, None)
            .expect("failed to create version");
        let version2 = RawFileVersion::new(content, None, None)
            .expect("failed to create version");

        // Same content produces same hash
        assert_eq!(version1.content_hash(), version2.content_hash());
    }

    #[test]
    fn raw_file_version_compression() {
        let content = "a".repeat(1000);
        let version = RawFileVersion::new(&content, None, None)
            .expect("failed to create version");

        // Compression should reduce size significantly
        assert!(
            version.compressed_size() < content.len(),
            "Compressed size should be smaller than original"
        );
    }

    #[test]
    fn raw_file_version_with_timestamps() {
        let now = SystemTime::now();
        let version = RawFileVersion::new("test", Some(now), Some(now))
            .expect("failed to create version");

        assert_eq!(version.created_at(), Some(now));
        assert_eq!(version.modified_at(), Some(now));
        assert!(version.recorded_at() >= now);
    }
}
