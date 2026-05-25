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

pub use error::{
    VaultFileError, VaultPathError, VaultProcessError, VaultRepositoryError,
};
pub use model::{DirId, DirView, FileId, FileView, FsEntryView};
pub use processor::{ScanMode, VaultProcessReport, VaultProcessor};
pub use repository::{ReadRepository, Repository, WriteRepository};
pub use storage::RedbRepository;
