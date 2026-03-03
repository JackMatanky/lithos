//! Note adapters for storage and markdown ingestion.
//!
//! Adapters bridge infrastructure concerns (database access, file I/O,
//! markdown parsing) into the note domain while keeping the domain free of
//! crate-specific dependencies.

pub mod command;
pub mod query;
pub mod reader;
pub(crate) mod tag_scanner;
pub(crate) mod task_parser;

// Extractors (Phase 2+)
pub(super) mod extract_heading;
pub(super) mod extract_link;
pub(super) mod extract_list;
pub(super) mod extract_section;
