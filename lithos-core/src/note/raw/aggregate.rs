use super::{
    RawBlockRef, RawFrontmatter, RawHeading, RawInlineField, RawLink, RawList,
    RawListItem, RawReferenceLink, RawSection, RawTag,
};
use crate::note::paths::NotePath;

/// Raw note container with extracted, unvalidated data.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawNote<'source> {
    pub path: NotePath,
    pub frontmatter: Option<RawFrontmatter<'source>>,
    pub headings: Vec<RawHeading<'source>>,
    pub sections: Vec<RawSection>,
    pub links: Vec<RawLink<'source>>,
    pub tags: Vec<RawTag<'source>>,
    pub lists: Vec<RawList>,
    pub list_items: Vec<RawListItem<'source>>,
    pub inline_fields: Vec<RawInlineField<'source>>,
    pub reference_links: Vec<RawReferenceLink<'source>>,
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
        path: NotePath,
        frontmatter: Option<RawFrontmatter<'source>>,
        headings: Vec<RawHeading<'source>>,
        sections: Vec<RawSection>,
        links: Vec<RawLink<'source>>,
        tags: Vec<RawTag<'source>>,
        lists: Vec<RawList>,
        list_items: Vec<RawListItem<'source>>,
        inline_fields: Vec<RawInlineField<'source>>,
        reference_links: Vec<RawReferenceLink<'source>>,
        block_refs: Vec<RawBlockRef<'source>>,
    ) -> Self {
        Self {
            path,
            frontmatter,
            headings,
            sections,
            links,
            tags,
            lists,
            list_items,
            inline_fields,
            reference_links,
            block_refs,
        }
    }

    /// Converts this raw note into an owned variant suitable for returning
    /// across file ingestion boundaries.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawNote<'static> {
        RawNote {
            path: self.path,
            frontmatter: self.frontmatter.map(RawFrontmatter::into_owned),
            headings: self
                .headings
                .into_iter()
                .map(RawHeading::into_owned)
                .collect(),
            sections: self.sections,
            links: self.links.into_iter().map(RawLink::into_owned).collect(),
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            lists: self.lists.into_iter().map(RawList::into_owned).collect(),
            list_items: self
                .list_items
                .into_iter()
                .map(RawListItem::into_owned)
                .collect(),
            inline_fields: self
                .inline_fields
                .into_iter()
                .map(RawInlineField::into_owned)
                .collect(),
            reference_links: self
                .reference_links
                .into_iter()
                .map(RawReferenceLink::into_owned)
                .collect(),
            block_refs: self
                .block_refs
                .into_iter()
                .map(RawBlockRef::into_owned)
                .collect(),
        }
    }
}
