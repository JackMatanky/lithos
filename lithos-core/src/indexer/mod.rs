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
    dead_code,
    unused_imports,
    reason = "Domain types are implemented ahead of usage in subsequent issues"
)]

mod entry;
mod error;
mod model;
pub(crate) mod port;
pub(crate) mod report;
pub(crate) mod repository;
mod scan;
pub(crate) mod scanner;
mod service;
pub(crate) mod storage;
mod summary;

pub(crate) use entry::{DirIndexEntry, FileIndexEntry, IndexStatus};
pub(crate) use error::{IndexerError, IndexerRepositoryError, ScannerError};
pub(crate) use model::{
    DirRecord, FileRecord, FsParentId, FsRecordId, FsRecordType,
};
pub(crate) use port::{ScanEntry, ScannerPort};
pub(crate) use report::{
    IndexNodeFailure, IndexReport, SkipReason, SkippedEntry,
};
pub(crate) use repository::{ReadRepository, Repository, WriteRepository};
pub(crate) use scan::{IndexOptions, IndexScope, ScanFilters};
#[cfg(test)]
pub(crate) use storage::InMemoryRepository;
pub(crate) use storage::RedbRepository;
pub(crate) use summary::{DeletedNodes, IndexResult, IndexedNodes};

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
