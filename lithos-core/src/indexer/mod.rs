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
mod scan;
pub(crate) mod scanner;
mod summary;

pub(crate) use entry::{DirIndexEntry, FileIndexEntry, IndexStatus};
pub(crate) use error::IndexerError;
pub(crate) use model::{DirRecord, FileRecord, FsRecordId, FsRecordType};
pub(crate) use scan::{IndexOptions, IndexScope, ScanFilters};
pub(crate) use summary::{
    DeletedNodes, IndexNodeFailure, IndexReport, IndexResult, IndexedNodes,
};
