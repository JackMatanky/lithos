//! Zstd compression utilities for raw file storage.

use std::io::Read as _;

/// Compression level (3 = balanced speed/ratio).
const COMPRESSION_LEVEL: i32 = 3;

/// Compress string content using zstd.
///
/// # Errors
/// Returns error if compression fails.
#[inline]
pub fn compress(content: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(content.as_bytes(), COMPRESSION_LEVEL)
}

/// Decompress zstd data to string.
///
/// # Errors
/// Returns error if decompression fails or output is not UTF-8.
#[inline]
pub fn decompress(compressed: &[u8]) -> Result<String, DecompressionError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_roundtrip() {
        let original = "hello world".repeat(100);
        let compressed = compress(&original).expect("compression failed");
        let decompressed =
            decompress(&compressed).expect("decompression failed");

        assert_eq!(original, decompressed);
        assert!(
            compressed.len() < original.len(),
            "Compression should reduce size for repetitive data"
        );
    }

    #[test]
    fn compression_reduces_size() {
        // Repetitive content compresses well
        let content = "a".repeat(1000);
        let compressed = compress(&content).expect("compression failed");

        // Expect at least 10x compression ratio (compare directly)
        let expected_max_size = 100; // 1000 / 10
        assert!(
            compressed.len() < expected_max_size,
            "Compression ratio should be at least 10x for highly repetitive \
             data"
        );
    }

    #[test]
    fn decompress_empty() {
        let compressed = compress("").expect("compression failed");
        let decompressed =
            decompress(&compressed).expect("decompression failed");
        assert_eq!(decompressed, "");
    }
}
