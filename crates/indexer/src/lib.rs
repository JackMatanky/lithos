#![feature(trivial_bounds)]
//! Indexer bounded context for filesystem node scanning and indexing.
//!
//! This module owns the filesystem scanning and indexing pipeline. It sits
//! after Config (receiving resolved scope specs) and feeds Schema, Note, and
//! Template contexts with indexed node data for downstream processing.
//!
//! # Pipeline position
//!
//! Config → Indexer → Schema, Note, Template

#![allow(
    private_interfaces,
    private_bounds,
    missing_docs,
    clippy::missing_errors_doc,
    clippy::missing_inline_in_public_items,
    dead_code,
    unused_imports,
    reason = "Domain types are implemented ahead of usage in subsequent issues"
)]

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
pub use model::FsRecordId;
pub(crate) use model::{DirRecord, FileRecord, FsParentId, FsRecordType};
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

    #[test]
    fn test_indexer_exports() {
        let _: Option<&dyn ReadRepository> = None;
        let _: Option<&dyn WriteRepository> = None;
        let _: Option<&dyn Repository> = None;
        let _: Option<RedbRepository> = None;
        let _: Option<InMemoryRepository> = None;
        let _: Option<&dyn ScannerPort> = None;
    }
}
