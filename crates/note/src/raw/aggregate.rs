//! Raw note aggregate and dual-write accumulator.

use super::{
    RawBlockRef, RawFrontmatter, RawHeading, RawInlineFieldToken, RawLink,
    RawListItem, RawSection, RawTag,
};

/// Unvalidated extraction output for a single markdown note.
///
/// All fields contain data extracted during a single-pass parse of the source
/// text. The struct is `#[non_exhaustive]` to allow future artifact types to be
/// added without breaking downstream code.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawNote<'source> {
    pub frontmatter: Option<RawFrontmatter<'source>>,
    pub headings: Vec<RawHeading<'source>>,
    pub sections: Vec<RawSection>,
    pub links: Vec<RawLink<'source>>,
    pub tags: Vec<RawTag<'source>>,
    pub list_items: Vec<RawListItem<'source>>,
    pub inline_fields: Vec<RawInlineFieldToken<'source>>,
    pub block_refs: Vec<RawBlockRef<'source>>,
}

impl<'source> RawNote<'source> {
    /// Create a new raw note container.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "RawNote bundles full extraction output"
    )]
    pub fn new(
        frontmatter: Option<RawFrontmatter<'source>>,
        headings: Vec<RawHeading<'source>>,
        sections: Vec<RawSection>,
        links: Vec<RawLink<'source>>,
        tags: Vec<RawTag<'source>>,
        list_items: Vec<RawListItem<'source>>,
        inline_fields: Vec<RawInlineFieldToken<'source>>,
        block_refs: Vec<RawBlockRef<'source>>,
    ) -> Self {
        Self {
            frontmatter,
            headings,
            sections,
            links,
            tags,
            list_items,
            inline_fields,
            block_refs,
        }
    }

    /// Pushes a list item and applies the dual-write invariant.
    ///
    /// List item tags and inline fields appear in both the item itself (for
    /// item-level queries) and the global collections (for note-level queries).
    /// This is the single location that invariant is enforced.
    pub(crate) fn accept_list_item(&mut self, item: RawListItem<'source>) {
        self.tags.extend(item.tags.iter().cloned());
        self.inline_fields.extend(item.inline_fields.iter().cloned());
        self.list_items.push(item);
    }

    /// Converts this raw note into an owned variant suitable for returning
    /// across file ingestion boundaries.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawNote<'static> {
        RawNote {
            frontmatter: self.frontmatter.map(RawFrontmatter::into_owned),
            headings: self
                .headings
                .into_iter()
                .map(RawHeading::into_owned)
                .collect(),
            sections: self.sections,
            links: self.links.into_iter().map(RawLink::into_owned).collect(),
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            list_items: self
                .list_items
                .into_iter()
                .map(RawListItem::into_owned)
                .collect(),
            inline_fields: self
                .inline_fields
                .into_iter()
                .map(RawInlineFieldToken::into_owned)
                .collect(),
            block_refs: self
                .block_refs
                .into_iter()
                .map(RawBlockRef::into_owned)
                .collect(),
        }
    }
}
