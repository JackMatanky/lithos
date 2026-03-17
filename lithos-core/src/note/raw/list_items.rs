//! Raw list item types.

use crate::note::position::SourceByteOffset;

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
    position: SourceByteOffset,
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
        position: SourceByteOffset,
        parent: Option<SourceByteOffset>,
    ) -> Self {
        Self {
            list_type,
            depth,
            text,
            task_kind,
            position,
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

    /// Return the source byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.position
    }

    /// Return the parent list item position, if any.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<SourceByteOffset> {
        self.parent
    }
}
