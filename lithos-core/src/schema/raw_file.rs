//! Raw file storage types for versioned schema files.

use std::time::SystemTime;

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use super::{compression, hash::Blake3Hash, ring_buffer::RingBuffer};

/// A single version of a raw file (content + metadata + hash).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawFileVersion {
    /// Compressed file content (zstd level 3).
    compressed_content: Vec<u8>,
    /// Blake3 hash of uncompressed content.
    content_hash: Blake3Hash,
    /// File creation timestamp (from filesystem).
    #[rkyv(with = Map<AsUnixTime>)]
    created_at: Option<SystemTime>,
    /// File modification timestamp (from filesystem).
    #[rkyv(with = Map<AsUnixTime>)]
    modified_at: Option<SystemTime>,
    /// When this version was recorded in the database.
    #[rkyv(with = AsUnixTime)]
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

/// Raw schema file with version history (up to 5 versions).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawSchemaFile {
    /// File path relative to vault root.
    file_path: String,
    /// Version history (ring buffer, max 5 versions).
    versions: RingBuffer<RawFileVersion, 5>,
}

impl RawSchemaFile {
    /// Create a new raw schema file with initial version.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        file_path: String,
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let mut versions = RingBuffer::new();
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        versions.push(version);

        Ok(Self {
            file_path,
            versions,
        })
    }

    /// Add a new version (evicts oldest if at capacity).
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn add_version(
        &mut self,
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<(), std::io::Error> {
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        self.versions.push(version);
        Ok(())
    }

    /// Get the current (most recent) version.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&RawFileVersion> {
        self.versions.current()
    }

    /// Get file path.
    #[inline]
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Get version history iterator (oldest to newest).
    #[inline]
    pub fn versions(&self) -> impl Iterator<Item = &RawFileVersion> {
        self.versions.iter()
    }

    /// Get version count.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Raw property bank file (singleton, up to 5 versions).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct RawPropertyBankFile {
    /// Version history (ring buffer, max 5 versions).
    versions: RingBuffer<RawFileVersion, 5>,
}

impl RawPropertyBankFile {
    /// Create a new property bank file with initial version.
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn new(
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<Self, std::io::Error> {
        let mut versions = RingBuffer::new();
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        versions.push(version);

        Ok(Self {
            versions,
        })
    }

    /// Add a new version (evicts oldest if at capacity).
    ///
    /// # Errors
    /// Returns error if compression fails.
    #[inline]
    pub fn add_version(
        &mut self,
        content: &str,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
    ) -> Result<(), std::io::Error> {
        let version = RawFileVersion::new(content, created_at, modified_at)?;
        self.versions.push(version);
        Ok(())
    }

    /// Get the current (most recent) version.
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<&RawFileVersion> {
        self.versions.current()
    }

    /// Get version history iterator (oldest to newest).
    #[inline]
    pub fn versions(&self) -> impl Iterator<Item = &RawFileVersion> {
        self.versions.iter()
    }

    /// Get version count.
    #[inline]
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
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

    #[test]
    fn raw_schema_file_creation() {
        let file = RawSchemaFile::new(
            "schemas/test.toml".into(),
            "schema: test",
            None,
            None,
        )
        .expect("failed to create file");

        assert_eq!(file.file_path(), "schemas/test.toml");
        assert_eq!(file.version_count(), 1);
        assert!(file.current().is_some());
    }

    #[test]
    fn raw_schema_file_add_version() {
        let mut file = RawSchemaFile::new(
            "schemas/test.toml".into(),
            "version 1",
            None,
            None,
        )
        .expect("failed to create file");

        file.add_version("version 2", None, None)
            .expect("failed to add version");

        assert_eq!(file.version_count(), 2);
        let current = file.current().expect("no current version");
        let content = current.content().expect("failed to decompress");
        assert_eq!(content, "version 2");
    }

    #[test]
    fn raw_schema_file_version_limit() {
        let mut file = RawSchemaFile::new("test.toml".into(), "v1", None, None)
            .expect("failed to create file");

        // Add 5 more versions (total 6, should evict oldest)
        for i in 2i32..=6i32 {
            file.add_version(&format!("v{i}"), None, None)
                .expect("failed to add version");
        }

        // Should have exactly 5 versions (v2-v6, v1 evicted)
        assert_eq!(file.version_count(), 5);

        // Verify oldest is v2, newest is v6
        let versions: Vec<_> = file.versions().collect();
        assert_eq!(
            versions.first().and_then(|v| v.content().ok()).as_deref(),
            Some("v2")
        );
        assert_eq!(
            versions.last().and_then(|v| v.content().ok()).as_deref(),
            Some("v6")
        );
    }

    #[test]
    fn raw_property_bank_file_creation() {
        let file = RawPropertyBankFile::new("properties: []", None, None)
            .expect("failed to create file");

        assert_eq!(file.version_count(), 1);
        assert!(file.current().is_some());
    }

    #[test]
    fn raw_property_bank_file_add_version() {
        let mut file = RawPropertyBankFile::new("version 1", None, None)
            .expect("failed to create file");

        file.add_version("version 2", None, None)
            .expect("failed to add version");

        assert_eq!(file.version_count(), 2);
        let current = file.current().expect("no current version");
        let content = current.content().expect("failed to decompress");
        assert_eq!(content, "version 2");
    }
}
