//! Raw section extraction helpers.

use crate::note::{
    error::NoteError,
    parser::ast::{AstNode, AstNodeKind},
    position::SourceByteRange,
};

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
}

/// Raw section range with optional heading reference id.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawSection {
    kind: RawSectionKind,
    range: SourceByteRange,
    heading_id: Option<usize>,
    depth: u32,
}

impl RawSection {
    /// Create a raw section entry.
    #[inline]
    #[must_use]
    pub fn new(
        kind: RawSectionKind,
        range: SourceByteRange,
        heading_id: Option<usize>,
        depth: u32,
    ) -> Self {
        Self {
            kind,
            range,
            heading_id,
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

    /// Return the optional heading id reference.
    #[inline]
    #[must_use]
    pub const fn heading_id(&self) -> Option<usize> {
        self.heading_id
    }

    /// Return the section nesting depth.
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }
}

/// Build raw sections from AST nodes.
pub(crate) fn extract_sections(
    nodes: &[AstNode],
) -> Result<Vec<RawSection>, NoteError> {
    let mut sections = Vec::new();

    for node in nodes {
        let kind = match node.kind() {
            AstNodeKind::Heading {
                ..
            } => RawSectionKind::Heading,
            AstNodeKind::Paragraph {
                ..
            } => RawSectionKind::Paragraph,
            AstNodeKind::ListItem {
                ..
            } => RawSectionKind::List,
            AstNodeKind::ListStart {
                ..
            }
            | AstNodeKind::ListEnd => continue,
            AstNodeKind::CodeBlock {
                ..
            } => RawSectionKind::CodeBlock,
            AstNodeKind::BlockQuote {
                ..
            } => RawSectionKind::BlockQuote,
            AstNodeKind::Link {
                ..
            } => continue,
        };
        sections.push(RawSection::new(kind, node.range(), None, 0));
    }

    Ok(sections)
}
