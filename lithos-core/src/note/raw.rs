//! Raw note types and helpers for zero-copy ingestion.

use std::{borrow::Cow, sync::Arc, time::SystemTime};

use crate::{
    config::{frontmatter::FrontmatterConfigSpec, task::TaskConfigSpec},
    note::{
        paths::NotePath,
        position::{SourceByteOffset, SourceByteRange},
    },
};

/// Raw block reference token extracted from note text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawBlockRef<'source> {
    pub id: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawBlockRef<'source> {
    /// Create a raw block reference.
    #[inline]
    #[must_use]
    pub const fn new(
        id: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            position,
        }
    }
}

/// Input format for frontmatter parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawFrontmatterFormat {
    /// YAML frontmatter block.
    Yaml,
    /// TOML frontmatter block.
    Toml,
}

/// Raw frontmatter block captured from metadata events.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawFrontmatter<'source> {
    pub spec: Arc<FrontmatterConfigSpec>,
    pub kind: RawFrontmatterFormat,
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawFrontmatter<'source> {
    /// Create a raw frontmatter block.
    #[inline]
    #[must_use]
    pub const fn new(
        spec: Arc<FrontmatterConfigSpec>,
        kind: RawFrontmatterFormat,
        text: Cow<'source, str>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            spec,
            kind,
            text,
            range,
        }
    }
}

/// Raw heading extracted from the AST.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawHeading<'source> {
    pub level: u8,
    pub text: Cow<'source, str>,
    pub range: SourceByteRange,
    pub position: SourceByteOffset,
}

impl<'source> RawHeading<'source> {
    /// Create a new raw heading entry.
    #[inline]
    #[must_use]
    pub const fn new(
        level: u8,
        text: Cow<'source, str>,
        range: SourceByteRange,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            level,
            text,
            range,
            position,
        }
    }
}

/// Raw inline field extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawInlineField<'source> {
    pub key: Cow<'source, str>,
    pub value: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawInlineField<'source> {
    /// Create a raw inline field entry.
    #[inline]
    #[must_use]
    pub const fn new(
        key: Cow<'source, str>,
        value: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            key,
            value,
            position,
        }
    }
}

/// Raw link style before validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawLinkStyle {
    Wiki,
    Markdown,
}

/// Raw link extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawLink<'source> {
    pub style: RawLinkStyle,
    pub is_embed: bool,
    pub target: Cow<'source, str>,
    pub alias: Option<Cow<'source, str>>,
    pub anchor: Option<Cow<'source, str>>,
    pub position: SourceByteOffset,
}

impl<'source> RawLink<'source> {
    /// Create a new raw link.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw links store full source context"
    )]
    pub const fn new(
        style: RawLinkStyle,
        is_embed: bool,
        target: Cow<'source, str>,
        alias: Option<Cow<'source, str>>,
        anchor: Option<Cow<'source, str>>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            style,
            is_embed,
            target,
            alias,
            anchor,
            position,
        }
    }
}

/// Raw task marker kind extracted from a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawTaskMarker {
    /// Unchecked task marker (typically `[ ]`).
    Unchecked(char),
    /// Checked task marker (typically `[x]`).
    Checked(char),
    /// Task marker with a non-standard symbol.
    Other(char),
}

impl RawTaskMarker {
    /// Create a raw task marker from a character.
    #[inline]
    #[must_use]
    pub fn from_char(marker: char) -> Self {
        match marker {
            ' ' => Self::Unchecked(marker),
            'x' | 'X' => Self::Checked(marker),
            _ => Self::Other(marker),
        }
    }

    /// Returns the raw marker character.
    #[inline]
    #[must_use]
    pub const fn marker(self) -> char {
        match self {
            Self::Unchecked(marker)
            | Self::Checked(marker)
            | Self::Other(marker) => marker,
        }
    }
}

/// Raw list type extracted from markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListType {
    Ordered(u64),
    Unordered,
}

/// Raw list nesting depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawListDepth {
    Root,
    Nested(u8),
}

/// Raw list item extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawListItem<'source> {
    pub list_type: RawListType,
    pub depth: RawListDepth,
    pub text: Cow<'source, str>,
    pub task_marker: Option<RawTaskMarker>,
    pub range: SourceByteRange,
    pub parent: Option<SourceByteOffset>,
}

impl<'source> RawListItem<'source> {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw list items capture full source metadata"
    )]
    pub const fn new(
        list_type: RawListType,
        depth: RawListDepth,
        text: Cow<'source, str>,
        task_marker: Option<RawTaskMarker>,
        range: SourceByteRange,
        parent: Option<SourceByteOffset>,
    ) -> Self {
        Self {
            list_type,
            depth,
            text,
            task_marker,
            range,
            parent,
        }
    }
}

/// Raw reference-style link definition.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawReferenceLink<'source> {
    pub id: Cow<'source, str>,
    pub target: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawReferenceLink<'source> {
    /// Create a new raw reference link definition.
    #[inline]
    #[must_use]
    pub const fn new(
        id: Cow<'source, str>,
        target: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            target,
            position,
        }
    }
}

