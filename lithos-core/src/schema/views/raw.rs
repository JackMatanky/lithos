//! Raw file version views for schema persistence.
//!
//! **Migration Status**: Placeholder - types still in `storage.rs`.
//!
//! ## Pending Migration
//!
//! The following types should be moved here from `storage.rs`:
//! - `RawSchemaFile` - Raw schema file with version history
//! - `RawPropertyBankFile` - Raw property bank file with version history
//! - `RawFileVersion` - Single version of a raw file
//! - `FileChange` - Change detection enum
//! - `DecompressionError` - Decompression error type
//! - `diff_raw_files()` - File comparison function
//!
//! ## Rationale
//!
//! These types are views over raw file storage and should be separated from
//! the main storage module to allow `storage.rs` to focus on the Repository
//! trait and implementation.

#![expect(clippy::pub_use, reason = "Temporary re-export during migration")]

// TODO: Move types from storage.rs
// For now, re-export from storage.rs to maintain compatibility
pub use super::super::storage::{
    DecompressionError, FileChange, RawFileVersion, RawPropertyBankFile,
    RawSchemaFile, diff_raw_files,
};
