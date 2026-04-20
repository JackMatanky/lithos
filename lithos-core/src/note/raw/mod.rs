//! Raw note types and helpers for zero-copy ingestion.

pub(crate) mod aggregate;
pub(crate) mod block_ref;
pub(crate) mod frontmatter;
pub(crate) mod heading;
pub(crate) mod inline_field;
pub(crate) mod link;
pub(crate) mod list;
pub(crate) mod section;
pub(crate) mod tag;
pub(crate) mod value;

pub type RawBlockRef<'source> = block_ref::RawBlockRef<'source>;
pub type RawFieldValue<'source> = value::RawFieldValue<'source>;
pub type RawFrontmatter<'source> = frontmatter::RawFrontmatter<'source>;
pub type RawHeading<'source> = heading::RawHeading<'source>;
pub type RawInlineFieldToken<'source> =
    inline_field::RawInlineFieldToken<'source>;
pub type RawLink<'source> = link::RawLink<'source>;
pub type RawLinkStyle = link::RawLinkStyle;
pub type RawList = list::RawList;
pub type RawListDepth = list::RawListDepth;
pub type RawListItem<'source> = list::RawListItem<'source>;
pub type RawListItemText<'source> = list::RawListItemText<'source>;
pub type RawListKind = list::RawListKind;
pub type RawNote<'source> = aggregate::RawNote<'source>;
pub type RawSection = section::RawSection;
pub type RawSectionKind = section::RawSectionKind;
pub type RawTag<'source> = tag::RawTag<'source>;