/// Raw section kinds derived from AST nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawSectionKind {
    /// Heading section.
    Heading,
    /// Paragraph section.
    Paragraph,
    /// Code block section.
    CodeBlock,
    /// Block quote section.
    BlockQuote,
    /// List section.
    List,
    /// Frontmatter section.
    Frontmatter,
}

/// Raw section range with optional heading reference id.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawSection {
    pub kind: RawSectionKind,
    pub range: SourceByteRange,
    pub depth: u32,
}

impl RawSection {
    /// Create a raw section entry.
    #[inline]
    #[must_use]
    pub const fn new(
        kind: RawSectionKind,
        range: SourceByteRange,
        depth: u32,
    ) -> Self {
        Self {
            kind,
            range,
            depth,
        }
    }
}

/// Raw tag token extracted from text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTag<'source> {
    pub value: Cow<'source, str>,
    pub position: SourceByteOffset,
}

impl<'source> RawTag<'source> {
    /// Create a raw tag token.
    #[inline]
    #[must_use]
    pub const fn new(
        value: Cow<'source, str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            value,
            position,
        }
    }
}

/// Raw task extracted from a checkbox list item.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTask<'source> {
    pub spec: Arc<TaskConfigSpec>,
    pub task_marker: RawTaskMarker,
    pub text: Cow<'source, str>,
    pub tags: Vec<RawTag<'source>>,
    pub inline_fields: Vec<RawInlineField<'source>>,
    pub range: SourceByteRange,
}

impl<'source> RawTask<'source> {
    /// Create a raw task entry.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw tasks capture full extraction payload"
    )]
    pub fn new(
        spec: Arc<TaskConfigSpec>,
        task_marker: RawTaskMarker,
        text: Cow<'source, str>,
        tags: Vec<RawTag<'source>>,
        inline_fields: Vec<RawInlineField<'source>>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            spec,
            task_marker,
            text,
            tags,
            inline_fields,
            range,
        }
    }
}

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
    pub list_items: Vec<RawListItem<'source>>,
    pub tasks: Vec<RawTask<'source>>,
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
        source_hash: Box<str>,
        source_bytes: u64,
        created_at: Option<SystemTime>,
        modified_at: Option<SystemTime>,
        frontmatter: Option<RawFrontmatter<'source>>,
        headings: Vec<RawHeading<'source>>,
        sections: Vec<RawSection>,
        links: Vec<RawLink<'source>>,
        tags: Vec<RawTag<'source>>,
        list_items: Vec<RawListItem<'source>>,
        tasks: Vec<RawTask<'source>>,
        inline_fields: Vec<RawInlineField<'source>>,
        reference_links: Vec<RawReferenceLink<'source>>,
        block_refs: Vec<RawBlockRef<'source>>,
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
            list_items: self
                .list_items
                .into_iter()
                .map(RawListItem::into_owned)
                .collect(),
            tasks: self.tasks.into_iter().map(RawTask::into_owned).collect(),
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

impl RawBlockRef<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawBlockRef<'static> {
        RawBlockRef {
            id: Cow::Owned(self.id.into_owned()),
            position: self.position,
        }
    }
}

impl RawFrontmatter<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawFrontmatter<'static> {
        RawFrontmatter {
            spec: self.spec,
            kind: self.kind,
            text: Cow::Owned(self.text.into_owned()),
            range: self.range,
        }
    }
}

impl RawHeading<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawHeading<'static> {
        RawHeading {
            level: self.level,
            text: Cow::Owned(self.text.into_owned()),
            range: self.range,
            position: self.position,
        }
    }
}

impl RawInlineField<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawInlineField<'static> {
        RawInlineField {
            key: Cow::Owned(self.key.into_owned()),
            value: Cow::Owned(self.value.into_owned()),
            position: self.position,
        }
    }
}

impl RawLink<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawLink<'static> {
        RawLink {
            style: self.style,
            is_embed: self.is_embed,
            target: Cow::Owned(self.target.into_owned()),
            alias: self.alias.map(|alias| Cow::Owned(alias.into_owned())),
            anchor: self.anchor.map(|anchor| Cow::Owned(anchor.into_owned())),
            position: self.position,
        }
    }
}

impl RawListItem<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawListItem<'static> {
        RawListItem {
            list_type: self.list_type,
            depth: self.depth,
            text: Cow::Owned(self.text.into_owned()),
            task_marker: self.task_marker,
            range: self.range,
            parent: self.parent,
        }
    }
}

impl RawReferenceLink<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawReferenceLink<'static> {
        RawReferenceLink {
            id: Cow::Owned(self.id.into_owned()),
            target: Cow::Owned(self.target.into_owned()),
            position: self.position,
        }
    }
}

impl RawTag<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawTag<'static> {
        RawTag {
            value: Cow::Owned(self.value.into_owned()),
            position: self.position,
        }
    }
}

impl RawTask<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawTask<'static> {
        RawTask {
            spec: self.spec,
            task_marker: self.task_marker,
            text: Cow::Owned(self.text.into_owned()),
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            inline_fields: self
                .inline_fields
                .into_iter()
                .map(RawInlineField::into_owned)
                .collect(),
            range: self.range,
        }
    }
}
