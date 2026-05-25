//! Vault context for file and folder metadata tracking.
//!
//! This module owns vault-wide file discovery and metadata persistence. It
//! routes markdown files into the note processor while tracking all files and
//! folders in dedicated vault tables.

#![expect(
    clippy::pub_use,
    reason = "Vault module re-exports types for ergonomic access"
)]
#![expect(
    clippy::module_name_repetitions,
    reason = "Vault module names are explicit and scoped"
)]

/// Vault error types.
pub mod error;
/// Vault domain types.
pub mod model;
/// Vault processor pipeline.
pub mod processor;
/// Repository traits for vault persistence.
pub mod repository;
/// Vault repository and redb adapter.
pub mod storage;
/// Legacy vault storage (to be removed after migration).
pub mod storage_legacy;

pub use error::{
    VaultFileError, VaultPathError, VaultProcessError, VaultRepositoryError,
};
pub use model::{DirId, DirView, FileId, FileView, FsEntryView};
pub use processor::{ScanMode, VaultProcessReport, VaultProcessor};
// Re-export table constants for legacy storage
pub(crate) use storage::tables::{
    DIR_ID_BY_PATH, DIR_VIEWS, FILE_ID_BY_PATH, FILE_IDS_BY_BASENAME,
    FILE_IDS_BY_FORMAT, FILE_IDS_BY_PARENT, FILE_VIEWS,
};
// Export legacy storage types until migration complete
pub use storage_legacy::{RedbRepository, Repository};
