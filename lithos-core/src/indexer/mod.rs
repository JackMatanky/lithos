//! Indexer bounded context for filesystem node scanning and indexing.
//!
//! This module owns the filesystem scanning and indexing pipeline. It sits
//! after Config (receiving resolved scope specs) and feeds Schema, Note, and
//! Template contexts with indexed node data for downstream processing.
//!
//! # Pipeline position
//!
//! Config → Indexer → Schema, Note, Template

pub mod entry;
pub mod error;
pub mod model;
pub mod scan;
pub mod summary;
