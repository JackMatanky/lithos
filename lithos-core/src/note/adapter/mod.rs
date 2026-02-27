//! Note adapters for storage and markdown ingestion.
//!
//! Adapters bridge infrastructure concerns (database access, file I/O,
//! markdown parsing) into the note domain while keeping the domain free of
//! crate-specific dependencies.

pub mod command;
pub mod query;
pub mod reader;
pub(crate) mod task_parser;
