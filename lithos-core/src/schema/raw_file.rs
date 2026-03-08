//! Raw file storage types for versioned schema files.

use std::{io::Read as _, time::SystemTime};

use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use super::hash::Blake3Hash;

// ─────────────────────────────────────────────────────────────────────────────
//  Compression Utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Compression level (3 = balanced speed/ratio).
const COMPRESSION_LEVEL: i32 = 3;

/// Compress string content using zstd.
///
/// # Errors
/// Returns error if compression fails.
#[inline]
fn compress(content: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(content.as_bytes(), COMPRESSION_LEVEL)
}

/// Decompress zstd data to string.
///
/// # Errors
/// Returns error if decompression fails or output is not UTF-8.
#[inline]
fn decompress(compressed: &[u8]) -> Result<String, DecompressionError> {
    let mut decompressed = Vec::new();
    zstd::Decoder::new(compressed)?.read_to_end(&mut decompressed)?;
    String::from_utf8(decompressed).map_err(DecompressionError::InvalidUtf8)
}

/// Decompression errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DecompressionError {
    /// I/O error during decompression.
    #[error("decompression I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Decompressed data is not valid UTF-8.
    #[error("decompressed data is not valid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

// ─────────────────────────────────────────────────────────────────────────────
//  Ring Buffer (Fixed-Size Version History)
// ─────────────────────────────────────────────────────────────────────────────

/// Fixed-size ring buffer for versioned file storage (compile-time size, zero
/// allocation).
///
/// This ring buffer uses `u8` indices for memory efficiency (5 versions max).
/// All arithmetic and indexing is safe by design (modulo wraparound prevents
/// out-of-bounds).
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
struct RingBuffer<T, const N: usize> {
    items: [Option<T>; N],
    head: u8, // Next write position
    len: u8,  // Current count (0..=N)
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Create empty ring buffer.
    #[inline]
    const fn new() -> Self {
        Self {
            items: [const { None }; N],
            head: 0,
            len: 0,
        }
    }

    /// Push item (evicts oldest if full).
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Ring buffer arithmetic is modulo-bounded (0..N)"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "All indices are modulo N (cannot exceed array bounds)"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "u8 <-> usize conversions safe for N = 5"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "N is constrained to 5 (ring buffer for file versions)"
    )]
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Modulo operation is fundamental to ring buffer wraparound"
    )]
    fn push(&mut self, item: T) {
        self.items[self.head as usize] = Some(item);
        self.head = (self.head + 1) % (N as u8);
        if self.len < N as u8 {
            self.len += 1;
        }
    }

    /// Get most recent item.
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Ring buffer arithmetic is modulo-bounded (0..N)"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "All indices are modulo N (cannot exceed array bounds)"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "u8 <-> usize conversions safe for N = 5"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "N is constrained to 5 (ring buffer for file versions)"
    )]
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Modulo operation is fundamental to ring buffer wraparound"
    )]
    fn current(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.head + (N as u8) - 1) % (N as u8);
        self.items[idx as usize].as_ref()
    }

    /// Get item at index (0 = oldest, len-1 = newest).
    #[inline]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Ring buffer arithmetic is modulo-bounded (0..N)"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "All indices are modulo N (cannot exceed array bounds)"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "u8 <-> usize conversions safe for N = 5"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "N is constrained to 5 (ring buffer for file versions)"
    )]
    #[expect(
        clippy::integer_division_remainder_used,
        reason = "Modulo operation is fundamental to ring buffer wraparound"
    )]
    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len as usize {
            return None;
        }
        let offset =
            (self.head + (N as u8) - self.len + index as u8) % (N as u8);
        self.items[offset as usize].as_ref()
    }

    /// Number of items.
    #[inline]
    #[expect(
        clippy::as_conversions,
        reason = "u8 -> usize is always safe and lossless"
    )]
    const fn len(&self) -> usize {
        self.len as usize
    }

    /// Iterate over items (oldest to newest).
    #[inline]
    fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len()).filter_map(move |i| self.get(i))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Raw File Version Storage
// ─────────────────────────────────────────────────────────────────────────────

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
        let compressed_content = compress(content)?;
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
    pub fn content(&self) -> Result<String, DecompressionError> {
        decompress(&self.compressed_content)
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

    mod diff_raw_files_tests {
        use super::*;

        #[test]
        fn unchanged_when_same_hash_and_path() {
            let v1 = RawFileVersion::new("content", None, None).unwrap();
            let v2 = RawFileVersion::new("content", None, None).unwrap();

            let change = diff_raw_files(&v1, &v2, "file.json", "file.json");
            assert_eq!(change, FileChange::Unchanged);
        }

        #[test]
        fn modified_when_hash_differs() {
            let v1 = RawFileVersion::new("old content", None, None).unwrap();
            let v2 = RawFileVersion::new("new content", None, None).unwrap();

            let change = diff_raw_files(&v1, &v2, "file.json", "file.json");
            assert_eq!(change, FileChange::Modified);
        }

        #[test]
        fn renamed_when_same_hash_different_path() {
            let v1 = RawFileVersion::new("content", None, None).unwrap();
            let v2 = RawFileVersion::new("content", None, None).unwrap();

            let change = diff_raw_files(&v1, &v2, "old.json", "new.json");
            assert_eq!(change, FileChange::Renamed);
        }

        #[test]
        fn modified_trumps_renamed() {
            let v1 = RawFileVersion::new("old", None, None).unwrap();
            let v2 = RawFileVersion::new("new", None, None).unwrap();

            // Different hash AND different path -> still Modified
            let change = diff_raw_files(&v1, &v2, "old.json", "new.json");
            assert_eq!(change, FileChange::Modified);
        }
    }

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

// ─────────────────────────────────────────────────────────────────────────────
//  File Change Detection
// ─────────────────────────────────────────────────────────────────────────────

/// Type of change detected between two file versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileChange {
    /// Content hash unchanged (file not modified).
    Unchanged,
    /// Content hash changed (file was modified).
    Modified,
    /// Same hash but different path (file was renamed).
    Renamed,
}

/// Compare two file versions to detect changes.
///
/// This helper enables accurate change detection:
/// - **Unchanged**: Hash matches, path matches → no action needed
/// - **Modified**: Hash differs → re-parse and re-resolve
/// - **Renamed**: Hash matches but path differs → update path, no re-parse
///
/// # Examples
/// ```
/// # use lithos_core::schema::raw_file::{RawFileVersion, FileChange, diff_raw_files};
/// let v1 = RawFileVersion::new("content", None, None).unwrap();
/// let v2 = RawFileVersion::new("content", None, None).unwrap();
/// assert_eq!(diff_raw_files(&v1, &v2, "same.json", "same.json"), FileChange::Unchanged);
///
/// let v3 = RawFileVersion::new("changed", None, None).unwrap();
/// assert_eq!(diff_raw_files(&v1, &v3, "file.json", "file.json"), FileChange::Modified);
///
/// assert_eq!(diff_raw_files(&v1, &v2, "old.json", "new.json"), FileChange::Renamed);
/// ```
#[inline]
#[must_use]
pub fn diff_raw_files(
    cached: &RawFileVersion,
    current: &RawFileVersion,
    cached_path: &str,
    current_path: &str,
) -> FileChange {
    if cached.content_hash() != current.content_hash() {
        FileChange::Modified
    } else if cached_path != current_path {
        FileChange::Renamed
    } else {
        FileChange::Unchanged
    }
}
