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
pub enum RawListKind {
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

/// Raw list container extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawList {
    pub kind: RawListKind,
    pub depth: RawListDepth,
    pub range: SourceByteRange,
    pub task_spec: Arc<TaskConfigSpec>,
    pub item_positions: Vec<SourceByteOffset>,
}

impl RawList {
    /// Create a new raw list container.
    #[inline]
    #[must_use]
    pub fn new(
        kind: RawListKind,
        depth: RawListDepth,
        range: SourceByteRange,
        task_spec: Arc<TaskConfigSpec>,
        item_positions: Vec<SourceByteOffset>,
    ) -> Self {
        Self {
            kind,
            depth,
            range,
            task_spec,
            item_positions,
        }
    }

    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawList {
        RawList {
            kind: self.kind,
            depth: self.depth,
            range: self.range,
            task_spec: self.task_spec,
            item_positions: self.item_positions,
        }
    }
}

/// Raw list item extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawListItem<'source> {
    pub list_kind: RawListKind,
    pub depth: RawListDepth,
    pub text: Cow<'source, str>,
    pub task_marker: Option<RawTaskMarker>,
    pub range: SourceByteRange,
    pub parent: Option<SourceByteOffset>,
    pub task_payload: Option<RawTaskPayload<'source>>,
}

impl<'source> RawListItem<'source> {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw list items capture full source metadata"
    )]
    pub fn new(
        list_kind: RawListKind,
        depth: RawListDepth,
        text: Cow<'source, str>,
        task_marker: Option<RawTaskMarker>,
        range: SourceByteRange,
        parent: Option<SourceByteOffset>,
        task_payload: Option<RawTaskPayload<'source>>,
    ) -> Self {
        Self {
            list_kind,
            depth,
            text,
            task_marker,
            range,
            parent,
            task_payload,
        }
    }
}

/// Raw task fields extracted from a list item.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTaskFields<'source> {
    pub fields: Vec<RawInlineField<'source>>,
}

impl<'source> RawTaskFields<'source> {
    /// Create a new raw task fields container.
    #[inline]
    #[must_use]
    pub fn new(fields: Vec<RawInlineField<'source>>) -> Self {
        Self {
            fields,
        }
    }
}

/// Raw task payload extracted from a checkbox list item.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTaskPayload<'source> {
    pub fields: RawTaskFields<'source>,
    pub text_full: Cow<'source, str>,
    pub task_marker: RawTaskMarker,
    pub tags: Vec<RawTag<'source>>,
    pub range: SourceByteRange,
}

impl<'source> RawTaskPayload<'source> {
    /// Create a raw task payload.
    #[inline]
    #[must_use]
    pub fn new(
        fields: RawTaskFields<'source>,
        text_full: Cow<'source, str>,
        task_marker: RawTaskMarker,
        tags: Vec<RawTag<'source>>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            fields,
            text_full,
            task_marker,
            tags,
            range,
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
            list_kind: self.list_kind,
            depth: self.depth,
            text: Cow::Owned(self.text.into_owned()),
            task_marker: self.task_marker,
            range: self.range,
            parent: self.parent,
            task_payload: self.task_payload.map(RawTaskPayload::into_owned),
        }
    }
}

impl RawTaskFields<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawTaskFields<'static> {
        RawTaskFields::new(
            self.fields.into_iter().map(RawInlineField::into_owned).collect(),
        )
    }
}

impl RawTaskPayload<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawTaskPayload<'static> {
        RawTaskPayload {
            fields: self.fields.into_owned(),
            text_full: Cow::Owned(self.text_full.into_owned()),
            task_marker: self.task_marker,
            tags: self.tags.into_iter().map(RawTag::into_owned).collect(),
            range: self.range,
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
