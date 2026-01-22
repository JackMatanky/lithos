//! Service provider interface ports.

pub mod fs;

// Re-export with File prefix for public API
pub use fs::{
    Content as FileContent, Format as FileFormat, Reader as FileReaderPort,
};
