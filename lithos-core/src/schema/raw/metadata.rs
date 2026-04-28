//! Raw metadata types for schema and property bank definitions.

/// File metadata for raw schema/property-bank ingestion payloads.
///
/// Alias to filesystem-native [`crate::fs::FileStats`] so the raw layer carries
/// created/modified timestamps and file size captured at ingestion time.
pub type RawFileStats = crate::fs::FileStats;
