//! Filesystem-based SPI implementations.

pub mod reader;

/// Type alias for the filesystem file reader adapter.
pub type FileReaderAdapter = reader::Reader;
