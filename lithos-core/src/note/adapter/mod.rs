//! Note adapters for storage and markdown ingestion.
//!
//! Adapters bridge infrastructure concerns (database access, file I/O,
//! markdown parsing) into the note domain while keeping the domain free of
//! crate-specific dependencies. Markdown parsing uses extractor state
//! machines orchestrated by the reader to emit domain entities.

pub mod command;
pub mod ingestor;
pub mod query;
pub mod reader;
pub mod stored;

// Extractors (Phase 2+)
pub(super) mod extract_frontmatter;
pub(super) mod extract_heading;
pub(super) mod extract_link;
pub(super) mod extract_list;
pub(super) mod extract_section;
pub(super) mod extract_tag;
