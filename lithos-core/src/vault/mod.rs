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
/// Vault repository and redb adapter.
pub mod storage;

pub use error::{
    VaultFileError, VaultPathError, VaultProcessError, VaultRepositoryError,
};
pub use model::{
    DirId, DirView, FileId, FileView, FsEntryView, NormalizedPath, VaultFile,
    VaultFolder, VaultPath,
};
pub use processor::{ScanMode, VaultProcessReport, VaultProcessor};
pub use storage::{RedbRepository, Repository};

/// `VAULT_FILES_BY_PATH` stores serialized [`VaultFile`] values keyed by
/// vault-relative paths.
pub(crate) const VAULT_FILES_BY_PATH: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("vault_files_by_path");

/// `VAULT_FOLDERS_BY_PATH` stores serialized [`VaultFolder`] values keyed by
/// vault-relative paths.
pub(crate) const VAULT_FOLDERS_BY_PATH: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("vault_folders_by_path");
