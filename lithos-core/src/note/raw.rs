//! Raw note types and helpers.

use std::time::SystemTime;

use crate::note::{
    paths::NotePath,
    position::{SourceByteOffset, SourceByteRange},
};

// --- block_refs.rs ---
/// Raw block reference token extracted from note text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawBlockRef {
    id: Box<str>,
    position: SourceByteOffset,
}

impl RawBlockRef {
    /// Create a raw block reference.
    #[inline]
    #[must_use]
    pub fn new(id: Box<str>, position: SourceByteOffset) -> Self {
        Self {
            id,
            position,
        }
    }

    /// Return the raw block reference id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the source byte position for the block reference.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
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
pub struct RawFrontmatter {
    kind: RawFrontmatterFormat,
    text: Box<str>,
    range: SourceByteRange,
}

impl RawFrontmatter {
    /// Create a raw frontmatter block.
    #[inline]
    #[must_use]
    pub fn new(
        kind: RawFrontmatterFormat,
        text: Box<str>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            kind,
            text,
            range,
        }
    }

    /// Return the metadata block kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> RawFrontmatterFormat {
        self.kind
    }

    /// Return the raw frontmatter text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the source byte range for the frontmatter block.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

/// Raw heading extracted from the AST.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawHeading {
    level: u8,
    text: Box<str>,
    range: SourceByteRange,
    position: SourceByteOffset,
}

impl RawHeading {
    /// Create a new raw heading entry.
    #[inline]
    #[must_use]
    pub fn new(
        level: u8,
        text: Box<str>,
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

    /// Return the raw heading level.
    #[inline]
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Return the raw heading text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the byte range for the heading.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Return the start byte offset for the heading.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

/// Raw inline field extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawInlineField {
    key: Box<str>,
    value: Box<str>,
    position: SourceByteOffset,
}

impl RawInlineField {
    /// Create a raw inline field entry.
    #[inline]
    #[must_use]
    pub fn new(
        key: Box<str>,
        value: Box<str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            key,
            value,
            position,
        }
    }

    /// Return the raw key string.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Return the raw value string.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the source byte position of the field key.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
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
pub struct RawLink {
    style: RawLinkStyle,
    is_embed: bool,
    target: Box<str>,
    alias: Option<Box<str>>,
    anchor: Option<Box<str>>,
    position: SourceByteOffset,
}

impl RawLink {
    /// Create a new raw link.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw links store full source context"
    )]
    pub fn new(
        style: RawLinkStyle,
        is_embed: bool,
        target: Box<str>,
        alias: Option<Box<str>>,
        anchor: Option<Box<str>>,
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

    /// Return the raw link style.
    #[inline]
    #[must_use]
    pub const fn style(&self) -> RawLinkStyle {
        self.style
    }

    /// Return true if this link is an embed.
    #[inline]
    #[must_use]
    pub const fn is_embed(&self) -> bool {
        self.is_embed
    }

    /// Return the raw target string.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Return the alias text, if present.
    #[inline]
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Return the raw anchor text, if present.
    #[inline]
    #[must_use]
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// Return the source byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

/// Raw task marker kind extracted from a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawTaskKind {
    /// Unchecked task marker (typically `[ ]`).
    Unchecked(char),
    /// Checked task marker (typically `[x]`).
    Checked(char),
    /// Task marker with a non-standard symbol.
    Other(char),
}

impl RawTaskKind {
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
    Ordered {
        start: u64,
    },
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
pub struct RawListItem {
    list_type: RawListType,
    depth: RawListDepth,
    text: Box<str>,
    task_kind: Option<RawTaskKind>,
    range: SourceByteRange,
    parent: Option<SourceByteOffset>,
}

impl RawListItem {
    /// Create a new raw list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Raw list items capture full source metadata"
    )]
    pub fn new(
        list_type: RawListType,
        depth: RawListDepth,
        text: Box<str>,
        task_kind: Option<RawTaskKind>,
        range: SourceByteRange,
        parent: Option<SourceByteOffset>,
    ) -> Self {
        Self {
            list_type,
            depth,
            text,
            task_kind,
            range,
            parent,
        }
    }

    /// Return the list type.
    #[inline]
    #[must_use]
    pub const fn list_type(&self) -> RawListType {
        self.list_type
    }

    /// Return the list nesting depth.
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> RawListDepth {
        self.depth
    }

    /// Return the raw list item text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the raw task marker kind, if present.
    #[inline]
    #[must_use]
    pub const fn task_kind(&self) -> Option<RawTaskKind> {
        self.task_kind
    }

    /// Return the source byte range.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Return the parent list item position, if any.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<SourceByteOffset> {
        self.parent
    }
}

/// Raw reference-style link definition.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawReferenceLink {
    id: Box<str>,
    target: Box<str>,
    position: SourceByteOffset,
}

impl RawReferenceLink {
    /// Create a new raw reference link definition.
    #[inline]
    #[must_use]
    pub fn new(
        id: Box<str>,
        target: Box<str>,
        position: SourceByteOffset,
    ) -> Self {
        Self {
            id,
            target,
            position,
        }
    }

    /// Return the definition id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the raw target string.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Return the source byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
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
    kind: RawSectionKind,
    range: SourceByteRange,
    depth: u32,
}

impl RawSection {
    /// Create a raw section entry.
    #[inline]
    #[must_use]
    pub fn new(
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

    /// Return the section kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> RawSectionKind {
        self.kind
    }

    /// Return the section byte range.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Return the section nesting depth.
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }
}

/// Raw tag token extracted from text.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTag {
    value: Box<str>,
    position: SourceByteOffset,
}

impl RawTag {
    /// Create a raw tag token.
    #[inline]
    #[must_use]
    pub fn new(value: Box<str>, position: SourceByteOffset) -> Self {
        Self {
            value,
            position,
        }
    }

    /// Return the raw tag token value.
    #[inline]
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return the source byte position of the tag token.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }
}

/// Raw task extracted from a checkbox list item.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawTask {
    task_kind: RawTaskKind,
    text: Box<str>,
    tags: Vec<Box<str>>,
    inline_fields: Vec<RawInlineField>,
    range: SourceByteRange,
}

impl RawTask {
    /// Create a raw task entry.
    #[inline]
    #[must_use]
    pub fn new(
        task_kind: RawTaskKind,
        text: Box<str>,
        tags: Vec<Box<str>>,
        inline_fields: Vec<RawInlineField>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            task_kind,
            text,
            tags,
            inline_fields,
            range,
        }
    }

    /// Return the task marker kind.
    #[inline]
    #[must_use]
    pub const fn task_kind(&self) -> RawTaskKind {
        self.task_kind
    }

    /// Return the raw task text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return raw tag tokens found in the task text.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Box<str>] {
        &self.tags
    }

    /// Return raw inline fields parsed from the task text.
    #[inline]
    #[must_use]
    pub fn inline_fields(&self) -> &[RawInlineField] {
        &self.inline_fields
    }

    /// Return the source byte range.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }
}

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
