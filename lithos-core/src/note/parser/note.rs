//! Parser output container for note ingestion.

use super::{ast::AstNode, frontmatter::MetadataBlock};

/// Parsed note output containing the minimal AST plus raw frontmatter block.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ParsedNote {
    nodes: Vec<AstNode>,
    frontmatter: Option<MetadataBlock>,
}

impl ParsedNote {
    /// Create a new parsed note output.
    #[inline]
    #[must_use]
    pub fn new(
        nodes: Vec<AstNode>,
        frontmatter: Option<MetadataBlock>,
    ) -> Self {
        Self {
            nodes,
            frontmatter,
        }
    }

    /// Return parsed AST nodes in source order.
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &[AstNode] {
        &self.nodes
    }

    /// Return the raw frontmatter block if present.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&MetadataBlock> {
        self.frontmatter.as_ref()
    }
}
