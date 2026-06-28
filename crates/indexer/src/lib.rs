#![feature(trivial_bounds)]
// Enforce R15: fail the build if a direct dependency stops being used. Scoped
// to this crate (the workspace-wide lint is too noisy across other crates).
#![deny(unused_crate_dependencies)]
//! Indexer bounded context for filesystem node scanning and indexing.
//!
//! This module owns the filesystem scanning and indexing pipeline. It sits
//! after Config (receiving resolved scope specs) and feeds Schema, Note, and
//! Template contexts with indexed node data for downstream processing.
//!
//! # Pipeline position
//!
//! Config → Indexer → Schema, Note, Template

mod builder;
mod entry;
mod error;
mod model;
pub(crate) mod port;
pub(crate) mod report;
pub(crate) mod repository;
mod scan;
pub mod scanner;
mod service;
pub mod storage;
mod summary;

pub use entry::{DirIndexEntry, FileIndexEntry, IndexStatus};
pub use error::{IndexerError, IndexerRepositoryError, ScannerError};
pub use model::{DirRecord, FileRecord, FsParentId, FsRecordId};
pub use port::{ScanEntry, ScannerPort, WalkIter};
pub use report::{IndexNodeFailure, IndexReport, SkipReason, SkippedEntry};
pub use repository::{ReadRepository, Repository, WriteRepository};
pub use scan::{IndexOptions, IndexScope, ScanFilters};
pub use service::IndexerService;
#[cfg(test)]
pub(crate) use storage::InMemoryRepository;
pub use storage::RedbRepository;
pub use summary::{DeletedNodes, IndexResult, IndexedNodes};

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time guard that the crate's public surface stays exported.
    /// Naming any of these types fails to compile if a `pub use` is dropped.
    /// Never executed — it only needs to type-check.
    #[expect(
        dead_code,
        reason = "compile-time export guard; intentionally never called"
    )]
    fn public_export_surface() {
        let _: Option<DirIndexEntry> = None;
        let _: Option<FileIndexEntry> = None;
        let _: Option<IndexStatus> = None;
        let _: Option<IndexerError> = None;
        let _: Option<IndexerRepositoryError> = None;
        let _: Option<ScannerError> = None;
        let _: Option<DirRecord> = None;
        let _: Option<FileRecord> = None;
        let _: Option<FsParentId> = None;
        let _: Option<FsRecordId> = None;
        let _: Option<ScanEntry> = None;
        let _: Option<WalkIter> = None;
        let _: Option<IndexNodeFailure> = None;
        let _: Option<IndexReport> = None;
        let _: Option<SkipReason> = None;
        let _: Option<SkippedEntry> = None;
        let _: Option<IndexOptions> = None;
        let _: Option<IndexScope> = None;
        let _: Option<ScanFilters> = None;
        let _: Option<IndexerService> = None;
        let _: Option<RedbRepository> = None;
        let _: Option<DeletedNodes> = None;
        let _: Option<IndexResult> = None;
        let _: Option<IndexedNodes> = None;
    }
}
