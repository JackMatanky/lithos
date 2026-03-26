//! Raw note types and helpers for zero-copy ingestion.

#![expect(
    clippy::pattern_type_mismatch,
    reason = "Pattern matching style is clear in context"
)]
#![expect(
    clippy::iter_over_hash_type,
    reason = "Hash iteration order doesn't affect correctness here"
)]
#![expect(clippy::pub_use, reason = "Re-export raw DTOs for note::raw API")]

pub(crate) mod block_ref;
pub(crate) mod field_value;
pub(crate) mod frontmatter;
pub(crate) mod heading;
pub(crate) mod inline_field;
pub(crate) mod link;
pub(crate) mod list;
pub(crate) mod note;
pub(crate) mod reference_link;
pub(crate) mod section;
pub(crate) mod tag;

pub use block_ref::RawBlockRef;
pub use field_value::RawFieldValue;
pub use frontmatter::{RawFrontmatter, RawFrontmatterFormat};
pub use heading::RawHeading;
pub use inline_field::RawInlineField;
pub use link::{RawLink, RawLinkStyle};
pub use list::{
    RawList, RawListDepth, RawListItem, RawListKind, RawTaskMarker,
};
pub use note::RawNote;
pub use reference_link::RawReferenceLink;
pub use section::{RawSection, RawSectionKind};
pub use tag::RawTag;
