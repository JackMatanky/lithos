//! Raw note extraction layer (AST → Raw*).

pub(crate) mod block_refs;
pub(crate) mod extract;
pub(crate) mod frontmatter;
pub(crate) mod headings;
pub(crate) mod links;
pub(crate) mod list_items;
pub(crate) mod note;
pub(crate) mod sections;
pub(crate) mod tags;
pub(crate) mod task_tokens;
pub(crate) mod tasks;

pub use extract::extract_raw_note;
pub use note::RawNote;
