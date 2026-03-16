//! Raw note container for extraction outputs.

use std::time::SystemTime;

use super::{
    block_refs::RawBlockRef, frontmatter::RawFrontmatter, headings::RawHeading,
    inline_fields::RawInlineField, links::RawLink, list_items::RawListItem,
    reference_links::RawReferenceLink, sections::RawSection, tags::RawTag,
    tasks::RawTask,
};
use crate::note::paths::NotePath;

/// Raw note container with extracted, unvalidated data.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawNote {
    path: NotePath,
    source_hash: Box<str>,
    source_bytes: u64,
    created_at: Option<SystemTime>,
    modified_at: Option<SystemTime>,
    frontmatter: Option<RawFrontmatter>,
    headings: Vec<RawHeading>,
    sections: Vec<RawSection>,
    links: Vec<RawLink>,
    tags: Vec<RawTag>,
    list_items: Vec<RawListItem>,
    tasks: Vec<RawTask>,
    inline_fields: Vec<RawInlineField>,
    reference_links: Vec<RawReferenceLink>,
    block_refs: Vec<RawBlockRef>,
}

impl RawNote {
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
        frontmatter: Option<RawFrontmatter>,
        headings: Vec<RawHeading>,
        sections: Vec<RawSection>,
        links: Vec<RawLink>,
        tags: Vec<RawTag>,
        list_items: Vec<RawListItem>,
        tasks: Vec<RawTask>,
        inline_fields: Vec<RawInlineField>,
        reference_links: Vec<RawReferenceLink>,
        block_refs: Vec<RawBlockRef>,
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
            list_items,
            tasks,
            inline_fields,
            reference_links,
            block_refs,
        }
    }

    /// Return the note path for this raw note.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    /// Return the source hash for this note.
    #[inline]
    #[must_use]
    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    /// Return the source byte length.
    #[inline]
    #[must_use]
    pub const fn source_bytes(&self) -> u64 {
        self.source_bytes
    }

    /// Return the file creation timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn created_at(&self) -> Option<SystemTime> {
        self.created_at
    }

    /// Return the file modification timestamp, if available.
    #[inline]
    #[must_use]
    pub const fn modified_at(&self) -> Option<SystemTime> {
        self.modified_at
    }

    /// Return the raw frontmatter block, if present.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&RawFrontmatter> {
        self.frontmatter.as_ref()
    }

    /// Return extracted raw headings.
    #[inline]
    #[must_use]
    pub fn headings(&self) -> &[RawHeading] {
        &self.headings
    }

    /// Return extracted raw sections.
    #[inline]
    #[must_use]
    pub fn sections(&self) -> &[RawSection] {
        &self.sections
    }

    /// Return extracted raw links.
    #[inline]
    #[must_use]
    pub fn links(&self) -> &[RawLink] {
        &self.links
    }

    /// Return extracted raw tags.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[RawTag] {
        &self.tags
    }

    /// Return extracted raw list items.
    #[inline]
    #[must_use]
    pub fn list_items(&self) -> &[RawListItem] {
        &self.list_items
    }

    /// Return extracted raw tasks.
    #[inline]
    #[must_use]
    pub fn tasks(&self) -> &[RawTask] {
        &self.tasks
    }

    /// Return extracted raw inline fields.
    #[inline]
    #[must_use]
    pub fn inline_fields(&self) -> &[RawInlineField] {
        &self.inline_fields
    }

    /// Return extracted raw reference link definitions.
    #[inline]
    #[must_use]
    pub fn reference_links(&self) -> &[RawReferenceLink] {
        &self.reference_links
    }

    /// Return extracted raw block references.
    #[inline]
    #[must_use]
    pub fn block_refs(&self) -> &[RawBlockRef] {
        &self.block_refs
    }
}
