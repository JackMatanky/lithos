//! Filesystem-based SPI implementations.

pub mod loader;

/// Type alias for the filesystem file reader adapter.
pub type FileReaderAdapter = loader::FileReader;
