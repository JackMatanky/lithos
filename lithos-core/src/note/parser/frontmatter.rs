//! Frontmatter metadata block types for the parser boundary.
//!
//! The parser captures raw metadata block text and its fence kind. Parsing and
//! validation occur later in the raw layer.

/// Metadata block fence kind detected by the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetadataBlockKind {
    /// YAML `---` frontmatter block.
    YamlStyle,
    /// TOML `+++` frontmatter block.
    PlusesStyle,
}

/// Raw frontmatter block captured by the parser boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MetadataBlock {
    kind: MetadataBlockKind,
    text: Box<str>,
    range: crate::note::position::SourceByteRange,
}

impl MetadataBlock {
    /// Creates a new metadata block payload.
    #[inline]
    #[must_use]
    pub fn new(
        kind: MetadataBlockKind,
        text: Box<str>,
        range: crate::note::position::SourceByteRange,
    ) -> Self {
        Self {
            kind,
            text,
            range,
        }
    }

    /// Returns the metadata block kind.
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> MetadataBlockKind {
        self.kind
    }

    /// Returns the raw metadata block text.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the byte range for the metadata block.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> crate::note::position::SourceByteRange {
        self.range
    }
}

impl From<pulldown_cmark::MetadataBlockKind> for MetadataBlockKind {
    #[inline]
    fn from(kind: pulldown_cmark::MetadataBlockKind) -> Self {
        match kind {
            pulldown_cmark::MetadataBlockKind::YamlStyle => Self::YamlStyle,
            pulldown_cmark::MetadataBlockKind::PlusesStyle => Self::PlusesStyle,
        }
    }
}
