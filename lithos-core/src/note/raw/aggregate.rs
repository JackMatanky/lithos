use std::time::SystemTime;

use super::{
    RawBlockRef, RawFrontmatter, RawHeading, RawInlineField, RawLink, RawList,
    RawListItem, RawReferenceLink, RawSection, RawTag,
};
use crate::note::{paths::NotePath, scanner::ScannedArtifact};

/// Raw note container with extracted, unvalidated data.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawNote<'source> {
    pub path: NotePath,
    pub source_hash: Box<str>,
    pub source_bytes: u64,
    pub created_at: Option<SystemTime>,
    pub modified_at: Option<SystemTime>,
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
    pub master_artifacts: Vec<ScannedArtifact<'source>>,
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
        source_hash: Box<str>,
        source_bytes: u64,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
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
        master_artifacts: Vec<ScannedArtifact<'source>>,
    ) -> Self {
        Self {
            path,
            source_hash,
            source_bytes,
            created_at,
            modified_at,
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
            master_artifacts,
        }
    }

    /// Converts this raw note into an owned variant suitable for returning
    /// across file ingestion boundaries.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawNote<'static> {
        RawNote {
            path: self.path,
            source_hash: self.source_hash,
            source_bytes: self.source_bytes,
            created_at: self.created_at,
            modified_at: self.modified_at,
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
            master_artifacts: self
                .master_artifacts
                .into_iter()
                .map(ScannedArtifact::into_owned)
                .collect(),
        }
    }
}
