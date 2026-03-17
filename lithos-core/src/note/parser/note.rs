//! Parser output container for note ingestion.
//!
//! Stores the minimal AST and optional raw frontmatter block produced by the
//! parser boundary. This type is consumed by raw extraction rather than domain
//! conversion.

use super::{ast::Node, frontmatter::MetadataBlock};

/// Parsed note output containing the minimal AST plus raw frontmatter block.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ParsedNote {
    nodes: Vec<Node>,
    frontmatter: Option<MetadataBlock>,
    reference_links: Vec<ReferenceLinkDefinition>,
    source_hash: Box<str>,
    source_bytes: u64,
}

impl ParsedNote {
    /// Creates a new parsed note output.
    #[inline]
    #[must_use]
    pub fn new(
        nodes: Vec<Node>,
        frontmatter: Option<MetadataBlock>,
        reference_links: Vec<ReferenceLinkDefinition>,
        source_hash: Box<str>,
        source_bytes: u64,
    ) -> Self {
        Self {
            nodes,
            frontmatter,
            reference_links,
            source_hash,
            source_bytes,
        }
    }

    /// Returns parsed AST nodes in source order.
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Returns the raw frontmatter block if present.
    #[inline]
    #[must_use]
    pub fn frontmatter(&self) -> Option<&MetadataBlock> {
        self.frontmatter.as_ref()
    }

    /// Returns parsed reference-style link definitions.
    #[inline]
    #[must_use]
    pub fn reference_links(&self) -> &[ReferenceLinkDefinition] {
        &self.reference_links
    }

    /// Returns the source hash for this markdown input.
    #[inline]
    #[must_use]
    #[expect(dead_code, reason = "Reserved for future parser consumers")]
    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }

    /// Returns the source hash as an owned string.
    #[inline]
    #[must_use]
    pub fn source_hash_boxed(&self) -> Box<str> {
        self.source_hash.clone()
    }

    /// Returns the source byte length.
    #[inline]
    #[must_use]
    pub const fn source_bytes(&self) -> u64 {
        self.source_bytes
    }
}

/// Reference-style link definition captured from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ReferenceLinkDefinition {
    id: Box<str>,
    target: Box<str>,
    position: crate::note::position::SourceByteOffset,
}

impl ReferenceLinkDefinition {
    /// Creates a new reference link definition.
    #[inline]
    #[must_use]
    pub fn new(
        id: Box<str>,
        target: Box<str>,
        position: crate::note::position::SourceByteOffset,
    ) -> Self {
        Self {
            id,
            target,
            position,
        }
    }

    /// Returns the definition id.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the raw target string.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns the source byte position for the definition.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> crate::note::position::SourceByteOffset {
        self.position
    }
}
