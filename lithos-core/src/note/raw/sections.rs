//! Raw section extraction helpers.

use crate::note::{
    error::NoteError,
    parser::ast::{Node, NodeKind},
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

/// Build raw sections from AST nodes.
pub(crate) fn extract_sections(
    nodes: &[Node],
) -> Result<Vec<RawSection>, NoteError> {
    let mut sections = Vec::new();
    walk_sections(nodes, 0, &mut sections)?;
    Ok(sections)
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &NodeKind"
)]
fn walk_sections(
    nodes: &[Node],
    depth: u32,
    sections: &mut Vec<RawSection>,
) -> Result<(), NoteError> {
    for node in nodes {
        match node.kind() {
            NodeKind::Heading {
                ..
            } => {
                sections.push(RawSection::new(
                    RawSectionKind::Heading,
                    node.range(),
                    depth,
                ));
            }
            NodeKind::Paragraph {
                ..
            } => {
                sections.push(RawSection::new(
                    RawSectionKind::Paragraph,
                    node.range(),
                    depth,
                ));
            }
            NodeKind::List {
                items,
                ..
            } => {
                walk_sections(items, depth.saturating_add(1), sections)?;
            }
            NodeKind::ListItem {
                children,
                ..
            } => {
                sections.push(RawSection::new(
                    RawSectionKind::List,
                    node.range(),
                    depth,
                ));
                walk_sections(children, depth.saturating_add(1), sections)?;
            }
            NodeKind::CodeBlock {
                ..
            } => {
                sections.push(RawSection::new(
                    RawSectionKind::CodeBlock,
                    node.range(),
                    depth,
                ));
            }
            NodeKind::BlockQuote {
                nodes: quote_nodes,
                ..
            } => {
                sections.push(RawSection::new(
                    RawSectionKind::BlockQuote,
                    node.range(),
                    depth,
                ));
                walk_sections(quote_nodes, depth.saturating_add(1), sections)?;
            }
        }
    }
    Ok(())
}
